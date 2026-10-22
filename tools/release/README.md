# Release tools

| Script | Does |
|---|---|
| `version.mjs` | One version for the whole game: `web/package.json` (shown in the menu and Credits), the Rust workspace in `Cargo.toml`, and the five workspace crates' entries in `Cargo.lock`. `node tools/release/version.mjs` prints it and fails if they differ (CI runs it; with `--locked` everywhere, a stale lock would fail the build anyway). `node tools/release/version.mjs 0.3.0` sets all of them. |
| `wasm-opt.mjs` | Optimises the simulation's and the voice's WebAssembly with binaryen's `wasm-opt -O3` (the exact-pinned `binaryen` devDependency) and strips the name and producers sections (`--names` keeps the name section, for CI's crash-stack artifact). The features are passed explicitly; never `-tnh` or `-iit`, because with `panic = "abort"` every panic is a trap. The production build runs it on the built files (`vite.config.ts`, the `release-wasm` plugin), about a minute the first time and then cached in `web/.assets/wasm-opt/`; the dev server keeps the WASM with its names. |
| `licenses.mjs` | Writes `dist/THIRD-PARTY-LICENSES.txt` in production builds: the npm packages found in the bundles (the page's and every worker's), the Rust crates in both WASM files (`cargo metadata --locked --offline`), files shipped as they are (Inter, the meshopt decoder), the texts kept in `texts/` (the Zstandard decoder's, shipped as a file, and TalkingHead's, whose expressions are adapted in code) and the licences of the voice model, the dictionary and the language model, each text once. The build fails if a licence text is missing. |

How to make a release is in the README, under [Versions and releasing](../../README.md#versions-and-releasing): the version and the CHANGELOG section (its date set by hand), `git push origin main`, the dry run (watched by its run id), the tag, the live checks of the large files, and `gh release create … --verify-tag`. The workflow is `.github/workflows/pages.yml`. If a release goes wrong, fix forward or redeploy the last good 0.3.x: 0.3.0 can't open 0.3.1's saves (format 32), 0.2.0 can't open 0.3.0's (format 29), and v0.1.0 can't open 0.2.0's saves database.

Numbers to know:
- `wasm-opt`: on an M3 Pro, the simulation's WASM took 48–55 s from cold and the voice's 3 s. Expect 1.5–3 minutes on a CI runner from cold; CI keeps `web/.assets/wasm-opt/` between runs (`actions/cache`), so an unchanged WASM is instant. It made the two files 620 KB smaller raw and 44 KB smaller gzipped, with no change in speed.
- The site (`web/dist`) was 489 MB in the first CI dry run (assets 56 MB, voice 181 MB, language model 248 MB; the largest file the language model's weights, 255,530,880 bytes). CI fails over 900 MB, below Pages' limit of 1 GB.
- The live checks after a release: `llm/lfm2.5-350m/model_q4.onnx_dataz` is 255,530,880 bytes and `voice/supertonic-3/vector_estimator.onnxz` 119,568,968 bytes.

CI keeps a WASM with function names as an artifact (`wasm-with-names-<commit>`) for 90 days: the site's own optimisations with the name section kept (`wasm-opt.mjs --names`), so its function numbers match a crash stack from the site.
