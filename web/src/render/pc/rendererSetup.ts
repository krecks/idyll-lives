import * as pc from './engine';
import type { GarageInfo, ObjectPlacement, WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { FocusPlot } from '../shared/focus';
import { RectGrid, distanceToRect } from '../shared/chunks';
import type { Landscape } from './nature';
import type { Drape } from './townGhosts';
import { LAND_CELL, NONE } from '../../game/land';
import { WATER_Y } from '../shared/nature';
import { createLighting, lightingAt, type Lighting, type RGB } from '../shared/environment';
import { LOOK } from '../look';
import { toLinear } from '../shared/math';

export const LAND_SLICE_MS = 6;

declare global {
  interface Window {
    /** The renderer's own hook for the dev tools and the benches (`?debug`); see `render/types.ts`. */
    __renderer?: unknown;
    __scene?: unknown;
    __app?: unknown;
  }
}

/** The town view's buildings fade in over this much of its blend (a dither: nothing pops in past `TOWN_FROM`). */
export const TOWN_FADE = 0.03;
/** A far bike's frame (sRGB). */
export const BIKE: readonly [number, number, number] = [0.22, 0.22, 0.24];

/** Longest wait (ms) for the birds of a game's first build (their models built): they'd pop in in play otherwise. */
export const BIRDS_WAIT_MS = 3000;
/** Idle time (ms) a frame must have left for a model file to be read ahead of time (`parseAhead`). */
export const PARSE_IDLE_MS = 8;
/** How long (ms) a catalog card must be pointed at before its model is read (`prepareAsset`). */
export const PREPARE_AFTER_MS = 150;
/** Light is recomputed when the game's clock moved this many minutes (fast: the preset's `lightingStep`), at most every `LIGHT_MIN_MS`. */
export const LIGHT_STEP = 0.5;
export const LIGHT_MIN_MS = 100;
/** Frame after frame failing this many times is drawing broken for good. */
export const FAILED_FRAMES_LIMIT = 60;
/** Nothing waits for frames longer than this (ms). */
export const FRAME_WAIT_LIMIT_MS = 15_000;
/** The longest step a frame takes (ms): a stall isn't a jump. */
export const MAX_FRAME_MS = 100;
/** Frames still drawn once a full-screen menu covers the view (its frosted glass shows the last). */
export const HOLD_FRAMES = 2;
/**
 * The paused game's frames go on being drawn this long (ms) after a change before the picture is held
 * (`heldFrames.ts`): longer than the slowest rate anything is drawn again at (the sun's map and the lamps'
 * shadows 15 a second on Low, the reflections 10).
 */
export const GAME_SETTLE_MS = 250;
/** While water is in view, a view that would be held is drawn this often (ms: 30 a second) so it goes on flowing. */
export const WATER_FRAME_MS = 1000 / 30;
/** Metres the sun's shadow square reaches past the viewed lot. */
export const SHADOW_MARGIN = 10;
/** The game camera's target eases to the ground under it over this (s): `settleTarget`. */
export const TARGET_EASE = 0.25;
/** The basement's pit (and the ground dug away over it) reaches this far past the plot (m), clear of walls on its edge. */
export const PIT_MARGIN = 0.3;
/** The doors and gates opening or falling shut kept for the world's sound between its looks (`doorSounds`). */
export const DOOR_SOUNDS = 16;
/** Cells (m) the residents on the move are sorted into for the gates (`swingDoors`). */
export const GATE_CELL = 8;
/** Camera distance below which the viewed lot's roof is hidden even with walls up. */
export const ROOF_MIN_DISTANCE = 15;
/** A roof's height (m) over a house's top storey, at most, for a click on it (`buildingAt`). */
export const PICK_ROOF = 4;
/** Main-thread time (ms) a world build runs before the frames it runs behind get one (`pace`). */
export const BUILD_SLICE_MS = 8;
/**
 * How long after the game comes on screen the menus' town is let go (`releaseTown`): its destroy
 * (some 20 ms) after the reveal's frames and the fly-in's last ones.
 */
export const TOWN_RELEASE_MS = 1000;
/** The day of the week the menus' town is shown on (0 Monday): a working day, its shops and offices open by day. */
export const MENU_WEEKDAY = 2;
/** The same while the camera roams (a big world, or once it has settled on a lot of its own). */
export const ROAM_SLICE_MS = 4;
/**
 * The same behind the loading screen (`setCovered`, and the warm-up): nobody sees the frames, and
 * each one given costs the build a frame's wait. At 4 ms a slice a game's first build ran a quarter of
 * the time; at this it runs most of it, and the loading screen's bar still moves (20 frames a second).
 */
export const COVERED_SLICE_MS = 40;
/**
 * The same while the menus' town is built again over the one on screen (step 2's edits): the old
 * town stays drawn and the camera still, so a frame given up now and then shows nothing, and at 8 ms
 * a slice an edit's town took twice as long to come.
 */
export const AGAIN_SLICE_MS = 32;
/**
 * Daylight doesn't reach under the ground or into a room without windows: no sun, only this much
 * of the sky fill and the environment (as seen, intensity times exposure: the same by day and by
 * night). A windowless room up here keeps that little, so its lamps tell. The basement in view
 * gets more, a soft fill to see it by: all its rooms are windowless, and residents switch an empty
 * room's lamps off (sim-core `lights.rs`), so it would be lit only where someone is. Still well
 * under daylight, so a lit room shows warm against the rest. By night its fill is as bright as by
 * day (lamps or not): the night's dim, deep blue sky lifted to it, its colour's brightness against
 * noon's too (`skyLuma`), up to `BASEMENT_NIGHT_GAIN` times; a windowless room's share stops at the
 * night's own.
 */
export const WINDOWLESS_SKY = 0.12;
export const WINDOWLESS_ENV = 0.12;
export const BASEMENT_SKY = 0.35;
export const BASEMENT_ENV = 0.35;
export const BASEMENT_NIGHT_GAIN = 12;
/** How much of the moonlight grade (`nightGrade.ts`) is left while the basement is in view. */
export const BASEMENT_NIGHT_GRADE = 0.3;
/** How bright the sky fill's colours are at noon (`skyLuma`): the basement's fill is matched to it. */
export const NOON_SKY_LUMA = skyLuma(lightingAt(13 * 60, LOOK, createLighting()));

/** Residents this far (m) outside the viewed lot are still drawn (walking up to it). */
export const SIM_VIEW_MARGIN = 2;
/** How far round the watch rectangle the sim keeps residents in full detail (m: sim-core `lod::VIEW_MARGIN`; `reportWatch`). */
export const DETAIL_MARGIN = 5;
/**
 * At super speed (`setDetailTight`), the sim's full-detail area is the streamed chunks within this
 * many chunks of the one the camera looks at, not all of them: a resident out of it takes a turn
 * every game minute, which at super speed is every step or two anyway, and the town ticks faster.
 */
export const TIGHT_CHUNKS = 1;
/** Residents further than this (m) from the camera's target, off the viewed lot, have no contact shadow. */
export const FAR_SHADOW = 40;
/** `?residents=lot`: residents drawn only on the viewed lot, as before life on every lot. */
export const RESIDENTS_LOT_ONLY = typeof location !== 'undefined' && new URLSearchParams(location.search).get('residents') === 'lot';

/** The camera's layers besides the world's that it keeps while the menus' town is shown (`showLayers`). */
export const TOWN_CAMERA_LAYERS: readonly number[] = [pc.LAYERID_DEPTH, pc.LAYERID_IMMEDIATE, pc.LAYERID_UI];

/** A directional light's way before it's turned (PlayCanvas shines it along its entity's -y). */
export const DOWN = new pc.Vec3(0, -1, 0);
/** (the warm-up's red copies, `drawRedCopies`.) */
export const IDENTITY = Float32Array.of(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);

/** No neighbours' houses (to look through). */
export const NO_HOUSES: readonly (readonly [number, number, number, number])[] = [];

/** Whether two worlds' garages are the same (field by field, as `sameObject`). */
export function sameGarages(a: readonly GarageInfo[] | undefined, b: readonly GarageInfo[] | undefined): boolean {
  if (a === b) return true;
  if (!a || !b || a.length !== b.length) return false;
  return a.every((g, i) => {
    const h = b[i];
    return g.plot === h.plot && g.x0 === h.x0 && g.z0 === h.z0 && g.x1 === h.x1 && g.z1 === h.z1 && g.rot === h.rot;
  });
}

/** Whether two lists of flat records are the same, field by field (the porches, the porch pieces). */
export function sameList<T extends object>(a: readonly T[] | undefined, b: readonly T[] | undefined): boolean {
  if (a === b) return true;
  if ((a?.length ?? 0) !== (b?.length ?? 0)) return false;
  if (!a || !b) return true;
  return a.every((p, i) => {
    const q = b[i] as Record<string, unknown>;
    return Object.entries(p).every(([k, v]) => q[k] === v);
  });
}

/**
 * Whether two placements draw the same: everything an object is drawn from, but its open state and
 * growth (drawn without a build: see `patchable`). (Wear, quality and what it's worth don't change the model.)
 */
export function sameObject(a: ObjectPlacement, b: ObjectPlacement): boolean {
  // (Field by field: a big world's lot sends 6 000 objects with every opened door, and their keys as JSON
  // took 4-5 ms of each such message.)
  const pair = (u: readonly number[] | undefined, v: readonly number[] | undefined) => u === v || (!!u && !!v && u[0] === v[0] && u[1] === v[1]);
  return (
    a.id === b.id &&
    a.def === b.def &&
    a.x === b.x &&
    a.z === b.z &&
    a.rot === b.rot &&
    a.turn === b.turn &&
    pair(a.offset, b.offset) &&
    pair(a.size, b.size) &&
    a.tall === b.tall &&
    a.lift === b.lift &&
    a.w === b.w &&
    a.d === b.d &&
    (a.colors === b.colors || (!!a.colors && !!b.colors && JSON.stringify(a.colors) === JSON.stringify(b.colors)))
  );
}

/** `?stream=1`: the street streams its chunks round the camera in any town (else only in a big world's). */
export const STREAM_FLAG = typeof location !== 'undefined' && new URLSearchParams(location.search).get('stream') === '1';

/** The plot (by index) under a ground point, else the nearest within `reach` m (every plot looked at: see `PlotFinder` for each frame's). */
export function plotNear(world: WorldStructure, x: number, z: number, reach: number): FocusPlot | null {
  let best: FocusPlot | null = null;
  let bestD = reach;
  world.plots.forEach((p, i) => {
    const d = distanceToRect(x, z, p.x, p.z, p.w, p.d);
    if (d < bestD || (d === 0 && !best)) [best, bestD] = [{ id: i, x: p.x, z: p.z, w: p.w, d: p.d }, d];
  });
  return best;
}

/**
 * `plotNear` for the roaming camera, each frame: the plots in a grid of 16 m cells (made once a
 * world), and the last answer kept while the point, the reach and the world are the same (a still
 * camera asks nothing of the grid). The same plot as `plotNear` gives for a finite reach.
 */
export class PlotFinder {
  private world: WorldStructure | null = null;
  private grid = new RectGrid<{ x: number; z: number; w: number; d: number; i: number }>(16);
  private readonly asked = { x: NaN, z: NaN, reach: NaN };
  private answer: FocusPlot | null = null;

  find(world: WorldStructure, x: number, z: number, reach: number): FocusPlot | null {
    if (world !== this.world) {
      this.world = world;
      this.grid = new RectGrid(16);
      world.plots.forEach((p, i) => this.grid.add({ x: p.x, z: p.z, w: p.w, d: p.d, i }));
      this.asked.x = NaN;
    }
    const a = this.asked;
    if (x === a.x && z === a.z && reach === a.reach) return this.answer;
    a.x = x;
    a.z = z;
    a.reach = reach;
    if (!Number.isFinite(reach)) return (this.answer = plotNear(world, x, z, reach));
    // (In plot order, as `plotNear` goes: the first of plots as near wins.)
    let best = -1;
    let bestD = reach;
    for (const c of this.grid.near({ x, z, w: 0, d: 0 }, reach)) {
      const d = distanceToRect(x, z, c.x, c.z, c.w, c.d);
      if (d < bestD || (d === bestD && c.i < best) || (d === 0 && best < 0)) [best, bestD] = [c.i, d];
    }
    const p = best >= 0 ? world.plots[best] : null;
    if (p?.x === this.answer?.x && p?.z === this.answer?.z && this.answer?.id === best) return this.answer;
    return (this.answer = p && { id: best, x: p.x, z: p.z, w: p.w, d: p.d });
  }
}

/** A world build stopped because the roaming camera settled on another lot before it was done (`pace`). */
export class BuildSuperseded extends Error {}

/** The same part of the town in view (the messages make a new rect each time). */
/** Where step 2's ghosts lie on a landscape (`townGhosts.ts`): its ground, or its water's top where it is wet (a bridge's ghost over a river). */
export function groundOrWater(scape: Landscape): Drape {
  const field = scape.heightField;
  return (x, z) => {
    const h = scape.heightAt(x, z);
    if (!field) return scape.waterEdge(x, z) > 0 ? Math.max(h, WATER_Y) : h;
    const [i, j] = [Math.round(x / LAND_CELL), Math.round(z / LAND_CELL)];
    if (i < 0 || j < 0 || i >= field.cols || j >= field.rows) return h;
    const k = j * field.cols + i;
    return field.wl[k] !== NONE && field.wl[k] > field.h[k] ? Math.max(h, field.wl[k] / 100) : h;
  };
}

/** A number of each landscape's own, for what is made from it (`LandHaze.set`). */
const landscapeIds = new WeakMap<object, number>();
let landscapesSeen = 0;
export function landscapeId(scape: object): number {
  const id = landscapeIds.get(scape) ?? ++landscapesSeen;
  landscapeIds.set(scape, id);
  return id;
}

export function sameView(a: ViewRect | null, b: ViewRect | null): boolean {
  return a === b || (!!a && !!b && a.x === b.x && a.z === b.z && a.w === b.w && a.d === b.d);
}

/** `?lightmix=sun,sky,env`: scales of the look's sun, sky fill and environment light (1 each by default). */
export function lightMixFromURL(): [number, number, number] | null {
  const v = new URLSearchParams(location.search).get('lightmix');
  if (!v) return null;
  const [sun = 1, sky = 1, env = 1] = v.split(',').map(Number);
  return [sun, sky, env].map((x) => (Number.isFinite(x) ? x : 1)) as [number, number, number];
}

/** How bright a sky fill of `l`'s colours is, for an intensity of 1: the luminance of its sky and ground, averaged (linear). */
export function skyLuma(l: Lighting): number {
  const luma = (c: RGB) => 0.2126 * toLinear(c.r) + 0.7152 * toLinear(c.g) + 0.0722 * toLinear(c.b);
  return Math.max((luma(l.skyColor) + luma(l.groundColor)) / 2, 1e-3);
}

/** A hex colour (sRGB) in linear light (`pow(c, 2.2)`). */
export function linearHex(hex: string): RGB {
  const v = parseInt(hex.slice(1), 16);
  return { r: toLinear(((v >> 16) & 255) / 255), g: toLinear(((v >> 8) & 255) / 255), b: toLinear((v & 255) / 255) };
}

/** A linear colour (the look's keys are used as linear) as PlayCanvas takes it (sRGB, linearised when lit). */
export function gammaColor(c: RGB, scale = 1, out = new pc.Color()): pc.Color {
  const g = (v: number) => Math.pow(Math.max(0, v * scale), 1 / 2.2);
  return out.set(g(c.r), g(c.g), g(c.b), 1);
}

