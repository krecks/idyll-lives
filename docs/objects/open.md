# Objects: decisions to review, and what is still open

The calls taken while the objects were built, checked and fixed (2026-10-14 to 2026-10-17), for the user to review,
and what is known not to be right yet. Each decision is reversible; its commit says how. What
a single object does is on its category page ([README.md](README.md)); this page collects what cuts across them.

## Decisions to review

### Engine and rules
- **Lamps** switch per lamp for the player (click: Turn on / Turn off), per room for residents, instantly (nobody
  walks to the switch): on when someone awake comes into a dark room (19:00–06:30, or a basement), off when the
  last one leaves or sleeps. Fires (`light.switch: false`) and garden lamps aren't residents' to switch; with free
  will off, lamps stay as the player left them. Room scores count every lamp. Save v28 (`lights.rs`). A lamp switched
  off stays off when moved, with its room or on a top, and through undo and redo.
- **A lamp's light** sits at its shade (content `light.at` [x, z], object frame): the arc, table and nightstand lamps.
  The wall lights keep theirs out from the wall (at the bulb they lost the warm pool on the wall), the fireplaces in
  front of the fire.
- **Clocks tell the game's time** on every preset (state, not ambient), updated each game minute (every 5 on Low),
  within 30 m of the camera or the ambient range (`motions.clock`, `CLOCK_RANGE`). No second hands: at speed 1 a game minute is a real second.
- **Ambient clips** stay still on Low and hold while paused (E2); clocks are the exception.
- **Things over windows:** only `overWindow` things (the curtains) hang over one; a window can't be built behind a
  picture. **Tall floor things** (`fillsWall`, 35 of them: fridges, wardrobes, showers, the fireplace…) keep the wall
  and ceiling above them clear; the hanging plant (`hangsLow`) blocks the floor under it.
- **Wall things and cut walls:** a wall thing hides whenever the cutaway drops its wall, from either side; hidden, a
  lamp still lights and a dartboard or mirror can still be used. While one is in hand, wall things show (walls down
  or cut away), and a taken spot names what hangs there.
