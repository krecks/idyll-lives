# Art pipeline

Scripts that produced the current textures, placeholder models and manifest entries.
They are only needed to regenerate or change the art; the results already live in
`web/public/assets/` (see `CREDITS.md` there for sources and licences — all CC0).

Run Python with `-I` (isolated mode); downloads are treated as data only.

| Script | Purpose |
|---|---|
| `ph_tex.py <outdir> <res> <maps> <id>…` | Download Poly Haven texture maps (CC0) |
| `process.py <src_root> <dst_root>` | Convert downloaded maps into the game's `albedo/normal/arm` JPGs (needs Pillow) |
| `process_house.py <src_root> <dst_root> [set…]` | House/interior textures (siding, brick, roof, stone, floors, carpet, wallpaper, trim, door, lawn) from Poly Haven + ambientCG 1K-JPG downloads: greyscale detail maps, siding stagger, orientation fixes, ARM packing (needs Pillow + numpy) |
| `materials.py` | Writes `materials.json`: PBR material + environment entries |
| `models.py` | Writes `models.json`: detailed placeholder models (primitive parts with finishes) |
| `objects_extra.py [out.json]` | Writes `objects_extra.json`: placeholders for the newer objects (treadmill, weightBench, chessTable, computerDesk, easel, mirror, workbench, telescope, piano, lamp, diningTable) and the designs `furniture.py` draws from (the former `@modern` / `@cozy` / `@minimal` variants, no longer written to a manifest). Validates fields, colours, footprint bounds (~5 cm margin) and ≤4 finishes; exits non-zero on violations |
| `apply_entries.py <manifest> <entries.json>` | Merges generated entries into `manifest.json` in place |
| `clouds.py <out.png>` | Generates the procedural, tileable cloud layer (`textures/sky/clouds.png`) |
| `furniture.py [out.json]` | Entries for the base objects (only plain `model.<id>` keys: objects have no styles), all primitive parts (some reused or recoloured from `objects_extra.py`, the rest from `furniture_prims.py`). Validates footprints; apply with `apply_entries.py` |
| `furniture_prims.py` | Primitive-part pieces that replaced the downloaded Kenney / Poly Haven furniture (fridges, sink cabinet, toilet, showers, camel-back sofa, bergere, wing and rocking chairs, potted plants, CRT telly, steel desk, tea sets, cabinets, stoves, washer-dryer, crates, ferns...), each drawn in the box the old file was fitted into so seats, tops and backs stay put. Imported by `furniture.py` and `pack_<trait>.py` |
| `pack_<trait>.py` | Model entries for one personality pack (`model.<trait>.*`), all primitive parts, written with `apply_pack.py` |
| `nature.py <manifest> \| --pack <out.json>` | Trees (oak, maple, birch, cherry blossom, apple, Japanese maple, magnolia, weeping willow, pine, blue spruce, Italian cypress, each with a `.far` LOD), bushes, hedge, lawn tuft, wildflowers, the flower bed and the garden catalog's models (`model.garden.*`: flowers, shrubs, fruit and vegetables, house plants, decor), all primitive parts: crowns of lumpy `blob`s on the leafy `foliage` finish over `bark` limbs, conifers as stacked cones, flowers as discs, cups and balls. Replaces those entries of the base manifest by key (everything else untouched) or writes them as a pack to preview with `?pack=`. Keeps each key's `vary`; checks colours and at most four material groups and prints parts and triangles per model. The `foliage` and `bark` finishes get wind sway and a per-instance tint in `render/babylon/nature.ts` |
| `catalog.py` | The catalogue's models from development build 0.20 (`content/furniture.json`) (only plain `model.<id>` keys: objects have no styles; the style designs stay in the script as parts to draw from), all primitive parts (reusing shapes from `catalog2.py`); writes its own compact manifest `web/public/assets/catalog/manifest.json` (listed in the base manifest's `packs`; one entry per line). Things on a wall hang with their back on the wall face (`z = -0.43`), ceiling things hang from 2.8 m. Validates footprints and finishes |
| `community.py` | The town's community things (`content/community.json`: street furniture, playground, food carts and vending machines, shop and café fittings) and the street dressing props (`model.street.*`, drawn as thin instances by `render/babylon/street.ts`), all primitive parts with at most four finishes each; writes `web/public/assets/community/manifest.json` (in the base manifest's `packs`). Validates footprints and finishes |
| `catalog2.py` | The second catalogue's models (`content/catalog2/*.json`: separate tables and chairs, counters, sectional pieces, beds, tabletop things, lamps, garden things), all primitive parts; writes `web/public/assets/catalog2/manifest.json` (in the base manifest's `packs`). Things on the `surface` layer stand on y = 0 at their middle (the game lifts them onto their top). Also records each model's top (`TOPS`: rectangle and height), seat (`SEATS`) and bulb height (`LIGHTS`), with `surface_of`, `seat_of`, `radius_of` to keep the content's `surface`, `seat` and `radius` in line with the drawing. Validates footprints, tops and finishes (at most 4 per model) |
| `apply_pack.py <trait> <entries.json>` | Merges entries into `web/public/assets/packs/<trait>/manifest.json` in place |
| `blender/` | The Blender-built glTF models: `build.py <family>` builds a family script from `blender/families/` (the garden decor, the kitchen fridge, the living room's wall bed) on the `blender/gamekit/` toolkit, exports its `.glb` files with their moving parts, writes its pack and runs `check_glb.py`. See [blender/README.md](blender/README.md), the guide for people and AI agents |
| `blender/check_glb.py` | The gate for glTF models (plain Python, no Blender): checks every `.glb` the manifests use against its entry (origin and size, triangles and draw calls against the budgets, materials mapped to finishes, one-sided, no unused UVs, extensions the game reads, texture sizes) and exits 1 on an error. Inside Blender, `-- --preview` also re-imports each file and renders it to `previews/rt_<name>.png` |

Nature and garden flow: `python3 -I -B nature.py ../../web/public/assets/manifest.json`. To look at the results without
starting a game, run `pnpm dev` and open `/gallery.html` (dev only): it draws models in a grid with the game's loader,
finishes and plant shader (`?filter=garden`, `?keys=model.tree,model.pine`, `?cols=`, `?gap=`, `?pack=`; drag to orbit).

Typical flow: download → `process.py` → copy into `web/public/assets/textures/` →
`materials.py` / `models.py` → `apply_entries.py web/public/assets/manifest.json materials.json` (and `models.json`).

Furniture flow: `catalog.py`, `catalog2.py` and `community.py` write their own manifests; `furniture.py` and
`pack_<trait>.py` → `apply_entries.py` / `apply_pack.py`. Every model is primitive parts (`placeholder`), including the
shapes `lathe` (`profile`) and `torus`, drawn by `render/babylon/models.ts`.

Blender flow (real glTF art): `python3 -B tools/art/blender/build.py <family>` (Blender 5.2 LTS; `--preview` renders
instead of exporting). Each family script in `blender/families/` builds its models on `blender/gamekit/`, exports `.glb`
files into `web/public/assets/models/<family>/` and writes its override pack (`packs/<pack>/manifest.json`, listed in the
base manifest's `packs`), which keeps each key's `placeholder` as the fallback and maps the file's materials onto the
game's finishes. `tools/assets/optimize.mjs` makes the meshopt copy for the build. How to make a new object or moving
part: [blender/README.md](blender/README.md); the standard: [docs/design/objects.md](../../docs/design/objects.md).
