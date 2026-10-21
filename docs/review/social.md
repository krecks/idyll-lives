# Social interactions: authenticity ledger

What it covers: the 19 socials (`base.json` `socials`), the 5 phone actions (`base.json` `phone.actions`, from teens up) and group conversations (`crates/sim-core/src/conversation.rs`, up to 4). Poses and pairs: `web/src/render/babylon/characters.ts` (`EMBRACE`, face moods). Rubric I10–I12 and A7.

- **Evidence:** `bench-results/review/social/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Cross-cutting wave (2026-10-21):** pairs of every height on `review-social`, a copy of `review-stage` with the relationships set for each social (Ada 40 and Ben 22 a couple, Ada and Gus 70 in love, friends at 90, Ben–Cal, Cal–Dee and Gus–Ben enemies), and `review-social-fail` (each target can't stand the actor, so every roll fails); no command sets a relationship, so the states were edited (`bench-results/review/scripts/mkstates.py`). Three shots 0.9 s apart side-on to the pair once both stand still, and one from the other side (`social/pairs/<social>-<pair>-NN.png`, `-back.png`), with `__sims()` logged (`social/pairs/scenes.log`). Then `calling-out`, `group-talk`, `goodbye-hug`, `goodbye-wave`, the phone by stage on `review-stage` and on `calling-out` (text, call, invite). Contact sheets in `social/sheets/`. No social has a model: M is `n/a` throughout.

## Socials (19)

Label, animation and minutes from content.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `introduce` Introduce yourself (talk, 8 min) | n/a | OK | I-social-03 | I-social-03 S3 clip (the chat's talk, nothing of its own; not run: everyone in the review states has met, so judged from `chat`, the same clip) | `social/sheets/talks.png` |
| `chat` Chat (talk, 20 min) | n/a | OK | I-social-02 | I-social-02 S2 code (a failed chat differs only by the bubble and the listener's head shakes) | `social/pairs/chat-adult-elder-01.png`, `social/sheets/fails.png` |
| `joke` Tell a joke (laugh, 10 min) | n/a | OK | I-social-02 | I-social-02 S2 code (the one told a bad joke laughs along anyway) | `social/sheets/talks.png`, `social/pairs/fail-joke-ya-adult-01.png` |
| `compliment` Compliment (talk, 5 min) | n/a | OK | I-social-03 | I-social-03 S3 clip | `social/sheets/talks.png` |
| `deepTalk` Deep conversation (talk, 40 min) | n/a | OK | OK | | `social/sheets/talks.png` |
| `hug` Hug (hug, 4 min) | n/a | OK | I-social-02 | I-social-02 S2 code (a refused hug: the two stand still 0.6 m apart); A-social-02 S3 code (seeded; not seen) | `social/sheets/hugs.png`, `social/sheets/fails.png` |
| `flirt` Flirt (flirt, 10 min) | n/a | I-social-03 | I-social-02 | I-social-03 S3 clip (the talk with a head tilt); I-social-02 S2 code (a refused flirt looks like an accepted one) | `social/sheets/romance.png`, `social/pairs/fail-flirt-adult-elder-01.png` |
| `sweetTalk` Sweet talk (flirt, 20 min) | n/a | I-social-03 | OK | I-social-03 S3 clip | `social/sheets/romance.png` |
| `kiss` Kiss (kiss, 4 min) | n/a | A-social-01 | I-social-02 | A-social-01 S2 code (confirmed and widened: the arms hang at the sides); I-social-02 S2 code (a refused kiss: nothing happens) | `social/sheets/romance.png`, `social/pairs/fail-kiss-ya-adult-01.png` |
| `askPartner` Ask to be partners (talk, 10 min) | n/a | OK | I-social-03 | I-social-03 S3 clip | `social/sheets/romance.png` |
| `breakUp` Break up (argue, 10 min) | n/a | A-residents-07 | I-social-03 | I-social-03 S3 clip; A-residents-07 S2 code (residents ledger: no body language) | `social/pairs/breakUp-adult-ya-01.png` |
| `insult` Insult (argue, 5 min) | n/a | A-residents-07 | I-social-03 | I-social-03 S3 clip; A-residents-07 S2 code (residents ledger) | `social/sheets/talks.png` |
| `argue` Argue (argue, 20 min) | n/a | A-residents-07 | OK | A-residents-07 S2 code (residents ledger: a faster talk, head shakes and a slight lean; no anger in the body) | `social/sheets/talks.png` |
| `prank` Prank (laugh, 8 min) | n/a | OK | I-social-03 | I-social-03 S3 clip (the joke's laugh; no prank is played) | `social/sheets/fights.png` |
| `fight` Start a fight (fight, 8 min) | n/a | A-social-03 | A-social-03 | A-social-03 S2 clip (punches at the attacker's own height, over a child's head); I-systems-04 S2 code (systems ledger, widened: anyone may fight a child) | `social/sheets/fights.png` |
| `apologize` Apologize (talk, 10 min) | n/a | OK | I-social-03 | I-social-03 S3 clip | `social/pairs/apologize-teen-ya-01.png` |
| `tryForBaby` Try for a baby (kiss, 30 min) | n/a | A-social-01 | I-social-05 | A-social-01 S2 code (confirmed); I-social-05 S3 decision (a 30-minute standing kiss wherever the two meet) | `social/pairs/tryForBaby-adult-ya-01.png` |
| `goodbyeHug` Hug goodbye (hug, 3 min) | n/a | OK | OK | A-social-02 S3 code (seeded; not seen: the housemate stays 1.2 m off) | `social/sheets/goodbye.png`, `social/pairs/goodbyeHug-adult-child-01.png` |
| `goodbye` Say goodbye (wave, 3 min) | n/a | OK | OK | | `social/sheets/goodbye.png`, `social/pairs/goodbye-adult-elder-01.png` |

Seen across the pairs: talkers stand 1.0–1.6 m apart (a conversation's real distance), face each other, look at the other's head (the child tips the head back to an adult, `social/sheets/talks.png`), and the lips move. The hug follows every pair of heights: the taller bends over a child (`shoulderDrop`, `EMBRACE.bend`) and the arms go round the waist or shoulders (`social/sheets/hugs.png`: adult–child, elder–child, woman–man, young man–child). The faces and the reaction bubbles differ by outcome (`bubbles.ts` `iconFor`); the bodies don't (I-social-02).

## Phone actions (5)

Played with the `phone` clip, sitting on a `lounge` seat within 5 m when one is free.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `text` Text (10 min) | n/a | OK | I-social-04 | I-social-04 S2 clip (texting is a call at the ear; fixed: the phone in front, `phoneScreen`); I-social-01 S2 code (confirmed) | `social/scenes/phone-text-01.png` |
| `call` Call (25 min) | n/a | OK | I-social-01 | I-social-01 S2 code (confirmed: the hand at the ear holds nothing) | `social/scenes/phone-call-01.png` |
| `invite` Invite over (5 min) | n/a | OK | OK | I-social-01 S2 code (a call: at the ear is right; I-social-04 was filed here too) | `social/scenes/visit-*.png` |
| `askOver` Ask to come over (5 min) | n/a | OK | OK | I-social-01 S2 code (a call, at the ear; not run: `meet: there`, the same clip as `invite`) | `social/scenes/phone-call-01.png` |
| `browse` Scroll the phone (20 min) | n/a | OK | I-social-04 | I-social-04 S2 clip (scrolling is a call at the ear; fixed: the phone in front, `phoneScreen`); I-social-01 S2 code. The child is refused ("too young for a phone"), teen, adult and elder scroll | `social/sheets/phone.png` |

## Group conversations (6)

`conversation.rs` (`JOIN_RANGE` 2.6 m).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Joining in (walking over, taking a place) | n/a | A-social-04 | OK | Free will on after Ada and Ben start a chat (`review-group`): lonely Gus walks over and takes the third place within 2 s; Cal and Dee, as lonely, start a talk of their own instead. A-social-04 S3 code (the last metre or two into the place is a jump, `take_place`) | `social/sheets/group.png` |
| Roles: speaker and listeners | n/a | OK | OK | The one speaking talks, the other two listen with the idle and now and then a nod (`Anim.Talk`: `yes` on a good outcome) | `social/sheets/group.png` |
| Glances between members | n/a | OK | n/a | The speaker looks at the one talked to and now and then at a listener (`characters.ts:793`, `audience`); the listeners at the speaker. From stills only | `social/sheets/group.png`, `systems/scenes/group-top.png` |
| Spacing and facing in the group | n/a | OK | OK | Three in a triangle 1.0 m a side, each turned to the group's middle (`open_up`), on `review-group` and `group-talk` alike; A-social-02 S3 code (seeded; not seen) | `social/sheets/group.png`, `systems/scenes/grouptalk-00.png` |
| Talking seated (`sitTalk`) | n/a | n/a | I-social-06 | I-social-06 S3 code (nobody ever talks seated: a social stands both up) | `residents/strip-sitTalk.png` (pilot) |
| Leaving a conversation | n/a | OK | OK | When the chat ends the listener stays for the leader's next social with someone there or goes (Gus and Ben start a deep talk of their own; Ada goes to cook) | `systems/scenes/group-late-01.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-social-01 — A kiss between residents of different heights meets the forehead

