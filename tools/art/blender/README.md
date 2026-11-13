# Blender models

The game's objects are built here. Python scripts build each model in Blender 5.2 LTS and export it as
glTF (`.glb`), with its moving parts and its entry in an asset pack. This page is the guide for people
and AI agents alike: the folder, the steps, the checks and the traps. The rules (sizes, budgets,
finishes, moving parts, what to look at) are in [docs/design/objects.md](../../../docs/design/objects.md).
The content fields are in [docs/content-packs.md](../../../docs/content-packs.md).

```
centres.json      where each model built here is centred at rest, for the gate only (the build writes it)
leafcards.py      renders the trees' leaf-card atlas (`cargo art --leafcards`)
gamekit/          the toolkit every family imports as G
families/         one script per family: garden_decor, kitchen (the fridge's doors), living (the wall
                  bed that folds down); _template.py (a chest with a lid and a drawer) is the one to copy
previews/         renders from --preview, a folder per family (ignored by git)
```

The runner and the gate are Rust (`crates/tools`): `cargo art` runs Blender on a family, then the gate;
`cargo check-glb` (no Blender) checks every .glb the manifests use against its entry. The Python left
here runs inside Blender (`gamekit/manifest.py` is plain Python, no bpy).

## Working as an agent: the loop

1. **Read** this page and the family script closest to what you're making. Then read objects.md's
   sections for what you touch.
2. **Find the facts the game already has.** For an object that exists:
   - its content entry (`grep -rn '"id": "<id>"' web/public/content`);
   - its model (`web/public/assets/models/<family>/<id>.glb`), whose size and heights it keeps.

   For a new object, decide the footprint first; it fixes where the game puts residents (below).
3. **Build and look.** Run `cargo art <family> --preview`, then open the PNGs in `previews/<family>/` (you can read
   images). Iterate until it looks right from the front and each clip looks right open.
4. **Export.** Run `cargo art <family>`, which exports, writes the pack and runs the gate
   (`cargo check-glb`). It must say `0 failed`; read every warning. A model that is right to fail a strict
   check takes `accept={'<check>': '<why>'}` on its `Model` (tools/art/README.md, "The gate").
5. **Validate.** Install the Khronos validator in a scratch directory (`npm i gltf-validator`), outside
   the repo, and run `validateBytes` on the file: 0 errors, 0 warnings.
6. **Wire it in** outside this folder when the object is new (below). Run
   `cargo test -p sim-core --test sim shipped_content::`, and `cargo test -p sim-core` if you touched the sim.
7. **Look in the game.** Start your own dev server and drive it headless (below). Check WebGPU and
   `&renderer=webgl`, at rest and with each clip playing.
8. **Finish up.**
   - Add a line to `web/public/assets/CREDITS.md`.
   - Commit only the `.glb` files you meant to change. Re-exports aren't byte-for-byte repeatable, so
     `git checkout` the rest.
   - Never commit `previews/`.

Done means: the gate passes, the validator is clean, the tests pass, and you've seen it right in the
game. Report what you checked, and anything you didn't.

## Commands

```sh
cargo art kitchen                  # build, export, register, check
cargo art kitchen --only fridge    # one model
cargo art garden_decor --preview   # previews/garden_decor/<id>.png (+ <id>@<clip>.png), no export
cargo art living --preview --lit   # and <id>.lit.png: EEVEE, sun and sky, the finishes' textures (judge detail here)
cargo art kitchen --preview --sheet  # and <id>.sheet.png: four sides, the baked shading, each clip at 0/25/50/75/100 %
cargo art kitchen --no-pack        # export without writing the pack
cargo art --all                    # every family (not _template)
cargo art _template --preview      # the template only previews
cargo art --centres                # centres.json from every family (runs Blender, builds nothing)
cargo art --leafcards              # the trees' leaf-card atlas (textures/leafcards/albedo.png), from leaves it generates
cargo check-glb [model.fridge …]   # the gate alone (no Blender)
cargo check-glb --keys             # the keys a build may write
cargo check-glb --preview          # the gate, then Blender renders each file re-imported (<family>/rt_<name>.png)
```

`centres.json` holds each model's `centre` (key → `[x, depth]`; `[0, 0]`, the middle, for most): where
`G.settle()` centred it, so the gate can tell a file that is off it (a stale export, a part moved). The
game doesn't read it, so it stays out of the packs it downloads. A build writes its models' lines.
`--centres` writes every family's (Blender runs `gamekit/centres.py`, which imports the family scripts
without building anything): run it after changing a `centre=` without rebuilding (the gate then warns
of each file off its new centre), or after adding or removing models. A key it doesn't list isn't built
here (a third-party file), and its centre isn't checked. Commit it with the models.

`cargo art` finds Blender in `$BLENDER`, then `/Applications/Blender.app`, then `PATH`. It prints the
`[gamekit]` lines (sizes, triangles, what was built) and, when a build fails, the Python traceback. It
exits 1 if a build or the gate fails. Run Blender only through it (or with `--background
--factory-startup --python-exit-code 1`), so the export settings stay fixed and errors aren't swallowed.

## Coordinates, cheat sheet

| | Game | Blender (in a family script) |
|---|---|---|
| Axes | x right, y up, z towards the front | x left, y towards the back, z up |
| A point | (x, height, depth) | (−x, −depth, height) |
| Front | +z | −y |
| Origin | middle of the footprint, on the floor | the same: `G.settle()` puts it there |

