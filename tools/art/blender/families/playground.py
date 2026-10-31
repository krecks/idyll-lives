# The playground (CM1): the slide (`model.slide`), the climbing frame (`model.climbingFrame`), the seesaw
# (`model.seesaw`), the spring rider (`model.springRider`), the merry-go-round (`model.merryGoRound`), the
# basketball hoop (`model.basketballHoop`), the ping-pong table (`model.pingPongTable`) and the park chess
# table (`model.parkChessTable`). Notes: playground.md.
#
# Each is built where the game places it (`F`, game axes) and stood on the floor without moving it across
# (`G.settle(centre=False)`, `centre=` on its `Model`), so the seats stay where the content puts its riders:
# `seat` heights and forward offsets in web/public/content/community.json, slots spread across the width.
# Town things, seen from far: every model has a far version (`far=0.25`).
#
# Moving parts:
#   seesaw         the plank with its seats and handles rocks ±12° about the axle (`seesaw`, a loop; riders ride)
#   springRider    the horse rocks ±10° on top of its spring (`rock`, a loop; the rider rides)
#   merryGoRound   the platform turns once in 3.5 s (`spin`, a loop; the riders ride round with it)
#   basketballHoop the net sways and stretches after each shot (`swish`, 1.4 s into the use, then every 3.2 s)
#   parkChessTable a white pawn slides one square on (`move`)
#
# Run:  cargo art playground [--only seesaw] [--preview]
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#2E6FA8', 'metal'),         # painted steel frames: the slide's tower, the seesaw's stand
    chute=('#F2C14E', 'gloss'),        # the slide's chute
    roof=('#C8402F', 'gloss'),         # the slide's roof
    tread=('#C9A06A', 'wood'),         # the slide's deck and ladder rungs
    rail=('#C3C7CB', 'metal'),         # galvanised rails, handles and poles
    steel=('#5C6168', 'metal'),        # axles, brackets, hubs, base plates
    rubber=('#2A2C2E', 'matte'),       # bumpers, grips, the ball's seams
    plank=('#F2C14E', 'gloss'),        # the seesaw's plank
    seat=('#D9483B', 'gloss'),         # the seesaw's seats
    horse=('#D9483B', 'gloss'),        # the spring rider's horse
    saddle=('#2E6FA8', 'gloss'),       # its saddle
    mane=('#F2F0EA', 'matte'),         # its mane and tail
    eye=('#1A1A1A', 'gloss'),
    grip=('#F2C14E', 'gloss'),         # the spring rider's handles and foot pegs
    spring=('#3C6B3A', 'metal'),       # the spring rider's coil
    platform=('#C8402F', 'gloss'),     # the merry-go-round's deck
    deck=('#F2C14E', 'gloss'),         # its stripes
    timber=('#C49A62', 'wood'),        # the climbing frame
    rope=('#D8C9A6', 'fabric'),        # its net
    shingle=('#B5463A', 'satin'),      # its roof
    flag=('#F2C14E', 'satin'),
    post=('#3E4347', 'metal'),         # the hoop's post, arm and ballast
    board=('#F4F4F2', 'gloss'),        # the backboard
    board_mark=('#C8402F', 'gloss'),   # its border and target square
    rim=('#E0602A', 'gloss'),          # the hoop
    net=('#F2F2F0', 'fabric'),         # the net (its own part: one draw)
    ball=('#D86A2A', 'matte'),
    table_top=('#2F6B4F', 'satin'),    # the ping-pong table's top
    lines=('#F2F2F0', 'satin'),        # its white lines
    concrete=('#A9ADB0', 'matte'),     # its legs
    ping=('#F7F5EE', 'gloss'),         # the ping-pong ball
    stone=('#C9C2B4', 'matte'),        # the chess table's top
    stool=('#8E8C88', 'matte'),        # its pedestal and stools
    square_light=('#E8DFC8', 'satin'),
    square_dark=('#3A3330', 'satin'),
    piece_white=('#F2EDE2', 'gloss'),
    piece_black=('#2A2624', 'gloss'),
)

F = G.GameFrame()
X = Vector((1, 0, 0))
Z = Vector((0, 0, 1))


