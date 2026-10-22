# Cheerful pack — notes

`pack_cheerful.py` builds the six P2 cheerful models into `web/public/assets/models/pack_cheerful/` and their
entries in `packs/pack_cheerful/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sat in Validate › Cheerful until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
with one moving part. Flat shapes (the jukebox's arch, the lamp's hills, the booth's pleated curtains, the hammock's
sling) are meshes made from points in the family (`_slab`, `_ring`, `_pleats`, `_obj`).

| Model | Moving part and clip | Played by |
|---|---|---|
| `jukebox` | the top record of the stack in the window, hinged on its front edge: `record` swings it up 90° to stand in the window (0.8 s); its label (with a coral stripe) is a part inside it (`G.attach`): `spin` turns it at 33⅓ a minute, a loop. The arch bands and the bubble tubes glow at night (`amber`, `sunny`: M-packs-07) | `dance` |
| `karaokeMachine` | the stand's upper tube, boom and microphone: `mic_up` raises them 25 cm out of the lower tube (0.8 s). The lyrics screen glows while it is sung to (`lyrics`, `screen`: M-packs-07) | `sing` |
| `mocktailCart` | the cocktail shaker, hinged on its foot's edge: `shake` tips it 35° over the pink drink (0.6 s) | `mix` |
| `rainbowHammock` | the sling with its spreader bars, ropes, hooks and pillow, pivoting on the hooks: `sway` rocks it 5° each way (2.4 s), a loop; `ride: sway`. A 3 × 1 stand, the sling 2 m between the spreader bars at ±1.015 (`furniture.json` `sling`: low 0.5, end 0.9, half 1.015, head 0.75), so a grown-up lies in it near flat (2026-10-21: 2 × 1, a 1.5 m sling, the knees up: M-packs-06, Q-07) | `swing`, `doze` (loops) |
| `sunriseLamp` | the sun, set behind the hills: `sunrise` slides it 22 cm up into the sky (1.5 s). The lamp has a content `light` (2.5 m, at 1.25 m), switched as a lamp, its sky glowing with it (M-packs-07) | `stretch` |
| `partyBooth` | the right side curtain, hanging closed along its rail: `curtain` gathers it to the back (scaled to a quarter along its rail, 0.8 s), `hold` 1.5 s | `party` |

**Draw calls: 3 each** (the lamp 2). The still body takes its plain finishes and at most one textured finish, the
moving part one more:
- the jukebox's speaker cloth is the body's `fabric`; the record is plain gloss;
- the karaoke's stage rug is the body's `fabric`; the microphone part is plain;
- the cart's canopy is the body's `fabric`; its glasses are opaque (a drink-coloured tumbler under a clear rim
  ring, no glass draw), the pineapple's crown plain satin;
- the hammock's stand is all `wood` (nothing plain on it, so one draw); the sling's six stripes are `fabric` and
  the spreader bars, ropes, hooks and pillow plain, on the moving part (two draws);
- the booth's upholstery and its fixed left curtain are the body's `fabric`, the moving curtain `fabric`;
- the lamp has no textured finish: the case, scene and mat plain, the sun plain.

**Night glow:** the sunrise lamp's sky (`dawn`) glows `#FFB070` at night (one more draw at night). The sun itself
does not: a glow on it showed its outline through the hills while it is set, so it is plain.

Deviations from the plan:
- the jukebox's L spin (E1): the record's label spins (a part inside the record, so it swings up with it); the
  vinyl itself looks the same turned. The label costs a fourth draw (the most allowed);
- the booth's curtain is drawn aside (gathered towards the back panel along its rail) rather than swung on a
  hinge: swung, it would sweep through the backrest or out past the footprint;
- the hammock's throw is left out (it read as a blob on the cupped sling).

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit (README "The toolkit"); keys, sizes, centres, parts, clips, motions, paint and
draws as before. Triangles in the table below (was: before the pass).
- `jukebox`: the cabinet and plinth extruded from rounded outlines; the arch one extruded piece with the window cut
  through it, its bands stepping out; sunburst rays behind the record; the song cards in a chrome frame with rows
  of titles; six turned push buttons in chrome rings; a moulded brass grille frame (`kit.moulding`) with chrome
  bars bursting from a hub; turned feet, caps and collars on the bubble tubes; two grooves on the record.
