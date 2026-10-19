# Garden things (`families/garden_things.py`)

OU5 of `docs/objects/outdoor.md`: the pond, the small garden things and the garden lights. One pack,
`packs/garden_things/manifest.json`. Built 2026-10-16; in Validate › Garden Things until signed off (back to Outdoor ›
Decor, the two lights to Outdoor › Lights).

A family of its own rather than more of `outdoor.py`: OU3 and OU4 were built into `outdoor.py` at the same time, and
this kept the three apart. Each model has its own `body` colour and finish (stones, a wooden box, a cream birdhouse, a
red kennel, iron), so each build makes its own palette (`_own`), as `street_furniture.py` does.

- **Garden pond** (`garden.pond`, 2x2): a ring of 18 flat rim stones (`body`, "Stones") and pebbles round an opaque
  `gloss` water block 7 cm deep with a darker middle, as the fountains' water; six notched lily pads, a pink and a
  white water lily, reeds and three bulrushes at the back left (satin: still, so the pond stays at three draws). Two
  moving parts:
  - `ripples`: three faint rings round where the koi comes up; `ripple` spreads them 30 % and back in 3 s, ambient
    (`idle`, not on Low).
  - `koi`: hidden under the opaque surface at rest; `fish` lifts it 4.2 cm (its back and fin out) and turns it 30°,
    held for the whole of `feedFish`; `surface` brings it up and down again in 2.4 s, played every 7 s while a
    resident watches the fish (`fish`, `every`), so a 20-minute watch isn't one koi parked at the top.
- **Planter box** (`planterBox`): a plank trough on corner legs, capped with a rim (`body` "Box", wood), soil, twelve
  leafy clumps on `foliage` (the nature shader's sway and tint) with five-petal flowers sat on them, a few leaves
  over the front rim. Centred 9 cm back, as the old part's trough. Far version: the same build at low detail
  (`far_of`, 1,272 triangles).
- **Birdhouse** (`birdhouse`): a gabled cream house (`body` "House") with a red roof and ridge ("Roof"), an entry hole
  with a ring, a perch and corner trim, on a 1.5 m wooden post with a bracket.
- **Bird feeders** (`birdFeederPole`): a black shepherd's hook ("Pole") with two curling arms, each with a seed tube
  (`glass` round the seed, a green cap and tray, "Feeders") and a perch ring; the robin the description promises is on
  one. `bird`: a blue tit, built full size and scaled to nothing, appears on the other perch for `birdwatch` (held
  until the watching ends).
- **Dog house** (`dogHouse`): a red plank kennel (`body` "House"), four overlapping shingle rows a side and a ridge
  ("Roof"), white corner boards, an arched door, a name plate reading REX in strokes, a plinth and a water bowl.
  Letters are mirrored in the build: game x runs to the viewer's left in Blender's front view.
- **Sundial** (`sundial`): a turned stone baluster on a square step (`body` "Plinth"), a copper dial with a rim,
  twelve hour marks and a boss ("Dial"), and the gnomon along the noon line.
- **Stepping stones** (`steppingStones`, 1x3, rug layer): four slabs where the old ones are, 4 cm high with
  rounded edges (`body` "Stones"); on the lawn they show above the grass.
- **Garden lamp post** (`lampPost`): a turned iron base, post, collar and cup (`body` "Post"), four brackets, a lantern
  of four frosted panes in a frame under a pyramid roof and finial ("Lantern"). The panes are the content's light
  (2.3 m, asserted) and glow at night (`glow`). Far version `far=0.25`. Like every garden lamp it is the player's to
  switch, not the residents' (`lights.rs`).
- **String lights** (`stringLights`, 2x1): two posts ("Posts") in weighted pots, nine festoon bulbs ("Bulbs") in
  sockets on a sagging wire, hung from eyes on the posts; the bulbs average 2.1 m, the content's light (asserted), and
  glow at night. The wire and bulbs are one part hinged on the line between the eyes: `sway` rocks it 4° in 3.2 s,
  ambient.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| garden.pond | 1.99 × 0.80 × 1.82 | 4408 | 3 | Stones |
| planterBox | 0.94 × 0.62 × 0.50 | 3752 (far 1272) | 3 | Box |
| birdhouse | 0.31 × 1.91 × 0.32 | 1028 | 2 | House, Roof |
| birdFeederPole | 0.85 × 1.91 × 0.27 | 2932 | 3 | Pole, Feeders |
| dogHouse | 0.98 × 0.97 × 0.99 | 2224 | 2 | House, Roof |
| sundial | 0.36 × 0.96 × 0.36 | 1316 | 1 | Plinth, Dial |
| steppingStones | 0.59 × 0.04 × 2.54 | 320 | 1 | Stones |
| lampPost | 0.32 × 2.73 × 0.32 | 744 (far 182) | 1 | Post, Lantern |
| stringLights | 1.97 × 2.37 × 0.25 | 2524 | 3 | Posts, Bulbs |

`check-glb`: 9 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings on each file and far version.

**In the game** (2026-10-16, headless, WebGPU and WebGL2): the gallery next to the old parts; the clips bench
(`fish`, `surface`, `ripple`, `bird`, `sway`); in play, all nine bought in House 0's garden (`bathroom-check`), the
dog house at all four facings, the planter box and the string lights recoloured, a resident feeding the fish (koi up),
watching the fish (`surface` repeating, ripples ambient) and watching the birds (the blue tit on its perch), by day
and at 22:00 (the lamp's panes and the bulbs lit).
