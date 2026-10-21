# The couch potato pack (P3): the grand cinema wall, the Pixel Arcade console, the Cloud Nine power recliner, the
# sofa-side snack fridge, the living-room workout game and the lava glow lamp.
# Notes: pack_couchPotato.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The cinema's two side curtains are drawn apart (`curtains`), the console's disc tray slides out
# (`tray`), the recliner's back tips and its footrest rises (`recline`), the fridge door swings (`fridge_open`),
# the balance board tilts (`board`) and the lava lamp's wax rises (`blobs`) while more wax drifts round all the
# time (`drift`, ambient). The screens the residents watch (the cinema's, the console's) stand on the wall at the
# back, the seats in front of them, the sitters facing the object's back (content `seat.facing: "back"`, as the
# piano): each is built as its old part first stood (the seats at the back, the screen at the front edge facing
# them) and turned round (`_turn_round`), as is its old part.
#
# Run:  cargo art pack_couchPotato [--only recliner] [--preview]
import math
import os
import sys

import bmesh
import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # the cinema wall
    leather=('#7A2E2E', 'satin'),        # the twin recliners; plain, so the curtains take the textured draws
    cinema_frame=('#5C6168', 'satin'),   # the console, posts and speakers; its main part
    curtain=('#8E2A3A', 'fabric'),
    pelmet=('#3A3D41', 'satin'),
    # screens
    screen_back=('#CDB08A', 'satin'),    # the oak acoustic backs, plain
    slat=('#B08E66', 'satin'),
    bezel=('#1B2A3A', 'gloss'),
    picture=('#3BA3E0', 'gloss', {'glow': '#3BA3E0', 'screen': True}),
    picture_b=('#D6A64A', 'gloss', {'glow': '#D6A64A', 'screen': True}),
    graphite=('#5C6168', 'metal'),
    speaker_grey=('#6E7277', 'satin'),
    speaker_cone=('#3A3D41', 'satin'),
    soda=('#C9483B', 'gloss'),
    straw=('#F2F2F0', 'satin'),
    popcorn=('#F6E7B8', 'matte'),
    dark=('#2B2C2E', 'satin'),
    # the game console
    gamer=('#3F6E9E', 'fabric'),          # the gaming loveseat; its main part
    mustard=('#D6A64A', 'satin'),
    console=('#E8E8E6', 'gloss'),
    disc=('#C9CCD0', 'chrome'),
    led=('#3BA3E0', 'gloss'),
    box_red=('#B33A32', 'satin'),
    box_blue=('#3E8EDE', 'satin'),
    # the recliner
    plush=('#C08A52', 'satin'),           # the recliner's microsuede; its main part
    plinth=('#6E5038', 'satin'),
    teal=('#4E7A7F', 'gloss'),
    throw=('#D9C7A6', 'satin'),
    # the snack fridge
    mint=('#9FD3C7', 'gloss'),            # the fridge; its main part
    mint_trim=('#7FB5A9', 'satin'),
    lid=('#E9E4DA', 'fabric'),            # the padded lid
    inside=('#F2F4F4', 'satin'),
    chrome=('#D0D3D6', 'chrome'),
    menu=('#F2A541', 'gloss'),
    can_a=('#C9483B', 'gloss'),
    can_b=('#3E8EDE', 'gloss'),
    can_c=('#9DAF8C', 'gloss'),
    takeout=('#F2EDE2', 'satin'),
    bowl=('#F2EDE2', 'gloss'),
    crisps=('#E9B54A', 'matte'),
    # the workout game
    fit_stand=('#E8E8E6', 'satin'),       # the TV stand; its main part
    oak=('#CDB08A', 'satin'),
    fit_mat=('#7FC4A4', 'fabric'),
    mat_inner=('#9FD8BB', 'fabric'),
    towel=('#E9B54A', 'fabric'),
    tv_frame=('#D5D6D4', 'satin'),
    tv_screen=('#4FC3A1', 'gloss', {'glow': '#4FC3A1', 'screen': True}),
    tv_floor=('#3FA88A', 'gloss'),
    tv_white=('#F4F1EA', 'gloss'),
    tv_orange=('#F2A541', 'gloss'),
    board=('#3E8EDE', 'satin'),
    grip=('#2B2C2E', 'matte'),
    bottle=('#3E8EDE', 'gloss'),
    weight=('#C9483B', 'satin'),
    # the lava lamp
    pouf=('#5A4A7A', 'satin'),            # the pouf; its main part (plain: the lamp's glass takes the extra draw)
    lamp_metal=('#4E7A7F', 'metal'),
    vessel=('#E96A9A', 'glass', {'glow': '#FF6FA8'}),   # the lit glass: it and the wax glow at night (the lamp's `light`)
    wax=('#F2914A', 'gloss', {'glow': '#FFA060'}),
    table_white=('#E8E8E6', 'satin'),
    rug=('#E9B54A', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


# --- meshes from points (as pack_cheerful) --------------------------------------------------------------------
def _obj(name, verts, faces, mats, sharp=35):
    """A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in mats:
        mesh.materials.append(m)
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.set_sharp_from_angle(angle=math.radians(sharp))
    return obj


def _pleats(F, name, x0, x1, y0, y1, z, folds, amp, mat, t=0.012):
    """A pleated curtain hanging in the plane at depth `z` from `y1` down to `y0`, across `x0`..`x1`."""
    steps = folds * 4
    rows = []
    for i in range(steps + 1):
        x = x0 + (x1 - x0) * i / steps
        dz = amp * math.sin(math.pi * 2 * i / 4)
        rows.append([F.at(x, y1, z + dz - t / 2), F.at(x, y1, z + dz + t / 2),
                     F.at(x, y0, z + dz + t / 2), F.at(x, y0, z + dz - t / 2)])
    verts = [v for r in rows for v in r]
    faces = [(0, 1, 2, 3), (4 * steps + 3, 4 * steps + 2, 4 * steps + 1, 4 * steps)]
    for i in range(steps):
        a, b = 4 * i, 4 * (i + 1)
        faces += [(a + k, a + (k + 1) % 4, b + (k + 1) % 4, b + k) for k in range(4)]
    return _obj(name, verts, faces, [mat], sharp=60)


def _cushion(F, name, w, h, d, at, mat, rot=(0, 0, 0), r=0.05):
    """A soft block: a box rounded by `r` all round."""
    return F.box(name, w, h, d, at, mat, rot=rot, bevel=min(r, w / 2.2, h / 2.2, d / 2.2), bevel_segments=3)


def _turn_round(objs):
    """Turns a model built in place half a turn about the footprint's middle (each part about its own origin,
    which moves round with it), as pack_bookworm turns the writer's bureau. A part's slides then go the other way."""
    for o in objs:
        G.turn(o, 180, 'Z')
        o.location = (-o.location.x, -o.location.y, o.location.z)


def _screen(F, x0, x1, y0, y1, z, stripe):
    """A screen held between two slim posts at depth `z`, its picture facing the seats behind it (-Z), the oak back
    with slats on its other side (to the wall, once the model is turned round)."""
    w, h = x1 - x0, y1 - y0
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    F.box(_name('screen_back'), w, h, 0.05, (cx, cy, z), M.screen_back, bevel=0.012)
    F.box(_name('bezel'), w - 0.03, h - 0.03, 0.004, (cx, cy, z - 0.027), M.bezel, bevel=0.002)
    F.box(_name('picture'), w * 0.82, h * 0.7, 0.002, (cx, cy + h * 0.04, z - 0.0295), M.picture)
    F.box(_name('picture_b'), w * 0.3, h * 0.12, 0.002, (cx - w * 0.2, cy - h * 0.28, z - 0.031), M.picture_b)
    F.box(_name('stripe'), w * 0.5, 0.02, 0.006, (cx, y0 + 0.06, z + 0.027), stripe)
    n = max(3, int(w / 0.09))
    for i in range(1, n):
        F.box(_name('slat'), 0.012, h - 0.04, 0.006, (x0 + w * i / n, cy, z + 0.026), M.slat)
    for x in (x0 + 0.02, x1 - 0.02):
        F.box(_name('post'), 0.04, y1 + 0.02, 0.04, (x, (y1 + 0.02) / 2, z + 0.01), M.graphite, bevel=0.008)
        F.box(_name('post_foot'), 0.08, 0.02, 0.12, (x, 0.01, z - 0.04), M.graphite, bevel=0.006)


# --- the grand cinema wall (2x1): the screen on the wall, twin recliners in front of it, a console between --------
# (built with the seats at the back and the screen at the front edge facing them, then turned round)
SCREEN_Z = 0.42


def _cinema_seat(F, x, w, outer):
    """One recliner of the pair centred on (x, 0), facing the screen (before the turn, in front): seat top 0.45, an
    arm outside."""
    _cushion(F, _name('seat_base'), w, 0.3, 0.62, (x, 0.2, -0.06), M.leather, r=0.06)
    _cushion(F, _name('seat'), w - 0.16, 0.12, 0.56, (x, 0.39, -0.02), M.leather, r=0.05)
    _cushion(F, _name('seat_back'), w, 0.66, 0.18, (x, 0.64, -0.31), M.leather, rot=(-14, 0, 0), r=0.07)
    _cushion(F, _name('seat_pillow'), w - 0.16, 0.36, 0.12, (x, 0.68, -0.2), M.leather, rot=(-14, 0, 0), r=0.05)
    _cushion(F, _name('arm'), 0.1, 0.32, 0.62, (outer, 0.48, -0.06), M.leather, r=0.045)
    F.box(_name('seat_foot'), w - 0.1, 0.05, 0.56, (x, 0.025, -0.06), M.dark, bevel=0.01)


def cinemaWall():
    F = G.GameFrame()
    with G.new_parts() as still:
        for s in (-1, 1):
            _cinema_seat(F, s * 0.5, 0.78, s * 0.86)
        # the console between the seats: cup holder with a soda, a bucket of popcorn
        F.box('console', 0.22, 0.52, 0.6, (0, 0.26, -0.06), M.cinema_frame, bevel=0.03)
        F.cyl('cup_ring', 0.045, 0.02, (0, 0.53, 0.08), M.dark, verts=14)
        G.revolve('soda', [(0, 0), (0.035, 0), (0.045, 0.14), (0, 0.14)], at=F.at(0, 0.52, 0.08), material=M.soda,
                  segments=14)
        F.cyl('soda_lid', 0.047, 0.01, (0, 0.665, 0.08), M.straw, verts=14)
        G.rod('straw', F.at(0.01, 0.65, 0.08), F.at(0.02, 0.76, 0.07), 0.005, material=M.straw, verts=6)
        G.revolve('bucket', [(0, 0), (0.06, 0), (0.08, 0.12), (0, 0.12)], at=F.at(0, 0.52, -0.16), material=M.soda,
                  segments=14)
        for k in range(7):
            a = 51 * k
            F.sphere(_name('popcorn'), 0.024, (0.035 * G.cosd(a), 0.645 + 0.01 * (k % 2), -0.16 + 0.035 * G.sind(a)),
                     M.popcorn, segments=7, rings=5)
        F.box('remote', 0.04, 0.02, 0.15, (-0.3, 0.465, 0.1), M.dark, rot=(0, 20, 0), bevel=0.008)
        # the screen (at the front edge until turned round), facing the seats; a pelmet over it for the curtains
        _screen(F, -0.92, 0.92, 0.88, 1.42, SCREEN_Z, M.picture)
        F.box('pelmet', 1.6, 0.1, 0.1, (0, 1.47, SCREEN_Z - 0.06), M.pelmet, bevel=0.012)
        # the surround speakers at the screen's corners
        for x in (-0.86, 0.86):
            F.box(_name('speaker'), 0.16, 0.84, 0.16, (x, 0.42, 0.33), M.speaker_grey, bevel=0.02)
            for y, r in ((0.6, 0.05), (0.76, 0.03)):
                F.cyl(_name('cone'), r, 0.012, (x, y, 0.247), M.speaker_cone, rot=(90, 0, 0), verts=14)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two curtains, closed over the picture, each gathered by `curtains` to its own side.
    zc = SCREEN_Z - 0.06
    parts = []
    for s, name in ((-1, 'curtain_left'), (1, 'curtain_right')):
        with G.new_parts() as cur:
            _pleats(F, name, s * 0.01, s * 0.78, 0.9, 1.42, zc - 0.012, 8, 0.016, M.curtain)
        parts.append(G.hinged(G.join(cur, name), F.at(s * 0.78, 1.42, zc - 0.012)))
    _turn_round(G.meshes())
    objs = G.settle()      # on its Model centre: the screen's back on the wall face
    for cur in parts:
        G.squash(cur, 'curtains', (0.22, 1, 1), seconds=1.2)
    return objs


# --- the Pixel Arcade console (1x1): the screen and console on the wall, a gaming loveseat for two in front -------
# (built with the loveseat at the back and the screen at the front edge facing it, then turned round)
def gameConsole():
    F = G.GameFrame()
    with G.new_parts() as still:
        _cushion(F, 'base', 0.88, 0.26, 0.56, (0, 0.16, -0.12), M.gamer, r=0.07)
        for x in (-0.21, 0.21):
            _cushion(F, _name('seat'), 0.4, 0.12, 0.5, (x, 0.39, -0.08), M.gamer, r=0.05)
        _cushion(F, 'back', 0.88, 0.5, 0.16, (0, 0.56, -0.34), M.gamer, rot=(-10, 0, 0), r=0.07)
        for s in (-1, 1):
            _cushion(F, _name('arm'), 0.08, 0.28, 0.5, (s * 0.405, 0.42, -0.12), M.gamer, r=0.035)
            F.cyl(_name('foot'), 0.02, 0.03, (s * 0.38, 0.015, -0.12), M.dark, verts=10)
        _cushion(F, 'pillow', 0.3, 0.28, 0.1, (0.24, 0.64, -0.24), M.mustard, rot=(-10, 0, 20), r=0.04)
        for x in (-0.2, 0.2):                                     # the controllers on the seats
            F.box(_name('pad'), 0.14, 0.03, 0.08, (x, 0.465, 0.06), M.console, bevel=0.012)
            for dx in (-0.045, 0.045):
                F.sphere(_name('grip'), 0.025, (x + dx, 0.46, 0.085), M.console, segments=10, rings=6)
            F.sphere(_name('button'), 0.008, (x + 0.03, 0.482, 0.05), M.soda, segments=6, rings=4)
            F.cyl(_name('stick'), 0.008, 0.012, (x - 0.03, 0.486, 0.05), M.dark, verts=8)
        # the screen (at the front edge until turned round) and the shelf between its posts, with the console and two
        # games
        _screen(F, -0.4, 0.4, 0.85, 1.32, SCREEN_Z, M.mustard)
        F.box('shelf', 0.74, 0.03, 0.12, (0, 0.62, 0.39), M.screen_back, bevel=0.006)
        F.box('console', 0.3, 0.06, 0.1, (0, 0.665, 0.39), M.console, bevel=0.012)
        F.box('tray_slot', 0.14, 0.012, 0.004, (0, 0.67, 0.339), M.dark)
        F.box('power', 0.05, 0.006, 0.003, (0.1, 0.65, 0.339), M.led)
        F.box('game_a', 0.14, 0.18, 0.02, (-0.27, 0.725, 0.43), M.box_red, rot=(0, 0, 4), bevel=0.003)
        F.box('game_b', 0.14, 0.18, 0.02, (-0.24, 0.725, 0.405), M.box_blue, rot=(0, 0, -3), bevel=0.003)
    G.join(still, 'body', origin=(0, 0, 0))
    # The disc tray in the console's slot, facing the players: `tray` slides it out with a disc on it.
    with G.new_parts() as tray:
        F.box('tray', 0.13, 0.008, 0.09, (0, 0.67, 0.385), M.dark, bevel=0.002)
        F.cyl('tray_disc', 0.04, 0.003, (0, 0.6755, 0.385), M.disc, verts=24)
    tray = G.join(tray, 'tray')
    _turn_round(G.meshes())
    objs = G.settle()      # on its Model centre: the screen's back on the wall face
    G.slide(tray, 'tray', (0, -0.09, 0), seconds=0.6)    # 9 cm towards the players (+Z game, once turned round)
    return objs


# --- the Cloud Nine power recliner (1x1): deep seat between rolled arms, the back and footrest power out --------
BACK_HINGE = (0.42, -0.21)     # (height, depth) the back tips about
FOOT_HINGE = (0.4, 0.25)       # and the footrest rises about


def recliner():
    F = G.GameFrame(shift=0.03)        # 3 cm forward of the wall: room for the back to tip
    with G.new_parts() as still:
        F.box('plinth', 0.6, 0.05, 0.6, (0, 0.025, -0.06), M.plinth, bevel=0.012)
        _cushion(F, 'base', 0.58, 0.3, 0.6, (0, 0.2, -0.06), M.plush, r=0.05)
        _cushion(F, 'seat', 0.56, 0.11, 0.54, (0, 0.395, -0.02), M.plush, r=0.05)
        for s in (-1, 1):
            _cushion(F, _name('arm'), 0.16, 0.5, 0.7, (s * 0.37, 0.3, -0.08), M.plush, r=0.06)
            F.cyl(_name('arm_roll'), 0.085, 0.7, (s * 0.375, 0.56, -0.08), M.plush, rot=(90, 0, 0), verts=20)
            F.cyl(_name('roll_end'), 0.06, 0.012, (s * 0.375, 0.56, 0.272), M.plinth, rot=(90, 0, 0), verts=16)
        F.cyl('cup_ring', 0.045, 0.02, (0.37, 0.65, 0.12), M.dark, verts=14)
        F.cyl('cup', 0.035, 0.12, (0.37, 0.7, 0.12), M.teal, verts=14)
        F.box('remote', 0.05, 0.02, 0.14, (-0.37, 0.655, 0.1), M.dark, bevel=0.008)
        F.box('remote_keys', 0.02, 0.004, 0.1, (-0.37, 0.666, 0.11), M.slat)
        F.box('throw', 0.02, 0.36, 0.4, (-0.456, 0.47, -0.12), M.throw, bevel=0.006)       # over the left arm
        F.box('throw_top', 0.2, 0.025, 0.4, (-0.38, 0.655, -0.12), M.throw, bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    by, bz = BACK_HINGE
    with G.new_parts() as back:                    # the back and its pillow, leaning back a little at rest
        _cushion(F, 'back', 0.6, 0.56, 0.15, (0, 0.67, -0.27), M.plush, rot=(-8, 0, 0), r=0.06)
        _cushion(F, 'back_pillow', 0.5, 0.3, 0.09, (0, 0.69, -0.18), M.plush, rot=(-8, 0, 0), r=0.04)
        _cushion(F, 'head_pillow', 0.46, 0.13, 0.11, (0, 0.88, -0.21), M.plush, rot=(-8, 0, 0), r=0.05)
    back = G.hinged(G.join(back, 'back'), F.at(0, by, bz))
    fy, fz = FOOT_HINGE
    with G.new_parts() as foot:                    # the footrest, folded down under the seat's front
        _cushion(F, 'footrest', 0.5, 0.22, 0.08, (0, fy - 0.12, fz + 0.04), M.plush, r=0.035)
    foot = G.hinged(G.join(foot, 'footrest'), F.at(0, fy, fz))
    objs = G.settle(centre=False)
    G.swing(back, 'recline', -11, seconds=1.2, axis='X')
    G.swing(foot, 'recline', -80, seconds=1.2, axis='X')
    return objs


# --- the sofa-side snack fridge (1x1): a mini fridge with a padded lid, a takeout screen on its door ------------
FR = (0.0, -0.02, 0.56, 0.37, 0.46)     # x, depth, width, height, depth of the cabinet


def snackFridge():
    F = G.GameFrame()
    x, z, w, h, d = FR
    z0, z1 = z - d / 2, z + d / 2
    t = 0.025
    with G.new_parts() as still:
        # the cabinet, open at the front: back, sides, floor and top, a white inside with a shelf of snacks
        F.box('cab_back', w, h - 0.03, t, (x, 0.03 + (h - 0.03) / 2, z0 + t / 2), M.mint, bevel=0.008)
        for s in (-1, 1):
            F.box(_name('cab_side'), t, h - 0.03, d - 0.03, (x + s * (w / 2 - t / 2), 0.03 + (h - 0.03) / 2, z - 0.015),
                  M.mint, bevel=0.008)
        F.box('cab_floor', w, t, d - 0.03, (x, 0.03 + t / 2, z - 0.015), M.mint, bevel=0.006)
        F.box('cab_top', w, t, d - 0.03, (x, h - t / 2, z - 0.015), M.mint, bevel=0.006)
        F.box('liner', w - 2 * t, h - 0.03 - 2 * t, 0.004, (x, 0.03 + (h - 0.03) / 2, z0 + t + 0.002), M.inside)
        F.box('shelf', w - 2 * t, 0.012, d - 0.08, (x, 0.2, z - 0.04), M.inside)
        for k, (cx, m) in enumerate(((-0.17, M.can_a), (-0.1, M.can_b), (-0.03, M.can_c), (0.04, M.can_a))):
            F.cyl(_name('can'), 0.03, 0.11, (cx, 0.262, -0.02 + 0.03 * (k % 2)), m, verts=14)
            F.cyl(_name('can_top'), 0.026, 0.004, (cx, 0.319, -0.02 + 0.03 * (k % 2)), M.chrome, verts=12)
        F.box('takeout_box', 0.16, 0.08, 0.14, (0.14, 0.247, -0.03), M.takeout, bevel=0.006)
        F.box('takeout_band', 0.161, 0.02, 0.141, (0.14, 0.25, -0.03), M.menu)
        F.box('pizza', 0.4, 0.035, 0.3, (0, 0.075, -0.06), M.takeout, bevel=0.004)
        F.box('pizza_logo', 0.12, 0.002, 0.08, (0, 0.093, -0.02), M.can_a)
        F.box('trim', w - 0.08, 0.012, 0.02, (x, h - 0.03, z1 - 0.004), M.mint_trim, bevel=0.004)
        for fx in (x - w / 2 + 0.06, x + w / 2 - 0.06):
            for fz in (z0 + 0.06, z1 - 0.06):
                F.cyl(_name('foot'), 0.02, 0.03, (fx, 0.015, fz), M.mint_trim, verts=10)
        F.box('kick', 0.5, 0.02, 0.04, (x, 0.04, z1 - 0.02), M.graphite, bevel=0.006)
        # the padded lid (the ottoman's top, 0.45) and a bowl of crisps on it
        _cushion(F, 'lid', 0.58, 0.08, 0.5, (x, 0.41, z), M.lid, r=0.035)
        G.revolve('bowl', [(0, 0), (0.06, 0), (0.11, 0.06), (0.105, 0.062), (0.055, 0.008), (0, 0.008)],
                  at=F.at(0.13, 0.45, -0.12), material=M.bowl, segments=18)
        for k in range(8):
            a = 45 * k
            r = 0.05 if k % 2 else 0.025
            F.sphere(_name('crisp'), 0.03, (0.13 + r * G.cosd(a), 0.5 + 0.006 * (k % 3), -0.12 + r * G.sind(a)), M.crisps,
                     scale=(1, 1, 0.35), rot=(20 * (k % 3), 30 * k, 0), segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door with its takeout screen and handle, hinged on its right edge: `fridge_open` swings it open.
    hx = x + w / 2 - 0.01
    with G.new_parts() as door:
        F.box('door', w - 0.02, h - 0.06, 0.03, (x, 0.03 + (h - 0.03) / 2, z1 + 0.003), M.mint, bevel=0.012)
        F.box('door_screen', 0.18, 0.12, 0.006, (0.08, 0.24, z1 + 0.02), M.bezel, bevel=0.004)
        F.box('door_menu', 0.15, 0.09, 0.002, (0.08, 0.24, z1 + 0.0235), M.menu)
        F.box('door_pick', 0.04, 0.03, 0.002, (0.04, 0.26, z1 + 0.0245), M.can_a)
        F.box('handle', 0.018, 0.2, 0.018, (x - w / 2 + 0.06, 0.2, z1 + 0.05), M.chrome, bevel=0.007)
        for y in (0.13, 0.27):
            F.box(_name('handle_post'), 0.014, 0.014, 0.034, (x - w / 2 + 0.06, y, z1 + 0.032), M.chrome)
    door = G.hinged(G.join(door, 'door'), F.at(hx, 0.2, z1 + 0.003))
    objs = G.settle(centre=False)
    G.swing(door, 'fridge_open', -100, seconds=0.9)
    return objs


# --- the living-room workout game (1x1): a TV on a slim stand, a step mat and a balance board before it ---------
def fitnessGame():
    F = G.GameFrame()
    with G.new_parts() as still:
        for y in (0.2, 0.56):
            F.box(_name('stand_shelf'), 0.8, 0.04, 0.3, (0, y, -0.28), M.oak, bevel=0.01)
        for x in (-0.37, 0.37):
            F.box(_name('stand_side'), 0.04, 0.58, 0.28, (x, 0.29, -0.28), M.fit_stand, bevel=0.01)
        # the TV on its foot, a workout on the screen, the sensor bar on top
        y0, w, h = 0.58, 0.8, 0.53
        ph = h - 0.07
        py = y0 + 0.07 + ph / 2
        zc = -0.35
        F.box('tv_foot', 0.3, 0.014, 0.14, (0, y0 + 0.007, zc), M.tv_frame, bevel=0.006)
        F.box('tv_neck', 0.06, 0.07, 0.03, (0, y0 + 0.045, zc - 0.03), M.tv_frame, bevel=0.01)
        F.box('tv', w, ph, 0.04, (0, py, zc - 0.01), M.tv_frame, bevel=0.012)
        F.box('tv_screen', w - 0.03, ph - 0.03, 0.004, (0, py + 0.003, zc + 0.011), M.tv_screen)
        F.box('tv_band', w - 0.03, 0.1, 0.003, (0, py - ph / 2 + 0.07, zc + 0.0135), M.tv_floor)
        F.sphere('coach_head', 0.035, (-0.12, py + 0.06, zc + 0.014), M.tv_white, scale=(1, 0.1, 1), segments=12, rings=8)
        F.box('coach', 0.06, 0.13, 0.004, (-0.12, py - 0.04, zc + 0.014), M.tv_orange, bevel=0.0015)
        F.box('score_a', 0.2, 0.04, 0.003, (0.2, py + 0.14, zc + 0.014), M.tv_white)
        F.box('score_b', 0.14, 0.03, 0.003, (0.17, py + 0.09, zc + 0.014), M.picture_b)
        F.box('sensor', 0.3, 0.04, 0.05, (0, y0 + h + 0.025, zc - 0.005), M.tv_white, bevel=0.015)
        F.sphere('sensor_eye', 0.015, (-0.08, y0 + h + 0.025, zc + 0.02), M.soda, scale=(1, 1, 0.4), segments=8, rings=6)
        # on the shelves: a towel and a water bottle; on the floor: the step mat and a kettlebell
        F.box('towel', 0.3, 0.03, 0.2, (-0.2, 0.235, -0.28), M.towel, bevel=0.01)
        G.revolve('bottle', [(0, 0), (0.035, 0), (0.035, 0.16), (0.02, 0.19), (0.02, 0.21), (0, 0.21)],
                  at=F.at(0.25, 0.22, -0.26), material=M.bottle, segments=14)
        F.box('mat', 0.7, 0.012, 0.56, (0, 0.006, 0.17), M.fit_mat, bevel=0.004)
        F.box('mat_inner', 0.62, 0.004, 0.48, (0, 0.014, 0.17), M.mat_inner, bevel=0.002)
        G.revolve('kettlebell', [(0, 0), (0.07, 0), (0.085, 0.06), (0.07, 0.12), (0.03, 0.14), (0, 0.14)],
                  at=F.at(0.38, 0, 0.36), material=M.weight, segments=16)
        G.torus('kettle_handle', 0.045, 0.012, at=F.at(0.38, 0.16, 0.36), material=M.weight, rot=(90, 0, 0),
                major_segments=14, minor_segments=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The balance board on its rocker, on the mat: `board` tilts it 10° to one side.
    with G.new_parts() as board:
        F.cyl('board_deck', 0.2, 0.025, (0, 0.1, 0.2), M.board, scale=(1.3, 1, 1), verts=32, bevel=0.008)
        for dx in (-0.14, 0.14):
            F.box(_name('board_grip'), 0.12, 0.004, 0.16, (dx, 0.1135, 0.2), M.grip, bevel=0.002)
        F.cyl('board_rocker', 0.045, 0.06, (0, 0.057, 0.2), M.grip, rot=(90, 0, 0), verts=20)
    board = G.hinged(G.join(board, 'board'), F.at(0, 0.012, 0.2))
    objs = G.settle(centre=False)
    G.swing(board, 'board', 10, seconds=0.8, axis='Y')
    return objs


# --- the lava glow lamp (1x1): a floor pouf to zone out on, a side table with a big lava lamp --------------------
# The pouf is turned to face the lamp at its front right: the sitter faces it (content `places`, facing TURN°),
# coming from the tile to its left. 2026-10-21: the lamp stood behind the sitter's shoulder (I-packs-11).
LAMP = (0.32, 0.25)      # the side table's middle (x, depth)
POUF = (-0.04, -0.03)    # the pouf's middle (x, depth): the sitter's place
TURN = 50                # the way the pouf and its sitter face, degrees from the front towards the right
VESSEL = (0.65, 0.42)    # the glass's foot height and height


def _vessel_r(y):
    """The glass's inside radius at height `y` (it bulges a third of the way up)."""
    y0, h = VESSEL
    u = (y - y0) / h
    return 0.046 + 0.03 * math.sin(math.pi * min(1.0, max(0.0, u) / 0.8)) if u < 0.8 else 0.046 - (u - 0.8) * 0.02


