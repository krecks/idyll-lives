# Pilot notes: the base set and the residents

The pilot of the authenticity review ([plan](../../plan/authenticity-review.md) §6.1 step 1, 2026-10-21): the 28 objects of `web/public/content/base.json` and the residents, reviewed by one agent on one core, headless. This page is for the owner, before the other waves start: what worked and what didn't in the rubric, the severities, the ledger format and the tools; the changes proposed; what each step cost; and the findings.

Evidence: `bench-results/review/pilot/` in the review worktree (gallery, game-camera, clip, play, pose and character shots, contact sheets, scratch scripts and `log.md`).

## What was done

- **Rows filled:** all 28 objects in their category ledgers (kitchen 2, bathroom 3, bedroom 2, living 5, garden 5, outdoor 1, fitness 2, hobbies 5, office 1, lighting 1, kids 1), each with M, A and I verdicts, the shots and a "Real uses against the content's" table (I1) under the ledger's rows. In `residents.md`, 144 of 161 rows; the 17 left are `ACTIONS` entries no base object uses, marked for the wave that owns their objects.
- **Findings:** 24 new, 10 seeded ones confirmed (one raised to S1 and widened), 5 seeded ones marked "not checked in the pilot" (speed and hidden-tab issues the evidence can't show). Table at the end.
- **Evidence:** 56 gallery shots, 85 clip shots, 112 play records (adult, then child, teen and elder on subsets, then night), 124 pose shots, 79 character sheets and clip strips, 73 shots of scripted scenes (care with a real baby, coffee with a machine, the pine walked past, the base sink in generated kitchens, a walk upstairs, night), `pnpm check:clips`.

## The rubric

**What worked**

- **M first, then A, then I** (§6.2) is the right order: most A and I findings needed the model's numbers (the telescope's eyepiece height, the cot's mattress, the bench's slats) that the M pass had already looked up.
- **M2 strict, M1 ±10 %** (§9.1) was easy to apply and caught the right things: the workbench 7 cm low, the bench seat 1 cm over the range and 4 cm off `SEATS`.
- **I1 with the real uses next to the content's** turned up the most useful findings. The base set is plainer than its own siblings (I-kitchen-02), and the bookshelf and the tree have the right use in the wrong pose (I-living-03, I-garden-02).
- **The §9 decisions removed most arguments**: durations "by design", props and age rules as `code`, `rig.rs` filed only.

**What didn't**

- **M14's head ratios don't fit the style.** Every stage is about half a head bigger than the rubric's ranges, on purpose, so the rubric line reads as failing everywhere (M-residents-01). A rubric line every item fails tells you nothing.
- **A and I overlap for resident clips at an object.** Whether the stargazer's pose fits the telescope is A1 (contact) and I3 (pose). I filed each under I with A1 in its Rubric field and put the id in both cells. One rule would keep the ledgers consistent.
- **There is no line for "the right use, no rule about where or when".** Stargazing at noon indoors was filed as I1 and I6 (I-hobbies-03). It isn't coverage or age.
- **The one finding that breaks belief most isn't an object's.** The command `use` skips the user checks free will makes (I-kids-03). Every object ledger will meet it again (children at stoves, adults in cots), and §3.3 doesn't say who owns such a cause.
- **"Moving part … at the hand" (A10)** can't be judged from shots taken 1 s apart. It needs the motion log against the resident's clip time, which `play.jsonl` has but no table shows.

**Proposed changes (for approval)**

1. **M14:** re-base the ratios on the style, half a head bigger at every stage: baby about 1:3.5, child 1:4.5–5, teen 1:6–6.5, adult 1:6.5–7, elder a little shorter with a stoop. Keep "decided at triage" for any `rig.rs` change.
2. **§3.3, new line I16. Conditions:** a use whose real thing needs a time, a place or a state (night for stargazing, outdoors, someone to play with, a baby in the crib) has it or is filed.
3. **§3.2 A1, add a sentence:** "A resident's pose at an object (reach, eye line, contact with its parts) is filed under I3 with A1 in its Rubric field; the A column names the same id."
4. **§7.1, add:** "A finding whose cause is in the sim or the renderer and not in one object (a `Command::Use` rule, an `ACTIONS` entry, a clip) goes in the ledger of the first object it was seen on, with every other affected item named, and the other ledgers' rows point to it. The coordinator moves it to `systems.md` or `residents.md` at the merge." The pilot did this for I-kids-03, I-held-01 and I-bathroom-04.
5. **§3.1 M2, add a measuring tolerance:** "A functional height up to 1 cm out (5 mm on table tops) counts as inside, as the modelling grid and bevels move it that much." The dining table's 0.765 m top and the 0.92 m basin rim are then in. Without it, the strict rule files findings nobody would fix.
6. **Residents ledger:** move the 45 `ACTIONS` rows out of `residents.md` into the object waves (each wave judges the entries its objects use, at those objects), or into the cross-cutting wave. They can't be judged on their own; the 17 the base set doesn't use stay pending.

