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
# Built with the forms and the kit (README "The toolkit"): stuffed, piped and rolled upholstery (`F.soft`, swept
# arms), screens and cabinets extruded from rounded outlines, speaker drivers turned in one piece, pleated curtains
# with a heading, lofted cans, cups and glass.
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


def _paint(obj, material, test):
    """Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a stripe,
    a surround, with no more triangles."""
    obj.data.materials.append(material)
    k = len(obj.data.materials) - 1
    for p in obj.data.polygons:
        if test(p.center):
            p.material_index = k
    return obj


def _pleats(F, name, x0, x1, y0, y1, z, folds, amp, mat, t=0.01):
    """A pleated curtain across `x0`..`x1` hanging in the plane at depth `z` from its heading at `y1` to its hem at
    `y0`: its folds shallow under the heading tape, deepening as it falls; a turned-up hem thickens its foot."""
    n = folds * 4
    h = y1 - y0
    rows = [(y0, 1.0, 1.5, 0.0), (y0 + 0.012, 1.0, 1.5, 0.02), (y0 + 0.04, 1.0, 1.0, 0.05), (y0 + h * 0.45, 0.9, 1.0, 0.25),
            (y1 - 0.06, 0.7, 1.0, 0.4), (y1 - 0.05, 0.4, 1.3, 0.42), (y1, 0.36, 1.3, 0.42)]
    verts, m = [], 0
    for y, af, tf, phase in rows:
        front = [(x0 + (x1 - x0) * k / n, amp * af * math.sin(2 * math.pi * folds * k / n + phase) + t * tf / 2)
                 for k in range(n + 1)]
        ring = front + [(u, o - t * tf) for u, o in reversed(front)]
        m = len(ring)
        verts += [F.at(u, y, z + o) for u, o in ring]
    faces = []
    for r in range(len(rows) - 1):
        a, b = r * m, (r + 1) * m
        faces += [(a + i, a + (i + 1) % m, b + (i + 1) % m, b + i) for i in range(m)]
    for base in (0, (len(rows) - 1) * m):
        faces += [(base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k) for k in range(n)]
    return _obj(name, verts, faces, [mat], sharp=70)


def _turn_round(objs):
    """Turns a model built in place half a turn about the footprint's middle (each part about its own origin,
    which moves round with it), as pack_bookworm turned the writer's bureau. A part's slides then go the other way."""
    for o in objs:
        G.turn(o, 180, 'Z')
        o.location = (-o.location.x, -o.location.y, o.location.z)


def _driver(F, name, x, y, z, r, facing=-1, segments=16):
    """A speaker driver at (x, y) on a face at depth `z`, facing +z (`facing` 1) or -z: its flange, a rolled
    surround and the cone dished in to a domed dust cap, one turned piece; the flange and surround in the dark."""
    prof = [(r + 0.012, -0.008), (r + 0.012, 0.002), (r + 0.004, 0.004), (r - 0.002, 0.009), (r - 0.008, 0.004),
            (r * 0.36, -0.02), (r * 0.3, -0.016), (r * 0.15, -0.012), (0, -0.011)]
    return G.revolve(name, prof, at=F.at(x, y, z), material=M.speaker_cone, segments=segments, rot=(90 * facing, 0, 0))


def _screen(F, x0, x1, y0, y1, z, stripe):
    """A screen held between two slim posts at depth `z`, its picture facing the seats behind it (-Z), the oak back
    with slats on its other side (to the wall, once the model is turned round)."""
    w, h = x1 - x0, y1 - y0
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    F.extrude(_name('screen_back'), G.rounded_rect(w, h, 0.012, corner=1), 0.05, (cx, cy, z), M.screen_back,
              bevel=0.008, bevel_segments=1)
    F.extrude(_name('bezel'), G.rounded_rect(w - 0.03, h - 0.03, 0.01, corner=1), 0.008, (cx, cy, z - 0.028), M.bezel,
              bevel=0.002, bevel_segments=1)
    F.box(_name('picture'), w * 0.82, h * 0.7, 0.002, (cx, cy + h * 0.04, z - 0.0325), M.picture)
    F.box(_name('picture_b'), w * 0.3, h * 0.12, 0.002, (cx - w * 0.2, cy - h * 0.28, z - 0.034), M.picture_b)
    F.box(_name('stripe'), w * 0.5, 0.02, 0.006, (cx, y0 + 0.06, z + 0.027), stripe)
    n = max(3, int(w / 0.06))
    for i in range(1, n):
        F.box(_name('slat'), 0.016, h - 0.05, 0.006, (x0 + w * i / n, cy, z + 0.026), M.slat)
    for x in (x0 + 0.02, x1 - 0.02):
        F.box(_name('post'), 0.04, y1 + 0.02, 0.04, (x, (y1 + 0.02) / 2, z + 0.01), M.graphite, bevel=0.008,
              bevel_segments=1)
        F.extrude(_name('post_foot'), G.rounded_rect(0.08, 0.12, 0.02, corner=2), 0.02, (x, 0.01, z - 0.04), M.graphite,
                  plane='top', bevel=0.005, bevel_segments=1)