- **Game coordinates.** `G.GameFrame(shift)` takes game numbers as they are (`F.box(name, w, h, d, (x, y, z), M.colour)`).
  Use it to copy content heights. `shift` moves every depth forward (+z; a negative one back), to draw a thing
  built against the back of its tile centred on it.
- **Footprint.** `[w, d]` in whole 1 m tiles, before turning. A 2 × 3 object runs from −1 to +1 across and
  −1.5 to +1.5 deep.
- **Against a wall.** Its back is on the wall's face, 7 cm into the tile: at depth −0.43 in a 1-deep
  footprint, −1.43 in a 3-deep one. Built on the tile's edge, it sits inside the wall.
- **Residents** stand on the tile in front of the footprint to use it.
  - Sitting and lying, they are moved to the footprint's middle (slots spread across the width), or to
    each slot's own place and facing (content `places`: a picnic table's two benches, the ends of a
    ping-pong table). Build the seats where the places are.
  - Lying, the head is about 0.84 m behind that middle and the feet 0.86 m in front, whatever the
    footprint.
  - A bed's mattress is wherever its footprint's middle is. Its height is the renderer's
    `beds` table in `web/src/render/shared/characters/furniture.json` (default 0.5 m), not the model's.

## A new object

1. **Copy** `families/_template.py` to `families/<family>.py`, or add a model to an existing family.
   Families group what belongs together. A family's models go to
   `web/public/assets/models/<family>/<id>.glb` and its entries to `packs/<pack>/manifest.json`; the build
   writes both, so never edit a pack by hand. A pack entry keeps the `vary` of the entry it overrides; the
   build doesn't copy it.
