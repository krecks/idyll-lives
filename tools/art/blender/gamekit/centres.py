# `cargo art --centres`, inside Blender: writes centres.json (where each model is centred at rest, for the
# gate) from every family script's models, as a build writes it for the models it builds. It imports the
# family scripts without building anything (a family builds only when run as the script), so it takes a few
# seconds. The runner starts it as
#   Blender --background --factory-startup --python-exit-code 1 --python tools/art/blender/gamekit/centres.py
import glob
import importlib.util
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # tools/art/blender
FAMILIES = os.path.join(HERE, 'families')
sys.path[:0] = [FAMILIES, HERE]

import gamekit as G  # noqa: E402
from gamekit.manifest import CENTRES, write_centres  # noqa: E402


def families():
    """The family scripts (not the `_` templates), sorted."""
    return sorted(os.path.splitext(os.path.basename(p))[0] for p in glob.glob(os.path.join(FAMILIES, '*.py'))
                  if not os.path.basename(p).startswith('_'))


def centres():
    """Writes centres.json from every family script's models."""
    found, by = {}, {}
    for name in families():
        spec = importlib.util.spec_from_file_location(name, os.path.join(FAMILIES, f'{name}.py'))
        module = importlib.util.module_from_spec(spec)
        sys.modules.setdefault(name, module)  # (a family that imports another's helpers: plants)
        spec.loader.exec_module(module)
        for family in (v for v in vars(module).values() if isinstance(v, G.Family)):
            for model in family.models:
                key = family.key(model)
                if key in found and by[key] != family.name:
                    sys.exit(f'{key}: built by both {by[key]} and {family.name}')
                found[key], by[key] = model.centre, family.name
    write_centres(found, replace=True)
    off = sum(1 for c in found.values() if tuple(c) != (0, 0))
    print(f'[gamekit] wrote {os.path.relpath(CENTRES)}: {len(found)} models of {len(set(by.values()))} families ({off} off the middle)')


if __name__ == '__main__':
    centres()
