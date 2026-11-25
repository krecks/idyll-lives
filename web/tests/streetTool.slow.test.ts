// Step 2's Street tool's live checks (`ui/townedit/streetLive.ts`) on a generated river world: every live
// verdict is `drawStreet`'s, within the per-move budget. (The tool on a hand-made town: `streetTool.test.ts`.)
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { Edited } from '../src/game/edit/edited';
import type { Pt } from '../src/game/roads';
import { liveStreet, unfinished, verdictOf } from '../src/ui/townedit/streetLive';

const refusal = (e: Edited) => ('refused' in e ? e.refused : null);

// On a generated river world: random strokes in and round the main town, each judged by the live
// checks and by drawStreet. Every reason the live checks give (the slope is not one) is
// drawStreet's own; drawStreet refuses nothing the live checks pass but what they leave to it.
describe('on a generated world', async () => {
  usePublicFetch();
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { mulberry32 } = await import('../src/game/noise');
  const { townIssues } = await import('../src/ui/townedit/problems');
  const real = await loadContent();
  useTownContent(real);
  const templates = await templatesOf(real);
  const world = generateWorld(real, templates, 'river', 4242, undefined, undefined, { town: 'village', villages: 'few' });

  test('every live verdict is drawStreet’s, within the per-move budget', () => {
    const a0 = world.open!.areas[0];
    const rand = mulberry32(9);
    let [agreed, refused, bridged, cut, checked] = [0, 0, 0, 0, 0];
    const left: string[] = [];
    const blocking: string[] = [];
    const base = townIssues(world, templates, real);
    const times: number[] = [];
    for (let k = 0; k < 400; k++) {
      const pts: Pt[] = [];
      let [x, z] = [a0.x - 60 + rand() * (a0.w + 120), a0.z - 60 + rand() * (a0.d + 120)];
      let way = rand() * Math.PI * 2;
      for (let i = 2 + Math.floor(rand() * 3); i > 0; i--) {
        pts.push([x, z]);
        way += (rand() - 0.5) * 1.6;
        const l = 20 + rand() * 80;
        [x, z] = [x + Math.cos(way) * l, z + Math.sin(way) * l];
      }
      const kind = rand() < 0.15 ? 'trail' : 'street';
      const t0 = performance.now();
      const live = liveStreet(world, templates, pts, { kind });
      times.push(performance.now() - t0);
      const verdict = verdictOf(world, templates, real, pts, { kind, clear: true }, 1);
      const why = refusal(verdict);
      const liveWhy = live.why;
      // (What it makes blocks nothing in the Problems list the town didn't have: every street bridged or cutting a footpath, and one in eight of the rest.)
      const crossed = 'draft' in verdict && verdict.draft.roads.some((r, i) => r.kind === 'trail' && world.roads[i] !== r);
      if (crossed) cut++;
      if ('draft' in verdict && (k % 8 === 0 || crossed || verdict.draft.roads.some((r, i) => r.bridge && world.roads[i] !== r))) {
        checked++;
        const fresh = townIssues(verdict.draft, templates, real, base).blocking.map((i) => i.text);
        if (fresh.length) blocking.push(`${JSON.stringify(pts.map(([x, z]) => [Math.round(x), Math.round(z)]))} ${kind}: ${fresh.join('; ')}`);
      }
      if (live.bridges.length) bridged++;
      if (liveWhy) {
        refused++;
        expect(why).toBe(liveWhy);
        agreed++;
      } else if (why && !unfinished(why)) left.push(why);
      else agreed++;
    }
    times.sort((a, b) => a - b);
    const median = times[times.length >> 1];
    const p95 = times[Math.floor(times.length * 0.95)];
    console.log(`streetTool: ${agreed} of 400 strokes judged alike (${refused} refused alike, ${bridged} bridged, ${cut} cutting a footpath; ${checked} drawn checked for problems); left to drawStreet: ${left.length ? left.join('; ') : 'none'}; live checks ${median.toFixed(2)} ms median, ${p95.toFixed(2)} ms p95`);
    // (What the live checks leave to drawStreet: the network.)
    for (const why of left) expect(why).toMatch(/joins none/);
    expect(agreed + left.length).toBe(400);
    expect(blocking).toEqual([]);
    expect(median).toBeLessThan(5);
  });
});
