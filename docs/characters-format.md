# Residents: the character format

The residents (bodies, faces, hair, clothes) are generated in code by `cargo characters` (in Rust,
`crates/tools/src/characters`), and their animation clips are made in code in
`web/src/render/babylon/characters/clips`. This page is the contract between the generator, the
clips and the runtime (`web/src/render/babylon/characters*`). Everything here is original work.

```
cargo characters build                                # writes web/public/assets/characters, then checks it
cargo characters check [set dir]                      # the set against this contract
cargo characters diff <a> <b>                         # two sets, within the quantisation
node --experimental-strip-types tools/characters/check_clips.ts   # the clips against the rig (see Checking clips)
cd web && pnpm check:clips                            # the same on both bodies at every life stage (CI)
```

The build is deterministic (seeded noise only): the same code gives the same bytes.

## Conventions

- Babylon's left-handed space, metres, y up. A resident faces +z; their left is -x.
- Every rest local rotation is the identity. A bone's frame at rest is model space, and its local
  translation is the offset from its parent's joint. `restQ`, `pre`, `post` and `sourceRestQ` are
  absent or identity.
- The rest pose is an A-pose: arms 45 degrees down, legs straight and vertical, a gently curled
  open hand.
- Meshes are written with clockwise front faces (Babylon's left-handed convention). Inside the
  generator every mesh is counter-clockwise from outside; `write.py` turns the faces round.

## Skeleton (33 bones)

| # | Bone | Parent |
|---|---|---|
| 0 | root | - |
| 1 | pelvis | root |
| 2-4 | spine_01, spine_02, spine_03 | chain |
| 5 | neck_01 | spine_03 |
| 6 | Head | neck_01 |
| 7-14 | clavicle_l, upperarm_l, lowerarm_l, hand_l, thumb_01_l, thumb_02_l, fingers_01_l, fingers_02_l | spine_03, chain; thumbs and fingers off the hand |
| 15-22 | the same on the right (`_r`) | |
| 23-26 | thigh_l, calf_l, foot_l, ball_l | pelvis, chain |
| 27-30 | thigh_r, calf_r, foot_r, ball_r | pelvis, chain |
| 31-32 | eye_l, eye_r | Head |

- `clipBones` is 31: clips store bones 0-30. The eyes come last and are turned procedurally
  (`addLocal`; +x turns the front down, +y turns it to the resident's right).
- The runtime looks these up by name: `pelvis spine_01 spine_02 spine_03 neck_01 Head clavicle_* upperarm_* lowerarm_* hand_* thigh_* calf_* foot_* eye_*`.
  Fingers match `/^(index|middle|ring|pinky|thumb|fingers)_0(\d)/`, legs `/^(thigh|calf|foot|ball)_/`.
- `hand_*` is the wrist joint. `grip` (optional) is the left palm centre relative to the wrist; the
  right mirrors x.
- Proportions per family and life stage are in `rig.rs` (`proportions`). The stage heights stay
  within 3% of `target_height` (what cribs, kids' beds, desks and seats were fitted to).
- The reference skeleton for the clips is the male young adult: `sourcePelvis` is its pelvis rest
  position, and `reference` gives its body name and its thigh and shin lengths (for leg IK).

## rig.json (version 4)

| Key | Meaning |
|---|---|
| `version` | 4 |
| `bones`, `parents`, `clipBones`, `head`, `face.bones` | the skeleton above (parents come before children) |
| `fps` | 30 (clip baking) |
| `sourcePelvis`, `sourceRestQ` | the reference pelvis; identity rotations (`NB * 4`) |
| `reference` | `{ body, thigh, shin }`: the clips' reference body and leg lengths |
| `fieldScale`, `toneScale`, `stageScale` | int16 per unit for fields and tone, per metre for stage deltas (4000, 4000, 5000) |
| `morphs` | `{ channels, scale: 30000, normalScale: 63 }` |
| `anims`, `clips` | optional baked clips; absent: the runtime bakes `clips/` at load |
| `bodies.<family>` | `male`, `female` (below) |

A body entry:

| Key | Meaning |
|---|---|
| `mesh` | `<family>.bin` |
| `stage` | `youngAdult` (the base stage) |
| `height` | top of the head (m); the base is drawn at `BASE_HEIGHT`, so its scale is the resident's height factor |
| `restT`, `inverseBind` | per bone: local translation (`NB * 3`), inverse bind matrix (`NB * 16`, column-major) |
| `pelvisScale` | pelvis height / the reference pelvis height (scales clip pelvis motion) |
| `seatDrop` | seated, the pelvis joint's height above the seat surface (m) |
| `eyeY` | the eye centres' height (m) |
| `grip` | see above (optional) |
| `skinRef` | `[1, 1, 1]` |
| `garments` | id -> cut, every id of the family in the creator's order (see Clothes) |
| `bodyChunks` | `{ "body": [] }` (clothes are painted, skin is never hidden) |
| `parts` | part name -> byte offsets in `mesh` |
| `stages` | other life stages: `{ mesh, height, restT, inverseBind, pelvisScale, seatDrop, eyeY, grip, parts }` |

No textures: `textures`, `cloth`, `hairTextures`, `hairCards` are gone, and so are UVs.

## Binaries

`<family>.bin`, per part (offsets 4-byte aligned):

| Array | Type |
|---|---|
| `positions` | f32 x3 |
| `normals` | i16 x4 (/ 32767; w unused) |
| `joints` | u8 x4 |
| `weights` | u8 x4, summing to 255 |
| `fields` | i16 x4 (/ fieldScale) |
| `tone` | i16 x4 (/ toneScale) |
| `index` | u16 x3 per triangle (clockwise) |
| `morph` (optional) | `count`, `ranges` (channel c: `ranges[c] .. ranges[c + 1]`), `morphVerts` u16 (the moved vertices), `morphSlots` u16, `morphPositions` i16 x4 (/ scale), `morphNormals` i8 x4 (/ normalScale) |

`<family>.<stage>.bin`: per part, `positions` i16 x4 deltas from the base (/ stageScale) and
`normals` i8 x4 (/ 127). Every part has the same vertices at every stage; fields, tone, weights,
indices and morphs come from the base.

The runtime interleaves field, morph slot and tone into one vertex buffer (`simField` 0,
`simMorph` 4, `simTone` 8; stride 12 floats; parts no morph moves get slot -1), so a draw uses 7
vertex buffers (position, normal, joints, weights, that one, the instance matrices and settings)
of WebGPU's 8.

## Parts

Every body has:

| Part | What | Skinning | Material |
|---|---|---|---|
| `body` | the skin, with the eyelids and lips (from the face) | body bones; lids and lips Head | `body`: skin colour, painted clothes |
| `lod.body` | the low-detail body (far residents, every shadow), with the eyelids (they blink) | body bones; lids Head | `body` |
| `eyes` | two eyeballs | `eye_l`, `eye_r` | `eye` (procedural iris) |
| `lashes`, `brows` | lashes on the upper lid rims; brows | Head | `lash` (dark), `brow` (hair colour x 0.45), opaque |
| `teeth`, `tongue` | teeth; the mouth plate (dark, tone.y = 1) and tongue | Head | `mouth` |
| `hair.<style>` | every hairstyle but `none` (`short crop layered tousled sideSwept afro bob long ponytail braid bun`) | Head, blending to neck_01 / spine_03 below the jaw | `hair` |
| `hair.beard` | facial hair (male only; hidden for baby, child, teen) | Head; follows the mouth's morphs | `hair` |
| `top.*`, `bottom.*`, `shoes.*` | shells, only where the silhouette needs one: jacket and suit (torso and upper arms, open in a V; the sleeves are painted), the collars of polo and shirt, the skirt, every shoe's soles and the boots' cuffs | from the body | `cloth` (flat colour) |
| `lod.<shell>` | a low-detail copy of every shell | | `cloth` |

`hair.none` has no part: a bald resident's scalp keeps the skin colour (appearance texel 7 w = 0).

The body (`body.rs`) is a control cage of superellipse rings (`cage.rs`; the head's cage
in `head.rs`) fitted so its limit surface passes through the ring points and subdivided once
(`subdiv.rs`): `body` is about 3.5k triangles of skin plus the face's lids and lips (budget
4.5k), `lod.body` the cage at its limit plus the lids (budget 1.2k). Skin weights are set per ring
and subdivide with the positions. Every stage of a family has the same cage.

The head interface (`head_frame(family, stage)`, `head.rs`): `sdf` is a smooth signed distance
to the head and neck as drawn (the ears left out; about 1 mm near the surface), `surface_z` the
front surface's z; `ears` boxes round the ears (hair goes round or over them); `centre` / `radii` a
loose ellipsoid round the skull (with a margin; fit close things to `sdf`); eyes, mouth, brows,
hairline and chin as before.

## Fields (4 per vertex)

| Part | x | y | z | w |
|---|---|---|---|---|
| body | sleeve parameter (-1 torso, 0 shoulder .. 1 wrist, > 1 hand) | height (m, base body) | crew neckline (m, > 0 covered: distance from the neck's axis plus depth below a curve round the neck's base); head: -1 - scalp weight | scoop neckline (women: round and low in front; men: a V); head: -1 - scalp weight |
| shells | hem distance (m) | flag: 1 sole, 2 waistband, 3 skirt | category: 0 top, 1 bottom, 2 shoes | 0 |
| eyes | offset from the iris centre on the eye's front (m), x | y | iris radius (m) | pupil radius (m) |
| hair | 0.1 (unused) | | | |

## Tone (4 per vertex)

x = lip / cheek blush (0..1; on the tongue part, the tongue's red), y = cavity darkening (armpits,
between fingers, under the chin, between the lips, deep in the hair; the mouth plate is 1), z = hair
clump bands (0..1), w reserved. The eyelids take the tone of the skin they lie on (build.rs). Every
stage uses the base stage's tone; the shader deepens the cavities with the resident's age
(appearance texel 4 g).

The jacket, suit and skirt shells use their tone's xyz for their seated shape instead (m, in the
rest space, before skinning; w 0): what moves each vertex from where a seated pose (`seated_pose`
with the thighs turned 76° forward, as `sit` has them) skins it to where its seated shape is skinned
(the skirt without its flare, its hollows between the legs bridged, the jacket without its hem's
flare), out of the seated skin and no lower than the seat (`seated_tone`, clothes.rs). The runtime
adds it as far as the thighs are turned forward (`writeSit`: from 15° to 76°, the mean of both, at
the body's size; appearance texel 4 b), so a seated skirt lies on the lap and the jacket's hem on it
instead of standing out round the seat as they hang standing.

## Clothes

Clothes are painted on the body by its shader. Each garment's cut (metres on the base body's
fields) decides where it covers:

- top: `(sleeve - x) * 0.55 > 0`, `y > hem`, `z > crew` (if set), `w > scoop` (if set);
- bottom: `y < waist`, `y > leg`, `x < -0.5` (not on the arms);
- shoes: `y < shoe`.

A top with `over: true` (its hem is below the waistband: shirt, jacket, suit) is painted over the
bottom; otherwise the bottom wins. Shoes win over both. Edges are anti-aliased; painted cloth is
rougher than skin. Shoes have a `colour` (sRGB hex). Skirts set `skirt: true` and their `leg` cut
paints the thighs (the shell hangs over them). Babies don't wear skirts (the shell can't follow
their drawn-up, spread legs): a baby whose outfit has one wears `bottom.shorts` in the top's colour,
a romper (outfit.ts).