## Severity levels

- They worked for objects. S1 came out rare (one, I-kids-03) and every S2 is something a player watching a resident would notice.
- **Proposed:** add to S1 "an adult using a child's or a baby's thing, or a child an adult's, as if it fitted". Without it, the S1 example "a child-sized adult chair" covers only the size.
- **Proposed:** for decisions, give the severity of the worse outcome. A `decision` finding (the bed's name, the sink's kind) was hard to rate: the problem is real but its size depends on the answer.

## Ledger format

- **Worked:** the table row plus a Findings section; ids per ledger and letter; grouping one cause into one finding.
- **Didn't:**
  - The Shots cell gets long; four paths are about the limit. **Proposed:** name a contact sheet per object (`sheets/obj-<id>.png`, below) plus at most two more shots.
  - A finding seen on several ledgers' rows has to be named in each row's Findings cell with "(xyz ledger)". That worked but is easy to miss.
  - **Proposed:** a "Real uses against the content's (I1)" table under each group, as the pilot added. It is where the real-world uses are listed, which the row format has no room for.
- **Status values used:** `open`, `confirmed (pilot …)`, `confirmed and widened`, `confirmed fixed on <item>` and `seeded …; not checked in the pilot`. **Proposed:** these become the list in README.md.

## Tools

**Worked**

- `check.mjs gallery --each --scale all`, `--camera game`, `clips --each` and `play --state review-stage --objects … --use all` all ran first time, with no page errors.
- `play`'s `--sim <n>` gives the other life stages on `review-stage` for free.
- `cargo review sizes|uses` tables and the heights warnings are the best starting point; each flag led to a finding or to an artefact (below).
- `pnpm check:clips` (20 s) answers the clip questions at every stage.

**Problems**

1. **Images in the agent's context.** After a few dozen single PNGs, earlier ones dropped out of the agent's context ("media removed"), and a first pass over about 60 shots was lost. The workaround: 2 × 2 contact sheets with ffmpeg (`scripts/sheets.mjs`: gallery, game camera, play, clip open per object; seats by stage), with notes written right after each batch. **Proposed:** `check.mjs --sheet` writes `sheet-<id>.png` per object; reviewers read sheets, not single shots, and keep notes as they go.
2. **What the tables measure is sometimes the wrong thing:**
   - the dining table's H is its chairs' backs;
   - the stairs' H is the handrail over the top;
   - the crib's "mattress 0.78" is the changing pad under slot 1;
   - the computer desk's "H 1.21 > 1.21" is rounding;
   - the workbench, chess table and sink have "no top to measure".

   **Proposed:** `cargo review sizes` measures the top of every kind with a `surface` range even without a content `surface` (as the heights check measures seats), measures a set's table without its chairs, prints 3 decimals in the comparison, and the crib check takes slot 0 only.
3. **`play` can't set up a scene.** A crib's care uses run on the new, empty crib; the coffee needs a machine on the lot; `gourmet` needs a skill; the night needs the sim's clock (`--hour` only lights the renderer). A scratch script did these. **Proposed:**
   - `--use-existing <def>` (use the state's own object, e.g. `review-stage`'s crib with its baby);
   - `--with <def>@x,z,rot` (buy a helper first);
   - `--skills <id>=<level>`;
   - and `review-stage` at night as a state of its own.
4. **`poses.html` is hard to read.** The whole-page views have labels drawn away from their stations, and `focus=<n>` counts stations in the filtered list (one past the end hangs: no `__ready`). **Proposed:** labels on the station's floor, and `focus` by label.
5. **The sandbox's checks cost time:**
   - Bash heredocs that contain the word "git" are refused;
   - `node $VAR/script.mjs` is refused;
   - the Write tool can't write outside the agent's worktree, so the log and scripts are written in the worktree's gitignored `bench-results/` and copied.

   **Proposed** for the brief of the next waves: "write scratch files in your worktree's `bench-results/review/<cat>/` and copy them to the shared evidence folder; use literal absolute paths".
