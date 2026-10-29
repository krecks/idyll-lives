# The kitchen's cookers (K2): the electric cooker, the gas range, the oven tower, the range cooker, the
# hob counter and the microwave counter; and the small appliances (K3). Notes: kitchen_cooking.md.
#
# A cooker is a box on the floor with a hob on top and an oven below; its door drops on a bottom hinge,
# and a knob turns (with a ring of flame on the gas ones) while a resident cooks. The parts are placed
# in game axes (`F`): the old depths are shifted back by their depth centre, so
# the model is centred on its tile (see GameFrame).
#
# Run:  cargo art kitchen_cooking [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402

import gamekit as G  # noqa: E402

M = G.Palette(
    enamel=('#F4F3EF', 'gloss'),     # a white cooker body; the main part the player colours (`paint`)
    enamel2=('#EFE5D3', 'gloss'),    # a cream range cooker
    cab=('#EDEBE6', 'satin'),        # cabinet fronts (the oven tower, the hob counter)
    worktop=('#E8E6E1', 'gloss'),    # a counter worktop
    dark=('#2B2C2E', 'satin'),       # oven fronts, plinths and control strips
    hob=('#1B1E22', 'gloss'),        # a ceramic/induction hob and an oven window (opaque, glossy black)
    glass=('#1D2228', 'glass'),      # the microwave's see-through window
    plate=('#E9EAEC', 'gloss'),      # the microwave's turntable plate; plates in a dishwasher
    chrome=('#D0D3D6', 'chrome'),    # handles and knobs
    grate=('#22252A', 'satin'),      # burner heads and grates
    flame=('#3E7CC2', 'gloss'),      # a gas flame ring (gloss, not emissive)
    heat=('#D2402A', 'gloss'),       # a ceramic ring glowing hot (gloss red, so it reads by day)
    charcoal=('#33363A', 'satin'),   # the microwave's body
    display=('#5EC28B', 'gloss'),    # the microwave's display
    tray=('#8E9196', 'metal'),       # an oven tray and racks
    pot=('#6E7377', 'metal'),        # a stock pot
    pan=('#4A4E52', 'satin'),        # a frying pan
    steel=('#C9CBCC', 'metal'),      # a stainless appliance front
    toaster=('#C05B4D', 'gloss'),    # a toaster's body
    jug=('#DCE6EA', 'gloss'),        # a blender's jug (opaque, so it costs no extra draw call)
    bean=('#5A3A26', 'gloss'),       # a coffee machine's bean hopper
    cup=('#F2F1EE', 'gloss'),        # a cup
    coffee=('#9A6B3F', 'gloss'),     # the stream of coffee
    fruit=('#E0A33E', 'gloss'),      # a piece of fruit in the bowl
    bread=('#D29A55', 'satin'),      # bread rolls on the oven tray, toast's crust
    crumb=('#EED9A8', 'satin'),      # toast's face
    banana=('#E6C65A', 'satin'),     # a banana in the fruit bowl
    towel=('#CFDCE6', 'satin'),      # the range cooker's tea towel (plain: no draw call of its own)
)

SECONDS = 1.4   # a door drops, a drawer slides or a knob turns in this long
# catalog2's 0.79-deep counter numbers drawn 0.66 deep against the wall (kitchen_counters.py COUNTER66)
COUNTER66 = ((-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23))
HOLD = 3.0      # seconds a door stays open once a use starts
# the microwave's plate turns once its door has shut (`door_open` opens in 1.0 s and plays back from its hold, 1.2 s)
TURNTABLE = {'clip': 'turntable', 'delay': 2.2, 'every': 1.9}   # (a hair under its 2 s turn: no pause between turns)


# --- parts every model here shares ---------------------------------------------------------------------------

def _pull(F, name, a, b, face, material=None, out=0.034, w=0.016, t=0.02, r=0.007, verts=8):
    """A bar handle in one piece from face point `a` (x, height) to `b`: out from the face at game depth `face` on
    rounded stand-offs, along, and back in, its front `out` proud; a rounded `w` × `t` section (`r` None: a round bar
    of radius w / 2)."""
    (x0, y0), (x1, y1) = a, b
    n = math.hypot(x1 - x0, y1 - y0)
    ux, uy = (x1 - x0) / n, (y1 - y0) / n
    e, o = min(0.012, n / 4), face + out - (w if r is None else t) / 2
    pts = [(x0, y0, face - 0.004), (x0, y0, o - e), (x0 + ux * e, y0 + uy * e, o), (x1 - ux * e, y1 - uy * e, o),
           (x1, y1, o - e), (x1, y1, face - 0.004)]
    kw = dict(radius=w / 2, verts=verts) if r is None else dict(profile=G.rounded_rect(w, t, r, corner=1))
    return F.sweep(name, pts, kw.pop('radius', None), material or M.chrome, up=(uy, 0, ux), resolution=2, angle=60, **kw)


def _fillet(points, r, steps=3):
    """A polyline's corners rounded by arcs about `r` long each way (a quadratic curve through `steps` - 1 more
    points), for `kit.tube(..., round=1)`: bent tube whose long straights don't make the curve overshoot its corners,
    as the Catmull-Rom rounding does between a long span and a short one."""
    out = [points[0]]
    for a, p, b in zip(points, points[1:], points[2:]):
        da = [p[i] - a[i] for i in range(3)]
        db = [b[i] - p[i] for i in range(3)]
        la, lb = math.sqrt(sum(c * c for c in da)), math.sqrt(sum(c * c for c in db))
        k = min(r, la / 2, lb / 2)
        s = [p[i] - da[i] / la * k for i in range(3)]
        e = [p[i] + db[i] / lb * k for i in range(3)]
        for j in range(steps + 1):
            t = j / steps
            out.append(tuple((1 - t) ** 2 * s[i] + 2 * (1 - t) * t * p[i] + t * t * e[i] for i in range(3)))
    return out + [points[-1]]


def _shaker(F, name, w, h, at, material, t=0.02, rail=0.06, r=0.004):
    """A cabinet door or drawer front: a frame (stiles and rails, one slab with a hole) round a panel set 6 mm back,
    lightly puffed so the baked shading darkens only its border. `at` is the middle of the slab."""
    x, y, z = at
    hole = G.rounded_rect(w - 2 * rail, h - 2 * rail, 0.003, corner=1)
    F.extrude(f'{name}_frame', G.rounded_rect(w, h, r, corner=1), t, at, material, holes=[hole], bevel=0.003,
              bevel_segments=1)
    F.soft(f'{name}_panel', w - 2 * rail - 0.002, h - 2 * rail - 0.002, t * 0.6, (x, y, z + t * 0.2 - 0.006), material,
           puff=0.002, round=0.003, round_segments=1, flat=2)


def _knob(F, name, x, y, z, r=0.022, d=0.03, material=None, mark=True):
    """A control knob facing the front, `d` deep about game depth `z`: a skirt on the panel, a grip and a dished
    cap, with a pointer line so a turn shows."""
    prof = [(0.0, 0.0), (r * 1.2, 0.0), (r * 0.9, d * 0.3), (r * 0.88, d * 0.92), (r * 0.74, d), (0.0, d * 0.95)]
    G.revolve(name, prof, at=F.at(x, y, z - d / 2), material=material or M.chrome, segments=10, rot=(90, 0, 0))
    if mark:
        F.box(f'{name}_mark', 0.004, r * 0.75, 0.004, (x, y + r * 0.42, z + d / 2 - 0.0005), M.dark)