- **No offset through a diagonal wall**, in the sim and build mode (straight walls already refused it).
- **A bought crop starts planted** (growth 0); crops already on a lot or in a house file start ripe.
- **The hot tub:** the last one out puts the cover back on (`closeAfter` on Soak). An order to close something
  someone else got into first is dropped with a story line ("… didn't close the cedar hot tub: someone else got in
  first", event `notClosed`), not queued.
- **An ordered use holds against needs** with free will off.
- **A whole use** (content `whole`: the naps on the window seat and hearth settee) starts only with nobody else on
  the object and fills every slot. An order for one while someone is on the object is refused at once ("someone is
  using the …"), even if it is queued and the seat might be free by then; an order to sit beside a napper is taken
  and silently dropped when it comes up, as for any full object. Both could instead wait for a free place (a general
  change to how full objects take orders). A tired resident doesn't choose the nap while someone sits there.
- **Putting clean things away** goes to the nearest cupboard or counter (`cupboard` station, `putAwayAt:
  ["cupboard", "prep"]`, `cooking.rs`); the wall cabinets, sideboard, buffet and china cabinet open a door for it.
- **New uses with clips** are not autonomous, so generated houses behave as before: "Brush your teeth" (mirror
  cabinet), "Take out a blanket" (cedar chest), "File papers" (filing cabinet), "Take a fresh towel" (tall bathroom
  cabinet), "Sort the washing" (hamper), "Sort the recycling", "Pick an outfit" (three-drawer nightstand).
- **The gate** warns of a use clip over 3 s (loops, `idle`, `night`, `open`, growth are left alone); it skips the
  `outside` groups by design, and the wall-face check for `stairs: true`; a model may reach below the floor only where the
  thing it stands for does (a rock sunk in the ground, the selection marker).

### Residents with objects
- **Held things** are drawn in the object's colours; the weight bench's bar leaves its hooks 0.8 s in.
- **Seats facing the object's back** (`seat.facing: "back"`): the dressing table, the piano, the writing bureau, the
  cinema wall and the game console; the console's sitters sit 8 cm further back, so their knees clear the shelf. The
  digital piano and sewing table face the room (they read as tables).
- **Chairs at a separate table** are drawn pulled in by up to 0.45 m while sat on (`tableTuck.ts`; the sim's
  positions don't change), overlapping into the table's tile; a plain "sit" on a dining chair pulls it in too. The
  executive chair comes in only 6 cm (its desk's pedestals are where the knees go): the writer leans in further
  (`sitHandsFar`) rather than sliding sideways to the knee hole.
- **Lying:** heels sink about as deep as the buttocks (4–5 cm into the mattress) and the calves and thighs touch the
  top or stand 1–2 cm proud (a straight leg can't lie flatter without bending the knee backwards). Children lie on
  the mattress (`lieBack` scaled by size), so their heads sink 3.6 cm into the pillow, as grown-ups' do.
- **Bathers** sit along the tub; the sofa bed's user stands 1 m back; the lazy pack's doze and power nap are sitting
  naps; the leather armchair reads sitting down.
- **Hugs:** each steps in until the chests meet, the arms round the other's back; a grown-up hugging a child bends
  over (hands on the child's shoulders) rather than crouching, whose knees would push the child back. The teddy is
  hugged crouching, grown-ups too (standing doesn't reach past its feet).
- **Herbs are picked standing** (`pick`, `STAND_BACK` -0.5: a negative stand-back steps in; the planter's watering
  too). The four desks with a chair built in close write with `sitHandsNear`.
- **The drinking fountain** is drunk from bent over the spout (`fountainDrink<cm>`, five heights). **The phone box's
  door opens outwards**, the caller steps in and it shuts behind them, then opens to let them out (motions `out`).
- **The climbing frame** is climbed round in 13 s (was 8 s): up the net, across the deck, down the fireman's pole;
  the net-only round is gone. Grown-ups cross the deck in a deep crouch (the roof leaves about 1.1 m), children
  nearly upright. A second climber takes the other half of the net and waits on the deck for the first to slide.
- **`seat.recline`:** the Adirondack chair 24° (hips 10 cm back), the garden chair 18°.

### Models
- **One counter depth, 0.66 m**, and **every kitchen worktop 0.95 m** (cookers and the foodie chef's island keep
  their own heights); the hob glass stands 4 mm proud.
- **Islands** have no built-in stools and are turned round, cupboards at the front where residents stand, the top
  overhanging the stools housegen puts behind; their modelled tops are empty.
- **Beds keep their place; the head end is tucked:** headboards 3.5–4 cm thick, so mattresses, pillows and sleepers
  stay put and no foot passes its tile. The six-seat, farmhouse and banquet tables are 0.88 m deep, as the four-seat.
- **A quilt per body**, shaped over its own lying pose, coming up from the foot once the sleeper lies flat; the
  turned-down duvet flattens and narrows under it.
- **Finishes reach the shader** (gloss, chrome and metal); metals reflect the sky just above the horizon where they
  would reflect the ground (`SKY_FOR_GROUND = 0.3`). **Hardware is metal** (handles, knobs, pulls); the dresser, low
  dresser, dressing table and writing bureau went to 4 draws for it.
- **Mirrors reflect on every preset:** the nearest on screen on Low (1024×512, 10 a second) and Medium (4× MSAA, 30 a
  second), redrawn only when something it shows changes; two live on High. Medium with a mirror in view and
  something moving costs about 6 % fps and 1 ms CPU on the M3.
- **Kept at 4 draws:** the washer and dryer, the phone box, the potting bench, the grandfather clock (hands moving,
  door and pendulum still), the jukebox, the bunk, the greenhouse, the stereo, and the eight pack models `chefStove`,
  `whirlpoolTub`, `pizzaOven`, `recordCabinet`, `lavaLamp`, `soloSauna`, `glasshouse`, `veggieBed`.
- **Kept as they are:** pictures as flat shapes (a texture would double their draws); one sitter sinking every sofa
  cushion; pillows not following; the foosball's two rods; the curtains stretching to close; standing mirrors
  upright; the base shower's sliding door shut; the retired `@modern`/`@cozy`/`@minimal` style variants.
- **Colour parts:** the wardrobe has three (Body, Doors, Handles); the sewing table's legs, apron and drawer have
  their own (Legs); the acoustic guitar and ukulele have a "Top"; the books and bookends list Book 1–5; the garden
  decor has colour parts; flowers are colourable only where their petals are one colour.
- **Text:** the horse statue is "Bronze horse", the pedestal vase "blue"; the stairs are in living with no group.
- **The colour studio** opens the item as in use once a part or colour is picked, has an "At rest / In use" switch,
  runs ambient clips on every preset, frames a moving part by its posed vertices and turns a model about the middle
  of its bounds (`itemStage.ts`).
- **Catalog pictures:** the smallest view is 0.6 m across, and the contact shadow is the model's own size.
- **Speed and size picks:** the treadmill belt runs 0.4 m/s (faster blurs at 2.5× speed); the parasol stirs 3°;
  houseplants turn without touching the wall (at most 44 cm from their middle).

### Generated houses
- Cooking stations count as a use (an island's stools go behind, its front stays clear); free tables try both
  facings; a wall thing with a use hangs only where its front tile is free and reachable.
- Small things stay off hobs, grills and sinks; curtains hang only over a window. The generated houses check fails
  on either.
- Islands with stools in 34 of 60 large houses (the check wants a third); stairs keep off a room's only outside wall;
  furniture never shuts free floor off.

## Still open

### Residents
- **The weight bench's lie:** the legs stretch out past the bench's end in mid-air (a bench press would have the feet
  on the floor either side). Deferred from 0.3.0: it needs a bespoke feet-down lie pose, not a data tweak.
- **A kiss between residents of different heights:** the taller one's mouth meets the other's forehead. It needs the
  taller one to bend and the shorter one to tilt the head up. Deferred from 0.3.0: needs two-body procedural work.
- Other children gather round a hug and stand right next to the pair.
- **Chairs and tops the tuck doesn't reach:** an office chair at the standing desk (top 0.57 m over the seat), a kids'
  chair at a coffee table (the hands hover), a chair facing a desk with its own chair built in (laptop desk, kids'
  desk) from behind. Deferred from 0.3.0: each needs its own reach/clip adjustment.
- **Getting up from a chair:** the sim puts the resident on the tile in front at once, so `standUp` plays while they
  glide back. Deferred from 0.3.0 unless the stand-up animation is reworked.
- **Climbers** who start at other offsets could still meet at the pole.
- **`jogAlong` has a seam** where its loop wraps (`calf_r`, `foot_r`; since before fbf8883). Its feet don't slide:
  `check_clips.ts` had tested them as standing still. This and the clips' failures on other bodies and life stages
  are the known ones in `tools/characters/clip-baseline.json`.

### Rendering
- **The town map's backdrop** blocks the world like the old pie menu's did (left as a panel).
- **Night glow draws through other parts** of a model (the grill's coals through its shut lid).
- **Stairs seen from some basement views** lose part of their top end (a renderer cut-away, not the model).
- **Cooking props** (chopping board, food) are drawn at a station's middle, which on `grillStation` is the grate.
- **Wall things shown while one is held** float over lowered walls (pictures on outside walls too).
- **Mirrors on High** log an occasional `GL_INVALID_VALUE: glGetProgramiv`; not reproduced in 5 runs, probably
  Babylon's parallel shader compile.
- **`partBox`** draws spheres and tori up to 5 % / 2.5 % bigger than Babylon's meshes (never smaller).

### Content and sim
- **Parts with no use, left decorative for 0.3.0:** the executive desk's drawer (its clip would need a seated use
  on the desk it faces); the nesting tables' small table; the high chair's tray (the sim never seats a baby in it);
  the laundry basket's lid (not built); the display cabinet's door; the kettle, bread bin and laptop (content rules
  forbid uses on things on a top: sim work). These are modelled parts without a clip, so no clip is orphaned. The
  three freezers' `freezer_open` clips now play for "Get some ice cream".
- **Uses end once every need they raise is full**, by design; it makes clips easy to miss in checks.
- **Small things on tops:** every one of the 180 sample houses has them.
- **Residents didn't reach things in a crowded garden** once; a sim test of eleven in a row
  (`crates/sim-core/tests/crowded_garden.rs`) couldn't reproduce it.

### Models
- The entry console's mirror and hook board, and the spritz station's round mirror, hang in the air when the piece
  stands free; the party bar's "Back bar" colours the back cabinet, not the bottle shelves; bathers' shins go through
  the front of the heart tub. Deferred from 0.3.0 unless the model is touched for another reason.
- The power blender's blades loop was never built.

### Tools
- The lamp build (`_bulb`) checks only the light's height, not `at`.
- `pnpm wasm` doesn't run in a worktree with a symlinked `node_modules` (pnpm's deps check); its cargo +
  wasm-bindgen command works.

### Not looked at
- The production build's frame time; the town overview at full night; far versions in the town overview.
- The held things that no recipe in the test house makes (the tasting board, pizza, pie, slices, sandwich, grill
  platter); a meal from the fridge eaten at a pulled-in chair (eating was checked with the café's cake).

## Notes on the sim

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  the whole sim-core suite times the same with and without it (towns are ~60 Sims). It catches `.partners` changed
  through `get_mut` instead of `set_partners`.
