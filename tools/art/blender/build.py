# Builds Blender families and checks what they wrote (plain Python; it runs Blender for you).
#
#   python3 -B tools/art/blender/build.py kitchen                 build, export and register a family
#   python3 -B tools/art/blender/build.py kitchen --only fridge   one model of it
#   python3 -B tools/art/blender/build.py --all                   every family
#   python3 -B tools/art/blender/build.py garden_decor --preview  render previews/ instead of exporting
#   python3 -B tools/art/blender/build.py --centres               write centres.json from every family, no Blender
#
# A family is a script in families/ (those starting with `_` are templates). Blender comes from
# $BLENDER, the macOS app, or PATH; the models are made with Blender 5.2 LTS. After exporting it runs
# check_glb.py on the keys it built and exits 1 if a build or the check fails.
#
# `--centres` writes centres.json (where each model is centred at rest, for the gate: check_glb.py
# CENTRES) for every model of every family, as a build writes it for the models it builds. It reads the
# family scripts without building anything: they're imported with Blender's modules stood in for, so
# it takes a second and needs no Blender. Run it after changing a Model's `centre` without a rebuild
# (the gate then says which files are off it), or after adding or removing models.
import glob
import importlib.util
import os
import re
import shutil
import subprocess
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
FAMILIES = os.path.join(HERE, 'families')


def blender():
    for path in (os.environ.get('BLENDER'), '/Applications/Blender.app/Contents/MacOS/Blender', shutil.which('blender')):
        if path and os.path.exists(path):
            return path
    sys.exit('Blender not found: install Blender 5.2 LTS or set $BLENDER')


def families():
    return sorted(os.path.splitext(os.path.basename(p))[0] for p in glob.glob(os.path.join(FAMILIES, '*.py'))
                  if not os.path.basename(p).startswith('_'))


class _Stand_in:
    """Anything Blender's modules give, enough to import a family script without running its builds."""
    def __getattr__(self, name):
        if name.startswith('__') and name.endswith('__'):
            raise AttributeError(name)
        return _Stand_in()

    def __call__(self, *a, **k):
        return _Stand_in()

    def __iter__(self):
        return iter(())

    def __getitem__(self, k):
        return _Stand_in()

    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False

    def _op(self, *a):
        return _Stand_in()

    __add__ = __radd__ = __sub__ = __rsub__ = __mul__ = __rmul__ = __matmul__ = __rmatmul__ = _op
    __truediv__ = __rtruediv__ = __neg__ = __pos__ = _op


class _Materials(_Stand_in):
    def new(self, name, *a, **k):
        made = _Stand_in()
        made.name = name  # (materials.py checks the name it gets back: a family's palette is made as it's imported)
        return made


class _Module(types.ModuleType):
    def __getattr__(self, name):
        if name.startswith('__') and name.endswith('__'):
            raise AttributeError(name)
        return _Stand_in()


def centres():
    """Writes centres.json from every family script's models (`--centres`), without Blender."""
    for name in ('bpy', 'bpy.types', 'bpy.props', 'bmesh', 'bmesh.ops', 'mathutils', 'mathutils.noise',
                 'mathutils.geometry', 'mathutils.bvhtree', 'mathutils.kdtree'):
        sys.modules[name] = _Module(name)
    data = _Stand_in()
    type(data).materials = _Materials()
    sys.modules['bpy'].data = data
    sys.modules['bpy'].app = app = _Stand_in()
    app.version = (5, 2, 0)  # (no warning off the Blender the models are made with: none runs)
    sys.dont_write_bytecode = True
    sys.path[:0] = [FAMILIES, HERE]
    import gamekit as G
    from check_glb import CENTRES, write_centres
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
    print(f'wrote {os.path.relpath(CENTRES)}: {len(found)} models of {len(set(by.values()))} families ({off} off the middle)')


def main(argv):
    if '--centres' in argv:
        return centres()
    names = [a for a in argv if not a.startswith('-')]
    if '--only' in argv:
        names.remove(argv[argv.index('--only') + 1])
    if '--all' in argv:
        names = families()
    if not names:
        print(__doc__ if __doc__ else '', end='')
        sys.exit(f'name a family ({", ".join(families())}) or --all')
    passthrough = []
    for flag in ('--preview', '--no-pack'):
        if flag in argv:
            passthrough.append(flag)
    if '--only' in argv:
        passthrough += ['--only', argv[argv.index('--only') + 1]]
    exe, built, failed = blender(), [], []
    for name in names:
        script = os.path.join(FAMILIES, f'{name}.py')
        if not os.path.exists(script):
            sys.exit(f'no family {name!r} ({", ".join(families())})')
        cmd = [exe, '--background', '--factory-startup', '--python-exit-code', '1', '--python', script, '--', *passthrough]
        run = subprocess.run(cmd, capture_output=True, text=True)
        out = run.stdout.splitlines()
        for line in out:
            if line.startswith('[gamekit]'):
                print(line, flush=True)
        if run.returncode:
            # Blender prints the script's traceback to stdout
            start = next((i for i, line in enumerate(out) if line.startswith('Traceback')), max(0, len(out) - 20))
            print('\n'.join(out[start:]), run.stderr[-2000:], sep='\n', flush=True)
            failed.append(name)
        built += re.findall(r'^\[gamekit\] built (\S+)', run.stdout, re.M)
    if built:
        check = subprocess.run([sys.executable, '-I', '-B', os.path.join(HERE, 'check_glb.py'), *built])
        if check.returncode:
            failed.append('check_glb')
        print('Exports are not byte-for-byte repeatable: commit only the models you meant to change '
              '(git checkout the others).')
    if failed:
        sys.exit(f'failed: {", ".join(failed)}')


if __name__ == '__main__':
    main(sys.argv[1:])
