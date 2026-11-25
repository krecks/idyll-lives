# Romantic pack — notes

`crates/art/packs/src/pack_romantic` builds the six P15 romantic models into `web/public/assets/models/pack_romantic/` and their
entries in `packs/pack_romantic/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sat in Validate › Romantic until checked in play.

Each is built where the game places it (`F`, game axes; `settle!(centre = false)` and `.centre(..)` on its `Model`).
Seats: the diners at 0.45 on a slipper chair at each end of the candlelit table, facing each other across its
0.8 × 0.72 top (content `places` `[±0.62, 0.1]` facing ±90; G-39, 2026-10-21: they sat side by side behind a
0.3 m top before, their legs through its front skirt), the writer at 0.45 in front of the bureau at depth 0.2, facing it (`furniture.json` `back`, the hips at 0.25; M-packs-05), the
bathers at 0.20 (`furniture.json` `romantic.heartTub`), the swing's seat top at 0.45 over z = 0.

| Model | Moving part and clip | Played by |
|---|---|---|
| `candlelitTable` | the two candle flames, origin just under the shorter wick and flattened inside the wax: `candles` grows them (0.6 s); they glow at night (`flame`, `flame_core` `glow`; M-packs-07) | `candlelitDinner`, `shareDessert` |
| `heartTub` | the water with its petals and bubbles, origin just under the tub's floor, flattened: `fill` raises it to 7 cm under the rim (1.2 s) | `petalBath` |
| `vanity` | the middle drawer with a lipstick and a compact in it: `drawer_open` pulls it 20 cm out (0.7 s), `hold` 2 s | `primp` |
| `gramophone` | the tone arm, resting beside the record: `needle` swings it 30° onto the record (0.8 s); the platter and record: `platter` turns them once in 1.8 s (a loop) | `listen`, `waltz` (`uses` and `loops`) |
| `letterBureau` | the writing flap over the lower middle pigeonholes, a letter on its inside, hinged on its foot: `fall_front` drops it 90° onto the desk towards the writer (0.8 s). Its back on the wall, the hutch on the desk's back, the chair in front | `writeLetter`, `writePoem` |
| `roseArbourSwing` | the swing (seat, back, arms, cushions, chains), pivoting on the beam: `swing` rocks it 7° each way in 2 s (a loop) and the riders ride it (`ride`); a bloom on the front left post, 1.15 m up: `cut` shrinks it away (0.6 s), 0.6 s into the use, for the one tending that post (`slot` 0) | `swing` (`loops`); `tendRoses` |

The gramophone's "L the platter spins (E1)" is built: the platter is a second moving part, and the model still
takes 3 draws because nothing in it is textured. The vanity's mirror (E6) is mirror glass (`mirror`, `front`), a
flat oval on the still body, `gloss` as every file mirror: see-through `glass` showed the rose-gold back through it
on Medium and Low, where nothing reflects.

**Draw calls: 3 each.** The still body takes its plain finishes and at most one textured finish (or the glass),
the moving parts the rest:
- the table: the chairs' velvet, the cloth and the runner are `fabric`; the table top under the cloth is plain;
- the tub: the towels are `fabric`; the water and petals are one moving part (gloss and satin);
- the vanity: 2 (the mirror is plain gloss now), its pouf plain satin velvet;
- the gramophone: all plain (the cabinet is satin wood colours), for the two moving parts;
- the bureau: the chair's velvet is `fabric`;
- the arbour: all plain, for the swing and the bloom: its leaves are satin, lumpy `boulder!`s, and the swing's
  cushion is satin.

The heart tub is two prisms over a heart curve (`loft!` through `_heart_ring`s): the shell, hollow inside a smaller heart, on a plinth,
its floor at 0.2 m. Seated bathers (the renderer's chair pose at 0.35 m) put their shins down through the front of
the shell at x = ±0.5 where the heart narrows, as they do in the old part: the pose, not the model.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.romantic.candlelitTable` | 1.80 × 1.10 × 0.80 m | 5,594 (was 4,216) | 5,594 | Body, Fabric, Tablecloth |
| `model.romantic.heartTub` | 1.87 × 0.78 × 1.82 m | 6,260 (was 5,264) | 6,260 | Body, Fabric |
| `model.romantic.vanity` | 0.80 × 1.62 × 0.82 m | 4,238 (was 4,404) | 4,670 | Body, Fabric, Rose gold |
| `model.romantic.gramophone` | 0.74 × 1.58 × 0.70 m | 4,884 (was 2,720) | 4,884 | Body, Horn |
| `model.romantic.letterBureau` | 0.84 × 1.19 × 0.91 m | 3,018 (was 2,788) | 3,018 | Body, Fabric |
| `model.romantic.roseArbourSwing` | 1.83 × 2.61 × 0.82 m | 11,734 (was 7,036) | 12,528 | Body, Fabric |

Body is the table's gold legs, the tub's shell, the vanity's and bureau's ivory, the gramophone's cabinet and the
arbour's paint; Fabric the chairs' velvet, the towels, the pouf and the swing's cushion. The gramophone has no
fabric; its second part is the horn.

