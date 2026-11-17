# `cargo check-glb --preview`, inside Blender: re-imports each model file the gate checked and renders it to
# previews/<family>/rt_<name>.png, next to the build's previews, so export problems show. The gate runs it as
#   Blender --background --factory-startup --python-exit-code 1 --python tools/art/blender/gamekit/review.py -- model.fridge=<its .glb> …
# (each checked model whose file is there, as the manifests stack: the gate finds them).
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402

import gamekit as G  # noqa: E402


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


if __name__ == '__main__':
    for arg in sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []:
        key, path = arg.split('=', 1)
        png, = review([path])
        print(f'[gamekit] previewed {key} -> {os.path.relpath(png, os.path.dirname(G.PREVIEWS))}')
