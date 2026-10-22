# Farm — notes

`farm.py` builds the eight CM5 farm models (docs/objects/community.md) into `web/public/assets/models/farm/` and their
entries in `packs/farm/manifest.json`: the red barn (`model.farm.barn`), the grain silo (`model.farm.silo`), the
little tractor (`model.farm.tractor`), the round hay bale (`model.farm.hayBale`), the stack of hay bales
(`model.farm.hayStack`), the scarecrow (`model.farm.scarecrow`), the water trough (`model.farm.trough`) and the
chicken coop (`model.farm.coop`). Built 2026-10-14; they sat in Validate › Farm until checked in play.

They stand on the farm lots of farm worlds, placed by id in `web/src/game/town.ts` `farmstead` (the farmyard behind
the house, hay on the fallow field, a scarecrow in a crop row), not by category or group. Each is built in the game's
axes (`F`) and centred on its footprint by `G.settle`; the sizes are the old parts', within 3 % (but the bale's and the tractor's, below).

## Far versions (E7)

Every model has a `<id>.far.glb` for the town overview (the menus' town, the new-world screen):

- **Built by hand** where decimation looked bad (the detail pass of 2026-10-22 left them as they were, but the coop's
  and the hay stack's, new):
  - the barn: walls, roof, plinth, corner boards, the doors' white X, the windows as flat panels, the cupola and
    vane (596 triangles);
  - the silo: pad, a plain drum, the cap and vent, the ladder a flat strip (556);
  - the tractor: plain cylinder tyres without lugs, plain fenders and roll frames, no grille, vents, weights or levers
    (1,352). Decimated, it lost its lowest lug vertices and floated 1.7 cm;
  - the round bale: a coarser roll, one dark ring on each end, flat twine bands (396). Decimated, the twine
    turned to shards;
  - the hay stack: three plain bales and their twine (108);
  - the trough: plain sides and rim, no corrugations (208). Decimated, the corrugations made spikes;
  - the coop: house, roof, legs, one-piece ramp, trims, the hatch up over the dark pop hole, the windows as dark
    panels in frames, the nest box (264). Decimated, its legs and ramp went to spikes.
- **Decimated** (`far=0.25`): the scarecrow.

## The models

Detail pass (2026-10-22): each with what the real thing has where it reads, the sizes, parts, clips, paint and
draws as they were.

- **Barn**: a gambrel-roofed barn, board-and-batten siding, white corner boards and fascia. The metal roof has
  standing seams from eave to ridge under a rolled ridge cap; the stone plinth two courses of dressed stones, their
  joints staggered. At the front are sliding double doors on an iron rail (white frames and X braces, upright bar
  pulls, guides at their feet) and a hayloft door on strap hinges with a knob, a hay beam and pulley. At the back are
  a small door and loft door, with three four-pane windows down each side, each with a sloped sill and a drip cap,
  and a louvred cupola saddled on the ridge (a band, corner boards, a ball under the weathervane). All plain
  finishes: one draw call.
- **Silo**: a galvanised drum, corrugated between stiffening rings every 0.6 m, anchor brackets round its foot on a
  concrete pad. A red domed cap with twelve seams and a rolled eave, a vent and a roof hatch. A ladder up the front
  on round rungs, its rails one bent piece each over the eave to the walkway (a top and a mid rail on posts). A hinged
  door with a handle at the foot, and the unloading chute one bent tube into a hopper. The ladder has no safety cage:
  a cage would reach past the 3 × 3 footprint.
- **Tractor**: big rear wheels with chevron tread lugs (each a bar slanting in from a shoulder), wheel nuts and the
  rims' lips, under swept fenders with a rolled edge; small front wheels. A lofted hood rounding in to a grille frame
  with five slots, headlamps in bezels, side vents and a fuel cap; an exhaust with its muffler and rain flap, an air
  pre-cleaner. It has a padded seat on its pan and a steering wheel on three spokes by a dash with two gauges, under a
  canopy on two bent roll frames; five suitcase weights with handles on a bracket in front; tail lamps, a drawbar,
  a three-point hitch with its top link and the PTO stub behind. A compact tractor, as its name says: 1.62 m over
  the rear fenders (2026-10-21; it was 1.98, a utility tractor's track).
- **Round bale**: a roll lying on its side, a little lumpy, each end rolled in one long spiral to a tuft at its core
  (`hay_dark`), three bands of twine with two tied ends hanging, wisps of straw on its top and ends. 1.2 m across and
  1.2 m long, up to an adult's chest, on 2 × 2 tiles (2026-10-21; it was 0.99 m on one, waist-high); `town.ts`
  `farmstead` places it on the fallow field by its footprint.
- **Hay stack**: three square bales, two side by side and one on top turned 3°, each pressed in nine uneven flakes, a
  little proud of each other, tied twice with round twine, the cut ends darker, straw on top.
- **Scarecrow**: a rough, slightly crooked post and a cross-bar in a plaid shirt (a body rounded in section, sleeves
  bagging round the bar, the check lines following its front, buttons, a pocket, a collar and cuffs) and patched,
  hemmed trousers. A rope belt knotted in front with its ends hanging. Straw pokes out at the hands, ankles, waist
  and neck in tapering stalks. A sack head gathered and tied at the neck has crossed-stitch eyes, a straw nose and a
  stitched smile, under a floppy hat with a dented crown, a brim drooping in waves and a band.
- **Trough**: an oval galvanised stock tank with corrugated sides, riveted seams where its ends join and a rolled
  rim, full of water, on two skids. A float valve hangs inside the rim at one end, fed by a pipe from the ground
  through the end with its tap, and a drain plug at the other; the tank stays at the old part's 0.65 m.
- **Coop**: a wooden house on four braced legs, in lap siding, with a red gable roof of staggered shingles and a
  ridge cap. White corner boards, a wired window with a sill on each side and a slatted vent in each gable, a nest box
  with a sloping lid on two hinges on the back, a ramp of three planks with cleats up to the pop hole (its hatch
  braced, Z-wise), and a feeder hanging under the house.

## Clips and parts

- **Scarecrow**: `sway`, ambient (`motions: {'idle': 'sway'}`). The whole scarecrow is one node, its origin at the
  foot of the post. `G.rock` tips it 2.5° to each side about Blender Y over 3 s (a slow sway is longer than 1 s;
  `check-glb` doesn't mind) and ends where it starts.
- **Coop** (`close`, `night`): the pop-hole hatch is its own node (`hatch`, in `coop_trim`), its origin the middle
  of its top edge. By day it stands 32 cm up its runners, the dark pop hole open for the hens; from dusk to dawn
  `close` slides it down over the hole (1 s), as a keeper shuts the hens in. The coop has no use, so the night is
  what moves it; a use (feeding the hens) could play it too. It costs a draw: 3 now.
- The rest are static, each one node.

## Colour parts

The table says Paint, Roof. The barn, silo and coop take those with their real materials. The rest take what fits:

| Model | `paint` |
|---|---|
| barn | `barn` Paint, `barn_roof` Roof, `barn_trim` Trim |
| silo | `silo` Paint, `silo_cap` Roof |
| tractor | `tractor` Paint, `rims` Wheels |
| hayBale, hayStack | `twine` Twine (the hay itself stays hay) |
| scarecrow | `shirt` Shirt, `trousers` Trousers, `hat` Hat |
| trough | `trough` Paint |
| coop | `coop` Paint, `coop_roof` Roof, `coop_trim` Trim (the hatch too) |

## Numbers

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.farm.barn` | 6.04 × 6.00 × 5.94 m | 7,446 | 596 | 1 |
| `model.farm.silo` | 2.80 × 6.90 × 2.72 m | 9,396 | 556 | 1 |
| `model.farm.tractor` | 1.62 × 2.51 × 3.30 m | 9,240 | 1,352 | 1 |
| `model.farm.hayBale` | 1.23 × 1.23 × 1.23 m | 3,168 | 396 | 2 (hay fabric + plain) |
| `model.farm.hayStack` | 1.91 × 0.93 × 0.90 m | 3,588 | 108 | 2 |
| `model.farm.scarecrow` | 1.00 × 2.12 × 0.55 m | 4,034 | 999 | 3 (post wood, clothes fabric, plain) |
| `model.farm.trough` | 1.88 × 0.66 × 0.75 m | 2,700 | 208 | 1 |
| `model.farm.coop` | 1.74 × 1.88 × 1.97 m | 2,624 | 264 | 3 (walls wood + plain, the hatch) |

Before the detail pass (2026-10-22): barn 2,842, silo 4,348, tractor 5,680, round bale 2,144, hay stack 468,
scarecrow 2,290, trough 1,012, coop 760. Each is inside its size class's target (objects.md: over 2.5 m 12,000,
1-2.5 m 8,000); the bales, placed two to six a farm, at under half of it.

`check-glb`: 8 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings for every file (the
models and their far versions; 2026-10-22, the 12 the detail pass changed).

## Rebuild

```
cargo art farm            # the 8 .glb and 8 .far.glb, the pack, check-glb
cargo art farm --preview  # previews/farm/<id>.png, <id>.far.png, scarecrow@sway.png (--lit: <id>.lit.png)
```
