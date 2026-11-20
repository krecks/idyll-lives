/**
 * Dresses the viewed lot's structure like a family home: walls with exterior siding or brick
 * and interior wallpaper (tinted per room), corner posts, stone plinth, trims, baseboards,
 * windows with frames and shutters, door casings, per-room floors (wood, tile, carpet), fake
 * contact shadows along walls, and pitched roofs (gable or hip) whose eaves rest on the walls.
 * The viewed house's windows are clear, alpha-blended glass, so its rooms show from outside.
 * Neighbouring houses are dressed boxes (`silhouettes`): their own doors and windows (with
 * curtains, glowing at night), porches and garages; `street.ts` lays out the rest.
 *
 * Wall layout comes from the simulation's structure (`WorldStructure.walls`: every wall edge;
 * `openings`: doors and windows placed in them; `diagonals`: walls corner to corner across a
 * tile, with doors and windows too), so build-mode edits arrive as new structures. A diagonal
 * splits its tile into two halves with their own rooms: floors, wall faces, lawn and shrubs follow
 * the half they are on. Where diagonals meet other walls, the corner post is a mitred outline
 * (`slanted.ts`). Houses with diagonal sides get a hip roof following their outline when it is
 * convex; otherwise the usual rectangles cover the split tiles whole (a deeper eave there).
 * Where the storey above leaves an L, a U or a ring of roof and no diagonal wall is in view, it is
 * roofed as one area (hips, ridges and valleys meeting) rather than a rectangle at a time.
 * Every wall vertex carries cutaway data (`geometry.ts`); the vertex shader lowers walls
 * between the camera and the rooms behind them, so the cutaway costs no CPU.
 *
 * Walls have looks chosen in Build mode (`WallEdge.faces` / `form`, `Opening.style`, the same on
 * diagonals; see `setLooks`): each face can be covered (paint, wallpaper, siding, brick, wood,
 * stone, tiles: a finish tinted per covering, else the automatic look above), a wall can be half
 * height (posts follow the tallest wall they join), and doors and windows come in styles (leaf
 * and paint; opening heights, glazing bars and shutters).
 *
 * Everything is generated in a few milliseconds per structure and merged into one mesh per
 * material (about 15 draw calls for a whole house).
 *
 * Engine-neutral: `HousePlanner` writes the geometry (`Geo`) of each mesh with what it is drawn
 * with (`HousePart`: a surface of the material library, a room's clear glass, the contact
 * shadows) and the renderer uploads it (`pc/house.ts`), in this order.
 */

