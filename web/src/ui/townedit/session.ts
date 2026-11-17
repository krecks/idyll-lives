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
 *
 * An edit the settle refuses (a road too steep, say) is taken back, with every edit after it; what it
 * was and why stays for the editor to show (`refusals`): the road or lot red for `REFUSED_SHOW` ms
 * (`refusalGhosts`), a line under the tools (`refusalLine`) and an entry for the Problems list
 * (`refusalIssues`), these two until the next edit. A settle that fails outright takes back every
 * edit since the town on screen was settled, so nothing is left as a ghost over it; a town settled
 * well but not put on screen is still the one settled.
 */

import type { Edited } from '../../game/edit/edited';
import { TownHistory } from '../../game/edit/history';
import { withUses } from '../../game/edit/uses';
import type { Pt } from '../../game/roads';
import type { NeighbourhoodDraft } from '../../game/town';
import type { TownGhosts } from '../../render/types';
import type { Issue, TownIssues } from './problems';
import type { Batch, Settled } from './settle';
import { sentence } from './words';

/** How long edits pause (ms) after one made in a run (`apply`'s `run`) before the town is settled and built again with them. */
export const QUIET = 1200;
/** The same after an edit made on its own (a button, a drag let go, an undo): the town is built again soon after. */
export const QUIET_ONE = 250;
/** How long after an edit (ms) the Problems list is checked again (the ghosts first). */
export const CHECK_AFTER = 40;
/** How long (ms) what a settle refused stays red over the town. */
export const REFUSED_SHOW = 4000;
/** Why every edit since the last settle was taken back when the settle failed outright. */
export const SETTLE_FAILED = 'the town could not be built again with your changes, so they were taken back';

/**
 * An edit the settle took back (`landFollow.ts` `EditRefusal`), as the editor shows it: why, in words (a clause),
 * where (`at`: a road's points with its `width`, else a lot's or a building's corners; none if not known), and
 * until when (`now()` ms) it stays red.
 */
export interface SettleRefusal {
  why: string;
  at?: Pt[];
  width?: number;
  until: number;
}

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
  /** The town on screen kept, with `plan`'s zones, for a plan that changed its zones alone (no settle, nothing to build again). */
  kept?(settled: NeighbourhoodDraft, plan: NeighbourhoodDraft): void;
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

/** A refusal said: why, and that the change was taken back (the failure says so itself). */
const said = (why: string) => (why === SETTLE_FAILED ? sentence(why) : `${sentence(why)} The change was taken back.`);

/**
 * Whether `b` is `a` but for its zones (`open.uses`): every other part the same object, as an edit of the zones
 * alone (`uses.ts` `paintUses`) leaves them. What needs no settle.
 */
