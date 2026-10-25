/**
 * The engine-neutral half of the lawn's grass (`pc/grass.ts` draws it): where it grows, how dense
 * it is drawn with distance, and the gusts that roll across it.
 *
 * The blades themselves are built on the GPU, each from a hash of its patch and its number, so
 * nothing here places a blade. What the GPU reads is made here:
 *
 * - `GrassField`: the land round the town in two textures. `colour` (RGBA, 1 texel a metre over the
 *   town and `GRASS.margin` round it): the ground's own colour under the grass (sRGB, the terrain's
 *   vertex colour, `LandscapeLayout.terrainColour`) and, in alpha, how grassy the land is out
 *   there (roads, water, sand, fields, rock: none). `town` (one byte a texel, `GRASS.townTexels`
 *   a metre over the town's ground): the lawn as it grows now, none indoors, under furniture, on
 *   patios, streets, paths and fields, nor right by a wall (`townGrass`, again on `refreshLawn`).
 * - Which 2 m patches have any grass at all (`patches`), and each one's height and slope.
 * - `gustNoise`: a small tileable noise the gusts are read from, carried along the wind (frozen
 *   turbulence: the pattern drifts, it doesn't change), and `gustAt`, its CPU twin.
 *
 * All of it is deterministic: the same town grows the same lawn and the same gusts.
 */

import { groundDepth, type WorldStructure } from '../../core/protocol';
import type { RGB } from './environment';
import { bareTiles, fbm, lawnMask, smoothstep, splitHalf, WIND } from './nature';

/** Blades a patch at full density (`GRASS.perPatch`), and the draws: each a step of √2 sparser (`GRASS.lods`). */
const PER_PATCH = 448;
const LOD_COUNT = 7;

export const GRASS = {
  /** How far the grass grows round the town (m). */
  margin: 40,
  /** The town mask's texels a metre. */
  townTexels: 4,
  /** Patches (m square): the grass is drawn a patch at a time. */
  patch: 2,
  /** No grass this close to a wall (m): it would poke through. */
  wallClear: 0.22,
  /** Blades a patch at full density (112 a square metre). */
  perPatch: PER_PATCH,
  /** Full density within this distance of the eye (m); past it, the share drawn falls as `near / d` and each blade widens to keep the cover. */
  near: 20,
  /**
   * The draws: each with a share of `perPatch` (√2 apart) and its blades' segments: curved (3) up
   * close, then 2, then a single triangle past about 28 m, where a curve is a pixel or two.
   */
  lods: Array.from({ length: LOD_COUNT }, (_, k) => ({ blades: Math.round(PER_PATCH / Math.SQRT2 ** k), segments: Math.max(1, 3 - k) })),
  /**
   * Gusts: the noise's texels across, the metres it spans, the second octave's span, weight and
   * turn against the first (radians: the two never line up), how fast they roll (m/s along
   * `WIND.dir`), and the noise's values spread into calm (`calm` and under) and full gusts (`strong`).
   */
  gust: { size: 64, span: 56, span2: 19, weight2: 0.35, turn: 0.6, speed: 4.5, calm: 0.3, strong: 0.75 },
  /** The ground texture's mean linear albedo (`textures/grass_lush`): the blades take the ground's colour times it. */
  groundAlbedo: 0.7,
} as const;

/** How the grass is drawn (`QualitySettings.grass`). */
export interface GrassQuality {
  /** Drawn within this distance of the eye (m); the ground carries the lawn past it. */
  radius: number;
  /** The share of the blades drawn (1 as made); fewer are drawn wider, covering the ground as well. */
  density: number;
  /** The nearest draw used (`GRASS.lods`): 1 leaves out the curved, 3-segment blades. */
  nearest: number;
}

/** What the grass needs to know of the land (`LandscapeLayout`): its height, and its colour and how grassy it is. */
export interface GrassLand {
  heightAt(x: number, z: number): number;
  /** The terrain's colour at a point of height `h` (sRGB, into `out`), and how much of it is lawn (0..1). */
  terrainColour(x: number, z: number, h: number, out: RGB): number;
  distanceToTown(x: number, z: number): number;
}

