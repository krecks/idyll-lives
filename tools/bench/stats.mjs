#!/usr/bin/env bun
/**
 * Numbers from pictures, so that "the lawn is less even" or "the shade is deeper" is measured and not only
 * said: for every PNG of a shot folder (`render.mjs --shots`, `play.mjs`), how bright it is, how much of it is
 * clipped, how saturated, how much of the frame is one tone, and how much contrast it has at four sizes.
 *
 *   bun tools/bench/stats.mjs <dir>              a table of the folder's pictures
 *   bun tools/bench/stats.mjs <dirA> <dirB>      the pictures both have: A, B and the change
 *   bun tools/bench/stats.mjs --steady <dirA> <dirB>
 *     how much a picture changes when the camera moves a quarter of a pixel: A and B are the same build's shots
 *     of the same poses, B with `--extra 'tx=0.004&tz=0.004'` (4 mm at the play camera, where a pixel is 16 mm;
 *     both, because a pose that sets one of them itself keeps its own). A steady
 *     picture hardly changes; detail finer than a pixel (grass blades, a tiling texture far off) comes out
 *     different every time, which in play is shimmer as the camera moves, and which one still does not show.
 *     Compare the figure before a change and after it.
 *   --json                                        the same as JSON
 *
 * The columns (brightness and contrast in the picture's own 0–255 values, as the screen shows them):
 *   p5 / p50 / p95   brightness at the 5th, 50th and 95th percentile
 *   clip             % of pixels with a channel at 250 or more; `dark`: % at brightness 5 or less
 *   sat              mean saturation, 0–100
 *   even             % of the frame within a step of its most common tone (brightness and colour in 16 steps
 *                    each, after an 8 px blur): a picture that is mostly one lawn scores high
 *   fine             contrast finer than 8 px (texture, noise, edges)
 *   c8 / c64 / c256  contrast between 8 px blocks inside their 64 px block, 64 px blocks inside their 256 px
 *                    block, and between 256 px blocks: detail, things, and the picture's big shapes of light
 *
 * These describe a picture; none of them says it is better. They are for checking what the eye claims.
 */

import { readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const sharp = (await import(createRequire(join(ROOT, 'web/package.json')).resolve('sharp'))).default;

const args = process.argv.slice(2);
const json = args.includes('--json');
const steady = args.includes('--steady');
const dirs = args.filter((a) => !a.startsWith('--'));
if (dirs.length < 1 || dirs.length > 2 || (steady && dirs.length !== 2)) {
  console.error('usage: stats.mjs <dir> | <dirA> <dirB> | --steady <dirA> <dirB>   [--json]');
  process.exit(2);
}

const pngs = (dir) => readdirSync(dir).filter((f) => f.endsWith('.png')).sort();

/** A picture's brightness (Rec. 709, of the encoded values) and its RGB, without alpha. */
async function load(file) {
  const { data, info } = await sharp(file).removeAlpha().raw().toBuffer({ resolveWithObject: true });
  const n = info.width * info.height;
  const luma = new Float32Array(n);
  for (let i = 0, p = 0; i < n; i++, p += 3) luma[i] = 0.2126 * data[p] + 0.7152 * data[p + 1] + 0.0722 * data[p + 2];
  return { rgb: data, luma, w: info.width, h: info.height };
}

/** Block means of `src` (w × h) in blocks of `size`; the last row and column of blocks may be narrower. */
function blocks(src, w, h, size) {
  const bw = Math.ceil(w / size);
  const bh = Math.ceil(h / size);
  const sum = new Float64Array(bw * bh);
  const count = new Uint32Array(bw * bh);
  for (let y = 0; y < h; y++) {
    const row = Math.floor(y / size) * bw;
    for (let x = 0; x < w; x++) {
      const b = row + Math.floor(x / size);
      sum[b] += src[y * w + x];
      count[b]++;
    }
  }
  const mean = new Float32Array(bw * bh);
  for (let i = 0; i < mean.length; i++) mean[i] = sum[i] / count[i];
  return { mean, w: bw, h: bh, size };
}

/** RMS of `fine` minus the block of `coarse` it lies in (`ratio`: fine blocks per coarse block, each way). */
function band(fine, fw, fh, coarse, ratio) {
  let s = 0;
  for (let y = 0; y < fh; y++) {
    const row = Math.floor(y / ratio) * coarse.w;
    for (let x = 0; x < fw; x++) {
      const d = fine[y * fw + x] - coarse.mean[row + Math.floor(x / ratio)];
      s += d * d;
    }
  }
  return Math.sqrt(s / (fw * fh));
}

function percentiles(luma) {
  const hist = new Uint32Array(256);
  for (let i = 0; i < luma.length; i++) hist[Math.min(255, Math.round(luma[i]))]++;
  const at = (q) => {
    let acc = 0;
    for (let v = 0; v < 256; v++) {
      acc += hist[v];
      if (acc >= q * luma.length) return v;
    }
    return 255;
  };
  return { p5: at(0.05), p50: at(0.5), p95: at(0.95), dark: (100 * hist.slice(0, 6).reduce((a, b) => a + b, 0)) / luma.length };
}

async function measure(file) {
  const { rgb, luma, w, h } = await load(file);
  const n = w * h;
  let clip = 0;
  let sat = 0;
  for (let p = 0; p < n * 3; p += 3) {
    const max = Math.max(rgb[p], rgb[p + 1], rgb[p + 2]);
    const min = Math.min(rgb[p], rgb[p + 1], rgb[p + 2]);
    if (max >= 250) clip++;
    sat += max > 0 ? (max - min) / max : 0;
  }
  // The most common tone: brightness and the two colour differences of the 8 px blocks, in 16 steps each.
  const channel = (c) => {
    const one = new Float32Array(n);
    for (let i = 0; i < n; i++) one[i] = rgb[i * 3 + c];
    return blocks(one, w, h, 8).mean;
  };
  const [r, g, b] = [channel(0), channel(1), channel(2)];
  const tone = (i) => {
    const y = 0.2126 * r[i] + 0.7152 * g[i] + 0.0722 * b[i];
    const step = (v) => Math.max(0, Math.min(15, Math.floor(v / 16)));
    return [step(y), step((b[i] - y) / 1.8556 + 128), step((r[i] - y) / 1.5748 + 128)];
  };
  const bins = new Map();
  const tones = [];
  for (let i = 0; i < r.length; i++) {
    const t = tone(i);
    tones.push(t);
    const key = (t[0] << 8) | (t[1] << 4) | t[2];
    bins.set(key, (bins.get(key) ?? 0) + 1);
  }
  let mode = 0;
  let most = 0;
  for (const [key, c] of bins) if (c > most) [mode, most] = [key, c];
  const m = [mode >> 8, (mode >> 4) & 15, mode & 15];
  const even = tones.filter((t) => Math.abs(t[0] - m[0]) <= 1 && Math.abs(t[1] - m[1]) <= 1 && Math.abs(t[2] - m[2]) <= 1).length / tones.length;

  const b8 = blocks(luma, w, h, 8);
  const b64 = blocks(luma, w, h, 64);
  const b256 = blocks(luma, w, h, 256);
  const all = { mean: new Float32Array([b256.mean.reduce((a, v) => a + v, 0) / b256.mean.length]), w: 1 };
  return {
    ...percentiles(luma),
    clip: (100 * clip) / n,
    sat: (100 * sat) / n,
    even: 100 * even,
    fine: band(luma, w, h, b8, 8),
    c8: band(b8.mean, b8.w, b8.h, b64, 8),
    c64: band(b64.mean, b64.w, b64.h, b256, 4),
    c256: band(b256.mean, b256.w, b256.h, all, Infinity),
  };
}

/** How much the picture changed between two shots a quarter of a pixel apart: the mean and the 99th percentile of the brightness difference. */
async function moved(fileA, fileB) {
  const [a, b] = await Promise.all([load(fileA), load(fileB)]);
  if (a.w !== b.w || a.h !== b.h) throw new Error(`${fileA} and ${fileB} differ in size`);
  const hist = new Uint32Array(256);
  let sum = 0;
  for (let i = 0; i < a.luma.length; i++) {
    const d = Math.abs(a.luma[i] - b.luma[i]);
    sum += d;
    hist[Math.min(255, Math.round(d))]++;
  }
  let acc = 0;
  let p99 = 255;
  for (let v = 0; v < 256; v++) {
    acc += hist[v];
    if (acc >= 0.99 * a.luma.length) {
      p99 = v;
      break;
    }
  }
  return { mean: sum / a.luma.length, p99, over8: (100 * hist.slice(9).reduce((x, y) => x + y, 0)) / a.luma.length };
}

const COLUMNS = ['p5', 'p50', 'p95', 'clip', 'dark', 'sat', 'even', 'fine', 'c8', 'c64', 'c256'];
const digits = (c) => (c.startsWith('p') ? 0 : c === 'clip' || c === 'dark' ? 2 : 1);
const fmt = (v, c) => v.toFixed(digits(c));
const signed = (v, c) => `${v >= 0 ? '+' : ''}${v.toFixed(Math.max(1, digits(c)))}`;

if (steady) {
  const [a, b] = dirs;
  const shared = pngs(a).filter((f) => pngs(b).includes(f));
  const rows = [];
  for (const f of shared) rows.push({ picture: f, ...(await moved(join(a, f), join(b, f))) });
  if (json) console.log(JSON.stringify(rows, null, 1));
  else {
    console.log('| picture | mean change | 99th percentile | % of pixels changed by more than 8 |\n|---|---|---|---|');
    for (const r of rows) console.log(`| ${r.picture} | ${r.mean.toFixed(2)} | ${r.p99} | ${r.over8.toFixed(2)} |`);
  }
} else if (dirs.length === 1) {
  const rows = [];
  for (const f of pngs(dirs[0])) rows.push({ picture: f, ...(await measure(join(dirs[0], f))) });
  if (json) console.log(JSON.stringify(rows, null, 1));
  else {
    console.log(`| picture | ${COLUMNS.join(' | ')} |\n|---|${COLUMNS.map(() => '---').join('|')}|`);
    for (const r of rows) console.log(`| ${r.picture} | ${COLUMNS.map((c) => fmt(r[c], c)).join(' | ')} |`);
  }
} else {
  const [a, b] = dirs;
  const shared = pngs(a).filter((f) => pngs(b).includes(f));
  const rows = [];
  for (const f of shared) rows.push({ picture: f, a: await measure(join(a, f)), b: await measure(join(b, f)) });
  if (json) console.log(JSON.stringify(rows, null, 1));
  else {
    console.log(`A = ${a}\nB = ${b}\n\nEach cell: A → B (change).\n`);
    console.log(`| picture | ${COLUMNS.join(' | ')} |\n|---|${COLUMNS.map(() => '---').join('|')}|`);
    for (const r of rows) console.log(`| ${r.picture} | ${COLUMNS.map((c) => `${fmt(r.a[c], c)} → ${fmt(r.b[c], c)} (${signed(r.b[c] - r.a[c], c)})`).join(' | ')} |`);
  }
}