import type { DoorStyleDef, FloorCoveringDef, LightAim, RoofColorDef, RoofStyleDef, WallCoveringDef, WindowStyleDef } from '../../content/content';
import { objectCentre, storeyOfRow, storeyRow, type StoreyLayout, type PorchInfo, type PorchPieceInfo, type WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import { Geo, LOOK, type V3 } from './geometry';
import { tuned } from '../tune';
import type { P2 } from './slanted';
import { WALL_HALF, WALL_HEIGHT } from './walls';

/** A pattern over a surface's texture (the material libraries' `SurfacePattern`). */
export type SurfacePattern = 'stripes' | 'whitewash';

/** What a house mesh is drawn with: a surface (manifest key, vertex-coloured), a room's clear glass (0: outdoors; `GLASS_OUT`: looked out through), or the contact shadows. */
export type HouseSurface = { kind: 'surface'; key: string; pattern?: SurfacePattern } | { kind: 'glass'; room: number } | { kind: 'contactShadow' } | { kind: 'neighbourGlass'; lit: boolean };

/**
 * One mesh of a house: its geometry and name, what it's drawn with, and how. Empty ones are
 * listed too (the renderers skip them), so a door's parts stay together.
 */
export interface HousePart {
  geo: Geo;
  name: string;
  surface: HouseSurface;
  /** It follows the cutaway (its geometry's cutaway data is uploaded; the material has the cutaway). */
  cut: boolean;
  /** It casts the sun's shadows. */
  cast: boolean;
  /** A roof or a gable: drawn only with walls up. */
  roof: boolean;
  /** Only the sun's shadow map sees it (no camera draws it, and it receives no shadows). */
  shadowOnly: boolean;
  /** See-through glass: drawn after the other blended surfaces (`pc/meshes.ts` `drawAfter`). */
  glass: boolean;
}

export { WALL_HALF, WALL_HEIGHT };
export const WALL_STUB = 0.35;
export const T = WALL_HALF; // half wall thickness (sim-core WALL_THICKNESS / 2)
/** How far a post's face against a wall sits inside it (m): its edges on the wall's faces showed as lines. */
export const INSIDE = 0.002;
export const DOOR_HEIGHT = 2.1;
export const PLINTH = 0.32;
// Window opening (matches sim-core `WINDOW_SILL` / `WINDOW_HEAD`); the glazed part is narrower
// than the edge, with wall piers either side.
export const SILL = 0.9;
export const HEAD = 2.1;
export const [SILL0, HEAD0] = [SILL, HEAD];
export const WIN_INSET = 0.12;
/** Height of a half wall (`form` 1). */
export const HALF_WALL_HEIGHT = 1.05;
export const PITCH = Math.tan((34 * Math.PI) / 180);
export const OVERHANG = 0.4;
export const ROOF_THICKNESS = 0.14;

/** Asset keys of the surfaces used; all are swappable through the manifest. */
export const HOUSE_MATERIALS = {
  siding: 'material.wall.siding',
  brick: 'material.wall.brick',
  interior: 'material.wall.interior',
  stone: 'material.wall.stone',
  trim: 'material.trim',
  roof: 'material.roof',
  glass: 'material.window',
  glassInner: 'material.window.inner',
  /** See-through glazing of the viewed house (alpha-blended, fresnel reflections). */
  clearGlass: 'material.window.clear',
  floorWood: 'material.floor.wood',
  floorTile: 'material.floor.tile',
  floorCarpet: 'material.floor.carpet',
  plaster: 'material.wall',
  panelling: 'material.finish.wood',
} as const;

/** The surface of each wall-covering finish. */
export const FINISH_MATERIALS: Record<WallCoveringDef['finish'], string> = {
  plaster: HOUSE_MATERIALS.plaster,
  wallpaper: HOUSE_MATERIALS.interior,
  siding: HOUSE_MATERIALS.siding,
  brick: HOUSE_MATERIALS.brick,
  wood: HOUSE_MATERIALS.panelling,
  stone: HOUSE_MATERIALS.stone,
  tile: HOUSE_MATERIALS.floorTile,
};

/** The surface of each floor-covering finish. */
export const FLOOR_MATERIALS: Record<FloorCoveringDef['finish'], string> = {
  wood: HOUSE_MATERIALS.floorWood,
  whitewash: HOUSE_MATERIALS.floorWood,
  tile: HOUSE_MATERIALS.floorTile,
  carpet: HOUSE_MATERIALS.floorCarpet,
  stone: HOUSE_MATERIALS.stone,
};

/** The finishes drawn with a pattern over their surface (wall and floor finishes alike). */
export const FINISH_PATTERNS: Partial<Record<WallCoveringDef['finish'] | FloorCoveringDef['finish'], SurfacePattern>> = { wallpaper: 'stripes', whitewash: 'whitewash' };

/** The surface (and its pattern) a wall or floor covering of `finish` is drawn with; null for a finish there isn't. */
export function coveringSurface(kind: 'wall' | 'floor', finish: string): { key: string; pattern?: SurfacePattern } | null {
  const key = kind === 'wall' ? FINISH_MATERIALS[finish as WallCoveringDef['finish']] : FLOOR_MATERIALS[finish as FloorCoveringDef['finish']];
  return key ? { key, pattern: FINISH_PATTERNS[finish as WallCoveringDef['finish']] } : null;
}

/**
 * What each textured surface makes of a vertex colour from far off (linear light, a channel each): its texture's
 * mean times its material's colour (`cargo check-glb --built` measures a wall "as drawn" so). A house's far
 * versions are drawn in the plain finish, with no texture: they take their colours through this (`asDrawn`), or a
 * brick house was a cream box from across the town and turned red on the way to it. The gate holds the two
 * together (`built.farNear`: walls, roof colour) and its report gives both colours; a texture set whose mean
 * moves is another row here.
 */
export const SURFACE_TONE: Record<string, V3> = {
  [HOUSE_MATERIALS.siding]: [0.6805, 0.6805, 0.6805],
  [HOUSE_MATERIALS.brick]: [0.1525, 0.0969, 0.0793],
  [HOUSE_MATERIALS.roof]: [0.6002, 0.6002, 0.6002],
  [HOUSE_MATERIALS.stone]: [0.2523, 0.1815, 0.1097],
  [HOUSE_MATERIALS.plaster]: [0.7245, 0.6978, 0.6462],
  [HOUSE_MATERIALS.interior]: [0.7495, 0.7495, 0.7495],
  [HOUSE_MATERIALS.panelling]: [0.8002, 0.8002, 0.8002],
  [HOUSE_MATERIALS.floorTile]: [0.692, 0.6303, 0.5662],
};

export const toLinear = (v: number) => (v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4));
export const toSrgb = (v: number) => (v <= 0.0031308 ? v * 12.92 : 1.055 * Math.pow(v, 1 / 2.4) - 0.055);

/**
 * The colour (sRGB hex) that shows on surface `on` (a manifest key; none: the plain finish) as `hex` shows on
 * surface `key`: `hex` through `key`'s tone and back out of `on`'s (no brighter than white).
 */
export function asDrawn(hex: string, key: string, on?: string): string {
  const v = parseInt(hex.slice(1), 16);
  const [from, to] = [SURFACE_TONE[key] ?? [1, 1, 1], (on && SURFACE_TONE[on]) || [1, 1, 1]];
  const c = [(v >> 16) & 255, (v >> 8) & 255, v & 255].map((x, i) => Math.round(Math.min(1, toSrgb((toLinear(x / 255) * from[i]) / to[i])) * 255));
  return `#${c.map((x) => x.toString(16).padStart(2, '0')).join('')}`;
}

