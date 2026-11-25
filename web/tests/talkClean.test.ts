// What the language model wrote, made into a line (`talk/clean.ts`): what's said is kept; labels, quotes, narration,
// stage directions and blocked words go. And a model file whose bytes aren't the ones pinned (`ml/assets.ts`) is
// refused and not cached. No model needed.
import { afterEach, describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { fetchCached } from '../src/ml/assets';
import { cleanLine } from '../src/talk/clean';

const { blocklist } = JSON.parse(readFileSync(new URL('../public/content/talk/en.json', import.meta.url), 'utf8')) as { blocklist: string[] };

describe('cleanLine', () => {
  const CLEAN: [string, string | null][] = [
    ['Time to cook something nice.', 'Time to cook something nice.'],
    ['Mia says: "How was your day, Tom?"', 'How was your day, Tom?'],
    ["Mia's line: \"Pasta, fresh pasta!\"", 'Pasta, fresh pasta!'],
    ['"Hey Tom!" Mia waves.', 'Hey Tom!'],
    ['Mia smiles as she drifts off to sleep.', null],
    ['Tom laughs and says "good one"', null],
    ['*stretches* Time for bed', 'Time for bed.'],
    ['Mia, stop it!', 'Stop it!'],
    ['I love this 😀 so much!!', 'I love this so much!!'],
    ['What the hell is this crap?', null],
    ['Mia', null],
    ['One. Two. Three. Four.', 'One. Two. Three. Four.'],
    ['One. Two. Three. Four. Five. Six.', 'One. Two. Three. Four. Five.'],
    ['Look at [read](/ɹˈɛd/) this', 'Look at this.'],
    ['<|im_end|>', null],
    ['Sure, as an AI I can help.', null],
  ];
  test.each(CLEAN)('%p', (raw, want) => {
    expect(cleanLine(raw, { me: 'Mia', names: ['Tom'], blocklist })).toBe(want);
  });

  // Cut short (out of tokens or time): the unfinished sentence goes (with nothing finished, the line).
  const CUT: [string, string | null][] = [
    ['What a day. I think I will', 'What a day.'],
    ['What a day. I think I will!', 'What a day. I think I will!'],
    ['What a day and', null],
  ];
  test.each(CUT)('cut short: %p', (raw, want) => {
    expect(cleanLine(raw, { me: 'Mia', names: ['Tom'], blocklist, cut: true })).toBe(want);
  });
});

describe('fetchCached', () => {
  const realFetch = globalThis.fetch;
  afterEach(() => {
    globalThis.fetch = realFetch;
  });

  test('a file whose SHA-256 differs throws and is not cached', async () => {
    globalThis.fetch = (async () => new Response(new Uint8Array([1, 2, 3]))) as unknown as typeof fetch;
    const put: string[] = [];
    const cache = { match: async () => undefined, put: async (url: string) => void put.push(url) } as unknown as Cache;
    await expect(fetchCached(cache, 'https://x.test/model.json', () => {}, '0'.repeat(64))).rejects.toThrow('SHA-256 differs');
    expect(put).toEqual([]);
  });
});
