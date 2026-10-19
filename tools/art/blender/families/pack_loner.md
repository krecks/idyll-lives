# Loner pack — notes

`pack_loner.py` builds the six P11 loner models into `web/public/assets/models/pack_loner/` and their entries in
`packs/pack_loner/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sit in
Validate › Loner until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
keeping its footprint, its wall side (the pod, the bench and the hatch against the wall face at depth −0.43, the
sauna at −0.93 in its 1 × 2) and the seats at 0.45 m (the pod's cushion, the radio and puzzle chairs at the tile's
middle, the sauna's bench). The pod stood 3 cm into the wall; it is centred on its `Model` `centre` now
(`G.settle()`), its back on the face, as the wall-standing ones' old parts stood, so the model and the wall
agree within a centimetre.

| Model | Moving part and clip | Played by |
|---|---|---|
| `quietPod` | the pram hood, an arc of padded ribs pivoting across the wings: `shell` swings it 40° forward over the reader (0.9 s) | `retreat`, `read` |
| `shortwaveRadio` | the tuning knob on the receiver's face: `dial` turns it 150° (1 s); the antenna's inner section: `antenna` runs it 45 cm up out of its tube (1 s) | `tune` (dial); `airwaves` (antenna and dial) |
| `puzzleTable` | a patch of seven pieces, built flat across with its origin on its left edge: `pieces` grows it across the gap in the puzzle (0.9 s) | `piece`, `newPuzzle` |
| `bonsaiBench` | a leggy twig with two tufts, origin where it leaves the pad: `trim` shrinks it away (0.7 s); `mist` nods it 10° each way (1.2 s, a loop) | `shape`; `tend` (loop) |
| `soloSauna` | the glass door with its cedar pull, hinged on its post: `door_open` swings it 60° out (0.8 s), `hold` 1.5 s | `sweat` |
| `deliveryHatch` | the steel hatch door, hinged on its edge: `hatch` swings it 105° open (0.8 s) over a recess with a parcel, `hold` 2 s | `order`, `treat` |

`tend` (Mist and feed) nods the twig under the mister for as long as it lasts (`loops`), on the part `trim`
already moves, so it costs no draw. A mist puff of its own would be a fourth draw.

**Draw calls: 3 each, the sauna 4.**
- The pod: its upholstery (`shell`, fabric) on the still body and the hood; its foot is plain satin oak.
- The radio desk has two moving parts, so it is all plain (its oak is satin), 1 + 1 + 1.
- The puzzle table: textured oak top and chair, plain sage frame; the patch of pieces plain. The magnifier's lens
  is plain gloss (no glass).
- The bonsai bench: its leaves (`leaf`, foliage) take the textured draw; the trunk is plain satin (no bark) and the
  bench plain satin teak. The twig is `leaf` only.
- The sauna: textured cedar plus plain on the still body, and the door's glass and its plain cedar pull: 4. The
  glass is the point (the resident is seen sweating inside); the side windows of the old part are solid cedar
  (a fifth draw otherwise).
- The hatch: plain slate cabinet with a textured oak ledge and cap; the door plain.

**Warnings left (`check_glb`):**
- `shortwaveRadio`: height 1.33 m against the old part's 1.77 m. The antenna is down at rest and reaches 1.77 m
  when `antenna` runs it up for `airwaves`; the old part always has it up.
- `soloSauna`: 4 draw calls (above).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.loner.quietPod` | 0.84 × 1.54 × 0.77 m | 3,300 | 3 | Body, Wood, Cushions |
| `model.loner.shortwaveRadio` | 0.84 × 1.33 × 0.68 m | 3,564 | 3 | Body, Wood, Seat |
| `model.loner.puzzleTable` | 0.87 × 1.01 × 0.72 m | 2,516 | 3 | Body, Wood |
| `model.loner.bonsaiBench` | 0.86 × 1.31 × 0.75 m | 5,056 | 3 | Body, Wood, Pot |
| `model.loner.soloSauna` | 0.94 × 2.09 × 1.43 m | 3,728 | 4 | Wood, Trim |
| `model.loner.deliveryHatch` | 0.84 × 1.95 × 0.84 m | 2,232 | 3 | Body, Wood |

The plan's colour parts are "Body, Wood" for all six; the sauna is all wood, so its parts are the cedar (Wood)
and the dark cedar trim (the door pull).

`check_glb.py`: 6 models, 0 failed, the two warnings above. Khronos validator: 0 errors, 0 warnings each.
On the clips bench (WebGPU and WebGL), every clip at rest and open, the radio's dial from behind; colours
on the bench (`&colors=`). In play (`wants-asking`, WebGPU): all six bought in the living room; the pod (`retreat`:
the hood swings over the seated reader), the sauna (`sweat`: the door opens and shuts after 1.5 s with the resident
seated inside), the radio (`tune`) and the hatch (`order`: it opens as the resident arrives) used. At night
(`office-night`, WebGL): the sauna at all four facings in the yard, the other five coloured.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_loner            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_loner --preview  # previews/<id>.png, <id>@<clip>.png
```
