# Objects

Every object in the game, 541 of them, has a model built in Blender (`tools/art/blender`), exported as a `.glb`,
with the moving parts it can show. Blender is the only source: the primitive parts the models replaced are gone, and
the keys never changed, so saves and content need no migration. All 541 were built and checked in play by 2026-10-17,
as were the 36 models that aren't buyable: the 20 things residents carry and cook with (`families/held.py`), the 9 street props
(`families/street.py`), and the lawn's grass tufts and wildflowers, the woods' rocks and the selection marker
(`families/ground.py`). Every model key in the manifests has a `.glb`.

How to build an object is in [docs/design/objects.md](../design/objects.md) (the standard: scale, budgets,
materials, moving parts, the gate) and [tools/art/blender/README.md](../../tools/art/blender/README.md) (the toolkit
and the agent loop). This folder is the reference for what was built: one page per buy category, with what each
object is, what it shows and what its check in play found, and [open.md](open.md) for the decisions to review and
what is still open.

## Categories

| Category | Page | Items | Groups |
|---|---|---|---|
| Kitchen | [kitchen.md](kitchen.md) | 72 | Kitchen sinks (2), K1 Fridges (7), K2 Cookers (6), K3 Small appliances and dishwashers (4), K4 Base counters (8), K5 Tall and wall storage (11), K6 Islands and bars (5), K7 Dining tables (13), K8 Chairs and stools (12), K9 Dining storage (4) |
| Bathroom | [bathroom.md](bathroom.md) | 26 | Sinks (5), B1 Toilets (5), B2 Showers (4), B3 Baths (3), B4 Laundry (6), B5 Bathroom storage and mirrors (3) |
| Bedroom | [bedroom.md](bedroom.md) | 27 | BR1 Beds (9), BR2 Nightstands and small storage (8), BR3 Wardrobes and drawers (6), BR4 Dressing and mirrors (4) |
| Kids | [kids.md](kids.md) | 20 | KD1 Baby and kids' beds (4), KD2 Toys (10), KD3 Kids' study and art (6) |
| Living room | [living.md](living.md) | 50 | L1 Sofas and sectionals (12), L2 Armchairs and small seats (11), L3 Coffee and side tables (9), L4 Media (8), L5 Shelves (9), L6 Stairs (1) |
| Office | [office.md](office.md) | 14 | O1 Desks (8), O2 Desk chairs (3), O3 Storage and boards (3) |
| Fitness | [fitness.md](fitness.md) | 5 | F1 Home gym (5) |
| Hobbies | [hobbies.md](hobbies.md) | 19 | H1 Music (6), H2 Games (6), H3 Crafts and science (7) |
| Decor | [decor.md](decor.md) | 22 | D1 Wall art and clocks (6), D2 Ornaments (10), D3 Textiles (6) |
| Tabletop | [tabletop.md](tabletop.md) | 39 | TT1 Kitchenware (11), TT2 Tableware (7), TT3 Books and small plants (7), TT4 Small ornaments (6), TT5 Bath, desk and toys (8) |
| Lighting | [lighting.md](lighting.md) | 21 | LT1 Ceiling lights (7), LT2 Wall lights (3), LT3 Floor lamps (5), LT4 Table lamps (6) |
| Outdoor | [outdoor.md](outdoor.md) | 43 | OU1 Patio seating and tables (11), OU2 Loungers, swings and the hot tub (5), OU3 Grills and fire (3), OU4 Play and yard (9), OU5 Pond, small garden things and lights (9), Garden decor (6) |
| Garden plants and trees | [garden.md](garden.md) | 53 | G1 Houseplants (10), G2 Flowers (15), G3 Shrubs (11), G4 Edibles (5), G5 Trees (12) |
| Community and farm | [community.md](community.md) | 36 | CM1 Playground (8), CM2 Street furniture (10), CM3 Food stalls and vending (5), CM4 Shops and café (5), CM5 Farm (8) |
| Personality packs | [packs.md](packs.md) | 94 | P1–P16: one group per pack (bookworm … slob, and the example pack) |

The sinks and the garden decor were built before the groups were coded. The group codes (K1, O2, CM5 …) are how family scripts and their notes (`tools/art/blender/families/*.md`) refer to
a set of objects.

Each group's table lists, per item:
- the id and name;
- its size and layer;
- its uses (interaction id and animation), which are what trigger the clips;
- **Home**: its buy category and group (`living / —` is the category with no group);
- the parts the player may colour (`paint`, labelled);
- the clip's kind, and what the clip shows.

Under each table, **In play** says what its check in play found, fixed and decided.

## Moving parts: the kinds

