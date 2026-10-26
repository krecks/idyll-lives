// The gaits the residents play on their way (`render/shared/characters/gait.ts`, as `residents.ts` `choose` picks
// them): at each game speed, on each body at each life stage (height factor 1, the clips' own ground speeds and the
// stages' leg lengths from the built set), how much of the way the feet slide, the gait's cap falling short of the
// ground speed. `bun test`.
//
// (Written for A-residents-03: at game speed 3 the run kept grown-ups' and teens' feet on the ground but children's
// slid 11–14 %, their run capped at the grown-ups' four strides a second. The run's beat now quickens on shorter
// legs, `runMaxRate`: 0 at speed 3 on every stage. Speeds 4 and 5 keep their slide: won't fix.)
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { GAME_RATE } from '../src/render/shared/gameSpeed';
import { RUN_MAX_RATE } from '../src/render/shared/characters/actions';
import { gaitFor, gaitSlide } from '../src/render/shared/characters/gait';
import { locomotionClips } from '../src/render/shared/characters/clips/locomotion';

const WEB = new URL('../', import.meta.url).pathname;
/** Metres a Sim walks per second at game speed 1 (sim-core `world::WALK_SPEED`), and the fastest walkers' factor (traits and perks). */
const WALK = 1.5;
const FASTEST = 1.38;

describe('gaits', () => {
  const rig = JSON.parse(readFileSync(`${WEB}public/assets/characters/rig.json`, 'utf8')) as {
    bodies: Record<string, { stage: string; pelvisScale: number; stages: Record<string, { pelvisScale: number }> }>;
  };
  const speeds = new Map(locomotionClips().map((c) => [c.name, c.options.speed]));
  const speedOf = (key: string) => speeds.get(key);
  // (A body's own stage, the young adult, is the body itself.)
  const legs = Object.entries(rig.bodies).flatMap(([body, b]) =>
    [[b.stage, b] as const, ...Object.entries(b.stages)]
      .filter(([stage]) => stage !== 'baby')
      .map(([stage, s]) => ({ who: `${body} ${stage}`, legs: s.pelvisScale })),
  );

  test('every stage keeps its feet on the ground up to game speed 3', () => {
    const table: string[] = [];
    let worst = 0;
    for (const { who, legs: l } of legs) {
      const row = [1, 2, 3, 4, 5].map((s) => gaitSlide(WALK * GAME_RATE[s], l, speedOf));
      // (What the run's old cap, the same four strides a second on every leg, left.)
      const before = (v: number) => {
        const { gear, rate } = gaitFor(v, l, -1, speedOf);
        const capped = gear === 2 ? Math.min(rate, RUN_MAX_RATE) : rate;
        return Math.max(0, 1 - (capped * speedOf(['walk', 'jogAlong', 'runAlong'][gear])! * l) / v);
      };
      const at3 = WALK * GAME_RATE[3];
      const fastest = gaitSlide(at3 * FASTEST, l, speedOf);
      const pc = (x: number) => `${(x * 100).toFixed(0)} %`;
      table.push(`${who.padEnd(17)} ${row.map((x) => pc(x).padStart(5)).join(' ')}   speed 3 before ${pc(before(at3))}; the fastest ${pc(fastest)}, before ${pc(before(at3 * FASTEST))}`);
      worst = Math.max(worst, ...row.slice(0, 3));
    }
    if (process.env.GAIT_TABLE) console.log(table.join('\n'));
    expect(worst, `foot slide at speeds 1-5:\n${table.join('\n')}`).toBeLessThan(1e-6);
  });

  test("a child's run takes more strides a second than a grown-up's, and no more than it needs at speed 3", () => {
    const child = Math.min(...legs.filter((x) => x.who.endsWith('child')).map((x) => x.legs));
    const { gear, rate } = gaitFor(WALK * GAME_RATE[3], child, -1, speedOf);
    expect(gear).toBe(2);
    expect(rate).toBeGreaterThan(RUN_MAX_RATE);
    // (The cap scales as the square root of the legs: under 5.5 strides a second on the shortest.)
    expect(gaitFor(1000, child, 2, speedOf).rate).toBeLessThan(5.5);
    expect(gaitFor(1000, 1.1, 2, speedOf).rate).toBe(RUN_MAX_RATE);
  });
});
