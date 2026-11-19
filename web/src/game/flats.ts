/**
 * Flats: a building of several homes on one lot, each a plot of its own (`FlatInfo`, the town
 * file's `plots[].of`) on one storey. A walk-up of 2 or 3 storeys round a stair hall, its flats
 * generated from the house generator's `flat` class, its lot a front strip with a bay for some of
 * its flats and a yard behind. The same model makes the flats over a shop, a café or a restaurant.
 */

import type { LotLook } from '../core/protocol';
import type { Residents } from './housegen';
import type { Door, Placement, PlotSlot, Rect, Templates, Wall } from './town';

type Rand = () => number;

/** A flats building as a slot takes it: its template (`flats:<seed>:<storeys>:<wide>`), its lot's size and its look. */
export type FlatsSlot = Pick<PlotSlot, 'template' | 'w' | 'd' | 'dress'>;

/** One storey of a flats building (its hall, stairs and the flats' own walls and things), in plot coordinates (street at z = 0) on its storey's rows. */
export interface StoreyShell {
  storey: number;
  walls: Wall[];
  doors: Door[];
  windows?: Door[];
  objects: Placement[];
}

/**
 * A flat of a building: its storey, its rectangle (plot coordinates on the ground's rows), the landing tile before its
 * own door and the tile behind it (plot coordinates on its storey's rows), its bedrooms.
 */
export interface FlatUnit {
  storey: number;
  rect: Rect;
  door: [number, number];
  inside: [number, number];
  bedrooms: number;
}

/**
 * A flats building laid out (`flatsTemplate`): its lot, its storeys (every storey's shell: storey 0 the hall
 * and the ground floor's flats, or none over a place), its flats, the bays in its front strip (plot
 * coordinates; one each for the first flats), the things in its yard, its street door (`entry`) and its look.
 */
export interface FlatsTemplate {
  id: string;
  lot: { width: number; depth: number };
  storeys: number;
  shell: StoreyShell[];
  units: FlatUnit[];
  bays: Rect[];
  yard: Placement[];
  entry: [number, number];
  look: LotLook;
}

/**
 * A flats building for a lot in `zone` (the centre or the rest of a town), drawn from `rand`; none
 * until the generator makes flats (`flats.ts` is filled in by the flats stream).
 */
export function flatsFor(t: Templates, zone: 'centre' | 'town', rand: Rand): FlatsSlot | null {
  void t;
  void zone;
  void rand;
  return null;
}

/**
 * The building of a flats template id (`flatsFor`'s, or `upperFor`'s over a place), each flat furnished for
 * its residents (by flat, in `units`' order; `'shell'`: walls, kitchen and bathroom only; undefined: for
 * sale); none until the generator makes flats.
 */
export function flatsTemplate(t: Templates, id: string, residents?: readonly (Residents | 'shell' | undefined)[]): FlatsTemplate | undefined {
  void [t, id, residents];
  return undefined;
}

/**
 * What stands over a named place's ground floor (`place`: its layout and lot), `storeys` more storeys of it:
 * flats over a shop, a café or a restaurant (`use` `flats`: their own street door and stair hall beside the
 * place's door), or an office's or a hall's own upper floors (`office`). Its id, for `flatsTemplate` (an
 * office's floors have no flats); null where none is made (none until the generator makes them).
 */
export function upperFor(t: Templates, place: Pick<PlotSlot, 'template' | 'w' | 'd'>, use: 'flats' | 'office', storeys: number, rand: Rand): string | null {
  void [t, place, use, storeys, rand];
  return null;
}
