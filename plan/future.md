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
