# Nature lover pack — notes

`pack_natureLover.py` builds the seven P12 nature lover models into `web/public/assets/models/pack_natureLover/`
and their entries in `packs/pack_natureLover/manifest.json`, an override pack listed after
`packs/natureLover/manifest.json`. Their primitive-part placeholders (`tools/art/pack_natureLover.py`, in
`packs/natureLover`) stay as the fallback. Built 2026-10-14; they sit in Validate › Nature Lover until checked in
play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
keeping its footprint, its wall side (the potting bench and the shower against the wall face at depth −0.43), the
hammock's sling low point at 0.5 m (no `BEDS` entry: the default mattress height) and the glasshouse bench's seats
at 0.45 m.

| Model | Moving part and clip | Played by |
|---|---|---|
| `veggieBed` | the plants (foliage), the fruit (tomatoes, carrots, courgettes) and one lettuce, origins on the soil: `grow` holds each where the crop's stage is (ripe at rest; picked: the fruit gone; growing: the plants at 0.6 height; planted: 0.25); `harvest` presses the lettuce down into the soil (0.8 s); `rustle` sways the plants and the fruit 3° back and forth about the soil line (0.8 s, every 2.4 s), on top of the stage | `growth`; `pickSnack`; `tendVeggies` |
| `pottingBench` | a seedling in a pot on the worktop, built flat on its soil: `seedling` grows it up (1 s) | `potSeedlings` |
| `birdFeeder` | a bluebird on the tray, built at full size and scaled to nothing: `bird` brings it in (0.5 s) | `birdwatch` |
| `gardenHammock` | the sling with its spreader bars, ropes and pillow, pivoting on the line between the hooks: `sway` rocks it 8° each way and home (2.4 s), a loop; `ride` carries the resident | `swing`, `doze` (loops) |
| `cedarShower` | streaks of rain from the copper head, built full length, origin at the head: `water_on` grows them down (0.6 s) | `rinse` |
| `mossTerrarium` | the glass cloche: `lid_open` lifts it 22 cm (0.7 s) | `mist` |
| `glasshouse` | the glass door, hinged on its post, and a roof vent hinged at the ridge: `door_open` swings the door 70° out and lifts the vent 25° (0.9 s), one clip on both | `unwind`, `orchids` |

`tieBouquet` grows a tied bouquet (three stems, three flower heads and a twine wrap, plain) on the potting bench's
worktop front (`bouquet`, 0.8 s), held while it is tied and gone when it ends, as the kids' painting. Its origin is
inside the worktop, so it flattens out of sight at rest. It is a part of its own: the bench takes 4 draws (7,704
triangles: its pots leave out their undersides and, under the soil, their insides, `_pot(trim=True)`).

`tendVeggies` rustles the bed (`rustle`, 0.6 s in and every 2.4 s while tended): the plants and the fruit sway
3° back and forth about their shared origin on the soil line through the bed's middle. A use clip on a growing
part plays on top of the growth stage (objectMotion.ts, `partPose`: the stage first), so a just-planted bed's
seedlings sway by their own height and keep their size. No new part: still 4 draws.

**Draw calls: 3 each; the veggie bed, the glasshouse and the potting bench 4.**
- The cedar is plain satin on the veggie bed, the feeder and the glasshouse bench, so the leaves take the textured
  draw; the hammock's stand and the shower's slats are textured `wood`.
- The veggie bed: a plain still body (box, soil, stakes), the plants (foliage), the fruit (plain) and the picked
  lettuce (foliage): 4. The growth stages need the plants and the fruit apart (the fruit goes when picked, the
  plants stay), and the lettuce is a third part.
- The glasshouse: plain frame and plants plus its glass on the still body, the glass door and the glass vent: 4.
  Its plants are plain satin (ferns, palms, orchids, baskets): the glass is the point, and a foliage draw would make
  5. The door is frameless glass (a frame would be a second draw on it).
- The hammock: the still stand is one draw (wood); the sling part takes its fabric (canvas, stripes, ropes, pillow)
  and the spreader bars' wood. The placeholder's grass tuft is left out.
- The shower's potted plant, the feeder's tube (plain gloss, no glass) and the bench's seedling are plain.

**Found in the game:** a part in a textured finish (`foliage`) that is grown from nothing (`G.grow` from 0.001) never
showed on the clips bench, WebGPU or WebGL; the same part in plain satin did, so the seedling is plain satin. Fixed
in the renderer since: such a part takes its normals and UVs from its grown shape (`models.ts`), and the foliage
flutter is in world metres however far a part is scaled (`nature.ts`). A grown part doesn't flutter (it stays flat and
hidden at rest), so a foliage seedling would work now; this one stays plain satin. The veggie bed's foliage
parts start at full size and shrink, which works.

**The harvest is down into the soil, not a uniform shrink.** The lettuce has two clips (`grow` and `harvest`); the
exporter samples a clip's unkeyed scale axes from the scene's last state, so a uniform `harvest` left `grow`
starting at 0.001 across (check_glb: "starts away from where the file has it"). `harvest` keys only the height, and
the lettuce sits 2 cm into the soil, so flattened it is hidden.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.natureLover.veggieBed` | 1.87 × 1.30 × 0.91 m | 4,780 | 4 | Wood, Posts & rails |
| `model.natureLover.pottingBench` | 1.76 × 1.79 × 0.60 m | 7,704 | 4 | Wood, Frame |
| `model.natureLover.birdFeeder` | 0.71 × 1.78 × 0.68 m | 3,372 | 3 | Wood, Roof |
| `model.natureLover.gardenHammock` | 1.88 × 1.11 × 0.68 m | 2,416 | 3 | Wood, Fabric, Stripes |
| `model.natureLover.cedarShower` | 0.90 × 2.08 × 0.88 m | 5,736 | 3 | Wood, Copper |
| `model.natureLover.mossTerrarium` | 0.48 × 1.17 × 0.70 m | 2,140 | 3 | Wood |
| `model.natureLover.glasshouse` | 1.89 × 2.48 × 1.86 m | 7,496 | 4 | Wood, Frame, Cushions |

`check_glb.py`: 7 models, 0 failed; warnings left: the veggie bed's, the glasshouse's and the potting bench's 4
draw calls (above).
Khronos validator: 0 errors, 0 warnings each.
On the clips bench (WebGPU and WebGL): every clip at rest and open (the veggie bed's `harvest` alone, `&clip=`), the
glasshouse's vent from above, the hammock from its end. Gallery: each next to its placeholder. In play
(`wants-asking`, WebGPU, the yard): the hammock (`swing`: the resident lies in it and rides the sway), the
glasshouse (`unwind`: seated on the bench, the door and vent open), the shower (`rinse`: the rain runs over the
resident), the feeder (`birdwatch`: the bird lands) used; the veggie bed seen ripe and then picked (the fruit gone,
the plants kept). At night (`office-night`, WebGL): the glasshouse at all four facings, the hammock, veggie bed,
feeder and potting bench coloured. Not seen used in play: the potting bench, the terrarium and `pickSnack`
(clips bench only). Residents lie along the depth, across the 2 × 1 hammock's sling, as on the placeholder: the
lying direction is the engine's, not the model's.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_natureLover            # the seven .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_natureLover --preview  # previews/<id>.png, <id>@<clip>.png
```
