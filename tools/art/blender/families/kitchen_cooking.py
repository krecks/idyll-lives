# The kitchen's cookers (K2): the electric cooker, the gas range, the oven tower, the range cooker, the
# hob counter and the microwave counter. Notes: kitchen_cooking.md.
#
# A cooker is a box on the floor with a hob on top and an oven below; its door drops on a bottom hinge,
# and a knob turns (with a ring of flame on the gas ones) while a resident cooks. The parts are placed
# in game axes (`F`): the old depths are shifted back by their depth centre, so
# the model is centred on its tile (see GameFrame).
#
# Run:  cargo art kitchen_cooking [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    enamel=('#F4F3EF', 'gloss'),     # a white cooker body; the main part the player colours (`paint`)
    enamel2=('#EFE5D3', 'gloss'),    # a cream range cooker
    cab=('#EDEBE6', 'satin'),        # cabinet fronts (the oven tower, the hob counter)
    worktop=('#E8E6E1', 'gloss'),    # a counter worktop
    dark=('#2B2C2E', 'satin'),       # oven fronts, plinths and control strips
    hob=('#1B1E22', 'gloss'),        # a ceramic/induction hob and an oven window (opaque, glossy black)
    glass=('#1D2228', 'glass'),      # the microwave's see-through window
    plate=('#E9EAEC', 'gloss'),      # the microwave's turntable plate
    chrome=('#D0D3D6', 'chrome'),    # handles and knobs
    grate=('#22252A', 'satin'),      # burner heads and grates
    flame=('#3E7CC2', 'gloss'),      # a gas flame ring (gloss, not emissive)
    heat=('#D2402A', 'gloss'),       # a ceramic ring glowing hot (gloss red, so it reads by day)
    charcoal=('#33363A', 'satin'),   # the microwave's body
    display=('#5EC28B', 'gloss'),    # the microwave's display
    tray=('#8E9196', 'metal'),       # an oven tray
    pot=('#6E7377', 'metal'),        # a stock pot
    pan=('#4A4E52', 'satin'),        # a frying pan
    steel=('#C9CBCC', 'metal'),      # a stainless appliance front
    toaster=('#C05B4D', 'gloss'),    # a toaster's body
    jug=('#DCE6EA', 'gloss'),        # a blender's jug (opaque, so it costs no extra draw call)
    bean=('#5A3A26', 'gloss'),       # a coffee machine's bean hopper
    cup=('#F2F1EE', 'gloss'),        # a cup
    coffee=('#9A6B3F', 'gloss'),     # the stream of coffee
    fruit=('#E0A33E', 'gloss'),      # a piece of fruit in the bowl
)

SECONDS = 1.4   # a door drops, a drawer slides or a knob turns in this long
# catalog2's 0.79-deep counter numbers drawn 0.66 deep against the wall (kitchen_counters.py COUNTER66)
COUNTER66 = ((-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23))
HOLD = 3.0      # seconds a door stays open once a use starts
# the microwave's plate turns once its door has shut (`door_open` opens in 1.0 s and plays back from its hold, 1.2 s)
TURNTABLE = {'clip': 'turntable', 'delay': 2.2, 'every': 1.9}   # (a hair under its 2 s turn: no pause between turns)


def _knob(F, name, x, y, z, r=0.022, d=0.03, material=None, mark=True):
    """A control knob: a short cylinder facing the front, with a mark so a turn shows."""
    F.cyl(name, r, d, (x, y, z), material or M.chrome, rot=(90, 0, 0), verts=12)
    if mark:
        F.box(f'{name}_mark', 0.006, 0.02, 0.006, (x, y + r * 0.55, z + d / 2), M.dark)


def _knob_obj(F, name, x, y, z, r=0.022, d=0.03):
    """A knob as its own object, its origin at its centre, so a clip turns it in place."""
    with G.new_parts() as parts:
        _knob(F, name, x, y, z, r, d)
    return G.join(parts, name, origin=F.at(x, y, z))


def _flame(F, name, x, y, z, r):
    """A gas flame ring, built full size but scaled to nothing, grown up in every axis while the
    burner is on (the `water_on` way, so it is hidden at rest)."""
    with G.new_parts() as parts:
        G.torus(name, r, 0.013, at=F.at(x, y, z), material=M.flame, major_segments=16, minor_segments=8)
    obj = G.join(parts, name, origin=F.at(x, y, z))
    obj.scale = (0.001, 0.001, 0.001)
    return obj


def _light(flame):
    """Grows a flame ring in all three axes, so `burner_on` shows it from nothing."""
    for axis in 'XYZ':
        G.grow(flame, 'burner_on', seconds=0.5, axis=axis)


