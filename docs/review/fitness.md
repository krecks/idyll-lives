# Fitness: authenticity ledger

What it covers: the 5 objects of [docs/objects/fitness.md](../objects/fitness.md), in its groups and order.

- **Evidence:** `bench-results/review/fitness/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Wave rows** (activities wave, fitness and hobbies, 2026-10-21): Shots are paths under `bench-results/review/` in the review worktree (`fitness/…`, `hobbies/…`); the shots this wave took are in `fitness/extra/` and `hobbies/extra/`, how in `hobbies/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## F1. Home gym (5)

Family: `tools/art/blender/families/fitness.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `treadmill` Treadmill | OK | A-fitness-02 | I-bathroom-04 | A-fitness-02 S3 motion; I-bathroom-04 S3 code (systems ledger: no sound) | `sheets/obj-treadmill.png`, `play/treadmill-run-*.png`, `clips/clips-treadmill-all-t*.png`, `play-child/treadmill-run-*.png`, `play-elder/treadmill-run-*.png` |
| `weightBench` Weight bench | M-fitness-01 | A-fitness-01 | I-systems-04 | A-fitness-01 S2 clip (seeded; confirmed); M-fitness-01 S3 model; I-systems-04 S2 code (systems ledger: an 8-year-old bench-presses the loaded bar, wave shot `fitness/extra/child-weightBench-lift-01.png`) | `sheets/obj-weightBench.png`, `play/weightBench-lift-*.png`, `clips/clips-weightBench-all-t*.png` |
| `exerciseBike` Exercise bike | OK | A-hobbies-01 | A-hobbies-01, I-hobbies-09 | A-hobbies-01 S2 code (systems ledger: gets on through the handlebars); I-hobbies-09 S3 code (hobbies ledger: no flywheel whirr) | `fitness/sheets/obj-exerciseBike.png`, `fitness/extra/side-exerciseBike-cycle-00.png`, `fitness/sheets/age-exerciseBike.png` |
| `yogaMat` Yoga mat | M-fitness-02 | OK | I-fitness-01 | M-fitness-02 S3 model; I-fitness-01 S3 clip | `fitness/sheets/obj-yogaMat.png`, `fitness/clips/clips-yogaMat-all-t99.png`, `fitness/play/c1/yogaMat-yoga-01.png` |
| `dumbbells` Dumbbell rack | OK | OK | I-fitness-02, I-systems-04 | I-fitness-02 S3 model; I-systems-04 S2 code (systems ledger: a child curls the adult pair) | `fitness/sheets/obj-dumbbells.png`, `fitness/extra/side-dumbbells-lift-00.png`, `fitness/extra/child-dumbbells-lift-01.png` |

Wave notes on the rows:
- `exerciseBike`: 0.53 × 1.11 × 0.90 m and the saddle at 0.75 m (content 0.76, `reference.json` 0.75–1.05) are in range; flywheel to the front, console, cranks and stabilisers are all there. The pedals turn under the feet at every stage, the hands on the grips (`sitHands`, `ACTIONS.exercise`, `characters.ts:481`); a child's feet still reach the pedals (`fitness/sheets/age-exerciseBike.png`, `fitness/extra/child-exerciseBike-cycle-01.png`). The rider gets on through the handlebars and flywheel, from the front tile to a saddle 0.12 m behind the middle (A-hobbies-01).
- `yogaMat`: the `sizes` table measures the mat rolled up at rest (W 0.92 is the block and the bottle, H 0.23 the bottle), so its three "out" are the rest pose, not a finding; the flat mat is M-fitness-02. The seat (0.03 m) is on the unrolled mat (1.6 cm thick); the mat unrolls as the resident sits and rolls up after.
- `dumbbells`: D 0.71 m includes the rubber floor mat in front; the rack itself is 0.41 m deep (0.30–0.60). Curls alternate, a dumbbell in each fist, in the rack's colours (`writeHeld`, `characters.ts:1721`).
- `treadmill`, `weightBench`: pilot rows. The wave adds only the child at the bench (`--sim 3`).

## Real uses against the content's (I1)

Filled by the pilot for its rows and by the activities wave for the rest; durations are judged by §9 decision 2 (none of these is more than 3× from its tag's median: by design).

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `treadmill` | walk or run, warm up | go for a run (60) | OK |
| `weightBench` | bench press, lift weights | lift weights (45, lying) | OK |
| `exerciseBike` | cycle, ride a programme on the console | cycle (45, seated) | OK |
| `yogaMat` | a yoga flow of standing and lying poses, stretch, meditate | do yoga (30, seated cross-legged) | the poses missing: it is a meditation (I-fitness-01) |
| `dumbbells` | curls, presses, rows | lift dumbbells (30, standing curls) | OK |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-fitness-01 — The weight bench's lie: the legs stretch past the bench in mid-air

- **Severity:** S2
- **Rubric:** A1, A2
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/furniture.json:55` (`beds.weightBench`, read as `BEDS` at `characters.ts:260`), `web/src/render/babylon/characters/clips/lie.ts`
- **Evidence:** The lifter lies along the bench and the legs stretch out past its end in mid-air; in a real bench press the feet are on the floor either side. Needs a bespoke feet-down lie pose, not a data tweak. Deferred from 0.3.0.
  Pilot: `play/weightBench-lift-02.png`: the legs lie straight along the pad and the heels rest on the leg-roller pads, not on the floor; the bar is in the hands over the chest and gone from the rack (`bar_off` after 0.8 s). Same for the child and elder (`play-child/`, `play-elder/`). The `BEDS` row is now in `furniture.json` (the old `characters.ts:315` line no longer holds it).
