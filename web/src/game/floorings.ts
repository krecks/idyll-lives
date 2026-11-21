/**
 * The floors a household's home has laid (`housegen.json` `floorings`): a floor covering for each room
 * of a generated house or flat by what the room is (`PlannedRoom.kind`), drawn by weight, the house's
 * class (`HouseTemplate.class`: a villa's hall marble, a cottage's flagstone) weighing in where it says
 * more. A house lays one covering a kind (both bathrooms alike), a bedroom now and then one of its own; the
 * entrance hall never the living room's, so the way in is a floor of its own. The town lays them in the
 * town file (`layoutTown` `floors`), as Build mode's Floor tool lays a covering, so the simulation knows
 * them (a carpet the mop leaves), saves keep them and a player may lay another.
 */

import { weighted } from './housegen';
import type { PlannedRoom } from './town';

/** Covering ids (content `floorCoverings`) by weight. */
export type FloorWeights = Record<string, number>;

export interface FlooringSpec {
  /** By room kind (`hall`: the landing upstairs); a kind not listed keeps the automatic floor. */
  rooms: Record<string, FloorWeights>;
  /** A basement's rooms, by kind, before `rooms`. */
  below?: Record<string, FloorWeights>;
  /** By house class (its label), by room kind: in place of `rooms`' weights. */
  classes?: Record<string, Record<string, FloorWeights>>;
  /** How often a bedroom after the first has a covering of its own (0..1). */
  ownBedroom?: number;
}

/** A floor laid: `w` × `d` tiles from `(x, z)` in one covering (by id), as the town file has them. */
export interface LaidFloor {
  x: number;
  z: number;
  w: number;
  d: number;
  covering: string;
}

/** The rooms' kinds whose covering must not be the one beside them: the entrance hall's, the living room's. */
const APART: readonly [string, string][] = [['entry', 'living']];

/**
 * A house's floor picker: the covering of a room of `kind` (`below`: in the basement), null to keep the
 * automatic floor. `known`: the coverings the content has (ids missing from it are passed over). The same
 * `rng` draws give the same floors, so a house keeps its floors as long as its town keeps its world code.
 */
export function floorPicker(spec: FlooringSpec, cls: string | undefined, known: ReadonlySet<string>, rng: () => number): (kind: string, below?: boolean) => string | null {
  const chosen = new Map<string, string | null>();
  let bedrooms = 0;
  const weightsOf = (kind: string, below: boolean): FloorWeights | undefined =>
    (below ? spec.below?.[kind] : undefined) ?? (cls ? spec.classes?.[cls]?.[kind] : undefined) ?? spec.rooms[kind];
  const draw = (kind: string, below: boolean): string | null => {
    const weights = { ...weightsOf(kind, below) };
    for (const id of Object.keys(weights)) if (!known.has(id)) delete weights[id];
    // Kept apart from a kind already chosen (else that one is drawn again away from it).
    for (const [a, b] of APART) {
      const other = kind === a ? b : kind === b ? a : null;
      const theirs = other ? chosen.get(other) : null;
      if (theirs && Object.keys(weights).length > 1) delete weights[theirs];
    }
    return weighted(rng, weights);
  };
  return (kind, below = false) => {
    const key = below ? `below:${kind}` : kind;
    if (kind === 'bedroom' && bedrooms++ > 0 && rng() < (spec.ownBedroom ?? 0)) return draw(kind, below);
    if (!chosen.has(key)) chosen.set(key, draw(kind, below));
    return chosen.get(key)!;
  };
}

/** The floors of a storey's rooms, as a picker lays them (`below`: a basement's). */
export function laidFloors(rooms: readonly PlannedRoom[] | undefined, pick: (kind: string, below?: boolean) => string | null, below = false): LaidFloor[] {
  const out: LaidFloor[] = [];
  // The entrance hall before the rest, so the living room keeps away from it whichever comes first in the plan.
  const order = [...(rooms ?? [])].sort((a, b) => Number(b.kind === 'entry') - Number(a.kind === 'entry'));
  for (const r of order) {
    const covering = r.w > 0 && r.d > 0 ? pick(r.kind, below) : null;
    if (covering) out.push({ x: r.x, z: r.z, w: r.w, d: r.d, covering });
  }
  return out;
}
