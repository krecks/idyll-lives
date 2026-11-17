# Art pipeline

The tools that produced the current textures and materials (`cargo tex`, in Rust: `crates/tools/src/tex.rs`), and
the Blender pipeline that builds every model. They are only needed to regenerate or change the art; the results
already live in `web/public/assets/` (see `CREDITS.md` there for sources and licences — all CC0). Models come from
Blender alone (`blender/families/`); the primitive-part scripts that came before are gone.

Run them from the repo root. The textures are generated, not downloaded.

| Command | Purpose |
|---|---|
| `cargo tex synth [set…]` | Generate every `albedo/normal/arm` texture set (21 sets: bark, brick, floors, foliage, grass, paving, roof, sidewalk, walls and wood) from tileable height fields; no downloads |
| `cargo tex materials` | Writes `materials.json` in the current directory: PBR material + environment entries (the finishes in glTF's terms; refused when one is outside the ranges of `crates/tools/src/pbr.rs`) |
| `cargo tex clouds <out.png>` | Generates the procedural, tileable cloud layer (`textures/sky/clouds.png`) |
| `blender/` | The Blender-built glTF models, every model in the game: `cargo art <family>` builds a family script from `blender/families/` on the `blender/gamekit/` toolkit, exports its `.glb` files with their moving parts, writes its pack and runs the gate. See [blender/README.md](blender/README.md), the guide for people and AI agents |
| `cargo art --leafcards` | Renders the trees' leaf-card atlas (`textures/leafcards/albedo.png`: three cut-out clusters of leaves and a dense wall of them) in Blender from leaves `blender/leafcards.py` generates; the `leafcards` finish every broadleaved tree's crown is cut from (`blender/families/trees.md`) |
| `cargo check-glb` | The gate for glTF models (`crates/tools/src/checkglb.rs`, no Blender): checks that every model the content draws has a `.glb`, and every `.glb` the manifests use against its entry and its content (origin on the floor and at its `centre`, the wall's face, the footprint, a top's `radius`, triangles and draw calls against the budgets, materials mapped to finishes, one-sided, no unused UVs, extensions the game reads, texture sizes) and exits 1 on an error ("The gate" below). `--validate` also runs the Khronos glTF validator over each file and its far version (`tools/art/validate.ts`, under Bun: `bun install` in `web/` first). `--preview` also has Blender re-import each file and render it to `blender/previews/<family>/rt_<name>.png` |
| `cargo check-glb --against <rev\|dir> [keys… \| --family <f>] [--dir <built>]` | Whether a build of the models is the build of a git revision (or of a directory: a checkout, or an assets root) (`crates/tools/src/checkglb/against.rs`; it reads only the `.glb` files, the packs and `centres.json`, so it holds for any generator). Each model and far version of the working tree, or of `--dir` (a build of some models compares those), is `same` (the same nodes, materials, triangles and clips to the bit, in whatever order the file has them), `near` (every metric within its limit: origins, clips sampled at 60 Hz, perches, the surfaces' distance both ways, normals, texture, baked shading, triangles, no new finding of the gate; the limits open the file) or `CHANGED`, printed with its metric, part, value, limit and worst point; exits 1 on any. One that is right as it is takes a line with its reason in `port-accepted.json` (`{"<key>": {"<metric>": "<why>"}}`, a far version's key `<key> far`); one that is not needed is an error. It lists the far versions Blender's decimate made as `decimated`: no two builds of those are alike, so they are held to a rule of their own (size within 2 %, on the model's floor, at most a tenth more triangles, within 6 cm) |

| `cargo check-glb --built <dir>` | The same gate over what the game builds as it runs (houses, the street, the backdrop, fences, cars: "The built world" below), from the samples `bun tools/built/dump.ts <dir>` writes there with the game's own generators |

## The gate

