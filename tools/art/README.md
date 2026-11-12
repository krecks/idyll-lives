# Art pipeline

The tools that produced the current textures and materials (`cargo tex`, in Rust: `crates/tools/src/tex.rs`), and
the Blender pipeline that builds every model. They are only needed to regenerate or change the art; the results
already live in `web/public/assets/` (see `CREDITS.md` there for sources and licences — all CC0). Models come from
Blender alone (`blender/families/`); the primitive-part scripts that came before are gone.

Run them from the repo root. The textures are generated, not downloaded.

| Command | Purpose |
|---|---|
| `cargo tex synth [set…]` | Generate every `albedo/normal/arm` texture set (20 sets: bark, brick, floors, foliage, grass, paving, roof, sidewalk, walls and wood) from tileable height fields; no downloads |
| `cargo tex materials` | Writes `materials.json` in the current directory: PBR material + environment entries |
| `cargo tex clouds <out.png>` | Generates the procedural, tileable cloud layer (`textures/sky/clouds.png`) |
| `blender/` | The Blender-built glTF models, every model in the game: `cargo art <family>` builds a family script from `blender/families/` on the `blender/gamekit/` toolkit, exports its `.glb` files with their moving parts, writes its pack and runs the gate. See [blender/README.md](blender/README.md), the guide for people and AI agents |
| `cargo art --leafcards` | Renders the trees' leaf-card atlas (`textures/leafcards/albedo.png`: three cut-out clusters of leaves and a dense wall of them) in Blender from leaves `blender/leafcards.py` generates; the `leafcards` finish every broadleaved tree's crown is cut from (`blender/families/trees.md`) |
| `cargo check-glb` | The gate for glTF models (`crates/tools/src/checkglb.rs`, no Blender): checks that every model the content draws has a `.glb`, and every `.glb` the manifests use against its entry and its content (origin on the floor and at its `centre`, the wall's face, the footprint, a top's `radius`, triangles and draw calls against the budgets, materials mapped to finishes, one-sided, no unused UVs, extensions the game reads, texture sizes) and exits 1 on an error ("The gate" below). `--preview` also has Blender re-import each file and render it to `blender/previews/<family>/rt_<name>.png` |

## The gate

`cargo check-glb` (and `cargo art`, which runs it) prints `ok` or `FAIL` per model, then its errors, warnings and
accepted findings, and exits 1 if any model fails. `tools/ci-local.sh models` runs it, and `tools` runs it again in
`ll-tools`' tests: the tree has 0 errors.

**Strict checks** have one right answer, so they are errors, not warnings that pile up unread
(`checkglb::STRICT`, by name):

| Name | Fails when |
|---|---|
| `radius` | a thing on a top reaches over 10 % past its content `radius` (it overlaps what stands beside it) |
| `footprint` | indoors, it reaches over 2 cm past its footprint (or its content `overhang`) |
| `sweep` | a part a loop or ambient clip spins sweeps over 1 cm past its footprint |
| `wall` | its back reaches over 1 cm into the wall |
| `thin` | on the floor, it or a moving part is lower than the floor drawn 1 cm up (not a held `tool.*`) |
| `float` | a floor thing floats above its origin (one on a wall or the ceiling hangs at its height) |
| `centre` | a Blender-built model is over 2 cm off its `Model`'s `centre` |
| `flipped` | faces over 1 cm² have normals turned against their winding (lit as if from behind) |
| `flat` | a part flattened at rest shows its flat shape (Blender README trap "Grown from nothing") |
| `unplayed` | a clip plays for no use |
| `cubic` | a clip has cubic-spline keys (played as straight lines) |
| `cameras`, `lights` | the file has cameras or lights (the game drops them) |
| `mirror` | mirror glass is see-through or isn't flat |
| `materials` | materials unmapped, double-sided, see-through (BLEND), glowing by day, or with UVs nothing reads |
| `far` | a far version isn't the full model's size |

**Accepting one.** A model that is right as it is takes `accept` in its pack entry: `{"<name>": "<why>"}`, set on its
`Model` (`G.Model(..., accept={'footprint': 'why'})`, which writes it), or in the entry of a pack written by hand. The
gate then prints the finding as `accepted: … (<name>: <why>)`. An `accept` that names no strict check, gives no
reason, or that the model no longer needs is an error, so stale ones don't hide the next finding. The curtains take
`footprint`: the rail runs past the window so the open curtains stack beside the glass (`blender/families/decor.md`).

**Warnings** are what only an eye can judge, and the budgets' aims: a loop that ends turned (a fan a quarter round)
or moved at an even speed (a belt's slats: right if it looks the same), a part turning over 12° in 1/60 s, a use's
clip over 3 s, over 3 draw calls or the triangle target, a far version over half the full model's triangles, a
texture of its own, and the functional heights against the content (`docs/design/objects.md`). Read each one.

Texture flow: `cargo tex synth` writes the 19 texture sets into `web/public/assets/textures/` and the sidewalk into
`web/public/assets/street/sidewalk/` (fixed filenames and resolutions, so the manifest's material bindings stay the
same). The one texture it doesn't make is the trees' leaf-card atlas, rendered by `cargo art --leafcards`. `cargo tex materials` writes the PBR material entries; `cargo tex clouds` regenerates the sky's cloud layer.
All three are deterministic: the same source makes the same output.

To look at the models without starting a game, run `bun run dev` and open `/gallery.html` (dev only): it draws models in
a grid with the game's loader, finishes and plant shader (`?filter=garden`, `?keys=model.tree,model.pine`, `?cols=`,
`?gap=`, `?pack=`; drag to orbit).

Blender flow (real glTF art): `cargo art <family>` (Blender 5.2 LTS; `--preview` renders
instead of exporting). Each family script in `blender/families/` builds its models on `blender/gamekit/`, exports `.glb`
files into `web/public/assets/models/<family>/` and writes its override pack (`packs/<pack>/manifest.json`, listed in the
base manifest's `packs`), which maps the file's materials onto the game's finishes. `tools/assets/optimize.mjs` makes the meshopt copy for the build. How to make a new object or moving
part: [blender/README.md](blender/README.md); the standard: [docs/design/objects.md](../../docs/design/objects.md).