> Cross-cutting wave: confirmed and widened. The arms hang at the sides through every kiss.

- **Severity:** S2
- **Rubric:** A7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:346` (`EMBRACE`), `:2654-2658` (`Anim.Kiss`: `idle` and a step in), `:3125-3129` (the kiss layer: a lean and a head tilt, nothing for the height or the arms)
- **Evidence:** The taller one's mouth meets the shorter one's forehead. The taller one needs to bend and the shorter one to tilt the head up: two-body procedural work. Deferred from 0.3.0. Cross-cutting wave: `social/pairs/kiss-adultF-yaM-back.png` (Ben 1.77 m, Ada 1.67 m): his mouth is at her brow, 10 cm apart in height is enough; `social/pairs/kiss-elder-adult-01.png` (Gus, stooped, and Ada) meet nearer the mouth. In every kiss, and the 30 minutes of `tryForBaby`, both stand with the arms hanging at their sides, touching only at the face (`social/sheets/romance.png`); the hug beside it already holds the other (`hugArms`). A kiss holds the other at the waist or the shoulders.
- **Status:** fixed (43ca1cc1, 2d887ff9)
- **After:** the kiss meets at the lips at any two heights (`KISS_FIT`: the taller bends and bows the head, the shorter lifts the face, with the look at the other's head), the taller's arms round the waist, the shorter's up round the shoulders: Ada 1.67 and Ben 1.77 both ways, Gus (elder) and Ada (`bench-results/review/fixes/clips-social/sheets/romance.png`, `pairs/kiss-adultF-yaM-00.png`, `pairs/kiss-yaM-adultF-01.png`, `pairs/kiss-elder-adult-01.png`; before `social/pairs/kiss-adultF-yaM-back.png`). `tryForBaby` plays the same kiss.
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred); wave: cross-cutting

### A-social-02 — Other children gather round a hug, right next to the pair

- **Severity:** S3
- **Rubric:** A7, I11
- **Fix class:** code
- **Where:** `crates/sim-core/src/conversation.rs:537` (`take_place`, where someone joining in stands; `GROUP_SPACING` 0.6 m at `:42`; the cause is not traced yet)
- **Evidence:** Children nearby walk up to a hug and stand right against the two hugging, closer than anyone would. Cross-cutting wave: not seen. A hug isn't `joinable`, so nobody joins it as a listener; in `goodbye-hug` the housemate stays 1.2 m off (`social/sheets/goodbye.png`), and in the hug pairs on `review-social` nobody came over.
- **Status:** not checked (not reproduced in the review states; the hug isn't joinable, so the cause is likely free will walking to the pair for something else)
- **Source:** `docs/objects/open.md` Still open › Residents

### I-social-01 — Phone actions are mimed: there is no phone in the hand

> Consolidation: the community wave saw the phone box's receiver stay on its hook while the caller holds a phone that isn't there to the ear.

- **Severity:** S2
- **Rubric:** I12, I7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:530` (`ACTIONS.phone`), `web/src/render/babylon/characters/clips/idle.ts:132` (`phonePose`)
- **Evidence:** The rubric records that no phone, book, cup or cutlery prop exists today. Props in hand are new behaviour: filed as `code` only, planned separately (§9, decision 3). Cross-cutting wave: every phone action, at every stage that may use one, holds an empty hand to the ear (`social/sheets/phone.png`, `social/scenes/phone-text-01.png`, `social/scenes/phone-call-01.png`); the same at the objects that use `phone` (the video-call nook, the shortwave radio, `systems/sheets/act-phone-meditate-idle.png`). The phone booth is the one place a receiver is there (its model).
- **Status:** confirmed
- **Source:** `plan/authenticity-review.md` §3.3 (I7) and §9 (decision 3); wave: cross-cutting

