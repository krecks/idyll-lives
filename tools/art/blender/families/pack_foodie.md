# Foodie pack — notes

`pack_foodie.py` builds the six P6 foodie models into `web/public/assets/models/pack_foodie/` and their entries in
`packs/pack_foodie/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sit in
Validate › Foodie until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face (depth −0.43). The island's top is at 0.9, the content's `surface.height` (the
old part's was 0.83), so what is put on it sits on it; its stools keep the default seat (0.45, the sitters at
x ±0.5 in the middle, facing the top), with their legroom open right through under the drawer bank.

| Model | Moving part and clip | Played by |
|---|---|---|
| `herbGarden` | a basil sprig in the pot on the lower shelf, built full size and scaled to nothing on the soil: `sprig` grows it (0.9 s) | `tendHerbs` |
| `deliPantry` | the two lower doors, hinged on their outer edges: `doors_open` swings them out 100° (0.8 s), `hold` 2.5 s | `nibble`, `cheeseBoard` |
| `espressoBar` | the coffee from the left group's spout (grown down, 0.4 s) and the steam wand on its ball joint (swung out 40°, 0.6 s): `brew`, `hold` 4 s | `espresso`, `latteArt` |
| `cookbookLectern` | the right-hand page, origin on the spine and tilted onto the desk (`rotation_mode` YXZ): `page` turns it 172° about the spine (0.9 s), until the read ends | `studyRecipes`, `foodMagazines` |
| `pizzaOven` | the iron grille door and the peel with a pizza: `door_off` (1.4 s) lifts the door off the mouth and sets it on the ground before the left pillar, then carries the peel from the counter round in front of the oven and in through the mouth | `bakePizza` |
| `pizzaOven` | flat flames in the mouth behind the grille, built full height and flattened onto the sill: `flames` grows them (0.8 s) | `warmUp`, `bakePizza` |
| `chefsIsland` | the middle drawer on the cook's side: `drawer_open` pulls it out 12 cm (0.6 s), `hold` 2.5 s | `tastingMenu` |

The espresso's clip keys two parts; the pizza oven's `door_off` keys two parts with their own timing (`_clip`
with keyframes: the peel waits for the door, then goes up, forward, along and in, so it never passes through the
dome or the door).

**Draw calls: 3 each, the pizza oven 4.** So that they stay at 3:
- herbs, pots and soil are plain (satin, matte), the herb garden's cedar its one `wood`;
- the pantry and the espresso cart are all plain (their oak-coloured shelves and tops satin), as their two moving
  parts take the other draws;
- the lectern and the island keep `wood` (walnut; the stools' oak) beside their plain finishes and one moving part.

The pizza oven has three moving parts (door, peel, fire: the plan's door and peel for `bakePizza`, flames for
`warmUp`), so its body is all plain (the logs satin, not bark): 4 draws, `check_glb.py`'s one warning. The fire
shows through the grille door (a solid door would hide it for `warmUp`); `bakePizza` also lights it, so the door
comes off a burning oven (the plan names `flames` for `warmUp` only).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.foodie.herbGarden` | 0.86 × 1.29 × 0.76 m | 6,708 | 3 | Body, Pots |
| `model.foodie.deliPantry` | 0.88 × 1.99 × 0.57 m | 4,392 | 3 | Body, Worktop, Shelves |
| `model.foodie.espressoBar` | 0.76 × 1.51 × 0.51 m | 5,364 | 3 | Body, Worktop, Machine |
| `model.foodie.cookbookLectern` | 0.84 × 1.15 × 0.59 m | 2,276 | 3 | Body, Stool & ledge, Cookbook |
| `model.foodie.pizzaOven` | 1.89 × 1.90 × 0.92 m | 5,056 | 4 | Body, Worktop, Brick |
| `model.foodie.chefsIsland` | 1.86 × 1.13 × 0.77 m | 6,288 | 3 | Body, Worktop, Stools |

The herb garden has no worktop; its second part is the pots. The rosemary stands 7 cm over the old part's.

`check_glb.py`: 6 models, 0 failed; the pizza oven's 4-draw warning above. Khronos validator: 0 errors, 0 warnings
each. Gallery next to the old parts; clips bench at rest and open on WebGPU and WebGL, coloured (pantry, oven).
In play (`wants-asking`, WebGPU): the pizza oven baked in the yard (door down, peel in, fire, all back after),
the island sat at for a bite, the pantry opened for a snack, the herbs tended, the cookbook read (the page over),
the espresso bar used (the resident hides the stream from the camera; seen on the bench). At night
(`office-night`, WebGL): the oven in four facings, the island, the herb garden, the pantry and the lectern turned.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_foodie            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_foodie --preview  # previews/<id>.png, <id>@<clip>.png
```
