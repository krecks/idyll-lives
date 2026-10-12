#!/usr/bin/env node
/**
 * Fetches the resident voice model and the pronunciation dictionary into `web/public/voice/`
 * (not in git; see docs/design/voices.md).
 *
 * - Supertone's Supertonic 3 (BigScience OpenRAIL-M: its licence, with its use restrictions, is
 *   fetched with it and goes along), its config, text indexer and ten voice styles as published,
 *   and its four ONNX graphs with their weights stored as 16-bit floats (`fp16.mjs`: half the
 *   size, the same arithmetic; a `NOTICE` says so) and gzipped (`.onnxz`, unpacked by the
 *   browser): 398 MB as published, 185 MB here. https://huggingface.co/Supertone/supertonic-3
 * - Misaki's US English dictionaries (Apache-2.0), for mouth shapes: https://github.com/hexgrad/misaki
 *
 * Everything is pinned to a revision and checked against its SHA-256 (the converted graphs too:
 * the conversion is deterministic), so every build speaks the same. Files already present with
 * the right hash (unpacked) are kept. Usage: `node tools/voice/fetch.mjs` (`pnpm voice` builds
 * the voice WASM, then runs this).
 */

import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { gunzipSync, gzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import { weightsToFp16 } from './fp16.mjs';

const OUT = fileURLToPath(new URL('../../web/public/voice/', import.meta.url));

const SUPERTONIC = 'https://huggingface.co/Supertone/supertonic-3/resolve/3cadd1ee6394adea1bd021217a0e650ede09a323';
const MISAKI = 'https://raw.githubusercontent.com/hexgrad/misaki/fba1236595f2d2bf21d414ba6e57d25256afada3/misaki/data';

/** The model's folder under `voice/`: a new version gets a new folder, so browsers never mix versions. */
export const MODEL_DIR = 'supertonic-3/';

/**
 * Supertonic 3's graphs: as published, and with their weights in 16-bit floats (`fp16.mjs`),
 * saved gzipped under `<name>.onnxz`.
 */
const GRAPHS = [
  ['duration_predictor', 'c3eb91414d5ff8a7a239b7fe9e34e7e2bf8a8140d8375ffb14718b1c639325db', '622de4d364dcb42259986c32d5ca4a0f521fddfaed09ffb155aa8cf3d7e4d864'],
  ['text_encoder', 'c7befd5ea8c3119769e8a6c1486c4edc6a3bc8365c67621c881bbb774b9902ff', '7533e6a775956007f8dc848d008751e7dbbcf8399cefb5ef180737541c3e0e0c'],
  ['vector_estimator', '883ac868ea0275ef0e991524dc64f16b3c0376efd7c320af6b53f5b780d7c61c', '27a86f93335fd6edec7210c24dddb4f0de700688f9f0d18ab6418bd852742402'],
  ['vocoder', '085de76dd8e8d5836d6ca66826601f615939218f90e519f70ee8a36ed2a4c4ba', '373aca3e481c5478af6e66773a58dcd1bdd6e3f874d676dc3b90ffb7e268452a'],
].map(([name, sha256, fp16]) => ({ url: `${SUPERTONIC}/onnx/${name}.onnx`, name, sha256, fp16 }));

/** Supertonic 3's other files, as published (`path` in the repository, saved under its last part). */
const FILES = [
  ['onnx/tts.json', '42078d3aef1cd43ab43021f3c54f47d2d75ceb4e75f627f118890128b06a0d09'],
  ['onnx/unicode_indexer.json', '9bf7346e43883a81f8645c81224f786d43c5b57f3641f6e7671a7d6c493cb24f'],
  ['voice_styles/F1.json', 'bbdec6ee00231c2c742ad05483df5334cab3b52fda3ba38e6a07059c4563dbc2'],
  ['voice_styles/F2.json', '7c722c6a72707b1a77f035d67f0d1351ba187738e06f7683e8c72b1df3477fc6'],
  ['voice_styles/F3.json', '12f6ef2573baa2defa1128069cb59f203e3ab67c92af77b42df8a0e3a2f7c6ab'],
  ['voice_styles/F4.json', 'c2fa764c1225a76dfc3e2c73e8aa4f70d9ee48793860eb34c295fff01c2e032b'],
  ['voice_styles/F5.json', '45966e73316415626cf41a7d1c6f3b4c70dbc1ba2bee5c1978ef0ce33244fc8d'],
  ['voice_styles/M1.json', 'e35604687f5d23694b8e91593a93eec0e4eca6c0b02bb8ed69139ab2ea6b0a5b'],
  ['voice_styles/M2.json', 'b76cbf62bac707c710cf0ae5aba5e31eea1a6339a9734bfae33ab98499534a50'],
  ['voice_styles/M3.json', 'ea1ac35ccb91b0d7ecad533a2fbd0eec10c91513d8951e3b25fbba99954e159b'],
  ['voice_styles/M4.json', 'ca8eefad4fcd989c9379032ff3e50738adc547eeb5e221b82593a6d7b3bac303'],
  ['voice_styles/M5.json', 'dd22b92740314321f8ae11c5e87f8dd60d060f15dd3a632b5adf77f471f77af2'],
  ['LICENSE', '0d944a9110fed9a9602d60e0423a272903e7bd21ab060490774efc77c2275e9f'],
].map(([path, sha256]) => ({ url: `${SUPERTONIC}/${path}`, out: path.split('/').pop(), sha256 }));
/** Files earlier versions wrote that are no longer used (they would be copied into the build). */
/** Files in the model's folder earlier versions wrote (the graphs as published). */
const OBSOLETE_IN_MODEL = GRAPHS.map((g) => `${g.name}.onnx`);
const OBSOLETE = ['pocket-en-2026-04', 'presets-2.pcm', 'presets-2.json', 'presets-2.stamp', 'presets-1.pcm', 'presets-1.json', 'presets-1.stamp', 'paradee-8m.onnx', 'paradee-8m-edit1.onnx', 'paradee-8m.json', 'kitten-nano-0.8-edit1.onnx', 'kitten-nano-0.8-edit2.onnx', 'kitten-nano-0.8-voices.f32'];
const LEXICON = [
  { url: `${MISAKI}/us_gold.json`, sha256: 'dc414872a49a28ae6c141463d502fd945f3b2fde040484fdc47d00cc4612686f' },
  { url: `${MISAKI}/us_silver.json`, sha256: 'de8f67be911bb6c659187b4a65fd966b6a30e56350e0f790d763210b053ac475' },
];

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

async function download({ url, sha256: expected }) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  const bytes = Buffer.from(await res.arrayBuffer());
  const actual = sha256(bytes);
  if (actual !== expected) throw new Error(`${url}: SHA-256 ${actual}, expected ${expected}`);
  return bytes;
}

