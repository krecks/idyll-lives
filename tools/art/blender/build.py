# Builds Blender families and checks what they wrote (plain Python; it runs Blender for you).
#
#   python3 -B tools/art/blender/build.py kitchen                 build, export and register a family
#   python3 -B tools/art/blender/build.py kitchen --only fridge   one model of it
#   python3 -B tools/art/blender/build.py --all                   every family
#   python3 -B tools/art/blender/build.py garden_decor --preview  render previews/ instead of exporting
#
# A family is a script in families/ (those starting with `_` are templates). Blender comes from
# $BLENDER, the macOS app, or PATH; the models are made with Blender 5.2 LTS. After exporting it runs
# check_glb.py on the keys it built and exits 1 if a build or the check fails.
import glob
import os
import re
import shutil
import subprocess
import sys

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


def main(argv):
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