def lavaLamp():
    F = G.GameFrame()
    tx, tz = LAMP
    y0, h = VESSEL
    with G.new_parts() as still:
        px, pz = POUF
        fx, fz = G.sind(TURN), G.cosd(TURN)                     # the way it faces (game x, depth)
        _cushion(F, 'pouf', 0.62, 0.36, 0.6, (px, 0.18, pz), M.pouf, rot=(0, 0, -TURN), r=0.15)
        _cushion(F, 'pouf_top', 0.54, 0.1, 0.52, (px, 0.4, pz), M.pouf, rot=(0, 0, -TURN), r=0.05)
        _cushion(F, 'lean', 0.5, 0.36, 0.14, (px - 0.26 * fx, 0.6, pz - 0.26 * fz), M.pouf, rot=(-12, 0, -TURN), r=0.06)
        F.cyl('rug', 0.2, 0.012, (-0.22, 0.006, 0.27), M.rug, verts=28)
        # the side table and the lamp: a metal foot, the glass, a cap; a pool of wax at the bottom
        F.cyl('table_top', 0.13, 0.03, (tx, 0.5, tz), M.oak, verts=24, bevel=0.006)
        F.cyl('table_stem', 0.02, 0.48, (tx, 0.25, tz), M.table_white, verts=10)
        F.cyl('table_foot', 0.1, 0.02, (tx, 0.01, tz), M.table_white, verts=20)
        G.revolve('lamp_foot', [(0, 0), (0.07, 0), (0.05, 0.12), (0.045, 0.14), (0, 0.14)], at=F.at(tx, 0.515, tz),
                  material=M.lamp_metal, segments=20)
        prof = [(0.0, 0.0)] + [(_vessel_r(y0 + h * k / 12) + 0.003, h * k / 12) for k in range(13)] + [(0.0, h)]
        G.revolve('vessel', prof, at=F.at(tx, y0, tz), material=M.vessel, segments=24)
        G.revolve('lamp_cap', [(0, 0), (0.044, 0), (0.02, 0.08), (0, 0.08)], at=F.at(tx, y0 + h, tz),
                  material=M.lamp_metal, segments=16)
        F.sphere('wax_pool', 0.045, (tx, y0 + 0.02, tz), M.wax, scale=(1, 1, 0.35), segments=14, rings=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The wax that rises while a resident zones out (`blobs`, slow) ...
    with G.new_parts() as rise:
        for dx, y, r in ((0.0, 0.7, 0.03), (0.012, 0.79, 0.024), (-0.016, 0.745, 0.018)):
            F.sphere(_name('blob'), r, (tx + dx, y, tz), M.wax, scale=(1, 1, 1.3), segments=12, rings=8)
    rise = G.hinged(G.join(rise, 'blobs'), F.at(tx, y0, tz))
    # ... and the wax that drifts round all the time near the camera (`drift`, ambient)
    with G.new_parts() as drift:
        for a, y, r in ((30, 0.9, 0.022), (200, 0.98, 0.017)):
            F.sphere(_name('drift'), r, (tx + 0.022 * G.cosd(a), y, tz + 0.022 * G.sind(a)), M.wax, scale=(1, 1, 1.3),
                     segments=10, rings=6)
    drift = G.hinged(G.join(drift, 'drift'), F.at(tx, y0, tz))
    objs = G.settle(centre=False)   # built in place: the sitter's place is content `places`
    G.slide(rise, 'blobs', (0, 0, 0.14), seconds=2.5)
    G.spin(drift, 'drift', 1, seconds=8.0)
    return objs


FAMILY = G.Family(
    'pack_couchPotato', M,
    [G.Model('cinemaWall', cinemaWall, size=(1.88, 1.52, 0.91), key='model.couchPotato.cinemaWall',
             centre=(0, 0.025), motions={'uses': {'marathon': 'curtains', 'channelSurf': 'curtains'}},
             paint=[('cinema_frame', 'Body'), ('leather', 'Upholstery'), ('curtain', 'Curtains')]),
     G.Model('gameConsole', gameConsole, size=(0.89, 1.34, 0.9), key='model.couchPotato.gameConsole',
             centre=(0, 0.02), motions={'uses': {'play': 'tray', 'speedrun': 'tray'}, 'hold': 1.5},
             paint=[('console', 'Body'), ('gamer', 'Upholstery'), ('mustard', 'Cushion')]),
     G.Model('recliner', recliner, size=(0.94, 0.949, 0.76), key='model.couchPotato.recliner',
             centre=(-0.01, -0.02), motions={'uses': {'recline': 'recline', 'doze': 'recline'}, 'ride': 'recline'},
             paint=[('plinth', 'Body'), ('plush', 'Upholstery'), ('throw', 'Throw')]),
     G.Model('snackFridge', snackFridge, size=(0.58, 0.538, 0.539), key='model.couchPotato.snackFridge',
             centre=(0, 0), motions={'uses': {'snack': 'fridge_open', 'takeout': 'fridge_open'}},
             paint=[('mint', 'Body'), ('lid', 'Upholstery')]),
     G.Model('fitnessGame', fitnessGame, size=(0.865, 1.155, 0.88), key='model.couchPotato.fitnessGame',
             centre=(0.032, 0.01), motions={'uses': {'workout': 'board'}},
             paint=[('fit_stand', 'Body'), ('fit_mat', 'Mat'), ('board', 'Board')]),
     G.Model('lavaLamp', lavaLamp, size=(0.896, 1.15, 0.894), key='model.couchPotato.lavaLamp',
             centre=(0.0019, 0.0232), motions={'uses': {'zoneOut': 'blobs'}, 'idle': 'drift'},
             paint=[('lamp_metal', 'Body'), ('pouf', 'Upholstery'), ('vessel', 'Glass')])],
    comment='Blender-built couch potato pack models (tools/art/blender/families/pack_couchPotato.py). '
            'Listed in the base manifest\'s packs. The materials map onto the game\'s finishes so a model '
            'costs one draw call per finish group. The cinema\'s curtains are drawn apart, the console\'s '
            'tray slides out, the recliner reclines, the fridge door swings, the balance board tilts and '
            'the lava lamp\'s wax rises and drifts.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
