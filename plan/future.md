# Future: what to consider

Things found while doing the bathroom tasks (B1–B5, 2026-10-14) that are worth a decision or a task later.
None of them blocks the plan. Each says where it lives.

## Validate and sign-off

- **Validate is crowded.** It holds the sinks, sofas, fridges, cookers, appliances, counters, toilets, showers,
  baths, laundry and bathroom storage groups, plus the kitchen storage and bedroom groups other sessions are
  adding. The plan asks for about three at a time. Sign tasks off (`16-sign-off.md`) after review, so the
  catalogue stays usable.
- **Several sessions edit the same content files** (`base.json`, `furniture.json`, `catalog2/*.json`,
  `CREDITS.md`, the base manifest's `packs`). Each commit had to stage only its own hunks. A tool that moves
  items into Validate and back (sign-off) by id would make this safer than hand edits; the bathroom tasks used
  a throwaway script that edits the JSON as text, keeping its formatting.

## Content and placeholders

- **The wall-hung toilet's placeholder seat is at 0.6 m**, while its content `seat` (and the model) is 0.45 m.
  If the file fails to load, the resident sits inside the fallback's bowl. Fix the placeholder in
  `tools/art/catalog2.py` (`toiletWallHung`).
- **Some placeholders sit inside the wall.** The compact toilet's back is at -0.455 and the wall-hung toilet's
  panel at -0.48, past the wall face (-0.43). The models stand on the wall face, so `check_glb.py` warns that
  they're off centre from their placeholders, and the wall-hung one is deeper than its placeholder. Moving the
  placeholders forward would clear the warnings.
- **The classic toilet** has a close-coupled cistern, as its content and placeholder say. The plan's "high
  cistern and chain" would be a new object (a Victorian toilet), not this one.
- **Things that need a use first ("+use").** Their moving part is built separate, with no clip yet:
  - the mirror cabinet's mirrored door and the tall bathroom cabinet's upper door (`door`, hinged on the left);
  - the laundry basket's lid (not built: the basket is open, full of washing).
  
  Once content gives them an interaction, add a `G.swing` and `motions`. A still part costs no draw call now; a
  moving one will cost one.

## Draw calls

- **The washer and the dryer take 4 draw calls**, the most allowed: the cabinet, the door ring, the door's glass
  and the drum. An opaque window would save one, but then the drum can't be seen turning. Keep, or revisit if
  laundry rooms are slow on the Intel HD 620 target.
- **The base shower's sliding door stays shut.** A moving glass door would be a 4th draw call for the shower
  (body, glass, rain). The plan's "door opening first" is possible now (E5 holds) if it's worth the cost.

## Mirrors

- **File mirrors are `gloss`, not `glass` (fixed 2026-10-14).** Only the High preset reflects live; elsewhere a
  `glass` mirror was see-through and showed what is behind it. `check_glb.py` now warns of see-through mirror
  glass, and the docs and gamekit say `gloss`.
- **The bedroom mirrors still use `glass`** with a silvering plate behind (`families/bedroom_dressing.py`:
  `mirror`, `mirrorFloor`, `mirrorWall`, `dressingTable`; the gate warns of each). Switching them to `gloss`
  and dropping the plates saves a draw call per mirror.

## Testing in the browser

- **`__world()` is the static lot**: no resident positions, actions or needs. Tests can't wait for "the resident
  is at the object" and have to time their screenshots. A debug hook for the sims' live state would make in-play
  checks reliable.
- **Residents with full needs skip uses.** A resident with a full bladder walks to the toilet and away without
  sitting. The bathroom checks used a copy of `wants-asking` with low needs (`debug-states/bathroom-check.json`,
  local only). A sim test that dumps a "needy" state would make this repeatable.
- **The gallery's camera** looks at the models' backs by default (`alpha` -π/2) and from too far for small things;
  wall things hang above the frame. A front three-quarter default, or a `target` parameter, would help.
- **The checking scripts** (gallery and placeholder side by side, clips bench at set times, buy and use in play
  with bursts of screenshots, `?quality=`) were written per session. Worth keeping one in `tools/bench/`.
- **Not checked yet** for the bathroom: the catalog pictures, the colour studio's UI (the colours were checked on
  the clips bench, `&colors=`), and B5 turned to all four facings.

