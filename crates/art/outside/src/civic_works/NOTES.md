# Civic works — notes

`crates/art/outside/src/civic_works` (`cargo art civic_works`) builds the eleven big working things of the town's named places (`content/civicWorks.json`, the
`community` buy category) into `web/public/assets/models/civic_works/` and their entries in
`packs/civic_works/manifest.json`: the fire station's fire engine, fire pole and hose rack, the garage's car lift, tyre
rack and tool chest, the newspaper's printing press, the server rack, and the builders' yard's timber stack, cement
mixer and the town's flagpole (`model.<id>`). Built 2026-11-17; before them the objects were drawn with stand-ins (the
tractor, the corner desk, a floor lamp, the spice rack …).

Each is built where the game places it (`F`, game axes, the front to +z) and has a palette of its own over shared
colours (`_palette`, as the street furniture's do), its main part `body`, listed first in `paint`. The things that
stand against a wall have their backs on its face (−0.43): the hose rack on it (`.on_wall()`, at its real height),
the server rack, the tyre rack and the tool chest on the floor (`settle!(centre = false)` and `.centre(..)`). The fire
engine, the car lift and the outdoor things (timber stack, cement mixer, flagpole) have far versions built by hand,
the model's own build at low detail (`.far_low()`): plain shutters, fewer sides, no rungs, nuts, ribs or chevrons.

## The fire engine (3 × 6)

A UK appliance, 2.5 m wide over its mirrors, 3.17 m to its ladders, 5.84 m long, its cab to the front: a crew cab drawn
from the side (`fextrude!`, its front wheel's arch cut out, its windscreen raked back) and a body of lockers (the rear
wheel's arch cut out), in gloss red. Four lockers a side and one at the back behind aluminium roller shutters (`slats`:
one outline of stepped slats, extruded), each with its chrome drop bar; tinted windows, door seams, handles and grab
rails, steps, mirrors on chrome arms, a yellow reflective band, black arch trims, a side guard between the wheels. The
front: grille bars, headlamps in chrome bezels, indicators, blue grille lights, fog lamps, a white sign over the
windscreen; the light bar on the cab (blue either side of white). The back: tail lamps, beacons, red and yellow chevrons
on the bumper. The roof: a walkway, the triple ladder on two gantries, a roof ladder, two ribbed hard suction hoses
with brass ends. Wheels: tyres with rounded shoulders, silver dished rims, nuts and caps, pairs at the back. All plain
finishes: one draw.

## The car lift (3 × 5)

A two-post lift (blue posts on foot plates, carriages, hydraulic rams, a control box with its buttons and cable, a
chequer floor plate between the posts) holding a hatchback on its four arms, sills 1.27 m up. The car: a body drawn from
the side with empty arches, a glasshouse of tinted glass leaning in to its roof (`tapered`) under a roof, spoiler and
pillars in its colour, lamps, grille, plates (white at the front, yellow at the back), mirrors, door seams and handles;
in its arches black liners, brake discs with red calipers, hubs with studs, struts and springs; under it an exhaust,
silencer and tank. Its four wheels (alloys, five spokes, `alloy`): two stacked under its nose, one lying under its
tail, one leaning on the right post. One draw.

## The printing press (2 × 2)

A small web press in machine green: a base cabinet with panels and vents, two cast side frames, the impression
cylinder, five ink rollers under a hood, the ink fountain, a reel of newsprint on its stand at the back with the web
running up over a guide roller, a cutter at the front over a delivery table with a pile of the day's paper (headline,
columns, photo), and a control desk on the right frame. Moving parts:

- **The plate and blanket cylinders** (`plate_cylinder`, `blanket_cylinder`): each on its axle, with a clamp gap and
  printed blocks round it. `rollers` turns them against each other, a turn in 1.6 s.

`motions`: `{"loops": {"watch": "rollers"}}`: they run while a resident watches. Three draws (the still body and
the two cylinders).

## The rest

- **Fire pole** (1 × 1): a 75 mm polished brass pole (`metal`, roughness 0.16) from a floor flange up to a collar at
  the ceiling (2.8 m, where ceiling things hang from), on a round rubber landing mat (0.84 m, moulded rings).
- **Hose rack** (1 × 1, on the wall): a red rail and two saddles, a red hose coiled four times over one with its branch
  (nozzle) hanging, a white one coiled three and a half times over the other with a brass coupling.
- **Server rack** (1 × 1): a black 42U cabinet, 0.6 × 2.0 × 0.89 m, a glass door (`glass`) in a frame with a swing
  handle, roof vents and a cable brush; inside on its rails a UPS with a screen, servers with drive bays, switches,
  patch panels with coloured leads, blanks. Its LEDs and the UPS's screen glow at night; half its LEDs are a part
  (`leds`) whose `blink` clip pushes them back into their bezels twice in 2.2 s. `motions`: `{"idle": "blink"}`
  (near the camera, not on Low). Three draws (plain, glass, the LEDs).
