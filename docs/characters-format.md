# Residents: the character format

The residents (bodies, faces, hair, clothes) are generated in code by `cargo characters` (in Rust,
`crates/tools/src/characters`), and their animation clips are made in code in
`web/src/render/shared/characters/clips`. This page is the contract between the generator, the
clips and the runtime (`web/src/render/shared/characters/`, drawn by `web/src/render/pc/characters.ts`).
Everything here is original work.

```
cargo characters build                                # writes web/public/assets/characters, then checks it
cargo characters check [set dir]                      # the set against this contract
cargo characters diff <a> <b>                         # two sets, within the quantisation
bun tools/characters/check_clips.ts                   # the clips against the rig (see Checking clips)
cd web && bun run check:clips                         # the same on both bodies at every life stage (CI)
```

The build is deterministic (seeded noise only): the same code gives the same bytes.

## Conventions

- The game's left-handed space (Babylon's, which the PlayCanvas renderer keeps under its mirrored world
  root), metres, y up. A resident faces +z; their left is -x.
- Every rest local rotation is the identity. A bone's frame at rest is model space, and its local
  translation is the offset from its parent's joint. `restQ`, `pre`, `post` and `sourceRestQ` are
  absent or identity.
- The rest pose is an A-pose: arms 45 degrees down, legs straight and vertical, a gently curled
  open hand.
- Meshes are written with clockwise front faces (the left-handed convention, as Babylon had it). Inside the
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

## rig.json (version 5)

| Key | Meaning |
|---|---|
| `version` | 5 |
| `bones`, `parents`, `clipBones`, `head`, `face.bones` | the skeleton above (parents come before children) |
| `fps` | 30 (clip baking) |
| `sourcePelvis`, `sourceRestQ` | the reference pelvis; identity rotations (`NB * 4`) |
| `reference` | `{ body, thigh, shin }`: the clips' reference body and leg lengths |
| `fieldScale`, `toneScale`, `stageScale` | int16 per unit for fields and tone, per metre for stage deltas (4000, 4000, 5000) |
| `morphs` | `{ channels, scale: 30000, normalScale: 63, identity: { first, count, eyes } }`: `identity` says which of the channels are the identity channels (the last `count`, from `first`) and, per body, the eyeballs a channel moves at weight 1 (`eyes.<family>.<channel>`: the right eye's centre x, y, z in metres, the left's x mirrored, and how much larger both are, of 1). See Morph channels |
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
| `faceScale` | how far this body's face takes a resident's identity (1 on the base stage; see Morph channels) |
| `grip` | see above (optional) |
| `skinRef` | `[1, 1, 1]` |
| `garments` | id -> cut, every id of the family in the creator's order (see Clothes) |
| `bodyChunks` | `{ "body": [] }` (clothes are painted, skin is never hidden) |
| `parts` | part name -> byte offsets in `mesh` |
| `hd` | the close-up body, in a file of its own: `{ mesh: "<family>.hd.bin", parts: { "hd.body": … } }`, laid out as `mesh` and `parts` are (see Close-up body) |
| `stages` | other life stages: `{ mesh, height, restT, inverseBind, pelvisScale, seatDrop, eyeY, grip, faceScale, parts, hd }` (`hd`: `{ mesh: "<family>.<stage>.hd.bin", parts: { "hd.body": … } }`, the close-up body's deltas) |

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

The runtime interleaves a part's position, normal, joints, weights, field, morph slot and tone into
one vertex buffer of 80 bytes a vertex (`residentMesh` in `pc/characterMaterial.ts`; parts no morph
moves get slot -1); the instance matrices and settings are the draw's other buffers.

## Parts

Every body has:

| Part | What | Skinning | Material |
|---|---|---|---|
| `body` | the skin, with the eyelids and lips (from the face) and the ears | body bones; lids, lips and ears Head | `body`: skin colour, painted clothes |
| `lod.body` | the low-detail body (far residents, every shadow), with the eyelids (they blink) and plain ears | body bones; lids and ears Head | `body` |
| `eyes` | two eyeballs | `eye_l`, `eye_r` | `eye` (procedural iris) |
| `lashes`, `brows` | lashes on the upper lid rims; brows | Head | `lash` (dark), `brow` (hair colour x 0.45), opaque |
| `teeth`, `tongue` | teeth; the mouth plate (dark, tone.y = 1) and tongue | Head | `mouth` |
| `hair.<style>` | every hairstyle but `none` (`short crop layered tousled sideSwept afro bob long ponytail braid bun`) | Head, blending to neck_01 / spine_03 below the jaw | `hair` |
| `hair.beard` | facial hair (male only; hidden for baby, child, teen) | Head; follows the mouth's morphs and the face's identity | `hair` |
| `hair.<style>.wisps`, `hair.beard.wisps` | the soft edges of each: thin quads past its rims, hems and tips (below), drawn only close up | their hair's, vertex for vertex | `wisp`: the hair's colour and light, cut out in strands |
| `top.*`, `bottom.*`, `shoes.*` | shells: jacket and suit (torso and upper arms, open in a V; the sleeves are painted), the collars of polo and shirt, the skirt, the server's apron (the front from the waist to above the knees), every shoe (below) | from the body | `cloth` (flat colour) |
| `bottom.trousers` | the cloth of every bottom with legs (trousers, suit trousers, shorts, capri: one mesh, 824 triangles): the body's own surface from the highest waistband to the lowest hem, cut to the resident's waist and length by the shader (see Clothes) | as the skin | `cut` (cloth whose opacity is the cut) |
| `top.long` | the cloth of every regular top (tee, v-neck, polo, long top, shirt, tank top, blouse: one mesh, 1,212 triangles on a man and 1,160 on a woman): the body's own surface under any of them, cut to the resident's sleeves, hem and neckline by the shader (see Clothes) | as the skin | `cut` |
| `lod.<shell>` | a low-detail copy of every shell but `bottom.trousers` and `top.long` (far off, the low-detail body is painted) | | `cloth` |

`hair.none` has no part: a bald resident's scalp keeps the skin colour (appearance texel 7 w = 0).

### Close-up body

`hd.body` is the body one subdivision finer (level 2 where `body` is level 1: its skin in four times the
triangles, 20,200 with the same lids, lips and ears after it), with fields, tone, occlusion and morphs
of its own, worked out at its own vertices. It is drawn in place of `body` only where one resident fills
the view: the creator's stage, where `body`'s outline shows its edges as corners of 8 to 20 degrees
(3 to 4 px at the face view). In play it shows nothing, even at the closest zoom (6 m), and a portrait
(160 px over 0.44 m, shown at 26 to 60 px) would move its outline by less than half a pixel, so
neither uses it.

It is stored, not made at load: its skin is snapped to the body's surface and its sockets carved at
its own vertices, as its fields, tone and morphs are worked out there, at every stage and for every
identity channel, and a subdivision step at load reproduces none of that. So that play never
downloads it, it lives in files of its own: `<family>.hd.bin` (the part) and `<family>.<stage>.hd.bin`
(its deltas), 1.29 MB gzipped over the 3.85 MB play fetches in all. The creator fetches a family's
(about 330 KB gzipped) with the first resident of that family on its stage, and a stage's (46 to 74 KB)
with the first of that stage; until they arrive, and if they fail to, the stage draws `body`
(`loadCloseUp` in `characters/data.ts`).

Hair (`hair.rs`, up to 7.4k triangles a style, one draw) is closed shapes in locks, so it breaks
its outline and shades in clumps. Every style's layout is a shape, and it is cut at `DETAIL` (1.5)
times the rows the layout asks for and its locks on `LOCK` (6) columns each, not 4 (the same locks,
more finely cut): at 3k a cap was 40 columns and 8-16 rows, which read
as plates with a polyline for a hem; at 5.2k the curtains were ribbons of six or nine, the width of a
thumb at arm's length, so the layered styles now ask for ten locks over four rows and the cap of the
short ones for 52 columns and 18 rows:

- every style has a skull cap: rows from the crown to the hairline in locks 4 of its layout's columns
  wide (6 as built, `hair::LOCK`: the detail adds columns to a lock, not locks), which
  leave the crown in a whorl, the grooves between them deepening to the hairline (none over the
  ears); a parting where the style has one (short and side-swept at the side, long and buns in the
  middle) is a groove the columns turn onto; short locks past the hairline at the temples and nape;
  the rim comes down to the skin and goes round the ears in an arc, as a cut does: above an ellipse
  round each ear's faces (seen from the head's centre, about 6 mm out), the hairline and the arc
  rounded where they meet, on more columns there (7 for every 16 round the head, each side). The
  afro's stands off the ears instead (its rim at the hairline, out past them) and closes under
  itself in a hem, in to the skin and, over an ear, above its ellipse;
- curtains (long, bob, layered) and fringes (bob) are sheets with a thickness: each row a closed
  section with rounded edges, a rounded hem curled in, in locks (thinner between them, the edge
  locks fuller), each lock's tip hanging lower than the partings either side of it. A curtain comes
  out from under the cap and falls straight from the scalp's widest, out over the ears, its inner
  face no nearer the back than `back(y)` (the long style's further off at the shoulders, which rise
  under it sitting down);
- the long style's fringe is two lofted locks a side from the parting past the cheek; side-swept,
  three locks swept across the forehead; tousled, a lumpy cap and 22 tapering locks lifting at their
  tips; the afro, a rounded shell in 46 round puffs; the ponytail's tail, a loft in four twisting locks;
  the braid, three strands plaited (each a loft), a tie and a tuft; buns, twisted, with a band.
- the beard is a shell over the jaw in locks, their tips below the chin, thinning to the skin at
  its lower edge; the sideburns stop in front of the ears.