## Toolkit

- **`gamekit` `floor=False`** (a thing on a wall at its real height) came from another session and was committed
  with B4, because the towel rail needed it.
- **A still part's draws**: `check_glb.py` counts a separate node with no clip as part of the still body (one draw
  for the lot). That's how the game draws it too, so a door can be split off early for free.
- **Finding a model's `centre`**: wall-standing models need `centre=` to match their measured bounds to 2 mm, found
  by trial. `build.py` could print the measured centre when the size check fails, as it prints the size.

## Kids (KD1–KD3, built 2026-10-14)

- **Validate is crowded** (see above): the kids' tasks added three more groups (Baby And Beds, Toys, Kids Study).
- **Placeholders drawn open.** The toy box's and the toy chest's placeholders (`catalog.py` `toyBox`,
  `catalog2.py` `toyChest`) draw the lid open, so the fallback looks open, its picking and shadow box is taller
  (0.84 and 0.95 m against the closed 0.51 and 0.53 m), and `check_glb.py` warns the models are off centre and
  short. Closing the placeholders' lids would clear the warnings.
- **`check_glb.py` measures turned placeholder parts unturned.** `placeholder_box` ignores `rotate`, so the high
  chair's splayed legs and the play tent's poles read as wider than they are, and the gate warns the models are
  narrower than their placeholders. Worth fixing in the gate before more splayed things (easels, tripods) come.
- **Colour slots that differ from the plan.** The ball pit's are Pit and Rim, not "Balls": the balls are five
  colours, and one slot would recolour a fifth of them. The kids' desk has a third slot, Chair (its chair is part of
  the model). The bean bag's yellow tag isn't paintable.
- **The train table's track is a ring**, not the placeholder's oval, so the train can `spin` round its middle. An
  oval would need a path clip (not in gamekit).
- **Things that need a use first ("+use").** The high chair's `tray` is a part of its own, with no clip: once
  the chair gets an interaction (a meal, a baby sitting), add `G.slide(tray, …)` and `motions`. It costs a draw
  call already.
- **Clips that close while the use goes on.** The toy box and chest shut their lid after `hold` 2 s, as the plan
  says, while the child plays on for up to 30 minutes; the desk's drawer shuts after 2 s too. The chalk drawing and
  the painting vanish when the use ends (the clip plays back). Keeping a finished picture would need state in the
  sim (like the open flag).
- **The crib's mobile turns for the whole night.** It loops while the baby lies in the crib (`loops.lie`, 12 hours),
  near the camera only. If that's too busy, play it for awake babies only (needs a use or tag that tells them apart).
- **The car bed has no quilt.** It isn't in `BLANKET_BEDS`, so a sleeper lies on top of the turned-down duvet (it
  reads as tucked in at the feet). Adding it to `BLANKET_BEDS` (characters.ts) would match the kids' bed.
- **Kids' uses end early once the need is full** (`world.rs`, "satisfied"): a child who has just played rides the
  rocking horse for 5 minutes, not 15, and a full-fun child stops at once. That's by design, but it makes a use's
  clip easy to miss when checking in play.
- **Checking in play** (for the next tasks with kids' things): `__send({type: 'adopt', household: 0, child})` makes
  a baby (with a crib) or a child; fast-forward past school (on weekdays 8–15) and `setAutonomy` off. The household
  strip shows an activity's label while the resident is still walking to it, and `__world()` has no positions or
  actions, so take a few shots after the label appears and use the ones where they've arrived. Seen so in play:
  the crib's rail down for a feed, the toy chest's lid, the doll house's front, the horse ridden, the tent's flap,
  the train, the tower of blocks, the chalk drawing, the painting and the car bed's duvet; four facings for the toy
  box and the rocking horse. Not caught in play (seen only on the clips bench): the toy box's lid and the desk's
  drawer (2 s holds), the bookshelf's book and the teddy (short uses), the kids' bed's duvet (under the game's
  quilt), and the ball pit's bounce and the bean bag's squash (too small to tell in the shots).

## Community and farm (CM1–CM5, built 2026-10-14)