- **Tyre rack** (2 × 1): orange uprights with slots and bracing, grey step beams with price tags, three rows of eight
  new tyres standing between the beams, a label on each tread. The rack is 0.66 m deep, a tyre's height.
- **Tool chest** (1 × 1): a red roller cabinet on four castors (brakes on the front pair), seven drawers with aluminium
  pulls, one pulled out on a foam liner of six spanners, a push handle on its side, a rubber top with a socket case
  and a spanner on it; a top box with four drawers, a lid with a hasp, handles on its ends.
- **Timber stack** (2 × 1, outdoors): three bearers, four layers of planks on stickers and four posts on top, a blue
  tarpaulin (`fabric`, `soft!`, rumpled) over the middle with the timber's ends showing, two ropes down to the
  bearers, an old tyre on top. Three draws (wood, fabric, plain).
- **Cement mixer** (1 × 1, outdoors): an orange drum tilted 55°, crusted grey inside its mouth, round its lip and in
  dribbles down its front, a ring gear round it, on a yoke with a tilting wheel, a motor under a cover with its switch
  and lead, a tube stand on two wheels and two feet with a handle.
- **Flagpole** (1 × 1, outdoors): a 6 m white pole tapering from 55 to 30 mm on a flange and a concrete pad, a gilt
  ball, a cleat and its halyard; the town's flag (1.5 × 0.9 m, `fabric`: blue, a gold band, a white canton), rippled,
  flying out past its tile (outdoors, which the footprint check allows). The flag is a part (`flag`): `flutter`
  sways it 7° about the pole in 3.4 s (`{"idle": "flutter"}`). Its box is off the tile's middle: `.centre(0.6368, 0.0)`.

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.fireEngine` | 2.51 × 3.17 × 5.84 m | 11,880 (3,616) | 1 | Paint, Lockers, Stripes, Ladders |
| `model.carLift` | 2.94 × 2.76 × 4.09 m | 10,772 (2,836) | 1 | Lift, Car, Wheels |
| `model.printingPress` | 1.72 × 1.51 × 1.86 m | 3,912 | 3 | Press |
| `model.firePole` | 0.84 × 2.80 × 0.84 m | 2,612 | 1 | Pole, Mat |
| `model.hoseRack` | 0.88 × 0.79 × 0.30 m | 2,328 | 1 | Rack, Hose, Second hose |
| `model.serverRack` | 0.60 × 2.00 × 0.89 m | 4,976 | 3 | Cabinet, Servers |
| `model.tyreRack` | 1.96 × 2.00 × 0.66 m | 7,940 | 1 | Frame, Beams |
| `model.toolChest` | 0.79 × 1.43 × 0.66 m | 2,804 | 1 | Chest |
| `model.timberStack` | 1.86 × 0.60 × 0.93 m | 3,132 (930) | 3 | Tarpaulin, Timber |
| `model.cementMixer` | 0.80 × 1.16 × 0.97 m | 4,280 (1,360) | 1 | Mixer, Stand |
| `model.flagPole` | 1.87 × 6.17 × 0.60 m | 1,332 (436) | 2 | Pole, Flag |

All under their size class's target (the fire engine and the car lift, over 2.5 m: 12,000). `check-glb`: 11 models, 0
failed, no warnings. Khronos validator: 0 errors, 0 warnings on all sixteen files (models and far versions).

## Rebuild

```
cargo art civic_works              # the .glb and .far.glb files, the pack, check-glb
cargo art civic_works --preview    # previews/civic_works/<id>.png, <id>@<clip>.png, <id>.far.png (--sheet: four sides)
```

The Workbench previews draw glass opaque and metal dark: the server rack's servers show only in the game
(`bun tools/bench/check.mjs gallery --keys model.serverRack`).

## Edges at the play camera (2026-11-25)

The gate's `edges` found 137 (car lift 47, tyre rack 30, printing press 25, fire engine 12, flag pole 7, server
rack 6, cement mixer 4, the timber stack, hose rack and fire pole 1); none now, none accepted. The press's, yard's
and the station's revolves, sweeps, lofts and extrusions that named no bevel take the crate's `eased` (a 4 mm chamfer
on turns over 50°; not the mixer's tow handle), as do the lift's brakes, springs, pads, pillars, lamps, floor plate,
exhaust and the wheel's rim, the engine's arches, beacons, wheel caps and head-lamp bezels, the tyre rack's and the
timber stack's tyres (their open beads); a blank server panel takes the 2 mm chamfer the others have.

Triangles back under the aims: the tyre rack's tyres sixteen round (eased, twenty went to 8,452), its uprights'
slots one quad each (`mark`: 1,152 triangles as boxes); the fire engine's outer tyres 24 round and its inner 16, its
rims 16, its arches ten steps, its shutters' slats an open ribbon (`ribbon`: an extrusion's end caps and back, never
seen, cost as much again); the lift's wheels 24 round, its silencer 12 and its rods 8. Car lift 12,308 → 11,956, fire
engine 13,532 → 11,972, tyre rack 8,452 → 7,876.
