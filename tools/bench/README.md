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

# Checking things in the browser

`check.mjs` takes the screenshots an object task needs, headless on the real GPU and at 1 % volume, with
the camera never taking over on its own (the director's delay 0) and free will off (`--free-will` keeps
it). It starts its own Vite from this checkout on the port you give (`--port`), or uses `--url` of a server
you started yourself. Pictures, and `<what>.json` with what each one shows, go to
`bench-results/check-<time>/` or `--out <dir>`.

| Check | Does |
|---|---|
| `gallery` | `/gallery.html?keys=…` from the front right. `--alpha --beta --radius --target --cols` |
| `clips` | `/clips.html?keys=…` held at each of `--times` (default `0,0.3,0.6,1,99`; 99 is open), one picture each. `--clip --colors --target` |
| `play` | Opens `?state=<--state, default bathroom-check>&debug`, adds money, buys `--object` at `--at x,z,rot` (`--turn`, `--colors`), sends `--use` (an interaction id or index) for `--sim`, waits up to `--wait` s until `__sims()` says the resident is at it and using it, then takes `--shots` pictures `--every` ms, logging the resident's pose and the clips playing on it. The camera follows the resident (`--radius`, `--beta`, `--alpha`, `--y`). |

Common options: `--quality low|medium|high` (`?quality=`), `--renderer webgl`, `--size 1280x800`,
`--extra 'k=v&k=v'`, `--headed` (a window, only to measure timing).

```sh
node tools/bench/check.mjs gallery --port 5227 --keys model.dartboard,model.tableClock
node tools/bench/check.mjs clips --port 5227 --keys model.fridge --clip door_open --times 0,0.2,0.4,99
node tools/bench/check.mjs play --port 5227 --object toiletClassic --at 23,14,0 --use use --shots 6 --every 500
node tools/bench/check.mjs play --port 5227 --object toiletClassic --at 24,17,1 --state office-night --renderer webgl --quality low
```

## States for checks

The simulation's tests write these with `DUMP_STATES=1 cargo test -p sim-core --test check_states`
(`crates/sim-core/tests/check_states.rs`) into `debug-states/` (gitignored); open one with
`?state=<name>&debug` on the dev server. Each test checks its state is still fit for use as content changes.

| State | What's in it |
|---|---|
| `bathroom-check` | The house of the town `wants.rs` makes (seed 21), 10:00 on day 1. Resident 0 at home, doing nothing, no job, free will off, every need at 0.8: nothing pulls them away from a `use`, and a use that fills a need lasts (a full need ends it at once). $1,000,000. Free wall in the living room at `[23, 14, rot]` and `[24, 17, rot]`. |
| `needy-check` | The same with free will on and low needs (bladder 0.05, hunger 0.2, energy 0.25, hygiene 0.3, the rest 0.5): they go to the toilet, then eat, wash and sleep on their own. |
| `office-night` | `bathroom-check` at 21:30: lamps lit, night lighting. |
| `town-things` | A public lot ("Town things", plot 1: `__showPlot(1)`) with every community thing in rows by group (street, playground, stalls, farm; the shops against a wall at the back), all facing the street, beside a home with a small house. Resident 0 stands at the lot's front, free will off, needs at 0.8. |

The game's own Free will setting (on by default) is applied to the player's household as the page opens,
over what the state says: send `setAutonomy` off (`check.mjs` does), or turn it off in Settings. Other
states the tests write: `wants-asking` (its resident is asking to buy something and doesn't take a `use`),
`group-talk`, `basement`, `water-sounds` and more (`grep -rn dump_state crates/sim-core/tests`).

## Hooks under `?debug`

| Hook | Does |
|---|---|
| `__sims()` | Every resident as the newest snapshot shows them: `x`, `z`, `storey`, `yaw`, `pose` (`stand`, `sit`, `lie`), `moving`, the `object` they use or walk to (`__world().objects` id), its `def` and `interaction` ids, `using` (at it, not walking), `action` (the animation), `held`, `recipe`, `partner`, `needs` by id (the player's household and the inspected), `doing` and `queue` (as the resident's panel). Wait on it rather than timing screenshots: `page.waitForFunction(() => __sims()[0].using && __sims()[0].pose === 'sit')`. |
| `__world()` | The lot as drawn: objects, walls, plots. |
| `__send(command)` | Any simulation command (`web/src/core/protocol.ts`): `buy`, `use`, `setAutonomy`, `setSpeed`, `cancel`, `adopt`… |
| `__addFunds(amount)` | Money for the player's household (negative takes it away); for long sessions, where buys start failing ("not bought"). |
| `__camera(pose?)` | Reads the camera's pose; with a (partial) pose `{alpha, beta, radius, target: [x, y, z]}` or `{y}` (the height it looks at) puts it there. Stops the camera watching on its own. |
| `__controls.follow(id, view?)` | The camera follows a resident; `view` `{radius, beta, alpha, y}` also zooms and aims (a dartboard above the resident: `y: 1.6`). `follow(null)` stops. The rest of `__controls` is the HUD's controls. |
| `__content` | The game's content: `__content.object('<id>').interactions` for interaction indices. |
| `__renderer`, `__scene` | The renderer (`__renderer.motion.states.get(objectId)`: the clips playing on an object) and the Babylon scene. |
| `__showPlot(i)`, `__items()`, `__crash(what)` | Look at another lot; the food and dishes about; fail on purpose. |

## Sharing one machine with other sessions

Several sessions (and agents) often work at once, each in its own worktree:

- **Ports.** Start your own dev server from your own checkout, on a port of your own, with `--strictPort`;
  check it's free first (`lsof -iTCP:<port> -sTCP:LISTEN`). Never use 5199 (the main checkout's) or a
  server another session started: with the port taken, `--strictPort` fails, and a `curl` or browser
  would answer from someone else's code. A port given up is soon taken by someone else: pick a fresh one
  each run, and stop your server when you're done. `check.mjs --port` refuses a port in use.
- **Reloads.** Other sessions' edits in a shared checkout reload the dev server's page mid-run, and a
  content file being written can be invalid for a moment ("the game data failed to load"). Work in a
  worktree; if a run fails that way, run it again.
- **Scratch.** Give every session or agent its own folder in the scratchpad (as with ports): parallel
  agents writing helper scripts into one folder overwrote each other's. Keep screenshots, copies of
  content and probes there, never in the repo.
- **Shared files.** Write a shared file only after its new text is built and parsed (`t = fix(s);
  json.loads(t); open(p, 'w').write(t)`: opening it for writing first empties it, and once emptied
  `furniture.json`). Moving items into Validate and back: `cargo content`, which does that.