What a resident wears changes with what it does (`Wear`, outfit.ts; characters.ts `wearFor`): its day
clothes; pyjamas for a night's sleep in bed (a `sleep` use lying down: the `top.long` and
`bottom.trousers` cuts in a sleep colour, barefoot); the day clothes barefoot for a nap on something
slept in; a towel under a shower and sat in a bath (the bottom's cut alone, from under the arms, or
round a man's hips, to the thigh); swimwear in a hot tub (a swimsuit, the `top.tank` cut down to the
top of the legs, or trunks). Only the day clothes draw their shells: the others leave them out by the
resident's row (`bare`), so a change of clothes draws nothing more. The towel's and swimwear's heights
come from the base body's joints at rest (`OutfitFit`: under the arms half way between `spine_03` and
`upperarm_l`, the hips and the knees).

Garment ids per family (outfit.ts):

- male: `top.tee top.vneck top.polo top.long top.shirt top.jacket top.suit`, `bottom.trousers bottom.shorts bottom.suit`;
- female: `top.tee top.tank top.blouse top.long top.shirt top.jacket`, `bottom.trousers bottom.shorts bottom.capri bottom.skirt`;
- both: `shoes.sneakers shoes.trainers shoes.boots shoes.dress`.

## Appearance texels (per resident, after the bone matrices)