/** The grass of a town (see the file's header). Game space; `x0`, `z0`: the corner of the colour texture and of the patches. */
export interface GrassField {
  x0: number;
  z0: number;
  /** The colour texture's texels (1 a metre) across and deep. */
  w: number;
  d: number;
  /** RGBA bytes, rows along x: sRGB ground colour, alpha the land's grassiness. */
  colour: Uint8Array;
  /** The town mask: `townW` × `townD` bytes over the town's ground from (0, 0), `GRASS.townTexels` a metre. */
  townW: number;
  townD: number;
  town: Uint8Array;
  /** Patches across and deep from (x0, z0), and per patch: 1 where any grass grows. */
  pw: number;
  pd: number;
  patches: Uint8Array;
  /** Per patch: the ground's height at its corner (x0, z0 side) and its slope along x and z (3 floats). */
  ground: Float32Array;
}

/** The grass of a town as the land grows it (the town's mask comes with `townGrass`, again whenever the lot changes). */
export async function grassField(world: WorldStructure, land: GrassLand, pace?: () => Promise<void>): Promise<GrassField> {
  const m = GRASS.margin;
  const [tw, td] = [world.width, groundDepth(world)];
  const x0 = -m;
  const z0 = -m;
  const w = Math.ceil(tw + m * 2);
  const d = Math.ceil(td + m * 2);
  const colour = new Uint8Array(w * d * 4);
  const tmp: RGB = { r: 0, g: 0, b: 0 };
  for (let j = 0; j < d; j++) {
    if (j % 16 === 0) await pace?.();
    for (let i = 0; i < w; i++) {
      // (Texel centres: the GPU filters between them.)
      const x = x0 + i + 0.5;
      const z = z0 + j + 0.5;
      const lawn = land.terrainColour(x, z, land.heightAt(x, z), tmp);
      // Fading out towards the margin's edge, where the blades stop.
      const edge = 1 - smoothstep(m * 0.7, m * 0.95, land.distanceToTown(x, z));
      const o = (j * w + i) * 4;
      colour[o] = byte(tmp.r);
      colour[o + 1] = byte(tmp.g);
      colour[o + 2] = byte(tmp.b);
      colour[o + 3] = byte(lawn * edge);
    }
  }
  const p = GRASS.patch;
  const pw = Math.ceil(w / p);
  const pd = Math.ceil(d / p);
  // The ground's height at each patch's middle, then its slope from its neighbours' (one-sided at the edges).
  const mid = new Float32Array(pw * pd);
  for (let j = 0; j < pd; j++) {
    if (j % 16 === 0) await pace?.();
    for (let i = 0; i < pw; i++) mid[j * pw + i] = land.heightAt(x0 + (i + 0.5) * p, z0 + (j + 0.5) * p);
  }
  const ground = new Float32Array(pw * pd * 3);
  for (let j = 0; j < pd; j++) {
    for (let i = 0; i < pw; i++) {
      const at = (a: number, b: number) => mid[Math.min(pd - 1, Math.max(0, b)) * pw + Math.min(pw - 1, Math.max(0, a))];
      const sx = (at(i + 1, j) - at(i - 1, j)) / (p * (Math.min(pw - 1, i + 1) - Math.max(0, i - 1)));
      const sz = (at(i, j + 1) - at(i, j - 1)) / (p * (Math.min(pd - 1, j + 1) - Math.max(0, j - 1)));
      const k = (j * pw + i) * 3;
      ground[k] = mid[j * pw + i] - (sx + sz) * (p / 2);
      ground[k + 1] = sx;
      ground[k + 2] = sz;
    }
  }
  const field: GrassField = { x0, z0, w, d, colour, townW: 0, townD: 0, town: new Uint8Array(0), pw, pd, patches: new Uint8Array(pw * pd), ground };
  townGrass(field, world);
  return field;
}

/** 0..1 as a byte. */
function byte(v: number): number {
  return Math.max(0, Math.min(255, Math.round(v * 255)));
}

