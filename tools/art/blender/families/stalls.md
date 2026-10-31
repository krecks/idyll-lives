# Food stalls and vending — notes

`stalls.py` builds the five CM3 models into `web/public/assets/models/stalls/` and their entries in
`packs/stalls/manifest.json`: the ice cream cart (`model.iceCreamCart`), the hot dog stand (`model.hotDogStand`),
the snack machine (`model.vendingSnack`), the drinks machine (`model.vendingDrink`) and the market stall
(`model.marketStall`). Built 2026-10-14; they sat in Validate › Stalls until checked in play
(back to community / food).

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`):
the carts and the stall across their 2 × 1 tiles, the machines with their backs on the wall face (depth −0.43) of
their 1 × 1 tile. They are placed on town lots and seen from far, so each has a far version (`_far(build)`: the
model at rest, joined, the vertices that sit on one another merged, decimated to a quarter with its feet and outermost
faces kept; decimated unmerged, the drinks machine's cans' turned ends tore its cabinet into spikes 26 cm below the
floor).

## The carts

- **Ice cream cart.** A rounded cream gloss body with two rounded pink bands and a cone badge on its front, on a steel
  chassis with four spoked wheels (rounded tyres, rims, domed hubs) on axles, a push bar bent in one piece at its right
  (game +x) with a rubber grip, and a steel deck round a shallow freezer well with six turned tubs of scooped ice cream
  sunk into the top, a scoop resting on the back; the lids have pressed ribs, the sliding one a bent grip along it. Two lids on rails: the right
  one fixed, the left one (`lid`) slides right over it, `lid_open`, 0.58 m in 1 s, a second into the sale (`delay`
  1, `hold` 1.4: the game's vendor reaches in then, `characters/clips/serve.ts`). A cone with a scoop stands on the back of the deck as a sign.
  - **The parasol** stands in a socket at the back of the top, behind the lid's rails, and leans 11° forward so
    its canopy sits over the middle of the cart and inside the footprint's depth. A centred upright pole would
    pass through the sliding lid. The canopy is 16 gores, pink and white (`parasol`, `parasol_white`, a loft
    sagging between the ribs, with alternating face materials), a scalloped valance hanging from its rim, ribs
    under it, a runner on the pole and a turned finial; 1.5 × as wide as deep, as the old part was.
- **Hot dog stand.** A rounded red gloss cart with a rounded steel top and a trim rail, the same spoked wheels, a framed
  sign on its front (a bun, a sausage with rounded ends and a mustard zigzag), squeeze bottles, a napkin box with a
  napkin pulled up, a stack of paper trays, four posts on feet and a striped awning (six slats, red and white, a
  scalloped valance) leaning down to the front. The hot plate (a steel tray, a ribbed dark plate, five bowed sausages
  and two split buns) has a hinged, rounded steel lid (`lid`) with a bar handle on its back edge:
  `lid_open` lifts it 75° in 0.9 s a second into the sale, for 1.4 s, as the vendor behind it takes a sausage.

## The machines

One shell for both (`_cabinet`): a back block on levelling feet, side walls, a header with a lit sign (six letters on
it), a keypad column on the right (display, twelve keys, a card reader with its slot, a coin slot, a coin return with
its cup), a rounded bezel round the window, and a kick panel round an open pick-up tray with its PUSH label. The inside is
lined dark; the tray's well is open at the front, so what drops into it shows.

- **Snack machine.** Five steel shelves of snacks, two deep, each in its wire spiral, with label strips and numbers;
  every other one a crisp packet (puffed, a crimped seal), the rest boxes with labels; behind a glass window (`glass`,
  see-through: one draw call). `drop`: the front snack of the second shelf's second column is pushed 5 cm forward
  (0.15 s) and slides down in front of the shelves (0.9 s) to the tray, tipping back 15° as it lands. Its shelf
  keeps the gap. No `hold`: it sits in the tray while the resident eats.
- **Drinks machine.** A lit front panel (`lit_panel`) with two bands and bubbles, three big turned cans (necked ends,
  a lid rim, a label band) standing proud of it, a lit button and a price under each; the can that drops is turned too. `drop`: a can lying across, hidden inside above the tray, drops 0.18 m into it (0.7 s)
  and rolls 4 cm to the front.
- **Night glow (E8).** The header signs (`lit_sign`, glow `#FFE2A6`) and the drinks machine's front (`lit_panel`,
  glow `#CFE8FF`) glow from dusk, as the windows do; nothing glows by day.

## The market stall

Four walnut posts (the back two taller) with knee braces, beams, a honey table top of four planks with an apron and a
slatted lower shelf on rails, a striped awning (seven slats, green and white, a scalloped valance) leaning down to the
front, a framed chalk board hung on strings from the front beam (chalked lines, a drawn apple), four open slatted crates
with corner posts tilted 10° to the front, heaped in two layers with lemons, apples, oranges and tomatoes, a price card
on a stick in each, a set of scales and paper bags, and two slatted spare crates on the lower shelf. Static. It is
placed ten times in the town, so it stays under half its size class's target (4,000).

| Model | Size (w × h × d) | Triangles (far) | Draws | Clip | Colour parts |
|---|---|---|---|---|---|
| `model.iceCreamCart` | 1.73 × 2.35 × 0.96 m | 6,444 (1,543) | 3 | `lid_open` (iceCream, delay 1 s, hold 1.4 s) | Body, Parasol, Bands |
| `model.hotDogStand` | 1.58 × 2.38 × 0.98 m | 5,652 (1,382) | 3 | `lid_open` (hotDog, delay 1 s, hold 1.4 s) | Body, Awning |
| `model.vendingSnack` | 0.90 × 1.88 × 0.75 m | 6,366 (1,440) | 3 | `drop` (snack) | Body |
| `model.vendingDrink` | 0.90 × 1.88 × 0.79 m | 3,552 (770) | 2 | `drop` (drink) | Body |
| `model.marketStall` | 1.84 × 2.14 × 0.98 m | 3,884 (898) | 3 | — | Body, Awning |

Detail pass 2026-10-22 (the forms and the kit). Before it: the ice cream cart 3,778, the hot dog stand 3,532, the
snack machine 3,132, the drinks machine 3,312, the market stall 5,248.

Draw calls: the carts take the plain body, the fabric awning or parasol and the lid; the snack machine the plain
body, its glass and the snack; the drinks machine the body and the can (plus one at night for the glow); the stall
wood, fabric and plain.

`check-glb`: 5 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings for all ten files (the
models and their far versions).

## Rebuild

```
cargo art stalls            # the five .glb and .far.glb, the pack, check-glb
cargo art stalls --preview  # previews/stalls/<id>.png, <id>@<clip>.png, <id>.far.png
```
