/**
 * Step 2's editing, as state: the town's plan draft (what the player's edits make, `game/edit/`),
 * the town settled from it and on screen, undo and redo (`TownHistory`), and what is wrong with the
 * plan (the Problems list). No Svelte in it, so the tests can drive it; `editor.svelte.ts` wraps it
 * for the screens.
 *
 * Editing is two-layer (decision 10). An edit takes at once: the plan changes, and the screen shows
 * the change as ghosts over the town (`ghosts.ts` `editGhosts`, the plan against the town on
 * screen). Once edits have paused (`QUIET` ms after one made in a run, as the Lot tool's homes put
 * down click after click; `QUIET_ONE` after one on its own) the plan is settled (`settle.ts`
 * `settle`) and the town built again from it (`show`); edits made meanwhile wait for the next pause. One settle at a
 * time: a pause that comes while one runs is taken once it is done. `flush` settles now (the way on
 * to step 3 takes the town as it stands).
 */

import type { Edited } from '../../game/edit/edited';
import { TownHistory } from '../../game/edit/history';
import type { NeighbourhoodDraft } from '../../game/town';
import type { TownIssues } from './problems';
import type { Batch, Settled } from './settle';

/** How long edits pause (ms) after one made in a run (`apply`'s `run`) before the town is settled and built again with them. */
export const QUIET = 1200;
/** The same after an edit made on its own (a button, a drag let go, an undo): the town is built again soon after. */
export const QUIET_ONE = 250;
/** How long after an edit (ms) the Problems list is checked again (the ghosts first). */
export const CHECK_AFTER = 40;

/** Timers, as `setTimeout`'s (the tests' are their own). */
export interface Timers {
  set(fn: () => void, ms: number): unknown;
  clear(handle: unknown): void;
}

/** What the session needs from outside it. */
export interface SessionHooks {
  /** A plan settled (`settle.ts` `settle`; `ops`: edits since the last, `batch` the plans they went through). Rejects with `Superseded` if `signal` aborts it. */
  settle(plan: NeighbourhoodDraft, ops: number, signal: AbortSignal, batch: Batch): Promise<Settled>;
  /** The settled town put on screen; resolves once it is. `plan`: what it was settled from. */
  show(settled: Settled, plan: NeighbourhoodDraft): Promise<void>;
  /** Something the screens show changed (the plan, the town on screen, the problems, whether a settle runs). */
  changed(): void;
  /**
   * What is wrong with a plan (`problems.ts` `townIssues`; `base`: the town as it was laid's). It may
   * answer later (a big town's checks take seconds: they run in a worker, `checkJob.ts`); one runs at a time.
   */
  check?(plan: NeighbourhoodDraft, base?: TownIssues): TownIssues | Promise<TownIssues>;
  /** A settle refused an edit, or failed: why (shown to the player). */
  report?(why: string): void;
  timers?: Timers;
  now?: () => number;
  /** The pauses (ms) after an edit made in a run and after one on its own (`QUIET`, `QUIET_ONE`). */
  quiet?: number;
  quietOne?: number;
}

/** One settle's timing (ms): the settle itself, putting the town on screen, and from the last edit before it to the town on screen. */
export interface SettleTiming {
  settle: number;
  show: number;
  fromEdit: number;
}

const realTimers: Timers = { set: (fn, ms) => setTimeout(fn, ms), clear: (h) => clearTimeout(h as ReturnType<typeof setTimeout>) };
const NO_ISSUES: TownIssues = { blocking: [], warnings: [] };

export class EditSession {
  readonly history: TownHistory;
  /** The plan draft as the edits left it. */
  plan: NeighbourhoodDraft;
  /** The settled town on screen, and the plan it was settled from. */
  settled: NeighbourhoodDraft;
  settledFrom: NeighbourhoodDraft;
  /** What is wrong with the plan, and with the town as it was laid (its problems block nothing). */
  issues: TownIssues = NO_ISSUES;
  private baseIssues: TownIssues | undefined;
  /** A settle under way. */
  settling = false;
  readonly timings: SettleTiming[] = [];
  private readonly timers: Timers;
  private readonly now: () => number;
  private readonly quiet: number;
  private readonly quietOne: number;
  private timer: unknown = null;
  /** The Problems list's next check, waiting for its moment; the checks under way. */
  private checkTimer: unknown = null;
  private checking: Promise<void> | null = null;
  /** The plan `issues` are of. */
  private checkedFor: NeighbourhoodDraft | null = null;
  private running: Promise<void> | null = null;
  private again = false;
  private control: AbortController | null = null;
  private lastEdit = 0;
  /** Edits since the town on screen was settled, and the plans they made, in turn. */
  private ops = 0;
  private taken: NeighbourhoodDraft[] = [];
  /** What is over a slope limit in the town on screen (its settle's `marks`), once known. */
  private marks: readonly string[] | undefined;
  private disposed = false;

