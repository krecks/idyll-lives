/**
 * Step 2's settle: the town's plan draft, as the player's edits left it, made the town the next
 * steps and the renderer read (decision 10: done once edits pause, while the ghosts show them).
 *
 * Every edit's settle goes through `settle`, and only through it, so pointing it at the town
 * worker is one change here. For now the plan draft is the settled one: its land is as the town was
 * laid (graded under the town as generated), and only its preview is made, on the page, as the
 * menus make one for a town nobody lives in yet (`previewWorld`; `menuScene.prepare` keeps it for
 * the partial rebuild).
 */

import type { WorldStructure } from '../../core/protocol';
import { Superseded } from '../../game/townJobs';
import { loadTemplates, type NeighbourhoodDraft } from '../../game/town';
import { previewWorld } from '../../game/townPreview';
import { services } from '../services';

/** A plan draft settled: the town the next steps read, its preview (for the renderer), and the edits the settle refused, with why (by their place among the edits since the last settle). */
export interface Settled {
  draft: NeighbourhoodDraft;
  preview: WorldStructure;
  refused?: { op: number; why: string }[];
}

/**
 * `plan` settled (see the top of this file). `ops`: how many edits it is past the town settled
 * before it (for the refusals' places). `signal`: a newer settle replaces this one (it rejects with
 * `Superseded`).
 *
 * TODO(XH): settle in the town worker, as decision 10 has it: the land settled under the plan on
 * the bare land (`landFollow.ts` `settleEdit`: `settleTown`, `gentleUnder`, `gradeLand`, no
 * `giveUp`), its preview, its new houses and its problems. Replace the body with:
 *
 *   const made = await townJob(services.content, { kind: 'edit', plan, ops }, signal);
 *   keepHouses(made);
 *   return { draft: made.draft, preview: made.preview, refused: made.refused };
 *
 * (`townJobs.ts` `TownJob` `'edit'`; its `problems` may then replace the page's `townIssues` too.)
 */
export async function settle(plan: NeighbourhoodDraft, ops: number, signal?: AbortSignal): Promise<Settled> {
  void ops;
  const t = await loadTemplates();
  if (signal?.aborted) throw new Superseded();
  return { draft: plan, preview: previewWorld(services.content, t, plan, null) };
}
