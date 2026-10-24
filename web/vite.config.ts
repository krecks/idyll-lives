import { defineConfig, searchForWorkspaceRoot, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { cp, mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { basename, join, resolve } from 'node:path';
import { lstatSync, readFileSync, realpathSync } from 'node:fs';
import { execFileSync, spawnSync } from 'node:child_process';

// SharedArrayBuffer (zero-copy sim -> render snapshots, threads for the language model on the
// CPU) requires cross-origin isolation. Where production hosting can't send these two headers
// (GitHub Pages), a service worker adds them (public/isolation-sw.js, registered by index.html).
const crossOriginIsolation = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

/**
 * In a worktree whose `node_modules` is a link to the main checkout's: the server may read the linked
 * packages (else Vite refuses them with 403), and keeps its pre-bundled packages in this worktree
 * (`web/.vite-cache`) rather than in the shared `node_modules/.vite`, which every other worktree's
 * server would re-optimise and wipe under it. Null in a checkout with its own `node_modules`.
 */
const linkedModules = (() => {
  const dir = fileURLToPath(new URL('./node_modules', import.meta.url));
  try {
    return lstatSync(dir).isSymbolicLink() ? realpathSync(dir) : null;
  } catch {
    return null;
  }
})();

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
 * locally the commit and the build time, as the tree may hold changes. No dots, so a URL's last dot
 * is still its file's extension (loaders read a file's type from it).
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
            save: string | null;
            screenshot: string | null;
          };
          const slug = report.note.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 40);
          const name = report.createdAt.replace(/\.\d+Z$/, '').replace(/[:T]/g, '-') + (slug ? `-${slug}` : '');
          const dir = join(REPORTS, name);
          await mkdir(dir, { recursive: true });
          await writeFile(join(dir, 'report.json'), JSON.stringify(report, null, 2));
          // No save when the game had stopped (or none ran): the report alone.
          if (save !== null) await writeFile(join(dir, 'save.txt'), save);
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
 * One of the repo's tools (`crates/tools`, through the cargo aliases in .cargo/config.toml), run
 * from this folder: its output lines, or a build-stopping error.
 */
