# Fitness: authenticity ledger

What it covers: the 5 objects of [docs/objects/fitness.md](../objects/fitness.md), in its groups and order.

- **Evidence:** `bench-results/review/fitness/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## F1. Home gym (5)

Family: `tools/art/blender/families/fitness.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `treadmill` Treadmill | OK | A-fitness-02 | I-bathroom-04 | A-fitness-02 S3 motion; I-bathroom-04 S3 code (bathroom ledger: no sound) | `sheets/obj-treadmill.png`, `play/treadmill-run-*.png`, `clips/clips-treadmill-all-t*.png`, `play-child/treadmill-run-*.png`, `play-elder/treadmill-run-*.png` |
| `weightBench` Weight bench | M-fitness-01 | A-fitness-01 | OK | A-fitness-01 S2 clip (seeded; confirmed); M-fitness-01 S3 model | `sheets/obj-weightBench.png`, `play/weightBench-lift-*.png`, `clips/clips-weightBench-all-t*.png` |
| `exerciseBike` Exercise bike | — | — | — | | |
| `yogaMat` Yoga mat | — | — | — | | |
| `dumbbells` Dumbbell rack | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `treadmill` | walk or run, warm up | go for a run (60) | OK |
| `weightBench` | bench press, lift weights | lift weights (45, lying) | OK |

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
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred); pilot

### M-fitness-01 — The weight bench's pad is 50 cm high, over a flat bench's 40–48

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/fitness.py:111` (`weightBench`); `web/src/render/babylon/characters/furniture.json:55` (`beds.weightBench` 0.5)
- **Evidence:** `sizes-fitness.md` measures the lie height 0.50 m; `reference.json` `weightBench` gives a seat of 0.40–0.48 (a bench press bench is about 43–45 cm so the lifter's feet reach the floor). The sizes table can't measure it as a seat ("no seat to measure"): the content gives it a lie, not a seat. Lowering the pad goes with A-fitness-01's feet-down pose.
- **Status:** open
- **Source:** pilot

### A-fitness-02 — The treadmill's belt crawls under a running stride

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/fitness.py:100` (`G.scroll(slats, 'belt_run', (0, -SLAT, 0), seconds=0.25)`, `SLAT = 0.1` at `:55`)
- **Evidence:** One 0.1 m slat each 0.25 s is a belt speed of 0.4 m/s, a slow walk; the runner jogs (`jog`, 0.7 s a cycle) on it (`play/treadmill-run-01.png`, `clips/clips-treadmill-all-t*.png`). A run is 2.2–3.3 m/s; the belt would need about 0.04 s a slat (the README keeps a loop's steps above the 1/30 s sampling, so a longer slat pitch or a blurred texture). Close to the eye only; at the game camera the belt reads as moving.
- **Status:** open
- **Source:** pilot