- **Fixed after the build.**
  - The seesaw's placeholder (`community.py`) was built for 2 × 1 tiles while its content is 3 × 1, so the fallback,
    picking and shadow were 57 cm narrower than the model. It's now 3 × 1, with its seats on the slots (±0.75 m).
  - The shop counter's worktop and the bakery case's deck reached to −0.47, 4 cm into the wall (the README's "wall
    face" trap), placeholders included. Both models and both placeholders now stand 4 cm forward.
- **Validate is crowded** (see above): the community tasks added five more groups (Playground, Street, Stalls,
  Shops, Farm). All but Shops carry `"outside": true`, as their home groups do, so outdoor things keep counting as
  outdoors while they're in Validate; sign-off moves the items back and drops the groups.
- **One family per task**, not the plan's single `community.py`: `playground`, `street_furniture`, `stalls`,
  `shops` (CM1–CM4) and `farm`. The keys are unchanged (`model.<id>`, `model.farm.<id>`).
- **Far versions: decimation fails on some shapes.** A quarter of the triangles turned thin or inlaid parts into
  spikes (the bale's twine, the trough's ridges, the chess board) and lifted things off the floor by dropping their
  lowest vertices (the tractor 1.7 cm, the drinking fountain 3.9 cm, the statue 0.7 cm). Eight models have
  hand-built far versions instead. `far=0.25` in gamekit could keep the vertices near the floor, and `check_glb.py`
  could fail a far version that doesn't stand on the floor.
- **Far versions not seen in the town overview.** The playground's and the shops' were only seen as Blender
  previews. The street, stalls and farm ones were seen in the overview.
- **No town dump with every community thing.** The park in `wants-asking` (`__showPlot(7)`) has most of the
  playground and street things, but no market, shops, statue, post box, bus shelter or phone box. The agents made
  gitignored dumps (`debug-states/shops-town.json`, `stalls-town.json`) by swapping park objects. Sign-off would
  be easier with a committed dump of a town lot holding all 28 town things.
- **The phone box's door is one hinged leaf**, not the plan's folding door: two leaves, each with glass, would be
  6 draw calls (the most allowed is 4; the box is at 4). It swings out onto the tile where the caller stands, so
  its edge may pass through them mid-swing. Look at it in play; a door opening inward, or a folding door with
  opaque panes, would avoid that.
- **Clips start with the use.** The basketball hoop's `swish` is meant to play "after a shot", but it plays at the
  start of the use, for 0.6 s. A per-clip delay in `motions` would let a clip play part way into a use. The swish and
  the post box's `flap` weren't caught in play (the resident stands in front of the slot); both play on the clips
  bench.
- **Long loops.** The merry-go-round's `spin` is a whole turn in 3.5 s, not about 1 s: its riders ride it, so a
  quarter turn would snap them back each round, and a turn a second is too fast. The scarecrow's ambient `sway` is
  3 s. `check_glb.py` has no length check, so "about 1 s" is a rule only for clips residents wait on.
- **The street lamp's gate warning** (0.47 m deep against the placeholder's 0.65 m) is the gate measuring the
  placeholder's turned pyramid cap unturned (Kids: "`check_glb.py` measures turned placeholder parts unturned").
- **Things that need a use first ("+use").** The chicken coop's hatch is its own part, with its origin on its top
  edge, so a use can slide it up its runners. With no clip yet it's joined into the body's draw calls; with one it
  costs one more (3).
- **Colour slots that differ from the plan's tables**, where the table's labels don't fit: the playground uses
  Horse/Saddle/Spring, Platform/Rails/Stripes, Table/Legs and Table/Stools; the ice cream cart Body/Parasol/Bands; the
  machines Body only; the grocery shelves Shelves/Trim; the café table and chair Top/Frame and Frame/Seat; the farm
  Paint/Roof/Trim (barn, coop), Paint/Wheels (tractor), Twine (hay), Shirt/Trousers/Hat (scarecrow).
- **Holds the plan didn't give.** The bakery case's back door shuts after 2 s, rather than staying open for the whole
  10-minute pastry use. The hot dog stand's lid stays up for the whole use, as the plan says.
