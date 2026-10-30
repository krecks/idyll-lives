// Households' wealth (`game/wealth.ts`) with the shipped content: the grade someone may have reached by
// their age, the jobs neighbours are given, what a job pays (as sim-core works it out) and what a
// household has saved, which says which homes it can buy.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('wealth', async () => {
  // The loaders fetch from the page's address: here, files under web/public.
  const realFetch = globalThis.fetch;
  const realLocation = (globalThis as { location?: unknown }).location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url);
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { jobFor, savingsOf, topLevelAt, weeklyPay } = await import('../src/game/wealth');
  const content = await Content.load('http://localhost/content/base.json');
  const tracks = content.careerCategories.flatMap((c) => c.tracks);

  test('the highest grade rises with age and is none for a child', () => {
    expect(topLevelAt(content, 10)).toBe(-1);
    expect(topLevelAt(content, 18)).toBe(0);
    let last = 0;
    for (let age = 18; age <= 90; age++) {
      const top = topLevelAt(content, age);
      expect(top).toBeGreaterThanOrEqual(last);
      expect(top).toBeLessThan(content.careerGrades.length);
      last = top;
    }
    expect(topLevelAt(content, 50)).toBeGreaterThan(topLevelAt(content, 25));
    // Retired: the grade they had reached when they stopped.
    expect(topLevelAt(content, 80)).toBe(topLevelAt(content, content.retireAt));
  });

  test('a job is never above what the age allows, and a child has none', () => {
    let seed = 7;
    const rand = () => ((seed = (seed * 16807) % 2147483647) - 1) / 2147483646;
    expect(jobFor(content, 12, rand)).toBeUndefined();
    for (let i = 0; i < 2000; i++) {
      const age = 18 + Math.floor(rand() * 60);
      const job = jobFor(content, age, rand)!;
      expect(tracks.some((t) => t.id === job.career)).toBe(true);
      expect(job.level).toBeGreaterThanOrEqual(0);
      expect(job.level).toBeLessThanOrEqual(topLevelAt(content, age));
    }
  });

  test('pay is worked out as sim-core does', () => {
    // it.software: the IT field pays 1.25, 8 hours on 5 days; the track rises to 1.18 at the top grade.
    const category = content.careerCategories.find((c) => c.id === 'it')!;
    const track = category.tracks.find((t) => t.id === 'it.software')!;
    const shift = track.shift ?? category.shift!;
    for (const level of [0, 3, 9]) {
      const grade = content.careerGrades[level];
      const step = ((track.payFactor ?? 1) - 1) * (level / (track.titles.length - 1));
      const perShift = Math.round(grade.payPerHour * (category.payFactor ?? 1) * (1 + step) * shift.hours);
      expect(weeklyPay(content, 'it.software', level)).toBe(perShift * shift.days.length);
    }
    // Grade D: $20 an hour × 1.25 × (1 + 0.18 × 3/9) × 8 hours = $212 a day, five days.
    expect(weeklyPay(content, 'it.software', 3)).toBe(1060);
  });

  test('a young starter cannot buy a villa; a manager of fifty can', () => {
    const young = savingsOf(content, { members: [{ age: 22, job: { career: 'law.corporate', level: 0 } }] });
    const manager = savingsOf(content, { members: [{ age: 50, job: { career: 'law.corporate', level: 7 } }] });
    expect(young).toBeLessThan(200_000);
    expect(manager).toBeGreaterThanOrEqual(200_000);
    // Nobody on an entry job at 22 has saved $200k, whatever the field.
    for (const t of tracks) expect(savingsOf(content, { members: [{ age: 22, job: { career: t.id, level: 0 } }] })).toBeLessThan(200_000);
    // Children add nothing; someone between jobs saved a little.
    const base = content.economy.savings!.base;
    expect(savingsOf(content, { members: [{ age: 9 }] })).toBe(base);
    expect(savingsOf(content, { members: [{ age: 30 }] })).toBeGreaterThan(base);
  });
});
