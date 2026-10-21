# Rendering benchmark

The game has a rendering benchmark built in: open it with `?bench=<scene>` (`web/src/debug/bench.ts`, flags in `benchFlags.ts`). These scripts run it over a matrix in a real browser and make its scenes. The flags, what a result holds, and the numbers measured with it are in [docs/design/graphics.md](../../docs/design/graphics.md).

| Script | Does |
|---|---|
| `render.mjs` | Starts Vite (or uses `--url`), opens `?bench=` in Chromium on the real GPU at 1920×1080 and device pixel ratio 1, once per preset × backend × scenario × CPU slow-down × diagnostic, and writes JSON and a markdown table to `bench-results/<time>/` (medians with `--runs`). `--shots <dir>` takes reference screenshots of 9 fixed poses; `--compare <dirA> <dirB>` counts the pixels that differ. |
| `tools.mjs` | Times the repo's Rust tools (`crates/tools`: the character build and check, the object gate, the content editor, the texture commands), the release binaries run directly, median of `--runs` after a warm-up plus peak memory, into `bench-results/tools-<time>/`. It compared them with the Python scripts they replaced while those were in the tree (2026-10-20: 2–14× faster, `tex process` 0.8×). |
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
you started yourself. Pictures, and `<what>-<run>.json` with what each one shows (and, per job, every HTTP
error, failed request, page error and console error), go to `bench-results/check-<time>/` or `--out <dir>`;
`<run>` is `--name <run>`, else the time the run started, so runs into one `--out` keep their logs.
Before any page it fetches the sim's wasm and the asset manifest through the server and stops unless they
answer 200 (a missing or linked wasm package otherwise gives pictures of an empty scene).

