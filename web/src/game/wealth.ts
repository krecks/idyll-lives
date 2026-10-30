/**
 * What households earn and have saved: a grown-up's job (a career track and a grade) and its
 * pay, the highest grade someone could have reached at their age, and a household's savings,
 * which say which homes it can buy (`price.ts`). A young person starts at the bottom; the older
 * someone is, the higher they may have climbed and the more they have put by.
 */

import type { CareerCategory, Content } from '../content/content';
import type { SimDraft } from './household';

type Rand = () => number;
type Track = CareerCategory['tracks'][number];

/** Hours a week someone between jobs is counted as having worked (at the first grade's pay). */
const IDLE_HOURS = 40;
/** Years of odd jobs someone between jobs has saved from (see `savedBy`). */
const IDLE_YEARS = 5;

/** The career track `career` (an id) and the field it belongs to. */
export function careerTrack(content: Content, career: string): { category: CareerCategory; track: Track } | null {
  for (const category of content.careerCategories) {
    const track = category.tracks.find((t) => t.id === career);
    if (track) return { category, track };
  }
  return null;
}

/** A job's title ("Junior Developer"); empty for an unknown track. */
export function jobTitle(content: Content, job: NonNullable<SimDraft['job']>): string {
  const titles = careerTrack(content, job.career)?.track.titles ?? [];
  return titles[Math.max(0, Math.min(titles.length - 1, job.level))] ?? '';
}

/** What a job pays a week ($): the grade's pay an hour, the career's and track's pay factors, the shift's hours and days (as sim-core `content.rs` `careers`). */
export function weeklyPay(content: Content, career: string, level: number): number {
  const found = careerTrack(content, career);
  if (!found) return 0;
  const { category, track } = found;
  const top = Math.min(track.titles.length, content.careerGrades.length) - 1;
  if (top < 0) return 0;
  const gi = Math.max(0, Math.min(top, Math.floor(level)));
  const grade = content.careerGrades[gi];
  // Pay climbs in a straight line from the field's rate at the entry grade to the track's factor at the top one.
  const step = ((track.payFactor ?? 1) - 1) * (gi / Math.max(1, track.titles.length - 1));
  const shift = track.shift ?? category.shift;
  if (!shift) return 0;
  const perShift = Math.round(grade.payPerHour * (category.payFactor ?? 1) * (1 + step) * shift.hours);
  return perShift * shift.days.length;
}

/** The age grown-ups start work at. */
const workFrom = (content: Content): number => content.adultStages[0]?.from ?? 18;

/** Years someone `age` years old has worked (none before they grew up, none after retiring). */
function yearsWorked(content: Content, age: number): number {
  return content.isAdult(age) ? Math.max(0, Math.min(age, content.retireAt) - workFrom(content)) : 0;
}

/** The highest grade (0 the first) someone `age` years old may have reached; -1 if they're too young to work. */
export function topLevelAt(content: Content, age: number): number {
  if (!content.isAdult(age)) return -1;
  const per = content.economy.savings?.yearsPerGrade ?? 3.5;
  const grades = Math.max(1, content.careerGrades.length);
  return Math.max(0, Math.min(grades - 1, Math.floor(yearsWorked(content, age) / per)));
}

/** A job for a settled grown-up of `age`: a career track and a grade at most `topLevelAt(age)`, mostly lower; none for a child. */
export function jobFor(content: Content, age: number, rand: Rand): SimDraft['job'] {
  const top = topLevelAt(content, age);
  const tracks = content.careerCategories.flatMap((c) => c.tracks);
  if (top < 0 || !tracks.length) return undefined;
  const track = tracks[Math.min(tracks.length - 1, Math.floor(rand() * tracks.length))];
  // Most climb part of the way: a few reach the top their years allow, some stay near the bottom.
  const level = Math.max(0, top - Math.floor(rand() ** 1.5 * (top + 1)));
  return { career: track.id, level: Math.min(level, track.titles.length - 1) };
}

/**
 * What one grown-up has put by ($): a share of what they earned each year they worked, climbing a
 * grade every `yearsPerGrade` years up to their job's. Someone between jobs has a few years of odd
 * jobs at the first grade behind them (`IDLE_YEARS`), however old: no position, no villa.
 */
function savedBy(content: Content, member: Pick<SimDraft, 'age' | 'job'>): number {
  const s = content.economy.savings;
  if (!s) return 0;
  const job = member.job;
  const years = job ? yearsWorked(content, member.age) : Math.min(IDLE_YEARS, yearsWorked(content, member.age));
  const idle = (content.careerGrades[0]?.payPerHour ?? 0) * IDLE_HOURS;
  let earned = 0;
  for (let y = 0; y < years; y++) {
    earned += job ? weeklyPay(content, job.career, Math.min(job.level, Math.floor(y / s.yearsPerGrade))) : idle;
  }
  return s.rate * 52 * earned;
}

/** What a household has saved ($), from its grown-ups' ages and jobs. */
export function savingsOf(content: Content, household: { members: readonly Pick<SimDraft, 'age' | 'job'>[] }): number {
  const s = content.economy.savings;
  if (!s) return content.economy.startingFunds;
  const total = s.base + household.members.reduce((sum, m) => sum + savedBy(content, m), 0);
  return Math.round(total / 100) * 100;
}

/**
 * The money a household that saved `savings` starts with once it has paid `price` for its home
 * (`emptyLot`: a new household starting on an empty lot, which has money to build with besides).
 */
export function householdFunds(content: Content, savings: number, price: number, emptyLot = false): number {
  return fundsAfter(content, savings, price) + (emptyLot ? (content.economy.emptyLotFunds ?? 0) : 0);
}

/** The money a household that saved `savings` has to spend once it has paid `price` for its home (the rest stays put by). */
export function fundsAfter(content: Content, savings: number, price: number): number {
  const keep = content.economy.savings?.keep ?? 0;
  return content.economy.startingFunds + Math.max(0, Math.min(keep, savings - price));
}
