/**
 * The game's speed and what holds time still (`game/speed.ts`): loading, the menus, Build and
 * Create share one hold, and the player's speed comes back when nothing holds it.
 */
import { describe, expect, test } from 'bun:test';
import { GameSpeed } from '../src/game/speed';

/** A speed (for a game saved at `saved`), and what it sent the simulation. */
function speed(saved?: number): { speed: GameSpeed; sent: number[] } {
  const sent: number[] = [];
  return { speed: new GameSpeed((s) => sent.push(s), saved), sent };
}

describe('game speed', () => {
  test('loading holds time; the game starts at the speed chosen for it', () => {
    const { speed: s, sent } = speed();
    expect(s.running).toBe(0);
    s.choose(3);
    expect(sent).toEqual([]);
    s.hold('loading', false);
    expect(sent).toEqual([3]);
  });

  test('a window left while loading (pausing on blur) keeps time still, then gives the start speed back', () => {
    const { speed: s, sent } = speed();
    s.hold('menu', true);
    s.choose(2);
    s.hold('loading', false);
    expect(sent).toEqual([]);
    s.hold('menu', false);
    expect(sent).toEqual([2]);
  });

  test('Build under the pause menu: leaving Build does not run the world, closing the menu does', () => {
    const { speed: s, sent } = speed();
    s.choose(3);
    s.hold('loading', false);
    s.hold('build', true);
    s.hold('menu', true);
    s.hold('build', false);
    expect(s.running).toBe(0);
    s.hold('menu', false);
    expect(s.running).toBe(3);
    expect(sent).toEqual([3, 0, 3]);
  });

  test('Live does not come back stuck at 0 after a menu opened in Build', () => {
    const { speed: s } = speed();
    s.choose(2);
    s.hold('loading', false);
    s.hold('build', true);
    s.hold('menu', true);
    s.hold('menu', false);
    expect(s.running).toBe(0);
    s.hold('build', false);
    expect(s.running).toBe(2);
  });

  test('the planner holds time; a speed picked meanwhile is the one given back', () => {
    const { speed: s, sent } = speed();
    s.choose(1);
    s.hold('loading', false);
    s.hold('menu', true);
    s.choose(4);
    expect(s.running).toBe(0);
    s.hold('menu', false);
    expect(sent).toEqual([1, 0, 4]);
  });

  test('a hold set twice, or let go of twice, sends nothing more', () => {
    const { speed: s, sent } = speed();
    s.choose(2);
    s.hold('loading', false);
    s.hold('menu', true);
    s.hold('menu', true);
    s.hold('build', false);
    s.hold('menu', false);
    s.hold('menu', false);
    expect(sent).toEqual([2, 0, 2]);
  });

  test('a loaded game resumes at the speed it was saved at, held still until it is shown', () => {
    const { speed: s, sent } = speed(3);
    expect(s.running).toBe(0);
    s.hold('loading', false);
    expect(sent).toEqual([3]);
    s.choose(0);
    expect(s.last).toBe(3);
  });

  test('a game saved paused (under a menu, in Build) resumes at 1; a new one starts at 1', () => {
    const paused = speed(0);
    paused.speed.hold('loading', false);
    expect(paused.sent).toEqual([1]);
    const fresh = speed();
    fresh.speed.hold('loading', false);
    expect(fresh.sent).toEqual([1]);
  });

  test('pausing keeps the last speed to resume at', () => {
    const { speed: s } = speed();
    s.choose(4);
    s.choose(0);
    expect(s.chosen).toBe(0);
    expect(s.last).toBe(4);
  });

  test('super speed starting and ending on its own: the speed shown follows, nothing is sent back', () => {
    const { speed: s, sent } = speed();
    s.choose(2);
    s.hold('loading', false);
    // (A view from just after the speed was sent is from before it: not followed.)
    s.follow(4, performance.now());
    expect(s.chosen).toBe(2);
    s.follow(4, performance.now() + 1000);
    expect(s.chosen).toBe(4);
    s.follow(2, performance.now() + 2000);
    expect(s.chosen).toBe(2);
    expect(sent).toEqual([2]);
  });

  test('super speed picked while the simulation is at it by itself is sent, so it stays the player\'s', () => {
    const { speed: s, sent } = speed();
    s.choose(1);
    s.hold('loading', false);
    s.follow(4, performance.now() + 1000);
    expect(s.chosen).toBe(4);
    expect(sent).toEqual([1]);
    s.choose(4);
    expect(sent).toEqual([1, 4]);
    // (Under a hold it waits, as any pick does; another speed picked again is not sent again.)
    s.hold('menu', true);
    s.choose(4);
    expect(sent).toEqual([1, 4, 0]);
    s.hold('menu', false);
    s.choose(2);
    s.choose(2);
    expect(sent).toEqual([1, 4, 0, 4, 2]);
  });

  test("whose super speed it is: the player's pick, or the simulation's own", () => {
    const { speed: s } = speed();
    s.choose(2);
    s.hold('loading', false);
    s.follow(4, performance.now() + 1000);
    expect(s.superPicked).toBe(false);
    s.choose(4);
    expect(s.superPicked).toBe(true);
    s.choose(1);
    expect(s.superPicked).toBe(false);
  });

  test("only super speed's comings and goings are followed, and not under a hold", () => {
    const { speed: s } = speed();
    s.choose(3);
    s.hold('loading', false);
    const later = performance.now() + 1000;
    // (Another speed than the player's: a view from before their pick.)
    s.follow(1, later);
    expect(s.chosen).toBe(3);
    s.hold('menu', true);
    s.follow(4, later);
    expect(s.chosen).toBe(3);
    s.hold('menu', false);
    s.follow(4, performance.now() + 1000);
    expect(s.chosen).toBe(4);
    // Pausing at super speed resumes at it.
    s.choose(0);
    expect(s.last).toBe(4);
  });
});