/** How far a covering preview lies off the surface it covers, so it draws over it (metres). */
export const PROUD = 0.004;

/** Build-mode looks from content: wall coverings, door and window styles (indices as in the structure). */
export interface HouseLooks {
  coverings: readonly WallCoveringDef[];
  floors: readonly FloorCoveringDef[];
  doors: readonly DoorStyleDef[];
  windows: readonly WindowStyleDef[];
  roofs?: readonly RoofStyleDef[];
  roofColors?: readonly RoofColorDef[];
}

/** A roof's shape: gable, hip or flat, and the pitch (rise per run) of pitched ones. */
export interface RoofShape {
  shape: RoofStyleDef['shape'];
  pitch: number;
}

/** Pitch of a flat roof (just enough to draw it as a very low hip over diagonal outlines). */
export const FLAT_PITCH = 0.03;

/** A window's opening and dressing (the classic window without styles in content). */
export type WindowSpec = Pick<WindowStyleDef, 'panes' | 'sill' | 'head' | 'inset' | 'shutters'>;
export const CLASSIC_WINDOW: WindowSpec = { panes: 'cross', sill: SILL, head: HEAD, inset: WIN_INSET, shutters: 'house' };
export const PANEL_DOOR: Pick<DoorStyleDef, 'leaf' | 'color'> = { leaf: 'panel' };
/** Natural oak, light and golden, on the oak grain (the oak panelling's), for an oak leaf without a colour of its own. */
export const OAK = '#D4B48A';
/** A door leaf's thickness (m); it stands from 0.01 m to just under the head. */
export const LEAF_T = 0.045;
/** A single door's leaf is as wide as its opening (m): it hangs 2 cm off its jamb and shuts 1 cm short of the other. */
export const FILL = 1 - 2 * T - 0.03;
/**
 * A leaf that would open less than this much of its quarter turn slides into the wall instead
 * (see `doorway`): below 60 degrees a leaf hinged at a jamb reaches past the middle of the opening,
 * across the way residents walk through.
 */
export const SLIDE_BELOW = 2 / 3;
/** How much of a leaf slid into the wall still shows at the jamb (m): its edge. */
export const POCKET_SHOW = 0.02;

/** A wall's look as the structure sends it (absent fields: the default). */
export interface Look {
  faces?: [number, number];
  form?: number;
  style?: number;
  /** Doors and windows the player resized: sill and top, centimetres above the floor. */
  sill?: number;
  head?: number;
}

/**
 * Doors or windows side by side in a wall that make one: a wide window, a double door. `start`:
 * where the run begins along the line, `len` its edges, `index` this edge's place in it.
 */
export interface Run {
  start: number;
  len: number;
  index: number;
}

// Colour schemes (sRGB) chosen per house from its position, so a house always looks the same.
export const SIDING = ['#ECE6D8', '#C9D8C4', '#BCD0E0', '#EADBA8', '#D9C6B2', '#F4F2EC', '#A9BBA8', '#E6C9B8'];
export const BRICK = ['#FFFFFF', '#E8D8CC', '#D6C2B4'];
export const ACCENT = ['#3E5A4A', '#2F4058', '#7A3B34', '#4A4A4A', '#5B6E86', '#6E5A3E'];
/** The front door (and shutters) of about one house in twelve: somebody wanted it seen. */
export const LOUD = ['#E8457A', '#F2B51D', '#3FA7D6', '#7BC950', '#9B5DE5', '#FF7A3D'];
export const ROOF = ['#5C626A', '#6B4A3A', '#8C4E3C', '#4F5A4C', '#3E4148', '#7A6A5A'];
export const TRIM = '#F7F5F0';
/**
 * How the house is toned (`?tune=house.…` on the dev server). `cap`: the top of a wall, which is what a
 * cut wall shows of itself from the play camera: the wall's own section, a mid tone, where the trim's
 * off-white made every room's outline the brightest thing in the picture.
 */
export const HOUSE_LOOK = tuned('house', {
  cap: '#8D877E',
  /**
   * A floor's tone over its room, by vertex (no texture, no read): `floorWall` darker at the foot of a wall
   * (of the light it reflects), fading over `floorReach` m; `floorMid` the tone out in the room; lighter again,
   * up to the covering's own colour, by `floorWear` within `floorWearReach` m of a door, where feet wear it
   * pale; `floorVary` lighter and darker in patches a few metres across; and `floorWarm` warmer the darker it is.
   */
  floorWall: 0.34,
  floorReach: 1.6,
  floorMid: 0.95,
  floorWear: 1,
  floorWearReach: 1.7,
  floorVary: 0.07,
  floorWarm: 0.2,
  /**
   * A wall face's tone, by vertex: `wallFoot` darker at the floor, fading over `wallFootReach` m up it;
   * `wallCorner` darker into a room's corner, over `wallCornerReach` m along it; `wallVary` in patches.
   */
  wallFoot: 0.24,
  wallFootReach: 1.3,
  wallCorner: 0.28,
  wallCornerReach: 0.9,
  wallVary: 0.06,
  /**
   * The cap shingles of a roof's ridge and hips against its slopes (the colour's multiplier, sRGB). A little
   * darker only: they stand proud of the slopes, which shades them by day. A fifth darker (0.8), by night and
   * at dusk, when the slopes either side are one tone, a hip's cap was a hard dark stripe across the roof.
   */
  roofCap: 0.95,
  /** A valley's flashing against the slopes it lies between (the colour's multiplier): the valley's own shade, and metal under the shingles' edges. */
  roofValley: 0.8,
});

