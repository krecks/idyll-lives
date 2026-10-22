# The farm (docs/objects/community.md CM5): the red barn, the grain silo, the little tractor, the round hay bale,
# the stack of hay bales, the scarecrow, the water trough and the chicken coop (`model.farm.<id>`). They stand on
# the farm lots of farm worlds (web/src/game/town.ts `farmstead`) and are seen from afar in the town
# overview, so each has a far version (E7), built by hand (the model's plain shapes) but the scarecrow's, decimated.
# Notes: farm.md.
#
# Built in the game's axes (`F`: x, height, depth; front +depth), then stood on the floor and centred on the
# footprint (`G.settle`), the sizes the old parts'.
#
# Run:  cargo art farm [--only barn] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import shapes  # noqa: E402

F = G.GameFrame()

M = G.Palette(
    # the barn
    barn=('#A8322D', 'matte'),          # walls and door leaves (Paint)
    barn_roof=('#4B4F55', 'satin'),     # (Roof)
    barn_trim=('#F1EEE6', 'satin'),     # corner boards, door frames and braces, window frames (Trim)
    stone=('#8E8C88', 'matte'),         # the plinth
    shadow=('#2A2422', 'matte'),        # behind the doors, window panes, the cupola's louvres
    iron=('#2B302E', 'metal'),          # door rail, weathervane, hooks
    # the silo
    silo=('#C3C8CB', 'metal'),          # the corrugated drum (Paint)
    silo_cap=('#A8322D', 'satin'),      # the domed cap (Roof)
    concrete=('#9A958C', 'matte'),
    steel=('#5C6168', 'metal'),         # ladder, cage, chute
    # the tractor
    tractor=('#3E7D3A', 'gloss'),       # hood, fenders, canopy (Paint)
    rims=('#F2C14E', 'gloss'),          # (Wheels)
    tyre=('#26282B', 'matte'),
    seat=('#2B2C2E', 'satin'),
    lamp=('#F4F1E0', 'gloss'),
    tail_lamp=('#C8322A', 'gloss'),
    # hay
    hay=('#D9B95E', 'fabric'),
    hay_dark=('#B8964A', 'fabric'),     # the rolled spiral on a round bale's ends
    straw=('#D9B95E', 'matte'),         # loose straw, the scarecrow's stuffing
    twine=('#8C6A3A', 'matte'),         # (Twine)
    # the scarecrow
    post=('#7A5A3A', 'wood'),
    shirt=('#B8322B', 'fabric'),        # (Shirt)
    shirt_check=('#7E1F1B', 'fabric'),  # the plaid's dark lines
    trousers=('#2E4E7A', 'fabric'),     # (Trousers)
    patch=('#D9A64A', 'fabric'),
    hat=('#6E5A3A', 'fabric'),          # (Hat)
    sack=('#C9A86A', 'matte'),
    stitch=('#2A2624', 'matte'),
    # the trough
    trough=('#A9AFB3', 'metal'),        # (Paint)
    water=('#5B93A8', 'gloss'),
    # the coop
    coop=('#C8A46E', 'wood'),           # walls, ramp, legs (Paint)
    coop_roof=('#A8322D', 'satin'),     # (Roof)
    coop_trim=('#F1EEE6', 'satin'),     # corner boards, runners, the pop-hole hatch (Trim)
    coop_dark=('#3A2A1E', 'matte'),     # the pop hole behind the hatch, the window's wire
)


# --- helpers (game coordinates) ------------------------------------------------------------------------------
def _lift(axis, u, v, w):
    """A game point from a 2D point (u, v) on a plane and `w` across it: 'z' (x, y) at depth w, 'x' (depth,
    y) at x w, 'y' (x, depth) at height w."""
    return {'z': (u, v, w), 'x': (w, v, u), 'y': (u, w, v)}[axis]


