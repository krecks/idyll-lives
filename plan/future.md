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
- **The bedroom mirrors are `gloss` too (fixed 2026-10-14).** `mirror`, `mirrorFloor`, `mirrorWall` and
  `dressingTable` (`families/bedroom_dressing.py`) dropped their silvering plates, so each saves a draw call. Re-exported and
  validated; not looked at again in the game.

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

## Lighting (LT1–LT4, 2026-10-14)

None of these blocks sign-off. Each says where it lives.

- **The gate's size warnings on four-sided shades are false alarms.** `floorLampSimple`, `lampTableSquare` and
  `lampLantern` read as about half their placeholder's width: the gate measures a 4-segment cylinder turned
  45° by its unturned box. The drawn sizes match (checked side by side in the gallery). Teaching
  `check_glb.py` to measure turned placeholders would clear them for every square shade and roof.
- **Small depth and height differences from the placeholders.** `trackLights` and `wallLightOutdoor` are 3 cm
  deeper, `wallLampArm` 5 cm lower than the gate's estimate of the tilted placeholder shade. Picking and the
  contact shadow follow the placeholder, so these are not visible. Adjust the placeholders in
  `tools/art/catalog.py` and `catalog2.py` if they should agree exactly.
- **The pendant's light sits 20 cm above its bulb.** The content's `light.height` (2.1 m) is the top of the dome,
  as in the placeholder; the bulb hangs at 1.9 m. `_near_light` allows 25 cm. Lowering the content height
  to 1.9 m would put the light where the bulb is, but changes how the room is lit, so it is a content
  decision.
- **Not checked in play.** The surface lamps (`lampTableRound`, `lampTableSquare`, `lampBanker`,
  `lampMushroom`, `lampLantern`) were seen in the gallery but not placed on a table in a house. The wall
  lights were placed at one facing only (the test wall exists at one rotation). The clips bench cannot frame
  ceiling-height models, so the fan and lantern clips were checked in play instead. Do these at sign-off.
- **Ambient clips never show on Low.** The fan and the lantern are still on the Low preset and, when the game
  is paused, hold their pose. That is the engine's rule (E2); add a still-looking pose only if it matters.
- **A lamp has no on/off.** All lamps glow with the room's lamp level. A switch (per lamp or per room) would be
  an engine and content task, not a model one; the shades already carry `glow`.
- **Validate is over its limit.** About 25 groups are open now, against the plan's "about three at a time".
  Sign off finished tasks (`16-sign-off.md`) before opening more.
- **Shared content files.** Other sessions were editing `base.json`, `furniture.json`, `CREDITS.md` and the asset
  manifest while this task ran, so the commit was built from `HEAD` plus only this task's hunks. A
  sign-off/Validate tool that moves items by id would remove the need for hand-staging (see above).

## Bedroom (BR1–BR4, 2026-10-14)

None of these blocks sign-off. Each says where it lives.

- **The bunk's lower duvet turns down when someone sleeps in the upper bunk.** `bedBunk` has one `turn_down`
  clip for the whole object, played for any `sleep` or `nap`. Fixing it needs the engine to pick a clip by bunk
  slot (`BUNKS` in `characters.ts` already knows the two). Not tested with two sleepers at once.
- **The futon's placeholder is the bed lying flat (0.66 m high)**, while the model at rest is the folded sofa
  (0.99 m). The fallback, contact shadow and picking box are lower than what is drawn. The `check_glb.py` warning
  is expected. Raise the placeholder in `tools/art/catalog.py` if the fallback ever shows.
- **The dressing table faces the wall**: the stool is against the wall and the player sees the mirror's back.
  That is the placeholder's layout, kept. Turning it round means moving the seat's `fwd` in `SEATS`.
- **Standing mirrors stand upright**, where the placeholders lean back 6° and 8°. The game lays the reflection on
  the glass's box, so a tilted pane would reflect beside it. Revisit if the engine reflects tilted glass.
- **The four-poster has an open top** (a valance and curtains at the posts), not the placeholder's solid roof, so
  sleepers stay visible from above. The curtains read as pale strips from the game camera.