`check-glb`: 6 models, 0 failed, no warnings. (Its mirror-flatness check reused the name `size`, the file's size
for the summary line, so any model with mirror glass crashed the gate; renamed.) Khronos validator: 0 errors,
0 warnings each. Clips bench: every clip at rest and open on WebGPU and WebGL, the bureau and the gramophone's arm
from behind, the swing mid-rock from the side, the cut bloom zoomed. In play (`wants-asking`, WebGPU, in the yard):
the table (candlelitDinner, seated, candles lit), the tub (petalBath, seated in the filled water), the swing (swing,
seated, rocking) and the gramophone (listen) used by a resident; the vanity at all four facings, the gramophone and
bureau turned; the table, tub and arbour bought coloured; the same at night (`office-night`, WebGL).

## Detail pass (2026-10-22)

Roses are three thick turned cups (outer petals flaring, inner, bud: thick, so their insides show from above) with
`leaf!` leaves. Same keys, sizes, parts, clips, paint and draws.
- **Candlelit table:** the cloth lofted in one piece, rolling over the edge and falling in soft folds; the runner draped
  over both ends; chairs with a stuffed seat and a rounded back (extruded, bevelled) on tapered legs; plates turned with
  a gold rim, a dessert with cream and a berry, forks and knives, wine in turned glasses; dripped candles and teardrop
  flames; petals as leaf blades on the cloth.
- **Heart tub:** the shell lofted through heart-shaped rings (up the outside, over a rolled rim, down to the floor) on an
  eased plinth; the ice bucket with ring handles, a lofted bottle with foil; rolled towels; petals on the water as petal
  blades, heaps of foam.
- **Vanity:** a rounded top, the apron scalloped either side of the drawer, `kit::cabriole!` legs, the oval mirror in a
  swept moulded frame on a stand, lofted perfume bottles with turned stoppers, a necklace, a buttoned pouf on a ring with
  feet, a `kit::knob!` and a lipstick and compact in the drawer.
- **Gramophone:** a moulded cabinet top, raised-panel doors, brass corners, a bent crank, the horn's neck swept and its
  petals' seams, a grooved record, a turned sound box with its face.
- **Letter bureau:** the apron's lower edge shaped, tapered square legs, an arched crest, envelopes standing tied with
  ribbon, the flap a `kit::shaker!` with a knob, a quill with a vane (`leaf!`), a turned wax seal.
- **Rose arbour:** the arches swept, the lattice laths each one sweep, climbing canes up the posts with leaves in threes
  and blooms, a leafy arch; the swing slatted with spindles between rails, swept arms, a stuffed cushion and pillows.
  11,734 triangles (its class, over 2.5 m, is 12,000).

## The roses tended at a post (I-packs-12, 0.5.0)

*Tend the climbing roses* was played at the seat cushion from the tile in front. The tender now steps in to a front
post (`STEP_IN` lanes at -0.84 and 0.84 m, 0.75 m out, a child 0.58, in `characters/actions.ts`) and reaches to the
bloom there (`cutRose177`, `147`, `112` in `clips/uses.ts`). `BLOOM` came down from 1.36 to 1.15 m so a child's palm
(1.14-1.18 m at full stretch) reaches it, and `cut` waits 0.6 s for the hand and plays only for slot 0 (the left
post's tender). The swing keeps no `places`: a place serves every use of its slot, and the sitters' would be on the
footprint, where a tender can't stand. 11,734 triangles, 3 draws, as before.

## Rebuild

```
cargo art pack_romantic            # the six .glb, the pack, check-glb
cargo art pack_romantic --preview  # previews/pack_romantic/<id>.png, <id>@<clip>.png
```

## The soft forms pass (2026-11-13)

The rose arbour swing's pillows are sewn flat and stuffed (`waist`), its seat cushion `resting`. Besides, its soft boxes take the shared form's new shape (the shared `soft_box`: rounded edges turning in equal parts, so the same triangles stand closer to the curve; puffed boxes' corners pinched in; cords following the seams). Same sizes (to 1.2 mm), parts, clips, seat and mattress heights (`cargo review sizes` and `uses` unchanged). 3 models changed shape: `candlelitTable`, `letterBureau`, `roseArbourSwing`; 20,346 to 21,140 triangles.

## Gate to zero (2026-11-25)

Every hard edge the gate's `edges` found is eased (56 → 0). The heart tub's outline is laid evenly along its length
and smoothed over the four points either side of its point and its dip (moulded acrylic has no sharp corner), and its
rim's rolls and the plinth's top are drawn in square to the outline (`_heart_in`), not scaled about the middle, which
narrowed them to a knife edge at the point and the dip; the cloth rolls over the table's edge in four steps; the
mirror's moulding (`bevel_edges`), its back, the apron, the crest, the bureau's apron and the arbour's rafters (a
chamfered section, eased ends) are bevelled; the vases', the wine glasses', the horn's (`eased`) and the cut bloom's
rims are rolled (a rose 5.5 cm or more round only: a smaller one's petal edges are under 5 cm a side); the perfume
bottles take 12 sides. The arbour's climbing leaves take two rows (11,956 triangles, under its 12,000 aim; was
12,528).

Sizes (`cargo review sizes`): the candlelit table stands 1.6 cm lower (the cloth's top 0.756, the runner's 0.759, a
dining table's; the candles' tops 1.09); the vanity's pouf and the gramophone's record sleeves are `aside` (drawn, not
the piece's own size, no draw call more), the vanity measuring 0.80 × 1.62 × 0.37 m and the gramophone 0.66 × 1.50 ×
0.55 m; the gramophone's horn sits 4 cm lower on a shorter neck.