def _mesh(name, verts, faces, material):
    """A flat-shaded mesh from game points, its normals made to face out."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([F.at(*v) for v in verts], [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    shapes._recalc_normals(obj)
    return obj


def _prism(name, pts, w0, w1, material, axis='z'):
    """A convex outline `pts` on a plane (see `_lift`), extruded from `w0` to `w1` across it."""
    n = len(pts)
    verts = [_lift(axis, u, v, w0) for u, v in pts] + [_lift(axis, u, v, w1) for u, v in pts]
    faces = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return _mesh(name, verts, faces, material)


def _offset(line, t):
    """`line` (a polyline on a plane, u rising left to right) moved `t` to its left (up, for a roof), mitred."""
    def normal(a, b):
        dx, dy = b[0] - a[0], b[1] - a[1]
        k = math.hypot(dx, dy)
        return -dy / k, dx / k
    out = []
    for i, p in enumerate(line):
        ns = ([normal(line[i - 1], p)] if i else []) + ([normal(p, line[i + 1])] if i < len(line) - 1 else [])
        nx, ny = sum(q[0] for q in ns), sum(q[1] for q in ns)
        k = math.hypot(nx, ny)
        nx, ny = nx / k, ny / k
        s = t / (nx * ns[0][0] + ny * ns[0][1])
        out.append((p[0] + nx * s, p[1] + ny * s))
    return out


def _shell(name, line, t, w0, w1, material, axis='z'):
    """A roof or fender: the polyline `line` thickened `t` to its left, from `w0` to `w1` across."""
    inner, outer = line, _offset(line, t)
    n = len(line)
    verts = []
    for w in (w0, w1):
        verts += [_lift(axis, u, v, w) for u, v in inner] + [_lift(axis, u, v, w) for u, v in outer]
    I0, O0, I1, O1 = 0, n, 2 * n, 3 * n
    faces = []
    for i in range(n - 1):
        faces += [(O0 + i, O0 + i + 1, O1 + i + 1, O1 + i), (I0 + i, I1 + i, I1 + i + 1, I0 + i + 1),
                  (I0 + i, I0 + i + 1, O0 + i + 1, O0 + i), (I1 + i, O1 + i, O1 + i + 1, I1 + i + 1)]
    faces += [(I0, O0, O1, I1), (I0 + n - 1, I1 + n - 1, O1 + n - 1, O0 + n - 1)]
    return _mesh(name, verts, faces, material)


def _strut(name, a, b, width, w0, w1, material, axis='z'):
    """A flat bar from `a` to `b` on a plane (a door's brace), `width` wide, from `w0` to `w1` across."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    k = math.hypot(dx, dy)
    nx, ny = -dy / k * width / 2, dx / k * width / 2
    pts = [(a[0] - nx, a[1] - ny), (b[0] - nx, b[1] - ny), (b[0] + nx, b[1] + ny), (a[0] + nx, a[1] + ny)]
    return _prism(name, pts, w0, w1, material, axis)


def _stadium(half, n=8):
    """(cx, angle) samples round a stadium (a straight middle `2 * half` long along x, round ends), `n` + 1
    per end."""
    return [(half, -90 + 180 * i / n) for i in range(n + 1)] + [(-half, 90 + 180 * i / n) for i in range(n + 1)]


def _stadium_sweep(name, profile, half, material, n=8):
    """A closed profile [(r, height)] swept round a stadium (a stock tank's wall): x along, depth across."""
    ring = _stadium(half, n)
    m = len(profile)
    verts = [(cx + r * G.cosd(a), y, r * G.sind(a)) for cx, a in ring for r, y in profile]
    faces = []
    for s in range(len(ring)):
        s2 = (s + 1) % len(ring)
        for i in range(m):
            i2 = (i + 1) % m
            faces.append((s * m + i, s * m + i2, s2 * m + i2, s2 * m + i))
    return _mesh(name, verts, faces, material)


def _stadium_slab(name, r, half, y0, y1, material, n=8):
    pts = [(cx + r * G.cosd(a), r * G.sind(a)) for cx, a in _stadium(half, n)]
    return _prism(name, pts, y0, y1, material, axis='y')


def _frame(name, x0, y0, x1, y1, z, bar, depth, material, axis='z'):
    """A rectangular frame of bars `bar` wide on a face at `z` (see `_lift`), standing `depth` proud."""
    h = bar / 2
    bars = {'b': (x0 - h, y0 - h, x1 + h, y0 + h), 't': (x0 - h, y1 - h, x1 + h, y1 + h),
            'l': (x0 - h, y0 + h, x0 + h, y1 - h), 'r': (x1 - h, y0 + h, x1 + h, y1 - h)}
    for side, (u0, v0, u1, v1) in bars.items():
        _prism(f'{name}_{side}', [(u0, v0), (u1, v0), (u1, v1), (u0, v1)], z, z + depth, material, axis)


# --- the barn --------------------------------------------------------------------------------------------
BW, BD = 2.8, 2.75          # half the walls' width and depth
EAVE, KNEE, KNEE_X, RIDGE = 3.0, 4.3, 1.75, 4.95
ROOF_T, ROOF_D = 0.12, 2.92  # roof thickness, half its depth (an overhang at the gables)
BARN_WALL = [(-BW, 0), (BW, 0), (BW, EAVE), (KNEE_X, KNEE), (0, RIDGE), (-KNEE_X, KNEE), (-BW, EAVE)]


def _barn_roof_line():
    """The roof's underside: the walls' gambrel, carried 0.2 m past the eaves."""
    dx, dy = KNEE_X - BW, KNEE - EAVE
    k = math.hypot(dx, dy)
    ex, ey = BW - 0.2 * dx / k, EAVE - 0.2 * dy / k
    return [(-ex, ey), (-KNEE_X, KNEE), (0, RIDGE), (KNEE_X, KNEE), (ex, ey)]


def _barn_height(x):
    """The walls' top at `x`."""
    x = abs(x)
    if x > KNEE_X:
        return EAVE + (KNEE - EAVE) * (BW - x) / (BW - KNEE_X)
    return KNEE + (RIDGE - KNEE) * (KNEE_X - x) / KNEE_X


def _barn_shell(detail=True):
    """Walls, plinth, roof, ridge cap: what the far version keeps too."""
    _prism('walls', BARN_WALL, -BD, BD, M.barn)
    G.cube('plinth', (2 * BW + 0.08, 2 * BD + 0.08, 0.3), at=(0, 0, 0.15), material=M.stone, bevel=0.02 if detail else 0)
    _shell('roof', _barn_roof_line(), ROOF_T, -ROOF_D, ROOF_D, M.barn_roof)
    if not detail:
        G.cube('ridge', (0.2, 2 * ROOF_D + 0.04, 0.08), at=(0, 0, RIDGE + ROOF_T + 0.02), material=M.barn_roof)
        return
    # a rolled ridge cap, the metal roof's standing seams from eave to ridge, and the plinth's dressed stones
    F.sweep('ridge', [(0, RIDGE + ROOF_T + 0.01, -ROOF_D - 0.02), (0, RIDGE + ROOF_T + 0.01, ROOF_D + 0.02)], None, M.barn_roof,
            profile=[(0.035, 0.0), (0.0, 0.13), (-0.03, 0.1), (-0.03, -0.1), (0.0, -0.13)], up=(0, 0, 1), smooth=False)
    seam = _offset(_barn_roof_line(), ROOF_T - 0.005)
    for end, nxt in ((0, 1), (-1, -2)):                                   # stopped short of the eaves' edge
        (u0, v0), (u1, v1) = seam[end], seam[nxt]
        k = 0.1 / math.hypot(u1 - u0, v1 - v0)
        seam[end] = (u0 + (u1 - u0) * k, v0 + (v1 - v0) * k)
    n = 17
    for i in range(n):
        w = -ROOF_D + 0.18 + (2 * ROOF_D - 0.36) * i / (n - 1)
        F.sweep(f'seam{i}', [(u, v, w) for u, v in seam], None, M.barn_roof,
                profile=[(0.04, -0.01), (0.04, 0.01), (0.0, 0.018), (0.0, -0.018)], up=(1, 0, 0), smooth=False)
    _barn_stones()


def _barn_stones():
    """Two courses of dressed stones proud of the plinth all round, their joints staggered."""
    hx, hz = BW + 0.04, BD + 0.04
    for c, (y, h) in enumerate(((0.075, 0.13), (0.215, 0.13))):
        for face, (length, run) in enumerate(((2 * hx, 'x'), (2 * hz, 'z'))):
            k = max(1, round(length / 0.8))
            step = length / (k + 0.5 * (c % 2))
            starts = [-length / 2 + (step / 2 if c % 2 else 0) + i * step for i in range(k + (c % 2))]
            for i, a in enumerate(starts):
                b = min(length / 2, a + step)
                a = max(-length / 2, a)
                m, L = (a + b) / 2, b - a - 0.02
                for s in (-1, 1):
                    if run == 'x':
                        F.box(f'stone{c}{face}{i}{s}', L, h, 0.03, (m, y, s * (hz + 0.006)), M.stone, bevel=0.01, bevel_segments=1)
                    else:
                        F.box(f'stone{c}{face}{i}{s}', 0.03, h, L, (s * (hx + 0.006), y, m), M.stone, bevel=0.01, bevel_segments=1)


def _barn_doors(z, s, detail=True):
    """The big doors (front, s=1) or the small one (back, s=-1) on the gable face at depth `z`."""
    if s > 0:
        x0, x1, top = -1.3, 1.3, 2.6
        _prism('door_dark', [(x0, 0.3), (x1, 0.3), (x1, top), (x0, top)], z, z + 0.02, M.shadow)
        gap = 0.02
        for side, (a, b) in (('l', (x0, -gap)), ('r', (gap, x1))):
            _prism(f'leaf_{side}', [(a, 0.3), (b, 0.3), (b, top), (a, top)], z, z + 0.06, M.barn)
            _frame(f'leaf_{side}_frame', a + 0.06, 0.36, b - 0.06, top - 0.06, z + 0.06, 0.12, 0.04, M.barn_trim)
            _strut(f'leaf_{side}_x1', (a + 0.1, 0.4), (b - 0.1, top - 0.1), 0.1, z + 0.06, z + 0.09, M.barn_trim)
            _strut(f'leaf_{side}_x2', (b - 0.1, 0.4), (a + 0.1, top - 0.1), 0.1, z + 0.06, z + 0.09, M.barn_trim)
            if detail:
                hx = -0.16 if side == 'l' else 0.16
                G.kit.pull(F, f'handle_{side}', hx, 1.35, z + 0.09, 0.34, M.iron, r=0.014, out=0.06, upright=True, verts=6)
                F.box(f'guide_{side}', 0.12, 0.06, 0.05, ((a + b) / 2, 0.33, z + 0.085), M.iron, bevel=0.008, bevel_segments=1)
        # the rail the leaves hang from, and its hangers
        G.cube('rail', (2 * x1 + 1.6, 0.08, 0.1), at=F.at(0, top + 0.12, z + 0.06), material=M.iron)
        if detail:
            for x in (-1.0, -0.3, 0.3, 1.0):
                G.cylinder(f'hanger{x}', 0.05, 0.05, at=F.at(x, top + 0.12, z + 0.13), material=M.iron, rot=(90, 0, 0), verts=10)
        # the hayloft door and the hay beam above it
        hx, hy0, hy1 = 0.5, 3.2, 4.1
        _prism('loft', [(-hx, hy0), (hx, hy0), (hx, hy1), (-hx, hy1)], z, z + 0.05, M.barn)
        _frame('loft_frame', -hx, hy0, hx, hy1, z + 0.05, 0.12, 0.04, M.barn_trim)
        _strut('loft_x1', (-hx + 0.06, hy0 + 0.06), (hx - 0.06, hy1 - 0.06), 0.09, z + 0.05, z + 0.08, M.barn_trim)
        _strut('loft_x2', (hx - 0.06, hy0 + 0.06), (-hx + 0.06, hy1 - 0.06), 0.09, z + 0.05, z + 0.08, M.barn_trim)
        if detail:
            for y in (hy0 + 0.2, hy1 - 0.2):                                # its strap hinges, on the left
                F.box(f'loft_hinge{y}', 0.36, 0.045, 0.015, (-hx + 0.2, y, z + 0.1), M.iron, bevel=0.004, bevel_segments=1)
                G.cylinder(f'loft_pin{y}', 0.018, 0.07, at=F.at(-hx - 0.02, y, z + 0.1), material=M.iron, verts=6)
            G.kit.knob(F, 'loft_knob', hx - 0.14, (hy0 + hy1) / 2, z + 0.09, 0.025, M.iron, segments=8)
            G.cube('hay_beam', (0.14, 0.24, 0.14), at=F.at(0, 4.4, z + 0.11), material=M.barn_trim)
            G.torus('hay_pulley', 0.06, 0.016, at=F.at(0, 4.27, z + 0.17), material=M.iron, rot=(0, 90, 0), major_segments=12, minor_segments=6)
    else:
        z0 = z - 0.06
        _prism('back_door', [(0.6, 0.3), (1.6, 0.3), (1.6, 2.2), (0.6, 2.2)], z0, z, M.barn)
        _frame('back_door_frame', 0.6, 0.3, 1.6, 2.2, z0 - 0.04, 0.1, 0.04, M.barn_trim)
        _strut('back_door_x', (0.66, 0.36), (1.54, 2.14), 0.09, z0 - 0.04, z0, M.barn_trim)
        _prism('back_loft', [(-0.4, 3.3), (0.4, 3.3), (0.4, 4.0), (-0.4, 4.0)], z0, z, M.barn)
        _frame('back_loft_frame', -0.4, 3.3, 0.4, 4.0, z0 - 0.04, 0.1, 0.04, M.barn_trim)


def _barn_window(name, at, axis, s):
    """A four-pane window on a side wall (axis 'x': at=(depth, height), the wall at x s * BW)."""
    u, v = at
    w, h = 0.38, 0.38
    x = s * BW
    _prism(f'{name}_pane', [(u - w, v - h), (u + w, v - h), (u + w, v + h), (u - w, v + h)], x, x + s * 0.02, M.shadow, axis)
    _frame(f'{name}_frame', u - w, v - h, u + w, v + h, x, 0.1, s * 0.05, M.barn_trim, axis)
    _prism(f'{name}_mv', [(u - 0.025, v - h), (u + 0.025, v - h), (u + 0.025, v + h), (u - 0.025, v + h)], x, x + s * 0.04, M.barn_trim, axis)
    _prism(f'{name}_mh', [(u - w, v - 0.025), (u + w, v - 0.025), (u + w, v + 0.025), (u - w, v + 0.025)], x, x + s * 0.04, M.barn_trim, axis)
    # a sloped sill under it and a drip cap over it
    F.box(f'{name}_sill', 0.11, 0.045, 2 * w + 0.22, (x + s * 0.05, v - h - 0.075, u), M.barn_trim, bevel=0.012, bevel_segments=1)
    F.box(f'{name}_cap', 0.08, 0.07, 2 * w + 0.26, (x + s * 0.04, v + h + 0.085, u), M.barn_trim, bevel=0.012, bevel_segments=1)


# Where the battens stop: (u0, u1, y0, y1) round each door and window, per face.
BARN_OPENINGS = {
    'front': [(-1.38, 1.38, 0.0, 2.8), (-0.58, 0.58, 3.12, 4.18)],
    'back': [(0.53, 1.67, 0.0, 2.27), (-0.47, 0.47, 3.23, 4.07)],
    'side': [(c - 0.5, c + 0.5, 1.22, 2.25) for c in (-1.6, 0, 1.6)],
}


def _batten_spans(u, top, openings):
    """The heights a batten at `u` runs over, from the plinth to `top`, broken round `openings`."""
    spans = [(0.3, top)]
    for u0, u1, y0, y1 in openings:
        if u0 - 0.03 < u < u1 + 0.03:
            spans = [piece for a, b in spans for piece in ((a, min(b, y0)), (max(a, y1), b)) if piece[1] - piece[0] > 0.08]
    return spans


def _barn_battens():
    """Board-and-batten siding: raised battens in the walls' colour, round the doors and windows."""
    for s, face in ((1, 'front'), (-1, 'back')):
        for i in range(15):
            u = -2.45 + 0.35 * i
            for k, (a, b) in enumerate(_batten_spans(u, _barn_height(u) - 0.05, BARN_OPENINGS[face])):
                _prism(f'batten_{face}{i}_{k}', [(u - 0.03, a), (u + 0.03, a), (u + 0.03, b), (u - 0.03, b)],
                       s * BD, s * (BD + 0.03), M.barn)
    for s in (-1, 1):
        for i in range(15):
            u = -2.45 + 0.35 * i
            for k, (a, b) in enumerate(_batten_spans(u, EAVE - 0.05, BARN_OPENINGS['side'])):
                _prism(f'batten_side{s}{i}_{k}', [(u - 0.03, a), (u + 0.03, a), (u + 0.03, b), (u - 0.03, b)],
                       s * BW, s * (BW + 0.03), M.barn, 'x')


def _barn_corners():
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'corner{sx}{sz}', 0.16, EAVE - 0.3, 0.16, (sx * (BW - 0.06), 0.3 + (EAVE - 0.3) / 2, sz * (BD - 0.06)), M.barn_trim)
    # fascia boards along the gable ends' roof edge
    for sz in (-1, 1):
        line = _barn_roof_line()
        for i in range(len(line) - 1):
            _strut(f'fascia{sz}{i}', line[i], line[i + 1], 0.14, sz * (ROOF_D - 0.02), sz * (ROOF_D + 0.02), M.barn_trim)


