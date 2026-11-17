/**
 * The live 3D scene behind the new game's screens: which town the overview shows (the draft being
 * edited), its camera shots, lot highlights, labels pinned to lots and lot picking, and the main
 * menu's picture of it (`keepStill`; the menu itself shows no town). Screens call this; it reaches
 * the renderer through the host (never the sim worker).
 */

import { groundDepth, type WorldStructure } from '../core/protocol';
import type { LandCursor, LotHighlight, Renderer, TownBuildStep, TownGhosts, TownShot } from '../render/types';
import { areaFocus } from '../render/shared/overviewCamera';
import type { Land, LandRect } from './land';
import type { UseLayer } from './uses';
import { settings } from '../settings/settings.svelte';
import { services } from '../ui/services';
import { host } from './host';
import { loadTemplates, type NeighbourhoodDraft, type Templates } from './town';
import { keepStill, type MenuStill } from './menuStill';
import { previewWorld } from './townPreview';
import { perfPhase, perfTime, perfTimeAsync } from '../debug/perf';
import { layoutTags, type Box, type TagSpot } from './tagLayout';
import { roadIndex } from './roads';

/** Golden hour on the title screen: low warm sun, long shadows, no lamps yet. */
export const MENU_MINUTE = 18 * 60 + 20;
/** Late afternoon while building the household: warm but brighter (sim faces, town detail). */
const CREATE_MINUTE = 17 * 60 + 30;
/** The overview while choosing: clear afternoon light, houses read best from above. */
const OVERVIEW_MINUTE = 16 * 60 + 40;

/** Screen rectangle in CSS pixels (as `getBoundingClientRect`). */
export interface Frame {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export type Shot =
  /** Main menu: a slow, low orbit, the town to the right of the menu panel. */
  | { kind: 'menu' }
  /** Behind character creation: closer, slower. */
  | { kind: 'create' }
  /** The whole neighbourhood from above, framed into `frame`. */
  | { kind: 'overview'; frame: Frame }
  /** One lot as the game will first show it, framed into `frame`. */
  | { kind: 'lot'; plot: number; frame: Frame }
  /** The whole land from above, framed into `frame` (the new game's land, town and home steps), with or without a town on it. */
  | { kind: 'world'; frame: Frame }
  /**
   * One area (`OpenWorldDraft.areas` index: 0 the main town, then the villages) from above, framed into `frame`: its
   * lots, or its box while it has none. An area the world doesn't have frames the whole land.
   */
  | { kind: 'area'; area: number; frame: Frame };

/** How far either side of a point (CSS px) `MenuScene.scale` measures the land. */
const SCALE_SPAN = 4;

const NO_HIGHLIGHT: LotHighlight = { hover: null, selected: null, marked: [], outlines: false };

/** The menu's picture at most this wide (pixels); as the screen is, if smaller. */
const STILL_WIDTH = 1920;

class MenuScene {
  private templates: Templates | null = null;
  private draft: NeighbourhoodDraft | null = null;
  /** The lot shown as the player's house (null: none). */
  private home: number | null = null;
  private world: WorldStructure | null = null;
  private worldOf: NeighbourhoodDraft | null = null;
  private worldHome: number | null = null;
  /** Previews made elsewhere (the town worker: `townJobs.ts`), by draft, with no home. */
  private readonly prepared = new WeakMap<NeighbourhoodDraft, WorldStructure>();
  private shotKey = '';
  private wanted: { shot: Shot; duration: number; cut?: boolean } | null = null;
  /** The highlight asked for last (put back after the menu's picture is taken). */
  private lit: Partial<LotHighlight> = {};
  /** When the camera's glide to the current shot ends (performance.now() ms). */
  private glideEnd = 0;
  /** Tags of lots outside this screen area stay at its edge (pointing the way). */
  private bounds: Frame | null = null;
  /** The part of the world the shot frames (metres): lots well outside it have no tag. */
  private framed: TownShot['focus'] | null = null;
  /** Weak so a disposed renderer isn't pinned here after a graphics-preset change. */
  private renderer: WeakRef<Renderer> | null = null;
  private get live(): Renderer | undefined {
    return this.renderer?.deref();
  }
  private readonly labels = new Map<number, Label>();
  private readonly out = { x: 0, y: 0 };
  /** UI on top of the town that tags must keep clear of (buttons, hint bars). */
  private readonly obstacles = new Set<HTMLElement>();
  /** The last scale `scale` found (m a CSS px). */
  private lastScale = 1;
  /** Lots whose tags win when tags compete for space. */
  private focusPlots: [number | null, number | null] = [null, null];
  private readonly sizes = new ResizeObserver((entries) => {
    for (const e of entries) {
      for (const label of this.labels.values()) {
        if (label.el.firstElementChild !== e.target) continue;
        const r = e.target.getBoundingClientRect();
        // Measured without hover lift/scale changes mattering much; zoom (UI scale) is included.
        label.w = r.width;
        label.h = r.height;
      }
    }
  });

