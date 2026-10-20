/**
 * `dist/THIRD-PARTY-LICENSES.txt`, written by production builds (vite.config.ts): what the game
 * ships that others made, and their licence texts.
 *
 * - The npm packages bundled into its JavaScript, as the bundler saw them (the page and its
 *   workers), with the licence files from each package.
 * - The Rust crates compiled into the simulation's and the voice's WebAssembly: `cargo metadata`
 *   for the dependencies of sim-wasm and voice-wasm, the licence files from Cargo's sources.
 * - Files shipped as they are (the meshopt and Zstandard decoders), work adapted into the game's
 *   own code (TalkingHead's expressions), and the licences that came with the voice model, the
 *   dictionary and the language model (fetched next to them, see tools/voice/fetch.mjs and
 *   tools/llm/fetch.mjs). The art is listed in assets/CREDITS.md.
 *
 * Identical texts (the Apache-2.0 licence of many crates) are printed once and referred to.
 */

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, realpathSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web');
const LICENSE_FILE = /^(licen[cs]e|copying|unlicense|notice)([-._].*)?$/i;

/**
 * Packages whose npm release has no licence file: the text from their repository.
 * ONNX Runtime: https://github.com/microsoft/onnxruntime/blob/main/LICENSE
 */
const KNOWN = {
  'onnxruntime-web': ['LICENSE (github.com/microsoft/onnxruntime)', mit('Copyright (c) Microsoft Corporation')],
  'onnxruntime-common': ['LICENSE (github.com/microsoft/onnxruntime)', mit('Copyright (c) Microsoft Corporation')],
};

/**
 * Shipped or adapted work with no npm package to read the licence from: its texts, as published,
 * are kept in tools/release/texts/.
 */
const KEPT = [
  ['zstddec (decoders/zstddec.wasm, built by Babylon.js), with Zstandard', 'MIT; BSD-3-Clause', 'https://github.com/donmccurdy/zstddec', 'zstddec-LICENSE'],
];
const ADAPTED = [
  ["TalkingHead's emoji expressions, adapted for the residents' faces (render/babylon/characters/face.ts)", 'MIT', 'https://github.com/met4citizen/TalkingHead', 'TalkingHead-LICENSE'],
];

function kept([name, license, url, file]) {
  return { name, version: '', license, url, files: [[file, readFileSync(join(ROOT, 'tools/release/texts', file), 'utf8')]] };
}

function mit(copyright) {
  return `MIT License

${copyright}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
`;
}

/** The directory of the npm package a bundled module came from (none for our own code). */
export function packageDir(id) {
  const path = id.replace(/^\0/, '').split('?')[0];
  const i = path.lastIndexOf('/node_modules/');
  if (i < 0) return undefined;
  const parts = path.slice(i + '/node_modules/'.length).split('/');
  const name = parts[0].startsWith('@') ? `${parts[0]}/${parts[1]}` : parts[0];
  return path.slice(0, i + '/node_modules/'.length) + name;
}

/** The licence files in `dir`: [file name, text]. */
function licenceFiles(dir) {
  return readdirSync(dir)
    .filter((f) => LICENSE_FILE.test(f))
    .sort()
    .map((f) => [f, readFileSync(join(dir, f), 'utf8')]);
}

function npmPackage(dir) {
  const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
  const license = typeof pkg.license === 'string' ? pkg.license : (pkg.license?.type ?? pkg.licenses?.map((l) => l.type).join(' OR ') ?? 'unknown');
  const repo = typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url;
  const url = (pkg.homepage ?? repo ?? `https://www.npmjs.com/package/${pkg.name}`)
    .replace(/^(github:)?([\w.-]+\/[\w.-]+)$/, 'https://github.com/$2')
    .replace(/^git\+/, '')
    .replace(/\.git$/, '');
  const files = licenceFiles(dir);
  return { name: pkg.name, version: pkg.version, license, url, files: files.length ? files : KNOWN[pkg.name] ? [KNOWN[pkg.name]] : [] };
}