/**
 * A tone (a share of the light reflected, linear) as the vertex colour's multiplier (sRGB). `warm`: the
 * darker it is the warmer (`HOUSE_LOOK.floorWarm`), as a floor's shade at a wall is lit by the floor beside it.
 */
export function toneTint(lin: number, warm = 0): V3 {
  const l = Math.min(1, Math.max(0, lin));
  const v = (k: number) => Math.pow(l, k / 2.2);
  return [v(1 - warm), v(1), v(1 + warm * 1.3)];
}

/** The distance from `(px, pz)` to the segment `a`..`b`. */
export function segmentDistance(px: number, pz: number, ax: number, az: number, bx: number, bz: number): number {
  const [dx, dz] = [bx - ax, bz - az];
  const t = Math.min(1, Math.max(0, ((px - ax) * dx + (pz - az) * dz) / (dx * dx + dz * dz || 1)));
  return Math.hypot(px - ax - dx * t, pz - az - dz * t);
}
export const ROOM_WALLS: Record<RoomKind, string[]> = {
  living: ['#EFE3D3', '#E6DECF', '#EAE2D8', '#DCE3E6'],
  bedroom: ['#D8E2D0', '#DCD6E6', '#E6D4C8', '#D2DDE8'],
  bath: ['#DCE8EC', '#E6EEF0', '#E8E4F0'],
  kitchen: ['#F2E6CF', '#EFE8D6', '#E8EDDE'],
};
export const CARPET = ['#B8AE9C', '#9FAAB8', '#B49C8C', '#A8B29A', '#C4B4A8'];
/**
 * `boards`: the boards of a room nobody has laid a floor in, and content's `wood.oak` (base.json `floorCoverings`,
 * the same colour): oak a tone darker than the furniture that stands on it, as a tint over the floorboards' texture
 * (`textures/floor_wood`, pine's own colours, which honey, walnut and whitewashed boards tint their way). Untinted
 * until 2026-11-18: an orange floor was the strongest thing in a room. (`?tune=floor.boards:FFFFFF` on the dev server.)
 */
export const FLOOR = tuned('floor', { boards: '#C5D2EA' });
/** Curtain colours seen through neighbours' windows. */
export const CURTAINS = ['#F2EDE0', '#E8D9B8', '#C9D6E2', '#E2C4BE', '#F4F1EA', '#E6E0C8'];
/** Wall height of a garage (lower than the house). */
export const GARAGE_HEIGHT = 2.5;
/** How high a porch's posts reach, its canopy's eaves (and a porch piece's: `porchPiece`). */
export const PORCH_TOP = 2.34;

export type RoomKind = 'living' | 'bedroom' | 'bath' | 'kitchen';

export const BATH = ['toilet', 'shower', 'bathtub'];
export const KITCHEN = ['fridge', 'sink', 'stove', 'counter', 'diningTable'];
export const BEDROOM = ['bed'];
export const LIVING = ['sofa', 'tv', 'armchair', 'bookshelf', 'computerDesk', 'piano'];
/** The furniture that says what a room is for, by kind (the footsteps' sound reads a room's floor from it too: `audio/surfaces.ts`). */
export const ROOM_FURNITURE: Record<RoomKind, readonly string[]> = { bath: BATH, kitchen: KITCHEN, bedroom: BEDROOM, living: LIVING };

export interface RoomLight {
  x: number;
  y: number;
  z: number;
  /** The room it lights (`WorldStructure.rooms`, 0 outdoors). */
  room: number;
  /** Indoor area in tiles of the room it lights (0 outdoors). */
  area: number;
  /** A lamp the household bought, or the dim fill of a room without one. */
  kind: 'lamp' | 'fill';
  /** Lamps: reach in metres and relative brightness (content `light`). */
  range?: number;
  power?: number;
  /** A lamp with a switch (or a fire, or one that follows a use): its object id, to follow it on and off. */
  lamp?: number;
  /** Lit as bright by day as at night: it follows a use, so it's on when it's needed, not only after dark. */
  always?: boolean;
  /** Where its shade sends the light (content `light.aim`): a unit direction for a lamp that lights one side only (none: all round). */
  dir?: [number, number, number];
  /** It can cast shadows (`lampCasts`), where the preset has lamps that do (`pc/lampShadows.ts`). */
  casts?: boolean;
}

/**
 * The "room" of the pane a room looks out at the garden through (`HouseSurface` glass): it never glows. A pane
 * glows with the room behind it, seen from outside; the pane on the room's side of a window glowed with its own
 * room, a warm sheen over the dark lawn behind it, which came out olive green.
 */
