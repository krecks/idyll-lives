# Loner pack — notes

`pack_loner.py` builds the six P11 loner models into `web/public/assets/models/pack_loner/` and their entries in
`packs/pack_loner/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
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
| `soloSauna` | the glass door with its cedar pull, hinged on its post: `door_open` swings it 60° out (0.8 s), `hold` 1.5 s and `out` 1.5 s: it lets the bather in, shuts behind them and opens to let them out. The cabin is 1.64 m deep, the door at depth 0.7, so the seated bather's feet stay inside it (2026-10-21: the door at 0.42, the shoes at and past the glass: I-packs-17) | `sweat` |
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

**Warnings left (`check-glb`):**
- `shortwaveRadio`: height 1.33 m against the old part's 1.77 m. The antenna is down at rest and reaches 1.77 m
  when `antenna` runs it up for `airwaves`; the old part always has it up.
- `soloSauna`: 4 draw calls (above).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.loner.quietPod` | 0.84 × 1.55 × 0.78 m | 7,472 | 3 | Body, Wood, Cushions |
| `model.loner.shortwaveRadio` | 0.84 × 1.33 × 0.68 m | 3,526 | 3 | Body, Wood, Seat |
| `model.loner.puzzleTable` | 0.86 × 1.01 × 0.72 m | 4,530 | 3 | Body, Wood |
| `model.loner.bonsaiBench` | 0.86 × 1.31 × 0.75 m | 4,706 | 3 | Body, Wood, Pot |
| `model.loner.soloSauna` | 0.94 × 2.09 × 1.70 m | 5,004 | 4 | Wood, Trim |
| `model.loner.deliveryHatch` | 0.84 × 1.95 × 0.84 m | 2,680 | 3 | Body, Wood |

The plan's colour parts are "Body, Wood" for all six; the sauna is all wood, so its parts are the cedar (Wood)
and the dark cedar trim (the door pull).

`check-glb`: 6 models, 0 failed, the two warnings above. Khronos validator: 0 errors, 0 warnings each.
On the clips bench (WebGPU and WebGL), every clip at rest and open, the radio's dial from behind; colours
on the bench (`&colors=`). In play (`wants-asking`, WebGPU): all six bought in the living room; the pod (`retreat`:
the hood swings over the seated reader), the sauna (`sweat`: the door opens and shuts after 1.5 s with the resident
seated inside), the radio (`tune`) and the hatch (`order`: it opens as the resident arrives) used. At night
(`office-night`, WebGL): the sauna at all four facings in the yard, the other five coloured.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit, keeping every key, size, centre, part, hinge, clip, `motions` and `paint`
(triangles before → after: pod 3,300 → 7,472, radio 3,564 → 3,526, puzzle 2,516 → 4,530, bonsai 5,056 → 4,706,
sauna 3,728 → 5,004, hatch 2,232 → 2,680; draws unchanged). The radio desk stays under its 4,000 (0.35–1 m) target;
the bonsai's pads are jittered boulders, cheaper than its spheres were.

- `quietPod`: the shell in soft panels (a piped bucket, the back, wings with rolled front edges and piping, linings),
  a turned foot and stem; a piped seat cushion (top 0.45), a buttoned back cushion, a head roll; a book, a folded
  blanket, a gooseneck lamp. The hood: padded channels (soft) and a rolled lip.
- `shortwaveRadio`: a spindle chair swept in one piece (shaped seat, raked posts, bowed crest), a desk on splayed
  legs; the receiver with a chrome-bezelled dial and its scale, a meter with its needle, turned knobs, a slotted
  grille, a bent carry handle; a lofted microphone on a swept stem, headphones on a swept band with a cord, a logbook
  and a pencil, a mug. The dial is a turned knob with a skirt.
- `puzzleTable`: jigsaw pieces with knobs and sockets that nest (outlines), loose pieces, the box with a sun on its
  picture, the magnifier's clamp and joint; the same chair and desk as the radio's.
- `bonsaiBench`: a slatted teak top and shelf, a back rail; a lofted tray pot on feet, roots gripping it, a tapering
  twisting trunk (one sweep, jittered) and swept branches, jittered foliage pads; a coil of copper wire, a mister with
  its trigger, shears with blades and loops, rimmed spare pots, a soft sack with its top rolled. The twig: a tapered
  sweep and jittered tufts.
- `soloSauna`: the side walls in boards with a dark base board, a fascia round the roof; slats rounded, the bench's
  and tier's skirts in upright boards; the heater's vents and a cedar guard, more stones; a banded bucket and a cupped
  ladle; a louvred lamp, a thermometer with its face and needle; a towel over the bench's edge. The door: a bent cedar
  pull on roses.
- `deliveryHatch`: a moulded frame round the hatch (`kit.moulding`), a nosed cap, scroll brackets under the ledge, a
  soft bag, a lidded cup with a band, a bezelled screen, plaque screws, a drawer pull and letter flap, a bordered mat.
  The door: its window framed, a bar pull.

`check-glb`: 6 models, 0 failed; the sauna's 4-draw warning (as before). Khronos validator: 0 errors, 0 warnings
each.

## Rebuild

```
cargo art pack_loner            # the six .glb, the pack, check-glb
cargo art pack_loner --preview  # previews/pack_loner/<id>.png, <id>@<clip>.png
```