def _cupola(detail=True):
    """A louvred cupola on the ridge with a weathervane on top."""
    y0 = RIDGE + 0.02
    F.box('cupola', 0.7, 0.5, 0.7, (0, y0 + 0.25, 0), M.barn_trim)
    if detail:
        # its base saddled down onto the roof, a band under its roof, boards at its corners and a ball under the vane
        F.box('cupola_base', 0.8, 0.3, 0.8, (0, y0 - 0.1, 0), M.barn_roof, bevel=0.015, bevel_segments=1)
        F.box('cupola_band', 0.8, 0.06, 0.8, (0, y0 + 0.5, 0), M.barn_trim, bevel=0.012, bevel_segments=1)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(f'cupola_corner{sx}{sz}', 0.06, 0.46, 0.06, (sx * 0.345, y0 + 0.24, sz * 0.345), M.barn_trim)
        F.sphere('vane_ball', 0.05, (0, y0 + 0.84, 0), M.iron, segments=10, rings=6)
        for s in (-1, 1):
            for i in range(4):
                y = y0 + 0.12 + i * 0.08
                F.box(f'louvre_f{s}{i}', 0.5, 0.04, 0.02, (0, y, s * 0.355), M.shadow)
                F.box(f'louvre_s{s}{i}', 0.02, 0.04, 0.5, (s * 0.355, y, 0), M.shadow)
    F.cone('cupola_roof', 0.62, 0.02, 0.3, (0, y0 + 0.65, 0), M.barn_roof, verts=4, smooth=False, rot=(0, 0, 45))
    # the weathervane: a rod, an arrow, and the four points
    G.rod('vane_rod', F.at(0, y0 + 0.75, 0), F.at(0, 6.0, 0), 0.015, material=M.iron, verts=6)
    if detail:
        G.cube('vane_shaft', (0.55, 0.015, 0.015), at=F.at(0, 5.9, 0), material=M.iron)
        F.cone('vane_head', 0.06, 0.0, 0.14, (-0.32, 5.9, 0), M.iron, verts=4, rot=(0, 90, 0), smooth=False)
        G.cube('vane_tail', (0.14, 0.01, 0.12), at=F.at(0.24, 5.92, 0), material=M.iron)
        G.cube('vane_ns', (0.012, 0.3, 0.012), at=F.at(0, 5.82, 0), material=M.iron)
        G.cube('vane_ew', (0.3, 0.012, 0.012), at=F.at(0, 5.82, 0), material=M.iron)


def barn():
    _barn_shell()
    _barn_battens()
    _barn_corners()
    _barn_doors(BD, 1)
    _barn_doors(-BD, -1)
    for s in (-1, 1):
        for i, u in enumerate((-1.6, 0, 1.6)):
            _barn_window(f'win{s}{i}', (u, 1.72), 'x', s)
    _cupola()
    return G.settle([G.join_all('barn')])


def barn_far():
    """The barn from afar: the walls, roof, plinth, the doors' white X and the cupola, a few hundred
    triangles."""
    _barn_shell(detail=False)
    _barn_doors(BD, 1, detail=False)
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'corner{sx}{sz}', 0.16, EAVE - 0.3, 0.16, (sx * (BW - 0.06), 0.3 + (EAVE - 0.3) / 2, sz * (BD - 0.06)), M.barn_trim)
    for s in (-1, 1):
        for u in (-1.6, 0, 1.6):
            _prism(f'win{s}{u}', [(u - 0.43, 1.29), (u + 0.43, 1.29), (u + 0.43, 2.15), (u - 0.43, 2.15)], s * BW, s * (BW + 0.03), M.barn_trim, 'x')
            _prism(f'pane{s}{u}', [(u - 0.33, 1.39), (u + 0.33, 1.39), (u + 0.33, 2.05), (u - 0.33, 2.05)], s * BW, s * (BW + 0.04), M.shadow, 'x')
    _cupola(detail=False)
    return G.settle([G.join_all('barn_far')])


# --- the silo --------------------------------------------------------------------------------------------
SILO_R, SILO_TOP = 1.2, 6.0


def _silo_drum(detail=True):
    """The concrete pad, the corrugated drum (stiffening rings every 0.6 m) and the domed cap."""
    G.revolve('pad', [(0, 0), (1.35, 0), (1.35, 0.2), (1.3, 0.24), (0, 0.24)], material=M.concrete, segments=32 if detail else 16)
    prof = [(0, 0.24), (SILO_R, 0.24)]
    if detail:
        # stiffening rings every 0.6 m, the sheet between them corrugated (three waves a bay)
        for k in range(10):
            y0 = 0.24 + 0.6 * k
            for w in range(3):
                prof += [(SILO_R + 0.02, y0 + 0.1 + 0.15 * w + 0.0375), (SILO_R, y0 + 0.1 + 0.15 * w + 0.1125)]
            y = y0 + 0.6
            if y < SILO_TOP - 0.1:
                prof += [(SILO_R, y - 0.05), (SILO_R + 0.03, y - 0.02), (SILO_R + 0.03, y + 0.02), (SILO_R, y + 0.05)]
        prof = [p for p in prof if p[1] < SILO_TOP - 0.03]
    prof += [(SILO_R, SILO_TOP), (SILO_R + 0.04, SILO_TOP), (SILO_R + 0.04, SILO_TOP + 0.06), (0, SILO_TOP + 0.06)]
    G.revolve('drum', prof, material=M.silo, segments=24 if detail else 16, smooth=detail)
    cap = [(0, SILO_TOP + 0.04), (SILO_R + 0.06, SILO_TOP + 0.04)]
    rings = 7 if detail else 4
    for i in range(1, rings + 1):
        a = math.radians(90 * i / rings)
        cap.append(((SILO_R + 0.06) * math.cos(a) * 1.0, SILO_TOP + 0.04 + 0.62 * math.sin(a)))
    G.revolve('cap', cap, material=M.silo_cap, segments=32 if detail else 16)
    G.revolve('vent', [(0, SILO_TOP + 0.6), (0.16, SILO_TOP + 0.6), (0.16, SILO_TOP + 0.72), (0.24, SILO_TOP + 0.74),
                       (0.2, SILO_TOP + 0.84), (0.06, SILO_TOP + 0.9), (0, SILO_TOP + 0.9)],
              material=M.silo, segments=16 if detail else 8)


def silo():
    _silo_drum()
    # the cap's seams, from its eave over the dome to the vent, and a band round its eave
    for i in range(12):
        t = math.tau * i / 12
        pts = []
        for k in range(7):
            e = math.radians(90 * k / 6)
            r = (SILO_R + 0.06) * math.cos(e) + 0.012
            if r < 0.2:
                break
            pts.append((r * math.cos(t), r * math.sin(t), SILO_TOP + 0.05 + 0.62 * math.sin(e) + 0.005))
        G.sweep(f'cap_seam{i}', pts, 0.014, material=M.silo_cap, verts=4, resolution=2)
    G.torus('cap_eave', SILO_R + 0.06, 0.025, at=(0, 0, SILO_TOP + 0.05), material=M.silo_cap, major_segments=24, minor_segments=4)
    # anchor brackets round the foot, bolted into the pad
    for i in range(12):
        t = math.tau * (i + 0.5) / 12
        G.cube(f'anchor{i}', (0.05, 0.08, 0.14), at=((SILO_R + 0.03) * math.cos(t), (SILO_R + 0.03) * math.sin(t), 0.31),
               material=M.steel, rot=(0, 0, math.degrees(t)), bevel=0.008, bevel_segments=1)
    # the ladder up the front: its rails in one bent piece each, over the eave and up the cap to the walkway
    z = SILO_R + 0.15
    for x in (-0.2, 0.2):
        G.kit.tube(F, f'rail{x}', G.kit.fillet([(x, 0.3, z), (x, SILO_TOP + 0.05, z), (x, SILO_TOP + 0.4, z - 0.25)], 0.12),
                   0.02, M.steel, verts=6, round=1)
        for y in (1.0, 3.0, 5.0):
            F.box(f'stay{x}{y}', 0.03, 0.04, 0.15, (x, y, z - 0.075), M.steel, bevel=0.006, bevel_segments=1)
            F.cyl(f'stay_bolt{x}{y}', 0.018, 0.04, (x, y - 0.03, z - 0.14), M.steel, verts=6)
    for i in range(18):
        G.rod(f'rung{i}', F.at(-0.2, 0.6 + i * 0.3, z), F.at(0.2, 0.6 + i * 0.3, z), 0.013, material=M.steel, verts=6)
    # the walkway round the top: posts, a top rail and a mid rail
    for k, h in enumerate((0.3, 0.16)):
        G.torus(f'walkway{k}', SILO_R + 0.05, 0.02 if k == 0 else 0.014, at=(0, 0, SILO_TOP + h), material=M.steel,
                major_segments=32, minor_segments=4)
    for i in range(8):
        t = 22.5 + 45 * i
        G.cylinder(f'post{i}', 0.016, 0.32, at=(1.25 * G.cosd(t), 1.25 * G.sind(t), SILO_TOP + 0.15), material=M.steel, verts=6)
    # the roof hatch at the ladder's head, on its hinge
    G.cube('roof_hatch', (0.42, 0.36, 0.05), at=F.at(0, SILO_TOP + 0.36, SILO_R - 0.45), material=M.silo_cap,
           rot=(math.degrees(0.5), 0, 0), bevel=0.012, bevel_segments=1)
    # a framed door at the foot with its handle, and the unloading chute: one bent tube out and down into a hopper
    F.box('hatch', 0.04, 0.7, 0.6, (-SILO_R - 0.01, 0.75, 0), M.steel, bevel=0.01, bevel_segments=1)
    for y in (0.42, 1.08):
        F.box(f'hatch_hinge{y}', 0.03, 0.05, 0.14, (-SILO_R - 0.035, y, -0.27), M.iron, bevel=0.005, bevel_segments=1)
    hx = -SILO_R - 0.03
    F.sweep('hatch_handle', G.kit.fillet([(hx, 0.66, 0.18), (hx - 0.05, 0.66, 0.18), (hx - 0.05, 0.84, 0.18), (hx, 0.84, 0.18)], 0.02),
            0.012, M.iron, verts=6)
    G.kit.tube(F, 'chute', G.kit.fillet([(0.95, 0.85, -0.52), (1.32, 0.55, -0.72), (1.32, 0.3, -0.72)], 0.12), 0.11,
               M.steel, verts=10, round=1)
    F.box('hopper', 0.26, 0.2, 0.26, (1.32, 0.2, -0.72), M.steel, bevel=0.02, bevel_segments=1)
    return G.settle([G.join_all('silo')])


