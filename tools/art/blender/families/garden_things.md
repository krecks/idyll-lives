# Garden things (`families/garden_things.py`)

OU5 of `docs/objects/outdoor.md`: the pond, the small garden things and the garden lights. One pack,
`packs/garden_things/manifest.json`. Built 2026-10-16; they sat in Validate › Garden Things until signed off (back to Outdoor ›
Decor, the two lights to Outdoor › Lights).

A family of its own rather than more of `outdoor.py`: OU3 and OU4 were built into `outdoor.py` at the same time, and
this kept the three apart. Each model has its own `body` colour and finish (stones, a wooden box, a cream birdhouse, a
red kennel, iron), so each build makes its own palette (`_own`), as `street_furniture.py` does.

- **Garden pond** (`garden.pond`, 2x2): fifteen big coping stones lapping over the water's edge, each an irregular
  slab lofted through a smoothed outline (in at its foot, chamfered to its top, jittered), fifteen smaller ones set
  behind in their gaps and pebbles (`body`, "Stones"), a band of dark wet mud at the margin; an opaque `gloss` water
  block 7 cm deep with a darker middle, as the fountains' water; seven notched lily pads with turned-up rims, a pink
  and a white water lily in three rings of petals, a pink bud, a frog on a pad, reeds and three bulrushes (lofted
  heads on bending stems) at the back left, flag irises at the front right (satin: still, so the pond stays at three
  draws). Two moving parts:
  - `ripples`: three faint rings round where the koi comes up; `ripple` spreads them 30 % and back in 3 s, ambient
    (`idle`, not on Low).
  - `koi`: a lofted white body with an orange saddle and crown, a fanned tail, dorsal and pectoral fins; hidden under
    the opaque surface at rest; `fish` lifts it 4.2 cm (its back and fin out) and turns it 30°, held for the whole of
    `feedFish`; `surface` brings it up and down again in 2.4 s, played every 7 s while a resident watches the fish
    (`fish`, `every`), so a 20-minute watch isn't one koi parked at the top.
