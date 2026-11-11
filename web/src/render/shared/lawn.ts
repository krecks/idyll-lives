/**
 * The lawn's map: what the grass is like tile by tile over the town's ground, so that a lawn is not one
 * green from fence to fence. One RGBA texel a tile (a metre), which the blades and the ground under them
 * both read, smoothly between tiles (`pc/shaders/grass.ts`, `game_grassLawn`):
 *
 * - red, the tone: 128 as the land's colour has it, lighter above, darker below. Patches a few metres
 *   across (`patch`), a mown lot's stripes (`stripes`), and the damp, darker grass at the foot of walls,
 *   fences and hedges and where their shade lies most of the day (the sun stands in the south: the low z
 *   side, so shade falls towards high z).
 * - green, how dry: 128 neither; above, paler and yellower (the open middle of a lawn in summer); below,
 *   lusher and deeper (by a wall, under a tree).
 * - blue, bare earth: where feet wear the grass away (outside a door, at a gate, at a path's ends, under a
 *   swing, by a bench) and the soil round beds and trunks. The blades thin out over it and the ground
 *   takes the earth's colour.
 * - alpha, how rough: 0 on a plot, mown short and even; 255 on the land between and beyond the plots,
 *   where the grass grows long and in clumps; part of the way along a fence's foot, where no mower reaches.
 *
 * Drawn only: made from the world as it stands (plots, walls, doors, fences, paths, things), the same
 * town always the same lawn. The patches and stripes are made once a town (`base`); what stands on the
 * lawn is laid over them again whenever the lot changes (`lawnMap`).
 */

import { groundDepth, type WorldStructure } from '../../core/protocol';
import { hash, valueNoise } from '../../game/noise';
import { tuned } from '../tune';

/**
 * What goes into the map, each as a share of its channel's whole range (tone and dryness: of the way from
 * neither to full either way).
 *
 * `patch`, `patchTone`: the tone patches' two sizes (m across) and strengths; `drySpan`, `dry`: the dry
 * patches' size and strength. `roughPatch`: how much stronger both are on rough land. `stripes`: a mown
 * lot's stripes, `stripeWidth` tiles wide each. `wall`, `shade`, `fence`: the darker foot of a wall, more
 * on its shaded side (and half of it a tile further out), and of a fence or hedge; `lush`: how much lusher
 * the grass is there, as a share of how much darker. `fenceRough`: how rough the grass at a fence's foot.
 * `door`, `gate`, `path`: bare earth outside a door, at a gate and past a path's ends; `play`, `seat`,
 * `bed`, `trunk`: under play things, seats, beds and trunks (`THINGS`), and a share of it a tile round
 * them (`RING`, times `ring`). `crown`: the darker grass under a tree.
 */
export const LAWN_MAP = tuned('ground', {
  patch: [10, 3.7],
  patchTone: [0.6, 0.3],
  drySpan: 13,
  dry: 0.75,
  roughPatch: 1.3,
  stripes: 0,
  stripeWidth: 1,
  wall: 0,
  shade: 0,
  fence: 0,
  lush: 0.8,
  fenceRough: 0.6,
  door: 0,
  gate: 0,
  path: 0,
  play: 0,
  seat: 0,
  bed: 0,
  trunk: 0,
  ring: 1,
  crown: 0,
});

/** Things that wear or cover the lawn, by their kind's id: bare earth under them (a share of `LAWN_MAP`'s), and whether a tree. */
const THINGS: readonly [RegExp, 'play' | 'seat' | 'bed' | 'trunk'][] = [
  [/^(swingSet|seesaw|springRider|merryGoRound|slide|climbingFrame|basketballHoop|sandbox)$/, 'play'],
  [/^(parkBench|bench|benchGarden\d*|picnicTable)$/, 'seat'],
  [/^(raisedBed|planterBox|flowerbed|compostBin|garden\.(vegPatch|tomatoes|pumpkins|strawberries|roses|lavender|geraniums|hydrangea|boxwood)|natureLover\.veggieBed|bush)$/, 'bed'],
  [/^(tree|pine|garden\.(maple|cherry|birch|willow|spruce|japaneseMaple|palm|apple|lemon|oak)|appleTree|lemonTree)$/, 'trunk'],
];

/** How much of the earth under a thing lies a tile round it: feet wear the grass round a swing and before a bench, a bed has a narrow edge of soil. */
const RING = { play: 0.6, seat: 0.55, bed: 0.3, trunk: 0.45 } as const;