- **The dresser has two small drawers over two wide ones**, not the placeholder's 2×3, so "the middle drawer" is
  one drawer. The armoire and `nightstandDrawers` use satin, not wood, to stay within 3 draw calls.
- **Several bedroom models are off centre from their placeholders** (`nightstand`, `nightstandLamp`,
  `chestOfDrawers`, `dresserLow`, `armoire`): the placeholders reach 3.5–7 cm into the wall. Moving them forward
  would clear the warnings.
- **(+use) parts with no clip yet**: the `nightstandDrawers` top drawer, the `cedarChest` lid and the
  `laundryHamper` lid are built separate. Add `drawer_open`, `lid_open` and a lid lift when content gives them a use.
- **Mirrors reflect on High only**, and WebGL logs an occasional `GL_INVALID_VALUE: glGetProgramiv` there. It also
  shows with the placeholders, so it is the engine's mirror path, not the files.
- **Not checked**: all four facings, the colour studio, the beds at night, the nap clip in play, and two sleepers
  at once.

## Office (O1–O3, 2026-10-14)

- **Desk content surfaces reach into the wall.** `deskSimple`, `deskStanding` and `deskExecutive` have a `surface`
  rectangle (and placeholder) back edge at -0.46, past the wall face (-0.43); the models stop at the wall, so
  their tops are 3 cm shallower. Trim the `surface` rectangles in `catalog2/office.json` (and the placeholders in
  `tools/art/catalog2.py`) if things placed on them should not hang over the wall.
- **Things that need a use first ("+use").** The executive desk's middle drawer and the filing cabinet's top
  drawer are built as parts of their own, still. Add a `G.slide` and `motions` once content gives them an
  interaction (a resident filing papers, say).
- **The desks' own chairs don't swivel.** The computer, writing, laptop and gaming desks carry their placeholder's
  chair inside the still desk, because residents sit there. If they should turn, the chair would become a moving
  part of each desk, with `ride`, at the cost of a draw call per desk.
- **The chair swivel is not seen in play.** I only saw it in the clips bench: in the headless runs the resident
  left for the TV before sitting. Check it with a state where the sim is needy-free and next to the chair (see
  Testing in the browser).
- **The filing cabinet is 17 cm lower than its placeholder**, whose potted plant stood taller. The gate warns of
  it; the plant could be made taller if the silhouette matters.
- **Draw calls.** The computer desk, writing desk, laptop desk, gaming PC and standing desk each take 3 (still part,
  moving part, plus textured wood). Fine for Medium; recheck on the Intel HD 620 target with several in view.
- **One Validate group per task.** After the office there are 46 groups in Validate; sign-off is overdue (see
  Validate and sign-off).

## Fitness (F1, 2026-10-14)

