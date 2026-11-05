// The language model reads at most its `positions` (`llm/generate.ts`; `cargo fetch-llm` cuts
// LFM2.5's rotary tables to 4,096, `crates/tools/src/llmslim.rs`): a reply stops where they end,
// and a prompt that doesn't fit is refused, never run past them. A stand-in model that always
// writes " b", one token for each character of the prompt.
import { describe, expect, test } from 'bun:test';
import { generate, PrefixCache, type GenParams, type LlmEngine, type Tensor } from '../src/llm/generate';

class FakeTensor implements Tensor {
  constructor(
    readonly type: string,
    readonly data: unknown,
    readonly dims: readonly number[],
  ) {}
}

const VOCAB = [' a', ' b', '<|end|>'];

/** The engine, and the longest attention mask it was run with. */
function engine(positions: number): { engine: LlmEngine; longest: () => number } {
  let longest = 0;
  const session = {
    inputNames: ['input_ids', 'attention_mask'],
    outputNames: ['logits'],
    inputMetadata: [],
    async run(feeds: Record<string, Tensor>) {
      longest = Math.max(longest, feeds.attention_mask.dims[1]);
      const n = feeds.input_ids.dims[1];
      const logits = new Float32Array(n * VOCAB.length);
      for (let i = 0; i < n; i++) logits[i * VOCAB.length + 1] = 10;
      return { logits: new FakeTensor('float32', logits, [1, n, VOCAB.length]) };
    },
  };
  const tokenizer = {
    encode: (text: string) => ({ ids: [...text].map(() => 0) }),
    decode: (ids: number[]) => ids.map((id) => VOCAB[id]).join(''),
  };
  return {
    engine: { ort: { Tensor: FakeTensor }, session, tokenizer, stopIds: new Set([2]), prefixes: new PrefixCache(2), positions },
    longest: () => longest,
  };
}

const params = (maxTokens: number): GenParams => ({ maxTokens, temperature: 0, topK: 1, topP: 1, repetitionPenalty: 1, seed: 1 });

describe('language model positions', () => {
  test('a reply stops where the positions end', async () => {
    const e = engine(10);
    const r = await generate(e.engine, { parts: ['abc'] }, params(50));
    expect(r.ids.length).toBe(7);
    expect(r.stop).toBe('length');
    expect(e.longest()).toBeLessThanOrEqual(10);
  });

  test('a shorter reply asked for is as long as asked', async () => {
    const e = engine(10);
    const r = await generate(e.engine, { parts: ['ab', 'c'] }, params(4));
    expect(r.ids.length).toBe(4);
    expect(e.longest()).toBeLessThanOrEqual(10);
  });

  test('a prompt that fills the positions is refused', async () => {
    const e = engine(10);
    await expect(generate(e.engine, { parts: ['abcdefghij'] }, params(4))).rejects.toThrow('the model reads 10');
    expect(e.longest()).toBe(0);
  });
});