  /**
   * Shows a neighbourhood behind the menus (the renderer rebuilds it only when its layout
   * changed). Again after a game, too: starting one hides the overview. `home`: the lot the
   * player will live on (the chosen home, the household's), drawn as the game will show it.
   * `onStep`: the parts of the town as they are built (none for a town already shown). `covered`: no
   * town is in view meanwhile (a panel says how its making goes): it is built in longer slices.
   */
  async show(draft: NeighbourhoodDraft, home: number | null = null, onStep?: (done: TownBuildStep) => void, covered = false): Promise<void> {
    if (draft === this.draft && home === this.home && this.live?.townShown) return;
    this.draft = draft;
    this.home = home;
    perfPhase('menu: show town');
    this.templates ??= await perfTimeAsync('menu: loadTemplates', loadTemplates);
    const templates = this.templates;
    const made = home === null ? this.prepared.get(draft) : undefined;
    const world = this.world && this.worldOf === draft && this.worldHome === home ? this.world : (made ?? perfTime('menu: previewWorld', () => previewWorld(services.content, templates, draft, home)));
    const renderer = await host.rendererNow({ ...settings });
    if (this.draft !== draft || this.home !== home) return;
    this.world = world;
    this.worldOf = draft;
    this.worldHome = home;
    this.attach(renderer);
    for (const [plot, label] of this.labels) this.anchor(plot, label);
    this.applyShot();
    if (covered) renderer.setCovered(true);
    try {
      await perfTimeAsync('menu: showTown (renderer)', () => host.showTown(world, MENU_MINUTE, { ...settings }, onStep));
    } finally {
      if (covered) renderer.setCovered(false);
    }
  }

  /** Takes the town away (the main menu shows its picture). */
  hide(): Promise<void> {
    this.draft = null;
    this.home = null;
    return host.hideTown();
  }

  /**
   * Takes the main menu's picture of the town shown (its shot, at golden hour, no lot lit up) and
   * keeps it (`menuStill.ts`). `glide`: the camera glides there and stays (the main menu, whose town
   * goes once the picture is taken); else it cuts there and back, behind a loading screen (a new
   * game starting). Null if no town is on screen or the picture couldn't be drawn.
   */
  async keepStill(glide: boolean): Promise<MenuStill | null> {
    const renderer = this.live;
    const draft = this.draft;
    if (!renderer?.townShown || !draft || !this.world) return null;
    const before = this.wanted;
    const lit = this.lit;
    this.highlight({}, !glide);
    this.shoot({ kind: 'menu' }, glide ? 2.4 : 0, !glide);
    if (glide) await this.settled();
    // (The cut's light and pose, and the sun's map for them, are drawn by then.)
    await renderer.framesRendered(2);
    if (this.draft !== draft || !renderer.townShown) return null;
    const scale = Math.min(1, STILL_WIDTH / Math.max(1, window.innerWidth * (window.devicePixelRatio || 1))) * (window.devicePixelRatio || 1);
    const url = await perfTimeAsync('menu: still', () => renderer.captureThumbnail(window.innerWidth * scale, window.innerHeight * scale));
    if (!glide && before && this.draft === draft) {
      this.shoot(before.shot, 0, true);
      this.highlight(lit, true);
    }
    if (!url) return null;
    const picture = await (await fetch(url)).blob();
    await keepStill(picture, draft.name);
    return { url: URL.createObjectURL(picture), name: draft.name };
  }

