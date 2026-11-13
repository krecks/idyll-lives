/**
 * What the ground is made of at a point, for the footsteps' sound (`footsteps.ts`): read from the world's
 * structure the way the house and the street draw it, with nothing asked of the simulation.
 *
 * Indoors a tile has the floor covering laid on it in Build mode (its finish: wood, tile, carpet, stone), else
 * the floor its room gets by what stands in it (`render/shared/house.ts`: a bathroom and a kitchen tiles, a
 * bedroom carpet, anything else wood). Outdoors a road, its pavement and a path are paving, a patio what was
 * laid on it, the rest grass. No DOM or audio context here.
 */

import { groundDepth, type WorldStructure } from '../core/protocol';
import { ROOM_FURNITURE } from '../render/shared/house';
import { roadTiles, townRoads } from '../render/shared/roads';
import type { Surface } from './steps';

/** What a floor covering's finish (content `floorCoverings`) sounds like underfoot. */
const FINISH: Record<string, Surface> = { wood: 'wood', whitewash: 'wood', tile: 'tile', carpet: 'carpet', stone: 'paving' };

/** The floor a room gets by its furniture, as the house draws it (`HousePlanner.build`'s `kindOf`). */
function roomFloor(defs: ReadonlySet<string> | undefined): Surface {
  if (!defs) return 'wood';
  const has = (ids: readonly string[]) => ids.some((id) => defs.has(id));
  if (has(ROOM_FURNITURE.bath)) return 'tile';
  if (has(ROOM_FURNITURE.living)) return 'wood';
  if (has(ROOM_FURNITURE.bedroom) && !has(ROOM_FURNITURE.kitchen)) return 'carpet';
  if (has(ROOM_FURNITURE.kitchen) && !has(ROOM_FURNITURE.bedroom)) return 'tile';
  return 'wood';
}

/** What the world's structure says of its ground (`WorldStructure`). */
export type GroundWorld = Pick<WorldStructure, 'width' | 'depth' | 'storeyDepth' | 'rooms' | 'floors' | 'objects' | 'meta'>;

/** The ground of a world: what is underfoot at a tile point. */
export class Ground {
  private world: GroundWorld | null = null;
  /** The covering laid on each tile that has one (by `z * width + x`), and each room's own floor: worked out when first asked after the world changed. */
  private laid: Map<number, number> | null = null;
  private readonly floors = new Map<number, Surface>();
  private roads: Uint8Array | null = null;

  /** `finishes`: the content's floor coverings' finishes, in their order (a tile's covering is an index into them, plus 1). */
  constructor(private readonly finishes: readonly string[]) {}

  setWorld(world: GroundWorld | null): void {
    this.world = world;
    this.laid = null;
  }

  /** What is underfoot at tile point (`x`, `z`: the world's rows, a storey's own). */
  at(x: number, z: number): Surface {
    const w = this.world;
    if (!w) return 'grass';
    const tx = Math.floor(x);
    const tz = Math.floor(z);
    if (!(tx >= 0 && tx < w.width && tz >= 0 && tz < w.depth)) return 'grass';
    if (!this.laid) this.read(w);
    const i = tz * w.width + tx;
    const covering = this.laid!.get(i);
    const finish = covering ? this.finishes[covering - 1] : undefined;
    if (finish && FINISH[finish]) return FINISH[finish];
    const room = w.rooms[i];
    if (room) return this.floors.get(room) ?? 'wood';
    // Outdoors on a storey above the ground: a deck.
    const ground = groundDepth(w);
    if (tz >= ground) return 'wood';
    if (this.roads?.[i]) return 'paving';
    for (const p of w.meta?.paths ?? []) if (x >= p.x && x < p.x + p.w && z >= p.z && z < p.z + p.d) return 'paving';
    return 'grass';
  }

  private read(w: GroundWorld): void {
    this.laid = new Map();
    for (const [x, z, c] of w.floors ?? []) this.laid.set(z * w.width + x, c);
    // Each room's floor by the furniture that says what it is for.
    const telling = new Set(Object.values(ROOM_FURNITURE).flat());
    const defs = new Map<number, Set<string>>();
    for (const o of w.objects) {
      if (!telling.has(o.def)) continue;
      for (let z = o.z; z < o.z + o.d; z++) {
        for (let x = o.x; x < o.x + o.w; x++) {
          const room = x >= 0 && x < w.width && z >= 0 && z < w.depth ? w.rooms[z * w.width + x] : 0;
          if (!room) continue;
          let set = defs.get(room);
          if (!set) defs.set(room, (set = new Set()));
          set.add(o.def);
        }
      }
    }
    this.floors.clear();
    for (const [room, set] of defs) this.floors.set(room, roomFloor(set));
    // (The street's own map of its roads and pavements: made once per town, shared with it.)
    this.roads = w.meta ? roadTiles(townRoads(w as WorldStructure), w.width, groundDepth(w)) : null;
  }
}
