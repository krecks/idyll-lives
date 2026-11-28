/**
 * Build mode's Land tab: the world's land shaped in play. A `LandEditor` over the land in view
 * (surveyed through the renderer the first time the tab is used, and the simulation's water mask
 * asked for with it), the pointer turned into strokes (`LandStroke`), and each stroke, undo and
 * redo sent to the simulation as a `shapeLand` command: the tiles it touched (the whole land the
 * first time, with its header, in a world that had none), the cover and zones and the water mask
 * when they changed, and its price. Living charges it (a stroke stops short where the funds give
 * out; undo refunds it), Creative is free. A command the simulation refuses is taken back here as
 * if it had never been, and its reason told. Commands go one at a time: a stroke waits for the one
 * before it to be answered.
 */

import { groundDepth, type Rle, type ShapeLandCommand, type ShapeLandMeta, type WorldStructure, type WorldZone } from '../core/protocol';
import type { Renderer } from '../render/types';
import { costLabel, NO_LAND, stopsAtFunds, type LandState } from '../ui/land/landTools';
import { LandStroke, type LandSurface } from '../ui/land/landStroke';
import { encodeLand, encodeTiles, NONE, tilesOf, type Land, type LandPatch, type LandRect } from './land';
import type { Brush } from './landBrush';
import { bridgesMeta, bridgesOrWhy, withBridges, type BridgeChange } from './landBridges';
import { builtOf, LandEditor, type LandPrices } from './landEdit';
import { rle, unrle } from './landMasks';
import type { Rect } from './town';

/** What the Land tab needs of the game. */
export interface LandModeHost {
  renderer: Renderer;
  shapeLand(command: ShapeLandCommand): Promise<void>;
  waterMask(): Promise<{ water: Rle; width: number; depth: number }>;
  /** The world as the game has it now. */
  world(): WorldStructure | null;
  household(): number;
  prices(): LandPrices | null;
  creative(): boolean;
  funds(): number;
  brush(): Brush;
  /** How the land's shaping stands changed (the palette's buttons, the cursor tag). */
  changed(state: LandState): void;
  /** The simulation refused something: why, in its words (shown to the player). */
  refused(why: string): void;
  /** A mountain's ghost (SVG path, screen pixels; '' none). */
  ghost?(path: string): void;
}

/** Whether the patch's rectangle is wet anywhere it wasn't, or dry where it was (its old values against the land now). */
export function wetChanged(land: Land, patch: LandPatch): boolean {
  if (!patch.h || !patch.wl) return false;
  const { rect } = patch;
  const w = rect.i1 - rect.i0;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const [k, p] = [j * land.cols + i, (j - rect.j0) * w + (i - rect.i0)];
      const now = land.wl[k] !== NONE && land.wl[k] > land.h[k];
      const then = patch.wl[p] !== NONE && patch.wl[p] > patch.h[p];
      if (now !== then) return true;
    }
  }
  return false;
}

/** Whether a river's way (`Land.flow`) is laid anywhere in the patch's rectangle, before it or now. */
function flowTouched(land: Land, patch: LandPatch): boolean {
  if (!patch.flow || !land.flow) return false;
  if (patch.flow.some((v) => v !== 0)) return true;
  const { rect } = patch;
  for (let j = rect.j0; j < rect.j1; j++) for (let k = j * land.cols + rect.i0, end = j * land.cols + rect.i1; k < end; k++) if (land.flow[k]) return true;
  return false;
}

/**
 * The `shapeLand` command for the change `patch` made to `land` (its old values; the land holds the
 * new): the tiles its rectangle reaches (their rivers' ways too where a way was laid or taken away
 * there, `flowTouched`), or with `whole` (a world without `meta.land` yet) the whole land with its
 * header, since a tile not sent reads as flat and dry; the cover when it changed (or with the
 * header); `zones` and `water` as given (the editor's, when the cover or the water changed).
 */