def _burner(F, name, x, z, y, r=0.07):
    """A gas burner on a hob at height `y`: a dark head with a grate ring around it."""
    F.cyl(f'{name}_head', r, 0.02, (x, y, z), M.grate, verts=14)
    G.torus(f'{name}_grate', r + 0.014, 0.008, at=F.at(x, y + 0.013, z), material=M.grate,
            major_segments=16, minor_segments=6)


def _hot_ring(F, name, x, y, z, r):
    """A ceramic ring glowing red: a thin disc on the glass, hidden at rest (scaled to nothing) and grown
    by `_light` while the ring is on, as a gas flame is."""
    with G.new_parts() as parts:
        F.cyl(name, r, 0.003, (x, y, z), M.heat, verts=20)
    obj = G.join(parts, name, origin=F.at(x, y, z))
    obj.scale = (0.001, 0.001, 0.001)
    return obj


def _hob_ring(F, name, x, z, y):
    """A ring of a ceramic/induction hob, set into the glass."""
    G.torus(name, 0.07, 0.006, at=F.at(x, y, z), material=M.grate, major_segments=18, minor_segments=6)


def _oven_door(F, name, x, w, y, z, hinge_y):
    """An oven door: a dark front with a window and a bar handle, hinged along its bottom edge.
    Returns it; the caller swings it once the model has settled."""
    with G.new_parts() as parts:
        F.box(f'{name}_front', w, 0.50, 0.025, (x, y, z), M.dark, bevel=0.01)
        F.box(f'{name}_window', w - 0.14, 0.30, 0.01, (x, y + 0.01, z + 0.014), M.hob, bevel=0.006)
        F.box(f'{name}_handle', w - 0.12, 0.022, 0.022, (x, y + 0.27, z + 0.036), M.chrome, bevel=0.008)
        for s in (-1, 1):
            F.box(f'{name}_post{s}', 0.018, 0.018, 0.024, (x + s * (w / 2 - 0.06), y + 0.27, z + 0.016), M.chrome)
    return G.hinged(G.join(parts, name), F.at(x, hinge_y, z))


# --- electric cooker: 1x1, four ceramic rings and an oven ---------------------------------------------
def stoveElectric():
    F = G.GameFrame(shift=0.1295)
    with G.new_parts() as still:
        F.box('plinth', 0.56, 0.05, 0.56, (0, 0.025, -0.16), M.dark, bevel=0.005)
        F.box('carcass', 0.60, 0.82, 0.60, (0, 0.46, -0.16), M.enamel, bevel=0.02)
        F.box('hob', 0.60, 0.02, 0.60, (0, 0.88, -0.16), M.hob, bevel=0.004)
        F.box('back', 0.60, 0.09, 0.10, (0, 0.935, -0.41), M.enamel, bevel=0.008)
        F.box('strip', 0.56, 0.06, 0.012, (0, 0.845, 0.142), M.dark, bevel=0.003)
        for i, x in enumerate((-0.07, 0.07, 0.21), start=1):
            _knob(F, f'knob{i}', x, 0.845, 0.155)
        F.box('drawer', 0.56, 0.10, 0.02, (0, 0.12, 0.15), M.enamel, bevel=0.006)
        F.box('drawer_handle', 0.30, 0.02, 0.02, (0, 0.16, 0.17), M.chrome, bevel=0.005)
        for i, (x, z) in enumerate(((-0.16, -0.30), (0.16, -0.30), (-0.16, -0.05), (0.16, -0.05))):
            _hob_ring(F, f'ring{i}', x, z, 0.895)
        F.box('oven_cavity', 0.48, 0.40, 0.003, (0, 0.43, 0.1405), M.dark)   # inside the closed door
    G.join(still, 'body', origin=(0, 0, 0))
    door = _oven_door(F, 'oven_door', 0, 0.56, 0.43, 0.152, hinge_y=0.17)
    knob = _knob_obj(F, 'knob0', -0.21, 0.845, 0.155)
    # the front ring on the knob's side glows red while it's on, as the hob counter's does
    ring = _hot_ring(F, 'hot_ring', -0.16, 0.892, -0.05, 0.066)
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(ring)


