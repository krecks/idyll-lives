# Objects: decisions to review, and what is still open

The calls taken while the objects were built, checked and fixed (2026-10-14 to 2026-10-17), for the user to review,
and what is known not to be right yet. Each decision is reversible; its commit says how. What
a single object does is on its category page ([README.md](README.md)); this page collects what cuts across them.

## Decisions to review

### Engine and rules
- **Lamps** switch per lamp for the player (click: Turn on / Turn off), per room for residents, instantly (nobody
  walks to the switch): on when someone awake comes into a dark room (19:00–06:30, or a basement), off when the
  last one leaves or sleeps. A lamp bought is off (Q-11). Fires (`light.fire`) are lit by someone who sits down by
  them; garden lamps (`light.switch: false`) aren't residents' to switch. With free will off, lamps stay as the
  player left them. Room scores count every lamp. Save v28 (`lights.rs`). A lamp switched off stays off when moved,
  with its room or on a top, and through undo and redo.
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
  lamp still lights and a dartboard or mirror can still be used. While one is in hand the walls stand (drawn up,
  without the roof, whatever the wall mode) and go back as they were once it is put down or back, so it and the
  things already hung hang on walls, never over a lowered one (M-houses-01); a taken spot names what hangs there.
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
  sewing table faces the room (it reads as a table); the digital piano, rebuilt as an upright, faces the wall (G-29).