export function landCommand(o: { land: Land; patch: LandPatch; whole: boolean; household: number; cost: number; zones?: WorldZone[]; water?: Rle }): ShapeLandCommand {
  const { land, patch } = o;
  let meta: ShapeLandMeta['land'];
  if (o.whole) {
    const file = encodeLand(land);
    const byTile = (codes: string[]) => Object.fromEntries(codes.map((c, t) => [t, c] as const));
    meta = { ...file, h: byTile(file.h), wl: byTile(file.wl), ...(file.flow ? { flow: byTile(file.flow) } : {}) };
  } else {
    const { flow, ...tiles } = encodeTiles(land, tilesOf(patch.rect, land.cols));
    meta = { ...tiles, ...(flow && flowTouched(land, patch) ? { flow } : {}), ...(patch.cover ? { cover: rle(land.cover) } : {}) };
  }
  return { type: 'shapeLand', household: o.household, cost: o.cost, meta: { land: meta, ...(o.zones ? { zones: o.zones } : {}) }, ...(o.water ? { water: o.water } : {}) };
}

/** What the simulation's refusal says, as the player is told it. */
export function refusal(message: string): string {
  const money = /^not enough money: shaping the land costs \$(-?\d+)/.exec(message);
  if (money) return `Not enough money: that would cost $${Number(money[1]).toLocaleString('en-US')}.`;
  const text = message.replace(/^Error:\s*/, '');
  return `${text.charAt(0).toUpperCase()}${text.slice(1)}${/[.!?]$/.test(text) ? '' : '.'}`;
}

/** A stroke in Living stops this many stamps (at its going rate) short of the funds: a frame may stamp a few times. */
const STAMPS_AHEAD = 4;

export class LandMode {
  private editor: LandEditor | null = null;
  private seeding: Promise<LandEditor | null> | null = null;
  private stroke: LandStroke | null = null;
  /** The world's `meta.land` holds the whole land (sent once, with the first stroke, in a world that had none). */
  private whole = false;
  /**
   * On a land the town follows: its lots (what a bridge may not stand on) and the water mask's size; each stroke's
   * bridges (`landBridges.ts`), by its patch, for undo and redo.
   */
  private bridging: { plots: Rect[]; width: number; depth: number } | null = null;
  private readonly bridged = new WeakMap<LandPatch, BridgeChange[]>();
  /** The command on its way, and the one after it waiting (none: free). */
  private sending: Promise<void> | null = null;
  private state: LandState = { ...NO_LAND };
  /** Why a stroke stopped short here (the funds), until the next one. */
  private note: string | null = null;
  private disposed = false;

  constructor(private readonly host: LandModeHost) {}

  /** The editor, surveyed the first time it is asked for (null if the land can't be: no world in view). */
  ready(): Promise<LandEditor | null> {
    this.seeding ??= this.seed();
    return this.seeding;
  }

  get current(): LandEditor | null {
    return this.editor;
  }

  /** Where the brush works: the game's renderer. */
  readonly surface: LandSurface = {
    pick: (x, y) => this.host.renderer.pickLand(x, y),
    changed: (rect, done) => this.host.renderer.landChanged(rect, done),
    cursor: (c) => {
      this.host.renderer.setLandBrush(c);
      const refused = c?.tone === 'bad';
      if (refused !== this.state.refused) this.set({ refused });
    },
    project: (x, y, z, out) => this.host.renderer.project(x, y, z, out),
  };

  private async seed(): Promise<LandEditor | null> {
    this.set({ surveying: true });
    try {
      const [land, mask] = await Promise.all([this.host.renderer.landOf(), this.host.waterMask()]);
      const world = this.host.world();
      if (this.disposed || !world) return null;
      // (The ground storey's rows: a world with storeys stacks them under its depth.)
      const built = builtOf({ ...world, depth: groundDepth(world) });
      if (built.width !== mask.width || built.depth !== mask.depth) console.warn(`[land] the water mask is ${mask.width} × ${mask.depth}, the world ${built.width} × ${built.depth}`);
      // (On a land the town follows, the earth tools shape built land too: the roads and lots move with it, the land
      // settled under them at the stroke's end.)
      const settle = (rect: LandRect, before: Int16Array) => this.host.renderer.settleLand?.(rect, before) ?? null;
      const follow = land.seeded === 'follow' ? { earthOnBuilt: true, settle, bridges: true } : {};
      this.bridging = land.seeded === 'follow' ? { plots: built.plots.slice(), width: mask.width, depth: mask.depth } : null;
      const editor = LandEditor.forWorld(land, { ...built, width: mask.width, depth: mask.depth }, { water: mask.water, zones: world.meta?.zones ?? [] }, follow);
      this.whole = !!world.meta?.land;
      this.host.renderer.setLand(land);
      this.editor = editor;
      this.stroke = new LandStroke(editor, this.surface, {
        brush: () => this.host.brush(),
        goOn: (e) => this.goOn(e),
        ended: (patch) => this.ended(patch),
        ghost: (d) => this.host.ghost?.(d),
      });
      this.set({ surveying: false, ready: true });
      return editor;
    } catch (err) {
      console.error('[land] surveying the land failed:', err);
      this.seeding = null;
      this.set({ surveying: false });
      this.host.refused("The land couldn't be surveyed. Try again in a moment.");
      return null;
    }
  }

