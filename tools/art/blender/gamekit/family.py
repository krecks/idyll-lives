# A family: the models one script builds, and the loop every family script ends with. Each model is a
# build function and the facts the game needs; `Family.run` builds, checks and exports each one into
# `models/<family>/<id>.glb` and writes the family's pack entries. See families/_template.py.
import os
import sys

import bpy

from . import motion
from .export import export_glb, materials_used, stats, write_pack
from .parts import check_size
from .preview import preview, preview_clips
from .scene import ASSETS, PREVIEWS, meshes, reset


class Model:
    """
    One model.
      id         the file's name: models/<family>/<id>.glb
      build      a function that builds it in the empty scene (Blender axes, metres), standing on the
                 floor and centred on its footprint; it may return its objects (default: every mesh)
      size       (width, height, depth) in game metres, asserted within `tolerance` (fraction, or 3)
      key        the manifest key it draws for (default model.<family>.<id>)
      motions    the pack entry's `motions` for its clips: {'uses': {use: clip}, 'hold': seconds,
                 'open': clip held open while the object is open (content `leaves`)}
      paint      the materials a player's colour goes on (the pack's `paint`)
      placeholder  primitive parts for a key the manifests don't have yet (the fallback)
      centre     (game x, depth) the model's bounds are centred on at rest, for one that stands off the
                 middle of its footprint (default: the middle)
    """

    def __init__(self, id, build, size, key=None, tolerance=0.02, motions=None, paint=None, placeholder=None,
                 centre=(0.0, 0.0)):
        self.id, self.build, self.size, self.key = id, build, size, key
        self.tolerance, self.motions, self.paint, self.placeholder = tolerance, motions, paint, placeholder
        self.centre = centre


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
        objs = model.build() or meshes()
        bpy.context.scene.frame_set(1)
        check_size(model.size, model.tolerance, objs, name=model.id, centre=model.centre)
        stats(model.id)
        return objs

    def run(self, argv):
        """
        The command line, after Blender's `--`:
          (nothing)          build, check and export every model, then write the pack
          --only a,b         just these ids (the pack keeps the others' entries)
          --preview          render previews/<id>.png (and <id>@<clip>.png) instead of exporting
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
        entries = {}
        for model in models:
            objs = self.make(model)
            if '--preview' in args:
                stem = os.path.join(PREVIEWS, model.id)
                os.makedirs(PREVIEWS, exist_ok=True)
                preview(stem + '.png')
                preview_clips(stem)
                print(f'[gamekit] previewed {self.key(model)} -> {stem}.png')
                continue
            clips = motion.clips(objs)
            path = export_glb(os.path.join(ASSETS, 'models', self.name, f'{model.id}.glb'), objs, clips=bool(clips))
            self._check_motions(model, clips)
            entry = {'url': f'../../models/{self.name}/{model.id}.glb',
                     'materials': self.palette.finishes(materials_used(objs))}
            if model.motions:
                entry['motions'] = model.motions
            if model.paint:
                entry['paint'] = model.paint
            if model.placeholder:
                entry['placeholder'] = model.placeholder
            entries[self.key(model)] = entry
            print(f'[gamekit] built {self.key(model)} -> {os.path.relpath(path, ASSETS)}')
        if entries and '--no-pack' not in args:
            path = write_pack(self.pack, entries, self.comment)
            print(f'[gamekit] wrote {os.path.relpath(path, ASSETS)}')

    def _check_motions(self, model, clips):
        names = {name for _, name, _ in clips}
        motions = model.motions or {}
        played = set(motions.get('uses', {}).values()) | ({motions['open']} if motions.get('open') else set())
        missing = sorted(played - names)
        assert not missing, f'{model.id}: `motions` plays {", ".join(missing)}, which it has no clip for'