| Texel | rgb | w |
|---|---|---|
| 0 | skin colour (linear) | top: sleeve cut |
| 1 | top colour | top: hem |
| 2 | bottom colour | top: crew cut |
| 3 | hair colour (plain) | top: scoop cut |
| 4 | r = 1 if the top is `over` the bottom; g = age (0 young, 0.4 adult, 1 elder); b = how far the shells take their seated shape (0..1, at the body's size; below) | bottom: waist |
| 5 | skin reference (1) | bottom: leg cut |
| 6 | shoe colour | shoes: top height |
| 7 | eye colour | 1 if the resident wears hair (the scalp takes the hair colour) |

Then 12 texels of face morph (channel, weight) pairs. Disabled cuts are -10 (+10 for "below" cuts).

`appearance.build` (0.85 slim .. 1.2 broad; missing = 1) widens bones across their axis in
`skin()`: `K = W * (I + (g - 1)(I - a a^T)) * IBM`, the torso by g, the neck by half and the limbs
by 60% of it. The simulation keeps the appearance opaque, so old saves are unaffected.

## Morph channels

`morphs.channels` is the 15 visemes in the voice engine's order, then 43 ARKit units:

- visemes: `sil PP FF TH DD kk CH SS nn RR aa E I O U`;
- units: `browDownLeft browDownRight browInnerUp browOuterUpLeft browOuterUpRight cheekSquintLeft cheekSquintRight eyeBlinkLeft eyeBlinkRight eyeLookDownLeft eyeLookDownRight eyeLookUpLeft eyeLookUpRight eyeSquintLeft eyeSquintRight eyeWideLeft eyeWideRight jawForward jawOpen mouthClose mouthDimpleLeft mouthDimpleRight mouthFrownLeft mouthFrownRight mouthFunnel mouthLeft mouthLowerDownLeft mouthLowerDownRight mouthPressLeft mouthPressRight mouthPucker mouthRollLower mouthRollUpper mouthShrugLower mouthShrugUpper mouthSmileLeft mouthSmileRight mouthStretchLeft mouthStretchRight mouthUpperUpLeft mouthUpperUpRight noseSneerLeft noseSneerRight`.

`Left` is the resident's left (-x). Units are authored at ARKit strength. The head itself never
deforms: channels move the lids, lashes, brows, lips, mouth plate, teeth, tongue and beard. Morphs
are built at the base stage and shared by the other stages.

## Clips

`clips/dsl.ts` defines clips as key poses (`clip`) or functions of the phase (`cycle`), with
layers (`layer`), leg IK (`legIK`) and mirroring; `clips/index.ts` lists them and bakes them into
the runtime's `Clip`s at load (30 fps; loops sample `i / frames`, one-shots get one more frame).

- A pose names bones with Euler angles in degrees, `[x, y, z]`, applied about x, then y, then z of
  the parent's frame (model axes at rest). The signs match `addRotation`: -x on a thigh or upper
  arm raises it forward, +x on a calf bends the knee, -x on a forearm bends the elbow, +z on the
  left upper arm brings it down to the side.
- `thigh` sets `thigh_l` and the mirrored `thigh_r` (x, -y, -z); `thigh_r` sets one side and wins.
  Bones a pose doesn't name rest. `move` is the pelvis offset from rest (m, reference skeleton).
- Clips hold fists (`GRIP`); the runtime opens the hands towards the rest (open) pose per activity.
- Clips start from the relaxed stance (feet under the hips, upright, arms hanging loose); the
  runtime adds no stance corrections of its own (only the arms a little out over a skirt and an
  elder's stoop), so a clip looks in game as it was authored.
- Seated clips share `sit`'s pelvis. The runtime fits it to each body and seat: the pelvis joint
  goes `seatDrop` above the seat's surface and at the seat's forward offset (`SEATS`, from
  characters/furniture.json, or the content's `seat`). It fits the legs too (`characters/seatFit.ts`, shared with
  `check_clips.ts`): each ankle down to the floor under the knee, the thigh sloping off the seat's front by up to
  0.3 rad, past that the feet hanging (a child on a grown-up's chair), the knees coming up on a low seat; on a deep
  seat (`front`) the hips move forward until the knees clear its front (`perch`). Relaxing, the feet go 12 cm
  further out. A seat under 0.2 m is the floor's (`floorSit`): sat on by the floor clips, cross-legged on a cushion
  (`sitFloor`) or on the ground against a tree's trunk (`sitGround`), each with its own sitting down and getting up
  and its own pelvis. Lying plays `lie` turned flat on the mattress (`LIE_*`);
  the blanket is fitted to `lie` too. A baby lying (in the crib) plays its own clips the same way,
  `babyLieAsleep` or `babyLieAwake`, centred on the mattress.
- Carried things sit on the palms (`grip`): the right one, or between both.
- Every name the runtime plays, with its loop flag and length (seconds; the runtime sequences
  object use, sitting down and standing up on these lengths):

| Clip | Loop | s | Played for |
|---|---|---|---|
| idle | yes | 2.5 | standing about, and object use that needs no hands |
| talk | yes | 2.93 | talking, singing (with `phone` on top), the mirror |
| walk | yes | 1.1 (1.1 m/s) | moving (played at the ground speed) |
| jog | yes | 0.7 | running on the spot, treadmills, exercise |
| jogAlong | yes | 0.7 (2.5 m/s) | moving fast: the jog's pose given a stride, the hips 6 cm lower (the knees bent through the stance), played at the ground speed when the walk would need more than 3× its own pace (`GAIT_MAX_RATE`); back to walking below 0.8 of that (`JOG_BACK`) |
| runAlong | yes | 0.6 (5.8 m/s) | moving faster still (game speed 3 and up, children from 2): a run with a 3.5 m stride, each foot down a sixth of the time, played at the ground speed when the jog would need more than 3×, up to 4× (`RUN_MAX_RATE`); back to the jog below 0.8 of the switch |
| sit | yes | 1.67 | seated |
| sitTalk | yes | 2.93 | seated in a conversation |
| sitDown | no | 1.33 | sitting down, and onto the edge of a bed before lying down |
| standUp | no | 1.07 | getting up from a seat or out of bed |
| sitFloor, sitDownFloor, standUpFloor | yes, no, no | 1.67, 1.6, 1.47 | on a floor cushion (a seat under 0.2 m: the floor cushion, the electric fireplace's hearth cushion): cross-legged, the hands on the knees; down to it through a squat and a hand on the floor, and up again (clips/seated.ts `FLOOR_SITS`) |
| sitGround, sitDownGround, standUpGround | yes, no, no | 1.67, 1.6, 1.47 | on the ground (a seat under 0.07 m: a tree's shade) against the trunk, leant back, the legs out long, crossed at the ankles, the hands in the lap |
| sitHands | yes | 1.67 | seated at a desk or keys (type, write, play, music) |
| sitHandsNear | yes | 1.67 | the same at a shallow desk with its chair built in close (`NEAR_DESKS`) |
| sitHandsFar | yes | 1.67 | the same reaching further out, leaning in more: at a desk or table a chair can't be pulled in to far enough (drawer pedestals where the knees go: `TUCK`) |
| interact | no | 2.0 | reaching to use something |
| pickUp | no | 0.87 | lifting, serving, fetching |
| spell | yes | 2.1 | hands busy at a counter (cook, wash, paint, tinker) |
| chop, stir, flip | yes | 1.6, 2.0, 2.0 | the cooking steps of those names |
| eat, drink | no | 1.33, 2.0 | eating and sips (standing, and now and then as the seated upper body) |
| carry, carryOne | yes | 1.1 | the upper body while carrying something in both hands / on the right palm (`held`, content `hold`) |
| lie | yes | 2.4 | lying (turned flat by the runtime, played at a third of the speed) |
| babyLieAsleep, babyLieAwake | yes | 9, 6 | a baby lying in the crib, asleep (still, quick breaths, a twitch) or awake (kicking, waving, turning the head); legs drawn up, hands by the head; turned flat like `lie`, played at full speed |
| foldArms | yes | 2.5 | watching, relaxing, idle variations |
| yes / no | no / yes | 2.5 | listening, arguing |
| phone, lantern | yes | 2.93, 2.5 | the phone; reading (a book at the chest) |
| rail | yes | 2.5 | leaning on a rail (the older layouts' telescope, `USE`) |
| eyepiece181, 163, 147, 129, 112 | yes | 6 | bent down to the telescope's eyepiece (the tube aimed up: the eye at 0.765 m, 0.3 m out), the right hand on the tube, the left on the knee, a look up at the sky each loop; a size per height (cm, `STEP_IN`, for `look`) |
| pickApple177, 147, 112; pickLemon177, 147, 112 | no | 3.3 | reaching up to the fruit tree's picked fruit (the apple 1.84 m up, the lemon 1.62 m), on tiptoe if high, the left hand on the bough, a twist off and down to the chest; then `eat` (`STEP_IN`, for `eat`); a size per height |
| shower, showerChild | yes | 14 | showering: the hair lathered and rinsed, a 50° turn in two steps to soap the arms and the chest, back, the face rinsed; a child (under 1.29 m, `SHOWER_CHILD`) turns only the chest |
| strum, strumShort | yes | 2 | a guitar (or a ukulele, `strumShort`, held higher) across the body: the neck out to the left 30° up in the body's plane, the left hand on it, the right strumming; `writeHeld` draws the instrument from its rest point (`GUITAR`) to the left palm |
| push | yes | 2.67 | mopping, vacuuming |
| water, harvest, plant, kneel, crouch | yes, yes, no, yes, yes | 3.8, 2.5, 2.77, 5.2, 2.93 | gardening, smelling flowers |
| pick | yes | 3.0 | picking herbs from a waist-high planter |
| dance | yes | 1.0 | dancing, fitness games |
| punchJab, punchCross, hit | no | 0.87, 1.0, 0.33 | heavy bags and fights |
| wave | no | 2.2 | the household creator's greetings |
| fountainDrink181, 163, 147, 129, 112 | yes | 3 | bent over a drinking fountain's spout, sipping from its water (a size per height, in cm; the smaller two on tiptoe at the water's falling end); `STEP_IN` steps them up to it |
| climb172a, 147a, 112a; climb172b, 147b, 112b | yes | 13 | a round on the climbing frame: up its rope net three ropes, hands and feet in turn, over the edge onto the deck, ducking under the roof across it to the fireman's pole, round the pole and down it (hands and legs round it), and back round to the net; a size per height (cm), and one per lane of the net (`a`, `b`: the second waits on the deck for the first to slide; one stepping in while someone goes round waits at their lane until that round is back at its start, so the two stay in step: `inTurn`) |
| dartThrow | yes | 7.2 | darts (`ACTIONS.play` at the `dartboard`): side-on at the line, three throws (aim, the forearm back, through, the hand following to the board), a step back to read the score and in again; the first dart goes as the board's darts grow in |
| pingPong | yes | 1.8 | table tennis (`ACTIONS.play` at the `pingPongTable`): the ready stance, a forehand and a backhand, the bat mimed in a closed hand |
| poolShot172, 147, 112 | yes | 8 | the pool table (`STEP_IN`, a lane each for two): bent over the long rail as far as the height needs, the left hand bridging on the cloth, two practice strokes and the shot, then up to chalk the cue (mimed); a size per height (cm) |
| foosball172, 147, 112 | yes | 3.2 | the foosball table (`STEP_IN`, a pair of rods each for two): hands on two handles, the wrists turning the rods (the right in time with the table's moving rods) and sliding them; a size per height |
| smashPlate172, 147, 112 | yes | 3.4 | the plate-smash bin (`STEP_IN`): a plate taken from the stack (mimed), up over the right shoulder and thrown overarm down into the crate; a size per height |
| boulder172, 147, 112 | yes | 16 | the bouldering wall (`STEP_IN`, up on its crash mat): hands to the start holds, the feet onto the low ones, up the wall's holds hand and foot in turn to the top, a drop to the mat onto bent knees; the route scaled to the size and put on the model's nearest holds |
| ballPit112 | yes | 12 | the ball pit (`STEP_IN`, over its rim, `over`): wading and a jump in the balls, down to sit in them tossing balls up, up again; a child's size (clips/play.ts) |
| tentPlay112 | yes | 24 | the play tent (`STEP_IN`): down on hands and knees at its door, crawling in, sitting back on the heels to play on its mat, crawling back out; a child's size |
| yoga | yes | 48 | a yoga flow on the mat (`STEP_IN` onto its front end, open hands): mountain, hands to the heart, upward salute, forward fold, half lift, a short downward dog (heels pedalling), a lunge up to warrior I, back to the dog, a child's pose, sitting on the heels, up through a squat |

### Checking clips

`tools/characters/check_clips.ts [rig.json] [--body male|female|all] [--stage <stage>[,…]|all]
[--update-baseline]` bakes the clips as the runtime does and checks loop flags, NaNs, loop seams,
lengths, sliding feet, the seated pelvis and feet (with the runtime's leg fit, `seatFit.ts`: on a 0.45 m seat, a
child's on the kids' chair, and the knees clear of every deep seat's `front`), the floor sits' knees and ankles
against the floor, `lie` and the baby's clips (the list is in its header). The default is the reference: `male` at
`youngAdult`, plus `baby` for the baby's clips.
Each body and stage plays the clips on its own skeleton (`restT`, the pelvis motion and ground
speed scaled by `pelvisScale`, `walk`'s hip roll), the sized clips at the size its height picks,
babies only their own; the height slider, build, head size, stoop and the runtime's fits to seats
and beds other than the seated legs' are not covered. Planted feet are checked on each skeleton as it is, so a clip that turns the
hips over a planted foot slides on hips wider for their legs than the reference's (a child's, a little a
woman's): place the feet for them (`footIK`'s `wide`, a size per height) or keep the hips still. The known failures are in `tools/characters/clip-baseline.json`: they
print as `known` and don't fail, one that no longer fails prints as `fixed` (take it out), and only
a new one exits 1. `--update-baseline` rewrites the file for the bodies and stages checked. CI runs
`pnpm check:clips` (in `web/`: `--body all --stage all`).

In dev, editing a clip file re-bakes the clips in place (Vite HMR).

## Seats and beds

`web/src/render/babylon/characters/furniture.json` holds where residents sit and lie, by object id: `seats`
(`SEATS`), `defaultSeat`, `beds` (`BEDS`), `defaultBed` and `bunks` (`BUNKS`). `characters.ts` reads it with its
`Seat` and `Bed` types (the fields are documented there); `cargo check-glb` measures the meshes against it and
`cargo review sizes` against real-world sizes. An object in `seats` takes that seat over the content's `seat`
(`seatOf`); one in neither sits at `defaultSeat`. Heights are m above the floor, `fwd` m forward of the footprint's
middle, `recline` in radians.

Why some entries are as they are:
- Chairs at desks and tables (`chessTable`, `computerDesk`, `diningTable`, the 0.20 dining sets and desks) sit in the
  back half of the footprint, facing the work surface. The pack writing desks (`gloomy.journalDesk`,
  `neat.organizerDesk`, `romantic.letterBureau`, `hotHeaded.grudgeJournal`) are their own chairs (`NEAR_DESKS`): the
  hands on the top or the flap, in front of the hutch.
- `fireplaceElectric` sits cross-legged on the floor cushion in front of the fire, facing it (`back`), the hips on its
  back half (0.36 m out) so the knees stop short of the surround.
- Deep seats (the sofas, sectionals, sofa bed, armchairs, ottoman, bean bags and four pack seats) give `front`:
  where the seat ends in front, m forward of the footprint's middle as `fwd` (`cargo check-glb` measures it: from
  the hips forward while the model reaches within 6 cm of the seat's height, and warns past 3 cm). The hips sit
  forward until the knees clear it (`perch`). The content-seated ones (`sofaThreeSeat`, `sofaBed`, the sectionals,
  `armchairWingback`, `ottoman`, the bean bags) copy their content `seat` into the row; the recliners, whose
  content seats put the legs out, keep theirs (no `perch` with the legs out).
- Trees (`tree`, the seven `garden.*` trees with "Relax in the shade"): on the ground (`h` 0, not measured: nothing
  of the model's is sat on) against the trunk, `fwd` the hips' distance from it, `shade`: facing out on the trunk's
  front or a side, whichever the sun's light falls towards as they sit down (`shadeSide`; the renderer's sun).
- Baths (`bathtub`, `bathtubClawfoot`, `bathtubFreestanding`): on the tub's floor, along the tub, the back at the
  end without the taps, the legs out along it towards them (`along`, `legsOut`).
- The pack tubs (`gloomy.soakingTub`, `romantic.heartTub`, `hotHeaded.coldPlunge`): on the floor at the back, the
  legs out with only `room` m for them ahead of the hips, so the heels stop inside the front wall and the knees
  come up.
- `gardenChair`: on its slats' tops (0.43), against its slanted back, `recline` 0.314 rad (18°; the Adirondack
  chair's content `seat.recline` is in degrees).
- `chessTable`, `sewingTable`, `hotHeaded.thunderDrums`: their chairs' and the drum stool's pads (0.475).
- `weightBench`: lifting, the feet down on the floor either side of the bench, `feet` m out from its middle
  (`benchLegs`).
- `kidsBedCar`: the race car's mattress ends at its bonnet, 34 cm short of the footprint's end (`foot`).
- Hammocks (`hammock`, `cheerful.rainbowHammock`, `natureLover.gardenHammock`): lain in the sagging sling (`sling`,
  from their models' builds).
- `bedDouble`, `bedQueenUpholstered`, `bedCanopy`: their mattress tops; `bedDaybed` its daybed top.
- `sofaBed`: the head on the seat, the rest on the mattress pulled out in front (`bed_out`, `fwd`).
- `gloomy.windowSeat`, `kind.hearthSettee` (2 wide): dozed on along the cushion, head to its left, whichever slot
  (the nap takes both, content `whole`).
- `bunks.bedBunk`: the lower and the upper mattress tops, `[lower, upper]` (content `bunk`).

## Where things are

| What | Files |
|---|---|
| the contract | `rig.rs`, `head.rs` (the head interface), `contract.rs`, `write.rs`, `build.rs`, `check.rs`, `check_clips.ts`, `clips/dsl.ts`, this page |
| body and clothes | `body.rs`, `cage.rs`, `subdiv.rs`, `clothes.rs`, `garments.rs`, `meshops.rs` |
| face and hair | `face.rs`, `hair.rs`, `prims.rs`; the eye, hair and brow branches of `characters/material.ts`; `characters/face.ts` |
| clips | `clips/*` (with `clips/ik.ts`) |
| the runtime | `characters.ts` (activities, seats, beds, carrying, faces), `characters/{data,pose,material,outfit,blanket}.ts`; the creator in `ui/screens/CreateHousehold.svelte` and `render/preview` |

See also `tools/characters/README.md`.

## Dev benches

- `/characters.html`: a grid of residents. `?family=male|female`, `?stage=<id>|all` (default: every
  stage), `?hair=all`, `?tops=all`, `?skins=all`, `?builds=all`, `?face=moods|visemes`,
  `?clip=<name>&t=<s>`, `?view=side|back`, `?closeup=1`, `?build=1.2`, `?cols=`, `?manual`
  (`__bench.advance(s)`); sets `window.__ready`.
- `/poses.html`: residents at real furniture, seats and beds, cooking, carrying, socials.
  `?only=<labels>` picks stations, `?seats=sofa,toilet` seats a man and a woman on each object,
  `?clip=<name>` plays one clip on everyone, `?clip=<name>&strip=8` shows it held at 8 times on a
  man, a woman and a child; `?manual` with `__poses.advance(s)` and `__poses.focus(n)`.