- **Lifting draws nothing in the hands.** The `lift` animation is `pickUp` standing and a bench press lying, with
  no bar or dumbbell. So the bench's bar and the rack's dumbbells stay put (the plan's `bar_off` and `pair_off`
  clips aren't built). If `lift` gets a held weight (a `tool.*` prop, as `STATION_PROPS`), join the bar and one
  pair separate in `fitness.py` and `G.squash` them to 0.001 for the use, so there are never two.
- **The bike's rider doesn't pedal.** The `exercise` sitting pose keeps the legs still while the pedals turn. A
  pedalling leg cycle in `characters.ts`, timed to `pedals_turn` (1 turn in 1.2 s), would match them.
- **The treadmill's belt isn't timed to the jog.** The slats run at 0.2 m/s, slower than a jog, so they read as
  movement, not speed. Faster slats (a shorter `belt_run`) blur at 30 samples a second; check before changing.
- **`G.scroll` relies on the game jumping a loop home.** It's seamless only while the part's ends are hidden and
  the move is exactly one repeat. `check_glb.py` warns of every even-speed loop for that reason; a future conveyor
  (a shop till, a factory) must hide its ends the same way.
- **The rolled yoga mat is much smaller than its placeholder** (the flat mat). Picking and the shadow follow the
  placeholder, the whole tile, which suits the unrolled mat. `check_glb.py` warns of it; it's expected.
- **Thin floor things must be over 1 cm thick.** The house's floor is drawn 1 cm up, so a flat part lower than that
  disappears in a house (it showed in the clips bench, on plain ground). Worth a `check_glb.py` warning for a
  moving or floor-layer part whose top is under 1.2 cm.
- **Treadmill runners stand on the belt by def name** (`useDef.includes('treadmill')`, as the trampoline). A pack
  treadmill would get it for free; a renamed one wouldn't. A content field (a standing height, like `seat`) would
  be cleaner if more equipment needs it.
- **Validate took away free turning (fixed).** Free turning comes from the category (`base.json`
  `objectRules.freeRotation`), so items moved to `validate` lost it and `public_places` failed 4 tests. `validate`
  is now in that list. Side effect: while in Validate, any 1x1 non-wall item can turn, and a save that turned one
  would fail to load once it returns to a category without turning. Early release, so left; the exact fix is a
  per-item `"freeRotation"` on the items that had it, which sign-off (`16-sign-off.md`) could carry back.
- **`retirement::the_basic_pension_pays_for_a_home_of_one` fails** (3 rent payments in 4 weeks, expected 4). It
  fails the same with and without the F1 and `freeRotation` changes, so it comes from other work in the tree
  (sim timing or content). Not looked into.
- **Not checked in play**: the belt loop's wrap close up (checked in the clips bench and by its numbers), two
  residents on one item, and the fitness items on a town lot.

## Decor (D1–D3, 2026-10-14)

None of these blocks sign-off. Each says where it lives.

- **Fixed: residents warmed up sitting inside the fire.** The fireplace had no `seat`, so the `relax` sit used the
  default (0.45 m at the footprint's middle), which is the firebox's mouth. The model now has a raised hearth bench
  (0.42 m, `HEARTH` in `families/decor.py`) with the firebox above it, and the content has
  `"seat": {"height": 0.42, "forward": 0.02}`. Residents sit on the bench with the fire behind them (checked in
  play, WebGPU and WebGL). The fire's `light.height` went from 0.5 to 0.7 m, where the flames now are. The
  placeholder (`tools/art/catalog.py` `fireplace`) still has a floor-level hearth, so if the file fails to load,
  the resident sits in the air in front of it. Raise the placeholder's hearth to match.
- **Wall things can hang over windows.** `backed_by_wall` (`crates/sim-core/src/world.rs`) accepts a window edge,
  so the curtains can hang there, but so can a wall clock or a painting (seen in play: the clock over a window).
  A content flag for the things that belong over a window (curtains, blinds) would keep the rest off them.
- **Wall and ceiling things overlap tall floor things.** A painting bought on the fireplace's tile disappears
  inside the chimney breast. A hanging plant over a bookcase hangs into it, and residents walk through its pot,
  which hangs 0.97–1.5 m up (as the placeholder did). Floor things that fill the wall's height (fireplace,
  grandfather clock, wardrobes) could take the wall slot, and low ceiling things could block the tile.
- **Placeholders that reach into the wall.** `grandfatherClock` (the base is 4 cm in) and `fireplace` (5 cm) reach
  past the wall face (-0.43). The models stop at the face, so `check_glb.py` warns they're off centre. The curtain
  placeholders stand 5 cm off the wall, while the models' rail brackets reach it. Moving the placeholders in
  `tools/art/catalog.py` would clear the warnings.
- **The clocks don't tell the game's time.** The wall clock's hour and minute hands are fixed at ten past ten. Only
  the second hand goes round (`hands`, one turn in 60 s), which is much longer than the usual ~1 s clip. Ambient
  clips run on game time, so the hand speeds up with the game. A motion kind that sets a clip's time from the game
  clock would let both clocks show the hour (an engine task, like E4's growth).
- **The grandfather clock takes 4 draw calls**, the most allowed: the walnut body (plain plus wood), the door (wood)
  and the brass pendulum. The trunk door has an open window, not glass, because glass would be a fifth. Its
  pendulum is ambient, so it's still on Low.
- **The curtains stretch to close.** `close` scales each gathered panel 2.2× (`curtains`) and 2.7×
  (`curtainsLong`) along the rail, so the pleats, the fabric texture and the cloth tabs widen with it. It reads
  fine from the game camera. If it shows up close, use a separate drawn panel that grows instead. Closed at night
  in play (checked for `curtains`; `curtainsLong` only on the clips bench).
- **The hanging plant sways twice.** `sway` is a use clip that ends at rest (`G.rock`), held 1.6 s and then played
  back, so it swings, settles, and swings once more. That was seen on the clips bench. In play the resident
  watered it, but the camera didn't catch the sway.
- **The pictures are flat shapes**, not textures, so each costs one draw call with its frame. The budget allows one
  512 px texture per object, if the art should look painted.
- **Art polish.** The ginger jar's sprigs read as small crosses from the game camera. The cat statue's haunch is a
  plain ball, and the coat on the coat stand is a capsule.
- **Not checked**: the catalogue pictures; the wall things at all four facings (the test wall has one rotation);
  `statueHorse` was bought at three facings, and the fourth buy was refused (not looked into, probably its front
  tile was taken); the fireplace's light at night since the change to 0.7 m.

## Outdoor (OU1 patio, OU2 loungers; 2026-10-14)

- **Fixed:** the parasol canopy was 2.04 m across (an octagon with its flats on the axes), against the placeholder's
  2.2 m. It is 2.2 m flat to flat now.
- **The teak patio table is 4 cm off centre** (`check_glb.py` warning). Its planks follow the content's top
  (-0.43 to 0.35 deep), but the placeholder's aprons reached to 0.40. Widening the model to 0.83 m deep, or
  trimming the placeholder in `tools/art/catalog2.py`, would clear it.
- **The hot tub cover stays up after a soak.** Soak `leaves: open`, so the cover holds up until someone does "Put the
  cover on" (`coverOn`, not autonomous). Tried once: the resident went off to chat instead of capping it. Worth an
  autonomous trigger or a timeout, and a look at why the order didn't take.
- **Residents didn't always reach a chair or swing in a crowded garden.** With eleven things placed in a row, the
  patio chair, swing set and hot tub were sometimes skipped or took a long walk. They worked alone with free tiles
  in front. Probably the approach tile was taken; not looked into.
- **Seating at tables.** The patio chair uses `seatAt: table`, and sat correctly at the round table. The bistro set's
  two chairs are part of one object, so only the content's two slots seat at it.
- **Not checked:** OU2 on WebGL2 and at night, the colour studio on these models, the catalogue pictures, and the
  hot tub's `coverOn` clip in play. The clips bench was only run for the parasol.
- **The parasol's `stir` is gentle** (1.2° in 2.4 s). It reads on the clips bench; in play from the game camera it
  may be too small to notice.
- **Hammock and sun lounger sizes** were set from the built model (not the placeholder): the lounger is 4.7 cm
  deeper than its placeholder, with the back raised 35°.

## Working in one checkout

- **A shared file was truncated (2026-10-14, decor session).** A one-liner,
  `open(p, 'w').write(fix(open(p).read()))`, opens the file for writing (emptying it) before `fix` runs. `fix`
  failed, and `web/public/content/furniture.json` was left empty. It was restored from `HEAD`. The lost uncommitted
  edits were re-applied: fitness, sofas and patio by the decor session, hobbies and outdoor loungers by their own. Write to a shared file
  only after the new text is built and parsed: `t = fix(s); json.loads(t); open(p, 'w').write(t)`.
- **Sessions sharing one checkout** also commit each other's staged files by accident, and race on `base.json`,
  `furniture.json`, `CREDITS.md` and the asset manifest. A worktree per session (as the personality-packs session
  uses) and the Validate tool suggested above would remove most of this.
# Future: things to consider

Notes that came up while doing the plan and don't belong to a task yet. Add to the end.

## Tabletop (TT1–TT5, built 2026-10-14)

- **Validate is crowded.** It holds about 25 groups; README asks for about three at a time. Sign off
  (16-sign-off.md) some groups before opening more.
- **Wide items don't fit a counter.** `plateSet`, `teapotSet`, `espressoMachineSmall` and `monitor` have content
  `radius` 0.23–0.3 m (taken from the placeholders), more than a counter top's depth with its edge margin, so a
  counter refuses them; tables and desks take them. Either shrink the radius in `catalog2/tabletop.json` or leave
  them for tables.
- **Mirrored items.** The Blender build and the game's placeholders are mirrored in x. The monitor and the place
  setting were flipped (`done(..., mirror=True)` in `families/tabletop.py`) to match their placeholders; the
  chopping board matches as built. Check any new asymmetric model in the gallery next to its placeholder.
- **Clips to add when uses exist** (the parts are already separate, hinged where they'd turn):
  - `kettle`: the `lid` lifts about Blender X;
  - `breadBin`: the `roll_top` rolls back about the bin's axis;
  - `laptopClosed`: the `lid` opens about its back edge.
- **The clock's `hands` clip is 60 s** (once round a minute), against the usual "about 1 s". Check how the game
  samples long ambient clips on Low-end laptops; it is off on Low.
- **Not checked in play:** the colour studio UI itself (colours were bought through the `buy` command), and a
  close WebGL view in a house (only the gallery was checked on WebGL).
- **Draw calls.** Seven models take two (see-through glass: `cookieJar`, `wineGlasses`, `snowGlobe`; separate lid or
  hand: `kettle`, `breadBin`, `laptopClosed`, `tableClock`). Opaque glass would save one each.

## Gate

- `check_glb.py` fails three garden shrubs (`bush`, `garden.azalea`, `garden.rosebush`) on their far versions
  reaching 0.5–1.1 cm below the floor. Not from tabletop; found while running the whole gate.

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

## Personality packs (P1–P16, built 2026-10-14)

All 94 pack models are built (`families/pack_<trait>.py`, notes in `pack_<trait>.md`) on branch `v0.3/packs`
(worktree `.claude/worktrees/v03-packs`), not yet merged into `v0.3/object-motion`. `check_glb.py`: 151 models,
0 failed; Khronos validator 0/0 on every pack file; `shipped_content` passes.

### Merging and Validate

- **Not merged yet.** `v0.3/packs` branches from `bb8baab`. Merging it will conflict in `base.json` (the
  `validate` groups), the base manifest's `packs` list and `CREDITS.md`: keep both sides (the packs only append).
  It also carries the renderer and toolkit fixes below (`check_glb.py`, `models.ts`, `nature.ts`,
  `characters.ts`, `gamekit/motion.py`).
- **Validate holds 16 more groups**: bookworm, cheerful, couchPotato, energetic, example, foodie, gloomy, hotHeaded,
  kind, lazy, loner, natureLover, neat, outgoing, romantic, slob. Sign them off pack by pack; "Back to" is each
  pack table's column (the pack objects had no group before).
- **The example pack's own "Wellness" category** is empty while its two objects are in Validate.

### Fixed

- **`check_glb.py` crashed on any model with mirror glass**: the flatness check reused the name `size`
  (`b345f0f`).
- **A moving part grown from nothing in a textured finish never showed** (a foliage seedling, WebGPU and WebGL).
  It is baked flat at rest, so its normals weren't unit length and its UVs collapsed (the normal map drew
  nothing), and the leaf flutter, added before the instance matrix, was stretched 1000× along its height. It now
  takes its normals, winding and UVs from its grown shape, and the flutter is divided by the instance's scale
  (`e0030eb`). At rest the flutter lifted its flat shape into view; grown parts no longer flutter (`f9822b8`).
- **Residents lay across the 2×1 pack hammocks** (`rainbowHammock`, `gardenHammock`): a `BEDS` entry can say
  `across`, and they now lie along the sling, head by the pillow (`8bc2668`).
- **The two pack recliners' backs reclined behind an upright sitter**: both ride the `recline` clip
  (`22bd4c1`).
- **Bathers sat on the heart tub's front rim**, shins through it, where the heart narrows: `SEATS` moves them
  36 cm back into the lobes (`2106cc2`).
- **The dance floor keyed its tiles through gamekit's private `motion._clip`**: `G.turns` (parts taking turns in
  one clip) is public now (`d7b0d65`).

### Still to decide or do

- **Hammocks**: residents lie straight in a sagging sling, so their feet reach past one spreader bar and the
  pillow sits beside the head rather than under it. A lying pose that follows the sag would fit better.
- **Grown parts' normals follow the pose matrix, not its inverse transpose**, so a grown part's slanted faces
  shade as if they faced along the grow axis (water, flames, petals; hardly visible).
- **One part plays one clip's pose at a time**, so a part can't swing and spin: the jukebox's record swings up but
  doesn't spin (plan's L E1); the power blender's blades aren't built (the smoothie would hide them anyway).
