# Bathroom — notes

`bathroom.py` builds the five bathroom sinks into `web/public/assets/models/bathroom/` and their entries
in `packs/bathroom/manifest.json`: `model.sink` (`sink.glb`), `model.sinkPedestal` (`sinkPedestal.glb`),
`model.vanityDouble` (`vanityDouble.glb`), `model.sinkVessel` (`sinkVessel.glb`) and `model.sinkConsole`
(`sinkConsole.glb`). They replace the primitive-part placeholders in `catalog.py` / `catalog2.py`, which
stay as the fallback. First made 2026-10-14. The pack is listed last in the base manifest's `packs`, so it
overrides the base (`model.sink`), `catalog` and `catalog2` entries by key.

## Shared

- **Palette.** One `G.Palette` for the family (`M`): `body` (the painted cabinet / apron), `porcelain`,
  `stone`, `wood`, `trim`, `metal`, `chrome`, `brass`, `blackmetal`, `fabric`, `dark`, `teal`, `water`, and
  `towel` (a plain satin, so a towel does not cost an extra draw call). `PAINT = ['body']`.
- **Water on use.** Each sink has one `water` moving part. The stream is built full length from the spout
  tip into the basin, its origin on the spout tip; before `G.settle()` it is scaled to 0.001 along its
  length, so it is hidden at rest and does not affect the bounds. `G.grow(water, 'water_on', seconds=0.4)`
  keys it from there to scale 1. `motions` plays `water_on` for the `wash` animation tag, with no `hold`,
  so the water runs for the whole use and retracts afterwards (the `wash` tag covers both the `wash` and
  `washUp` interactions).
- **Basins.** `_basin` is a `G.revolve` with thickness (an oval bowl); `_rect_basin` is a rectangular shell
  (floor and four thin walls) for the base cabinet sink, which has a rectangular worktop cut-out. The solid
  cabinet top must not sit above the basin floor, so each sink has painted "shoulders" filling from the
  basin floor up to the worktop — otherwise the painted cabinet shows through as the basin interior.

## The models

| Model | Size (w × h × d) | Triangles | Draws | Note |
|---|---|---|---|---|
| `model.sink` | 0.86 × 1.26 × 0.62 m | 3,320 | 2 | base cabinet, shaker doors, rectangular inset basin, gooseneck |
| `model.sinkPedestal` | 0.56 × 1.06 × 0.47 m | 1,540 | 2 | column, oval bowl, tap deck with soap and a tooth mug |
| `model.vanityDouble` | 1.90 × 1.12 × 0.56 m | 4,396 | 3 | two-basin cabinet, two taps (no mirrors) |
| `model.sinkVessel` | 0.84 × 1.16 × 0.54 m | 1,920 | 3 | counter with a raised vessel bowl and a folded towel |
| `model.sinkConsole` | 0.60 × 1.06 × 0.48 m | 1,108 | 3 | open console on two legs, oval bowl |

All match their placeholder bounds (`check_glb.py`: ok; Khronos validator: 0 errors, 0 warnings). Each
lists the parts the player may colour in `paint`, the `body` first.

## The double vanity has no mirrors

It had two (frames in the file, the glass as primitive `parts`). They were taken off on 2026-10-14: the
player chooses the mirror they hang above it (the wall mirror, the mirror cabinet), as for any sink. Its
placeholder (`catalog.double_vanity`) has none either.

## Rebuild

```
python3 -B tools/art/blender/build.py bathroom            # the five .glb, the pack, check_glb
python3 -B tools/art/blender/build.py bathroom --preview  # previews/sink.png, sink@water_on.png, …
```
