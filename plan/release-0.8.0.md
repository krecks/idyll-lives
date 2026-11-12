# Idyll Lives 0.8.0 — Release Plan

- **Branch:** `release/0.8.0`, cut from `main` at `f3abe844` (v0.7.0) on 2026-11-12; to be merged into `main` by pull request; tag `v0.8.0` on the merge
- **Base:** v0.7.0 at `f3abe844`
- **Release type:** features: big towns only, real junctions and the traffic's own clock, residents with real proportions and faces, the look at the play camera, stairs in three shapes, four more bikes; save format 37

## 1. What ships

The owner's call (2026-11-12): every branch with finished work, together, as 0.8.0.

- **`feat/big-worlds-only`** (`06d00894`): only big towns; junctions lit by phases with a walk phase, crosswalks and stop lines, drawn as junctions (pads, kerb returns, markings, walk heads); the traffic's own clock (`DRIVE_SCALE`, save 37); resolution as 720p–2160p; the dev play log; garden gates against houses; residents on the land; bedtime for a family with a grown child and the sofa slept on under a blanket; a house that looks the same from the street and inside.
- **`feat/new-items`** (`bee2871b`): an open-tread flight and a spiral staircase; the mountain, road, cargo and e-bike; roofs that meet at a corner roofed whole.
- **`feat/models`** (`972bd6bb`, all of `feat/look-jump` and the `jump/*` branches): residents' real proportions, a face per resident, ears, skin, eyes, hair, hands and clothes; classic shading the default; contact shading on Medium and High; rooms in the shine; leaf cards on the broadleaves; the look jump at the play camera (lawn map, warm sun and cool shade, cloud shadows, worn streets and paths, hip caps, petals, plants lit from above); the close-up body only in the creator.