# --- gas range: 1x1, four burners with flames, a taller back guard and an oven -------------------------
def stoveGas():
    F = G.GameFrame(shift=0.112)
    with G.new_parts() as still:
        F.box('plinth', 0.88, 0.06, 0.60, (0, 0.03, -0.16), M.dark, bevel=0.005)
        F.box('carcass', 0.92, 0.78, 0.64, (0, 0.45, -0.14), M.enamel, bevel=0.02)
        F.box('hob', 0.92, 0.03, 0.64, (0, 0.855, -0.14), M.dark, bevel=0.004)
        F.box('back', 0.92, 0.12, 0.04, (0, 0.99, -0.45), M.enamel, bevel=0.008)
        F.box('strip', 0.86, 0.06, 0.012, (0, 0.80, 0.152), M.dark, bevel=0.003)
        for i, x in enumerate((-0.16, 0.0, 0.16), start=1):
            _knob(F, f'knob{i}', x, 0.80, 0.165)
        F.box('drawer', 0.88, 0.10, 0.02, (0, 0.11, 0.16), M.enamel, bevel=0.006)
        F.box('drawer_handle', 0.30, 0.02, 0.02, (0, 0.15, 0.18), M.chrome, bevel=0.005)
        for i, (x, z) in enumerate(((-0.26, -0.30), (0.26, -0.30), (-0.26, -0.02), (0.26, -0.02))):
            _burner(F, f'burner{i}', x, z, 0.87)
        # a stock pot and a frying pan on the hob, reaching the model's height and depth
        F.cyl('pot', 0.13, 0.20, (-0.26, 0.99, -0.30), M.pot, verts=16, bevel=0.006)
        F.cyl('pot_lid', 0.135, 0.023, (-0.26, 1.1015, -0.30), M.pot, verts=16)
        F.cyl('pan', 0.11, 0.045, (0.26, 0.9, -0.02), M.pan, verts=16)
        G.rod('pan_handle', F.at(0.37, 0.905, -0.02), F.at(0.37, 0.905, 0.234), 0.012, material=M.pan, verts=10)
        F.box('oven_cavity', 0.78, 0.40, 0.003, (0, 0.42, 0.1805), M.dark)   # inside the closed door
    G.join(still, 'body', origin=(0, 0, 0))
    door = _oven_door(F, 'oven_door', 0, 0.88, 0.42, 0.17, hinge_y=0.17)
    knob = _knob_obj(F, 'knob0', -0.32, 0.80, 0.165)
    flame = _flame(F, 'flame', -0.26, 0.885, -0.02, 0.075)
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(flame)


# --- oven tower: 1x1, two built-in ovens in a tall cabinet ---------------------------------------------
def wallOven():
    F = G.GameFrame(shift=0.102)
    with G.new_parts() as still:
        F.box('plinth', 0.66, 0.06, 0.56, (0, 0.03, -0.15), M.dark, bevel=0.005)
        F.box('cabinet', 0.70, 1.98, 0.60, (0, 1.05, -0.15), M.cab, bevel=0.012)
        F.box('cornice', 0.70, 0.07, 0.62, (0, 2.075, -0.15), M.cab, bevel=0.008)
        # a lower oven (still) and a bottom and top cabinet door
        F.box('oven_low', 0.60, 0.50, 0.02, (0, 0.98, 0.16), M.dark, bevel=0.008)
        F.box('window_low', 0.46, 0.30, 0.01, (0, 0.99, 0.172), M.hob, bevel=0.006)
        F.box('handle_low', 0.48, 0.022, 0.022, (0, 1.25, 0.235), M.chrome, bevel=0.008)
        for s in (-1, 1):
            F.box(f'post_low{s}', 0.018, 0.018, 0.024, (s * 0.24, 1.25, 0.215), M.chrome)
        F.box('door_bottom', 0.60, 0.52, 0.02, (0, 0.40, 0.16), M.cab, bevel=0.008)
        F.box('door_top', 0.60, 0.16, 0.02, (0, 1.96, 0.16), M.cab, bevel=0.008)
        for y, name in ((0.40, 'bottom'), (1.96, 'top')):
            F.box(f'pull_{name}', 0.012, 0.12, 0.02, (0.24, y, 0.18), M.chrome, bevel=0.004)
        F.box('oven_cavity', 0.52, 0.42, 0.003, (0, 1.62, 0.1505), M.dark)   # inside the closed upper door
    G.join(still, 'body', origin=(0, 0, 0))
    # the upper oven door (hinged at its bottom) and its tray
    with G.new_parts() as parts:
        F.box('oven_door', 0.60, 0.50, 0.025, (0, 1.62, 0.16), M.dark, bevel=0.01)
        F.box('window_up', 0.46, 0.30, 0.01, (0, 1.63, 0.176), M.hob, bevel=0.006)
        F.box('handle_up', 0.48, 0.022, 0.022, (0, 1.89, 0.235), M.chrome, bevel=0.008)
        for s in (-1, 1):
            F.box(f'post_up{s}', 0.018, 0.018, 0.024, (s * 0.24, 1.89, 0.215), M.chrome)
    door = G.hinged(G.join(parts, 'oven_door'), F.at(0, 1.37, 0.16))
    with G.new_parts() as parts:
        F.box('tray', 0.46, 0.03, 0.34, (0, 1.60, 0.0), M.tray, bevel=0.004)
        F.box('tray_lip', 0.46, 0.05, 0.012, (0, 1.60, 0.164), M.tray, bevel=0.003)
    tray = G.join(parts, 'tray', origin=F.at(0, 1.60, -0.17))
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.slide(tray, 'oven_open', (0, -0.10, 0), seconds=SECONDS)


