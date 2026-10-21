# Decor

Mostly still things. All 22 are built by `tools/art/blender/families/decor.py` (notes: `families/decor.md`), each
in its place: the wall things (pictures, clocks, shelf, curtains) at their real height with their
back on the wall face, the hanging plant from the 2.8 m ceiling, the grandfather clock and the fireplace with their
backs on the wall face, the rest centred on their tile or flat on the floor (the rugs). The pictures are flat colour
shapes (matte, one draw call with the frame), not textures.

What moves: the clocks' hands tell the game's time (`motions.clock`, `at` 610: ten past ten at rest, the catalogue
picture), on every preset, set each game minute (every 5 on Low) within 30 m of the camera or the ambient range
(`objectMotion.ts` `CLOCK_RANGE`); there is no second hand, as a game minute is a real second at normal speed. The
hanging plant sways once when watered, the fire burns while a resident warms up by it, and the curtains are drawn
at night (E3).

Rules for the category: only `overWindow` things (the curtains) hang over a window, and no window can be built behind
a picture. The grandfather clock and the fireplace are `fillsWall` (they keep the wall and ceiling above them
clear); the hanging plant is `hangsLow` (it blocks the floor under it). A wall thing hides with its wall whenever the
cutaway drops it, from either side.

## D1. Wall art and clocks (6)

- **Family:** `tools/art/blender/families/decor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `printFramed` Framed print | 1x1 · wall | — | decor / wall | Frame | Static | Static. |
| `photoPrint` Photo print | 1x1 · wall | — | decor / wall | Frame | Static | Static. |
| `paintingClassic` Oil painting | 1x1 · wall | admire:look | decor / wall | Frame | Static | Static. |
| `canvasAbstract` Abstract canvas | 2x1 · wall | admire:look | decor / wall | Frame | Static | Static. |
| `wallClock` Wall clock | 1x1 · wall | — | decor / wall | Case, Face | Clock (`motions.clock`) | `hour_hand`, `minute_hand`: the hands tell the game's time; no second hand (a game minute is a real second). |
| `wallShelf` Floating shelves | 1x1 · wall | — | decor / wall | Shelves | Static | Static. |

**In play** (signed off 2026-10-16):
- Each hung on all four walls of the living room (the 2-wide abstract canvas on the bathroom's and bedroom's plain
  walls, where two plain tiles side by side are free of windows), walls up, by day and at night, on WebGPU and
  WebGL2, Medium and Low, every part coloured.
- A resident admires the oil painting and the abstract canvas facing it (the "look" pose).
- The wall clock's hands keep the game's time (ten past ten in the morning, ten to ten at night). Its case's well
  ends behind the face, so the face shows white by default and "Face" colours it.
- With the cutaway dropping a wall (an inner wall, or an outer wall seen from outside), the things on it hide with
  it rather than float over the stub; each knows from when it is placed which views drop its wall
  (`BabylonRenderer.ts`). Checked on inner and outer walls from all four sides; floor things against a wall stay.
- Since the authenticity review (G-15): the grandfather clock is wound stepped in at its trunk door (the weights'
  chains), the hands on it, with no stoop to the floor; a reach up to the dial is a clip still to make.

## D2. Ornaments (10)

- **Family:** `tools/art/blender/families/decor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `grandfatherClock` Grandfather clock | 1x1 · floor | wind:tinker | decor / ornaments | Case, Face | Clock (`motions.clock`) | `hour_hand`, `minute_hand`: the hands tell the game's time. The case door and the pendulum are still, to stay within 4 draws (`families/decor.md`). |
| `vaseTall` Tall vase | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `urnPorcelain` Porcelain urn | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `vasePedestal` Vase on a pedestal | 1x1 · floor | — | decor / ornaments | Vase, Pedestal | Static | Static. |
| `bustMarble` Marble bust | 1x1 · floor | admire:look | decor / ornaments | Bust, Plinth | Static | Static. |
| `statueCat` Cat statue | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `statueHorse` Bronze horse | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `coatRack` Coat stand | 1x1 · floor | — | decor / ornaments | Body | Static | Static. Decor by decision (Q-19: kind `coatStand` marked decor); it carries its coat and hat. |
| `hangingPlant` Hanging plant | 1x1 · ceiling | water:water | decor / ornaments | Pot, Cord | Use | `sway`: the pot sways 5° and settles for water (`hold` 0.8 s, one sway). |
| `fireplace` Fireplace | 2x1 · floor | relax:relax | decor / ornaments | Surround, Mantel | Use | `flames`: the flames grow on the logs for relax (gloss; the light is the content's). |

**In play** (signed off 2026-10-16):
- The eight standing pieces stand free in all four facings; the bust is admired from each side.
- The grandfather clock is wound (the "tinker" pose at its case) against all four living-room walls, its hands
  keeping the game's time.
- The hanging plant, watered, sways out and settles after its `hold`.
- The fireplace has a raised hearth bench (content `seat`: height 0.42, forward 0.02): a resident warming up sits on
  it with the flames growing behind her and dying down when she gets up. Its `light` (height 0.7, `switch: false`:
  residents don't switch it) lights the room at night.
- The horse statue is a bronze horse standing on an oak plinth (model and its name and description agree); the pedestal vase is blue.

## D3. Textiles (6)

- **Family:** `tools/art/blender/families/decor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `curtains` Curtains | 1x1 · wall | — | decor / textiles | Fabric, Rail | Night (E3) | `close`: drawn closed at night (E3), open by day. |
| `curtainsLong` Floor-length curtains | 2x1 · wall | — | decor / textiles | Fabric, Rail | Night (E3) | `close` (E3). |
| `rugRound` Round rug | 2x2 · rug | — | decor / textiles | Rug | Static | Static, flat (rug layer). |
| `rugLarge` Large area rug | 3x2 · rug | — | decor / textiles | Rug | Static | Static, flat. |
| `rugRunner` Hall runner | 1x3 · rug | — | decor / textiles | Rug | Static | Static, flat. |
| `rugPattern` Patterned rug | 3x2 · rug | — | decor / textiles | Rug | Static | Static, flat. |

**In play** (signed off 2026-10-16):
- The curtains hang over windows on all four sides, open by day and drawn closed at night, on WebGPU and WebGL2 at
  Low; they stay drawn over sleepers. The two pleated panels widen from their outer ends until they meet (the
  fabric stretches to close, kept so).
- The rugs lie flat in all four facings. Each rug's "Rug" colours its field; borders and motifs keep their colours.
- A 2-wide curtain over the house's 1-wide windows hangs with the window in one half of its rail (the placement,
  not the model).
