/**
 * What a town edit gives (step 2's tools, `game/edit/`): the town's plan draft with the edit made,
 * and what it cleared out of the way (lots and backdrop buildings, by their index in the draft it
 * was made on), or why it was refused. An edit is a pure function from draft to draft; the land is
 * settled under the result later (`landFollow.ts` `settleEdit`).
 */

import type { NeighbourhoodDraft } from '../town';

export type Edited = { draft: NeighbourhoodDraft; cleared: { lots: number[]; backdrop: number[] } } | { refused: string };