  /** A draft's preview (`previewWorld` with no home), already made: `show` uses it. */
  prepare(draft: NeighbourhoodDraft, world: WorldStructure): void {
    this.prepared.set(draft, world);
  }

  /** The town currently shown. */
  get shown(): NeighbourhoodDraft | null {
    return this.draft;
  }

  /** Points the camera (applied once the town is shown); repeated identical shots are ignored. `cut`: there at once, its light too. */
  shoot(shot: Shot, duration = 1.6, cut = false): void {
    this.wanted = { shot, duration, cut };
    this.applyShot();
  }

  private applyShot(): void {
    const world = this.world;
    const renderer = this.live;
    if (!world || !renderer || !this.wanted) return;
    const { shot, duration, cut } = this.wanted;
    this.bounds = 'frame' in shot ? shot.frame : null;
    const townShot = this.townShot(shot, world, duration);
    // What the camera frames is the town as far as the tags go (see `positionLabels`); before the key
    // check, since a new world of the same size and shot can leave the key unchanged.
    this.framed = 'frame' in shot ? townShot.focus : null;
    // (An area's shot is framed again when its box changes, a village moved or a town laid on a bare land; not for
    // a lot more or less in it, which would take the player's pan away at every edit.)
    const box = shot.kind === 'area' ? world.meta?.areas?.[shot.area] : undefined;
    const area = box ? `|${box.x},${box.z},${box.w},${box.d}` : '';
    const key = `${JSON.stringify(shot)}|${world.width}x${groundDepth(world)}${area}|${window.innerWidth}x${window.innerHeight}`;
    if (key === this.shotKey) return;
    const first = this.shotKey === '' || !!cut;
    this.shotKey = key;
    renderer.setTownShot(townShot, first);
    // (A cut is once: the shot applied again later, for a new town or screen size, glides.)
    if (cut) this.wanted = { shot, duration };
    this.glideEnd = performance.now() + (first ? 0 : (townShot.duration ?? 0) * 1000);
  }

  /** Resolves once the camera has finished gliding (also to shots set while waiting). */
  async settled(): Promise<void> {
    for (let wait = this.glideEnd - performance.now(); wait > 0; wait = this.glideEnd - performance.now()) {
      await new Promise((resolve) => setTimeout(resolve, wait));
    }
  }

  /** Lights lots up (`snap`: at once, not eased; for a picture). */
  highlight(h: Partial<LotHighlight>, snap = false): void {
    this.lit = h;
    this.focusPlots = [h.selected ?? null, h.hover ?? null];
    this.live?.setLotHighlight({ ...NO_HIGHLIGHT, ...h }, snap);
  }

  /** Keeps lot tags clear of `el` (a button or bar over the town). Returns a detach function. */
  avoid(el: HTMLElement): () => void {
    this.obstacles.add(el);
    return () => this.obstacles.delete(el);
  }

  /** Orbit / zoom from the player's drag and wheel on the town. */
  nudge(dAlpha: number, zoom: number): void {
    this.live?.nudgeTown(dAlpha, zoom);
  }

  /** The player's pan: the camera's target moved (`dx`, `dz`) m over the ground (kept over the land). */
  pan(dx: number, dz: number): void {
    this.live?.panTown(dx, dz);
  }

  /**
   * The player's pan by a drag from screen point (`x0`, `y0`) to (`x1`, `y1`) (CSS pixels, a pointer's last and new
   * place): the land under the pointer follows it, also from over the sky. For a middle-button or Space drag in any
   * step, with or without a tool. The camera stays over the land.
   */
  drag(x0: number, y0: number, x1: number, y1: number): void {
    this.live?.dragTown(x0, y0, x1, y1);
  }

  /** The player's orbit, zoom and pan let go: the camera glides back to the shot (`seconds`; 0 cuts). */
  reframe(seconds = 1.2): void {
    this.live?.reframeTown(seconds);
  }

