/**
 * What a town edit gives (step 2's tools, `game/edit/`): the town's plan draft with the edit made,
 * and what it cleared out of the way (lots and backdrop buildings, by their index in the draft it
 * was made on), or why it was refused. An edit is a pure function from draft to draft; the land is
 * settled under the result later (`landFollow.ts` `settleEdit`).
 *
 * An edit that was made but left something the player should know of (a village with no road to it,
 * an old road left as a dead end because homes stand on it) says so in `notes`, as clauses.
 *
 * A street drawn (`streets.ts` `drawStreet`) says which roads it made in `made` (its own pieces, by
 * their index in the draft: not the footpaths it cut, which changed besides).
 */

import type { NeighbourhoodDraft } from '../town';

export type Edited = { draft: NeighbourhoodDraft; cleared: { lots: number[]; backdrop: number[] }; notes?: string[]; made?: number[] } | { refused: string };
