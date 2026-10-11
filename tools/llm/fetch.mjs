#!/usr/bin/env node
/**
 * Fetches the default language model that writes residents' lines into `web/public/llm/` (not
 * in git; see `web/src/llm/models.ts`). The other built-in models, and models players add,
 * download from Hugging Face in the browser.
 *
 * - LFM2.5-350M (Liquid AI, LFM Open License v1.0), ONNX export by onnx-community, its 4-bit
 *   build as it is: https://huggingface.co/onnx-community/LFM2.5-350M-ONNX
 *
 * Everything is pinned to a revision and checked against its SHA-256. Files already present with
 * the right hash are kept. Big files are streamed. Usage: `node tools/llm/fetch.mjs`.
 */

import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { mkdir, rename, rm } from 'node:fs/promises';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const OUT = fileURLToPath(new URL('../../web/public/llm/lfm2.5-350m/', import.meta.url));
const LFM = 'https://huggingface.co/onnx-community/LFM2.5-350M-ONNX/resolve/7dd4999565b0342c381ba90a3d8fc467d6df19c4';

const FILES = [
  { path: 'tokenizer.json', sha256: '29d43b4be8e8a896fefd7cd836ca6d6b4eedd249f823866ce0453b368e646f49' },
  { path: 'tokenizer_config.json', sha256: 'e70cd070c2696bf1b0f1ad303c34502753670cca6f844637821cb1c37e6bea8d' },
  { path: 'onnx/model_q4.onnx', sha256: 'd1a705712e93aafaba1346b32245fa59e7857a46e2272003c0e8c524977e0de8' },
  { path: 'onnx/model_q4.onnx_data', sha256: '71ec6ad38a4c463dcb3dba671d06a1d9861be3a23e51290d818b95c0b7d2a5db' },
];
/** Files earlier versions put into the site that are no longer used. */
const OBSOLETE = ['config.json', 'model_i8.onnx', 'model_i8.onnx_data', 'model_q4f16.onnx', 'model_q4f16.onnx_data'];

async function fileSha256(path) {
  const hash = createHash('sha256');
  await pipeline(createReadStream(path), hash);
  return hash.digest('hex');
}

async function existing(path, expected) {
  try {
    return (await fileSha256(path)) === expected;
  } catch {
    return false;
  }
}

/** Streams the download to `path` (via a temporary file), checking its hash on the way. */
async function download(url, path, expected) {
  const res = await fetch(url);
  if (!res.ok || !res.body) throw new Error(`${url}: HTTP ${res.status}`);
  const hash = createHash('sha256');
  const tmp = `${path}.part`;
  const body = Readable.fromWeb(res.body);
  body.on('data', (chunk) => hash.update(chunk));
  await pipeline(body, createWriteStream(tmp));
  const actual = hash.digest('hex');
  if (actual !== expected) {
    await rm(tmp, { force: true });
    throw new Error(`${url}: SHA-256 ${actual}, expected ${expected}`);
  }
  await rename(tmp, path);
}

await mkdir(OUT, { recursive: true });
for (const name of OBSOLETE) await rm(OUT + name, { force: true });
for (const file of FILES) {
  const name = file.path.replace('onnx/', '');
  if (await existing(OUT + name, file.sha256)) continue;
  console.log(`llm: downloading ${name}`);
  await download(`${LFM}/${file.path}`, OUT + name, file.sha256);
}