The manifest field and the gamekit helper for each are under [Engine features](#engine-features).

| Kind | Plays | Manifest (`motions`) | Example |
|---|---|---|---|
| **Use** | While a resident uses the object (by interaction id, else by animation tag). Played back at the end of the use, or after its `hold` (per clip: E5). `out` opens it again for a while at the end (the phone box lets the caller out). | `uses` | Fridge door, sink water, toilet lid |
| **Held open** | While the sim's `open` flag is set (an interaction's `when` / `leaves`) | `open` (+ content, C1) | Wall bed, futon, sofa bed |
| **Loop (E1)** | Repeats for as long as the use lasts | `loops` | Rocking chair, treadmill belt, drum spin |
| **Ambient (E2)** | Always, on its own, near the camera (not on Low) | `idle` | Ceiling fan, fountain water, the parasol |
| **Clock** | The game's time, set each game minute (each five on Low) within 30 m of the camera or the ambient range, on every preset | `clock` | Wall clock, grandfather clock, alarm clock |
| **Night (E3)** | Held while the lamps are lit | `night` | Curtains drawn; shades glow with a material's `glow` (E8) |
| **Growth (E4)** | Follows the crop's growth stage from the sim (content `grows`) | `growth` | Vegetables ripening |
| **Static** | No clip | — | Tables, rugs, ornaments |

Residents sitting or lying on a moving seat ride it with `ride` (a rocking chair, a swing). A use clip plays on top of
the growth pose (`partPose`).

- **No use yet:** "(+use)" in a table marks a part built separately for a clip that needs an interaction first, so the
  clip can be added without remodelling.
- **Rules every clip keeps (`check_glb.py`):**
  - node transforms only (no skins or morph targets), starting at rest;
  - about 1 s long (the gate warns of a use clip over 3 s);
  - 1–3 draw calls (4 at most);
  - water, flames and contents that appear are `G.grow`n from 0.001 (gloss, never emissive).

## Engine features

Before this work an object clip played only while a resident used the object, or while the sim held it `open`
(`web/src/render/babylon/objectMotion.ts`, `models.ts` `motionsOf` / `clipPose`): forward, held, then back, with one
`hold` per model. These features were added first (2026-10-14) so every kind of clip could be built. The family
notes and scripts refer to them by these names.

| Name | What it is | Use it as |
|---|---|---|
| T0 | Clips bench and gamekit helpers | `/clips.html?keys=model.<id>` (`&clip=`, `&t=`, `&renderer=webgl`): every clip of each model in a loop (rest, play, hold, back), a button per clip, `t` to hold a time for screenshots. `G.spin(obj, clip, turns, seconds, axis)` (whole turns ending at rest), `G.rock(…, degrees, …)` (out, back past rest, home), `G.squash(obj, clip, scale, seconds)` (to any size and back: cushions, a held weight hidden), `G.grow`; the boulder shape is `G.boulder` |
| E1 | Loops while in use | `motions.loops: {use: clip}`: repeats rest to rest for as long as the use lasts, finishes its round, never plays back. Runs on game time (stops when paused). `check_glb` fails a loop that doesn't end at rest |
| E2 | Ambient clips | `motions.idle: clip`, looping always; drawn only within `ambientRange` of the camera (Low 0, Medium 40 m, High 60 m), off on Low, shadows not redrawn for them. 94 ambient things cost 0.09 ms (clips) and 0.15–0.3 ms (frame) of CPU on an M3 Pro (`docs/design/graphics.md`) |
| E3 | Night state | `motions.night: clip`, held while the lamp level is up, dusk to dawn (`house.ts`, `BabylonRenderer.ts`); curtains stay drawn over sleepers |
| E4 | Growth state | content `grows: {days, fallow}`, harvest `"when": "ripe"`, `"leaves": "picked"`; `motions.growth: clip`, made with `G.stages` (ripe at rest, then picked, growing, planted). The sim advances the stage, the renderer holds the clip there |
| E5 | Per-use hold | `motions.uses: {use: [{clip, hold}, clip]}`: several clips per use, each with its own `hold` (a shower's door shuts after 1 s while the water runs) |
| E6 | Mirrors from files | a material's `"mirror": "front"` (gamekit: `glass=('#hex', 'glass', 'front')`) |
| E7 | Level of detail | an entry's `far` file, `<id>.far.glb`; `G.Model(far=0.25)` writes it (trees, hedges, street props, the farm) |
| E8 | Finishes by name, packs that extend, night glow | materials named `stone@matte`; a pack entry without `vary` keeps the one it overrides; a material's `"glow": "#hex"` (gamekit: `{'glow': '#hex'}`) follows the lamp level, never emissive at noon |
| C1 | Content: things that stay open | interactions with `when` / `leaves` (`docs/content-packs.md`): the futon, the sofa bed, the washing line and the hot tub stay open (`crates/sim-core/tests/open_objects.rs`) |

Added after, in the same spirit:
- **Riders:** `motions.ride: clip`: residents sitting or lying on the object move with that clip's part (a rocking
  chair's seat, a swing, a hammock, an office chair's swivel).