const [TONE, DRY, EARTH, ROUGH] = [0, 1, 2, 3];

/** What `base` was made for, per map (the town's plots and seed), and the map before anything stood on it. */
const bases = new WeakMap<Uint8Array, { key: string; base: Uint8Array }>();

/** The patches, the stripes and where the plots are: the map with nothing standing on it. */
function baseMap(world: WorldStructure, tw: number, td: number): Uint8Array {
  const M = LAWN_MAP;
  const seed = world.meta?.seed ?? 1;
  const base = new Uint8Array(tw * td * 4);
  // Which plot a tile is on (0: none), and each plot's stripes: along x (1), along z (2) or none.
  const plot = new Uint16Array(tw * td);
  const stripes = [0];
  world.plots.forEach((p, k) => {
    const h = hash(p.x, p.z, seed + 411);
    stripes.push(h < 0.42 ? 1 : h < 0.84 ? 2 : 0);
    for (let z = Math.max(0, p.z); z < Math.min(td, p.z + p.d); z++) plot.fill(k + 1, z * tw + Math.max(0, p.x), z * tw + Math.min(tw, p.x + p.w));
  });
  const [a, b] = M.patch;
  const stripeWidth = Math.max(1, Math.round(M.stripeWidth));
  for (let z = 0; z < td; z++) {
    for (let x = 0; x < tw; x++) {
      const o = (z * tw + x) * 4;
      // (The outermost tiles stay as the land beyond the town is: the texture's edge is read on past it.)
      const edge = x === 0 || z === 0 || x === tw - 1 || z === td - 1;
      const on = edge ? 0 : plot[z * tw + x];
      const more = on ? 1 : M.roughPatch;
      // (Each noise turned its own way off the tiles' rows: a patch that lined up with the mown stripes read as a wider stripe.)
      const [px, pz] = [x + 0.5, z + 0.5];
      const [ax, az] = [px * 0.848 - pz * 0.53, px * 0.53 + pz * 0.848];
      const [bx, bz] = [px * 0.469 + pz * 0.883, -px * 0.883 + pz * 0.469];
      let tone = ((valueNoise(ax / a, az / a, seed + 401) - 0.5) * 2 * M.patchTone[0] + (valueNoise(bx / b + 7.3, bz / b - 2.1, seed + 402) - 0.5) * 2 * M.patchTone[1]) * more;
      const dry = (valueNoise(bx / M.drySpan - 3.7, az / M.drySpan + 9.2, seed + 403) - 0.5) * 2 * M.dry * more;
      const way = stripes[on];
      if (way) tone += (Math.floor((way === 1 ? z : x) / stripeWidth) % 2 ? 1 : -1) * M.stripes;
      base[o + TONE] = edge ? 128 : signed(tone);
      base[o + DRY] = edge ? 128 : signed(dry);
      base[o + ROUGH] = on ? 0 : 255;
    }
  }
  return base;
}

/** -1..1 as a byte about 128. */
function signed(v: number): number {
  return Math.max(0, Math.min(255, Math.round(128 + v * 127)));
}

/**
 * The lawn's map of `world` now (`tw` × `td` tiles of 4 bytes, rows along x), into `into` when it is that
 * size (the same town again: its patches and stripes are kept). `bare`: the tiles no lawn grows on
 * (`bareTiles`).
 */
