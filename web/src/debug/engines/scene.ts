/**
 * The engine comparison's scene (`engines.html`, see `bench.ts`): one description that the
 * Babylon.js and PlayCanvas drivers build the same way. Rooms of the game's own furniture (glTF
 * from `public/assets/models`, loaded as they are, without the game's finishes or soft look),
 * laid out on a grid, lit by a sun and a flat ambient, with an orbiting camera that sees all of it.
 * Right-handed and Y up in both engines, metres.
 */

/** The furniture of one room, from `public/assets/models/` (about 220 mesh parts and 95k triangles). */
export const MODELS = [
  'living/sofa',
  'living/armchair',
  'living/coffeeTable',
  'living/bookcaseTall',
  'bedroom/bedDouble',
  'bedroom/bedSingle',
  'dining/tableDining4',
  'dining/chairWood',
  'dining/sideboard',
  'kitchen/fridge',
  'kitchen/counterSinkDouble',
  'bathroom/toilet',
  'bathroom/bathtub',
  'bathroom/sink',
  'lighting/tableLamp',
  'office/officeChair',
  'office/deskSimple',
  'bedroom_storage/dresser',
  'bedroom_storage/wardrobe',
  'laundry/laundryWasher',
].map((path) => `${import.meta.env.BASE_URL}assets/models/${path}.glb`);

/** A garden round the house (`lot` cases), and the trees round it, drawn as instances (as the game draws nature). */
export const GARDEN = ['garden/birdbath', 'garden/boulder', 'garden/fountain', 'garden/gnome', 'garden/lantern', 'garden/wheelbarrow'].map(
  (path) => `${import.meta.env.BASE_URL}assets/models/${path}.glb`,
);
export const TREES = ['trees/apple', 'trees/birch', 'trees/cherry', 'trees/cypress'].map((path) => `${import.meta.env.BASE_URL}assets/models/${path}.glb`);

/** After the scene's lighting: none, the game's chain (MSAA into an HDR target, tone mapping, vignette, colour grading), or that with bloom. */
export type Post = 'none' | 'game' | 'bloom';

/** What a case draws on top of the furniture. */
export interface CaseSpec {
  /** Each copy of a model as hardware instances of one mesh (Babylon's thin instances), not its own meshes. */
  instanced: boolean;
  /** Spot lamps hanging over the rooms, clustered in both engines. */
  lamps: number;
  /** The sun casts shadows (one 2048² map, 3×3 PCF). */
  shadows: boolean;
  /** Rooms of furniture, instead of the `rooms` flag. */
  rooms?: number;
  /** Instanced trees round the house, and the garden's models. */
  trees?: number;
  garden?: boolean;
  /** Walking residents (`residents.ts`), each skinned on the GPU by the engine. */
  residents?: number;
  post?: Post;
  /** The camera's distance from the middle (m), instead of one fitted to the rooms. */
  orbit?: number;
}

/** The game's viewed lot, about its size on Medium (see `bench-results`: ~290 draws, ~180 shadow draws): one house of furniture, a garden, a household walking, a sun with shadows, lamps, the game's post-processing, at the game's camera distance. */
const LOT: CaseSpec = { instanced: false, lamps: 8, shadows: true, rooms: 1, trees: 300, garden: true, residents: 8, post: 'game', orbit: 24 };

export const CASES: Record<string, CaseSpec> = {
  furniture: { instanced: false, lamps: 0, shadows: false },
  instanced: { instanced: true, lamps: 0, shadows: false },
  lamps: { instanced: false, lamps: 32, shadows: false },
  shadows: { instanced: false, lamps: 0, shadows: true },
  game: { instanced: false, lamps: 32, shadows: true },
  lot: LOT,
  'lot-nopost': { ...LOT, post: 'none' },
  'lot-bloom': { ...LOT, post: 'bloom' },
  'lot-crowd': { ...LOT, residents: 32 },
};

/** A room's side (m), and how far apart its models stand (5 by 4). */
const ROOM = 8;
const SPACING = 1.6;
const COLUMNS = 5;

export interface Placement {
  model: number;
  x: number;
  z: number;
  /** Turn round Y, radians. */
  rotY: number;
}

/** The rooms in a square grid, centred on the origin. */
function roomOrigin(room: number, rooms: number): [number, number] {
  const side = Math.ceil(Math.sqrt(rooms));
  const offset = ((side - 1) * ROOM) / 2;
  return [(room % side) * ROOM - offset, Math.floor(room / side) * ROOM - offset];
}