/** The crates (not ours) that sim-wasm and voice-wasm depend on, as built for wasm32. */
function rustCrates() {
  const meta = JSON.parse(
    execFileSync('cargo', ['metadata', '--format-version', '1', '--locked', '--offline', '--filter-platform', 'wasm32-unknown-unknown'], { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 }),
  );
  const byId = new Map(meta.packages.map((p) => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
  const seen = new Set();
  const stack = meta.packages.filter((p) => p.name === 'sim-wasm' || p.name === 'voice-wasm').map((p) => p.id);
  if (stack.length !== 2) throw new Error('licenses: sim-wasm or voice-wasm missing from cargo metadata');
  while (stack.length) {
    const id = stack.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    // Normal dependencies (proc macros included); not build scripts' or tests'.
    for (const dep of nodes.get(id).deps) if (dep.dep_kinds.some((k) => k.kind === null)) stack.push(dep.pkg);
  }
  return [...seen]
    .map((id) => byId.get(id))
    .filter((p) => p.source !== null)
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((p) => {
      const dir = p.manifest_path.replace(/[/\\]Cargo\.toml$/, '');
      return { name: p.name, version: p.version, license: p.license ?? 'unknown', url: p.repository ?? `https://crates.io/crates/${p.name}`, files: licenceFiles(dir) };
    });
}

/**
 * The file's text. `packageDirs`: the npm packages the bundles hold; `header`: the first lines.
 * Fails if something has no licence text at all (add it to `KNOWN` above).
 */
export function thirdPartyLicenses({ packageDirs, header }) {
  const npm = [...new Set(packageDirs.map((d) => realpathSync(d)))].map(npmPackage).sort((a, b) => a.name.localeCompare(b.name));
  const rust = rustCrates();
  const asIs = (name, files) => {
    const pkg = npmPackage(realpathSync(join(WEB, 'node_modules', name)));
    return { ...pkg, name: `${name} ${pkg.version} (${files})`, version: '' };
  };
  const shipped = [asIs('@fontsource-variable/inter', 'the Inter font: assets/inter-*.woff2, its licence also in assets/inter-LICENSE.txt'), asIs('meshoptimizer', 'decoders/meshopt_decoder.js'), ...KEPT.map(kept)];
  const data = [
    ['Supertonic 3 voice model (voice/supertonic-3/)', 'BigScience OpenRAIL-M', 'https://huggingface.co/Supertone/supertonic-3', 'web/public/voice/supertonic-3/LICENSE.txt'],
    ['Misaki US English dictionaries (voice/en-us.lexz)', 'Apache-2.0', 'https://github.com/hexgrad/misaki', 'web/public/voice/misaki-LICENSE.txt'],
    ['LFM2.5-350M language model (llm/lfm2.5-350m/)', 'LFM Open License v1.0', 'https://huggingface.co/LiquidAI/LFM2.5-350M', 'web/public/llm/lfm2.5-350m/LICENSE.txt'],
  ].map(([name, license, url, file]) => {
    const path = join(ROOT, file);
    if (!existsSync(path)) throw new Error(`licenses: ${file} missing (run pnpm voice and pnpm llm)`);
    return { name, version: '', license, url, files: [[file.split('/').pop(), readFileSync(path, 'utf8')]] };
  });

  const texts = new Map();
  const ref = ([, text]) => {
    const key = createHash('sha256').update(text.replace(/\s+/g, ' ').trim()).digest('hex');
    if (!texts.has(key)) texts.set(key, { n: texts.size + 1, text, users: [] });
    return texts.get(key);
  };
  const out = [header.trim(), ''];
  const section = (title, list) => {
    out.push(`== ${title} ==`, '');
    for (const p of list) {
      if (!p.files.length) throw new Error(`licenses: no licence text for ${p.name} ${p.version}`);
      const refs = p.files.map((f) => {
        const t = ref(f);
        t.users.push(`${p.name} ${p.version}`.trim());
        return `[${t.n}] ${f[0]}`;
      });
      out.push(`${p.name}${p.version ? ` ${p.version}` : ''}: ${p.license}, ${p.url}`, `  licence text: ${refs.join(', ')}`);
    }
    out.push('');
  };
  section('JavaScript bundled into assets/*.js (npm packages)', npm);
  section('Rust crates compiled into assets/sim_wasm_bg-*.wasm and voice_wasm_bg-*.wasm', rust);
  section('Files shipped as they are', shipped);
  section("Adapted into the game's own code", ADAPTED.map(kept));
  section('Voice model, dictionary and language model (the copies next to them are the same texts)', data);
  out.push('== Licence texts ==', '');
  for (const t of texts.values()) out.push(`---- [${t.n}] for ${t.users.join(', ')} ----`, '', t.text.trim(), '', '');
  return { text: out.join('\n'), counts: { npm: npm.length, rust: rust.length, texts: texts.size } };
}
