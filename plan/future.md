# Future: things to consider

Notes that came up while doing the plan and don't belong to a task yet. Add to the end.

The notes from the object tasks (bathroom to packs, 2026-10-14) were worked through in one fix-up round the
same evening (merges into `v0.3/object-motion`, from `bb0a267` lamps to `f0528c5` outdoor and packs). What
was fixed is in those commits; what's below is what that round **decided** (for review) and what is **still
open**. Each line says where it lives.

## Next: sign-off

- **Validate holds 493 items in 72 groups**, against the plan's "about three at a time". Sign tasks off
  (`16-sign-off.md`) after review. `python3 -B tools/content/validate.py list | out --group <g> [--dry-run]`
  moves items back to their "Back to" by id, keeps the files' formatting and drops emptied groups. Items whose
  home turns them freely carry `"freeRotation": true` (Validate itself no longer turns anything).
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

## Still open

Placement
- **The steam mop dock's placeholder** reaches 2 cm into the wall.
- **Houseplants into the wall:** `garden.monstera` 24 cm, `palm` 6, `bostonFern` 4. The plants' 10–15 % size
  differences from their placeholders are real now (the gate measures turned parts exactly), and `tree`,
  `tree.maple`, `grass`, `geraniums`, `succulents` still warn.
- **The kitchen worktops** differ by 1 cm in height (0.96 vs 0.95).
- **Housegen puts small things on the hob** (a bread bin on `cooktopCounter`).
- **`GameFrame(shift=…)`'s comment** says it moves depths back; a positive shift moves them forward (+z).

Uses and clips
- **Parts still with no use:** the executive desk's drawer (residents use the desk through its chair, and a
  clip plays only on the object used: needs a seat's use to play a clip on the desk it faces); the nesting
  tables' small table (no use reads naturally, and one would need its front tile free beside a sofa); the high
  chair's tray (the sim never seats a baby in it); the laundry basket's lid (not built); the kettle, bread bin
  and laptop (content rules forbid uses on things on a top: sim work).
- **Lettuces don't show at the planted stage**; they appear from growing on.
- **Moving a lamp turns it back on** (the move and undo code don't carry `off`; one line next to each `open` copy).
- **An ordered "Put the cover on"** is dropped silently when another resident is in or heading to the tub.
- **Kids' (and all) uses end once every need they raise is full**, by design; it makes clips easy to miss in
  checks.
- **The potting bench** is 8,216 triangles (aim 8,000).

Engine
- **Mirrors on High** log an occasional `GL_INVALID_VALUE: glGetProgramiv`; not reproduced in 5 runs, probably
  Babylon's parallel shader compile, not ours.
- **`partBox`** draws spheres and tori up to 5 % / 2.5 % bigger than Babylon's meshes (never smaller).
- **Residents didn't reach things in a crowded garden** once; a sim test of eleven in a row (`tests/`) couldn't
  reproduce it.

## Not checked yet

- In play: the bike from its left side, children in the car bed, the garden hammock; the pedal bin and drawer
  counter during a real meal (clips bench only); the fourth facing of the kitchen models; the bathroom, bedroom,
  kitchen and outdoor things on a town lot beyond the samples; the catalog pictures of the other groups.
- After the second round: a baby in the crib and a second sleeper in the upper bunk, the garden chairs and
  outdoor bar, the long island, the hamper in a house (it played in the agent's test), all on WebGL2. A sleeper
  in `bedDouble` shows their shoes through the quilt halfway down (seen once, likely older than this round).

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
