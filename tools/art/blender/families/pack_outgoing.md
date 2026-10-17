# Outgoing pack — notes

`pack_outgoing.py` builds the six P14 outgoing models into `web/public/assets/models/pack_outgoing/` and their
entries in `packs/pack_outgoing/manifest.json`, an override pack listed after `packs/outgoing/manifest.json`. Their
primitive-part placeholders (`tools/art/pack_outgoing.py`, in `packs/outgoing`) stay as the fallback. Built
2026-10-14; they sit in Validate › Outgoing until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face (depth −0.43; the placeholders' backs were 2 cm into the wall).

| Model | Moving part and clip | Played by |
|---|---|---|
| `partyBar` | the beer tap's lever, hinged on the faucet: `tap` pulls it 40° towards the front (0.5 s), `hold` 2 s | `mixDrinks`, `soda` |
| `conversationSectional` | the three seat cushions (tops at 0.45), origin on their underside: `sink_in` presses them 20 % (0.6 s) | `hangOut` |
| `karaokeStage` | both mic stands' upper tubes, booms and mics, lowered at rest: `mic_up` raises them 20 cm (0.8 s) | `practise`, `perform` |
| `videoCallNook` | the ring light and its phone clip, pivoting in the yoke: tipped back 60° at rest (face to the ceiling), `ring_light` swings it up to face the caller (0.7 s) | `call` |
| `danceFloor` | the sixteen tiles in two checkerboard parts: `tiles` (a loop, 1.2 s) lifts one set 2.5 cm and back, then the other | `dance` (`loops`) |
| `photoWall` | a new snapshot in the board's bottom row, built full size and shrunk to nothing: `photo` grows it onto the board (0.8 s) | `reminisce` |

The dance floor's table entry asks for the loop itself (`tiles`, E1), built as two parts on one clip with
`G.turns` (the two sets take turns lifting), and it starts and ends at rest.

**Draw calls: 3 each.** The still body takes its plain finishes and at most one textured finish, the moving parts
the rest. So that it stays at 3:
- the bar: the counter is coral satin, its top, shelves and back cabinet `wood`; the stool tops are plain satin;
- the sectional: frame, back and arms `fabric` on the still body (with the back cushions and the pouf), the seat
  cushions `fabric` on their moving part; the console, the plant and the throw pillows are plain;
- the karaoke stage: the deck is `wood`, the speakers plain satin (no speaker cloth draw);
- the video-call corner: the desk top and the chair's shell and legs are `wood`, the chair's cushions plain satin;
- the dance floor: no textured finish; the two tile sets are the two moving parts (gloss);
- the photo wall: the frame and legs are `wood`, the cork plain matte.

No glass anywhere: the bar's glasses and bottles are plain gloss. The bar's flutes are on the `body` material, so
colouring the body colours the whole front (the tap's handle too). Nothing glows at night (the table asks for no
night clip; the dance floor's tiles and the photo wall's string lights could take a `glow` later, one more draw
each at night).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.outgoing.partyBar` | 1.84 × 2.00 × 0.88 m | 7,872 | 3 | Body, Upholstery, Back bar |
| `model.outgoing.conversationSectional` | 1.90 × 1.42 × 1.86 m | 4,612 | 3 | Body, Upholstery |
| `model.outgoing.karaokeStage` | 1.90 × 1.63 × 0.89 m | 2,532 | 3 | Body, Speakers |
| `model.outgoing.videoCallNook` | 0.84 × 1.28 × 0.79 m | 3,132 | 3 | Body, Upholstery, Wood |
| `model.outgoing.danceFloor` | 1.92 × 2.33 × 1.88 m | 3,632 | 3 | Body, Speakers |
| `model.outgoing.photoWall` | 0.90 × 2.16 × 0.49 m | 3,634 | 3 | Body, Board |

"Upholstery" (the plan's second colour part) fits the bar's stool tops, the sectional's cushions and the desk
chair; the stage and the dance floor take their speakers instead, the photo wall its cork board.

`check_glb.py`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Clips bench: every
clip at rest and open on WebGPU and WebGL (the dance floor mid-loop, each tile set up in turn); the bar coloured
there. In play (`wants-asking`, WebGPU, in the yard): the sectional (hangOut, seated), the dance floor (dance,
the loop going), the bar (mixDrinks) and the video-call corner (call, seated facing the desk, the ring up) used by
a resident; the video-call corner at all four facings, the stage and photo wall turned; the bar and sectional
bought coloured; the same at night (`office-night`, WebGL).

## Rebuild

```
python3 -B tools/art/blender/build.py pack_outgoing            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_outgoing --preview  # previews/<id>.png, <id>@<clip>.png
```
