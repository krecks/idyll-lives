# Media — notes

`media.py` builds the eight L4 living-room media models (docs/objects/living.md) into `web/public/assets/models/media/` and
their entries in `packs/media/manifest.json`, a pack listed in the base manifest's `packs` and the only manifest
that defines these keys (`tv`, the old `tv_set`; `tvWall`, `tvRetro`, `homeCinema`, `stereo`, `boombox`,
`tvStandLow`, `fireplaceElectric`). Built 2026-10-16; they sat
in Validate › Media until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`), its
back on the wall face (-0.43). The old parts of the portable TV, the bench, the home cinema, the stereo and the
fireplace reached 1.5–6 cm into the wall; they are moved forward onto its face (`shift(…, dz=…)`). Residents watch standing on the tile in front (the `watch` pose), so every screen faces the front (+Z), as
the old parts did and as housegen expects (`"opposite": "sofa"`).

| Model | Moving parts and clip | Played by |
|---|---|---|
| `tv` | none: a slim set on its stand on an oak media unit, a soundbar in front | — |
| `tvWall` | none: a thin panel on a wall bracket, a soundbar on two brackets under it (wall layer, `floor=False`) | — |
| `tvRetro` | the rabbit ears, origin at their foot: `antenna` turns them 20° and stretches them 1.6× up (0.9 s), telescoping | `watch` (until it ends) |
| `tvStandLow` | the TV with the lift's lid on its top edge, down in the bench at rest: `tv_up` lifts it 60 cm (1.6 s) | `watch` (sinks back at the end) |
| `homeCinema` | the projector screen, its origin in the case on the wall, flattened into it at rest: `screen_down` unrolls it (1.4 s) | `watch`, `film` |
| `stereo` | the record on the turntable (platter, vinyl, label with a stripe): `record` turns it once in 1.8 s, 33 rpm (a loop; 2026-10-21, for the CD tray, which no longer moves); the two woofer cones: `pulse` swells them forward and back (0.5 s, `squash(back=True)`, `every` 0.45 s) | `listen`, `dance` |
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
- The stereo: wood and plain, the turning record and the pulsing cones: 4. The fourth is the E1 loop the plan asks for;
  `check-glb` warns "4 draw calls, aim for 3 at most".

**The bench's TV lift.** A 0.7 m TV can't hide in a 0.49 m bench, so the lift sits in a hood at the bench's back, as
wide as the TV and 0.31 m above the top: at rest the hood is closed by the lid on the TV's top edge; for `watch` the
TV rises 60 cm (its screen's middle at 1.03 m, the old part's at 0.95). The model is 0.80 m high at rest
against the old part's 1.33.

**The fire's floor cushion.** "Warm up by the fire" sits the resident (`pose: sit`), and with no seat they sat at the
default 0.45 m in the footprint's middle, in mid-air with their back to the fire. The model has a floor cushion in front
(0.14 m high, 0.29 m forward) and `characters.ts` `SEATS.fireplaceElectric` (`h: 0.14, fwd: 0.3, back: true`) seats them
on it, facing the flames; the old part had the cushion too. The model's depth is 0.92 m (the old part's surround
0.40; `check-glb` warns).

Other warnings left, by design: the portable TV's height (1.28 m with its ears out, 1.13 at rest), the wall TV floating
above its origin (it hangs on the wall).

## Detail (2026-10-22)

Rebuilt with the forms and the kit, same sizes, centres, parts, clips, glow, paint and draws:

- **TV:** the unit's doors framed round panels (`kit.shaker`) with slim upright bar pulls, a reveal round them, turned
  feet; the set a rounded panel with a vented back (`_flat_screen`), a swept neck, an oval foot; a soundbar swept round
  with a ribbed grille and a light; the pebbles (jittered) in their bowl on two books; a remote with its keys.
- **Wall TV:** a bracket with two arms, the rounded panel, a standby light, a cable cover, a swept soundbar, ribbed.
- **Portable TV:** a rounded cabinet; the bezel extruded round the opening, the tube lofted into a dome; turned knobs,
  the channel dial's numbers, a badge; tapered legs in metal ferrules; the ears swept as telescoping sections on a
  pivot.
- **TV bench:** framed doors with bar pulls, a reveal, tapered legs; a plant of swept leaves in a rimmed pot, a game
  controller; the TV's logo and the lid's finger grip.
- **Home cinema:** bent sled feet, slab fronts with routed grips, a swept soundbar with a ribbed grille, the projector's
  lens glass and vents, the player's disc slot, rounded towers on plinths with drivers (rim, cone, dust cap) in rings and
  a badge; the screen case swept round, its pull bar and pull loop.
- **Stereo:** splayed legs, the amplifier's brushed face with turned knobs and a power light, the turntable's arm on a
  turned pivot with its counterweight, headshell and speed knob; a grooved record with a spindle on a lipped platter;
  turned woofer cones, drivers for tweeters, bass ports.
- **Boombox:** a rounded case, swept trim and speaker rings, ribbed grilles, tweeters, the handle bent in one piece with
  a grip, a tapering aerial on a hinge, piano keys, a tuning needle, side dials, the cassette's label and the door's tab.
- **Electric fireplace:** beads panelling the piers and the frieze (`kit.moulding`), a moulded firebox frame, a fluted
  back, a switch; jittered logs and stones on the embers; the hearth cushion plump and piped (`F.soft`); the mantel on
  two corbels, a lofted vase with a candle, a moulded picture frame with a landscape.

| Model | Size at rest (w × h × d) | Triangles (before → now) | Draws | Colour parts |
|---|---|---|---|---|
| `model.tv` | 0.91 × 1.07 × 0.44 m | 2,144 → 3,548 | 2 | Body (`tv_cabinet`), Trim (`tv_oak`) |
| `model.tvWall` | 1.24 × 0.87 × 0.10 m | 492 → 884 | 1 | Body (`wall_frame`), Trim (`wall_bar`) |
| `model.tvRetro` | 0.94 × 1.13 × 0.49 m | 1,892 → 3,452 | 3 | Body (`retro_wood`), Trim (`retro_bezel`) |
| `model.tvStandLow` | 1.82 × 0.80 × 0.46 m | 2,688 → 4,452 | 3 | Bench (`bench`), TV (`bench_tv`) |
| `model.homeCinema` | 1.88 × 1.65 × 0.45 m | 2,408 → 6,248 | 3 | Body (`cinema_unit`), Trim (`cinema_oak`) |
| `model.stereo` | 0.92 × 0.84 × 0.42 m | 3,480 → 3,800 | 4 | Body (`stereo_wood`), Trim (`stereo_dark`) |
| `model.boombox` | 0.60 × 0.95 × 0.44 m | 3,732 → 2,864 | 3 | Body (`boombox_body`), Trim (`boombox_trim`) |
| `model.fireplaceElectric` | 1.76 × 1.32 × 0.92 m | 3,580 → 4,906 | 3 | Surround (`surround`), Firebox (`firebox`) |

`check-glb`: 8 models, 0 failed, the warnings above. Khronos validator: 0 errors, 0 warnings each. Checked: the
gallery next to the old parts; the clips bench (rest, 0.4 s, open) on WebGPU; in play (`bathroom-check`): each bought
and used (the portable TV, the bench and the home cinema watched with their clips, the stereo danced to, the boombox
listened to, the fire warmed up by on its cushion, the TV and the wall TV watched); all eight in the living room by day
and at night on WebGPU and WebGL (the screens glowing at night); the portable TV turned four ways and the stereo two;
colours on the painted parts.

## Rebuild

```
cargo art media            # the eight .glb, the pack, check-glb
cargo art media --preview  # previews/media/<id>.png, <id>@<clip>.png
```
