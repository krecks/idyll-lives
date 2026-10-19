# Writing a model out: the .glb (docs/design/objects.md, "Export settings") and the family's pack entry
# ("Optimisation and registration").
import json
import os
import sys

import bpy

from .scene import ASSETS, meshes

# The gate's readings, shared: the manifests as the game stacks them, and which keys the game draws.
# (gamekit/__init__ imports this module first, so this puts check_glb.py's folder on the path for family.py too.)
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from check_glb import known_keys, manifests  # noqa: E402


def export_glb(path, objects=None, texcoords=False, clips=False):
    """
    Writes a .glb (all mesh objects unless given), +Y up so Blender -Y is the game's +Z front.
    Without textures the UVs stay out (`texcoords`): the game gives flat-coloured parts its own.
    `clips`: a model with moving parts writes its clips, one per NLA track name, and the scene's
    current frame as the rest pose (frame 1, each clip's first key).
    """
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.object.select_all(action='DESELECT')
    objects = objects if objects is not None else meshes()
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    extra = {'export_animation_mode': 'NLA_TRACKS'} if clips else {'export_animations': False}
    bpy.ops.export_scene.gltf(filepath=path, export_format='GLB', use_selection=True,
                              export_apply=True, export_yup=True, export_materials='EXPORT',
                              export_texcoords=texcoords, **extra)
    return path


def stats(name='model'):
    """Prints and returns the scene's triangles."""
    objs = meshes()
    tris = 0
    for o in objs:
        o.data.calc_loop_triangles()
        tris += len(o.data.loop_triangles)
    print(f'[gamekit] {name}: {tris} triangles in {len(objs)} meshes')
    return tris


def materials_used(objs=None):
    """The names of the materials on `objs` (default: every mesh)."""
    return {s.material.name for o in (objs if objs is not None else meshes()) for s in o.material_slots if s.material}


def write_pack(pack, entries, comment, code=(), known=None):
    """
    Writes the override pack `packs/<pack>/manifest.json` and lists it in the base manifest's `packs`.
    `entries` is {key: fields}, e.g. {'url', 'materials', 'motions', 'paint'}. The game keeps the `vary`
    of the entry it overrides (registry.ts `extended`); any other fields of that entry are copied, read
    from the manifests now so the copy can't go stale. A key with no entry below is written as it is.
    Entries of the pack that aren't given are kept.
    A new key must be one the game draws: a content object's or item's model, one a manifest has, or
    one of `code` (keys the game draws by code: `Model(code=True)`). Anything else is a misspelt `key=`,
    which would write an entry nothing draws (the object a box). `Family.run` refuses those before it
    builds anything and passes the keys it knows (`known`, check_glb `known_keys`); this is the backstop.
    """
    path = os.path.join(ASSETS, 'packs', pack, 'manifest.json')
    rel = os.path.relpath(path, ASSETS)
    with open(os.path.join(ASSETS, 'manifest.json')) as f:
        base = json.load(f)
    # The model entries of every manifest but this pack, later ones overriding (the game's order).
    below = {k: e for p, m in manifests(os.path.join(ASSETS, 'manifest.json')) if os.path.relpath(p, ASSETS) != rel
             for k, e in (m.get('entries') or {}).items() if e.get('type') == 'model'}
    try:
        with open(path) as f:
            kept = json.load(f)['entries']
    except FileNotFoundError:
        kept = {}
    known = known_keys(ASSETS, path) if known is None else known
    for key in entries:
        if key not in known and key not in code:
            raise ValueError(f'{key}: no content object names this model and no manifest has it: a misspelt key=? '
                             '(a model the game draws by code: Model(code=True))')
    for key, fields in entries.items():
        under = below.get(key)
        entry = {'type': 'model', **fields}
        for k, v in (under or {}).items():
            if k not in entry and k not in ('url', 'far', 'materials', 'motions', 'paint', 'vary', 'sink', 'centre'):
                entry[k] = v
        kept[key] = entry
    manifest = {'$comment': comment, 'version': 1, 'entries': kept}
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        json.dump(manifest, f, indent=2)
        f.write('\n')
    if rel not in base['packs']:
        base['packs'].append(rel)
        with open(os.path.join(ASSETS, 'manifest.json'), 'w') as f:
            json.dump(base, f, indent=2)
            f.write('\n')
    return path