| Check | Does |
|---|---|
| `gallery` | `/gallery.html?keys=…` from the front right. `--alpha --beta --radius --target --cols`; `--each`: one picture per key, `gallery-<key>.png`, on one page; `--scale adult\|child\|all` stands residents beside each model; `--camera game` takes one picture per model with the game's Live camera aimed at it |
| `clips` | `/clips.html?keys=…` held at each of `--times` (default `0,0.3,0.6,1,99`; 99 is open), one picture each. `--each`: each key on its own, `clips-<key>-<clip>-t<t>.png`, one page for all. `--clip --colors --target` |
| `play` | Opens `?state=<--state, default bathroom-check>&debug`, adds money (and `--skills`), then for each of `--use-existing a,b` and `--objects a,b@x,z,rot,…` (or `--objects-file <path>`, below) on the same page: buys its `--with` helpers and it (or takes the state's own), runs each `--use` (`all`, the default, an interaction id or an index) for `--sim`, takes `--shots` (3) pictures `--every` (1000) ms once the resident is using it, framing the object, then sells what it bought and the resident walks back to where they started. One JSON line per use in `<out>/play-<run>.jsonl`. |

`play` in more detail:
- **Where.** `def@x,z,rot`, else `--at`, else on a `review-*` state (`review-stage`, `-night`,
  `review-garden`) a free spot by the object's layer and footprint (below), else wherever the game
  finds room (things on a surface always go there: the stage's counter).
- **Objects file.** One thing a line, `#` comments:
  `def[@x,z,rot | @existing] [with helper[@x,z,rot] helper…]`, e.g.
  `officeChair with deskSimple@10,13,2`, `cafeChair with cafeTable@10,13,0`,
  `stoolBarHigh with counter@10,13,0` (a stool needs a counter or a bar, not a table; the counter's
  back to it, as a counter facing the stool would have its front tile blocked), `crib@existing`.
- **Helpers.** `--with deskSimple@10,13,2,…` (for every object) and a line's `with …` (added to them):
  bought before the object, where given or wherever the game finds room, kept while its uses run, sold
  after it (a helper that can't be bought fails the object, `helper not bought`). On the stage the
  object stands at `(10 − ⌊w/2⌋, 12 − ⌊d/2⌋, 0)` facing +z, so a table or desk in front of a chair goes
  at z = 13, facing it (rot 2) for a desk. Records name them (`with`). The stage's own kitchen (a
  fridge, a coffee machine, a counter) needs no helpers for cooking or coffee.
- **The state's own.** `--use-existing crib,garden.tomatoes` (or `def@existing` in the file): the first
  object of that kind the state has (the stage's crib with the baby in it, `review-garden`'s ripe
  crops), run before the `--objects`, never sold (records: `existing: true`).
- **Skills.** `--skills cooking=5,creativity=3` sets the resident's skills each time the state opens
  (`Command::SetSkill`, debug only, as `addFunds`; an id the content doesn't have stops the run).
  Records carry them (`skills`).
- **Waiting.** For every loading screen ("Moving in", "Getting ready", "Drawing portraits", "Building the
  world", "Warming up") before each buy and each picture; `goto` times out at 90 s. A use's cap (`--wait`,
  60 s) counts from when it is sent; a use nobody takes on (nothing in the resident's queue for 4 s: a
  baby's use, one needing a skill or something near) fails at once. The game's toasts meanwhile (why
  a buy or a use was refused: "needs a table", "would hang over the edge") go into the record
  (`toasts`) and a failed buy's error.
- **Speed.** `--walk` (3 = 10×, never more) on the way, `--speed` (1) from 3 tiles off the object once
  the resident is going to it, as short uses are over between frames at 30× (a cook's steps at the
  fridge and the counter go fast too).
- **Logging.** In the page, every animation frame: each clip on the object (`__renderer.motion`: its kind,
  length, furthest time, and when it moved, was open and back at rest) and each change in the
  resident's pose, `using`, animation and use. A record holds the object, the use (and its pose and anim),
  where it stands, `outcome` (`using`, `not taken`, `timeout`, `skipped`, `not bought`, `no uses`), when
  they were using it, `played` (clips that moved, not the ambient, growth, night or clock ones), each
  picture's file with the pose, clips and the object's place then, the needs before and after, timings,
  fps and errors.
- **Open things.** A use that needs the object open or closed (`when`) is set up with the use that
  leaves it so (fold the wall bed down first), and anything left open is shut before it is sold. The
  helper uses are records too (`helper: open|close`).
- **Failures.** Each object gets one retry of what failed (afresh from the state unless all were uses not
  taken); then the failure is recorded and the run goes on. A crop's "when ripe" uses are skipped unless
  it is ripe (bought new, it is just planted: `--use-existing` it on `review-garden`). The state is
  opened again when the resident's lowest need is below 0.25.
- **Camera.** `__controls.follow(null)`, aimed at the footprint's middle from in front and 0.7 rad to the
  side, set again before each picture; `--alpha --beta --radius --y` override. The radius is the
  larger of what the footprint needs and what the model as drawn needs to be whole in the picture
  inside the HUD (`__check.fit` over its vertices: a tree's crown, a wardrobe, a shower); a model over
  1.6 m is looked at halfway up. Not drawn yet: its height from `__renderer.objectBox`, else a guess.
  The game camera comes no nearer than 6. Records give the view (`view`: the pose, the model's height
  and bounds, and where they came from). The old single object (`--object deskSimple --at 23,14,0
  --use sit`, 6 shots 500 ms apart) still follows the resident, as does `--follow`.
- **Light.** `--hour 12` fixes the renderer's light (the state sets the clock); on a `*-night` state
  there's no default, so the light is the sim's clock's; `--hour state` for that anywhere.
- **Names.** Pictures are `<def>-<use>-<n>.png` (a kind used twice in a run: with its place); with
  `--name <run>` they start with it, so runs (an adult's, then a child's) into one `--out` keep theirs.
  A picture written over from an earlier run is said once.

Common options: `--quality low|medium|high` (`?quality=`), `--renderer webgl`, `--size 1280x800`,
`--extra 'k=v&k=v'`, `--headed` (a window, only to measure timing).

```sh
node tools/bench/check.mjs gallery --port 5227 --keys model.dartboard,model.tableClock
node tools/bench/check.mjs gallery --port 5227 --keys model.fridge,model.diningChair --scale all --camera game
node tools/bench/check.mjs gallery --port 5227 --each --keys model.fridge,model.toilet,model.bed
node tools/bench/check.mjs clips --port 5227 --keys model.fridge --clip door_open --times 0,0.2,0.4,99
node tools/bench/check.mjs clips --port 5227 --each --keys model.fridge,model.toilet,model.bed --times 0,0.5,99
node tools/bench/check.mjs play --port 5227 --state review-stage --objects toilet,fridge,bed,dartboard@15,12,3 --use all
node tools/bench/check.mjs play --port 5227 --state review-stage --objects-file /path/to/kitchen.txt
node tools/bench/check.mjs play --port 5227 --state review-stage --objects officeChair --with deskSimple@10,13,2 --name chair
node tools/bench/check.mjs play --port 5227 --state review-stage --objects cafeChair --use coffee --with cafeTable@10,13,0
node tools/bench/check.mjs play --port 5227 --state review-stage --use-existing crib --sim 4 --name crib-elder
node tools/bench/check.mjs play --port 5227 --state review-stage --objects chefStove --skills cooking=5
node tools/bench/check.mjs play --port 5227 --state review-garden --use-existing garden.tomatoes,raisedBed --use harvest
node tools/bench/check.mjs play --port 5227 --state review-stage-night --objects telescope@10,22,0,lamp
node tools/bench/check.mjs play --port 5227 --object toiletClassic --at 23,14,0 --use use --shots 6 --every 500
node tools/bench/check.mjs play --port 5227 --object toiletClassic --at 24,17,1 --state office-night --renderer webgl --quality low
```

On the test run of 2026-10-21 (headless, Medium, one core) an object took 9–26 s with all its uses (a
use 3–9 s: walking there 3–5 s, then 3 pictures), the state 8 s to open; `gallery --each` and `clips
--each` about 2 s a key.

## Debug pages

Dev-only pages on the dev server (not in the build); each sets `window.__ready` once it's drawn, and the
flags are at the top of its file in `web/src/debug/`. The ones the review uses:

| Page | Flags |
|---|---|
| `/gallery.html` | `keys=model.a,model.b` (or `filter=`), `cols`, `gap`, `labels=0`, `alpha`, `beta`, `radius`, `target`. `scale=adult` stands a man and a woman on each model's own left (the right of the picture, away from the camera), facing the camera (the game's `Characters`, at the idle), `scale=child` a child, `scale=all` a woman, a teen, a child and an elder (at most 64 residents: the first models get them). `camera=game` is the game's Live camera as a game starts (radius 24, beta 0.86, its 0.7 lens) aimed at the floor under the first model, or under `at=<key or index>`; `__gallery.aim(n)` aims it at model n. |
| `/poses.html` | `only=`, `seats=`, `focus=`, `clip=`, `strip=N` (the clip held at N times on a man, a woman, a child, a teen and an elder), `bottom=`, `manual`. `stage=baby\|child\|teen\|youngAdult\|adult\|elder` puts everyone placed at that stage where sim-core lets the stage take the place (a baby only lies in a crib; children and teens don't care for a baby, flirt or kiss), with the game's look for it (an elder's stoop and grey hair); with `strip`, the rows are a man and a woman at that stage. |
| `/characters.html` | `stage=child` or `stage=all`, `hair=all`, `tops=all`, `face=moods`, `clip=walk&t=0.4`, `view=side` (more in `characters.ts`). |
| `/clips.html` | `keys=`, `clip=`, `t=` (99: open), `colors=`, `target=` (see `check.mjs clips`). |

```
/gallery.html?keys=model.fridge,model.diningChair,model.bed&scale=all
/gallery.html?keys=model.fridge,model.diningChair,model.bed&scale=adult&camera=game&at=model.bed
/poses.html?stage=child
/poses.html?strip=8&clip=sitDown
/poses.html?seats=sofa,diningChair&stage=elder
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
| `review-stage` | For looking at one thing after another (`play --objects`): an empty 32 × 32 home lot, the street at z = 0, with one walled room from `(4, 6)` to `(16, 18)` (12 × 12 tiles; the door at `x = 5` in the front wall, windows at `z = 14, 15` in the left wall; a crib at `(13, 6)`; along the right wall, facing −x, a counter at `(15, 7)` for things on a top, a fridge at `(15, 8)` and a coffee machine at `(15, 9)`, so cooking at a stove bought and coffee at a table need nothing more) and a garden behind it. One resident per life stage, ids 0–5: adult (40), young adult (22), teen (15), child (8), elder (70), baby (0.5, in the crib); all but the baby in the front yard from `(20.5, 3.5)`. Saturday 10:00, free will off, every need at 0.5 (a use that fills one lasts), $1,000,000. Free spots for a thing of footprint `w × d` (as `play` places them by default, on every `review-*` state; all checked with the kitchen in, 2026-10-21): **floor** in the middle `(10 − ⌊w/2⌋, 12 − ⌊d/2⌋, 0)` (also rugs and ceiling things); **back wall** inside `(10 − ⌊w/2⌋, 18 − d, 2)` (wall things, and floor things that fill a wall: fridges, wall beds, wardrobes); **windows** `(4, 14, 1)` (curtains); **left wall** `(4, 8–12, 1)`; **right wall** `(16 − d, 10–16, 3)`; **front wall** `(7–12, 6, 0)`; **back wall outside** `(4–15, 18, 0)`; **garden** `(10 − ⌊w/2⌋, 22, 0)` (outdoor things, up to 6 × 6; free from z = 19 to 31, and x = 18 to 31 beside the room). The kitchen at `(15, 7–9, 3)` and the crib are outside them all. |
| `review-stage-night` | `review-stage` at 22:00 on the same Saturday (the clock set, not run, so nobody ages): night lighting by the sim's own clock (`play` sets no `--hour` on a `*-night` state). The lamps as the game leaves them: with free will off nobody switches one, and a lamp bought comes on. For stargazing, lamps, night clips. |
| `review-garden` | `review-stage` with every crop kind (content `grows`: `garden.vegPatch`, `garden.tomatoes`, `garden.strawberries`, `garden.pumpkins`, `raisedBed`, `natureLover.veggieBed`, in content order) in a row at the back of the garden, `(2, 29)`, `(5, 29)`, … every 3 tiles, facing the house, all ripe (a crop comes ripe with a lot; one bought is just planted). For their "when ripe" uses: `--use-existing <def>`. A harvested crop lies fallow until the state is opened again. The garden's free spot holds up to 6 × 6 (z 22–27). |

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
- **Worktrees.** Copy (don't link) the git-ignored wasm packages into a worktree
  (`web/src/core/wasm-pkg`, `web/src/voice/wasm-pkg`; a link is refused with 403), and `debug-states/`
  too (a run can change a shared state). `web/node_modules` may be a link to the main checkout's:
  `vite.config.ts` then lets the server read the linked folder and keeps its pre-bundled packages in
  `web/.vite-cache` (git-ignored), not in the shared `node_modules/.vite`, which every other worktree's
  server would re-optimise under it. A checkout with its own `node_modules` is unchanged.
- **Reloads.** Other sessions' edits in a shared checkout reload the dev server's page mid-run, and a
  content file being written can be invalid for a moment ("the game data failed to load"). Work in a
  worktree; if a run fails that way, run it again.
- **Scratch.** Give every session or agent its own folder in the scratchpad (as with ports): parallel
  agents writing helper scripts into one folder overwrote each other's. Keep screenshots, copies of
  content and probes there, never in the repo.
- **Shared files.** Write a shared file only after its new text is built and parsed (`t = fix(s);
  json.loads(t); open(p, 'w').write(t)`: opening it for writing first empties it, and once emptied
  `furniture.json`). Moving items into Validate and back: `cargo content`, which does that.