def silo_far():
    """The silo from afar: pad, plain drum, cap and vent; the ladder a flat strip."""
    _silo_drum(detail=False)
    F.box('ladder', 0.44, SILO_TOP - 0.3, 0.04, (0, 0.3 + (SILO_TOP - 0.3) / 2, SILO_R + 0.15), M.steel)
    return G.settle([G.join_all('silo_far')])


# --- the tractor -----------------------------------------------------------------------------------------
# A compact tractor's track: 1.58 m over the rear tyres (it was 1.98, a utility tractor's).
REAR_R, REAR_W, REAR_X, REAR_Z = 0.725, 0.38, 0.6, -0.75
FRONT_R, FRONT_W, FRONT_X, FRONT_Z = 0.39, 0.26, 0.5, 1.3


def _wheel(name, x, z, r, w, lugs, segments=28, detail=True):
    """A tyre with a yellow rim and dark hub, its axle along x, on the ground at (x, z); `lugs` chevron tread lugs, each
    a bar slanting from a shoulder in to the middle, the sides taking turns."""
    at = F.at(x, r, z)
    hw = w / 2
    s = 1 if x > 0 else -1
    if not lugs:
        G.cylinder(f'{name}_tyre', r, w, at=at, material=M.tyre, verts=segments, rot=(0, 90, 0))
    else:
        G.revolve(f'{name}_tyre', [(r * 0.6, -hw), (r - 0.06, -hw), (r - 0.015, -hw + 0.03), (r, -hw + 0.08), (r, hw - 0.08),
                                   (r - 0.015, hw - 0.03), (r - 0.06, hw), (r * 0.6, hw), (r * 0.6, -hw)],
                  at=at, material=M.tyre, segments=segments, rot=(0, 90, 0))
    G.cylinder(f'{name}_rim', r * 0.6, w - 0.04, at=at, material=M.rims, verts=segments - 4, rot=(0, 90, 0))
    G.cylinder(f'{name}_dish', r * 0.42, w + 0.02, at=at, material=M.rims, verts=segments - 8, rot=(0, 90, 0))
    G.cylinder(f'{name}_hub', r * 0.16, w + 0.04, at=at, material=M.iron, verts=12 if detail else 8, rot=(0, 90, 0))
    if detail:
        # the rim's lip (a big wheel's), and the wheel nuts round the hub on the outside
        if r > 0.5:
            G.torus(f'{name}_lip', r * 0.6, 0.012, at=F.at(x + s * (hw - 0.02), r, z), material=M.rims, rot=(0, 90, 0),
                    major_segments=segments - 4, minor_segments=4)
        for k in range(6):
            a = math.tau * k / 6
            G.cylinder(f'{name}_nut{k}', 0.016, 0.03, at=F.at(x + s * (hw + 0.005), r + r * 0.25 * math.sin(a), z + r * 0.25 * math.cos(a)),
                       material=M.iron, verts=6, rot=(0, 90, 0))
    h = 0.05 if r > 0.5 else 0.035
    for i in range(lugs):
        a = math.tau * i / lugs
        side = 1 if i % 2 else -1
        pts = []
        for u, da in ((0.47, 0.0), (0.04, 0.16 if r > 0.5 else 0.2)):           # from the shoulder in, slanting forward
            b = a + da * (r * 0.0 + 1.0) * 0.5
            pts.append(F.at(x + side * u * w, r + (r - 0.012) * math.sin(b), z + (r - 0.012) * math.cos(b)))
        mid = F.at(x, r + math.sin(a), z + math.cos(a))
        up = tuple(mid[k] - at[k] for k in range(3))
        G.sweep(f'{name}_lug{i}', pts, None, material=M.tyre, up=up, smooth=False,
                profile=[(h + 0.012, -0.035), (h + 0.012, 0.035), (0.0, 0.045), (0.0, -0.045)])


def _hood(detail=True):
    """The hood: a rounded box, lofted, its nose rounding in to the grille."""
    n = 20 if detail else 10
    secs = []
    for z, rx, ry in ((-0.1, 0.39, 0.25), (1.3, 0.39, 0.25), (1.52, 0.375, 0.235), (1.62, 0.34, 0.205)):
        secs.append([F.at(u, 0.97 + v, z) for u, v, _ in G.ring(rx, ry, 0, n, square=6)])
    G.loft('hood', secs, material=M.tractor, angle=50)


def tractor(detail=True):
    green = M.tractor
    for s in (-1, 1):
        _wheel(f'rear{s}', s * REAR_X, REAR_Z, REAR_R, REAR_W, 24 if detail else 0, 28 if detail else 16, detail)
        _wheel(f'front{s}', s * FRONT_X, FRONT_Z, FRONT_R, FRONT_W, 18 if detail else 0, 28 if detail else 14, detail)
        # the fender over each rear wheel: a rounded sheet with a rolled outer edge, and a step
        arc = [(s * 0.595, REAR_R + 0.82 * math.sin(math.radians(a)), REAR_Z + 0.82 * math.cos(math.radians(a)))
               for a in range(160, 4, -12 if detail else -26)]
        F.sweep(f'fender{s}', arc, None, green, profile=G.rounded_rect(0.39, 0.04, 0.015, corner=2 if detail else 1), up=(1, 0, 0),
                angle=50)
        if detail:
            F.sweep(f'fender_lip{s}', [(s * 0.79, y, z) for _, y, z in arc], 0.02, green, verts=6)
        F.box(f'step{s}', 0.3, 0.04, 0.3, (s * 0.38, 0.62, 0.08), M.iron, bevel=0.01 if detail else 0, bevel_segments=1)
    # chassis, axles, transmission housing under the cab, platform
    F.box('chassis', 0.56, 0.32, 2.6, (0, 0.55, 0.2), M.iron, bevel=0.02)
    F.box('front_axle', 1.0, 0.1, 0.12, (0, FRONT_R, FRONT_Z), M.iron)
    F.box('rear_axle', 1.2, 0.16, 0.16, (0, REAR_R, REAR_Z), M.iron)
    F.box('housing', 0.74, 0.55, 0.9, (0, 0.85, -0.7), green, bevel=0.05, bevel_segments=3)
    F.box('platform', 1.0, 0.05, 0.55, (0, 1.12, -0.35), M.iron)
    # the hood, its grille and lamps
    _hood(detail)
    F.box('grille', 0.6, 0.38, 0.04, (0, 0.95, 1.61), M.iron)
    # the canopy on its two bent frames, the seat
    for sx in (-1, 1):
        G.kit.tube(F, f'rops{sx}', G.kit.fillet([(sx * 0.6, 1.13, -1.1), (sx * 0.6, 2.37, -1.1), (sx * 0.6, 2.37, 0.06),
                                                (sx * 0.6, 1.13, 0.06)], 0.1, steps=3 if detail else 1),
                   0.032, green, verts=8 if detail else 4, round=1)
    F.box('seat_back', 0.46, 0.34, 0.08, (0, 1.55, -0.86), M.seat, bevel=0.03 if detail else 0)
    if not detail:
        # from afar: the lamps, exhaust and canopy; no grille bars, vents, weights or levers
        G.cylinder('exhaust', 0.045, 0.75, at=F.at(0.24, 1.55, 1.22), material=M.iron, verts=6)
        F.box('canopy', 1.32, 0.08, 1.38, (0, 2.42, -0.52), green)
        return G.settle([G.join_all('tractor_far')])
    F.soft('canopy', 1.32, 0.08, 1.38, (0, 2.42, -0.52), green, round=0.035, round_segments=2)
    F.extrude('grille_frame', G.rounded_rect(0.64, 0.42, 0.04, corner=2), 0.03, (0, 0.95, 1.635), green,
              holes=[[(u, v + dy) for u, v in G.rounded_rect(0.52, 0.042, 0.018, corner=2)] for dy in (-0.14, -0.07, 0.0, 0.07, 0.14)])
    for s in (-1, 1):
        G.cylinder(f'lamp{s}', 0.055, 0.04, at=F.at(s * 0.26, 1.12, 1.63), material=M.lamp, verts=12, rot=(90, 0, 0))
        G.torus(f'lamp_bezel{s}', 0.058, 0.012, at=F.at(s * 0.26, 1.12, 1.65), material=green, rot=(90, 0, 0),
                major_segments=12, minor_segments=4)
        for i in range(4):
            F.box(f'vent{s}{i}', 0.02, 0.05, 0.25, (s * 0.392, 1.0 + i * 0.07, 1.1), M.iron)
    # a fuel cap on the hood, and the front weights: five suitcase weights on their bracket, each with its handle
    F.cyl('fuel_cap', 0.045, 0.03, (0.0, 1.225, 0.1), M.iron, verts=10)
    F.box('weight_bracket', 0.66, 0.06, 0.1, (0, 0.47, 1.64), M.iron)
    for k in range(5):
        x = -0.26 + 0.13 * k
        F.box(f'weight{k}', 0.12, 0.22, 0.13, (x, 0.6, 1.69), M.iron, bevel=0.015, bevel_segments=1)
        F.sweep(f'weight_handle{k}', G.kit.fillet([(x, 0.7, 1.68), (x, 0.75, 1.68), (x, 0.75, 1.73), (x, 0.7, 1.73)], 0.015,
                                                   steps=1), 0.01, M.iron, verts=4)
    # the exhaust stack with its muffler and rain flap, and the air pre-cleaner
    G.cylinder('exhaust', 0.045, 0.75, at=F.at(0.24, 1.55, 1.22), material=M.iron, verts=10)
    G.cylinder('muffler', 0.065, 0.3, at=F.at(0.24, 1.45, 1.22), material=M.iron, verts=10)
    G.cube('exhaust_cap', (0.1, 0.1, 0.008), at=F.at(0.24, 1.94, 1.21), material=M.iron, rot=(18, 0, 0))
    G.cylinder('precleaner', 0.055, 0.3, at=F.at(-0.24, 1.33, 1.22), material=M.iron, verts=10)
    G.cylinder('precleaner_bowl', 0.075, 0.08, at=F.at(-0.24, 1.5, 1.22), material=M.lamp, verts=12)
    # the dashboard with its gauges, the steering wheel on three spokes, the seat
    F.box('dash', 0.5, 0.3, 0.14, (0, 1.32, -0.05), green, bevel=0.03)
    for k, x in enumerate((-0.1, 0.1)):
        G.cylinder(f'gauge{k}', 0.035, 0.02, at=F.at(x, 1.42, -0.125), material=M.seat, verts=10, rot=(80, 0, 0))
    G.rod('column', F.at(0, 1.4, -0.08), F.at(0, 1.62, -0.28), 0.025, material=M.iron, verts=8)
    G.torus('steering', 0.17, 0.018, at=F.at(0, 1.64, -0.3), material=M.seat, rot=(-55, 0, 0), major_segments=18, minor_segments=6)
    hub = F.at(0, 1.64, -0.3)
    for k in range(3):
        a = math.tau * k / 3 + 0.5
        tip = F.at(0.165 * math.cos(a), 1.64 + 0.165 * math.sin(a) * math.cos(math.radians(55)),
                   -0.3 - 0.165 * math.sin(a) * math.sin(math.radians(55)))
        G.rod(f'spoke{k}', hub, tip, 0.01, material=M.seat, verts=4)
    G.cylinder('seat_post', 0.04, 0.2, at=F.at(0, 1.22, -0.62), material=M.iron, verts=8)
    F.box('seat_pan', 0.48, 0.03, 0.44, (0, 1.31, -0.62), M.iron, bevel=0.01, bevel_segments=1)
    F.soft('seat', 0.46, 0.08, 0.42, (0, 1.36, -0.62), M.seat, round=0.03, puff=0.012, round_segments=1, flat=2)
    F.soft('seat_back_pad', 0.42, 0.3, 0.05, (0, 1.56, -0.81), M.seat, round=0.02, puff=0.01, round_segments=1, flat=2)
    # tail lamps on the rear frame, the drawbar and the three-point hitch with its top link, and the PTO stub
    for s in (-1, 1):
        F.box(f'tail{s}', 0.07, 0.1, 0.03, (s * 0.6, 1.5, -1.145), M.tail_lamp, bevel=0.008, bevel_segments=1)
    F.box('drawbar', 0.14, 0.06, 0.32, (0, 0.5, -1.38), M.iron)
    for s in (-1, 1):
        G.rod(f'link{s}', F.at(s * 0.2, 0.6, -1.0), F.at(s * 0.3, 0.42, -1.5), 0.03, material=M.iron, verts=6)
        F.sphere(f'link_ball{s}', 0.04, (s * 0.3, 0.42, -1.5), M.iron, segments=8, rings=5)
    G.rod('top_link', F.at(0, 1.0, -1.12), F.at(0, 0.85, -1.48), 0.025, material=M.iron, verts=6)
    G.rod('pto', F.at(0, 0.62, -1.13), F.at(0, 0.62, -1.24), 0.03, material=M.iron, verts=8)
    return G.settle([G.join_all('tractor')])


