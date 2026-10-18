# Idyll Lives

A home-building game with a living simulation that runs in the browser: you build and design your home, and its residents live in it on their own while you watch (see "Game direction" in [PLAN.md](PLAN.md)). The game logic is written in Rust and compiled to WebAssembly; rendering runs on the GPU through WebGPU, with a WebGL2 fallback. The interface is Svelte 5 with plain CSS. Desktop only.

**▶ Play it in your browser: https://krecks.github.io/idyll-lives/** (desktop Chrome, Edge, Brave or Safari; other browsers use the WebGL2 fallback). Resident voices and the language model for their lines need WebGPU.

See [PLAN.md](PLAN.md) for the research, architecture, milestones and known issues, and [CHANGELOG.md](CHANGELOG.md) for what each release brought.

## Run it

Requirements: Rust through rustup (the version is pinned in `rust-toolchain.toml`: 1.97.0 with clippy, rustfmt and the `wasm32-unknown-unknown` target), `wasm-bindgen-cli` 0.2.129 (it must match the `wasm-bindgen` crate in `Cargo.lock`), Node 22 and pnpm.

```sh
rustup toolchain install       # in the repo: the pinned toolchain, as CI does
cargo install wasm-bindgen-cli --version 0.2.129
cd web
pnpm install
pnpm dev          # builds both WASM modules, fetches the voice and language models, then starts Vite on http://localhost:5173
```

The first `pnpm dev` or `pnpm build` downloads the voice model (398 MB as published, stored as 185 MB) and the language model (294 MB of weights, stored gzipped as 255 MB) into `web/public/voice/` and `web/public/llm/` (not in git). Every file is pinned and checked by SHA-256, and files already there are kept.

| Command (in `web/`) | What it does |
|---|---|
| `pnpm dev` | `wasm`, `voice` and `llm`, then the dev server |
| `pnpm build` / `pnpm preview` | Production build / serve it |
| `pnpm wasm` | Build the simulation's WASM (release, `--locked`) |
| `pnpm voice` | Build the voice WASM and fetch the voice model and dictionary (`tools/voice/fetch.mjs`) |
| `pnpm llm` | Fetch the language model (`tools/llm/fetch.mjs`) |
| `pnpm check` | Type-check TypeScript and Svelte |
| `pnpm lint` | Lint the web code |
| `pnpm test:sim` | Run the Rust simulation tests |
| `pnpm test:voice` | Run the voice crate's tests (needs `pnpm voice` for the dictionary) |
| `pnpm check:voice` | Check the voices in Node against golden hashes (`tools/voice/check.mjs`) |
| `pnpm check:llm` | Check the language model's prompts, cleaning and golden lines in Node (`tools/llm/check.mjs`; several minutes) |
| `pnpm devices:llm` | Run the voices and language model's device test in Chromium (`tools/llm/devices.mjs`) |
| `node ../tools/llm/bench.mjs` | Benchmark the language model on its own: the game's worker, model and prompts with no game around it (load, prompt, tokens a second, the game's five lines), cold and with the page reloaded; writes `bench-results/llm-*/` |
| `pnpm assets` | Build or update the compressed asset copies (the build does this too) |
| `pnpm decoders` | Refresh `public/decoders` after a Babylon upgrade |