- **No per-shot event**: the streetball net sways when shooting starts and ends, not after each shot.
- **The `clean` animation holds no prop**: the steam mop stays leaning out of its dock while mopping.
- **Uses that play no clip**: `loner.bonsaiBench` `tend`, `natureLover.veggieBed` `tendVeggies`,
  `natureLover.pottingBench` `tieBouquet`.
- **Eight models are at 4 draw calls** (the most; the gate warns): `chefStove` (knob and flame ring),
  `whirlpoolTub` (the bubbles loop; a plain cedar surround would make it 3 but lose the wood), `foodie.pizzaOven`
  (door, peel and fire), `gloomy.recordCabinet` (platter loop), `couchPotato.lavaLamp` (glass and drifting wax),
  `loner.soloSauna` (glass door), `natureLover.glasshouse` (glass door and vent), `natureLover.veggieBed` (plants,
  fruit and lettuce apart for the growth stages).
- **Models smaller or larger than their placeholders** (the fallback, if a file fails to load):
  `loner.shortwaveRadio` (1.33 m tall at rest, its antenna up only in use; placeholder 1.77), `lazy.napPod` (0.94 m
  deep, inside its tile; placeholder 1.15 overhangs it), `couchPotato.recliner` (0.76 deep, placeholder 0.89 with
  the footrest out), `couchPotato.snackFridge` (0.58 wide), `energetic.trampoline` (0.63 tall; placeholder 0.83),
  `energetic.recoveryPod` (1.18 tall, its canopy open at rest). Update the placeholders, or accept. (The gate's
  warning says picking and shadow follow the placeholder; the renderer seems to click by the drawn outline.
  Check, and correct the message if so.)
