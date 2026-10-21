# Romantic pack — notes

`pack_romantic.py` builds the six P15 romantic models into `web/public/assets/models/pack_romantic/` and their
entries in `packs/pack_romantic/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sat in Validate › Romantic until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
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
| `roseArbourSwing` | the swing (seat, back, arms, cushions, chains), pivoting on the beam: `swing` rocks it 7° each way in 2 s (a loop) and the riders ride it (`ride`); a bloom on the front left post: `cut` shrinks it away (0.6 s) | `swing` (`loops`); `tendRoses` |

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
- the arbour: all plain, for the swing and the bloom: its leaves are satin, lumpy `G.boulder`s, and the swing's
  cushion is satin.

The heart tub is two prisms over a heart curve (`_prism`): the shell, hollow inside a smaller heart, on a plinth,
its floor at 0.2 m. Seated bathers (the renderer's chair pose at 0.35 m) put their shins down through the front of
the shell at x = ±0.5 where the heart narrows, as they do in the old part: the pose, not the model.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.romantic.candlelitTable` | 1.80 × 1.10 × 0.80 m | 4,216 | 3 | Body, Fabric, Tablecloth |
| `model.romantic.heartTub` | 1.87 × 0.78 × 1.82 m | 5,264 | 3 | Body, Fabric |
| `model.romantic.vanity` | 0.80 × 1.64 × 0.82 m | 4,404 | 3 | Body, Fabric, Rose gold |
| `model.romantic.gramophone` | 0.74 × 1.58 × 0.70 m | 2,720 | 3 | Body, Horn |
| `model.romantic.letterBureau` | 0.84 × 1.18 × 0.91 m | 2,788 | 3 | Body, Fabric |
| `model.romantic.roseArbourSwing` | 1.84 × 2.63 × 0.82 m | 7,036 | 3 | Body, Fabric |

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

## Rebuild

```
cargo art pack_romantic            # the six .glb, the pack, check-glb
cargo art pack_romantic --preview  # previews/pack_romantic/<id>.png, <id>@<clip>.png
```