export function usesOnly(a: NeighbourhoodDraft, b: NeighbourhoodDraft): boolean {
  if (a === b) return false;
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]) as Set<keyof NeighbourhoodDraft>;
  for (const k of keys) if (k !== 'open' && a[k] !== b[k]) return false;
  if (a.open === b.open || !a.open || !b.open) return a.open === b.open;
  const open = new Set([...Object.keys(a.open), ...Object.keys(b.open)]) as Set<keyof NonNullable<NeighbourhoodDraft['open']>>;
  for (const k of open) if (k !== 'uses' && a.open[k] !== b.open[k]) return false;
  return true;
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
  /** What the last settle took back (see the top of this file); none once an edit is made. */
  refusals: readonly SettleRefusal[] = [];
  private refusalTimer: unknown = null;

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
    if (this.refusalTimer !== null) this.timers.clear(this.refusalTimer);
    this.control?.abort();
  }

  /** The line under the tools about what the last settle took back (the first, if several), as a sentence; null: nothing, or an edit made since. */
  get refusalLine(): string | null {
    const first = this.refusals[0];
    return first ? said(first.why) : null;
  }

  /** What the last settle took back, red over the town for `REFUSED_SHOW` ms after: a road as a `bad` road, a lot or building as a `bad` box. Null: nothing. */
  refusalGhosts(): TownGhosts | null {
    const now = this.now();
    const out: TownGhosts = { roads: [], lots: [], boxes: [] };
    for (const r of this.refusals) {
      if (now >= r.until || !r.at?.length) continue;
      if (r.width) out.roads.push({ points: r.at.map(([x, z]): [number, number] => [x, z]), width: r.width, tone: 'bad' });
      else {
        const xs = r.at.map((p) => p[0]);
        const zs = r.at.map((p) => p[1]);
        const [x, z] = [Math.min(...xs), Math.min(...zs)];
        out.lots.push({ x, z, w: Math.max(...xs) - x, d: Math.max(...zs) - z, tone: 'bad' });
      }
    }
    return out.roads.length || out.lots.length ? out : null;
  }

  /** What the last settle took back, as Problems list entries (where each is: its middle); none once an edit is made. */
  refusalIssues(): Issue[] {
    return this.refusals.map((r) => {
      const at = r.at?.length ? r.at[Math.floor(r.at.length / 2)] : null;
      const mid = at && r.at!.length === 2 && !r.width ? [(r.at![0][0] + r.at![1][0]) / 2, (r.at![0][1] + r.at![1][1]) / 2] : at;
      return { text: said(r.why), ...(mid ? { at: { x: mid[0], z: mid[1] } } : {}) };
    });
  }

  /** What a settle took back, kept for the editor to show (see `refusals`); the editor told again once the red is due to go. */
  private refused(list: readonly Omit<SettleRefusal, 'until'>[]): void {
    const until = this.now() + REFUSED_SHOW;
    this.refusals = list.map((r) => ({ ...r, until }));
    if (this.refusalTimer !== null) this.timers.clear(this.refusalTimer);
    this.refusalTimer = this.timers.set(() => {
      this.refusalTimer = null;
      if (!this.disposed) this.hooks.changed();
    }, REFUSED_SHOW);
  }

  private take(draft: NeighbourhoodDraft, run = false): void {
    this.refusals = [];
    this.plan = draft;
    this.lastEdit = this.now();
    // A plan that differs from the one on screen by its zones alone (a stroke of the zone brush, its undo or
    // redo: `uses.ts` `needsSettle`) needs no settle: the town on screen takes its zones (`withUses`), and
    // nothing is asked of the worker. Not while a settle runs: its town is about to replace the one on screen.
    if (!this.running && usesOnly(this.settledFrom, draft)) {
      this.clearTimer();
      [this.ops, this.taken] = [0, []];
      this.settled = withUses(this.settled, draft);
      this.settledFrom = draft;
      this.hooks.kept?.(this.settled, draft);
      this.hooks.changed();
      this.recheck(false);
      return;
    }
    this.ops++;
    this.taken.push(draft);
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
          this.refused(s.refused!.map(({ why, at, width }) => ({ why, ...(at ? { at } : {}), ...(width ? { width } : {}) })));
          this.hooks.changed();
          this.recheck(true);
        }
        // (Settled well but not shown: still the town settled, so its ghosts don't stay over it for good.)
        try {
          await this.hooks.show(s, back ?? plan);
        } catch (err) {
          if (control.signal.aborted || (err instanceof Error && err.name === 'Superseded')) throw err;
          console.error('[town] showing the settled town failed:', err);
        }
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
        if (this.disposed) return;
        this.hooks.report?.(SETTLE_FAILED);
        // (Every edit since the town on screen was settled taken back, those made meanwhile too: the
        // plan is the town on screen's again, so no ghost of them stays over it, and nothing is tried
        // again until the next edit.)
        this.again = false;
        const back = this.settledFrom;
        this.history.revert(back);
        this.plan = back;
        [this.ops, this.taken] = [0, []];
        this.refused([{ why: SETTLE_FAILED }]);
        this.hooks.changed();
        this.recheck(true);
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