- **Status:** confirmed (pilot)
- **After (model half, 8b6b2364):** the bench is ready for a feet-down lie: its pad at 0.44 m (`beds.weightBench`), the leg rollers at its foot gone (where the shins would come down). The lie itself is still the clip's: the legs lie along the pad (`fixes/blender-hobbies/poses/poses-seats-adult.png`).
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred); pilot

### M-fitness-01 — The weight bench's pad is 50 cm high, over a flat bench's 40–48

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/fitness.py:111` (`weightBench`); `web/src/render/babylon/characters/furniture.json:55` (`beds.weightBench` 0.5)
- **Evidence:** `sizes-fitness.md` measures the lie height 0.50 m; `reference.json` `weightBench` gives a seat of 0.40–0.48 (a bench press bench is about 43–45 cm so the lifter's feet reach the floor). The sizes table can't measure it as a seat ("no seat to measure"): the content gives it a lie, not a seat. Lowering the pad goes with A-fitness-01's feet-down pose.
- **Status:** fixed (8b6b2364)
- **After:** a flat bench, the pad's top at 0.44 m (`fitness.py` `PAD_TOP`; `characters/furniture.json` `beds.weightBench` 0.44, was 0.5), the leg-roller arm gone; `cargo review sizes`: mattress 0.44, inside 0.40–0.48. `fixes/blender-hobbies/gallery/gallery-weightBench.png`, `play/adult-weightBench-lift-01.png`. The feet-down pose is A-fitness-01's clip.
- **Source:** pilot

### A-fitness-02 — The treadmill's belt crawls under a running stride

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/fitness.py:100` (`G.scroll(slats, 'belt_run', (0, -SLAT, 0), seconds=0.25)`, `SLAT = 0.1` at `:55`)
- **Evidence:** One 0.1 m slat each 0.25 s is a belt speed of 0.4 m/s, a slow walk; the runner jogs (`jog`, 0.7 s a cycle) on it (`play/treadmill-run-01.png`, `clips/clips-treadmill-all-t*.png`). A run is 2.2–3.3 m/s; the belt would need about 0.04 s a slat (the README keeps a loop's steps above the 1/30 s sampling, so a longer slat pitch or a blurred texture). Close to the eye only; at the game camera the belt reads as moving.
- **Status:** fixed (abfc32d9)
- **After:** the slats are 30 cm apart and `belt_run` moves them one on in 0.13 s: 2.25 m/s, a run, and under half a pitch a frame at 60 frames a second up to the fastest game speed, so they're seen going back (`fixes/motions/g1718/clips-treadmill-all-t*.png`; `cargo check-glb`: the loop moves them 30 cm, the belt's warning).
- **Source:** pilot

### M-fitness-02 — The yoga mat unrolls to 0.9 m, half a real mat's length