/**
 * The lawn as it grows on the town's ground now, into `field.town` (and the patches with any grass
 * again): none indoors (on a tile split by a diagonal wall, on its indoor half), under furniture,
 * on patios, streets, paths and fields, nor within `GRASS.wallClear` of a wall.
 */
export function townGrass(field: GrassField, world: WorldStructure): void {
  const n = GRASS.townTexels;
  const [tw, td] = [world.width, groundDepth(world)];
  const W = tw * n;
  const D = td * n;
  const town = field.town.length === W * D ? field.town : new Uint8Array(W * D);
  field.town = town;
  field.townW = W;
  field.townD = D;
  const mask = lawnMask(world);
  const bare = bareTiles(world, mask);
  const walls = wallSegments(world, tw, td);
  for (let j = 0; j < D; j++) {
    for (let i = 0; i < W; i++) {
      const fx = (i + 0.5) / n;
      const fz = (j + 0.5) / n;
      const tx = Math.floor(fx);
      const tz = Math.floor(fz);
      const t = tz * tw + tx;
      let grows = bare[t] === 0;
      if (bare[t] > 1) {
        // A split tile: grass on its outdoor half only.
        grows = splitHalf(mask.splitAxis.get(t) ?? 'dp', fx - tx, fz - tz) !== bare[t] - 2;
      }
      if (grows && nearWall(walls.get(t), fx, fz)) grows = false;
      town[j * W + i] = grows ? 255 : 0;
    }
  }
  markPatches(field);
}

/** Wall edges and diagonals as segments, by the town tiles they pass through or touch. */
function wallSegments(world: WorldStructure, tw: number, td: number): Map<number, [number, number, number, number][]> {
  const byTile = new Map<number, [number, number, number, number][]>();
  const add = (ax: number, az: number, bx: number, bz: number): void => {
    const seg: [number, number, number, number] = [ax, az, bx, bz];
    for (let z = Math.max(0, Math.min(az, bz) - 1); z <= Math.min(td - 1, Math.max(az, bz)); z++) {
      for (let x = Math.max(0, Math.min(ax, bx) - 1); x <= Math.min(tw - 1, Math.max(ax, bx)); x++) {
        const i = z * tw + x;
        const list = byTile.get(i);
        if (list) list.push(seg);
        else byTile.set(i, [seg]);
      }
    }
  };
  // (The ground storey's only: the other storeys' rows follow its, and their first row's walls would clear its far edge.)
  for (const e of world.walls) {
    if (e.axis === 'h') {
      if (e.z <= td) add(e.x, e.z, e.x + 1, e.z);
    } else if (e.z < td) add(e.x, e.z, e.x, e.z + 1);
  }
  for (const d of world.diagonals ?? []) {
    if (d.z >= td) continue;
    if (d.axis === 'dp') add(d.x, d.z, d.x + 1, d.z + 1);
    else add(d.x, d.z + 1, d.x + 1, d.z);
  }
  return byTile;
}

/** Whether (px, pz) is within `GRASS.wallClear` of one of `segs`. */
function nearWall(segs: [number, number, number, number][] | undefined, px: number, pz: number): boolean {
  if (!segs) return false;
  for (const [ax, az, bx, bz] of segs) {
    const [vx, vz] = [bx - ax, bz - az];
    const t = Math.max(0, Math.min(1, ((px - ax) * vx + (pz - az) * vz) / (vx * vx + vz * vz)));
    if (Math.hypot(px - (ax + vx * t), pz - (az + vz * t)) < GRASS.wallClear) return true;
  }
  return false;
}

/** How much grass grows at (x, z): the town's mask over the town, the land's grassiness everywhere (0..1, unfiltered). */
export function grassAt(field: GrassField, x: number, z: number): number {
  const i = Math.floor(x - field.x0);
  const j = Math.floor(z - field.z0);
  if (i < 0 || j < 0 || i >= field.w || j >= field.d) return 0;
  let g = field.colour[(j * field.w + i) * 4 + 3] / 255;
  const n = GRASS.townTexels;
  const ti = Math.floor(x * n);
  const tj = Math.floor(z * n);
  if (ti >= 0 && tj >= 0 && ti < field.townW && tj < field.townD) g = Math.min(g, field.town[tj * field.townW + ti] / 255);
  return g;
}