### I-social-02 — Success and failure look the same: a refused hug or kiss is two people standing still, and a bad joke is laughed at

- **Severity:** S2
- **Rubric:** I10
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2611-2665` (`choose`: the outcome is read only for `Anim.Talk`, as the listener's `yes` / `no`; hug, kiss, flirt and laugh play the same whatever it is), `:3110-3120` (the laugh layer on both of them); the outcome is in the snapshot already (`k.outcome`, `:2245`)
- **Evidence:** `social/sheets/fails.png` (`review-social-fail`, every roll failed): a refused hug, adult–child and teen–child, is the two standing 0.6 m apart with the arms down for the shortened 1.6 minutes, no step in and no step back; a refused kiss the same; a failed flirt is the accepted one. `social/pairs/fail-joke-ya-adult-01.png`: Ada, who can't stand Ben, tips her head back and laughs at his joke with him, under a "bad" bubble. Only the face (`EXPRESSIONS`) and the bubble (`bubbles.ts:167` `iconFor`) say it went badly. A refused hug is a step back with the hands up, a refused kiss a turn of the head, a flat joke a blank face.
- **Status:** fixed (43ca1cc1, 2d887ff9)
- **After:** the snapshot shows a hug's or a kiss's outcome from the start (the body answers it; talk still halfway, conversation.rs test `a_refused_hug_shows_at_once_a_chat_halfway_through`). A refused hug or kiss: the one asked leans away, the head turned aside, a hand up palm out, and steps back 0.14 m (`refuse`), the one who asked drops the shoulders and bows the head, a hand to the neck (`rebuffed`), with put-off and sad faces (`bench-results/review/fixes/clips-social/sheets/fails.png`: adult–child, teen–child, young adult–adult; before `social/sheets/fails.png`). A joke that falls flat: the listener folds the arms with a bored face, the teller shrugs (`bench-results/review/fixes/clips-social/sheets/flat-joke.png`, before `social/pairs/fail-joke-ya-adult-01.png`); one that lands is laughed at (`laugh`: the head back, a hand on the belly), the teller laughing along between turns. A flirt or sweet talk turned down: `rebuffed` and folded arms (`GESTURES`). Note: in `review-social-fail` the joke still lands on most tries (`joke` acceptance base 0.6); the flat one came from retrying, the teller walked elsewhere first, until the `bad` bubble showed.
- **Source:** wave: cross-cutting

### I-social-03 — Most socials are the chat's talk with nothing of their own

- **Severity:** S3
- **Rubric:** I10, A6
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:2617-2635` (`Anim.Talk`, `Anim.Laugh`, `Anim.Flirt`, `Anim.Argue` all `talk` / `idle` / `no`), `:3121-3124` (the flirt's head tilt); `web/public/content/base.json:1085` (`socials`: introduce, compliment, askPartner and apologize are `talk`, prank is `laugh`)
- **Evidence:** `social/sheets/talks.png`, `social/sheets/romance.png`, `social/sheets/fights.png`: introducing oneself, a compliment, asking to be partners and apologising are the same talk as a chat, with no handshake, no hand on the heart, no bow of the head; a prank is the joke's laugh with nothing played on the other; flirting and sweet talk are the talk with the head tilted; insults, an argument and a break-up a faster talk with head shakes. The face changes with each (`EXPRESSIONS`). Related: A-residents-07 (no body language for feelings, residents ledger). Small gestures per social (a handshake for `introduce`, a shrug and open hands for `apologize`, a point for `insult`) would tell them apart at the game camera.
- **Status:** fixed (43ca1cc1, 2d887ff9)
- **After:** each main social has its own looping body clip on the speaker's turn (`GESTURES`, `clips/social.ts`): compliment (an open hand offered, two light claps), deep talk (fingertips together, then the chin on the hand), flirt (a hand in the hair, the other on the hip), sweet talk (the hands at the heart, a reach to the other's arm), argue (palms up, a chopping hand), insult (a dismissive flick), apologize (a hand on the heart, then open palms), joke (the story in the hands, the arms thrown wide), introduce (a hand to the chest, then out to shake), ask to be partners (hands clasped, then held out), break up (a hand to the brow, both sweeping apart), prank (a creep and "boo!") (`bench-results/review/fixes/clips-social/sheets/talks.png`, `sheets/romance.png`; the bench sheets in the log; before `social/sheets/talks.png`). Chat keeps `talk`.
- **Source:** wave: cross-cutting

