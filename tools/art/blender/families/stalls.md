# Food stalls and vending — notes

`stalls.py` builds the five CM3 models into `web/public/assets/models/stalls/` and their entries in
`packs/stalls/manifest.json`: the ice cream cart (`model.iceCreamCart`), the hot dog stand (`model.hotDogStand`),
the snack machine (`model.vendingSnack`), the drinks machine (`model.vendingDrink`) and the market stall
(`model.marketStall`). They replace the primitive-part placeholders of `tools/art/community.py` ("Carts and
machines"), which stay as the fallback. Built 2026-10-14; they sit in Validate › Stalls until checked in play
(back to community / food).

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`):
the carts and the stall across their 2 × 1 tiles, the machines with their backs on the wall face (depth −0.43) of
their 1 × 1 tile. They are placed on town lots and seen from far, so each has a far version (`far=0.25`: the model
at rest, joined, a quarter of the triangles).

## The carts

- **Ice cream cart.** A cream gloss body with two pink bands, four wheels, a push bar at its right (game +x) and a
  steel deck round a shallow freezer well with six tubs of ice cream sunk into the top. Two lids on rails: the right
  one fixed, the left one (`lid`) slides right over it, `lid_open`, 0.58 m in 1 s; `hold` 2 s, so it shuts while
  the resident eats. A cone with a scoop stands on the back of the deck as a sign.
  - **The parasol** stands in a socket at the back of the top, behind the lid's rails, and leans 11° forward so
    its canopy sits over the middle of the cart and inside the footprint's depth. A centred upright pole would
    pass through the sliding lid. The canopy is 16 gores, pink and white (`parasol`, `parasol_white`, one cone
    with alternating face materials), 1.5 × as wide as deep, as the placeholder's is (0.94 m deep).
- **Hot dog stand.** A red gloss cart with a steel top, a sign with a hot dog on its front, sauces and napkins, four
  posts and a striped awning (six slats, red and white, with a valance) leaning down to the front. The hot plate
  (a steel tray, a dark plate, five sausages and two buns) has a hinged steel lid (`lid`) on its back edge:
  `lid_open` lifts it 75° in 0.9 s while a resident buys a hot dog (no `hold`: up for the whole use).

## The machines

One shell for both (`_cabinet`): a back block, side walls, a header with a lit sign, a keypad column on the right
(display, twelve keys, a card reader, a coin return), and a kick panel round an open pick-up tray. The inside is
lined dark; the tray's well is open at the front, so what drops into it shows.

- **Snack machine.** Five steel shelves of snacks, two deep, with label strips, behind a glass window (`glass`,
  see-through: one draw call). `drop`: the front snack of the second shelf's second column is pushed 5 cm forward
  (0.15 s) and slides down in front of the shelves (0.9 s) to the tray, tipping back 15° as it lands. Its shelf
  keeps the gap. No `hold`: it sits in the tray while the resident eats.
- **Drinks machine.** A lit front panel (`lit_panel`) with two bands, three big cans standing proud of it, a button
  and a price under each. `drop`: a can lying across, hidden inside above the tray, drops 0.18 m into it (0.7 s)
  and rolls 4 cm to the front.
- **Night glow (E8).** The header signs (`lit_sign`, glow `#FFE2A6`) and the drinks machine's front (`lit_panel`,
  glow `#CFE8FF`) glow from dusk, as the windows do; nothing glows by day.

## The market stall

Four walnut posts (the back two taller), beams, a honey table top with an apron and a lower shelf, a striped awning
(seven slats, green and white, with a valance) leaning down to the front, a chalk board hung from the front beam,
four open crates tilted 10° to the front heaped with lemons, apples, oranges and tomatoes, and two spare crates on
the lower shelf. Static.

| Model | Size (w × h × d) | Triangles (far) | Draws | Clip | Colour parts |
|---|---|---|---|---|---|
| `model.iceCreamCart` | 1.73 × 2.36 × 0.97 m | 3,778 (944) | 3 | `lid_open` (iceCream, hold 2 s) | Body, Parasol, Bands |
| `model.hotDogStand` | 1.58 × 2.39 × 0.98 m | 3,532 (878) | 3 | `lid_open` (hotDog) | Body, Awning |
| `model.vendingSnack` | 0.90 × 1.88 × 0.75 m | 3,132 (704) | 3 | `drop` (snack) | Body |
| `model.vendingDrink` | 0.90 × 1.88 × 0.79 m | 3,312 (814) | 2 | `drop` (drink) | Body |
| `model.marketStall` | 1.84 × 2.15 × 0.98 m | 5,248 (1,252) | 3 | — | Body, Awning |

Draw calls: the carts take the plain body, the fabric awning or parasol and the lid; the snack machine the plain
body, its glass and the snack; the drinks machine the body and the can (plus one at night for the glow); the stall
wood, fabric and plain.

`check_glb.py`: 5 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings for all ten files (the
models and their far versions).

## Rebuild

```
python3 -B tools/art/blender/build.py stalls            # the five .glb and .far.glb, the pack, check_glb
python3 -B tools/art/blender/build.py stalls --preview  # previews/<id>.png, <id>@<clip>.png, <id>.far.png
```