- **Placeholders residents go through**: the chef's island's front panel (seated legs), the gloomy journal desk's
  apron (hands); the models fixed both. The chef's island top is 0.9 m as its content `surface.height`; the
  placeholder's is 0.83. The chef's stove placeholder sits 2.5 cm inside the wall.
- **Changes from the plan worth a look**: the fireside lounge has an open grate under a hood (the placeholder's
  stove hid its fire from the camera); the library wall has no fabric (colour parts Wood, Ladder); the cello's music
  stand is gone (it stood where the cello is played); the rain lamp has no glass; the gloomy window seat's pane is
  opaque; the neat entry console's mirror is chrome, not live mirror glass; several soft or wooden parts are plain
  satin to stay at 3 draws (named in each `.md`); some colour labels differ from the plan's column.

### Not checked

- Uses seen only on the clips bench, never in play: the tea trolley, bureau, bedside stack and library wall
  (bookworm); karaoke, photo wall, vanity, letter bureau, `tendRoses`, spritz, `quickRound`, `armrestSoda`
  (P14–P16); the puzzle table, bonsai bench, potting bench, terrarium, `pickSnack` (P11–P13); the quilt flap and the
  giving jar's coin (P9–P10); the mocktail shaker (P2); the espresso bar's coffee (hidden behind the resident).
- The hammock and heart-tub fixes were checked on the pose bench (`/poses.html`), not with a resident in play:
  in `wants-asking`, R0 walks off to other things even after a `cancel` and `use`.
- Short-hold clips (the bar's tap, the microwave door) not caught mid-motion in play.
- The colour studio UI (colours were checked by buying with `colors` and on the clips bench).
- Night only on WebGL; not every model at all four facings; catalog pictures.
- `debug-states/office-night.json` isn't in the main checkout (the agents copied it from a worktree). Buy
  `colors` take integers, not `"#hex"` strings.
## Simulation speed (checked 2026-10-14)

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  timing the whole sim-core suite with and without it showed no difference (about 18 s either way; towns are ~60
  Sims). It catches `.partners` changed through `get_mut` instead of `set_partners`. `Sim::refresh_buffs`' extra
  walks over `feelings` (a few entries) aren't worth a change either.