def _popcorn(F, x, y, z, r, h, stripe, n=7):
    """A striped popcorn bucket standing on `y`, popped corn heaped over its rim."""
    b = G.revolve(_name('bucket'), [(0, 0), (r * 0.75, 0), (r, h), (r * 0.94, h), (r * 0.72, 0.006), (0, 0.006)],
                  at=F.at(x, y, z), material=M.straw, segments=16)
    _paint(b, stripe, lambda c: math.atan2(c.y, c.x) % (math.pi / 4) < math.pi / 8)
    for k in range(n):
        a = 360 * k / n
        rr = r * 0.55 if k % 2 else r * 0.3
        F.sphere(_name('popcorn'), r * 0.32, (x + rr * G.cosd(a), y + h + 0.006 * (k % 3), z + rr * G.sind(a)), M.popcorn,
                 segments=6, rings=4)
    F.sphere(_name('popcorn'), r * 0.36, (x, y + h + 0.02, z), M.popcorn, segments=6, rings=4)


def _cup(F, x, y, z, r, h, material):
    """A soda cup standing on `y`: a tapered body, its domed lid and a straw through it."""
    G.revolve(_name('soda'), [(0, 0), (r * 0.78, 0), (r, h), (0, h)], at=F.at(x, y, z), material=material, segments=12)
    G.revolve(_name('soda_lid'), [(0, 0), (r * 1.06, 0), (r * 1.06, 0.006), (r * 0.8, 0.014), (0, 0.016)],
              at=F.at(x, y + h, z), material=M.straw, segments=12)
    G.kit.tube(F, _name('straw'), [(x + 0.006, y + h, z), (x + 0.012, y + h + 0.1, z - 0.006)], 0.0045, M.straw, verts=6,
               round=1)


# --- the grand cinema wall (2x1): the screen on the wall, twin recliners in front of it, a console between --------
# (built with the seats at the back and the screen at the front edge facing them, then turned round)
SCREEN_Z = 0.42


def _cinema_seat(F, x, w, outer):
    """One recliner of the pair centred on (x, 0), facing the screen (before the turn, in front): a stuffed, piped
    seat cushion (top 0.45), a back of three pillowed rolls leaning back, a padded arm outside, on a dark plinth."""
    F.soft(_name('seat_base'), w, 0.28, 0.62, (x, 0.2, -0.06), M.leather, round=0.06, flat=1, round_segments=1)
    F.soft(_name('seat'), w - 0.18, 0.09, 0.54, (x, 0.393, -0.01), M.leather, round=0.035, puff=0.012, flat=2)
    a = math.radians(14)
    up, back = (math.cos(a), math.sin(a)), (0.32, -0.23)
    F.soft(_name('seat_shell'), w, 0.68, 0.12, (x, back[0] + 0.33 * up[0], back[1] - 0.33 * up[1] - 0.03), M.leather,
           rot=(-14, 0, 0), round=0.05, flat=1, round_segments=1)
    for k, (t, hh) in enumerate(((0.12, 0.22), (0.33, 0.2), (0.53, 0.18))):
        F.soft(_name('seat_roll'), w - 0.16, hh, 0.13, (x, back[0] + t * up[0], back[1] - t * up[1] + 0.02), M.leather,
               rot=(-14, 0, 0), round=0.05, puff=0.025, flat=2, round_segments=1)
    F.soft(_name('arm'), 0.11, 0.3, 0.62, (outer, 0.47, -0.06), M.leather, round=0.045, flat=1, round_segments=1)
    F.soft(_name('arm_pad'), 0.13, 0.05, 0.5, (outer, 0.63, -0.03), M.leather, round=0.022, flat=1, round_segments=1)
    F.extrude(_name('seat_foot'), G.rounded_rect(w - 0.1, 0.56, 0.04, corner=2), 0.05, (x, 0.025, -0.06), M.dark,
              plane='top')