def _knob_obj(F, name, x, y, z, r=0.022, d=0.03):
    """A knob as its own object, its origin at its centre, so a clip turns it in place."""
    with G.new_parts() as parts:
        _knob(F, name, x, y, z, r, d)
    return G.join(parts, name, origin=F.at(x, y, z))


def _flame(F, name, x, y, z, r):
    """A gas flame: a ring of blue tongues round the burner, built full size but scaled to nothing, grown up in every
    axis while the burner is on (the `water_on` way, so it is hidden at rest)."""
    with G.new_parts() as parts:
        G.torus(name, r, 0.009, at=F.at(x, y, z), material=M.flame, major_segments=14, minor_segments=5)
        for i in range(12):
            a = math.tau * i / 12
            F.cone(f'{name}_tongue{i}', 0.008, 0.0, 0.03, (x + (r + 0.004) * math.cos(a), y + 0.016,
                                                          z + (r + 0.004) * math.sin(a)), M.flame, verts=5)
    obj = G.join(parts, name, origin=F.at(x, y, z))
    obj.scale = (0.001, 0.001, 0.001)
    return obj


def _light(flame):
    """Grows a flame ring in all three axes, so `burner_on` shows it from nothing."""
    for axis in 'XYZ':
        G.grow(flame, 'burner_on', seconds=0.5, axis=axis)


def _burner(F, name, x, z, y, r=0.07):
    """A gas burner on a hob at height `y`: a spreader bowl, a black cap, and a cast-iron pan support (a square of bar
    with four fingers reaching in), its top 2 cm up, where a pan stands."""
    bx, by, bz = F.at(x, y - 0.01, z)
    G.revolve(f'{name}_head', [(r * 0.85, 0.0), (r * 0.85, 0.008), (r * 0.62, 0.016),
                               (0.0, 0.016)], at=(bx, by, bz), material=M.grate, segments=10)
    G.revolve(f'{name}_cap', [(r * 0.6, 0.0), (r * 0.6, 0.004), (r * 0.5, 0.008), (0.0, 0.009)],
              at=(bx, by, bz + 0.016), material=M.pan, segments=10)
    side = 2 * r + 0.06
    loop = G.rounded_rect(side, side, 0.02, corner=1)
    F.sweep(f'{name}_grate', [(x + u, y + 0.016, z + v) for u, v in loop], 0.005, M.grate, verts=4, closed=True)
    for i in range(4):
        a = math.tau * i / 4 + math.pi / 4
        c, s = math.cos(a), math.sin(a)
        reach = side / 2 * 1.38
        F.sweep(f'{name}_finger{i}', [(x + c * reach * 0.98, y + 0.016, z + s * reach * 0.98),
                                      (x + c * r * 0.72, y + 0.016, z + s * r * 0.72)], 0.005, M.grate, verts=4)


def _hot_ring(F, name, x, y, z, r):
    """A ceramic ring glowing red: a thin disc on the glass, hidden at rest (scaled to nothing) and grown
    by `_light` while the ring is on, as a gas flame is."""
    with G.new_parts() as parts:
        F.cyl(name, r, 0.003, (x, y, z), M.heat, verts=20)
    obj = G.join(parts, name, origin=F.at(x, y, z))
    obj.scale = (0.001, 0.001, 0.001)
    return obj


def _hob_ring(F, name, x, z, y, r=0.07, inner=True):
    """A cooking zone printed on a ceramic/induction hob: a ring (and a smaller one inside it) on the glass."""
    G.torus(name, r, 0.0035, at=F.at(x, y, z), material=M.grate, major_segments=20, minor_segments=4)
    if inner:
        G.torus(f'{name}_in', r * 0.55, 0.003, at=F.at(x, y, z), material=M.grate, major_segments=14, minor_segments=4)


def _hollow(obj, F, name, w, h, d, at, material=None):
    """Cuts an oven's cavity out of `obj` (a carcass box) through its front: a w × h × d box at game `at`, its walls
    taking the cavity's dark enamel (or `material`)."""
    cutter = F.box(name, w, h, d, at, material or M.dark)
    b = obj.modifiers.new('cut', 'BOOLEAN')
    b.operation, b.object, b.solver, b.material_mode = 'DIFFERENCE', cutter, 'EXACT', 'TRANSFER'
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    for m in list(obj.modifiers):
        bpy.ops.object.modifier_apply(modifier=m.name)
    bpy.data.objects.remove(cutter)
    return obj


def _rack(F, name, x, y, z, w, d, bars=5, material=None):
    """A wire rack lying at height `y`: a rounded frame of rod and `bars` rods across it, front to back."""
    loop = G.rounded_rect(w, d, 0.015, corner=2)
    F.sweep(name, [(x + u, y, z + v) for u, v in loop], 0.003, material or M.tray, verts=4, closed=True)
    for i in range(bars):
        u = -w / 2 + w * (i + 1) / (bars + 1)
        F.sweep(f'{name}_bar{i}', [(x + u, y, z - d / 2), (x + u, y, z + d / 2)], 0.0025, material or M.tray, verts=4,
                caps=False)


def _oven_door(F, name, x, w, y, z, hinge_y, h=0.50, out=0.0345):
    """An oven door: a dark front framing a window of black glass, a bar handle on posts along its top rail, an inner
    pane behind (what shows when it's down); hinged along its bottom edge. Returns it; the caller swings it once the
    model has settled."""
    with G.new_parts() as parts:
        window = [(u, v + 0.01) for u, v in G.rounded_rect(w - 0.14, 0.30, 0.02, corner=2)]
        F.extrude(f'{name}_front', G.rounded_rect(w, h, 0.012, corner=2), 0.025, (x, y, z), M.dark, holes=[window],
                  bevel=0.004, bevel_segments=1)
        F.box(f'{name}_window', w - 0.13, 0.31, 0.008, (x, y + 0.01, z - 0.006), M.hob)
        F.box(f'{name}_inner', w - 0.08, h - 0.08, 0.006, (x, y, z - 0.014), M.tray)
        _pull(F, f'{name}_handle', (x - (w / 2 - 0.06), y + 0.205), (x + (w / 2 - 0.06), y + 0.205), z + 0.0125,
              out=out, w=0.022, r=None, verts=8)
    return G.hinged(G.join(parts, name), F.at(x, hinge_y, z))


def _oven(F, carcass, name, x, y, z, w, h, d):
    """An oven's inside, cut out of the carcass behind its door (front at depth `z`): dark enamel walls, a wire rack
    on runners and a lamp in the back corner."""
    _hollow(carcass, F, f'{name}_cavity', w, h, d, (x, y, z - d / 2 + 0.002))
    _rack(F, f'{name}_rack', x, y - h * 0.12, z - d / 2, w - 0.02, d - 0.04)
    for s in (-1, 1):
        F.box(f'{name}_runner{s}', 0.012, 0.008, d - 0.04, (x + s * (w / 2 - 0.006), y - h * 0.12 - 0.006, z - d / 2),
              M.tray)
    F.cyl(f'{name}_lamp', 0.02, 0.008, (x - w / 2 + 0.05, y + h / 2 - 0.05, z - d + 0.004), M.plate, verts=10,
          rot=(90, 0, 0))