export const GLASS_OUT = -1;

/**
 * Whether a lamp can cast shadows: one that lights the room below it or round it. Not a fire or a light that
 * follows a use (their light comes from inside a thing that would shadow all of it: a fireplace, a stove under
 * its hood), nor a lamp aimed up (its light is on the ceiling, which the game doesn't draw from above). A
 * lamp that can doesn't cast in another's light either: it would have to be left out of its own, and all copies
 * of a model are one draw.
 */
export function lampCasts(light: LampLight): boolean {
  return light.while === undefined && !light.fire && light.aim !== 'up';
}

/** Below the ceiling (m) where an uplighter's light comes off it (`aim: 'ceiling'`), and how much further it reaches from there. */
export const BOUNCE_DROP = 0.3;
export const BOUNCE_REACH = 1.5;
/** A wall light aimed `out`: away from its wall and down (a unit vector's forward and down parts). */
export const OUT_FORWARD = 0.85;
export const OUT_DOWN = -0.53;

/** Per room: indoor area in tiles and the sums of tile centres (x, z) weighted by area. */
export type RoomTiles = ReadonlyMap<number, { n: number; sx: number; sz: number }>;

/**
 * The lights of the viewed house: every lamp, and a dim fill from the ceiling of each room
 * without one. Each room's first light comes before any room's second (the biggest rooms' first,
 * the garden's last), so with few lights to go round every room still gets one. Cheap: run on every change.
 * `inView`: whether a lot tile is the viewed house's (on any storey: `rooms` are all of them).
 */
export function placeLights(world: WorldStructure, inView: (x: number, z: number) => boolean, rooms: RoomTiles, lightOf: (def: string) => LampLight | undefined): RoomLight[] {
  const W = world.width;
  const roomAt = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < world.depth ? world.rooms[z * W + x] : 0);
  const lamps: { x: number; z: number; lift: number; room: number; light: LampLight; id: number; facing: [number, number] }[] = [];
  for (const o of world.objects) {
    const light = inView(o.x, o.z) ? lightOf(o.def) : undefined;
    if (!light) continue;
    // A lamp on a table shines from its own height above the top, and from its shade (`at`, turned with it).
    const { x, z } = objectCentre(o);
    const [ax, az] = light.at ?? [0, 0];
    const yaw = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180;
    const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
    // (Model x and z turned as the object is drawn: x -> (c, -s), z -> (s, c).)
    lamps.push({ x: x + ax * c + az * s, z: z - ax * s + az * c, lift: o.lift ?? 0, room: roomAt(o.x, o.z), light, id: o.id, facing: [s, c] });
  }
  const areaOf = (r: number) => rooms.get(r)?.n ?? 0;
  /** Where a lamp's shade sends its light: down, up, out from its wall and down, or all round (an uplighter's from under the ceiling it lights). */
  const aimed = (l: (typeof lamps)[number]): Pick<RoomLight, 'y' | 'dir' | 'range'> => {
    const y = l.lift + (l.light.height ?? 1.55);
    switch (l.light.aim) {
      case 'down':
        return { y, dir: [0, -1, 0], range: l.light.range };
      case 'up':
        return { y, dir: [0, 1, 0], range: l.light.range };
      case 'out':
        return { y, dir: [l.facing[0] * OUT_FORWARD, OUT_DOWN, l.facing[1] * OUT_FORWARD], range: l.light.range };
      case 'ceiling':
        return { y: Math.max(y, WALL_HEIGHT - BOUNCE_DROP), range: (l.light.range ?? 4.5) + BOUNCE_REACH };
      default:
        return { y, range: l.light.range };
    }
  };
  const lights = [
    ...lamps.map((l): RoomLight => ({ x: l.x, z: l.z, ...aimed(l), room: l.room, area: areaOf(l.room), kind: 'lamp', power: l.light.intensity, ...(l.light.switch === false && l.light.while === undefined ? {} : { lamp: l.id }), ...(l.light.while === undefined ? {} : { always: true }), ...(lampCasts(l.light) ? { casts: true } : {}) })),
    // (A light that follows a use isn't the room's lamp: a room whose only light is a hood still gets its dim fill.)
    ...[...rooms.entries()].filter(([r]) => !lamps.some((l) => l.room === r && l.light.while === undefined)).map(([r, t]): RoomLight => ({ x: t.sx / t.n, y: 2.45, z: t.sz / t.n, room: r, area: t.n, kind: 'fill' })),
  ];
  // How many lights of its room come before each.
  const nth = new Map<RoomLight, number>();
  const counts = new Map<number, number>();
  for (const l of lights) {
    const k = counts.get(l.room) ?? 0;
    nth.set(l, k);
    counts.set(l.room, k + 1);
  }
  return lights.sort((a, b) => nth.get(a)! - nth.get(b)! || b.area - a.area);
}