def cinemaWall():
    F = G.GameFrame()
    with G.new_parts() as still:
        for s in (-1, 1):
            _cinema_seat(F, s * 0.5, 0.78, s * 0.86)
        # the console between the seats: a padded lid, two cup holders, a soda, a bucket of popcorn
        F.extrude('console', G.rounded_rect(0.22, 0.6, 0.04, corner=2), 0.5, (0, 0.25, -0.06), M.cinema_frame,
                  plane='top', bevel=0.02, bevel_segments=1)
        F.soft('console_lid', 0.2, 0.04, 0.42, (0, 0.515, -0.14), M.leather, round=0.018, flat=1, round_segments=1)
        for z in (0.08, 0.17):
            G.torus(_name('cup_ring'), 0.045, 0.006, at=F.at(0, 0.503, z), material=M.graphite, major_segments=12,
                    minor_segments=3)
            F.cyl(_name('cup_well'), 0.042, 0.004, (0, 0.5, z), M.dark, verts=12)
        _cup(F, 0, 0.47, 0.08, 0.045, 0.16, M.soda)
        _popcorn(F, 0, 0.535, -0.18, 0.075, 0.12, M.soda)
        F.extrude('remote', G.rounded_rect(0.045, 0.16, 0.018, corner=2), 0.018, (-0.3, 0.465, 0.1), M.dark,
                  plane='top', rot=(0, 0, 20), bevel=0.004, bevel_segments=1)
        # the screen (at the front edge until turned round), facing the seats; a pelmet over it for the curtains
        _screen(F, -0.92, 0.92, 0.88, 1.42, SCREEN_Z, M.picture)
        F.extrude('pelmet', G.rounded_rect(1.6, 0.1, 0.02, corner=2), 0.1, (0, 1.47, SCREEN_Z - 0.06), M.pelmet,
                  bevel=0.012, bevel_segments=1)
        F.box('pelmet_bead', 1.56, 0.012, 0.012, (0, 1.425, SCREEN_Z - 0.11), M.graphite, bevel=0.004, bevel_segments=1)
        # the surround speakers at the screen's corners: rounded towers, two drivers and a port each, on spikes
        for x in (-0.86, 0.86):
            F.extrude(_name('speaker'), G.rounded_rect(0.16, 0.16, 0.03, corner=2), 0.8, (x, 0.44, 0.33),
                      M.speaker_grey, plane='top', bevel=0.012, bevel_segments=1)
            _driver(F, _name('woofer'), x, 0.6, 0.25, 0.05, segments=12)
            _driver(F, _name('tweeter'), x, 0.76, 0.25, 0.028, segments=12)
            F.cyl(_name('port'), 0.014, 0.01, (x, 0.3, 0.248), M.dark, rot=(90, 0, 0), verts=10)
            for dx in (-0.05, 0.05):
                for dz in (0.28, 0.38):
                    F.cone(_name('spike'), 0.012, 0.004, 0.04, (x + dx, 0.02, dz), M.graphite, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two curtains, closed over the picture, each gathered by `curtains` to its own side.
    zc = SCREEN_Z - 0.06
    parts = []
    for s, name in ((-1, 'curtain_left'), (1, 'curtain_right')):
        with G.new_parts() as cur:
            _pleats(F, name, s * 0.01, s * 0.78, 0.9, 1.42, zc - 0.012, 6, 0.018, M.curtain)
        parts.append(G.hinged(G.join(cur, name), F.at(s * 0.78, 1.42, zc - 0.012)))
    _turn_round(G.meshes())
    objs = G.settle()      # on its Model centre: the screen's back on the wall face
    for cur in parts:
        G.squash(cur, 'curtains', (0.22, 1, 1), seconds=1.2)
    return objs


# --- the Pixel Arcade console (1x1): the screen and console on the wall, a gaming loveseat for two in front -------
# (built with the loveseat at the back and the screen at the front edge facing it, then turned round)
def _pad(F, x, y, z, turn=0.0):
    """A game controller lying on `y` at (x, z), facing +z: a body with two grips, two sticks, a cross and four
    buttons."""
    body = G.smooth_outline([(-0.07, 0.03), (0.07, 0.03), (0.075, 0.0), (0.065, -0.05), (0.04, -0.055), (0.025, -0.02),
                             (-0.025, -0.02), (-0.04, -0.055), (-0.065, -0.05), (-0.075, 0.0)], 2)
    F.extrude(_name('pad'), body, 0.026, (x, y + 0.013, z), M.console, plane='top', rot=(0, 0, turn), bevel=0.006,
              bevel_segments=1)
    for dx in (-0.025, 0.025):
        F.cyl(_name('stick'), 0.009, 0.012, (x + dx, y + 0.032, z - 0.005), M.dark, verts=8)
    F.box(_name('cross'), 0.024, 0.004, 0.008, (x - 0.045, y + 0.027, z + 0.008), M.dark)
    F.box(_name('cross'), 0.008, 0.004, 0.024, (x - 0.045, y + 0.027, z + 0.008), M.dark)
    for k, (dx, dz) in enumerate(((0.045, 0.0), (0.045, 0.016), (0.037, 0.008), (0.053, 0.008))):
        F.cyl(_name('button'), 0.004, 0.005, (x + dx, y + 0.0285, z + dz), (M.soda, M.led, M.mustard, M.box_blue)[k],
              verts=6)


def gameConsole():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the loveseat: a piped base, two stuffed seat cushions, a back with two cushions, rolled arms, splayed legs
        F.soft('base', 0.88, 0.24, 0.56, (0, 0.17, -0.12), M.gamer, round=0.06, piping=0.005, flat=1)
        for x in (-0.21, 0.21):
            F.soft(_name('seat'), 0.4, 0.1, 0.5, (x, 0.388, -0.08), M.gamer, round=0.035, puff=0.012, piping=0.005, flat=2)
            F.soft(_name('back_cushion'), 0.38, 0.36, 0.13, (x, 0.6, -0.31), M.gamer, rot=(-10, 0, 0), round=0.045,
                   puff=0.025, flat=2)
        F.soft('back', 0.88, 0.52, 0.1, (0, 0.55, -0.355), M.gamer, rot=(-10, 0, 0), round=0.045, flat=1,
               round_segments=1)
        for s in (-1, 1):
            roll = [(0.48 + 0.045 * math.sin(math.radians(a)), s * 0.04 * math.cos(math.radians(a)) + s * 0.005)
                    for a in range(-40, 230, 30)]
            F.sweep(_name('arm'), [(s * 0.4, 0, -0.39), (s * 0.4, 0, 0.15)], None, M.gamer,
                    profile=[(0.27, s * -0.035), (0.27, s * 0.035)] + roll + [(0.44, s * -0.035)], angle=50)
            for lz in (-0.37, 0.13):
                F.sweep(_name('foot'), [(s * 0.37, 0.06, lz), (s * 0.385, 0.0, lz + 0.01 * (1 if lz > 0 else -1))], 0.016,
                        M.dark, verts=8, scale=[1.0, 0.7])
        F.soft('pillow', 0.3, 0.28, 0.1, (0.24, 0.64, -0.24), M.mustard, rot=(-10, 0, 20), round=0.04, puff=0.03, flat=2)
        for x, t in ((-0.2, 10), (0.2, -15)):                     # the controllers on the seats
            _pad(F, x, 0.45, 0.06, t)
        # the screen (at the front edge until turned round) and the shelf between its posts, with the console and two
        # games
        _screen(F, -0.4, 0.4, 0.85, 1.32, SCREEN_Z, M.mustard)
        F.extrude('shelf', G.rounded_rect(0.74, 0.12, 0.01, corner=1), 0.03, (0, 0.62, 0.39), M.screen_back, plane='top',
                  bevel=0.005, bevel_segments=1)
        F.extrude('console', G.rounded_rect(0.3, 0.1, 0.02, corner=2), 0.06, (0, 0.665, 0.39), M.console, plane='top',
                  bevel=0.01, bevel_segments=1)
        F.box('console_vent', 0.24, 0.004, 0.06, (0, 0.6955, 0.39), M.dark)
        F.box('tray_slot', 0.14, 0.012, 0.004, (0, 0.67, 0.339), M.dark)
        F.box('power', 0.05, 0.006, 0.003, (0.1, 0.65, 0.339), M.led)
        F.extrude('game_a', G.rounded_rect(0.14, 0.18, 0.008, corner=1), 0.02, (-0.27, 0.725, 0.43), M.box_red,
                  rot=(0, 4, 0), bevel=0.003, bevel_segments=1)
        F.extrude('game_b', G.rounded_rect(0.14, 0.18, 0.008, corner=1), 0.02, (-0.24, 0.725, 0.405), M.box_blue,
                  rot=(0, -3, 0), bevel=0.003, bevel_segments=1)
        for k, z in enumerate((0.43, 0.405)):
            F.box(_name('game_art'), 0.1, 0.1, 0.002, (-0.27 + 0.03 * k, 0.74, z - 0.011), (M.mustard, M.led)[k],
                  rot=(0, 4 - 7 * k, 0))
    G.join(still, 'body', origin=(0, 0, 0))
    # The disc tray in the console's slot, facing the players: `tray` slides it out with a disc on it.
    with G.new_parts() as tray:
        F.box('tray', 0.13, 0.008, 0.09, (0, 0.67, 0.385), M.dark, bevel=0.002, bevel_segments=1)
        F.cyl('tray_disc', 0.04, 0.003, (0, 0.6755, 0.385), M.disc, verts=24)
        F.cyl('tray_hub', 0.008, 0.004, (0, 0.677, 0.385), M.dark, verts=8)
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
        F.extrude('plinth', G.rounded_rect(0.6, 0.6, 0.06, corner=2), 0.05, (0, 0.025, -0.06), M.plinth, plane='top',
                  bevel=0.01, bevel_segments=1)
        F.soft('base', 0.58, 0.3, 0.6, (0, 0.2, -0.06), M.plush, round=0.05, flat=1, round_segments=1)
        F.soft('seat', 0.56, 0.1, 0.54, (0, 0.388, -0.02), M.plush, round=0.04, puff=0.012, piping=0.005, flat=2)
        for s in (-1, 1):
            # the arm: a panel rising into a fat roll, swept front to back; a plinth-coloured cap on the roll's front
            roll = [(0.56 + 0.085 * math.sin(math.radians(a)), s * 0.085 * math.cos(math.radians(a)) + s * 0.005)
                    for a in range(-50, 231, 28)]
            F.sweep(_name('arm'), [(s * 0.37, 0, -0.43), (s * 0.37, 0, 0.27)], None, M.plush,
                    profile=[(0.06, s * -0.08), (0.06, s * 0.08)] + roll + [(0.5, s * -0.08)], angle=55)
            F.cyl(_name('roll_end'), 0.06, 0.012, (s * 0.375, 0.56, 0.272), M.plinth, rot=(90, 0, 0), verts=14)
            F.cyl(_name('roll_button'), 0.018, 0.008, (s * 0.375, 0.56, 0.281), M.plush, rot=(90, 0, 0), verts=8)
        G.torus('cup_ring', 0.042, 0.006, at=F.at(0.37, 0.648, 0.12), material=M.dark, major_segments=12, minor_segments=3)
        G.revolve('cup', [(0, 0), (0.03, 0), (0.037, 0.12), (0.034, 0.12), (0.03, 0.11), (0, 0.11)],
                  at=F.at(0.37, 0.64, 0.12), material=M.teal, segments=12)
        F.extrude('remote', G.rounded_rect(0.05, 0.14, 0.02, corner=2), 0.02, (-0.37, 0.655, 0.1), M.dark, plane='top',
                  bevel=0.005, bevel_segments=1)
        F.box('remote_keys', 0.02, 0.004, 0.1, (-0.37, 0.666, 0.11), M.slat)
        # a throw over the left arm: across the roll and hanging down its outside, soft and creased
        t = 0.012
        pts = G.kit.fillet([(-0.29, 0.62, -0.12), (-0.36, 0.665, -0.12), (-0.45, 0.64, -0.12), (-0.474, 0.56, -0.12),
                            (-0.474, 0.3, -0.12)], 0.04)
        throw = F.sweep('throw', pts, None, M.throw, profile=G.rounded_rect(t, 0.4, t / 2, corner=1), up=(0, 0, 1))
        G.jitter(throw, 0.005, size=0.1, seed=3.0, up=0.5)
    G.join(still, 'body', origin=(0, 0, 0))
    by, bz = BACK_HINGE
    with G.new_parts() as back:                    # the back: a shell, three pillowed rolls and a head pillow
        F.soft('back', 0.6, 0.56, 0.13, (0, 0.67, -0.28), M.plush, rot=(-8, 0, 0), round=0.06, flat=1, round_segments=1)
        for k, (y, z, hh) in enumerate(((0.55, -0.2, 0.15), (0.71, -0.22, 0.16))):
            F.soft(_name('back_roll'), 0.5, hh, 0.1, (0, y, z), M.plush, rot=(-8, 0, 0), round=0.04, puff=0.022, flat=2)
        F.soft('head_pillow', 0.46, 0.13, 0.11, (0, 0.88, -0.21), M.plush, rot=(-8, 0, 0), round=0.05, puff=0.02, flat=2)
    back = G.hinged(G.join(back, 'back'), F.at(0, by, bz))
    fy, fz = FOOT_HINGE
    with G.new_parts() as foot:                    # the footrest, folded down under the seat's front
        F.soft('footrest', 0.5, 0.22, 0.08, (0, fy - 0.12, fz + 0.04), M.plush, round=0.035, flat=1)
    foot = G.hinged(G.join(foot, 'footrest'), F.at(0, fy, fz))
    objs = G.settle(centre=False)
    G.swing(back, 'recline', -11, seconds=1.2, axis='X')
    G.swing(foot, 'recline', -80, seconds=1.2, axis='X')
    return objs


# --- the sofa-side snack fridge (1x1): a mini fridge with a padded lid, a takeout screen on its door ------------
FR = (0.0, -0.02, 0.56, 0.37, 0.46)     # x, depth, width, height, depth of the cabinet


def _can(F, x, y, z, material, r=0.03, h=0.11):
    """A drinks can standing on `y`: a body necked in at the top and the foot, its lid and ring pull."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.8), (0.008, 1.0), (h - 0.012, 1.0), (h - 0.002, 0.84), (h, 0.84)]
    G.loft(_name('can'), [G.ring(r * k, r * k, bz + hh, 10, at=(bx, by)) for hh, k in rows], material=material)
    F.cyl(_name('can_top'), r * 0.8, 0.002, (x, y + h, z), M.chrome, verts=10)


def snackFridge():
    F = G.GameFrame()
    x, z, w, h, d = FR
    z0, z1 = z - d / 2, z + d / 2
    t = 0.025
    with G.new_parts() as still:
        # the cabinet, open at the front: back, sides, floor and top, a white liner inside with a shelf of snacks
        F.box('cab_back', w, h - 0.03, t, (x, 0.03 + (h - 0.03) / 2, z0 + t / 2), M.mint, bevel=0.008, bevel_segments=1)
        for s in (-1, 1):
            F.box(_name('cab_side'), t, h - 0.03, d - 0.03, (x + s * (w / 2 - t / 2), 0.03 + (h - 0.03) / 2, z - 0.015),
                  M.mint, bevel=0.008, bevel_segments=1)
        F.box('cab_floor', w, t, d - 0.03, (x, 0.03 + t / 2, z - 0.015), M.mint, bevel=0.006, bevel_segments=1)
        F.box('cab_top', w, t, d - 0.03, (x, h - t / 2, z - 0.015), M.mint, bevel=0.006, bevel_segments=1)
        F.box('liner', w - 2 * t, h - 0.03 - 2 * t, 0.004, (x, 0.03 + (h - 0.03) / 2, z0 + t + 0.002), M.inside)
        F.box('liner_floor', w - 2 * t, 0.004, d - 0.06, (x, 0.03 + t + 0.002, z - 0.03), M.inside)
        F.box('shelf', w - 2 * t, 0.012, d - 0.08, (x, 0.2, z - 0.04), M.inside)
        F.box('shelf_lip', w - 2 * t, 0.02, 0.006, (x, 0.206, z1 - 0.043), M.chrome)
        for k, (cx, m) in enumerate(((-0.17, M.can_a), (-0.1, M.can_b), (-0.03, M.can_c), (0.04, M.can_a))):
            _can(F, cx, 0.206, -0.02 + 0.03 * (k % 2), m)
        F.box('takeout_box', 0.16, 0.08, 0.14, (0.14, 0.247, -0.03), M.takeout, bevel=0.006, bevel_segments=1)
        F.box('takeout_band', 0.161, 0.02, 0.141, (0.14, 0.25, -0.03), M.menu)
        F.box('pizza', 0.4, 0.035, 0.3, (0, 0.075, -0.06), M.takeout, bevel=0.004, bevel_segments=1)
        F.box('pizza_lid_line', 0.402, 0.003, 0.302, (0, 0.085, -0.06), M.slat)
        F.box('pizza_logo', 0.12, 0.002, 0.08, (0, 0.093, -0.02), M.can_a)
        F.box('trim', w - 0.08, 0.012, 0.02, (x, h - 0.03, z1 - 0.004), M.mint_trim, bevel=0.004, bevel_segments=1)
        for fx in (x - w / 2 + 0.06, x + w / 2 - 0.06):
            for fz in (z0 + 0.06, z1 - 0.06):
                G.revolve(_name('foot'), [(0, 0), (0.018, 0), (0.022, 0.02), (0.02, 0.03), (0, 0.03)], at=F.at(fx, 0, fz),
                          material=M.mint_trim, segments=8)
        F.box('kick', 0.5, 0.02, 0.04, (x, 0.04, z1 - 0.02), M.graphite, bevel=0.006, bevel_segments=1)
        # the padded lid (the ottoman's top, 0.45), piped, and a bowl of crisps on it
        F.soft('lid', 0.58, 0.07, 0.5, (x, 0.405, z), M.lid, round=0.03, puff=0.01, piping=0.006, flat=2)
        for bx in (-0.12, 0.12):
            F.sphere(_name('lid_button'), 0.01, (bx, 0.452, z), M.lid, scale=(1, 1, 0.5), segments=6, rings=4)
        G.revolve('bowl', [(0, 0), (0.06, 0), (0.11, 0.06), (0.105, 0.062), (0.055, 0.008), (0, 0.008)],
                  at=F.at(0.13, 0.45, -0.12), material=M.bowl, segments=16)
        for k in range(8):
            a = 45 * k
            r = 0.05 if k % 2 else 0.025
            crisp = F.sphere(_name('crisp'), 0.03, (0.13 + r * G.cosd(a), 0.5 + 0.006 * (k % 3), -0.12 + r * G.sind(a)),
                             M.crisps, scale=(1, 1, 0.3), rot=(20 * (k % 3), 30 * k, 0), segments=8, rings=4)
            G.jitter(crisp, 0.004, size=0.02, seed=k * 1.3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door with its takeout screen and handle, hinged on its right edge: `fridge_open` swings it open.
    hx = x + w / 2 - 0.01
    with G.new_parts() as door:
        F.extrude('door', G.rounded_rect(w - 0.02, h - 0.06, 0.02, corner=2), 0.03, (x, 0.03 + (h - 0.03) / 2, z1 + 0.003),
                  M.mint, bevel=0.008, bevel_segments=1)
        seal = G.rounded_rect(w - 0.07, h - 0.1, 0.03, corner=2)
        F.sweep('door_seal', [(x + u, 0.03 + (h - 0.03) / 2 + v, z1 - 0.014) for u, v in seal], 0.006, M.graphite, verts=4,
                closed=True)
        F.box('door_liner', w - 0.1, h - 0.13, 0.008, (x, 0.03 + (h - 0.03) / 2, z1 - 0.014), M.inside)
        F.extrude('door_screen', G.rounded_rect(0.18, 0.12, 0.012, corner=2), 0.006, (0.08, 0.24, z1 + 0.02), M.bezel,
                  bevel=0.002, bevel_segments=1)
        F.box('door_menu', 0.15, 0.09, 0.002, (0.08, 0.24, z1 + 0.0235), M.menu)
        for k in range(3):
            F.box(_name('door_item'), 0.06, 0.012, 0.002, (0.1, 0.27 - 0.025 * k, z1 + 0.0245), M.takeout)
        F.box('door_pick', 0.04, 0.03, 0.002, (0.04, 0.26, z1 + 0.0245), M.can_a)
        G.kit.pull(F, 'handle', x - w / 2 + 0.06, 0.2, z1 + 0.018, 0.16, M.chrome, r=0.007, out=0.0345, upright=True)
    door = G.hinged(G.join(door, 'door'), F.at(hx, 0.2, z1 + 0.003))
    objs = G.settle(centre=False)
    G.swing(door, 'fridge_open', -100, seconds=0.9)
    return objs


# --- the living-room workout game (1x1): a TV on a slim stand, a step mat and a balance board before it ---------
def fitnessGame():
    F = G.GameFrame()
    with G.new_parts() as still:
        for y in (0.2, 0.56):
            F.extrude(_name('stand_shelf'), G.rounded_rect(0.8, 0.3, 0.02, corner=2), 0.04, (0, y, -0.28), M.oak,
                      plane='top', bevel=0.008, bevel_segments=1)
        for x in (-0.37, 0.37):
            F.extrude(_name('stand_side'), G.rounded_rect(0.28, 0.58, 0.03, corner=2), 0.04, (x, 0.29, -0.28),
                      M.fit_stand, plane='side', bevel=0.008, bevel_segments=1)
        F.box('stand_back', 0.7, 0.3, 0.012, (0, 0.38, -0.425), M.fit_stand)
        # the TV on its foot, a workout on the screen, the sensor bar on top
        y0, w, h = 0.58, 0.8, 0.53
        ph = h - 0.07
        py = y0 + 0.07 + ph / 2
        zc = -0.35
        F.extrude('tv_foot', G.rounded_rect(0.3, 0.14, 0.03, corner=2), 0.014, (0, y0 + 0.007, zc), M.tv_frame,
                  plane='top', bevel=0.004, bevel_segments=1)
        F.sweep('tv_neck', [(0, y0 + 0.01, zc - 0.03), (0, y0 + 0.08, zc - 0.03)], None, M.tv_frame,
                profile=G.rounded_rect(0.06, 0.03, 0.01, corner=1), up=(1, 0, 0))
        F.extrude('tv', G.rounded_rect(w, ph, 0.015, corner=2), 0.04, (0, py, zc - 0.01), M.tv_frame, bevel=0.008,
                  bevel_segments=1)
        F.box('tv_screen', w - 0.03, ph - 0.03, 0.004, (0, py + 0.003, zc + 0.011), M.tv_screen)
        F.box('tv_band', w - 0.03, 0.1, 0.003, (0, py - ph / 2 + 0.07, zc + 0.0135), M.tv_floor)
        # the coach on the screen: a head, a body and arms raised
        F.cyl('coach_head', 0.03, 0.003, (-0.12, py + 0.06, zc + 0.0145), M.tv_white, rot=(90, 0, 0), verts=12)
        F.extrude('coach', [(-0.03, -0.09), (0.03, -0.09), (0.03, 0.0), (0.07, 0.07), (0.055, 0.08), (0.02, 0.03),
                            (-0.02, 0.03), (-0.055, 0.08), (-0.07, 0.07), (-0.03, 0.0)], 0.003, (-0.12, py - 0.04, zc + 0.0145),
                  M.tv_orange)
        F.box('score_a', 0.2, 0.04, 0.003, (0.2, py + 0.14, zc + 0.014), M.tv_white)
        F.box('score_b', 0.14, 0.03, 0.003, (0.17, py + 0.09, zc + 0.014), M.picture_b)
        F.extrude('sensor', G.rounded_rect(0.3, 0.04, 0.02, corner=3), 0.05, (0, y0 + h + 0.025, zc - 0.005), M.tv_white,
                  bevel=0.008, bevel_segments=1)
        F.box('sensor_glass', 0.2, 0.02, 0.004, (0, y0 + h + 0.025, zc + 0.021), M.bezel)
        F.sphere('sensor_eye', 0.012, (-0.08, y0 + h + 0.025, zc + 0.022), M.soda, scale=(1, 1, 0.4), segments=8, rings=4)
        # on the shelves: a folded towel and a water bottle; on the floor: the step mat and a kettlebell
        F.soft('towel', 0.3, 0.03, 0.2, (-0.2, 0.235, -0.28), M.towel, round=0.012, puff=0.004, flat=1)
        F.soft('towel_top', 0.3, 0.022, 0.18, (-0.2, 0.26, -0.29), M.towel, round=0.01, puff=0.004, flat=1)
        bx, by, bz = F.at(0.25, 0.22, -0.26)
        G.loft('bottle', [G.ring(r, r, bz + hh, 12, at=(bx, by)) for hh, r in ((0, 0.03), (0.004, 0.035), (0.08, 0.032),
                                                                                (0.15, 0.035), (0.17, 0.03), (0.19, 0.02))],
               material=M.bottle)
        G.revolve('bottle_cap', [(0, 0), (0.022, 0), (0.022, 0.025), (0.012, 0.035), (0.012, 0.05), (0, 0.05)],
                  at=(bx, by, bz + 0.19), material=M.tv_white, segments=10)
        F.soft('mat', 0.7, 0.012, 0.56, (0, 0.006, 0.17), M.fit_mat, round=0.006, flat=1, round_segments=1)
        F.box('mat_inner', 0.62, 0.004, 0.48, (0, 0.0135, 0.17), M.mat_inner, bevel=0.002, bevel_segments=1)
        # the step pads printed round its edge: an arrow at each side, pointing out
        arrow = [(0.0, 0.05), (0.045, 0.0), (0.018, 0.0), (0.018, -0.04), (-0.018, -0.04), (-0.018, 0.0), (-0.045, 0.0)]
        for k, (ax, az, a) in enumerate(((0.0, 0.37, 0), (0.0, -0.03, 180), (0.24, 0.17, -90), (-0.24, 0.17, 90))):
            c, s = G.cosd(a), G.sind(a)
            F.extrude(_name('arrow'), [(ax + u * c - v * s, az - (u * s + v * c)) for u, v in arrow], 0.003,
                      (0, 0.0165, 0), (M.tv_orange, M.board)[k % 2], plane='top')
        # a pair of dumbbells on the bottom shelf
        for dz in (-0.33, -0.23):
            F.sweep(_name('dumbbell_bar'), [(-0.02, 0.25, dz), (0.16, 0.25, dz)], 0.011, M.grip, verts=8)
            for dx in (-0.01, 0.15):
                F.cyl(_name('dumbbell_end'), 0.03, 0.04, (dx, 0.25, dz), M.weight, rot=(0, 90, 0), verts=8, bevel=0.006,
                      bevel_segments=1)
        # the kettlebell: a round bell, its flat foot, a handle arching over it
        kx, kz = 0.38, 0.36
        bx, by, bz = F.at(kx, 0, kz)
        G.loft('kettlebell', [G.ring(r, r, bz + hh, 14, at=(bx, by)) for hh, r in ((0.0, 0.05), (0.01, 0.065), (0.04, 0.085),
                                                                                    (0.08, 0.085), (0.11, 0.07), (0.13, 0.04),
                                                                                    (0.136, 0.02))], material=M.weight)
        G.kit.tube(F, 'kettle_handle', G.kit.fillet([(kx - 0.05, 0.11, kz), (kx - 0.055, 0.19, kz), (kx + 0.055, 0.19, kz),
                                                     (kx + 0.05, 0.11, kz)], 0.035), 0.012, M.weight, verts=8, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The balance board on its rocker, on the mat: `board` tilts it 10° to one side.
    with G.new_parts() as board:
        F.extrude('board_deck', G.rounded_rect(0.52, 0.4, 0.18, corner=6), 0.025, (0, 0.1, 0.2), M.board, plane='top',
                  bevel=0.008, bevel_segments=1)
        for dx in (-0.14, 0.14):
            F.extrude(_name('board_grip'), G.rounded_rect(0.12, 0.16, 0.03, corner=2), 0.004, (dx, 0.1135, 0.2), M.grip,
                      plane='top')
        F.cyl('board_rocker', 0.045, 0.06, (0, 0.057, 0.2), M.grip, rot=(90, 0, 0), verts=16)
        F.box('board_mount', 0.08, 0.02, 0.07, (0, 0.082, 0.2), M.grip)
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


def _blob(F, name, x, y, z, r, seed, stretch=1.3):
    """A blob of wax: a round drop stretched upright, wobbled."""
    b = F.sphere(name, r, (x, y, z), M.wax, scale=(1, 1, stretch), segments=10, rings=6)
    return G.jitter(b, r * 0.12, size=r * 1.2, seed=seed)


def lavaLamp():
    F = G.GameFrame()
    tx, tz = LAMP
    y0, h = VESSEL
    with G.new_parts() as still:
        px, pz = POUF
        fx, fz = G.sind(TURN), G.cosd(TURN)                     # the way it faces (game x, depth)
        F.soft('pouf', 0.62, 0.36, 0.6, (px, 0.18, pz), M.pouf, rot=(0, 0, -TURN), round=0.13, flat=2)
        F.soft('pouf_top', 0.54, 0.1, 0.52, (px, 0.385, pz), M.pouf, rot=(0, 0, -TURN), round=0.045, puff=0.015,
               piping=0.007, flat=2)
        F.sphere('pouf_button', 0.016, (px, 0.452, pz), M.pouf, scale=(1, 1, 0.5), segments=8, rings=4)
        F.soft('lean', 0.5, 0.36, 0.14, (px - 0.26 * fx, 0.6, pz - 0.26 * fz), M.pouf, rot=(-12, 0, -TURN), round=0.06,
               puff=0.012, flat=2)
        F.cyl('rug', 0.2, 0.012, (-0.22, 0.006, 0.27), M.rug, verts=24, bevel=0.004, bevel_segments=1)
        # the side table: a turned stem on a round foot, a lipped oak top
        F.cyl('table_top', 0.13, 0.03, (tx, 0.5, tz), M.oak, verts=20, bevel=0.008, bevel_segments=1)
        G.revolve('table_stem', [(0, 0), (0.1, 0), (0.1, 0.012), (0.08, 0.02), (0.022, 0.035), (0.018, 0.2), (0.024, 0.3),
                                 (0.018, 0.46), (0.04, 0.485), (0, 0.485)], at=F.at(tx, 0, tz), material=M.table_white,
                  segments=14)
        # the lamp: a tapered metal foot, the glass, a cone cap; a pool of wax at the bottom
        G.revolve('lamp_foot', [(0, 0), (0.07, 0), (0.072, 0.008), (0.05, 0.12), (0.052, 0.13), (0.045, 0.14), (0, 0.14)],
                  at=F.at(tx, 0.515, tz), material=M.lamp_metal, segments=16)
        prof = [(0.0, 0.0)] + [(_vessel_r(y0 + h * k / 12) + 0.003, h * k / 12) for k in range(13)] + [(0.0, h)]
        G.revolve('vessel', prof, at=F.at(tx, y0, tz), material=M.vessel, segments=20)
        G.revolve('lamp_cap', [(0, 0), (0.046, 0), (0.044, 0.01), (0.02, 0.08), (0.014, 0.085), (0, 0.085)],
                  at=F.at(tx, y0 + h, tz), material=M.lamp_metal, segments=14)
        _blob(F, 'wax_pool', tx, y0 + 0.02, tz, 0.045, 1.0, stretch=0.35)
    G.join(still, 'body', origin=(0, 0, 0))
    # The wax that rises while a resident zones out (`blobs`, slow) ...
    with G.new_parts() as rise:
        for k, (dx, y, r) in enumerate(((0.0, 0.7, 0.03), (0.012, 0.79, 0.024), (-0.016, 0.745, 0.018))):
            _blob(F, _name('blob'), tx + dx, y, tz, r, 2.0 + k)
    rise = G.hinged(G.join(rise, 'blobs'), F.at(tx, y0, tz))
    # ... and the wax that drifts round all the time near the camera (`drift`, ambient)
    with G.new_parts() as drift:
        for k, (a, y, r) in enumerate(((30, 0.9, 0.022), (200, 0.98, 0.017))):
            _blob(F, _name('drift'), tx + 0.022 * G.cosd(a), y, tz + 0.022 * G.sind(a), r, 7.0 + k)
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
