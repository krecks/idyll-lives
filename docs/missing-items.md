# Missing items

*Checked 2026-10-30 against `main` (58f4b1a1) + `feat/new-items`. Sources: content JSON, asset
manifests, `docs/objects/*.md`, [objects/open.md](objects/open.md), [authenticity-review.md](authenticity-review.md)
§7 and §9, and [PLAN.md](../PLAN.md) §8.*

## Verdict: no catalog item is missing

Every buyable object the docs describe exists in content and has a model:

| Check | Result |
|---|---|
| Object rows in `docs/objects/*.md` | 543 |
| Objects in `web/public/content/**/*.json` | 543 |
| Doc ids with no content entry | 0 |
| Content objects with no doc row | 0 |
| Content objects with no buy category | 0 |
| `model` keys referenced by content | 578 |
| Model keys missing from the manifests | 0 |
| Manifest entries whose file is missing | 0 |
| `item` ids in content | 43 |
| Unresolved `item` references | 0 |

**Doc bug (fixed 2026-10-30):** [objects/README.md](objects/README.md) said "541 of them", but its own
category table sums to **542** (72+26+27+20+50+14+5+19+23+39+21+43+53+36+94), and content had 542.
Cause: the prose was written on 2026-10-17, when there were 541; `curtainsWide` was added on
2026-10-21 and the table was updated, the prose was not. The count was then 542, and the wide curtains
were named as the one object not in the 2026-10-17 play check.

The empty vase, added 2026-10-30, brings both counts to **543** (tabletop 39 → 40); `item` ids to **43**
(the bunch of cut flowers).

So what is missing is not objects — it is **uses, clips, behaviours and polish** around them.

## 1. Modelled parts with no use or clip

| Object | Part | What is missing | Source | Effort |
|---|---|---|---|---|
| `nestingTables` Nesting tables | `small_table` | Small table never slides out; still decorative | decor decision 4 | S |
| `displayCabinet` Display cabinet | `door` | Glass door never swings; still decorative | decor decision 4 | S |
| `deskExecutive` Executive desk | drawer | Drawer modelled but static — **won't fix** | I-office-02, Q-19 | — |
| `grandfatherClock` Grandfather clock | trunk door | Not a part of its own; clock is wound at the dial — **won't fix** | A-decor-01 | — |

No modelled part is left unbuilt. The laundry basket has no lid by design (bathroom review, 2026-10-21):
clean washing is heaped above its rim, and the lidded one is the bedroom's `laundryHamper`. The 2026-10-30 check
that listed a basket lid here was reading a stale line in `open.md`, now corrected.

These are the only remaining "(+use)" parts. Kettle lid, bread-bin roll-top, laptop lid and the high
chair's tray were closed in 0.5.0 (I-tabletop-01, I-kids-01).

## 2. Model work still accepted as incomplete

| Object | What is left | Source | Effort |
|---|---|---|---|
| Hot tub | Cover folds back as one squashed part, not two halves (a fourth draw call) | A-outdoor-01 | S once the draw is allowed |
| Grandfather clock | Trunk door never opens (fifth draw call) | A-decor-01 | — (accepted) |

The rest of §9.2 of the authenticity review (chess table for two, low apples, food wrappers, arc-lamp
overhang, washing line piece by piece, high-chair tray, kettle/bread-bin/laptop lid clips, bakery-case
door) shipped in 0.5.0 — see open.md, "Fixed in 0.5.0".

## 3. Behaviours not in the game yet

| Item | What is missing | Source | Effort |
|---|---|---|---|
| Servers | No apron; they appear at their post at opening time instead of walking in | open.md, "Content and sim" | S–M |
| Flowers for a vase | No flower can be cut to put in a vase | I-garden-08, Q-18 polish | M |
| Furniture resizing | Resize arrows stay off (`RESIZING`, `buildmode.ts`); moving parts do not follow a resize yet | PLAN.md roadmap "later" | L |
| Smarter thoughts | No LLM re-rank of the utility AI's top-3; research only | PLAN.md roadmap "later", [design/smarter-thoughts.md](design/smarter-thoughts.md) | L (research) |

## 4. Not yet checked (not known broken)

| Item | Why it is open |
|---|---|
| Six held foods no test recipe makes: tasting board, pizza, pie, slices, sandwich, grill platter | Never checked in hand |
| A meal from the fridge eaten at a pulled-in chair | Eating was only checked with the café's cake |
| Sounds by ear (G-53) | All 34 were checked by spectrum, timing and level, never listened to |
| Low on WebGL2 orbits | 11–16 % over Babylon's GPU time (0.4–0.6 ms; budget 10 %), about noise level |
| `lampFloorArc` overhang | Fixed in 0.5.0; the lamp build check still only checks light height, not `at` |

**Done 2026-10-30:** `curtainsWide` checked in play on the stage's two-wide window: closed over both edges at
22:00, stacked at each end by day at 10:00, WebGPU and WebGL2. Its "In play" line is in
[objects/decor.md](objects/decor.md) D3.

## 5. Accepted / won't fix

| Item | Why |
|---|---|
| Feet slide at game speeds 4 and 5 | 45–90 m/s; no gait steps without strobing |
| Uses end once every need they raise is full | By design (I-systems-02) |
| Plates pile up with neither sink nor bin | Every generated home has a sink (I-systems-03) |
| Breakfast at a dining set from nowhere | Kept as a game shortcut (I-kitchen-10) |
| Executive desk drawer static | Q-19 |
| Large sofas, chaise and long coffee table fill their tiles | Q-08 |
| Cactus trio and succulent bowl are floor pots | Q-08 |
| Flat kerb | Q-13 |

