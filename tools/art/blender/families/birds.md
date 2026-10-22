# Birds — notes

`birds.py` builds the garden birds into `web/public/assets/models/birds/` and their entries in `packs/birds/manifest.json`:
the blue tit, chaffinch (a male), willow warbler, blackbird (a male) and collared dove (`model.bird.*`). Nothing in the content
names them: the renderer flies them about by code and perches them on the trees and shrubs (`perches`, below), so their
`Model`s take `code=True`. Built 2026-10-22.

## What the renderer gets

Each file is five rigid parts, the scene's top-level nodes, no clips:

| Node | What | Its origin |
|---|---|---|
| `body` | the torso with the wings folded on it, and the legs | between the feet, on the perch: the model's origin |
| `head` | head, bill, eyes (a blackbird's eye ring) | the neck joint |
| `tail` | the tail | its base, under the rump |
| `wing.L` | the bird's left wing, spread for flight: level, flat in the game's XZ plane, out along game −X (glTF/Blender +X) | the shoulder |
| `wing.R` | its right wing, out along game +X | the shoulder |

The bird faces the game's +Z (glTF +Z, Blender −Y), in a perched pose: the torso raised `pitch` degrees from level, the
tail hanging back and down. The spread wings are saved at scale 1. The renderer hides them (scale 0) while the bird is
perched and flaps them about the body's forward axis in flight. Part names are exact. A species' extra part would go
inside one of these five.

The entry's `bird` holds what the renderer needs of its size, in metres:

```json
"bird": { "length": 0.25, "wingspan": 0.36, "perchHeight": 0.063, "pitch": 20 }
```

- `length`: bill tip to tail tip, the field guide's number; the file measures that from its `head` to its `tail`.
- `wingspan`: tip to tip across the spread wings, the file's width.
- `perchHeight`: the body's middle over the feet. The renderer keeps flights clear of obstacles by it.
- `pitch`: how far the perched torso is raised from level, in degrees. The renderer pitches the bird nose-down by this
  much in flight to level it.

`cargo check-glb` checks all of them against the file within 5 %, and checks that the five parts are there, that no
part is saved at scale 0 and that each wing is spread to its own side.

| Model | Real (length, wingspan) | Size w × h × d, as perched | Tail | Body middle | Triangles (body, head, tail, each wing) | Draws |
|---|---|---|---|---|---|---|
| `model.bird.blueTit` | 12 cm, 18.5 cm | 0.185 × 0.072 × 0.105 m | 4.4 cm | 3.0 cm | 736 (342, 138, 44, 106); was 484 | 1 |
| `model.bird.chaffinch` | 14.5 cm, 26 cm | 0.26 × 0.071 × 0.134 m | 5.8 cm | 3.3 cm | 736; was 484 | 1 |
| `model.bird.warbler` | 11 cm, 19 cm | 0.19 × 0.054 × 0.103 m | 4.1 cm | 2.8 cm | 736; was 484 | 1 |
| `model.bird.blackbird` | 25 cm, 36 cm | 0.36 × 0.111 × 0.241 m | 9.5 cm | 6.3 cm | 852 (378, 174, 44, 128); was 500 | 1 |
| `model.bird.collaredDove` | 32 cm, 51 cm | 0.51 × 0.111 × 0.312 m | 14.3 cm | 5.9 cm | 884 (426, 158, 44, 128); was 484 | 1 |

Up to 40 fly about at once (High), so a bird keeps to half its size's target: 750 for the three songbirds (under
0.35 m); the blackbird and the dove (0.35–1 m, half the target 2,000) take more points round, as they are drawn bigger.

The depth is less than the length because the bird is perched, tilted and its tail down. A tit's or a chaffinch's
tail hangs 5–8 mm under its perch, so their `sink` is 1 cm. Each model's `centre` is its box's middle: its feet are on
the origin, and its tail reaches farther back than its bill reaches forward. No far version: a bird is a few pixels
across from the game's camera, and the gate doesn't ask for one.

## How a bird is made

- **Lofted rings, coloured by zone.** The torso, the head, the folded and the spread wings and the tail are each a tube
  of rings (`loft`). Every face takes the material of the plumage zone it is in (`zones`): the body by how far along it
  is and how far round from the top, the head by how far forward and how far round, the folded wing band by band from
  the shoulder (its outer face and its lower edge), the tail by how far along and which face (top, outer edges, under),
  the spread wing by how far out and which band front to back. So the markings are one mesh per part, with no textures.