### A-social-03 — Fist fights are aimed at the attacker's own height: a teen punches over a child's head

- **Severity:** S2
- **Rubric:** A7
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:2636-2647` (`Anim.Fight`: `punchJab` / `punchCross` / `hit` at the clip's heights, a step in and back; nothing for the partner's height, as the hug has `shoulderDrop`), `:3144-3160` (the lean and the guard)
- **Evidence:** `social/sheets/fights.png`: Cal (teen, 1.51 m) against Dee (child, 1.09 m) throws his punches at his own chest height, a head above hers, from 1.2 m off; she guards a face nobody aims at. Young adult against teen and elder against young adult land near the face. Who may fight whom is I-systems-04's (systems ledger, widened: nothing stops a teen or an adult starting a fight with a child, `conversation.rs:63` `social_allowed` checks only the relationship and, for romance, age).
- **Status:** fixed (43ca1cc1, 2d887ff9)
- **After:** the attacker's punches are aimed at the other's head (`AIM`: the arms lowered or raised to it, the back bent over a shorter one): young adult on teen, adult on teen, elder on young adult (`bench-results/review/fixes/clips-social/pairs/fight-ya-teen-01.png`, `fight-adult-teen-01.png`, `fight-elder-ya-01.png`; before `social/sheets/fights.png`). Teen on child is no longer allowed (G-45).
- **Source:** wave: cross-cutting

### I-social-04 — Every phone action is a call: texting and scrolling hold the phone to the ear

- **Severity:** S2
- **Rubric:** I12
- **Fix class:** clip
- **Where:** `web/public/content/base.json:1912` (`phone.anim: "phone"`, one animation for all five actions); `web/src/render/babylon/characters.ts:530` (`ACTIONS.phone`: `phone`, upper `phone`); `web/src/render/babylon/characters/clips/idle.ts:132` (`phonePose`: the right hand at the ear, the left arm across the waist)
- **Evidence:** `social/sheets/phone.png`: "Scroll the phone" and "Text" are the call pose, hand to the ear, the head nodding as if listening, for 10 and 20 minutes; only `call` (and `invite`, `askOver`, short calls) fit it. The objects that use `phone` get the same: a video call at the nook's laptop is held to the ear in front of the screen, and so is the shortwave radio (`systems/sheets/act-phone-meditate-idle.png`). Texting and scrolling look down at a phone held in front at chest height, thumbs moving (the reading clip `lantern`'s arms are close). A per-action `anim` (content) and a "phone in hand" clip.
- **Status:** fixed (43ca1cc1, 2d887ff9)
- **After:** a phone action's own `anim` (content `phone.actions[].anim`; tests/phone.rs `texting_and_scrolling_look_at_the_screen_a_call_is_at_the_ear`): texting and scrolling play `phoneScreen`, the phone held in both hands before the chest, the head bowed, the thumbs tapping and swiping (`phoneText`; seated as the upper body); calls, invitations and asking over stay at the ear (`bench-results/review/fixes/clips-social/sheets/phone.png`, `sheets/phone-sofa.png`; before `social/sheets/phone.png`). The phone itself is still mimed (I-social-01, G-70).
- **Source:** wave: cross-cutting

### I-social-05 — "Try for a baby" is a 30-minute standing kiss, wherever the two happen to be

- **Severity:** S3
- **Rubric:** I16, I6
- **Fix class:** decision
- **Where:** `web/public/content/base.json:1678` (`tryForBaby`: `animation: kiss`, 30 minutes, no place)
- **Evidence:** `social/pairs/tryForBaby-adult-ya-01.png`: Ada and Ben stand kissing in the front garden for half an hour, the children a few metres off. Trying for a baby needs a place and privacy (a bed, a closed door). For the owner: keep it a kiss (the game's tone), or make it a use of a double bed behind a door, which is new behaviour (`code`).
- **Status:** open
- **Source:** wave: cross-cutting

### I-social-06 — Nobody ever talks seated: a social stands both of them up

- **Severity:** S3
- **Rubric:** I11, I1
- **Fix class:** code
- **Where:** `crates/sim-core/src/conversation.rs:756-776` (`engage`: `end_activity` on the target), `crates/sim-core/src/world.rs:5289-5300` (`end_activity`: back to standing); the seated clip exists (`characters.ts:2581`, `sitTalk`)
- **Evidence:** Socials happen only standing: the one talked to drops what they do and stands, and a conversation can't start from a seat, so two on a sofa or a family at the dinner table never talk, and `sitTalk` (`residents/strip-sitTalk.png`, pilot) is seen only on `poses.html`. People talk most sitting down: over a meal, on the sofa, at the café table.
- **Status:** open
- **Source:** wave: cross-cutting (code read; every pair run stood up first)

### A-social-04 — Someone joining a group jumps the last metre or two into their place

- **Severity:** S3
- **Rubric:** A4, I11
- **Fix class:** code
- **Where:** `crates/sim-core/src/conversation.rs:476-483` (the joiner stops once within `JOIN_RANGE` 2.6 m of the pair's middle) and `:537-575` (`take_place`: `sims[j].pos = p`, the chosen spot set at once)
- **Evidence:** The joiner walks until within 2.6 m of the middle of the two talking, then is put on a spot about 1 m from each of them in one tick, up to 1.5–2 m away, and the renderer slides them there. `review-group` (`systems/scenes/scenes.log`): Gus covers 3.4 m in the 1.6 s between two shots at speed 1, about a metre more than a walk does. The shots are too far apart to see the slide itself (a timed, headed check would). Route the joiner to the spot instead.
- **Status:** open
- **Source:** wave: cross-cutting (code read, with the positions logged)
