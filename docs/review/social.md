# Social interactions: authenticity ledger

What it covers: the 19 socials (`base.json` `socials`), the 5 phone actions (`base.json` `phone.actions`, from teens up) and group conversations (`crates/sim-core/src/conversation.rs`, up to 4). Poses and pairs: `web/src/render/babylon/characters.ts` (`EMBRACE`, face moods). Rubric I10–I12 and A7.

- **Evidence:** `bench-results/review/social/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Socials (19)

Label, animation and minutes from content.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `introduce` Introduce yourself (talk, 8 min) | — | — | — | | |
| `chat` Chat (talk, 20 min) | — | — | — | | |
| `joke` Tell a joke (laugh, 10 min) | — | — | — | | |
| `compliment` Compliment (talk, 5 min) | — | — | — | | |
| `deepTalk` Deep conversation (talk, 40 min) | — | — | — | | |
| `hug` Hug (hug, 4 min) | — | — | — | A-social-02 S3 code (seeded) | |
| `flirt` Flirt (flirt, 10 min) | — | — | — | | |
| `sweetTalk` Sweet talk (flirt, 20 min) | — | — | — | | |
| `kiss` Kiss (kiss, 4 min) | — | — | — | A-social-01 S2 code (seeded) | |
| `askPartner` Ask to be partners (talk, 10 min) | — | — | — | | |
| `breakUp` Break up (argue, 10 min) | — | — | — | | |
| `insult` Insult (argue, 5 min) | — | — | — | | |
| `argue` Argue (argue, 20 min) | — | — | — | | |
| `prank` Prank (laugh, 8 min) | — | — | — | | |
| `fight` Start a fight (fight, 8 min) | — | — | — | | |
| `apologize` Apologize (talk, 10 min) | — | — | — | | |
| `tryForBaby` Try for a baby (kiss, 30 min) | — | — | — | A-social-01 S2 code (seeded) | |
| `goodbyeHug` Hug goodbye (hug, 3 min) | — | — | — | A-social-02 S3 code (seeded) | |
| `goodbye` Say goodbye (wave, 3 min) | — | — | — | | |

## Phone actions (5)

Played with the `phone` clip, sitting on a `lounge` seat within 5 m when one is free.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `text` Text (10 min) | — | — | — | I-social-01 S2 code (seeded) | |
| `call` Call (25 min) | — | — | — | I-social-01 S2 code (seeded) | |
| `invite` Invite over (5 min) | — | — | — | I-social-01 S2 code (seeded) | |
| `askOver` Ask to come over (5 min) | — | — | — | I-social-01 S2 code (seeded) | |
| `browse` Scroll the phone (20 min) | — | — | — | I-social-01 S2 code (seeded) | |

## Group conversations (6)

`conversation.rs` (`JOIN_RANGE` 2.6 m).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Joining in (walking over, taking a place) | — | — | — | | |
| Roles: speaker and listeners | — | — | — | | |
| Glances between members | — | — | — | | |
| Spacing and facing in the group | — | — | — | | |
| Talking seated (`sitTalk`) | — | — | — | | |
| Leaving a conversation | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-social-01 — A kiss between residents of different heights meets the forehead

- **Severity:** S2
- **Rubric:** A7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:430` (`EMBRACE`)
- **Evidence:** The taller one's mouth meets the shorter one's forehead. The taller one needs to bend and the shorter one to tilt the head up: two-body procedural work. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L118)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred)

### A-social-02 — Other children gather round a hug, right next to the pair

- **Severity:** S3
- **Rubric:** A7, I11
- **Fix class:** code
- **Where:** `crates/sim-core/src/conversation.rs:240` (where someone joining in stands; the cause is not traced yet)
- **Evidence:** Children nearby walk up to a hug and stand right against the two hugging, closer than anyone would.
- **Status:** seeded (from `docs/objects/open.md` L120)
- **Source:** `docs/objects/open.md` Still open › Residents

### I-social-01 — Phone actions are mimed: there is no phone in the hand

- **Severity:** S2
- **Rubric:** I12, I7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (`ACTIONS.phone`), `web/src/render/babylon/characters/clips/idle.ts` (`phone` clip)
- **Evidence:** The rubric records that no phone, book, cup or cutlery prop exists today. Props in hand are new behaviour: filed as `code` only, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §3.3 I7)
- **Source:** `plan/authenticity-review.md` §3.3 (I7) and §9 (decision 3)
