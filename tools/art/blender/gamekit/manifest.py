# The manifests and the content as the game reads them, for the Blender side: which keys a build may write
# (`known_keys`), which clips an entry's `motions` plays (`played`), and where each model is centred
# (centres.json). Plain Python, no bpy: the gate (crates/tools/src/checkglb.rs, `cargo check-glb`) reads the
# same files the same way, and check_glb.py imports these.
import json
import os

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # tools/art/blender
ASSETS = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'assets'))
CONTENT = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'content'))
# Where each Blender-built model is centred at rest (its Model's `centre`, (0, 0) for most): {key: [x, depth]},
# for the gate only (the game doesn't read it, so it stays out of the manifests it downloads). A build writes
# its models' (gamekit/family.py), `cargo art --centres` every family's (gamekit/centres.py). A key that isn't
# in it isn't built here (a third-party file): its centre isn't checked.
CENTRES = os.path.join(HERE, 'centres.json')


def read_centres(path=CENTRES):
    """{key: [x, depth]}: centres.json (empty if there's none yet)."""
    try:
        with open(path) as f:
            return json.load(f)
    except FileNotFoundError:
        return {}


def write_centres(centres, path=CENTRES, replace=False):
    """Writes `centres` ({key: (x, depth)}) over centres.json's entries for those keys, keeping the others
    (`replace`: dropping them): one key a line, sorted, to 0.1 mm."""
    out = {} if replace else read_centres(path)
    out.update({key: [round(float(v), 4) + 0.0 for v in c] for key, c in centres.items()})
    lines = [f'  {json.dumps(key)}: {json.dumps(c)}' for key, c in sorted(out.items())]
    with open(path, 'w') as f:
        f.write('{\n' + ',\n'.join(lines) + '\n}\n')
    return path


def manifests(path):
    """[(path, manifest)] for a manifest and the packs it lists, depth first, in order. A manifest that can't
    be read raises ValueError naming it."""
    try:
        with open(path) as f:
            m = json.load(f)
    except (OSError, ValueError) as err:
        raise ValueError(f'{os.path.relpath(path, os.path.dirname(ASSETS))}: {err}') from None
    out = [(path, m)]
    for pack in m.get('packs', []):
        out += manifests(os.path.normpath(os.path.join(os.path.dirname(path), pack)))
    return out


def uses(motions):
    """[(use, {clip, hold, delay, every, slot, out})] for each clip an entry's `motions.uses` plays (a clip,
    {clip, ...} or a list of them)."""
    return [(use, u if isinstance(u, dict) else {'clip': u})
            for use, value in (motions.get('uses') or {}).items() for u in (value if isinstance(value, list) else [value])]


def played(motions):
    """{clip: what plays it} for an entry's `motions`: `uses`, `loops`, `open`, `idle`, `night`, `growth`,
    `ride` (the clip whose part carries its users) and `clock` (`hour`, `minute`: hands set by the time)."""
    out = {}
    for use, u in uses(motions):
        out.setdefault(u['clip'], f'use {use!r}')
    for use, clip in (motions.get('loops') or {}).items():
        out.setdefault(clip, f'loop {use!r}')
    for kind in ('open', 'idle', 'night', 'growth', 'ride'):
        if motions.get(kind):
            out.setdefault(motions[kind], kind)
    for hand in ('hour', 'minute'):
        if (motions.get('clock') or {}).get(hand):
            out.setdefault(motions['clock'][hand], f'clock {hand}')
    return out


def content():
    """(objects, items, buyCategories) of the game's content: base.json and the files its `include` lists,
    merged as the game merges them (sim_core::merge_content: each file's lists after the ones before).
    A file that can't be read raises ValueError naming it."""
    def read(name):
        try:
            with open(os.path.join(CONTENT, name)) as f:
                data = json.load(f)
        except (OSError, ValueError) as err:
            raise ValueError(f'web/public/content/{name}: {err}') from None
        if not isinstance(data, dict):
            raise ValueError(f'web/public/content/{name}: a content file is a JSON object')
        return data
    base = read('base.json')
    files = [base] + [read(name) for name in base.get('include', [])]
    return tuple([d for f in files for d in f.get(k) or []] for k in ('objects', 'items', 'buyCategories'))


def model_key(d, prefix):
    """The model a content object (`prefix` model) or item (item) draws, as the renderer picks it
    (`def.model ?? `model.${id}``): its `model`, or `<prefix>.<id>`. An empty `model` stays empty (an error)."""
    return d['model'] if d.get('model') is not None else f'{prefix}.{d["id"]}'


def content_keys(defs=None):
    """{model key: what draws it ('object fridge')}: the model each content object and item draws
    (`model_key`). The game's own models (the rock, the marker, the street's) are drawn by code and aren't here."""
    objects, items, _ = defs or content()
    keys = {}
    for kind, prefix, found in (('object', 'model', objects), ('item', 'item', items)):
        for d in found:
            keys.setdefault(model_key(d, prefix), f'{kind} {d["id"]}')
    return keys


def known_keys(assets=ASSETS, pack=None):
    """The model keys a build may write: every content object's and item's model, and every model entry of the
    manifests (`assets`/manifest.json and its packs) and of `pack` (a pack's manifest.json, whether `packs` lists
    it or not). Any other key is a misspelt `key=`, refused (gamekit `Family.run`, `write_pack`) unless its
    `Model` says the game draws it by code (`code=True`). Raises ValueError on content it can't read."""
    known = set(content_keys()) - {''}
    stack = manifests(os.path.join(assets, 'manifest.json'))
    if pack and os.path.exists(pack) and os.path.normpath(pack) not in {os.path.normpath(p) for p, _m in stack}:
        stack += manifests(pack)
    known |= {k for _path, m in stack for k, e in (m.get('entries') or {}).items() if e.get('type') == 'model'}
    return known
