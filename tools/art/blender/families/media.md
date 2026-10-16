# Media — notes

`media.py` builds the eight L4 living-room media models (plan/05-living.md) into `web/public/assets/models/media/` and
their entries in `packs/media/manifest.json`, an override pack over the base manifest (`tv`, furniture.py `tv_set`),
`catalog/manifest.json` (`tvWall`, `tvRetro`, `homeCinema`, `stereo`, `boombox`) and `catalog2/manifest.json`
(`tvStandLow`, `fireplaceElectric`). Their primitive-part placeholders stay as the fallback. Built 2026-10-16; they sit
in Validate › Media until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`), its
back on the wall face (-0.43). The placeholders of the portable TV, the bench, the home cinema, the stereo and the
fireplace reached 1.5–6 cm into the wall; they are moved forward onto its face (`shift(…, dz=…)` in catalog.py and
catalog2.py). Residents watch standing on the tile in front (the `watch` pose), so every screen faces the front (+Z), as
the placeholders' do and as housegen expects (`"opposite": "sofa"`).

| Model | Moving parts and clip | Played by |
|---|---|---|
| `tv` | none: a slim set on its stand on an oak media unit, a soundbar in front | — |
| `tvWall` | none: a thin panel on a wall bracket, a soundbar on two brackets under it (wall layer, `floor=False`) | — |
| `tvRetro` | the rabbit ears, origin at their foot: `antenna` turns them 20° and stretches them 1.6× up (0.9 s), telescoping | `watch` (until it ends) |
| `tvStandLow` | the TV with the lift's lid on its top edge, down in the bench at rest: `tv_up` lifts it 60 cm (1.6 s) | `watch` (sinks back at the end) |
| `homeCinema` | the projector screen, its origin in the case on the wall, flattened into it at rest: `screen_down` unrolls it (1.4 s) | `watch`, `film` |
| `stereo` | the CD tray with a disc: `tray` slides it 11 cm out (0.6 s), `hold` 1.2 s; the two woofer cones: `pulse` swells them forward and back (0.5 s, `squash(back=True)`) | `listen`, `dance` (`uses` and `loops`) |
| `boombox` | the cassette door, hinged at its foot: `cassette` tips it 32° open (0.5 s), `hold` 1.2 s | `listen` |
| `fireplaceElectric` | the flames (five tongues and their cores), origin inside the log bed, flattened into it at rest: `flames` grows them (0.8 s) | `relax` |

**Night glow (E8).** The flat screens (`screen`, `#8DB4E2`) and the portable TV's tube (`crt`) glow at night. The lift
TV glows only while it is up (down, its screen is inside the bench) and the projection screen (`projection`) only while
it is down: at night both glow while watched. The fireplace's ember strip glows at night (its content light is on all
night, `switch: false`), and the flames glow while they burn. The stereo's and boombox's little displays don't (a draw at
night each for a few pixels).

**Draw calls: 3 each, the TV 2, the wall TV 1, the stereo 4.**
- The TV, the portable TV, the bench, the home cinema and the fireplace: one textured `wood` group (the unit, the
  cabinet, the oak top, the mantel) plus the plain group, plus the moving part. The portable TV's speaker grille is
  wooden slats over plain cloth (a `fabric` grille would have been a fourth).
- The stereo: wood and plain, the tray and the pulsing cones: 4. The fourth is the E1 loop the plan asks for;
  `check_glb` warns "4 draw calls, aim for 3 at most".

**The bench's TV lift.** A 0.7 m TV can't hide in a 0.49 m bench, so the lift sits in a hood at the bench's back, as
wide as the TV and 0.31 m above the top: at rest the hood is closed by the lid on the TV's top edge; for `watch` the
TV rises 60 cm (its screen's middle at 1.03 m, the placeholder's at 0.95). The model is 0.80 m high at rest against the
placeholder's 1.33 (`check_glb` warns of the height).

**The fire's floor cushion.** "Warm up by the fire" sits the resident (`pose: sit`), and with no seat they sat at the
default 0.45 m in the footprint's middle, in mid-air with their back to the fire. The model has a floor cushion in front
(0.14 m high, 0.29 m forward) and `characters.ts` `SEATS.fireplaceElectric` (`h: 0.14, fwd: 0.3, back: true`) seats them
on it, facing the flames; the placeholder has the cushion too. The model's depth is 0.92 m (the placeholder's surround
0.40; `check_glb` warns).

Other warnings left, by design: the portable TV's height (1.28 m with its ears out, 1.13 at rest), the wall TV floating
above its origin (it hangs on the wall).

| Model | Size at rest (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.tv` | 0.91 × 1.07 × 0.44 m | 2,144 | 2 | Body (`tv_cabinet`), Trim (`tv_oak`) |
| `model.tvWall` | 1.24 × 0.87 × 0.10 m | 492 | 1 | Body (`wall_frame`), Trim (`wall_bar`) |
| `model.tvRetro` | 0.94 × 1.13 × 0.49 m | 1,892 | 3 | Body (`retro_wood`), Trim (`retro_bezel`) |
| `model.tvStandLow` | 1.82 × 0.80 × 0.46 m | 2,688 | 3 | Bench (`bench`), TV (`bench_tv`) |
| `model.homeCinema` | 1.88 × 1.65 × 0.45 m | 2,408 | 3 | Body (`cinema_unit`), Trim (`cinema_oak`) |
| `model.stereo` | 0.92 × 0.84 × 0.42 m | 3,668 | 4 | Body (`stereo_wood`), Trim (`stereo_dark`) |
| `model.boombox` | 0.60 × 0.95 × 0.44 m | 3,732 | 3 | Body (`boombox_body`), Trim (`boombox_trim`) |
| `model.fireplaceElectric` | 1.76 × 1.31 × 0.92 m | 3,580 | 3 | Surround (`surround`), Firebox (`firebox`) |

`check_glb.py`: 8 models, 0 failed, the warnings above. Khronos validator: 0 errors, 0 warnings each. Checked: the
gallery next to the placeholders; the clips bench (rest, 0.4 s, open) on WebGPU; in play (`bathroom-check`): each bought
and used (the portable TV, the bench and the home cinema watched with their clips, the stereo danced to, the boombox
listened to, the fire warmed up by on its cushion, the TV and the wall TV watched); all eight in the living room by day
and at night on WebGPU and WebGL (the screens glowing at night); the portable TV turned four ways and the stereo two;
colours on the painted parts.

## Rebuild

```
python3 -B tools/art/blender/build.py media            # the eight .glb, the pack, check_glb
python3 -B tools/art/blender/build.py media --preview  # previews/<id>.png, <id>@<clip>.png
```
