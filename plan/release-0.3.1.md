# Idyll Lives 0.3.1 — Release Plan

- **Branch:** `main`; tag `v0.3.1` on d4caf7b7
- **Base:** v0.3.0 at `e725166`
- **Status:** **Released 2026-10-22**: https://github.com/krecks/idyll-lives/releases/tag/v0.3.1 (dry run 39724920321, the tag's deploy 39726588344, live checks passed).
- **Release type:** authenticity: every model, animation and interaction checked against the real thing, and what was found fixed

## 1. What ships

The authenticity review ([plan](authenticity-review.md), results in [docs/authenticity-review.md](../docs/authenticity-review.md), ledgers in [docs/review/](../docs/review/README.md)): 342 findings, 281 fixed (all 8 that broke belief), 9 in part; the owner signed off every decision on 2026-10-22 (§11.3 there). Notes for players are the `## 0.3.1` section of [CHANGELOG.md](../CHANGELOG.md).

What a player or a tester should know:
- **Saves:** format 32 (30 laundry, 31 outfits, 32 the school day). 0.3.0's saves load and play on; 0.3.0 can't open 0.3.1's (it keeps and skips them), so if this release goes wrong, fix forward rather than redeploy 0.3.0 for long.
- **Old saves next to grown footprints:** things placed beside an object whose footprint grew (hammocks, the ping-pong and park chess tables, the drum kit, the yoga mat, the digital piano, the swing set, the hay bale, the garden bench) may overlap it in an old save. Early release: not repaired.
- **Deferred to a later release** (owner decision 3): props in hand (G-70), body language and turning on the spot (G-71), privacy (G-72), doors opened by hand.

## 2. Version

`node tools/release/version.mjs 0.3.1` set `web/package.json`, the workspace in `Cargo.toml` and the five crates in `Cargo.lock`. CHANGELOG has `## 0.3.1 — 2026-10-22` (the tag's run checks it). README says 0.3.1 and save format 32; PLAN.md's status line points here.

## 3. Release gates (local, 2026-10-22)

Run on the M3 Pro from the repository root (web/ for the pnpm ones). Results:

| Gate | Result | Time |
|---|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass | 30 s |
| `cargo test -p sim-core --locked` | pass | 2 min 19 s |
| `cargo test -p ll-tools --release --locked` | pass (577 models in the gate test; `cargo check-glb` 587, 0 failed) | 1 min 16 s |
| `cargo run -p sim-core --example tickbench --release` | 850 ns/tick with socials, 1.17 µs without | 47 s |
| `cargo test -p voice --release --locked` | pass | 7 s |
| `node tools/release/version.mjs` | 0.3.1 everywhere | — |
| generated houses (180) and worlds (24), `--ignored` | pass | 1 min 3 s |
| `century` (`--ignored`) | **fail: seed 7 with lower detail off dips to 7 residents at day 132** (floor 8; it recovers to 21 by day 200); seeds 11 and 19 and every lower-detail run pass. At 0.3.0 seed 11 was the one that dipped: the test is chaotic. Owner's call, as at 0.3.0. | 42 s |
| `soak` (`--ignored`) | pass, 4 of 4 | 1 min 9 s |
| `balance` (`--ignored`) | inside PLAN.md's targets | 33 s |
| `pnpm check:voice` | pass | 34 s |
| `pnpm check:clips` | 168 clips, 0 new, 0 known | 2 s |
| `pnpm check` (svelte-check) | 0 errors, 0 warnings | 5 s |
| `pnpm lint` | clean | 15 s |
| `pnpm build` | pass; `dist` 627 MB (CI fails over 900) | 1 min 30 s |

After a `/simplify` pass on top of the release commit (30 `simplify:` commits, no behaviour change: the sim's snapshot and save hashes over 6 game days on four runs, all 168 clips and `cargo review` output identical before and after), clippy, sim-core (641), ll-tools (68), the generated houses and worlds, `check:clips`, svelte-check, eslint and `pnpm build` were run again and pass; tickbench within noise (755–878 ns/tick with socials).

Not run locally: `node tools/bench/render.mjs` (headed timing; the review's lighting work measured Medium WebGPU at 3.35 → 3.47 ms GPU, the browser smoke test ran all presets on both backends without errors: `bench-results/review/smoke/index.md`).

## 4. Still to do (by the owner's go-ahead)

1. ~~`git push origin main`~~ done 2026-10-22.
2. ~~Dry run~~ done: the first (39721490811, on cfaa270b) failed with a SIGSEGV in `ll-tools`' `tests/review.rs` on x86-64, a miscompile of `review::size_row` by Rust 1.97.0 (reproduced in an amd64 container; valgrind: an invalid read at review.rs:551; 1.97.1 passes); d4caf7b7 pins 1.97.1 and the second (39724920321) passed. `gh workflow run pages.yml --ref main -f deploy=false`, then `gh run watch $(gh run list --workflow pages.yml -L1 --json databaseId -q '.[0].databaseId') --exit-status`.
3. ~~Tag and push~~ done (39726588344): `git tag -a v0.3.1 -m "Idyll Lives 0.3.1: the authenticity review"`, `git push origin v0.3.1`; watch the tag's run (`--event push`), starting no other run while it waits.
4. ~~Live checks~~ done: menu v0.3.1, Credits "Version 0.3.1 (build d4caf7b)", Shared memory on, a played household runs at 60 fps with no page errors, the LLM file 200 and 255,530,880 bytes, the vector estimator 119,568,968 bytes. Not run: loading a 0.3.0 save on the live site (`old_saves.rs` covers the migration). The checks: the menu and Credits show 0.3.1 (build `<tagged commit>`); F3 Shared memory on, a game runs; `llm/lfm2.5-350m/model_q4.onnx_dataz` 200 and 255,530,880 bytes; `voice/supertonic-3/vector_estimator.onnxz` 119,568,968 bytes; a 0.3.0 save loads.
5. ~~Release~~ done: `gh release create v0.3.1 --verify-tag --title "Idyll Lives 0.3.1" --notes-file plan/release-0.3.1-notes.md`.
6. Manual checklist before or after tagging: `docs/authenticity-review.md` §10 (the pie menu greying, a headed listen to the new sounds, a joiner walking into a group were never shot by an agent).

## 5. Risks

- **The generated-worlds check is random** (`tools/worlds/sample.ts` picks households with `Math.random`): a rare layout can fail it in CI; I-systems-17 fixed the one seen, 132 draws since passed.
- **Chaotic long tests:** `shipped_content`'s personality run and the busy soak depend on the course of a run; both pass, but content changes flip them (§11.3 (b) in the review).
- **`century`** is chaotic: at 0.3.0 seed 11 dipped below its floor of 8 residents, now seed 7 with lower detail off does (§3). It isn't in CI. Either loosen the floor (several seeds, a share to pass) or look at why small towns shrink mid-century; the owner decided at 0.3.0 to release with it open.