- **Parallel agents sharing a scratchpad** overwrote each other's helper scripts. Give each its own folder in the
  scratchpad (as with dev server ports).

## Hobbies (H1–H3, built 2026-10-14)

- **Grown parts that flatten onto a visible face show at rest.** A part `G.grow`n from 0.001 along one axis keeps
  its other two: the dartboard's darts, flattened onto the board's face, left their flights as three coloured
  crosses, and the flask's liquid, flattened onto the flask's floor, showed as an orange disc through the glass. Both
  are fixed by putting the part's origin inside something opaque (the board, the bench top), so it flattens out of
  sight. Worth a line in the README's traps, and perhaps a gate check: a part scaled near 0 at rest whose flattened
  plane is outside the rest of the model.
- **Instruments in the hands.** The guitars and the ukulele are parts of their own with no clip: the `music`
  animation doesn't draw an instrument in the resident's hands, so hiding it from the stand (`off_stand`) would only
  make it vanish. Once the renderer can draw a held prop with a hand pose, add `G.squash(part, 'off_stand', 0.001)`
  and the use to `motions`.
- **Seats face the front.** A seated use puts the resident behind the object facing +z (the `seat`'s `forward`
  is negative), as at the desks. The piano follows that: against a wall, the pianist sits between the wall and the
  keys, and the case's back faces the room. A real upright stands with its back to the wall, which needs a seat
  facing the object's back (a `facing` on `seat`) and a use from behind. The digital piano and the sewing table are
  the same, but read as tables.
- **Parts hidden at rest skew the gate's size checks.** The dartboard (darts hidden) warns of its depth (0.07 m
  against 0.19 m) and its centre; the pottery wheel (the pot a low lump at rest) of its height. Clicking, the
  fallback and the shadow follow the placeholder, so nothing breaks, but the warnings are noise on every such model.
  The gate could measure parts hidden at rest at their open pose.
- **The pool table's cue** lies on the cloth; the placeholder's stuck out past the table, so the model is 1.40 m
  deep against 1.69 m (a gate warning). The placeholder's picking box still includes the old cue.
- **Draw calls shaped the models.** The foosball kicks two of its four rods (four turning rods would be six draw
  calls), the drums' hi-hat is still and the throne satin, and the chess chair's pad and the sewing table are satin
  rather than fabric or wood. Moving parts that share a draw call would allow all four rods.
- **Testing in play.**
  - Needs beat a queued use even with free will off (`setAutonomy`): in `wants-asking` the resident went for a snack
    or the toilet. Re-send the `use` until its label shows in the resident's panel. A test state with needs topped
    up would save that.
  - Residents take a few seconds to sit down: a screenshot right after the label shows can catch them standing (the
    chess player seemed not to sit at first; they do).
  - `__controls.follow(sim)` pans to the resident but doesn't zoom; scroll the wheel over the middle of the canvas.
    The renderer's `setCameraPose` isn't exposed under `?debug`, so a wall thing above the resident (the dartboard)
    can't be framed in play; exposing it would help.
  - The clips bench aims at most 0.75 m up, so wall things need `__scene.activeCamera.target.y` set from the page; a
    `&target=` parameter would do it.
  - After a long session the household runs out of money and buys fail silently ("not bought"): use a fresh page
    for every few items.
- **Working in one checkout.** Another session reset `furniture.json` from HEAD and dropped this task's category
  edits, which had to be put back; the index is shared too, and other sessions had files staged. The commit was
  built from HEAD plus this task's edits only, in a temporary index (`GIT_INDEX_FILE`), with an `update-ref` that
  fails if HEAD moved, and the shared index updated for those paths, keeping what others had staged. Also, check a
  port with `lsof` before starting vite: with the port taken, `--strictPort` fails and `curl` answers from another
  session's server.
- **Validate holds about 47 groups**, against the plan's "about three at a time". Signing tasks off
  (16-sign-off.md) would make the catalog usable again.

## Garden plants and trees (G1–G5, built 2026-10-14)

`families/plants.py` (houseplants, flowers, shrubs, edibles) and `families/trees.py`; notes in `plants.md` and
`trees.md`. Fixed after the build: far versions over half their model's triangles (13) or more than 5 % off its size
(4), and the hydrangea's mopheads too low. Left to decide:

- **Performance isn't measured.** Shrubs dress every lot and trees stand by the thousand in the woods, all as thin
  instances. The models cost about what their placeholders did (a shrub 600–2,300 triangles, a tree 1,100–1,900,
  conifers 330–420), and the far woods now draw the files' far versions (120–700 triangles) where they drew the
  `*.far` placeholder keys (100–250). Run the render bench (`tools/bench/render.mjs`, `orbit-day`, Low and Medium)
  against the commit before G3, on the Intel HD 620 target if possible.
- **The far woods change** is in `nature.ts` `forests`: a model with a far file of its own draws that, not its
  `*.far` key. The `*.far` placeholder keys of the Blender trees (`model.tree.far`, `model.pine.far` …) are now
  unused except as fallbacks' fallbacks; remove them from `nature.py` at sign-off, or keep them for when a file
  fails to load.
- **Placeholder bounds overestimate turned round parts.** `check_glb.py` `placeholder_box`, like `models.ts`
  `partBox`, takes the box of each turned part's box, so a cone turned about its own axis counts its box's diagonal:
  the spruce's placeholder reads 7.8 m across for cones 5.4 m wide. That gives size warnings for the spruce, pine,
  cypress, oak, maple, palm, grass, geraniums and succulents, and a picking box and shadow bigger than the fallback
  draws. An exact box for round shapes (an ellipsoid's or a cylinder's, from the rotation) in both would clear them.
- **Four draw calls** on the bonsai, the apple tree and the lemon tree: the still part takes three (plain, bark or
  wood, foliage) and the moving part (a sprig, an apple, a lemon) one more. Making the bonsai's table and trunk plain,
  or dropping the clips, would bring them to three; kept, as within "at most 4".
- **Crops' growth stages** (`G.stages`) scale the leaves and the crop about the middle of the bed, so seedlings
  (planted) cluster towards the middle, most visibly on the pumpkins and strawberries. One node a plant would cost a
  draw call each. A stage clip that moves as well as scales (seedlings sunk into the soil instead of shrunk), or a
  gamekit `stages` that keys location too, would keep each plant in place.
- **A picked vegetable patch keeps its lettuces.** Its leaves stay at full size when picked (the `PLANTS` stages),
  and lettuces and cabbages are leaves; only the cabbage hearts and carrots go. Put the lettuces in the crop part if
  a harvested patch should look harvested.
- **A crop bought new is ripe** (no `growth` in the sim means ripe), so a resident harvests it straight away, as one
  did in play. Content or the sim could start a bought crop planted.
- **Colour parts differ from the plan's "Pot or box".** Mound beds colour their soil ("Bed") and, where the petals
  are one colour, the petals; shrubs without a pot colour their flowers (hydrangea, azalea, lavender, roses); the
  meadow and the trees have none, the lemon tree its pot. Decide at sign-off whether flowers should be colourable
  everywhere (several beds have mixed colours, which a single colour would flatten).
- **Houseplants have no far version**: they stand indoors, and the town overview draws gardens and streets. A
  houseplant put out in the garden is drawn whole there.
- **The lot dressing still mixes in placeholders**: `natureDecor` picks `model.bush.small`, `model.bush.flowering`
  and the street's `model.hedge`, which the plan doesn't cover, next to the Blender bush, hydrangea, roses, lavender
  and boxwood. Build them in `plants.py` (the `shrub` and `hedge_block` helpers make them) or point them at the
  Blender shrubs.
- **Wind and tint.** The leaves sway in the wind (checked: two frames 1.5 s apart differ over the fern and not over
  the cacti). The per-instance tint on thin instances, the other half of the nature shader, wasn't checked apart.
- **Far versions by hand, not by decimation.** gamekit's `far=0.25` decimates; on small, separate parts (flowers,
  leaves, lumps) it pushed vertices below the floor and shrank thin parts, failing the size check. `plants.py`'s
  `far_of` / `low` (the same build at low detail, drawing every random number the full build draws) worked for all
  43 outdoor plants and trees; it could move into gamekit for other families' far versions (street props, the farm).
