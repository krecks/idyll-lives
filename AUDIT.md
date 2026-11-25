# Audit — idyll-lives

Six-area review (Review, Bugs, Performance, Maintainability, Simplification, Split large files) of the
Rust workspace + Bun/Svelte/PlayCanvas front end. Every finding below was read in the source; the quote is
the offending line. Statuses: **applied** (fixed on the branch named), **proposed** (written up in
[Proposals](#proposals), left unapplied).

## Baseline

| What | Command | Where it comes from | State |
| --- | --- | --- | --- |
| Format | `cargo fmt --all --check` | `.github/workflows/ci.yml` → `tools/ci-local.sh fmt` | clean |
| Lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` | `tools/ci-local.sh clippy` | clean |
| Rust tests (core) | `cargo test -p sim-core --locked` | `tools/ci-local.sh sim` | pass |
| Rust tests (tools + art) | `cargo test -p ll-tools -p sound -p artkit -p ll-art -p art-nature -p art-wet -p art-rooms -p art-packs -p art-outside -p art-things --profile tools --locked` | `tools/ci-local.sh tools` | pass |
| Web install | `cd web && bun install --frozen-lockfile` | `tools/ci-local.sh web` | ok |
| Web tests | `cd web && bun run test` (= `bun test --parallel --timeout 30000 --path-ignore-patterns='**/*.slow.test.ts' tests`) | `web/package.json` `scripts.test` | pass (1,772 tests over 166 files, 0 failed) |
| Everything, as CI runs it | `tools/ci-local.sh` (`--quick` for the fast subset) | `tools/ci-local.sh` | not run end-to-end (see Open questions) |

Baseline run, taken before any change: `cargo test --workspace --no-fail-fast` → **exit 0**, 92 test
binaries, **1642 passed, 0 failed, 68 ignored**. There are no pre-existing failures.

Toolchain: `rust-toolchain.toml` pins `1.100.0` (clippy + rustfmt + `wasm32-unknown-unknown`); the pin is
deliberate — the file records that 1.97.0 miscompiles `review::size_row` (SIGSEGV in `ll-tools`
`tests/review.rs`). Cargo profiles matter when reproducing: `tools` is release-like without fat LTO,
`panic = "unwind"`, incremental; `release` is `panic = "abort"`.

Size: 454,260 lines of Rust over 1,190 files, plus `web/src` ≈ 177,863 lines. Largest sources are
`web/src/render/shared/characters/residents.ts` (5,095), `web/src/render/pc/PlayCanvasRenderer.ts`
(4,600), `crates/sim-core/src/cars/tests.rs` (1,862, tests), `crates/sim-core/src/content/load/mod.rs`
(1,234), `crates/sim-core/src/cars/route.rs` (1,093), `crates/artkit/src/raster.rs` (1,111),
`crates/tools/src/tex.rs` (1,031).

## Findings by area

### Bugs

| # | Sev | Location | What is wrong | Why it matters | Smallest fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| B1 | medium | `crates/sim-core/src/save/read.rs:599` | `_ => h.funds += s.paid,` — the fallback arm of `match object` refunds a setup whose `s.sim` is out of range, not just one whose object is gone from the content | A corrupt or hand-edited save mints money: a `setups[]` entry naming a sim that is not in town is refunded as if the object had been deleted | Refund only when `object` is `None`; drop the setup without a refund when the sim is gone | applied (`audit/bugs`) |
| B2 | medium | `crates/sim-core/src/world/tick/advance.rs:44` | `let ticks = TICKS_PER_STEP[self.speed as usize];` — a five-entry table indexed by an unclamped `pub speed: u8`; the same expression is at `:90` in `step_rate` | `save::read` validates the saved speed, but nothing stops another writer (wasm binding, test, future code) from setting 5+, which panics in debug and reads the wrong entry in release | `self.speed.min(SUPER_SPEED) as usize` at both sites | applied (`audit/bugs`) |
| B3 | medium | `crates/sim-core/src/life/work.rs:77` | `households[sim.household as usize].funds += shift_pay(content, &job, &sim.skills);` — plain add, while the promotion bonus 20 lines below (`:98`) is saturating, with the comment "a content pack's pay can be near `i64::MAX`, and a promotion must not panic" | The same overflow the code already guards against for the bonus is unguarded for the shift pay: debug panic, wrap in release | `saturating_add(shift_pay(...))` | applied (`audit/bugs`) |
| B4 | medium | `crates/tools/src/fp16.rs:84` | `if at + len > buf.len()` where `len` came from `varint(...)? as usize` | A length varint near `u64::MAX` overflows `at + len`: debug builds panic on the overflow, release builds wrap, pass the check, and then panic at `&buf[at..at + len]` (`:87`) with `start > end`. Any malformed `.onnx` reaches this | `at.checked_add(len).filter(|&end| end <= buf.len())` | applied (`audit/bugs`) |
| B5 | low | `crates/tools/src/fp16.rs:42` | `scale = scale.wrapping_mul(128);` | Protobuf caps a varint at 10 bytes; this loop wraps instead, so an over-long varint silently decodes to a small number instead of being rejected — the reader accepts what it should refuse | Count bytes and error past 10 | applied (`audit/bugs`) |
| B6 | low | `crates/sim-core/src/save/read.rs:580` | `world.items.list.dedup_by_key(|it| it.id);` | Two items sharing an id silently lose one; `next_id` is then taken past the surviving id, so nothing downstream can notice the loss | Detect the duplicate and return `Error::new("corrupt save: two items share an id")` | proposed (P1) |
| B7 | low | `crates/sim-core/src/travel/roads.rs:112` | `(0..self.pts.len() - 1)` | Underflows (panic) for a road with no points; with one point the range is empty and the trailing `.unwrap_or((0, (0.0, f32::INFINITY)))` silently answers "no projection" | Early-return `(0, (0.0, f32::INFINITY))` when `pts.len() < 2` | applied (`audit/bugs`) |
| B8 | low | `crates/sim-core/src/rooms/score.rs:70` | `let day = (f32::from(room.windows) * rules.window_tiles / tiles).min(1.0);` | Divides by `room.tiles` with no guard, while the size terms at `:64`/`:66` use `.max(1e-3)`. A zero-tile room gives `NaN` (0/0) and poisons the overall score. Same at `:71` (`a.lit / tiles`), `:74`, `:38`, `:194` (`dirt / room.tiles`) | `.max(1e-3)` on the divisor, as the neighbours already do | proposed (P2) |
| B9 | low | `crates/sim-core/src/cars/route.rs:324` | `if pts.len() < 2 \|\| self.lane.len() != pts.len() { return Some(LANE); }` | The doc above says "None: nobody comes the other way at either", and the sibling guards (`room_here` `:312`, `room_at` `:335`) return `0.0`; this one claims oncoming traffic. Defensible (it keeps a broken route's car in its lane) but it contradicts its own doc and hides a desync | Keep the value, say why, and `debug_assert!` the invariant | proposed (P3) |

### Performance

| # | Sev | Location | What is wrong | Why it matters | Smallest fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| P1 | low | `crates/sim-core/src/world/outings.rs:52` | `.filter(\|(_, r)\| r.kind == crate::travel::RoadKind::Trail).map(\|(i, r)\| (i, r.project(at)))` — every road is projected onto (linear in its segments) to find the nearest trail | Called per sim routing a trail walk (`crates/sim-core/src/world/route.rs:635`), so a town with many roads pays O(roads × segments) per walk | Index trailheads in `Travel` when the graph is built (P4) | applied (`audit/performance`, `5708bfa2d`) |
| P2 | low | `crates/sim-core/src/rooms/score.rs:115` | `let mut by_id = vec![usize::MAX; most + 1];` (and `:116`, `:127` `vec![Things::default(); rooms.len()]`) | Three allocations per rescore; the rescore runs on every object change (`crates/sim-core/src/world/rooms.rs:254`), which a busy lot does constantly | Keep the three as scratch in `RoomTiles` (P5) | applied (`audit/performance`, `0f9e26069`) |
| P3 | low | `crates/sim-core/src/world/navigation.rs:9` | `let centres: Vec<[f32; 2]> = self.objects.iter().map(\|o\| o.centre(&self.content)).collect();` | Recomputes every object's centre and reallocates on each `refresh_index` (every object-version change) | Cache the centres beside `objects_version` | applied (`audit/performance`, `dd91ddcf0`) |
| P4 | low | `crates/sim-core/src/cars/bodies.rs:12` | `pub fn hidden(cars: &[Car]) -> Vec<bool>` — two `Vec`s and a sort | Called on every snapshot write (`crates/sim-wasm/src/lib.rs:203`), i.e. per frame; the only per-frame allocation left in `snapshot::write` | Take an `&mut Vec<bool>` scratch, or skip the sort when nothing is out of sight (it already returns early, but still allocates) | applied (`audit/performance`, `a1fae0b4b`) |

The hot paths are otherwise deliberately allocation-free (`crates/sim-core/src/world/tick/step.rs` has no
`collect`; the tick's few `Vec`s, `advance.rs:175`/`:218`/`:743`, are on rare events and bounded by
household size). The remaining cost centres are algorithmic, not per-line.

### Maintainability

| # | Sev | Location | What is wrong | Why it matters | Smallest fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| M1 | medium | `crates/sim-core/src/life/work.rs:77` (idiom repeated 12+ times over 10 files) | `households[sim.household as usize]` is open-coded everywhere — `life/work.rs` ×4, `planner/goals.rs` ×3, `cooking/cook.rs` ×2, `cooking/eat.rs` ×2, `hosting.rs`, `carry.rs`, `life/visits.rs`, `phone/autonomy.rs`, `taxi/plan.rs`, `world/tick/step.rs` | `Sim::household` is a raw `u32` index into a `Vec`: every site re-does the cast, and a bad one panics mid-tick with no context | `World::household_of(&self, sim: usize) -> Option<&Household>` plus a `funds_of` for the two mutable cases, then use it | proposed (P6) |
| M2 | low | `crates/sim-core/src/life/schedule.rs:14` | `debug_assert!(` … "a sim's household is out of range" | The invariant "every sim's household index is valid" is checked only in debug builds; release builds carry on into `w.households[w.sims[i].household as usize]` (`:35`, `:37`) and panic there. The loader does validate (`crates/sim-core/src/world/residents.rs:49-54`), so it holds today — the assert is the only guard for new writers | Validate in `World::refresh_*` (or turn the accessor of M1 into the guard) | proposed (P6) |
| M3 | low | `crates/sim-core/src/snapshot/write.rs:464` | `/// Writes the snapshot into `out`, which must hold at least `CAPACITY` floats.` | The contract is a doc comment only; the first short slice panics with "range end index … out of range" from `:484`/`:504` instead of naming the buffer | `debug_assert!(out.len() >= CAPACITY)` and one helper for `HEADER_LEN + i * SIM_STRIDE` | applied (`audit/maintainability`) |
| M4 | low | `crates/sim-core/src/content/load/mod.rs:5` | one `impl Content` block, lines 5–1224 | Every content kind's validation lives in one function chain while `load/{access,helpers,objects,people,visits}.rs` already exist for the helpers: the file is the last place a kind is added by editing a 1,200-line block | Split along the existing banners (P7) | proposed |
| M5 | low | `crates/tools/src/tex.rs` | five concerns (CLI, numpy arithmetic, clouds, detail maps, materials) behind `// ---` banners | `tex/{pil,synth,leafcards,jpeg}.rs` already exist for exactly this reason; the file is 1,031 lines | Move the materials section out (applied on `audit/split`) | applied (`audit/split`) |

### Simplification

| # | Sev | Location | What is wrong | Why it matters | Smallest fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| S1 | low | `crates/sim-core/src/cars/route.rs:311` | `room_here`, `lane_at` (`:322`) and `room_at` (`:333`) each repeat the guard `if pts.len() < 2 \|\| self.room.len() != pts.len()` and `pts.partition_point(\|p\| p.2 <= t).clamp(1, pts.len() - 1)` | Three copies of the same four lines: the index arithmetic has to be read three times, and a fix to one (B9) is easy to miss in the others | One `fn span(&self, lens: usize, t: f32) -> Option<usize>`, each body one line | applied (`audit/simplification`) |
| S2 | low | `crates/sim-core/src/life/work.rs:67` | `take_breaks(sim, content, Some(&mut households[sim.household as usize].funds));` | The same index expression is written at `:67`, `:77` and `:98`; three chances to diverge, and it reads as an unrelated lookup each time | One `let h = sim.household as usize;` (or the M1 accessor) | applied (`audit/simplification`) |
| S3 | low | `crates/sim-core/src/travel/roads.rs:108` | `pub fn project(&self, p: [f32; 2]) -> (f32, f32) { self.project_on(p).1 }` | Two public names for one function, and the wrapper hides that `project_on` also answers the segment — which is what the trail lookup (P1) needs | Drop `project` and use `project_on(...).1`, or document the pair | proposed |

### Split large files

| # | Sev | Location | What is wrong | Why it matters | Smallest fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| SP1 | medium | `crates/tools/src/tex.rs:1031` | numpy arithmetic 65–122, `clouds.py` 123–222, detail maps 223–337, `materials.py` 338–902, tests 903–1031, all in one file | 564 lines of `materials.py` (JSON building) sit beside the image code they never call; the banner is already the module boundary | Move 338–902 to `crates/tools/src/tex/materials.rs`, re-export | applied (`audit/split`) |
| SP2 | medium | `crates/sim-core/src/content/load/mod.rs:1234` | one `impl Content` over every content kind | The largest non-test file in `sim-core` and the one every content change touches; the sibling modules prove the split works | Split by kind along the banners (P7) | proposed |
| SP3 | medium | `web/src/render/shared/characters/residents.ts:5095` | the whole character build in one module | The largest file in the repo; every character tweak loads it whole | Needs a plan (P8) | proposed |
| SP4 | low | `crates/sim-core/src/cars/route.rs:1093` | builder (`:69`–`:224`, `:348`–`:892`) and queries (`:225`–`:975`) in one file | The query half is a pure read API over the point arrays and moves without touching the builder | Move the query block to `cars/route/query.rs` (P9) | proposed |

## Test coverage gaps

- No test loads a save whose `setups[].sim` is out of range (B1).
- No test sets `World::speed` outside `0..=SUPER_SPEED` (B2).
- `crates/tools/src/fp16.rs` tests cover the nbits8 conversion; the protobuf reader's error paths (B4, B5) are untested.
- No test covers duplicate item ids in a save (B6).
- No test projects onto a road with fewer than two points (B7).
- No test scores a zero-tile room (B8) — `rooms` construction may make it impossible; see Open questions.
- `web/src/render/pc` has no headless GL coverage: `bun run test` exercises the DOM-facing parts, not `PlayCanvasRenderer.update`.

## Applied changes per branch

Filled in as each branch lands; see the git log of each branch for the exact commits.

| Branch | Commits | Findings |
| --- | --- | --- |
| `audit/report` | `d4d4db99f` | this report; no code change on this branch |
| `audit/bugs` | `e2156b99e` B2 · `842cdb0b0` B3 · `30cad03bb` B5 · `23dcc0595` B4 · `fd4e0a6b8` B7 · `8ed7cb2f2` B1 · `c97edcc18` B1's test | B1, B2, B3, B4, B5, B7 |
| `audit/performance` | `dd91ddcf0` P3 · `0f9e26069` P2 · `5708bfa2d` P1 · `a1fae0b4b` P4 | P1, P2, P3, P4 |
| `audit/maintainability` | `b8b46ec9f` | M3 |
| `audit/simplification` | `e07708f7c` S1 · `8cd3676bc` S2 | S1, S2 |
| `audit/split` | `ae27d079d` | SP1 (M5) |

Every branch was cut from `audit/report` and leaves the suite green; the status column above names the
branch a finding is applied on, so a finding marked `applied` is on that branch, not on this one. Each
branch was checked with the CI's own first two steps — `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets --locked -- -D warnings` — both clean; `audit/bugs` and
`audit/split` each carry one extra style-only commit for the formatting those checks caught.

### Measured: `audit/performance` before and after

`cargo run --release --example tickbench` (starter lot, 4 sims, one day, three runs each):
1.004–1.013 µs/tick on `audit/report`, 1.008–1.025 µs/tick on `audit/performance` — no change beyond
noise. `tickbench`'s lot has no roads, no trails and no cars, so it reaches none of P1–P4.

A throwaway example (a counting `GlobalAlloc`, not committed) over one day of the real save
`crates/sim-core/tests/saves/v035_normal.json` (57 sims, 1904 objects, 13 cars):

| | `audit/report` | `audit/performance` |
| --- | --- | --- |
| per tick | 18.14 µs | 18.07–18.11 µs |
| allocations over the day | 1,454,594 (13.47/tick) | 1,454,594 (13.47/tick) |
| `snapshot::write` (tools, tests) | 1.07 µs, 1 alloc/call | 1.04–1.69 µs, 1 alloc/call |
| `snapshot::write_with` (the wasm path) | — | 1.66 µs, 0 allocs/call |
| `cars::hidden` | 20 ns, 1.00 allocs/call | `hidden_into`, 10 ns, 0.00 allocs/call |

So P4 halves the call's time and allocates nothing per frame on the wasm path, and P1–P3 change
nothing measurable on these two workloads: neither routes a trail walk nor changes an object during
the measured day, so `trail_step`, `rescore_things` and `refresh_index`'s rebuild never run. Their
wins stand on the code (no allocation, and no O(roads) projection per walk) and are unmeasured; a
town with trails, and a lot with heavy object churn, would be needed to show them.

## Proposals

### P1 — what a save with two items of one id should do (B1, B6)

- Problem: the loader repairs corruption silently in some places (`setups` refunds, duplicate item ids) and refuses it in others (`expecting` parents, unknown household). Repair that moves money is indistinguishable from a bug.
- Options: (a) refuse — `Err` on a duplicate id and on a `setups[].sim` that is out of range; (b) repair without money movement; (c) keep as is.
- Trade-off: (a) turns a load that works today into a failed load for players with an already-corrupt file; (b) keeps them playing but still loses the item; (c) keeps a silent money printer.
- Effort: ~1 h. Risk: low for (b), medium for (a) — it needs a release note and a `save_version` bump.

### P2 — zero-tile rooms (B8)

- Problem: five divisions by `room.tiles` are unguarded while neighbouring terms guard theirs.
- Options: (a) `.max(1e-3)` on each divisor; (b) `debug_assert!(room.tiles >= 1.0)` in `score_things` and leave the arithmetic; (c) prove the invariant in `rooms::compute` and document it.
- Trade-off: (a) hides an invariant break behind a plausible score; (b) fails loudly in the tests that build rooms; (c) costs a read of the flood fill.
- Effort: 30 min. Risk: low. Recommended: (c) then (b).

### P3 — `lane_at`'s desync guard (B9)

- Problem: `Some(LANE)` on a broken route contradicts the doc's "None: nobody comes the other way".
- Options: (a) comment + `debug_assert!`; (b) return `None` (behaviour change for a desync only); (c) fold into S1's `span` and document there.
- Trade-off: (b) makes a broken route's cars drive down the middle; (a)/(c) keep them in lane. Effort: 15 min. Risk: low.

### P4 — a trailhead index (P1) — applied (`audit/performance`, `5708bfa2d`)

- Problem: `trail_step` projects onto every road to find the nearest trail, per walk.
- Options: (a) keep a `Vec<u32>` of trail roads in `Travel`, built with the graph, so only trails are projected; (b) a coarse grid of trails, as `ObjectIndex` does for objects.
- Trade-off: (a) is ~10 lines and removes the road-kind filter and most projections; (b) is the right answer if trails grow numerous. Effort: 1–2 h. Risk: low (the chosen trail must stay the same — assert it in a test comparing both).

### P5 — scratch for the rescore (P2, P3) — applied (`audit/performance`, `0f9e26069`, `dd91ddcf0`)

- Problem: `rescore_things` allocates three `Vec`s per call; `refresh_index` rebuilds `centres`.
- Options: (a) move them into `RoomTiles`/`ObjectIndex` as scratch fields; (b) a thread-local; (c) leave it.
- Trade-off: (a) adds state to a struct that is otherwise a pure description of the rooms — it needs a `resize` + `fill` discipline at each entry; measurable only under heavy object churn. Effort: 2 h. Risk: low. Do it with a benchmark (`cargo run -p sim-core --example tickbench`) before and after.

### P6 — a household accessor, then a `HouseholdId` (M1, M2)

- Problem: `sim.household as usize` indexing is open-coded at 12+ sites and panics mid-tick when wrong.
- Options: (a) `World::household_of(sim) -> Option<&Household>` + `funds_of`; (b) a `HouseholdId(u32)` newtype with a checked `get` on the `Vec`; (c) leave it.
- Trade-off: (a) is mechanical and keeps the panic for callers that unwrap; (b) removes the class of bug but touches the save format types. Effort: (a) 2 h, (b) a day. Risk: low for (a). Recommended: (a) now, (b) with the next save-format change.

### P7 — splitting `content/load/mod.rs` (SP2, M4)

- Problem: 1,220 lines of one `impl Content` covering every content kind.
- Options: (a) one `impl Content` block per file (`load/objects.rs`, `load/people.rs`, …) keeping the `Content` fields private to the crate; (b) a `ContentLoader` struct with a method per kind.
- Trade-off: (a) is a pure move and keeps the call order visible in `from_json`; (b) changes the shape and the diff. Effort: 3 h. Risk: low (compile-checked), but the file is under active development — coordinate.

### P8 — splitting `residents.ts` (SP3)

- Problem: 5,095 lines build a resident: looks, meshes, materials, animation, portraits.
- Options: (a) split by the existing function groups into `characters/residents/{looks,body,materials,motion}.ts`; (b) leave it and split only the pure helpers.
- Trade-off: (a) is a large mechanical diff in a hot file; the PlayCanvas `Entity` graph is threaded through most functions, so the seams are the ones that take `app`/`root` as arguments. Effort: 1 day. Risk: medium (no automated visual check). Recommended: (b) first.

### P9 — `cars/route.rs` queries (SP4, S1)

- Problem: 1,093 lines mixing the route builder with a read-only query API.
- Options: (a) `cars/route/query.rs` for `room_here`/`lane_at`/`room_at`/`pass_at`/`layer_at`/`kind_at`/`ahead_by`/`at`; (b) leave it.
- Trade-off: (a) is a move plus `use super::*`, and the query block is the part that keeps growing. Effort: 1 h. Risk: low.

## Open questions

- **Coverage of this audit.** `crates/artkit` (31,309), `crates/art/*` (~180k), `crates/sound`, `crates/voice` and most of `web/src` were sampled, not audited: `cars/route.rs`, `cars/bodies.rs`, `rooms/`, `travel/roads.rs`, `world/outings.rs`, `world/navigation.rs`, `snapshot/write.rs`, `tools/src/fp16.rs`, `tools/src/tex.rs`, `artkit/src/bvh.rs` and `render/pc/PlayCanvasRenderer.ts` were read in part. Every delegate dispatched to cover the rest was killed by the harness's five-minute idle watchdog (11 runs, `exit 143`, "timed out: no output for 5m"), so the procedural-art kernels, the sound engine and the Svelte UI deserve a second pass.
- **Can a room have zero tiles?** B8 assumes it is possible; `rooms::compute`'s flood fill may make it impossible. Not proven either way.
- **Can a road have fewer than two points?** B7 guards it; `Road::new` and the graph builder were not traced to the end.
- `tools/ci-local.sh` (the full CI) was not run end to end: it also builds the generated content, the clips and the wasm bundles, which takes far longer than the test suites this audit ran per change. Each branch was checked with `cargo test -p sim-core`, `cargo test -p ll-tools --profile tools` where relevant, plus `cargo fmt --check` and `cargo clippy -D warnings` for the touched crates.
