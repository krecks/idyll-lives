# Authenticity review: ledgers

One ledger per category for the review of models, animations and interactions planned in [plan/authenticity-review.md](../../plan/authenticity-review.md). Each ledger has a table per group with a row per item, and a Findings section at the end. Known issues from earlier docs are already filed in them as seeded findings, for reviewers to confirm or close.

Rubric: [§3 of the plan](../../plan/authenticity-review.md#3-rubrics). Real-world ranges: [reference.md](reference.md) (written before Phase 1). Evidence for a ledger goes in `bench-results/review/<cat>/`.

## Ledgers

| Ledger | Items | Seeded findings |
|---|---|---|
| [Bathroom](bathroom.md) | 26 | 1: I-bathroom-01 |
| [Bedroom](bedroom.md) | 27 | 0 |
| [Community and farm](community.md) | 36 | 1: A-community-01 |
| [Decor](decor.md) | 22 | 0 |
| [Fitness](fitness.md) | 5 | 1: A-fitness-01 |
| [Garden plants and trees](garden.md) | 53 | 1: I-garden-01 |
| [Hobbies](hobbies.md) | 19 | 0 |
| [Kids](kids.md) | 20 | 3: A-kids-01, I-kids-01, I-kids-02 |
| [Kitchen](kitchen.md) | 72 | 1: I-kitchen-01 |
| [Lighting](lighting.md) | 21 | 1: M-lighting-01 |
| [Living room](living.md) | 50 | 5: M-living-01, M-living-02, A-living-01, I-living-01, I-living-02 |
| [Office](office.md) | 14 | 3: A-office-01, I-office-01, I-office-02 |
| [Outdoor](outdoor.md) | 43 | 1: M-outdoor-01 |
| [Personality packs](packs.md) | 94 | 5: M-packs-01, M-packs-02, M-packs-03, A-packs-01, A-packs-02 |
| [Tabletop](tabletop.md) | 39 | 4: I-tabletop-01, I-tabletop-02, I-tabletop-03, I-tabletop-04 |
| [Residents](residents.md) | 161 | 8: A-residents-01, A-residents-02, A-residents-03, A-residents-04, A-residents-05, A-residents-06, A-residents-07, A-residents-08 |
| [Houses](houses.md) | 83 | 2: M-houses-01, A-houses-01 |
| [Town](town.md) | 42 | 5: M-town-01, M-town-02, M-town-03, M-town-04, A-town-01 |
| [Held items and tools](held.md) | 45 | 1: I-held-01 |
| [Social interactions](social.md) | 30 | 3: A-social-01, A-social-02, I-social-01 |
| [Systems](systems.md) | 46 | 6: A-systems-01, I-systems-01, I-systems-02, I-systems-03, I-systems-04, I-systems-05 |
| **Total** | **948** | **52** |

The pilot (the 28 `base.json` objects and the residents) filled its rows on 2026-10-21; what it found about the rubric, the format and the tools, and the changes it proposes, are in [pilot-notes.md](pilot-notes.md).

The 15 object ledgers follow `docs/objects/<cat>.md` group by group and hold all 541 buyable objects, each in exactly one ledger.

**Consolidated (2026-10-21)** into [docs/authenticity-review.md](../authenticity-review.md): 36 findings with a shared cause moved to [systems.md](systems.md) and [residents.md](residents.md), keeping their ids, with a pointer (`### <id> — moved to <ledger>.md`) left where they were; 15 duplicates closed into the older finding; every open finding put in one fix group (G-01 to G-72) or listed as not checked.

## Writing a finding

From [§4 of the plan](../../plan/authenticity-review.md#4-findings-format-and-severity):

**Ids:** `M-<cat>-NN`, `A-<cat>-NN`, `I-<cat>-NN` (for example `M-kitchen-07`), where the category is the `docs/objects/` page or `residents`, `houses`, `town`, `social`, `systems`.

**Severity**
- **S1 — breaks belief:** wrong at a glance in normal play (a resident floats above the bed, a door opens into the wall, a child-sized adult chair, a use from the back of the object, an adult using a child's or a baby's thing, or a child an adult's, as if it fitted).
- **S2 — noticeably wrong:** a careful player sees it (hands hover, a counter 10 cm too low, a 2-minute shower, a missing obvious use).
- **S3 — polish:** only on close inspection or a side-by-side.

A `decision` finding takes the severity of its worse outcome.

**Fix class:** `model` (Blender rebuild), `content` (numbers or fields), `renderer-table` (`characters/furniture.json`, `ACTIONS`, `STATION_PROPS`), `clip` (resident clip), `motion` (object clip or manifest `motions`), `code` (new behaviour), `decision` (needs the owner).

**Ledger row** (`docs/review/<cat>.md`, one table per group, in the order of `docs/objects/<cat>.md`):

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fridge` | OK | A-kitchen-03 | OK | A-kitchen-03 S2 motion: freezer door opens at use start, not at the hand | `kitchen/sheets/obj-fridge.png` |

Shots stay in the review worktree's gitignored `bench-results/review/<cat>/`, not in git. The Shots cell names the object's contact sheet (`<cat>/sheets/obj-<id>.png`, from `check.mjs sheet`) and at most two more shots, as paths under `bench-results/review/`.

Under each group, a table **Real uses against the content's** (I1) lists each item's real-world uses next to the content's uses.

A **resident's pose at an object** (reach, eye line, contact with its parts) is filed under I3 with A1 in its Rubric field; the A column names the same id.

A **cause shared by many objects** (a sim rule such as `Command::Use`, an `ACTIONS` entry, a clip) goes in the ledger of the first object it was seen on, naming every other affected item; other rows point to it with "(<ledger> ledger)". The coordinator moves it to `systems.md` or `residents.md` at the merge. Each object wave judges the `ACTIONS` entries its objects use, at those objects.

In a ledger's Findings section a finding is written as:

```markdown
### A-kitchen-03 — The freezer door opens at use start, not at the hand

- **Severity:** S2
- **Rubric:** A10
- **Fix class:** motion
- **Where:** `web/public/assets/packs/kitchen/manifest.json` (`freezer_open`)
- **Evidence:** `review/kitchen/fridge-*.png`; what was seen, measured or compared
- **Status:** one of `open`, `seeded (from <source>)`, `confirmed`, `confirmed and widened`, `confirmed fixed on <item>`, `not checked (<why>)`, `closed (<why>)`; in Phase 4 also `fixed (<commit>)`, `later`, `won't fix (<why>)`
- **Source:** who found it, in which pass
```

The held-items ledger uses `held` as its category (`I-held-01`).

Ids are numbered per ledger and letter (`M-kitchen-01`, `A-kitchen-01`, `I-kitchen-01`, …) and never reused. The ledger row names the id in its M, A or I cell and in Findings.

## Owner decisions (2026-10-21)

From [§9 of the plan](../../plan/authenticity-review.md#9-owner-decisions-2026-10-21):

1. **Sizes:** functional heights (seat, counter, table, desk, mattress, basin, toilet seat) must be inside the real-world range. Overall size and parts may stray about ±10 % for readability from the camera.
2. **Durations:** today's short durations are by design. The ledger records them as "by design" and flags only outliers against similar uses (more than 3× from the median for the tag).
3. **New behaviour** (props in hand, emotional body language, a turn-on-the-spot clip, doors opened by hand, hard age rules, a privacy flag): the review files it as `code` findings only. It is planned separately later.
4. **Decor without uses** (default): decor stays decor. A clear missing use is an I1 finding, decided at triage.
5. **Residents' proportions** (default): changes to `rig.rs` are filed only and decided at triage.
6. **Pace** (default): the review runs as its own workstream now, on branch `review/authenticity`, through Phase 4.

After the pilot (2026-10-21) the owner approved: M14's head ratios based on the style (half a head bigger); a new rubric line **I16 conditions** (a use that needs a time, place or state has it); a **1 cm measuring tolerance** on functional heights (5 mm on table tops); the bookkeeping rules above; the S1 example for adults and children using each other's things; and decisions rated by their worse outcome.

## Tooling

Known tool problems, kept here rather than in a ledger:

- **The lamp build checks only the light's height, not `at`** (`_bulb`, `tools/art/blender/families/lighting.py:94`). From `docs/objects/open.md` L158.
- **`pnpm wasm` doesn't run in a worktree with a symlinked `node_modules`** (pnpm's deps check); its cargo + wasm-bindgen command works. From `docs/objects/open.md` L159.
- **`partBox` bounds spheres and tori up to 5 % / 2.5 % bigger than Babylon draws them** (never smaller; `web/src/render/babylon/models.ts:377`). It matters when measuring the code-built fallback models. From `docs/objects/open.md` L137.
- **Previews overwrite each other:** ids used in two families (bench, flowers, hedge, arcadeCabinet, puzzleTable, recliner) write the same preview file (`previews/` under `tools/art/blender`, made by `cargo art --preview` and ignored by git), and 161 models have no preview. From `plan/authenticity-review.md` §5.2 (E1).

## Known issues not in a ledger

Listed in the seed sources but not about models, animation or interactions:

- The town map's backdrop blocks the world (a UI panel; `docs/objects/open.md` L130).
- Mirrors on High log an occasional `GL_INVALID_VALUE: glGetProgramiv` (no visible fault, not reproduced; `docs/objects/open.md` L135).
- The production build's frame time (performance; `docs/objects/open.md` L163).
- Fridges and cookers count as one kind for wants (balance; `docs/release-0.2.0-review.md` §4, F9).
- Residents choose things 57–115 game minutes' walk away (autonomy balance; R3-17).
- Low keeps its forward lamps dark by day (performance; P1-02).

## Seed sources

- `docs/objects/open.md` "Still open" (L113 to the end).
- `plan/release-0.3.0.md` §1.4 (model and animation polish), §2.5 (parts with no use), §3.2 and §3.3 (carry-over).
- `docs/release-0.2.0-review.md` §4 "Found but not fixed" (R1-04, R1-05, R1-12, R1-15 and the other model, animation and interaction rows).
- `CHANGELOG.md` 0.3.0 (furniture can't be resized: moving parts don't follow a resize).
- `plan/authenticity-review.md` §3 and §9: the gaps the rubric itself names as known (no props in hand, body language, age rules, privacy, doors by hand, turning on the spot).