- **Mirrors and glow on moving parts:** a mirror cabinet's door reflects as it swings, a laptop's lid glows as it moves.
- **Tint slots by name:** `tunic#body` (with a finish: `tunic@fabric#body`), as a `parts` entry's `tint`.
- **Beds** (`characters.ts` `BEDS`): `fwd` (m) lies the sleeper forward (the sofa bed), `foot` ends the quilt (the car
  bed), `across` lies a napper along a two-seat seat; content `whole` gives a use every slot of its object (a nap on
  the window seat).
- **Held things:** content `held` draws the object's part in the resident's hands (the bench's bar, a dumbbell pair,
  the guitars) while the object hides its own (T0 `squash` to 0), in the object's colours.
- **Seats:** `seat.facing: "back"` (the sitter faces the object's back: the piano, the dressing table, the cinema
  wall), `seat.recline` (degrees); a chair facing a separate table is drawn pulled in to it while sat on
  (`tableTuck.ts`).
- **Lights:** content `light.at` [x, z] in the object's frame puts the light at the shade.

## Adding an object

Follow `tools/art/blender/README.md` ("Working as an agent: the loop") and `docs/design/objects.md`. In short:

1. **Read** the content entry and the notes of the family it belongs in. The size, the top height (`surface`), the
   seat height (`seat`), the mattress height (`BEDS`, `BLANKET_BEDS` in `web/src/render/babylon/characters.ts`) and
   the light position (`light`) are facts the model must keep; the family script asserts them.
2. **Build** in a family (copy `families/_template.py`, or add to an existing family):
   - one material per part, named for what it is;
   - `paint` with labelled parts, the main part first, e.g. `[('body', 'Cabinet'), ('stone', 'Worktop')]`;
   - the moving parts joined one per part (`G.join`), and the clips from `G.swing` / `G.slide` / `G.grow` (plus
     `spin`, `rock`, `squash`).
3. **Preview** (`build.py <family> --preview`) and look at every model at rest and with every clip.
4. **Export** (`python3 -B tools/art/blender/build.py <family> [--only <id>]`). `check_glb.py` must say `0 failed`;
   read every warning. The Khronos validator must report 0 errors and 0 warnings.
5. **Test.** `cargo test -p sim-core --test shipped_content`, and the whole `sim-core` suite if content changed.
6. **Check it in play** (below).
7. **Finish.** Add a line to `web/public/assets/CREDITS.md`. Commit only the `.glb` files you meant to change:
   re-exports aren't byte-for-byte the same, so `git checkout` the rest. Never commit `previews/`. Add the object's
   row to its category page here.

## Checking an object in play

