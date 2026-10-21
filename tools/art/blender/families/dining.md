# Dining — notes

`dining.py` builds the kitchen's **K7 dining tables**, **K8 chairs and stools** and **K9 dining storage** into
`web/public/assets/models/dining/` and their entries in `packs/dining/manifest.json` (listed in the base
manifest's `packs`, and the only manifest that defines these keys).

K7 (2026-10-14): the dining table with its two chairs (`model.diningTable`), the bistro set
(`model.bistroSet`), the glass and farmhouse dining sets (`model.diningGlass`, `model.diningOak`) and the second
catalogue's chair-less tables: `tableSquare2`, `tableBistroRound`, `tableDining4`, `tableDining6`,
`tableBanquet8`, `tableRound4`, `tableSquare8`, `tableFarmhouse6`, `tableBarHigh`.

K8 (2026-10-14): the bar stool (`model.barStool`) and dining chair (`model.diningChair`), and the second
catalogue's `stoolCounter`, `stoolBarHigh`, `chairWood`, `chairUpholstered`, `chairModern`, `chairWindsor`,
`chairBentwood`, `chairFolding`, `chairArm` and `benchDining`.

K9 (2026-10-14): catalog2's `sideboard`, `buffet`, `chinaCabinet` and `barCart`.

## Facts kept

- **Table tops** at the content's `surface.height`: 0.75 for catalog2's tables (`TT`) and the bistro set, 1.06 for
  the high café table (`BT`), 0.76 for the glass set, 0.765 for the dining table and the farmhouse set. Tabletop
  things bought onto `tableDining4` stand on its top (seen in play). The plank joints on the banquet and farmhouse
  tops are flush (0.5 mm proud), so nothing sinks.
- **Seats** at `characters.ts` SEATS or the content's `seat`: the seat's (hard) top at that height, the pad or
  cushion a little above, as the old parts: the sets' chairs 0.46 (behind the table, `fwd` −0.19 / −0.22 /
  −0.24), the bar stool 0.74 (sitters at 0.72), the counter stool 0.685 (0.68), the high stool 0.805 (0.8), the
  chairs 0.46–0.48, the carver chair's board 0.465 under a cushion to 0.52 (0.5), the bench 0.465 (0.47, two
  slots). catalog2's seats are centred 5 cm forward (`SZ`), as the old parts.
- **The sets' chairs** are part of the table (as the old parts drew them): residents sit on them.
- **Storage** stands where the game places it against the wall (`centre=` the old part's middle, then
  `G.settle()`), so the sideboard's and buffet's tops (0.8, 0.9) are where the content's `surface` is.

## Materials

Each part the player colours has a material of its own, even where two share a colour (`top_oak`, `legs_oak`).
Tops and legs are `wood` (or `gloss`/`satin` white, `satin` black, `glass`), frames `metal`/`chrome`, pads
`fabric`; what stands on the sets (plates, food, glasses, a vase of flowers, a cup) is plain `gloss`/`satin`, so
it costs no draw call. The bar cart's trays, bottles and wine glasses are `glass` (one group).

## Clips