# --- range cooker: 2x1, six burners, two ovens and a chrome rail ---------------------------------------
def rangeCooker():
    F = G.GameFrame(shift=0.1025)
    with G.new_parts() as still:
        F.box('plinth', 1.54, 0.06, 0.62, (0, 0.03, -0.15), M.dark, bevel=0.005)
        F.box('carcass', 1.58, 0.80, 0.66, (0, 0.46, -0.14), M.enamel2, bevel=0.02)
        F.box('hob', 1.58, 0.03, 0.66, (0, 0.875, -0.14), M.dark, bevel=0.004)
        F.box('back', 1.58, 0.24, 0.04, (0, 1.01, -0.46), M.enamel2, bevel=0.008)
        F.box('strip', 1.5, 0.06, 0.012, (0, 0.82, 0.202), M.dark, bevel=0.003)
        for i, x in enumerate((-0.36, -0.12, 0.12, 0.36, 0.6), start=1):
            _knob(F, f'knob{i}', x, 0.82, 0.215)
        # the right oven door is still; the left one drops (built below)
        F.box('oven_right', 0.72, 0.50, 0.025, (0.40, 0.42, 0.20), M.dark, bevel=0.01)
        F.box('window_right', 0.58, 0.30, 0.01, (0.40, 0.43, 0.214), M.hob, bevel=0.006)
        F.box('handle_right', 0.60, 0.022, 0.022, (0.40, 0.69, 0.236), M.chrome, bevel=0.008)
        for s in (-1, 1):
            F.box(f'post_right{s}', 0.018, 0.018, 0.024, (0.40 + s * 0.30, 0.69, 0.216), M.chrome)
        for i, (x, z) in enumerate(((-0.55, -0.30), (0.0, -0.30), (0.55, -0.30),
                                    (-0.55, -0.02), (0.0, -0.02), (0.55, -0.02))):
            _burner(F, f'burner{i}', x, z, 0.89)
        F.cyl('pot', 0.14, 0.20, (-0.55, 1.02, -0.30), M.pot, verts=16, bevel=0.006)
        F.cyl('pot_lid', 0.145, 0.02, (-0.55, 1.13, -0.30), M.pot, verts=16)
        # the chrome rail across the front, below the hob
        G.rod('rail', F.at(-0.7, 0.92, 0.263), F.at(0.7, 0.92, 0.263), 0.012, material=M.chrome, verts=10)
        for s in (-1, 1):
            G.rod(f'rail_post{s}', F.at(s * 0.7, 0.90, 0.19), F.at(s * 0.7, 0.90, 0.263), 0.01, material=M.chrome, verts=8)
        F.box('oven_cavity', 0.62, 0.40, 0.003, (-0.40, 0.42, 0.1905), M.dark)   # inside the closed door
    G.join(still, 'body', origin=(0, 0, 0))
    door = _oven_door(F, 'oven_door', -0.40, 0.72, 0.42, 0.20, hinge_y=0.17)
    knob = _knob_obj(F, 'knob0', -0.6, 0.82, 0.215)
    flame = _flame(F, 'flame', 0.0, 0.905, -0.02, 0.075)
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(flame)


# --- hob counter: 1x1, an induction hob in a worktop over a base unit -----------------------------------
def cooktopCounter():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        F.box('plinth', 0.96, 0.10, 0.64, (0, 0.05, -0.14), M.dark, bevel=0.005)
        F.box('carcass', 1.0, 0.80, 0.70, (0, 0.50, -0.11), M.cab, bevel=0.012)
        F.box('worktop', 1.0, 0.05, 0.79, (0, 0.925, -0.085), M.worktop, bevel=0.008)
        F.box('hob', 0.62, 0.012, 0.50, (0, 0.948, -0.10), M.hob, bevel=0.004)     # 4 mm proud: a pan stands on it
        for i, (x, z) in enumerate(((-0.16, -0.24), (0.16, -0.24), (-0.16, 0.03), (0.16, 0.03))):
            _hob_ring(F, f'ring{i}', x, z, 0.95)
        for i, x in enumerate((-0.14, 0.14), start=1):
            _knob(F, f'knob{i}', x, 0.84, 0.252)
        for i, y in enumerate((0.28, 0.52, 0.76)):
            F.box(f'drawer{i}', 0.94, 0.20, 0.02, (0, y, 0.25), M.cab, bevel=0.006)
            F.box(f'pull{i}', 0.30, 0.02, 0.02, (0, y + 0.07, 0.27), M.chrome, bevel=0.004)
    G.join(still, 'body', origin=(0, 0, 0))
    knob = _knob_obj(F, 'knob0', -0.42, 0.84, 0.252)
    flame = _hot_ring(F, 'hot_ring', 0.16, 0.9545, -0.24, 0.066)
    G.settle()
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(flame)