  // ---- The pointer (`session.ts`'s `PointerInput`, on the Land tab) ---------------------------

  /** Left button pressed at canvas CSS pixel (`x`, `y`): a stroke begins (true: the gesture is the brush's, the camera stays). */
  press(x: number, y: number): boolean {
    const s = this.stroke;
    if (!s) {
      void this.ready();
      return true;
    }
    // (One command at a time: a stroke waits for the last to be answered.)
    if (this.sending) return true;
    this.note = null;
    if (!s.down(x, y)) return false;
    this.host.renderer.setLeftDragCamera(false);
    this.set({ stroking: true, cost: 0 });
    return true;
  }

  /** The pointer moved: the cursor follows it, a stroke under way is dragged on (its cost comes each frame: `goOn`). */
  hover(x: number, y: number): void {
    this.stroke?.hover(x, y);
  }

  release(): void {
    this.host.renderer.setLeftDragCamera(true);
    this.stroke?.up();
    this.set({ stroking: false });
  }

  /** The stroke under way dropped (right-click, Escape); true if there was one. */
  cancel(): boolean {
    const was = !!this.stroke?.active;
    this.stroke?.cancel();
    this.host.renderer.setLeftDragCamera(true);
    this.host.ghost?.('');
    this.set({ stroking: false, cost: 0 });
    this.refresh();
    return was;
  }

  leave(): void {
    this.stroke?.leave();
  }

  /** The tab put away: the brush's cursor and ghost gone, a stroke under way dropped. */
  putAway(): void {
    this.cancel();
    this.host.renderer.setLandBrush(null);
  }

  dispose(): void {
    this.disposed = true;
    this.putAway();
  }

  // ---- Undo, redo, reset -----------------------------------------------------------------------

  async undo(): Promise<void> {
    await this.sending;
    const e = this.editor;
    const patch = e?.undo();
    if (!e || !patch) return;
    const bridges = this.bridgesTo(patch, 'before');
    this.surface.changed(patch.rect, true);
    await this.send(
      patch,
      () => {
        this.bridgesTo(patch, 'after');
        const back = e.redo();
        if (back) this.surface.changed(back.rect, true);
      },
      bridges,
    );
  }

  async redo(): Promise<void> {
    await this.sending;
    const e = this.editor;
    const patch = e?.redo();
    if (!e || !patch) return;
    const bridges = this.bridgesTo(patch, 'after');
    this.surface.changed(patch.rect, true);
    await this.send(
      patch,
      () => {
        this.bridgesTo(patch, 'before');
        const back = e.undo();
        if (back) this.surface.changed(back.rect, true);
      },
      bridges,
    );
  }

  /** The editor's roads with `patch`'s bridges as they were (`before`) or as it made them (`after`); the command's `meta.roads` for it (none if it made none). */
  private bridgesTo(patch: LandPatch, side: 'before' | 'after'): ShapeLandMeta['roads'] {
    const changes = this.bridged.get(patch);
    if (!changes || !this.editor) return undefined;
    this.editor.setRoads(withBridges(this.editor.roads, changes, side));
    return bridgesMeta(changes, side);
  }

  /** The land put back as it was surveyed (one more stroke: undone as one). */
  async reset(): Promise<void> {
    await this.sending;
    const e = this.editor;
    if (!e || !e.reset()) return;
    // (The reset's patch is the last on the undo stack: undone, it hands back the patch that redoes it.)
    const undo = e.undo();
    const patch = undo && e.redo();
    if (!patch) return;
    this.surface.changed(patch.rect, true);
    await this.send(patch, () => {
      const back = e.undo();
      e.dropRedo();
      if (back) this.surface.changed(back.rect, true);
    });
  }

  // ---- Strokes ---------------------------------------------------------------------------------

  /** What the stroke under way has cost so far (0 in Creative or without prices). */
  private running(): number {
    const prices = this.host.prices();
    return this.editor && prices && !this.host.creative() ? this.editor.running(prices) : 0;
  }