- **Stools** (`barStool`, `stoolCounter`, `stoolBarHigh`): the seat (with the high stool's back) is a part of its
  own on a swivel plate, pivoting about its middle. `swivel` turns it 20° in 0.5 s for `sit` and `drink`; `hold`
  0.6 s, then it turns back. `ride: swivel` carries the sitter round with it, as the office chairs do. The legs,
  foot ring and hub are the still part. The wood stools' feet and swivel plate are in the frame's wood, so a
  stool draws in 3 groups (base wood; seat wood and fabric).
- **Bar cart**: the cocktail shaker is a part of its own pivoting at its foot, in the middle of the top tray's
  front edge, before the resident who mixes from the front tile. `shake` lifts it 10 cm and tips it 30° towards
  them (one clip, both channels) in 0.6 s for `mix`; `hold` 1.5 s.
- **Storage doors:** the sideboard's four fluted doors, the buffet's five panelled doors and the china cabinet's two
  glazed doors are parts of their own, hinged on their outer edges. All three are `cupboard` stations (content
  `stations`): clean things are put away in the nearest cupboard or counter (cooking `putAwayAt`), and one door
  swings open 100° in 0.9 s while they are (`door_open`, `fetch`, `hold` 1.5 s): the sideboard's second, the
  buffet's wide middle one, the china cabinet's left glazed one. One door each, as each opening door costs draws of
  its own (the sideboard's walnut and fluting two). Behind the sideboard's and buffet's is a real cupboard
  (`_cabinet(hollow=…)`) with a shelf and stacks of plates and bowls (`_crockery`); the china cabinet's opens onto
  its plates. The other doors are still (no draw call of their own). The china cabinet's base doors and the
  buffet's drawers are in the body.

## K7 — dining tables

| Model | Size (w × h × d) | Triangles | Draws | Notes |
|---|---|---|---|---|
| `model.diningTable` | 1.62 × 1.03 × 0.87 m | 4,816 | 3 | oak top to 0.765, runner, vase, two place settings, two ladder-back chairs |
| `model.bistroSet` | 0.62 × 1.00 × 0.91 m | 3,496 | 3 | round oak top on a black column, cup and vase; a café chair with a looped back |
| `model.diningGlass` | 1.70 × 1.11 × 0.91 m | 6,068 | 2 | glass top on chrome sled legs; two dark chairs with grey seats |
| `model.diningOak` | 1.76 × 1.10 × 0.92 m | 5,700 | 3 | thick oak top with breadboard ends on square tapered legs; two chairs |
| `model.tableSquare2` | 0.82 × 0.75 × 0.82 m | 1,388 | 1 | round tapered legs with collars, aprons |
| `model.tableBistroRound` | 0.72 × 0.75 × 0.72 m | 1,196 | 1 | white gloss top, black column and cast foot |
| `model.tableDining4` | 1.84 × 0.75 × 0.88 m | 588 | 1 | square tapered oak legs, aprons |
| `model.tableDining6` | 2.84 × 0.75 × 0.88 m | 1,812 | 1 | walnut, six round tapered legs with collars |
| `model.tableBanquet8` | 3.84 × 0.75 × 0.88 m | 1,728 | 1 | planked top, six legs with feet, an H-stretcher |
| `model.tableRound4` | 1.52 × 0.75 × 1.52 m | 1,864 | 1 | birch, turned column on four splayed feet |
| `model.tableSquare8` | 1.86 × 0.75 × 1.86 m | 1,020 | 1 | white satin top, black legs braced by a square frame |
| `model.tableFarmhouse6` | 2.86 × 0.75 × 0.88 m | 1,968 | 1 | planked top on two braced trestles and a wedged stretcher |
| `model.tableBarHigh` | 0.70 × 1.06 × 0.70 m | 1,324 | 1 | dark top at bar height, column, foot ring |

## K8 — chairs and stools

| Model | Size (w × h × d) | Triangles | Draws | Clips (uses) |
|---|---|---|---|---|
| `model.barStool` | 0.39 × 0.74 × 0.39 m | 1,584 | 3 | `swivel` 0.5 s (sit, drink; `hold` 0.6 s; ride) |
| `model.stoolCounter` | 0.39 × 0.69 × 0.39 m | 1,584 | 3 | `swivel` (as the bar stool) |
| `model.stoolBarHigh` | 0.39 × 1.12 × 0.40 m | 1,748 | 2 | `swivel` (seat and back turn) |
| `model.diningChair` | 0.44 × 1.00 × 0.46 m | 1,900 | 2 | static (painted ladder-back, tie-on pad) |
| `model.chairWood` | 0.44 × 0.90 × 0.45 m | 1,136 | 1 | static (spindle back) |
| `model.chairUpholstered` | 0.46 × 0.99 × 0.50 m | 804 | 2 | static |
| `model.chairModern` | 0.44 × 0.79 × 0.50 m | 1,732 | 1 | static (curved shell on a sled frame) |
| `model.chairWindsor` | 0.47 × 0.98 × 0.47 m | 1,004 | 1 | static (saddle seat, bow bent from one tube) |
| `model.chairBentwood` | 0.43 × 0.97 × 0.45 m | 1,868 | 2 | static (back legs bend into the back loop) |
| `model.chairFolding` | 0.46 × 0.89 × 0.48 m | 1,176 | 1 | static |
| `model.chairArm` | 0.57 × 0.92 × 0.47 m | 1,588 | 2 | static |
| `model.benchDining` | 1.60 × 0.47 × 0.36 m | 1,080 | 1 | static |

## K9 — dining storage

| Model | Size (w × h × d) | Triangles | Draws | Clips (uses) |
|---|---|---|---|---|
| `model.sideboard` | 1.82 × 1.10 × 0.67 m | 6,132 | 4 | `door_open` (putting clean things away, `fetch`) |
| `model.buffet` | 1.88 × 0.90 × 0.67 m | 4,216 | 3 | `door_open` (putting clean things away, `fetch`) |
| `model.chinaCabinet` | 0.98 × 2.10 × 0.55 m | 5,608 | 4 | `door_open` (putting clean things away, `fetch`) |
| `model.barCart` | 0.77 × 1.09 × 0.45 m | 4,468 | 3 | `shake` 0.6 s (mix, `hold` 1.5 s) |

`check-glb`: 29 models, 0 failed; two warnings, the sideboard's and the china cabinet's 4 draw calls. Khronos
validator: 0 errors, 0 warnings each. Seen in the gallery next to their files (WebGPU), the clips bench (WebGPU and
WebGL2), the pose bench (`/poses.html?seats=…`: a man and a woman on every chair, stool, bench and set) and in play
(stools sat on with the swivel playing, WebGPU and WebGL2; the bar cart's shake; the oak and bistro sets sat at;
the storage against the wall). All 29 sat in Validate › Tables, Chairs and Dining Storage until signed off.

## Rebuild

```
cargo art dining            # the .glbs, the pack, check-glb
cargo art dining --preview  # previews/<id>.png, <id>@<clip>.png
```
