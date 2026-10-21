# Fitness: authenticity ledger

What it covers: the 5 objects of [docs/objects/fitness.md](../objects/fitness.md), in its groups and order.

- **Evidence:** `bench-results/review/fitness/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## F1. Home gym (5)

Family: `tools/art/blender/families/fitness.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `treadmill` Treadmill | — | — | — | | |
| `weightBench` Weight bench | — | — | — | A-fitness-01 S2 clip (seeded) | |
| `exerciseBike` Exercise bike | — | — | — | | |
| `yogaMat` Yoga mat | — | — | — | | |
| `dumbbells` Dumbbell rack | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-fitness-01 — The weight bench's lie: the legs stretch past the bench in mid-air

- **Severity:** S2
- **Rubric:** A1, A2
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:315` (`BEDS.weightBench`), `web/src/render/babylon/characters/clips/lie.ts`
- **Evidence:** The lifter lies along the bench and the legs stretch out past its end in mid-air; in a real bench press the feet are on the floor either side. Needs a bespoke feet-down lie pose, not a data tweak. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L116)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred)
