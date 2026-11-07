# Idyll Lives 0.7.0 — Release Plan

- **Branch:** `release/0.7.0`, cut from `main` at `70280ee9` on 2026-11-03 (0.6.0 released, and #52's three sim fixes); to be merged into `main` by pull request; tag `v0.7.0` on the merge
- **Base:** v0.6.0 at `f4482e7f`
- **Status:** **in release**: everything below is merged into `release/0.7.0` (at `f5c9a1d9` when these notes were drafted). The bus is dropped (the owner's call, 2026-11-05): `fix/car-guard` takes it out, stops and all, and a shared work taxi takes its place; the cars keep what they learnt with it (taking turns where their ways cross or meet, pulling over on 6 m lanes). 0.6.1 (PR #53) was folded into 0.7.0 and closed.
- **Release type:** features: getting about town (bikes and e-scooters, lifts, a shared work taxi), hosting, purchases set up at home, career pacing, visits; save format still 35

## 1. What ships

Player notes: the `## 0.7.0` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.7.0-notes.md](release-0.7.0-notes.md).

| Merge | What |
|---|---|
| on the branch (`5741effb`, `01f1e26c`, `c3567369`) | The feed and the Journal keep to the household's own news (0.6.1's fix); a picked door's arrows go with the door; visitors let in when someone awake is home, and visits that fit the way, the hour and the time free |
| `feat/career-pacing` (`948606c8`) | Promotions after a working week (`life::shift_performance`, `careerRules.performanceSlowdown`, `lateGraceMinutes`, `lateShare`); the Career tab's "about N more workdays" |
| `feat/new-item-setup` (`301b73ca`) | A wished-for purchase comes as a parcel the resident carries in and sets up (`setup.rs`, `objectRules.setUp`, the `parcel` item; saves' `setups`) |
| `feat/bikes` (`282bc621`, with `feat/ride-core`) | Bikes and e-scooters ridden to work and school (`ride.rs`, content `vehicle`), the three models (`families/yard.py`), the riders' clips, new towns' rides (`game/townRides.ts`), Furnish's *Bikes & scooters* |
| `feat/lifts` (`c0587654`) | Lifts in a friend's car next door (`cars::lift_seat`) |
| `feat/hosting` (`0a80db02`) | A host's offer of a bite (`hosting.rs`, `visits.offer`), eating beside one's company; also the basement's fence posts (`7e9907f0`), Firefox's model loading (`6ada1144`) and households nobody plays buying with next week's bills left (`8682a8ef`) |
| `feat/town-bus` (`a178eeab`) | A timetabled bus through the towns' stops: **dropped** on `fix/car-guard` (`bus.rs`, content `bus`, the stops, `render/pc/buses.ts` all out; a town file's or a save's stops are no ways out, dropped as they load); a shared work taxi replaces it |
| #54 `feat/modern-console-tv` (`8fe2b8f3`) | The Console TV redrawn (later replaced by the portable tube set, 657aedf6) |
| #55 `fix/food-cooking` (`b457700f`) | Food and cooking reviewed and fixed: serving, clearing, scraping, eating, the cook's tools, the cookers' and held food's models |
| `simplify/sim-core`, `simplify/bus-hosting`, `simplify/web` (`d97f6d05`, `9da73623`, `6a44a4a8`) | `/simplify` passes over the branch |
| `fix/riders-and-vehicles` (through `f5c9a1d9`, with `fix/commute-timing` and `fix/model-gate`) | The code review's fixes: bodies kept clear of cars, a car never driving through anyone, the bike or walk decided once, lateness by the way's due time, a guest's promised bite; the gate's strict checks as errors (`checkglb::STRICT`, `accept`); the armoire, party booth, shower stall, seedling and yoga mat; the monitor's radius |

`main` brought along #52 (a car's facing within a half turn, meeting halfway over the stairs, an adopted child beside a parent).

- **Saves:** format 35, unchanged. 0.7.0's additions are optional fields (`setups`, `job.late`, the `served` story with `refs.item`), left out when empty; riding and a host's offer aren't saved (`save.rs`'s header); a save written while the bus was in (its stops among the exits) loads without them. 0.6.0 opens 0.7.0's saves (no `deny_unknown_fields`; the version check is `> 35`), without the bikes and e-scooters (unknown objects are dropped) and without a parcel still waiting to be set up (its money is gone with it). A monitor on a turned corner desk that no longer fits its 0.27 m radius is left out on load.
- **Known:** see section 5.

