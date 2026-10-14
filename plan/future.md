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
