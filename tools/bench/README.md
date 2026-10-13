# Rendering benchmark

The game has a rendering benchmark built in: open it with `?bench=<scene>` (`web/src/debug/bench.ts`, flags in `benchFlags.ts`). These scripts run it over a matrix in a real browser and make its scenes. The flags, what a result holds, and the numbers measured with it are in [docs/design/graphics.md](../../docs/design/graphics.md).

| Script | Does |
|---|---|
| `render.mjs` | Starts Vite (or uses `--url`), opens `?bench=` in Chromium on the real GPU at 1920×1080 and device pixel ratio 1, once per preset × backend × scenario × CPU slow-down × diagnostic, and writes JSON and a markdown table to `bench-results/<time>/` (medians with `--runs`). `--shots <dir>` takes reference screenshots of 9 fixed poses; `--compare <dirA> <dirB>` counts the pixels that differ. |
| `scene.mjs` | Makes a scene, `web/public/bench/<name>.json.gz`: a new game as the menus make one, with a household of four in the first home for sale, played at full speed to `--hour` on day 1, paused and saved. |

Both need `pnpm wasm` first and Playwright (`PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for a browser of your own: Chrome or Chrome for Testing, not `chrome-headless-shell`, which draws in software). Measurements run in a window, so their timing is the real display's. Shots and `scene.mjs`, which time nothing, run headless: no window takes the focus, and it's still the real GPU. `--headless` and `--headed` choose for either. A window's shots come out about 2 of 255 brighter than headless ones (the screen's colour profile), so compare shots taken the same way: two headless runs of the same build differ in at most 0.12 % of their pixels. Both play the game at 1 % volume (`quiet.mjs`).

```sh
# The production build (what players get), presets × both backends, no vsync, median of 3
node tools/bench/render.mjs --preview --build --presets low,medium,high --backends webgpu,webgl --scenarios orbit-day,orbit-night --throughput --runs 3

# What a feature costs: Medium with and without shadows and nature
node tools/bench/render.mjs --preview --presets medium --diags none,noshadows,nonature --throughput --runs 3

# At 3x CPU slow-down (CDP), standing in for slower laptops
node tools/bench/render.mjs --preview --throttle 1,3

# Does Medium still look the same? Shots of a change against shots of the base
node tools/bench/render.mjs --preview --shots /tmp/shots-new --presets medium
node tools/bench/render.mjs --compare /tmp/shots-base /tmp/shots-new --out /tmp/shots-diff

# A new scene
node tools/bench/scene.mjs --name village --world medium --hour 18
```

The options are documented at the top of each script. To compare two builds, measure them back to back on the same machine, with nothing else running, and take medians of at least 3 runs; GPU times on Apple hardware vary by about ±1 ms between runs. Shots of two runs of the same build taken in a window differed by at most 1 of 255 in a few pixels, apart from a far road's lane markings (P1-17 in the release review); headless ones in at most 0.12 % of their pixels (above).

On a laptop without the repo, open the deployed game instead, e.g. `https://krecks.github.io/idyll-lives/?bench=village&quality=low&hour=13&res=1` (add `&renderer=webgl` for WebGL2), and press "Copy result" when it's done.