/** What a lamp object gives (content `light`). */
export interface LampLight {
  range?: number;
  intensity?: number;
  height?: number;
  /** Where the light is off the object's middle (m, x and z in its own frame, turned with it): an arc lamp's shade. */
  at?: [number, number];
  /** False: no switch, lit every night. */
  switch?: boolean;
  /** A fire: lit and put out as a lamp is switched (sim-core `lights.rs`), its flames burning while it's lit. */
  fire?: boolean;
  /** Lit only while someone does one of these animations at an object on its tiles (sim-core `lights.rs`). */
  while?: string | string[];
  /** Where its shade sends the light (`ObjectDef.light.aim`). */
  aim?: LightAim;
}

/** How a storey is built (`HousePlanner.build`); the ground storey needs none of it. */
export interface StoreyBuild {
  /** Above the ground: no stone base, no garden. */
  upper?: boolean;
  /** The basement (with `upper`: no garden): outside its walls is earth, faced in stone all the way up. */
  basement?: boolean;
  /** Tiles with a room upstairs (no roof over them here). */
  covered?: (x: number, z: number) => boolean;
  /** Stairwell tiles (`z * width + x`): open to the storey below, so no floor. */
  holes?: ReadonlySet<number>;
  /** Landings at the top of the stairs (same keys): the stairwell's railing leaves them open. */
  landings?: ReadonlySet<number>;
  /** Build mode's preview: walls, doors and windows only, uncut (they stand while walls are down). */
  preview?: boolean;
  /** A preview's walls, openings and diagonal walls already standing (see `build`): not drawn, but looked up like its own. */
  standing?: Pick<WorldStructure, 'walls' | 'openings' | 'diagonals'>;
}

export interface HousePlan {
  /** Its meshes, in the order they're drawn (`HousePart` says which cast, which are roofs). */
  parts: HousePart[];
  /** Indoor rooms of the viewed lot: tiles and their centre sums (for `placeLights`). */
  rooms: RoomTiles;
  /** Garden dressing: shrubs along the foundation and trees on open lawn (x, z, scale, yaw). */
  shrubs: [number, number, number, number][];
  trees: [number, number, number, number][];
  surfaces: HouseSurfaces;
  /** Door leaves, every one of them: shut at rest, they swing (or slide) open for someone going through. */
  doors: DoorPlan[];
}

/**
 * A door leaf, as a renderer has it (`M`: its meshes): drawn open, its meshes turn about the hinge
 * (their pivot) by up to `close` radians to shut it. One that slides is turned shut and moves by up
 * to `slide`. Positions are in the storey's metres.
 */
export interface DoorLeaf<M> {
  meshes: M[];
  /** The middle of its doorway, in a wall along x (`axis` h), along z (v) or along a diagonal (its unit direction, x and z). */
  centre: [number, number];
  axis: 'h' | 'v' | P2;
  /** Turn about y that shuts it, and how much of it back it swings open (1: all the way, against the wall). */
  close: number;
  opens: number;
  /** A front door (outdoors on one side). */
  front: boolean;
  /** A leaf that slides into the wall instead (it could swing only a little way open): how far, open. */
  slide?: [number, number];
}

/** A door leaf that swings as planned: its parts (trim, wood, glass; empty ones too) and the point they turn about (`pivot`, x and z). */
export type DoorPlan = Omit<DoorLeaf<never>, 'meshes'> & { parts: HousePart[]; pivot: [number, number] };

/** What a doorway needs to know of its leaf, before its meshes exist: `room`, the one whose light its glass is in. */
export type SwingSpec = Omit<DoorLeaf<never>, 'meshes'> & { hinge: [number, number]; room: number };

/**
 * Where coverings lie in the viewed house, for Build mode's previews: each wall face
 * (`axis:x:z:side`, where a covering goes: openings left out, above the plinth outdoors) and each
 * floor tile (`x:z`), a hair proud of the real surface and with its cutaway data.
 */
export interface HouseSurfaces {
  faces: Map<string, Geo>;
  floors: Map<string, Geo>;
}

export type EdgeKind = 'wall' | 'door' | 'window';

/** A neighbouring house drawn as a dressed box (see `HousePlanner.silhouettes`). */
export interface SilhouetteSpec {
  /** Wall bounding box (grid lines). */
  x0: number;
  z0: number;
  x1: number;
  z1: number;
  /** Origin of the house's lot: its look (siding, roof, accent) is chosen from it, as for the game's house (every storey of it). */
  lot: { x: number; z: number };
  /** The player's house in the menus' town. */
  home?: boolean;
  /**
   * Only what the game doesn't draw of it (the viewed lot's): its garage and its porch, the porch
   * going with its wall in the cutaway (`porchDoors`); the game draws the house itself.
   */
  dressingOnly?: boolean;
  /** The street side: the outward normal (x, z) of the wall that faces the street ([0, 1]: the wall at z1, [-1, 0]: the wall at x0...). */
  front: [number, number];
  /** Centre of the front door along the front wall (x, or z on a wall along z), where the garden path meets the house, if known (the flower beds and pots keep clear of it). */
  door: number | null;
  /** The porches over its doors (the simulation's, `WorldStructure.porches`; `porchDoors`). */
  porches?: readonly PorchInfo[];
  /** The porch pieces on its lot (`WorldStructure.porchPieces`; `HouseBuilder.porchPiece`): in its colours. */
  pieces?: readonly PorchPieceInfo[];
  /**
   * The viewed lot's (`dressingOnly`): whether a wall (or a door or a window in one) stands on a
   * ground-storey edge. A canopy piece whose back is against one goes with that wall in the
   * cutaway, its lamp too, as the porch's canopy does (`canopyBack`).
   */
  walled?: (axis: 'h' | 'v', x: number, z: number) => boolean;
  /** Garage beside the house (wall centre lines), its door facing the street, or `door` (its outward normal: one the player built). */
  garage: { x0: number; z0: number; x1: number; z1: number; door?: [number, number] } | null;
  /** The roof the player chose (`PlotInfo.roof`); absent: the house's own look. */
  roof?: [number, number];
  /** The wall covering most of its outside wears (`HousePlanner.exteriorCover`: 0 or absent, its own look). */
  cover?: number;
  /** Storeys the house has (absent: one). */
  storeys?: number;
  /** Someone's home: its window glass is kept apart, lit only while someone is up (see `SilhouettePlan.homes`). */
  lived?: boolean;
}

