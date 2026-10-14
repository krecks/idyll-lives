# Lazy pack — notes

`pack_lazy.py` builds the six P10 lazy models into `web/public/assets/models/pack_lazy/` and their entries in
`packs/pack_lazy/manifest.json`, an override pack listed after `packs/lazy/manifest.json`. Their primitive-part
placeholders (`tools/art/pack_lazy.py`, in `packs/lazy`) stay as the fallback. Built 2026-10-14; they sit in
Validate › Lazy until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
What the renderer fixes is kept: the bed's mattress top at 0.49 (`BEDS` default 0.5; it is in `BLANKET_BEDS`, so no
duvet is modelled, only a throw folded at the foot), the pod's and the poufs' seats at 0.45, the vibration plate's
deck at 0.125 (content `stand.height` 0.12, where the resident stands).

| Model | Moving part and clip | Played by |
|---|---|---|
| `snoozecloudBed` | the head section of the mattress and its pillows, hinged on its front edge: `head_up` raises it 30° (1.2 s), `hold` 2 s | `sleepIn` |
| `napPod` | the hood, a band of a sphere about the egg's upper half, folded back over its top: `hood` swings it 60° forward, half over the opening (1 s) | `powerNap`, `curlUp` |
| `quiltTable` | the quilt's back edge, hung from the table top: `quilt` lifts it 70° over the sitters' laps (0.8 s) | `tuckIn`, `doze` |
| `deliveryLocker` | the left locker of the middle row, hinged on its outer edge: `door_open` swings it 105° open on a takeout bag (0.7 s), `hold` 2 s | `takeout`, `noodles` |
| `vibePlate` | the deck the resident stands on: `shake` rocks it ±3° side to side, four times a second (0.25 s, a loop) | `shake` (loop) |
| `smartHub` | the tablet, hinged in its dock: `screen_up` tilts it from lying back (72°) up to 15° (0.8 s) | `program`, `story` |

Where the plan's table says "Loop (E1) … Until then static" (the plate), the loop itself is built.

**Deviations.**
- `head_up` has `hold` 2 s (the plan names none). Sleepers lie with their heads at the back of the bed (0.84 m behind
  its middle), on the head section: held up for the whole nap it covers their heads. Raised for 2 s it shows as they
  lie down and settles flat for the nap. The two accent cushions stand at the bed's outer corners, clear of the heads.
- `shake` is 0.25 s long, not about 1 s: one slow rock a second reads as tilting, not vibrating.
- The nap pod keeps inside its 1x1 footprint (0.88 × 0.94 m); its placeholder overhangs it front and back (1.15 m
  deep), hence `check_glb`'s one warning (z size 0.94 m against 1.15 m). The egg is 1.42 m tall with its hood, so a
  sitter's head (about 1.3 m) is inside it.
- Colour parts: the plan's "Body, Upholstery" fit the bed, the pod and the quilt table. The locker, the plate and the
  hub have no upholstery: they offer Body with Doors and Trim, Trim and Deck, and Speaker.

**Draw calls: 3 or fewer.**
- the bed: base, headboard and feet plain, the mattress foot and throw `fabric`; the head section all `fabric` (the
  pillows too);
- the pod: the egg, stem and base plain, the seat, back cushion and rug `fabric`; the hood `fabric`;
- the quilt table: the top plain satin (`oak`), the quilt, poufs and cushions `fabric`; the flap `fabric`;
- the locker (2): all plain, the door plain;
- the plate (2): all plain, the deck plain;
- the hub: the table `wood`, the speaker plain satin, the tablet plain.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.lazy.snoozecloudBed` | 1.86 × 1.29 × 1.86 m | 4,772 | 3 | Body, Upholstery (the mattress), Throw |
| `model.lazy.napPod` | 0.88 × 1.42 × 0.94 m | 4,308 | 3 | Body (the egg), Upholstery (the cushions), Hood |
| `model.lazy.quiltTable` | 1.74 × 0.74 × 0.89 m | 3,032 | 3 | Body (the table top), Upholstery (the quilt), Poufs |
| `model.lazy.deliveryLocker` | 0.86 × 1.96 × 0.81 m | 3,104 | 2 | Body, Doors, Trim |
| `model.lazy.vibePlate` | 0.67 × 1.16 × 0.76 m | 1,380 | 2 | Body, Trim, Deck |
| `model.lazy.smartHub` | 0.60 × 0.81 × 0.84 m | 2,236 | 3 | Body (the table), Speaker |

`check_glb.py`: 6 models, 0 failed; one warning (the pod's depth, above). Khronos validator: 0 errors, 0 warnings
each. Clips bench: every clip at rest and open on WebGPU and WebGL (the bed from the side, the quilt from behind).
In play (`wants-asking`, WebGPU): each object used by a resident: the bed's head rising as they lie down for a lie-in
and settling, the pod's hood over a power nap, sitting tucked in at the quilt table (the flap's lift seen on the clips bench; in play the poufs hide it), the locker door open for a takeout,
shaking on the plate, programming on the hub; the quilt table turned to all four facings in the yard, the others to
one or two; colours on the clips bench; at night (`office-night`, WebGL) in the house.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_lazy            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_lazy --preview  # previews/<id>.png, <id>@<clip>.png
```