- no hair comes within 2 mm of the head (`sdf`) or 1.5 mm of an ear (the ear pieces' faces), and no
  edge passes through an ear, at every life stage at rest (`check`).

Solid hair ends in hard edges. Past them go wisps (`hair.rs` `Strip`, `wisps`; a part of their own
per style, at most 600 triangles, 150 to 470 as built): rows of quads, each growing from two
neighbouring vertices of the solid hair (the same positions, normals, weights and tone, so a row
starts exactly on the hair and is shaded as it is; a wisp's two vertices are its root, then its
end) out to where the wisps end, kept clear of the head and the ears as the hair is. The shader
draws strands on them and cuts the rest away (below). Where:

- a cap's rim, on over the skin the way its columns come down to it (walked along the head's
  field, which knows the neck): 9 mm over the forehead, 13 mm at the temples, 16 mm at the nape
  (0.04, 0.055 and 0.07 head heights), shorter towards an ear and none round it; six tenths of that
  on hair drawn back tight (ponytail, braid, buns); only in front of the curtains where a style has
  them (layered, long); none on the bob, whose fringe and curtain cover its rim;
- the hem of every curtain and fringe: from the outer face's last row down in front of the hem's
  lip, as one that far (fields.w), and on below it, turned in half as far as the hem is (0.05 to
  0.08 head heights; the bob's fringe 0.035);
- a lofted lock's tip: a brush from its last ring but one (two on the thicker locks) on past its
  end, drawn in to 0.45 of that ring's breadth; over the outer face only on the flat locks that lie
  on the head (the long style's face locks, the tousled tufts), all round on the others (the
  side-swept locks, the ponytail's tail, the braid's tuft);
- the afro, whose outline is no edge: tufts all over it, laid like shingles (a row round the shell
  from every third ring, on every third column, each on down over two rings and lifting 0.06 head
  heights off it), their strands loose curls, so its outline is strands' ends from every side;
- the beard: below its lower edge and up each cheek from its upper one.

Left solid: the buns and their bands, the ponytail's band, the braid's strands and tie (tight hair,
whose outline is smooth), the sides of the curtains and fringes (they lie on the cheeks and the
temples), the beard's edge under the nose, round the lips and at the sideburns.

The shader (`hair` branch of `characters/material.ts`) takes the hair colour with the clump bands
(tone.z), and on dark hair lifts the clumps and a sheen band round the crown (where the strands, taken
straight down the head from the normal, lie square to the half-way vector between the view and a light
from above) towards a lighter tint; both even out under a pixel or two. The material has no PBR sheen
(linked to the albedo it took the colour down to 8 %).

The wisps are that hair, cut out (`character.ts` `WISP`, `RESIDENT_WISP_OPACITY`): along the edge
(fields.x) strands stand in cells, 500 a metre, in three layers, each sparser than the last and set
off from it; a strand is a random share of the quad long (fields.y past fields.w), longer and shorter
in tufts along the edge, as wide as its cell at its root, where the row is one with the solid hair,
parting from its neighbours and thinning to its tip, leaning a little on the way (a loose one,
fields.z = 1: waving, and broken along its length). No texture: the strands are drawn from the
fields, and where they are a pixel wide or less they blur into their mean cover. The cover is the
material's opacity: under 0.1 nothing is drawn (the alpha test), over it the wisp blends over what
is behind it and, where the target is multisampled (Medium, High, the creator, the portraits),
covers that share of the pixel's samples (alpha to coverage), so a strand's edge is as smooth as a
solid one's; on Low the blend alone smooths it. The blend adds the alphas up, which keeps a
transparent canvas (the creator's stage, the portraits) covered under them. The wisps are drawn
after everything solid, from both sides (their faces' winding is free: the one part `check` does
not hold to it), cast no shadow, write depth, and are lit by the normals of the hair they grow
from, turned to the viewer where a quad stands across the view. Drawn only where a face is 4 px
per 5 cm or more (`parts.ts` `WISPS_PX`; thinned out by the shader below 7): not at the game's
default camera (no draw call), all of them zoomed in.

The body (`body.rs`) is a control cage of superellipse rings (`cage.rs`; the head's cage
in `head.rs`) fitted so its limit surface passes through the ring points and subdivided once
(`subdiv.rs`): `body` is about 4.8k triangles of skin plus the face's lids and lips and the ears (5.9k
in all, budget 6k), `lod.body` the cage at its limit plus the lids and plain ears (budget 1.4k; the
fingers are on it too). Skin
weights are set per ring and subdivide with the positions. Every stage of a family has the same cage.

A hand is a palm with the ball of the thumb on it, a knuckle ring, four fingers and a thumb. A
finger is a 6-sided tube in two bands: from the knuckles straight to just short of its middle
joint (`fingers_02`'s joint, 0.42 palm lengths past the knuckle, so the outer band is about as long
as the inner one), then on to a narrower ring before the rounded tip, curled a little at rest.
Neighbouring fingers share the edges down into a crotch past the knuckles and stand apart beyond it,
so the baked shade (tone w) darkens the gaps. The thumb is a 6-sided tube from the palm's thumb side
and the palm next to it. Weights: the knuckle ring is about half `hand` and half `fingers_01` (more
of the fingers' on the palm side), a finger `fingers_01` up to the middle joint and `fingers_02`
past it (on `body` all of it from just past the joint, so the outer band stays round when it
folds), so the clips' fist (`GRIP`) bends at the knuckles and the middle joints, the tips fold back
onto the palm and all four fingers curl together; the thumb's as before (`thumb_01` / `thumb_02`).
A small child's hand is a little wider, its fingers plumper and shorter.

The head's cage (`head.rs`) is a grid of rows (from the jaw up) and columns (degrees round the head)
under a crown patch: 8 rings of 12 round the whole head, and over the face a finer grid whose rows
and columns run only part of the way, each ending on a whole one (so every face is a quad, or a
pentagon or hexagon where a line ends on its side): columns at 7 degrees beside the nose's middle
(from the mouth to over the eyes) and at 40 at the eyes' outer corners (from the nose's tip to the
brow), rows at the mouth and the nose's base (to 28 degrees) and under and over the eyes (to 50).
The points lie on a smooth head (a soft union of a cranium, cheeks-and-jaw, a muzzle and a chin
ellipsoid: the face from the nose down stands up rather than sloping back to the chin); the features
are offsets on them: the nose (a bridge from between the eyes to a round tip over the nose's base,
its wings beside the tip and the base), the upper lip rounded forward with a groove down its middle
(the philtrum), the brow. Between the grid's vertices the drawn head is snapped onto the
smooth one (the cage's limit alone dents it by up to 4 mm there, where the grid's spacing is uneven
and its partial lines end: patches on the cheeks), and the eye sockets are carved into every point
of it (`sockets`): round each eye a shallow hollow (the eyes' fronts are about level with the face
round them) that the eyeball stands out of, deepest at the eye's corners and going on down behind the
eyeball under the lids; the LOD has one vertex in each, set back. The lids cover the sockets, so the
skin's normals are the head's without them (and its shade is baked so, below). The LOD keeps the
8 x 12 grid (`Cage::lod_faces`): the finer grid's vertices are in `lod.body` but unused. On the face
(`face.rs`): lids reaching from a seam on the face round each socket (0.5 mm in front of it) to their
edges on the eyeball, lying on the face as it is without the socket, or on the eye's bulge where that
stands out, so a closed lid is the face with the eye's round under it and no socket shows round it;
the lids meet on the eye's front at its corners, the upper has a crease above its edge. The lids'
surface is the base stage's face as far from the eye in eye radii (`Face::lid_z`), so the channels,
made on the base stage, close the lids over the eye at every stage. Full lips, tubes along the mouth,
the upper with a cupid's bow over the philtrum; brows thickest at the inner end, laid by the eye
(`head.rs` `brow_layout`, in eye radii from its centre): from over its inner corner to a little past
its outer one, on the brow bone (not round the temple); a man's lower, straighter and heavier, a
woman's higher, thinner and arched further out, its tail dropping.

The ears are not the cage's (`ear.rs`): each is a piece of its own set into the side of the smooth
head, as the lids and lips are set on the face, and comes after them in `body` (the left one, then the
right: 169 vertices and 324 triangles each; `lod.body` has a plain flap of 24 triangles a side). An ear
is laid out on a sheet hinged on the head along the ear's front edge and turned out towards the back
until its back edge stands 13.5 mm off the head (on a grown man: 20 to 25 degrees from the head's side),
leaning back a little at the top; a real ear's size and place (about 62 mm high and 34 wide on a man,
58 and 31 on a woman, from the brow to the nose's base; by the head's height at the other stages, a
baby's a tenth smaller). The mesh is a polar grid round the bowl's middle: the bowl's floor (the
concha, 2 mm over the skin), its rim (the antihelix behind and above it, the tragus in front), the
groove inside the helix (the scapha), the helix's crest, the ear's edge between two rings close to it
(the edge's turn is in two narrow bands, so no broad one is shaded as turning away from the eye, which
the sky's reflection draws as a white line), then round to the back: under the helix, under the bowl's
rim (the ear is a shell 3 or 4 mm thick, standing free of the head behind the bowl), and a foot ring 2
mm inside the head, so nothing is stitched to the skin. The lobe has no rolled rim, and the front edge
runs into the cheek. An ear's normals are its own triangles' (weighted by their angle at the vertex,
and leant until no corner faces away from its triangle; the edge is the front's and the back's, so it
shades round), turned to the head's where the ear enters the head (the foot, the front edge), where it
also takes the skin's tone: the join shows no crease and no line. It is built from the stage's smooth
head with the shared face (the identity channels leave the ears and the skin by them alone), is
skinned to `Head`, and no channel moves it (`check`). `cargo characters ratios` prints each body's ear
(what of it stands out of the head), and how many of its triangles' corners are shaded as facing away
from the triangle (none; `check` fails on one).

Grown faces and bodies differ by family more than the shared shapes do, as far as the stage is grown
(`rig::maturity`: none for babies and children, 0.6 for teens): a man's jaw wider and squarer, his
chin broad, his nose larger, his brow ridge heavier, his neck and arms thicker, his chest and
shoulders broader (shoulder joints at 0.136 of the height, a woman's 0.115) over narrower hips and
thighs, his waist nearly as wide as his hips (0.83 of them across, a young adult's); a
woman's jaw narrower, her cheekbones higher, her chin small and pointed, her nose smaller, her eyes a
twentieth larger, her lips fuller (at every stage: `face.rs` `style`), her hips and thighs wider, her waist 0.6 of her hips across (a
young adult's; the stages' own waists widen it with age), with a gentle
bust (`head.rs` `grown`, `nose`, `brow`; `body.rs` `Shape::m`).

The head interface (`head_frame(family, stage)`, `head.rs`): `sdf` is a smooth signed distance
to the head and neck as drawn (the ears, pieces set into it, left out; about 1 mm near the surface), `surface_z` the
front surface's z; `ears` boxes round the ears (hair goes round or over them); `centre` / `radii` a
loose ellipsoid round the skull (with a margin; fit close things to `sdf`); eyes, mouth, brows,
hairline and chin as before.

## Fields (4 per vertex)

| Part | x | y | z | w |
|---|---|---|---|---|
| body | sleeve parameter (-1 torso, 0 shoulder .. 1 wrist, > 1 hand) | height (m, base body) | crew neckline (m, > 0 covered: distance from the neck's axis plus depth below a curve round the neck's base); head: -1 - scalp weight (the ears: -1) | scoop neckline (women: round and low in front; men: a V); head: -1 - scalp weight (the ears: -1) |
| shells | hem distance (m; a shoe's from its collar, negative on its lining; 0.5, out of every reach, on a shell the shader cuts) | flag: 1 sole, 2 waistband, 3 skirt or apron (a hem band), 4 light midsole, 5 polished, 6 cut to length by the shader (`bottom.trousers`) | category: 0 top, 1 bottom, 2 shoes | shoes: across the instep (m from its middle; over 1 off its front); a shell the shader cuts: the body's height field there (m on the base body, as the cuts are); else 0 |

A shell the shader cuts has no seated corrective, and `top.long` has the body's other three fields in its tone's
x, y and z instead (the sleeve parameter, the crew and the scoop neckline: its cut is on all four); w is the
occlusion as everywhere.
| eyes | offset from the iris centre on the eye's front (m), x | y | iris radius (m) | pupil radius (m) |
| hair | 0.1 (unused) | | | |
| wisps | along the edge (m, from a start of the row's own) | 0 at the root (on the hair) .. 1 where the quad ends | how loose the strands are: 0 combed, 1 curls | the share of y over which the strands are still one (solid: a hem's lip) |

## Tone (4 per vertex)

x = lip / cheek blush (0..1; on the tongue part, the tongue's red; on the hair and its wisps, how loose
a vertex hangs, 0..1: see Hair and hems that move), y = shade drawn by hand (the
upper lids' crease, the nostrils: the nose's underside either side of its middle, the teeth's sides,
deep in the hair; the mouth plate is
1), and on the lips its opposite sign: negative, how much of the lip's colour they take (up to 0.9,
less at the mouth's corners and towards the line between them, which the occlusion bakes: the
shader tints the skin red-rose by it and takes the gloss off, `character.ts` `LIP_MIX`), z = hair
clump bands (0..1: light in a lock's middle, dark in the parting between two), on the teeth their
count from the middle one (a tooth's width a unit: the shader draws a dark line at every whole
number, `character.ts` `TOOTH_GAP`) and on the
body the face's age folds (0..1: a soft line from beside
each nose wing down past the mouth's corner, `HeadFrame::fold`, each vertex's the mean over its
triangles weighted by its barycentric weight, as the triangles can show it; the shader darkens the
skin by `0.3 z age`, appearance texel 4 g, so young adults show none; on the lips instead, how far
the lip turns down at rest, 0..1, which the shader shades by `LIP_SHADE`: baked, so it is the lip's shape
whichever way the head faces), w = the baked ambient occlusion
(below). A wisp's two vertices take the tone of the hair vertex it grows from, w included (in
`build.rs`, after the bake). On `body` the eyelids take the tone of the face they lie on (as it is without the socket, at
their nearest point), w included (the upper lid keeps its crease), and its normals there, in every
shape of the face (build.rs `lid_shading`); on `lod.body` the mean tone of the skin round them. The
ears have a blush of their own (most on the lobe), a shade in the bowl's middle for the canal (y) and
part of their baked occlusion (0.7 of it on the front, 0.4 on the back, which faces the head a
centimetre off and would be as dark as the inside of a mouth: an ear is thin and lit through); where
an ear runs into the head (its foot, its front edge) it takes the tone of the skin there instead, so
the join shows no line. Every stage uses the base stage's tone.

w is how much of the sky a vertex does not see at rest, 0 (open) to 1 (enclosed), in 32 steps
(`ao.rs`, after the furniture's `gamekit/shade.py`): 96 rays per vertex (48 on the hair, whose shade is
the depth into a broad volume) over the hemisphere about its
normal (cosine-weighted, a Fibonacci spiral twisted by the vertex's index), started 1.5 mm off the
surface; a hit counts `1 - (d / reach)^2`, the reach being 0.12 m on the skin (a fold, a seam, the
crease beside the nose), 0.15 m on the shells and 0.34 m on the hair (a fifth of the body's height,
where the shade is the depth into the volume). A vertex whose rays mostly meet a back face first is
buried in another part and stays 0. On the head (from 0.1 hh under the chin up) a skin vertex takes
instead the mean over its triangles (four points each, weighted by its barycentric weight), so a
crease's shade narrower than the triangles (the skin against the lips, beside the nose) is not
caught at a few vertices and spread in spikes; and the skin is baked as it is without the eye
sockets (the lids cover them). The shade then eases over the mesh twice (once on `lod.body`, not at
all on the hair): a vertex takes half its own and half its neighbours' mean, the face's vertices
left alone, so a crease that lands on two of a ring's eight vertices reads as a crease and not as a
blotch. What occludes what:

- `body`: itself (its lids, lips and ears included) and the teeth and tongue, not the eyes;
- `lod.body`: the same, its own bake;
- shells: themselves and `body`; their `lod.` copies themselves and `lod.body`;
- `hair.*`, `hair.beard`: themselves, `body` and the eyes (one body serves every style and outfit,
  so hair and clothes don't shade the skin).

The shader takes the colour (after the painted clothes, on the body) times `1 - 0.8 min(k w, 1)`: as
dark as the furniture's at most, reached once two thirds of the sky are hidden (k = 1.5; on the
body 1.5 + 0.6 age, appearance texel 4 g, so elders' creases are deeper).

The jacket, suit, skirt and apron shells use their tone's xyz for their seated shape instead (m, in the
rest space, before skinning; w is still the occlusion): what moves each vertex from where a seated
pose (`seated_pose` with the thighs turned 76° forward, as `sit` has them) skins it to where its
seated shape is skinned (the skirt without its flare, its hollows between the legs bridged, the
jacket without its hem's flare, the apron as it hangs), out of the seated skin and no lower than the
seat (`seated_tone`, clothes.rs). The runtime
adds it as far as the thighs are turned forward (`writeSit`: from 15° to 76°, the mean of both, at
the body's size; appearance texel 4 b), so a seated skirt lies on the lap, and the jacket's hem and
the apron on it, instead of standing out round the seat as they hang standing.

## Clothes

Clothes are painted on the body by its shader, and near the camera a bottom with legs and a regular top are
cloth of their own over that (the trousers' and the long top's shells, below). Each garment's cut (metres on the base body's
fields) decides where it covers:

- top: `(sleeve - x) * 0.55 > 0`, `y > hem`, `z > crew` (if set), `w > scoop` (if set);
- bottom: `y < waist`, `y > leg`, `x < -0.5` (not on the arms);
- shoes: `y < shoe`, the sock's top (under the shoe's shell, and the sock in its collar's opening).

A top with `over: true` (its hem is below the waistband: the tee, the v-neck, the polo and the long top worn
out, the shirt, the jacket, the suit) is painted over the bottom; otherwise (the tank top, the blouse, tucked
in) the bottom wins. Shoes win over both. Edges are anti-aliased; painted cloth is
rougher than skin. Shoes have a `colour` (sRGB hex). Skirts set `skirt: true` and their `leg` cut
paints the thighs (the shell hangs over them). Babies don't wear skirts (the shell can't follow
their drawn-up, spread legs): a baby whose outfit has one wears `bottom.shorts` in the top's colour,
a romper (outfit.ts).

Painted cloth stands off the skin (character.ts `EASE`, the body's vertex shader): each covered vertex is
drawn out along its normal, as far as the garment is loose (appearance texel 8; outfit.ts `EASE_OF`), so
a tee hangs from the chest past the waist (4 mm of cloth, up to 2 cm more below the chest), a trouser leg
falls nearly straight from the knee over the shoe (up to 2.6 cm at the ankle) and a cuff or a hem stands a
step off the arm, with a shade under it (`EDGE_SHADE`). The amounts are metres on the base body by the
fields' heights, scaled to the life stage (the vertex's height over its field's). What clings (swimwear,
the towel) and what lies under a shell (the shirt in a jacket, a skirt's painted thighs) has none; under a
skirt's shell the top lies flat from the waistband down (texel 8 b). No triangle, byte or draw is added.

Each painted cut edge is drawn as fabric (material.ts `simHem`, `BAND`): a turned band along it, a
little darker (lighter on dark cloth), with a thin stitch line in it. The band is 2.2 cm at a cuff,
2.4 cm at a top's or a leg's hem, a 1.6 cm rib at a crew neck, a 1.1 cm binding at a scoop, a 3.4 cm
waistband and a 1.2 cm cuff at the sock's top (m on the fields, as the cuts). Bands and stitches fade
out as they come under a few pixels: they show at the closest game camera (6 m) and are gone at the
default one (24 m). The skirt's, the jacket's, the suit's and the apron's shells have the hem band
too, from their `fields.x`: the distance from the hem (the jacket's and the apron's: from their
nearest open edge — the hem, the lapels and the neck for the jacket, the hem, the sides and the
waistband for the apron — not the sleeves' ends; 0 on the turned rim). The collars have none (0.05:
out of reach).

A bottom with legs (`bottom.trousers`, `bottom.suit`, `bottom.shorts`, `bottom.capri`; `outfit.ts` `LEGGED`) is
worn near the camera as one shell, `bottom.trousers` (clothes.rs `trousers`, through the jacket's path:
`clip_by_field` on the bottom's predicate, `shell_on` at every stage): the body's own level-1 surface from
6 mm over the highest of their waistbands to 6 mm under the lowest of their hems, the arms and hands left
out, with the body's height in fields.w. The shader does the rest per resident, so shorts and capri are the
same mesh:

- lifted off the skin (vertex, `GAME_SIM_CUT`) by the bottom's ease, the same `EASE` and looseness the
  painted cloth has, so the shell near the camera and the paint on the low-detail body far off have one
  outline;
- cut at the resident's waist and leg cuts (texels 4 a, 5 a) per fragment: its opacity is 1 between them
  and 0 past them, a pixel between (the resident feature's `opacity` slot; the material is alpha-tested at
  0.5 with alpha to coverage, so the hem's edge is as smooth as a mesh's where the target is multisampled);
- drawn from both sides, the inside (back faces) at half the colour (`CUT.lining`); the waistband's and the
  hems' bands from the same heights as the painted ones;
- the body under it (texel 8 a = 1; `body` and `hd.body`, not `lod.body`, whose material has
  `GAME_SIM_FAR`): its bottom is still painted in the bottom's colour, at half of it (the inside of the
  legs, seen up a leg or under a hem) and with no ease; from 2 to 5 cm inside the cuts it is drawn in 6 mm
  (not under a top worn over the bottom, which is cloth out past the shell), so no skin comes through at
  any distance or on the finer close-up body; a top tucked in loses its ease under the waistband.

It has no `lod.` copy and no seated corrective (its tone's xyz are 0), casts no shadow of its own (the
low-detail body casts the resident's) and does not sway (its hem distance is out of reach). Babies, pyjamas,
the towel and swimwear stay painted (`bare`, texel 8 a = 0). A resident near the camera in a bottom with legs
costs 824 triangles more, and each body and life stage in view one draw more.

A regular top (`top.tee`, `top.vneck`, `top.polo`, `top.long`, `top.shirt`, `top.tank`, `top.blouse`;
`outfit.ts` `REGULAR`; not the jacket, the suit or the apron, shells of their own over a painted shirt) is worn
the same way as one shell, `top.long` (clothes.rs `tops`): the body's surface wherever any of them covers, with
6 mm to spare. Its cut is the top's whole predicate (sleeve, hem, crew, scoop), from the body's four fields
(fields.w and the tone's x, y, z); its ease is the painted top's (`EASE`: the hang from the chest, the
sleeves, flat under a skirt's shell), but no thinner than 0.7 of it at the neckline (the painted cloth thins
to a quarter there; a shell 1 mm off the skin lies in it), and none under the trousers' waistband for a top
tucked in, which also ends 2 cm under the waistband whatever its own hem (`CUT.tuck`: the bottom is over it
there). Its bands are the painted ones (cuffs, hem, the neck's rib or binding); the polo's and the shirt's
collars are their own shells on it as before. The body under it keeps the top's paint without ease, drawn in
6 mm and at half its colour from 2 to 5 cm inside the cuts (texel 8 a = 2, or 3 with the trousers' shell). A
resident drawn with the close-up body (the creator's stage) keeps the top painted (`outfitFor`'s `painted`):
at the face view a level-1 shell's shoulders and neckline are corners where the close-up body's are round. A
resident near the camera in a regular top costs 1,160 to 1,212 triangles more, and each body and stage in view
one draw more.

The shirt under an open jacket or suit has its own colour (outfit.ts `shirtUnder`): white under a
suit, a charcoal tee under a light jacket, a white or grey one under a dark one. It is painted on the
torso from the waistband up to the jacket's neck line (1 cm under `neck_01`, where clothes.rs cuts the
shell outside the V; `OutfitFit.neck`), and above that only as a 1.2 cm collar at the neckline, so the
jacket's colour still reaches the neck on the shoulders.

A shoe's shell (`shoes.*`, clothes.rs `shoe`) is its upper and two soles. The upper is the foot's
skin under the collar, lifted 3-6 mm off it (more on a toe box, round a heel counter, in a padded
collar, at a boot's top; a dress shoe's toe drawn out forward) and skinned as the skin is, so it
moves with the foot; its edges the collar crosses are split, and every point lies on the curved
skin. The collar runs round the ankle, low at the sides (a boot's up the shin; a dress shoe's low,
with a throat down the instep), its lining a band turned in to the skin. The sole stands a few mm out
round the upper, from the floor (as the old soles: the clips and the seat fit stand on it) to its
top; heels are thicker, and a heel's arch and the toe lift off the floor. Laces are bars across the
instep (fields w and x); a sole is dark, or a thin dark welt over a light midsole over a dark outsole;
dress shoes are
polished. The low-detail copy is the low-detail body's skin, unsplit and unlined, on the same
soles (fewer sides).

What a resident wears changes with what it does (`Wear`, outfit.ts; characters.ts `wearFor`): its day
clothes; pyjamas for a night's sleep in bed (a `sleep` use lying down: the `top.long` and
`bottom.trousers` cuts in a sleep colour, barefoot); the day clothes barefoot for a nap on something
slept in; a towel under a shower and sat in a bath (the bottom's cut alone, from under the arms, or
round a man's hips, to the thigh); swimwear in a hot tub (a swimsuit, the `top.tank` cut down to the
top of the legs, or trunks). Pyjamas have the bands of the cuts they reuse (cuffs, hems, a crew rib,
an elastic waistband); the towel and swimwear are `plain` (no bands: a towel has no stitched cuff, and
swimwear's edges are bound flat). Only the day clothes draw their shells: the others leave them out by the
resident's row (`bare`), so a change of clothes draws nothing more. The towel's and swimwear's heights
come from the base body's joints at rest (`OutfitFit`: under the arms half way between `spine_03` and
`upperarm_l`, the hips and the knees).

The day clothes are one of the resident's outfits (sim-core `Sim::outfit`, in the snapshot's `sims[].outfit`):
0 is the one made in the creator, as `appearance` has it; the others (`OUTFITS`, three in all) are picked from
the id (outfit.ts `spareLook`: garments as `pickOutfit` picks a new resident's, a top or a bottom other than
the creator's, and a top in another colour from `palette.outfit`), skin and hair as they are. A use with
`changesOutfit` (the wardrobes and chests of drawers' "Pick an outfit") puts on the next one halfway through,
with the doors or the drawer open, and the resident walks off in it; it is kept, and saved, until they change
again. Pyjamas, the towel and swimwear are made from whichever outfit is worn. Its shells change with it (the
world is drawn again for it, as for a cupboard opened).

Garment ids per family (outfit.ts):

- male: `top.tee top.vneck top.polo top.long top.shirt top.jacket top.suit`, `bottom.trousers bottom.shorts bottom.suit`;
- female: `top.tee top.tank top.blouse top.long top.shirt top.jacket`, `bottom.trousers bottom.shorts bottom.capri bottom.skirt`;
- both: `shoes.sneakers shoes.trainers shoes.boots shoes.dress`;
- workwear, worn by the town's staff and never picked for a resident (`WORKWEAR`): `top.apron`, a white shirt
  painted on the body under an apron shell (`clothes.rs` `apron`), its own colour the top's.

## Appearance texels (per resident, after the bone matrices)

| Texel | rgb | w |
|---|---|---|
| 0 | skin colour (linear) | top: sleeve cut |
| 1 | top colour | top: hem |
| 2 | bottom colour | top: crew cut |
| 3 | hair colour (plain) | top: scoop cut |
| 4 | r = 1 if the top is `over` the bottom, or 1 + n + y / 4 with a shirt under an open jacket (n: 1 white, 2 charcoal, 3 grey; y: the jacket's neck line, m), else 0, or -1 if `plain` (no bands); g = age (0 young, 0.4 adult, 1 elder); b = how far the shells take their seated shape (0..1, at the body's size; below) | bottom: waist |
| 5 | skin reference (1) | bottom: leg cut |
| 6 | shoe colour | shoes: the sock's top |
| 7 | eye colour | 1 if the resident wears hair (the scalp takes the hair colour) |
| 8 | r, g = how loose the top and the bottom are (0..1: how far the painted cloth stands off the skin, `EASE`); b = 1 if the bottom has a shell round the hips (a skirt) | 1 if the bottom is worn as the trousers' shell (a bottom with legs, in day clothes, not a baby), 2 if the top is worn as the long top's (a regular top, likewise; not on the close-up body), 3 for both |
| 9 | the hair's lag: how far a loose end of the hair is drawn from where the head carries it (m, world axes, at looseness 1) | the shoulder line (m, at rest on the resident's body) |
| 10 | the hems' lag: the same for a shell's hem | the height under which a shell's hem is loose (m, at rest on the resident's body: 8 cm over the pelvis joint on a grown one) |

Then 20 texels of face morph (channel, weight) pairs, two a texel, a weight of 0 unused (`layout.ts`):

| Texels | What | Written |
|---|---|---|
| 12 (`EXPRESSION_TEXELS`) | the expression: the 24 strongest of the expression channels (`face.ts` `FaceMixer`); a first channel of -1 means no morphs at all on this face this frame (too small on screen, `parts.ts` `MORPH_PX`) | every frame the face is posed |
| 8 (`IDENTITY_TEXELS`) | the resident's own face: the 16 identity channels at the look's weights, signed, times the body's `faceScale` (`identity.ts` `writeIdentity`) | once, when the resident is built |

A row is 33 x 3 + 11 + 20 = 130 texels (128 before the lag texels). Disabled cuts are -10 (+10 for "below" cuts).
Texels 0 to 8 are the outfit's, written when a resident changes clothes (`writeOutfit`); 9 and 10 are written
every frame a resident is posed (`residents.ts` `writeSway`).

### Hair and hems that move

Hair that hangs and the hems of the shells move after the body, with no bones of their own
(`characters/sway.ts`). Per resident there are two lag vectors, the hair's and the hems', each a point on
a damped spring towed by a place fixed to the head (behind the neck) or to the hips (between the
thighs), so setting off, stopping, a turn of the head or the hips' sway swing it. The spring is stepped
in closed form from the towing point's change of velocity each frame, so the motion is the same at any
frame rate; the air holds it back by 2 cm per m/s (a runner's hair streams); what is drawn is no further
than 13 cm (hair) or 7 cm (hems) on a grown body, keeps part of the vertical lag only and hardly any
downward, scales with the body, and has a dead band of 1.5 mm, so a resident standing still (breathing,
shifting their weight) moves nothing. None far off (`poseFar`), lying down (by how upright the head and
the hips are), on a new resident's first frame, or, for the hems, seated (the seated share, texel 4 b).
The game being paused holds it.

The character shader (`character.ts` `gameSimSway`, in the per-material `transformInstancingVS`, GLSL
and WGSL) adds `lag x looseness` to the skinned position of every hair and shell vertex:

- hair and wisps: looseness is tone.x, baked by `hair.rs` (`Comp::sway`, `finish`): 0 on everything that
  lies on the head, and on the long style's curtain, the ponytail's tail and the braid (strands, tie
  and tuft) rising to 1 at the tips: along a tail or a braid as the 1.3rd power of how far along it a
  vertex is, on the curtain as the 1.5th power of its depth below the ears (0.4 head heights over the
  chin), all of it 1.2 head heights further down. A wisp has its hair vertex's tone, so it goes with it;
- shells: from the distance to the hem (fields.x): all of it at the hem, none 30 cm up a skirt or 12 cm
  up a top's shell (the jacket, the suit, the apron), as the 1.5th power between; only under the hips
  (texel 10 w: a jacket's lapels and collar are open edges too), not on shoes; the `lod.` copies and
  the shadow copies the same.

Never into the body: the part of the offset that points in to the body's own axis (the vertical
through the rest pose's origin, as the vertex's skin turns it) is left out, so hair swings round the
neck and out from it and a hem off the legs. Under the shoulder line (texel 9 w) the body is taken as
a wall across, the back: hair there keeps its way along the back and loses all of it towards the back
(with the axis alone, hair beside the spine went into the shoulder blades on a stop). The normals are
left as they are. Every preset has it: a texel read and a dozen operations per hair or shell vertex,
and two springs per resident on the CPU.

`appearance.face` is the resident's own face: identity channel name (without `id.`) -> -1..1, 0 or
missing as the shared face has it (household.ts `FACE_CHANNELS`, `FACE_GROUPS` for the creator's
sliders). `randomLook` draws one from a stream of its own (about normal, 0.36 wide, so most values are
within 0.6 and a face differs from the shared one in a few things), seeded by the rest of the look,
so it takes nothing from the look's dice and a town made from a seed is the town it was, with faces;
a child made by the
simulation (`lookFromSeed`) is somewhere between its parents channel by channel, with a little of its
own. A look without one (an older save) gets a face from the resident's id (`faceFromId`), so nobody
shares a face. Portraits are cached by the appearance, and by the id for a look without a face
(`portraits.ts` `lookKey`).

`appearance.build` (0.85 slim .. 1.2 broad; missing = 1) widens bones across their axis in
`skin()`: `K = W * (I + (g - 1)(I - a a^T)) * IBM`, the torso by g, the neck by half and the limbs
by 60% of it. The simulation keeps the appearance opaque, so old saves are unaffected.

## Morph channels

`morphs.channels` is the 15 visemes in the voice engine's order, then 43 ARKit units:

- visemes: `sil PP FF TH DD kk CH SS nn RR aa E I O U`;
- units: `browDownLeft browDownRight browInnerUp browOuterUpLeft browOuterUpRight cheekSquintLeft cheekSquintRight eyeBlinkLeft eyeBlinkRight eyeLookDownLeft eyeLookDownRight eyeLookUpLeft eyeLookUpRight eyeSquintLeft eyeSquintRight eyeWideLeft eyeWideRight jawForward jawOpen mouthClose mouthDimpleLeft mouthDimpleRight mouthFrownLeft mouthFrownRight mouthFunnel mouthLeft mouthLowerDownLeft mouthLowerDownRight mouthPressLeft mouthPressRight mouthPucker mouthRollLower mouthRollUpper mouthShrugLower mouthShrugUpper mouthSmileLeft mouthSmileRight mouthStretchLeft mouthStretchRight mouthUpperUpLeft mouthUpperUpRight noseSneerLeft noseSneerRight`.

`Left` is the resident's left (-x). Units are authored at ARKit strength. An expression never
deforms the head: these channels move the lids, lashes, brows, lips, mouth plate, teeth, tongue and
beard (with its wisps). Morphs are built at the base stage and shared by the other stages.

### Identity channels (a face per resident)

After the 58 expression channels come 16 identity channels (`contract.rs` `IDENTITY`,
`morphs.identity`), which give a resident a face of its own. The runtime sets them once per resident
from `appearance.face` (above); a weight runs from -1 to 1 and 0 is the family's shared face.

| Channel | At +1 (at -1 the opposite) | Moves at most (mm: man, woman) |
|---|---|---|
| `id.jawWidth` | the jaw wider below the eyes | 4.5, 3.3 a side |
| `id.jawSquare` | the jaw's corners stand out (square; -1 soft) | 3.7, 3.3 |
| `id.chinLength` | the chin longer | 4.7, 3.2 |
| `id.chinWidth` | the chin broader (-1 pointed) | 3.8, 2.0 |
| `id.cheekbones` | the cheekbones stand out | 3.0, 2.7 |
| `id.cheekFullness` | fuller cheeks beside the mouth (-1 hollow) | 3.3, 3.2 |
| `id.noseLength` | the nose's tip further out and lower | 4.0, 2.4 |
| `id.noseWidth` | the nose's wings wider | 2.4, 1.4 a side |
| `id.noseBridge` | the bridge higher between the eyes | 2.9, 1.8 |
| `id.browRidge` | a heavier brow ridge (the brows go with it) | 2.3, 2.1 |
| `id.eyeSpacing` | the eyes further apart | 2.4, 2.3 an eye |
| `id.eyeTilt` | the eyes' outer corners higher (7 degrees) | 2.1 (lids) |
| `id.eyeSize` | larger eyes, opened wider (7 % of the eyeball, the brows higher over them) | 2.7 (brows) |
| `id.mouthWidth` | a wider mouth (15 %) | 3.6, 3.4 a corner |
| `id.lipFullness` | fuller lips (34 % taller) | 1.9, 2.5 |
| `id.faceLength` | a longer lower face: the chin, the mouth and the nose's tip lower | 4.8, 4.2 |

(`cargo characters faces` prints the table as built.)

A channel is a difference (`identity.rs`): the base stage's head, lids, lashes, brows, lips, mouth
plate and teeth built with one of the generator's numbers at +1 (`head.rs`, `face.rs`: an
`Identity`), minus the same built at -1, halved, vertex by vertex. Everything on the face is made
from the head's frame, so it follows the skin by construction; the beard and any hair that lies on
the face (a long style's locks on the cheeks; within 6 mm of the skin all of it, none past 3 cm)
take the skin's shape where they lie, and the wisps by the same rule (a wisp's root is where a
vertex of its hair is, and moves as it does; its end follows the skin under it). The three that add a shape to the head (`jawSquare`,
`cheekbones`, `cheekFullness`: an ellipsoid joined to the smooth head, placed so it stands that far
out of either family's) are the head with it minus the shared head. The cranium above the hairline,
the ears, the back of the head and the neck stay as they are (`identity.rs` `Mask`), so hairlines
and hairstyles fit every face. Under the lids, the skin in an eye's socket goes with the eyeball
rather than by the difference of two sockets.

The eyeballs turn on bones, so the two channels that move or size them (`eyeSpacing`, `eyeSize`)
say so in `morphs.identity.eyes`, and `pose.ts` `skin` moves the eye bones' joints and scales them
(`Placement.eyes`, summed over the resident's weights once, with the texels).

Identity channels are on `body` (the skin with its lids and lips; not its ears), `lashes`, `brows`, `teeth`,
`tongue`, `hair.beard` and the hair that touches the face, with their wisps; not on `lod.body` or the shadow copies,
and they are skipped with the expression when the face is small on screen. The shapes are metres on
the base stage's head, so a body's `faceScale` scales a resident's weights: the stage's head height
over the base stage's, times 0.6 on a baby and 0.8 on a child, whose faces are round and unformed
(a man's: 0.42 baby, 0.69 child, 0.94 teen, about 1 from then on; a woman's 0.45, 0.70, 0.95).

`cargo characters check` holds the channels to the creator's list (`FACE_CHANNELS`), no identity
channel to more than 6 mm and none to a low-detail part, and every hairstyle (its wisps too) 2 mm clear of the head
at each channel's two ends, at every life stage (the head's samples moved as the channel moves the
skin, `HeadFrame::moved`).

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
- Clips hold fists (`GRIP`: the fingers 100 degrees at the knuckles and 110 at the middle joints, the
  tips on the palm, the thumb across them); the runtime opens the hands towards the rest (open) pose
  per activity.
- Clips start from the relaxed stance (feet under the hips, upright, arms hanging loose); the
  runtime adds no stance corrections of its own (only the arms a little out over a skirt, an
  elder's stoop and the body language below), so a clip looks in game as it was authored.
- Body language (`characters/bodyLanguage.ts`): a layer over every clip for the emotion the face shows, by emotion
  id (`BEARINGS`): the back bent or held up, the head bowed or raised, the shoulders dropped or drawn up, the arms
  held in or loose, the elbows bent, the walk's arm swing smaller or larger (sad: the head 12° down, the back 8°, a
  swing under half the clip's; happy: up straight, the swing 30 % larger; angry: upright, the arms in and the elbows
  bent 16°). Standing free or walking it takes all of it (but the arms while carrying); using a thing standing,
  inside one, in a hug, kiss, fight or cuddle only the head; seated the head and, sat plainly, a third of the back;
  lying, a baby or under stage directions (the creator, portraits) none. It eases in over about a second. The pace
  is sim-core's: an emotion's `walkSpeed` (content `emotions`), the walk matched to it as to any.
- Setting off and stopping (`characters/stride.ts`): a foot on the floor stays where it is. A gait begun from a stand
  starts where it is nearest to that stand (`gaitPhase`: one foot under the hip, the other passing it, on either
  foot; from a stop cut short, the stride that was left), its legs take the gait's pose in 0.1 s while the rest of
  the body fades (0.2 s, shorter on shorter legs), the foot it stands on kept where it stood, and until the fade is
  over the cycle goes on by the way the body has gone, not by the clock (the speed the rate is matched to is taken
  as it is then, not eased). A gait left for a standing pose stops with the body, and the feet come to where that
  pose has them one at a time, each in a step through the air (0.12 s and 0.5 s per metre it goes, 0.3 s at most;
  lifted 22 % of the way, 4.5 cm at most): first the one in the air, or with both down the one behind, the other
  kept where it stands meanwhile; the fade is as long as its two steps. The legs are put on those points (`legPin`:
  turned at the hip and bent at the knee, as additive rotations) over whatever the pelvis' fade does above them; a
  fade cut short holds them as drawn (`legBake`) and steps on from there. Not for a baby, nor to a pose that is no
  stand (onto a saddle, a seat's clip part way): those fade as before. The walk's hip roll (`GAIT_SWAY`) turns the
  pelvis and leaves the legs as the clip has them (the thighs turned back by it, the body lifted by what the
  standing hip drops): rolled with the pelvis, the standing foot went 6 cm sideways over the floor and back at
  every step of a woman's walk. Measured on the runtime by `tools/characters/transitions.ts` (below).
- Turning on the spot (`characters/turns.ts`): standing free, a turn of 35° or more steps round with a turn clip
  (`turnR90`, `turnL180Child`, …: below), the Sim's yaw following the clip's steps (`turnShare`) so a planted foot
  stays put; smaller turns pivot on the feet. Not while walked in, inside or up on a thing, getting up, holding or
  lifting a baby, or in a hug, kiss or fight.
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
- Clips in sizes (`street.ts` `sizes`: things at fixed heights, the size picked by the resident's height) are fitted
  by the legs: a size's metres are the object's over its height's share of the reference times `legShare` (a child's
  legs are 0.9 of a grown-up's for the height, from 1.25 m up to 1.4 m in between), as the runtime moves the clip's
  pelvis by the body's `pelvisScale` (A-residents-12; the rides in `rides.ts` were already).
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
| runAlong | yes | 0.6 (5.8 m/s) | moving faster still (game speed 3 and up, children from 2): a run with a 3.5 m stride, each foot down a sixth of the time, played at the ground speed when the jog would need more than 3×, up to 4× (`RUN_MAX_RATE`) on a grown-up's legs and quicker on shorter ones, as the square root of their length (`gait.ts` `runMaxRate`: 5.3–5.4× on a child's, so no stage's feet slide at speed 3); back to the jog below 0.8 of the switch |
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
| buy, sell | no | 5.0 | at a vendor's (`clips/serve.ts`, sim-core `staff.rs`): the customer orders pointing at it, waits, takes it in the right hand and pays with the left; the vendor behind listens and nods, turns and reaches down into the goods, holds it out over the top and hands it over, takes the money and puts it away. Played together from the moment the vendor serves; what changes hands is drawn on the vendor's right palm between picking it up and handing it over (0.42–0.62 of the clip), on the customer's from then on (`HANDED`) |
| spell | yes | 2.1 | hands busy at a counter (cook, wash, paint, tinker) |
| chop, stir, flip | yes | 1.6, 2.0, 2.0 | the cooking steps of those names |
| eat, drink | no | 1.33, 2.0 | eating and sips (standing, and now and then as the seated upper body); the fork, spoon or chopsticks and the cup are drawn in the hand (`ACTIONS.eat`, `ACTIONS.drink` `prop`) |
| holdCup | yes | 2.5 | the cup held before the chest between sips (standing, and as the seated upper body: `ACTIONS.drink` `between`) |
| toss | no | 3.2 | a wish at the town fountain (`ACTIONS.look`): the coin looked at in the palm, held to the chest, the arm back and an underarm toss over the rim (the coin in the hand until 0.63 of it), the follow-through |
| scatter | yes | 3 | feeding the pigeons on the park bench (`ACTIONS.feed`, the seated upper body): the bag of seed on the left palm over the lap, the right hand dipping in and flinging a pinch ahead |
| shake | yes | 2 | a cocktail shaken at the bar cart and the mocktail cart (`ACTIONS.drink`): the shaker before the chest, the right hand round its tin and the left over its cap, eight hard shakes along its line and a look at it |
| carry, carryOne | yes | 1.1 | the upper body while carrying something in both hands / on the right palm (`held`, content `hold`) |
| lie | yes | 2.4 | lying (turned flat by the runtime, played at a third of the speed) |
| babyLieAsleep, babyLieAwake | yes | 9, 6 | a baby lying in the crib, asleep (still, quick breaths, a twitch) or awake (kicking, waving, turning the head); legs drawn up, hands by the head; turned flat like `lie`, played at full speed |
| foldArms | yes | 2.5 | watching, relaxing, idle variations; unamused at a joke that fell flat, unmoved by an apology or a compliment |
| yes / no | no / yes | 2.5 | listening, arguing |
| phone, lantern | yes | 2.93, 2.5 | a call, the phone at the ear (the palm over the ear, the fingers up the side of the head); reading (an open book across both palms at the chest) |
| phoneText | yes | 3.0 | texting and scrolling (`ACTIONS.phoneScreen`, content `phone.actions[].anim`): the phone held in both hands before the chest, the head bowed over it, the thumbs tapping and swiping; the upper body of it seated |
| compliment, deepTalk, flirt, sweetTalk, argue, insult, apologize, joke, introduce, askPartner, breakUp, prank | yes | 3, 4, 3.2, 3.4, 2.2, 2.4, 3.4, 2.8, 3, 3.2, 3, 2.6 | the socials' own gestures on a speaker's turn (`GESTURES`, `clips/social.ts`): an open hand offered and two claps; the fingertips together, then the chin on the hand; a hand in the hair, the other on the hip; the hands clasped at the heart, a reach to the other's arm; palms up, then a chopping hand; a dismissive flick; a hand on the heart, then open palms; the story told with the hands, then the arms thrown wide; a hand to the chest, then held out to shake; hands clasped low, then held out; a hand to the brow, then both sweeping apart; a creep, "boo!" with clawed hands, a laugh |
| laugh, shrug, rebuffed, refuse | yes | 2, 2.4, 2.6, 2.4 | how a social is taken (`GESTURES`, `REFUSED`): laughing at a joke that landed (the head back, a hand on the belly, bent over once); a teller's shrug at a joke that fell flat; turned down (the shoulders dropped, the head bowed aside, a hand to the neck); turning down a hug or a kiss (leaning away, the head turned aside, a hand up, palm out, a step back) |
| rail | yes | 2.5 | leaning on a rail (the older layouts' telescope, `USE`) |
| eyepiece181, 163, 147, 129, 112 | yes | 6 | bent down to the telescope's eyepiece (the tube aimed up: the eye at 0.765 m, 0.3 m out), the right hand on the tube, the left on the knee, a look up at the sky each loop; a size per height (cm, `STEP_IN`, for `look`) |
| pickApple177, 147, 112; pickLemon177, 147, 112 | no | 3.3 | reaching up to the fruit tree's picked fruit (the apple 1.84 m up, the lemon 1.62 m), on tiptoe if high, the left hand on the bough, a twist off and down to the chest; then `eat` (`STEP_IN`, for `eat`); a size per height |
| shower, showerChild | yes | 14 | showering: the hair lathered and rinsed, a 50° turn in two steps to soap the arms and the chest, back, the face rinsed; a child (under 1.29 m, `SHOWER_CHILD`) turns only the chest |
| strum, strumShort | yes | 2 | a guitar (or a ukulele, `strumShort`, held higher) across the body: the neck out to the left 30° up in the body's plane, the left hand on it, the right strumming; `writeHeld` draws the instrument from its rest point (`GUITAR`) to the left palm |
| push | yes | 2.67 | mopping, vacuuming |
| water, harvest, plant, kneel, crouch | yes, yes, no, yes, yes | 3.8, 2.5, 2.77, 5.2, 2.93 | gardening, smelling flowers |
| pick | yes | 3.0 | picking herbs from a waist-high planter |
| dance | yes | 1.0 | dancing, fitness games |
| punchJab, punchCross, hit | no | 0.87, 1.0, 0.33 | heavy bags and fights (the attacker's punches aimed at the other's head at any two heights by the runtime, `AIM`) |
| wave | no | 2.2 | the household creator's greetings |
| fountainDrink181, 163, 147, 129, 112 | yes | 3 | bent over a drinking fountain's spout, sipping from its water (a size per height, in cm; the smaller two on tiptoe at the water's falling end); `STEP_IN` steps them up to it |
| climb172a, 147a, 112a; climb172b, 147b, 112b | yes | 13 | a round on the climbing frame: up its rope net three ropes, hands and feet in turn, over the edge onto the deck, ducking under the roof across it to the fireman's pole, round the pole and down it (hands and legs round it), and back round to the net; a size per height (cm), and one per lane of the net (`a`, `b`: the second waits on the deck for the first to slide; one stepping in while someone goes round waits at their lane until that round is back at its start, so the two stay in step: `inTurn`) |
| dartThrow | yes | 7.2 | darts (`ACTIONS.play` at the `dartboard`): side-on at the line, three throws (aim, the forearm back, through, the hand following to the board), a step back to read the score and in again; the first dart goes as the board's darts grow in |
| pingPong | yes | 1.8 | table tennis (`ACTIONS.play` at the `pingPongTable`): the ready stance, a forehand and a backhand, the bat in the closed right hand |
| poolShot172, 147, 112 | yes | 8 | the pool table (`STEP_IN`, a lane each for two): bent over the long rail as far as the height needs, the left hand bridging on the cloth, two practice strokes and the shot, then up to chalk the cue held upright; the cue in the hands (`placeHand` `cue`); a size per height (cm) |
| foosball172, 147, 112 | yes | 3.2 | the foosball table (`STEP_IN`, a pair of rods each for two): hands on two handles, the wrists turning the rods (the right in time with the table's moving rods) and sliding them; a size per height |
| smashPlate172, 147, 112 | yes | 3.4 | the plate-smash bin (`STEP_IN`): a plate taken from the stack (mimed), up over the right shoulder and thrown overarm down into the crate; a size per height |
| boulder172, 147, 112 | yes | 16 | the bouldering wall (`STEP_IN`, up on its crash mat): hands to the start holds, the feet onto the low ones, up the wall's holds hand and foot in turn to the top, a drop to the mat onto bent knees; the route scaled to the size and put on the model's nearest holds |
| ballPit112 | yes | 12 | the ball pit (`STEP_IN`, over its rim, `over`): wading and a jump in the balls, down to sit in them tossing balls up, up again; a child's size (clips/play.ts) |
| tentPlay112 | yes | 24 | the play tent (`STEP_IN`): down on hands and knees at its door, crawling in, sitting back on the heels to play on its mat, crawling back out; a child's size |
| yoga | yes | 48 | a yoga flow on the mat (`STEP_IN` onto its front end, open hands): mountain, hands to the heart, upward salute, forward fold, half lift, a short downward dog (heels pedalling), a lunge up to warrior I, back to the dog, a child's pose, sitting on the heels, up through a squat |
| slide125, 112, 99 | yes | 14 | a round on the slide (`STEP_IN` at the run-out's foot; `clips/rides.ts`): round its right side to the ladder, up the five rungs hand and foot in turn, a step up onto the deck and stood under the roof, sat at the top holding the arch, down the chute sat up (the back leant back up to 25°, the hands on the chute's sides, the legs ahead along it), upright on the run-out, up off it and a turn back to the start; a child's sizes (cm), fitted by the legs (a child's `pelvisScale`, 0.51 of the height); a planted foot pivots on its ball as the body turns over it |
| bunkUp172, 147, 112; bunkDown172, 147, 112 | no | 7, 6.5 | the upper bunk's ladder (`BUNK_LADDER`, getting into the bed's second slot and up from it): from the tile in front round the foot end to the ladder, up its four rungs, onto the mattress on hands and knees through the gap in the guard rail, sat with the legs swung round and lain back, ending in the `lie` pose (the pelvis turned flat in the clip, so the fade to `lie` is short); down the same way backwards and a walk round to the tile, sim-core waiting for them (`LADDER_DOWN_TICKS`); placed in the bed's frame (`Player.frame`); a size per height (grown-up, teen, child) |
| rideHorse112, rideSpring112 | yes | 2.4, 2 | astride the rocking horse and the spring rider (`RIDES`, a child's): the thighs either side of the barrel, the hands on the handles, the feet down by the rockers or on the pegs; the ride's `ride` part rocks them |
| rideSeesaw147, 112 | yes | 2.8 | astride the seesaw's end, facing along it to the middle (its slot's place, content `places`), the hands on the handle bar, the feet tucked clear of the ground |
| rideRound147, 112 | yes | 2.4 | sat on the merry-go-round's platform at a slot's place, facing in between two rails: leant in, a hand on each rail, the legs out towards the column, the knees up |
| rideSwing172, 147, 112 | yes | 1.6 | the swing: both hands on the chains, pumping in time with the swing (`RIDES` `sync`: the clip held to its loop): leant back with the legs out on the way forward, tucked under the seat on the way back |
| rideBike172, 147; rideBikeChild112 | yes | 3 | riding a bike to work or school (`vehicles.ts` `MOUNTS`, sim-core `ride.rs`; the city bike in a grown-up's and a teen's size, the kids' bike in a child's): sat on the saddle (the root as on a seat over it), leant 20° (a child 14°) to the bars, the hands on the grips, the feet by the pedals with the cranks level; the runtime puts the feet on the pedals as the cranks go round with the back wheel (`pedalRide`, `LEG_CLIPS`) |
| rideScooter177, 147 | yes | 3 | riding an e-scooter (`MOUNTS.eScooter`): stood on its deck (the root raised to it), the left foot ahead of the right, leant in a little, the hands on the grips; a grown-up's and a teen's size |
| putAway177, 147, 112; putAwayGlass…, putAwayCorner…, putAwayHutch… | yes | 3.2 | clean things put away (`fetch`, `STEP_IN` up to the worktop under the wall cabinet, the glazed and the corner one, and to the china cabinet): up on tiptoe to the pulls of the doors that open, drawn open along their swing, the stack lifted from the worktop onto the shelf, a door pushed shut as the model's closes; the smallest size (a child, who can't reach) puts it on the worktop's back, looking up. Reaching up (clips/reach.ts `upFor`): on tiptoe, the shoulder lifted and leant in as far as the hands need |
| windClock177, 147, 112 | yes | 4.5 | the grandfather clock wound (`tinker`, `STEP_IN`): the key into each winding hole under the 1.77 m dial for four turns of the wrist, the other hand on the hood; the smallest pulls the weights' chains hand over hand in the trunk |
| fillFeeder177, 147, 112 | yes | 4 | the songbird feeder refilled (`garden`, `STEP_IN` at its post): the scoop (mimed) tipped twice over the house's open top, the other hand steadying it; the smallest tips it onto the seed tray |
| writeBoard177, 147, 112 | yes | 6 | planning on the whiteboard (`STEP_IN` up to it): three lines written at shoulder to head height, the marker off the board between words, the weight and the head with the hand |
| hoopShot177, 147, 112 | yes | 3.2 | "Shoot some hoops" (`STEP_IN` back out on the court, 2.5 m from the hoop's middle, walked to backwards): dribbles, the gather, the dip, a jump shot let go at the top towards the 2.94 m rim, the follow-through held; the smallest heaves it two-handed from the chest; the ball mimed. The round is the net's `swish` period |
| sitHandsLow177, 147, 112 | yes | 5 | seated at a top less than `LOW_TOP` (0.2 m for the reference height) over the seat (the kids' chair at the coffee table, `tuckLow`): bent well over it, the left hand flat on it, the right drawing; `sit`'s pelvis |
| kneelTable177, 147, 112 | yes | 6 | at the train table (`play`, `STEP_IN`, a lane each): kneeling up at its edge, leant in, the right hand pushing the engine along the track and the left on the top, now and then sitting back for a look |
| brushHair | yes | 5 | getting ready at the dressing table (the seated upper body, `ACTIONS.wash`): four brush strokes from the crown down the right side, the head tilting into each, then the left hand smoothing the other side |
| warmHands | yes | 7 | warming up standing at a fire (`ACTIONS.relax` at `WARM_AT`: the fire pit): leant a little to it, the hands held out over it, palms to the flames, turned now and then, then brought together and rubbed (clips/uses.ts) |
| plateHold177, 147, 112 | yes | 3 | stood on the vibration plate (`GRIPPED`): both hands on its grips (fore-and-aft bars 0.26 m either side, 0.96 m over the deck, 0.12 m ahead), the weight shifting a little; the plate's buzz on top (`activityPose`); a size per height |
| lieCurl | yes | 2.4 | dozing on a bed too short to lie on flat (`CURLED_BEDS`: the window seat): curled up on the left side facing the room, the knees drawn up, the head on the pillow on the left forearm; turned flat like `lie` |
| lieSide | yes | 2.4 | lying at full length on the right side (`SIDE_BEDS`: the daybed): the back to the object's back rail, the legs along the mattress, the right one straight and the left a little over it, the right forearm under the cheek on the pillow, the left hand in front of the chest; turned flat like `lie` |
| turnR30 … turnR180, turnL30 … turnL180 (every 5°), and each `…Child` | no | 0.8 (up to 90°), 1.1 (to 150°), 1.4 | turning on the spot (`turns.ts`), made in the turning body's frame: the foot on the side of the turn steps out on a low arc and the other follows (two steps, three, four for an about-turn), each planted foot turned back against the body's turn so it stays put on the floor, the hips midway between the ways the feet face, the head ahead into the turn. The feet are placed for hips a little wider for their legs than the reference's, a child's size for theirs (`TURN_HIPS`). Baked the first time a resident turns that far (`turnDef`, about a millisecond); the runtime plays the nearest size and turns by the turn itself |

### Checking clips

`tools/characters/check_clips.ts [rig.json] [--body male|female|all] [--stage <stage>[,…]|all]
[--update-baseline]` bakes the clips as the runtime does and checks loop flags, NaNs, loop seams,
lengths, sliding feet, the seated pelvis and feet (with the runtime's leg fit, `seatFit.ts`: on a 0.45 m seat, a
child's on the kids' chair, and the knees clear of every deep seat's `front`), the floor sits' knees and ankles
against the floor, `lie` and the baby's clips, and the turns on the spot, every size each way, a foot on the floor
staying put as the clip's share of the turn turns the body (the list is in its header). It also holds how far a
foot on the floor travels while a resident sets off and comes to a stop (check `travel` on `walk`: 70 mm at most
over a start, 60 mm over a stop): `tools/characters/transitions.ts` runs the residents' own code
(`residents.ts`, with a view that draws nothing) on a scripted snapshot, twelve residents per body and stage
standing, walking at 1.5 m/s and stopping at twelve parts of the stride, and adds up the way each foot's contact
point (the heel or the ball, whichever is lower) goes over the floor while it is within 4 mm of its standing
height, from a tick before the start to 0.5 s after it and from a tick before the stop to 0.9 s after it. Before
`stride.ts` that was 314 to 486 mm over a start and 193 to 278 mm on average (231 to 421 at worst) over a stop;
it is 29 to 48 mm (56 at worst) and 10 to 15 mm (41 at worst). `bun tools/characters/blend_analysis.ts` prints
the table (its section 6), with the travel of a second of plain walking beside it (12 to 47 mm: the standing
foot's toes swivel a little as the hips turn over it; 40 to 227 mm before the hip roll left the legs alone). The default is the reference: `male` at
`youngAdult`, plus `baby` for the baby's clips.
Each body and stage plays the clips on its own skeleton (`restT`, the pelvis motion and ground
speed scaled by `pelvisScale`, `walk`'s hip roll), the sized clips at the size its height picks,
babies only their own; the height slider, build, head size, stoop and the runtime's fits to seats
and beds other than the seated legs' are not covered. Planted feet are checked on each skeleton as it is, so a clip that turns the
hips over a planted foot slides on hips wider for their legs than the reference's (a child's, a little a
woman's): place the feet for them (`footIK`'s `wide`, a size per height) or keep the hips still. The known failures are in `tools/characters/clip-baseline.json`: they
print as `known` and don't fail, one that no longer fails prints as `fixed` (take it out), and only
a new one exits 1. `--update-baseline` rewrites the file for the bodies and stages checked. CI runs
`bun run check:clips` (in `web/`: `--body all --stage all`).

In dev, editing a clip file re-bakes the clips in place (Vite HMR).

## Seats and beds

`web/src/render/shared/characters/furniture.json` holds where residents sit and lie, by object id: `seats`
(`SEATS`), `defaultSeat`, `beds` (`BEDS`), `defaultBed`, `bunks` (`BUNKS`) and `footstools` (`FOOTSTOOLS`). `characters.ts` reads it with its
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
- `barStool`, `stoolCounter`, `stoolBarHigh`: a foot ring (`rest`: its top's height and how far forward its front is);
  where the feet hang over the floor, they go on it instead, the balls of the feet on its front (`REST_BACK`), if the
  legs reach it (a child's still hang).
- `weightBench`: lifting, the feet down on the floor either side of the bench, `feet` m out from its middle
  (`benchLegs`).
- `kidsBedCar`: the race car's mattress ends at its bonnet, 34 cm short of the footprint's end (`foot`).
- Hammocks (`hammock`, `cheerful.rainbowHammock`, `natureLover.gardenHammock`): lain in the sagging sling (`sling`,
  from their models' builds).
- `bedDouble`, `bedQueenUpholstered`, `bedCanopy`: their mattress tops; `bedDaybed` its daybed top, its quilt tucked in
  between the back, the arms and the rail (`tuck`: the quilt's width, no side drop showing; `foot` 0.12, the quilt ending
  just past the toes inside the foot end's arm). The daybed is lain on at full length on the side (`SIDE_BEDS`, the
  `lieSide` clip): turned a quarter about its length, its back to the back rail (a sofa by day is lounged on, not
  lain flat on the back).
- `sofaBed`: the head on the seat, the rest on the mattress pulled out in front (`bed_out`, `fwd`).
- The sofas slept on for the night (`sofa`, `loveseat`, `sofaDesign`, `sofaThree`, `sofaThreeSeat`: `SOFA_NIGHTS`, content
  "Sleep on the sofa"): lain on along the seat cushions at their height (`across`), moved towards one arm (`fwd`,
  along the sofa) so the head is by it and clear of it; curled up on the side on the two-seaters, 1.5 to 1.66 m between
  their arms (`CURLED_BEDS`, the `lieCurl` clip, a quilt of its own draped over that pose: a throw
  pulled up over the shoulder and the arm on it to the neck, the head out, ending just past the feet and hanging only a
  little over the seat cushion's front edge, `CURL_QUILT`), at full length on the back on the three-seaters (2.5 m). The blanket is 0.9 m across (`tuck`), the seat's depth
  and over its front edge, clear of the back cushions, and its foot end stops 0.3 m short of the footprint's end, inside
  the arm (`foot`).
- `gloomy.windowSeat`, `kind.hearthSettee` (2 wide): dozed on along the cushion, head to its left, whichever slot
  (the nap takes both, content `whole`).
- `bunks.bedBunk`: the lower and the upper mattress tops, `[lower, upper]` (content `bunk`).
- `footstools`: things a seated resident puts their feet up on (`FOOTSTOOLS`, `footstoolFor`, `feetUp`): their
  tops' width and depth as made (m, before turning; resized, they scale), the top at their seat's height. A
  resident sitting (`sit`, `sitTalk`) on anything else, facing a footstool in front of them that nobody sits on,
  its near half within the legs' reach (`FEET_UP`: the heels 0.14 m in from its edge, up to 0.3 m to one side,
  the legs turned to it), has the legs out with the heels on it. Not on a seat with its own legs out or
  pedals (the recliner's footrest, a chaise, a tub, a bike), facing a seat's back, pulled up to a table, at the
  desk clips, or for an instrument, the toilet, a bath or the dressing table (`NO_FEET_UP`). The sim doesn't
  know: an armchair with one in front is sat in from beside it (sim-core `approach_tiles`).

### Walking in

sim-core moves a standing resident onto a seat, into a shower, up to someone they start talking to or onto the spot
it stands them at in one tick. Drawn, a step of more than 0.25 m is walked (characters.ts `WALK_IN`, `walksIn`): the
`walk` clip at 0.8 of its pace for the body, from where they were drawn to where they will stand, then a turn to the
way sim-core faces them, and only then the seat's `sitDown` or the use's clips. The Sim is already there (the seat
is found, a chair pulled in, from the place sim-core gave them); only its drawing is on the way. To or from a tile
behind a chair built into its table or desk (a top sat at by a use of its own, `builtIn`), the step goes round a
corner 0.55 m to the chair's side (`cornerOf`), and so does rising off it (`riseViaX`). A shower with a door
(`WALK_WAIT`: the base `shower`) is waited at until the door has slid open (0.7 s), walked into through its opening
and out the same way as the use ends, when it opens again; the towel goes on once they are in and stays on until they
are out (`toweled`: while stepping out and `TOWEL_OUT`, 0.8 s, after). Not walked: a baby, one held, a resident on
directions, using a thing from inside (`STEP_IN`), rising, on a bunk's ladder, out of bed, into a bath or a seat with
the legs out, and onto a seat in a frame sat astride (pedals: the bikes; `STRADDLED`: the rocking horse, the spring
rider, the seesaw), which they settle onto as before rather than walk through.

## Where things are

| What | Files |
|---|---|
| the contract | `rig.rs`, `head.rs` (the head interface), `contract.rs`, `write.rs`, `build.rs`, `check.rs`, `check_clips.ts`, `clips/dsl.ts`, this page |
| body and clothes | `body.rs`, `cage.rs`, `subdiv.rs`, `clothes.rs`, `garments.rs`, `meshops.rs` |
| face and hair | `face.rs`, `hair.rs`, `prims.rs`; the eye, hair and brow branches of `render/pc/shaders/character.ts`; `characters/face.ts` |
| a face per resident | `identity.rs` (the channels as differences, the mask), the amounts in `head.rs` and `face.rs`; `characters/identity.ts` (the row's identity texels, the eyes), `game/household.ts` (`Appearance.face`, `randomFace`, `FACE_GROUPS`) |
| hair and hems that move | `characters/sway.ts` (the springs), `residents.ts` `writeSway`, `character.ts` `gameSimSway`, `hair.rs` `Comp::sway` |
| clips | `clips/*` (with `clips/ik.ts`) |
| setting off and stopping | `characters/stride.ts` (the gait's first phase, the legs' own fade, the feet's steps and pins), `residents.ts` (`start`, `fadeInto`, `holdFade`, the gait going on by the way gone); measured by `tools/characters/transitions.ts` |
| the runtime | `render/shared/characters/residents.ts` (activities, seats, beds, carrying, faces; its tables in `actions.ts`, `seats.ts`, `socials.ts`, `expressions.ts`, `parts.ts`; drawn through `view.ts` by `pc/characters.ts`), `shared/characters/{data,pose,outfit,blanket}.ts`, `pc/characterMaterial.ts` and `pc/shaders/character.ts` (the shading), `pc/poseTexture.ts`; the creator in `ui/screens/CreateHousehold.svelte` and `render/preview` |

See also `tools/characters/README.md`.

## Dev benches

- `/characters.html`: a grid of residents. `?family=male|female`, `?stage=<id>|all` (default: every
  stage), `?hair=all` (or `?hair=long,beard`: only those), `?tops=all`, `?skins=all`, `?builds=all`, `?face=moods|visemes`,
  `?faces=all` (each identity channel at -1 and +1 beside the shared face, and eight random faces; `?only=jawWidth,noseLength`,
  `?per=2` channels a line, `?faces=random`, `?faces=ends` for the sliders' corners; `?hairStyle=bob`, `?beard=1`; the
  `characters-identity*` pages of `tools/bench/render.mjs`),
  `?clip=<name>&t=<s>`, `?view=side|back`, `?closeup=1` (`?y=`: the height it looks at), `?build=1.2`, `?cols=`, `?manual`
  (`__bench.advance(s)`); `?hair=long,ponytail,braid&body=walk&stop=1.5` walks a sheet's residents and stops them
  (hair and hems swinging on; `&sway=0`: rigid, to compare; `&alpha=-1.57` from behind, `&alpha=0` in profile); `?hd=1` draws everyone with the close-up body (the `characters-creator` pages of
  `tools/bench/render.mjs` frame one resident as the creator's face view does, to shoot both); sets `window.__ready`.
- `/previews.html?stage=male|female&zoom=1`: the household creator's stage itself (`?hd=0`: with the body, where it
  draws the close-up body; the `previews-stage` pages).
- `/poses.html`: residents at real furniture, seats and beds, cooking, carrying, socials.
  `?only=<labels>` picks stations, `?seats=sofa,toilet` seats a man and a woman on each object,
  `?clip=<name>` plays one clip on everyone, `?clip=<name>&strip=8` shows it held at 8 times on a
  man, a woman and a child; `?top=`, `?bottom=`, `?hair=braid` dress everyone who can wear it;
  `?manual` with `__poses.advance(s)` and `__poses.focus(n)`.
