// `llm/cpuGraph.ts`: the language model's `GatherBlockQuantized` (no processor kernel in the
// WebGPU build the game loads) rebuilt from standard operators gives the same embedding. A small
// model with one such node runs before and after on ONNX Runtime's plain WASM build, which has
// the kernel.
import { describe, expect, test } from 'bun:test';
import { createRequire } from 'node:module';
import { forCpu } from '../src/llm/cpuGraph';

const ort = createRequire(import.meta.url)('onnxruntime-web') as typeof import('onnxruntime-web');
ort.env.logLevel = 'error';

const utf8 = new TextEncoder();
function varint(v: number): number[] {
  let n = BigInt.asUintN(64, BigInt(v));
  const out: number[] = [];
  while (n >= 0x80n) {
    out.push(Number(n & 0x7fn) | 0x80);
    n >>= 7n;
  }
  out.push(Number(n));
  return out;
}
const join = (parts: (Uint8Array | number[])[]) => Uint8Array.from(parts.flatMap((p) => [...p]));
const bytes = (no: number, data: Uint8Array) => join([varint((no << 3) | 2), varint(data.length), data]);
const str = (no: number, s: string) => bytes(no, utf8.encode(s));
const int = (no: number, v: number) => join([varint(no << 3), varint(v)]);

/** A tensor's type: element type and dims (a name for a free one). */
const valueInfo = (name: string, elem: number, dims: (number | string)[]) =>
  join([str(1, name), bytes(2, bytes(1, join([int(1, elem), bytes(2, join(dims.map((d) => bytes(1, typeof d === 'number' ? int(1, d) : str(2, d)))))])))]);
const tensor = (name: string, type: number, dims: number[], raw: Uint8Array) => join([...dims.map((d) => int(1, d)), int(2, type), str(8, name), bytes(9, raw)]);
const attr = (name: string, v: number) => join([str(1, name), int(3, v), int(20, 2)]);

const VOCAB = 7;
const WIDTH = 64;
const BLOCK = 16;
const BLOCKS = WIDTH / BLOCK;

/** A model whose one node reads the rows `ids` asks for out of a 4-bit table (zero points given, or not). */
function model(withZeroPoints: boolean): { graph: Uint8Array; table: Float32Array } {
  const rnd = (i: number) => (Math.sin(i * 12.9898) * 43758.5453) % 1;
  const quant = Uint8Array.from({ length: (VOCAB * WIDTH) / 2 }, (_, i) => Math.floor(Math.abs(rnd(i + 1)) * 256));
  const scales = Float32Array.from({ length: VOCAB * BLOCKS }, (_, i) => 0.01 + Math.abs(rnd(i + 500)) * 0.1);
  const zp = Uint8Array.from({ length: (VOCAB * BLOCKS) / 2 }, (_, i) => Math.floor(Math.abs(rnd(i + 900)) * 256));
  // The table it stands for: a byte's low half first, each block centred on its zero point (8 without them) and scaled.
  const nibble = (arr: Uint8Array, i: number) => (i % 2 ? arr[i >> 1] >> 4 : arr[i >> 1] & 15);
  const table = Float32Array.from({ length: VOCAB * WIDTH }, (_, i) => {
    const row = Math.floor(i / WIDTH);
    const blk = row * BLOCKS + Math.floor((i % WIDTH) / BLOCK);
    return (nibble(quant, i) - (withZeroPoints ? nibble(zp, blk) : 8)) * scales[blk];
  });
  const inputs = ['quant', 'ids', 'scales', ...(withZeroPoints ? ['zp'] : [])];
  const node = join([...inputs.map((i) => str(1, i)), str(2, 'embeds'), str(3, 'gather'), str(4, 'GatherBlockQuantized'), str(7, 'com.microsoft'), ...[attr('bits', 4), attr('block_size', BLOCK), attr('gather_axis', 0), attr('quantize_axis', 1)].map((a) => bytes(5, a))]);
  const graph = join([
    bytes(1, node),
    str(2, 'g'),
    bytes(5, tensor('quant', 2, [VOCAB, WIDTH / 2], quant)),
    bytes(5, tensor('scales', 1, [VOCAB, BLOCKS], new Uint8Array(scales.buffer))),
    ...(withZeroPoints ? [bytes(5, tensor('zp', 2, [VOCAB, BLOCKS / 2], zp))] : []),
    bytes(11, valueInfo('ids', 7, ['batch', 'seq'])),
    bytes(12, valueInfo('embeds', 1, ['batch', 'seq', WIDTH])),
  ]);
  const opset = (domain: string, version: number) => bytes(8, join([str(1, domain), int(2, version)]));
  return { graph: join([int(1, 10), opset('', 21), opset('com.microsoft', 1), bytes(7, graph)]), table };
}

async function embed(graph: Uint8Array, ids: number[][]): Promise<Float32Array> {
  const session = await ort.InferenceSession.create(graph, { executionProviders: ['wasm'], logSeverityLevel: 3 });
  const feed = new ort.Tensor('int64', BigInt64Array.from(ids.flat(), BigInt), [ids.length, ids[0].length]);
  const out = await session.run({ ids: feed });
  await session.release();
  return out.embeds.data as Float32Array;
}

describe('forCpu', () => {
  const ids = [
    [3, 0, 6],
    [1, 1, 5],
  ];
  for (const withZeroPoints of [true, false]) {
    test(`the same rows as GatherBlockQuantized (${withZeroPoints ? 'with' : 'without'} zero points)`, async () => {
      const { graph, table } = model(withZeroPoints);
      const patched = forCpu(graph);
      expect(new TextDecoder().decode(patched)).not.toContain('GatherBlockQuantized');
      const expected = Float32Array.from(ids.flat().flatMap((id) => [...table.subarray(id * WIDTH, (id + 1) * WIDTH)]));
      const [before, after] = [await embed(graph, ids), await embed(patched, ids)];
      expect(before.length).toBe(expected.length);
      for (let i = 0; i < expected.length; i++) {
        expect(before[i]).toBeCloseTo(expected[i], 5);
        expect(after[i]).toBeCloseTo(expected[i], 5);
      }
    });
  }

  test('a graph without the node is unchanged', () => {
    const plain = join([int(1, 10), bytes(7, join([str(2, 'empty')]))]);
    expect(forCpu(plain)).toBe(plain);
  });
});
