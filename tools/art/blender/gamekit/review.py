# `cargo check-glb --preview`, inside Blender: re-imports each model file the gate checked and renders it to
# previews/<family>/rt_<name>.png, next to the build's previews, so export problems show. The gate runs it as
#   Blender --background --factory-startup --python-exit-code 1 --python tools/art/blender/gamekit/review.py -- model.fridge …
# (no keys: every model file of the manifests).
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402

import gamekit as G  # noqa: E402
from gamekit.manifest import ASSETS, manifests  # noqa: E402


def review(paths):
    """Inside Blender: re-imports each file and renders it next to the build's previews, in its family's
    folder (models/<family>/<id>.glb: previews/<family>/rt_<id>.png)."""
    out = []
    for path in paths:
        G.reset()
        bpy.ops.import_scene.gltf(filepath=path)
        family = os.path.basename(os.path.dirname(path))
        folder = os.path.join(G.PREVIEWS, family) if family != 'models' else G.PREVIEWS
        os.makedirs(folder, exist_ok=True)
        out.append(G.preview(os.path.join(folder, 'rt_' + os.path.splitext(os.path.basename(path))[0] + '.png')))
    return out


def files(keys):
    """{key: its .glb} for `keys` (all if none), as the manifests stack: the last entry's `url`, if a file."""
    found = {}
    for path, m in manifests(os.path.join(ASSETS, 'manifest.json')):
        for key, entry in (m.get('entries') or {}).items():
            if entry.get('type') == 'model':
                url = entry.get('url', '')
                found[key] = os.path.normpath(os.path.join(os.path.dirname(path), url)) if url and '://' not in url else None
    return {k: found[k] for k in sorted(found) if found[k] and os.path.exists(found[k]) and (not keys or k in keys)}


if __name__ == '__main__':
    keys = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    for key, path in files(keys).items():
        png, = review([path])
        print(f'[gamekit] previewed {key} -> {os.path.relpath(png, os.path.dirname(G.PREVIEWS))}')