  /** Before each frame of a stroke: in Living, it stops where one more frame would pass the funds. */
  private goOn(e: LandEditor): boolean {
    if (this.host.creative()) return true;
    const cost = this.running();
    const stamps = e.current?.stamps ?? 0;
    this.set({ cost });
    if (!stopsAtFunds(cost, stamps, this.host.funds(), false, STAMPS_AHEAD)) return true;
    this.note = `Out of money: the stroke stopped at ${costLabel(cost, false, Infinity).text}.`;
    return false;
  }

  /** A stroke ended (it changed something): sent; refused, taken back. */
  private ended(patch: LandPatch): void {
    this.host.renderer.setLeftDragCamera(true);
    this.set({ stroking: false, cost: 0 });
    void this.commit(patch);
  }

  /** Sends a stroke the editor has ended (`patch`: what `end` returned); refused, it is taken back. Resolves once answered. */
  commit(patch: LandPatch): Promise<void> {
    const e = this.editor!;
    this.lastRefusal = null;
    const takeBack = () => {
      this.bridgesTo(patch, 'before');
      const back = e.undo();
      e.dropRedo();
      if (back) this.surface.changed(back.rect, true);
    };
    // Water across a road: the road bridged there, or the stroke refused (`landBridges.ts`).
    const b = this.bridging;
    if (b && wetChanged(e.land, patch)) {
      const made = bridgesOrWhy(e.roads, b.plots, unrle(e.water(), b.width * b.depth), b.width, b.depth);
      if ('refused' in made) {
        takeBack();
        this.refuse(made.refused);
        this.refresh();
        return Promise.resolve();
      }
      if (made.changes.length) {
        this.bridged.set(patch, made.changes);
        e.setRoads(made.roads);
        return this.send(patch, takeBack, bridgesMeta(made.changes, 'after'));
      }
    }
    return this.send(patch, takeBack);
  }

  /**
   * Sends the change `patch` holds the old values of (a stroke, an undo, a redo) at its price, and
   * waits for the answer; refused, `takeBack` puts the editor back as it was and the reason is told.
   */
  private send(patch: LandPatch, takeBack: () => void, roads?: ShapeLandMeta['roads']): Promise<void> {
    const e = this.editor!;
    const prices = this.host.prices();
    const cost = prices && !this.host.creative() ? e.cost(patch, prices) : 0;
    const cover = !!patch.cover;
    // (A stroke that bridged a road sends the water with its decks dry, whether or not the rest changed.)
    const water = wetChanged(e.land, patch) || !!roads;
    const whole = !this.whole;
    let command: ShapeLandCommand;
    try {
      command = landCommand({
        land: e.land,
        patch,
        whole,
        household: this.host.household(),
        cost,
        ...(cover || water ? { zones: e.zonesOf() } : {}),
        ...(water ? { water: e.water() } : {}),
      });
      if (roads) command = { ...command, meta: { ...command.meta, roads } };
    } catch (err) {
      takeBack();
      this.refuse(err instanceof Error ? err.message : String(err));
      this.refresh();
      return Promise.resolve();
    }
    this.set({ busy: true });
    this.refresh();
    const run = this.host
      .shapeLand(command)
      .then(
        () => {
          if (whole) this.whole = true;
        },
        (err: unknown) => {
          takeBack();
          this.refuse(refusal(err instanceof Error ? err.message : String(err)));
        },
      )
      .finally(() => {
        if (this.sending === run) this.sending = null;
        this.set({ busy: false });
        this.refresh();
      });
    this.sending = run;
    return run;
  }

  /** Why the last command sent was refused (null: it wasn't), for `window.__land` (`?debug`). */
  lastRefusal: string | null = null;

  /** A refusal told to the player, and kept (`lastRefusal`). */
  private refuse(why: string): void {
    this.lastRefusal = why;
    this.host.refused(why);
  }

  /** The palette's buttons and hint from the editor. */
  private refresh(): void {
    const e = this.editor;
    if (!e) return;
    const { heights, water, cover } = e.changed();
    this.set({ canUndo: e.canUndo, canRedo: e.canRedo, changed: heights || water || cover, hint: this.note ?? e.hint });
  }

  private set(change: Partial<LandState>): void {
    const next = { ...this.state, ...change };
    if ((Object.keys(change) as (keyof LandState)[]).every((k) => this.state[k] === next[k])) return;
    this.state = next;
    this.host.changed(next);
  }
}