- **The detail pass (2026-10-22).** Same parts, origins, pivots and `bird` facts as before (the node translations are
  the old files' to the digit: the joints are placed by the old straight-line body, `body_point(smooth=False)`); more
  rings and shape:
  - the torso 10 points round (12: blackbird, dove) through nine stations, its fullness eased between them (a monotone
    curve: a plain Hermite bulged 2 mm over the stations and failed the height);
  - the head 10 (12) points round through five stations, its bands placed on the markings (crown, over the eye,
    through the eye, cheek, throat); a little flatter than round (0.86) so the crown stands where the old eight-sided
    head's did;
  - the bill a six-sided loft with its ridge curving to the tip; each foot three toes forward and one back;
  - the folded wing a five-point lens in six bands: coverts, the bar, flight feathers with their pale lower edges;
  - the tail a flat six-sided fan, its outer feathers' edges a zone of their own, its end notched (tit, finch,
    warbler) or rounded (blackbird, dove: `tailEnd`);
  - the spread wing an aerofoil through eleven stations, its trailing edge stepping out and in at each feather's end
    (secondaries, then the primaries fanning from the wrist), its top in three bands (coverts, the greater coverts'
    tips where a bar is, flight feathers).
- **Markings, true to life.** Blue tit: blue crown ringed white, white forehead, black stripe through the eye to the
  nape, white cheek, the dark blue collar from the nape round to the black bib, green-olive back, yellow breast with
  the dark line down the belly, blue wing with one white bar and paler-edged flight feathers. Chaffinch (male):
  blue-grey crown and nape, black forehead, rusty-pink face and breast fading to a pale belly, chestnut mantle,
  olive-green rump, black wing with the white shoulder patch, the white bar and the yellowish edges, white outer tail
  feathers. Willow warbler: olive above, pale yellow eyebrow, dusky eye-stripe, yellowish throat and breast, whitish
  belly. Blackbird (male): black, orange bill and eye ring. Collared dove: buff-grey, the narrow black half-collar with
  its white edge, a blue-grey band along the arm, dark primaries, the tail pale-ended above and black at the base with a
  broad white end underneath.
- **Materials.** There is one material per zone (`BASE`): `back`, `rump`, `cap`, `brow`, `mask`, `face`, `throat`,
  `nape`, `collar`, `breast`, `belly`, `wing`, `bar`, `flight`, `edge`, `tip`, `under`, `tail`, `outer`, `tailtip`,
  `undertail`, `bill`, `eye`, `ring`, `leg`. Each species colours them itself (`palette`); a zone added in the detail
  pass that a species doesn't colour takes the zone it stands for (`SAME`: `rump` the back, `edge` the flight
  feathers, `outer` and `undertail` the tail). They are all plain finishes (matte feathers, satin bill and legs, a
  gloss eye), which the game merges into one draw call for the whole bird. A bird uses 14–19 of them.
- **Sizes.** Each species has its field-guide length and wingspan and a few numbers that shape it (`BLUE_TIT` …). The
  tail is as long as makes the bird its length, bill tip to tail tip (`_reach`). Each tail comes out near the real one:
  tit 4.4 cm (real about 5), chaffinch 5.8 (6.5), warbler 4.1 (4.8), blackbird 9.5 (10), dove 14.3 (14). Spread wings
  are rounded for the songbirds, longer and pointed for the dove (`ROUNDED`, `POINTED`).

## Looking at them

`cargo art birds --preview` renders each bird into `previews/birds/<id>.png`: the front three-quarter with its wings
spread. `views` adds `<id>.side.png`, `.front.png` and `.three_quarter.png` perched (the wings scaled away, as the game
draws a perched bird), `.top.png` spread, and `.at6m.png` and `.at20m.png`: the bird on a lawn beside a 1.77 m figure,
seen as the game's camera sees it (a 0.8 rad field of view, 38 degrees down). At real size, a tit is a few pixels
across at 20 m and a blackbird or a dove about ten. In the game's gallery (`/gallery.html?keys=model.bird.blackbird&scale=adult`,
or `--camera game`) they draw as built.

## Rebuild

```
cargo art birds            # the .glb files, the pack, check-glb
cargo art birds --preview  # previews/birds/<id>.png and the views above
```
