# Farm — notes

`farm.py` builds the eight CM5 farm models (docs/objects/community.md) into `web/public/assets/models/farm/` and their
entries in `packs/farm/manifest.json`: the red barn (`model.farm.barn`), the grain silo (`model.farm.silo`), the
little tractor (`model.farm.tractor`), the round hay bale (`model.farm.hayBale`), the stack of hay bales
(`model.farm.hayStack`), the scarecrow (`model.farm.scarecrow`), the water trough (`model.farm.trough`) and the
chicken coop (`model.farm.coop`). Built 2026-10-14; they sit in Validate › Farm until checked in play.

They stand on the farm lots of farm worlds, placed by id in `web/src/game/town.ts` `farmstead` (the farmyard behind
the house, hay on the fallow field, a scarecrow in a crop row), not by category or group. Each is built in the game's
axes (`F`) and centred on its footprint by `G.settle`; the sizes are the old parts', within 3 %.

## Far versions (E7)

Every model has a `<id>.far.glb` for the town overview (the menus' town, the new-world screen):

- **Built by hand** where decimation looked bad:
  - the barn: walls, roof, plinth, corner boards, the doors' white X, the windows as flat panels, the cupola and
    vane (596 triangles);
  - the silo: pad, a plain drum, the cap and vent, the ladder a flat strip (556);
  - the tractor: plain cylinder tyres without lugs, no grille bars, vents or levers (1,472). Decimated, it lost its
    lowest lug vertices and floated 1.7 cm;
  - the round bale: a coarser roll, one dark ring on each end, flat twine bands (396). Decimated, the twine
    turned to shards;
  - the trough: plain sides and rim, no corrugations (208). Decimated, the corrugations made spikes.
- **Decimated** (`far=0.25`): the hay stack, the scarecrow and the coop.

## The models

- **Barn**: a gambrel-roofed barn, board-and-batten siding, white corner boards and fascia. At the front are
  sliding double doors on an iron rail (white frames and X braces) and a hayloft door with a hay beam and pulley.
  At the back are a small door and loft door, with three four-pane windows down each side, and a louvred cupola
  with a weathervane on the ridge. All plain finishes: one draw call.
- **Silo**: a galvanised drum with stiffening rings every 0.6 m, a red domed cap with a vent, a ladder up the front, a
  rail round the top, a hatch and an unloading chute at the foot, on a concrete pad. The ladder has no safety cage:
  a cage would reach past the 3 × 3 footprint.
- **Tractor**: big rear wheels with tread lugs under green fenders, small front wheels, a hood with a grille,
  headlights and side vents, an exhaust stack and air pre-cleaner. It has a seat and steering wheel under a canopy
  on four posts, front weights and a drawbar. Tail lamps sit on the rear posts.
- **Round bale**: a roll lying on its side, its ends rolled in rings (`hay_dark`), three bands of twine.
- **Hay stack**: three square bales, two side by side and one on top turned 3°, each tied twice, the cut ends darker.
- **Scarecrow**: a post and cross-bar in a plaid shirt (dark check lines, collar, cuffs) and patched trousers.
  Straw pokes out at the hands, ankles, waist and neck. A sack head tied at the neck has a stitched face and a
  floppy hat with a band.
- **Trough**: an oval galvanised stock tank with corrugated sides and a rolled rim, full of water, on two skids. A
  float valve hangs inside the rim at one end, so the tank stays at the old part's 0.65 m.
- **Coop**: a wooden house on four legs with a red gable roof and ridge cap. White corner boards, a wired window
  on each side, a nest box with a sloping lid on the back, and a ramp with cleats up to the pop hole.

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
| `model.farm.barn` | 6.04 × 6.00 × 5.94 m | 2,842 | 596 | 1 |
| `model.farm.silo` | 2.77 × 6.90 × 2.72 m | 4,348 | 556 | 1 |
| `model.farm.tractor` | 1.98 × 2.50 × 3.28 m | 5,680 | 1,472 | 1 |
| `model.farm.hayBale` | 0.88 × 1.00 × 1.00 m | 2,144 | 396 | 2 (hay fabric + plain) |
| `model.farm.hayStack` | 1.89 × 0.92 × 0.90 m | 468 | 92 | 2 |
| `model.farm.scarecrow` | 0.99 × 2.13 × 0.54 m | 2,290 | 570 | 3 (post wood, clothes fabric, plain) |
| `model.farm.trough` | 1.84 × 0.66 × 0.74 m | 1,012 | 208 | 1 |
| `model.farm.coop` | 1.74 × 1.88 × 1.97 m | 760 | 168 | 3 (walls wood + plain, the hatch) |

`check-glb`: 8 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings for all 16 files (the
models and their far versions).

## Rebuild

```
cargo art farm            # the 8 .glb and 8 .far.glb, the pack, check-glb
cargo art farm --preview  # previews/<id>.png, <id>.far.png, scarecrow@sway.png
```
