# Writing a model out: the .glb (docs/design/objects.md, "Export settings") and the family's pack entry
# ("Optimisation and registration").
import json
import os

import bpy

from .scene import ASSETS, meshes


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


def write_pack(pack, entries, comment):
    """
    Writes the override pack `packs/<pack>/manifest.json` and lists it in the base manifest's `packs`.
    `entries` is {key: fields}, e.g. {'url', 'materials', 'motions', 'paint'}. The game keeps the `vary`
    of the entry it overrides (registry.ts `extended`); any other fields of that entry are copied, read
    from the manifests now so the copy can't go stale. A key with no entry below is written as it is.
    Entries of the pack that aren't given are kept.
    """
    path = os.path.join(ASSETS, 'packs', pack, 'manifest.json')
    rel = os.path.relpath(path, ASSETS)
    with open(os.path.join(ASSETS, 'manifest.json')) as f:
        base = json.load(f)
    below = _entries_below(rel)
    try:
        with open(path) as f:
            kept = json.load(f)['entries']
    except FileNotFoundError:
        kept = {}
    for key, fields in entries.items():
        under = below.get(key)
        entry = {'type': 'model', **fields}
        for k, v in (under or {}).items():
            if k not in entry and k not in ('url', 'far', 'materials', 'motions', 'paint', 'vary', 'sink'):
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


def _entries_below(pack_rel):
    """The model entries of every manifest but `pack_rel`, later ones overriding (the game's order)."""
    out = {}

    def walk(path):
        rel = os.path.relpath(path, ASSETS)
        with open(path) as f:
            m = json.load(f)
        if rel != pack_rel:
            out.update({k: e for k, e in m.get('entries', {}).items() if e.get('type') == 'model'})
        for p in m.get('packs', []):
            walk(os.path.normpath(os.path.join(os.path.dirname(path), p)))
    walk(os.path.join(ASSETS, 'manifest.json'))
    return out