  /** The preview of the town shown (its built land: `landEdit.ts` `builtOf`), once it is shown. */
  get preview(): WorldStructure | null {
    return this.live?.townShown ? this.world : null;
  }

  // ---- The land in view, for step 1's brushes (`Renderer.landOf` and the rest) ----------------

  /** The town's land as a height field (a copy): its `meta.land`, or surveyed from the land as drawn (100–160 ms). */
  landOf(): Promise<Land> {
    const r = this.live;
    if (!r?.townShown) return Promise.reject(new Error('no town is shown'));
    return r.landOf();
  }

  /** The town drawn from `land` from now on (the brushes change it in place); null: from its landscape again. */
  setLand(land: Land | null): void {
    if (this.live?.townShown) this.live.setLand(land);
  }

  /** Samples of the land given to `setLand` changed: drawn again now, the rest once the stroke is `done`. */
  landChanged(rect: LandRect, done: boolean): void {
    if (this.live?.townShown) this.live.landChanged(rect, done);
  }

  /** Where the land is under a screen point (CSS pixels; a town point and its height, m), if it is. */
  landAt(x: number, y: number): { x: number; y: number; z: number } | null {
    const r = this.live;
    return r?.townShown ? r.pickLand(x, y) : null;
  }

  /** Shows the land brush's cursor (null hides it). */
  brush(cursor: LandCursor | null): void {
    if (this.live?.townShown || !cursor) this.live?.setLandBrush(cursor);
  }

  /**
   * Step 2's edits drawn over the town (`TownGhosts`: streets being drawn, lots placed or to be removed, villages'
   * boxes) until the town is built again with them; null: none. Cheap: every pointer move. They stay drawn while the
   * town is built again (`show`) and over the new one: the editor takes them away once it shows what they were.
   */
  ghosts(g: TownGhosts | null): void {
    this.live?.setTownGhosts(g);
  }

  /**
   * Step 2's painted uses (`uses.ts`) drawn over the town; null: none. `hatched`: a byte per cell of `layer`, set
   * where a zoned cell is too far from a street to be built on. Cheap enough for every stamp of a stroke.
   */
  setUses(layer: UseLayer | null, hatched?: Uint8Array | null): void {
    this.live?.setTownUses(layer, hatched);
  }

  /**
   * How many m a CSS px is on the land at screen point (`x`, `y`) (CSS pixels), across the screen: from where the land
   * is under the points `SCALE_SPAN` px either side of it. Where there is no land under them (the sky, say), the last
   * scale found (1 m before any).
   */
  scale(x: number, y: number): number {
    const [a, b] = [this.landAt(x - SCALE_SPAN, y), this.landAt(x + SCALE_SPAN, y)];
    if (a && b) {
      const m = Math.hypot(b.x - a.x, b.y - a.y, b.z - a.z) / (2 * SCALE_SPAN);
      if (m > 0 && Number.isFinite(m)) this.lastScale = m;
    }
    return this.lastScale;
  }

  /**
   * The road under a screen point (CSS pixels), if any: its index in the town shown (`NeighbourhoodDraft.roads`, the
   * preview's `meta.roads`) and how far along it (m) the point is. On a road's surface out to its sidewalks, or within
   * `ROAD_PICK` m of a narrow one (a trail); of those, the one whose middle is nearest. As precise as `landAt`.
   */
  roadAt(x: number, y: number): { road: number; s: number } | null {
    const at = this.landAt(x, y);
    const roads = this.world?.meta?.roads;
    if (!at || !roads?.length) return null;
    const index = roadIndex(roads);
    let reach = ROAD_PICK;
    for (const r of roads) reach = Math.max(reach, r.width / 2);
    let best: { road: number; s: number; dist: number } | null = null;
    for (const [road, m] of index.around(at.x, at.z, reach)) {
      if (m.dist > Math.max(roads[road].width / 2, ROAD_PICK) || (best && best.dist <= m.dist)) continue;
      best = { road, s: m.s, dist: m.dist };
    }
    return best && { road: best.road, s: best.s };
  }