  /**
   * `settled`: the town on screen (as laid, or settled before); `history`: the edits made before (kept
   * while the player goes back and forth through the steps), else none; `from`: the plan `settled`
   * came from (else `settled` itself). A plan past it is settled after the first pause. `marks`: what
   * in `settled` is over a slope limit (`Settled.marks`), if known: the first settle need not find it.
   */
  constructor(
    settled: NeighbourhoodDraft,
    private readonly hooks: SessionHooks,
    history?: TownHistory,
    from?: NeighbourhoodDraft,
    marks?: readonly string[],
  ) {
    this.history = history ?? new TownHistory(from ?? settled);
    this.plan = this.history.draft;
    this.settled = settled;
    this.settledFrom = from ?? settled;
    this.timers = hooks.timers ?? realTimers;
    this.now = hooks.now ?? (() => performance.now());
    this.quiet = hooks.quiet ?? QUIET;
    this.quietOne = hooks.quietOne ?? QUIET_ONE;
    this.marks = marks;
    this.lastEdit = this.now();
    this.recheck(true);
    if (this.pending) this.schedule(this.quietOne);
  }

  /** The Problems list is of the plan as it stands. */
  get checked(): boolean {
    return !this.hooks.check || this.checkedFor === this.plan;
  }

  /** The plan has edits the town on screen hasn't yet. */
  get pending(): boolean {
    return this.plan !== this.settledFrom;
  }
  get edits(): number {
    return this.history.edits;
  }
  get canUndo(): boolean {
    return this.history.canUndo;
  }
  get canRedo(): boolean {
    return this.history.canRedo;
  }

  /**
   * An edit made (`game/edit/`): taken on (null), or why it was refused. An edit that changes nothing
   * is not one. `run`: one of a run of edits (more are likely to follow it soon): the town is built
   * again once they pause for longer.
   */
  apply(edited: Edited, run = false): string | null {
    if ('refused' in edited) return edited.refused;
    if (edited.draft === this.plan) return null;
    this.history.push(edited.draft);
    this.take(edited.draft, run);
    return null;
  }

  /** The last edit undone; false if there was none. */
  undo(): boolean {
    const d = this.history.undo();
    if (d) this.take(d);
    return !!d;
  }

  redo(): boolean {
    const d = this.history.redo();
    if (d) this.take(d);
    return !!d;
  }

  /** Back to the town as it was laid, every edit dropped. */
  reset(): void {
    const base = this.history.reset();
    if (base !== this.plan) this.take(base);
    else this.hooks.changed();
  }

  /** The town settled from the plan as it stands, now (the timer not waited for); resolves once it is on screen. */
  async flush(): Promise<void> {
    this.clearTimer();
    // (A settle under way is waited for; then the plan, if it moved on meanwhile, settled too.)
    for (let k = 0; k < 4 && this.pending && !this.disposed; k++) await (this.running ?? this.run());
  }

  /** The town settled and on screen, and the Problems list checked, as the plan stands now (the way on to step 3 waits for both). */
  async ready(): Promise<void> {
    await this.flush();
    if (this.checkTimer !== null) {
      this.timers.clear(this.checkTimer);
      this.checkTimer = null;
    }
    if (!this.checked) await (this.checking ?? this.runChecks());
  }

  /** The session let go: no settle starts after this, and one under way is dropped. */
  dispose(): void {
    this.disposed = true;
    this.clearTimer();
    if (this.checkTimer !== null) this.timers.clear(this.checkTimer);
    this.control?.abort();
  }

  private take(draft: NeighbourhoodDraft, run = false): void {
    this.plan = draft;
    this.ops++;
    this.taken.push(draft);
    this.lastEdit = this.now();
    this.hooks.changed();
    this.recheck(false);
    this.schedule(run ? this.quiet : this.quietOne);
  }