/** Marks the patches with any grass (`field.patches`). */
function markPatches(field: GrassField): void {
  const p = GRASS.patch;
  const n = GRASS.townTexels;
  field.patches.fill(0);
  for (let j = 0; j < field.pd; j++) {
    for (let i = 0; i < field.pw; i++) {
      const px = field.x0 + i * p;
      const pz = field.z0 + j * p;
      let any = false;
      // The land's grassiness over the patch (its texels, and the filter's reach past them).
      for (let z = Math.floor(pz - field.z0) - 1; z <= Math.floor(pz - field.z0) + p && !any; z++) {
        for (let x = Math.floor(px - field.x0) - 1; x <= Math.floor(px - field.x0) + p && !any; x++) {
          if (x < 0 || z < 0 || x >= field.w || z >= field.d) continue;
          if (field.colour[(z * field.w + x) * 4 + 3] > 0) any = true;
        }
      }
      // In town, the lawn's mask too: a patch all indoors, or all street, has none.
      if (any && px + p > 0 && pz + p > 0 && px < field.townW / n && pz < field.townD / n) {
        any = false;
        for (let z = Math.max(0, Math.floor(pz * n) - 1); z < Math.min(field.townD, Math.ceil((pz + p) * n) + 1) && !any; z++) {
          for (let x = Math.max(0, Math.floor(px * n) - 1); x < Math.min(field.townW, Math.ceil((px + p) * n) + 1) && !any; x++) {
            if (field.town[z * field.townW + x] > 0) any = true;
          }
        }
        // (Part of a patch at the town's edge lies outside its mask: there, the land's grassiness counts.)
        if (!any && (px < 0 || pz < 0 || px + p > field.townW / n || pz + p > field.townD / n)) any = true;
      }
      field.patches[j * field.pw + i] = any ? 1 : 0;
    }
  }
}

// ---- Density with distance ------------------------------------------------------------

/**
 * The share of a patch's blades drawn at `d` m from the eye: `density` (the preset's) within
 * `GRASS.near`, falling as `near / d` past it. Each blade drawn widens by its inverse (up to
 * `BLADE.maxWiden`, `shaders/grass.ts`), so the cover stays the same.
 */
export function grassShare(d: number, density = 1): number {
  return Math.min(1, GRASS.near / Math.max(d, 1e-3)) * density;
}

/** The draw (`GRASS.lods`) for a patch whose nearest point is `d` m from the eye: the sparsest that still has every blade its share asks for. */
export function grassLod(d: number, density = 1): number {
  const want = grassShare(d, density) * GRASS.perPatch;
  const lods = GRASS.lods;
  for (let k = lods.length - 1; k > 0; k--) if (lods[k].blades >= want) return k;
  return 0;
}

// ---- Gusts ------------------------------------------------------------------------------

/**
 * The gusts' noise: `GRASS.gust.size` square, tileable (`fbm`'s three octaves over a lattice of 4
 * cells that wraps round), one byte a texel. Read with wrapping, linear filtering.
 */
export function gustNoise(): Uint8Array {
  const n = GRASS.gust.size;
  const out = new Uint8Array(n * n);
  // (Three octaves sum to at most 0.875.)
  for (let j = 0; j < n; j++) for (let i = 0; i < n; i++) out[j * n + i] = byte(fbm((i / n) * 4, (j / n) * 4, 101, 3, 4) / 0.875);
  return out;
}

/** `noise` (`gustNoise`) at (u, v) in tiles, as the GPU reads it: wrapping, linear between texel centres. */
export function sampleGust(noise: Uint8Array, u: number, v: number): number {
  const n = GRASS.gust.size;
  const x = u * n - 0.5;
  const z = v * n - 0.5;
  const xi = Math.floor(x);
  const zi = Math.floor(z);
  const fx = x - xi;
  const fz = z - zi;
  const at = (a: number, b: number) => noise[(((b % n) + n) % n) * n + (((a % n) + n) % n)] / 255;
  const top = at(xi, zi) + (at(xi + 1, zi) - at(xi, zi)) * fx;
  const bottom = at(xi, zi + 1) + (at(xi + 1, zi + 1) - at(xi, zi + 1)) * fx;
  return top + (bottom - top) * fz;
}

