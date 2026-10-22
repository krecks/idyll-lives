# A family: the models one script builds, and the loop every family script ends with. Each model is a
# build function and the facts the game needs; `Family.run` builds, checks and exports each one into
# `models/<family>/<id>.glb` and writes the family's pack entries. See families/_template.py.
import os
import sys

import bpy

from . import motion, parts, perch
from .export import export_glb, materials_used, stats, write_pack
from .manifest import known_keys, played, write_centres  # the keys, `motions` and centres as the gate reads them
from .parts import check_size, join
from .preview import lit, preview, preview_clips, preview_perches
from .scene import ASSETS, PREVIEWS, meshes, reset

# A decimated far version (`far=ratio`) keeps every vertex up to this height (m) over the model's lowest point.
FAR_FEET = 0.01


class Model:
    """
    One model.
      id         the file's name: models/<family>/<id>.glb
      build      a function that builds it in the empty scene (Blender axes, metres), standing on the
                 floor and centred on its footprint; it may return its objects (default: every mesh)
      size       (width, height, depth) in game metres, asserted within `tolerance` (fraction, or 3)
      key        the manifest key it draws for (default model.<family>.<id>)
      motions    the pack entry's `motions` for its clips (docs/design/objects.md, "Moving parts"):
                   'uses': {use: clip, or {'clip', 'hold'}, or a list of them}, played by a use (an
                           interaction id or animation tag), each back after the model's 'hold'
                           seconds or its own (absent: when the use ends);
                   'hold': seconds;
                   'loops': {use: clip} going round while the use lasts (G.spin, G.rock);
                   'open': clip held open while the object is open (content `leaves`);
                   'idle': clip going round all the time near the camera (G.spin, G.rock);
                   'night': clip held open from dusk to dawn (curtains drawn);
                   'growth': a crop's clip, held where its growth stage is (G.stages);
                   'ride': the clip whose (first) part carries the residents sitting or lying on it
                           (a rocking chair's seat, a swing), played by `loops` or `uses`
      paint      the parts the player may colour (the pack's `paint`): [(material, label)], the main part
                 first (the one the game's quick colour row paints), e.g. [('body', 'Cabinet'),
                 ('stone', 'Worktop')]; a bare material name is its own label
      parts      primitive parts drawn with the file (a mirror a file can't reflect yet)
      centre     (game x, depth) the model's bounds are centred on at rest, for one that stands off the
                 middle of its footprint (default: the middle); `G.settle()` centres it there. A floor thing
                 that stands against a wall (a counter, a fridge) takes (0, -0.43 + depth / 2), so its back is
                 on the wall face (-0.43) and it meets the wall and its neighbours, and its content `surface`
                 stays on its top: the fridge, 0.722 deep, takes (0, -0.069). Written into centres.json (the
                 middle too), for the gate only (`cargo check-glb`: centred there, and against the wall)
      floor      False for a thing on a wall: built at its real height, not stood on the floor
      sink       how far the model may reach below the floor (default 0), for the few the game sinks
                 (the woods' rock, the selection marker the game centres on a resident)
      code       True for a model the game draws by code, not for a content object or item (a held tool.*,
                 a model.street.* prop, the woods' rock): its key needn't be one the content names. The build
                 refuses any other key that no content object or item names and no manifest has (a misspelt `key=`)
      far        its far version, models/<family>/<id>.far.glb, drawn where it's only seen from afar (the
                 town overview, the far woods): a share of the triangles to keep (0.25: the model at rest,
                 joined, decimated but for what stands on the floor, without what's hidden at rest), or a
                 build function of its own (`G.far_of(build)`: the model's build again at low detail,
                 gamekit/far.py); the entry's `far`. For what is placed by the hundred or seen far off:
                 trees, street props.
      bird       the entry's `bird` for a bird the game flies about by code (families/birds.py): {'length': bill to tail
                 tip, 'wingspan', 'perchHeight': its body's middle over its feet, 'pitch': degrees its perched torso is raised from level},
                 metres; `cargo check-glb` checks them against the file
      habitat    what its perches are on, the entry's `habitat` beside them, so the game's birds needn't guess it from
                 the key: 'tree' (broadleaved and fruit trees), 'conifer', 'bush' (shrubs) or 'hedge'
    Perches the build finds (`gamekit/perch.py`: a tree's outer twigs) are written as the entry's `perches`.
    """

    def __init__(self, id, build, size, key=None, tolerance=0.02, motions=None, paint=None, parts=None,
                 centre=(0.0, 0.0), floor=True, far=None, sink=0.0, code=False, bird=None, habitat=None):
        self.id, self.build, self.size, self.key = id, build, size, key
        self.tolerance, self.motions, self.paint = tolerance, motions, paint
        self.parts = parts
        self.centre, self.floor, self.sink = centre, floor, sink
        self.far = far
        self.code = code
        self.bird = bird
        self.habitat = habitat