  /** Projects a town point onto the screen (CSS pixels); false when it is behind the camera. */
  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean {
    const r = this.live;
    return !!r?.townShown && r.project(x, y, z, out);
  }

  /** Plot under a screen point (CSS pixels), if any. */
  plotAt(x: number, y: number): number | null {
    const r = this.live;
    const ground = r?.townShown ? r.pick(x, y).ground : null;
    if (!ground || !this.world) return null;
    const p = this.world.plots.find((p) => ground.x >= p.x && ground.x < p.x + p.w && ground.z >= p.z && ground.z < p.z + p.d);
    return p ? p.id : null;
  }

  /**
   * Pins `el` above a plot (its house's roof); positioned every frame while shown. `rank`: its kind's place when tags
   * compete for room (the lower first); `capped`: a household's, of which only so many show at once (`tagLayout.ts`).
   * Returns a detach function.
   */
  pin(plot: number, el: HTMLElement, rank = 0, capped = false): () => void {
    const entry: Label = { el, x: 0, y: -1000, z: 0, px: NaN, py: NaN, shown: false, crowded: false, edge: false, w: 150, h: 40, spot: { sx: 0, sy: 0, w: 150, h: 40, rank, capped, wins: null, x: 0, y: 0, crowded: false } };
    el.style.visibility = 'hidden';
    this.labels.set(plot, entry);
    this.anchor(plot, entry);
    const tag = el.firstElementChild;
    if (tag) this.sizes.observe(tag);
    return () => {
      if (tag) this.sizes.unobserve(tag);
      if (this.labels.get(plot) === entry) this.labels.delete(plot);
    };
  }

  private showLabel(label: Label): void {
    label.el.style.visibility = label.shown && !label.crowded ? 'visible' : 'hidden';
  }

  /** Label anchor: above the middle of the house's roof (or the middle of the lot). */
  private anchor(plot: number, label: { x: number; y: number; z: number; px: number }): void {
    const p = this.world?.plots[plot];
    if (!p) return;
    const [x0, z0, x1, z1] = p.house ?? [p.x + p.w / 2, p.z + p.d / 2, p.x + p.w / 2, p.z + p.d / 2];
    label.x = (x0 + x1) / 2;
    label.y = p.house ? 7.2 : 5;
    label.z = (z0 + z1) / 2;
    label.px = NaN;
  }

  /** Where the game camera will look at a lot first (as the renderer frames a lot on arrival). */
  private lotAlpha(plot: number, world: WorldStructure): number {
    const p = world.plots[plot];
    const dx = p.entry ? p.entry[0] - (p.x + p.w / 2) : 0;
    const dz = p.entry ? p.entry[1] - (p.z + p.d / 2) : 1;
    return Math.atan2(dz, dx) - Math.PI * 0.12;
  }

  private townShot(shot: Shot, world: WorldStructure, duration: number): TownShot {
    const t = this.baseShot(shot, world, duration);
    // Reduced motion: no drifting camera, short glides.
    if (document.documentElement.classList.contains('reduce-motion') || matchMedia('(prefers-reduced-motion: reduce)').matches) {
      return { ...t, sway: 0, drift: 0, duration: Math.min(t.duration ?? 0, 0.25) };
    }
    return t;
  }