# --- microwave counter: 1x1, a microwave on a small base unit ------------------------------------------
def microwave():
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        F.box('plinth', 0.96, 0.10, 0.58, (0, 0.05, -0.15), M.dark, bevel=0.005)
        F.box('carcass', 1.0, 0.82, 0.62, (0, 0.51, -0.16), M.cab, bevel=0.012)
        F.box('worktop', 1.0, 0.04, 0.66, (0, 0.93, -0.15), M.worktop, bevel=0.008)
        F.box('door_front', 0.60, 0.52, 0.02, (0, 0.40, 0.16), M.cab, bevel=0.008)
        F.box('pull', 0.30, 0.02, 0.02, (0, 0.62, 0.18), M.chrome, bevel=0.004)
        # the microwave: a box on the worktop, seen from the front its door on the left and the controls
        # on the right (the game's -x is the viewer's right): a hollow body, the turntable inside
        F.box('micro_floor', 0.50, 0.02, 0.36, (-0.20, 0.96, -0.28), M.charcoal, bevel=0.004)
        F.box('micro_top', 0.50, 0.02, 0.36, (-0.20, 1.24, -0.28), M.charcoal, bevel=0.004)
        F.box('micro_back', 0.50, 0.30, 0.02, (-0.20, 1.1, -0.45), M.charcoal, bevel=0.004)
        F.box('micro_side', 0.02, 0.30, 0.36, (0.04, 1.1, -0.28), M.charcoal, bevel=0.004)
        F.box('micro_column', 0.12, 0.30, 0.36, (-0.39, 1.1, -0.28), M.charcoal, bevel=0.004)
        F.box('micro_face', 0.12, 0.28, 0.02, (-0.38, 1.1, -0.09), M.dark, bevel=0.004)
        F.box('micro_screen', 0.08, 0.06, 0.006, (-0.38, 1.19, -0.078), M.display, bevel=0.002)
        for i, y in enumerate((1.05, 0.99)):
            F.box(f'button{i}', 0.05, 0.02, 0.006, (-0.38, y, -0.078), M.chrome, bevel=0.002)
        F.cyl('micro_foot', 0.012, 0.02, (-0.05, 0.96, -0.10), M.dark, verts=8)
        F.cyl('micro_foot2', 0.012, 0.02, (-0.35, 0.96, -0.10), M.dark, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door, hinged on its left edge (seen from the front), the grip by the controls
    with G.new_parts() as parts:
        F.box('micro_door', 0.38, 0.28, 0.02, (-0.14, 1.1, -0.09), M.charcoal, bevel=0.006)
        F.box('micro_window', 0.28, 0.20, 0.008, (-0.14, 1.11, -0.08), M.glass, bevel=0.004)
        F.box('micro_grip', 0.02, 0.20, 0.02, (-0.31, 1.1, -0.07), M.chrome, bevel=0.004)
    door = G.hinged(G.join(parts, 'micro_door'), F.at(0.05, 1.1, -0.09))
    # the glass turntable inside, with a mark so its turn shows
    with G.new_parts() as parts:
        F.cyl('turntable', 0.13, 0.012, (-0.14, 0.978, -0.27), M.plate, verts=20)
        F.box('turntable_mark', 0.03, 0.006, 0.03, (-0.22, 0.986, -0.27), M.dark, bevel=0.002)
    turn = G.join(parts, 'turntable', origin=F.at(-0.14, 0.978, -0.27))
    G.settle()
    G.swing(door, 'door_open', -100, seconds=1.0, axis='Z')
    G.spin(turn, 'turntable', turns=1, seconds=2.0)


# --- small appliances: a coffee machine, a breakfast counter and two dishwashers ---------------------

def _counter(F, w=1.0, body=None, top=None):
    """A base unit like the old part's `k_base`: a plinth, a carcass and a worktop to 0.95, with a
    drawer over two doors on the front (the front face at depth 0.14)."""
    body, top = body or M.cab, top or M.worktop
    F.box('plinth', w - 0.04, 0.10, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.005)
    F.box('carcass', w - 0.004, 0.82, 0.60, (0, 0.51, -0.16), body, bevel=0.01)
    F.box('worktop', w, 0.04, 0.66, (0, 0.93, -0.15), top, bevel=0.006)
    F.box('drawer', w - 0.10, 0.14, 0.02, (0, 0.85, 0.155), body, bevel=0.006)
    F.box('drawer_pull', 0.18, 0.02, 0.02, (0, 0.85, 0.175), M.chrome, bevel=0.004)
    for s in (-1, 1):
        F.box(f'door{s}', w / 2 - 0.06, 0.72, 0.02, (s * w / 4, 0.48, 0.155), body, bevel=0.006)
        F.box(f'door_pull{s}', 0.02, 0.12, 0.02, (s * (w / 4 - 0.16), 0.80, 0.175), M.chrome, bevel=0.004)


def _stream(F, name, x, top_y, z, bottom_y, material, r=0.012):
    """A thin stream from (x, top_y, z) down to bottom_y, joined with its origin at the top and scaled
    to nothing along its length, so it is hidden until `grow` runs the clip (the `water_on` way)."""
    length = max(0.05, top_y - bottom_y)
    with G.new_parts() as parts:
        F.cyl(name, r, length, (x, (top_y + bottom_y) / 2, z), material, verts=8)
    obj = G.join(parts, name, origin=F.at(x, top_y, z))
    obj.scale[2] = 0.001  # hidden at rest; `grow` runs it out along its length
    return obj


# --- coffee machine: 1x1, a counter with an espresso machine on the worktop ----------------------------
def coffeeMachine():
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _counter(F)
        # the machine: a body, a warming top, a group head, a drip tray, a gauge and a bean hopper
        F.box('machine', 0.34, 0.30, 0.32, (-0.14, 1.1, -0.26), M.charcoal, bevel=0.03)
        F.box('warming', 0.34, 0.03, 0.32, (-0.14, 1.265, -0.26), M.steel, bevel=0.008)
        F.cyl('group', 0.04, 0.06, (-0.14, 1.18, -0.11), M.steel, verts=14)
        F.box('spout', 0.022, 0.022, 0.022, (-0.14, 1.145, -0.11), M.steel, bevel=0.004)
        F.box('drip', 0.30, 0.02, 0.26, (-0.14, 0.955, -0.10), M.steel, bevel=0.004)
        F.cyl('gauge', 0.03, 0.012, (-0.06, 1.19, -0.096), M.steel, rot=(90, 0, 0), verts=14)
        F.cone('hopper', 0.055, 0.045, 0.12, (-0.06, 1.34, -0.30), M.bean, verts=14)
    G.join(still, 'body', origin=(0, 0, 0))
    # the cup, on the drip tray's front edge, sliding back under the spout for `brew`
    with G.new_parts() as parts:
        F.cyl('cup', 0.035, 0.05, (-0.14, 0.99, 0.0), M.cup, verts=14)
        F.box('cup_handle', 0.012, 0.03, 0.02, (-0.095, 0.995, 0.0), M.cup, bevel=0.004)
    cup = G.join(parts, 'cup', origin=F.at(-0.14, 0.965, 0.0))
    stream = _stream(F, 'stream', -0.14, 1.134, -0.11, 0.99, M.coffee, r=0.012)
    G.settle()
    G.slide(cup, 'brew', (0, 0.11, 0), seconds=0.6)
    G.grow(stream, 'brew', seconds=0.4)


# --- breakfast counter: 1x1, a counter with a toaster and a blender on the worktop ---------------------
def breakfastCounter():
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _counter(F)
        # the toaster
        F.box('toaster', 0.28, 0.18, 0.16, (0.30, 1.04, -0.30), M.toaster, bevel=0.05)
        for s in (-1, 1):
            F.box(f'slot{s}', 0.10, 0.012, 0.03, (0.30, 1.127, -0.30 + s * 0.035), M.dark, bevel=0.004)
        # the blender: a base, a jug and a fruit bowl beside them
        F.box('blender', 0.16, 0.12, 0.16, (0.0, 1.01, -0.32), M.charcoal, bevel=0.03)
        F.cone('jug', 0.06, 0.075, 0.26, (0.0, 1.2, -0.32), M.jug, verts=16)
        F.cyl('bowl', 0.13, 0.06, (-0.28, 0.98, 0.05), M.jug, verts=16)
        F.sphere('fruit1', 0.04, (-0.30, 1.03, 0.02), M.fruit, segments=10, rings=8)
        F.sphere('fruit2', 0.038, (-0.24, 1.03, 0.08), M.toaster, segments=10, rings=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # the toaster's carriage, its lever and its toast standing up at rest: it goes down for toast, and
    # coming back when the use ends pops the toast up (one part, so the smoothie's swirl has a draw call)
    with G.new_parts() as parts:
        F.box('lever', 0.02, 0.03, 0.03, (0.45, 1.05, -0.30), M.dark, bevel=0.006)
        for s in (-1, 1):
            F.box(f'toast{s}', 0.10, 0.12, 0.03, (0.30, 1.11, -0.30 + s * 0.035), M.fruit, bevel=0.01)
    toast = G.join(parts, 'toast', origin=F.at(0.30, 1.05, -0.30))
    # the blender's lid, lifting off for `blend`
    with G.new_parts() as parts:
        F.cyl('lid', 0.078, 0.03, (0.0, 1.345, -0.32), M.charcoal, verts=16)
        F.cyl('lid_knob', 0.02, 0.02, (0.0, 1.37, -0.32), M.charcoal, verts=10)
    lid = G.join(parts, 'lid', origin=F.at(0.0, 1.33, -0.32))
    # the smoothie at the jug's brim, under the lid, with a streak so its swirl shows once the lid is up
    with G.new_parts() as parts:
        F.cyl('smoothie', 0.07, 0.004, (0.0, 1.332, -0.32), M.fruit, verts=16)
        F.box('smoothie_streak', 0.05, 0.002, 0.012, (0.02, 1.335, -0.32), M.toaster, bevel=0.001)
    swirl = G.join(parts, 'smoothie', origin=F.at(0.0, 1.332, -0.32))
    G.settle()
    G.slide(toast, 'toast_down', (0, 0, -0.05), seconds=0.5)
    G.slide(lid, 'blend', (0, 0, 0.10), seconds=0.8)
    G.spin(swirl, 'blades', turns=1, seconds=1.0)


# --- dishwashers: a door dropping open over a sliding lower rack ---------------------------------------
def _dishwasher(F, w, h, top_d, top_z, front, door_h, body, panel, top):
    """A dishwasher: a tub whose door drops on its bottom hinge and whose lower rack slides out."""
    with G.new_parts() as still:
        F.box('plinth', w - 0.04, 0.10, top_d - 0.10, (0, 0.05, top_z - 0.03), M.dark, bevel=0.005)
        F.box('carcass', w - 0.02, h - 0.14, top_d - 0.06, (0, h / 2 - 0.02, top_z - 0.01), body, bevel=0.012)
        face = top_z - 0.01 + (top_d - 0.06) / 2 + 0.005               # 5 mm proud of the carcass, inside the door
        F.box('tub', w - 0.14, door_h - 0.04, top_d - 0.12, (0, 0.12 + door_h / 2, face - (top_d - 0.12) / 2), M.dark, bevel=0.006)
        F.box('worktop', w, 0.04, top_d, (0, h - 0.02, top_z), top, bevel=0.006)
        F.box('strip', w - 0.10, 0.09, 0.02, (0, h - 0.12, front - 0.01), M.dark, bevel=0.004)
        F.sphere('button', 0.012, (w / 2 - 0.14, h - 0.12, front + 0.002), M.display, segments=8, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door, hinged on its bottom edge (its bottom at y 0.12)
    with G.new_parts() as parts:
        F.box('door', w - 0.10, door_h, 0.02, (0, 0.12 + door_h / 2, front), panel, bevel=0.01)
        F.box('handle', w - 0.50, 0.02, 0.03, (0, 0.12 + door_h - 0.04, front + 0.015), M.chrome, bevel=0.006)
    door = G.hinged(G.join(parts, 'door'), F.at(0, 0.12, front))
    # the lower rack, sliding out to the front
    with G.new_parts() as parts:
        F.box('rack', w - 0.16, 0.06, top_d - 0.26, (0, 0.30, top_z - 0.01), M.tray, bevel=0.004)
        for s in (-1, 1):
            F.box(f'rack_side{s}', 0.02, 0.10, top_d - 0.26, (s * (w / 2 - 0.09), 0.34, top_z - 0.01), M.tray, bevel=0.003)
    rack = G.join(parts, 'rack', origin=F.at(0, 0.30, top_z - 0.01))
    G.settle()
    G.swing(door, 'door_open', 85, seconds=SECONDS, axis='X')
    G.slide(rack, 'door_open', (0, -0.30, 0), seconds=SECONDS)


def dishwasher():
    F = G.GameFrame(shift=0.1375)
    _dishwasher(F, w=1.0, h=0.95, top_d=0.66, top_z=-0.15, front=0.175, door_h=0.72,
                body=M.cab, panel=M.steel, top=M.cab)


def dishwasherCompact():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    _dishwasher(F, w=1.0, h=0.95, top_d=0.79, top_z=-0.085, front=0.25, door_h=0.66,
                body=M.cab, panel=M.steel, top=M.cab)


# A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
# its top under its content `surface`.
FAMILY = G.Family(
    'kitchen_cooking', M,
    [G.Model('stoveElectric', stoveElectric, size=(0.60, 0.98, 0.661), key='model.stoveElectric', centre=(0, -0.0995),
             # cook turns the front-left knob and its ring glows for the whole use; bake drops the oven door
             # for HOLD s. By the cooking step's tag too (`stir` on the hob, `bake` in the oven), for a meal
             # started at the fridge (snapshot.rs: no interaction at a station that isn't the recipe's origin).
             motions={'uses': {'cook': 'burner_on', 'stir': 'burner_on', 'bake': {'clip': 'oven_open', 'hold': HOLD}}},
             paint=[('enamel', 'Body'), ('chrome', 'Knobs'), ('hob', 'Hob')]),
     G.Model('stoveGas', stoveGas, size=(0.92, 1.113, 0.716), key='model.stoveGas', centre=(0, -0.078),
             # cook and gourmet (and any `stir` step) light the front burner; bake drops the oven door
             motions={'uses': {'cook': 'burner_on', 'gourmet': 'burner_on', 'stir': 'burner_on',
                               'bake': {'clip': 'oven_open', 'hold': HOLD}}},
             paint=[('enamel', 'Body'), ('chrome', 'Knobs'), ('grate', 'Hob')]),
     G.Model('wallOven', wallOven, size=(0.70, 2.11, 0.696), key='model.wallOven', centre=(0, -0.077),
             # roast and bake (the interactions, and any `bake` step) drop the upper oven door and pull its tray out
             motions={'uses': {'roast': 'oven_open', 'bake': 'oven_open'}, 'hold': HOLD},
             paint=[('cab', 'Tower'), ('chrome', 'Handles')]),
     G.Model('rangeCooker', rangeCooker, size=(1.58, 1.13, 0.755), key='model.rangeCooker', centre=(0, -0.0525),
             # cook and gourmet (and any `stir` step) light a burner; bake drops the left oven door
             motions={'uses': {'cook': 'burner_on', 'gourmet': 'burner_on', 'stir': 'burner_on',
                               'bake': {'clip': 'oven_open', 'hold': HOLD}}},
             paint=[('enamel2', 'Body'), ('chrome', 'Knobs & rail'), ('grate', 'Hob')]),
     G.Model('cooktopCounter', cooktopCounter, size=(1.0, 0.955, 0.66), key='model.cooktopCounter', centre=(0, -0.10),
             # cook (and any `stir` step) turns a knob and the front-right ring glows red
             motions={'uses': {'cook': 'burner_on', 'stir': 'burner_on'}},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('hob', 'Hob')]),
     G.Model('microwave', microwave, size=(1.0, 1.25, 0.66), key='model.microwave', centre=(0, -0.095),
             # reheat (and a meal's `bake` step) opens the door to put the dish in, shuts it, then turns the
             # plate while it heats: from the door shut (TURNTABLE), round
             # and round (`every`, a turn each 2 s)
             motions={'uses': {use: ['door_open', TURNTABLE] for use in ('reheat', 'bake')}, 'hold': 1.2},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('charcoal', 'Microwave')]),
     G.Model('coffeeMachine', coffeeMachine, size=(1.0, 1.40, 0.66), key='model.coffeeMachine', centre=(0, -0.097),
             # espresso slides a cup under the spout and grows a stream of coffee
             motions={'uses': {'espresso': 'brew'}, 'hold': 2.0},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('charcoal', 'Machine')]),
     G.Model('breakfastCounter', breakfastCounter, size=(1.0, 1.38, 0.66), key='model.breakfastCounter', centre=(0, -0.097),
             # toast presses the lever and the toast down (back up, popping, after `hold`); smoothie lifts the
             # lid and swirls the smoothie for as long as it blends
             motions={'uses': {'toast': 'toast_down', 'smoothie': 'blend'}, 'loops': {'smoothie': 'blades'}, 'hold': 2.0},
             paint=[('cab', 'Counter'), ('toaster', 'Toaster'), ('charcoal', 'Blender')]),
     G.Model('dishwasher', dishwasher, size=(1.0, 0.95, 0.685), key='model.dishwasher', centre=(0, -0.088),
             # dishes (loading it) and clearing away at it (the `wash` tag) drop the door and slide the lower
             # rack out, held open while it is loaded
             motions={'uses': {'dishes': 'door_open', 'wash': 'door_open'}},
             paint=[('cab', 'Body'), ('chrome', 'Handle'), ('steel', 'Panel')]),
     G.Model('dishwasherCompact', dishwasherCompact, size=(1.0, 0.95, 0.66), key='model.dishwasherCompact', centre=(0, -0.10),
             # dishes and clearing away, as the dishwasher
             motions={'uses': {'dishes': 'door_open', 'wash': 'door_open'}},
             paint=[('cab', 'Body'), ('chrome', 'Handle'), ('steel', 'Panel')])],
    comment='Blender-built kitchen cookers (tools/art/blender/families/kitchen_cooking.py). Listed in the '
            'base manifest\'s packs. Each cooker is a box on the floor with a hob on top and an oven '
            'below: its door drops on a bottom hinge (`oven_open`) while a resident bakes, and a knob '
            'turns with a ring of flame on the gas ones (`burner_on`) while they cook. The microwave '
            'swings its door (`door_open`) and turns its turntable (`turntable`).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