## 2. Version

`cargo release-version 0.7.0` (by the owner); CHANGELOG `## 0.7.0 — <date>` (written as "unreleased": set the date by hand); README 0.7.0 and the saves (done on `docs/0.7.0`); PLAN.md's status line points here.

## 3. Release gates (local)

`tools/ci-local.sh` (every CI step, on CI's Bun, and the web's types and lint), on `docs/0.7.0` (the release branch at `f5c9a1d9` plus these docs):

| Gate | Result |
|---|---|
| `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (148 + 713, 22 ignored) |
| generated houses and worlds (`--ignored`, Cosy/Normal three per kind, Big one per kind) | pass (3 + 2) |
| `bun run check:clips` | 321 clips, 0 new, 0 known |
| `bun run check:chunks` | 195 anchors and names found in PlayCanvas 2.23.3 |
| `bun run test` (each file isolated) | pass (293 in 51 files) |
| `cargo test -p ll-tools --release --locked` | pass (114) |
| `cargo review sizes`, `cargo review uses --check`, `cargo check-glb` | pass (651 models, 0 failed) |
| `bun run check`, `bun run lint` | 0 errors, 0 warnings; clean |

Run 2026-11-03/04 on CI's Bun 1.4.3: all green.

Still to run before the release PR, as for 0.6.0: the voice's tests (`cargo test -p voice --release --locked`, `bun run check:voice`), tickbench, the production build and its size, the generated-world tests ten times or more on fresh worlds, and the game played: a new Cosy, Normal and Big game (riders on the road, a bike's spot empty while it's out, a work taxi calling and colleagues getting in, a lift, the household-only feed and Journal), a 0.6.0 save loading, no console errors, and the production benchmark on WebGPU and WebGL2.

## 4. Release steps

1. Merge `fix/car-guard` (the bus out, the work taxi in, the cars' turns), then `cargo release-version 0.7.0` and the CHANGELOG's date; commit on the release branch.
2. Pull request `release/0.7.0` → `main`; CI passes on it; merge (a merge commit).
3. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
4. Tag and push (after asking the owner): `git tag -a v0.7.0 -m "Idyll Lives 0.7.0: getting about town"`, `git push origin v0.7.0`; watch the tag's run.
5. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors, the large files' sizes.
6. Take the owner's section out of the notes, then `gh release create v0.7.0 --verify-tag --title "Idyll Lives 0.7.0" --notes-file plan/release-0.7.0-notes.md`.

## 5. Open points (for the owner)

- **Money after faster promotions:** the balance test's richest-household cap went from $30k to $50k (measured $37–43k) and median weekly savings from $624 to about $1.6–2k; the knobs (grade pay in `careers.json`, the living costs) are unchanged so far.
- **The 30-minute late grace** (`careerRules.lateGraceMinutes`): set while about half of the arrivals at work were late; lower it once the bikes and the work taxi make commutes punctual.
- **Only 4 idea toasts show at once** (`ui/planner/ideaToasts.ts` `MAX_OPEN`): a purchase request can be pushed off screen (it is still answerable in the Goals panel). A purchase is set up even with free will off, since accepting it was the player's choice (`setup::who`).
- **Nothing to drink to serve:** a host's offer brings only food (an `eat` use a guest could have themselves).
- **Firefox:** the model-load fix (`6ada1144`) is not verified in Firefox.
- **Parked-car tiles are coarse.**
- `vehicle.roadsOnly` is read but not kept to: an e-scooter goes wherever the walk goes (`docs/content-packs.md`, `vehicle`).
- The ignored long soaks `a_big_town_with_a_busy_player_*` already fail on `main` (a stuck resident, nobody born); not caused by 0.7.0.