- `karaokeMachine`: a rounded cabinet with a band round its sides and front; each driver one turned piece (flange,
  rolled surround painted in the trim colour, dished cone, dust cap); a rounded port; top knobs; bent carrying
  handles; the screen in a rounded case on a hinged neck, a bouncing ball over the lyrics; the stage rug's border;
  a weighted mic foot, clutch and screw; the spotlight's can turned with a lip, cooling fins, a yoke bent round it
  with knobs; the mic's boom bent, a tapered handle and band.
- `mocktailCart`: rounded shelves, each gallery one closed loop on spindles; turned finials, feet caps; a grip on
  the handle; the canopy's stripes swept in an arch; a lofted jug with juice in its mouth, a spout and handle; ice
  bucket handles; a lumpy pineapple with a crown of extruded leaves; a halved lemon; straws bent once; citrus
  wheels on two rims; lofted syrup bottles with label bands and pourers; a rounded fridge drawer with a seam and
  bar pull; a turned shaker with its cap.
- `rainbowHammock`: the stand's beam, posts curving out and braces swept, rounded feet, turned post tops; the
  sling's edges corded, tassels in the stripes' colours hanging from them; round spreader bars with ball ends;
  rings and bent hooks; seven ropes a side; a stuffed pillow.
- `sunriseLamp`: a stepped turned base, a collar on the stem, a dimmer on the foot; the case a turned drum with a
  rolled edge; clouds in the dawn sky; a halo round the sun; the rolled mat's layers drawn as spirals on its ends; a
  cork block.
- `partyBooth`: a piped, stuffed seat; the backrest seven swept channels rolling over, piped along the top; piped
  wings; each side of the frame one bent brass tube on a foot; a rounded marquee; the bulbs on wires; round and
  square confetti; a rounded tablet; a striped popcorn bowl; both curtains pleated with a heading and a turned-up
  hem.

| Model | Size (w × h × d) | Triangles (2026-10-22) | Draws | Colour parts |
|---|---|---|---|---|
| `model.cheerful.jukebox` | 0.75 × 1.43 × 0.50 m | 5,728 (was 4,324) | 4 | Body, Trim |
| `model.cheerful.karaokeMachine` | 0.76 × 1.76 × 0.85 m | 4,688 (was 3,684) | 3 | Body, Trim, Rug |
| `model.cheerful.mocktailCart` | 0.84 × 1.64 × 0.57 m | 6,638 (was 6,852) | 3 | Body, Trim, Frame |
| `model.cheerful.rainbowHammock` | 2.84 × 1.17 × 0.70 m | 4,536 (was 3,048) | 3 | Body, Trim, Pillow |
| `model.cheerful.sunriseLamp` | 0.78 × 1.64 × 0.87 m | 3,580 (was 2,868) | 2 | Body, Trim, Mat |
| `model.cheerful.partyBooth` | 1.90 × 1.64 × 1.83 m | 7,652 (was 7,972) | 3 | Body, Trim, Curtain |

The painted materials: the jukebox `cherry`, `amber`; the karaoke `speaker`, `speaker_trim`, `stage`; the cart
`mint`, `canopy`, `brass`; the hammock `wood`, `spreader`, `pillow`; the lamp `housing`, `lamp_metal`, `yoga`; the
booth `booth`, `piping`, `curtain`.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Checked: the gallery
next to the old parts; the clips bench at rest and open on WebGPU and WebGL (the jukebox from behind, the
hammock's sway at its furthest from the side); colours on the clips bench; in play (`wants-asking`, WebGPU by day):
the party booth used (the curtain draws aside and closes after its hold), the hammock swung in (the resident lies
across the sling, as on the old part, and rides the sway), the jukebox danced to (the record up, back down
after), the karaoke, the cart and the lamp used (the sun up while stretching); four facings of the jukebox and a
turned booth in the yard; `office-night` on WebGL and WebGPU (the lamp's sky glowing).

Residents lie across the hammock (the game lays them along the footprint's depth, and the 2x1 sling runs
across): kept from the old part, as the footprint is fixed.

## Rebuild

```
cargo art pack_cheerful            # the six .glb, the pack, check-glb
cargo art pack_cheerful --preview  # previews/pack_cheerful/<id>.png, <id>@<clip>.png
```