# --- hay -------------------------------------------------------------------------------------------------
# A round bale as they come off the baler: 1.2 m across and 1.2 m long, up to an adult's chest (it was 0.99 x 0.86 m,
# waist-high), on 2 x 2 tiles. Its rings and twine are spaced for its size.
BALE_R, BALE_L = 0.6, 1.2
TWINE = (-0.35, 0.0, 0.35)


def hayBale():
    """A round bale lying on its side: its hay a little lumpy, each end rolled in one long spiral to a tuft at its core,
    three bands of twine round it, their ends tucked in, and loose wisps of straw on its top and ends."""
    y = BALE_R - 0.012     # a little settled into the ground
    hl = BALE_L / 2
    bale = G.revolve('bale', [(0, -hl), (BALE_R - 0.09, -hl), (BALE_R - 0.03, -hl + 0.02), (BALE_R, -hl + 0.08),
                              (BALE_R + 0.004, -0.3), (BALE_R + 0.005, 0), (BALE_R + 0.004, 0.3), (BALE_R, hl - 0.08),
                              (BALE_R - 0.03, hl - 0.02), (BALE_R - 0.09, hl), (0, hl)],
                     at=F.at(0, y, 0), material=M.hay, segments=32, rot=(0, 90, 0))
    G.jitter(bale, 0.007, 0.12, seed=2.0)
    for s in (-1, 1):
        # the roll's spiral on each end, from the core out
        pts = []
        for k in range(84):
            t = k / 83
            a = s * t * 4.2 * math.tau
            r = 0.05 + 0.45 * t
            pts.append((s * (hl - 0.002), y + r * math.sin(a), r * math.cos(a)))
        F.sweep(f'spiral{s}', pts, 0.013, M.hay_dark, verts=4, scale=[0.7, 1.0, 1.0, 1.1])
        G.sphere(f'core{s}', 0.07, at=F.at(s * (hl - 0.01), y, 0), material=M.hay_dark, scale=(0.35, 1, 1), segments=8, rings=5)
    for k, x in enumerate(TWINE):
        G.torus(f'twine{k}', BALE_R + 0.006, 0.008, at=F.at(x, y, 0), material=M.twine, rot=(0, 90, 0),
                major_segments=32, minor_segments=4)
    # the twine's tied ends, hanging a little down the front
    for k, x in enumerate(TWINE[:2]):
        F.sweep(f'twine_end{k}', [(x, y + 0.3, BALE_R - 0.05), (x + 0.03, y + 0.15, BALE_R - 0.005), (x + 0.05, y - 0.02, BALE_R + 0.004)],
                0.006, M.twine, verts=4, resolution=2)
    # loose wisps of straw lying on it
    for i in range(18):
        a = math.radians(40 + 100 * ((i * 0.618) % 1.0))
        x = -hl + 0.08 + (BALE_L - 0.16) * ((i * 0.381) % 1.0)
        d = math.radians(-25 + 50 * ((i * 0.7) % 1.0))
        p0 = (x, y + (BALE_R + 0.008) * math.sin(a), (BALE_R + 0.008) * math.cos(a))
        p1 = (x + 0.12 * math.sin(d), y + (BALE_R + 0.012) * math.sin(a + 0.12), (BALE_R + 0.012) * math.cos(a + 0.12))
        F.sweep(f'wisp{i}', [p0, p1], 0.005, M.straw, verts=3, scale=[1.0, 0.5])
    for s in (-1, 1):
        for i in range(5):
            a = math.tau * i / 5 + 0.4 * s
            r = 0.2 + 0.07 * i
            p0 = (s * (hl - 0.004), y + r * math.sin(a), r * math.cos(a))
            p1 = (s * (hl + 0.006), y + (r + 0.1) * math.sin(a + 0.3), (r + 0.1) * math.cos(a + 0.3))
            F.sweep(f'end_wisp{s}{i}', [p0, p1], 0.006, M.straw, verts=3, scale=[1.0, 0.4])
    return G.settle([G.join_all('hayBale')])


def hayBale_far():
    """The bale from afar: a plainer roll, one dark ring on each end, the twine flat bands."""
    y, hl = BALE_R - 0.012, BALE_L / 2
    G.revolve('bale', [(0, -hl), (BALE_R - 0.06, -hl), (BALE_R, -hl + 0.07), (BALE_R + 0.005, 0), (BALE_R, hl - 0.07),
                       (BALE_R - 0.06, hl), (0, hl)], at=F.at(0, y, 0), material=M.hay, segments=14, rot=(0, 90, 0))
    for s in (-1, 1):
        G.cylinder(f'end{s}', 0.36, 0.01, at=F.at(s * (hl - 0.002), y, 0), material=M.hay_dark, rot=(0, 90, 0), verts=10)
    for k, x in enumerate(TWINE):
        G.cylinder(f'twine{k}', BALE_R + 0.02, 0.016, at=F.at(x, y, 0), material=M.twine, rot=(0, 90, 0), verts=14)
    return G.settle([G.join_all('hayBale_far')])


def _square_bale(name, x, y, detail=True):
    """A small square bale lying flat, `x` across and its middle `y` up: pressed in flakes along its length, each a
    little proud of the next, tied twice round its length, the cut ends darker."""
    if not detail:
        G.cube(name, (0.92, 0.86, 0.44), at=F.at(x, y, 0), material=M.hay)
        for dz in (-0.22, 0.22):
            G.cube(f'{name}_twine{dz}', (0.935, 0.02, 0.455), at=F.at(x, y, dz), material=M.twine)
        return
    n = 9
    edges = [-0.46 + 0.92 * (i + 0.18 * math.sin(i * 2.3 + x * 7)) / n if 0 < i < n else -0.46 + 0.92 * i / n for i in range(n + 1)]
    for i in range(n):
        u0, u1 = edges[i], edges[i + 1]
        k = 1.0 + 0.01 * math.sin(i * 2.7 + x * 5) + 0.006 * math.cos(i * 1.3)
        F.soft(f'{name}_flake{i}', u1 - u0 + 0.004, 0.44 * k - 0.004, 0.86 * (2 - k) - 0.004,
               (x + (u0 + u1) / 2, y + 0.004 * math.sin(i * 1.9), 0.005 * math.cos(i * 2.2)),
               M.hay_dark if i in (0, n - 1) else M.hay, round=0.014, round_segments=1, flat=1)
    for dz in (-0.22, 0.22):
        loop = [(x + u, y + v, dz) for u, v in G.rounded_rect(0.935, 0.455, 0.04, corner=2)]
        F.sweep(f'{name}_twine{dz}', loop, 0.008, M.twine, verts=4, closed=True)
    for i in range(4):                                                    # loose straw on its top
        u = -0.35 + 0.23 * i
        F.sweep(f'{name}_wisp{i}', [(x + u, y + 0.223, -0.3 + 0.17 * i), (x + u + 0.1, y + 0.226, -0.24 + 0.17 * i)], 0.005,
                M.straw, verts=3, scale=[1.0, 0.5])