2. **Palette.** One material per colour, named for what it is, with its finish:
   `M = G.Palette(body=('#F3F3F1', 'gloss'), handle=('#D0D3D6', 'chrome'))`.
   - The names become the material names in the file.
   - The finishes become the pack's `materials`, so they're declared only once.
   - The finishes are the base manifest's `material.finish.*`: matte, satin, gloss, metal and chrome
     (plain), plus fabric, wood, bark and foliage (textured: one more draw call each), leafcards (a tree's
     leaf cards, cut out of their atlas: `families/plants.py` `clad` puts them there) and glass.
   - Name the main part the player may colour `body`; list it first in `paint`, then any other parts
     worth colouring, each with what the player calls it. Leave details (drains, water, food) out.
   - Hardware (handles, knobs, pulls, latches, taps) takes a `metal` or `chrome` colour: chrome or
     steel, brass, or black iron as the piece wants. Coloured by the player it stays metal (tinted).
     On a still part with no other plain material it costs a draw call (objects.md, "Hardware is metal").
   - Mirror glass takes the side it reflects to: `glass=('#DCE6EA', 'gloss', 'front')`. Make it a flat
     piece, on the still body or one moving part (a cabinet's door); the game reflects the room in the
     nearest mirror on screen (two on High). Keep it `gloss`, as the mirrored parts always were: the others
     show their glass, and see-through `glass` would show what is behind it (a frame's backing, a door).
   - A lamp's shade or a screen that glows at night takes its glow's colour:
     `shade=('#F2E6CC', 'fabric', {'glow': '#FFD9A0'})`. Nothing glows by day.
   - A plain finish may shine more or less than the finish does: `marble=('#EDEAE4', 'gloss', {'roughness':
     0.28})` (polished stone, not the mirror gloss is), and `metallic` (0 or 1). It goes in the pack entry and
     the game keeps it per vertex, so it costs no draw call. Worktops, marble and fruit have their own.
   - Textured finishes (wood, fabric, bark, foliage) get UVs made for each part as it's joined (`gamekit/uv.py`):
     a box along its own longest sides (wood's grain, up its picture, runs along a leg, a rail, a table's
     length), round about its own z for cylinders, cones, lathes and level-ringed lofts, along its path for a
     sweep (bark up a branch). A part built in place at an angle keeps its grain; one built level and turned
     by `rot=` turns its grain with it.
3. **Build** in a function, in metres, from `G.cube`/`sphere`/`cylinder`/`cone`/`torus`/`rod`/`revolve`/`boulder`
   (sizes are full extents; `bevel` rounds edges) and the forms (`gamekit/forms.py`), which make what boxes can't:
   - `G.sweep` (`F.sweep`): a round tube or any profile along a path, curved through the points (`resolution`),
     tapering (`scale`): frames and rails in one piece, rolled arms, handles, cabriole legs, branches;
   - `G.loft` through `G.ring`s (ellipses to rounded squares): bottles, vases, lamp bases;
   - `G.extrude` (`F.extrude`): an outline with holes into a slab: a carved apron, a headboard, a grille;
     `G.rounded_rect` and `G.smooth_outline` draw outlines;
   - `G.soft_box` (`F.soft`): cushions, pillows, mattresses, duvets: rounded, puffed, pinched, tufted, rumpled, with
     piping (below, "Soft things");
   - `G.mirrored`, `G.cut` (a boolean), `G.jitter` (a hand-made wobble);
   - `G.kit`: whole parts more than one family uses (`kit.tube`, `kit.cabriole`, `kit.moulding`).

   **Soft things.** `G.soft_box` is a box with what stuffing and seams do to one, each a few centimetres at most (a
   detail under a centimetre doesn't show at the play camera, where a pixel is 4 mm at its nearest) and none of it
   past the box, so the size you give is the size it has:
   - `puff`: its faces crowned (a number: the two across its thin side; three: each pair's own);
   - `pinch`: its corners pulled in and thinned where the seams meet, its sides bowing out between them, as a boxed
     cushion's do. On by default, 0.6 of the puff: two cushions side by side part in a V at their corners;
   - `waist`: its sides drawn in between corners that stand out as ears: a pillow sewn flat and stuffed;
   - `sag=0.014`: a seat cushion's top sinking over its front third, where legs go over it; its middle and back, where
     the sitter's hips rest (the content's `seat` height), stay;
   - `overhang`: its sides drawn in between its seams, the top standing over the boxing;
   - `tufts=(across, along)`, `tuft` (how deep), `diamond` (rows staggered), `button` (a material: a covered button in
     each dent): on the face that looks up, or to the front for a back;
   - `rumple`, `rumple_size`, `seed`: bedding: its top sunk in soft folds lying askew, what hangs down its sides
     drawn in in flutes to a square hem; only down and in, so a duvet stays under the game's quilt (`calm`: smooth
     where a runner lies across it);
   - `piping`: a cord along the seams, following them;
   - `resting=True`: it lies on its underside or leans on its back: that face's middle is one flat piece, and its seam
     has no cord. A third fewer triangles: give it to every cushion, mattress and cover that lies on something.
   Its steps (`flat`, `round_segments`) are chosen to keep its rounded edges within 1.5 mm of their curve and its
   crowned faces within 2 mm; give them only to save triangles on something small. A seat cushion a clip sinks (`G.squash`) is its own part, as before
   (`families/living.py` `_cushion`, `_seat`; `bedroom.py` `_pillow`, `_duvet`).

   Round things take as many sides as keep their outline within a millimetre of the circle when the family gives
   no count (`G.sides(radius)`: a polygon's sides stand `r × (1 − cos(π / n))` in): a lathe (`G.revolve`) 24 up to
   12 cm in radius and more above (32 at 20 cm, 52 at half a metre, 64 at most), a swept tube 12 up to 3 cm,
   `kit.tube` 8 up to 13 mm; a cabriole leg has 12. A far version built at low detail (`G.far_of`) keeps the least.
   A count the family gives is kept as it is: `cargo review facets` lists what shows its flat sides (over 1.5 mm).

   Spend the triangles where they show (silhouettes, edges, what a hand touches) and keep hidden parts plain
   (`families/living.py` armchair and sofa, `kitchen.py` fridge, `trees.py` oak). End with:
   - `G.join_all('<id>')` for a model with no moving parts;
   - one `G.join` per part for a model with moving parts (below);
   - then `G.settle()`, which stands it on the floor, centred, by moving it, never scaling.

   A model that stands off the middle of its footprint (the wall bed, at the back of its 2 × 3 tiles) is
   built in place and uses `G.settle(centre=False)`, with `centre=(x, depth)` on its `Model`.
4. **Declare it**: `G.Model('<id>', build, size=(w, h, d), key='model.<id>', paint=[('body', 'Cabinet'), ('stone', 'Worktop')])`.
   - `size` is asserted after the build, within 2 % (`tolerance`). It's the real size, never stretched.
     Take it from the content, or from the `[gamekit]` line once the model is right.
   - `key` is only needed when it isn't `model.<family>.<id>`; an object's content names its `model`.
     Before it builds anything, the build refuses a key that no content object or item names and no
     manifest has (a misspelt `key=`). A model the game draws by code, not for a content object or
     item (a held `tool.*`, a `model.street.*` prop, the woods' rock), takes `code=True` so its first
     build isn't refused; once its pack has it, it builds either way.
   - `far=0.25` also writes `<id>.far.glb`, its far version: the model at rest, joined and decimated
     to a quarter of the triangles (`G.decimated`: vertices sitting on one another merged first, all but
     what stands within 1 cm of the floor or 3 mm of its box, so it keeps its feet and its size, and each
     face the decimation turned round turned back), drawn where it's only seen from afar (the town
     overview, the far woods). For what is placed by the hundred or seen far off: trees, hedges, street
     props, the farm's buildings.
     `far=build_function` builds one by hand instead. `--preview` renders `<id>.far.png`.
   - `far=G.far_of(build)` builds it by hand from the model's own build, run again at low detail: the
     build asks `G.low(full, reduced)` for each count, segment number or bevel, and `G.is_low()` for what
     a far version leaves out (a flower's centre; moving parts, joined into one still part). Better than
     decimating small, separate parts (flowers, leaves, lumps, twine), which turns them into spikes and
     lifts the model off the floor. Draw every random number the full build draws, so the far version's
     parts stand where the model's do (`families/plants.py`, `trees.py`). The gate fails a far version
     whose lowest point is more than 0.5 cm off the model's.
   - **Perches** (trees, shrubs, hedges): where the game's birds sit, the entry's `perches`. The build finds them
     on its own meshes (`G.perch.around`, lines in from outside; `G.perch.find`, where they meet the leaves, spread
     and kept) with random numbers of its own, so the meshes don't change. `G.settle` moves them with the
     model (`families/trees.py`, `plants.py` `perches`). Its `Model` says what they are on, the entry's `habitat`
     beside them: `habitat='tree'` (broadleaved and fruit trees), `'conifer'`, `'bush'` (shrubs) or `'hedge'`.
     `--preview` renders `<id>@perches.png` (and `.top`) with a red ball on each. A bird the game flies by code takes
     `bird={'length', 'wingspan', 'perchHeight', 'pitch'}` (`families/birds.py`, birds.md). The gate checks them all
     against the file.
5. **Look**: `cargo art <family> --preview`, then open `previews/<family>/<id>.png`. The preview looks at the front
   three-quarter, so sides facing away (the inside of side shelves) may not show; the game will. Once exported, the
   inspector (`bun run dev`, `/inspect.html?key=model.<id>`) shows it as the game draws it, from any side, with its
   footprint, clips and gate report, and rebuilds it each time `cargo art` writes it again.
6. **Export**: `cargo art <family>`.

### Wiring a new object into the game

- **Content.** Add an entry to a content file (`web/public/content/catalog2/<room>.json` for the catalogs).
  - Fields: `id`, `name`, `model`, `footprint`, `price`, `category`, `group` (a group of that category in
    `base.json` `buyCategories`), `slots`, `description`, `interactions`.
  - Each interaction needs an `anim` (one of `base.json` `animations`).
  - Copy an object like it (`bedDouble`, `sofaBed`).
- **Room kind.** Rooms are typed by their objects' interaction tags, the first matching `roomKinds` entry
  winning. Anything with a `sleep` tag makes its room a bedroom. Only `nap`, as on the sofa bed, leaves a
  living room a living room.
- **Opening and closing.** For something that opens and stays open (the wall bed), use the interaction
  fields `when` and `leaves`, and the model's `motions.open` (below).
- **Renderer tables** (`web/src/render/shared/characters/seats.ts`), keyed by object id:
  - `BEDS` (from `furniture.json`'s `beds`): mattress-top height, for anything residents lie on;
  - `BLANKET_BEDS`: whether sleepers get a blanket. Then don't model a full duvet: it shows doubled.
- **Tests.**
  - `cargo test -p sim-core --test sim shipped_content::` checks every object's and item's model has its file
    (and its `far` file) once the manifests are stacked (a pack left out of `packs` fails it), every
    interaction has an animation, and the categories and groups exist.
  - Changed the sim? Run `cargo test -p sim-core`, then `bun run wasm` in `web/` before testing in the
    browser.

## A moving part (door, lid, drawer, fold-down bed)

```python
with G.new_parts() as parts:                         # everything made in the block
    F.box('door_fridge', 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel=0.01)
    _handle(...)
door = G.hinged(G.join(parts, 'door_fridge'), F.at(0.364, 0.643, 0.195))   # origin on the hinge
...
G.settle()                                           # before the clips: they key the rest pose
G.swing(door, 'fridge_open', -102, seconds=1.75)     # turn about its own Z (axis='X' for a lid)
G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=0.8)   # pull out to the front
```

The clips gamekit makes:

| Clip | Does | For |
|---|---|---|
| `G.swing(part, clip, degrees, seconds, axis='Z', ease='smooth')` | turns from rest by `degrees`; `ease='back_out'` swings `overshoot` (4 %) past open and settles | doors (`back_out`), lids, a bed folding down |
| `G.slide(part, clip, (x, y, z), seconds, ease='smooth')` | moves from rest | drawers, a cover sliding off |
| `G.grow(part, clip, seconds, axis='Z')` | scales from how it was built (small, hidden) to full size | water, flames, washing on a line |
| `G.squash(part, clip, scale, seconds, back=False, split=0.5)` | scales from rest to `scale` (a number or Blender (x, y, z)); `back`: and home again, the turn `split` of the way through | a cushion sinking, a bag flattening, something hidden while held (0.001) |
| `G.spin(part, clip, turns, seconds, axis='Z')` | whole turns at an even speed, ending at rest | wheels, platters, fans, the train |
| `G.rock(part, clip, degrees, seconds, axis='X')` | out, back past rest and home, as a pendulum | rocking chairs, swings, the seesaw |
| `G.scroll(part, clip, (x, y, z), seconds)` | moves by one repeat of a pattern at an even speed; the game jumps it home unseen | a treadmill's belt slats, a conveyor |
| `G.turns(parts, clip, (x, y, z), seconds)` | the parts take turns moving out and back, ending at rest | a dance floor's tiles lifting in two sets |
| `G.path(part, clip, [(seconds, (x, y, z), (rx, ry, rz)), …], linear=False)` | moves and turns from rest through keys (offsets in metres, turns in degrees, Blender axes; the first key is rest at 0 s); `linear`: an even speed, for a loop | the pool rack lifted off and hung up, the robot vacuum's round |
| `G.stages(part, clip, picked, growing, planted, seconds, sink)` | a crop's scale at each growth stage, ripe at rest; `sink`: how far down into the soil at each | vegetables, fruit, a veggie bed |
| `G.wobble(part, clip, degrees, seconds, axis='X', swings=3, delay=0)` | knocked out by `degrees`, swinging back and forth smaller each time, home at rest | what hangs or stands loose beside a part that moves: a towel on a door, a pan on its hook |
| `G.keys(part, clip, prop, index, [(seconds, value[, ease]), …])` | keys one channel (`'location'`, `'rotation_euler'`, `'scale'`, Blender axis 0–2) at those times, the first at 0 s at rest; each key's ease to the next | a clip none of the others make: water rising after the tap, a koi surfacing |
| `G.attach(part, parent)` | (no clip) puts a part inside another, to move with it and by its own clips | a lever on a door, a label on a record |

`ease` is one of `G.EASES`: `'smooth'` (the default, slow out of each key and into the next), `'linear'`, `'out'`,
`'in'`, `'in_out'`, `'expo_out'`, `'back_out'` (past the end and back) and `'bounce_out'`. A use's clip plays back
to close, so its curve runs the other way round then: a door that overshoots opening opens a hair further, then
swings shut and stops. Overshoot only where there's room past open (a door, not a lid against a wall or a bed
onto the floor).

The first four go from rest to open: the game holds them open and plays them back. `spin` and `rock`
end where they start, so they also loop (`motions.loops`, `idle`), as does `turns`. `scroll` loops too: it ends one slat on,
so hide the part's ends under covers (the treadmill's hood and rear cap). Keep a spin under about a turn a
second (two at most): the game blends between samples 1/60 s apart (`cargo check-glb` warns past 12° in that). A part that looks the same turned (a fan's four
blades) can turn by less (a quarter) and loop.

A part that is easier to build open (a bed that folds down) is built in its open pose and `hinged`. It's
then turned to rest with `G.turn(part, -90, 'X')` and swung open again by the opposite angle.

How the parts work:
- **Nodes.** Each moving part is its own node, named for what it is. The still part is one more node,
  joined with `origin=(0, 0, 0)`. Each part costs a draw call (plus one per textured finish in it).
- **Parts inside parts.** `G.attach(part, parent)` (after `G.settle`, before the clips) puts a part inside
  another: it goes with the parent's clips and plays its own on top, about its own origin (the retro
  fridge's lever on its door, the jukebox's label spinning on the record that swings up). Clips of one
  part play together; for a swing and a spin, give each its own node this way.
- **Clips.** A clip is an NLA track. Its first key (frame 1) is the rest pose, which is where the game
  draws the part closed. Angles turn about the part's own axis: about Z, negative opens a door hinged on
  its right; about X, negative lifts a lid's front.
- **When they play** is the model's `motions`:
  - `{'uses': {'fetch': 'fridge_open'}, 'hold': 3.5}`: a use, by interaction id or animation tag, plays
    a clip for `hold` seconds (absent: until the use ends).
  - `{'uses': {'shower': [{'clip': 'door_open', 'hold': 1}, 'water_on']}}`: several clips for one use,
    each with its own `hold` (a door shut after 1 s while the water runs).
  - `{'clip': 'fridge_open', 'delay': 0.3}`: the clip starts 0.3 s into the use (its `hold` counts from
    then): the retro fridge's lever turns first, then the door swings.
  - `{'clip': 'swish', 'delay': 1.4, 'every': 3.2}`: the clip plays from rest to rest every 3.2 s while
    the use lasts, the first 1.4 s in (the hoops' nets after each shot). It must end where it starts. It is also
    the loop that waits: a washer's drum turning once its door has shut (`delay` 3.5, `every` 1.1 for its 1.2 s
    turn), the seated hobbies' wheels once the resident has sat down. Make `every` a hair under the clip's length
    so the rounds run on without a pause.
  - `{'clip': 'turn_down_upper', 'slot': 1}`: only for the resident in that slot (`bedBunk`: one duvet
    per bunk).
  - `{'clip': 'door_open', 'delay': 0.3, 'hold': 2.4, 'out': 1.6}`: the clip opens again for 1.6 s when
    the use ends (the phone box's door, opened for the caller to step in and shut behind them, letting
    them out again: `STEP_IN` in characters.ts times their steps to it).
  - `{'loops': {'relax': 'rocking'}}`: a clip that goes round while the use lasts, then finishes its
    round. It must end where it starts: make it with `G.rock`, `G.spin` or `G.turns` (or `G.scroll`, one repeat on).
  - `{'ride': 'rocking'}`: the residents sitting or lying on it move with that clip's (first) part: a
    rocking chair's seat, a swing. Join everything they sit on into that part.
  - A part the resident holds while using it (content `held`: a guitar, a barbell, a mop) is drawn in their
    hands by the game, in the part's own frame: give it its origin where it's held from (a guitar's foot, a
    bar's middle, a mop head's foot) and its axes as it's held (a guitar upright, face to the front). Turn it
    into place on the object as the node's rotation (`obj.rotation_euler`), not with `G.turn`, which bakes the
    turn into the mesh. Hide it while it's used with `G.squash(part, 'off_stand', 0.001, seconds=0.2)` in `uses`.
  - `{'idle': 'blades_turn'}`: a clip that goes round all the time near the camera (a fan, a pendulum), not
    on the Low preset. It too ends where it starts.
  - `{'night': 'close'}`: a clip held open from dusk to dawn (curtains drawn).
  - `{'clock': {'hour': 'hour_hand', 'minute': 'minute_hand', 'at': 610}}`: a clock's hands, each a
    whole turn (`G.spin`), held where the game's time has them, on every preset; `at` is what they show
    at rest, in minutes past twelve (`families/decor.py` `_clock_hands`).
  - `{'growth': 'grow'}`: a crop's clip, held where its growth stage is (content `grows`). Make it with
    `G.stages(part, 'grow', picked=…, growing=…, planted=…)`, a scale per stage (ripe is the rest
    pose): the fruit `0.001, 0.001, 0.001`, the plants `1, 0.6, 0.25`. Plants spread over a bed in one
    part shrink towards its middle; scale them in height only and sink them instead
    (`planted=(1, 1, 0.5), sink=(0, …, …)`), so each seedling comes up where its plant stands
    (`families/plants.py` `_crop`). A use's clip on a growing part plays on top of its stage (the veggie
    bed's plants rustle while tended, `families/pack_natureLover.py`).
  - `'open': 'bed_down'`: the clip is held open while the object is open (the sim's `open`, set by
    interactions with `leaves`; content `when` offers them).
  - Also list the uses that open it in `uses`, so the clip starts with the use.
- **Timing.** A clip starts when the use starts, once the resident has arrived (a bed folds down as they
  lie down), or its `delay` later. Keep a use's clips short, about 1 s: the gate warns of one over 3 s. Loops,
  ambient (`idle`), `night`, `open` and `growth` clips keep their own pace (a merry-go-round's round in
  3.5 s) and aren't checked; an ambient clip runs on game time and is off on Low. The clock is the
  residents' animation time. Clock hands are the exception: the game's time sets them, on every preset.
- **Checks.** The build fails if `motions` names a clip the model doesn't have. `cargo check-glb` fails
  a clip that nothing plays, a a `delay`, `hold`, `every`, `out` or `slot` that isn't one, and fails a loop
  (or `idle`, an `every` clip, a clock hand) that doesn't end where it starts; one that ends
  a whole fraction of a turn round (a four-bladed fan a quarter turn), or moved at an even speed (`G.scroll`), is
  a warning to check. A part a clip grows (darts, a pot thrown) is measured grown, not at rest; one flattened to
  nothing at rest warns if its flat shape shows (traps).
- **Previews.** `--preview` renders each clip open on its own: `previews/<family>/<id>@<clip>.png` (a `spin` half
  way round, a `rock` at its furthest). `--sheet` adds `<id>.sheet.png`: its four sides (front and back
  three-quarter, side, top) and its baked shading on the first row, then a row per clip at 0, 25, 50, 75 and 100 %
  of the way through (a door through the cabinet half way, a lid closing past rest). Previews draw faces from the
  front only, as the game does: one built inside out shows as a hole.
- **In the game's renderer.** The clips bench, `/clips.html?keys=model.<id>` (below), plays every clip
  of the model as the game does.
- **Examples.**
  - `families/kitchen.py`: two doors, played by uses.
  - `families/living.py`: a bed built open, folded to rest, and held down while open.
  - `families/_template.py`: a lid and a drawer.

## Checking in the game (headless)

`tools/bench/check.mjs` takes the usual pictures in one go, headless on the real GPU and at 1 % volume,
with its own dev server on a port you give it (options and the rules for ports and scratch folders in
[tools/bench/README.md](../../bench/README.md)). Pictures go to `bench-results/check-<time>/` or `--out`:

```sh
bun tools/bench/check.mjs gallery --port 5227 --keys model.<id> --cols 4
bun tools/bench/check.mjs clips --port 5227 --keys model.<id> --times 0,0.3,0.6,1,99
bun tools/bench/check.mjs play --port 5227 --object <id> --at 23,14,0 --use <interaction> --shots 6
```

Add `--renderer webgl` for WebGL2 and `--quality low` for the Low preset. By hand, or for what the script
doesn't do, start your own dev server from your checkout (`cd web && bunx vite --port <free port> --strictPort`,
never one that's already running) and drive Playwright headless with full Chrome for Testing, at 1 % volume
(`page.addInitScript(quiet)` from `tools/bench/quiet.mjs`):

- **Inspector.** `/inspect.html?key=model.<id>` shows one model: `&clip=<name>&t=0.4` holds a clip there,
  `&show=wire,box,foot,radius,axes,scale,far` turns overlays on, `&alpha=…&beta=…&radius=…` move the camera (it
  frames the model without them). `window.__inspect` has `select(key)`, `set(t)`, `play()` and `solo(name)`.
- **Gallery.** `/gallery.html?keys=model.<id>&cols=4` shows the models from the front right, as close as
  frames them, at their middle height (so things on a wall are in view). `&alpha=…&beta=…&radius=…` and
  `&target=1.4` (the height looked at; `&target=x,y,z` a point) move the camera. `&scale=adult` (or
  `child`, `all`) stands residents beside each model for scale; `&camera=game` looks from as far as
  players do (the game's Live camera).
- **Clips bench.** `/clips.html?keys=model.<id>` plays every clip of the model in turn, as the game plays
  it: a use's clip opens, holds and closes; a loop goes round; an ambient clip plays all the time. A
  button per clip plays only that one (`&clip=<name>`); `&t=0.5` holds every clip at 0.5 s for a
  screenshot (`&t=99`: open). `&renderer=webgl` for WebGL2, `&colors=body:%23hex` for colours, `&target=`
  as in the gallery (things that all hang on a wall are aimed at by default). Without `keys` it shows
  every model with `motions`.
- **A house.** Open `/?state=bathroom-check&debug`: the house with resident 0 at home, doing nothing,
  needs at 0.8 (a use that fills a need lasts), plenty of money. `needy-check` has low needs and free
  will on, `office-night` is the same house at 21:30, `town-things` a town lot with every community
  thing (`__showPlot(1)`). Make them with `DUMP_STATES=1 cargo test -p sim-core --test sim check_states::`
  (`debug-states/`, gitignored; `tools/bench/README.md` says more).
  - Wait until `__sims()` answers and the strip shows "House 0" (not "Drawing portraits…").
  - Turn free will off: `__send({type: 'setAutonomy', enabled: false, household: 0})` (the game's own
    Free will setting is applied as the page opens, over the state's).
  - Buy with `__send({type: 'buy', household: 0, object: '<id>', at: [x, z, rot]})`. `[x, z]` is the
    footprint's corner, against a wall at rot 0. The living room has free wall at `[23, 14, rot]` and
    `[24, 17, rot]`. Find it with `__world().objects`. Out of money: `__addFunds(100000)`.
  - Use it with `__send({type: 'use', sim: 0, object: <id>, interaction: <index>})` (the ids:
    `__content.object('<id>').interactions`).
  - Wait for the use, don't time it: `__sims()[0]` has `object`, `def`, `interaction`, `using` (at it, not
    walking), `pose` (`stand`/`sit`/`lie`), `action`, `x`, `z` and `needs`.
  - See a clip play with `__renderer.motion.states.get(<id>)` (each clip's name, time and kind).
  - Frame it: `__controls.follow(0, {radius: 6, beta: 0.9, y: 1.4})` follows the resident from that
    distance, angle and height; `__camera({alpha, beta, radius, target: [x, y, z]})` puts the camera
    anywhere (`__camera()` reads it). Either stops the camera watching on its own.
  - `{type: 'setSpeed', speed: 0..5}` pauses or speeds up time.
  - Click the object to see its pie menu.
  - `Escape` closes a menu, but opens the pause menu when none is open.
- **Renderers.** Add `&renderer=webgl` to check WebGL2 as well as WebGPU.

## Traps (each of these happened)

- **Wall face.** A model built on the tile's back edge sits inside the house wall. Wall faces are 7 cm in.
  The gate fails a model (indoors, or on the wall layer) whose back reaches over 1 cm past the face.
- **Against a wall, on the floor.** Centred on its tile, a counter or a fridge stands 3–13 cm off the wall
  and off its content `surface`. Give its `Model` a `centre` (`G.settle()` centres it there), as the kitchen
  families do: the back of a counter or a fridge is 0.43 m in from the middle of its tile. A failed size or
  centre check prints the measured centre, to copy into `centre=`. The gate fails a file over 2 cm off
  its `centre` (`centres.json`). No content fact says a thing stands against a wall (a counter and an
  island have the same top; `fillsWall` only keeps the wall over it clear), so the gate can't tell one
  standing off it: check by eye.
- **Thin on the floor.** A house's floor is drawn 1 cm up and hides anything lower: a mat or a belt must
  reach 1.2 cm. The gate fails a floor model, or a moving part, whose top is lower (not a held tool,
  `tool.*`, which is only ever drawn in a hand).
- **Grown from nothing.** A part `G.grow`n from 0.001 along one axis keeps its other two, so at rest it
  lies flat where its origin is, and shows there: the dartboard's flights as crosses on its face, a flask's
  liquid as a disc through the glass. Put its origin inside something opaque (the board, the bench top),
  so it flattens out of sight. The gate fails a flat shape 3 cm across or more that shows.
- **Settle first.** Call `settle()` before `swing`/`slide`: clips key the rest pose where it is. It
  refuses after.
- **Size check.** It counts everything at rest: knobs, handles, a folded bed's foot rail. Set `size` from
  the measured `[gamekit]` line once the model looks right.
- **Off-centre models.** These need `centre=` on the `Model`. Otherwise the size check fails.
- **Lopsided turning parts.** `G.settle()` centres the model's box, which moves a part that goes round
  off the middle when it reaches farther one way (five fan blades: 3.3 cm), and it then sweeps past its
  tile. Build it with its axis on the middle, `G.settle(centre=False)`, and give the `Model` the box's
  middle as its `centre` (`families/lighting.py` `ceilingFan`). The gate fails a part a loop or ambient
  clip spins (G.spin) whose circle reaches over 1 cm past the footprint, indoors.
- **Clicking.** An object is clicked by its drawn outline at rest, never past its tiles. An object held open
  is clicked by its whole tiles. The contact shadow is the model's own shape at rest, seen from above: darkest
  where it touches the floor, lighter the higher a part is over it (`web/src/render/shared/contactShadow.ts`).
- **Doubled covers.** A per-sleeper blanket (`BLANKET_BEDS`) and a modelled duvet show double.
- **Room kind.** A `sleep` interaction turns a living room into a bedroom.
- **Mirrors past the nearest.** Only the nearest mirror on screen reflects (two on High). Mirror glass
  mapped to see-through `glass` showed the brass behind a round mirror and the door behind a cabinet's:
  map it to `gloss`.
- **Exports.** They differ on every run, so commit only what you meant to change.
- **Curves that overshoot.** `sweep` (and `kit.tube`) with `resolution` over 1 curves through the points
  (Catmull-Rom), which bulges past them where a long straight meets a short leg: a shower frame 12 cm too
  big, a range's rail 2 cm past its posts. For bent tube and corners, round the corners with `kit.fillet`
  and sweep at `resolution=1`; keep `resolution` for paths that are curves all along.
- **Shading by corners.** The baked shading is per vertex, so a face is as dark as its corners: a big flat
  face whose corners sit in a crease (a recessed panel, glass in a rim) goes dark all over. Corners buried
  inside another part are left unshaded; give a face that meets a crease an edge band (`kit.shaker` does).
  (Shading each corner along its own face's normal was tried: no better in the game, and it streaked the long
  triangles of an extruded frame.)
- **Smooth, sharp and bevelled.** Smooth parts keep edges over `G.SHARP` (40°) sharp, so a cylinder's cap stays
  flat (it used to shade into the side); round things with few sides stay round (`G.round_angle`: a 6-sided
  leg, an 8-sided tube); a lathe (`G.revolve`) keeps only turns over 70° sharp (a jar's coarse shoulder stays
  round); spheres, rings and boulders are smooth all over. A `bevel` is smooth with hardened normals: the faces
  it joins stay flat, the edge rounds. `G.shade(obj, smooth, angle)` and `G.bevel_edges(obj, width)` do the same
  to a mesh made by hand. (Raising `SHARP` to 50° rounded a 45° chamfer's steps; the bevel's `MITER_ARC` turned
  the normals of a frame's inner corners round: both showed a shaker frame's corners as seams.)
- **`up` is Blender's.** `F.sweep` maps the points to game axes but passes `up` (which way a profile's first
  side faces) as it is, in Blender axes (x left, y back, z up): `up=(0, 1, 0)` on a path along game x flipped a
  plinth's profile down through the floor.
- **Measuring parts in the making.** `with G.new_parts() as parts:` fills `parts` when the block ends: measure
  them after it, not inside.
- **Blender errors.** Without `--python-exit-code 1`, Blender exits 0 on a Python error. `cargo art`
  passes it.

## The toolkit (`gamekit/`)

| Module | What |
|---|---|
| `scene.py` | `reset`, `meshes`, the paths (`ASSETS`, `PREVIEWS`), `cosd`/`sind`; warns off Blender 5.2 |
| `materials.py` | `Palette` (colour + finish), `mat` (one material by hand), `FINISHES` (read from the base manifest) |
| `shapes.py` | the primitives: `cube`, `sphere`, `cylinder`, `cone`, `torus`, `rod`, `revolve`, `boulder` |
| `forms.py` | what primitives can't make: `sweep`, `loft`/`ring`, `extrude` (with holes), `soft_box`, `rounded_rect`, `smooth_outline`, `mirrored`, `cut`, `jitter` |
| `kit.py` | parts several families use, in game axes: `tube`, `cabriole`; `pull` (a bar pull on two stand-offs), `knob` (a turned knob), `shaker` (a frame-and-panel door or drawer front); `fillet` (a path's corners rounded, no overshoot); `songbird` (a perched robin, tit or sparrow: lofted body, beak, tail, legs; `head` tips it to drink); `moulding` (a profile run round an outline, its corners mitred: picture frames, panel beads, plinths, cornices); `scroll` (a flat iron scroll curling in, tapering: bench arms, brackets, a table's spider); `bench_end` (a cast-iron bench end in one piece: legs, upright, seat rail, arm with its scroll, stretcher, feet) |
| `place.py` | `GameFrame`: parts placed in game coordinates (`at`, `box`, `cyl`, `cone`, `sphere`); `depths=` redraws a thing's numbers at another depth (the kitchen counters' one depth) |
| `parts.py` | `new_parts`, `join`, `join_all`, `set_origin`, `hinged`, `turn`, `settle`, `measure`, `game_size`, `check_size`; `fit` (third-party files only: it stretches) |
| `motion.py` | the clips: `swing` (turn), `slide` (move), `grow` (appear from nothing), `squash` (scale), `spin` (whole turns home), `rock` (out, past rest and home), `turns` (parts out and back in turn), `scroll` (one repeat on, a belt), `path` (through keys), `stages` (a crop's growth); `clips` |
| `export.py` | `export_glb` (the fixed export settings, the shading baked first), `write_pack`, `stats`, `materials_used` |
| `shade.py` | `bake_ao`: the creases' shading baked into a grey vertex colour (COLOR_0), which the game multiplies in; the floor shades the foot of a model on the ground (`Model(ground=)`); a tree's leaf cards keep the shading they were laid with |
| `preview.py` | `preview` (front three-quarter, Workbench; or `view=` side, front, top), `lit` (EEVEE, as the game lights it: `--lit`), `preview_clips`, `preview_perches` |
| `family.py` | `Model`, `Family` and its `run`: the command line every family script ends with |
| `far.py` | `far_of` (a far version from the model's own build at low detail), `low`, `is_low` |
| `perch.py` | `G.perch`: where birds sit on a model, found on its meshes (`around`; `find`: `surface`, `bare`, `spread`, `keep`), the entry's `perches` |
| `manifest.py` | plain Python, no bpy: the manifests and content as the gate reads them (`manifests`, `known_keys`, `played`), and centres.json (`read_centres`, `write_centres`) |
| `review.py` | run by `cargo check-glb --preview`: re-imports each checked file and renders `previews/<family>/rt_<name>.png` |
| `centres.py` | run by `cargo art --centres`: imports every family (no build) and writes centres.json |
