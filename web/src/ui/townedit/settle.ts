/**
 * Step 2's settle: the town's plan draft, as the player's edits left it, made the town the next
 * steps and the renderer read (decision 10: done once edits pause, while the ghosts show them).
 *
 * Every edit's settle goes through `settle`, and only through it: the town worker's `edit` job
 * (`townJobs.ts` `editTown`: `landFollow.ts` `settleEdit`), the land settled under the plan as laid
 * (gentle ground, graded) giving nothing up; an edit that takes the land over a slope limit is
 * refused, and the plan before it settled instead. Its preview comes with it (`menuScene.prepare`
 * keeps it for the partial rebuild) and its new houses (kept on the page: `keepHouses`).
 */

import type { WorldStructure } from '../../core/protocol';
import { editTown, keepHouses } from '../../game/townJobs';
import type { NeighbourhoodDraft } from '../../game/town';
import { services } from '../services';

/**
 * A plan draft settled: the town the next steps read, its preview (for the renderer), the edits the
 * settle refused, with why (by their place among the edits since the last settle), the plan that was
 * settled (`step`: its index among the batch's `steps`) and what in it is over a slope limit (`marks`:
 * the next settle's `before`).
 */
export interface Settled {
  draft: NeighbourhoodDraft;
  preview: WorldStructure;
  refused?: { op: number; why: string }[];
  step?: number;
  marks?: readonly string[];
}

/** The batch a settle is of: the plans it went through, the first the plan the town on screen was settled from, the last the one settled; and that settle's `marks`, if known. */
export interface Batch {
  steps: readonly NeighbourhoodDraft[];
  before?: readonly string[];
}

/**
 * `plan` settled (see the top of this file). `ops`: how many edits it is past the town settled
 * before it (for the refusals' places); `batch`: the plans those edits went through (without, none
 * is refused). `signal`: a newer settle replaces this one (it rejects with `Superseded`).
 */
export async function settle(plan: NeighbourhoodDraft, ops: number, signal?: AbortSignal, batch?: Batch): Promise<Settled> {
  // (The Problems list is checked on its own, as the plan changes: `checkJob.ts`.)
  const made = await editTown(services.content, { plan, ops, ...(batch ? { steps: batch.steps, ...(batch.before ? { before: batch.before } : {}) } : {}), check: false }, signal);
  keepHouses(made);
  return { draft: made.draft, preview: made.preview, refused: made.refused, step: made.step, marks: made.marks };
}