def hayStack(detail=True):
    """Three square bales: two side by side and one on top, a little askew."""
    _square_bale('bale_l', -0.48, 0.225, detail)
    _square_bale('bale_r', 0.48, 0.225, detail)
    with G.new_parts() as top:
        _square_bale('bale_top', 0.0, 0.685, detail)
    joined = G.join(top, 'bale_top', origin=F.at(0, 0.685, 0))
    joined.rotation_euler[2] = math.radians(3)
    return G.settle([G.join_all('hayStack' if detail else 'hayStack_far')])


# --- the scarecrow ---------------------------------------------------------------------------------------
POST_Z = -0.05


def _straw_tuft(name, at, direction, n=5, length=0.13, spread=18):
    """A fan of straw ends poking out at `at` (a Blender point) along `direction` (Blender (x, y, z)), each a thin
    tapering stalk, bent a little."""
    dx, dy, dz = direction
    k = math.sqrt(dx * dx + dy * dy + dz * dz)
    dx, dy, dz = dx / k, dy / k, dz / k
    for i in range(n):
        t = (i - (n - 1) / 2) / max(1, n - 1)
        a = math.radians(spread * t * 2)
        # turn the direction about the vertical a little, and droop alternate ones
        ex = dx * math.cos(a) - dy * math.sin(a)
        ey = dx * math.sin(a) + dy * math.cos(a)
        ez = dz - (0.25 if i % 2 else 0.05)
        ln = length * (0.85 + 0.3 * ((i * 0.618) % 1.0))
        mid = (at[0] + ex * ln * 0.5, at[1] + ey * ln * 0.5, at[2] + ez * ln * 0.5 + 0.006)
        end = (at[0] + ex * ln, at[1] + ey * ln, at[2] + (ez - 0.12) * ln)
        G.sweep(f'{name}{i}', [at, mid, end], 0.008, material=M.straw, verts=3, scale=[1.0, 0.8, 0.3], smooth=False)


# The shirt's body, a rounded section at each height: (height, half its width, half its depth).
TORSO = [(0.97, 0.205, 0.125), (1.1, 0.2, 0.12), (1.3, 0.205, 0.118), (1.47, 0.215, 0.115), (1.54, 0.18, 0.1), (1.585, 0.085, 0.07)]
SQ = 3.0


def _torso_depth(x, y):
    """How far in front of the post the shirt's front is at (x, height y): on its rounded section there."""
    for (y0, rx0, rz0), (y1, rx1, rz1) in zip(TORSO, TORSO[1:]):
        if y <= y1:
            k = max(0.0, (y - y0) / (y1 - y0))
            rx, rz = rx0 + (rx1 - rx0) * k, rz0 + (rz1 - rz0) * k
            return rz * max(0.0, 1 - abs(x / rx) ** SQ) ** (1 / SQ)
    return 0.0


def scarecrow():
    # the post, rough and a little crooked, and the cross-bar inside the sleeves
    post = F.sweep('post', [(0, 0.0, POST_Z), (0.006, 0.9, POST_Z + 0.004), (-0.004, 1.9, POST_Z)], 0.04, M.post, verts=8,
                   scale=[1.05, 1.0, 0.92])
    G.jitter(post, 0.006, 0.2, seed=1.5, up=0.0)
    F.sweep('crossbar', [(-0.43, 1.45, POST_Z), (0.43, 1.45, POST_Z + 0.01)], None, M.post,
            profile=G.rounded_rect(0.06, 0.055, 0.012, corner=1), up=(0, 0, 1))
    # the shirt: a body rounded in section, sleeves bagging round the bar, collar, cuffs, its plaid lines and buttons
    G.loft('shirt', [[F.at(u, y, v) for u, v, _ in G.ring(rx, rz, 0, 16, square=SQ)] for y, rx, rz in TORSO], material=M.shirt)
    for s in (-1, 1):
        F.sweep(f'sleeve{s}', [(s * 0.14, 1.47, 0), (s * 0.24, 1.455, 0.005), (s * 0.33, 1.435, -0.003), (s * 0.4, 1.42, 0)],
                0.075, M.shirt, verts=10, resolution=2, scale=[1.0, 0.92, 1.02, 0.95])
        G.cylinder(f'cuff{s}', 0.08, 0.04, at=F.at(s * 0.4, 1.42, 0), material=M.shirt_check, rot=(0, 90 + s * 12, 0), verts=10)
        _straw_tuft(f'hand{s}', F.at(s * 0.42, 1.42, 0), (-s, 0, -0.1), length=0.07)
        for i, x in enumerate((0.22, 0.3)):
            G.torus(f'check_sleeve{s}{i}', 0.077, 0.008, at=F.at(s * x, 1.455 - 0.15 * (x - 0.16), 0), material=M.shirt_check,
                    rot=(0, 90 + s * 12, 0), major_segments=10, minor_segments=4)
    for i, x in enumerate((-0.12, 0.0, 0.12)):                              # the plaid, on the shirt's front
        F.sweep(f'check_v{i}', [(x, y, _torso_depth(x, y) + 0.004) for y in (1.0, 1.15, 1.3, 1.45)], 0.009, M.shirt_check,
                verts=4, resolution=2)
    for i, y in enumerate((1.08, 1.22, 1.36)):
        F.sweep(f'check_h{i}', [(x, y, _torso_depth(x, y) + 0.004) for x in (-0.19, -0.1, 0.0, 0.1, 0.19)], 0.009,
                M.shirt_check, verts=4, resolution=2)
    for i, y in enumerate((1.12, 1.25, 1.38, 1.5)):
        F.sphere(f'button{i}', 0.014, (0.0, y, _torso_depth(0.0, y) + 0.008), M.stitch, segments=6, rings=4)
    F.box('pocket', 0.09, 0.09, 0.012, (-0.1, 1.38, _torso_depth(-0.1, 1.38) + 0.002), M.shirt_check, bevel=0.004, bevel_segments=1)
    F.soft('collar', 0.26, 0.05, 0.2, (0, 1.58, 0), M.shirt_check, round=0.02, round_segments=1)
    # trousers hanging below, a patch on one knee, straw at the ankles, a rope belt tied in front
    F.soft('seat', 0.38, 0.16, 0.22, (0, 0.95, 0), M.trousers, round=0.05, round_segments=2)
    for s in (-1, 1):
        F.sweep(f'leg{s}', [(s * 0.09, 0.97, 0), (s * 0.105, 0.72, 0.012), (s * 0.12, 0.49, 0.022)], 0.072, M.trousers, verts=10,
                resolution=2, scale=[1.0, 0.92, 1.05])
        F.sweep(f'hem{s}', [(s * 0.12, 0.505, 0.022), (s * 0.12, 0.485, 0.022)], 0.079, M.trousers, verts=10)
        _straw_tuft(f'ankle{s}', F.at(s * 0.12, 0.47, 0.02), (s * 0.3, 0, -1), length=0.09)
    F.soft('patch', 0.09, 0.1, 0.016, (0.105, 0.7, 0.084), M.patch, round=0.006, round_segments=1)
    for k in range(4):                                                        # its big stitches
        F.box(f'patch_stitch{k}', 0.005, 0.02, 0.006, (0.06 + 0.03 * k, 0.755, 0.093), M.stitch)
    F.sweep('belt', [(0.22 * math.cos(math.tau * k / 16), 0.995, 0.135 * math.sin(math.tau * k / 16)) for k in range(16)],
            0.014, M.twine, verts=5, closed=True)
    F.sphere('belt_knot', 0.026, (0.06, 0.99, 0.135), M.twine, segments=8, rings=5)
    for k, dx in enumerate((-0.01, 0.02)):
        F.sweep(f'belt_end{k}', [(0.06 + dx, 0.98, 0.14), (0.065 + dx * 2, 0.9, 0.145)], 0.009, M.twine, verts=4)
    _straw_tuft('waist', F.at(0, 0.98, 0.1), (0, -1, -0.4), n=4, length=0.08)
    # the sack head, gathered and tied at the neck, with a stitched face: crossed eyes, a nose, a stitched smile
    G.sphere('head', 0.17, at=F.at(0, 1.78, 0), material=M.sack, scale=(0.95, 0.9, 1.06), segments=14, rings=9)
    F.cone('gather', 0.1, 0.05, 0.08, (0, 1.64, 0), M.sack, verts=10)
    G.torus('neck_tie', 0.06, 0.015, at=F.at(0, 1.635, 0), material=M.twine, major_segments=10, minor_segments=4)
    _straw_tuft('neck', F.at(0, 1.62, 0.04), (0, -1, -0.5), n=4, length=0.07)
    for s in (-1, 1):
        for k in (-1, 1):
            G.cube(f'eye{s}{k}', (0.05, 0.008, 0.012), at=F.at(s * 0.06, 1.82, 0.152), material=M.stitch, rot=(0, 45 * k, 0))
    F.cone('nose', 0.025, 0.0, 0.07, (0, 1.76, 0.18), M.straw, verts=6, rot=(90, 0, 0))
    smile = [(0.085 * G.sind(a), 1.725 - 0.03 * G.cosd(a), 0.0) for a in range(-55, 56, 22)]
    smile = [(x, y, 0.153 * math.sqrt(max(0.0, 1 - (x / 0.16) ** 2 - ((y - 1.78) / 0.18) ** 2)) + 0.003) for x, y, _ in smile]
    F.sweep('smile', smile, 0.005, M.stitch, verts=4)
    for i, (x, y, z) in enumerate(smile[1:-1]):
        F.box(f'smile_stitch{i}', 0.005, 0.03, 0.008, (x, y, z), M.stitch)
    # the floppy hat: a dented crown, a brim drooping in waves, a band; straw sticking out under it
    brim = G.revolve('brim', [(0.14, 0.0), (0.2, -0.008), (0.27, -0.022), (0.275, -0.012), (0.2, 0.004), (0.14, 0.012)],
                     at=F.at(0, 1.92, 0), material=M.hat, segments=24, rot=(4, -3, 0))
    for v in brim.data.vertices:
        r = math.hypot(v.co.x, v.co.y)
        if r > 0.16:
            v.co.z -= 0.035 * ((r - 0.16) / 0.11) * (0.5 + 0.5 * math.sin(3 * math.atan2(v.co.y, v.co.x) + 0.7))
    G.revolve('crown', [(0.0, 0.17), (0.06, 0.155), (0.1, 0.18), (0.12, 0.165), (0.145, 0.01), (0.148, -0.005), (0.0, -0.005)],
              at=F.at(0, 1.93, 0), material=M.hat, segments=14, rot=(4, -6, 0))
    G.revolve('hatband', [(0.146, 0.008), (0.149, 0.01), (0.146, 0.05), (0.141, 0.052)], at=F.at(0, 1.93, 0), material=M.shirt_check,
              segments=14, rot=(4, -6, 0))
    for s in (-1, 1):
        _straw_tuft(f'hair{s}', F.at(s * 0.15, 1.86, 0), (-s, 0, -0.6), n=4, length=0.08)
    scare = G.join(G.meshes(), 'scarecrow', origin=F.at(0, 0, POST_Z))
    G.settle()
    # a slow sway in the wind about the foot of its post, side to side
    G.rock(scare, 'sway', 2.5, seconds=3.0, axis='Y')
    return G.meshes()