/** The second octave's turn's cosine and sine (`GRASS.gust.turn`; the shader's constants). */
export const GUST_ROTATION = [Math.cos(GRASS.gust.turn), Math.sin(GRASS.gust.turn)] as const;

/**
 * The gust at (x, z) at wind time `t` (s), 0 (calm) .. 1 (a strong gust), as the grass shader reads
 * it: two octaves of `noise`, carried along `WIND.dir` at `GRASS.gust.speed`.
 */
export function gustAt(noise: Uint8Array, t: number, x: number, z: number): number {
  const { span, span2, weight2, speed, calm, strong } = GRASS.gust;
  const px = x - WIND.dir[0] * speed * t;
  const pz = z - WIND.dir[1] * speed * t;
  const [c, s] = GUST_ROTATION;
  const a = sampleGust(noise, px / span, pz / span);
  const b = sampleGust(noise, (px * c - pz * s) / span2, (px * s + pz * c) / span2);
  return smoothstep(calm, strong, (a + b * weight2) / (1 + weight2));
}

// ---- Residents pushing it aside -------------------------------------------------------------

/** How many residents push the grass at once (the shader's `game_grassPush`). */
export const GRASS_PUSHERS = 16;

/** A resident pushing the grass aside (game space), easing in and out. */
export interface GrassPusher {
  x: number;
  z: number;
  /** 0..1. */
  strength: number;
}

/**
 * The residents pushing the grass, frame by frame: of those standing on the ground (`add`), the
 * `GRASS_PUSHERS` nearest the eye push it, each easing in over a fifth of a second as it starts
 * (and out as it stops: it walked indoors, away or out of the nearest), so no blade snaps.
 */
export class GrassPush {
  private readonly strength = new Map<number, GrassPusher>();
  /** This frame's residents (`count` of them; the entries are kept and reused, frame after frame). */
  private readonly near: { id: number; x: number; z: number; d: number }[] = [];
  private count = 0;
  /** Those of them pushing this frame, sorted by `d`. */
  private readonly active: { id: number; x: number; z: number; d: number }[] = [];
  private readonly kept = new Set<number>();
  private readonly out: GrassPusher[] = [];

  /** Starts a frame's residents. */
  begin(): void {
    this.count = 0;
  }

  /** Resident `id` stands on the ground at (x, z), `d` m from the eye. */
  add(id: number, x: number, z: number, d: number): void {
    const r = this.near[this.count] ?? (this.near[this.count] = { id, x, z, d });
    r.id = id;
    r.x = x;
    r.z = z;
    r.d = d;
    this.count++;
  }

  /** The pushers after `dt` s (the nearest first), easing as they come and go. */
  end(dt: number): readonly GrassPusher[] {
    const near = this.active;
    near.length = 0;
    for (let k = 0; k < this.count; k++) near.push(this.near[k]);
    near.sort((a, b) => a.d - b.d);
    const step = Math.min(1, dt * 5);
    const kept = this.kept;
    kept.clear();
    for (let k = 0; k < Math.min(near.length, GRASS_PUSHERS); k++) {
      const r = near[k];
      kept.add(r.id);
      let p = this.strength.get(r.id);
      if (!p) this.strength.set(r.id, (p = { x: r.x, z: r.z, strength: 0 }));
      p.x = r.x;
      p.z = r.z;
      p.strength = Math.min(1, p.strength + step);
    }
    for (const [id, p] of this.strength) {
      if (kept.has(id)) continue;
      p.strength -= step;
      if (p.strength <= 0) this.strength.delete(id);
    }
    this.out.length = 0;
    // (Those easing out are still the nearest few, but never more than the shader takes.)
    for (const p of this.strength.values()) this.out.push(p);
    this.out.sort((a, b) => b.strength - a.strength);
    this.out.length = Math.min(this.out.length, GRASS_PUSHERS);
    return this.out;
  }
}
