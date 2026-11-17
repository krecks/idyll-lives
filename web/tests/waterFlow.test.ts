// The water's flow as the renderer draws it (`render/shared/waterFlow.ts`): a generated river runs along its course,
// one way all along; a drawn river along its drag; a still lake and the sea have no way and no speed; water stepping
// down runs from the higher level to the lower, fast and foaming at the step; narrow water runs faster than wide and
// slows at the shore; a rectangle worked out on its own is the same as the whole land's there. `bun test tests/waterFlow.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { blankLand, LAND_CELL, NONE, SEA_CM, wetAt, type Land } from '../src/game/land';
import { LandEditor } from '../src/game/landEdit';
import { flowTexels, riverCourse, waterFlow } from '../src/render/shared/waterFlow';

const WEB = new URL('../', import.meta.url).pathname;
const FREE = () => 1;
const at = (land: Land, x: number, z: number) => Math.round(z / LAND_CELL) * land.cols + Math.round(x / LAND_CELL);
const line = (a: [number, number], b: [number, number], n = 40): [number, number][] => Array.from({ length: n + 1 }, (_, k) => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);

/** A `size` m land at `height` m everywhere. */
function flat(size: number, height = 3): Land {
  const land = blankLand(size, size);
  land.h.fill(height * 100);
  return land;
}

describe('a generated river', async () => {
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { useTownContent } = await import('../src/game/town');
  const { landDraft } = await import('../src/game/macro');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  test('runs along its course, one way all along, slowing at the banks', () => {
    for (const seed of [4242, 77]) {
      const draft = landDraft(content, t, 'river', { size: 'small', hills: 'rolling' }, seed);
      const land = draft.open!.land!;
      // Its course: the middle of the water zone's two banks (west to east, then back).
      const poly = draft.open!.zones.find((z) => z.kind === 'water')!.poly;
      const half = poly.length / 2;
      const middle = (x: number) => {
        const s = Math.max(0, Math.min(half - 2, Math.floor((x - poly[0][0]) / (poly[1][0] - poly[0][0]))));
        const zAt = (q: number) => (poly[q][1] + poly[poly.length - 1 - q][1]) / 2;
        return [zAt(s), zAt(s + 1), poly[s + 1][0] - poly[s][0]] as const;
      };
      const course = riverCourse(land);
      expect(course).not.toBeNull();
      const flow = waterFlow(land, course);
      let [along, n, east, west, fastest, bank] = [0, 0, 0, 0, 0, 1];
      for (let j = 0; j < land.rows; j++) {
        for (let i = 8; i < land.cols - 8; i++) {
          const k = j * land.cols + i;
          if (!wetAt(land, i, j) || land.wl[k] !== SEA_CM) continue;
          const [z0, z1, dx] = middle(i * LAND_CELL);
          const l = Math.hypot(dx, z1 - z0);
          along += Math.abs(flow.wx[k] * (dx / l) + flow.wz[k] * ((z1 - z0) / l));
          n++;
          if (flow.wx[k] > 0) east++;
          else west++;
          fastest = Math.max(fastest, flow.speed[k]);
          // (At the bank: a sample with dry ground beside it.)
          if (!wetAt(land, i, j - 1) || !wetAt(land, i, j + 1)) bank = Math.min(bank, flow.speed[k] / Math.max(1e-6, flow.speed[k - land.cols * 3] || flow.speed[k + land.cols * 3]));
        }
      }
      expect(n).toBeGreaterThan(500);
      expect(along / n).toBeGreaterThan(0.97);
      expect(Math.min(east, west)).toBe(0);
      expect(fastest).toBeGreaterThan(0.5);
      expect(bank).toBeLessThan(0.6);
    }
  });
});

describe('still water', () => {
  test('a lake and the sea have no way, no speed and no foam', () => {
    const land = flat(256);
    const editor = new LandEditor(land, FREE);
    editor.stroke({ tool: 'lake', radius: 30, strength: 0.6 }, [[128, 128]]);
    // The sea along the south edge.
    for (let j = land.rows - 20; j < land.rows; j++) for (let i = 0; i < land.cols; i++) [land.h[j * land.cols + i], land.wl[j * land.cols + i]] = [-200, SEA_CM];
    expect(riverCourse(land)).toBeNull();
    const flow = waterFlow(land, riverCourse(land));
    let wet = 0;
    for (let k = 0; k < land.h.length; k++) {
      if (land.wl[k] === NONE) continue;
      wet++;
      expect(flow.speed[k]).toBe(0);
      expect(flow.foam[k]).toBe(0);
      expect(flow.wx[k]).toBe(0);
      expect(flow.wz[k]).toBe(0);
    }
    expect(wet).toBeGreaterThan(2000);
  });
});

