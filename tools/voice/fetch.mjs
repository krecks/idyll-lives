#!/usr/bin/env node
/**
 * Fetches the resident voice model, its bundled voices and the pronunciation dictionary into
 * `web/public/voice/` (not in git; see docs/design/voices.md).
 *
 * - Kyutai's Pocket TTS (CC BY 4.0), the ONNX export of its English 2026-04 bundle, used as
 *   published (int8): https://huggingface.co/KevinAHM/pocket-tts-onnx
 * - Its tokenizer: https://huggingface.co/kyutai/pocket-tts-without-voice-cloning
 * - The bundled voices (`PRESETS` in `web/src/voice/voices.ts`): https://huggingface.co/kyutai/tts-voices
 * - Misaki's US English dictionaries (Apache-2.0), for mouth shapes: https://github.com/hexgrad/misaki
 *
 * Everything is pinned to a revision and checked against its SHA-256, so every build speaks the
 * same. The voices are turned into clips as the voice encoder takes them (our voice WASM's
 * `clip`: 24 kHz, trimmed, at most 10 s, loudness evened out) and packed into one 16-bit file
 * with an index; both are pinned by their own SHA-256 too. Files already present with the right
 * hash are kept. Needs the voice WASM built first (`pnpm voice` does both). Usage:
 * `node tools/voice/fetch.mjs`.
 */

import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';

const OUT = fileURLToPath(new URL('../../web/public/voice/', import.meta.url));
const WEB = new URL('../../web/', import.meta.url);

const POCKET = 'https://huggingface.co/KevinAHM/pocket-tts-onnx/resolve/58a6d00cf13d239b6748cb0769f35c580a8f606c/onnx/english_2026-04';
const TOKENIZER = 'https://huggingface.co/kyutai/pocket-tts-without-voice-cloning/resolve/575d14372df89a4534bcb989870a34e77544b09f/languages/english_2026-04';
const MISAKI = 'https://raw.githubusercontent.com/hexgrad/misaki/fba1236595f2d2bf21d414ba6e57d25256afada3/misaki/data';

/** The model's folder under `voice/`: a new bundle gets a new folder, so browsers never mix versions. */
export const MODEL_DIR = 'pocket-en-2026-04/';

/** Pocket TTS's files, as published. */
const FILES = [
  { url: `${POCKET}/flow_lm_main_int8.onnx`, out: 'flow_lm_main_int8.onnx', sha256: 'f9bd8106b79a0192c1c43399ab938fb24900a95c1c599870d75a884e99000116' },
  { url: `${POCKET}/flow_lm_flow_int8.onnx`, out: 'flow_lm_flow_int8.onnx', sha256: '3dd781ee5abee9e195320bf0106bebd6372a852b3b36352524ee78b40554635d' },
  { url: `${POCKET}/mimi_decoder_int8.onnx`, out: 'mimi_decoder_int8.onnx', sha256: '3630450a3297a101792a6ac66619ebc70ab916b265e6220c2afaef8b1673f925' },
  { url: `${POCKET}/mimi_encoder_int8.onnx`, out: 'mimi_encoder_int8.onnx', sha256: 'abb46f1ad0b83c3840fe38c3652ab0eda7d1d97a63f1953bbbff5d04d8f4c855' },
  { url: `${POCKET}/text_conditioner_int8.onnx`, out: 'text_conditioner_int8.onnx', sha256: '24c0642bff11a4e1435c004d0c2aa6c68e093fadfa35f0f0dba849ac67f25ac3' },
  { url: `${POCKET}/bos_before_voice.npy`, out: 'bos_before_voice.npy', sha256: 'f46edf4f7007b7ba4ea58831f49d003e59e167b4641c44bb3addfe9231a780b1' },
  { url: `${POCKET}/bundle.json`, out: 'bundle.json', sha256: 'bab643150f437f37df080a710520ff39ed9ebd9a339f8ebdc739f7eddfc28b3f' },
  { url: `${TOKENIZER}/tokenizer.json`, out: 'tokenizer.json', sha256: 'f498428e1eafee50492f7be13dc9bfafcfc12e508cd0eb1b01c92ecd5d8c6687' },
];
/** The packed voices (16-bit, 24 kHz, in `PRESETS` order) and their index, pinned. */
const PRESET_PCM = { out: 'presets-2.pcm', sha256: '54e1209d043c8fd1054b5529cde5a1bdcc1cabb4ef909a6fa16328756af803dc' };
const PRESET_INDEX = { out: 'presets-2.json', sha256: '1d64e66515c74349ae0e129997d2eea855d7daa3ea1e608f4a22482f50ac864b' };
/** Files earlier versions wrote that are no longer used (they would be copied into the build). */
const OBSOLETE = ['presets-1.pcm', 'presets-1.json', 'presets-1.stamp', 'paradee-8m.onnx', 'paradee-8m-edit1.onnx', 'paradee-8m.json', 'kitten-nano-0.8-edit1.onnx', 'kitten-nano-0.8-edit2.onnx', 'kitten-nano-0.8-voices.f32'];
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

