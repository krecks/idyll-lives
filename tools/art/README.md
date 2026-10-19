# Art pipeline

Scripts that produced the current textures and materials, and the Blender pipeline that builds every model.
They are only needed to regenerate or change the art; the results already live in `web/public/assets/`
(see `CREDITS.md` there for sources and licences — all CC0). Models come from Blender alone
(`blender/families/`); the primitive-part scripts that came before are gone.

Run Python with `-I` (isolated mode); downloads are treated as data only.

| Script | Purpose |
|---|---|
| `ph_tex.py <outdir> <res> <maps> <id>…` | Download Poly Haven texture maps (CC0) |
| `process.py <src_root> <dst_root>` | Convert downloaded maps into the game's `albedo/normal/arm` JPGs (needs Pillow) |
| `process_house.py <src_root> <dst_root> [set…]` | House/interior textures (siding, brick, roof, stone, floors, carpet, wallpaper, trim, door, lawn) from Poly Haven + ambientCG 1K-JPG downloads: greyscale detail maps, siding stagger, orientation fixes, ARM packing (needs Pillow + numpy) |
| `materials.py` | Writes `materials.json`: PBR material + environment entries |
| `clouds.py <out.png>` | Generates the procedural, tileable cloud layer (`textures/sky/clouds.png`) |
| `blender/` | The Blender-built glTF models, every model in the game: `build.py <family>` builds a family script from `blender/families/` on the `blender/gamekit/` toolkit, exports its `.glb` files with their moving parts, writes its pack and runs `check_glb.py`. See [blender/README.md](blender/README.md), the guide for people and AI agents |
| `blender/check_glb.py` | The gate for glTF models (plain Python, no Blender): checks that every model the content draws has a `.glb`, and every `.glb` the manifests use against its entry and its content (origin on the floor and at its `centre`, the wall's face, the footprint, a top's `radius`, triangles and draw calls against the budgets, materials mapped to finishes, one-sided, no unused UVs, extensions the game reads, texture sizes) and exits 1 on an error. Inside Blender, `-- --preview` also re-imports each file and renders it to `previews/rt_<name>.png` |

Texture flow: download → `process.py` (`process_house.py` for the house's) → copy into
`web/public/assets/textures/` → `materials.py` → merge its entries into `web/public/assets/manifest.json` by hand
(they are few and stable). `clouds.py` regenerates the sky's cloud layer.

To look at the models without starting a game, run `pnpm dev` and open `/gallery.html` (dev only): it draws models in
a grid with the game's loader, finishes and plant shader (`?filter=garden`, `?keys=model.tree,model.pine`, `?cols=`,
`?gap=`, `?pack=`; drag to orbit).

Blender flow (real glTF art): `python3 -B tools/art/blender/build.py <family>` (Blender 5.2 LTS; `--preview` renders
instead of exporting). Each family script in `blender/families/` builds its models on `blender/gamekit/`, exports `.glb`
files into `web/public/assets/models/<family>/` and writes its override pack (`packs/<pack>/manifest.json`, listed in the
base manifest's `packs`), which maps the file's materials onto the game's finishes. `tools/assets/optimize.mjs` makes the meshopt copy for the build. How to make a new object or moving
part: [blender/README.md](blender/README.md); the standard: [docs/design/objects.md](../../docs/design/objects.md).