## 6. Tooling not there yet

Full list: [authenticity-review.md](authenticity-review.md) §7. Highest-value gaps:

**Done 2026-10-30 (content):**

- Range-hood light: the hood comes on while the hob under it cooks (`light.while`,
  `["stir", "bake"]`), by day as at night, and goes off when the cooking is done. It has no switch of
  its own; the simulation decides (`lights.rs`). Any task counts, not only a plain use: a recipe's
  Cook step is `TaskKind::Cook`, so the light watches the step's animation at the station
  (`crates/sim-core/tests/hood.rs`). Evidence: `bench-results/check-hood/` — `hood-night-final`
  (idle 22:02: no light; cooking 22:34: the wall, the hob and the floor under the hood lit) and
  `hood-day-final` (cooking 10:40: the wall under the hood lit by day too).

- `cargo review sizes` measures the wrong face (basin drain/underside, closed toilet lid, kettle grate,
  pool cloth) and includes non-object geometry.
- `cargo check-glb` heights probe one point, so slats/planks give false warnings (the gate itself runs
  in the tool tests).
- `cargo review uses` does not flag the ladder shelf's missing use.
- `check.mjs play`: no `--use ripe` (a ripe use still needs `--use-existing`), no whole-recipe
  following, no `--views`; needs carry over between uses.
- `check.mjs sheet`, `poses.html`, gallery and previews each have known blind spots (see §7).
- CI runs clips, the GLB gate and, since 2026-10-30, `review sizes` (a content object no kind covers
  fails); still not `review uses` (it always exits 0) or the browser checks.

**Done 2026-10-30 (tooling):**

- `play --shot-idle`: a picture of each thing as bought, before any use moves it (`<object>-idle.png`).
- `play --top`: the camera above, looking down (who sits where at a table; the game camera stops at 1.35 rad).
- `play` picture names carry the `--with` helpers, so a helper run no longer overwrites the plain one
  (`chairWood-with-coffeeTable-sit-00.png`).
- Each `play` picture's `sim` records `distance`: how far the resident stands from the object's nearest
  face (a use from the back or the side is visible in the log).
- CI's **Object sizes** step runs `cargo review sizes` and fails when an object maps to no kind.
- `play --walk-shots <n>`: pictures on the way to a use, before it begins (`<object>-walk-00.png`), for
  the walk itself and for a door's swing, which is over by the time the resident is using anything.

**Done 2026-10-30 (doors by hand):**

- A house door's leaf is timed by the walker: the lead follows the game's speed
  (`doorLead(rate)` = 0.75 m, or 1.5 m/s × rate × 0.45 s), so at 1× it starts a hand's reach out and
  stands open as they arrive, and at 10× it is still open 0.6 m out. The old fixed 1.5 m lead left the
  leaf a quarter open at 3× (`web/src/render/shared/doors.ts`).
- The resident's right arm reaches for the handle as the leaf swings (`doorHand`, 0..1 over the last
  0.75 m, not while carrying a baby or holding something), and only for a door that actually swings:
  one standing open has no handle to reach.
- Tests: `web/tests/doors.test.ts` (7: the lead at each game speed, the swing finishing before the
  walker arrives, the hand out as the leaf starts and back once through, a shut leaf clear of anyone
  within 1.1 m, a wall along z). Pictures: `bench-results/check-doors/door-front-*.png` (shut, the hand
  on the leaf mid-swing, through, shut behind, and the same on WebGL2).

**Done 2026-10-30 (cut flowers for a vase):**

- The raised flower bed gains **"Cut some flowers"** (`cut`, 6 min, `anim: tend`, the bunch in the hand):
  `web/public/content/base.json`.
- **`vaseEmpty`** ("Empty vase", bought, price 15, tabletop / plants) takes the bunch: "Put the flowers in"
  (3 min, `brings: cutFlowers`, `leaves: open`) grows the vase's own bunch up out of the mouth over 0.9 s
  (`flowers_open`) and it stays up while the vase is open, saved that way; "Take the flowers out" (2 min) runs
  the same clip back along the use (`{'clip': 'flowers_open', 'over': -1}`), so the flowers go down as they are
  lifted out.
- The bunch is one new held item (`item.cutFlowers`, `hold: hand`, 0.37 m, 624 tris, one draw) built from the
  same six heads as the vase of flowers (`VASE_HEADS` / `BUNCH`), so a bunch cut in the garden and one bought
  arranged look alike (G-10); the empty vase shares the filled one's body (`VASE_BODY`).
- Tests: `crates/sim-core/tests/flowers.rs` (the bed gives a bunch in hand, the vase refuses `takeOut` while
  closed, carries the bunch in the arms on the way, opens, keeps it open through a save and load, closes when
  they are taken out). In play: `bench-results/check-flowers/` (`cut3-flowerbed-cut-00.png`,
  `vase2-vaseEmpty-{idle,putFlowers-01}.png`; `flowers3-*` the take-out run).
- Docs: `docs/objects/tabletop.md` (40 things, `vaseEmpty` in TT3), `docs/review/tabletop.md`,
  `docs/objects/garden.md` and `docs/review/garden.md` (`flowerbed`'s `cut`), `docs/authenticity-review.md`
  (Q-18, G-10, G-68, §9.1 I-garden-08).

## Suggested next items

Nothing from the review ledgers is left unbuilt: the remaining work is the not-yet-checked list in §4, the
tooling in §6, and the per-bed cutting that Q-18 left open (the tulips', daffodils', lilies', cosmos' and
daisies' "cut a bunch", which the raised bed's one use stands in for).