describe('a step', () => {
  // A channel 12 m wide along x: level 1.2 m west of x 100, 0.6 m east of it (no way drawn).
  const land = flat(256);
  for (let j = 0; j < land.rows; j++) {
    for (let i = 0; i < land.cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      if (Math.abs(z - 128) > 6 || x < 20 || x > 236) continue;
      const k = j * land.cols + i;
      land.wl[k] = x < 100 ? 120 : 60;
      land.h[k] = -50;
    }
  }
  const flow = waterFlow(land, null);

  test('foams and runs fast where the level drops, and runs from the higher water to the lower', () => {
    const k = at(land, 98, 128);
    expect(flow.foam[k]).toBeGreaterThan(0.9);
    expect(flow.speed[k]).toBeGreaterThan(0.9);
    expect(flow.wx[k]).toBeGreaterThan(0.95);
    // Upstream, the higher water runs towards the step; well away from it there is no foam.
    for (const x of [30, 60, 90]) expect(flow.wx[at(land, x, 128)]).toBeGreaterThan(0.9);
    expect(flow.foam[at(land, 60, 128)]).toBe(0);
    // Downstream, the lower water has nowhere lower to go: it lies still.
    expect(flow.speed[at(land, 180, 128)]).toBe(0);
  });

  test('the flow map: way times speed about 128, foam in blue', () => {
    const texels = new Uint8Array(land.cols * land.rows * 4);
    flowTexels(flow, { i0: 0, j0: 0, i1: land.cols, j1: land.rows }, texels);
    const k = at(land, 98, 128);
    expect(texels[k * 4]).toBeGreaterThan(240);
    expect(Math.abs(texels[k * 4 + 1] - 128)).toBeLessThan(12);
    expect(texels[k * 4 + 2]).toBeGreaterThan(230);
    const still = at(land, 180, 128);
    expect([texels[still * 4], texels[still * 4 + 1], texels[still * 4 + 2]]).toEqual([128, 128, 0]);
  });
});

describe('a drawn river', () => {
  test('runs along the drag, faster where narrow than where wide, slower at the shore', () => {
    const land = flat(512);
    const editor = new LandEditor(land, FREE);
    editor.stroke({ tool: 'river', radius: 4, strength: 0.4 }, line([20, 100], [490, 100], 160));
    editor.stroke({ tool: 'river', radius: 30, strength: 0.4 }, line([20, 380], [490, 380], 160));
    const flow = waterFlow(land, riverCourse(land));
    const [narrow, wide, shore] = [at(land, 250, 100), at(land, 250, 380), at(land, 250, 380 - 28)];
    expect(flow.wx[narrow]).toBeGreaterThan(0.99);
    expect(flow.wx[wide]).toBeGreaterThan(0.99);
    expect(flow.speed[narrow]).toBeGreaterThan(0.8);
    expect(flow.speed[wide]).toBeLessThan(flow.speed[narrow] * 0.6);
    expect(flow.speed[wide]).toBeGreaterThan(0.2);
    expect(flow.speed[shore]).toBeLessThan(flow.speed[wide] * 0.6);
  });

  test('a rectangle worked out on its own is the whole land worked out there', () => {
    const land = flat(256);
    const editor = new LandEditor(land, FREE);
    editor.stroke({ tool: 'river', radius: 5, strength: 0.4 }, [...line([20, 40], [200, 120], 80), ...line([200, 120], [60, 230], 80).slice(1)]);
    const whole = waterFlow(land, null);
    const rect = { i0: 40, j0: 30, i1: 90, j1: 80 };
    const part = waterFlow(land, null, rect);
    for (let j = 0; j < land.rows; j++) {
      for (let i = 0; i < land.cols; i++) {
        const k = j * land.cols + i;
        const inside = i >= rect.i0 && i < rect.i1 && j >= rect.j0 && j < rect.j1;
        if (!inside) {
          expect(part.speed[k]).toBe(0);
          continue;
        }
        expect(part.wx[k]).toBeCloseTo(whole.wx[k], 5);
        expect(part.wz[k]).toBeCloseTo(whole.wz[k], 5);
        expect(part.speed[k]).toBeCloseTo(whole.speed[k], 5);
        expect(part.foam[k]).toBeCloseTo(whole.foam[k], 5);
      }
    }
  });
});