An object is right when it has been checked in play and looks and moves right:
- buy it and place it in all four facings;
- colour every part in the colour studio (every slot a distinct colour, far from each part's default), "At rest" and
  "In use", turning the turntable;
- let a resident use it, so every clip plays and shuts when it should (sit and lie uses again ~4 s in);
- by day and at night, on WebGPU and `&renderer=webgl`, on the Medium and Low presets;
- check the neighbours that share code you touched (a pose, a `BEDS` entry, a family helper).

Tools:
- `/gallery.html?keys=…` (WebGPU and `&renderer=webgl`), and the clips bench
  (`/clips.html`, T0) at t=0 and t=99 first: it finds clips that show nothing faster than play does.
- `tools/bench/check.mjs gallery | clips | play` runs the checks headless at 1 % volume on its own port. In the page,
  `__sims()` says where residents are and what they use; `__addFunds`, `__camera`, `__controls.follow` with zoom;
  `__renderer.motion.states`.
- Debug states (`?state=<name>`): `DUMP_STATES=1 cargo test -p sim-core --test check_states` writes `bathroom-check`,
  `needy-check`, `office-night` and `town-things` (all 36 community and farm things); `--test play_checks` writes
  `family-check` (`generated-<i>` with `--ignored`, `HOUSES=…`, `PICK=…`). Scripts must send `setAutonomy` off.
- Residents' clips: `node --experimental-strip-types tools/characters/check_clips.ts` (`tools/characters/README.md`);
  the poses per action are the table in `web/src/render/babylon/characters.ts`.
- Several agents at once: one worktree each, its own port and scratch folder (`tools/bench/README.md`), one CPU core
  each. In a shared checkout, build a commit from `HEAD` plus your own hunks, and write a shared file only after the
  new text is built and parsed.

## Validate and `validate.py`

While the objects were built, each set went into a temporary **Validate** buy category (a group per set) to be found
and checked in one place, and back home once checked. The category and its icon (`icon.category.validate`) were
removed when the last set left. `tools/content/validate.py` moves items in and out by id, keeping the content files'
formatting:

```
python3 -B tools/content/validate.py list
python3 -B tools/content/validate.py in <id>... --group <group> [--label "Label"]
python3 -B tools/content/validate.py out --group <group> [--to <category>/<group>] [--dry-run]
```

Without `--to`, `out` sends each item to the **Home** column of the tables on these pages. To use it again, add the
`validate` category back to `web/public/content/base.json` with an empty `groups` (and its icon), and remove it when
the last group leaves. Items whose home category turns them freely (`objectRules.freeRotation`) carry
`"freeRotation": true` while in Validate. Run `cargo test -p sim-core --test shipped_content` after.

## Lessons for the tooling

The check of 2026-10-16/17 ran 14 agents at once, one worktree each. The same faults cost nearly every one of them
time; fix them before the next large check.

**Setup**
- Worktree vite: the git-ignored wasm packages must be *copied* into the worktree (a symlink is refused with 403 and
  the sim, font and environment light silently fail: "lights only" pictures). Vite needs a wrapper config that adds
  the main checkout's `node_modules` to `server.fs.allow` **and** its own `cacheDir`: the shared `node_modules/.vite`
  is re-optimised by every agent's server (ENOTEMPTY at start, reloads mid-run, lost environment maps). Better still,
  the repo's `vite.config.ts` could do both when `node_modules` is a symlink.
- Setup must fail loudly: fetch the wasm through the new server (expect 200) and load the check house once
  (`__sims()` answers, no "using lights only") before any run; the runner should record HTTP ≥ 400 and console
  errors per job.
- The agent sandbox refuses `bash script.sh`, `cd … &&` chains, `$VAR` in command or option position and loops
  calling bash: ship setup as plain commands, with absolute paths, and a node launcher that sets PORT/PLAYWRIGHT/CHROME
  itself.
- Debug states are shared through the link: give each worktree a copy, or a run can change the check house.
- One CPU core per agent (1 page, no browser and Blender at once), never more agents than cores.

**The runner**
- Reuse one page for many jobs (sell, buy the next facing): a fresh page cost 10–75 s under load; reuse made jobs
  5–15 s.
- Wait for every loading text ("Moving in", "Getting ready", "Drawing portraits", "Building the world", "Warming up")
  before buying and again before each shot; time out `goto` at 90 s; count the cap from page ready and from when the
  use is sent.
- Walk fast and slow down only within a few tiles of the object (never above 10×: 30× skips short uses).
- Sample clips in the page every animation frame, not from node every 100 ms (short clips were missed).
- Ignore always-running clips (`idle`, `growth`, `night`, clock hands) for at-it / picture / shut; know held-open
  things (`motions.open`) and close them with a `then=<use>`.
- Frame the object, not the resident: clear the follow (`__controls.follow(null)`), aim at the footprint's middle with
  a radius from its size, re-aim just before the shot (the intro tween and the game's auto-watch override it); a back
  view; walls kept up for wall things or an uncut wall chosen.
- Print the resident's needs (a use ends when its need is full: toilets, swings, sunbathing), with
  `drain=<need>:<level>` and `cancel=<s>`. Shots less than about 0.8 s apart freeze the game between them.
- Record what each shot shows (object found, its facing, colours set) and flag mismatches: a late buy once put the
  wrong facing's object in a plausible picture.
- Generate the jobs from content + manifest (main use = first with a clip, extra runs only for uses whose pose, anim
  or clip differ, outdoor things to the yard, radius from the footprint), and a report per group from the results.

**Colours and the studio**
- Pick test colours far from each part's default (read it from the GLB), or run a second rotated colour run: a part
  whose default is already the test colour proves nothing.
- One studio script that clicks a distinct swatch per part, shoots, turns the turntable and takes "In use". Wait for
  the stage to draw (poll the canvas) instead of a fixed delay; loop-only models have no "In use".
- Contact sheets from a node script: the play shots plus the studio in one row per item, with a label tile per shot.

**States, spots and gates**
- A spot table per check house: wall spots with and without windows, plain 2-wide walls, the yard.
- Debug states for what play can't stage quickly: a ripe crop or a `__debugSet` for growth and needs, a two-storey
  house with a basement (stairs), a two-resident house (double beds, bunks), `family-night` (children at night).
- `check_glb.py` could flag coplanar faces of different materials in one part (three z-fight fixes: duvet end, chest
  lid, bread bin) and parts scaled to a sliver at rest that are still visible.
- `gamekit` smooth-shades cylinders without a bevel, so flat caps can shade like domes (the bread bin).