export function lawnMap(world: WorldStructure, bare: Uint8Array, into?: Uint8Array): Uint8Array {
  const M = LAWN_MAP;
  const [tw, td] = [world.width, groundDepth(world)];
  const out = into && into.length === tw * td * 4 ? into : new Uint8Array(tw * td * 4);
  const key = `${world.meta?.seed ?? 1}:${tw}x${td}:${world.plots.map((p) => `${p.x},${p.z},${p.w},${p.d}`).join(';')}`;
  let made = bases.get(out);
  if (!made || made.key !== key) bases.set(out, (made = { key, base: baseMap(world, tw, td) }));
  out.set(made.base);
  // (Never the outermost tiles: they stay as the land beyond the town.)
  const inside = (x: number, z: number) => x > 0 && z > 0 && x < tw - 1 && z < td - 1;
  const add = (x: number, z: number, channel: number, amount: number): void => {
    if (!inside(x, z)) return;
    const o = (z * tw + x) * 4 + channel;
    out[o] = Math.max(0, Math.min(255, out[o] + Math.round(amount * (channel === TONE || channel === DRY ? 127 : 255))));
  };
  /** Damp, darker and lusher grass on a tile. */
  const damp = (x: number, z: number, amount: number): void => {
    add(x, z, TONE, -amount);
    add(x, z, DRY, -amount * M.lush);
  };
  // The foot of walls, fences and hedges: darker on both sides, more where the shade lies (towards high z).
  const foot = (e: { axis: 'h' | 'v'; x: number; z: number }, amount: number, rough: number): void => {
    if (e.axis === 'h') {
      if (e.z > td) return;
      damp(e.x, e.z - 1, amount * 0.6);
      damp(e.x, e.z, amount + M.shade);
      damp(e.x, e.z + 1, M.shade * 0.5);
      if (rough) for (const z of [e.z - 1, e.z]) add(e.x, z, ROUGH, rough);
    } else {
      if (e.z >= td) return;
      for (const x of [e.x - 1, e.x]) {
        damp(x, e.z, amount * 0.8 + M.shade * 0.3);
        if (rough) add(x, e.z, ROUGH, rough);
      }
    }
  };
  for (const e of world.walls) foot(e, M.wall, 0);
  for (const e of world.fences ?? []) {
    if (e.kind === 'gate') {
      // Feet through a gate.
      const [dx, dz] = e.axis === 'h' ? [0, 1] : [1, 0];
      if (e.z < td) for (const s of [-1, 0]) add(e.x + dx * s, e.z + dz * s, EARTH, M.gate);
    } else foot(e, M.fence, M.fenceRough);
  }
  // Outside a door: the tile it opens onto, and a little of the next one out.
  const rooms = world.rooms;
  const outdoors = (x: number, z: number) => x >= 0 && z >= 0 && x < tw && z < td && rooms[z * world.width + x] === 0;
  for (const o of world.openings) {
    if (o.kind !== 'door' || o.z > td || (o.axis === 'v' && o.z >= td)) continue;
    const [dx, dz] = o.axis === 'h' ? [0, 1] : [1, 0];
    const [inX, inZ] = [o.x - dx, o.z - dz];
    for (const [x, z, s] of [[o.x, o.z, 1], [inX, inZ, -1]] as const) {
      if (!outdoors(x, z) || outdoors(s > 0 ? inX : o.x, s > 0 ? inZ : o.z)) continue;
      add(x, z, EARTH, M.door);
      add(x + dx * s, z + dz * s, EARTH, M.door * 0.35);
    }
  }
  // Past a path's two ends, where feet leave it for the grass.
  const grows = (x: number, z: number) => x >= 0 && z >= 0 && x < tw && z < td && bare[z * tw + x] === 0;
  for (const p of world.meta?.paths ?? []) {
    const along = p.d >= p.w;
    const [c, lo, hi] = along ? [Math.floor(p.x + p.w / 2), Math.floor(p.z) - 1, Math.ceil(p.z + p.d)] : [Math.floor(p.z + p.d / 2), Math.floor(p.x) - 1, Math.ceil(p.x + p.w)];
    for (const e of [lo, hi]) {
      const [x, z] = along ? [c, e] : [e, c];
      if (grows(x, z)) add(x, z, EARTH, M.path);
    }
  }
  // Under and round the things that wear the lawn or stand in soil, and the darker grass under a tree.
  for (const o of world.objects) {
    if (o.z >= td || !outdoors(o.x, o.z)) continue;
    const kind = THINGS.find(([re]) => re.test(o.def))?.[1];
    if (!kind) continue;
    const amount = M[kind];
    for (let z = o.z - 1; z <= o.z + o.d; z++) {
      for (let x = o.x - 1; x <= o.x + o.w; x++) {
        const under = x >= o.x && z >= o.z && x < o.x + o.w && z < o.z + o.d;
        add(x, z, EARTH, under ? amount : amount * RING[kind] * M.ring);
      }
    }
    if (kind === 'trunk') {
      const [cx, cz] = [o.x + (o.w - 1) / 2, o.z + (o.d - 1) / 2];
      for (let z = Math.floor(cz) - 2; z <= Math.ceil(cz) + 2; z++) {
        for (let x = Math.floor(cx) - 2; x <= Math.ceil(cx) + 2; x++) {
          const r = Math.hypot(x - cx, z - cz);
          if (r < 2.6) damp(x, z, M.crown * (1 - r / 2.6));
        }
      }
    }
  }
  return out;
}