- **Testing crops.** A crop's stages can be seen without waiting game days: write a state with `objects[].growth`
  set (−0.5 picked, 0.05 planted, 0.75 growing, 1 ripe) into `debug-states/` and open it with `?state=`.

## Living room (L1–L3, built 2026-10-14)

Fixed after the build:
- **The recliner's back tipped 8 cm into the wall.** Its seat and back turn back 25° about the sitter's hips, and
  it stood on the wall's face. It now stands 13 cm off the wall (the tipped back stops 4 cm short), with the hips
  12 cm forward to match (content `seat.forward` 0.12, `RC_HIPS`). Its placeholder (`catalog2.py`) was redrawn as
  the model stands at rest, footrest folded down on a skirted base, so the fallback, picking and shadow match it.
- **The bergère armchair was over its triangle target** (4,496 of 4,000): its back channels have fewer bevel
  segments now (3,880).
- **The Blender README sent play tests to `wants-asking`**, whose resident never takes a `use`. It now says so,
  and how to see a clip play (`__renderer.motion.states`).

Open:
- **`sideTable` has two placeholders.** The base manifest's (`furniture_prims.py` `side_table`, plain wood with a
  shelf) is drawn over by the catalog pack's (`catalog.py` `side_table`, oak with a drawer and a knob), which the
  model copies. `check_glb.py` warns that they differ (it did before the model). Keep one: drop the catalog
  override, or give `furniture.py` the catalog design. Either regenerates a manifest other sessions are editing.
- **The chaise's "Put your feet up" sits upright** at the back of the chaise (content `seat.forward` -0.3), the
  lower legs bent down into the long seat. A legs-out sitting pose, or lying with the head raised, would suit it
  (`characters.ts`, content `pose`).
- **One sitter sinks every seat cushion.** A sofa's seat cushions are one part (`seat`), so `sink_in` sinks them
  all. Per-slot cushions would need a clip per slot and the game to play the one under each resident.
- **Pillows don't follow the sinking seat**: they stay on the back cushions, up to 1 cm above the sunk seat.
- **The coffee table's placeholder is 6 cm taller** than the model (`check_glb.py` warns): it has magazines on the
  top, which the model keeps in its drawer. Dropping them from the placeholder would clear the warning.
- **The recliner's footrest, up, reaches 8 cm past the front of its tile**, and its seat front lifts 14 cm as it
  tips (a rocker-recliner, not a wall-hugger). Fine in play; a sliding mechanism would be closer to life.
- **The rocking chair's rock takes 1.4 s**, longer than the "about 1 s" rule, so it reads as a calm rock.
- **Clips still to add when content gives a use** ("+use"): the nesting tables' small table slides out (it's a part
  of its own, `small_table`).
- **The style variants have no models.** `catalog.py` still builds `@modern`, `@cozy` and `@minimal` placeholders
  (loveseat, three-seat and designer sofas, lounge chair, coffee and side tables), but no manifest lists them; only
  the default look is built.
- **Not checked**: two residents on one sofa, the colour studio's UI (colours checked on the clips bench), the
  catalog pictures, and the sofas on a town lot.
- **Testing in a shared checkout**: other sessions' edits reload the dev server's page mid-run, and once a content
  file was briefly invalid ("the game data failed to load") while one was being written. A port given up by one
  session (5293) was taken by another within minutes; pick a fresh one each run.

## Kitchen (K1–K9, built and reviewed 2026-10-14)

All 68 kitchen things are built (K1–K9) and sit in Validate. The review fixed what it found in the models
(bf5291a): the floor models now stand against the wall, hob flames last the whole cook, the ovens, dishwashers
and the microwave open onto an inside, the toast pops up, the island's drawer opens where the resident stands.
These are what's left.

### Placement

