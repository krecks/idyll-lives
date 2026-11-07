// Doors and gates open for the one going through them (`render/shared/doors.ts`): who a leaf swings
// for, and when. `bun test`.
//
// (Written for A-houses-01, doors by hand: the leaf used to start a fixed 1.5 m ahead, which at game
// speed 3 (10x) left it a quarter open as the walker reached it — they walked through the leaf. The
// lead now follows the game's rate, and the resident's hand goes out with the leaf, so the door opens
// as the hand reaches the handle.)
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { PLAYBACK_RATE } from '../src/render/shared/gameSpeed';
import { DOOR_SWING, MOVER, PACE, doorLead, goingThrough } from '../src/render/shared/doors';

/**
 * Walkers at (x, z) heading for (wx, wz), as the renderer packs them: where they are, the point `ahead` m on
 * towards it, their row.
 */
function movers(...who: [number, number, number, number, number][]): number[] {
  const out: number[] = [];
  who.forEach(([x, z, wx, wz, ahead], i) => {
    const len = Math.hypot(wx - x, wz - z) || 1;
    out.push(x, z, x + ((wx - x) / len) * ahead, z + ((wz - z) / len) * ahead, i);
  });
  return out;
}

describe('doors', () => {
  const door: [number, number] = [0, 0];
  const lead = doorLead(1);

  test('the pace is how fast sim-core walks them (`world::WALK_SPEED`, m a real second), and the residents step it', () => {
    const sim = (file: string) => readFileSync(new URL(`../../crates/sim-core/src/${file}`, import.meta.url), 'utf8');
    // (A tick's worth of it, as residents.ts works out how far a walker gets between snapshots: PACE / ticksPerSecond.)
    const walk = /const WALK_SPEED: f32 = ([\d.]+) \/ TICKS_PER_SECOND as f32;/.exec(sim('world.rs'));
    expect(walk).not.toBeNull();
    expect(PACE).toBe(Number(walk![1]));
  });

  test('a hand reaches the leaf: the lead is a hand at speed 1, and the swing fits in it', () => {
    expect(lead).toBeCloseTo(0.75, 6);
    // A walker at the ordinary pace: the leaf is open by the time they are.
    expect(lead / PACE).toBeGreaterThanOrEqual(DOOR_SWING);
    // The quickest (traits and perks, 1.38x) push it open as they go through: four fifths of the way
    // by the time they arrive, which is what a hand on a swinging door does.
    const fast = lead / (PACE * 1.38);
    expect(fast / DOOR_SWING).toBeGreaterThan(0.75);
  });

  test('at every game speed the swing finishes before the walker arrives', () => {
    for (const s of [1, 2, 3, 4, 5]) {
      const rate = PLAYBACK_RATE[s];
      const l = doorLead(rate);
      const arrive = l / (PACE * rate);
      // In the sim's real time the leaf needs DOOR_SWING; the renderer scales its step by the rate, up to 10x.
      const swing = DOOR_SWING / Math.min(Math.max(rate, 1), 10);
      expect(swing).toBeLessThanOrEqual(arrive + 1e-9);
      // And never opens further off than the swing needs, nor nearer than the hand.
      expect(l).toBeCloseTo(Math.max(0.75, PACE * rate * DOOR_SWING), 6);
    }
  });

  test('the leaf swings for one going through it, not for one walking past', () => {
    // Straight through the middle of a doorway in a wall along x (z = 0), from a hand's reach short.
    expect(goingThrough(movers([0, -0.7, 0, 2, 1.5]), 'h', door, false, lead)).toBe(1);
    expect(goingThrough(movers([0, 0.7, 0, -2, 1.5]), 'h', door, false, lead)).toBe(-1);
    // Past it, 2 m to the side.
    expect(goingThrough(movers([2, -0.7, 2, 2, 1.5]), 'h', door, false, lead)).toBe(0);
    // Up to it and away again (someone waiting outside).
    expect(goingThrough(movers([0, -2.5, 0, -1.2, 1.5]), 'h', door, false, lead)).toBe(0);
    // Still a stride and more off: not yet (it opens at the hand, `doorLead`).
    expect(goingThrough(movers([0, -1.4, 0, 2, 1.5]), 'h', door, false, lead)).toBe(0);
    expect(goingThrough(movers([0, -1.4, 0, 2, 1.5]), 'h', door, false, doorLead(3))).toBe(1);
  });

  test('the hand comes out as the leaf starts, and is at the handle when they are', () => {
    const hand = new Float32Array(2);
    // 2 m short: nothing out.
    goingThrough(movers([0, -2, 0, 2, 1.5]), 'h', door, false, lead, hand);
    expect(hand[0]).toBe(0);
    // Just started (the leaf's first frame): out a little.
    hand.fill(0);
    goingThrough(movers([0, -0.74, 0, 2, 1.5]), 'h', door, false, lead, hand);
    expect(hand[0]).toBeLessThan(0.1);
    // A hand's reach short: out.
    hand.fill(0);
    goingThrough(movers([0, -0.6, 0, 2, 1.5]), 'h', door, false, lead, hand);
    expect(hand[0]).toBeGreaterThan(0.2);
    // In the doorway: out all the way.
    hand.fill(0);
    goingThrough(movers([0, -0.1, 0, 2, 1.5]), 'h', door, false, lead, hand);
    expect(hand[0]).toBe(1);
    // Through and away: down again.
    hand.fill(0);
    goingThrough(movers([0, 0.6, 0, 2, 1.5]), 'h', door, false, lead, hand);
    expect(hand[0]).toBeLessThan(0.5);
  });

  test('a door that stands open keeps clear of anyone still walking near it', () => {
    // Open, and someone on the move 0.9 m short of it: it stays open (the leaf would sweep them).
    expect(goingThrough(movers([0, -0.9, 0, -3, 1.5]), 'h', door, true, lead)).not.toBe(0);
    // Nobody near: shut.
    expect(goingThrough(movers([0, -3, 0, -5, 1.5]), 'h', door, true, lead)).toBe(0);
  });

  test('a doorway in a wall along z works the same, across x', () => {
    expect(goingThrough(movers([-0.7, 0, 2, 0, 1.5]), 'v', door, false, lead)).toBe(1);
    expect(goingThrough(movers([0.7, 0, -2, 0, 1.5]), 'v', door, false, lead)).toBe(-1);
  });

  test('the movers are packed as the renderer packs them', () => {
    expect(movers([1, 2, 3, 4, 1.5]).length).toBe(MOVER);
  });
});
