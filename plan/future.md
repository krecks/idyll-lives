# Future: things to consider

Notes that came up while doing the plan and don't belong to a task yet. Add to the end.

The notes from the object tasks (bathroom to packs, 2026-10-14) were worked through in one fix-up round the
same evening (merges into `v0.3/object-motion`, from `bb0a267` lamps to `f0528c5` outdoor and packs). What
was fixed is in those commits; what's below is what that round **decided** (for review) and what is **still
open**. Each line says where it lives.

## Next: sign-off

- **Validate holds 501 items in 73 groups**, against the plan's "about three at a time". Sign tasks off
  (`16-sign-off.md`) after review. `python3 -B tools/content/validate.py list | out --group <g> [--dry-run]`
  moves items back to their "Back to" by id, keeps the files' formatting and drops emptied groups. Items whose
  home turns them freely carry `"freeRotation": true` (Validate itself no longer turns anything).
- **Everything done is in Validate now** (2026-10-15): the base `fridge` (Validate › Fridges), the `wallBed`
  (Validate › Sofas) and the six garden decor models (new Validate › Garden Decor) had never been put in; their
  homes are listed in `16-sign-off.md`. The 39 items outside Validate are those of tasks not started (L4–L6,
  OU3–OU5). Validate changes an item's buy group, and the sim treats things of one group with a use in common as
  the same kind (`planner::same_kind`): with all six fridges in Validate › Fridges, other households upgrade
  their fridges, as they will again once the fridges are all back in kitchen / appliances.
- **The example pack's own "Wellness" category** is empty while its two objects are in Validate.

## Decided in the fix-up round (review these)

Each is reversible; the commit says how.

Engine and rules
- **Lamps** switch per lamp for the player (click: Turn on / Turn off), per room for residents, instantly (nobody
  walks to the switch): on when someone awake comes into a dark room (19:00–06:30, or a basement), off when the
  last one leaves or sleeps. Fires (`light.switch: false`) and garden lamps aren't residents' to switch; with free
  will off, lamps stay as the player left them. Room scores count every lamp. Save v28 (`lights.rs`).
- **Clocks tell the game's time** on every preset (state, not ambient), updated each game minute (every 5 on
  Low), only within the ambient range. No second hands: at speed 1 a game minute is a real second.
- **Ambient clips** stay still on Low and hold while paused (engine rule E2); clocks are the exception.
- **Things over windows:** only `overWindow` things (the curtains) hang over one; a window can't be built behind
  a picture. **Tall floor things** (`fillsWall`, 35 of them: fridges, wardrobes, showers, the fireplace…) keep
  the wall and ceiling above them clear; the hanging plant (`hangsLow`) blocks the floor under it.
- **A bought crop starts planted** (growth 0); crops already on a lot or in a house file start ripe.
- **The hot tub:** the last one out puts the cover back on (`closeAfter` on Soak).
- **An ordered use holds against needs** with free will off (the sim did; the game kept re-sending the free-will
  setting, now only on a change).
- **The gate** warns of a use clip over 3 s (loops, `idle`, `night`, `open`, growth are left alone).

Models and content
- **One counter depth, 0.66 m** (the 0.79 m catalog2 counters redrawn, `catalog2.py shallow`, `GameFrame(depths=)`).
- **Islands have no built-in stools**; they're turned round, cupboards at the front where residents stand, the
  top overhanging the stools housegen puts behind. Their modelled tops are empty.
- **The ceramic hob** shows a red ring while cooking, not a gas flame.
- **The dressing table and the piano** face the wall, their sitters facing the object's back (`SEATS` `back`,
  content `seat.facing: "back"`). The digital piano and sewing table still face the room (they read as tables).
- **Residents hold what they use** (content `held`): the bench's bar, a dumbbell pair, the guitars and ukulele,
  the steam mop; the object hides its own part meanwhile. The bench is 2 draws, the rack and instruments 3.
- **New uses with clips** (not autonomous, so generated houses behave as before): "Brush your teeth" at the
  mirror cabinet (2 draws), "Take out a blanket" at the cedar chest (3), "File papers" at the filing cabinet (3).
  The pedal bin's lid and the drawer counter's drawer play on uses they already had.
- **Holds:** toy box and chest lids stay open for the whole play; the kids' desk drawer keeps 2 s; the crib mobile
  turns only while someone plays with the baby; the hanging plant sways once.
- **The coop's hatch** is open by day and shut at night (no use to wire it to; 3 draws).
- **The grandfather clock** stays at 4 draws with its hands moving; its door and pendulum are still now. A satin
  body would give the pendulum back but lose the walnut grain.
