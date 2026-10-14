# Romantic pack — notes

`pack_romantic.py` builds the six P15 romantic models into `web/public/assets/models/pack_romantic/` and their
entries in `packs/pack_romantic/manifest.json`, an override pack listed after `packs/romantic/manifest.json`. Their
primitive-part placeholders (`tools/art/pack_romantic.py`, in `packs/romantic`) stay as the fallback. Built
2026-10-14; they sit in Validate › Romantic until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
Seats keep the placeholder's: the diners and the writer at 0.45 on the footprint's middle line, the bathers at
0.35 (`characters.ts` `SEATS`/`BEDS` `romantic.heartTub`), the swing's seat top at 0.45 over z = 0.

| Model | Moving part and clip | Played by |
|---|---|---|
| `candlelitTable` | the two candle flames, origin just under the shorter wick and flattened inside the wax: `candles` grows them (0.6 s) | `candlelitDinner`, `shareDessert` |
| `heartTub` | the water with its petals and bubbles, origin just under the tub's floor, flattened: `fill` raises it to 7 cm under the rim (1.2 s) | `petalBath` |
| `vanity` | the middle drawer with a lipstick and a compact in it: `drawer_open` pulls it 20 cm out (0.7 s), `hold` 2 s | `primp` |
| `gramophone` | the tone arm, resting beside the record: `needle` swings it 30° onto the record (0.8 s); the platter and record: `platter` turns them once in 1.8 s (a loop) | `listen`, `waltz` (`uses` and `loops`) |
| `letterBureau` | the writing flap over the lower pigeonholes, a letter on its inside, hinged on its foot: `fall_front` drops it 90° onto the desk towards the writer (0.8 s) | `writeLetter`, `writePoem` |
| `roseArbourSwing` | the swing (seat, back, arms, cushions, chains), pivoting on the beam: `swing` rocks it 7° each way in 2 s (a loop) and the riders ride it (`ride`); a bloom on the front left post: `cut` shrinks it away (0.6 s) | `swing` (`loops`); `tendRoses` |

The gramophone's "L the platter spins (E1)" is built: the platter is a second moving part, and the model still
takes 3 draws because nothing in it is textured. The vanity's mirror (E6) is mirror glass (`mirror`, `front`), a
flat oval on the still body.

**Draw calls: 3 each.** The still body takes its plain finishes and at most one textured finish (or the glass),
the moving parts the rest:
- the table: the chairs' velvet, the cloth and the runner are `fabric`; the table top under the cloth is plain;
- the tub: the towels are `fabric`; the water and petals are one moving part (gloss and satin);
- the vanity: the mirror takes the extra draw, so the pouf is plain satin velvet;
- the gramophone: all plain (the cabinet is satin wood colours), for the two moving parts;
- the bureau: the chair's velvet is `fabric`;
- the arbour: all plain, for the swing and the bloom: its leaves are satin, lumpy `G.boulder`s, and the swing's
  cushion is satin.

The heart tub is two prisms over a heart curve (`_prism`): the shell, hollow inside a smaller heart, on a plinth,
its floor at 0.2 m. Seated bathers (the renderer's chair pose at 0.35 m) put their shins down through the front of
the shell at x = ±0.5 where the heart narrows, as they do in the placeholder: the pose, not the model.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.romantic.candlelitTable` | 1.70 × 1.12 × 0.74 m | 4,132 | 3 | Body, Fabric, Tablecloth |
| `model.romantic.heartTub` | 1.87 × 0.78 × 1.82 m | 5,264 | 3 | Body, Fabric |
| `model.romantic.vanity` | 0.80 × 1.64 × 0.82 m | 4,404 | 3 | Body, Fabric, Rose gold |
| `model.romantic.gramophone` | 0.74 × 1.58 × 0.70 m | 2,720 | 3 | Body, Horn |
| `model.romantic.letterBureau` | 0.84 × 1.18 × 0.74 m | 3,004 | 3 | Body, Fabric |
| `model.romantic.roseArbourSwing` | 1.84 × 2.63 × 0.82 m | 7,036 | 3 | Body, Fabric |

Body is the table's gold legs, the tub's shell, the vanity's and bureau's ivory, the gramophone's cabinet and the
arbour's paint; Fabric the chairs' velvet, the towels, the pouf and the swing's cushion. The gramophone has no
fabric; its second part is the horn.

`check_glb.py`: 6 models, 0 failed, no warnings. (Its mirror-flatness check reused the name `size`, the file's size
for the summary line, so any model with mirror glass crashed the gate; renamed.) Khronos validator: 0 errors,
0 warnings each. Clips bench: every clip at rest and open on WebGPU and WebGL, the bureau and the gramophone's arm
from behind, the swing mid-rock from the side, the cut bloom zoomed. In play (`wants-asking`, WebGPU, in the yard):
the table (candlelitDinner, seated, candles lit), the tub (petalBath, seated in the filled water), the swing (swing,
seated, rocking) and the gramophone (listen) used by a resident; the vanity at all four facings, the gramophone and
bureau turned; the table, tub and arbour bought coloured; the same at night (`office-night`, WebGL).

## Rebuild

```
python3 -B tools/art/blender/build.py pack_romantic            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_romantic --preview  # previews/<id>.png, <id>@<clip>.png
```