`cargo check-glb` (and `cargo art`, which runs it) prints `ok` or `FAIL` per model, then its errors, warnings and
accepted findings, and exits 1 if any model fails. `tools/ci-local.sh models` runs it with `--validate`, and `tools`
runs it again in `ll-tools`' tests: the tree has 0 errors.

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
| `edges` | hard edges of 5 cm or more show unbevelled at the play camera (a pixel there is 4 mm): convex, turning 60° or more, split in the file (a bevel with hardened normals passes, and a chamfer), both faces 12 mm or more wide, on a piece of 10 cm or more; not an underside, on the floor, covered by another part or seen from none of the camera's views; not leaves, bark, glass or what glows, a far version or a held `tool.*` (`crates/tools/src/edges.rs`; `cargo review edges <key>` lists a model's with where they are). Counted against the baseline (below) |
| `texels` | a material on a textured finish (not the leaf cards) is laid outside 200 to 1600 px a metre (the play camera shows 250), stretched over 2× on more than 5 % of its area, or has UVs of no area on more than 1 % of it; not a far version (`crates/tools/src/texels.rs`; `cargo review texels` gives each finish as the models wear it). Counted against the baseline |
| `validator`, `reference`, `pbr` | counted against the baseline; `validator` with `--validate` (errors, warnings and unused objects but the finishes' UVs: `checkglb/validate.rs`), the others names kept for checks not in the gate yet |

**Accepting one.** A model that is right as it is takes `accept` in its pack entry: `{"<name>": "<why>"}`, set on its
`Model` (`G.Model(..., accept={'footprint': 'why'})`, which writes it), or in the entry of a pack written by hand. The
gate then prints the finding as `accepted: … (<name>: <why>)`. An `accept` that names no strict check, gives no
reason, or that the model no longer needs is an error, so stale ones don't hide the next finding (a counted
check's only where it ran). The curtains take
`footprint`: the rail runs past the window so the open curtains stack beside the glass (`blender/families/decor.md`).

**The baseline.** A strict check that came in after the models it finds (`checkglb::COUNTED`) counts its findings
per model against `gate-baseline.json`, `{"<check>": {"<model key>": <count>}}` (`checkglb/baseline.rs`). A model at
its count gets its findings as warnings; more, or any in a model the file doesn't have, fail as strict findings, so
new models are strict from the start. Fewer fail as stale until `cargo check-glb --update-baseline` lowers the
entry: it only lowers counts and takes keys out, never raises or adds, so the file only shrinks. A check with no
section is strict for every model. An entry that `accept`s a counted check takes its findings instead, and the
file may keep none for it. `reference` is `cargo review sizes --check`'s, by object id (its own `--update-baseline`).

**Warnings** are what only an eye can judge, and the budgets' aims: a loop that ends turned (a fan a quarter round)
or moved at an even speed (a belt's slats: right if it looks the same), a part turning over 12° in 1/60 s, a use's
clip over 3 s, over 3 draw calls or the triangle target, a far version over half the full model's triangles, a
texture of its own, and the functional heights against the content (`docs/design/objects.md`). Read each one.

## The built world

Houses, the neighbours' and the far houses, the backdrop's buildings, roads, sidewalks, junctions, bridges, fences and
cars are no model files: the game builds them as it runs (`web/src/render/shared`). They are held to the models' rules
all the same. `bun tools/built/dump.ts target/built` writes a fixed set of samples (`tools/built/samples.ts`: house
plans, each as the viewed house, as the neighbour's silhouette, as the street's far house and as the town view's box; a
backdrop building of each use, dressed and as its box; a street's block; each fence style; each car), a `.glb` a sample,
made by those generators with no browser, and the same bytes on every machine. `cargo check-glb --built target/built`
(`crates/tools/src/checkglb/built.rs`) reads them and counts, per sample, against the baseline's sections of these
names:

| Name | Counts |
|---|---|
| `built.edges` | the models' `edges`. A house is judged as the play camera sees it: whole with its roof on (it shows from 15 m out), and nearer, a storey at a time without roofs, with its walls standing and with them cut away each of the eight ways the cutaway stands, each by the views that make it stand so; an edge counts once however many of those show it |
| `built.texels` | the models' `texels`, over each part on a textured material (a material projected from above has x and z for UVs) |
| `built.farNear` | where a far version is off what it is the far version of: what its roofs cover by over 10 % or 1.5 m, its top by over 0.5 m, its roof's kind or its pitch by over 4°, a chimney on one of them only, most of its walls or of its roof another colour as drawn (the vertex colour times the material's and its texture's mean) by over 28 of 255 in a channel |

Far versions are judged by `built.farNear` alone. `tools/ci-local.sh built` runs both commands (about 15 s);
`--report` prints what was measured as tables (each sample's triangles and draw calls, where the edges are, how each
material's texture is laid, each far version's look beside its near one's, and what the houses have of bevels,
mouldings, gutters and cap shingles). A sample's file opens in any glTF viewer, in its vertex colours, as PlayCanvas
draws it (the game's z mirrored). `--update-baseline` lowers the sections; `--seed-baseline` writes one the file has
none of. A new sample, or a change to a generator, that brings findings fails until they are fixed, or until its
section is taken out and seeded again with the reason in the commit.

Texture flow: `cargo tex synth` writes the 20 texture sets into `web/public/assets/textures/` and the sidewalk into
`web/public/assets/street/sidewalk/` (fixed filenames and resolutions, so the manifest's material bindings stay the
same). The one texture it doesn't make is the trees' leaf-card atlas, rendered by `cargo art --leafcards`. `cargo tex materials` writes the PBR material entries; `cargo tex clouds` regenerates the sky's cloud layer.
All three are deterministic: the same source makes the same output. The tools' tests hold the committed sets, under the
manifest's materials, to a physically based standard (`crates/tools/src/tex/synth/standard.rs`: albedo, roughness,
metalness, normals, ambient occlusion and texels a metre, with the breaks known today listed there), and
`cargo review sizes` their patterns (a brick, a board, a tile) and the kerb to real sizes (`surfaces` in
`docs/review/reference.json`).

To look at the models without starting a game, run `bun run dev` and open `/gallery.html` (dev only): it draws models in
a grid with the game's loader, finishes and plant shader (`?filter=garden`, `?keys=model.tree,model.pine`, `?cols=`,
`?gap=`, `?pack=`; drag to orbit). To look at one model closely, open `/inspect.html?key=model.<id>` (or
the main menu's "Inspect models", under Settings on the dev server): every model by
family on the left; its content footprint, the wall's face, a top's disc, its box, axes, wireframe and a 1.75 m figure
over it; its size, triangles, draw calls and meshes' finishes, and the gate's report on it (`cargo check-glb <key>`, a
button) on the right; and a timeline that plays its clips as the game does, or scrubs one. When `cargo art` writes its
file again the inspector rebuilds it in place, so it can stay open beside Blender; a `.glb` dropped onto the page is shown
too (with its own materials). The keys and the URL options are in `web/src/debug/inspect.ts`.

Blender flow (real glTF art): `cargo art <family>` (Blender 5.2 LTS; `--preview` renders
instead of exporting). Each family script in `blender/families/` builds its models on `blender/gamekit/`, exports `.glb`
files into `web/public/assets/models/<family>/` and writes its override pack (`packs/<pack>/manifest.json`, listed in the
base manifest's `packs`), which maps the file's materials onto the game's finishes. `cargo assets` makes the quantised, meshopt-compressed copy for the build. How to make a new object or moving
part: [blender/README.md](blender/README.md); the standard: [docs/design/objects.md](../../docs/design/objects.md).
