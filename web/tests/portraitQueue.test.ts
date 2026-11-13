// The portraits' queue (`render/preview/portraits.ts`): what's on screen is drawn first, a request whose face has
// gone isn't drawn for nobody, and a batch's Sims are put on their rows together (one build of the studio's
// residents, not one per row: posing them two a frame built them all again each time). On a stand-in studio: no GPU.
import { describe, expect, mock, test } from 'bun:test';
import type { SimLook } from '../src/render/types';

mock.module('../src/debug/perf', () => ({ perfStart: (name: string) => ({ name, start: 0, ms: 0 }), perfEnd: () => 0 }));
mock.module('../src/render/pc/shaders/softLook', () => ({ shadingFromSettings: () => 'soft' }));
mock.module('../src/render/preview/studio', () => ({ Studio: { aim: () => {} } }));
const { lookKey, Portraits } = await import('../src/render/preview/portraits');

type Rows = readonly (readonly [number, { look: SimLook } | null])[];

function studio() {
  const built: Rows[] = [];
  const fake = {
    built,
    setRows: (rows: Rows) => built.push(rows),
    characters: { heads: new Float32Array(300 * 3), visible: new Uint8Array(300), directions: [] as unknown[] },
  };
  return fake;
}

const look = (n: number): SimLook => ({ gender: 'female', appearance: { skin: `#${String(n).padStart(6, '0')}`, top: 'top.tee', face: 'face.a' } as unknown as SimLook['appearance'] });
const ask = (p: InstanceType<typeof Portraits>, n: number, priority?: 'now' | 'idle', signal?: AbortSignal) => p.request(lookKey(look(n)), look(n), priority, signal);
/** The looks a batch put on its rows, by their skin's number. */
const drawn = (rows: Rows) => rows.flatMap(([, r]) => (r ? [Number(String((r.look.appearance as { skin: string }).skin).slice(1))] : []));

describe('portrait queue', () => {
  test('faces on screen are drawn first, the rest after them, in the order asked', () => {
    const p = new Portraits();
    const s = studio();
    void ask(p, 1, 'idle');
    void ask(p, 2, 'idle');
    void ask(p, 3, 'now');
    void ask(p, 4);
    // (The idle one asked for again from screen comes up with the others on it.)
    void ask(p, 2, 'now');
    p.direct(s as never, 0);
    expect(s.built).toHaveLength(1);
    expect(drawn(s.built[0])).toEqual([2, 3, 4, 1]);
  });

  test("a batch's Sims are put on their rows together: one build", () => {
    const p = new Portraits();
    const s = studio();
    for (let i = 1; i <= 40; i++) void ask(p, i);
    p.direct(s as never, 0);
    p.direct(s as never, 16);
    p.direct(s as never, 32);
    expect(s.built).toHaveLength(1);
    expect(drawn(s.built[0])).toHaveLength(32);
  });

  test("a face that went away isn't drawn, and its asker gets nothing", async () => {
    const p = new Portraits();
    const s = studio();
    const gone = new AbortController();
    const asked = ask(p, 1, 'now', gone.signal);
    void ask(p, 2);
    // (Asked twice, withdrawn once: still drawn for the other.)
    const twice = new AbortController();
    void ask(p, 3, 'now', twice.signal);
    void ask(p, 3);
    gone.abort();
    twice.abort();
    expect(await asked).toBeNull();
    p.direct(s as never, 0);
    expect(drawn(s.built[0])).toEqual([2, 3]);
    // Already withdrawn: never queued.
    const late = new AbortController();
    late.abort();
    expect(await ask(p, 9, 'now', late.signal)).toBeNull();
  });
});