/** Every model once per room, turned by a quarter as its place and room say. */
export function placements(rooms: number): Placement[] {
  const out: Placement[] = [];
  const rows = Math.ceil(MODELS.length / COLUMNS);
  for (let room = 0; room < rooms; room++) {
    const [ox, oz] = roomOrigin(room, rooms);
    for (let model = 0; model < MODELS.length; model++) {
      const col = model % COLUMNS;
      const row = Math.floor(model / COLUMNS);
      out.push({
        model,
        x: ox + (col - (COLUMNS - 1) / 2) * SPACING,
        z: oz + (row - (rows - 1) / 2) * SPACING,
        rotY: ((model + room) % 4) * (Math.PI / 2),
      });
    }
  }
  return out;
}

/** A small deterministic random sequence (the same scene in both engines). */
function random(seed: number): () => number {
  let state = seed >>> 0;
  return () => ((state = (Math.imul(state, 1664525) + 1013904223) >>> 0) / 2 ** 32);
}

/** The trees: scattered between 10 and 34 m from the middle, of the four kinds in turn. */
export function treePlacements(count: number): Placement[] {
  const next = random(7);
  return Array.from({ length: count }, (_, i) => {
    const angle = next() * Math.PI * 2;
    const distance = 10 + next() * 24;
    return { model: i % TREES.length, x: Math.cos(angle) * distance, z: Math.sin(angle) * distance, rotY: next() * Math.PI * 2 };
  });
}

/** The garden's models: each once, round the house 6.5 m out. */
export function gardenPlacements(): Placement[] {
  return GARDEN.map((_, model) => {
    const angle = (model / GARDEN.length) * Math.PI * 2 + 0.3;
    return { model, x: Math.cos(angle) * 6.5, z: Math.sin(angle) * 6.5, rotY: angle };
  });
}

/** Where the lamps hang: spread over the rooms in turn, 2.4 m up. */
export function lampPositions(rooms: number, count: number): [number, number, number][] {
  const perRoom = Math.ceil(count / rooms);
  const grid = Math.ceil(Math.sqrt(perRoom));
  return Array.from({ length: count }, (_, i) => {
    const room = i % rooms;
    const slot = Math.floor(i / rooms);
    const [ox, oz] = roomOrigin(room, rooms);
    const step = ROOM / grid;
    return [ox + ((slot % grid) + 0.5) * step - ROOM / 2, 2.4, oz + (Math.floor(slot / grid) + 0.5) * step - ROOM / 2];
  });
}

/** Lamps: warm, reaching 5 m, a 120° cone straight down. */
export const LAMP = { color: [1, 0.78, 0.55] as const, intensity: 3, range: 5, angle: (Math.PI * 2) / 3 };
/** The sun: low enough for long shadows, cast towards where the camera starts (`cameraAt`), white. */
export const SUN = { direction: [0.6, -0.8, 0.45] as const, intensity: 2.2 };
export const AMBIENT = 0.35;
export const CLEAR = [0.62, 0.76, 0.88] as const;
export const FLOOR = 0.55;

/** Half the width of the furnished area (m). */
export function extent(rooms: number): number {
  return (Math.ceil(Math.sqrt(rooms)) * ROOM) / 2;
}

/** The camera: a vertical field of view, near and far planes, and one turn round the rooms per this many seconds. */
export const CAMERA = { fov: Math.PI / 4, near: 0.3, far: 500, period: 20 };

/** The floor's side (m): the rooms with a margin, or out past the trees. */
export function floorSize(rooms: number, spec: CaseSpec): number {
  return spec.trees ? 72 : extent(rooms) * 2 + 4;
}

/** How far the camera orbits from the middle (m): the case's own distance, or far enough to see every room. */
export function orbitRadius(rooms: number, spec: CaseSpec): number {
  return spec.orbit ?? extent(rooms) * 2.6 + 4;
}

/** The camera `t` seconds into the run: orbiting at 40° above the floor. */
export function cameraAt(t: number, radius: number): { position: [number, number, number]; target: [number, number, number] } {
  const angle = (t / CAMERA.period) * Math.PI * 2;
  const pitch = (40 * Math.PI) / 180;
  return {
    position: [Math.cos(angle) * Math.cos(pitch) * radius, Math.sin(pitch) * radius, Math.sin(angle) * Math.cos(pitch) * radius],
    target: [0, 0.6, 0],
  };
}

/** One engine set up for a case: draws a frame, says how many draw calls it took. */
export interface Driver {
  /** What it runs on, as the engine reports it (e.g. `webgpu`). */
  readonly api: string;
  readonly version: string;
  /** Draws a frame `t` seconds into the run: the engine's whole frame (update, culling, submit). */
  frame(t: number): void;
  /** Draw calls of the last frame. */
  drawCalls(): number;
  /** Releases the engine and its GPU device (a browser can slow down for every one still alive). */
  dispose(): void;
}

export interface DriverOptions {
  canvas: HTMLCanvasElement;
  api: 'webgpu' | 'webgl2';
  width: number;
  height: number;
  rooms: number;
  spec: CaseSpec;
}