async function existing(path, expected) {
  try {
    return sha256(await readFile(path)) === expected;
  } catch {
    return false;
  }
}

/** A mono 16-bit PCM WAV file: its samples and rate. */
function readWav(bytes) {
  let rate = 0;
  let channels = 0;
  let bits = 0;
  for (let p = 12; p + 8 <= bytes.length; ) {
    const id = bytes.toString('ascii', p, p + 4);
    const len = bytes.readUInt32LE(p + 4);
    if (id === 'fmt ') {
      channels = bytes.readUInt16LE(p + 10);
      rate = bytes.readUInt32LE(p + 12);
      bits = bytes.readUInt16LE(p + 22);
    } else if (id === 'data') {
      if (channels !== 1 || bits !== 16) throw new Error(`WAV: ${channels} channels, ${bits} bits (expected mono 16-bit)`);
      const samples = new Float32Array(len >> 1);
      for (let i = 0; i < samples.length; i++) samples[i] = bytes.readInt16LE(p + 8 + i * 2) / 32768;
      return { samples, rate };
    }
    p += 8 + len + (len & 1);
  }
  throw new Error('WAV: no data');
}

await mkdir(OUT + MODEL_DIR, { recursive: true });

for (const file of FILES) {
  const path = OUT + MODEL_DIR + file.out;
  if (await existing(path, file.sha256)) continue;
  console.log(`voice: downloading ${file.out}`);
  await writeFile(path, await download(file));
}
for (const name of OBSOLETE) await rm(OUT + name, { force: true });

// The bundled voices, as clips for the encoder. Rebuilt when the list or our WASM (which makes the
// clips) changes; different clips need new pins, and a new file name so browsers don't keep the
// old ones.
const { PRESETS, PRESET_REVISION } = await import(new URL('src/voice/voices.ts', WEB).href);
const voiceWasm = await readFile(new URL('src/voice/wasm-pkg/voice_wasm_bg.wasm', WEB));
const presetStamp = sha256(JSON.stringify([PRESETS.map((p) => [p.name, p.sha256]), sha256(voiceWasm)]));
const stampPath = OUT + PRESET_PCM.out.replace(/\.pcm$/, '.stamp');
let builtStamp = '';
try {
  builtStamp = await readFile(stampPath, 'utf8');
} catch {
  // Not built yet.
}
if (builtStamp !== presetStamp || !(await existing(OUT + PRESET_PCM.out, PRESET_PCM.sha256)) || !(await existing(OUT + PRESET_INDEX.out, PRESET_INDEX.sha256))) {
  const { initSync, clip } = await import(new URL('src/voice/wasm-pkg/voice_wasm.js', WEB).href);
  initSync({ module: voiceWasm });
  console.log(`voice: building ${PRESET_PCM.out} (${PRESETS.length} voices)`);
  const clips = [];
  for (const p of PRESETS) {
    const { samples, rate } = readWav(await download({ url: `https://huggingface.co/kyutai/tts-voices/resolve/${PRESET_REVISION}/${p.file}`, sha256: p.sha256 }));
    clips.push(clip(samples, rate));
  }
  const total = clips.reduce((n, c) => n + c.length, 0);
  const pcm = Buffer.alloc(total * 2);
  const index = [];
  let at = 0;
  PRESETS.forEach((p, i) => {
    index.push({ name: p.name, offset: at, length: clips[i].length });
    for (const v of clips[i]) pcm.writeInt16LE(Math.max(-32768, Math.min(32767, Math.round(v * 32767))), (at++) * 2);
  });
  const json = Buffer.from(JSON.stringify(index) + '\n');
  for (const [file, bytes] of [
    [PRESET_PCM, pcm],
    [PRESET_INDEX, json],
  ]) {
    const actual = sha256(bytes);
    if (actual !== file.sha256) throw new Error(`${file.out}: SHA-256 ${actual}, expected ${file.sha256}`);
    await writeFile(OUT + file.out, bytes);
  }
  await writeFile(stampPath, presetStamp);
  console.log(`voice: ${(total / 24000).toFixed(1)} s of voices, ${(pcm.length / 1e6).toFixed(1)} MB`);
}

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