Not in it (the owner's call): `feat/work-jitter`, `fix/vehicle-speeds` (superseded by the drive clock), `backup/picture-settings-2026-11-11`, `wip/inspect-debug`.

### The merge

- `feat/models` against the other two: the road surface keeps both the junctions' pads and markings and the worn asphalt (`roadMeshes` takes the junctions and then `worn`; a pad is toned `STREET_LOOK.junction`, the tone the worn roads fade to where they meet; a junction's corner sidewalk and its stretch under the arms' in the street look's walk, kerb and gutter shade, with the course of kerb stones at its back); the presets keep `contactShading` and `cloudShadows` without `pixelBudget`; settings default to classic shading and 1080p; the bench keeps both sides' cameras and the new resolution flag.
- Tests: the residents' recording re-recorded with both sides' changes; a plain roof's triangle bound raised for the hip caps (24 → 40); the view stubs' `setMorphs`; `web/tests/residentsGround.test.ts` (left untracked on `feat/big-worlds-only`) added.
- **A fix the merge brought to light** (`db303aea`): on seed base 0 a work taxi in Primrosebrook stood pulled over its full room on a lane bending left, a corner 5 mm over the kerbside furniture. Each branch alone passed; together the households' days and bikes differ. Pull-over room was worked out as on a straight road; it now takes off how far a body's corners stand out on the outside of a bend to the left (`cars::bend_out`, the chord's sagitta, about 7 cm for a taxi on a 40 m bend). `grid_identity` re-blessed (the two saved towns with traffic, after two game hours; their structure unchanged).
- **A taxi across the grass** (fixed after the merge, in `feat/big-worlds-only`'s code: that branch alone failed seed base 19 with a taxi 6.17 m off the paving): where a lane goes a little past a road's end and back before it turns off, `cars::round_corners` rounded the corner from that step back, and took out every point of the road before it as lying ahead of the curve's start; the taxi drove 130 m straight across the grass between two lanes (Applewold). Such a corner is left as it is now, and rounding takes out only the points about a corner. `grid_identity` unchanged.
- **The review's gates** (`a00b3ccb`): `feat/new-items` added six objects without kinds in `docs/review/reference.json` or the shipped count: the open flight is a straight stair's kind, the spiral a new `house.spiralStairs`, and the four bikes have kinds of their own (556 objects).

## 2. Version

`cargo release-version 0.8.0`; CHANGELOG `## 0.8.0 — 2026-11-12` (the Unreleased section, with entries added for big towns only, the stairs, the bikes, the residents, classic and contact shading, rooms in the shine and the leaves); README 0.8.0, save format 37 and the way back to 0.7.0; PLAN.md's status line, the 0.8.0 paragraph, the roadmap (0.7.1's leftovers as 0.8.1, the M1 work as 0.9.0); "dropped in 0.7.1" in the code's comments as 0.8.0.

## 3. Release gates (local)

On `release/0.8.0`, CI's Bun 1.4.3:

| Gate | Result |
|---|---|
| `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (185 + 789, 23 ignored) |
| generated houses and worlds, seed base 0 | pass (4), after `db303aea` and again after `48134d64` |
| `bun run check:clips` | 321 clips, 0 new, 0 known |
| `bun run check:chunks` | 235 anchors and names found in PlayCanvas 2.23.3 |
| `bun run test` | pass (419 in 78 files) |
| `cargo test -p ll-tools --release --locked` | pass, after `a00b3ccb` |
| `cargo review sizes`, `cargo review uses --check`, `cargo check-glb` | pass |
| `bun run check`, `bun run lint` | 0 errors, 0 warnings; clean |
| generated, seed bases 0 and 16–23 (16–20 first, 21–23 unseen), after `48134d64` | 0, 17, 18, 19, 22, 23 pass; 16 (Sandsea: a street tile fenced off), 20 and 21 (the sampler: two roads side by side) fail in the world generator, the same on `feat/big-worlds-only` and `feat/new-items` alone (known, PLAN.md's 0.8.1 row) |

Pictures (headless Chromium on the M3 Pro, the dev server, `render.mjs --shots`): the village and forest scenes (town, wide, street, play) on Medium/WebGPU, and a real Big world (`bench=bigworld`) on Medium and Low, WebGPU and WebGL2: the junctions' pads the worn roads' tone, the sidewalks turning the corners on their kerbs, the crosswalks and stop lines; the worn street, its kerbs and the porch.

## 4. Release steps

1. Push `release/0.8.0`; pull request into `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end (tickbench, the voice tests, `check:voice`, the build under 900 MB).
3. Tag and push: `git tag -a v0.8.0 -m "Idyll Lives 0.8.0: big towns that work"`, `git push origin v0.8.0`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the large files' sizes.
5. `gh release create v0.8.0 --verify-tag --title "Idyll Lives 0.8.0" --notes-file plan/release-0.8.0-notes.md`.

## 5. Open points (for the owner)

- **World generator leftovers** (not from this release): seed base 16's Sandsea has a street tile shut off by fences; the sampler finds two roads laid side by side on bases 20 (Honey Bottom) and 21 (Ferry Walk), as on 6, 10 and 34 before.
- **Big towns' traffic:** fewer cars held up in view and for far less long than in 0.7.0, but not none (PLAN.md's 0.8.1 row); the busy-looking knot at Canalside, worldgen's shallow country joins, walkers cutting a corner at a kerb wait.
- **Hip caps only on a plain hip roof:** a roof that meets itself at a corner (an L, a U or a ring, roofed whole since 0.8.0) has none.
- **The mountain bike's bars** are 0.61 m across against a real one's 0.70–0.80 (the size review lists it).
- **The new kinds' ranges** (spiral stair, the four bikes) are from common dimensions with Neufert as the source, not researched one by one.
- **Not measured on an M1:** contact shading (+0.5 ms GPU by day on the M3 Pro), the leaf cards, the look jump.
- **The house look mismatch:** a chimney and a painted exterior still differ between far and in-game.