- **The jukebox** (4 draws, record swings and label spins) and **the bunk** (4 draws, a duvet per bunk).
- **The treadmill belt** runs 0.4 m/s (was 0.2): faster blurs at 30 samples a second at 2.5× speed.
- **The parasol stirs 3°** (1.2° didn't show from the game camera).
- **Kept at 4 draws:** the washer and dryer (the drum shows turning), the phone box, the potting bench, and the
  eight pack models (`chefStove`, `whirlpoolTub`, `pizzaOven`, `recordCabinet`, `lavaLamp`, `soloSauna`,
  `glasshouse`, `veggieBed`). **Kept shut:** the base shower's sliding door.
- **Kept as they are:** pictures as flat shapes (a texture would double their draws); one sitter sinking every
  sofa cushion (per-slot clips exist now, but the cushions are one part); pillows not following; the desks'
  fixed chairs; the foosball's two rods; the curtains stretching to close; standing mirrors upright; the
  `@modern`/`@cozy`/`@minimal` style code in `catalog.py`; the `wallLampArm` placeholder.
- **Garden:** the lot dressing's `bush.small`, `bush.flowering` and `hedge` are Blender models; a picked
  vegetable patch is bare (all its plants under the soil); the unused `*.far` tree keys are gone; flowers are
  colourable only where their petals are one colour.

## Decided in the second round (2026-10-15, review these)

Merges `7a0ba3c` (veggie bed, bureau), `85650aa` (beds), `224b52f` (tables), `110aee3` (parts), and `b392650`,
`1571669` (housegen).

- **Beds keep their place; the head end is tucked:** headboards 3.5–4 cm thick (were 5.5–6), posts and boards
  2.5–5 cm forward, the car bed's body shorter at the back, so mattresses, pillows and sleepers stay put and no
  foot passes its tile. The crib, ball pit and play mat moved 3–3.5 cm forward; the ball pit, crib and rain
  shower are 1 cm smaller so their fronts end on the tile; showers 2–3 cm forward; the rocking horse rocks ±14°.
- **The six-seat, farmhouse and banquet tables are 0.88 m deep** (as the four-seat), the marble coffee table
  0.88 across: shallower, still centred, so housegen's chairs line up on both sides. The chaise and pottery
  wheel moved 2 cm forward (its stool 30 cm across); the foosball's rods stop at the wall face.
- **The small and long islands are turned round** as the kitchen island (cupboards at the front).
- **Outdoor things:** the gate skips the `outside` groups by design. Only those with a real back stand on the
  wall face now (garden chair, Adirondack chair, outdoor bar); swings, slide, phone box, stalls, fountain, farm
  and the playground stand free and keep their depth.
- **Putting clean things away goes to the nearest cupboard or counter** (new `cupboard` station,
  `putAwayAt: ["cupboard", "prep"]`, `cooking.rs`): the wall cabinets, sideboard, buffet and china cabinet open a
  door for it (the sideboard and buffet got a real cupboard inside). New non-autonomous uses: "Take a fresh
  towel" (tall bathroom cabinet), "Sort the washing" (hamper, lid at 70° so it stays out of the wall), "Sort the
  recycling" (lids one after another); the three-drawer nightstand's drawer opens on "Pick an outfit".
- **A use clip plays on top of the growth pose** (`partPose`, documented in `objects.md`): tending the veggie
  bed sways its plants 3° about the soil line, every 2.4 s.
- **The writing bureau is turned round** (back on the wall face, the writer in front facing it, `seat.facing:
  "back"`).
- **Housegen:** cooking stations count as a use (an island's stools go behind, its front stays clear for the
  cook); free tables try both facings (48 spots, was 24); a wall thing with a use hangs only where its front tile
  is free and reachable (the mirror cabinet's "Brush your teeth" had left 7 of 180 sample houses failing). Sample
  houses: 0 problems; islands with two stools 19 of 60 large (was 28, some with stools on the cook's side).

## Decided in the third round (2026-10-15, review these)

Merges `1c26a81` (worktops, potting bench), `702e57b` (play-check states), `25a31ad` (plants, quilts), `384cd54`
(lamp, hot tub, housegen), and `b07e885` (catalog pictures).

- **Every kitchen worktop is 0.95 m** (7 counters and appliances came down 1 cm with what stands on them; the
  butcher block's top is 5 mm thicker; the hob glass stands 4 mm proud, so a pan no longer sinks into it).
  Cookers and the foodie pack's chef's island (0.9) keep their own heights.
- **The potting bench** is 7,704 triangles: its pots leave out undersides and the inside under the soil.
- **Houseplants turn without touching the wall** (at most 44 cm from their middle): the monstera 0.78 m across
  (was 1.33), the palm and fern pulled in. Plant size warnings are gone: the placeholders of the oak, maple,
  grass, succulents, hosta and meadow changed, the geraniums' model (it was lopsided).
- **The vegetable patch's lettuces** are 17 cm heads, so a top shows when just planted.
- **A quilt per body:** each life stage's body gets a quilt shaped over its own lying pose (children's shoes,
  hands and forearms poked through the grown-up one, in every bed); a draw call only while someone with that
  body is home. The quilt comes up from the foot once the sleeper lies flat (it rose through swinging legs).
  `BEDS` has a `foot` (the car bed's quilt ends at its mattress).
- **A lamp switched off stays off** when moved, with its room or on a top, and through undo and redo.
- **An order to close something someone else got into first** (the hot tub's cover) is dropped with a story line
  ("… didn't close the cedar hot tub: someone else got in first", event `notClosed`), not queued.
- **Housegen** keeps small things off hobs, grills and sinks (61 on `cooktopCounter` in the sample before), and
  hangs curtains only over a window (206 of 345 were on plain walls; now all 321 are over one). The generated
  houses check fails on either.
- **Catalog pictures:** the smallest view is 0.6 m across (was 1.1), so a mug is a third of the picture; the
  contact shadow is the model's own size, no longer cut off square at the edges (picture cache `v2`).

## Still open

- **Beds that model their own duvet** (`bedDouble`, `bedCanopy`, `bedKing`, `bed`, `bedSingle`) show its
  turned-down band at the edges of a sleeper's quilt (older than round three).
- **Islands with two stools** in the generated sample: 16 of 60 large houses; the check fails below 15. Each
  housegen change reshuffles the random draws, so the next one may tip it.
- **A houseplant moved off its tile's middle** (quarter tiles, Shift) can still reach the wall.
- **The garden lantern** is 0.58 m deep against its placeholder's 0.47 (the gate warns).
- **The Adirondack chair's sitter** sits upright instead of leaning into its slanted back.
- **A game whose story is empty** doesn't show its first event in the feed (the first batch counts as history).
- **Parts still with no use:** the executive desk's drawer (residents use the desk through its chair, and a
  clip plays only on the object used: needs a seat's use to play a clip on the desk it faces); the nesting
  tables' small table (no use reads naturally, and one would need its front tile free beside a sofa); the high
  chair's tray (the sim never seats a baby in it); the laundry basket's lid (not built); the kettle, bread bin
  and laptop (content rules forbid uses on things on a top: sim work).
- **Kids' (and all) uses end once every need they raise is full**, by design; it makes clips easy to miss in
  checks.
- **Mirrors on High** log an occasional `GL_INVALID_VALUE: glGetProgramiv`; not reproduced in 5 runs, probably
  Babylon's parallel shader compile, not ours.
- **`partBox`** draws spheres and tori up to 5 % / 2.5 % bigger than Babylon's meshes (never smaller).
- **Residents didn't reach things in a crowded garden** once; a sim test of eleven in a row (`tests/`) couldn't
  reproduce it.

## Checked in play (2026-10-15, WebGPU and WebGL2)

The bike from both sides, a baby in the crib, two children in the bunk (a duvet each), children in the car bed,
the hammock, the garden and Adirondack chairs and the outdoor bar, the long island (cook at the front, stool
behind), the hamper, the pedal bin and drawer counter through a whole meal, all four facings of the kitchen
models, generated houses room by room, and all 501 Validate pictures. States: `DUMP_STATES=1 cargo test -p
sim-core --test play_checks` (`family-check`; `generated-<i>` with `--ignored`, `HOUSES=…`, `PICK=…`).

## Testing (how, after the fix-up round)

- `tools/bench/check.mjs gallery | clips | play` does the per-task checks headless at 1 % volume on its own port;
  `__sims()` says where residents are and what they use; `__addFunds`, `__camera`, `__controls.follow` with zoom.
- `DUMP_STATES=1 cargo test -p sim-core --test check_states` writes `bathroom-check`, `needy-check`,
  `office-night` and `town-things` (all 36 community and farm things); scripts must still send `setAutonomy` off.
- One worktree per agent, its own port and scratch folder (`tools/bench/README.md`): nine agents ran in parallel
  this way with no lost edits. In a shared checkout, build a commit from `HEAD` plus your own hunks, and write a
  shared file only after the new text is built and parsed.

## Simulation speed (checked 2026-10-14)

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  timing the whole sim-core suite with and without it showed no difference (about 18 s either way; towns are ~60
  Sims). It catches `.partners` changed through `get_mut` instead of `set_partners`. `Sim::refresh_buffs`' extra
  walks over `feelings` (a few entries) aren't worth a change either.
- **`retirement::the_basic_pension_pays_for_a_home_of_one`** passes (5 of 5 runs); its earlier failure was other
  work in the tree.
