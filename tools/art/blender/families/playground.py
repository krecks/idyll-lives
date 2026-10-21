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
    bat=('#C8322A', 'gloss'),
    bat_blue=('#2E4E8F', 'gloss'),
    bat_handle=('#C9A06A', 'satin'),
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


def _chute(name, steps=22):
    """The chute: a U-section trough swept down a smoothstep curve (flat at the top and at the run-out)."""
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
    vs, faces = [], []
    for i, p in enumerate(path):
        t = (path[min(i + 1, steps)] - path[max(i - 1, 0)]).normalized()
        up = t.cross(X).normalized()
        if up.z < 0:
            up = -up
        for u, v in section:
            vs.append(p + X * u + up * v)
    for i in range(steps):
        for k in range(n):
            k2 = (k + 1) % n
            faces.append((i * n + k, i * n + k2, (i + 1) * n + k2, (i + 1) * n + k))
    for base in (0, steps * n):   # the ends: the U as three quads
        q = [base + k for k in range(n)]
        faces += [(q[0], q[1], q[6], q[7]), (q[1], q[2], q[5], q[6]), (q[2], q[3], q[4], q[5])]
    obj = _mesh(name, vs, faces, M.chute, smooth=False)
    return obj


# Headroom on the deck (M-community-07): the roof's eaves 1.03 m over it (a child stands under them), an open way in
# from the ladder between its handrails (no bar across the deck's back), and an arch over the chute's mouth rising
# 0.55 m over its floor (a child sits up under it).
SLIDE_EAVES = 2.46
SLIDE_ARCH = (0.27, 0.55)   # the arch over the chute's mouth: half its width, its rise over the floor


def slide():
    with G.new_parts():
        # the tower: four posts, a deck, side rails, beams round the top and a roof
        for x in (-0.28, 0.28):
            for z in (-0.31, -0.69):
                F.cyl(f'post{x}{z}', 0.035, SLIDE_EAVES, (x, SLIDE_EAVES / 2, z), M.body, verts=12)
            for k, y in enumerate((1.62, 1.86)):
                G.rod(f'side{x}{k}', F.at(x, y, -0.31), F.at(x, y, -0.69), 0.02, material=M.body, verts=8)
            G.rod(f'beam_side{x}', F.at(x, SLIDE_EAVES - 0.03, -0.31), F.at(x, SLIDE_EAVES - 0.03, -0.69), 0.025,
                  material=M.body, verts=8)
        for z in (-0.31, -0.69):
            G.rod(f'beam{z}', F.at(-0.28, SLIDE_EAVES - 0.03, z), F.at(0.28, SLIDE_EAVES - 0.03, z), 0.025,
                  material=M.body, verts=8)
        F.box('deck', 0.62, 0.05, 0.44, (0, 1.405, -0.5), M.tread, bevel=0.008)
        F.box('deck_frame', 0.6, 0.06, 0.06, (0, 1.35, -0.31), M.body)
        F.box('deck_frame_back', 0.6, 0.06, 0.06, (0, 1.35, -0.69), M.body)
        G.cone('roof', 0.53, 0.02, 0.3, at=F.at(0, SLIDE_EAVES + 0.15, -0.5), material=M.roof, verts=4, rot=(0, 0, 45))
        G.sphere('roof_knob', 0.035, at=F.at(0, SLIDE_EAVES + 0.31, -0.5), material=M.roof, segments=10, rings=6)
        # the ladder behind, leaning on the deck; its rails go on up as handrails either side of the way in
        for x in (-0.25, 0.25):
            G.rod(f'ladder{x}', F.at(x, 0.0, -0.94), F.at(x, 1.42, -0.7), 0.025, material=M.body, verts=8)
            G.rod(f'ladder_hand{x}', F.at(x, 1.42, -0.7), F.at(x, 1.95, -0.69), 0.025, material=M.body, verts=8)
            G.rod(f'ladder_rail{x}', F.at(x, 1.95, -0.69), F.at(x * 1.12, 1.95, -0.69), 0.025, material=M.body, verts=8)
        for i in range(1, 6):
            t = i / 6
            F.box(f'rung{i}', 0.48, 0.03, 0.08, (0, 1.42 * t, -0.94 + 0.24 * t), M.tread, bevel=0.006)
        # the chute, an arch over its mouth and the legs under its run-out
        _chute('chute')
        rx, ry = SLIDE_ARCH
        arch = [F.at(rx * math.cos(math.pi * i / 14), SLIDE_TOP[0] + ry * math.sin(math.pi * i / 14), -0.27) for i in range(15)]
        _tube('arch', arch, 0.018, M.body, verts=6)
        for x in (-0.18, 0.18):
            G.rod(f'leg{x}', F.at(x, 0.0, 0.84), F.at(x, 0.29, 0.84), 0.025, material=M.body, verts=8)
            F.cyl(f'foot{x}', 0.045, 0.02, (x, 0.01, 0.84), M.steel, verts=10)
    G.join_all('slide')
    G.settle(centre=False)
    return G.meshes()