6. **Vite** started in the background from the worktree root serves 404s (the root is `web/`): `vite <abs>/web --config <abs>/web/vite.config.ts` works.

## Time per step

One core, headless, Medium. Wall clock from `log.md`. 11:38 to 12:55 in all, about 77 minutes for 28 objects and the residents.

| Step | For the 28 objects | Per object |
|---|---|---|
| Setup (merge, wasm, states) | 1 min | — |
| `gallery --each --scale all` | 62 s | 2 s |
| `gallery --camera game --scale adult` | 31 s | 1 s |
| `clips --each` (17 with motions) | 80 s | 4.5 s |
| `play --use all` (adult) | 9.5 min | 20 s |
| `play` child, elder, teen (subsets: 34 objects) | 11 min | 19 s |
| poses (seats and beds × 4 stages, focus shots) | 5.5 min | 12 s per seat or bed |
| scripted scenes (care, coffee, night, pine, kitchens, stairs) | 6.5 min | — |
| characters and strips (residents) | 2.5 min | — |
| contact sheets | 10 s | — |
| reading the evidence and filing (incl. the lost first pass, about 10 min) | about 35 min | about 1 min per object, plus 10 min for the residents |

For the next waves the evidence costs about 1 minute an object on one core (play dominates); the judging and filing about 1–2 minutes an object for plain ones and 5–10 for ones with findings.

## Findings by severity and fix class

New and confirmed, pilot rows only (24 new, 10 confirmed; 5 seeded ones not checked are left out).

| Fix class | S1 | S2 | S3 | Total |
|---|---|---|---|---|
| content | | I-living-03 | I-kitchen-02, I-kitchen-01 (confirmed fixed) | 3 |
| renderer-table | | I-kids-04 | | 1 |
| clip | | I-garden-02, I-hobbies-02, A-fitness-01 (c), A-residents-09 (c), A-residents-11 (c) | I-bathroom-02, A-residents-02 (c), A-residents-10 (c) | 8 |
| motion | | | A-fitness-02 | 1 |
| model | | M-kitchen-01, A-bathroom-01, M-garden-01, M-hobbies-01 | M-outdoor-02, M-fitness-01, M-hobbies-02, M-kids-01, M-residents-02 | 9 |
| code | I-kids-03 (c, raised) | I-hobbies-03, I-kids-05, I-held-01 (c), A-residents-07 (c) | I-bathroom-04, A-residents-08 (c) | 7 |
| decision | | | M-bathroom-01, M-bedroom-01, I-bathroom-03, I-hobbies-01, M-residents-01 | 5 |
| **Total** | 1 | 15 | 18 | 34 |

(c) = a seeded finding confirmed by the pilot.

**S1**
- **I-kids-03:** a direct use skips the user rules free will keeps: an adult lies in the cot, a child or teen feeds and changes the baby, an adult does homework (`world.rs:2981`).

**S2**
- **M-kitchen-01:** the base dining table is a 38 cm deep plank with both diners side by side.
- **A-bathroom-01:** the shower's glass door is part of the still body; the resident walks through it.
- **M-garden-01:** the pine's lowest tier, 6 m across from 1.4 m, swallows residents walking by.
- **M-hobbies-01:** the workbench is 7 cm low (0.78 m) and reads as a steel desk.
- **I-garden-02:** "Relax in the shade" is 30 minutes standing by the trunk.
- **I-hobbies-02:** the stargazer stands upright beside the telescope, the eye 0.6 m above the eyepiece.
- **I-hobbies-03:** stargazing at noon, indoors; the night schedule even weights it down.
- **I-kids-04:** feeding the baby is the gardening kneel at the cot rail.
- **I-kids-05:** the baby is changed over the cot rail; the changing table is never used.
- **I-living-03:** reading at the bookshelf, and TV with no seat near, are an hour on foot; the bookshelf's uses lack `seatNear`.
- Confirmed:
  - A-fitness-01: the bench press legs;
  - A-residents-09: seated feet off the floor;
  - A-residents-11: shins lost on the sofa, widened to the child and teen;
  - I-held-01: no props, widened to the base set's tools;
  - A-residents-07: no body language.