export interface SilhouettePlan {
  /** The street's houses' meshes (one per material for the whole street; `cast`, `cut` as for a house). */
  parts: HousePart[];
  /** The window glass of each lived-in house (not in `parts`): drawn lit or dark (`neighbourPanes`). */
  homes: { spec: SilhouetteSpec; panes: Geo }[];
  /**
   * Porch and garage lamps: x, y, z and the wall's outward normal (nx, nz: 0, 0 for a canopy's
   * lamp, hanging under its ridge, its pool of light under it); a lamp on the viewed lot's porch (or
   * a canopy piece's against a wall) also the cutaway's look bits (`LOOK`) that drop its wall, to
   * hide it with it.
   */
  lamps: [number, number, number, number, number, number?][];
}

/** A door a porch stands over (`porchDoors`): its wall's axis, the wall's outer face, its outward normal and the door's middle along it. */
export interface PorchDoor {
  axis: 'h' | 'v';
  plane: number;
  n: V3;
  mid: number;
}

/**
 * Where a house's porches stand: over the doors the simulation has them over (`SilhouetteSpec.porches`:
 * the town's over its front doors, and those the player built), on the side each stands on. On the
 * viewed lot's house (`dressingOnly`) every one; on a silhouette only those over a door in the
 * outline of its box (one set back in the house would stand inside the box).
 */
export function porchDoors(b: SilhouetteSpec): PorchDoor[] {
  const out: PorchDoor[] = [];
  for (const p of b.porches ?? []) {
    const s = p.out > 0 ? 1 : -1;
    const [at, a, a0, a1, line] = p.axis === 'h' ? [p.z, p.x, b.x0, b.x1, s > 0 ? b.z1 : b.z0] : [p.x, p.z, b.z0, b.z1, s > 0 ? b.x1 : b.x0];
    if (!b.dressingOnly && (at !== line || a < a0 || a >= a1)) continue;
    out.push({ axis: p.axis, plane: at + s * T, n: p.axis === 'h' ? [0, 0, s] : [s, 0, 0], mid: a + 0.5 });
  }
  return out;
}

/** The way a porch piece faces (`rot`: 0 towards +z, 1 +x, 2 -z, 3 -x) as x and z. */
export function pieceFacing(rot: number): [number, number] {
  return ([[0, 1], [1, 0], [0, -1], [-1, 0]] as const)[((rot % 4) + 4) % 4] as [number, number];
}

/**
 * The grid edges along a canopy piece's back (the side away from the way it faces), as
 * `['h' | 'v', x, z]`: a wall on any of them and it stands against the house (`SilhouetteSpec.walled`).
 */
export function canopyBack(p: Pick<PorchPieceInfo, 'x' | 'z' | 'w' | 'd' | 'rot'>): ['h' | 'v', number, number][] {
  const [fx, fz] = pieceFacing(p.rot);
  const out: ['h' | 'v', number, number][] = [];
  if (fz) for (let x = p.x; x < p.x + p.w; x++) out.push(['h', x, fz > 0 ? p.z : p.z + p.d]);
  else for (let z = p.z; z < p.z + p.d; z++) out.push(['v', fx > 0 ? p.x : p.x + p.w, z]);
  return out;
}

/** The cutaway's look bits (`LOOK`) that drop a wall facing `n` outwards: the camera looking at it from outside. */
export function lookAt(n: V3): number {
  return n[0] > 0 ? LOOK.NegX : n[0] < 0 ? LOOK.PosX : n[2] > 0 ? LOOK.NegZ : LOOK.PosZ;
}

