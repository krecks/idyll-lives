/**
 * The background stream's order (`assets/stream.ts`): in the order asked, until the game ranks the
 * waiting files by where the camera is (`reprioritize`); files it doesn't rank come last.
 */
import { describe, expect, test } from 'bun:test';
import { background, foreground, reprioritize } from '../src/assets/stream';

describe('streaming by priority', () => {
  test('waiting files start by rank, the unranked last, ties in the order asked', async () => {
    const started: string[] = [];
    let release!: () => void;
    // A load someone waits for holds the background back while the queue fills.
    const held = foreground(() => new Promise<void>((r) => (release = r)));
    const loads = ['far', 'near', 'unknown', 'focus', 'near2'].map((url) => background(async () => void started.push(url), url));
    const rank: Record<string, number> = { far: 2.5, near: 1.1, focus: 0, near2: 1.1 };
    reprioritize((url) => rank[url] ?? Infinity);
    release();
    await held;
    await Promise.all(loads);
    reprioritize(null);
    expect(started).toEqual(['focus', 'near', 'near2', 'far', 'unknown']);
  });
});
