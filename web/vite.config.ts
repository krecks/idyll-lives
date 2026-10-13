import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { cp, mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { join, resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';

// SharedArrayBuffer (zero-copy sim -> render snapshots, threads for the language model on the
// CPU) requires cross-origin isolation. Where production hosting can't send these two headers
// (GitHub Pages), a service worker adds them (public/isolation-sw.js, registered by index.html).
const crossOriginIsolation = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as { version: string };

/** The commit the game is built from, shown next to the version (Credits, bug reports): CI's, else git's HEAD. */
function commit(): string {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA.slice(0, 7);
  try {
    return execFileSync('git', ['rev-parse', '--short', 'HEAD'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch {
    return 'unknown';
  }
}
const COMMIT = commit();
/**
 * This build's id, a query on runtime fetches from public/ (src/assets/build.ts): the commit in CI;
 * locally the commit and the build time, as the tree may hold changes. No dots: Babylon reads a
 * texture's type from the URL's last dot.
 */
const BUILD_ID = process.env.GITHUB_SHA ? COMMIT : `${COMMIT}-${Date.now().toString(36)}`;

/** Where debug reports from the game land (dev server only): `<repo>/debug-reports/<name>/`. */
const REPORTS = fileURLToPath(new URL('../debug-reports', import.meta.url));
/** Game states written by the simulation's tests (`DUMP_STATES=1 cargo test`): `<repo>/debug-states/<name>.json`. */
const STATES = fileURLToPath(new URL('../debug-states', import.meta.url));
/** Rendering benchmark results (`?bench=<scene>`, src/debug/bench.ts): `<repo>/bench-results/<name>.json`. */
const BENCH_RESULTS = fileURLToPath(new URL('../bench-results', import.meta.url));

/**
 * Dev server endpoints for debug reports (see src/debug/report.ts):
 * POST /__debug/report writes report.json, save.txt and screenshot.jpg into a new folder;
 * GET /__debug/reports/<name>/<file> reads them back (`?debugReport=<name>` loads one);
 * GET /__debug/states/<name> reads a game state the tests wrote (`?state=<name>` loads one);
 * POST /__debug/bench writes a benchmark's result to `bench-results/`.
 */
function debugReports(): Plugin {
  return {
    name: 'debug-reports',
    apply: 'serve',
    configureServer(server) {
      server.middlewares.use('/__debug/report', async (req, res) => {
        if (req.method !== 'POST') return void ((res.statusCode = 405), res.end());
        try {
          const chunks: Buffer[] = [];
          for await (const chunk of req) chunks.push(chunk as Buffer);
          const { report, save, screenshot } = JSON.parse(Buffer.concat(chunks).toString('utf8')) as {
            report: { createdAt: string; note: string };
            save: string;
            screenshot: string | null;
          };
          const slug = report.note.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 40);
          const name = report.createdAt.replace(/\.\d+Z$/, '').replace(/[:T]/g, '-') + (slug ? `-${slug}` : '');
          const dir = join(REPORTS, name);
          await mkdir(dir, { recursive: true });
          await writeFile(join(dir, 'report.json'), JSON.stringify(report, null, 2));
          await writeFile(join(dir, 'save.txt'), save);
          if (screenshot) await writeFile(join(dir, 'screenshot.jpg'), Buffer.from(screenshot.slice(screenshot.indexOf(',') + 1), 'base64'));
          res.setHeader('Content-Type', 'application/json');
          res.end(JSON.stringify({ path: `debug-reports/${name}` }));
          server.config.logger.info(`[debug] report saved to debug-reports/${name}`, { timestamp: true });
        } catch (err) {
          res.statusCode = 500;
          res.end(String(err));
        }
      });
      server.middlewares.use('/__debug/bench', async (req, res) => {
        if (req.method !== 'POST') return void ((res.statusCode = 405), res.end());
        try {
          const chunks: Buffer[] = [];
          for await (const chunk of req) chunks.push(chunk as Buffer);
          const body = Buffer.concat(chunks).toString('utf8');
          const result = JSON.parse(body) as { startedAt: string; scene: string; label?: string };
          const slug = `${result.scene}-${result.label ?? ''}`.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 60);
          const name = `${result.startedAt.replace(/\.\d+Z$/, '').replace(/[:T]/g, '-')}-${slug}.json`;
          await mkdir(BENCH_RESULTS, { recursive: true });
          await writeFile(join(BENCH_RESULTS, name), JSON.stringify(result, null, 2));
          res.setHeader('Content-Type', 'application/json');
          res.end(JSON.stringify({ path: `bench-results/${name}` }));
        } catch (err) {
          res.statusCode = 500;
          res.end(String(err));
        }
      });
      server.middlewares.use('/__debug/states/', async (req, res) => {
        const name = decodeURIComponent((req.url ?? '').split('?')[0]).replace(/^\//, '');
        if (!/^[\w.-]+$/.test(name) || name.includes('..')) return void ((res.statusCode = 404), res.end());
        try {
          res.setHeader('Content-Type', 'application/json');
          res.end(await readFile(join(STATES, `${name.replace(/\.json$/, '')}.json`)));
        } catch {
          res.statusCode = 404;
          res.end();
        }
      });
      server.middlewares.use('/__debug/reports/', async (req, res) => {
        const [name, file] = decodeURIComponent((req.url ?? '').split('?')[0]).replace(/^\//, '').split('/');
        if (!name || !file || name.includes('..') || !['report.json', 'save.txt', 'screenshot.jpg'].includes(file)) {
          return void ((res.statusCode = 404), res.end());
        }
        try {
          res.end(await readFile(join(REPORTS, name, file)));
        } catch {
          res.statusCode = 404;
          res.end();
        }
      });
    },
  };
}

/**
 * Compressed copies of the assets (KTX2 textures, meshopt models, gzipped character data; see
 * tools/assets/optimize.mjs): encoded when the build starts (only files that changed since the
 * last build, the rest come from web/.assets/cache) and copied into dist/assets when it ends,
 * next to the sources. `LL_ASSETS=source` builds without them. The dev server serves sources.
 */
function optimizedAssets(): Plugin {
  const skip = process.env.LL_ASSETS === 'source';
  let outDir = '';
  let copies = '';
  return {
    name: 'optimized-assets',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    async buildStart() {
      if (skip) return;
      // Imported by URL so it runs from its own file (it starts worker threads on itself).
      const tool = await import(new URL('../tools/assets/optimize.mjs', import.meta.url).href);
      await tool.optimizeAssets({ log: (line: string) => this.info(line) });
      copies = tool.OUT;
    },
    async closeBundle() {
      if (copies) await cp(copies, join(outDir, 'assets'), { recursive: true });
    },
  };
}

/**
 * The release WebAssembly: once the build has written the simulation's and the voice's wasm, they
 * are optimised and stripped by binaryen's wasm-opt (tools/release/wasm-opt.mjs: -O3, no name or
 * producers sections; about a minute the first time, then cached). The dev server, and
 * `npm run wasm` behind it, keep them as built.
 */
function releaseWasm(): Plugin {
  let outDir = '';
  let log: (line: string) => void = () => {};
  return {
    name: 'release-wasm',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
      log = (line) => config.logger.info(line, { timestamp: true });
    },
    async closeBundle() {
      const tool = await import(new URL('../tools/release/wasm-opt.mjs', import.meta.url).href);
      const dir = join(outDir, 'assets');
      const files = (await readdir(dir)).filter((f) => /^(sim|voice)_wasm_bg-[\w-]+\.wasm$/.test(f));
      if (!files.some((f) => f.startsWith('sim_'))) throw new Error(`release-wasm: no sim_wasm_bg-*.wasm in ${dir}`);
      await Promise.all(
        files.map(async (f) => {
          const start = performance.now();
          const { before, after, cached } = await tool.optimizeWasm(join(dir, f));
          log(`wasm-opt ${f}: ${before} -> ${after} bytes${cached ? ' (cached)' : `, ${((performance.now() - start) / 1000).toFixed(0)} s`}`);
        }),
      );
    },
  };
}

/** The npm packages whose code the bundles hold (the page's and the workers'), by directory. */
const bundledPackages = new Set<string>();

/** Notes which npm packages end up in a bundle (the page's, and each worker's: see `worker.plugins`). */
function collectPackages(): Plugin {
  return {
    name: 'collect-packages',
    apply: 'build',
    async generateBundle(_, bundle) {
      const { packageDir } = await import(new URL('../tools/release/licenses.mjs', import.meta.url).href);
      for (const chunk of Object.values(bundle)) {
        if (chunk.type !== 'chunk') continue;
        for (const [id, mod] of Object.entries(chunk.modules)) {
          const dir = mod.renderedLength > 0 ? packageDir(id) : undefined;
          if (dir) bundledPackages.add(dir);
        }
      }
    },
  };
}

/**
 * The licences of what the game ships that others made (tools/release/licenses.mjs):
 * dist/THIRD-PARTY-LICENSES.txt, and Inter's licence next to its font files. Linked from Credits.
 */
function thirdPartyLicenses(): Plugin {
  let outDir = '';
  let log: (line: string) => void = () => {};
  return {
    name: 'third-party-licenses',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
      log = (line) => config.logger.info(line, { timestamp: true });
    },
    async closeBundle() {
      const tool = await import(new URL('../tools/release/licenses.mjs', import.meta.url).href);
      const header = `Idyll Lives ${pkg.version} (build ${COMMIT}): software and models by others that the game includes, and their licences.
The game itself is free software under the GNU General Public License v3.0 or later. The art is listed in assets/CREDITS.md.`;
      const { text, counts } = tool.thirdPartyLicenses({ packageDirs: [...bundledPackages], header });
      await writeFile(join(outDir, 'THIRD-PARTY-LICENSES.txt'), text);
      await cp(fileURLToPath(new URL('./node_modules/@fontsource-variable/inter/LICENSE', import.meta.url)), join(outDir, 'assets/inter-LICENSE.txt'));
      log(`THIRD-PARTY-LICENSES.txt: ${counts.npm} npm packages, ${counts.rust} Rust crates, ${counts.texts} licence texts`);
    },
  };
}

export default defineConfig({
  plugins: [svelte(), debugReports(), optimizedAssets(), releaseWasm(), collectPackages(), thirdPartyLicenses()],
  // The game's version (one number for web and Rust; `node ../tools/release/version.mjs <x.y.z>` bumps it).
  define: { __APP_VERSION__: JSON.stringify(pkg.version), __APP_COMMIT__: JSON.stringify(COMMIT), __BUILD_ID__: JSON.stringify(BUILD_ID) },
  server: { headers: crossOriginIsolation },
  preview: { headers: crossOriginIsolation },
  worker: { format: 'es', plugins: () => [collectPackages()] },
  // The voice and language model workers' imports: Vite's start-up scan doesn't reach worker
  // imports, so it would find them only when the first line is spoken, re-bundle and reload the page.
  optimizeDeps: { include: ['onnxruntime-web/wasm', 'onnxruntime-web/webgpu', '@huggingface/tokenizers'] },
  // The main chunk is about 1.7 MB (Babylon by module, see render/babylon/core.ts): warn if it grows.
  build: { target: 'es2022', chunkSizeWarningLimit: 2000 },
});