/** Wall edges of the lot: `h` edges run along x at a grid line z, `v` edges along z. */
export class Edges {
  private readonly h = new Map<number, EdgeKind>();
  private readonly v = new Map<number, EdgeKind>();
  constructor(private readonly stride: number) {}
  private key(x: number, z: number): number {
    return (z + 2) * this.stride + x + 2;
  }
  set(axis: 'h' | 'v', x: number, z: number, kind: EdgeKind): void {
    (axis === 'h' ? this.h : this.v).set(this.key(x, z), kind);
  }
  get(axis: 'h' | 'v', x: number, z: number): EdgeKind | undefined {
    return (axis === 'h' ? this.h : this.v).get(this.key(x, z));
  }
  *all(axis: 'h' | 'v'): Generator<[number, number, EdgeKind]> {
    for (const [k, kind] of axis === 'h' ? this.h : this.v) {
      yield [(k % this.stride) - 2, Math.floor(k / this.stride) - 2, kind];
    }
  }
}

export function hash(x: number, z: number, salt: number): number {
  let h = (Math.imul(x | 0, 374761393) + Math.imul(z | 0, 668265263) + Math.imul(salt, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

export function pick<T>(list: readonly T[], x: number, z: number, salt: number): T {
  return list[Math.floor(hash(x, z, salt) * list.length) % list.length];
}

/** Furniture that decides what a room is for (and so its automatic floor and wall look). */
export const ROOM_KIND_DEFS = new Set([...BATH, ...KITCHEN, ...BEDROOM, ...LIVING]);

/**
 * The objects `HousePlanner.build` depends on, as a key: room-kind furniture, and
 * whatever stands on the lot's tree spots; on a lot that isn't the player's home (it gets a
 * garden border around its furniture) every object. Other objects can change without
 * rebuilding the house. Stairs (`stairs`) on any storey of the lot in view count too: their
 * stairwells open the floor above.
 */
export function houseObjectKey(world: WorldStructure, view: ViewRect | null, stairs: (def: string) => boolean = () => false): string {
  const D = world.storeyDepth ?? world.depth;
  const flights = world.objects.filter((o) => stairs(o.def) && (!view || (o.x >= view.x && o.z % D >= view.z && o.x < view.x + view.w && o.z % D < view.z + view.d)));
  return `${houseObjectKeyOf(world, view)}|${flights.map((o) => `${o.x},${o.z},${o.rot}`).join(';')}`;
}

export function houseObjectKeyOf(world: WorldStructure, view: ViewRect | null): string {
  const inView = (x: number, z: number) => !view || (x >= view.x && z >= view.z && x < view.x + view.w && z < view.z + view.d);
  const homeId = world.households?.find((h) => h.player)?.plot;
  const home = homeId == null ? undefined : world.plots?.[homeId];
  const isHome = !!view && !!home && home.x === view.x && home.z === view.z;
  const spots = view
    ? [
        [view.x + 1, view.z + 1],
        [view.x + view.w - 2, view.z + 1],
        [view.x + 1, view.z + view.d - 2],
        [view.x + view.w - 2, view.z + view.d - 2],
      ]
    : [];
  const matters = (o: WorldStructure['objects'][number]) =>
    !isHome || ROOM_KIND_DEFS.has(o.def) || spots.some(([sx, sz]) => sx >= o.x && sx < o.x + o.w && sz >= o.z && sz < o.z + o.d);
  return world.objects
    .filter((o) => inView(o.x, o.z) && matters(o))
    .map((o) => `${o.def}@${o.x},${o.z},${o.w},${o.d}`)
    .join(';');
}

/** Look of one house, chosen deterministically from its position. */
export interface HouseScheme {
  brick: boolean;
  wall: string;
  accent: string;
  roof: string;
  hip: boolean;
  shutters: boolean;
  seedX: number;
  seedZ: number;
}

/** The origin of the lot a storey's view (`HousePlanner.build`'s, on that storey's rows) is of: on the ground's rows. */
export function lotOrigin(world: StoreyLayout, view: { x: number; z: number }): { x: number; z: number } {
  return { x: view.x, z: view.z - storeyRow(world, storeyOfRow(world, view.z)) };
}

export function houseScheme(x: number, z: number): HouseScheme {
  const brick = hash(x, z, 1) < 0.3;
  return {
    brick,
    wall: brick ? pick(BRICK, x, z, 2) : pick(SIDING, x, z, 2),
    accent: hash(x, z, 7) < 1 / 12 ? pick(LOUD, x, z, 8) : pick(ACCENT, x, z, 3),
    roof: pick(ROOF, x, z, 4),
    hip: hash(x, z, 5) < 0.45,
    shutters: hash(x, z, 6) < 0.6,
    seedX: x,
    seedZ: z,
  };
}

/** A house as its far versions are asked for (`HousePlanner.farHouse`, `farLook`): its lot's place (its look's), its box of walls, the roof the player chose, its storeys, and the covering most of its outside wears (`exteriorCover`). */
export interface FarPlot {
  x: number;
  z: number;
  house: readonly [number, number, number, number];
  roof?: [number, number];
  storeys?: number;
  cover?: number;
}

/** A Build mode preview laid in a covering's look (`HousePlanner.coverPreview`, `floorPreview`): one part, a hair proud of what it covers. */
export type LaidPreview = HousePart & { surface: { kind: 'surface'; key: string; pattern?: SurfacePattern } };

/**
 * The house's (and the street's houses') geometry, engine-neutral: what the renderer's house
 * builder (`pc/house.ts`) uploads and draws.
 */