- **Chairs at a separate table** are drawn pulled in by up to 0.45 m while sat on (`tableTuck.ts`; the sim's
  positions don't change), overlapping into the table's tile; a plain "sit" on a dining chair pulls it in too. The
  executive chair comes in only 6 cm (its desk's pedestals are where the knees go): the writer leans in further
  (`sitHandsFar`) rather than sliding sideways to the knee hole.
- **Lying:** heels sink about as deep as the buttocks (4–5 cm into the mattress) and the calves and thighs touch the
  top or stand 1–2 cm proud (a straight leg can't lie flatter without bending the knee backwards). Children lie on
  the mattress (`lieBack` scaled by size), so their heads sink 3.6 cm into the pillow, as grown-ups' do.
- **Bathers** sit on the tub's floor with their legs inside it (G-06); the sofa bed's user stands 1 m back; the lazy pack's doze and power nap are sitting
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
  turned-down duvet flattens and narrows under it. It takes its bed's bedding colour (per instance, one material),
  and with nobody under it no quilt is drawn at all.
- **Finishes reach the shader** (gloss, chrome and metal); metals reflect the sky just above the horizon where they
  would reflect the ground (`SKY_FOR_GROUND = 0.3`). **Hardware is metal** (handles, knobs, pulls); the dresser, low
  dresser, dressing table and writing bureau went to 4 draws for it.
- **Mirrors reflect on every preset:** the nearest on screen on Low (1024×512, 10 a second) and Medium (4× MSAA, 30 a
  second), redrawn only when something it shows changes; two live on High. Medium with a mirror in view and
  something moving costs about 6 % fps and 1 ms CPU on the M3.
- **Kept at 4 draws:** the washer and dryer, the phone box, the potting bench, the grandfather clock (hands and pendulum
  moving, the door still), the base and pack showers (their sliding doors, Q-05), the jukebox, the bunk, the greenhouse, the stereo, and the eight pack models `chefStove`,
  `whirlpoolTub`, `pizzaOven`, `recordCabinet`, `lavaLamp`, `soloSauna`, `glasshouse`, `veggieBed`.
- **Kept as they are:** pictures as flat shapes (a texture would double their draws); one sitter sinking every sofa
  cushion; pillows not following; the curtains stretching to close; standing mirrors upright; the retired `@modern`/`@cozy`/`@minimal` style variants.
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

The authenticity review (2026-10-21 to 22, [docs/authenticity-review.md](../authenticity-review.md)) fixed most of
what this list held, and 0.5.0 most of what it left, the new behaviour included; both are under "Fixed" below, with
their finding ids. What is still open is here, with its id; §9 of the review says what each would take.

### Residents
- **Props still mimed** (the rest of G-70): the tools (the watering can, the shears, the easel's brush, the chalk,
  the baby's bottle), the post box's card, the café's cake and the basketball (I-held-01); the phone box's receiver
  stays on its hook, its fourth draw call taken (I-social-01); no pigeons come down to the seed (I-community-11, new
  behaviour). A fork is 2 mm thick: from the game camera it is hardly seen.
- **Feet slide at game speeds 4 and 5** (48–81 % of the way: 45 and 90 m/s, faster than any gait steps without
  strobing; A-residents-03, G-25): won't fix. The clip check's baseline (`tools/characters/clip-baseline.json`) is
  empty.

### Content and sim
- **One-use machines:** the dishwasher and the neat pack's laundry centre are still loaded and run in one use; a
  machine that runs on its own (content `runs`) would fit them.
- **Vendors in older towns:** a café's or bakery's counter saved before 0.5.0 stands against its back wall, with no
  room behind it for a server, so it stays self-serve (new towns have the tile). Servers wear no apron and appear at
  their post at opening time rather than walking in.
- **The sounds want a listen by ear** (G-53): each was rendered and checked by its spectrum, timing and level, not
  heard (`bench-results/review/fixes/sounds/wav/`).
- **Later, as polish:** the range hood's light while the hob below is in use (its hum is in; Q-19); flowers cut for
  a vase (Q-18).
- **Parts with no use, left decorative:** the executive desk's drawer (I-office-02, won't fix); the nesting tables'
  small table and the display cabinet's door (decor, decision 4); the laundry basket's lid (not built). These are
  modelled parts without a clip, so no clip is orphaned. The grandfather clock's trunk door is not a part of its own
  (A-decor-01, won't fix): it would be a fifth draw, over what the gate passes, and the clock is wound at its dial.
- **Uses end once every need they raise is full**, by design (I-systems-02); it makes clips easy to miss in checks.

### Tools
- The review's tooling leftovers (`cargo review sizes`' measuring, `check.mjs play` with one resident, the debug
  states) are in [docs/authenticity-review.md](../authenticity-review.md) §7.

### Not looked at
- Low on WebGL2 against Babylon's figures since the woods went into squares: the machine was too busy for the 0.4.1
  method (`performance.md`, 0.5.0), and the squares' +0.2 ms of main thread wants a look.
- The held things that no recipe in the test house makes (the tasting board, pizza, pie, slices, sandwich, grill
  platter); a meal from the fridge eaten at a pulled-in chair (eating was checked with the café's cake).

### Fixed in 0.5.0
- **Residents:** body language per emotion, the posture, the arms and the walk's swing over every clip as far as
  what they do allows, and the pace from content `emotions` `walkSpeed` (G-71, A-residents-07); stepping round to turn
  on the spot (G-71, A-residents-08); children's feet at game speed 3 (11–14 % slide, now none: the run's beat
  quickens on shorter legs, `gait.ts`; A-residents-03 in part); getting out of bed before the lie has faded in (PR
  #31's held pose, 7 cm in a frame, now none).
- **Props in hand** (G-70): a book, a mug, a tumbler, a wine glass or a can, a fork, a spoon or chopsticks, a phone
  at the ear and in both hands, a coin tossed into the town fountain, a bag of bird seed on the park bench, a dart,
  each player's ping-pong bat and pool cue, and the bar carts' shaker shaken in both hands (I-held-01 in part,
  I-social-01 in part, I-hobbies-05, I-kitchen-13, I-community-11 in part); the pool table's balls break at the
  first shot (I-hobbies-05).
- **The food from town in its own wrapping:** a paper cup of ice cream, a hot dog in its bun, a packet of crisps, an
  apple and a pastry in a paper bag (M-held-03).
- **Fruit within reach** (I-garden-11, I-garden-12, the owner's decision): the apple tree's crown came down to 2.6 m
  and four low boughs carry eight apples from 1.12 to 1.75 m; the one picked to eat hangs at 1.13 m, where a child
  on tiptoe takes it, and "Pick the apples" reaches one per size. The lemon tree has a lemon at 1.62, 1.42 and 1.1 m
  for a grown-up, a teen and a child. Placed apple trees aren't turned at random any more (`trees.md`).
- **The home chess table seats two** (M-hobbies-11, Q-16): a chair at each end of 2 × 1 tiles, `places` facing each
  other, and *Play chess* with a partner, *Practise chess* alone or with nobody free (`hobbies.md`). An older save's
  table (save version 33) goes over its old tile where it fits, else at a free spot at home, else it is sold back.
- **Worked at the part** (I-packs-12): the bonsai bench from 0.6 m out, at the tree; the lectern's book read with the
  hands on it from 0.52 m out (`readLectern*`); the streetball hoop shot from 2.2 m out with a jump shot fitted to
  its rim (`streetShot*`); the arbour's roses cut at a front post (`cutRose*`, `STEP_IN`: the swing's seats can't take
  `places` for it).
- **Privacy** (G-72: I-systems-05, I-social-05): a bathroom someone is in is kept shut (`privacy.rs`): others wait
  outside its door (an hour at most) and go in after, its door stays shut to them in the drawing, and kept out with
  the bladder empty the accident comes. Visitors leave the household's own things alone (`private`: wardrobes,
  dressers, the bath cabinet). *Try for a baby* is a short ask, then the two in a double bed under the covers for
  half an hour behind the bedroom's shut door (Q-21).
- **Someone serves in town** (I-community-09, Q-17): staff stand behind the carts, the market stalls, the shop
  counters, the bakery cases and the café's coffee machine on public lots during their hours, and sell what they used
  to leave to an honesty box: ordered, handed over, paid for (`staff.rs`, the `buy` and `sell` clips). Closed outside
  their hours; self-serve, under the old labels, on a household's own lot, in the use's own time rather than the
  sold five minutes (a coffee poured at home takes 2 minutes; `selfServe.minutes`). The bakery case's back door is
  the baker's and opens as they reach in; the shop counter's till is turned to the shopkeeper.
- **Laundry:** the washer and the dryer are loaded in 5 minutes and run on their own (60 and 45 minutes, the
  drum turning and their sound with nobody there), then someone takes the washing out and carries it on in the
  basket to the dryer or the line; the dry washing is folded at the laundry basket, a towel in the hands
  (I-bathroom-07). The washing line dries the washing in 3 hours and takes it down a line at a time, as it is
  hung out (A-outdoor-02). A generated home with a washer and no dryer gets a washing line in its garden (40 of 180
  sample houses; none is left with nowhere to dry), and a load cut short goes back on its pile.
- **Models:** the arc lamp reaching over the next tile, placed so by build mode and the house generator
  (M-lighting-06); the hot tub's cover folding back in two halves (A-outdoor-01, a fourth draw).
- **Moving parts with a use:** the kettle's lid, the bread bin's roll-top and the laptop's lid open for their uses
  (I-tabletop-01); the high chair's tray slides out as the baby is put in (I-kids-01).
- **Sounds** (I-bathroom-04, G-53): the uses that were silent sound like their things, synthesised
  (`web/src/audio/things.ts`): the treadmill and bikes, the workbench and the bike stand, pool, foosball, ping-pong,
  darts, the arcade cabinets, the sewing machine, the chemistry set, the pottery wheel, the coffee machines, the
  blenders and the stand mixer, the microwaves (a hum, then a ping), toast (then the pop), an oven's fan, the pizza
  oven's fire (and the fireplace's and the fire pit's), the phone box, the vending machines, basketball, karaoke
  singing, a bowed cello, the punch bag and the smash bin; a fridge door and the pedal bin's lid, once as they're
  used. The cooker hood hums while the hob under it is in use (I-kitchen-09, Q-19).
- **Rendering:** each lamp lights only its own room on Low too, and on each half of a tile split by a diagonal wall,
  on every preset (M-lighting-08); the town map leaves the world its clicks and camera drags, as the pie menu does;
  with a wall thing in hand the walls stand, so nothing hangs over a lowered wall (M-houses-01); `partBox` is the box
  of the part's own mesh. Checked on PlayCanvas and found right: the stairs seen from the basement are whole to the
  top (M-living-02, fixed on Babylon by 5b1e7152, carried over by the port). The mirrors' `glGetProgramiv` log (a
  Babylon one) no longer shows: 6 runs on High on WebGL2, each in a new browser (the bench's mirror camera for 20 s,
  then the game in front of a mirror), without a single WebGL message. What the check found instead is fixed: the mirrors' warm-up drew each glass
  into its own reflection (WebGPU dropped that frame's commands, once a load), and a quilt drawn before its first
  update had no instance buffer (WebGL2 errors until Chrome stopped reporting any, a dropped frame on WebGPU).
- **Performance:** Low draws the woods in 100 m squares rather than eight sectors to the horizon, so far trees off
  screen are left out: on WebGL2, the wide view, 1.79 → 1.21 M triangles and −0.3 ms of GPU time, for +0.2 ms of
  main thread; the same picture to the pixel. Nature has switches of its own (`diag=nowoods,nofarwoods,noflowers,
  nograss`). Looked at, as they hadn't been: the production build's frame time (120 fps in every cell, p99 9.3 ms;
  the main thread 0.2–1.0 ms over Stage 11's on a shared machine), the town overview at full night and its far
  versions close up (right as they are). All in [performance.md](../design/performance.md) (0.5.0). 22 of the 30
  `as unknown as` in `web/src` are gone; the 8 left are where PlayCanvas's types say `private` or are wrong.
- **Tools:** the lamp build checks the light's `at`, not only its height (M-lighting-06).

### Fixed by the authenticity review
- **Residents:** the weight bench's feet-down bench press (A-fitness-01); a kiss between different heights
  (A-social-01); the tuck at the standing desk, the kids' chair at a coffee table and a chair facing a desk with its
  own (A-office-01, A-kids-01, I-office-01); getting up from a chair at the seat (A-residents-01, A-residents-14).
- **Rendering:** night glow through other parts, the grill's coals (M-outdoor-01); cooking props at a station's
  middle (I-systems-01).
- **Content and sim:** small things on tops are used where they stand (I-tabletop-01, I-tabletop-04); the high
  chair is used (I-kids-01).
- **Models:** the entry console's and the spritz station's mirrors on back boards (M-packs-01, M-packs-02); the party
  bar's back bar (M-packs-03); bathers' shins in the heart tub (A-packs-01); the power blender's blades (A-packs-02).

## Notes on the sim

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  the whole sim-core suite times the same with and without it (towns are ~60 Sims). It catches `.partners` changed
  through `get_mut` instead of `set_partners`.
