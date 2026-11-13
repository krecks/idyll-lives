/**
 * "Today in <town>": the household's next events, from what the views already say
 * about them: a shift (`JobView`'s days and start), school (the content's school days, for pupils), and a planned
 * visit (a routine block of a visiting activity, their own or the household's they keep). Birthdays aren't in
 * the views (only the age in whole years is), so they aren't here.
 */

import type { JobView, PlanView, Routine } from '../../core/protocol';

export type TodayKind = 'shift' | 'school' | 'visit';

export interface TodayEvent {
  sim: number;
  name: string;
  kind: TodayKind;
  /** Days from today (0: today) and the minute of that day it starts. */
  inDays: number;
  minute: number;
  /** Minutes from now. */
  wait: number;
}

/** A member of the household as `nextEvents` needs them. */
export interface TodayMember {
  id: number;
  name: string;
  job: Pick<JobView, 'days' | 'startHour'> | null;
  /** Goes to school (their life stage's `school`). */
  pupil: boolean;
  plan?: Pick<PlanView, 'routines' | 'skipHousehold'>;
}

export interface TodayRules {
  school: { start: number; days: readonly number[] } | null;
  /** Activities that are visits (content `activities`' `visit`). */
  visits: ReadonlySet<string>;
  /** The household's routine template (members keep it unless they skip a block). */
  householdRoutines: readonly Routine[];
}

const DAY = 24 * 60;

/** The household's events from now on, soonest first, up to a week ahead (one of each kind per member). */
export function nextEvents(members: readonly TodayMember[], rules: TodayRules, minute: number, weekday: number): TodayEvent[] {
  const out: TodayEvent[] = [];
  /** The first day from today (0) a weekday of `on` comes round at `start`, still to come. */
  const next = (on: (day: number) => boolean, start: number): { inDays: number; minute: number } | null => {
    for (let k = 0; k <= 7; k++) {
      if (!on((weekday + k) % 7)) continue;
      if (k === 0 && start <= minute) continue;
      return { inDays: k, minute: start };
    }
    return null;
  };
  const push = (m: TodayMember, kind: TodayKind, at: { inDays: number; minute: number } | null) => {
    if (at) out.push({ sim: m.id, name: m.name, kind, ...at, wait: at.inDays * DAY + at.minute - minute });
  };
  for (const m of members) {
    if (m.job) push(m, 'shift', next((d) => m.job!.days.includes(d), Math.round(m.job.startHour * 60)));
    else if (m.pupil && rules.school) {
      const days = rules.school.days;
      push(m, 'school', next((d) => days.includes(d), Math.round(rules.school.start * 60)));
    }
    if (m.plan) {
      const skip = new Set(m.plan.skipHousehold);
      const blocks = [...m.plan.routines, ...rules.householdRoutines.filter((r) => !skip.has(r.id))].filter((r) => rules.visits.has(r.activity));
      let best: { inDays: number; minute: number } | null = null;
      for (const r of blocks) {
        const at = next((d) => ((r.days >> d) & 1) === 1, r.start);
        if (at && (!best || at.inDays * DAY + at.minute < best.inDays * DAY + best.minute)) best = at;
      }
      push(m, 'visit', best);
    }
  }
  return out.sort((a, b) => a.wait - b.wait || a.sim - b.sim);
}

/** "in 40 min", "in 2 h 10", "tomorrow", "on Thu". */
export function waitText(e: Pick<TodayEvent, 'wait' | 'inDays'>, weekday: number, days: readonly string[]): string {
  if (e.inDays === 1 && e.wait >= 6 * 60) return 'tomorrow';
  if (e.inDays > 1) return `on ${days[(weekday + e.inDays) % 7]}`;
  if (e.wait < 60) return `in ${Math.max(1, Math.round(e.wait))} min`;
  const h = Math.floor(e.wait / 60);
  const m = Math.round(e.wait % 60);
  return m ? `in ${h} h ${String(m).padStart(2, '0')}` : `in ${h} h`;
}

/** What it is, in a word or two. */
export function eventText(kind: TodayKind): string {
  return kind === 'shift' ? 'work' : kind === 'school' ? 'school' : 'a visit';
}
