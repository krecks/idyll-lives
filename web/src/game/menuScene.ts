/**
 * The live 3D scene behind the menus: which town the overview shows (a showcase neighbourhood
 * on the main menu, the draft being edited during a new game), its camera shots, lot
 * highlights, labels pinned to lots and lot picking. Screens call this; it reaches the renderer
 * through the host (never the sim worker).
 */

import { groundDepth, type WorldStructure } from '../core/protocol';
import type { LotHighlight, Renderer, TownShot } from '../render/types';
import { settings } from '../settings/settings.svelte';
import { services } from '../ui/services';
import { host } from './host';
import { loadTemplates, populate, primeHouses, type NeighbourhoodDraft, type Templates } from './town';
import { keepHouses, shareHouses, townJob } from './townJobs';
import { loadShowcase, showcaseSeed } from './showcaseCache';
import { previewWorld } from './townPreview';
import { perfPhase, perfTime, perfTimeAsync } from '../debug/perf';
import { layoutTags, type Box, type TagSpot } from './tagLayout';

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
  | { kind: 'lot'; plot: number; frame: Frame };

const NO_HIGHLIGHT: LotHighlight = { hover: null, selected: null, marked: [], outlines: false };

class MenuScene {
  private templates: Templates | null = null;
  private showcaseMade: Promise<NeighbourhoodDraft> | null = null;
  private draft: NeighbourhoodDraft | null = null;
  /** The lot shown as the player's house (null: none). */
  private home: number | null = null;
  private world: WorldStructure | null = null;
  private worldOf: NeighbourhoodDraft | null = null;
  private worldHome: number | null = null;
  /** Previews made elsewhere (the town worker: `townJobs.ts`), by draft, with no home. */
  private readonly prepared = new WeakMap<NeighbourhoodDraft, WorldStructure>();
  private shotKey = '';
  private wanted: { shot: Shot; duration: number } | null = null;
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
   * The main menu's neighbourhood (made once per visit to the page). Its land is laid out in the
   * town worker (seconds for a big world: the page goes on meanwhile, the boot warming the portrait
   * studio up), then peopled here (`populate`, on the page's dice, as before) and its preview made in
   * the worker too (`prepare`d: `show` draws it as it comes).
   */
  showcaseDraft(): Promise<NeighbourhoodDraft> {
    this.showcaseMade ??= this.makeShowcase().catch((err: unknown) => {
      // (Asked for again, it is made again.)
      this.showcaseMade = null;
      throw err;
    });
    return this.showcaseMade;
  }

  private async makeShowcase(): Promise<NeighbourhoodDraft> {
    perfPhase('menu: showcase draft');
    // A returning visit: the same valley, its people and its preview, as the last visit left them
    // (`showcaseCache.ts`). Read it back (a few dozen milliseconds), not laid out again.
    const kept = await perfTimeAsync('menu: showcase kept', loadShowcase);
    if (kept) {
      primeHouses(kept.houses);
      shareHouses(kept.draft, kept.houses);
      this.prepare(kept.draft, kept.preview);
      return kept.draft;
    }
    const { content, assets } = services;
    const seed = showcaseSeed();
    const land = await perfTimeAsync('menu: showcase land', () => townJob(content, { kind: 'land', world: 'medium', seed }), `seed ${seed}`);
    keepHouses(land);
    const templates = (this.templates ??= await perfTimeAsync('menu: loadTemplates', loadTemplates));
    const draft = perfTime('menu: populate', () => populate(content, assets, templates, land.draft));
    const { preview } = await perfTimeAsync('menu: showcase preview', () => townJob(content, { kind: 'preview', town: draft, keep: true }));
    this.prepare(draft, preview);
    return draft;
  }

  /**
   * Shows a neighbourhood behind the menus (the renderer rebuilds it only when its layout
   * changed). Again after a game, too: starting one hides the overview. `home`: the lot the
   * player will live on (the chosen home, the household's), drawn as the game will show it.
   */
  async show(draft: NeighbourhoodDraft, home: number | null = null): Promise<void> {
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
    await perfTimeAsync('menu: showTown (renderer)', () => host.showTown(world, MENU_MINUTE, { ...settings }));
  }

  /** A draft's preview (`previewWorld` with no home), already made: `show` uses it. */
  prepare(draft: NeighbourhoodDraft, world: WorldStructure): void {
    this.prepared.set(draft, world);
  }

  /** The town currently shown. */
  get shown(): NeighbourhoodDraft | null {
    return this.draft;
  }

  /** Points the camera (applied once the town is shown); repeated identical shots are ignored. */
  shoot(shot: Shot, duration = 1.6): void {
    this.wanted = { shot, duration };
    this.applyShot();
  }

  private applyShot(): void {
    const world = this.world;
    const renderer = this.live;
    if (!world || !renderer || !this.wanted) return;
    const { shot, duration } = this.wanted;
    this.bounds = shot.kind === 'overview' || shot.kind === 'lot' ? shot.frame : null;
    const townShot = this.townShot(shot, world, duration);
    // What the camera frames is the town as far as the tags go (see `positionLabels`); before the key
    // check, since a new world of the same size and shot can leave the key unchanged.
    this.framed = shot.kind === 'overview' || shot.kind === 'lot' ? townShot.focus : null;
    const key = `${JSON.stringify(shot)}|${world.width}x${groundDepth(world)}|${window.innerWidth}x${window.innerHeight}`;
    if (key === this.shotKey) return;
    const first = this.shotKey === '';
    this.shotKey = key;
    renderer.setTownShot(townShot, first);
    this.glideEnd = performance.now() + (first ? 0 : (townShot.duration ?? 0) * 1000);
  }

  /** Resolves once the camera has finished gliding (also to shots set while waiting). */
  async settled(): Promise<void> {
    for (let wait = this.glideEnd - performance.now(); wait > 0; wait = this.glideEnd - performance.now()) {
      await new Promise((resolve) => setTimeout(resolve, wait));
    }
  }

  highlight(h: Partial<LotHighlight>): void {
    this.focusPlots = [h.selected ?? null, h.hover ?? null];
    this.live?.setLotHighlight({ ...NO_HIGHLIGHT, ...h });
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
