# Outdoor (`families/outdoor.py`)

The outdoor things, built over their primitive-part placeholders (`catalog.py`, `catalog2.py`, `furniture.py`),
which stay the fallback. One pack, `packs/outdoor/manifest.json`. Each model is built in place where its
placeholder stands (`F`, game axes), so seat heights (`catalog2.py` `SEATS`) and table tops (content `surface`)
stay where the game has them; models that stand off the middle of their footprint say so with `centre=`.

## OU1. Patio seating and tables (Validate › Patio)

Built 2026-10-14; they sit in Validate › Patio until signed off (back to Outdoor › Seating).

- **Park bench** (`bench`): cast-iron ends (legs, arm with a rounded end and a scroll, a stretcher, feet), four
  seat slats at 0.47 m and three back slats leaning 8°.
- **Folding bistro set** (`patioSet`): a slatted table on crossed iron legs (top at 0.72 m, where the content's
  `surface` is) between two folding chairs facing it; a cup and a jug on the table.
- **Picnic table** (`picnicTable`): five planks at 0.63 m, a two-plank bench each side, A-frames with bearers,
  a centre cleat and braces; a red gingham cloth, a basket and two cups.
- **Garden chair** (`gardenChair`) and **Adirondack chair** (`adirondackChair`): one slatted frame (seat sloping
  back, wide arms, stringers, braces); the garden chair is white with five parallel back slats and a top rail
  and a cup on its arm, the Adirondack green with a fanned, arched back.
- **Round patio table** (`patioTableRound`): a teal top with a ring of yellow and cream tiles, an iron rim, a
  pedestal and a domed foot. It is coloured as Frame, Top and Tiles (not "Slats": it has none).
- **Teak patio table** (`patioTable4`): five planks at 0.74 m on aprons, square legs and a low stretcher.
  `check_glb.py` warns it is 4 cm off centre: it follows the content's top (−0.43 to 0.35 deep); the
  placeholder's legs stood out past the front of it.
- **Patio chair** (`patioChair`): a bent green-iron frame, three back slats and a seat cushion (Frame, Cushion).
- **Garden bench** (`benchGarden3`): three seat and three back slats on three iron frames, arms at the ends.
- **Outdoor bar** (`outdoorBar`): a teak body with twenty front slats, a lighter counter at 1.06 m (the content's
  bar top), a back shelf and a steel foot rail (Body, Counter, Foot rail).
- **Parasol** (`parasol`): a weighted base and pole, an octagonal canopy on eight ribs and struts, with a valance.
  The canopy is its own part, hinged on the pole at 1.3 m: `stir` rocks it 1.2° in 2.4 s, an ambient clip
  (`idle`), so it moves in the wind near the camera (not on Low).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| bench | 1.74 × 0.89 × 0.53 | 2468 | 2 | Frame, Slats |
| patioSet | 0.60 × 0.93 × 1.86 | 2952 | 2 | Frame, Slats |
| picnicTable | 2.50 × 0.96 × 1.62 | 2908 | 3 | Frame, Planks, Cloth |
| gardenChair | 0.84 × 1.04 × 0.76 | 2320 | 1 | Chair |
| patioTableRound | 0.82 × 0.73 × 0.82 | 1624 | 1 | Frame, Top, Tiles |
| patioTable4 | 1.60 × 0.74 × 0.79 | 1728 | 1 | Frame, Top |
| patioChair | 0.46 × 0.93 × 0.45 | 1336 | 2 | Frame, Cushion |
| adirondackChair | 0.88 × 1.10 × 0.79 | 2160 | 1 | Chair |
| benchGarden3 | 2.70 × 0.91 × 0.60 | 2712 | 2 | Frame, Slats |
| outdoorBar | 2.00 × 1.06 × 0.89 | 3052 | 2 | Body, Counter, Foot rail |
| parasol | 2.21 × 2.46 × 2.21 | 1376 | 3 | Canopy, Pole, Edge |

`check_glb.py`: 0 failed (the one warning above). Khronos validator: 0 errors, 0 warnings each.

**In the game** (2026-10-14, headless, WebGPU and WebGL2): the gallery next to the placeholders; the clips bench
(`stir`); in play, all eleven bought in a garden (one chair at each of the four facings, three recoloured), a
resident relaxing in the Adirondack chair, sitting on both benches, the bistro set and the patio chair (at the
round table), and having a picnic at the picnic table, by day and on into the night.

## OU2. Loungers, swings and the hot tub (Validate › Loungers)

Built 2026-10-14; in Validate › Loungers until signed off (back to Outdoor › Seating; the swing set to Outdoor › Play).

- **Sun lounger** (`sunLounger`): a slatted frame, a seat cushion and a beach towel; the back is its own part, an
  upholstered pad and pillow hinged at the seat's end, raised 35°. `recline` lowers it 30° (nearly flat) for
  sunbathe and doze, so the lying resident (`BEDS` 0.35 m) lies along it; it rises again after.
- **Hammock** (`hammock`): a wooden stand; the sling (a sagging striped mesh, spreader bars, ropes, a pillow) is
  hinged on the line through the hooks (1.1 m). `sway` rocks it ±8° in 2 s, a loop for swing and doze, and the
  resident rides it (`ride`). They lie in the sling's dip (`characters.ts` `BEDS.hammock.sling`, its heights
  from `_sling`: 0.6 m in the middle, 0.86 at the spreader bars ±0.66, the pillow 0.5 towards the head), the
  head raised onto the pillow, knees up and the feet inside the spreader bar. A changed sling needs those numbers
  changed too; the two pack hammocks have theirs.
- **Porch swing** (`porchSwing`): an A-frame; the slatted seat with arms, cushion and chains swings from the beam,
  `swing` ±10° in 2 s while sat in, ridden. Chains are plain satin, so the seat stays one draw call.
- **Swing set** (`swingSet`): a round-timber A-frame; both seats and their chains are one part, `swing` ±20° in
  1.6 s while played on, ridden.
- **Cedar hot tub** (`hotTubCedar`): banded staves with grooves, a rim and a step; the water with bubbles is its own
  part (`bubbles`: a whole turn in 2 s, a loop while soaking); the cover is hinged at the back and `cover_off`
  lifts it 105° to stand behind the tub. It is the model's `open` clip too: soaking leaves the tub open (content
  `leaves`), so the cover stays up until a resident puts it back on.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| sunLounger | 0.62 × 0.76 × 1.87 | 2428 | 3 | Frame, Cushions, Towel |
| hammock | 0.70 × 1.15 × 1.86 | 1372 | 3 | Frame, Fabric, Stripes |
| porchSwing | 1.92 × 2.06 × 0.91 | 2536 | 3 | Seat, Frame, Chains, Cushion |
| swingSet | 1.92 × 1.97 × 0.91 | 1060 | 2 | Frame, Seats |
| hotTubCedar | 1.89 × 0.92 × 1.90 | 3508 | 3 | Cedar, Cover |

`check_glb.py`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

**In the game** (2026-10-14, headless WebGPU): all five bought; a resident sunbathing on the reclined lounger, lying
in the hammock, riding the porch swing and the swing set (frames in a row show seat and resident moving together),
and soaking in the hot tub (cover up, water bubbling; the cover stayed up after the soak).

## Rebuild

```
python3 -B tools/art/blender/build.py outdoor            # every .glb, the pack, check_glb
python3 -B tools/art/blender/build.py outdoor --preview  # previews/bench.png, parasol@stir.png, …
```