# --- the trough ------------------------------------------------------------------------------------------
TR_R, TR_HALF = 0.35, 0.55   # a stock tank: round ends of 0.35 m, a straight middle 1.1 m long


def trough():
    """A galvanised oval stock tank: corrugated sides with seams where its ends join, a rolled rim, water inside, on two
    skids; a float valve inside one end, fed by a pipe up from the ground through the end, with its tap, and a drain
    plug at the other."""
    y0, top = 0.1, 0.65
    prof = [(TR_R - 0.02, y0), (TR_R, y0 + 0.03)]
    for y in (0.2, 0.27, 0.34, 0.41, 0.48, 0.55):
        prof += [(TR_R, y - 0.025), (TR_R + 0.012, y), (TR_R, y + 0.025)]
    prof += [(TR_R, top - 0.04), (TR_R + 0.018, top - 0.02), (TR_R + 0.02, top), (TR_R, top + 0.004),
             (TR_R - 0.018, top - 0.015), (TR_R - 0.018, y0 + 0.04)]
    _trough_tank(prof, n=12)
    # the seams where the round ends meet the straight sides, riveted
    for x in (-TR_HALF, TR_HALF):
        for s in (-1, 1):
            F.box(f'seam{x}{s}', 0.03, top - y0 - 0.08, 0.008, (x, (top + y0) / 2, s * (TR_R + 0.012)), M.trough)
            for k in range(5):
                F.sphere(f'rivet{x}{s}{k}', 0.007, (x, y0 + 0.08 + k * 0.1, s * (TR_R + 0.017)), M.steel, segments=6, rings=4)
    # the float valve hung inside the rim at one end, and its feed pipe: up from the ground, over the rim and down in
    F.box('valve', 0.12, 0.09, 0.08, (-0.78, top - 0.04, -0.12), M.steel, bevel=0.01, bevel_segments=1)
    G.sphere('float', 0.055, at=F.at(-0.6, top - 0.09, -0.12), material=M.steel, segments=10, rings=6)
    G.rod('float_arm', F.at(-0.74, top - 0.04, -0.12), F.at(-0.6, top - 0.08, -0.12), 0.008, material=M.steel, verts=6)
    G.kit.tube(F, 'feed', G.kit.fillet([(-0.935, 0.0, -0.2), (-0.935, 0.5, -0.2), (-0.84, 0.5, -0.12)], 0.04), 0.016, M.steel,
               verts=6, round=1)
    F.cyl('feed_tap', 0.024, 0.05, (-0.935, 0.3, -0.2), M.steel, verts=8)
    G.rod('feed_lever', F.at(-0.935, 0.3, -0.2), F.at(-0.935, 0.3, -0.11), 0.008, material=M.iron, verts=4)
    # the drain plug low on the other end
    G.cylinder('drain', 0.03, 0.03, at=F.at(TR_HALF + TR_R + 0.01, y0 + 0.06, 0), material=M.steel, verts=6, rot=(0, 90, 0))
    return G.settle([G.join_all('trough')])


def _trough_tank(prof, n=8):
    y0, top = 0.1, 0.65
    _stadium_sweep('tank', prof, TR_HALF, M.trough, n)
    _stadium_slab('tank_floor', TR_R - 0.01, TR_HALF, y0, y0 + 0.045, M.trough, n)
    _stadium_slab('water', TR_R - 0.017, TR_HALF, y0 + 0.04, top - 0.07, M.water, n)
    for x in (-0.62, 0.62):
        F.box(f'skid{x}', 0.12, y0, 0.66, (x, y0 / 2, 0), M.steel, bevel=0.01 if n > 4 else 0, bevel_segments=1)


def trough_far():
    """The trough from afar: plain sides and rim (no corrugations), the water, the skids, the valve."""
    y0, top = 0.1, 0.65
    _trough_tank([(TR_R - 0.02, y0), (TR_R, y0 + 0.03), (TR_R + 0.02, top), (TR_R - 0.018, top), (TR_R - 0.018, y0 + 0.04)], n=4)
    F.box('valve', 0.12, 0.09, 0.08, (-0.78, top - 0.04, -0.12), M.steel)
    return G.settle([G.join_all('trough_far')])


# --- the chicken coop ------------------------------------------------------------------------------------
CF, CB = 0.3, -0.8           # the house's front and back (depth)
CX = 0.75                    # half its width
CY0, CEAVE, CRIDGE = 0.55, 1.45, 1.78
CRZ = (CF + CB) / 2          # the ridge's depth
HATCH = (0.17, 0.6, 0.95)    # the pop hole: half its width, its bottom and top
HATCH_UP = 0.32              # how far the hatch stands up its runners by day, the hole open


def _coop_roof_line():
    dz, dy = CRZ - CF, CRIDGE - CEAVE
    k = math.hypot(dz, dy)
    e = 0.16
    return [(CB + e * dz / k, CEAVE - e * dy / k), (CRZ, CRIDGE), (CF - e * dz / k, CEAVE - e * dy / k)]


def _coop_body(detail=True):
    wall = [(CB, CY0), (CF, CY0), (CF, CEAVE), (CRZ, CRIDGE), (CB, CEAVE)]
    _prism('house', wall, -CX, CX, M.coop, axis='x')
    _shell('roof', _coop_roof_line(), 0.05, -CX - 0.1, CX + 0.1, M.coop_roof, axis='x')
    G.cube('ridge_cap', (2 * CX + 0.24, 0.1, 0.05), at=F.at(0, CRIDGE + 0.06, CRZ), material=M.coop_roof)
    for sx in (-1, 1):
        for z in (CB + 0.1, CF - 0.1):
            F.box(f'leg{sx}{z}', 0.08, CY0 + 0.02, 0.08, (sx * (CX - 0.1), CY0 / 2, z), M.coop)
    F.box('floor', 2 * CX + 0.04, 0.05, CF - CB + 0.04, (0, CY0 + 0.01, (CF + CB) / 2), M.coop)
    # the ramp up to the pop hole, three planks with cleats across them
    a, b = (CF + 0.68, 0.0), (CF + 0.02, CY0 + 0.03)
    if not detail:
        _strut('ramp', a, b, 0.035, -0.15, 0.15, M.coop, axis='x')
        return
    for k, (w0, w1) in enumerate(((-0.15, -0.054), (-0.046, 0.046), (0.054, 0.15))):
        _strut(f'ramp{k}', a, b, 0.035, w0, w1, M.coop, axis='x')
    for k in range(1, 6):
        t = k / 6
        z, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
        F.box(f'cleat{k}', 0.28, 0.025, 0.025, (0, y + 0.03, z), M.coop_trim)
    # braces between the legs on each side
    for sx in (-1, 1):
        x = sx * (CX - 0.1)
        _strut(f'brace{sx}', (CB + 0.14, 0.1), (CF - 0.14, CY0 - 0.04), 0.05, x - 0.015, x + 0.015, M.coop, axis='x')


def _coop_siding():
    """Lap siding: boards a hand wide, each tilted out at its foot over the one below, round the pop hole and windows."""
    rows = [CY0 + 0.02 + 0.112 * k for k in range(9)]
    for k, (y0, y1) in enumerate(zip(rows, rows[1:])):
        for face, z, s in (('f', CF, 1), ('b', CB, -1)):
            spans = [(-CX + 0.05, CX - 0.05)]
            if face == 'f' and y0 < HATCH[1] + 0.36:
                spans = [(-CX + 0.05, -0.245), (0.245, CX - 0.05)]
            for i, (x0, x1) in enumerate(spans):
                _strut(f'board_{face}{k}{i}', (z + s * 0.017, y0), (z + s * 0.004, y1 + 0.006), 0.012, x0, x1, M.coop, axis='x')
    gable = rows + [CEAVE + 0.11 * k for k in range(1, 4)]
    for k, (y0, y1) in enumerate(zip(gable, gable[1:])):
        f = max(0.0, (CRIDGE - min(y1, CRIDGE)) / (CRIDGE - CEAVE)) if y1 > CEAVE else 1.0
        d0, d1 = CRZ - (CRZ - CB) * f + 0.03, CRZ + (CF - CRZ) * f - 0.03
        if d1 - d0 < 0.08:
            continue
        spans = [(d0, d1)]
        if y1 > 0.86 and y0 < 1.24:
            spans = [(d0, -0.39), (-0.01, d1)]
        for sx in (-1, 1):
            x = sx * CX
            for i, (w0, w1) in enumerate(spans):
                _strut(f'board_s{sx}{k}{i}', (x + sx * 0.017, y0), (x + sx * 0.004, y1 + 0.006), 0.012, w0, w1, M.coop, axis='z')


def _coop_shingles():
    """The roof in four courses of shingles a side, each course's tabs staggered over the last's joints."""
    line = _coop_roof_line()
    for side, (p0, p1) in enumerate(((line[0], line[1]), (line[2], line[1]))):
        (z0, y0), (z1, y1) = p0, p1
        dz, dy = z1 - z0, y1 - y0
        k = math.hypot(dz, dy)
        nz, ny = (-dy / k, dz / k) if side == 0 else (dy / k, -dz / k)   # out of the roof, up
        for c in range(4):
            t0, t1 = c / 4, min(1.0, (c + 1) / 4 + 0.05)
            a = (z0 + dz * t0 + nz * 0.068, y0 + dy * t0 + ny * 0.068)
            b = (z0 + dz * t1 + nz * 0.056, y0 + dy * t1 + ny * 0.056)
            n = 7
            span = 2 * CX + 0.2
            for i in range(n + (c % 2)):
                w0 = -span / 2 + span * (i - 0.5 * (c % 2)) / n
                w1 = w0 + span / n
                w0, w1 = max(w0, -span / 2) + 0.006, min(w1, span / 2) - 0.006
                _strut(f'shingle{side}{c}{i}', a, b, 0.014, w0, w1, M.coop_roof, axis='x')