- **Severity:** S3
- **Rubric:** M1, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/fitness.py:193` (`MAT_W, MAT_L = 0.62, 0.9`); the footprint `[1, 1]` at `web/public/content/furniture.json:3567`
- **Evidence:** `fitness/clips/clips-yogaMat-all-t99.png`, `fitness/play/c1/yogaMat-yoga-01.png`: unrolled, the mat is 0.62 × 0.90 m; a yoga mat is 0.61 × 1.73–1.83 (`reference.json` `yogaMat` 0.6–0.7 × 1.7–1.9). For the seated use it shows as a square cushion under a cross-legged resident; any standing or lying pose (I-fitness-01) needs the full length, so a 1 × 2 footprint. Width, thickness (1.6 cm, over the floor's 1 cm), the roll, block and bottle are fine. The `sizes` table can't see this: it measures the mat rolled up (row note).
- **Status:** fixed (8b6b2364)
- **After:** a 1.8 × 0.62 m mat on a 1 × 2 footprint (content), rolled at rest on the wall's face; the yogi stands 0.4 m in front of its middle (`clips/games.ts` `YOGA_AT`, was 0.3 on the 0.9 m mat), so the fold, the dog and the squat are on it. `fixes/blender-hobbies/play/flow-yogaMat-yoga-0*.png`, `clips/clips-yogaMat-all-t99.png`. The flattened mat's origin is under the roll now (`check-glb` still warns of 42 % of its strip showing, from 100 %).
- **Source:** wave: fitness

### I-fitness-01 — "Do yoga" is half an hour sitting cross-legged

- **Severity:** S3
- **Rubric:** I1, I3
- **Fix class:** clip
- **Where:** `web/public/content/furniture.json:3585` (`yoga`: `anim: meditate`, `pose: sit`); `web/src/render/babylon/characters.ts:532` (`ACTIONS.meditate`: the idle) and `:3269-3285` (the cross-legged layer)
- **Evidence:** `fitness/play/c1/yogaMat-yoga-01.png`: the resident sits cross-legged on the mat with the hands on the knees, breathing, eyes shut, for the whole 30 minutes; the content's description says "Yoga stretches the body". A sitting meditation is one part of a yoga session, so it reads as yoga at a glance; a careful player sees no pose ever change. A short sequence of held poses (downward dog, warrior, a forward fold, then the seated end) is new resident clips, and wants the full-length mat (M-fitness-02). The same `meditate` anim is used by `hotHeaded.json:93`, `:104`, where a meditation is what is meant.
- **Status:** fixed (4366ec55, 83d53185)
- **After:** a 48 s flow on the unrolled mat from its front end (`yoga`, `STEP_IN`; content `anim: exercise`, standing, the mat's `seat` gone): mountain, the hands to the heart, an upward salute, a forward fold, a half lift, the hands walked out to a short downward dog (heels pedalling), a lunge up to warrior I, back to the dog, a child's pose, sitting on the heels, up through a squat; open hands. `bench-results/review/fixes/clips-games/after-yogaMat-yoga-0*.png`, `child-yogaMat-yoga-0*.png`. The dog and lunge overhang the 0.9 m mat (G-40, M-fitness-02); the cool-down cushion keeps the seated meditation.
- **Source:** wave: fitness

### I-fitness-02 — The lifter curls beside the rack's rubber mat, not on it

- **Severity:** S3
- **Rubric:** I2, M4
- **Fix class:** model
- **Where:** `tools/art/blender/families/fitness.py:250` (`floor_mat` 0.6 × 0.36 at depth 0.2, inside the footprint)
- **Evidence:** `fitness/extra/side-dumbbells-lift-00.png`, `fitness/sheets/obj-dumbbells.png` (play): the resident stands on the tile in front, as every standing use does, and the mat lies empty between them and the rack. A mat in front of a rack is where you stand to lift. Either drop the mat (the rack alone fits 0.41 m deep) or move the rack back so the mat reaches the tile in front; the game can't stand them on the footprint.
- **Status:** fixed (8b6b2364)
- **After:** the rack stands back on the wall's face (`fitness.py` `RACK_SHIFT`) and a 0.7 × 0.48 m mat fills the front of its tile; the lifter steps in onto it (`characters.ts` `STAND_BACK.dumbbells` -0.6) and curls there. `fixes/blender-hobbies/play/adult-dumbbells-lift-01.png`. (`cargo review sizes` now measures 0.93 m deep, the mat counted with the rack: the tool's known overcount, §7.)
- **Source:** wave: fitness