def _strip(F, name, w, y, z, h=0.06, x=0.0, display=None):
    """A control fascia `w` × `h` on the front at depth `z` (its back), its edges rounded; `display`: a small clock
    display at that x."""
    F.extrude(name, G.rounded_rect(w, h, 0.008, corner=2), 0.012, (x, y, z + 0.006), M.dark, bevel=0.003,
              bevel_segments=1)
    if display is not None:
        F.box(f'{name}_display', 0.07, 0.025, 0.004, (display, y, z + 0.013), M.hob, bevel=0.002, bevel_segments=1)
        F.box(f'{name}_digits', 0.04, 0.012, 0.002, (display, y, z + 0.0155), M.display)


def _drawer_front(F, name, w, h, at, material, pull=0.3):
    """A storage drawer's front with a long bar pull along its top."""
    x, y, z = at
    F.extrude(name, G.rounded_rect(w, h, 0.008, corner=2), 0.02, at, material, bevel=0.004, bevel_segments=1)
    _pull(F, f'{name}_pull', (x - pull / 2, y + h / 2 - 0.035), (x + pull / 2, y + h / 2 - 0.035), z + 0.01, out=0.02,
          w=0.014, r=None, verts=6)


def _stock_pot(F, name, x, y, z, r, h, material):
    """A stock pot standing on game height `y`: a rolled rim, loop handles each side, a lid with a knob."""
    bx, by, bz = F.at(x, y, z)
    G.revolve(name, [(0.0, 0.0), (r * 0.96, 0.0), (r, 0.01), (r, h - 0.006), (r * 1.03, h), (r * 0.97, h),
                     (r * 0.97, h * 0.6), (0.0, h * 0.6)], at=(bx, by, bz), material=material, segments=16)
    G.revolve(f'{name}_lid', [(0.0, 0.0), (r * 1.02, 0.0), (r * 0.98, 0.012), (r * 0.5, 0.022), (0.0, 0.024)],
              at=(bx, by, bz + h), material=material, segments=16)
    F.sphere(f'{name}_knob', 0.014, (x, y + h + 0.03, z), M.pan, segments=8, rings=5, scale=(1, 1, 0.7))
    for s in (-1, 1):
        F.sweep(f'{name}_handle{s}', [(x + s * r * 0.98, y + h - 0.035, z - 0.03), (x + s * (r + 0.025), y + h - 0.03, z - 0.025),
                                      (x + s * (r + 0.025), y + h - 0.03, z + 0.025), (x + s * r * 0.98, y + h - 0.035, z + 0.03)],
                0.005, material, verts=6, resolution=2)


def _frying_pan(F, name, x, y, z, r, handle_to):
    """A frying pan on game height `y`: a flared shallow pan and a long handle reaching to game depth `handle_to`."""
    bx, by, bz = F.at(x, y, z)
    G.revolve(name, [(0.0, 0.0), (r * 0.8, 0.0), (r * 0.92, 0.008), (r, 0.045), (r * 0.96, 0.045), (r * 0.88, 0.012),
                     (0.0, 0.012)], at=(bx, by, bz), material=M.pan, segments=14)
    F.sweep(f'{name}_handle', [(x + r * 0.97, y + 0.03, z), (x + r + 0.04, y + 0.045, z + 0.02),
                               (x + r + 0.04, y + 0.06, handle_to - 0.02), (x + r + 0.04, y + 0.062, handle_to)],
            None, M.pan, profile=G.rounded_rect(0.022, 0.012, 0.005, corner=1), up=(0, 1, 0), resolution=2,
            scale=[0.7, 0.9, 1.0, 1.0])


def _oven_face(F, name, x, w, y, z, h=0.50, out=0.0345):
    """An oven's still front (one that never opens): as `_oven_door`, joined into the body."""
    window = [(u, v + 0.01) for u, v in G.rounded_rect(w - 0.14, 0.30, 0.02, corner=2)]
    F.extrude(f'{name}_front', G.rounded_rect(w, h, 0.012, corner=2), 0.025, (x, y, z), M.dark, holes=[window],
              bevel=0.004, bevel_segments=1)
    F.box(f'{name}_window', w - 0.13, 0.31, 0.008, (x, y + 0.01, z - 0.006), M.hob)
    _pull(F, f'{name}_handle', (x - (w / 2 - 0.06), y + 0.205), (x + (w / 2 - 0.06), y + 0.205), z + 0.0125,
          out=out, w=0.022, r=None, verts=8)


# --- electric cooker: 1x1, four ceramic rings and an oven ---------------------------------------------
def stoveElectric():
    """A freestanding electric cooker: a black ceramic hob with four printed zones, a rounded splash guard, a control
    fascia with knobs and a clock, an oven (a window in its door, a rack and a lamp inside) over a storage drawer."""
    F = G.GameFrame(shift=0.1295)
    with G.new_parts() as still:
        F.box('plinth', 0.56, 0.05, 0.54, (0, 0.025, -0.17), M.dark, bevel=0.005)
        carcass = F.box('carcass', 0.60, 0.82, 0.60, (0, 0.46, -0.16), M.enamel, bevel=0.01)
        _oven(F, carcass, 'oven', 0, 0.43, 0.14, 0.48, 0.38, 0.46)
        F.box('hob', 0.60, 0.02, 0.60, (0, 0.88, -0.16), M.hob, bevel=0.006, bevel_segments=2)
        F.box('back', 0.60, 0.09, 0.10, (0, 0.935, -0.41), M.enamel, bevel=0.02, bevel_segments=3)
        F.box('back_vent', 0.4, 0.004, 0.025, (0, 0.9775, -0.41), M.dark)
        _strip(F, 'strip', 0.56, 0.845, 0.136, display=0.0)
        for i, x in enumerate((-0.07, 0.07, 0.21), start=1):
            _knob(F, f'knob{i}', x, 0.845, 0.155)
        _drawer_front(F, 'drawer', 0.56, 0.10, (0, 0.12, 0.15), M.enamel)
        for i, (x, z) in enumerate(((-0.16, -0.30), (0.16, -0.30), (-0.16, -0.05), (0.16, -0.05))):
            _hob_ring(F, f'ring{i}', x, z, 0.8915, r=0.07 if i != 1 else 0.06, inner=i in (0, 2))
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
    """A gas range: a dark hob with four burners under cast-iron pan supports (a stock pot and a frying pan on two), a
    splash guard, a control fascia with five knobs, an oven with a window over a storage drawer."""
    F = G.GameFrame(shift=0.112)
    with G.new_parts() as still:
        F.box('plinth', 0.88, 0.06, 0.58, (0, 0.03, -0.17), M.dark, bevel=0.005)
        carcass = F.box('carcass', 0.92, 0.78, 0.64, (0, 0.45, -0.14), M.enamel, bevel=0.01)
        _oven(F, carcass, 'oven', 0, 0.42, 0.18, 0.78, 0.38, 0.5)
        F.box('hob', 0.92, 0.03, 0.64, (0, 0.855, -0.14), M.dark, bevel=0.006, bevel_segments=2)
        F.box('back', 0.92, 0.18, 0.04, (0, 0.96, -0.45), M.enamel, bevel=0.01, bevel_segments=2)
        _strip(F, 'strip', 0.86, 0.80, 0.18, display=None)
        for i, x in enumerate((-0.16, 0.0, 0.16, 0.32), start=1):
            _knob(F, f'knob{i}', x, 0.80, 0.207)
        _drawer_front(F, 'drawer', 0.88, 0.10, (0, 0.11, 0.17), M.enamel, pull=0.4)
        for i, (x, z) in enumerate(((-0.26, -0.30), (0.26, -0.30), (-0.26, -0.02), (0.26, -0.02))):
            _burner(F, f'burner{i}', x, z, 0.87)
        # a stock pot and a frying pan on the pan supports, the pan's handle reaching the model's front
        _stock_pot(F, 'pot', -0.26, 0.891, -0.30, 0.13, 0.18, M.pot)
        _frying_pan(F, 'pan', 0.26, 0.891, -0.02, 0.11, 0.246)
    G.join(still, 'body', origin=(0, 0, 0))
    door = _oven_door(F, 'oven_door', 0, 0.88, 0.42, 0.17, hinge_y=0.17)
    # the knob stands on the fascia, in front of where the old one was: it turns about the same front-to-back axis
    knob = _knob_obj(F, 'knob0', -0.32, 0.80, 0.207)
    G.set_origin(knob, F.at(-0.32, 0.80, 0.165))
    flame = _flame(F, 'flame', -0.26, 0.885, -0.02, 0.075)
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(flame)


