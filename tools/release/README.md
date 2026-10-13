# Release tools

| Script | Does |
|---|---|
| `version.mjs` | One version for the whole game: `web/package.json` (shown in the menu and Credits), the Rust workspace in `Cargo.toml`, and the workspace crates' four entries in `Cargo.lock`. `node tools/release/version.mjs` prints it and fails if they differ (CI runs it; with `--locked` everywhere, a stale lock would fail the build anyway). `node tools/release/version.mjs 0.3.0` sets all of them. |
| `wasm-opt.mjs` | Optimises the simulation's and the voice's WebAssembly with binaryen's `wasm-opt -O3` (the exact-pinned `binaryen` devDependency) and strips the name and producers sections. The features are passed explicitly; never `-tnh` or `-iit`, because with `panic = "abort"` every panic is a trap. The production build runs it on the built files (`vite.config.ts`, the `release-wasm` plugin), about a minute the first time and then cached in `web/.assets/wasm-opt/`; the dev server keeps the WASM with its names. |
| `licenses.mjs` | Writes `dist/THIRD-PARTY-LICENSES.txt` in production builds: the npm packages found in the bundles (the page's and every worker's), the Rust crates in both WASM files (`cargo metadata --locked --offline`), files shipped as they are (Inter, the meshopt decoder) and the licences of the voice model, the dictionary and the language model, each text once. The build fails if a licence text is missing. |

How to make a release (the dry run, the tag, the checks after) is in the README, under [Versions and releasing](../../README.md#versions-and-releasing). The workflow is `.github/workflows/pages.yml`.

Two numbers to know:
- `wasm-opt`: on an M3 Pro, the simulation's WASM took 48–55 s from cold and the voice's 3 s. Expect 1.5–3 minutes on a CI runner. It made the two files 620 KB smaller raw and 44 KB smaller gzipped, with no change in speed.
- The site (`web/dist`) was 522 MB when built for the 0.2.0 release review, most of it the voice and language models. CI fails over 900 MB, below Pages' limit of 1 GB.

CI keeps the WASM with its function names as an artifact (`wasm-with-names-<commit>`) for 90 days, to read the stack of a crash report.
