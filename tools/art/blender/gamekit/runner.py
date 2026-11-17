# Talking to the runner, `cargo art` (crates/tools/src/art.rs), which reads the content and the manifests and
# writes the packs and centres.json, so the Python here never reads them itself. It hands a build the model
# keys it may write (`$GAMEKIT_KNOWN`: a JSON list) and takes back what the build made (`$GAMEKIT_RESULT`: the
# file a build writes its result to, read once Blender has exited).
import json
import os
import sys


def _path(var):
    path = os.environ.get(var)
    if not path:
        sys.exit(f'[gamekit] no ${var}: run Blender through cargo art (or cargo check-glb --preview)')
    return path


def known_keys():
    """The model keys a build may write: every content object's and item's model, every manifest's model entries
    and every pack's own. Any other key is a misspelt `key=` (unless the game draws it by code: `code=True`)."""
    with open(_path('GAMEKIT_KNOWN')) as f:
        return set(json.load(f))


def report(result):
    """Hands `result` (a dict) back to the runner."""
    with open(_path('GAMEKIT_RESULT'), 'w') as f:
        json.dump(result, f)