# --- oven tower: 1x1, two built-in ovens in a tall cabinet ---------------------------------------------
def wallOven():
    """A tall cabinet under a moulded cornice holding two ovens and a control panel between them, a shaker door below
    and one above; the upper oven's door drops and its tray of bread rolls slides out."""
    F = G.GameFrame(shift=0.102)
    with G.new_parts() as still:
        F.box('plinth', 0.66, 0.06, 0.54, (0, 0.03, -0.16), M.dark, bevel=0.005)
        cabinet = F.box('cabinet', 0.70, 1.98, 0.60, (0, 1.05, -0.15), M.cab, bevel=0.006)
        _oven(F, cabinet, 'oven', 0, 1.62, 0.15, 0.52, 0.40, 0.46)
        F.box('cornice', 0.66, 0.05, 0.60, (0, 2.065, -0.16), M.cab)
        cove = [(0.0, -0.01), (0.0, 0.004)] + [(0.004 + 0.056 * math.sin(math.radians(a)),
                                                0.004 + 0.016 * (1 - math.cos(math.radians(a)))) for a in range(0, 91, 18)]
        cove += [(0.07, 0.02), (0.07, -0.01)]
        F.sweep('cornice_moulding', [(0.33, 2.04, -0.46), (0.33, 2.04, 0.14), (-0.33, 2.04, 0.14), (-0.33, 2.04, -0.46)],
                None, M.cab, profile=cove, up=(0, 0, 1), angle=50)
        # the lower oven (it stays shut), the controls between the two, the cabinet doors
        _oven_face(F, 'oven_low', 0, 0.60, 0.98, 0.16, out=0.0735)
        _strip(F, 'controls', 0.60, 1.30, 0.15, h=0.1, display=0.0)
        for s in (-1, 1):
            _knob(F, f'knob{s}', s * 0.2, 1.30, 0.177, r=0.024)
        _shaker(F, 'door_bottom', 0.60, 0.52, (0, 0.40, 0.16), M.cab)
        _pull(F, 'pull_bottom', (0.24, 0.5), (0.24, 0.62), 0.17, out=0.03, w=0.012, t=0.012, r=0.005)
        _shaker(F, 'door_top', 0.60, 0.16, (0, 1.96, 0.16), M.cab, rail=0.04)
        _pull(F, 'pull_top', (0.24, 1.9), (0.24, 2.0), 0.17, out=0.03, w=0.012, t=0.012, r=0.005)
    G.join(still, 'body', origin=(0, 0, 0))
    # the upper oven door (hinged at its bottom) and its tray of rolls
    door = _oven_door(F, 'oven_door', 0, 0.60, 1.62, 0.16, hinge_y=1.37, out=0.0735)
    with G.new_parts() as parts:
        _tray(F, 'tray', 0, 1.585, -0.015, 0.46, 0.31)
        for i in range(6):
            F.soft(f'roll{i}', 0.075, 0.045, 0.06, (-0.13 + 0.13 * (i % 3), 1.613, -0.09 + 0.15 * (i // 3)), M.bread,
                   round=0.02, round_segments=2, puff=0.006, flat=1)
    tray = G.join(parts, 'tray', origin=F.at(0, 1.60, -0.17))
    G.settle()
    G.swing(door, 'oven_open', 85, seconds=SECONDS, axis='X')
    G.slide(tray, 'oven_open', (0, -0.10, 0), seconds=SECONDS)


def _tray(F, name, x, y, z, w, d, h=0.022):
    """A baking tray on game height `y`: a flat floor inside a rolled rim `h` high, in one piece."""
    bx, by, bz = F.at(x, y, z)

    def ring(a, b, t):
        return G.ring(a, b, bz + t, 20, square=10, at=(bx, by))
    G.loft(name, [ring(w / 2 - 0.006, d / 2 - 0.006, 0.0), ring(w / 2, d / 2, h), ring(w / 2 - 0.008, d / 2 - 0.008, h),
                  ring(w / 2 - 0.014, d / 2 - 0.014, 0.004), ring(0.0, 0.0, 0.004)], material=M.tray)


# --- range cooker: 2x1, six burners, two ovens and a chrome rail ---------------------------------------
def rangeCooker():
    """A cream range cooker: six burners under cast-iron supports (a stock pot on one), a tall splash back, a control
    fascia with six knobs, two ovens with framed windows (the left one opens onto a rack), a chrome rail across the
    front with a tea towel over it."""
    F = G.GameFrame(shift=0.1025)
    with G.new_parts() as still:
        F.box('plinth', 1.54, 0.06, 0.6, (0, 0.03, -0.16), M.dark, bevel=0.005)
        carcass = F.box('carcass', 1.58, 0.80, 0.66, (0, 0.46, -0.14), M.enamel2, bevel=0.012)
        _oven(F, carcass, 'oven', -0.40, 0.42, 0.19, 0.62, 0.38, 0.52)
        F.box('hob', 1.58, 0.03, 0.66, (0, 0.875, -0.14), M.dark, bevel=0.006, bevel_segments=2)
        F.box('back', 1.58, 0.24, 0.04, (0, 1.01, -0.46), M.enamel2, bevel=0.012, bevel_segments=2)
        F.box('back_trim', 1.5, 0.012, 0.012, (0, 1.115, -0.436), M.chrome, bevel=0.004, bevel_segments=1)
        _strip(F, 'strip', 1.5, 0.82, 0.19, display=-0.24)
        for i, x in enumerate((-0.36, -0.12, 0.12, 0.36, 0.6), start=1):
            _knob(F, f'knob{i}', x, 0.82, 0.215)
        # the right oven stays shut; the left one drops (built below)
        _oven_face(F, 'oven_right', 0.40, 0.72, 0.42, 0.20)
        for i, (x, z) in enumerate(((-0.55, -0.30), (0.0, -0.30), (0.55, -0.30),
                                    (-0.55, -0.02), (0.0, -0.02), (0.55, -0.02))):
            _burner(F, f'burner{i}', x, z, 0.89)
        _stock_pot(F, 'pot', -0.55, 0.911, -0.30, 0.14, 0.19, M.pot)
        # the chrome rail across the front in one piece, posts and all, and a tea towel hung over it
        G.kit.tube(F, 'rail', _fillet([(-0.7, 0.90, 0.19), (-0.7, 0.92, 0.263), (0.7, 0.92, 0.263), (0.7, 0.90, 0.19)],
                                      0.03), 0.012, M.chrome, verts=10, round=1)
        F.sweep('towel', [(0.24, 0.73, 0.277), (0.24, 0.93, 0.276), (0.24, 0.935, 0.263), (0.24, 0.93, 0.25),
                          (0.24, 0.86, 0.246)], None, M.towel, profile=G.rounded_rect(0.17, 0.006, 0.003, corner=1),
                 up=(1, 0, 0), resolution=2, angle=70)
        for k, y in enumerate((0.76, 0.79)):
            F.box(f'towel_stripe{k}', 0.172, 0.012, 0.007, (0.24, y, 0.277), M.heat)
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
    """An induction hob in a steel trim set into a worktop, its touch controls printed along the front (no pan on it:
    the model is no taller than the hob); a control fascia with three knobs over two deep pan drawers."""
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        F.box('plinth', 0.96, 0.10, 0.64, (0, 0.05, -0.14), M.dark, bevel=0.005)
        F.box('carcass', 1.0, 0.80, 0.70, (0, 0.50, -0.11), M.cab, bevel=0.006)
        F.box('worktop', 1.0, 0.05, 0.79, (0, 0.925, -0.085), M.worktop, bevel=0.008, bevel_segments=2)
        F.box('hob', 0.62, 0.012, 0.50, (0, 0.948, -0.10), M.hob, bevel=0.004, bevel_segments=2)  # 4 mm proud: a pan stands on it
        F.extrude('hob_trim', G.rounded_rect(0.636, 0.43, 0.012, corner=2), 0.006, (0, 0.951, -0.10), M.steel,
                  plane='top', holes=[G.rounded_rect(0.616, 0.41, 0.008, corner=2)])
        for i, (x, z) in enumerate(((-0.16, -0.24), (0.16, -0.24), (-0.16, 0.03), (0.16, 0.03))):
            _hob_ring(F, f'ring{i}', x, z, 0.9525, r=0.075 if i % 3 else 0.065, inner=False)
        # the touch controls: a slider and a row of marks along the front
        F.box('slider', 0.2, 0.001, 0.012, (0, 0.9545, 0.12), M.grate)
        for i in range(4):
            F.box(f'touch{i}', 0.018, 0.001, 0.018, (-0.22 + 0.06 * i + (0.26 if i > 1 else 0), 0.9545, 0.12), M.grate)
        # the fascia with its knobs, and two deep drawers under it
        F.extrude('fascia', G.rounded_rect(0.94, 0.08, 0.008, corner=2), 0.012, (0, 0.84, 0.245), M.cab, bevel=0.003,
                  bevel_segments=1)
        for i, x in enumerate((0.14, -0.14), start=1):
            _knob(F, f'knob{i}', x, 0.84, 0.252)
        for i, (y, h) in enumerate(((0.28, 0.32), (0.62, 0.30))):
            _shaker(F, f'drawer{i}', 0.94, h, (0, y, 0.25), M.cab, rail=0.05)
            _pull(F, f'pull{i}', (-0.15, y + h / 2 - 0.05), (0.15, y + h / 2 - 0.05), 0.256, out=0.016, w=0.012,
                  r=None, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    knob = _knob_obj(F, 'knob0', -0.42, 0.84, 0.252)
    flame = _hot_ring(F, 'hot_ring', 0.16, 0.9545, -0.24, 0.066)
    G.settle()
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    _light(flame)


# --- microwave counter: 1x1, a microwave on a small base unit ------------------------------------------
def microwave():
    """A microwave on a base unit (a drawer over two doors): a rounded charcoal body with vents down its side, a
    control column with a display, buttons and a dial, a door with a window and a grip; inside, a bowl of soup on the
    turntable over its roller ring."""
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _counter(F)
        # the microwave: a rounded body, hollowed through the front of its left part (the control column on the right
        # as the viewer sees it: game -x)
        body = F.soft('micro_body', 0.50, 0.29, 0.36, (-0.20, 1.105, -0.28), M.charcoal, round=0.015, round_segments=2)
        _hollow(body, F, 'micro_cavity', 0.36, 0.25, 0.36, (-0.15, 1.10, -0.255), M.plate)
        for i in range(8):
            F.box(f'vent{i}', 0.003, 0.008, 0.16, (-0.451, 1.04 + 0.018 * i, -0.33), M.dark)
        for s in (-1, 1):
            for k in (-1, 1):
                F.cyl(f'foot{s}{k}', 0.012, 0.012, (-0.2 + s * 0.2, 0.956, -0.28 + k * 0.14), M.dark, verts=8)
        # the control column's face: a display, a grid of buttons, a dial and the door release
        F.extrude('micro_face', G.rounded_rect(0.11, 0.26, 0.01, corner=2), 0.006, (-0.39, 1.105, -0.097), M.dark)
        F.box('micro_screen', 0.08, 0.04, 0.004, (-0.39, 1.205, -0.093), M.hob, bevel=0.002, bevel_segments=1)
        F.box('micro_digits', 0.05, 0.016, 0.002, (-0.39, 1.205, -0.0905), M.display)
        for r in range(3):
            for c in range(2):
                F.box(f'button{r}{c}', 0.03, 0.016, 0.004, (-0.405 + 0.03 * c, 1.16 - 0.025 * r, -0.093), M.chrome,
                      bevel=0.0015, bevel_segments=1)
        _knob(F, 'dial', -0.39, 1.055, -0.087, r=0.018, d=0.014)
        F.box('release', 0.07, 0.022, 0.006, (-0.39, 1.005, -0.092), M.chrome, bevel=0.003, bevel_segments=1)
        # the turntable's roller ring under the plate
        G.torus('rollers', 0.08, 0.005, at=F.at(-0.14, 0.976, -0.27), material=M.dark, major_segments=14,
                minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door, hinged on its left edge (seen from the front): a frame round its window, a grip by the controls
    with G.new_parts() as parts:
        F.extrude('micro_door', G.rounded_rect(0.38, 0.28, 0.012, corner=2), 0.02, (-0.14, 1.1, -0.09), M.charcoal,
                  holes=[G.rounded_rect(0.27, 0.19, 0.01, corner=2)], bevel=0.003, bevel_segments=1)
        F.box('micro_window', 0.28, 0.20, 0.006, (-0.14, 1.10, -0.088), M.glass)
        F.sweep('micro_grip', [(-0.305, 1.03, -0.082), (-0.312, 1.04, -0.068), (-0.312, 1.16, -0.068),
                               (-0.305, 1.17, -0.082)], None, M.chrome, profile=G.rounded_rect(0.014, 0.01, 0.004, corner=1),
                up=(1, 0, 0), resolution=2)
    door = G.hinged(G.join(parts, 'micro_door'), F.at(0.05, 1.1, -0.09))
    # the turntable inside, with a bowl of soup on it, a mark so its turn shows
    with G.new_parts() as parts:
        bx, by, bz = F.at(-0.14, 0.972, -0.27)
        G.revolve('turntable', [(0.0, 0.0), (0.125, 0.0), (0.13, 0.008), (0.126, 0.012), (0.11, 0.006), (0.0, 0.006)],
                  at=(bx, by, bz), material=M.plate, segments=20)
        F.box('turntable_mark', 0.03, 0.006, 0.03, (-0.22, 0.98, -0.27), M.dark, bevel=0.002)
        G.revolve('soup_bowl', [(0.0, 0.0), (0.035, 0.0), (0.055, 0.04), (0.05, 0.04), (0.032, 0.008), (0.0, 0.008)],
                  at=F.at(-0.12, 0.978, -0.26), material=M.cup, segments=14)
        F.cyl('soup', 0.048, 0.004, (-0.12, 1.012, -0.26), M.fruit, verts=14)
    turn = G.join(parts, 'turntable', origin=F.at(-0.14, 0.978, -0.27))
    G.settle()
    G.swing(door, 'door_open', -100, seconds=1.0, axis='Z')
    G.spin(turn, 'turntable', turns=1, seconds=2.0)


# --- small appliances: a coffee machine, a breakfast counter and two dishwashers ---------------------

def _counter(F, w=1.0, body=None, top=None):
    """A base unit like the old part's `k_base`: a plinth, a carcass and a worktop to 0.95 with a rounded front edge,
    a shaker drawer over two shaker doors on the front (the front face at depth 0.165), bar pulls."""
    body, top = body or M.cab, top or M.worktop
    F.box('plinth', w - 0.04, 0.10, 0.54, (0, 0.05, -0.19), M.dark, bevel=0.005)
    F.box('carcass', w - 0.004, 0.82, 0.60, (0, 0.51, -0.16), body, bevel=0.006)
    F.box('worktop', w, 0.04, 0.66, (0, 0.93, -0.15), top, bevel=0.01, bevel_segments=2)
    _shaker(F, 'drawer', w - 0.02, 0.14, (0, 0.84, 0.155), body, rail=0.04)
    _pull(F, 'drawer_pull', (-0.09, 0.84), (0.09, 0.84), 0.165, out=0.02, w=0.012, r=None, verts=6)
    for s in (-1, 1):
        _shaker(F, f'door{s}', w / 2 - 0.015, 0.66, (s * w / 4, 0.44, 0.155), body)
        _pull(F, f'door_pull{s}', (s * 0.05, 0.62), (s * 0.05, 0.74), 0.165, out=0.02, w=0.012, r=None, verts=6)


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
    """A counter with an espresso machine on it: a rounded body with a steel cup-warmer and its rail on top, a group
    head with a portafilter, a slotted drip tray, a pressure gauge, a steam wand and a bean hopper; a canister and a
    sugar bowl beside it. The cup slides under the spout and the coffee runs."""
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _counter(F)
        bx, by, bz = F.at(-0.14, 0.95, -0.26)
        G.loft('machine', [G.ring(0.165 * k, 0.155 * k, bz + h, 24, square=5, at=(bx, by)) for h, k in
                           ((0.0, 0.97), (0.012, 1.0), (0.29, 1.0), (0.3, 0.98))], material=M.charcoal)
        F.box('warming', 0.32, 0.012, 0.3, (-0.14, 1.256, -0.26), M.steel, bevel=0.004, bevel_segments=1)
        G.kit.tube(F, 'warming_rail', _fillet([(-0.29, 1.262, -0.14), (-0.29, 1.28, -0.13), (0.01, 1.28, -0.13),
                                               (0.01, 1.262, -0.14)], 0.01, steps=2), 0.004, M.chrome, verts=6, round=1)
        # the group head and the portafilter locked in under it, its handle out to the side
        hx, hy, hz = F.at(-0.14, 1.15, -0.11)
        G.revolve('group', [(0.0, 0.0), (0.03, 0.0), (0.034, 0.012), (0.04, 0.03), (0.042, 0.06), (0.0, 0.06)],
                  at=(hx, hy, hz), material=M.steel, segments=14)
        G.revolve('portafilter', [(0.0, 0.0), (0.03, 0.0), (0.035, 0.022), (0.0, 0.022)], at=F.at(-0.14, 1.128, -0.11),
                  material=M.steel, segments=12)
        F.box('spout', 0.03, 0.012, 0.012, (-0.14, 1.122, -0.11), M.steel, bevel=0.003, bevel_segments=1)
        F.sweep('portafilter_handle', [(-0.10, 1.14, -0.1), (-0.04, 1.145, -0.06), (0.0, 1.15, -0.03)], 0.011, M.dark,
                verts=8, resolution=2, scale=[0.8, 1.0, 1.1])
        # the drip tray with its slotted grille
        F.box('drip', 0.3, 0.02, 0.24, (-0.14, 0.96, -0.11), M.steel, bevel=0.004, bevel_segments=1)
        slots = [[(u - 0.1 + 0.04 * i, v) for u, v in G.rounded_rect(0.02, 0.17, 0.008, corner=1)] for i in range(6)]
        F.extrude('drip_grille', G.rounded_rect(0.28, 0.22, 0.01, corner=1), 0.004, (-0.14, 0.971, -0.11), M.chrome,
                  plane='top', holes=slots)
        # the gauge, two buttons, the steam wand down the side, the hopper on top
        G.revolve('gauge', [(0.0, 0.0), (0.034, 0.0), (0.034, 0.008), (0.029, 0.01), (0.0, 0.006)],
                  at=F.at(-0.03, 1.2, -0.104), material=M.chrome, segments=14, rot=(90, 0, 0))
        F.cyl('gauge_face', 0.028, 0.002, (-0.03, 1.2, -0.096), M.cup, verts=14, rot=(90, 0, 0))
        F.box('gauge_needle', 0.003, 0.02, 0.002, (-0.025, 1.207, -0.094), M.heat, rot=(0, 30, 0))
        for i, x in enumerate((-0.23, -0.19)):
            F.cyl(f'button{i}', 0.011, 0.01, (x, 1.21, -0.101), M.chrome, verts=10, rot=(90, 0, 0))
        G.kit.tube(F, 'steam_wand', [(0.02, 1.17, -0.15), (0.06, 1.165, -0.13), (0.065, 1.08, -0.12), (0.06, 1.02, -0.115)],
                   0.0055, M.chrome, verts=6, round=3)
        hx, hy, hz = F.at(-0.06, 1.262, -0.30)
        G.loft('hopper', [G.ring(0.045 * k, 0.045 * k, hz + h, 12, square=3, at=(hx, hy)) for h, k in
                          ((0.0, 0.7), (0.03, 0.9), (0.11, 1.2), (0.114, 1.12))], material=M.bean)
        G.revolve('hopper_lid', [(0.0, 0.0), (0.056, 0.0), (0.05, 0.016), (0.012, 0.02), (0.012, 0.024), (0.0, 0.024)],
                  at=(hx, hy, hz + 0.114), material=M.charcoal, segments=12)
        # a coffee canister and a sugar bowl on the worktop
        _canister(F, 'canister', 0.25, 0.95, -0.34, 0.06, 0.16, M.bean, M.steel)
        _canister(F, 'sugar', 0.34, 0.95, -0.2, 0.045, 0.07, M.cup, M.cup)
    G.join(still, 'body', origin=(0, 0, 0))
    # the cup, on the drip tray's front edge, sliding back under the spout for `brew`
    with G.new_parts() as parts:
        _cup(F, 'cup', -0.14, 0.965, 0.0)
    cup = G.join(parts, 'cup', origin=F.at(-0.14, 0.965, 0.0))
    stream = _stream(F, 'stream', -0.14, 1.122, -0.11, 0.99, M.coffee, r=0.008)
    G.settle()
    G.slide(cup, 'brew', (0, 0.11, 0), seconds=0.6)
    G.grow(stream, 'brew', seconds=0.4)


def _cup(F, name, x, y, z):
    """An espresso cup on a saucer on game height `y`, its handle out to the side."""
    bx, by, bz = F.at(x, y, z)
    G.revolve(f'{name}_saucer', [(0.0, 0.0), (0.03, 0.0), (0.05, 0.008), (0.047, 0.01), (0.0, 0.004)],
              at=(bx, by, bz), material=M.cup, segments=14)
    G.revolve(name, [(0.0, 0.0), (0.022, 0.0), (0.033, 0.02), (0.035, 0.05), (0.031, 0.05), (0.029, 0.03), (0.0, 0.03)],
              at=(bx, by, bz + 0.006), material=M.cup, segments=14)
    F.cyl(f'{name}_crema', 0.03, 0.002, (x, y + 0.048, z), M.coffee, verts=12)
    F.sweep(f'{name}_handle', [(x + 0.032, y + 0.045, z), (x + 0.05, y + 0.042, z), (x + 0.05, y + 0.024, z),
                               (x + 0.034, y + 0.02, z)], 0.004, M.cup, verts=5, resolution=2)


def _canister(F, name, x, y, z, r, h, material, lid):
    """A round canister with a lid on game height `y`."""
    bx, by, bz = F.at(x, y, z)
    G.revolve(name, [(0.0, 0.0), (r * 0.96, 0.0), (r, 0.006), (r, h * 0.86), (0.0, h * 0.86)], at=(bx, by, bz),
              material=material, segments=14)
    G.revolve(f'{name}_lid', [(0.0, 0.0), (r * 1.03, 0.0), (r * 1.03, h * 0.1), (r * 0.6, h * 0.14), (r * 0.25, h * 0.14),
                              (r * 0.25, h * 0.16), (0.0, h * 0.16)], at=(bx, by, bz + h * 0.84), material=lid, segments=14)


# --- breakfast counter: 1x1, a counter with a toaster and a blender on the worktop ---------------------
def breakfastCounter():
    """A counter with a rounded two-slot toaster (its browning dial and crumb tray), a blender (a buttoned base and a
    jug with a handle) and a fruit bowl: oranges, an apple and a banana. The toast goes down and pops up; the
    blender's lid lifts while the smoothie swirls."""
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _counter(F)
        # the toaster: a rounded body, two slots in its top, a dial and a crumb tray
        F.soft('toaster', 0.28, 0.18, 0.16, (0.30, 1.04, -0.30), M.toaster, round=0.045, round_segments=3)
        for s in (-1, 1):
            F.box(f'slot{s}', 0.12, 0.012, 0.032, (0.30, 1.126, -0.30 + s * 0.035), M.dark, bevel=0.005,
                  bevel_segments=1)
        _knob(F, 'dial', 0.3, 1.0, -0.215, r=0.014, d=0.012, material=M.chrome)
        F.box('crumb_tray', 0.18, 0.012, 0.01, (0.30, 0.962, -0.218), M.chrome, bevel=0.003, bevel_segments=1)
        F.box('lever_slot', 0.006, 0.1, 0.012, (0.441, 1.06, -0.30), M.dark)
        # the blender: a tapered base with buttons, the jug with its handle
        bx, by, bz = F.at(0.0, 0.95, -0.32)
        G.loft('blender', [G.ring(0.08 * k, 0.08 * k, bz + h, 16, square=4, at=(bx, by)) for h, k in
                           ((0.0, 1.0), (0.09, 0.92), (0.12, 0.8))], material=M.charcoal)
        for i, x in enumerate((-0.03, 0.0, 0.03)):
            F.box(f'button{i}', 0.018, 0.012, 0.006, (x, 1.0, -0.247), M.chrome if i else M.display, bevel=0.002,
                  bevel_segments=1)
        G.loft('jug', [G.ring(0.06 * k, 0.06 * k, bz + h, 16, square=3, at=(bx, by)) for h, k in
                       ((0.12, 0.95), (0.13, 1.0), (0.38, 1.25), (0.38, 1.18), (0.135, 0.92))], material=M.jug)
        F.sweep('jug_handle', [(-0.07, 1.30, -0.32), (-0.115, 1.29, -0.32), (-0.118, 1.14, -0.32), (-0.066, 1.11, -0.32)],
                None, M.jug, profile=G.rounded_rect(0.022, 0.014, 0.006, corner=1), up=(0, 0, 1), resolution=2)
        # the fruit bowl: two oranges, an apple and a banana
        bx, by, bz = F.at(-0.28, 0.95, 0.05)
        G.revolve('bowl', [(0.0, 0.0), (0.06, 0.0), (0.12, 0.045), (0.13, 0.06), (0.122, 0.06), (0.11, 0.048),
                           (0.055, 0.008), (0.0, 0.008)], at=(bx, by, bz), material=M.jug, segments=18)
        for i, (x, z) in enumerate(((-0.31, 0.02), (-0.24, 0.09))):
            F.sphere(f'orange{i}', 0.04, (x, 1.0, z), M.fruit, segments=10, rings=7)
        F.sphere('apple', 0.038, (-0.22, 1.0, 0.0), M.toaster, segments=10, rings=7, scale=(1, 1, 0.92))
        F.cyl('apple_stalk', 0.003, 0.016, (-0.22, 1.042, 0.0), M.bean, verts=5, smooth=False)
        F.sweep('banana', [(-0.38, 1.02, 0.07), (-0.33, 1.0, 0.12), (-0.26, 1.005, 0.15), (-0.2, 1.03, 0.15)], 0.017,
                M.banana, verts=6, resolution=3, scale=[0.35, 1.0, 1.0, 0.45])
    G.join(still, 'body', origin=(0, 0, 0))
    # the toaster's carriage, its lever and its toast standing up at rest: it goes down for toast, and
    # coming back when the use ends pops the toast up (one part, so the smoothie's swirl has a draw call)
    with G.new_parts() as parts:
        F.box('lever', 0.024, 0.024, 0.036, (0.456, 1.08, -0.30), M.dark, bevel=0.008, bevel_segments=2)
        for s in (-1, 1):
            F.soft(f'toast{s}', 0.10, 0.12, 0.022, (0.30, 1.11, -0.30 + s * 0.035), M.bread, round=0.01,
                   round_segments=2, puff=0.004, flat=1)
            F.box(f'toast_face{s}', 0.086, 0.044, 0.023, (0.30, 1.142, -0.30 + s * 0.035), M.crumb)
    toast = G.join(parts, 'toast', origin=F.at(0.30, 1.05, -0.30))
    # the blender's lid, lifting off for `blend`
    with G.new_parts() as parts:
        G.revolve('lid', [(0.0, 0.0), (0.077, 0.0), (0.078, 0.022), (0.07, 0.03), (0.0, 0.03)],
                  at=F.at(0.0, 1.33, -0.32), material=M.charcoal, segments=16)
        F.cyl('lid_knob', 0.02, 0.02, (0.0, 1.37, -0.32), M.charcoal, verts=10, bevel=0.004, bevel_segments=1)
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


# --- dishwashers: a door dropping open over a sliding lower rack, a cabinet beside it under the worktop --
def _basket(F, name, x, y, z, w, d, h, bars=4):
    """A dishwasher's wire basket at height `y` (its floor): a floor frame with bars across, a top rail, four posts."""
    for k, yy in enumerate((y, y + h)):
        F.sweep(f'{name}_rail{k}', [(x + u, yy, z + v) for u, v in G.rounded_rect(w, d, 0.02, corner=2)], 0.004,
                M.tray, verts=4, closed=True)
    for i in range(bars):
        u = -w / 2 + w * (i + 1) / (bars + 1)
        F.sweep(f'{name}_bar{i}', [(x + u, y, z - d / 2), (x + u, y, z + d / 2)], 0.003, M.tray, verts=4, caps=False)
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.sweep(f'{name}_post{sx}{sz}', [(x + sx * (w / 2 - 0.006), y, z + sz * (d / 2 - 0.006)),
                                             (x + sx * (w / 2 - 0.006), y + h, z + sz * (d / 2 - 0.006))], 0.003,
                    M.tray, verts=4, caps=False)


def _dishwasher(F, w, h, top_d, top_z, front, door_h, body, panel, top, dw, drawers):
    """A dishwasher `dw` wide at the left of a `w`-wide unit under one worktop: a steel tub (an upper basket of cups and
    a spray arm in it) whose door drops on its bottom hinge and whose lower basket of plates slides out, a control
    fascia over the door. Beside it, the rest of the unit: a drawer over a door, or (`drawers`) a stack of three
    drawers."""
    mx = -w / 2 + dw / 2                                               # the machine's middle
    cw = w - dw
    cx = w / 2 - cw / 2                                                # the cabinet's
    # the carcass runs from the back up to the fronts' backs (the compact one stood 2 cm proud of its door and drawers,
    # hiding them), the tub cut out of it behind the door
    cb, cf = top_z - (top_d - 0.06) / 2 - 0.01, front - 0.01
    face = cf
    tub_d = top_d - 0.12
    tz = face - tub_d / 2
    with G.new_parts() as still:
        F.box('plinth', w - 0.04, 0.10, top_d - 0.12, (0, 0.05, top_z - 0.04), M.dark, bevel=0.005)
        carcass = F.box('carcass', w - 0.02, h - 0.14, cf - cb, (0, h / 2 - 0.02, (cf + cb) / 2), body, bevel=0.006)
        _hollow(carcass, F, 'tub', dw - 0.08, door_h - 0.04, tub_d, (mx, 0.12 + door_h / 2, tz + 0.003), M.tray)
        F.box('worktop', w, 0.04, top_d, (0, h - 0.02, top_z), top, bevel=0.01, bevel_segments=2)
        # the fascia over the door: a display and a button
        F.extrude('strip', G.rounded_rect(dw - 0.03, 0.065, 0.008, corner=2), 0.012, (mx, h - 0.075, front - 0.002),
                  M.dark, bevel=0.003, bevel_segments=1)
        F.box('display', 0.08, 0.022, 0.004, (mx - dw / 2 + 0.12, h - 0.075, front + 0.005), M.display)
        F.sphere('button', 0.011, (mx + dw / 2 - 0.08, h - 0.075, front + 0.004), M.chrome, segments=8, rings=5)
        # inside the tub: the spray arm and the upper basket with cups upside down in it
        F.box('spray_arm', dw - 0.2, 0.01, 0.04, (mx, 0.18, tz), M.tray, bevel=0.004, bevel_segments=1)
        F.cyl('spray_hub', 0.02, 0.02, (mx, 0.175, tz), M.tray, verts=10)
        _basket(F, 'upper', mx, 0.56, tz, dw - 0.14, tub_d - 0.06, 0.1, bars=3)
        for i in range(3):
            G.revolve(f'cup{i}', [(0.0, 0.0), (0.03, 0.0), (0.034, 0.01), (0.028, 0.06), (0.0, 0.06)],
                      at=F.at(mx - 0.12 + 0.12 * i, 0.625, tz), material=M.cup, segments=10, rot=(180, 0, 0))
        # the cabinet beside it, its fronts flush with the door
        if drawers:
            for k, (y0, y1) in enumerate(((0.12, 0.41), (0.42, 0.71), (0.72, h - 0.07))):
                _shaker(F, f'drawer{k}', cw - 0.03, y1 - y0, (cx, (y0 + y1) / 2, front), body, rail=0.045)
                _pull(F, f'drawer_pull{k}', (cx - (cw - 0.25) / 2, y1 - 0.045), (cx + (cw - 0.25) / 2, y1 - 0.045),
                      front + 0.01, out=0.015, w=0.012, r=None, verts=6)
        else:
            _shaker(F, 'drawer', cw - 0.03, 0.12, (cx, h - 0.13, front), body, rail=0.035)
            _pull(F, 'drawer_pull', (cx - (cw - 0.22) / 2, h - 0.13), (cx + (cw - 0.22) / 2, h - 0.13), front + 0.01,
                  out=0.015, w=0.012, r=None, verts=6)
            _shaker(F, 'cab_door', cw - 0.03, h - 0.33, (cx, 0.12 + (h - 0.33) / 2, front), body)
            _pull(F, 'cab_door_pull', (cx - cw / 2 + 0.06, h - 0.37), (cx - cw / 2 + 0.06, h - 0.25), front + 0.01,
                  out=0.015, w=0.012, r=None, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door, hinged on its bottom edge (its bottom at y 0.12): a steel front with a bar handle, its inner panel
    # and the detergent flap on the back
    with G.new_parts() as parts:
        F.soft('door', dw - 0.02, door_h, 0.02, (mx, 0.12 + door_h / 2, front), panel, round=0.006, round_segments=1)
        _pull(F, 'handle', (mx - (dw - 0.2) / 2, 0.12 + door_h - 0.05), (mx + (dw - 0.2) / 2, 0.12 + door_h - 0.05),
              front + 0.01, out=0.02, w=0.016, r=None, verts=8)
        F.box('door_inner', dw - 0.1, door_h - 0.08, 0.006, (mx, 0.12 + door_h / 2, front - 0.012), M.tray)
        F.box('detergent', 0.12, 0.07, 0.008, (mx + 0.05, 0.12 + door_h * 0.6, front - 0.016), M.plate,
              bevel=0.003, bevel_segments=1)
    door = G.hinged(G.join(parts, 'door'), F.at(mx, 0.12, front))
    # the lower basket of plates standing in a row and a bowl, sliding out to the front
    with G.new_parts() as parts:
        _basket(F, 'rack', mx, 0.27, top_z - 0.01, dw - 0.14, top_d - 0.26, 0.12)
        for i in range(6):
            F.cyl(f'plate{i}', 0.1, 0.008, (mx - 0.1 + 0.035 * i, 0.375, top_z - 0.06), M.plate, verts=16,
                  rot=(0, 90, 0))
        G.revolve('bowl', [(0.0, 0.0), (0.04, 0.0), (0.07, 0.05), (0.064, 0.05), (0.036, 0.006), (0.0, 0.006)],
                  at=F.at(mx + 0.15, 0.275, top_z + 0.06), material=M.plate, segments=12)
    rack = G.join(parts, 'rack', origin=F.at(mx, 0.30, top_z - 0.01))
    G.settle()
    G.swing(door, 'door_open', 85, seconds=SECONDS, axis='X')
    G.slide(rack, 'door_open', (0, -0.30, 0), seconds=SECONDS)


def dishwasher():
    """A 60 cm dishwasher beside a 40 cm cupboard (a drawer over a door)."""
    F = G.GameFrame(shift=0.1375)
    _dishwasher(F, w=1.0, h=0.95, top_d=0.66, top_z=-0.15, front=0.175, door_h=0.72,
                body=M.cab, panel=M.steel, top=M.cab, dw=0.6, drawers=False)


def dishwasherCompact():
    """A slimline 45 cm dishwasher beside a 55 cm stack of three drawers."""
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    _dishwasher(F, w=1.0, h=0.95, top_d=0.79, top_z=-0.085, front=0.25, door_h=0.66,
                body=M.cab, panel=M.steel, top=M.cab, dw=0.45, drawers=True)


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
             # espresso slides a cup under the spout and grows a stream of coffee; pour too, a second in (at a café
             # as the barista turns to the machine: the game's `sell` clip)
             motions={'uses': {'espresso': 'brew', 'pour': {'clip': 'brew', 'delay': 1.0}}, 'hold': 2.0},
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
