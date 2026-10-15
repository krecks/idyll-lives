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

## Still open

Placement
- **Freestanding things still reach 2–6 cm into the wall** (hidden unless the wall is cut away), model and
  placeholder both: the beds (moving them pushes the foot past the tile or the headboard onto the sleeper),
  crib, showers, play mat; the dining tables, small and long islands, foosball, pottery wheel, marble coffee
  table, chaise; 13 outdoor and community things (bistro set, adirondack and garden chairs, outdoor bar 6 cm,
  phone box, porch swing, swing set, slide, ping-pong table, playground, stalls, town fountain, farm);
  `soloSauna` and `danceFloor` 1 cm. The gate lists them all.
- **`fridgeRetro`** reaches 3.5 cm into the wall (re-exported for its lever from the old place): `catalog2.py`
  `shift(p, dz=0.03)` for the placeholder and `Model` centre about (0, -0.07).
- **The steam mop dock's placeholder** reaches 2 cm into the wall.
- **Houseplants into the wall:** `garden.monstera` 24 cm, `palm` 6, `bostonFern` 4. The plants' 10–15 % size
  differences from their placeholders are real now (the gate measures turned parts exactly), and `tree`,
  `tree.maple`, `grass`, `geraniums`, `succulents` still warn.
- **The kitchen worktops** differ by 1 cm in height (0.96 vs 0.95).
- **Housegen puts small things on the hob** (a bread bin on `cooktopCounter`).
- **`GameFrame(shift=…)`'s comment** says it moves depths back; a positive shift moves them forward (+z).

Uses and clips
- **Parts with no use yet** (built separate, no clip): the tall bathroom cabinet's door, `nightstandDrawers`'
  drawer, the hamper lid, the laundry basket's lid (not built), the high chair's tray (the sim only places a baby
  lying), the wall cabinets', sideboard's, buffet's and china cabinet's doors, the nesting tables' small table,
  the executive desk's drawer, the recycling bins' lids. The kettle, bread bin and laptop sit on tops, and
  nothing on a top has a use yet.
- **The veggie bed's `tendVeggies` plays no clip:** every part follows the growth stage, and a use clip would
  override it. Needs use clips that combine with the growth pose (`objectMotion.ts`).
- **The writing bureau (bookworm)** seats the writer between the wall and the bureau: give it
  `seat.facing: "back"` as the piano.
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
