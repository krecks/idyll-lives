# Art pipeline

The tools that produced the current textures and materials (`cargo tex`, in Rust: `crates/tools/src/tex.rs`), and
the Blender pipeline that builds every model. They are only needed to regenerate or change the art; the results
already live in `web/public/assets/` (see `CREDITS.md` there for sources and licences — all CC0). Models come from
Blender alone (`blender/families/`); the primitive-part scripts that came before are gone.

Run them from the repo root. Downloads are treated as data only.

| Command | Purpose |
|---|---|
| `cargo tex ph <outdir> <res> <maps> <id>…` | Download Poly Haven texture maps (CC0) |
| `cargo tex process <src_root> <dst_root>` | Convert downloaded maps into the game's `albedo/normal/arm` JPGs |
| `cargo tex house <src_root> <dst_root> [set…]` | House/interior textures (siding, brick, roof, stone, floors, carpet, wallpaper, trim, door, lawn) from Poly Haven + ambientCG 1K-JPG downloads: greyscale detail maps, siding stagger, orientation fixes, ARM packing |
| `cargo tex materials` | Writes `materials.json` in the current directory: PBR material + environment entries |
| `cargo tex clouds <out.png>` | Generates the procedural, tileable cloud layer (`textures/sky/clouds.png`) |
| `blender/` | The Blender-built glTF models, every model in the game: `cargo art <family>` builds a family script from `blender/families/` on the `blender/gamekit/` toolkit, exports its `.glb` files with their moving parts, writes its pack and runs the gate. See [blender/README.md](blender/README.md), the guide for people and AI agents |
| `cargo check-glb` | The gate for glTF models (`crates/tools/src/checkglb.rs`, no Blender): checks that every model the content draws has a `.glb`, and every `.glb` the manifests use against its entry and its content (origin on the floor and at its `centre`, the wall's face, the footprint, a top's `radius`, triangles and draw calls against the budgets, materials mapped to finishes, one-sided, no unused UVs, extensions the game reads, texture sizes) and exits 1 on an error. `--preview` also has Blender re-import each file and render it to `blender/previews/<family>/rt_<name>.png` |

Texture flow: download (`cargo tex ph`) → `cargo tex process` (`cargo tex house` for the house's) → copy into
`web/public/assets/textures/` → `cargo tex materials` → merge its entries into `web/public/assets/manifest.json` by hand
(they are few and stable). `cargo tex clouds` regenerates the sky's cloud layer.

To look at the models without starting a game, run `pnpm dev` and open `/gallery.html` (dev only): it draws models in
a grid with the game's loader, finishes and plant shader (`?filter=garden`, `?keys=model.tree,model.pine`, `?cols=`,
`?gap=`, `?pack=`; drag to orbit).

Blender flow (real glTF art): `cargo art <family>` (Blender 5.2 LTS; `--preview` renders
instead of exporting). Each family script in `blender/families/` builds its models on `blender/gamekit/`, exports `.glb`
files into `web/public/assets/models/<family>/` and writes its override pack (`packs/<pack>/manifest.json`, listed in the
base manifest's `packs`), which maps the file's materials onto the game's finishes. `tools/assets/optimize.mjs` makes the meshopt copy for the build. How to make a new object or moving
part: [blender/README.md](blender/README.md); the standard: [docs/design/objects.md](../../docs/design/objects.md).