function runTool(args: string[]): string[] {
  const result = spawnSync('cargo', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  if (result.status !== 0) throw new Error(`cargo ${args[0]}: ${(result.stderr || result.error?.message || 'failed').trim()}`);
  return result.stdout.trim().split('\n').filter((line) => line.length > 0);
}

/**
 * The release WebAssembly: once the build has written the simulation's and the voice's wasm, they
 * are optimised and stripped by binaryen's wasm-opt (`cargo wasm-opt`: -O3, no name or producers
 * sections; about a minute the first time, then cached). The dev server, and `bun run wasm` behind
 * it, keep them as built.
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
      const dir = join(outDir, 'assets');
      const files = (await readdir(dir)).filter((f) => /^(sim|voice)_wasm_bg-[\w-]+\.wasm$/.test(f));
      if (!files.some((f) => f.startsWith('sim_'))) throw new Error(`release-wasm: no sim_wasm_bg-*.wasm in ${dir}`);
      for (const line of runTool(['wasm-opt', ...files.map((f) => join(dir, f))])) log(line);
    },
  };
}

/**
 * The sky's lighting starts downloading with the page, next to the game's script and data, instead
 * of once the renderer starts and asks for it: a script in `index.html` fetches it, and
 * `render/pc/env.ts` takes the bytes (`self.__skyEnv`): the base manifest's `environment.sky`, a
 * `.pcenv` (0.3 MB, `cargo envconv`). (Not a `<link rel=preload>`: under the isolation service
 * worker Chrome doesn't hand a preload to `fetch()`, and the file would download twice.)
 */
function earlyEnvironment(): Plugin {
  let base = '/';
  return {
    name: 'early-environment',
    configResolved(config) {
      base = config.base;
    },
    transformIndexHtml() {
      const manifest = JSON.parse(readFileSync(new URL('./public/assets/manifest.json', import.meta.url), 'utf8')) as { entries: Record<string, { url?: string }> };
      const file = manifest.entries['environment.sky']?.url;
      if (!file) return [];
      // As the asset registry resolves it (src/assets/registry.ts, build.ts), so the URLs match.
      const url = new URL(file, `http://x${base}assets/manifest.json`);
      const href = JSON.stringify(`${url.pathname}${url.search ? `${url.search}&` : '?'}v=${BUILD_ID}`);
      const children = `self.__skyEnv = { url: ${href}, bytes: fetch(${href}).then((res) => (res.ok ? res.arrayBuffer() : null)).catch(() => null) };`;
      return [{ tag: 'script', children, injectTo: 'head' }];
    },
  };
}

/**
 * The asset manifests ship without their spaces (public/ keeps them readable): the base one is
 * 1.46 MB pretty-printed and 0.59 MB minified.
 */
function minifiedManifests(): Plugin {
  let outDir = '';
  return {
    name: 'minified-manifests',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    async closeBundle() {
      const dir = join(outDir, 'assets');
      for (const file of await readdir(dir, { recursive: true })) {
        if (basename(file) !== 'manifest.json') continue;
        const path = join(dir, file);
        await writeFile(path, JSON.stringify(JSON.parse(await readFile(path, 'utf8'))));
      }
    },
  };
}

/**
 * `bundle.json`: what every start fetches from public/ in one file (src/assets/bundle.ts):
 * `content/base.json` and the files it includes, the house templates, the asset manifests and the
 * packs they list, and the index of compressed copies; each file's text, minified, by its path
 * below the base URL. The files still ship one by one beside it.
 */
function dataBundle(): Plugin {
  let outDir = '';
  return {
    name: 'data-bundle',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    async closeBundle() {
      const files: Record<string, string> = {};
      /** Adds a file below public/ (or `from`); returns its JSON. */
      const add = async (path: string, from = join(fileURLToPath(new URL('./public/', import.meta.url)), path)) => {
        const json = JSON.parse(await readFile(from, 'utf8')) as Record<string, unknown>;
        files[path] = JSON.stringify(json);
        return json;
      };
      // As Content.load and AssetRegistry.load resolve them: relative to the file that lists them.
      const relative = (path: string, name: string) => new URL(name, `http://x/${path}`).pathname.slice(1);
      const base = await add('content/base.json');
      for (const name of (base.include as string[] | undefined) ?? []) await add(relative('content/base.json', name));
      await add('content/houses.json');
      await add('content/housegen.json');
      const manifests = async (path: string): Promise<void> => {
        const manifest = await add(path);
        for (const pack of (manifest.packs as string[] | undefined) ?? []) await manifests(relative(path, pack));
      };
      await manifests('assets/manifest.json');
      if (process.env.LL_ASSETS !== 'source') {
        const tool = await import(new URL('../tools/assets/optimize.mjs', import.meta.url).href);
        await add('assets/optimized.json', join(tool.OUT, 'optimized.json'));
      }
      await writeFile(join(outDir, 'bundle.json'), JSON.stringify(files));
    },
  };
}

/**
 * The isolation service worker (public/isolation-sw.js) gets this build's id, which names the
 * cache it keeps the build's hashed files in: every build is a new version of the worker, so
 * returning players' browsers install it and drop the last build's cache. The worker tells the
 * build's files by their names in `assets/`, so no file from public/ may look like one.
 */
function isolationWorker(): Plugin {
  let outDir = '';
  return {
    name: 'isolation-worker',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    async closeBundle() {
      const file = join(outDir, 'isolation-sw.js');
      const code = await readFile(file, 'utf8');
      const stamped = code.replace("const BUILD = 'dev';", `const BUILD = '${BUILD_ID}';`);
      if (stamped === code) throw new Error("isolation-worker: no `const BUILD = 'dev';` in isolation-sw.js");
      await writeFile(file, stamped);
      const pattern = /const HASHED = \/(.+)\/;/.exec(code)?.[1];
      if (!pattern) throw new Error('isolation-worker: no `const HASHED = /…/;` in isolation-sw.js');
      const hashed = new RegExp(pattern);
      const own = await readdir(fileURLToPath(new URL('./public/assets/', import.meta.url))).catch(() => [] as string[]);
      const clash = own.filter((name) => hashed.test(name));
      if (clash.length) throw new Error(`isolation-worker: public/assets/${clash[0]} looks like a hashed build file (the service worker would keep it for good)`);
    },
  };
}

/** The npm packages whose code the bundles hold (the page's and the workers'), by directory. */
const bundledPackages = new Set<string>();

/** The directory of the npm package a bundled module came from (none for our own code). */
function packageDir(id: string): string | undefined {
  const path = id.replace(/^\0/, '').split('?')[0];
  const i = path.lastIndexOf('/node_modules/');
  if (i < 0) return undefined;
  const parts = path.slice(i + '/node_modules/'.length).split('/');
  const name = parts[0].startsWith('@') ? `${parts[0]}/${parts[1]}` : parts[0];
  return path.slice(0, i + '/node_modules/'.length) + name;
}

/** Notes which npm packages end up in a bundle (the page's, and each worker's: see `worker.plugins`). */
function collectPackages(): Plugin {
  return {
    name: 'collect-packages',
    apply: 'build',
    async generateBundle(_, bundle) {
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
 * The licences of what the game ships that others made (`cargo licenses`):
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
      const header = `Idyll Lives ${pkg.version} (build ${COMMIT}): software and models by others that the game includes, and their licences.
The game itself is free software under the GNU General Public License v3.0 or later. The art is listed in assets/CREDITS.md.`;
      const packages = [...bundledPackages].flatMap((dir) => ['--package', dir]);
      for (const line of runTool(['licenses', '--header', header, '--out', join(outDir, 'THIRD-PARTY-LICENSES.txt'), ...packages])) log(line);
      await cp(fileURLToPath(new URL('./node_modules/@fontsource-variable/inter/LICENSE', import.meta.url)), join(outDir, 'assets/inter-LICENSE.txt'));
    },
  };
}

export default defineConfig({
  plugins: [svelte(), debugReports(), earlyEnvironment(), optimizedAssets(), minifiedManifests(), dataBundle(), isolationWorker(), releaseWasm(), collectPackages(), thirdPartyLicenses()],
  // The game's version (one number for web and Rust; `cargo release-version <x.y.z>` bumps it).
  define: { __APP_VERSION__: JSON.stringify(pkg.version), __APP_COMMIT__: JSON.stringify(COMMIT), __BUILD_ID__: JSON.stringify(BUILD_ID) },
  server: { headers: crossOriginIsolation, ...(linkedModules ? { fs: { allow: [searchForWorkspaceRoot(process.cwd()), linkedModules] } } : {}) },
  ...(linkedModules ? { cacheDir: fileURLToPath(new URL('./.vite-cache', import.meta.url)) } : {}),
  preview: { headers: crossOriginIsolation },
  worker: { format: 'es', plugins: () => [collectPackages()] },
  // The voice and language model workers' imports: Vite's start-up scan doesn't reach worker
  // imports, so it would find them only when the first line is spoken, re-bundle and reload the page.
  optimizeDeps: { include: ['onnxruntime-web/wasm', 'onnxruntime-web/webgpu', '@huggingface/tokenizers'] },
  // The main chunk is about 2.5 MB (PlayCanvas's release build and the game): warn if it grows.
  build: { target: 'es2022', chunkSizeWarningLimit: 2600 },
});