- **Other families' floor models stand off the wall too.** Centred on their tiles, they stand short of where their
  placeholders stood, against the wall. The kitchen is fixed by giving each `Model` its placeholder's `centre`,
  which `G.settle()` now uses (`gamekit/parts.py` `CENTRE`; README trap "Against a wall, on the floor"). Measured
  gap, model back vs placeholder back:
  - 15–22 cm: the bathroom sinks `sink`, `vanityDouble` (15), `sinkVessel`, `sinkConsole` (20), `sinkPedestal` (22);
  - 5–7 cm: `armoire`, `toyBox`, `toyChest`, `toiletWallHung`, `chestOfDrawers`, `dresserLow`, `fireplace`,
    `sofaBed`, `deskCorner`;
  - 3–4 cm: `wardrobeSingle`, `deskSimple`, `deskStanding`, `deskExecutive`, `bathtub`, `bathtubFreestanding`,
    `nightstand`, `nightstandLamp`, `grandfatherClock`, `lampFloorArc`, `filingCabinet`.

  Freestanding ones (`beanBag`, `rockingHorse`, `playTent`, the trees) can stay centred.
- **The gate doesn't catch it.** `check_glb.py` accepts a floor model centred on its tile even when its
  placeholder stands against the wall. A warning when the placeholder's back is at the wall face (depth ≤ −0.40)
  and the model's is more than 2 cm in front of it would have caught every case above.
- **Two counter depths.** The `catalog.py` counters and appliances (`counter`, `microwave`, `coffeeMachine`,
  `breakfastCounter`, `dishwasher`) are 0.66 m deep, the `catalog2.py` ones 0.79 m: in a mixed run the fronts step
  13 cm. One depth would mean moving their content `surface` (TOPS) with them.
- **The placeholders reach 1–5 cm into the wall**, and the models now follow them: hidden in the wall, but a
  wall cut away shows it.

### Residents and content

- **The kitchen island's stools.**
  - The island's model has two stools built in, as its placeholder did, but breakfast there is eaten standing
    beside them (`eat` doesn't sit at the island).
  - A resident making a sandwich stands at a stool's slot, half in the stool.
  - `housegen.json` also puts separate stools (`stoolCounter`, `stoolBarHigh`, `barStool`) in front of islands,
    so a generated `kitchenIsland` likely shows doubled stools.
  - Either seat breakfast on the built-in stools (a seat at 0.7 m), or drop them from the model and let the
    stools be objects.
- **The bar cart** is used from beside it, not from the front: the resident mixes at its side.
- **"(+use)" parts waiting for a use.** Each is built as a separate part with no clip: the drawer counter's three
  drawers, the wall cabinet's doors, the pedal bin's lid, the recycling bins' lids, and the doors of the
  sideboard, buffet and china cabinet.
- **`butcherBlock`'s surface is at 0.93 m**, but its top layer is at 0.945 m (from its TOPS entry): things on it
  sink 1.5 cm.
- **`diningChair`'s placeholder** in `catalog/manifest.json` differs from the base manifest's (a gate warning, as
  `consoleTable`'s): copy the current one.

### Clips and models

- **The retro fridge's lever.** The plan wanted the chrome lever to turn 30° before the door swings. That needs a
  clip that starts late (gamekit has no delay) and the lever as a child of the door node. For now it swings with
  the door; the plan row says so.
- **The hob counter (`cooktopCounter`) shows a blue gas flame on a ceramic hob**, as the plan asked. A glowing
  ring would suit it better, but `glow` (E8) is night only. Decide.
- **The microwave is mirrored**: its controls are on the viewer's left and its door is hinged on the right. Real
  ones have it the other way round.
- **Draw calls.**
  - Six models are at the 4-draw maximum: `fridgeFrenchDoor`, `wineFridge`, `stoveGas`, `rangeCooker`,
    `microwave`, `breakfastCounter`.
  - Tiny textured decorations cost a draw each: `counterCorner`'s wooden spoons and `fridgeMini`'s board. Mapping
    them to a plain finish would save one.
- **Validate is crowded.** The kitchen alone has nine groups in it: fridges, cookers, appliances, counters,
  kitchen storage, islands, tables, chairs and dining storage. Sign the kitchen off (16-sign-off.md) after a look
  in play.
- **Not checked**: all four facings of each model, the colour studio, night lighting, and catalog pictures.

## Simulation speed (checked 2026-10-14)

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  timing the whole sim-core suite with and without it showed no difference (about 18 s either way; towns are ~60
  Sims). It catches `.partners` changed through `get_mut` instead of `set_partners`. `Sim::refresh_buffs`' extra
  walks over `feelings` (a few entries) aren't worth a change either.