# --- swept shapes -------------------------------------------------------------------------------------------
def _mesh(name, verts, faces, material, smooth=True):
    """A mesh object from points and faces (Blender world), its normals pointing out."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.validate()
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    for p in mesh.polygons:
        p.use_smooth = smooth
    return obj


def _frames(points):
    """(tangent, normal, binormal) along a path, the normal carried along without twisting."""
    ts = [(points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]).normalized() for i in range(len(points))]
    n = ts[0].cross(Z if abs(ts[0].z) < 0.9 else X).normalized()
    out = []
    for i, t in enumerate(ts):
        if i:
            n = ts[i - 1].rotation_difference(t) @ n
        n = (n - t * n.dot(t)).normalized()
        out.append((t, n, t.cross(n)))
    return out


def _tube(name, points, radius, material, verts=8):
    """A round tube through `points` (Blender world), capped: rails, bent bars, a coil."""
    points = [Vector(p) for p in points]
    vs, faces = [], []
    for p, (t, n, b) in zip(points, _frames(points)):
        for k in range(verts):
            a = math.tau * k / verts
            vs.append(p + radius * (math.cos(a) * n + math.sin(a) * b))
    for i in range(len(points) - 1):
        for k in range(verts):
            k2 = (k + 1) % verts
            faces.append((i * verts + k, i * verts + k2, (i + 1) * verts + k2, (i + 1) * verts + k))
    faces.append(tuple(range(verts)))
    faces.append(tuple(range((len(points) - 1) * verts, len(points) * verts)))
    return _mesh(name, vs, faces, material)


def _bezier(a, b, c, steps):
    """Points of the quadratic curve from `a` towards `b` to `c`."""
    a, b, c = Vector(a), Vector(b), Vector(c)
    return [(1 - t) ** 2 * a + 2 * (1 - t) * t * b + t ** 2 * c for t in (i / steps for i in range(steps + 1))]


# --- the slide ---------------------------------------------------------------------------------------------
# Game axes: the tower at the back (deck at 1.4 m), the ladder behind it, the chute running down to the front.
SLIDE_TOP = (1.43, -0.29)   # (height, depth) where the chute leaves the deck
SLIDE_END = (0.28, 0.95)    # and where its run-out ends
SQ = [(-1, -1), (1, -1), (1, 1), (-1, 1)]


def _chute(name, steps=22):
    """The chute: a U-section trough swept down a smoothstep curve (flat at the top and at the run-out), a rolled lip
    along the top of each wall. Returns the trough."""
    (y0, z0), (y1, z1) = SLIDE_TOP, SLIDE_END
    path = []
    for i in range(steps + 1):
        t = i / steps
        s = t * t * (3 - 2 * t)
        path.append(Vector(F.at(0, y0 + (y1 - y0) * s, z0 + (z1 - z0) * t)))
    # the section, across (u) and up (v) from the path: a floor and two walls, as one closed outline
    w, h, th = 0.25, 0.16, 0.025
    section = [(-w, h), (-w, 0), (w, 0), (w, h), (w - th, h), (w - th, th), (-w + th, th), (-w + th, h)]
    n = len(section)
    vs, faces, lips = [], [], ([], [])
    for i, p in enumerate(path):
        t = (path[min(i + 1, steps)] - path[max(i - 1, 0)]).normalized()
        up = t.cross(X).normalized()
        if up.z < 0:
            up = -up
        for u, v in section:
            vs.append(p + X * u + up * v)
        for k, u in enumerate((-w + th / 2, w - th / 2)):
            lips[k].append(p + X * u + up * (h + 0.004))
    for i in range(steps):
        for k in range(n):
            k2 = (k + 1) % n
            faces.append((i * n + k, i * n + k2, (i + 1) * n + k2, (i + 1) * n + k))
    for base in (0, steps * n):   # the ends: the U as three quads
        q = [base + k for k in range(n)]
        faces += [(q[0], q[1], q[6], q[7]), (q[1], q[2], q[5], q[6]), (q[2], q[3], q[4], q[5])]
    obj = _mesh(name, vs, faces, M.chute, smooth=False)
    for k, pts in enumerate(lips):
        G.sweep(f'{name}_lip{k}', pts, 0.019, material=M.chute, verts=8)
    # the run-out's end, rolled under so no edge faces a child
    G.sweep(f'{name}_end', [Vector(F.at(-w + 0.01, y1 + 0.004, z1 + 0.002)), Vector(F.at(w - 0.01, y1 + 0.004, z1 + 0.002))],
            0.016, material=M.chute, verts=8)
    return obj


# Headroom on the deck (M-community-07): the roof's eaves 1.03 m over it (a child stands under them), an open way in
# from the ladder between its handrails (no bar across the deck's back), and an arch over the chute's mouth rising
# 0.55 m over its floor (a child sits up under it).
SLIDE_EAVES = 2.46
SLIDE_ARCH = (0.27, 0.55)   # the arch over the chute's mouth: half its width, its rise over the floor


def slide():
    with G.new_parts():
        # the tower: four posts on base plates, a guard either side (a frame round a panel with a porthole), beams
        # round the top under a hipped roof
        for x in (-0.28, 0.28):
            for z in (-0.31, -0.69):
                F.cyl(f'post{x}{z}', 0.035, SLIDE_EAVES, (x, SLIDE_EAVES / 2, z), M.body, verts=12)
                F.box(f'base_plate{x}{z}', 0.12, 0.012, 0.12, (x, 0.006, z), M.steel, bevel=0.003, bevel_segments=1)
                F.sphere(f'post_cap{x}{z}', 0.04, (x, SLIDE_EAVES, z), M.body, segments=10, rings=5)
            loop = G.kit.fillet([(x, 1.62, -0.5), (x, 1.62, -0.655), (x, 1.86, -0.655), (x, 1.86, -0.345), (x, 1.62, -0.345), (x, 1.62, -0.5)],
                                0.04)
            F.sweep(f'guard{x}', loop[:-1], 0.02, M.body, verts=8, closed=True)
            port = [(0.075 * math.cos(math.tau * k / 16), 0.075 * math.sin(math.tau * k / 16)) for k in range(16)]
            F.extrude(f'panel{x}', G.rounded_rect(0.28, 0.21, 0.03, corner=2), 0.014, (x, 1.74, -0.5), M.roof, plane='side',
                      holes=[port])
            F.box(f'beam_side{x}', 0.05, 0.06, 0.44, (x, SLIDE_EAVES - 0.03, -0.5), M.body, bevel=0.008, bevel_segments=1)
        for z in (-0.31, -0.69):
            F.box(f'beam{z}', 0.6, 0.06, 0.05, (0, SLIDE_EAVES - 0.03, z), M.body, bevel=0.008, bevel_segments=1)
        # the deck: boards across, a frame under them
        for i in range(5):
            F.box(f'deck{i}', 0.62, 0.05, 0.082, (0, 1.405, -0.72 + 0.044 + i * 0.088), M.tread, bevel=0.006,
                  bevel_segments=1)
        F.box('deck_frame', 0.6, 0.06, 0.06, (0, 1.35, -0.31), M.body)
        F.box('deck_frame_back', 0.6, 0.06, 0.06, (0, 1.35, -0.69), M.body)
        # the roof: a hip with an eave lip, its slopes swept in, and a ball on top
        def square(y, r):
            return [F.at(r * a, y, -0.5 + r * b) for a, b in SQ]
        G.loft('roof', [square(SLIDE_EAVES - 0.01, 0.31), square(SLIDE_EAVES, 0.375), square(SLIDE_EAVES + 0.03, 0.375),
                        square(SLIDE_EAVES + 0.1, 0.26), square(SLIDE_EAVES + 0.2, 0.13), square(SLIDE_EAVES + 0.29, 0.02)],
               material=M.roof, smooth=False)
        F.sphere('roof_knob', 0.04, (0, SLIDE_EAVES + 0.3, -0.5), M.roof, segments=10, rings=6)
        # the ladder behind, leaning on the deck; its rails, one bent tube each, go on up as handrails either side of
        # the way in and over to the posts
        for x in (-0.25, 0.25):
            F.sweep(f'ladder{x}', G.kit.fillet([(x, 0.0, -0.94), (x, 1.42, -0.7), (x, 1.95, -0.69), (x * 1.12, 1.95, -0.69)],
                                                0.06), 0.025, M.body, verts=8)
            F.cyl(f'ladder_foot{x}', 0.026, 0.012, (x, 0.006, -0.938), M.steel, verts=10)
        for i in range(1, 6):
            t = i / 6
            F.box(f'rung{i}', 0.48, 0.03, 0.08, (0, 1.42 * t, -0.94 + 0.24 * t), M.tread, bevel=0.008, bevel_segments=1)
            for g in (-1, 0, 1):            # grip grooves along each tread
                F.box(f'groove{i}{g}', 0.46, 0.004, 0.006, (0, 1.42 * t + 0.015, -0.94 + 0.24 * t + g * 0.022), M.steel)
        # the chute, an arch over its mouth and the legs under its run-out
        _chute('chute')
        rx, ry = SLIDE_ARCH
        arch = [(rx * math.cos(math.pi * i / 14), SLIDE_TOP[0] + ry * math.sin(math.pi * i / 14), -0.27) for i in range(15)]
        F.sweep('arch', arch, 0.02, M.body, verts=8)
        for x in (-0.18, 0.18):
            F.sweep(f'leg{x}', [(x, 0.0, 0.84), (x, 0.3, 0.84)], 0.025, M.body, verts=8)
            F.box(f'foot{x}', 0.09, 0.012, 0.09, (x, 0.006, 0.84), M.steel, bevel=0.003, bevel_segments=1)
        F.sweep('leg_bar', [(-0.18, 0.2, 0.84), (0.18, 0.2, 0.84)], 0.016, M.body, verts=6)
    G.join_all('slide')
    G.settle(centre=False)
    return G.meshes()


# --- the climbing frame ----------------------------------------------------------------------------------------
# A wooden fort on four posts: a deck at 1.1 m, railings on three sides, a rope net up the front, a
# fireman's pole beside it and a pitched roof with a flag.
def _board(name, a, b, w, t, material):
    """A sawn board from game point `a` to `b`, `w` wide (across, as near level as it goes) and `t` thick."""
    return F.sweep(name, [a, b], None, material, profile=[(-w / 2, -t / 2), (w / 2, -t / 2), (w / 2, t / 2), (-w / 2, t / 2)],
                   up=(0, 0, 1))


def climbingFrame():
    deck = 1.1
    with G.new_parts():
        for x in (-0.85, 0.85):
            for z in (-0.85, 0.85):
                F.box(f'post{x}{z}', 0.1, 2.3, 0.1, (x, 1.15, z), M.timber, bevel=0.012, bevel_segments=1)
                F.box(f'shoe{x}{z}', 0.12, 0.08, 0.12, (x, 0.04, z), M.steel, bevel=0.004, bevel_segments=1)
        # the deck: boards on two joists, knee braces from the posts up under it
        for i in range(9):
            F.box(f'deck{i}', 1.8, 0.06, 0.192, (0, deck, -0.9 + 0.1 + i * 0.2), M.timber, bevel=0.006, bevel_segments=1)
        for z in (-0.82, 0.82):
            F.box(f'joist{z}', 1.6, 0.12, 0.06, (0, deck - 0.09, z), M.timber)
        for x in (-0.85, 0.85):
            for z, dz in ((-0.85, 1), (0.85, -1)):
                _board(f'brace{x}{z}', (x, deck - 0.4, z + dz * 0.05), (x, deck - 0.12, z + dz * 0.33), 0.07, 0.04, M.timber)
        # railings: a top and bottom rail and flat balusters with rounded tops on the back and both sides
        for side in ('back', 'left', 'right'):
            for k, y in enumerate((deck + 0.12, deck + 0.5)):
                if side == 'back':
                    F.box(f'{side}_rail{k}', 1.6, 0.05, 0.05, (0, y, -0.85), M.timber, bevel=0.01, bevel_segments=1)
                else:
                    x = -0.85 if side == 'left' else 0.85
                    F.box(f'{side}_rail{k}', 0.05, 0.05, 1.6, (x, y, 0), M.timber, bevel=0.01, bevel_segments=1)
            for i in range(7):
                c = -0.6 + i * 0.2
                at = (c, deck + 0.31, -0.85) if side == 'back' else (-0.85 if side == 'left' else 0.85, deck + 0.31, c)
                w, d = (0.08, 0.02) if side == 'back' else (0.02, 0.08)
                F.box(f'{side}_baluster{i}', w, 0.34, d, at, M.timber, bevel=0.006, bevel_segments=1)
        # the roof: rafters, purlins, then five courses of shingles a slope lapping down from a ridge board, gables
        for z in (-0.85, 0.85):
            F.box(f'beam{z}', 1.8, 0.1, 0.08, (0, 2.25, z), M.timber, bevel=0.008, bevel_segments=1)
            for s in (-1, 1):
                _board(f'rafter{z}{s}', (s * 0.85, 2.3, z), (0, 2.71, z), 0.08, 0.05, M.timber)
            F.extrude(f'gable{z}', [(-0.82, 2.3), (0.82, 2.3), (0, 2.68)], 0.02, (0, 0, z - 0.03 * (1 if z > 0 else -1)),
                      M.timber)
        slope = 30
        for s in (-1, 1):
            for k in range(5):
                t = 0.06 + k * 0.2
                x, y = s * (0.02 + t * G.cosd(slope)), 2.74 - t * G.sind(slope) + 0.02
                F.box(f'shingle{s}{k}', 0.24, 0.03, 1.96, (x, y, 0), M.shingle, rot=(0, -s * (slope + 3), 0), bevel=0.006,
                      bevel_segments=1)
        F.sweep('ridge', [(0, 2.77, -0.97), (0, 2.77, 0.97)], None, M.shingle,
                profile=G.rounded_rect(0.06, 0.12, 0.025, corner=2), up=(0, 0, 1))
        # the flag on the ridge, at the back: a pole with a ball, the flag rippling out from it
        F.cyl('flagpole', 0.018, 0.82, (0, 3.04, -0.85), M.rail, verts=8)
        F.sphere('flag_ball', 0.03, (0, 3.45 - 0.03, -0.85), M.flag, segments=8, rings=5)
        F.sweep('flag', [(0.01, 3.33, -0.85), (0.1, 3.33, -0.82), (0.2, 3.32, -0.87), (0.3, 3.31, -0.84)], None, M.flag,
                profile=[(-0.1, -0.004), (0.1, -0.004), (0.1, 0.004), (-0.1, 0.004)], up=(0, 0, 1), resolution=3,
                scale=[1.0, 0.98, 0.95, 0.9])
        # the rope net up the front, from the deck's edge down to the ground a little out: a rope through each row of
        # knots, the knots where they cross, a bar along the top and stakes at its foot
        top, bottom = (deck - 0.02, 0.86), (0.04, 0.96)
        xs = [-0.75 + i * 0.18 for i in range(6)]
        rows = []
        for j in range(6):
            t = j / 5
            rows.append((top[0] + (bottom[0] - top[0]) * t, top[1] + (bottom[1] - top[1]) * t))
        for i, x in enumerate(xs):
            F.sweep(f'net_down{i}', [(x, deck + 0.02, 0.86), (x, *top), (x, *bottom), (x, 0.0, 0.97)], 0.014, M.rope, verts=6)
        for j, (y, z) in enumerate(rows):
            sag = [(xs[0] - 0.04, y, z)] + [p for a, b in zip(xs, xs[1:]) for p in ((a, y, z), ((a + b) / 2, y - 0.01, z))] + \
                [(xs[-1], y, z), (xs[-1] + 0.04, y, z)]
            F.sweep(f'net_across{j}', sag, 0.014, M.rope, verts=6)
            for i, x in enumerate(xs):
                F.sphere(f'knot{i}{j}', 0.022, (x, y, z), M.rope, segments=6, rings=4)
        F.sweep('net_bar', [(xs[0] - 0.06, deck + 0.02, 0.88), (xs[-1] + 0.06, deck + 0.02, 0.88)], 0.03, M.timber, verts=8)
        for x in (xs[0], xs[-1]):
            F.cyl(f'peg{x}', 0.03, 0.06, (x, 0.03, 0.96), M.timber, verts=8)
        # the fireman's pole, beside the net, hung from a bend off the front beam, a collar at its foot
        F.sweep('pole', G.kit.fillet([(0.55, 0.0, 0.95), (0.55, 2.04, 0.95), (0.55, 2.2, 0.85)], 0.06), 0.025, M.rail, verts=10)
        F.box('pole_bracket', 0.08, 0.1, 0.03, (0.55, 2.2, 0.83), M.steel, bevel=0.004, bevel_segments=1)
        F.cyl('pole_foot', 0.06, 0.02, (0.55, 0.01, 0.95), M.steel, verts=10)
    G.join_all('climbingFrame')
    G.settle(centre=False)
    return G.meshes()


# --- the seesaw ----------------------------------------------------------------------------------------------
# A 3 m plank on an A-frame stand. The content's two slots sit at x = ±0.75 m (3 m wide, 2 slots), at the
# seat height 0.56 m: the seats are there, level at rest. The plank, its seats and handles are one part
# turning about the axle (0.46 m up), so the riders go up and down with it.
AXLE = 0.46


def seesaw():
    with G.new_parts() as still:
        for z in (-0.17, 0.17):
            # each side of the stand an A of bent tube, its feet on a base plate, a boss for the axle at its apex
            F.sweep(f'leg{z}', G.kit.fillet([(-0.22, 0.02, z), (0, AXLE, z), (0.22, 0.02, z)], 0.05), 0.028, M.body, verts=10)
            F.box(f'foot{z}', 0.56, 0.04, 0.08, (0, 0.02, z), M.body, bevel=0.01, bevel_segments=1)
            for x in (-0.24, 0.24):
                F.cyl(f'bolt{z}{x}', 0.012, 0.012, (x, 0.044, z), M.steel, verts=6)
            F.cyl(f'boss{z}', 0.05, 0.05, (0, AXLE, z), M.steel, rot=(90, 0, 0), verts=16, bevel=0.006, bevel_segments=1)
            F.sweep(f'crossbar{z}', [(-0.12, 0.2, z), (0.12, 0.2, z)], 0.016, M.body, verts=8)
        G.rod('axle', F.at(0, AXLE, -0.21), F.at(0, AXLE, 0.21), 0.022, material=M.steel, verts=12)
        for z in (-0.202, 0.202):
            F.cyl(f'axle_nut{z}', 0.03, 0.012, (0, AXLE, z), M.steel, rot=(90, 0, 0), verts=6)
        for x in (-1.1, 1.1):   # half-buried tyres the low end comes down onto
            G.revolve(f'bumper{x}', [(0, 0.0), (0.12, 0.0), (0.13, 0.04), (0.13, 0.13), (0.12, 0.165), (0.09, 0.175),
                                     (0.06, 0.165), (0.055, 0.15), (0, 0.15)], at=F.at(x, 0, 0), material=M.rubber,
                      segments=18)
    G.join(still, 'stand', origin=(0, 0, 0))
    with G.new_parts() as parts:
        # the plank: rounded at its ends, a steel channel along its middle and straps over its ends
        F.extrude('plank', G.rounded_rect(2.5, 0.24, 0.1, corner=4), 0.06, (0, 0.48, 0), M.plank, plane='top', bevel=0.012,
                  bevel_segments=1)
        F.box('channel', 0.8, 0.03, 0.12, (0, 0.44, 0), M.steel, bevel=0.004, bevel_segments=1)
        F.cyl('sleeve', 0.045, 0.14, (0, AXLE, 0), M.steel, rot=(90, 0, 0), verts=16)
        F.box('bracket', 0.16, 0.04, 0.12, (0, 0.47, 0), M.steel, bevel=0.005, bevel_segments=1)
        for s in (-1, 1):
            # a moulded seat, dished, a lip rising at its outer end; its top where the rider sits (0.56 m)
            F.soft(f'seat{s}', 0.34, 0.04, 0.28, (s * 0.75, 0.532, 0), M.seat, round=0.018, puff=0.008, flat=2)
            F.sweep(f'seat_lip{s}', [(s * 0.92, 0.535, -0.12), (s * 0.94, 0.6, -0.1), (s * 0.94, 0.6, 0.1),
                                     (s * 0.92, 0.535, 0.12)], 0.02, M.seat, verts=8, resolution=2)
            F.soft(f'pad{s}', 0.14, 0.03, 0.18, (s * 1.12, 0.435, 0), M.rubber, round=0.012)
            # the handle: one bent tube rising either side of the plank and across, its grips where the hands go
            F.sweep(f'handle{s}', G.kit.fillet([(s * 0.47, 0.49, -0.135), (s * 0.47, 0.82, -0.135), (s * 0.47, 0.82, 0.135),
                                                (s * 0.47, 0.49, 0.135)], 0.035), 0.02, M.rail, verts=10)
            for z in (-1, 1):
                F.box(f'handle_plate{s}{z}', 0.08, 0.06, 0.008, (s * 0.47, 0.49, z * 0.128), M.steel, bevel=0.002,
                      bevel_segments=1)
                F.sweep(f'grip{s}{z}', [(s * 0.47, 0.82, z * 0.035), (s * 0.47, 0.82, z * 0.1)], 0.026, M.rubber, verts=10)
    plank = G.hinged(G.join(parts, 'plank'), F.at(0, AXLE, 0))
    G.settle(centre=False)
    G.rock(plank, 'seesaw', 12, seconds=1.4, axis='Y')
    return G.meshes()


# --- the spring rider --------------------------------------------------------------------------------------------
# A horse on a big coil spring. The saddle's top is the content's seat (0.62 m, 4 cm back: a rider's seat for a
# toddler to a six-year-old, EN 1176-6). The horse (all above the spring) rocks about the spring's top. Its parts are
# drawn for a spring top at HORSE_DRAWN and moved down onto SPRING_TOP (it was 16 cm higher, the saddle at 0.78).
SPRING_TOP = 0.26
HORSE_DRAWN = 0.42


def _body_rings(rows, n=12):
    """Loft rings along the horse's length: (depth, half width, half height, centre height) each, an ellipse."""
    return [[F.at(hw * math.cos(math.tau * k / n), y + hh * math.sin(math.tau * k / n), z) for k in range(n)]
            for z, hw, hh, y in rows]


