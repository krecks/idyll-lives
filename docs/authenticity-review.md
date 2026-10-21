# Authenticity review: models, animations and interactions

The review of every model, animation and interaction in the game against the real thing, planned in [plan/authenticity-review.md](../plan/authenticity-review.md). It started on 2026-10-21 from `main` at e725166 (v0.3.0), on the branch `review/authenticity`. Its record is one ledger per category in [docs/review/](review/README.md); this document consolidates them for the owner's triage.

**Status (2026-10-21):** Phases 0–3 are done: every ledger row has its verdicts and every finding its fields. The findings are grouped by the fix that would close them (§4) and wait for triage (Phase 4, plan §7).
- §1 sums up what was covered and found, §2 says how.
- §3 lists every S1 and S2 finding by category, with its fix group; S3 findings are counted, with a pointer to the ledger.
- §4 lists the 72 fix groups and a proposed order.
- §5 lists the decisions the owner is asked for, each with a recommendation; §6 the decisions already made.
- §7 lists the tooling problems the waves met.
- §8–§10 are for Phase 4: fixed, found but not fixed, and the manual test checklist.

Severity (plan §4):
- **S1, breaks belief:** wrong at a glance in normal play (a resident floats above the bed, a use from the back of the object, an adult using a child's or a baby's thing, or a child an adult's, as if it fitted).
- **S2, noticeably wrong:** a careful player sees it (hands hover, a counter 10 cm too low, a missing obvious use).
- **S3, polish:** only on close inspection or a side-by-side.

A `decision` finding takes the severity of its worse outcome. Line numbers refer to the branch at the time of writing (e725166 plus the review's tooling commits); the S1 and S2 findings' lines were checked against it.

## 1. Summary

**Scope.** 948 ledger rows, every one with an M, A and I verdict (or `n/a`):
- 541 buyable objects in 15 object ledgers (kitchen 72, bathroom 26, bedroom 27, living 50, office 14, kids 20, hobbies 19, fitness 5, tabletop 39, decor 22, lighting 21, outdoor 43, garden 53, community and farm 36, personality packs 94);
- 45 held items and tools, 161 resident rows (bodies, stages, hair, clothes, clips, `ACTIONS` entries, procedural layers), 83 house parts, 42 town things, 30 social rows and 46 system rows (cooking, baby care, lamps, doors, open states, crops, visitors).

**Evidence.** 8,509 shots, of them 605 contact sheets (one per object), and 67 play logs (`play.jsonl`: pose, clip and motion events per use), 5.6 GB in the review worktree's gitignored `bench-results/review/`. Each wave's steps, commands and problems are in its `bench-results/review/<cat>/review-log.md`.

**Findings.** 300 filed: 8 S1, 111 S2, 181 S3. At the consolidation:
- 36 findings whose cause is shared moved to [systems.md](review/systems.md) (24) and [residents.md](review/residents.md) (12), keeping their ids, with a pointer left where they were.
- 15 duplicates were closed into the older finding, which took their items.
- 264 findings are in 72 fix groups; 24 are closed (the 15 duplicates and 9 others), 2 were confirmed fixed in 0.3.0, and 10 are not checked (each with its reason, §3.2).
- By status: 208 open, 56 confirmed (most of them widened), 5 seeded and 7 not checked, 24 closed.

The 264 in groups, by severity and fix class:

| Fix class | S1 | S2 | S3 | Total |
|---|---|---|---|---|
| content | 1 | 12 | 25 | 38 |
| renderer-table |  | 15 | 13 | 28 |
| motion |  | 4 | 10 | 14 |
| clip | 1 | 13 | 9 | 23 |
| model | 1 | 26 | 41 | 68 |
| code | 5 | 27 | 30 | 62 |
| decision |  | 4 | 27 | 31 |
| **Total** | **8** | **101** | **155** | **264** |

**Main themes**
- **Who may use what.** Nothing stops a child pouring wine, using a stove or starting a fight with a younger child, or an adult lying in the cot or riding the toddlers' spring horse. A use ordered by the player skips even the rules free will keeps (baby care, homework). Three of the eight S1s are age rules (G-45), which owner decision 3 put off as new behaviour (Q-01).
- **Where residents stand and sit.** Two residents at one pool table stand inside each other (S1); diners and riders sit side by side; residents walk through a table to a seat behind it, and get into bed through the footboard; the upper bunk is sat on in mid-air (S1); people sit on the fire pit's rim and in the hearth's firebox (two S1s).
- **One clip for every object of a kind.** An `ACTIONS` entry plays the same clip at every object: the kneel at waist-high beds and at the cot, the hands at the chest a step short of a basin, standing at the foot of the slide (S1), the hand-weaving mime for darts, pool and ping-pong. The reach rarely meets the part used.
- **Cooking.** Every oven step stirs a frying pan; the hob only lights when the meal was started at it; the cook holds the ingredients and the bowl through every step.
- **Functional heights.** About twenty models are out of the real range or out of step with the renderer's seat tables: the workbench, the digital piano, the vessel basins, the kettle grill, the picnic table, the chef's island stools, the tubs.
- **Beds.** One blue checked quilt on every bed, a stray one at the world origin, pyjamas for every nap (shoes on), and in at the foot end.
- **Known gaps, confirmed and widened:** no props in hand, no body language, no privacy, doors by nearness. Filed as new behaviour (G-70 to G-72).
- **The personality packs** carry the most S2s (19): things used standing beside the surface they are for, and things that should glow staying dark at night.

## 2. How the review was done

- **Plan and rubric:** [plan/authenticity-review.md](../plan/authenticity-review.md) §3 (rubric lines M1–M18, A1–A13, I1–I16), §4 (ids, severity, fix class, the shared-cause rule). The ledger format and the bookkeeping rules: [docs/review/README.md](review/README.md).
- **Real-world ranges:** [docs/review/reference.md](review/reference.md), with sources, and its machine-readable twin `reference.json`.
- **Tools (Phase 0):** `cargo review sizes` (T3: each object's measured size and functional heights against `reference.json`) and `cargo review uses` (T4: every use's pose, clip, minutes, effects, motions and sound, with flags), both in `crates/tools/src/review.rs`; the heights checks in `cargo check-glb` (T1, T2: mesh, content and `characters/furniture.json` agree); `pnpm check:clips` (T5, `tools/characters/check_clips.ts`, every clip at every stage, in CI); `tools/bench/check.mjs` `gallery`, `clips`, `play` and `sheet`; `poses.html`; debug states such as `review-stage`, `review-stage-night`, `review-garden`, `review-baby`, `review-social`.
- **Pilot:** the 28 `base.json` objects and the residents, reviewed by one agent and then by the owner, who approved the changes in [pilot-notes.md](review/pilot-notes.md) (§6 below).
- **Waves:** 13 agents filed the other ledgers (rooms, activities, outside, packs, then the cross-cutting wave for socials, systems and the residents' pending rows), each in its own worktree on one core, headless at 1 % volume, reading only. A coordinator merged each wave.
- **Consolidation (this document):** shared causes moved to `systems.md` and `residents.md`, duplicates merged, the waves' notes on other ledgers carried into the findings, wrong line numbers corrected, and the findings grouped by fix.

## 3. Findings

Every S1 and S2 finding by ledger, with its fix group (§4); S3 findings are counted, with a link to the ledger. A finding moved at the consolidation is listed under the ledger it now lives in; its id still names where it was first seen. The ledgers hold the evidence, the file and line to change and the other items each finding reaches.

| Ledger | S1 | S2 | S3 | In groups | Closed | Not checked |
|---|---|---|---|---|---|---|
| [Kitchen](review/kitchen.md) |  | 4 | 12 | 16 | 2 |  |
| [Bathroom](review/bathroom.md) |  | 4 | 6 | 10 | 1 |  |
| [Bedroom](review/bedroom.md) | 1 |  | 10 | 11 |  |  |
| [Living room](review/living.md) |  | 2 | 7 | 9 | 5 | 1 |
| [Office](review/office.md) |  | 4 | 3 | 7 |  |  |
| [Kids](review/kids.md) |  | 6 | 5 | 11 | 1 |  |
| [Hobbies](review/hobbies.md) |  | 9 | 11 | 20 | 1 |  |
| [Fitness](review/fitness.md) |  | 1 | 5 | 6 |  |  |
| [Tabletop](review/tabletop.md) |  | 2 | 3 | 5 | 2 |  |
| [Decor](review/decor.md) | 1 | 1 | 5 | 7 |  |  |
| [Lighting](review/lighting.md) |  |  | 6 | 6 |  |  |
| [Outdoor](review/outdoor.md) | 1 | 7 | 6 | 14 | 2 |  |
| [Garden plants and trees](review/garden.md) |  | 3 | 10 | 13 |  | 1 |
| [Community and farm](review/community.md) | 1 | 5 | 10 | 16 | 1 |  |
| [Personality packs](review/packs.md) |  | 19 | 16 | 35 | 3 |  |
| [Held items and tools](review/held.md) |  | 2 | 1 | 3 | 2 |  |
| [Houses](review/houses.md) |  |  | 6 | 6 |  | 1 |
| [Town](review/town.md) |  | 1 | 4 | 5 | 1 | 2 |
| [Residents](review/residents.md) |  | 13 | 10 | 23 | 2 | 3 |
| [Social interactions](review/social.md) |  | 5 | 4 | 9 |  | 1 |
| [Systems](review/systems.md) | 4 | 13 | 15 | 32 | 3 | 1 |

### 3.1 S1 and S2 by category

#### Kitchen

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-kitchen-01 | S2 | motion | The dishwasher stays shut while it is loaded | G-16 |
| I-kitchen-07 | S2 | content | Breakfast at the kitchen island is eaten standing at its cupboards | G-03 |
| M-kitchen-01 | S2 | model | The base dining table is a 38 cm deep plank with both diners side by side | G-39 |
| M-kitchen-02 | S2 | model | Both dishwashers are a metre wide | G-32 |

S3: 12 in [kitchen.md](review/kitchen.md).

#### Bathroom

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-bathroom-01 | S2 | model | The shower's glass door never opens: the resident walks through it | G-33 |
| I-bathroom-05 | S2 | renderer-table | At the basins the washer stands a step back, the hands short of the tap | G-15 |
| I-bathroom-06 | S2 | content | Teeth are brushed at the wall cabinet only, with no basin, no water and nothing in hand | G-09 |
| M-bathroom-02 | S2 | model | The vessel basins' rims stand at 1.00–1.02 m, 9–11 cm over a basin's height | G-29 |

S3: 6 in [bathroom.md](review/bathroom.md).

#### Bedroom

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-bedroom-02 | S1 | code | The upper bunk's sleeper sits down in mid-air: no climb up the ladder | G-44 |

S3: 10 in [bedroom.md](review/bedroom.md).

#### Living room

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-living-03 | S2 | content | Reading at the bookshelf, and TV without a seat, are an hour on foot | G-03 |
| I-living-04 | S2 | decision | The sofa bed can't be slept in for a night | G-61 |

S3: 7 in [living.md](review/living.md).

#### Office

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-office-01 | S2 | content | A desk chair works at the standing desk: the top is 0.57 m over the seat | G-07 |
| I-office-03 | S2 | content | The office chair can only "Sit down": it can't be worked at a desk | G-09 |
| I-office-04 | S2 | renderer-table | Planning on the whiteboard is done from 1.4 m away, the hands in the air | G-15 |
| M-office-01 | S2 | model | The corner desk is a square slab with one corner cut, not an L | G-32 |

S3: 3 in [office.md](review/office.md).

#### Kids

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-kids-01 | S2 | renderer-table | A kids' chair at a coffee table: the hands hover | G-04 |
| I-kids-01 | S2 | code | The high chair's tray has no use: the sim never seats a baby in it | G-66 |
| I-kids-04 | S2 | renderer-table | Feeding the baby is the gardening kneel at the cot rail | G-14 |
| I-kids-05 | S2 | code | Changing the baby happens over the cot rail; the changing table is never used | G-66 |
| I-kids-06 | S2 | renderer-table | Toys are played with standing over them, the hands out in the air | G-14 |
| I-kids-07 | S2 | renderer-table | The ball pit and the play tent are used from outside | G-22 |

S3: 5 in [kids.md](review/kids.md).

#### Hobbies

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-hobbies-03 | S2 | motion | The pool rack floats 25 cm over the balls for the whole game | G-18 |
| I-hobbies-02 | S2 | clip | The stargazer stands upright beside the telescope, half a metre above the eyepiece | G-21 |
| I-hobbies-03 | S2 | code | Stargazing at noon, indoors | G-64 |
| I-hobbies-04 | S2 | clip | Darts, pool and foosball are played with the same hand-weaving clip | G-26 |
| I-hobbies-05 | S2 | code | Pool is played with the cue left on the cloth and the balls never broken | G-70 |
| I-hobbies-08 | S2 | clip | The guitar is held out in front of the chest, its neck pointing forward and up | G-21 |
| I-hobbies-10 | S2 | model | The potter's knees go into the wheel's body | G-31 |
| M-hobbies-01 | S2 | model | The workbench is 7 cm low and built like a steel office desk | G-29 |
| M-hobbies-04 | S2 | model | The digital piano's keys are 10 cm higher than the piano's, and half as wide again | G-29 |

S3: 11 in [hobbies.md](review/hobbies.md).

#### Fitness

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-fitness-01 | S2 | clip | The weight bench's lie: the legs stretch past the bench in mid-air | G-24 |

S3: 5 in [fitness.md](review/fitness.md).

#### Tabletop

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-tabletop-01 | S2 | code | No thing on a top can be used, though 20 of them are plainly for using | G-63 |
| I-tabletop-05 | S2 | content | The monitor fits on no counter and on almost no desk | G-07 |

S3: 3 in [tabletop.md](review/tabletop.md).

#### Decor

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-decor-01 | S1 | model | Warming up by the fire, the resident sits in the firebox's mouth with her back in the flames | G-28 |
| I-decor-02 | S2 | renderer-table | The grandfather clock is wound at arm's length, hands at the chest, the dial 0.6 m higher | G-15 |

S3: 5 in [decor.md](review/decor.md).

#### Lighting

S3: 6 in [lighting.md](review/lighting.md).

#### Outdoor

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-outdoor-01 | S1 | content | Warming up by the fire pit is sitting on its rim, back to the flames | G-01 |
| A-outdoor-01 | S2 | model | The hot tub's open cover stands upright, floating above and behind the tub | G-33 |
| I-outdoor-05 | S2 | content | The greenhouse is tended and picked from outside its door | G-02 |
| M-outdoor-03 | S2 | model | The picnic table is 7 cm low at the top and 2 cm low on the benches | G-30 |
| M-outdoor-04 | S2 | model | The charcoal grill's bowl stands 20 cm high: the grate at 0.97 m | G-29 |
| M-outdoor-05 | S2 | model | The hammock is 1.86 m long: residents lie in it folded, as in a sling chair | G-37 |
| M-outdoor-06 | S2 | decision | The rain barrel's downpipe ends in mid-air, wherever the barrel stands | G-36 |
| M-outdoor-07 | S2 | model | The garden boulders read as eggs, not rock | G-38 |

S3: 6 in [outdoor.md](review/outdoor.md).

#### Garden plants and trees

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-garden-05 | S2 | clip | Picking an apple or a lemon is eating at the trunk; the fruit shrinks away 1.2 m over the picker's head | G-21 |
| M-garden-01 | S2 | model | Trees whose foliage comes down to head height over a one-tile footprint: residents walk into them and use them from inside | G-38 |
| M-garden-02 | S2 | model | The weeping willow's curtains are thick solid spikes that stop 1–2.6 m above the lawn | G-38 |

S3: 10 in [garden.md](review/garden.md).

#### Community and farm

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-community-01 | S1 | clip | "Go down the slide" is standing at the foot of the chute | G-19 |
| I-community-03 | S2 | code | Two-player things played alone: the seesaw tips with one rider, ping-pong and park chess have no partner | G-65 |
| I-community-05 | S2 | renderer-table | "Shoot some hoops" is jogging on the spot under the rim | G-04 |
| I-community-06 | S2 | clip | Ping-pong is the hands-at-the-waist mime, the bats left on the table | G-26 |
| I-community-09 | S2 | code | Nobody serves: carts, the stall, the counters and the café sell themselves | G-67 |
| M-community-01 | S2 | model | The ping-pong table is about two thirds of a real one | G-32 |

S3: 10 in [community.md](review/community.md).

#### Personality packs

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-packs-01 | S2 | renderer-table | Bathers' shins go through the front of the heart tub | G-06 |
| A-packs-03 | S2 | motion | The washer, the baking oven and the glasshouse door stay open for the whole use | G-17 |
| A-packs-04 | S2 | model | The nap pod's hood closes onto the sitter's head | G-31 |
| A-packs-05 | S2 | renderer-table | Dozing on the window seat: face down, across the cushion, the head at the glass | G-04 |
| A-packs-06 | S2 | content | The recliners tip their backs and raise the footrests; the sitter stays upright, feet on the floor | G-02 |
| I-packs-01 | S2 | model | At the quilt table and the candlelit table the legs come out through the front | G-31 |
| I-packs-02 | S2 | content | Exercise and singing next to the board, the mat and the stage, not on them | G-02 |
| I-packs-03 | S2 | clip | Bouldering on the floor beside the crash mat | G-26 |
| I-packs-04 | S2 | model | On the vibration plate the resident turns their back to the handlebar | G-31 |
| I-packs-07 | S2 | content | The midnight snack is eaten sitting on the nightstand | G-03 |
| I-packs-08 | S2 | content | Primping at the vanity on foot; its stool is never sat on | G-02 |
| I-packs-09 | S2 | content | The glasshouse's orchids are tended from outside the door | G-02 |
| M-packs-01 | S2 | model | The entry console's mirror and hook board hang in the air when it stands free | G-36 |
| M-packs-02 | S2 | model | The spritz station's round mirror hangs in the air when it stands free | G-36 |
| M-packs-04 | S2 | model | The chef's island's stools are chair height under a counter | G-29 |
| M-packs-05 | S2 | model | The organizer desk and the love-letter bureau turn their backs to the room | G-41 |
| M-packs-06 | S2 | decision | The hammocks' 1.88 m frames: grown-ups lie with the knees up | G-37 |
| M-packs-07 | S2 | model | Things that glow in life stay dark at night; the mood lamps light nothing | G-35 |
| M-packs-11 | S2 | model | The candlelit table for two seats both diners side by side | G-39 |

S3: 16 in [packs.md](review/packs.md).

#### Held items and tools

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-held-01 | S2 | code | No book, cup or cutlery prop: residents mime them | G-70 |
| M-held-01 | S2 | model | Five items borrow a model that shows something else | G-34 |

S3: 1 in [held.md](review/held.md).

#### Houses

S3: 6 in [houses.md](review/houses.md).

#### Town

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| M-town-05 | S2 | decision | The street is 3.3 m between the kerbs, with a centre line | G-58 |

S3: 4 in [town.md](review/town.md).

#### Residents

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-bedroom-03 | S2 | code | Residents get into bed at its foot end, through the footboard, and slide up to the pillow | G-51 |
| A-bedroom-04 | S2 | code | A stray quilt lies at the world origin whenever a bed with a blanket is on the lot | G-46 |
| A-residents-01 | S2 | code | Getting up from a chair glides back to the tile | G-51 |
| A-residents-07 | S2 | code | Body language doesn't follow feelings; only the face does | G-71 |
| A-residents-09 | S2 | clip | Seated feet don't reach the floor for children, teens and elder women | G-20 |
| A-residents-11 | S2 | clip | An elder woman, a teen or a child on the sofa loses the shins | G-20 |
| I-bedroom-01 | S2 | code | Pyjamas for every nap, a doze in the garden too, and the shoes stay on | G-46 |
| I-garden-02 | S2 | clip | "Relax in the shade" is half an hour standing by the trunk | G-22 |
| I-garden-04 | S2 | renderer-table | Harvesting and tending don't follow the plant's height: the hand stops half a metre above low crops, and the tomatoes are tied up kneeling at the soil | G-14 |
| I-kitchen-03 | S2 | renderer-table | Hands at the cabinet doors, not on the tap, the appliance or the shelf | G-15 |
| I-living-06 | S2 | clip | Sitting on the floor is the chair sit: shins under the floor on the floor cushion, legs into the fire at the electric fireplace | G-22 |
| I-outdoor-03 | S2 | renderer-table | On the swing set the hands reach forward as at a desk, not up on the chains | G-23 |
| M-bedroom-04 | S2 | code | Every sleeper gets the same blue checked quilt, whatever the bed's own bedding | G-46 |

S3: 10 in [residents.md](review/residents.md).

#### Social interactions

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| A-social-01 | S2 | code | A kiss between residents of different heights meets the forehead | G-27 |
| A-social-03 | S2 | clip | Fist fights are aimed at the attacker's own height: a teen punches over a child's head | G-27 |
| I-social-01 | S2 | code | Phone actions are mimed: there is no phone in the hand | G-70 |
| I-social-02 | S2 | code | Success and failure look the same: a refused hug or kiss is two people standing still, and a bad joke is laughed at | G-27 |
| I-social-04 | S2 | clip | Every phone action is a call: texting and scrolling hold the phone to the ear | G-27 |

S3: 4 in [social.md](review/social.md).

#### Systems

| Id | Sev. | Fix class | Finding | Group |
|---|---|---|---|---|
| I-community-04 | S1 | code | Adults and elders ride the spring horse, the seesaw and the merry-go-round and climb the fort | G-45 |
| I-hobbies-07 | S1 | code | Two residents at a two-player table stand in one spot, inside each other | G-43 |
| I-kids-03 | S1 | code | A direct use skips the rules free will keeps: an adult lies in the crib, a child cares for the baby, an adult does homework | G-45 |
| I-kitchen-06 | S1 | code | Children pour wine and mix cocktails | G-45 |
| A-hobbies-01 | S2 | code | Residents walk through the table, the wheel or the drum kit to a seat behind it | G-51 |
| A-houses-01 | S2 | code | House doors open by nearness, not by hand | G-48 |
| A-kitchen-02 | S2 | motion | The hob, the oven and the microwave don't work when the meal was started somewhere else | G-16 |
| A-town-02 | S2 | code | Gates never open: residents walk through the shut leaf | G-48 |
| I-garden-03 | S2 | code | A crop is picked the moment the harvest starts: the bed empties, then the resident picks at nothing for 18 minutes | G-50 |
| I-kitchen-04 | S2 | code | Cooks chop with the crate in both hands and stir holding the bowl | G-49 |
| I-kitchen-05 | S2 | renderer-table | Every Cook step is stirring a frying pan, at a microwave, an oven tower or for a bake | G-16 |
| I-living-05 | S2 | code | Watching TV takes the nearest seat even when it faces away from the screen | G-47 |
| I-outdoor-07 | S2 | code | Sets for two seat their pair side by side: the bistro set takes one, the picnic table two on one bench | G-62 |
| I-systems-13 | S2 | code | Sitting on a thing moved off its tile's middle sits beside it (fix: slots, fixed) | G-62 |
| I-systems-01 | S2 | renderer-table | Cooking props are drawn at a station's middle: on the gas grill station, on the grate | G-08 |
| I-systems-04 | S2 | code | Nothing stops a child using a stove | G-45 |
| I-systems-10 | S2 | code | A cook waiting for a busy station stands where they are, and the next one stands inside them | G-43 |
| M-outdoor-01 | S2 | code | Night glow draws through other parts: the grill's coals through its shut lid | G-52 |

S3: 15 in [systems.md](review/systems.md).

### 3.2 Not checked

These have no group until someone checks them (§10 says how).

| Id | Sev. | Finding | Why not checked |
|---|---|---|---|
| A-residents-04 | S3 | The start and end of a walk jump at speeds 4–5 | CHANGELOG 0.3.0 says walks no longer jump at the start and end; needs a headed, timed run at speeds 4–5 |
| A-residents-05 | S3 | After a hidden tab, someone who sat down meanwhile replays the sit-down | a hidden tab; headless pages are never hidden |
| A-residents-06 | S3 | Walkers jog on the spot for about 0.25 s after a hidden tab | a hidden tab; headless pages are never hidden |
| A-social-02 | S3 | Other children gather round a hug, right next to the pair | not reproduced; a hug isn't joinable, so the cause is likely free will walking to the pair for something else |
| A-systems-01 | S3 | Moving parts don't follow a resize, so furniture can't be resized | furniture resizing is off in 0.3.0 |
| A-town-01 | S3 | The household creator's preview sways its plants with reduce motion on | needs the creator's preview running with reduce motion on; the code points the same way |
| I-garden-01 | S3 | Residents once didn't reach things in a crowded garden | seen once in play; the evidence places one object at a time |
| M-houses-01 | S3 | Wall things shown while one is held float over lowered walls | needs Build mode with a wall thing held |
| M-living-02 | S3 | The staircase loses part of its top end in some basement views | no basement view in the evidence |
| M-town-03 | S3 | A far road's lane markings change brightness between loads | needs the same view over several page loads compared pixel by pixel |

Fixed in 0.3.0 and confirmed by the review: M-living-01 (the sofa bed's mattress height), A-living-01 (the bean bag's snoozer).

## 4. Fix groups

A group is the fix that closes its findings: one `furniture.json` row family, one `ACTIONS` entry, one clip, one family script, one sim rule, one content rule. Bands follow plan §7.3: **tables** (content and renderer tables), **motions** (object clips and manifest `motions`), **resident clips**, **Blender** (family rebuilds) and **code**.

Size: **S**, content or table edits under an hour; **M**, a clip, a family rebuild or a contained sim or renderer change; **L**, new behaviour or several systems. "New behaviour" groups are code the game doesn't have yet; those owner decision 3 names (props in hand, body language, turning on the spot, doors by hand, age rules, privacy) are filed only and planned later. Questions (Q-nn) are in §5.

In all: 72 groups.
- Tables 15 (12 S, 3 M), motions 3 (1 S, 2 M), resident clips 9 (7 M, 2 L), Blender 15 (9 M, 6 L), code 30 (2 S, 13 M, 15 L).
- Of the code groups, 8 are new behaviour planned as their own items and 4 are new behaviour filed only (decision 3: G-45, G-70, G-71, G-72).
- Six groups hold the eight S1s: G-01, G-19, G-28, G-43, G-44, G-45.

| Group | Band | Title | Findings | Sev. | Fix class | Size | Depends on |
|---|---|---|---|---|---|---|---|
| G-01 | tables | Fire pit: warm up from a seat round it, facing the fire, not on its rim. | I-outdoor-01 | S1 | content | S | With `seatNear`, better after G-47 |
| G-02 | tables | Place and pose fields: inside, stand, seat, slot offsets. | I-outdoor-05, I-packs-09, I-packs-02, I-packs-08, A-packs-06, I-packs-12 | S2 | content | S | — |
| G-03 | tables | Long sit-down uses take a seat near (`seatNear`). | I-living-03, I-kitchen-07, I-outdoor-06, I-packs-07, I-packs-14 | S2 | content | S | G-47 (the seat faces the thing); what a reader does with no seat near: Q-14 |
| G-04 | tables | An existing clip or table row for single objects. | I-community-05, I-packs-10, I-residents-02, I-bedroom-07, I-bedroom-02, A-packs-05, A-kids-01 | S2 | renderer-table | S | — |
| G-05 | tables | Seat, bed and light heights that disagree with the mesh. | M-living-05, M-hobbies-09, M-outdoor-09, M-packs-13, M-kids-04, M-lighting-07 | S3 | renderer-table, content | S | — |
| G-06 | tables | Bathers sit on the tub floor with the legs inside the tub. | A-bathroom-04, A-packs-01 | S2 | renderer-table | S | — |
| G-07 | tables | Which seats and things go with which tops. | I-kitchen-14, A-office-01, I-office-01, I-tabletop-05 | S2 | content, renderer-table | S | — |
| G-08 | tables | Cooking props on the burner the clip lights and on the prep board. | I-systems-01 | S2 | renderer-table | S | — |
| G-09 | tables | Uses the siblings have, and one duration outlier. | I-kitchen-02, I-kitchen-08, I-bathroom-06, I-bathroom-08, I-bedroom-03, I-bedroom-05, I-bedroom-06, I-living-09, I-office-03, I-office-05, I-outdoor-08, I-packs-16, I-packs-18 | S2 | content | M | — |
| G-10 | tables | Garden uses: look up at tall flowers, admire the scentless ones. | I-garden-07, I-garden-08 | S3 | content | S | Cut flowers for a vase: Q-18 |
| G-11 | tables | Names, kinds and descriptions that don't match the model. | M-living-06, M-kids-03, M-community-05, M-bedroom-01, M-kitchen-04, M-lighting-04, M-packs-09, M-kitchen-03, M-living-03, M-garden-08, M-bathroom-01 | S3 | content (decision) | S | Q-08 (rename or rebuild); a rebuild moves the item to band D |
| G-12 | tables | Decor or a use (decision 4): the cooking wall, the coat stand, the farm, the desk drawer. | I-kitchen-09, I-decor-04, I-community-12, I-office-02 | S3 | decision | S | Q-19 |
| G-13 | tables | Uses end when the needs are full (as designed). | I-systems-02 | S3 | decision | S | Q-22 |
| G-14 | tables | Garden, care and play clips chosen by the thing's height. | I-garden-04, I-garden-06, I-kids-04, I-kids-06 | S2 | renderer-table | M | — |
| G-15 | tables | Reach the part used: stand closer, reach up or down. | I-kitchen-03, I-bathroom-05, I-hobbies-11, I-decor-02, I-residents-03, I-community-07, I-office-04 | S2 | renderer-table | M | A reach-up pose may need a clip (band C) |
| G-16 | motions | Cooking stations work for every recipe, and baking is baking. | I-kitchen-05, A-kitchen-02, A-kitchen-01 | S2 | motion, renderer-table, content | M | G-08 |
| G-17 | motions | Moving parts in time with the use: hold, delay, per slot. | A-packs-03, A-bathroom-03, A-hobbies-02, A-kitchen-03 | S2 | motion | S | — |
| G-18 | motions | Loops and parts that should move, and the rack that floats. | A-hobbies-03, A-packs-02, A-fitness-02, A-decor-01, A-outdoor-03, I-living-08, I-packs-15, A-bedroom-01 | S2 | motion | M | The fountain's sound is G-53 |
| G-19 | resident clips | The slide: up the ladder and down the chute. | I-community-01 | S1 | clip, renderer-table | M | Who may ride it: G-45 |
| G-20 | resident clips | Seated legs at every life stage: feet on the floor, shins out of the cushion. | A-residents-09, A-residents-11 | S2 | clip | M | — |
| G-21 | resident clips | Standing uses that need their own clip. | I-hobbies-02, I-garden-05, I-bathroom-02, I-hobbies-08 | S2 | clip | M | — |
| G-22 | resident clips | Sitting on the floor and getting into small places. | I-living-06, I-garden-02, I-kids-07 | S2 | clip, renderer-table | M | — |
| G-23 | resident clips | Rides: astride, the hands on the grips. | I-outdoor-03, I-community-13 | S2 | clip, renderer-table | M | — |
| G-24 | resident clips | Weight bench: feet on the floor, a lower pad. | A-fitness-01, M-fitness-01 | S2 | clip, model | M | — |
| G-25 | resident clips | Walking, jogging and climbing clips. | A-residents-02, A-residents-03, A-residents-10, A-community-01 | S3 | clip | M | — |
| G-26 | resident clips | Game and sport clips: throw, cue, rods, bat, climb, yoga. | I-hobbies-04, I-community-06, I-packs-13, I-packs-03, I-fitness-01 | S2 | clip | L | Bats and cue in hand: G-70; the yoga mat's length: G-40 |
| G-27 | resident clips | Social clips: partner heights, outcomes, gestures, phones. | A-social-01, A-social-03, I-social-02, I-social-03, I-social-04 | S2 | clip, code | L | Who may fight a child: G-45 |
| G-28 | Blender | The hearth: sit in front of the fire, not in it. | I-decor-01 | S1 | model, content | M | — |
| G-29 | Blender | Working heights inside the real range. | M-hobbies-01, M-hobbies-04, M-bathroom-02, M-outdoor-04, M-packs-04, M-hobbies-06, M-packs-08, I-garden-09 | S2 | model | L | — |
| G-30 | Blender | Seat and table heights outdoors and at play. | M-outdoor-03, M-outdoor-02, M-kitchen-05, M-community-03 | S2 | model, content, renderer-table | M | — |
| G-31 | Blender | Room for the body at the object: knees, heads, legs, feet. | I-hobbies-10, I-packs-01, A-packs-04, I-packs-17, I-packs-04, I-fitness-02 | S2 | model | L | — |
| G-32 | Blender | Sizes that break the use: the dishwashers, the ping-pong table, the corner desk. | M-kitchen-02, M-community-01, M-office-01 | S2 | model | M | — |
| G-33 | Blender | Doors, covers and curtains as moving parts. | A-bathroom-01, A-bathroom-02, A-outdoor-01 | S2 | model, motion | M | Q-05 (the shower door's draw call) |
| G-34 | Blender | Held items show the thing eaten and the dish emptied. | M-held-01, M-held-02 | S2 | model, content | M | — |
| G-35 | Blender | Glow at night: coloured shades, screens, candles, mood lamps. | M-packs-07, M-lighting-03, M-tabletop-01 | S2 | model, content | M | — |
| G-36 | Blender | Wall-hung parts on floor objects. | M-packs-01, M-packs-02, M-outdoor-06 | S2 | model (decision) | M | Q-06; or the wall rule of G-54 |
| G-37 | Blender | Hammocks long enough to lie in. | M-outdoor-05, M-packs-06 | S2 | model, content (decision) | M | Q-07 |
| G-38 | Blender | Trees, plants and stones read as themselves. | M-garden-01, M-garden-02, M-garden-03, M-garden-04, M-garden-05, M-garden-06, M-garden-07, M-outdoor-07 | S2 | model | L | — |
| G-39 | Blender | Dining and tea for two face across the table. | M-kitchen-01, M-packs-11, I-packs-11 | S2 | model, content | L | G-62 (a place and facing per slot); Q-04 |
| G-40 | Blender | Overall size and proportion polish. | M-bedroom-02, M-bedroom-03, M-hobbies-02, M-hobbies-03, M-hobbies-05, M-kids-01, M-kids-02, M-fitness-02, M-tabletop-02, M-packs-12, M-outdoor-08, M-community-02, M-community-04, M-living-04, M-lighting-06 | S3 | model | L | The arc lamp over the next tile needs the sim to allow it |
| G-41 | Blender | Parts wrong, missing or facing away. | M-packs-05, M-hobbies-07, M-hobbies-08, M-packs-03, M-packs-10, M-kitchen-06, M-decor-01, M-decor-02 | S2 | model | L | — |
| G-42 | Blender | Seated skirts and jackets drape. | M-residents-02 | S3 | model | M | — |
| G-43 | code | Two bodies in one place: standing users of shared things. | I-hobbies-07, I-community-08, I-systems-10 | S1 | code | M | Better with `play --sims` (tooling) |
| G-44 | code | Climb the bunk bed's ladder. | A-bedroom-02 | S1 | code, clip | M | Q-02; who may take the upper bunk: G-45 |
| G-45 | code | Age rules: who may use what, also on a direct use. **Fix now (owner, 2026-10-21).** | I-kids-03, I-kitchen-06, I-community-04, I-systems-04, I-kids-08 | S1 | code, content | L | Q-01: it needs the age-rule behaviour the owner put off (decision 3) |
| G-46 | code | Blankets, pyjamas and bath clothes. | M-bedroom-04, A-bedroom-04, I-bedroom-01, I-bathroom-03 | S2 | code (one decision) | M | Q-10 (bath clothes) |
| G-47 | code | The seat for watching faces the screen. | I-living-05 | S2 | code | S | — |
| G-48 | code | Doors and gates open for the one going through. | A-houses-01, A-houses-02, A-town-02 | S2 | code | M | Q-03; doors by hand stay new behaviour (decision 3) |
| G-49 | code | Cooking: hands free to cook, the dish to the table. | I-kitchen-04, I-held-04 | S2 | code | M | G-16 |
| G-50 | code | State changes when the hands do it: harvests and the washing line. | I-garden-03, A-outdoor-02 | S2 | code, motion | M | — |
| G-51 | code | Getting onto seats and into bed from the right side. | A-bedroom-03, A-hobbies-01, A-residents-01 | S2 | code | L | — |
| G-52 | code | Glow drawn through the parts in front of it. | M-outdoor-01 | S2 | code | M | — |
| G-53 | code | Sounds by the thing, not by the clip's tag. | I-bathroom-04 | S3 | code | M | — |
| G-54 | code | Where things may stand: walls behind, ceilings above, heads under. | M-bathroom-03, M-lighting-01, M-lighting-05 | S3 | code, content | M | Q-12 (pendants: shorter, or over tables only) |
| G-55 | code | Standing distances: the oche and the front door. | I-hobbies-06, I-systems-09 | S3 | code | M | — |
| G-56 | code | Talking seated, and joining a group on foot. | I-social-06, A-social-04 | S3 | code | M | — |
| G-57 | code | Houses built in code: door panels and wood, wallpapers, chimneys, railings. | M-houses-02, M-houses-03, M-houses-04, M-houses-05, M-houses-07 | S3 | code, content | M | — |
| G-58 | code | Street scale: the road, the kerb, the cars, the garages. | M-town-05, M-town-07, M-town-06, M-houses-06 | S2 | code (decision) | L | Q-13 (widen the street, or a one-lane mews) |
| G-59 | code | Town leftovers: far ground models, the unused hedge. | M-town-01, M-town-02 | S3 | code (decision) | S | Q-09 |
| G-60 | code | Lamps: where their light goes, when they are on, the fan and the fire. | M-lighting-02, M-community-06, I-lighting-01, A-lighting-01, I-decor-03 | S3 | code (one decision) | L | Q-11 (a lamp bought on or off) |
| G-61 | code | A night on the sofa bed. | I-living-04 | S2 | decision | M | Q-15 |
| G-62 | code | A place and a facing per slot. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-outdoor-07 | S2 | code | L | Then G-39 |
| G-63 | code | Uses on things that stand on tops. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-tabletop-01, I-tabletop-04 | S2 | code | L | Q-20 |
| G-64 | code | Conditions: a time, a place, something to work on. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-hobbies-03, I-kitchen-11, I-bathroom-07 | S2 | code | L | — |
| G-65 | code | Two-player things need a partner. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-community-03, I-hobbies-01 | S2 | code (decision) | L | G-43, G-62; Q-16 |
| G-66 | code | Babies at the high chair and the changing table. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-kids-01, I-kids-05 | S2 | code | L | — |
| G-67 | code | Food: who sells it, where it comes from, where it is eaten. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-community-09, I-community-10, I-kitchen-10, I-systems-03 | S2 | code (decisions) | L | Q-17 |
| G-68 | code | Crops: sowing, tending that helps, harvests to the kitchen. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-systems-08, I-garden-10 | S3 | code (decisions) | L | Q-18 |
| G-69 | code | Pairings: feet up on a footstool, changing outfit. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* | I-living-07, I-bedroom-04 | S3 | code | L | — |
| G-70 | code | Props in hand: book, cup, cutlery, phone, tools, the shaker, the cue. **New behaviour — filed only, planned later.** | I-held-01, I-social-01, I-kitchen-13, I-hobbies-05, I-community-11 | S2 | code | L | The coin toss half of I-community-11 is a table change |
| G-71 | code | Body language and turning on the spot. **New behaviour — filed only, planned later.** | A-residents-07, A-residents-08 | S2 | code | L | — |
| G-72 | code | Privacy: a private flag, a shut door, a place for a baby. **New behaviour — filed only, planned later.** | I-systems-05, I-social-05 | S3 | code (decision) | L | G-48; Q-21 |

### 4.1 Proposed order

Plan §7.3: content and renderer tables first (cheap and broad), then object clips and motions, resident clips, Blender rebuilds, and code last, each new behaviour as its own planned item. Within each band the S1 groups come first, then the groups that close the most S2s for the least work. S1 is fixed now unless the owner says otherwise (§7.2); the code S1s (5a) touch other files than the bands before them and can start at once, beside band 1.

Every fix follows plan §7.4: a rebuild passes `cargo art <family>`, `cargo check-glb` (T1) and `cargo review sizes` (T3) inside the budgets in `docs/design/objects.md`; a change to `minutes` or `effects` re-runs the balance test and the soak test; a changed clip passes `pnpm check:clips` at every stage; each fix updates its ledger row and `docs/objects/<cat>.md`.

**1. Content and renderer tables**

1. **G-01** (S1, S) Fire pit: warm up from a seat round it, facing the fire, not on its rim. Files: `web/public/content/furniture.json` (`firePit` `seat`, `relax`). Verify: `check.mjs play --state review-stage-night --objects firePit --use relax`, every stage (`--sim`); the age sheet.
2. **G-02** (S2, S) Place and pose fields: inside, stand, seat, slot offsets. Files: `catalog2/outdoor.json` (`greenhouse` `inside`), packs `natureLover`, `couchPotato`, `cheerful`, `outgoing`, `romantic`, `slob`, `loner`, `foodie`, `energetic`; `catalog2/living.json` (`recliner` `seat.recline`). Verify: `check.mjs play` each use; the door clips still read with someone inside; `poses.html` for the recliners.
3. **G-03** (S2, S) Long sit-down uses take a seat near (`seatNear`). Files: `base.json` (`bookshelf`), `furniture.json` (bookcases, coffee tables, `stereo`, `boombox`, `kitchenIsland`), `garden.json`, `catalog2/outdoor.json`, packs `bookworm`, `gloomy`, `romantic`, `lazy`, `neat`, `slob`. Verify: `check.mjs play --objects <def> --with sofa@…` for each; `cargo review uses`.
4. **G-04** (S2, S) An existing clip or table row for single objects. Files: `characters.ts` (`ACTIONS.exercise`, `phone`, `sing`, `wash`, `BLANKET_BEDS`, `TUCK`), `characters/furniture.json` (`beds.gloomy.windowSeat`), `tableTuck.ts`. Verify: `check.mjs play` each; `poses.html` for the window seat and the daybed; the kids' chair at the coffee table.
5. **G-05** (S3, S) Seat, bed and light heights that disagree with the mesh. Files: `web/src/render/babylon/characters/furniture.json`; `catalog2/living.json`, `catalog2/dining.json`, `furniture.json` (`tableLamp` `light.height`). Verify: `cargo check-glb` heights warnings (T1, T2) gone; `cargo review sizes`; `poses.html?seats=…` at every stage.
6. **G-06** (S2, S) Bathers sit on the tub floor with the legs inside the tub. Files: `characters/furniture.json` (`seats`, `beds`: the three baths, `gloomy.soakingTub`, `romantic.heartTub`, `hotHeaded.coldPlunge`). Verify: `poses.html?seats=bathtub…` and `check.mjs play … --use bath` at every stage; T1 heights.
7. **G-07** (S2, S) Which seats and things go with which tops. Files: `furniture.json` (`barStool`), `catalog2/dining.json` (stools), `catalog2/office.json` (`deskStanding` kind), `catalog2/tabletop.json` (`monitor` radius), `web/src/render/babylon/tableTuck.ts`. Verify: `check.mjs play --objects chairTask@11,12,3 --with deskStanding@10,12,0 --use write`; stools at a counter and a bar; a chair facing each dining set; the monitor bought on a counter and each desk.
8. **G-08** (S2, S) Cooking props on the burner the clip lights and on the prep board. Files: `characters.ts:546` (`STATION_PROPS`), `:2097` (`writeTool`): a hob spot per model. Verify: `check.mjs play` cook at `stoveGas`, `stoveElectric`, `cooktopCounter`, `rangeCooker`, `grillStation`; the held wave's recipe runs.
9. **G-09** (S2, M) Uses the siblings have, and one duration outlier. Files: `base.json`, `furniture.json`, `catalog2/{kitchen,bath,bedroom,living,office,outdoor}.json`, packs `cheerful`, `outgoing`, `neat`. Verify: `cargo review uses` (the "no use" flags go); `cargo test --release --test balance -- --ignored --nocapture` and the soak test (new effects, one `minutes`); play each new use.
10. **G-10** (S3, S) Garden uses: look up at tall flowers, admire the scentless ones. Files: `web/public/content/garden.json`. Verify: `check.mjs play --state review-garden` admire and smell; `cargo review uses`.
11. **G-11** (S3, S) Names, kinds and descriptions that don't match the model. Files: content names and descriptions; `docs/review/reference.json` kinds; or the family scripts if rebuilt. Verify: `cargo review sizes` (a renamed thing moves to its kind's range); `docs/objects/<cat>.md`.
12. **G-12** (S3, S) Decor or a use (decision 4): the cooking wall, the coat stand, the farm, the desk drawer. Files: content (`reference.json` kinds marked `decor`, or new uses). Verify: `cargo review uses`.
13. **G-13** (S3, S) Uses end when the needs are full (as designed). Files: `crates/sim-core/src/world.rs`. Verify: —.
14. **G-14** (S2, M) Garden, care and play clips chosen by the thing's height. Files: `web/src/render/babylon/characters.ts` (`ACTIONS.tend`, `harvest`, `smell`, `play`, by def as `look` already is). Verify: `check.mjs play` tend, harvest and smell at every crop and flower, the greenhouse, compost bin and herb garden; feed at the crib (the cuddle hold); the toy box, toy chest and train table; adult and child.
15. **G-15** (S2, M) Reach the part used: stand closer, reach up or down. Files: `characters.ts` (`STAND_BACK`, `STEP_IN`, `ACTIONS.cook`, `wash`, `fetch`, `tinker`, `garden`, `admire`, `read` by def). Verify: side shots of each use, adult and child: sinks and basins, microwave, toaster, blender, wall cupboards, chemistry bench, the clock, feeder, giving jar, statue, notice board, timetable, whiteboard.

**2. Object clips and motions**

1. **G-16** (S2, M) Cooking stations work for every recipe, and baking is baking. Files: `base.json` recipes (a `bake` and a `mix` tag), `characters.ts` (`ACTIONS`, `STATION_PROPS`), `kitchen_cooking.py` motions keyed by tag (and the packs' ovens), `objectMotion.ts:413`, `snapshot.rs:140`, `:156`. Verify: the kitchen wave's recipe scenes (`meal-fridge-stoveGas`, `-microwave`, `-wallOven`, a dishwasher-only lot); the held wave's recipe runs; `check.mjs clips`.
2. **G-17** (S2, S) Moving parts in time with the use: hold, delay, per slot. Files: family motions in `pack_neat.py`, `pack_kind.py`, `pack_natureLover.py`, `laundry.py`, `hobbies.py`, the double sink in `kitchen_counters.py`; then `cargo art`. Verify: `check.mjs play` `play.jsonl` clip events against the resident's clip time; `check.mjs clips`.
3. **G-18** (S2, M) Loops and parts that should move, and the rack that floats. Files: `hobbies.py`, `pack_energetic.py`, `fitness.py`, `decor.py`, `garden_decor.py`, `media.py`, `pack_neat.py`, `bedroom.py`, `kids.py`. Verify: `check.mjs clips --each` at t=0.5 and t=99; `cargo check-glb` draw budgets (the clock's pendulum is one more part).

**3. Resident clips**

1. **G-19** (S1, M) The slide: up the ladder and down the chute. Files: `characters.ts:193-199` (`STEP_IN`), `characters/clips/street.ts` (a round like the climbing frame's), `community.json:361`. Verify: `check.mjs play` the slide on `town-things`, child and adult; `pnpm check:clips` for the new round at every stage.
2. **G-20** (S2, M) Seated legs at every life stage: feet on the floor, shins out of the cushion. Files: `characters.ts` (seat fit per stage), `characters/clips/seated.ts`, seat depths in `characters/furniture.json`. Verify: `pnpm check:clips --body all --stage all` (the baseline shrinks); `poses.html?stage=all` on the sofas, armchairs, bean bag, kids' chair.
3. **G-21** (S2, M) Standing uses that need their own clip. Files: `characters.ts` (`ACTIONS.look` at the telescope, `eat`/a pick at the fruit trees, `shower`, `GUITAR`), `characters/clips/`. Verify: `check.mjs play` telescope, apple and lemon, all seven showers, the three guitars, from the side and the game camera; `pnpm check:clips`.
4. **G-22** (S2, M) Sitting on the floor and getting into small places. Files: `characters/clips/seated.ts` (a cross-legged or ground sit), `characters.ts` (`STEP_IN` for the ball pit and the tent), `furniture.json` (`fireplaceElectric` seat back), `base.json`, `garden.json` (`shade` pose). Verify: `poses.html` floor cushion and electric fireplace at every stage; the trees' shade; ball pit and tent with the child; `pnpm check:clips`.
5. **G-23** (S2, M) Rides: astride, the hands on the grips. Files: `characters.ts:477` (`ACTIONS.play` sitting for rides), a `legs: astride` seat, `characters/clips/seated.ts`. Verify: `poses.html` rides (`extra/poses/rides-child.png`); play the swing set, spring rider, seesaw, rocking horse.
6. **G-24** (S2, M) Weight bench: feet on the floor, a lower pad. Files: `characters/clips/lie.ts` (a feet-down bench lie), `characters/furniture.json:55`, `fitness.py:111`. Verify: `check.mjs play --objects weightBench --use lift` at every stage; `cargo check-glb`, T1.
7. **G-25** (S3, M) Walking, jogging and climbing clips. Files: `characters/clips/locomotion.ts:208`, `characters.ts:136` (`GAIT_MAX_RATE`), `characters/clips/street.ts:127-157` (climb lanes). Verify: `pnpm check:clips` (the baseline shrinks); headed timing at speeds 3–5 (R1-05's foot-slide measure); two climbers (needs `play --sims`).
8. **G-26** (S2, L) Game and sport clips: throw, cue, rods, bat, climb, yoga. Files: `characters.ts` (`ACTIONS.play`, `exercise` by def), new clips in `characters/clips/`. Verify: `check.mjs play` darts, pool, foosball, ping-pong, the smash bin, the bouldering wall, the yoga mat; `pnpm check:clips` at every stage.
9. **G-27** (S2, L) Social clips: partner heights, outcomes, gestures, phones. Files: `characters.ts` (`EMBRACE`, `Anim.Kiss`, `Anim.Fight`, `choose`, the social layers), `base.json` (`socials`, `phone` anim per action), `characters/clips/idle.ts`. Verify: `review-social` and `review-social-fail` pair runs; `social/sheets/*`; `pnpm check:clips`.

**4. Blender rebuilds**

1. **G-28** (S1, M) The hearth: sit in front of the fire, not in it. Files: `decor.py:697` (`hearth`), `:889`; `furniture.json:4094` (`fireplace` `seat`). Verify: `cargo art decor`, `cargo check-glb`; `check.mjs play` the fireplace's relax straight on; `poses.html?seats=fireplace`.
2. **G-29** (S2, L) Working heights inside the real range. Files: `hobbies.py` (workbench, digital piano, pool rails), `bathroom.py` (vessel basins), `outdoor.py` (kettle grill), `pack_foodie.py` (chef's island stools), `pack_gloomy.py`, `pack_hotHeaded.py` (desks), `plants.py` (bonsai table). Verify: `cargo review sizes` (T3), `cargo check-glb` heights (T1); play each use; the family `.md` notes.
3. **G-30** (S2, M) Seat and table heights outdoors and at play. Files: `outdoor.py` (picnic table, park bench), `dining.py:676` (carver), `playground.py` (spring rider, merry-go-round), with the content `seat`/`surface` and `SEATS` rows. Verify: `cargo review sizes`, T1/T2 three-way; `poses.html` at every stage.
4. **G-31** (S2, L) Room for the body at the object: knees, heads, legs, feet. Files: `hobbies.py:1365` (pottery wheel), `pack_lazy.py` (quilt table, nap pod, vibe plate), `pack_romantic.py:120`, `pack_loner.py:344` (sauna), `fitness.py:250` (rack mat). Verify: side and game-camera shots of each use at every stage.
5. **G-32** (S2, M) Sizes that break the use: the dishwashers, the ping-pong table, the corner desk. Files: `kitchen_cooking.py:423-476`, `playground.py:438-441` (and a 3 × 2 footprint in `community.json`), `office.py:448-466`. Verify: `cargo review sizes`; `check.mjs clips` (the dishwasher door); play at each.
6. **G-33** (S2, M) Doors, covers and curtains as moving parts. Files: `bathroom.py:554`, `:609-610`, `:999`; `outdoor.py:590-597`, `:568-569`. Verify: `check.mjs clips`; play the shower, the stall and the hot tub; draw calls (the shower door was left still for its 4th draw: Q-02).
7. **G-34** (S2, M) Held items show the thing eaten and the dish emptied. Files: `held.py` (cone, toast, glass, noodle box, carton, dirty boards and pie dish); `base.json` `items` and `emptyAs`. Verify: `held/sheets/obj-item.*`; the recipe and snack runs.
8. **G-35** (S2, M) Glow at night: coloured shades, screens, candles, mood lamps. Files: `lighting.py:26`, `tabletop.py:98`, the packs' glow materials; `light` on the three mood lamps. Verify: `review-stage-night` play shots; `cargo review uses` (a switchable lamp counts as a use).
9. **G-36** (S2, M) Wall-hung parts on floor objects. Files: `pack_neat.py:384`, `pack_kind.py:184-191`, `pack_slob.py:232`, `yard.py:776-782`. Verify: gallery and play free-standing and against a wall.
10. **G-37** (S2, M) Hammocks long enough to lie in. Files: `outdoor.py:481-499`, `pack_cheerful.py:424-434`, `pack_natureLover.py:354-359`; footprints in `furniture.json`, `packs/cheerful.json`, `packs/natureLover.json`; `characters/furniture.json` beds. Verify: `poses.html` hammocks at every stage; `cargo review sizes`.
11. **G-38** (S2, L) Trees, plants and stones read as themselves. Files: `trees.py`, `plants.py`, `garden_decor.py`. Verify: `cargo art` the three families; gallery at the game camera; the pilot's tree-walk scene; `cargo review sizes`.
12. **G-39** (S2, L) Dining and tea for two face across the table. Files: `dining.py:259` (base, glass and farmhouse sets), `pack_romantic.py:120`, `pack_kind.py:341`, `pack_couchPotato.py:425`; `base.json:3327`, `furniture.json:945`, `:1005`. Verify: `poses.html` with the real footprints and slots; two residents at coffee and dinner.
13. **G-40** (S3, L) Overall size and proportion polish. Files: `bedroom_storage.py`, `hobbies.py`, `kids.py`, `fitness.py`, `tabletop.py`, `pack_hotHeaded.py`, `pack_gloomy.py`, `pack_bookworm.py`, `outdoor.py`, `playground.py`, `farm.py`, `street_furniture.py`, `living.py`, `lighting.py`. Verify: `cargo review sizes` outliers; gallery; footprints that change (drum kit, yoga mat, arc lamp) re-checked in play.
14. **G-41** (S2, L) Parts wrong, missing or facing away. Files: `pack_neat.py`, `pack_romantic.py`, `hobbies.py`, `pack_outgoing.py`, `pack_bookworm.py`, `kitchen.py`, `kitchen_counters.py`, `decor.py`. Verify: gallery and game camera; play the organizer desk, bureau, foosball, sewing table.
15. **G-42** (S3, M) Seated skirts and jackets drape. Files: `crates/tools/src/characters/clothes.rs:10`, `garments.rs`; `cargo characters`. Verify: `residents/tops-female-sit-side.png` again; the age sheets on dining chairs and stools.

**5a. Code: the S1 groups**

1. **G-43** (S1, M) Two bodies in one place: standing users of shared things. Files: `world.rs:4959-4961` (a standing use keeps the tile it came to), `:357-365` (`front_tile`), `:322-333` (`slot_position`), `cooking.rs:760-763` (the queued cook's spot), `characters.ts` (`STEP_IN` lanes). Verify: the hobbies wave's `two.mjs` (two at the pool table and foosball); the cooking queue on `review-baby`; a sim test that two standing users of one object stand apart.
2. **G-44** (S1, M) Climb the bunk bed's ladder. Files: `characters.ts:2526-2541` (the bunk's upper sleeper), `:304` (`LIE_ENTRY`), a climb like the climbing frame's; `bedroom.py:225-229`. Verify: the bedroom wave's `scenes.mjs` phase `bunk`; `pnpm check:clips`.
3. **G-45** (S1, L) Age rules: who may use what, also on a direct use. **Fix now (owner, 2026-10-21).** Files: `crates/sim-core/src/world.rs:2981` (`Command::Use`), `:3838-3865` (free will's rules), `crates/sim-core/src/content.rs` (a stage rule on objects and uses), `web/src/game/session.ts:1160` (the pie menu), `conversation.rs:63`; content: the `toys` tag on the climbing frame, stages on the kids' furniture, the alcohol and the stoves. Verify: a sim test per rule (direct use refused, free will never picks it); `check.mjs play --sim 3` at the crib, the kids' furniture, the wine fridge and bar cart, the playground.

**5b. Code: contained changes**

1. **G-46** (S2, M) Blankets, pyjamas and bath clothes. Files: `characters/blanket.ts:181-246`, `characters.ts:1176`, `:1489-1497`, `:2373-2378`, `characters/outfit.ts:120-154`. Verify: the bedroom wave's quilt scene (nothing at the origin), every bed's sleep and nap, the lounger's doze; the showers and baths.
2. **G-47** (S2, S) The seat for watching faces the screen. Files: `world.rs:4463` (`seat_near_object`: add a facing test). Verify: the living wave's `tv-seat` run (`--objects homeCinema,tvWall@9,17,2 --with sofa@9,15,2`); a sim test.
3. **G-48** (S2, M) Doors and gates open for the one going through. Files: `BabylonRenderer.ts:3778-3794` (`swingDoors`), `:188-189` (`DOOR_SWING`, `DOOR_REACH`), `fences.ts:42`, `:110` (`gate`). Verify: the houses wave's door scenes (walking past, the bathroom passer, speed 3 headed); the gate walk.
4. **G-49** (S2, M) Cooking: hands free to cook, the dish to the table. Files: `cooking.rs:1501` (`held` through the next step), `:9-14` (serving), `characters.ts:2425` (`writeCarried`). Verify: the held wave's recipe runs (`timeline.jsonl`: nothing held while chopping or stirring); `cargo test -p sim-core` cooking tests.
5. **G-50** (S2, M) State changes when the hands do it: harvests and the washing line. Files: `world.rs:4971-4982` (`leaves` and `picks` at use start), `yard.py:853`, `:459`, `plants.py` `_crop`. Verify: `play --state review-garden --use-existing … --use harvest`; the clothesline; `open_objects.rs` tests.
6. **G-51** (S2, L) Getting onto seats and into bed from the right side. Files: `world.rs:4962-4968` (onto the slot), `end_activity`, `characters.ts` (`sitDown`, `standUp` at `:2755`, `LIE_ENTRY`), the beds' `approach` in content. Verify: the first play shot (`-00`) at the hobbies and beds; beds entered from the side; a headed, timed check of the stand-up (A-residents-01 was never timed).
7. **G-52** (S2, M) Glow drawn through the parts in front of it. Files: `web/src/render/babylon/models.ts:73` (`glows`); then the grill's coals (`outdoor.py:64`). Verify: the grill at night with the lid shut and open (`outdoor/extra/night/sheet-fire.png`).
8. **G-53** (S3, M) Sounds by the thing, not by the clip's tag. Files: `web/src/audio/world.ts:34` (`ACTION_SOUNDS`), `:185` (`soundOf`), `:659-678`. Verify: `cargo review uses` Sound column; a listening check, headed.
9. **G-54** (S3, M) Where things may stand: walls behind, ceilings above, heads under. Files: `content.rs:263` (`fillsWall`; a new `againstWall`), `home.rs:475`, `:491`, `buildmode.ts`; `hangsLow` on the pendants. Verify: sim tests for the refusals; play's free-spot rule (`check.mjs:561`); `cargo review sizes`.
10. **G-55** (S3, M) Standing distances: the oche and the front door. Files: `content.rs:340-346` (`Approach`), `world.rs:4917-4922` (the visitor's spot and yaw), `characters.ts:160` (`STAND_BACK` meanwhile). Verify: `play.jsonl` distance at the dartboard; the knock on `calling-out`.
11. **G-56** (S3, M) Talking seated, and joining a group on foot. Files: `conversation.rs:476-483`, `:537-575`, `:756-776`; `world.rs:5289-5300`. Verify: `review-group`; a seated pair at a table and on a sofa.
12. **G-57** (S3, M) Houses built in code: door panels and wood, wallpapers, chimneys, railings. Files: `house.ts:1348`, `:1776-1783`, `:1959-1960`; `assets/manifest.json:1144`, `:1164`; `base.json:3600-3621`. Verify: the houses wave's door and covering sheets; the town views.
13. **G-58** (S2, L) Street scale: the road, the kerb, the cars, the garages. Files: `web/src/game/town.ts:182`, `street.ts:47-51`, `:205-214`, `:754-757`, `:1131-1140`, `house.ts:1690`. Verify: the town views (`?bench=village`); sim tests that place lots.
14. **G-59** (S3, S) Town leftovers: far ground models, the unused hedge. Files: `nature.ts:580-582`, `street.ts:71`. Verify: `?bench=village` draw and triangle counts.
15. **G-60** (S3, L) Lamps: where their light goes, when they are on, the fan and the fire. Files: `BabylonRenderer.ts:3075`, `:4704-4724`, `house.ts:198-235`, `world.rs:1553`, `lights.rs`, `objectMotion.ts:239`, `furniture.json:4087`. Verify: `lamps-night`, `review-stage-night` and the lighting wave's night shots; the frame budget at night.
16. **G-61** (S2, M) A night on the sofa bed. Files: `catalog2/living.json:246`; `base.json:3679` (`roomKinds`: a sleep tag makes a bedroom). Verify: a sim test that the living room keeps its kind; a night on the sofa bed.

**5c. Code: new behaviour, each its own planned item**

1. **G-62** (S2, L) A place and a facing per slot. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `world.rs:321-333` (`slot_position`), `characters.ts:240-249` (`Seat`), content slots. Verify: `poses.html` with real footprints; two at the picnic table, seesaw, merry-go-round, park chess, ping-pong.
2. **G-63** (S2, L) Uses on things that stand on tops. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `content.rs:2738` (the rule), `world.rs` (using a thing on a top from its counter), `catalog2/tabletop.json`. Verify: sim tests; `cargo review uses` (20 "no use" flags go); play the kettle and toaster on a generated kitchen.
3. **G-64** (S2, L) Conditions: a time, a place, something to work on. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `content.rs:70` (`Interaction`: an hour, sky or condition field), `world.rs` free will, `base.json:1797-1809` (the night schedule). Verify: sim tests (stargazing only after dark under open sky; washing up only with dishes); `cargo review uses`.
4. **G-65** (S2, L) Two-player things need a partner. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `content.rs:70` (a partner condition on an interaction), `world.rs` (pairing two users). Verify: sim tests; two at the seesaw, ping-pong, park chess, foosball.
5. **G-66** (S2, L) Babies at the high chair and the changing table. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `content.rs:273-276`, `world.rs:322`, `characters.ts` (`CUDDLE`-style lift), `catalog2/dining.json:744`. Verify: `review-baby`; sim tests `babies`.
6. **G-67** (S2, L) Food: who sells it, where it comes from, where it is eaten. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `world.rs:3689-3712` (`requires`), `community.json` sales, `furniture.json` breakfasts, `cooking.rs`. Verify: the café, shop and stall runs; sim tests.
7. **G-68** (S3, L) Crops: sowing, tending that helps, harvests to the kitchen. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `world.rs:2765-2786` (`grow`), `content.rs:2064-2074` (`fallow`), `garden.json`, `catalog2/outdoor.json:342`. Verify: sim tests on growth; `review-garden` play.
8. **G-69** (S3, L) Pairings: feet up on a footstool, changing outfit. *New behaviour, fixed in this Phase 4 (owner, 2026-10-21).* Files: `furniture.json:1389`, `catalog2/living.json:572`; `outfit.ts:120` (`outfitFor`). Verify: play the armchair with a pouf in front; the wardrobe's outfit changes the clothes.

**5d. Code: new behaviour, filed only, planned later (owner decision 3)**

1. **G-70** (S2, L) Props in hand: book, cup, cutlery, phone, tools, the shaker, the cue. **New behaviour — filed only, planned later.** Files: `characters.ts` (`ACTIONS`, `writeProp`, `held` grips at `:371`), new prop models in `held.py`. Verify: play every use with a prop; `pnpm check:clips`.
2. **G-71** (S2, L) Body language and turning on the spot. **New behaviour — filed only, planned later.** Files: `characters.ts:684` (face moods), `characters/clips/locomotion.ts`. Verify: a headed, timed check.
3. **G-72** (S3, L) Privacy: a private flag, a shut door, a place for a baby. **New behaviour — filed only, planned later.** Files: `content.rs`, `BabylonRenderer.ts:3778`, `house.ts:757-760`, `base.json:1678` (`tryForBaby`). Verify: sim tests (visitors refused private things); door shots.

## 5. Decisions for the owner

Every `decision` finding (31) and every question a fix group raises, each as a choice with a recommendation. **The owner accepted every recommendation on 2026-10-21** (§11.1).

**Who may use what, and the S1s that need new code**

- **Q-01: age rules for the S1s (G-45: I-kids-03, I-kitchen-06, I-community-04, I-systems-04, I-kids-08).** The S1 rule now covers adults and children using each other's things, and three S1s are such cases: a child pours wine, an adult rides the toddlers' spring horse, an adult lies in the cot. Fixing them needs the hard age rules that decision 3 files as new behaviour, planned later. Fix them now, partly now, or later? **Recommendation:** now, in two steps. First the contained part (M): a direct use applies the rules free will already keeps (a baby's use for a baby, care by an adult, homework by a pupil), and the climbing frame gets the `toys` tag. Then plan the age rules as the first new-behaviour item: a life-stage range on objects and uses (the kids' furniture, the toddler rides, the alcohol, the stoves, the upper bunk), checked on free will and direct use alike, and for fights with a child. The kids' table as a place to eat (I-kids-08) waits for it.
- **Q-02: the bunk's ladder (G-44: A-bedroom-02, S1).** Climbing the ladder is new code, though not on decision 3's list. Fix now? **Recommendation:** yes, as a step-in round like the climbing frame's.
- **Q-03: doors (G-48: A-houses-01).** Doors by hand are decision 3's new behaviour, but the cheap part closes the S2: a door opens only for someone going through it, and an engaged bathroom's door stays shut to everyone but the one leaving. Do the cheap part now? **Recommendation:** yes; hands on handles later.
- **Q-04: dining for two (G-39: M-kitchen-01, M-packs-11, I-packs-11; G-62: I-outdoor-07).** Two diners face across a table only once a slot can have its own place and facing (new behaviour, L). Rebuild the tables first, or with it? **Recommendation:** plan G-62 right after the age rules and rebuild the base table, the two dining sets and the candlelit table with it, not before.

**Models: rebuild, rename or accept**

- **Q-05: the shower's glass door (G-33: A-bathroom-01).** It was left in the still body to keep the shower at three draw calls; residents walk through it. Allow a fourth draw for a sliding door? **Recommendation:** yes; the base shower is in every home.
- **Q-06: wall-hung parts on floor things (G-36: M-packs-01, M-packs-02, M-outdoor-06).** The entry console's mirror and hook board, the spritz station's mirror and the oven's jar shelf hang in the air when they stand free; the rain barrel's downpipe ends at 2.1 m. Back panels or posts, wall-only placement (G-54's rule), or drop the parts? **Recommendation:** back panels or posts for the three pack things; drop the downpipe for a lidded barrel.
- **Q-07: hammocks (G-37: M-packs-06, M-outdoor-05).** At 1.86–1.88 m grown-ups lie in them folded. A 3 × 1 footprint (real stands are 2.7–4 m), a sitting swing pose, or accept the knees? **Recommendation:** the 3 × 1 footprint.
- **Q-08: names or rebuilds (G-11).** **Recommendation:** rename where the model is right and the name isn't; widen the reference where whole tiles are the style:
  - M-bedroom-01: call the base "Double bed" a king-size bed, and the futon a king futon;
  - M-kitchen-03: name the tables by the seats they take ("Table for six" for the 1.84 m one) and widen the reference for whole-tile sideboards, bars and cookers;
  - M-kitchen-04: call the built-in fridge a counter-depth fridge (kind `fridgeAmerican`);
  - M-lighting-04: call the ceiling light a lantern pendant;
  - M-living-03: accept the large sofas, the chaise and the long coffee table (they fill their tiles), and fix the console table's "slim" text;
  - M-packs-09: call the spin bike a recumbent bike;
  - M-garden-08: keep the cactus trio and succulent bowl as floor pots (the tabletop has small ones);
  - M-bathroom-01: one kind for the base sink, with the kitchen sink's range (it reads right in both rooms).
- **Q-09: the street's own hedge (G-59: M-town-02).** Never drawn. **Recommendation:** drop it.

**Rules and shortcuts**

- **Q-10: bath clothes (G-46: I-bathroom-03).** Residents shower and bathe dressed, behind clear glass. A towel or swimwear outfit (drawn as the pyjamas are), frosted glass, or the style's convention? **Recommendation:** swimwear for the hot tubs and pools and a towel outfit for showers and baths, both through the pyjama path in `outfit.ts`.
- **Q-11: a lamp bought (G-60: I-lighting-01).** It comes on at once, at night in an empty room too. Off when bought, or on? **Recommendation:** off when bought; residents switch it on as they come in, as they do today.
- **Q-12: pendants (G-54: M-lighting-05).** They hang 1.80–2.04 m up where residents walk under them. A shorter drop, or `hangsLow` (over a table only)? **Recommendation:** `hangsLow` for the chandelier and the industrial pendant, a shorter drop for the ceiling pendant and the paper lantern.
- **Q-13: the street (G-58: M-town-05, M-town-07).** 3.3 m between the kerbs, with a centre line, and a 3.3 cm kerb. Widen the street to 8.5–9 m (it moves every lot), or drop the centre line and read it as a one-lane mews; a real kerb step, or flat? **Recommendation:** drop the centre line and keep the kerb flat now; widen the street only with a new town layout.
- **Q-14: no seat near (G-03).** With `seatNear` on the reading and listening uses, what does a resident do when no seat is near? **Recommendation:** stand, as the TV's watcher does today; sit on the floor once G-22 has a floor sit.
- **Q-15: a night on the sofa bed (G-61: I-living-04).** A `sleep` use would make the living room a bedroom. A sleep that doesn't type the room, or a napping sofa only? **Recommendation:** a sleep that doesn't type the room; a sofa bed is bought to sleep a guest.
- **Q-16: two-player things (G-65: I-hobbies-01, I-community-03).** Keep the home chess table and foosball as solo uses until partners exist? **Recommendation:** yes, as practice ("Practise chess", "Practise foosball"); the seesaw, ping-pong and park chess get a partner condition with G-65.
- **Q-17: food (G-67: I-community-10, I-kitchen-10, I-systems-03; with I-community-09).** Shop food is eaten in the aisle, breakfasts at dining sets come from nowhere, and with neither sink nor bin plates pile up. **Recommendation:** relabel the stall's bag of fruit as a piece of fruit and send shop food to a seat (content, now); keep the dining sets' breakfasts as a game shortcut; leave the plates (every generated home has a sink); vendors later.
- **Q-18: crops and flowers (G-68: I-systems-08, I-garden-10; G-10: I-garden-08).** Crops replant themselves, tending changes nothing, harvests feed on the spot, and no flower can be cut for a vase. **Recommendation:** keep the replanting and eating on the spot as shortcuts; give the fruit trees ripeness (`grows`); make the lemon a cooking or drink boost; cut flowers later, with a vase to put them in.
- **Q-19: decor or a use (G-12: I-kitchen-09, I-decor-04, I-community-12, I-office-02).** **Recommendation:** decor for the spice rack, pot rail, kitchen shelves, coat stand, executive desk's drawer and the farm (decision 4); a sit on the hay stack (content, cheap); the range hood's light and hum while the hob below is in use, later, as polish.
- **Q-20: things on tops in generated homes (G-63: I-tabletop-04).** Every generated kitchen has a kettle and a toaster nobody can use. Keep them until G-63, or leave them out? **Recommendation:** keep them; they dress the kitchen.
- **Q-21: "Try for a baby" (G-72: I-social-05).** A 30-minute standing kiss anywhere, the children a few metres off. Keep it, or a double bed behind a shut door (privacy, new behaviour)? **Recommendation:** keep it until the privacy flag (G-72), then move it to a bed.
- **Q-22: uses end when the needs are full (G-13: I-systems-02).** **Recommendation:** keep, by design; review states keep setting needs to 0.5.

**Planning**

- **Q-23: which new behaviour comes next.** **Recommendation:** in this order:
  - the age rules (Q-01);
  - a place and facing per slot (G-62);
  - props in hand (G-70: it closes S2s in every ledger);
  - uses on things on tops (G-63);
  - conditions (G-64);
  - then the rest.

## 6. Owner decisions so far

From planning (plan §9, 2026-10-21):
1. **Sizes:** functional heights (seat, counter, table, desk, mattress, basin, toilet seat) must be inside the real-world range. Overall size and parts may stray about ±10 % for readability from the camera.
2. **Durations:** today's short durations are by design. Only outliers against similar uses are flagged (more than 3× from the median for the tag).
3. **New behaviour** (props in hand, emotional body language, a turn-on-the-spot clip, doors opened by hand, hard age rules, a privacy flag): filed as `code` findings only, planned separately later.
4. **Decor without uses** (default): decor stays decor. A clear missing use is an I1 finding, decided at triage.
5. **Residents' proportions** (default): changes to `rig.rs` are filed only and decided at triage.
6. **Pace** (default): the review runs as its own workstream, on the branch `review/authenticity`, through Phase 4.

After the pilot (2026-10-21) the owner approved:
- M14's head ratios based on the style, half a head bigger at every stage (M-residents-01 closed on it);
- a new rubric line, I16 conditions;
- a measuring tolerance on functional heights: 1 cm, 5 mm on table tops;
- the bookkeeping rules: a resident's pose at an object filed under I3 with A1 in its Rubric field, a "Real uses against the content's" table under each group, a contact sheet per object in the Shots cell, and a shared cause filed once and moved to `systems.md` or `residents.md` at the merge;
- the S1 example for adults and children using each other's things;
- a `decision` finding rated by its worse outcome.

Also from the plan: S1 is fixed now unless the owner says otherwise (§7.2).

## 7. Tooling left to improve

From the coordinator's notes and each wave's `review-log.md`, deduplicated; the fixed ones are left out.

**`cargo review sizes` (T3, `crates/tools/src/review.rs`)**
- It measures the wrong face at some kinds:
  - basins: the drain or the slab's underside, so the vessel basins' 1.00 m rims were missed (M-bathroom-02);
  - toilets: the closed lid;
  - the kettle grill: a point under the grate;
  - the pool table: the cloth, not the rail.
  It wants the top up-facing rim of a basin kind, and the uses' open clips played.
- Its sizes include things that aren't the object's: the dresser's picture, the dressing table's stool, the climbing frame's flag, the desks' built-in chairs, the toy box's blocks, the yoga mat rolled up.
- Wrong kinds or ranges in `reference.json`: the seesaw's W and D swapped; the bakery case measured as a counter; `recoveryPod`, `quietPod` and `spinBike`; the WC range has no boxed-in cistern.
- No stool line for counter stools, so the chef's island stools (M-packs-04) weren't flagged.
- "No top to measure" where content has no `surface` (the workbench, the chess table).

**`cargo check-glb` heights (T1, T2)**
- It probes one point, so a gap between slats, planks or cushions gives a false warning (six outdoors, the park bench). It wants the highest point over a 5–10 cm strip.
- For a tub it reads the water's surface, not the tub's floor. The crib's check reads the changing pad under slot 1.
- Not in CI yet, nor `cargo review sizes` and `uses` (plan §1 success criteria); only `pnpm check:clips` is.

**`cargo review uses` (T4)**
- The ladder shelf has no use but isn't flagged.
- A flag for an odd sound for the kind (a sizzle away from a hob) would have found I-packs-19 at once.

**`tools/bench/check.mjs play`**
- One resident per object. Wanted: `--sims a,b` for pairs (the seesaw, ping-pong, the climbing frame, two at a pool table), and a social and phone mode (the cross-cutting wave wrote its own scenes).
- `--use ripe`, for every `when: ripe` use (the veggie bed's is `pickSnack`), and the growth stage in the record.
- It records clips only on the object used, not on the helpers a recipe walks to; it can't follow a whole recipe.
- Picture names leave out the helper, so one chair tried at several desks overwrites itself.
- The free-spot rule keys on `fillsWall`, so the wall bed was put mid-floor.
- The camera:
  - it is sized by the footprint, so it cuts tall trees and sits inside the phone box;
  - one camera per run (wanted: `--views a,b,c`, and a `--top` view for seats at tables);
  - no shot of an object as bought, with no use (wanted: `--shot-idle`, by day and at night).
- Needs carry over from one use to the next in a chunk, so short uses vary. It should reset them or log them at the use's start.
- Wanted in the log: the resident's distance to the object's nearest face, the slot against a `stand` surface, and the doors' state.
- A `--build` option (edges built first).

**`check.mjs sheet`**
- It finds no pictures from a run made with `--name`.
- For the beds after the pilot, the age sheets show only the elder.
- It takes no list of previews, and its `-01` pick for a bed comes before the blanket fades in.

**`poses.html`**
- It builds the `catalog2/dining` objects 1 × 1 and doesn't load `catalog2/bath.json`.
- `seats=` poses a plain sit with two sitters even on a one-seat thing, on the closed lid, and draws a bed that opens shut.
- It shows the cold plunge's cover on while someone sits in it.
- Labels are drawn away from their stations, and `focus=` counts in the filtered list.

**The gallery and previews**
- The gallery stands tabletop things on the ground and draws the range hood on the floor.
- Previews draw glass opaque, and ids used in two families overwrite each other's (E1).

**Debug states and commands**
- No command sets a relationship or a need; the cross-cutting wave hand-edited states with a scratch Python script, which the project's rules want in Rust (`DUMP_STATES` tests).
- Free will runs for a moment as a page opens.
- No state has the street (`world.meta` is null; only `?bench=village` shows it), and there is no night state of a town lot.
- `__world().objects` doesn't carry a lamp's `off`.
- `__controls.follow(null)` sends the view back to the home lot.
- `hud=0` doesn't hide the top bar.
- The roof hides when the camera is close.

**Known before the review** ([docs/review/README.md](review/README.md) "Tooling")
- The lamp build checks only the light's height, not `at` (`lighting.py:94`).
- `pnpm wasm` and `check:clips`' pre-script fail on a symlinked `node_modules`.
- `partBox` bounds spheres and tori a little big.

**The agents' sandbox (for the next briefs, not the game's tools)**
- A command is refused when it contains a `$VAR` path, an `awk` program, or a heredoc with "git" in it, even inside "digitalPiano" or "keyboardDigital".
- The Write tool refuses a script whose text names another worktree's path.
- Images drop out of an agent's context after a few dozen; contact sheets and notes written as you go fixed it.
- Agents sharing one scratch folder overwrote each other's scripts. Each wants its own.

## 8. Fixed

Phase 4. Each row will name the group, the commit, and the before and after shots.

| Group | Findings | Commit | Merge | Before → after |
|---|---|---|---|---|
| G-63 | I-tabletop-01, I-tabletop-04 | ed77dc36, 5209bd53 | | no thing on a top usable → 20 used at their top (`fixes/tops-conditions/tops-adult-*.png`, `generated-82-*.png`) |
| G-64 | I-hobbies-03, I-kitchen-11, I-bathroom-07 (in part) | 69d0bc33 | | stargazing at noon indoors, washing up with nothing to wash → refused with the reason; used after dark outside, with dishes (`fixes/tops-conditions/play-stars-*.jsonl`, `play-washup-*.jsonl`, `play-laundry.jsonl`) |
| (with G-45) | I-systems-11 | 9463d1a7 | | a child's coffee → teens and up (`tests/tops_and_conditions.rs`) |
| G-27 | A-social-01, A-social-03, I-social-02, I-social-03, I-social-04 | 43ca1cc1, 2d887ff9 | | `social/sheets/romance.png`, `fights.png`, `fails.png`, `talks.png`, `phone.png` → `fixes/clips-social/sheets/` (the same names, `flat-joke.png`, `phone-sofa.png`) |

## 9. Found but not fixed

Phase 4: the groups triaged to a later release or "won't fix", with the reason and a target.

| Group | Findings | Severity | Why not now | Recommendation | Target |
|---|---|---|---|---|---|

## 10. Manual test checklist

For each fixed object, from [docs/objects/README.md](objects/README.md) "Checking an object in play":
- [ ] Buy it and place it in all four facings.
- [ ] Colour every part in the colour studio (every slot a distinct colour, far from each part's default), "At rest" and "In use", turning the turntable.
- [ ] Let a resident use it, so every clip plays and shuts when it should (sit and lie uses again about 4 s in).
- [ ] By day and at night, on WebGPU and `&renderer=webgl`, on the Medium and Low presets.
- [ ] Check the neighbours that share code you touched (a pose, a `BEDS` entry, a family helper).

What the review learned to check as well:
- [ ] Every life stage that may use it: `poses.html?stage=all` and `check.mjs play --sim <n>` (feet on the floor, shins out of the cushion, the child's reach).
- [ ] Two residents at once on anything with two slots (by hand until `play --sims` exists).
- [ ] A side or top-down shot for contact: hands on the part used, legs not through the front, sitters not inside the object.
- [ ] A night pass for anything that should glow or give light (`review-stage-night`).
- [ ] `cargo check-glb` heights, `cargo review sizes` and `cargo review uses` before and after; `pnpm check:clips` for a changed clip; the balance and soak tests for changed `minutes` or `effects`.
- [ ] Listen: the use's sound starts and stops with it.

Only a person can check these, on hardware no agent had or at timings shots can't show:
- [ ] Timed, headed: a door at speed 3 (A-houses-02), the stand-up from a chair (A-residents-01), walks starting and stopping at speeds 4–5 (A-residents-04), moving parts at the hand (A10).
- [ ] A hidden tab (A-residents-05, A-residents-06): switch away mid-walk and mid-sit and back.
- [ ] Build mode with a wall thing held over lowered walls (M-houses-01); the stairs from a basement view (M-living-02); a wall removed under a ceiling lamp (M-lighting-01).
- [ ] The household creator's preview with reduce motion on (A-town-01).
- [ ] A far road's lane markings over several page loads (M-town-03).
- [ ] A crowded garden (I-garden-01) and children near a hug (A-social-02).

## 11. Phase 4

### 11.1 Triage

The owner's decision per group: fix now, a later release, or won't fix (with the reason). S1 groups are fix now unless the owner says otherwise.

Triage with the owner on 2026-10-21:

- **Scope:** every group but the new behaviour decision 3 names: G-01 to G-69 are fixed in this Phase 4, including the S3-only groups; G-70 (props in hand), G-71 (body language, turning on the spot) and G-72 (privacy), and doors opened by hand, are planned later.
- **Age rules now:** the owner chose the full age rules (G-45) in this Phase 4, not the two-step recommendation.
- **G-62 to G-69:** the eight new-behaviour items the consolidation marked as their own items are brought into this Phase 4 too.
- **Decisions:** every recommendation in §5 is accepted as written. Where a recommendation assumed a group would wait, the group's own fix now takes its place: Q-16 (chess and foosball practise alone) becomes the fallback when no partner is free, now that G-65 gives them partners; Q-18 keeps the crop shortcuts it names and G-68 builds the rest.

| Group | Decision | Release | Notes |
|---|---|---|---|
| G-01 | fix now | Phase 4 | |
| G-02 | fix now | Phase 4 | |
| G-03 | fix now | Phase 4 | |
| G-04 | fix now | Phase 4 | |
| G-05 | fix now | Phase 4 | |
| G-06 | fix now | Phase 4 | |
| G-07 | fix now | Phase 4 | |
| G-08 | fix now | Phase 4 | |
| G-09 | fix now | Phase 4 | |
| G-10 | fix now | Phase 4 | |
| G-11 | fix now | Phase 4 | |
| G-12 | fix now | Phase 4 | |
| G-13 | fix now | Phase 4 | |
| G-14 | fix now | Phase 4 | |
| G-15 | fix now | Phase 4 | |
| G-16 | fix now | Phase 4 | |
| G-17 | fix now | Phase 4 | |
| G-18 | fix now | Phase 4 | |
| G-19 | fix now | Phase 4 | |
| G-20 | fix now | Phase 4 | |
| G-21 | fix now | Phase 4 | |
| G-22 | fix now | Phase 4 | |
| G-23 | fix now | Phase 4 | |
| G-24 | fix now | Phase 4 | |
| G-25 | fix now | Phase 4 | |
| G-26 | fix now | Phase 4 | |
| G-27 | fix now | Phase 4 | |
| G-28 | fix now | Phase 4 | |
| G-29 | fix now | Phase 4 | |
| G-30 | fix now | Phase 4 | |
| G-31 | fix now | Phase 4 | |
| G-32 | fix now | Phase 4 | |
| G-33 | fix now | Phase 4 | |
| G-34 | fix now | Phase 4 | |
| G-35 | fix now | Phase 4 | |
| G-36 | fix now | Phase 4 | |
| G-37 | fix now | Phase 4 | |
| G-38 | fix now | Phase 4 | |
| G-39 | fix now | Phase 4 | |
| G-40 | fix now | Phase 4 | |
| G-41 | fix now | Phase 4 | |
| G-42 | fix now | Phase 4 | |
| G-43 | fix now | Phase 4 | |
| G-44 | fix now | Phase 4 | |
| G-45 | fix now | Phase 4 | The owner chose the full age rules now (2026-10-21), overriding decision 3 for age rules only. |
| G-46 | fix now | Phase 4 | |
| G-47 | fix now | Phase 4 | |
| G-48 | fix now | Phase 4 | The cheap part (Q-03): open for the one going through; doors opened by hand stay later. |
| G-49 | fix now | Phase 4 | |
| G-50 | fix now | Phase 4 | |
| G-51 | fix now | Phase 4 | |
| G-52 | fix now | Phase 4 | |
| G-53 | fix now | Phase 4 | |
| G-54 | fix now | Phase 4 | |
| G-55 | fix now | Phase 4 | |
| G-56 | fix now | Phase 4 | |
| G-57 | fix now | Phase 4 | |
| G-58 | fix now | Phase 4 | |
| G-59 | fix now | Phase 4 | |
| G-60 | fix now | Phase 4 | |
| G-61 | fix now | Phase 4 | |
| G-62 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-63 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-64 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-65 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-66 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-67 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-68 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-69 | fix now | Phase 4 | New behaviour the owner brought into this Phase 4 (2026-10-21). |
| G-70 | later | after this review | New behaviour the owner put off (decision 3): planned separately. |
| G-71 | later | after this review | New behaviour the owner put off (decision 3): planned separately. |
| G-72 | later | after this review | New behaviour the owner put off (decision 3): planned separately. |

### 11.2 Re-check

The Phase 0 evidence made again for the changed items, and each finding closed in its ledger with a before and after shot (plan §7.5).

### 11.3 Sign-off

To come.