# --- the climbing frame ----------------------------------------------------------------------------------------
# A wooden fort on four posts: a deck at 1.1 m, railings on three sides, a rope net up the front, a
# fireman's pole beside it and a pitched roof with a flag.
def climbingFrame():
    deck = 1.1
    with G.new_parts():
        for x in (-0.85, 0.85):
            for z in (-0.85, 0.85):
                F.box(f'post{x}{z}', 0.1, 2.3, 0.1, (x, 1.15, z), M.timber, bevel=0.01)
        F.box('deck', 1.8, 0.06, 1.8, (0, deck, 0), M.timber, bevel=0.006)
        for z in (-0.82, 0.82):
            F.box(f'joist{z}', 1.6, 0.12, 0.06, (0, deck - 0.09, z), M.timber)
        # railings: a top and bottom rail and balusters on the back and both sides
        for side in ('back', 'left', 'right'):
            for k, y in enumerate((deck + 0.12, deck + 0.5)):
                if side == 'back':
                    F.box(f'{side}_rail{k}', 1.6, 0.05, 0.05, (0, y, -0.85), M.timber)
                else:
                    x = -0.85 if side == 'left' else 0.85
                    F.box(f'{side}_rail{k}', 0.05, 0.05, 1.6, (x, y, 0), M.timber)
            for i in range(7):
                c = -0.6 + i * 0.2
                at = (c, deck + 0.31, -0.85) if side == 'back' else (-0.85 if side == 'left' else 0.85, deck + 0.31, c)
                F.box(f'{side}_baluster{i}', 0.04, 0.34, 0.04, at, M.timber)
        # the roof: two panels on rafters, a ridge beam, and beams across the front and back
        for s in (-1, 1):
            G.cube(f'roof{s}', (1.04, 1.96, 0.04), at=(s * 0.45, 0, 2.48), material=M.shingle, rot=(0, s * 30, 0),
                   bevel=0.008)
        F.box('ridge', 0.08, 0.08, 1.94, (0, 2.74, 0), M.timber)
        for z in (-0.85, 0.85):
            F.box(f'beam{z}', 1.8, 0.1, 0.08, (0, 2.25, z), M.timber)
            for s in (-1, 1):
                G.rod(f'rafter{z}{s}', F.at(s * 0.85, 2.3, z), F.at(0, 2.7, z), 0.03, material=M.timber, verts=6)
        # the flag on the ridge, at the back
        F.cyl('flagpole', 0.02, 0.8, (0, 3.05, -0.85), M.rail, verts=8)
        F.box('flag', 0.32, 0.2, 0.015, (0.17, 3.33, -0.85), M.flag)
        # the rope net up the front, from the deck's edge down to the ground a little out
        top, bottom = (deck - 0.02, 0.86), (0.04, 0.96)
        xs = [-0.75 + i * 0.18 for i in range(6)]
        for i, x in enumerate(xs):
            G.rod(f'net_down{i}', F.at(x, *top), F.at(x, *bottom), 0.014, material=M.rope, verts=6)
        for j in range(6):
            t = j / 5
            y, z = top[0] + (bottom[0] - top[0]) * t, top[1] + (bottom[1] - top[1]) * t
            G.rod(f'net_across{j}', F.at(xs[0], y, z), F.at(xs[-1], y, z), 0.014, material=M.rope, verts=6)
        for x in (xs[0], xs[-1]):
            F.cyl(f'peg{x}', 0.03, 0.06, (x, 0.03, 0.96), M.timber, verts=8)
        # the fireman's pole, beside the net, hung from a bar off the front beam
        F.cyl('pole', 0.025, 2.05, (0.55, 1.025, 0.95), M.rail, verts=10)
        G.rod('pole_arm', F.at(0.55, 2.04, 0.95), F.at(0.55, 2.2, 0.85), 0.025, material=M.rail, verts=8)
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
            for x in (-0.22, 0.22):
                G.rod(f'leg{z}{x}', F.at(x, 0.02, z), F.at(0, AXLE, z), 0.028, material=M.body, verts=10)
            F.box(f'foot{z}', 0.56, 0.04, 0.08, (0, 0.02, z), M.body, bevel=0.01)
            F.cyl(f'boss{z}', 0.05, 0.05, (0, AXLE, z), M.steel, rot=(90, 0, 0), verts=16)
        G.rod('axle', F.at(0, AXLE, -0.21), F.at(0, AXLE, 0.21), 0.022, material=M.steel, verts=12)
        for x in (-1.1, 1.1):   # rubber bumpers the low end comes down onto
            F.cyl(f'bumper{x}', 0.1, 0.16, (x, 0.08, 0), M.rubber, verts=16)
            G.torus(f'bumper_tyre{x}', 0.1, 0.03, at=F.at(x, 0.17, 0), material=M.rubber,
                    major_segments=16, minor_segments=6)
    G.join(still, 'stand', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.box('plank', 2.5, 0.06, 0.24, (0, 0.48, 0), M.plank, bevel=0.015)
        F.cyl('sleeve', 0.045, 0.14, (0, AXLE, 0), M.steel, rot=(90, 0, 0), verts=16)
        F.box('bracket', 0.16, 0.04, 0.12, (0, 0.47, 0), M.steel, bevel=0.005)
        for s in (-1, 1):
            F.box(f'seat{s}', 0.36, 0.05, 0.3, (s * 0.75, 0.535, 0), M.seat, bevel=0.018)
            G.rod(f'handle_post{s}', F.at(s * 0.47, 0.5, 0), F.at(s * 0.47, 0.82, 0), 0.022, material=M.rail, verts=10)
            G.rod(f'handle_bar{s}', F.at(s * 0.47, 0.82, -0.14), F.at(s * 0.47, 0.82, 0.14), 0.02, material=M.rail,
                  verts=10)
            for z in (-0.1, 0.1):
                G.rod(f'grip{s}{z}', F.at(s * 0.47, 0.82, z - 0.05), F.at(s * 0.47, 0.82, z + 0.05), 0.026,
                      material=M.rubber, verts=10)
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


def springRider():
    with G.new_parts() as still:
        F.cyl('base', 0.2, 0.03, (0, 0.015, 0), M.steel, verts=24, bevel=0.008)
        F.cyl('base_plate', 0.13, 0.02, (0, 0.04, 0), M.steel, verts=20)
        turns, r = 3.5, 0.1
        coil = []
        for i in range(int(turns * 14) + 1):
            a = math.tau * i / 14
            y = 0.06 + (SPRING_TOP - 0.08) * i / (turns * 14)
            coil.append(F.at(r * math.cos(a), y, r * math.sin(a)))
        _tube('coil', coil, 0.018, M.spring, verts=6)
    G.join(still, 'spring', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.cyl('top_plate', 0.12, 0.02, (0, HORSE_DRAWN, 0), M.steel, verts=20)
        F.cyl('mount', 0.05, 0.1, (0, HORSE_DRAWN + 0.06, 0), M.steel, verts=12)
        G.sphere('barrel', 0.15, at=F.at(0, 0.6, -0.02), material=M.horse, scale=(0.85, 2.15, 1.0), segments=20, rings=12)
        G.rod('neck', F.at(0, 0.66, 0.18), F.at(0, 0.9, 0.29), 0.075, material=M.horse, verts=14)
        G.sphere('head', 0.1, at=F.at(0, 0.96, 0.33), material=M.horse, scale=(0.85, 1.55, 1.0), rot=(-25, 0, 0),
                 segments=16, rings=10)
        G.sphere('muzzle', 0.07, at=F.at(0, 0.91, 0.44), material=M.mane, scale=(1.0, 1.1, 0.85), segments=14, rings=8)
        for s in (-1, 1):
            G.cone(f'ear{s}', 0.028, 0.004, 0.08, at=F.at(s * 0.045, 1.07, 0.29), material=M.horse, verts=8,
                   rot=(15, s * 12, 0))
            G.sphere(f'eye{s}', 0.016, at=F.at(s * 0.075, 1.0, 0.37), material=M.eye, segments=8, rings=6)
            G.cylinder(f'nostril{s}', 0.01, 0.01, at=F.at(s * 0.03, 0.9, 0.51), material=M.eye, rot=(90, 0, 0), verts=8)
            # legs: short stubs, front and back, so it reads as a horse
            for z in (0.18, -0.2):
                G.rod(f'leg{s}{z}', F.at(s * 0.08, 0.55, z), F.at(s * 0.1, 0.42, z + 0.04 * (1 if z > 0 else -1)),
                      0.04, material=M.horse, verts=10)
        F.box('mane', 0.04, 0.06, 0.26, (0, 0.96, 0.21), M.mane, rot=(-55, 0, 0), bevel=0.015)
        F.box('forelock', 0.05, 0.04, 0.06, (0, 1.06, 0.34), M.mane, bevel=0.012)
        G.rod('tail', F.at(0, 0.66, -0.31), F.at(0, 0.5, -0.41), 0.035, material=M.mane, verts=10)
        F.box('saddle', 0.28, 0.05, 0.26, (0, 0.755, -0.04), M.saddle, bevel=0.02)
        G.rod('handles', F.at(-0.16, 0.93, 0.24), F.at(0.16, 0.93, 0.24), 0.018, material=M.grip, verts=10)
        for s in (-1, 1):
            F.sphere(f'handle_end{s}', 0.026, (s * 0.16, 0.93, 0.24), M.grip, segments=10, rings=6)
        G.rod('pegs', F.at(-0.19, 0.5, 0.06), F.at(0.19, 0.5, 0.06), 0.016, material=M.grip, verts=10)
        for s in (-1, 1):
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
        F.cyl('ground_plate', 0.3, 0.03, (0, 0.015, 0), M.steel, verts=24, bevel=0.008)
        F.cyl('hub', 0.16, 0.12, (0, 0.09, 0), M.steel, verts=20)
    G.join(still, 'hub', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.cyl('platform', 0.95, 0.08, (0, 0.22, 0), M.platform, verts=48, bevel=0.012)
        G.torus('edge', 0.95, 0.03, at=F.at(0, 0.22, 0), material=M.rail, major_segments=48, minor_segments=6)
        F.cyl('skirt', 0.6, 0.05, (0, 0.155, 0), M.steel, verts=24)
        for a in (45, -45):
            F.box(f'stripe{a}', 0.22, 0.008, 1.72, (0, 0.263, 0), M.deck, rot=(0, 0, a))
        F.cyl('centre_disc', 0.2, 0.012, (0, 0.266, 0), M.deck, verts=24)
        F.cyl('column', 0.05, 0.7, (0, 0.59, 0), M.rail, verts=14)
        F.sphere('knob', 0.08, (0, 0.96, 0), M.deck, segments=14, rings=8)
        for k in range(4):
            t = math.radians(45 + 90 * k)
            c, s = math.sin(t), math.cos(t)
            pts = _bezier((0.05 * c, 0.9, 0.05 * s), (0.78 * c, 0.9, 0.78 * s), (0.78 * c, 0.25, 0.78 * s), 12)
            _tube(f'rail{k}', [F.at(*p) for p in pts], 0.022, M.rail, verts=8)
            F.cyl(f'rail_foot{k}', 0.04, 0.012, (0.78 * c, 0.266, 0.78 * s), M.steel, verts=10)
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
    """A diamond-mesh net: strands zigzagging from the rim (r 0.21) to a narrower bottom ring (r 0.13)."""
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
                for j in range(rows):
                    G.rod(f'{name}{i}{d}{j}', F.at(*pts[j]), F.at(*pts[j + 1]), 0.006, material=M.net, verts=4)
        G.torus(f'{name}_hem', r1, 0.008, at=F.at(0, y1, z), material=M.net, major_segments=16, minor_segments=4)
    return G.join(parts, name)


def basketballHoop():
    with G.new_parts() as still:
        F.box('ballast', 0.62, 0.22, 0.6, (0, 0.11, -0.17), M.post, bevel=0.03)
        F.box('ballast_lid', 0.5, 0.02, 0.48, (0, 0.225, -0.17), M.steel, bevel=0.008)
        F.cyl('pole', 0.05, 3.2, (0, 1.75, -0.3), M.post, verts=14)
        F.cyl('pole_cap', 0.055, 0.03, (0, 3.36, -0.3), M.steel, verts=14)
        for y, to in ((3.3, 3.45), (3.0, 3.08)):
            G.rod(f'arm{y}', F.at(0, y, -0.3), F.at(0, to, 0.02), 0.025, material=M.post, verts=8)
        bw, bh, by = BOARD_W, BOARD_H, BOARD_Y
        F.box('board', bw, bh, 0.03, (0, by, 0.04), M.board, bevel=0.008)
        for y in (by + bh / 2 - 0.015, by - bh / 2 + 0.015):
            F.box(f'border{y}', bw, 0.03, 0.034, (0, y, 0.04), M.board_mark)
        for x in (-bw / 2 + 0.015, bw / 2 - 0.015):
            F.box(f'border{x}', 0.03, bh, 0.034, (x, by, 0.04), M.board_mark)
        # the shooter's square, 0.59 x 0.45 m as on a regulation board, its foot level with the rim
        tw, th = 0.59, 0.45
        ty = RIM[0] + th / 2
        for w, h, x, y in ((tw, 0.025, 0, ty + th / 2), (tw, 0.025, 0, ty - th / 2), (0.025, th, -tw / 2, ty),
                           (0.025, th, tw / 2, ty)):
            F.box(f'target{x}{y}', w, h, 0.034, (x, y, 0.045), M.board_mark)
        F.box('rim_bracket', 0.16, 0.12, 0.05, (0, RIM[0] - 0.04, 0.075), M.rim, bevel=0.006)
        G.torus('rim', 0.22, 0.012, at=F.at(*(0, RIM[0], RIM[1])), material=M.rim, major_segments=28, minor_segments=6)
        # the ball, at rest on the ground in front
        F.sphere('ball', 0.12, (0.3, 0.12, 0.32), M.ball, segments=16, rings=10)
        G.torus('seam0', 0.121, 0.004, at=F.at(0.3, 0.12, 0.32), material=M.rubber, rot=(90, 0, 30),
                major_segments=20, minor_segments=4)
        G.torus('seam1', 0.121, 0.004, at=F.at(0.3, 0.12, 0.32), material=M.rubber, rot=(0, 0, 0),
                major_segments=20, minor_segments=4)
    G.join(still, 'hoop', origin=(0, 0, 0))
    net = G.hinged(_net('net'), F.at(0, RIM[0], RIM[1]))
    G.settle(centre=False)
    G.rock(net, 'swish', 9, seconds=0.6, axis='X')
    G.squash(net, 'swish', (0.92, 0.92, 1.18), seconds=0.6, back=True)
    return G.meshes()


# --- the ping-pong table ---------------------------------------------------------------------------------------
# A weatherproof table at the ITTF's size (2.74 x 1.525 m, its top 0.76 m up) on 3 x 2 tiles, the players at its two
# ends (content `places`): a green top on two concrete legs, a steel net 15.25 cm high reaching 15.25 cm past each
# side, two bats and a ball. Still. (It was 1.9 x 0.94 m, its top at 0.78.)
PONG = (2.74, 0.76, 1.525)   # the top's length, height and width


def pingPongTable():
    length, top, width = PONG
    hl, hw = length / 2, width / 2
    with G.new_parts():
        F.box('top', length, 0.03, width, (0, top - 0.015, 0), M.table_top, bevel=0.01)
        for x in (-0.85, 0.85):
            F.box(f'leg{x}', 0.12, top - 0.03, 1.0, (x, (top - 0.03) / 2, 0), M.concrete, bevel=0.015)
            F.box(f'foot{x}', 0.22, 0.04, 1.12, (x, 0.02, 0), M.concrete, bevel=0.01)
        y = top + 0.001
        for w, d, x, z in ((length, 0.02, 0, hw - 0.01), (length, 0.02, 0, -hw + 0.01), (0.02, width, hl - 0.01, 0),
                           (0.02, width, -hl + 0.01, 0), (length, 0.003, 0, 0)):
            F.box(f'line{x}{z}', w, 0.003, d, (x, y, z), M.lines)
        # the steel net: a plate between two posts, a white band along its top
        net = width + 2 * 0.1525
        F.box('net', 0.02, 0.135, net, (0, top + 0.0675, 0), M.steel)
        F.box('net_band', 0.026, 0.02, net, (0, top + 0.1425, 0), M.lines)
        for z in (-net / 2, net / 2):
            F.box(f'net_post{z}', 0.04, 0.17, 0.03, (0, top + 0.08, z), M.steel, bevel=0.005)
        # two bats and the ball
        for colour, x, z, a in ((M.bat, -0.8, 0.3, 20), (M.bat_blue, 0.85, -0.32, 200)):
            F.cyl(f'bat{x}', 0.075, 0.012, (x, top + 0.006, z), colour, verts=20)
            dx, dz = 0.11 * math.sin(math.radians(a)), 0.11 * math.cos(math.radians(a))
            G.rod(f'bat_handle{x}', F.at(x + 0.06 * dx / 0.11, top + 0.008, z + 0.06 * dz / 0.11),
                  F.at(x + dx, top + 0.008, z + dz), 0.014, material=M.bat_handle, verts=8)
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
    for s in (-1, 1):
        G.revolve(f'stool{s}', [(0, 0), (0.11, 0), (0.11, 0.03), (0.07, 0.06), (0.06, 0.4), (0.12, 0.43),
                                (0.12, 0.47), (0, 0.47)], at=F.at(s * STOOL, 0, 0), material=M.stool,
                  segments=segments)


def _piece(name, kind, i, j, material):
    """A low chess piece standing on square (i, j)."""
    x, z = _square(i, j)
    y = BOARD[0] + 0.002
    at = F.at(x, y, z)
    if kind == 'pawn':
        G.revolve(name, [(0, 0), (0.016, 0), (0.016, 0.006), (0.011, 0.01), (0.007, 0.03), (0.011, 0.033),
                         (0, 0.034)], at=at, material=material, segments=10)
        G.sphere(f'{name}_head', 0.011, at=(at[0], at[1], at[2] + 0.042), material=material, segments=8, rings=5)
    elif kind == 'rook':
        G.revolve(name, [(0, 0), (0.018, 0), (0.018, 0.007), (0.012, 0.012), (0.011, 0.04), (0.016, 0.044),
                         (0.016, 0.056), (0, 0.056)], at=at, material=material, segments=10)
    else:   # king or queen: taller, with a cap
        h = 0.07 if kind == 'king' else 0.064
        G.revolve(name, [(0, 0), (0.019, 0), (0.019, 0.007), (0.012, 0.014), (0.007, h - 0.02), (0.014, h - 0.012),
                         (0.009, h), (0, h)], at=at, material=material, segments=10)
        if kind == 'king':
            G.cube(f'{name}_cross', (0.006, 0.006, 0.022), at=(at[0], at[1], at[2] + h + 0.01), material=material)
        else:
            G.sphere(f'{name}_crown', 0.009, at=(at[0], at[1], at[2] + h + 0.004), material=material, segments=8, rings=5)


def parkChessTable():
    with G.new_parts() as still:
        G.revolve('pedestal', [(0, 0), (0.17, 0), (0.17, 0.04), (0.1, 0.08), (0.08, 0.6), (0.12, 0.68), (0.12, 0.7),
                               (0, 0.7)], at=F.at(0, 0, BOARD[1]), material=M.stool, segments=16)
        F.box('top', CHESS_TOP, 0.06, CHESS_TOP, (0, 0.73, BOARD[1]), M.stone, bevel=0.015)
        F.box('board', 8 * SQUARE + 0.02, 0.003, 8 * SQUARE + 0.02, (0, BOARD[0] - 0.0005, BOARD[1]), M.square_dark)
        F.box('board_light', 8 * SQUARE, 0.003, 8 * SQUARE, (0, BOARD[0] + 0.0005, BOARD[1]), M.square_light)
        for i in range(8):
            for j in range(8):
                if (i + j) % 2 == 0:
                    x, z = _square(i, j)
                    F.box(f'sq{i}{j}', SQUARE, 0.003, SQUARE, (x, BOARD[0] + 0.001, z), M.square_dark)
        _stools(16)
        # a game under way: white on the left, black across
        for k, (kind, i, j) in enumerate((('king', 4, 0), ('rook', 0, 0), ('pawn', 1, 1), ('pawn', 5, 1),
                                          ('pawn', 6, 2), ('queen', 2, 3))):
            _piece(f'white{k}', kind, i, j, M.piece_white)
        for k, (kind, i, j) in enumerate((('king', 4, 7), ('rook', 7, 7), ('pawn', 2, 6), ('pawn', 3, 5),
                                          ('pawn', 6, 6), ('queen', 5, 5))):
            _piece(f'black{k}', kind, i, j, M.piece_black)
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
        G.revolve('pedestal', [(0, 0), (0.17, 0), (0.17, 0.04), (0.1, 0.08), (0.08, 0.6), (0.12, 0.68), (0.12, 0.7),
                               (0, 0.7)], at=F.at(0, 0, BOARD[1]), material=M.stool, segments=12)
        F.box('top', CHESS_TOP, 0.06, CHESS_TOP, (0, 0.73, BOARD[1]), M.stone)
        F.box('board_light', 8 * SQUARE, 0.003, 8 * SQUARE, (0, BOARD[0] + 0.0005, BOARD[1]), M.square_light)
        for i in range(8):
            for j in range(8):
                if (i + j) % 2 == 0:
                    x, z = _square(i, j)
                    F.box(f'sq{i}{j}', SQUARE, 0.003, SQUARE, (x, BOARD[0] + 0.001, z), M.square_dark)
        _stools(12)
        _piece('white_king', 'king', 4, 0, M.piece_white)
        _piece('black_king', 'king', 4, 7, M.piece_black)
    G.join_all('parkChessTable_far')
    G.settle(centre=False)
    return G.meshes()


SEESAW = {'loops': {'seesaw': 'seesaw'}, 'ride': 'seesaw'}
SPRING = {'loops': {'ride': 'rock'}, 'ride': 'rock'}
MERRY = {'loops': {'spin': 'spin'}, 'ride': 'spin'}
# The net swishes after each shot: the first 1.4 s into the use, then every 3.2 s while it lasts.
HOOP = {'uses': {'shoot': {'clip': 'swish', 'delay': 1.4, 'every': 3.2}}}
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
