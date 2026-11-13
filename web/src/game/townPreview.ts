/**
 * The 3D preview of a neighbourhood draft for the menus, built on the main thread (no sim
 * worker): the same layout `assembleTown` hands the simulation, turned into the structure the
 * renderer draws (wall edges, openings, rooms, plots with their house outlines, objects).
 * Mirrors what sim-core reports for a loaded town (`view.rs`), so the menus and the game show
 * the same houses, gardens, street and landscape.
 */

import type { Content } from '../content/content';
import type { FenceEdge, ObjectPlacement, Opening, PlotInfo, WallEdge, WorldStructure } from '../core/protocol';
import { layoutTown, worldLook, type NeighbourhoodDraft, type Templates, type TownMeta } from './town';

const EMPTY_MESH = { positions: new Float32Array(0), normals: new Float32Array(0), uvs: new Float32Array(0), indices: new Uint32Array(0) };

/** World structure of a draft (`playerSlot` is drawn as the player's house, as in the game). */
export function previewWorld(content: Content, t: Templates, town: NeighbourhoodDraft, playerSlot: number | null = null): WorldStructure {
  const layout = layoutTown(content, t, town, playerSlot);
  const W = town.width;
  const D = town.depth;

  // Wall edges, as sim-core stores them: `h` (x, z) runs to (x + 1, z), `v` (x, z) to (x, z + 1).
  const h = new Uint8Array(W * (D + 1)); // 0 open, 1 wall, 2 door, 3 window
  const v = new Uint8Array((W + 1) * D);
  // The town from the street: ground storeys only (houses that are taller say so in `plots`).
  for (const [x0, z0, x1, z1] of layout.walls) {
    if (Math.max(z0, z1) > D) continue;
    if (z0 === z1) for (let x = Math.min(x0, x1); x < Math.max(x0, x1); x++) h[z0 * W + x] = 1;
    else for (let z = Math.min(z0, z1); z < Math.max(z0, z1); z++) v[z * (W + 1) + x0] = 1;
  }
  for (const [list, kind] of [
    [layout.doors, 2],
    [layout.windows, 3],
  ] as const) {
    for (const d of list) {
      if (d.z >= D) continue;
      if (d.axis === 'x' && h[d.z * W + d.x]) h[d.z * W + d.x] = kind;
      if (d.axis === 'z' && v[d.z * (W + 1) + d.x]) v[d.z * (W + 1) + d.x] = kind;
    }
  }
  const walls: WallEdge[] = [];
  const openings: Opening[] = [];
  const add = (axis: 'h' | 'v', x: number, z: number, e: number) => {
    if (!e) return;
    walls.push({ axis, x, z });
    if (e > 1) openings.push({ axis, x, z, kind: e === 2 ? 'door' : 'window' });
  };
  for (let z = 0; z <= D; z++) for (let x = 0; x < W; x++) add('h', x, z, h[z * W + x]);
  for (let z = 0; z < D; z++) for (let x = 0; x <= W; x++) add('v', x, z, v[z * (W + 1) + x]);

  // Indoors: tiles not reachable from the town's edge without crossing a wall (doors count as walls).
  const outdoor = new Uint8Array(W * D);
  const stack: number[] = [];
  const visit = (x: number, z: number) => {
    const i = z * W + x;
    if (!outdoor[i]) {
      outdoor[i] = 1;
      stack.push(i);
    }
  };
  for (let x = 0; x < W; x++) {
    visit(x, 0);
    visit(x, D - 1);
  }
  for (let z = 0; z < D; z++) {
    visit(0, z);
    visit(W - 1, z);
  }
  while (stack.length) {
    const i = stack.pop()!;
    const x = i % W;
    const z = (i - x) / W;
    if (x > 0 && !v[z * (W + 1) + x]) visit(x - 1, z);
    if (x < W - 1 && !v[z * (W + 1) + x + 1]) visit(x + 1, z);
    if (z > 0 && !h[z * W + x]) visit(x, z - 1);
    if (z < D - 1 && !h[(z + 1) * W + x]) visit(x, z + 1);
  }
  const rooms = Array.from(outdoor, (o) => (o ? 0 : 1));

  const plots: PlotInfo[] = layout.plots.map((p, id) => {
    // Bounding box of the walls on the plot (sim-core `house_bounds`): a wall on the plot's own edge only where the
    // plot's tile beside it is indoors (next door's wall on the shared line, a terrace's, is next door's).
    let box: [number, number, number, number] | null = null;
    const grow = (ax: number, az: number, bx: number, bz: number) => {
      box = box ? [Math.min(box[0], ax), Math.min(box[1], az), Math.max(box[2], bx), Math.max(box[3], bz)] : [ax, az, bx, bz];
    };
    const indoors = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && !outdoor[z * W + x];
    const [x1, z1] = [p.x + p.w, p.z + p.d];
    for (let z = p.z; z <= Math.min(z1, D); z++) {
      for (let x = p.x; x < Math.min(x1, W); x++) if (h[z * W + x] && !(z === p.z && !indoors(x, z)) && !(z === z1 && !indoors(x, z - 1))) grow(x, z, x + 1, z);
    }
    for (let z = p.z; z < Math.min(z1, D); z++) {
      for (let x = p.x; x <= Math.min(x1, W); x++) if (v[z * (W + 1) + x] && !(x === p.x && !indoors(x, z)) && !(x === x1 && !indoors(x - 1, z))) grow(x, z, x, z + 1);
    }
    return { id, name: p.name, x: p.x, z: p.z, w: p.w, d: p.d, public: p.public, entry: p.entry, house: box, storeys: p.storeys };
  });

  const objects: ObjectPlacement[] = layout.objects.filter((o) => o.z < D).map((o, id) => {
    const [fw, fd] = content.object(o.def)?.footprint ?? [1, 1];
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    return { id, def: o.def, x: o.x, z: o.z, rot: o.rot, turn: o.turn, offset: o.offset, w, d, quality: 0, sellValue: null };
  });

  // The town's garden fences, as the simulation reports them (styles by index).
  const style = (id: string) => Math.max(0, content.fenceStyles.findIndex((f) => f.id === id));
  const fences: FenceEdge[] = layout.fences.map((f) => ({ axis: f.axis, x: f.x, z: f.z, kind: f.kind, style: style(f.style) }));

  const meta: TownMeta = { kind: 'town', name: town.name, roads: town.roads, greens: town.greens, paths: layout.paths, facing: layout.facing, seed: town.seed ?? 1, world: worldLook(town, layout.fields), home: playerSlot, gardenFences: true };
  // A big world's land and areas, as the town file carries them (`assembleTown`).
  if (town.open) {
    const { zones, pads, areas, backdrop, places, centre } = town.open;
    // (And the living town's backdrop, places and centres: the land is cleared for them as in the game, and the
    // town view over a preview, the bench's Big world, draws them.)
    Object.assign(meta, { zones, pads, areas, ...(backdrop ? { backdrop } : {}), ...(places ? { places } : {}), ...(centre ? { centre } : {}) });
  }
  return {
    version: 0,
    lotVersion: 0,
    mode: 'living',
    width: W,
    depth: D,
    objects,
    sims: [],
    households: town.households.map((x, id) => ({ id, name: x.household.name, plot: x.slot, player: false })),
    plots,
    exits: [],
    rooms,
    walls,
    openings,
    fences,
    meta,
    meshes: { walls: EMPTY_MESH, wallsLow: EMPTY_MESH, floors: EMPTY_MESH },
    garages: layout.garages,
    porches: layout.porches,
    // The households' cars where the game parks them (the player's slot has one), as the street drew them.
    cars: layout.cars.map(({ plot, x, z, yaw, body, paint }) => ({ plot, x, z, yaw, body, paint })),
  };
}