  private baseShot(shot: Shot, world: WorldStructure, duration: number): TownShot {
    // A big world's main town (its first area); else the whole town.
    const main = world.meta?.areas?.[0];
    const town = main ? { x: main.x, z: main.z, w: main.w, d: main.d } : { x: 0, z: 0, w: world.width, d: groundDepth(world) };
    const inTown = main ? world.plots.filter((p) => p.x + p.w / 2 >= main.x && p.x + p.w / 2 <= main.x + main.w && p.z + p.d / 2 >= main.z && p.z + p.d / 2 <= main.z + main.d) : world.plots;
    // The built-up part (plots and street), without the empty margin.
    const plots = inTown.length
      ? {
          x: Math.min(...inTown.map((p) => p.x)),
          z: Math.min(...inTown.map((p) => p.z)),
          w: Math.max(...inTown.map((p) => p.x + p.w)) - Math.min(...inTown.map((p) => p.x)),
          d: Math.max(...inTown.map((p) => p.z + p.d)) - Math.min(...inTown.map((p) => p.z)),
        }
      : town;
    const W = window.innerWidth;
    const H = window.innerHeight;
    switch (shot.kind) {
      case 'menu':
        // Low, towards the evening sun: backlit roofs, glow over the hills.
        return { focus: plots, frame: { left: W * 0.36, top: H * 0.12, right: W * 0.99, bottom: H * 0.96 }, beta: 1.32, alpha: 0.55, zoom: 1.55, sway: 0.32, duration, minute: MENU_MINUTE };
      case 'create':
        return { focus: plots, frame: { left: W * 0.12, top: H * 0.1, right: W * 0.88, bottom: H * 0.98 }, beta: 1.3, alpha: 0.9, zoom: 1.8, sway: 0.25, duration, minute: CREATE_MINUTE };
      case 'world':
        // The whole land, from higher up than an area's shot (a map of it, the brushes' and the homes' ground), looking
        // north: square to the picture, the land fills the stage and the landscape's ring is seen at its sides only.
        // An area's shot looks the same way, so from one to the other the camera only comes down.
        return { focus: { x: 0, z: 0, w: world.width, d: groundDepth(world) }, frame: shot.frame, beta: WORLD_BETA, alpha: MAP_ALPHA, zoom: 1, drift: 0, duration, minute: OVERVIEW_MINUTE };
      case 'area': {
        const focus = areaFocus(world, shot.area);
        return { focus, frame: shot.frame, beta: AREA_BETA, alpha: MAP_ALPHA, zoom: 1, drift: 0, duration, minute: OVERVIEW_MINUTE };
      }
      case 'overview':
        // Among mountains, lower: the peaks round the valley in the picture too.
        return world.meta?.world?.landscape.mountains
          ? // The mountainside behind the town in the picture too (the camera looks from low z).
            { focus: { ...plots, d: plots.d + 190 }, frame: shot.frame, beta: 1.42, alpha: -Math.PI / 2 + 0.32, zoom: 1, drift: 0, duration, minute: OVERVIEW_MINUTE }
          : { focus: plots, frame: shot.frame, beta: 0.8, alpha: -Math.PI / 2 + 0.32, zoom: 1.04, drift: 0, duration, minute: OVERVIEW_MINUTE };
      case 'lot': {
        // The lot with a strip of its neighbours around it, from the side the game opens on.
        const p = world.plots[shot.plot];
        const mx = p.w * 0.45;
        const mz = p.d * 0.25;
        const focus = { x: p.x - mx, z: p.z - mz, w: p.w + mx * 2, d: p.d + mz * 2 };
        return { focus, frame: shot.frame, beta: 0.86, alpha: this.lotAlpha(shot.plot, world), zoom: 1, drift: 0, duration, minute: OVERVIEW_MINUTE };
      }
    }
  }

  private attach(renderer: Renderer): void {
    if (this.live === renderer) return;
    this.live?.onTownFrame(null);
    this.renderer = new WeakRef(renderer);
    this.shotKey = '';
    renderer.onTownFrame(this.positionLabels);
  }