From the repo root: `cargo clippy --workspace --all-targets -- -D warnings` (CI gates on it), `node tools/bench/render.mjs` (the rendering benchmark, see [docs/design/graphics.md](docs/design/graphics.md)) and `node tools/release/version.mjs` (the version, see [Versions and releasing](#versions-and-releasing)).

**The production build** also:
- runs binaryen's `wasm-opt -O3` on both WASM files and strips their names (`tools/release/wasm-opt.mjs`; about a minute the first time, then cached in `web/.assets/wasm-opt`). The dev server keeps the WASM as built, with its names;
- writes `bundle.json`: the content, house templates and asset manifests every start needs, in one file. The files still ship one by one beside it;
- writes `THIRD-PARTY-LICENSES.txt` with the licences of the code it bundles, and fails if one is missing;
- gives every runtime fetch from `public/` the build's id (`?v=<id>`), so a new release's data doesn't mix with files cached from the last one. `voice/` and `llm/` are pinned by SHA-256 and left out, and `index.html` itself isn't versioned (Pages lets the browser keep it for 10 minutes).

**Hosting:** the game needs cross-origin isolation for its fastest path (the simulation's snapshots in shared memory). The dev and preview servers send `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`. GitHub Pages can't, so `index.html` registers a service worker (`public/isolation-sw.js`) that adds the headers, and the page loads once more under it. The worker also keeps the build's hashed files (`assets/<name>-<hash>.js`, `.css`, `.wasm`, `.woff2`) in a cache of the build's own, so returning players load them without asking the server again; a new build's worker deletes the old cache. Without isolation the game still runs, but snapshots are copied instead of shared.

## Playing

**New game:** create a world (a small, medium or dense town, a farm, a seaside village, a mountain hamlet, a lakeside town or a forest clearing, each generated with its own name, streets and landscape) → choose the neighbour households → either play one of the households who already live there (**Play** on its card or its lot; they keep their home, jobs and friendships) or make your household in **Creation** (drag to turn a resident, scroll to zoom to the face; gender, who they're attracted to, skin, hair, clothes, traits, perks, starting bonds) → choose a vacant house → move in. Everyone in town lives, works, socialises and falls in and out of love on their own, your household included; you build their home and watch. Only your lot is drawn (or the lot of a resident you follow); the rest of the town keeps simulating in the background.

- **Two modes:** **Live** (L) is the household and the world in motion; **Build** (B) is the home: furnish it, build it and see how its rooms are doing (time stands still). **Creation** belongs to the new game; a home built first gets its family later from the empty card in the household strip.
- **Live:** leave the game alone and the camera follows what happens at home (conversations, fights, first kisses, someone using what you just built, guests); any input takes it back. The household strip at the bottom shows everyone at a glance; click a resident to look closer (and step in from their panel), **F** follows them, **H** shows the whole house, **J** opens the journal. A resident's panel shows their needs, what they're doing and their plans. Speeds 4 and 5 are time-lapse; quiet hours (everyone asleep or at work) skip ahead.
- **Planner (P):** each resident's week: drop activities on a calendar (train a skill at 18:00 Mon/Wed/Fri, sleep 23–7, cook dinner) and set life goals (get a job, get promoted, find love, save money). Plans are a strong nudge, not orders: urgent needs and personality can win, and each block shows whether it was kept and why not. Household blocks apply to everyone. Activities with nowhere to do them become wishes that open the catalog.

- **Jobs (1000 of them):** residents look for work on their own (and may quit or be let go); you can also pick a job yourself. 25 categories (journalism, detective work, fire & rescue, social media, IT, TV, mechanics, finance, …) × 4 tracks × 10 grades **A–J**. The grade sets pay per hour ($12/h at A up to $52/h at J) and the skill level the job expects. Open the job board from the *Career* tab, filter by category, grade or what your resident can take, and join any position you qualify for — up to two levels short is allowed *on probation*. Promotions need a full performance bar **and** the next grade's skills; a good worker climbs a grade every few weeks.
- **Skills:** Intelligence, Strength, Endurance, Charisma, Creativity, Technology, Handiness, Cooking, Writing, Perception, Dexterity, Business and Gardening (0–10). Work trains the job's skills; at home, residents practise with things you buy (treadmill, weight bench, chess table, computer desk, easel, mirror, workbench, telescope, piano, gourmet cooking, studying, tending the garden, pruning a bonsai). Traits give starting skills and faster learning in some.
- **Workweek:** the standard week is set by the job (mostly Mon–Fri). Skills change it: on probation residents work an extra day, two levels above the requirements a day less (*Flexible*), four levels above two days less (*Expert*). The weekly salary stays the same, so a shorter week pays more per shift.
- **Money:** household funds are in dollars. Rent and bills are charged every Sunday at noon: rent depends on the lot size, and bills grow with the value of everything the household owns and with how many live there (food and household goods, $150 a person). A working household saves a few hundred to a couple of thousand dollars a week: a new room every week or two. Items cost nothing per use; only takeout, deliveries and similar purchases do. A household that can't pay goes into debt and its residents worry about money.
- **Day and night:** Residents are active by day and sleep through the night (couples share a double bed), get up early for an early shift, eat and wash in the morning, socialise in the evening and use the bathroom before bed. Tiredness builds up faster when they stay up late.
- **Build › Furnish (V):** time stands still while you shop. The catalog shows every item as a 3D picture; point at one and move across its preview to turn it around. Furniture and training equipment cost money, and what you buy decides what your residents can do: point at an item to see the needs it fills (and how fast), the skills it trains, the feelings it can give, whether a skill makes it better (a skilled cook gets more out of the stove), any cost per use and how many residents fit. Filter by category (Kitchen, Bathroom, Bedroom, Kids, Living, Office, Fitness, Hobbies, Decor, Lighting, Outdoor, Garden) and its groups (Kitchen: Appliances · Counters & storage · Dining, …), search, sort by price or show only what you can afford. Pictures, curtains, shelves and wall lamps hang on walls (they turn to face away from the wall you point at), ceiling lamps and fans hang from a room's ceiling and rugs lie on the floor: they take no floor space, so a picture can hang over the sofa and a table can stand on a rug. Things on a wall go with their wall when walls are cut away, and a wall something hangs on can't be torn down until it's moved. Lamps light their room at night. Some furniture takes a **colour** of your choice (marked "Any colour" on its card): pick one of the swatches or a colour of your own while it's in hand, or recolour what you own for free; colours never change price or quality. Click an owned object to upgrade, move, rotate, recolour or sell it (60% back); drag it to move it anywhere on its tiles. Furniture can't be resized for now (moving parts such as a fridge's doors don't follow a resize yet); something resized in an older save keeps its size, and "Reset size" takes it back. With the Door or Window tool, click a door or window to resize it with its blue arrows (Shift: its sill and top to the centimetre): side by side, windows make one wide window and doors a double door.
- **Garden:** the *Garden* category (Trees · Shrubs & hedges · Flowers · Fruit & veg · Houseplants) has over 50 plants: oaks, maples, birches, cherry blossom, apple, magnolia, Japanese maple, weeping willow, spruce and cypress; roses, hydrangeas, lavender, hedges and topiary; beds of tulips, daffodils, daisies, sunflowers, poppies, lupins and more; vegetable patches, tomatoes, strawberries, herbs and pumpkins; and house plants from monstera to bonsai. Garden decor (bird bath, fountain, pond, gnome, lantern) is under *Outdoor*. Trees, beds and other garden things go outdoors only. Residents water, tend and trim them, smell the roses, relax in the shade and harvest a free meal; all of it trains **Gardening**, and good gardeners harvest more, feel proud of their beds and get Fresh herbs (better cooking for a while) from the herb planter. Gardening is the main skill of the Gardening & Horticulture career and helps farmers; Nature Lovers start with some and learn it faster.
- **Upgrades:** instead of pricier versions, objects get better by upgrading. Upgrades are bought in Build › Furnish (click the object → *Upgrade to ★n*) and happen at once; each quality star makes everything the object gives 25% better (a better shower cleans faster, a better treadmill trains faster).
- **Build › Construct (C; Wall · Room · Paint · Door · Window · Stairs · Remove):** Press on a corner and drag to draw a wall, with a live preview and its cost; release to build. Walls can be full or half height and come with a covering (paint colours, wallpaper, siding, brick, wood panelling, stone, tiles); the Paint tool re-covers the side of a wall you point at (drag along walls, or Shift-click for a whole room). Doors and windows come in styles (painted, oak, half-glazed and French doors; classic, cottage, picture, floor-length, high-light and transom windows); pick another style and click to replace one. Tearing down is free. Drag at about 45° and the wall runs diagonally, corner to corner across the tiles. Clicking a corner and then another also works. The Room tool draws all four walls of the rectangle you drag out. Put doors and windows into walls (straight or diagonal) with a click, and drag along walls with Remove to tear them down on your own lot. Esc or a right-click while drawing cancels. Windows let light in but not residents. The Stairs tool puts a staircase (1 × 4 m) in hand: it leads up to the storey above (build that first) and is climbed from the front; R turns it, click your stairs to move them and Delete sells them. A diagonal wall costs 1.414 × a straight one per tile (it is √2 m long) and splits its tile into two halves that can belong to different rooms; nobody can stand on that tile and furniture can't go there (a diagonal door tile can be walked through). Draw a room's walls first, then add its door; only a resident or furniture can't be shut in.
- **Visits:** neighbours visit friends on their own. Open the town map (**M**) to send your resident to visit anyone; the view follows them.

| Input | Action |
|---|---|
| Left-click a resident / their card, Tab | Look at them (their panel) |
| Left-click another resident while looking at one of yours | Social menu (chat, flirt, argue, …) with success chances |
| Left-click an object | Interaction menu |
| Left-click the floor | Walk there (while looking at one of yours); otherwise close the panel |
| F · H · J · P | Follow the resident · whole house · journal · planner |
| Left-drag / right-drag / wheel | Rotate / pan / zoom |
| Left-drag on the lot (Walls, Remove) | Draw or tear down walls along the drag (at 45°: diagonal); right-click or Esc cancels. The camera then turns with middle-drag or by dragging off the lot |
| Space, 0–5 | Pause, game speed (4, 5: time-lapse) |
| W | Walls up → cutaway → down |
| M | Town map (who's home, visiting) |
| L / B | Live / Build (B again goes back to Live) |
| V / C / O | Build › Furnish / Build › Construct / Build › Home (rooms tinted by how they're doing; repairs, adopting) |
| R, Delete | Rotate / sell the object in hand or selected (Build › Furnish) |
| Esc | Steps back: put down what's in hand, leave Build, close the journal, stop following, close a resident's panel — then the pause menu (save, load, settings, quit) |
| F3 | Performance overlay (with *Save debug report*) |
| F8 | Save a debug report: screenshot, game state, settings, the build's version and commit, and recent errors. With `pnpm dev` it goes to `debug-reports/<time>/` in the project; open `?debugReport=<time>` to load it with the same camera. Other builds download it as a file, which any build can play: paste it into Settings › Interface › Diagnostics › Game state. |

**Graphics:** Settings › Performance has three presets. Medium is the default and the game's intended look; Low is for older laptops (FXAA, 1024 shadows, the four lamps nearest the biggest rooms, half the lawn's tufts and flowers, no shadows from the woods round the town); High adds 4096 shadows and two live mirrors. Resolution is Auto (the screen's own pixels up to about 1920×1080, scaled up beyond) or Custom (50–100 %), and the frame rate cap (60 by default) is paced in whole display refreshes. Every knob, its cost, and the benchmark: [docs/design/graphics.md](docs/design/graphics.md).

URL options:

| Option | What it does |
|---|---|
| `?quality=low\|medium\|high` | Overrides the graphics preset for this page |
| `?renderer=webgl` | Draws with WebGL2 even where WebGPU works |
| `?bench=village` | The rendering benchmark (also `forest`), with `&quality=`, `&hour=`, `&cam=orbit\|wide\|build\|dusk`, `&res=`, `&cap=`, `&diag=` and more ([docs/design/graphics.md](docs/design/graphics.md)) |
| `?hour=13` | Holds the lighting at that hour |
| `?diag=noshadows,nonature,…` | Leaves features out of the picture to measure what they cost; `freeze` stops time for screenshots |
| `?debug` | Console helpers: `__controls`, `__world()`, `__sims()` (residents live: where, pose, what they use, needs), `__send(command)`, `__addFunds(n)`, `__camera(pose)`, `__showPlot(i)`, `__characters`, and `__crash('panic'\|'throw'\|'hang'\|'render')` to see how the game copes with a failure |
| `?seed=42` | The random seed of a new game |
| `?pack=packs/my-art/manifest.json` | Loads an extra asset pack (can be repeated) |
| `?snapshot=0` | Turns off WebGPU snapshot rendering (the menus' town) |
| `?lights=clustered\|forward` | Forces how lamps are lit |
| `?state=<name>`, `?debugReport=<name>` | Dev server only: starts in a game state written by the simulation's tests (`DUMP_STATES=1 cargo test`), or in a saved debug report |

## Architecture

```
Main thread                                   Sim worker
┌───────────────────────────────┐            ┌──────────────────────────┐
│ ui/      Svelte HUD (~10 Hz)  │◄─ ui ──────│ sim-wasm → sim-core      │
│ game/    composition + input  │── cmds ───►│ fixed 20 Hz tick         │
│ render/  Renderer (GPU)       │◄─ shared ──│ snapshot (seqlock, SAB)  │
│ assets/  manifest registry    │   memory   │ world geometry on change │
└───────────────────────────────┘            └──────────────────────────┘
```

**Module boundaries.** Each folder can be changed or replaced without touching the others.

| Module | Owns | Must not |
|---|---|---|
| `crates/sim-core` | All game rules: needs, AI, pathfinding, town, social life, saves, geometry generation | Know about browsers, rendering or asset files |
| `crates/sim-wasm` | JS bindings | Contain game logic |
| `web/src/core` | Worker, bridge, protocol, snapshot channel | Render or touch the DOM |
| `web/src/render` | Everything drawn; `types.ts` is the contract | Contain game rules; allocate per frame |
| `web/src/assets` | Key → file/placeholder resolution | Hard-code art anywhere else |
| `web/src/ui` | Svelte components, design tokens (`theme.css`) | Talk to the worker directly (use `services.controls`) |
| `web/src/game` | Wiring and input (the only place that knows every module) | Grow logic that belongs in a module |

**Performance rules**
- The CPU runs the game; the GPU draws it. The main thread reads one snapshot per tick and draws the world live; the menus' town replays recorded GPU commands (WebGPU snapshot rendering).
- Every object type is one draw call (thin instances); residents are thin instances whose matrix buffer is updated once per frame.
- No allocations in the frame loop or the sim tick. The snapshot layout is defined once, in Rust.
- The UI gets changes only, at about 10 Hz, never the whole view and never per frame.
- Import Babylon through `web/src/render/babylon/core.ts`, never from the `@babylonjs/core` root (that pulls in the whole engine).

## Replacing art

All art is referenced by **key**, never by path:

- `web/public/content/base.json`: gameplay data. Objects name their model key (`"model": "model.fridge"`); needs name their icon key.
- `web/public/assets/manifest.json`: maps keys to files or placeholders.

To use a real model, add a `url` to the entry (the placeholder stays as the fallback):

```json
"model.fridge": { "type": "model", "url": "models/fridge.glb", "scale": 1, "rotationY": 0, "placeholder": [ … ] }
```

To ship a whole art set without editing the base manifest, create a pack and list it under `"packs"` (or load it with `?pack=`):

```json
// web/public/assets/packs/hd/manifest.json — keys here override the base manifest
{ "version": 1, "entries": { "icon.need.hunger": { "type": "icon", "url": "icons/hunger.png", "mode": "image" } } }
```

Model conventions: 1 unit = 1 metre, origin at the footprint centre on the floor, front facing +Z. How to build a new object in Blender (scale, budgets, finishes, export, checks): [docs/design/objects.md](docs/design/objects.md). Monochrome SVG icons are recoloured by CSS (`mode: "mask"`); use `mode: "image"` for full-colour art.

## Adding gameplay content

Everything gameplay-related is JSON in `web/public/content/`:

- `base.json`: needs, genders, traits, perks, objects, **social interactions** (requirements, success chances, outcomes, animation), emotions, feelings, bond presets, social rules, **careers** (levels, pay, hours, workdays, work effects), **daily schedule** windows, visit rules, starting funds, story-feed texts.
- `houses.json`: house layouts and the park, stamped onto town plots.

Add an object to `content/base.json` (footprint, interactions, need effects, model key) and its model key to the manifest; homes are furnished from `houses.json` and `housegen.json`, or bought in the game. (`content/lots/starter.json` is only for the tests and the pack checker.) Content is checked when it loads: a value that would break the game is refused with the file and id in the message. `cargo test` validates the shipped content (`crates/sim-core/tests/shipped_content.rs`).

A production build serves the content from `bundle.json`, which wins over the loose files: to edit content inside a built site, rebuild it or delete `dist/bundle.json`.

Pack authors: see [docs/content-packs.md](docs/content-packs.md). The look of the world is described in [docs/design/look.md](docs/design/look.md), the graphics settings in [docs/design/graphics.md](docs/design/graphics.md).

## Versions and releasing

The current version is **0.2.0**. One version number covers the whole game: `web/package.json` (shown in the main menu and Credits, with the commit it was built from), the Rust workspace in `Cargo.toml` and the workspace crates' entries in `Cargo.lock`. `node tools/release/version.mjs` prints it and fails if they differ (CI checks this); `node tools/release/version.mjs 0.3.0` sets all three. Each release is a git tag `vX.Y.Z` with notes in [CHANGELOG.md](CHANGELOG.md).

Saves have their own format version, `SAVE_VERSION` in `crates/sim-core/src/save.rs`: 23 since 0.2.0 (0.1.0 wrote 17). Older saves are migrated when they load, and `crates/sim-core/tests/old_saves.rs` loads real saves from 0.1.0 and the build after it. A save from a newer version is kept and skipped, and a damaged one is removed with a notice.

**Releasing** (`.github/workflows/pages.yml`):
1. `node tools/release/version.mjs <x.y.z>` (it updates `Cargo.lock` too, so no `cargo build` is needed), and a `## <x.y.z> — <date>` section in CHANGELOG.md, with the date set by hand. Commit both on `main`, and `git push origin main`: the dry run builds what GitHub has.
2. A dry run on `main`: `gh workflow run pages.yml --ref main -f deploy=false`. `gh run watch` needs the run's id: `gh run list --workflow pages.yml -L1 --json databaseId -q '.[0].databaseId'`, then `gh run watch <id> --exit-status`. It runs everything a release runs (clippy with `-D warnings`, the simulation tests, tickbench, the voice tests, `pnpm check:voice`, `check` and `lint`, the build, a size check under 900 MB) without publishing. It works on any branch, and `-f deploy=true` publishes. Run it on `main` just before tagging: the caches it saves there (voice and language models, assets) can be read by the tag's run.
3. Tag and push: `git tag -a v<x.y.z> -m "..."` and `git push origin v<x.y.z>`. The tag's run checks that the tag matches the version and that CHANGELOG.md has its heading, builds, and publishes to GitHub Pages. Pushes to `main` alone don't publish. Watch it as in step 2 (`gh run list --workflow pages.yml --event push -L1 …`), and don't start other runs while it waits: GitHub keeps only one waiting run per concurrency group and cancels the older one.
4. Check the live site:
   - The menu shows the new version, and Credits "Version <x.y.z> (build <the tagged commit>)". (To find the version in the bundle, search `index-*.js` for it without double quotes: it sits in backticks.)
   - F3 shows Shared memory: on, and a game runs.
   - The large files download in full, which only the live site can show: `llm/lfm2.5-350m/model_q4.onnx_dataz` returns 200 and is 255,530,880 bytes (gunzipped, its SHA-256 is the one in `tools/llm/fetch.mjs`), and `voice/supertonic-3/vector_estimator.onnxz` is 119,568,968 bytes.
5. Publish the release notes: `gh release create v<x.y.z> --verify-tag --title "Idyll Lives <x.y.z>" --notes-file <the version's CHANGELOG section>`.

**If a release goes wrong,** fix forward or redeploy the last good 0.2.x (`gh workflow run pages.yml --ref <tag> -f deploy=true`), not v0.1.0: 0.2.0 upgrades the browser's saves database to version 2, which v0.1.0 can't open, so it would list no saves and couldn't save. Nothing is lost; the saves come back with any 0.2.x.

Each CI run keeps a WASM with function names as an artifact for 90 days. It is the un-optimised file, so its function numbers don't match a crash stack from the site (whose WASM is optimised and stripped).

## Licence

Idyll Lives is free software, released under the [GNU General Public License v3.0 (or later)](LICENSE). You may use, study, share, and modify it; if you distribute Idyll Lives or a modified version, you must make the source available under the same licence. Third-party parts keep their own licences: the art listed in [`web/public/assets/CREDITS.md`](web/public/assets/CREDITS.md) is CC0, the voice model (Supertone's Supertonic 3) is under the BigScience OpenRAIL-M licence (with its use restrictions), the dictionary is Apache-2.0, and the optional language model for residents' lines (Liquid AI's LFM2.5 350M) is under the LFM Open License v1.0 (all listed there). Every production build also writes `THIRD-PARTY-LICENSES.txt` with the licence texts of the code it bundles (Babylon.js, Svelte, ONNX Runtime Web, the Rust crates in the WASM and others); the in-game Credits link both.