def coop():
    with G.new_parts() as body:
        _coop_body()
        _coop_siding()
        _coop_shingles()
        # corner boards and the eave trim
        for sx in (-1, 1):
            for z in (CB, CF):
                F.box(f'corner{sx}{z}', 0.06, CEAVE - CY0, 0.06, (sx * (CX - 0.02), (CY0 + CEAVE) / 2, z + (0.02 if z == CF else -0.02)), M.coop_trim)
        # the pop hole, dark behind the hatch, and the runners the hatch slides up in
        hw, h0, h1 = HATCH
        F.box('pop_hole', 2 * hw, h1 - h0, 0.02, (0, (h0 + h1) / 2, CF + 0.005), M.coop_dark)
        for s in (-1, 1):
            F.box(f'runner{s}', 0.04, 0.72, 0.03, (s * (hw + 0.03), h0 + 0.33, CF + 0.02), M.coop_trim)
        F.box('runner_top', 2 * hw + 0.1, 0.04, 0.03, (0, h0 + 0.69, CF + 0.02), M.coop_trim)
        # a window with wire mesh on each side, its sill, and a slatted vent high in each gable
        for s in (-1, 1):
            x = s * CX
            _prism(f'win{s}', [(-0.35, 0.9), (-0.05, 0.9), (-0.05, 1.2), (-0.35, 1.2)], x, x + s * 0.01, M.coop_dark, axis='x')
            _frame(f'win{s}_frame', -0.35, 0.9, -0.05, 1.2, x, 0.04, s * 0.03, M.coop_trim, axis='x')
            for k in range(3):
                u = -0.35 + 0.075 * (k + 1)
                _prism(f'wire_v{s}{k}', [(u - 0.004, 0.9), (u + 0.004, 0.9), (u + 0.004, 1.2), (u - 0.004, 1.2)], x, x + s * 0.016, M.coop_trim, axis='x')
                v = 0.9 + 0.075 * (k + 1)
                _prism(f'wire_h{s}{k}', [(-0.35, v - 0.004), (-0.05, v - 0.004), (-0.05, v + 0.004), (-0.35, v + 0.004)], x, x + s * 0.016, M.coop_trim, axis='x')
            F.box(f'win_sill{s}', 0.05, 0.025, 0.38, (x + s * 0.03, 0.865, -0.2), M.coop_trim, bevel=0.006, bevel_segments=1)
            _prism(f'vent{s}', [(CRZ - 0.12, 1.53), (CRZ + 0.12, 1.53), (CRZ + 0.12, 1.63), (CRZ - 0.12, 1.63)], x, x + s * 0.025, M.coop_dark, axis='x')
            _frame(f'vent{s}_frame', CRZ - 0.12, 1.53, CRZ + 0.12, 1.63, x, 0.03, s * 0.035, M.coop_trim, axis='x')
            for k in range(2):
                v = 1.555 + 0.04 * k
                _strut(f'vent_slat{s}{k}', (CRZ - 0.11, v), (CRZ + 0.11, v), 0.012, x, x + s * 0.03, M.coop_trim, axis='x')
        # the nest box on the back, its lid sloping like the roof on two hinges, its latch
        F.box('nest', 0.9, 0.36, 0.14, (0, 0.92, CB - 0.07), M.coop)
        _strut('nest_lid', (CB - 0.17, 1.08), (CB + 0.01, 1.16), 0.035, -0.48, 0.48, M.coop_roof, axis='x')
        for x in (-0.3, 0.3):
            _strut(f'nest_hinge{x}', (CB - 0.06, 1.155), (CB + 0.02, 1.19), 0.008, x - 0.03, x + 0.03, M.iron, axis='x')
        F.box('nest_latch', 0.06, 0.04, 0.02, (0, 1.0, CB - 0.15), M.iron)
        # a feeder hanging under the house
        F.cyl('feeder', 0.08, 0.2, (0.3, 0.34, -0.3), M.trough, verts=12)
        F.cone('feeder_lid', 0.095, 0.02, 0.06, (0.3, 0.47, -0.3), M.trough, verts=12)
        F.cyl('feeder_pan', 0.13, 0.03, (0.3, 0.225, -0.3), M.trough, verts=14)
        G.rod('feeder_wire', F.at(0.3, 0.5, -0.3), F.at(0.3, CY0 - 0.015, -0.3), 0.005, material=M.iron, verts=4)
    G.join(body, 'coop', origin=(0, 0, 0))
    # the pop-hole hatch: its own part, its origin the middle of its top edge. By day it stands up in its runners,
    # the hole open for the hens; at night `close` slides it down over the hole (`motions.night`).
    with G.new_parts() as hatch_parts:
        hw, h0, h1 = HATCH
        F.box('hatch', 2 * hw + 0.02, h1 - h0 + 0.02, 0.025, (0, (h0 + h1) / 2, CF + 0.03), M.coop_trim, bevel=0.005)
        for y in (h0 + 0.05, h1 - 0.05):
            F.box(f'hatch_rail{y}', 2 * hw - 0.04, 0.035, 0.012, (0, y, CF + 0.048), M.coop_trim, bevel=0.003, bevel_segments=1)
        _strut('hatch_brace', (-hw + 0.04, h0 + 0.07), (hw - 0.04, h1 - 0.07), 0.03, CF + 0.042, CF + 0.054, M.coop_trim)
        G.sphere('hatch_knob', 0.016, at=F.at(0, h0 + 0.06, CF + 0.065), material=M.iron, segments=8, rings=5)
    hatch = G.join(hatch_parts, 'hatch', origin=F.at(0, h1 + 0.01, CF + 0.03))
    hatch.location.z += HATCH_UP
    G.settle()
    G.slide(hatch, 'close', (0, 0, -HATCH_UP), seconds=1.0)
    return G.meshes()


def coop_far():
    """The coop from afar: the house, roof, legs and ramp, its trims, the dark pop hole with the hatch up, the windows
    as dark panels in their frames, the nest box: a few hundred triangles (decimated, its legs and ramp went to
    spikes)."""
    _coop_body(detail=False)
    for sx in (-1, 1):
        for z in (CB, CF):
            F.box(f'corner{sx}{z}', 0.06, CEAVE - CY0, 0.06, (sx * (CX - 0.02), (CY0 + CEAVE) / 2, z + (0.02 if z == CF else -0.02)), M.coop_trim)
        x = sx * CX
        _prism(f'win{sx}', [(-0.37, 0.88), (-0.03, 0.88), (-0.03, 1.22), (-0.37, 1.22)], x, x + sx * 0.02, M.coop_trim, axis='x')
        _prism(f'pane{sx}', [(-0.33, 0.92), (-0.07, 0.92), (-0.07, 1.18), (-0.33, 1.18)], x, x + sx * 0.025, M.coop_dark, axis='x')
    hw, h0, h1 = HATCH
    F.box('pop_hole', 2 * hw, h1 - h0, 0.02, (0, (h0 + h1) / 2, CF + 0.005), M.coop_dark)
    F.box('hatch', 2 * hw + 0.02, h1 - h0 + 0.02, 0.025, (0, (h0 + h1) / 2 + HATCH_UP, CF + 0.03), M.coop_trim)
    F.box('nest', 0.9, 0.36, 0.14, (0, 0.92, CB - 0.07), M.coop)
    _strut('nest_lid', (CB - 0.17, 1.08), (CB + 0.01, 1.16), 0.035, -0.48, 0.48, M.coop_roof, axis='x')
    return G.settle([G.join_all('coop_far')])


# (width, height, depth): the old parts' bounds, give or take (each model's tolerance)
TOLERANCE = 0.03

FAMILY = G.Family(
    'farm', M,
    [G.Model('barn', barn, size=(6.04, 6.0, 5.94), tolerance=TOLERANCE, far=barn_far,
             paint=[('barn', 'Paint'), ('barn_roof', 'Roof'), ('barn_trim', 'Trim')]),
     G.Model('silo', silo, size=(2.77, 6.9, 2.72), tolerance=TOLERANCE, far=silo_far,
             paint=[('silo', 'Paint'), ('silo_cap', 'Roof')]),
     G.Model('tractor', tractor, size=(1.62, 2.5, 3.28), tolerance=TOLERANCE, far=lambda: tractor(detail=False),
             paint=[('tractor', 'Paint'), ('rims', 'Wheels')]),
     G.Model('hayBale', hayBale, size=(1.224, 1.21, 1.21), tolerance=TOLERANCE, far=hayBale_far,
             paint=[('twine', 'Twine')]),
     G.Model('hayStack', hayStack, size=(1.9, 0.915, 0.9), tolerance=TOLERANCE, far=lambda: hayStack(detail=False),
             paint=[('twine', 'Twine')]),
     G.Model('scarecrow', scarecrow, size=(0.985, 2.13, 0.54), tolerance=TOLERANCE, far=0.25,
             motions={'idle': 'sway'},
             paint=[('shirt', 'Shirt'), ('trousers', 'Trousers'), ('hat', 'Hat')]),
     G.Model('trough', trough, size=(1.84, 0.655, 0.74), tolerance=TOLERANCE, far=trough_far,
             paint=[('trough', 'Paint')]),
     G.Model('coop', coop, size=(1.74, 1.88, 1.97), tolerance=TOLERANCE, far=coop_far, motions={'night': 'close'},
             paint=[('coop', 'Paint'), ('coop_roof', 'Roof'), ('coop_trim', 'Trim')])],
    comment='Blender-built farm (tools/art/blender/families/farm.py, docs/objects/community.md CM5): the '
            'red barn, the grain silo, the little tractor, the round hay bale, the stack of hay bales, '
            'the scarecrow (an ambient sway), the water trough and the chicken coop (its pop-hole hatch '
            'shut at night, `close`). Each has a far version (`far`, E7) for the town overview. Listed in '
            'the base manifest\'s packs.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