  /**
   * Per overview frame: lays out the pinned tags in screen space (`tagLayout.ts`: each above its lot, inside the
   * stage and clear of the UI over the town, the selected and hovered lots' first, then households', then places').
   * Writes transforms only for tags that moved. A lot whose tag has no room above it (or that is well outside the
   * town in view) gets none: clamped, such tags floated over empty ground and stacked up in the haze under the header.
   */
  private readonly positionLabels = () => {
    const r = this.live;
    if (!r || !this.labels.size) return;
    const b = this.bounds;
    const area: Box = b ? { l: b.left + 8, t: b.top + 8, r: b.right - 8, b: b.bottom - 6 } : { l: 8, t: 8, r: window.innerWidth - 8, b: window.innerHeight - 8 };
    const blocks: Box[] = [];
    for (const el of this.obstacles) {
      if (!el.isConnected) continue;
      const o = el.getBoundingClientRect();
      if (o.width && o.height) blocks.push({ l: o.left - 6, t: o.top - 6, r: o.right + 6, b: o.bottom + 6 });
    }
    // A low camera still projects a hamlet over the horizon onto the stage, over empty ground: a lot
    // well outside the town the shot frames has no tag (unless it is the one selected or pointed at).
    const f = this.framed;
    const onTown = (plot: number) => {
      if (!f) return true;
      const p = this.world?.plots[plot];
      if (!p) return false;
      const x = p.x + p.w / 2;
      const z = p.z + p.d / 2;
      return x >= f.x - REACH && x <= f.x + f.w + REACH && z >= f.z - REACH && z <= f.z + f.d + REACH;
    };
    // The tag's box has to fit above its lot inside the stage. The selected and hovered lots' tags
    // are still shown when it doesn't (pulled in, and moved out from under the header): they win.
    const room = (x: number, y: number, h: number) => x >= area.l && x <= area.r && y - h >= area.t && y <= area.b;
    const wins = (plot: number) => (plot === this.focusPlots[0] ? 0 : plot === this.focusPlots[1] ? 1 : null);
    const placed: Label[] = [];
    for (const [plot, label] of this.labels) {
      const won = wins(plot);
      const visible = label.y > -100 && r.project(label.x, label.y, label.z, this.out) && (won !== null || (onTown(plot) && room(this.out.x, this.out.y, label.h + STEM)));
      if (visible !== label.shown) {
        label.shown = visible;
        this.showLabel(label);
      }
      if (!visible) continue;
      const spot = label.spot;
      spot.sx = this.out.x;
      spot.sy = this.out.y;
      spot.w = label.w;
      spot.h = label.h + STEM;
      spot.wins = won;
      placed.push(label);
    }
    layoutTags(placed.map((l) => l.spot), area, blocks);
    for (const label of placed) {
      const { x, y, crowded, sx, sy } = label.spot;
      if (crowded !== label.crowded) {
        label.crowded = crowded;
        this.showLabel(label);
      }
      if (crowded) continue;
      const edge = Math.abs(x - sx) > 2 || Math.abs(y - sy) > 2;
      if (edge !== label.edge) {
        label.edge = edge;
        label.el.classList.toggle('edge', edge);
      }
      if (Math.abs(x - label.px) < 0.3 && Math.abs(y - label.py) < 0.3) continue;
      label.px = x;
      label.py = y;
      label.el.style.transform = `translate3d(${x.toFixed(1)}px, ${y.toFixed(1)}px, 0)`;
    }
  };
}

/** The whole land's and an area's way round (as the camera's alpha): looking north, as a map. */
const MAP_ALPHA = -Math.PI / 2;
/** The tilt of the whole land's shot (from straight down): steep, so the far side is not lost in the distance. */
const WORLD_BETA = 0.62;
/** The tilt of an area's shot: the overview's, a little steeper (a village's houses stand clear of each other). */
const AREA_BETA = 0.74;
/** A road narrower than twice this (a trail) is picked within this (m) of its middle (`roadAt`). */
const ROAD_PICK = 1.5;
/** Height of a tag's stem below its body (see TownStage). */
const STEM = 10;
/** How far (m) outside the town a shot frames a lot may be and still have its tag (a lot or two around it). */
const REACH = 60;

interface Label {
  el: HTMLElement;
  /** Anchor in the world (above the house). */
  x: number;
  y: number;
  z: number;
  /** Position written last (screen). */
  px: number;
  py: number;
  shown: boolean;
  /** No free spot this frame (a small window): hidden rather than drawn over another tag. */
  crowded: boolean;
  /** Moved away from its lot (the stem is hidden). */
  edge: boolean;
  /** Tag size on screen (CSS pixels). */
  w: number;
  h: number;
  /** Its place in this frame's layout (`tagLayout.ts`). */
  spot: TagSpot;
}

export const menuScene = new MenuScene();
