# The farm (docs/objects/community.md CM5): the red barn, the grain silo, the little tractor, the round hay bale,
# the stack of hay bales, the scarecrow, the water trough and the chicken coop (`model.farm.<id>`). They stand on
# the farm lots of farm worlds (web/src/game/town.ts `farmstead`) and are seen from afar in the town
# overview, so each has a far version (E7): the barn and the silo a hand-built one, the rest decimated.
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
    G.cube('ridge', (0.2, 2 * ROOF_D + 0.04, 0.08), at=(0, 0, RIDGE + ROOF_T + 0.02), material=M.barn_roof)


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
                G.cylinder(f'handle_{side}', 0.025, 0.3, at=F.at(-gap * 6 if side == 'l' else gap * 6, 1.35, z + 0.12), material=M.iron, verts=8)
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


# Where the battens stop: (u0, u1, y0, y1) round each door and window, per face.
BARN_OPENINGS = {
    'front': [(-1.38, 1.38, 0.0, 2.8), (-0.58, 0.58, 3.12, 4.18)],
    'back': [(0.53, 1.67, 0.0, 2.27), (-0.47, 0.47, 3.23, 4.07)],
    'side': [(c - 0.45, c + 0.45, 1.27, 2.17) for c in (-1.6, 0, 1.6)],
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
        for k in range(9):
            y = 0.84 + 0.6 * k
            prof += [(SILO_R, y - 0.05), (SILO_R + 0.03, y - 0.02), (SILO_R + 0.03, y + 0.02), (SILO_R, y + 0.05)]
    prof += [(SILO_R, SILO_TOP), (SILO_R + 0.04, SILO_TOP), (SILO_R + 0.04, SILO_TOP + 0.06), (0, SILO_TOP + 0.06)]
    G.revolve('drum', prof, material=M.silo, segments=32 if detail else 16, smooth=False)
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
    # the ladder up the front, and a rail round the top
    z = SILO_R + 0.15
    for x in (-0.2, 0.2):
        G.cube(f'rail{x}', (0.04, 0.04, SILO_TOP - 0.3), at=F.at(x, 0.3 + (SILO_TOP - 0.3) / 2, z), material=M.steel)
        for y in (1.0, 3.0, 5.0):
            G.cube(f'stay{x}{y}', (0.03, 0.15, 0.03), at=F.at(x, y, z - 0.075), material=M.steel)
    for i in range(18):
        G.cube(f'rung{i}', (0.4, 0.025, 0.025), at=F.at(0, 0.6 + i * 0.3, z), material=M.steel)
    G.torus('walkway', SILO_R + 0.05, 0.025, at=(0, 0, SILO_TOP + 0.3), material=M.steel, major_segments=32, minor_segments=4)
    for i in range(8):
        a = 22.5 + 45 * i
        G.cube(f'post{i}', (0.03, 0.03, 0.3), at=(1.25 * G.cosd(a), 1.25 * G.sind(a), SILO_TOP + 0.15), material=M.steel)
    # a hatch at the foot, and the unloading chute out to one side
    F.box('hatch', 0.04, 0.7, 0.6, (-SILO_R - 0.01, 0.75, 0), M.steel)
    G.rod('chute', F.at(1.0, 0.75, -0.55), F.at(1.32, 0.45, -0.72), 0.12, material=M.steel, verts=12)
    G.rod('auger', F.at(1.32, 0.45, -0.72), F.at(1.32, 0.12, -0.72), 0.1, material=M.steel, verts=12)
    return G.settle([G.join_all('silo')])


def silo_far():
    """The silo from afar: pad, plain drum, cap and vent; the ladder a flat strip."""
    _silo_drum(detail=False)
    F.box('ladder', 0.44, SILO_TOP - 0.3, 0.04, (0, 0.3 + (SILO_TOP - 0.3) / 2, SILO_R + 0.15), M.steel)
    return G.settle([G.join_all('silo_far')])


# --- the tractor -----------------------------------------------------------------------------------------
REAR_R, REAR_W, REAR_X, REAR_Z = 0.725, 0.42, 0.76, -0.75
FRONT_R, FRONT_W, FRONT_X, FRONT_Z = 0.39, 0.28, 0.58, 1.3


def _wheel(name, x, z, r, w, lugs, segments=28):
    """A tyre with a yellow rim and dark hub, its axle along x, on the ground at (x, z); `lugs` tread lugs."""
    at = F.at(x, r, z)
    hw = w / 2
    if not lugs:
        G.cylinder(f'{name}_tyre', r, w, at=at, material=M.tyre, verts=segments, rot=(0, 90, 0))
    else:
        G.revolve(f'{name}_tyre', [(r * 0.6, -hw), (r - 0.06, -hw), (r - 0.015, -hw + 0.03), (r, -hw + 0.08), (r, hw - 0.08),
                                   (r - 0.015, hw - 0.03), (r - 0.06, hw), (r * 0.6, hw), (r * 0.6, -hw)],
                  at=at, material=M.tyre, segments=segments, rot=(0, 90, 0))
    G.cylinder(f'{name}_rim', r * 0.6, w - 0.04, at=at, material=M.rims, verts=segments - 4, rot=(0, 90, 0))
    G.cylinder(f'{name}_dish', r * 0.42, w + 0.02, at=at, material=M.rims, verts=segments - 8, rot=(0, 90, 0))
    G.cylinder(f'{name}_hub', r * 0.16, w + 0.04, at=at, material=M.iron, verts=12, rot=(0, 90, 0))
    for i in range(lugs):
        a = 360 * i / lugs
        off = (0.25 if i % 2 else -0.25) * w
        h = 0.05 if r > 0.5 else 0.03
        G.cube(f'{name}_lug{i}', (w * 0.48, 0.07 if r > 0.5 else 0.05, h),
               at=(at[0] + off, at[1] - (r + h / 2 - 0.01) * G.sind(a), at[2] + (r + h / 2 - 0.01) * G.cosd(a)),
               material=M.tyre, rot=(a, 0, 0))


def tractor(detail=True):
    green = M.tractor
    for s in (-1, 1):
        _wheel(f'rear{s}', s * REAR_X, REAR_Z, REAR_R, REAR_W, 24 if detail else 0, 28 if detail else 16)
        _wheel(f'front{s}', s * FRONT_X, FRONT_Z, FRONT_R, FRONT_W, 18 if detail else 0, 28 if detail else 14)
        # the fender over each rear wheel, and a step
        arc = [(REAR_Z + 0.82 * math.cos(math.radians(a)), REAR_R + 0.82 * math.sin(math.radians(a))) for a in range(160, 5, -15)]
        _shell(f'fender{s}', arc, 0.04, s * 0.53, s * 0.99, green, axis='x')
        F.box(f'step{s}', 0.3, 0.04, 0.3, (s * 0.38, 0.62, 0.08), M.iron)
    # chassis, axles, transmission housing under the cab, platform
    F.box('chassis', 0.56, 0.32, 2.6, (0, 0.55, 0.2), M.iron, bevel=0.02)
    F.box('front_axle', 1.0, 0.1, 0.12, (0, FRONT_R, FRONT_Z), M.iron)
    F.box('rear_axle', 1.2, 0.16, 0.16, (0, REAR_R, REAR_Z), M.iron)
    F.box('housing', 0.74, 0.55, 0.9, (0, 0.85, -0.7), green, bevel=0.05, bevel_segments=3)
    F.box('platform', 1.0, 0.05, 0.55, (0, 1.12, -0.35), M.iron)
    # the hood, its grille and lamps, the weights in front
    F.box('hood', 0.78, 0.5, 1.72, (0, 0.97, 0.76), green, bevel=0.09, bevel_segments=3)
    F.box('grille', 0.6, 0.38, 0.04, (0, 0.95, 1.625), M.iron)
    if not detail:
        # from afar: the lamps, exhaust, seat and canopy on its posts; no grille bars, vents or levers
        G.cylinder('exhaust', 0.045, 0.75, at=F.at(0.24, 1.55, 1.22), material=M.iron, verts=6)
        F.box('seat_back', 0.46, 0.34, 0.08, (0, 1.55, -0.86), M.seat)
        for sx in (-1, 1):
            for z in (-1.1, 0.06):
                F.box(f'post{sx}{z}', 0.06, 1.26, 0.06, (sx * 0.6, 1.76, z), green)
        F.box('canopy', 1.32, 0.08, 1.38, (0, 2.42, -0.52), green)
        return G.settle([G.join_all('tractor_far')])
    for i in range(5):
        F.box(f'grille_bar{i}', 0.62, 0.025, 0.03, (0, 0.82 + i * 0.065, 1.645), green)
    for s in (-1, 1):
        G.cylinder(f'lamp{s}', 0.055, 0.04, at=F.at(s * 0.26, 1.12, 1.63), material=M.lamp, verts=12, rot=(90, 0, 0))
        for i in range(4):
            F.box(f'vent{s}{i}', 0.02, 0.05, 0.25, (s * 0.392, 1.0 + i * 0.07, 1.1), M.iron)
    F.box('weights', 0.62, 0.22, 0.12, (0, 0.58, 1.66), M.iron, bevel=0.015)
    # the exhaust stack and the air pre-cleaner
    G.cylinder('exhaust', 0.045, 0.75, at=F.at(0.24, 1.55, 1.22), material=M.iron, verts=10)
    F.box('exhaust_cap', 0.1, 0.012, 0.1, (0.24, 1.94, 1.21), M.iron)
    G.cylinder('precleaner', 0.055, 0.3, at=F.at(-0.24, 1.33, 1.22), material=M.iron, verts=10)
    G.cylinder('precleaner_bowl', 0.075, 0.08, at=F.at(-0.24, 1.5, 1.22), material=M.lamp, verts=12)
    # the dashboard, steering wheel, seat and the canopy on its frame
    F.box('dash', 0.5, 0.3, 0.14, (0, 1.32, -0.05), green, bevel=0.03)
    G.rod('column', F.at(0, 1.4, -0.08), F.at(0, 1.62, -0.28), 0.025, material=M.iron, verts=8)
    G.torus('steering', 0.17, 0.018, at=F.at(0, 1.64, -0.3), material=M.seat, rot=(-55, 0, 0), major_segments=18, minor_segments=6)
    G.cylinder('seat_post', 0.04, 0.2, at=F.at(0, 1.22, -0.62), material=M.iron, verts=8)
    F.box('seat', 0.46, 0.08, 0.42, (0, 1.36, -0.62), M.seat, bevel=0.03)
    F.box('seat_back', 0.46, 0.34, 0.08, (0, 1.55, -0.86), M.seat, bevel=0.03)
    for sx in (-1, 1):
        for z in (-1.1, 0.06):
            F.box(f'post{sx}{z}', 0.06, 1.26, 0.06, (sx * 0.6, 1.76, z), green)
    F.box('canopy', 1.32, 0.08, 1.38, (0, 2.42, -0.52), green, bevel=0.03)
    # tail lamps and the drawbar at the back
    for s in (-1, 1):
        F.box(f'tail{s}', 0.07, 0.1, 0.03, (s * 0.6, 1.5, -1.145), M.tail_lamp)
    F.box('drawbar', 0.14, 0.06, 0.32, (0, 0.5, -1.38), M.iron)
    for s in (-1, 1):
        G.rod(f'link{s}', F.at(s * 0.2, 0.6, -1.0), F.at(s * 0.3, 0.42, -1.5), 0.03, material=M.iron, verts=6)
    return G.settle([G.join_all('tractor')])


# --- hay -------------------------------------------------------------------------------------------------
BALE_R, BALE_L = 0.495, 0.86


def hayBale():
    """A round bale lying on its side, its ends rolled in a spiral, three bands of twine round it."""
    y = BALE_R - 0.012     # a little settled into the ground
    hl = BALE_L / 2
    G.revolve('bale', [(0, -hl), (BALE_R - 0.09, -hl), (BALE_R - 0.03, -hl + 0.02), (BALE_R, -hl + 0.08),
                       (BALE_R + 0.005, 0), (BALE_R, hl - 0.08), (BALE_R - 0.03, hl - 0.02), (BALE_R - 0.09, hl), (0, hl)],
              at=F.at(0, y, 0), material=M.hay, segments=28, rot=(0, 90, 0))
    for s in (-1, 1):
        for k, r in enumerate((0.1, 0.2, 0.3, 0.39)):
            G.torus(f'spiral{s}{k}', r, 0.013, at=F.at(s * (hl - 0.004), y, 0), material=M.hay_dark, rot=(0, 90, 0),
                    major_segments=8 + 4 * k, minor_segments=4)
        G.sphere(f'core{s}', 0.05, at=F.at(s * (hl - 0.005), y, 0), material=M.hay_dark, scale=(0.3, 1, 1), segments=8, rings=5)
    for k, x in enumerate((-0.25, 0.0, 0.25)):
        G.torus(f'twine{k}', BALE_R - 0.003, 0.008, at=F.at(x, y, 0), material=M.twine, rot=(0, 90, 0),
                major_segments=28, minor_segments=4)
    return G.settle([G.join_all('hayBale')])


def hayBale_far():
    """The bale from afar: a plainer roll, one dark ring on each end, the twine flat bands."""
    y, hl = BALE_R - 0.012, BALE_L / 2
    G.revolve('bale', [(0, -hl), (BALE_R - 0.06, -hl), (BALE_R, -hl + 0.07), (BALE_R + 0.005, 0), (BALE_R, hl - 0.07),
                       (BALE_R - 0.06, hl), (0, hl)], at=F.at(0, y, 0), material=M.hay, segments=14, rot=(0, 90, 0))
    for s in (-1, 1):
        G.cylinder(f'end{s}', 0.3, 0.01, at=F.at(s * (hl - 0.002), y, 0), material=M.hay_dark, rot=(0, 90, 0), verts=10)
    for k, x in enumerate((-0.25, 0.0, 0.25)):
        G.cylinder(f'twine{k}', BALE_R + 0.02, 0.016, at=F.at(x, y, 0), material=M.twine, rot=(0, 90, 0), verts=14)
    return G.settle([G.join_all('hayBale_far')])


def _square_bale(name, x, y):
    G.cube(name, (0.92, 0.86, 0.44), at=F.at(x, y, 0), material=M.hay, bevel=0.035, bevel_segments=2)
    for dz in (-0.22, 0.22):
        G.cube(f'{name}_twine{dz}', (0.935, 0.02, 0.455), at=F.at(x, y, dz), material=M.twine)
    # the cut ends' straw stands a little proud, in the darker hay
    for s in (-1, 1):
        G.cube(f'{name}_end{s}', (0.012, 0.8, 0.38), at=F.at(x + s * 0.461, y, 0), material=M.hay_dark)


def hayStack():
    """Three square bales: two side by side and one on top, a little askew."""
    _square_bale('bale_l', -0.48, 0.225)
    _square_bale('bale_r', 0.48, 0.225)
    with G.new_parts() as top:
        _square_bale('bale_top', 0.0, 0.685)
    joined = G.join(top, 'bale_top', origin=F.at(0, 0.685, 0))
    joined.rotation_euler[2] = math.radians(3)
    return G.settle([G.join_all('hayStack')])


# --- the scarecrow ---------------------------------------------------------------------------------------
POST_Z = -0.05


def _straw_tuft(name, at, direction, n=5, length=0.13, spread=18):
    """A fan of straw ends poking out at `at` (a Blender point) along `direction` (Blender (x, y, z))."""
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
        end = (at[0] + ex * length, at[1] + ey * length, at[2] + ez * length)
        G.rod(f'{name}{i}', at, end, 0.012, material=M.straw, verts=4, smooth=False)


def scarecrow():
    # the post and the cross-bar inside the sleeves
    G.cylinder('post', 0.04, 1.9, at=F.at(0, 0.95, POST_Z), material=M.post, verts=8)
    F.box('crossbar', 0.86, 0.06, 0.06, (0, 1.45, POST_Z), M.post)
    # the shirt: body, sleeves, collar, cuffs, and its plaid lines and a patch
    F.box('shirt', 0.42, 0.6, 0.24, (0, 1.26, 0), M.shirt, bevel=0.06, bevel_segments=3)
    for s in (-1, 1):
        G.rod(f'sleeve{s}', F.at(s * 0.16, 1.47, 0), F.at(s * 0.4, 1.42, 0), 0.075, material=M.shirt, verts=10)
        G.cylinder(f'cuff{s}', 0.08, 0.04, at=F.at(s * 0.4, 1.42, 0), material=M.shirt_check, rot=(0, 90 + s * 12, 0), verts=10)
        _straw_tuft(f'hand{s}', F.at(s * 0.42, 1.42, 0), (-s, 0, -0.1), length=0.07)
    for i, x in enumerate((-0.12, 0.0, 0.12)):
        F.box(f'check_v{i}', 0.025, 0.5, 0.02, (x, 1.25, 0.122), M.shirt_check)
    for i, y in enumerate((1.08, 1.22, 1.36, 1.5)):
        F.box(f'check_h{i}', 0.38, 0.025, 0.02, (0, y, 0.122), M.shirt_check)
    for s in (-1, 1):
        for i, x in enumerate((0.22, 0.3)):
            G.torus(f'check_sleeve{s}{i}', 0.077, 0.008, at=F.at(s * x, 1.46 - 0.15 * (x - 0.16), 0), material=M.shirt_check,
                    rot=(0, 90 + s * 12, 0), major_segments=10, minor_segments=4)
    F.box('collar', 0.26, 0.05, 0.2, (0, 1.57, 0), M.shirt_check, bevel=0.015)
    # trousers hanging below, a patch on one knee, straw at the ankles, a twine belt
    F.box('seat', 0.38, 0.14, 0.2, (0, 0.95, 0), M.trousers, bevel=0.04)
    for s in (-1, 1):
        G.rod(f'leg{s}', F.at(s * 0.09, 0.95, 0), F.at(s * 0.12, 0.48, 0.02), 0.07, material=M.trousers, verts=10)
        _straw_tuft(f'ankle{s}', F.at(s * 0.12, 0.47, 0.02), (s * 0.3, 0, -1), length=0.09)
    F.box('patch', 0.08, 0.09, 0.02, (0.1, 0.7, 0.083), M.patch)
    F.box('belt', 0.43, 0.035, 0.25, (0, 0.99, 0), M.twine, bevel=0.01)
    _straw_tuft('waist', F.at(0, 0.98, 0.1), (0, -1, -0.4), n=4, length=0.08)
    # the sack head, tied at the neck, with a stitched face
    G.sphere('head', 0.17, at=F.at(0, 1.78, 0), material=M.sack, scale=(0.95, 0.9, 1.06), segments=14, rings=9)
    G.torus('neck_tie', 0.07, 0.015, at=F.at(0, 1.63, 0), material=M.twine, major_segments=10, minor_segments=4)
    _straw_tuft('neck', F.at(0, 1.62, 0.04), (0, -1, -0.5), n=4, length=0.07)
    for s in (-1, 1):
        G.sphere(f'eye{s}', 0.025, at=F.at(s * 0.06, 1.82, 0.148), material=M.stitch, scale=(1, 0.5, 1), segments=8, rings=5)
    F.cone('nose', 0.025, 0.0, 0.07, (0, 1.76, 0.18), M.straw, verts=6, rot=(90, 0, 0))
    for i in range(5):
        a = -50 + 25 * i
        x, y = 0.075 * G.sind(a), 1.72 - 0.03 * G.cosd(a)
        F.box(f'smile{i}', 0.02, 0.012, 0.012, (x, y, 0.138 - 0.004 * abs(i - 2)), M.stitch)
    # the floppy hat, straw sticking out under its brim
    G.cylinder('brim', 0.27, 0.025, at=F.at(0, 1.915, 0), material=M.hat, rot=(4, -3, 0), verts=18)
    F.cone('crown', 0.15, 0.11, 0.19, (0, 2.02, 0), M.hat, verts=14, rot=(4, -6, 0))
    G.cylinder('hatband', 0.147, 0.035, at=F.at(0, 1.95, 0), material=M.shirt_check, rot=(4, -3, 0), verts=14)
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
    """A galvanised oval stock tank: corrugated sides, a rolled rim, water inside, on two skids."""
    y0, top = 0.1, 0.65
    prof = [(TR_R - 0.02, y0), (TR_R, y0 + 0.03)]
    for y in (0.22, 0.32, 0.42, 0.52):
        prof += [(TR_R, y - 0.035), (TR_R + 0.014, y), (TR_R, y + 0.035)]
    prof += [(TR_R, top - 0.04), (TR_R + 0.018, top - 0.02), (TR_R + 0.02, top), (TR_R, top + 0.004),
             (TR_R - 0.018, top - 0.015), (TR_R - 0.018, y0 + 0.04)]
    _trough_tank(prof)
    # the float valve hung inside the rim at one end
    F.box('valve', 0.12, 0.09, 0.08, (-0.78, top - 0.04, -0.12), M.steel)
    G.sphere('float', 0.055, at=F.at(-0.6, top - 0.09, -0.12), material=M.steel, segments=10, rings=6)
    G.rod('float_arm', F.at(-0.74, top - 0.04, -0.12), F.at(-0.6, top - 0.08, -0.12), 0.008, material=M.steel, verts=6)
    return G.settle([G.join_all('trough')])


def _trough_tank(prof, n=8):
    y0, top = 0.1, 0.65
    _stadium_sweep('tank', prof, TR_HALF, M.trough, n)
    _stadium_slab('tank_floor', TR_R - 0.01, TR_HALF, y0, y0 + 0.045, M.trough, n)
    _stadium_slab('water', TR_R - 0.017, TR_HALF, y0 + 0.04, top - 0.07, M.water, n)
    for x in (-0.62, 0.62):
        F.box(f'skid{x}', 0.12, y0, 0.66, (x, y0 / 2, 0), M.steel)


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
    # the ramp up to the pop hole, with its cleats
    a, b = (CF + 0.68, 0.0), (CF + 0.02, CY0 + 0.03)
    _strut('ramp', a, b, 0.035, -0.15, 0.15, M.coop, axis='x')
    if detail:
        for k in range(1, 6):
            t = k / 6
            z, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
            F.box(f'cleat{k}', 0.28, 0.025, 0.025, (0, y + 0.03, z), M.coop_trim)


def coop():
    with G.new_parts() as body:
        _coop_body()
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
        # a window with wire mesh on each side
        for s in (-1, 1):
            x = s * CX
            _prism(f'win{s}', [(-0.35, 0.9), (-0.05, 0.9), (-0.05, 1.2), (-0.35, 1.2)], x, x + s * 0.01, M.coop_dark, axis='x')
            _frame(f'win{s}_frame', -0.35, 0.9, -0.05, 1.2, x, 0.04, s * 0.03, M.coop_trim, axis='x')
            for k in range(3):
                u = -0.35 + 0.075 * (k + 1)
                _prism(f'wire_v{s}{k}', [(u - 0.004, 0.9), (u + 0.004, 0.9), (u + 0.004, 1.2), (u - 0.004, 1.2)], x, x + s * 0.016, M.coop_trim, axis='x')
                v = 0.9 + 0.075 * (k + 1)
                _prism(f'wire_h{s}{k}', [(-0.35, v - 0.004), (-0.05, v - 0.004), (-0.05, v + 0.004), (-0.35, v + 0.004)], x, x + s * 0.016, M.coop_trim, axis='x')
        # the nest box on the back, its lid sloping like the roof
        F.box('nest', 0.9, 0.36, 0.14, (0, 0.92, CB - 0.07), M.coop)
        _strut('nest_lid', (CB - 0.17, 1.08), (CB + 0.01, 1.16), 0.035, -0.48, 0.48, M.coop_roof, axis='x')
        F.box('nest_latch', 0.06, 0.04, 0.02, (0, 1.0, CB - 0.15), M.iron)
    G.join(body, 'coop', origin=(0, 0, 0))
    # the pop-hole hatch: its own part, its origin the middle of its top edge. By day it stands up in its runners,
    # the hole open for the hens; at night `close` slides it down over the hole (`motions.night`).
    with G.new_parts() as hatch_parts:
        hw, h0, h1 = HATCH
        F.box('hatch', 2 * hw + 0.02, h1 - h0 + 0.02, 0.025, (0, (h0 + h1) / 2, CF + 0.03), M.coop_trim, bevel=0.005)
        F.box('hatch_brace', 2 * hw - 0.04, 0.03, 0.012, (0, (h0 + h1) / 2, CF + 0.048), M.coop_trim)
        G.sphere('hatch_knob', 0.016, at=F.at(0, h0 + 0.06, CF + 0.055), material=M.iron, segments=8, rings=5)
    hatch = G.join(hatch_parts, 'hatch', origin=F.at(0, h1 + 0.01, CF + 0.03))
    hatch.location.z += HATCH_UP
    G.settle()
    G.slide(hatch, 'close', (0, 0, -HATCH_UP), seconds=1.0)
    return G.meshes()


# (width, height, depth): the old parts' bounds, give or take (each model's tolerance)
TOLERANCE = 0.03

FAMILY = G.Family(
    'farm', M,
    [G.Model('barn', barn, size=(6.04, 6.0, 5.94), tolerance=TOLERANCE, far=barn_far,
             paint=[('barn', 'Paint'), ('barn_roof', 'Roof'), ('barn_trim', 'Trim')]),
     G.Model('silo', silo, size=(2.77, 6.9, 2.72), tolerance=TOLERANCE, far=silo_far,
             paint=[('silo', 'Paint'), ('silo_cap', 'Roof')]),
     G.Model('tractor', tractor, size=(1.98, 2.5, 3.28), tolerance=TOLERANCE, far=lambda: tractor(detail=False),
             paint=[('tractor', 'Paint'), ('rims', 'Wheels')]),
     G.Model('hayBale', hayBale, size=(0.88, 1.0, 1.0), tolerance=TOLERANCE, far=hayBale_far,
             paint=[('twine', 'Twine')]),
     G.Model('hayStack', hayStack, size=(1.9, 0.915, 0.9), tolerance=TOLERANCE, far=0.25,
             paint=[('twine', 'Twine')]),
     G.Model('scarecrow', scarecrow, size=(0.985, 2.13, 0.54), tolerance=TOLERANCE, far=0.25,
             motions={'idle': 'sway'},
             paint=[('shirt', 'Shirt'), ('trousers', 'Trousers'), ('hat', 'Hat')]),
     G.Model('trough', trough, size=(1.84, 0.655, 0.74), tolerance=TOLERANCE, far=trough_far,
             paint=[('trough', 'Paint')]),
     G.Model('coop', coop, size=(1.74, 1.88, 1.97), tolerance=TOLERANCE, far=0.25, motions={'night': 'close'},
             paint=[('coop', 'Paint'), ('coop_roof', 'Roof'), ('coop_trim', 'Trim')])],
    comment='Blender-built farm (tools/art/blender/families/farm.py, docs/objects/community.md CM5): the '
            'red barn, the grain silo, the little tractor, the round hay bale, the stack of hay bales, '
            'the scarecrow (an ambient sway), the water trough and the chicken coop (its pop-hole hatch '
            'shut at night, `close`). Each has a far version (`far`, E7) for the town overview. Listed in '
            'the base manifest\'s packs.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
