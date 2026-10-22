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

| Model | Real (length, wingspan) | Size w × h × d, as perched | Tail | Body middle | Triangles (body, head, tail, wings) | Draws |
|---|---|---|---|---|---|---|
| `model.bird.blueTit` | 12 cm, 18.5 cm | 0.185 × 0.072 × 0.106 m | 4.4 cm | 3.0 cm | 484 (234, 86, 28, 68 + 68) | 1 |
| `model.bird.chaffinch` | 14.5 cm, 26 cm | 0.26 × 0.072 × 0.135 m | 5.8 cm | 3.3 cm | 484 | 1 |
| `model.bird.warbler` | 11 cm, 19 cm | 0.19 × 0.054 × 0.103 m | 4.1 cm | 2.8 cm | 484 | 1 |
| `model.bird.blackbird` | 25 cm, 36 cm | 0.36 × 0.11 × 0.241 m | 9.5 cm | 6.3 cm | 500 (head 102: the eye rings) | 1 |
| `model.bird.collaredDove` | 32 cm, 51 cm | 0.51 × 0.11 × 0.312 m | 14.3 cm | 5.9 cm | 484 | 1 |

The depth is less than the length because the bird is perched, tilted and its tail down. A tit's or a chaffinch's
tail hangs 5–8 mm under its perch, so their `sink` is 1 cm. Each model's `centre` is its box's middle: its feet are on
the origin, and its tail reaches farther back than its bill reaches forward. No far version: a bird is a few pixels
across from the game's camera, and the gate doesn't ask for one.

## How a bird is made

- **Lofted rings, coloured by zone.** The torso, the head, the folded and the spread wings and the tail are each a tube
  of rings (`loft`), eight points round for the body and the head. Every face takes the material of the plumage zone it
  is in (`zones`): the body by how far along it is and how far round from the top, the head by how far forward
  and how far round, and the wings and tail by length. So the blue tit's blue cap, white brow and cheek, dark eye
  stripe and yellow breast, the chaffinch's blue-grey cap, pink-russet face and breast and white wing bars, the
  warbler's olive back, pale yellow eyebrow and underside, the blackbird's black with an orange bill and eye ring, and
  the dove's buff-grey, black half-collar and dark wingtips are all one mesh per part, with no textures.
- **Materials.** There is one material per zone (`BASE`): `back`, `cap`, `brow`, `mask`, `face`, `throat`, `nape`,
  `collar`, `breast`, `belly`, `wing`, `bar`, `flight`, `tip`, `under`, `tail`, `tailtip`, `bill`, `eye`, `ring`, `leg`.
  Each species colours them itself (`palette`). They are all plain finishes (matte feathers, satin bill and legs, a
  gloss eye), which the game merges into one draw call for the whole bird. A bird uses 13–17 of them. gamekit has no
  vertex colours, so a part takes up to nine materials (glTF primitives) where the brief asked for 1–2, but it is still
  that one draw.
- **Sizes.** Each species has its field-guide length and wingspan and a few numbers that shape it (`BLUE_TIT` …). The
  tail is as long as makes the bird its length, bill tip to tail tip (`_reach`). Each tail comes out near the real one:
  tit 4.4 cm (real about 5), chaffinch 5.8 (6.5), warbler 4.1 (4.8), blackbird 9.5 (10), dove 14.3 (14). Tail shapes are
  square-ended for the tit, finch and warbler, long and round-ended for the blackbird, long and broad with a pale end for
  the dove. Spread wings are rounded for the songbirds, longer and pointed for the dove (`ROUNDED`, `POINTED`).

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
