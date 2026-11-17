/**
 * Step 2's undo and redo: the town's plan drafts as the edits made them (unsettled, with no copy of
 * the land), the last `HISTORY_MOST` kept. Undo and redo give a draft to settle again like any edit;
 * an edit the settle refused is dropped (`revert`).
 */

import type { NeighbourhoodDraft } from '../town';

/** How many edits back undo reaches. */
export const HISTORY_MOST = 50;

export class TownHistory {
  private drafts: NeighbourhoodDraft[];
  private at = 0;
  /** How many edits the first draft kept is from the town as it was laid (more than `HISTORY_MOST` edits drop the oldest). */
  private dropped = 0;

  /** `base`: the town as it was laid (what `reset` goes back to). */
  constructor(readonly base: NeighbourhoodDraft) {
    this.drafts = [base];
  }

  /** The draft as it stands. */
  get draft(): NeighbourhoodDraft {
    return this.drafts[this.at];
  }

  /** How many edits the draft as it stands is from the town as it was laid (0: none, or all undone). */
  get edits(): number {
    return this.dropped + this.at;
  }

  get canUndo(): boolean {
    return this.at > 0;
  }

  get canRedo(): boolean {
    return this.at < this.drafts.length - 1;
  }

  /** An edit made: `draft` from now on (what could be redone is gone). */
  push(draft: NeighbourhoodDraft): void {
    this.drafts = [...this.drafts.slice(0, this.at + 1), draft];
    if (this.drafts.length > HISTORY_MOST + 1) {
      this.drafts.shift();
      this.dropped++;
    }
    this.at = this.drafts.length - 1;
  }

  /** The draft before the last edit; null if there is none. */
  undo(): NeighbourhoodDraft | null {
    if (!this.canUndo) return null;
    return this.drafts[--this.at];
  }

  /** The edit undone last made again; null if there is none. */
  redo(): NeighbourhoodDraft | null {
    if (!this.canRedo) return null;
    return this.drafts[++this.at];
  }

  /**
   * Back to `draft`, a plan the edits went through, when the settle refused the edit made on it
   * (`landFollow.ts` `settleEdit`): the edits since are dropped, none to redo (a refused edit
   * redone would only be refused again). A plan no longer kept (more than `HISTORY_MOST` back, or
   * dropped by an edit after an undo) is taken on as an edit. The draft as it stands then.
   */
  revert(draft: NeighbourhoodDraft): NeighbourhoodDraft {
    const at = this.drafts.lastIndexOf(draft, this.at);
    if (at < 0) this.push(draft);
    else {
      this.drafts = this.drafts.slice(0, at + 1);
      this.at = at;
    }
    return this.draft;
  }

  /** Back to the town as it was laid, every edit dropped (none can be redone). */
  reset(): NeighbourhoodDraft {
    this.drafts = [this.base];
    this.at = 0;
    this.dropped = 0;
    return this.base;
  }
}