def springRider():
    with G.new_parts() as still:
        # a round steel base plate bolted down, a plate for the coil on it, the coil swept in one piece
        F.cyl('base', 0.2, 0.03, (0, 0.015, 0), M.steel, verts=24, bevel=0.008, bevel_segments=1)
        for k in range(6):
            a = math.tau * k / 6
            F.cyl(f'base_bolt{k}', 0.012, 0.012, (0.17 * math.cos(a), 0.034, 0.17 * math.sin(a)), M.rubber, verts=6)
        F.cyl('base_plate', 0.13, 0.02, (0, 0.04, 0), M.steel, verts=20)
        turns, r = 3.5, 0.1
        steps = int(turns * 12)
        coil = [(r * math.cos(math.tau * i / 12), 0.06 + (SPRING_TOP - 0.08) * i / steps, r * math.sin(math.tau * i / 12))
                for i in range(steps + 1)]
        F.sweep('coil', coil, 0.018, M.spring, verts=6)
    G.join(still, 'spring', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.cyl('top_plate', 0.12, 0.02, (0, HORSE_DRAWN, 0), M.steel, verts=20)
        F.cyl('mount', 0.05, 0.1, (0, HORSE_DRAWN + 0.06, 0), M.steel, verts=12)
        # the body: a barrel lofted from the rump to the chest
        G.loft('barrel', _body_rings([(-0.33, 0.03, 0.03, 0.62), (-0.3, 0.1, 0.11, 0.615), (-0.22, 0.13, 0.15, 0.6),
                                      (-0.05, 0.125, 0.15, 0.59), (0.12, 0.125, 0.15, 0.6), (0.24, 0.11, 0.13, 0.63),
                                      (0.3, 0.06, 0.07, 0.65)], 14), material=M.horse)
        # the neck and head: one sweep up from the chest, arching over into the head, tapering to the muzzle
        F.sweep('neck', [(0, 0.66, 0.18), (0, 0.8, 0.26), (0, 0.95, 0.29), (0, 1.0, 0.32), (0, 0.97, 0.39), (0, 0.92, 0.45)],
                None, M.horse, profile=G.rounded_rect(0.16, 0.13, 0.06, corner=3), up=(0, -1, 0), resolution=3,
                scale=[1.1, 0.95, 0.95, 1.05, 0.95, 0.75], angle=70)
        F.sphere('muzzle', 0.07, (0, 0.91, 0.45), M.mane, scale=(1.0, 0.85, 1.0), segments=12, rings=7)
        for s in (-1, 1):
            F.sweep(f'ear{s}', [(s * 0.045, 1.03, 0.3), (s * 0.055, 1.08, 0.29), (s * 0.05, 1.1, 0.28)], 0.022, M.horse,
                    verts=6, resolution=2, scale=[1.0, 0.7, 0.15])
            F.sphere(f'eye{s}', 0.016, (s * 0.075, 1.0, 0.37), M.eye, segments=6, rings=4)
            F.cyl(f'nostril{s}', 0.01, 0.01, (s * 0.03, 0.9, 0.512), M.eye, rot=(90, 0, 0), verts=8)
            # legs: bent at the knee, tucked under as a rocking horse's are, hooves at their ends
            for z, dz in ((0.18, 1), (-0.2, -1)):
                F.sweep(f'leg{s}{z}', [(s * 0.08, 0.6, z), (s * 0.095, 0.5, z + dz * 0.04), (s * 0.1, 0.44, z + dz * 0.1)],
                        0.04, M.horse, verts=10, resolution=2, scale=[1.1, 0.85, 0.75])
                F.cyl(f'hoof{s}{z}', 0.032, 0.035, (s * 0.1, 0.43, z + dz * 0.115), M.eye, verts=10)
        # the mane in locks down the neck, a forelock, the tail swept out and down
        for i in range(6):
            t = i / 5
            y, z = 1.02 - 0.3 * t, 0.29 - 0.12 * t
            F.sweep(f'mane{i}', [(0, y, z), (0, y + 0.03, z - 0.05), (0, y - 0.01, z - 0.09)], None, M.mane,
                    profile=[(-0.022, -0.012), (0.022, -0.012), (0.022, 0.012), (-0.022, 0.012)], up=(0, 0, 1),
                    resolution=2, scale=[1.0, 0.8, 0.3])
        F.sphere('forelock', 0.03, (0, 1.07, 0.34), M.mane, scale=(0.9, 0.7, 1.3), segments=8, rings=5)
        F.sweep('tail', [(0, 0.66, -0.31), (0, 0.655, -0.365), (0, 0.58, -0.405), (0, 0.5, -0.405)], 0.035, M.mane, verts=10,
                resolution=2, scale=[0.8, 1.1, 1.0, 0.6])
        # the saddle: a padded seat (its top where the rider sits), a cantle behind, flaps down the sides
        F.soft('saddle', 0.27, 0.04, 0.26, (0, 0.75, -0.04), M.saddle, round=0.018, puff=0.01, flat=2, round_segments=1)
        F.sweep('cantle', [(-0.1, 0.77, -0.17), (0, 0.8, -0.18), (0.1, 0.77, -0.17)], 0.018, M.saddle, verts=8,
                resolution=2)
        for s in (-1, 1):
            F.box(f'flap{s}', 0.012, 0.12, 0.2, (s * 0.132, 0.69, -0.04), M.saddle, rot=(0, 0, s * 8), bevel=0.004,
                  bevel_segments=1)
        # the handles: a bar through the head, a rubber grip at each end; the foot pegs through the body
        F.sweep('handles', [(-0.11, 0.93, 0.24), (0.11, 0.93, 0.24)], 0.014, M.steel, verts=8)
        F.sweep('pegs', [(-0.15, 0.5, 0.06), (0.15, 0.5, 0.06)], 0.014, M.steel, verts=8)
        for s in (-1, 1):
            F.sweep(f'handle_grip{s}', [(s * 0.1, 0.93, 0.24), (s * 0.16, 0.93, 0.24)], 0.022, M.grip, verts=10,
                    scale=[1.0, 1.1])
            F.sphere(f'handle_end{s}', 0.024, (s * 0.16, 0.93, 0.24), M.grip, segments=8, rings=5)
            F.sweep(f'peg{s}', [(s * 0.13, 0.5, 0.06), (s * 0.18, 0.5, 0.06)], 0.022, M.grip, verts=10)
            F.cyl(f'peg_end{s}', 0.03, 0.02, (s * 0.19, 0.5, 0.06), M.grip, rot=(0, 90, 0), verts=10)
    horse = G.join(parts, 'horse')
    horse.data.transform(Matrix.Translation((0, 0, SPRING_TOP - HORSE_DRAWN)))
    horse = G.hinged(horse, F.at(0, SPRING_TOP, 0))
    G.settle(centre=False)
    G.rock(horse, 'rock', 10, seconds=1.0, axis='X')
    return G.meshes()


# --- the merry-go-round ----------------------------------------------------------------------------------------
# A 1.9 m platform, its top 0.26 m up (the content's seat: the riders sit on it), four bent rails to hold on to.
# The platform turns once in 3.5 s about the hub; the riders ride round with it.
def merryGoRound():
    with G.new_parts() as still:
        F.cyl('ground_plate', 0.3, 0.03, (0, 0.015, 0), M.steel, verts=24, bevel=0.008, bevel_segments=1)
        for k in range(8):
            a = math.tau * k / 8
            F.cyl(f'plate_bolt{k}', 0.012, 0.01, (0.26 * math.cos(a), 0.034, 0.26 * math.sin(a)), M.rubber, verts=6)
        G.revolve('hub', [(0, 0.03), (0.17, 0.03), (0.17, 0.05), (0.15, 0.06), (0.13, 0.13), (0.11, 0.15), (0, 0.15)],
                  at=F.at(0, 0, 0), material=M.steel, segments=20)
    G.join(still, 'hub', origin=(0, 0, 0))
    with G.new_parts() as parts:
        # the deck: a disc on a dished steel skirt, a rail round its edge, the stripes, anti-slip ribs between them
        G.revolve('platform', [(0, 0.18), (0.94, 0.18), (0.95, 0.19), (0.95, 0.25), (0.94, 0.26), (0, 0.26)],
                  at=F.at(0, 0, 0), material=M.platform, segments=48, smooth=False)
        G.torus('edge', 0.95, 0.03, at=F.at(0, 0.22, 0), material=M.rail, major_segments=48, minor_segments=6)
        G.revolve('skirt', [(0.12, 0.13), (0.3, 0.14), (0.62, 0.17), (0.8, 0.18), (0, 0.18)], at=F.at(0, 0, 0),
                  material=M.steel, segments=24)
        for a in (45, -45):
            F.box(f'stripe{a}', 0.22, 0.008, 1.72, (0, 0.263, 0), M.deck, rot=(0, 0, a))
        for k in range(4):
            a = 90 * k
            for j in (-1, 1):
                b = math.radians(a + j * 22)
                F.box(f'rib{k}{j}', 0.03, 0.006, 0.5, (0.6 * math.sin(b), 0.262, 0.6 * math.cos(b)), M.deck,
                      rot=(0, 0, -(a + j * 22)))
        F.cyl('centre_disc', 0.2, 0.012, (0, 0.266, 0), M.deck, verts=24)
        # the column: a turned post, a collar where the rails meet it and a knob on top
        G.revolve('column', [(0, 0.26), (0.08, 0.26), (0.08, 0.28), (0.055, 0.3), (0.05, 0.84), (0.07, 0.86), (0.07, 0.92),
                             (0.05, 0.94), (0, 0.94)], at=F.at(0, 0, 0), material=M.rail, segments=14)
        G.revolve('knob', [(0, 0.93), (0.05, 0.93), (0.08, 0.96), (0.075, 1.0), (0.04, 1.035), (0, 1.04)],
                  at=F.at(0, 0, 0), material=M.deck, segments=14)
        for k in range(4):
            t = math.radians(45 + 90 * k)
            c, s = math.sin(t), math.cos(t)
            pts = _bezier((0.05 * c, 0.9, 0.05 * s), (0.78 * c, 0.9, 0.78 * s), (0.78 * c, 0.25, 0.78 * s), 14)
            F.sweep(f'rail{k}', [tuple(p) for p in pts], 0.022, M.rail, verts=10)
            F.cyl(f'rail_foot{k}', 0.045, 0.014, (0.78 * c, 0.266, 0.78 * s), M.steel, verts=10, bevel=0.004,
                  bevel_segments=1)
            # a rubber grip where the riders' hands go, down the rail beside them
            g0, g1 = _bezier((0.05 * c, 0.9, 0.05 * s), (0.78 * c, 0.9, 0.78 * s), (0.78 * c, 0.25, 0.78 * s), 20)[10:16:5]
            F.sweep(f'rail_grip{k}', [tuple(g0), tuple(g1)], 0.027, M.rubber, verts=10)
    platform = G.hinged(G.join(parts, 'platform'), (0, 0, 0))
    G.settle(centre=False)
    G.spin(platform, 'spin', turns=1, seconds=3.5, axis='Z')
    return G.meshes()


# --- the basketball hoop ----------------------------------------------------------------------------------------
# A hoop on a post with a ballast base at the back of its tile; the rim at 3.05 m (regulation, park and driveway
# hoops alike) reaches forward from a 1.2 x 0.8 m board (a park one's; they were 2.94 m and 0.95 x 0.62). The net
# is its own part, hung from the rim: `swish` sways and stretches it once a ball goes through.
RIM = (3.05, 0.27)   # (height, depth) of the rim's middle
BOARD_W, BOARD_H, BOARD_Y = 1.2, 0.8, 3.3   # the backboard: its width, height and middle's height


def _net(name):
    """A diamond-mesh net: strands zigzagging from the rim (r 0.21) to a narrower bottom ring (r 0.13), each strand
    one cord, a hem round the bottom."""
    y0, y1, r0, r1, n = RIM[0] - 0.01, RIM[0] - 0.38, 0.205, 0.13, 12
    z = RIM[1]
    with G.new_parts() as parts:
        rows = 4
        for i in range(n):
            for d in (-1, 1):
                pts = []
                for j in range(rows + 1):
                    t = j / rows
                    a = math.tau * (i + d * (0.5 if j % 2 else 0)) / n
                    r = r0 + (r1 - r0) * t
                    pts.append((r * math.cos(a), y0 + (y1 - y0) * t, z + r * math.sin(a)))
                F.sweep(f'{name}{i}{d}', pts, 0.006, M.net, verts=4)
        G.torus(f'{name}_hem', r1, 0.008, at=F.at(0, y1, z), material=M.net, major_segments=16, minor_segments=4)
    return G.join(parts, name)


def basketballHoop():
    with G.new_parts() as still:
        # the base: a moulded ballast tank, its fill cap, two wheels at its front edge for rolling it
        bx, by, _ = F.at(0, 0, -0.17)
        G.loft('ballast', [G.ring(0.31, 0.3, 0.0, 24, square=6, at=(bx, by)), G.ring(0.31, 0.3, 0.18, 24, square=6, at=(bx, by)),
                           G.ring(0.28, 0.27, 0.22, 24, square=6, at=(bx, by)), G.ring(0.2, 0.19, 0.235, 24, square=5, at=(bx, by))],
               material=M.post, angle=50)
        F.cyl('ballast_cap', 0.05, 0.02, (0.16, 0.23, -0.05), M.steel, verts=12, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            F.cyl(f'wheel{s}', 0.05, 0.035, (s * 0.24, 0.05, 0.14), M.rubber, rot=(0, 90, 0), verts=14, bevel=0.008,
                  bevel_segments=1)
        # the pole in two telescoping sections, a clamp collar between them; a cap on top
        F.cyl('pole', 0.05, 1.85, (0, 0.21 + 0.925, -0.3), M.post, verts=14)
        F.cyl('pole_upper', 0.044, 1.36, (0, 2.0 + 0.68, -0.3), M.post, verts=14)
        G.revolve('pole_collar', [(0, -0.04), (0.058, -0.04), (0.062, -0.03), (0.062, 0.03), (0.055, 0.04), (0, 0.04)],
                  at=F.at(0, 2.02, -0.3), material=M.steel, segments=14)
        F.sweep('collar_knob', [(0, 2.02, -0.24), (0, 2.02, -0.2)], 0.012, M.rubber, verts=6)
        F.cyl('pole_cap', 0.055, 0.03, (0, 3.36, -0.3), M.steel, verts=14, bevel=0.006, bevel_segments=1)
        # two bent arms from the pole to the board's back, a strut between them
        for y, to in ((3.3, 3.45), (3.0, 3.08)):
            F.sweep(f'arm{y}', G.kit.fillet([(0, y, -0.3), (0, y, -0.2), (0, to, 0.0)], 0.04), 0.025, M.post, verts=8)
        F.sweep('arm_strut', [(0, 3.06, -0.18), (0, 3.38, -0.12)], 0.016, M.post, verts=6)
        # the backboard: a white board in a steel frame, its border and the shooter's square, a pad along its foot
        bw, bh, by = BOARD_W, BOARD_H, BOARD_Y
        F.box('board', bw - 0.02, bh - 0.02, 0.025, (0, by, 0.04), M.board, bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'board_frame', [(bw / 2, by - bh / 2), (bw / 2, by + bh / 2), (-bw / 2, by + bh / 2), (-bw / 2, by - bh / 2)],
                       [(0.0, -0.02), (0.0, 0.016), (0.022, 0.016), (0.022, -0.02)], M.post, at=0.04)
        for y in (by + bh / 2 - 0.045, by - bh / 2 + 0.045):
            F.box(f'border{y}', bw - 0.07, 0.03, 0.028, (0, y, 0.04), M.board_mark)
        for x in (-bw / 2 + 0.045, bw / 2 - 0.045):
            F.box(f'border{x}', 0.03, bh - 0.06, 0.028, (x, by, 0.04), M.board_mark)
        tw, th = 0.59, 0.45       # the shooter's square, 0.59 x 0.45 m as on a regulation board, its foot level with the rim
        ty = RIM[0] + th / 2
        F.extrude('target', G.rounded_rect(tw, th, 0.004, corner=1), 0.028, (0, ty, 0.045), M.board_mark,
                  holes=[G.rounded_rect(tw - 0.05, th - 0.05, 0.003, corner=1)])
        F.soft('pad', bw, 0.04, 0.06, (0, by - bh / 2 - 0.01, 0.04), M.rubber, round=0.015)
        # the rim on a flanged bracket with its spring housing, little hooks for the net under it
        F.box('rim_bracket', 0.16, 0.12, 0.05, (0, RIM[0] - 0.04, 0.075), M.rim, bevel=0.006, bevel_segments=1)
        F.box('rim_spring', 0.06, 0.07, 0.04, (0, RIM[0] - 0.05, 0.1), M.steel, bevel=0.006, bevel_segments=1)
        F.extrude('rim_flange', [(0.06, 0.0), (-0.06, 0.0), (-0.03, 0.12), (0.03, 0.12)], 0.006, (0, RIM[0] - 0.008, 0.06),
                  M.rim, plane='top')
        G.torus('rim', 0.22, 0.012, at=F.at(*(0, RIM[0], RIM[1])), material=M.rim, major_segments=28, minor_segments=6)
        for k in range(12):
            a = math.tau * k / 12
            x, z = 0.215 * math.cos(a), RIM[1] + 0.215 * math.sin(a)
            F.sweep(f'net_hook{k}', [(x, RIM[0] - 0.005, z), (x * 0.97, RIM[0] - 0.022, z - 0.0), (x * 0.95, RIM[0] - 0.014, z)],
                    0.0025, M.rim, verts=3)
    G.join(still, 'hoop', origin=(0, 0, 0))
    # the ball, at rest on the ground in front: its seams. A part of its own, gone while someone plays (it is in
    # their hands: actions.ts `PROPS.basketball`).
    with G.new_parts() as ball_parts:
        F.sphere('ball', 0.12, (0.3, 0.12, 0.32), M.ball, segments=16, rings=10)
        for i, rot in enumerate(((90, 0, 30), (0, 0, 0), (90, 0, 120))):
            G.torus(f'seam{i}', 0.1205, 0.0035, at=F.at(0.3, 0.12, 0.32), material=M.rubber, rot=rot,
                    major_segments=20, minor_segments=3)
    ball = G.hinged(G.join(ball_parts, 'ball'), F.at(0.3, 0.12, 0.32))
    net = G.hinged(_net('net'), F.at(0, RIM[0], RIM[1]))
    G.settle(centre=False)
    G.rock(net, 'swish', 9, seconds=0.6, axis='X')
    G.squash(net, 'swish', (0.92, 0.92, 1.18), seconds=0.6, back=True)
    G.squash(ball, 'ball_taken', 0.001, seconds=0.2)
    return G.meshes()


# --- the ping-pong table ---------------------------------------------------------------------------------------
# A weatherproof table at the ITTF's size (2.74 x 1.525 m, its top 0.76 m up) on 3 x 2 tiles, the players at its two
# ends (content `places`): a green top on two concrete legs, a steel net 15.25 cm high reaching 15.25 cm past each
# side, and a ball. Still. The bats are in the players' hands (held.py `tool.bat`; review I-community-06: they lay on
# the top while it was played). (It was 1.9 x 0.94 m, its top at 0.78.)
PONG = (2.74, 0.76, 1.525)   # the top's length, height and width


def pingPongTable():
    length, top, width = PONG
    hl, hw = length / 2, width / 2
    with G.new_parts():
        # the top: a slab with a band round its edge, a steel frame under it
        F.box('top', length, 0.03, width, (0, top - 0.015, 0), M.table_top, bevel=0.006, bevel_segments=1)
        G.kit.moulding(F, 'top_band', [(hl, -hw), (hl, hw), (-hl, hw), (-hl, -hw)],
                       [(0.0, -0.035), (0.0, -0.002), (0.006, -0.002), (0.006, -0.035)], M.steel, at=top, plane='top')
        for z in (-hw + 0.1, hw - 0.1):
            F.box(f'apron{z}', length - 0.3, 0.06, 0.04, (0, top - 0.06, z), M.steel, bevel=0.004, bevel_segments=1)
        # two concrete legs: a tapered pier with an arch through it, on a broad foot
        for x in (-0.85, 0.85):
            leg = [(-0.5, 0.04), (0.5, 0.04), (0.42, top - 0.03), (-0.42, top - 0.03)]
            arch = [(0.18 * math.cos(math.pi * k / 10), 0.04 + 0.32 * math.sin(math.pi * k / 10)) for k in range(11)]
            hole = [(0.18, 0.12)] + [(px, py + 0.08) for px, py in arch[1:-1]] + [(-0.18, 0.12)]
            F.extrude(f'leg{x}', leg, 0.12, (x, 0, 0), M.concrete, plane='side', holes=[hole], bevel=0.01, bevel_segments=1)
            F.box(f'foot{x}', 0.22, 0.04, 1.12, (x, 0.02, 0), M.concrete, bevel=0.01, bevel_segments=1)
            F.box(f'cap{x}', 0.16, 0.02, 0.88, (x, top - 0.04, 0), M.steel, bevel=0.004, bevel_segments=1)
        y = top + 0.001
        for w, d, x, z in ((length, 0.02, 0, hw - 0.01), (length, 0.02, 0, -hw + 0.01), (0.02, width, hl - 0.01, 0),
                           (0.02, width, -hl + 0.01, 0), (length, 0.003, 0, 0)):
            F.box(f'line{x}{z}', w, 0.003, d, (x, y, z), M.lines)
        # the steel net: a perforated plate (rows of holes as a darker band) between two posts clamped to the top
        net = width + 2 * 0.1525
        F.box('net', 0.012, 0.13, net - 0.04, (0, top + 0.07, 0), M.steel)
        for k in range(3):
            F.box(f'net_slot{k}', 0.014, 0.012, net - 0.08, (0, top + 0.03 + k * 0.035, 0), M.rubber)
        F.box('net_band', 0.026, 0.02, net, (0, top + 0.1425, 0), M.lines, bevel=0.004, bevel_segments=1)
        for z in (-net / 2 + 0.015, net / 2 - 0.015):
            F.box(f'net_post{z}', 0.04, 0.17, 0.03, (0, top + 0.08, z), M.steel, bevel=0.005, bevel_segments=1)
            F.box(f'net_clamp{z}', 0.06, 0.05, 0.05, (0, top - 0.015, math.copysign(hw - 0.005, z)), M.steel, bevel=0.005,
                  bevel_segments=1)
            F.box(f'net_arm{z}', 0.03, 0.02, abs(z) - hw + 0.03, (0, top + 0.005, math.copysign((abs(z) + hw) / 2, z)),
                  M.steel)
        # the ball
        F.sphere('ball', 0.02, (0.5, top + 0.02, -0.15), M.ping, segments=10, rings=6)
    G.join_all('pingPongTable')
    G.settle(centre=False)
    return G.meshes()


# --- the park chess table -----------------------------------------------------------------------------------------
# A stone table with the board inlaid, its top 0.7 m square, and a stone stool either side of it, left and right,
# where the content's two places seat the players facing each other across the board (x = ±0.6 m, 0.47 m up, on
# 2 x 1 tiles: their knees clear of each other under the top; they were ±0.42 on one tile). White plays from the
# left. A white pawn slides one square on (`move`) as they play.
BOARD = (0.762, 0.0)   # (height, depth) of the board's middle
SQUARE = 0.05
STOOL = 0.6            # the stools' middles, either side (the content's `places`)
CHESS_TOP = 0.7        # the top, square


def _square(i, j):
    """The game (x, depth) of square (file i, rank j), rank 0 nearest white on the left, file 0 on white's
    left (to the front)."""
    return -3.5 * SQUARE + j * SQUARE, BOARD[1] + 3.5 * SQUARE - i * SQUARE


def _stools(segments):
    """The two stools: a turned stone mushroom, a moulded foot, a waisted stem, a seat with a rounded edge."""
    for s in (-1, 1):
        G.revolve(f'stool{s}', [(0, 0), (0.11, 0), (0.11, 0.025), (0.095, 0.035), (0.075, 0.06), (0.062, 0.12),
                                (0.058, 0.3), (0.066, 0.37), (0.09, 0.41), (0.12, 0.43), (0.125, 0.45), (0.12, 0.465),
                                (0.1, 0.47), (0, 0.47)], at=F.at(s * STOOL, 0, 0), material=M.stool, segments=segments)


def _pedestal(segments):
    G.revolve('pedestal', [(0, 0), (0.17, 0), (0.17, 0.03), (0.15, 0.045), (0.11, 0.08), (0.09, 0.14), (0.08, 0.55),
                           (0.09, 0.6), (0.12, 0.66), (0.13, 0.68), (0.12, 0.7), (0, 0.7)], at=F.at(0, 0, BOARD[1]),
              material=M.stool, segments=segments)


def _piece(name, kind, i, j, material, low=False):
    """A chess piece standing on square (i, j): turned, a knight's head swept, the king's cross, the queen's crown."""
    x, z = _square(i, j)
    y = BOARD[0] + 0.002
    at = F.at(x, y, z)
    seg = 8 if low else 10
    foot = [(0, 0), (0.017, 0), (0.017, 0.005), (0.014, 0.008)]
    if kind == 'pawn':
        G.revolve(name, foot + [(0.008, 0.018), (0.006, 0.026), (0.011, 0.029), (0.006, 0.032), (0, 0.033)], at=at,
                  material=material, segments=seg)
        G.sphere(f'{name}_head', 0.011, at=(at[0], at[1], at[2] + 0.041), material=material, segments=8, rings=5)
    elif kind == 'rook':
        G.revolve(name, foot + [(0.011, 0.014), (0.01, 0.038), (0.016, 0.042), (0.016, 0.056), (0.011, 0.056),
                                (0.011, 0.05), (0, 0.05)], at=at, material=material, segments=seg)
    elif kind == 'knight':
        G.revolve(name, foot + [(0.012, 0.014), (0, 0.016)], at=at, material=material, segments=seg)
        facing = -1 if name.startswith('black') else 1
        F.sweep(f'{name}_head', [(x, y + 0.012, z), (x + facing * 0.004, y + 0.035, z), (x + facing * 0.012, y + 0.05, z),
                                 (x + facing * 0.024, y + 0.044, z)], None, material,
                profile=G.rounded_rect(0.02, 0.014, 0.005, corner=1), up=(0, 0, 1), resolution=2,
                scale=[1.3, 0.9, 0.9, 0.6])
    elif kind == 'bishop':
        G.revolve(name, foot + [(0.008, 0.02), (0.006, 0.034), (0.011, 0.037), (0.007, 0.04), (0.01, 0.048),
                                (0.008, 0.056), (0.003, 0.062), (0, 0.064)], at=at, material=material, segments=seg)
    else:   # king or queen: taller, with a cap
        h = 0.07 if kind == 'king' else 0.064
        G.revolve(name, [(0, 0), (0.019, 0), (0.019, 0.007), (0.012, 0.014), (0.007, h - 0.02), (0.012, h - 0.016),
                         (0.008, h - 0.013), (0.014, h - 0.004), (0.009, h), (0, h)], at=at, material=material, segments=seg)
        if kind == 'king':
            G.cube(f'{name}_cross', (0.006, 0.006, 0.022), at=(at[0], at[1], at[2] + h + 0.01), material=material)
            G.cube(f'{name}_bar', (0.016, 0.006, 0.006), at=(at[0], at[1], at[2] + h + 0.013), material=material)
        else:
            G.sphere(f'{name}_crown', 0.008, at=(at[0], at[1], at[2] + h + 0.004), material=material, segments=8, rings=5)


WHITE = (('king', 4, 0), ('rook', 0, 0), ('pawn', 1, 1), ('pawn', 5, 1), ('pawn', 6, 2), ('queen', 2, 3),
         ('knight', 5, 2), ('bishop', 3, 2))
BLACK = (('king', 4, 7), ('rook', 7, 7), ('pawn', 2, 6), ('pawn', 3, 5), ('pawn', 6, 6), ('queen', 5, 5),
         ('knight', 1, 5), ('bishop', 4, 6))


def _top():
    """The table's top: a slab with a chamfered edge and a moulded band under it, the board inlaid in a border."""
    F.box('top', CHESS_TOP, 0.05, CHESS_TOP, (0, 0.735, BOARD[1]), M.stone, bevel=0.015, bevel_segments=2)
    F.box('top_under', CHESS_TOP - 0.06, 0.02, CHESS_TOP - 0.06, (0, 0.705, BOARD[1]), M.stone, bevel=0.006,
          bevel_segments=1)
    F.box('board', 8 * SQUARE + 0.03, 0.003, 8 * SQUARE + 0.03, (0, BOARD[0] - 0.0005, BOARD[1]), M.square_dark)
    F.box('board_light', 8 * SQUARE, 0.003, 8 * SQUARE, (0, BOARD[0] + 0.0005, BOARD[1]), M.square_light)
    for i in range(8):
        for j in range(8):
            if (i + j) % 2 == 0:
                x, z = _square(i, j)
                F.box(f'sq{i}{j}', SQUARE, 0.003, SQUARE, (x, BOARD[0] + 0.001, z), M.square_dark)


def parkChessTable():
    with G.new_parts() as still:
        _pedestal(16)
        _top()
        _stools(16)
        # a game under way: white on the left, black across; a taken black pawn at white's side of the board
        for k, (kind, i, j) in enumerate(WHITE):
            _piece(f'white{k}', kind, i, j, M.piece_white)
        for k, (kind, i, j) in enumerate(BLACK):
            _piece(f'black{k}', kind, i, j, M.piece_black)
        G.revolve('taken', [(0, 0), (0.017, 0), (0.017, 0.005), (0.008, 0.02), (0, 0.022)],
                  at=F.at(-0.29, BOARD[0] - 0.002, 0.25), material=M.piece_black, segments=10)
    G.join(still, 'table', origin=(0, 0, 0))
    with G.new_parts() as parts:
        _piece('pawn', 'pawn', 4, 1, M.piece_white)
    x, z = _square(4, 1)
    pawn = G.hinged(G.join(parts, 'pawn'), F.at(x, BOARD[0], z))
    G.settle(centre=False)
    G.slide(pawn, 'move', (-SQUARE, 0, 0), seconds=0.6)   # one square on, towards black (game +x)
    return G.meshes()


def parkChessTableFar():
    """Its far version by hand (decimating it folds the inlaid board into spikes): the table, the board's
    squares and the stools at fewer sides, the two kings, no other pieces."""
    with G.new_parts():
        _pedestal(12)
        _top()
        _stools(12)
        _piece('white_king', 'king', 4, 0, M.piece_white, low=True)
        _piece('black_king', 'king', 4, 7, M.piece_black, low=True)
    G.join_all('parkChessTable_far')
    G.settle(centre=False)
    return G.meshes()


SEESAW = {'loops': {'seesaw': 'seesaw'}, 'ride': 'seesaw'}
SPRING = {'loops': {'ride': 'rock'}, 'ride': 'rock'}
MERRY = {'loops': {'spin': 'spin'}, 'ride': 'spin'}
# The net swishes after each shot: the first 1.4 s into the use, then every 3.2 s while it lasts.
HOOP = {'uses': {'shoot': [{'clip': 'swish', 'delay': 1.4, 'every': 3.2}, 'ball_taken']}}
CHESS = {'uses': {'chess': 'move'}}

FAMILY = G.Family(
    'playground', M,
    [G.Model('slide', slide, size=(0.75, 2.805, 1.934), key='model.slide', centre=(0, 0.0024), far=0.25,
             paint=[('body', 'Frame'), ('chute', 'Slide'), ('roof', 'Roof')]),
     G.Model('climbingFrame', climbingFrame, size=(1.814, 3.45, 1.99), key='model.climbingFrame', centre=(0, 0.015),
             far=0.25, paint=[('timber', 'Frame'), ('shingle', 'Roof'), ('rope', 'Net')]),
     G.Model('seesaw', seesaw, size=(2.5, 0.846, 0.42), key='model.seesaw', motions=SEESAW, far=0.25,
             paint=[('body', 'Frame'), ('seat', 'Seats'), ('plank', 'Plank')]),
     G.Model('springRider', springRider, size=(0.4, 0.949, 0.957), key='model.springRider', centre=(0, 0.0387),
             motions=SPRING, far=0.25, paint=[('horse', 'Horse'), ('saddle', 'Saddle'), ('spring', 'Spring')]),
     G.Model('merryGoRound', merryGoRound, size=(1.96, 1.04, 1.96), key='model.merryGoRound', motions=MERRY,
             far=0.25, paint=[('platform', 'Platform'), ('rail', 'Rails'), ('deck', 'Stripes')]),
     G.Model('basketballHoop', basketballHoop, size=(1.2, 3.7, 0.972), key='model.basketballHoop', centre=(0, 0.016),
             motions=HOOP, far=0.25, paint=[('post', 'Frame'), ('board', 'Backboard'), ('rim', 'Hoop')]),
     G.Model('pingPongTable', pingPongTable, size=(2.74, 0.925, 1.86), key='model.pingPongTable', far=0.25,
             paint=[('table_top', 'Table'), ('concrete', 'Legs')]),
     G.Model('parkChessTable', parkChessTable, size=(1.44, 0.855, 0.7), key='model.parkChessTable',
             motions=CHESS, far=parkChessTableFar, paint=[('stone', 'Table'), ('stool', 'Stools')])],
    comment='Blender-built playground (tools/art/blender/families/playground.py): a slide, a climbing '
            'frame, a seesaw, a spring rider, a merry-go-round, a basketball hoop, a ping-pong table and '
            'a park chess table. Listed in the base manifest\'s packs. The materials map onto the game\'s '
            'finishes so a model costs one draw call per finish group. The seesaw\'s plank, the spring '
            'rider\'s horse and the merry-go-round\'s platform go round while played on, carrying their '
            'riders (`ride`); the hoop\'s net swishes after a shot and a pawn moves on the chess table. '
            'Each has a far version for the town overview.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
