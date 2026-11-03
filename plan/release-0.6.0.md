# Idyll Lives 0.6.0 — Release Plan

- **Branch:** `release/0.6.0`, cut from `main` at `58f4b1a1` on 2026-10-30, merged into `main` by pull request (#50); tag `v0.6.0` on the merge
- **Base:** v0.5.1 at `7aa616f9`
- **Status:** **Released 2026-11-03**: https://github.com/krecks/idyll-lives/releases/tag/v0.6.0 (PR #50 merged as f4482e7f; CI 41559543728; dry run 41562436343 on main; the tag's deploy 41564007742; live checks passed).
- **Release type:** features: cars, bigger open worlds, garden fences and garages, the rebuilt models and generated textures, and a performance and loading review; save format 35

## 1. What ships

Player notes: the `## 0.6.0` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.6.0-notes.md](release-0.6.0-notes.md).

| PR | What |
|---|---|
| #41 | Cars and garages: households' cars to work and back, riders, parking, garages on the driveways; basement and lawn fixes (save format 34) |
| #42 | Organic towns: winding roads, lots fitted to them, home prices |
| #43 | One simulation test binary (`crates/sim-core/tests/sim/`), its slowest tests made quicker |
| #44 | Every texture generated in code (`cargo tex synth`), original CC0 |
| #45 | New items: cut flowers, the daisy chain, held dishes, the hood light, doors by hand, the servers' apron |
| #46 | Game kit quality pass: every model rebuilt (clips from 0 s, flat caps, rounded bevels), shaped contact shadows |
| #47 | Bigger, open worlds: Cosy, Normal and Big sizes, hamlets, country roads, a roaming camera, real garden fences, the Garage tool (save format 35) |
| #48 | The release review's fixes (CPU, GPU, workers, animations and models, loading, gameplay UI), two cleanup and two code-review passes, test files isolated |
| #49 | CI's steps in one script (`tools/ci-local.sh`), the tools built once |

The release review of the whole branch (`/code-review` on #50) found and fixed: public things in open towns not found from upstairs or the basement (`index.rs`, the ground under a stacked row), a corrupt save's run count overflowing its check (`save.rs` `expand`), and a v35 save whose pairs are all as new rebuilding its relationships on load. Left as it is: `lights::while_lit` turns each resident's object into a stable id and back (a scan of the objects per resident under a hood or a lamp that follows a use, per tick).

- **Saves:** format 35. 0.5.x's saves (format 33) open in 0.6.0 as Cosy towns with no cars in the simulation; 0.5.x can't open 0.6.0's saves (it keeps them and lists only those it can open), so 0.5.1 is a build to go back to only with that in mind.
- **Known:** loading a Big world holds the game once for 0.3–1.1 s while every resident's commute is worked out. Low on WebGL2's close orbits stay 11–16 % over Babylon's GPU time (`docs/design/performance.md`), as in 0.5.x.

## 2. Version

`cargo release-version 0.6.0`; CHANGELOG `## 0.6.0 — 2026-11-03` (the review's performance, loading and interface work added, saves corrected to format 35); README 0.6.0 and the saves; PLAN.md's status line points here.

## 3. Release gates (local, 2026-11-03)

`tools/ci-local.sh` (every CI step, on CI's Bun 1.4.3, and the web's types and lint), the voice's tests, the production build, and the game played.

| Gate | Result |
|---|---|
| `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (141 + 664) |
| generated houses and worlds (`--ignored`, Cosy/Normal three per kind, Big one per kind) | pass |
| `bun run check:clips` | 313 clips, 0 new, 0 known |
| `bun run check:chunks` | 195 anchors and names found in PlayCanvas 2.23.3 |
| `bun run test` (each file isolated) | pass (281) |
| `cargo test -p ll-tools --release --locked` | pass (112) |
| `cargo review sizes`, `cargo review uses --check`, `cargo check-glb` | pass (629 models, 0 failed) |
| `bun run check`, `bun run lint` | 0 errors, 0 warnings; clean |
| `cargo test -p voice --release --locked`, `bun run check:voice` | pass (32); pitch, speed and depth work |
| `cargo run -p sim-core --example tickbench` | 1.4 µs/tick with socials |
| `cargo release-version` | 0.6.0 everywhere |
| `vite build --base=/idyll-lives/` | pass, 675 MB (under 900) |
| The game | a saved game at speed 3 resumes at 3, residents act, pause and resume, birds, no console errors; the production build's benchmark (village, Medium) on WebGPU and WebGL2: 0.6.0, cross-origin isolated, 60 fps, no page errors |

## 4. Release steps

All done 2026-11-03. Live: the bundle's build is f4482e7 (the release merge), it reports 0.6.0 and is cross-origin isolated; the village benchmark (Medium) runs at 60 fps on WebGPU and WebGL2 with no page errors.

1. Pull request `release/0.6.0` → `main` (#50); CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.6.0 -m "Idyll Lives 0.6.0: cars, bigger open worlds, generated textures"`, `git push origin v0.6.0`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors.
5. `gh release create v0.6.0 --verify-tag --title "Idyll Lives 0.6.0" --notes-file plan/release-0.6.0-notes.md`.