  /** The Problems list checked again: now (`now`), else shortly (so the edit's ghosts show first). A check under way takes the new plan up when it is done. */
  private recheck(now: boolean): void {
    if (!this.hooks.check || this.checking) return;
    if (this.checkTimer !== null) this.timers.clear(this.checkTimer);
    this.checkTimer = null;
    if (now) void this.runChecks();
    else
      this.checkTimer = this.timers.set(() => {
        this.checkTimer = null;
        void this.runChecks();
      }, CHECK_AFTER);
  }

  /** The plan checked, again while it moved on meanwhile (the town as it was laid first, once: its problems block nothing). */
  private runChecks(): Promise<void> {
    const check = this.hooks.check;
    if (!check) return Promise.resolve();
    if (this.checking) return this.checking;
    const work = (async () => {
      while (!this.disposed && this.checkedFor !== this.plan) {
        const plan = this.plan;
        try {
          this.baseIssues ??= await check(this.history.base);
          const issues = plan === this.history.base ? this.baseIssues : await check(plan, this.baseIssues);
          if (this.disposed) return;
          // (Shown even if the plan moved on meanwhile: the last known, until the next is.)
          this.issues = issues;
          this.checkedFor = plan;
          this.hooks.changed();
        } catch (err) {
          console.error('[town] checking the edited town failed:', err);
          this.checkedFor = plan;
          return;
        }
      }
    })();
    this.checking = work.finally(() => (this.checking = null));
    return this.checking;
  }

  private clearTimer(): void {
    if (this.timer !== null) this.timers.clear(this.timer);
    this.timer = null;
  }

  private schedule(ms: number): void {
    this.clearTimer();
    if (this.disposed) return;
    this.timer = this.timers.set(() => {
      this.timer = null;
      if (this.running) this.again = true;
      else void this.run();
    }, ms);
  }

  /** The plan settled and put on screen (one at a time: see `flush` and `schedule`). */
  private run(): Promise<void> {
    if (this.running) return this.running;
    if (!this.pending || this.disposed) return Promise.resolve();
    const plan = this.plan;
    const ops = this.ops;
    const steps = [this.settledFrom, ...this.taken];
    const since = this.lastEdit;
    const control = (this.control = new AbortController());
    this.settling = true;
    this.hooks.changed();
    const work = (async () => {
      const t0 = this.now();
      try {
        const s = await this.hooks.settle(plan, ops, control.signal, { steps, ...(this.marks ? { before: this.marks } : {}) });
        const t1 = this.now();
        if (this.disposed) return;
        // (An edit refused: the plan it was made on is the one settled; it and every edit after it go, those made meanwhile too.)
        const back = s.refused?.length ? steps[s.step ?? 0] : null;
        if (back) {
          this.history.revert(back);
          this.plan = back;
          this.again = false;
          this.hooks.changed();
          this.recheck(true);
        }
        await this.hooks.show(s, back ?? plan);
        if (this.disposed) return;
        this.settled = s.draft;
        this.settledFrom = back ?? plan;
        this.marks = s.marks;
        if (back) [this.ops, this.taken] = [0, []];
        else {
          this.ops -= ops;
          this.taken = this.taken.slice(ops);
        }
        const t2 = this.now();
        this.timings.push({ settle: t1 - t0, show: t2 - t1, fromEdit: t2 - since });
        for (const r of s.refused ?? []) this.hooks.report?.(r.why);
      } catch (err) {
        if (control.signal.aborted || (err instanceof Error && err.name === 'Superseded')) return;
        console.error('[town] settling the edited town failed:', err);
        this.hooks.report?.('The town could not be built again with your changes.');
        // (Not tried again until the next edit: the same plan would fail the same way.)
        this.again = false;
        this.settledFrom = plan;
        this.marks = undefined;
        this.ops -= ops;
        this.taken = this.taken.slice(ops);
      }
    })();
    this.running = work.finally(() => {
      this.running = null;
      this.settling = false;
      if (this.control === control) this.control = null;
      if (!this.disposed) {
        this.hooks.changed();
        if (this.again && this.pending) {
          this.again = false;
          void this.run();
        }
      }
    });
    return this.running;
  }
}