/** Whether `path` holds the expected file (unpacked first if `packed`). */
async function existing(path, expected, packed = false) {
  try {
    const bytes = await readFile(path);
    return sha256(packed ? gunzipSync(bytes) : bytes) === expected;
  } catch {
    return false;
  }
}

/** What `NOTICE` says about the changed graphs (the licence asks for changes to be marked). */
const NOTICE = `Supertonic 3 (Supertone Inc.), https://huggingface.co/Supertone/supertonic-3, revision 3cadd1ee.
Licensed under the BigScience OpenRAIL-M licence (LICENSE), including its use restrictions.

Modified by Idyll Lives (tools/voice/fetch.mjs, tools/voice/fp16.mjs): in duration_predictor,
text_encoder, vector_estimator and vocoder, every float32 weight of 1,024 values or more is stored
as float16 and cast back to float32 when the model loads; the files are gzipped (.onnxz). Nothing
was retrained. tts.json, unicode_indexer.json and the voice styles (F1-F5, M1-M5) are unchanged.
`;

await mkdir(OUT + MODEL_DIR, { recursive: true });

for (const file of FILES) {
  const path = OUT + MODEL_DIR + file.out;
  if (await existing(path, file.sha256)) continue;
  console.log(`voice: downloading ${file.out}`);
  await writeFile(path, await download(file));
}
for (const g of GRAPHS) {
  const path = `${OUT}${MODEL_DIR}${g.name}.onnxz`;
  if (await existing(path, g.fp16, true)) continue;
  console.log(`voice: downloading ${g.name}.onnx, storing its weights as float16`);
  const { model, weights, saved } = weightsToFp16(await download(g));
  const actual = sha256(model);
  if (actual !== g.fp16) throw new Error(`${g.name}: float16 weights SHA-256 ${actual}, expected ${g.fp16}`);
  await writeFile(path, gzipSync(model, { level: 9 }));
  console.log(`voice: ${g.name}: ${weights} weights, ${(saved / 1e6).toFixed(1)} MB saved`);
}
await writeFile(`${OUT}${MODEL_DIR}NOTICE`, NOTICE);
for (const name of OBSOLETE) await rm(OUT + name, { force: true, recursive: true });
for (const name of OBSOLETE_IN_MODEL) await rm(OUT + MODEL_DIR + name, { force: true });

/**
 * The dictionary as `word<TAB>phonemes` lines, gzipped. Gold entries win over silver ones; for
 * words whose pronunciation depends on the part of speech only the default is kept (lines are
 * short and the engine has no tagger). Named `.lexz`, not `.gz`: servers send `.gz` files with
 * `Content-Encoding: gzip`, and the browser would unpack them on its own.
 */
const lexPath = OUT + 'en-us.lexz';
const stamp = LEXICON.map((l) => l.sha256.slice(0, 12)).join(' ');
let current = '';
try {
  current = await readFile(OUT + 'en-us.lex.stamp', 'utf8');
} catch {
  // Not built yet.
}
if (current !== stamp) {
  console.log('voice: building en-us.lexz');
  const [gold, silver] = await Promise.all(LEXICON.map(async (l) => JSON.parse((await download(l)).toString('utf8'))));
  const words = new Map();
  for (const dict of [silver, gold]) {
    for (const [word, value] of Object.entries(dict)) {
      const phonemes = typeof value === 'string' ? value : value?.DEFAULT;
      if (phonemes) words.set(word, phonemes);
    }
  }
  const lines = [...words].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([w, p]) => `${w}\t${p}`);
  await writeFile(lexPath, gzipSync(lines.join('\n') + '\n', { level: 9 }));
  await writeFile(OUT + 'en-us.lex.stamp', stamp);
  console.log(`voice: ${lines.length} words`);
}