- **Planter box** (`planterBox`): a plank trough on corner legs (`body` "Box", wood), a batten screwed down the middle
  of each long side, a moulded cap rail mitred round the top (`kit.moulding`), black steel corner brackets bolted
  round the front corners, soil, twelve leafy clumps on `foliage` (the nature shader's sway and tint) with five-petal
  flowers sat on them, a yellow eye in every other one, a few leaves over the front rim. Centred 9 cm back, as the
  old part's trough. Far version: the same build at low detail (`far_of`: no screws, brackets or eyes).
- **Birdhouse** (`birdhouse`): a gabled cream house (`body` "House") clad in lapped boards down its sides, a front
  gable board proud of the wall with the round entry hole cut through it (dark inside) and a ring round it, a dowel
  perch with a robin on it (`kit.songbird`), corner trim front and back; a red roof of two boards with three shingle
  courses each and a rounded ridge cap ("Roof"); a platform on two braces on a 1.5 m wooden post in a collar of earth.
- **Bird feeders** (`birdFeederPole`): a black shepherd's hook ("Pole"): a pole with a spike foot, a filleted step bar
  to tread it in and an acorn finial, two arms each one bent piece from a collar, curling over into a hook, a scroll
  under each; on each a seed tube hung by a wire bail: a domed green lid, `glass` round the seed, two feeding ports
  with a peg under each, a tray with a lip ("Feeders"), a perch ring on four spokes. The robin (`kit.songbird`) is on
  one. `bird`: a blue tit, built full size and scaled to nothing, appears on the other perch for `birdwatch` (held
  until the watching ends).
- **Dog house** (`dogHouse`): a red kennel (`body` "House") in lapped boards down its sides and back, a front gable
  board with the arched doorway cut through it, the boards' laps either side, a white trim round the arch and a sill,
  bargeboards up the gable, white corner boards; a roof of four staggered shingle courses a slope over an underlay,
  each shingle a little askew, and a rounded ridge cap ("Roof"); a name plate reading REX in strokes, a plinth, a
  water bowl and a chew bone. Letters are mirrored in the build: game x runs to the viewer's left in Blender's front
  view.
- **Sundial** (`sundial`): two chamfered square steps with lichen on them, a turned stone baluster (a torus and a
  scotia at its foot, a swelling vase, a ringed neck, a spreading capital) and a chamfered cap stone (`body`
  "Plinth"); a copper dial with a rim, a chapter ring, twelve hour marks, a dot at each half hour, a compass rose and a
  boss ("Dial"); the gnomon a pierced plate along the noon line.
- **Stepping stones** (`steppingStones`, 1x3, rug layer): four slabs where the old ones are, 4 cm high, rounded at the
  edges, their tops worn faintly uneven (`body` "Stones"); moss creeping in at their edges (plain matte, no foliage
  draw) and gravel between them; on the lawn they show above the grass.
- **Garden lamp post** (`lampPost`): a turned iron base (plinth, torus, bell, waisted neck), a fluted shaft (a sweep of
  a twenty-pointed profile) with a plain neck, three collars, a ladder bar with balls on its ends, a cup (`body`
  "Post"), four scroll brackets curling under the lantern's corners; the lantern ("Lantern"): four frosted panes in
  square corner bars, a band round them two thirds up, a lofted floor, a flared roof with an eave lip and a crown and
  spike on top. The panes are the content's light (2.3 m, asserted) and glow at night (`glow`). Far version
  `far=0.25`. Like every garden lamp it is the player's to switch, not the residents' (`lights.rs`).
- **String lights** (`stringLights`, 2x1): two chamfered posts with collars and pyramid caps ("Posts") in tapered pots
  with rolled rims, gravel and ivy at their feet, a hook on each; nine festoon bulbs ("Bulbs"), each a lofted globe on
  a neck in a ribbed socket, on a wire swept in one piece, hung from steel eyes screwed into the posts; the bulbs
  average 2.1 m, the content's light (asserted), and glow at night. The wire and bulbs are one part hinged on the line
  between the eyes: `sway` rocks it 4° in 3.2 s, ambient.

Detail pass (2026-10-22): rebuilt with the forms and the kit (sweeps, lofts, extrudes with holes, soft boxes,
`kit.moulding`, `kit.fillet`, `kit.songbird`), same sizes, centres, parts, clips, paint and draws. Garden things are
placed on many lots, so each aims at half its size-class target (objects.md, Budgets); the pond (5,524, class target
8,000) is over half, spent on its rim stones.

| Model | Size (w × h × d) | Triangles (before → after) | Draws | Colour parts |
|---|---|---|---|---|
| garden.pond | 2.00 × 0.80 × 1.83 | 4408 → 5524 | 3 | Stones |
| planterBox | 0.94 × 0.62 × 0.50 | 3752 → 3816 (far 1272 → 1288) | 3 | Box |
| birdhouse | 0.31 × 1.92 × 0.32 | 1028 → 1612 | 2 | House, Roof |
| birdFeederPole | 0.85 × 1.91 × 0.27 | 2932 → 3764 | 3 | Pole, Feeders |
| dogHouse | 0.99 × 0.97 × 0.99 | 2224 → 2324 | 2 | House, Roof |
| sundial | 0.36 × 0.96 × 0.36 | 1316 → 2284 | 1 | Plinth, Dial |
| steppingStones | 0.60 × 0.04 × 2.55 | 320 → 1680 | 1 | Stones |
| lampPost | 0.32 × 2.73 × 0.32 | 744 → 2080 (far 182 → 514) | 1 | Post, Lantern |
| stringLights | 1.97 × 2.37 × 0.25 | 2524 → 3674 | 3 | Posts, Bulbs |

`check-glb`: 9 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings on each file and far version.

**In the game** (2026-10-16, headless, WebGPU and WebGL2): the gallery next to the old parts; the clips bench
(`fish`, `surface`, `ripple`, `bird`, `sway`); in play, all nine bought in House 0's garden (`bathroom-check`), the
dog house at all four facings, the planter box and the string lights recoloured, a resident feeding the fish (koi up),
watching the fish (`surface` repeating, ripples ambient) and watching the birds (the blue tit on its perch), by day
and at 22:00 (the lamp's panes and the bulbs lit). After the detail pass (2026-10-22): the gallery (WebGPU and WebGL2)
and the clips bench at 0, 0.5 s and open, no errors; not played in a house again.