class Family:
    """
    name     the models' folder (models/<name>/) and the default key prefix (model.<name>.)
    palette  the family's G.Palette: the pack's `materials` come from it
    models   [Model]
    pack     the pack's folder (packs/<pack>/manifest.json), default `name`
    comment  the pack's $comment: what the family is and which script builds it
    """

    def __init__(self, name, palette, models, comment, pack=None):
        self.name, self.palette, self.models, self.comment = name, palette, models, comment
        self.pack = pack or name
        ids = [m.id for m in models]
        assert len(ids) == len(set(ids)), f'{name}: two models with one id'

    def key(self, model):
        return model.key or f'model.{self.name}.{model.id}'

    def make(self, model):
        """Builds one model in an empty scene and checks it; returns its objects."""
        reset()
        perch.clear()
        parts.CENTRE = model.centre
        try:
            objs = model.build() or meshes()
        finally:
            parts.CENTRE = (0.0, 0.0)
        bpy.context.scene.frame_set(1)
        check_size(model.size, model.tolerance, objs, name=model.id, centre=model.centre, floor=model.floor)
        stats(model.id)
        return objs

    def run(self, argv):
        """
        The command line, after Blender's `--`:
          (nothing)          build, check and export every model, then write the pack
          --only a,b         just these ids (the pack keeps the others' entries)
          --preview          render previews/<family>/<id>.png (and <id>@<clip>.png) instead of exporting
          --lit              with --preview: <id>.lit.png too, lit and textured as the game draws it
          --no-pack          export without writing the pack
        """
        args = argv[argv.index('--') + 1:] if '--' in argv else []
        only = None
        if '--only' in args:
            only = set(args[args.index('--only') + 1].split(','))
            unknown = only - {m.id for m in self.models}
            if unknown:
                sys.exit(f'{self.name}: no model {", ".join(sorted(unknown))}')
        models = [m for m in self.models if only is None or m.id in only]
        # A key the game wouldn't draw is a misspelt `key=`: refused before any model is built or exported, so
        # no file is written for an entry the pack then can't take. The keys it knows: the content's, the
        # manifests' and this pack's own (listed in `packs` or not), read once and handed to write_pack.
        known = None
        if '--preview' not in args:
            try:
                known = known_keys(ASSETS, os.path.join(ASSETS, 'packs', self.pack, 'manifest.json'))
            except ValueError as err:
                sys.exit(f'{self.name}: cannot read the content or the manifests: {err}')
            unknown = sorted(self.key(m) for m in models if not m.code and self.key(m) not in known)
            if unknown:
                sys.exit(f'{self.name}: no content object or item names {", ".join(unknown)} and no manifest has it: '
                         'a misspelt key=? (a model the game draws by code takes code=True)')
        entries, centres = {}, {}
        for model in models:
            objs = self.make(model)
            perches = perch.game()                         # before a far version is built
            if '--preview' in args:
                # By family: an id may be in two (bench, flowers, hedge ...), and one family's render would overwrite the other's.
                stem = os.path.join(PREVIEWS, self.name, model.id)
                os.makedirs(os.path.dirname(stem), exist_ok=True)
                preview(stem + '.png')
                if '--lit' in args:
                    lit(stem + '.lit.png')
                preview_clips(stem)
                print(f'[gamekit] previewed {self.key(model)} -> {stem}.png')
                if perches:
                    print(f'[gamekit] {model.id}: {len(perches)} perches')
                    preview_perches(stem + '@perches.png')
                if model.far:
                    self.make_far(model, objs)
                    preview(stem + '.far.png')
                continue
            clips = motion.clips(objs)
            path = export_glb(os.path.join(ASSETS, 'models', self.name, f'{model.id}.glb'), objs, clips=bool(clips))
            self._check_motions(model, clips)
            entry = {'url': f'../../models/{self.name}/{model.id}.glb',
                     'materials': self.palette.finishes(materials_used(objs))}
            if model.motions:
                entry['motions'] = model.motions
            if model.paint:
                entry['paint'] = paint_slots(model.paint, materials_used(objs))
            if model.parts:
                entry['parts'] = model.parts
            if model.sink:
                entry['sink'] = model.sink
            if model.bird:
                entry['bird'] = model.bird
            if perches:
                entry['perches'] = perches
                print(f'[gamekit] {model.id}: {len(perches)} perches')
            if model.habitat:
                assert perches, f'{model.id}: `habitat` says what its perches are on, and it has none'
                entry['habitat'] = model.habitat
            if model.far:
                far = export_glb(os.path.join(ASSETS, 'models', self.name, f'{model.id}.far.glb'), self.make_far(model, objs))
                entry['far'] = f'../../models/{self.name}/{model.id}.far.glb'
                print(f'[gamekit] built its far version -> {os.path.relpath(far, ASSETS)}')
            entries[self.key(model)] = entry
            centres[self.key(model)] = model.centre
            print(f'[gamekit] built {self.key(model)} -> {os.path.relpath(path, ASSETS)}')
        if centres:
            # Where each file is centred, for the gate only: not in the pack the game downloads.
            write_centres(centres)
        if entries and '--no-pack' not in args:
            path = write_pack(self.pack, entries, self.comment, code={self.key(m) for m in models if m.code}, known=known)
            print(f'[gamekit] wrote {os.path.relpath(path, ASSETS)}')

    def make_far(self, model, objs):
        """The far version of `model` (just built, its objects `objs`), in the scene: its objects."""
        if callable(model.far):
            reset()
            objs = model.far() or meshes()
            bpy.context.scene.frame_set(1)
            check_size(model.size, max(model.tolerance, 0.05), objs, name=f'{model.id}.far', centre=model.centre, floor=model.floor)
            stats(f'{model.id}.far')
            return objs
        # At rest, still, without what only shows while it moves (water grown from nothing).
        bpy.context.scene.frame_set(1)
        for o in objs:
            o.animation_data_clear()
        hidden = [o for o in objs if min(abs(k) for k in o.scale) < 0.01]
        for o in hidden:
            bpy.data.objects.remove(o, do_unlink=True)
        far = join([o for o in objs if o not in hidden], f'{model.id}_far', origin=(0, 0, 0))
        # What stands on the floor is left whole: decimating it lifted things off the ground (a tractor's
        # wheels 1.7 cm, a fountain's slab 4 cm) and pulled their feet in. The rest is decimated.
        bottom = min(v.co.z for v in far.data.vertices) if far.data.vertices else 0.0
        keep = far.vertex_groups.new(name='decimate')
        keep.add([v.index for v in far.data.vertices if v.co.z > bottom + FAR_FEET], 1.0, 'REPLACE')
        decimate = far.modifiers.new('far', 'DECIMATE')
        decimate.ratio = model.far
        decimate.vertex_group = 'decimate'
        mesh = far.evaluated_get(bpy.context.evaluated_depsgraph_get()).to_mesh()
        mesh.calc_loop_triangles()
        print(f'[gamekit] {model.id}.far: {len(mesh.loop_triangles)} triangles ({model.far:.0%} kept)')
        far.evaluated_get(bpy.context.evaluated_depsgraph_get()).to_mesh_clear()
        return [far]

    def _check_motions(self, model, clips):
        names = {name for _, name, _ in clips}
        missing = sorted(set(played(model.motions or {})) - names)
        assert not missing, f'{model.id}: `motions` plays {", ".join(missing)}, which it has no clip for'


def paint_slots(paint, used):
    """A model's `paint` as the pack writes it: [{'material', 'label'}], each a material the model uses."""
    slots = []
    for p in paint:
        material, label = (p, p.capitalize()) if isinstance(p, str) else p
        if material not in used:
            raise ValueError(f'paint: the model has no material {material!r} (it has {sorted(used)})')
        slots.append({'material': material, 'label': label})
    return slots
