# Blender models

The game's objects are built here: Python scripts that build each model in Blender 5.2 LTS and export
it as glTF (`.glb`), with its moving parts and its entry in an asset pack. The rules (sizes, budgets,
finishes, moving parts, what to look at) are in [docs/design/objects.md](../../../docs/design/objects.md).
This page covers the folder and the steps.

```
build.py          the runner (plain python3): runs Blender on a family, then the gate
check_glb.py      the gate (plain python3): every .glb the manifests use, against its entry
gamekit/          the toolkit every family imports as G
families/         one script per family of models; _template.py is the one to copy
previews/         renders from --preview (ignored by git)
```

## Commands

```sh
python3 -B tools/art/blender/build.py kitchen                  # build, export, register, check
python3 -B tools/art/blender/build.py kitchen --only fridge    # one model
python3 -B tools/art/blender/build.py garden_decor --preview   # previews/<id>.png (+ <id>@<clip>.png), no export
python3 -B tools/art/blender/build.py --all                    # every family
python3 -I -B tools/art/blender/check_glb.py [model.fridge …]  # the gate alone
```

`build.py` finds Blender in `$BLENDER`, then `/Applications/Blender.app`, then `PATH`. It exits 1 if a build
or the gate fails.

Exports aren't byte-for-byte repeatable: Blender writes the same triangles in a different order each run.
Commit only the models you meant to change, and `git checkout` the others.

## A new object

1. **Copy** `families/_template.py` to `families/<family>.py`, or add a model to an existing family.
   Families group what belongs together (`kitchen`, `garden_decor`). A family's models go to
   `web/public/assets/models/<family>/<id>.glb` and its entries to `packs/<pack>/manifest.json`.
2. **Palette.** One material per colour, named for what it is, with its finish:
   `M = G.Palette(body=('#F3F3F1', 'gloss'), handle=('#D0D3D6', 'chrome'))`.
   - The names become the material names in the file.
   - The finishes become the pack's `materials`, so they're declared only once.
   - Each finish is one of the base manifest's `material.finish.*`.
3. **Build** in a function, in metres, front −Y, from `G.cube`/`sphere`/`cylinder`/`cone`/`torus`/`rod`/
   `revolve`/`rock`. To copy numbers from a placeholder, use `G.GameFrame(shift)`: it takes game
   coordinates (x, height, depth) as they are. End with:
   - `G.join_all('<id>')` for a model with no moving parts;
   - one `G.join` per part for a model with moving parts (below);
   - then `G.settle()`, which stands it on the floor, centred.
4. **Declare it** in the family: `G.Model('<id>', build, size=(w, h, d), key='model.<id>', paint=['body'])`.
   - `size` is asserted after the build: the real size, never stretched.
   - `key` is only needed when it isn't `model.<family>.<id>`.
   - A key the manifests don't have yet needs a `placeholder`: a few primitive parts, the fallback.
5. **Look**: `build.py <family> --preview`, then open `previews/<id>.png`.
6. **Export**: `build.py <family>`. It writes the `.glb` files and the pack, then runs `check_glb.py`.
   Then run the Khronos validator and check in the game, as in objects.md's "Validation".
7. Add the object to `web/public/assets/CREDITS.md`, and update `families/<family>.md` if the family has notes.

## A moving part (door, lid, drawer)

```python
with G.new_parts() as parts:                         # everything made in the block
    F.box('door_fridge', 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel=0.01)
    _handle(...)
door = G.hinged(G.join(parts, 'door_fridge'), F.at(0.364, 0.643, 0.195))   # origin on the hinge
...
G.settle()                                           # before the clips
G.swing(door, 'fridge_open', -102, seconds=1.75)     # turn about its own Z (axis='X' for a lid)
G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=0.8)   # pull out to the front
```

The parts work like this:
- **Nodes.** Each moving part is its own node, named for what it is. The still part is one more node,
  joined with `origin=(0, 0, 0)`.
- **Clips.** A clip is an NLA track. Its first key (frame 1) is the rest pose, which is where the game
  draws the part closed.
- **Uses.** The model's `motions` says which use plays which clip, and for how long:
  `motions={'uses': {'fetch': 'fridge_open'}, 'hold': 3.5}`.
- **Checks.** The build fails if `motions` names a clip the model doesn't have. `check_glb.py` warns
  of a clip that no use plays.
- **Previews.** `--preview` renders each clip open on its own: `previews/<id>@<clip>.png`.
- **Examples.** `families/kitchen.py` has two doors. `families/_template.py` has a lid and a drawer.

## The toolkit (`gamekit/`)

| Module | What |
|---|---|
| `scene.py` | `reset`, `meshes`, the paths (`ASSETS`, `PREVIEWS`), `cosd`/`sind`; warns off Blender 5.2 |
| `materials.py` | `Palette` (colour + finish), `mat` (one material by hand), `FINISHES` (read from the base manifest) |
| `shapes.py` | the primitives: `cube`, `sphere`, `cylinder`, `cone`, `torus`, `rod`, `revolve`, `rock` |
| `place.py` | `GameFrame`: parts placed in game coordinates (`at`, `box`, `cyl`, `cone`, `sphere`) |
| `parts.py` | `new_parts`, `join`, `join_all`, `set_origin`, `hinged`, `settle`, `measure`, `game_size`, `check_size`; `fit` (third-party files only: it stretches) |
| `motion.py` | `swing`, `slide`, `clips` |
| `export.py` | `export_glb` (the fixed export settings), `write_pack`, `stats`, `materials_used` |
| `preview.py` | `preview` (front three-quarter, Workbench), `preview_clips` |
| `family.py` | `Model`, `Family` and its `run`: the command line every family script ends with |

Run Blender from scripts only (`--background --factory-startup`, as `build.py` does), so the export settings
stay fixed.
