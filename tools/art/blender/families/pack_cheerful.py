# The cheerful pack (P2): the sunburst jukebox, the spotlight karaoke machine, the fizz & sunshine mocktail cart,
# the rainbow hammock, the sunrise stretch lamp and the confetti party booth.
# Notes: pack_cheerful.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the jukebox's record swings up into the window (`record`), the microphone
# rises on its stand (`mic_up`), the cocktail shaker is taken off the cart (`shake`), the hammock sways with its sleeper
# (`sway`, a loop they ride), the sun rises in the lamp's frame (`sunrise`; its sky glows at night) and the
# booth's side curtain is drawn aside (`curtain`).
#
# Built with the forms and the kit (README "The toolkit"): bodies extruded from rounded outlines and bevelled,
# chrome and brass bent in one sweep, turned and lofted glassware, stuffed and channelled upholstery (`F.soft`,
# swept channels), pleated curtains with a heading and a hem.
#
# Run:  cargo art pack_cheerful [--only jukebox] [--preview]
import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # the jukebox
    cherry=('#D7263D', 'gloss'),       # the cabinet; its main part
    amber=('#F6A609', 'gloss', {'glow': '#FFB733'}),   # the arch band and the bubble tubes: lit at night
    sunny=('#FFE29A', 'gloss', {'glow': '#FFE7A8'}),   # the inner arch band, lit
    window=('#5B3A8C', 'gloss'),       # the record window's back
    grille=('#E9D8B0', 'fabric'),      # the speaker cloth
    cards=('#FFF2C8', 'gloss'),        # the song cards
    vinyl=('#2B2433', 'gloss'),
    label=('#F6C744', 'gloss'),
    coral=('#F28D8D', 'gloss'),
    mint=('#9ED8C6', 'gloss'),         # the buttons; the mocktail cart's shelves (its main part)
    sky=('#6EC1E4', 'gloss'),
    chrome=('#D0D3D6', 'chrome'),
    brass=('#C9A54A', 'metal'),
    # the karaoke machine
    speaker=('#8A6BC0', 'gloss'),      # the speaker cabinet; its main part
    speaker_trim=('#54456E', 'satin'),
    cone=('#3B2F4E', 'satin'),
    steel=('#C9CCD0', 'metal'),
    dark=('#2B2433', 'satin'),         # the screen's case, the mic, the tablet
    lyrics=('#3B2A6B', 'gloss', {'glow': '#8E7BFF', 'screen': True}),   # the lyrics screen: lit while it's sung to
    white=('#F4F4F2', 'gloss'),
    stand=('#3A3D41', 'satin'),        # stands and poles
    lens=('#FFF4B0', 'gloss'),
    stage=('#6A3FA0', 'fabric'),       # the round stage rug
    star=('#F6C744', 'satin'),
    # the mocktail cart
    canopy=('#F28D8D', 'fabric'),
    canopy_white=('#F4F4F2', 'fabric'),
    tyre=('#F2F2F0', 'satin'),
    porcelain=('#F6F3EA', 'gloss'),
    juice=('#FF9A3C', 'gloss'),
    ice=('#E6F7FF', 'gloss'),
    tumbler=('#E6F0F2', 'gloss'),      # the glasses' rims, opaque (no glass draw)
    pineapple=('#E8B33C', 'satin'),
    leaf=('#4F9A3E', 'satin'),
    lemon=('#F2E05C', 'gloss'),
    lime=('#8CC63F', 'gloss'),
    orange=('#F28C28', 'gloss'),
    drink_pink=('#F07AA0', 'gloss'),
    drink_sun=('#F6C744', 'gloss'),
    drink_teal=('#7FD4C1', 'gloss'),
    fridge=('#F2F2F0', 'satin'),
    # the hammock
    wood=('#C9905A', 'wood'),          # the stand
    spreader=('#B07A4A', 'satin'),     # the spreader bars
    rope=('#EDE4D3', 'matte'),
    stripe_red=('#E4572E', 'fabric'),
    stripe_orange=('#F28C28', 'fabric'),
    stripe_yellow=('#F6C744', 'fabric'),
    stripe_green=('#8CC63F', 'fabric'),
    stripe_blue=('#3FA7D6', 'fabric'),
    stripe_violet=('#7B5EA7', 'fabric'),
    pillow=('#F4F4F2', 'satin'),
    # the sunrise lamp
    housing=('#F4F1EA', 'gloss'),      # the disc's case and the base; its main part
    lamp_metal=('#E9DCC7', 'metal'),   # the ring and the stem
    dawn=('#FFCF9E', 'satin', {'glow': '#FFB070'}),   # the sky behind the sun: the lamp glows at night
    hills=('#F28C5A', 'satin'),
    hills_far=('#F6A97A', 'satin'),
    sun=('#FFE08A', 'satin'),          # plain: a glow on it shows through the hills while it is set
    yoga=('#F29E8E', 'matte'),
    yoga_roll=('#E9876F', 'matte'),
    # the party booth
    booth=('#F28D8D', 'fabric'),       # the upholstery; its main part
    piping=('#F6C744', 'satin'),
    panel=('#7A5FB8', 'satin'),
    curtain=('#6EC1E4', 'fabric'),
    tablet_face=('#E84F9A', 'gloss'),
    popcorn=('#FFF3C4', 'matte'),
    bulb_a=('#F6C744', 'gloss'),
    bulb_b=('#6EC1E4', 'gloss'),
    bulb_c=('#F28D8D', 'gloss'),
    bulb_d=('#8CC63F', 'gloss'),
    bulb_e=('#B8A1E3', 'gloss'),
    bulb_f=('#F28C28', 'gloss'),
    # added with the detail pass
    cloud=('#FFF1E2', 'satin'),        # the sunrise scene's clouds
    cork=('#C9A27A', 'matte'),         # the yoga block
)

BULBS = ('bulb_a', 'bulb_b', 'bulb_c', 'bulb_d', 'bulb_e', 'bulb_f')   # by name: made anew after each reset
STRIPES = ('stripe_red', 'stripe_orange', 'stripe_yellow', 'stripe_green', 'stripe_blue', 'stripe_violet')
WALL = -0.43    # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93   # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


# --- meshes from points ------------------------------------------------------------------------------------------
def _obj(name, verts, faces, mats, face_mats=None, sharp=35, bevel=0.0):
    """A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in mats:
        mesh.materials.append(m)
    if face_mats:
        for p, i in zip(mesh.polygons, face_mats):
            p.material_index = i
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.set_sharp_from_angle(angle=math.radians(sharp))
    if bevel:
        m = obj.modifiers.new('bevel', 'BEVEL')
        m.width, m.segments, m.limit_method, m.angle_limit = bevel, 2, 'ANGLE', math.radians(40)
    return obj


def _slab(F, name, pts, z0, z1, mat, bevel=0.0):
    """A flat shape: the outline `pts` [(x, y)] (game, in order round it) between depths `z0` and `z1`."""
    n = len(pts)
    verts = [F.at(x, y, z1) for x, y in pts] + [F.at(x, y, z0) for x, y in pts]
    faces = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return _obj(name, verts, faces, [mat], bevel=bevel)


def _tomb(hw, yb, yc, n=18):
    """An arch-topped outline: straight sides `hw` either side of x = 0 from `yb` up to `yc`, a half circle over."""
    return [(-hw, yb), (hw, yb)] + [(hw * math.cos(math.pi * k / n), yc + hw * math.sin(math.pi * k / n))
                                    for k in range(n + 1)]


def _paint(obj, material, test):
    """Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a
    surround, a label, with no more triangles."""
    obj.data.materials.append(material)
    k = len(obj.data.materials) - 1
    for p in obj.data.polygons:
        if test(p.center):
            p.material_index = k
    return obj


def _aim(obj, direction):
    """Turns a part built along Blender +Z (a revolve) to point along `direction` (Blender)."""
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = Vector(direction).to_track_quat('Z', 'Y')
    return obj


def _around(F, name, w, d, r, y, z_back, front, profile, mat, corner=3):
    """A band swept round a rounded block's front and sides (not its back, on the wall): the block `w` wide, from
    depth `z_back` to `front`, its upright corners rounded by `r`, at height `y`. `profile` is the band's
    cross-section (u up, v out)."""
    zc = (z_back + front) / 2
    loop = G.rounded_rect(w, front - z_back, r, corner=corner)
    pts = [(x, y, zc + z) for x, z in loop]
    # from the back of the right side round the front to the back of the left side
    k = max(range(len(pts)), key=lambda i: (pts[i][2] < z_back + 0.02 and pts[i][0] > 0, -pts[i][2]))
    pts = pts[k:] + pts[:k]
    pts = [p for p in pts if p[2] > z_back + 0.015]
    pts = [(pts[0][0], y, z_back + 0.005)] + pts + [(pts[-1][0], y, z_back + 0.005)]
    return F.sweep(name, pts, None, mat, profile=profile, up=(0, 0, 1))


def _bulb(F, x, y, z, k, r=0.028):
    F.sphere(_name('bulb'), r, (x, y, z), getattr(M, BULBS[k % len(BULBS)]), segments=6, rings=4)


def _pleats(F, name, x, y0, y1, z0, z1, folds, amp, mat, t=0.01, plane='x'):
    """A pleated curtain hanging from `y1` (its heading) down to `y0` (its hem), along `z0`..`z1` in the plane at
    `x` (or along x, `z0`..`z1` being x, in the plane at depth `x`: plane 'z'). Its folds are shallow under the
    heading tape, deepen as it falls and drift a little; a turned-up hem thickens its foot."""
    n = folds * 4
    h = y1 - y0
    rows = [(y0, 1.0, 1.5, 0.0), (y0 + 0.012, 1.0, 1.5, 0.02), (y0 + 0.04, 1.0, 1.0, 0.05), (y0 + h * 0.4, 0.92, 1.0, 0.25),
            (y1 - 0.07, 0.72, 1.0, 0.4), (y1 - 0.06, 0.4, 1.3, 0.42), (y1, 0.36, 1.3, 0.42)]
    verts, m = [], 0
    for y, af, tf, phase in rows:
        front = [(z0 + (z1 - z0) * k / n, amp * af * math.sin(2 * math.pi * folds * k / n + phase) + t * tf / 2)
                 for k in range(n + 1)]
        ring = front + [(u, o - t * tf) for u, o in reversed(front)]
        m = len(ring)
        verts += [F.at(x + o, y, u) if plane == 'x' else F.at(u, y, x + o) for u, o in ring]
    faces = []
    for r in range(len(rows) - 1):
        a, b = r * m, (r + 1) * m
        faces += [(a + i, a + (i + 1) % m, b + (i + 1) % m, b + i) for i in range(m)]
    for base in (0, (len(rows) - 1) * m):   # the top and bottom edges across the cloth's thickness
        faces += [(base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k) for k in range(n)]
    return _obj(name, verts, faces, [mat], sharp=70)


def _tumbler(F, x, z, drink, y=None, h=0.11, r=0.034, straw=None, slice_=None):
    """A tumbler full of `drink` (opaque: no glass draw): a thick clear foot, the drink, a clear rim, a bendy straw
    and (`slice_`, a material) a citrus wheel on the rim."""
    y = TOP if y is None else y
    F.cyl(_name('tumbler_foot'), r * 0.85, 0.012, (x, y + 0.006, z), M.tumbler, verts=12)
    G.revolve(_name('tumbler'), [(0, 0), (r * 0.86, 0), (r * 0.98, h - 0.012), (0, h - 0.012)],
              at=F.at(x, y + 0.012, z), material=drink, segments=12)
    G.torus(_name('rim'), r * 0.98, 0.004, at=F.at(x, y + h, z), material=M.tumbler, major_segments=12, minor_segments=3)
    G.kit.tube(F, _name('straw'), G.kit.fillet([(x + 0.008, y + 0.03, z), (x + 0.018, y + h + 0.04, z - 0.006),
                                                (x + 0.045, y + h + 0.06, z - 0.01)], 0.01, steps=2),
               0.0035, straw or M.coral, verts=5, round=1)
    if slice_:
        F.cyl(_name('slice'), 0.024, 0.006, (x - r * 0.9, y + h - 0.004, z), slice_, rot=(0, 0, 90), verts=12)


# --- the sunburst jukebox (1x1): an arch-topped cabinet, a record window, bubble tubes either side -------------
ARCH = 1.08     # the height of the arch's centre


def jukebox():
    F = G.GameFrame()
    zf = WALL + 0.46      # the cabinet's front
    zc = WALL + 0.23
    with G.new_parts() as still:
        F.extrude('plinth', G.rounded_rect(0.74, 0.48, 0.04, corner=2), 0.08, (0, 0.04, WALL + 0.24), M.chrome,
                  plane='top', bevel=0.012, bevel_segments=2)
        F.extrude('cabinet', G.rounded_rect(0.7, 0.46, 0.05, corner=3), 0.8, (0, 0.48, zc), M.cherry, plane='top',
                  bevel=0.02, bevel_segments=2)
        _around(F, 'belt', 0.712, 0.47, 0.055, 0.865, WALL, zf + 0.006, G.rounded_rect(0.03, 0.012, 0.005, corner=1),
                M.chrome)
        # the arched top round the record window, its bands stepping out, a sunburst on the window's back
        F.extrude('arch', _tomb(0.35, 0.88, ARCH), 0.46, (0, 0, zc), M.cherry, holes=[_tomb(0.2, 0.9, ARCH)],
                  bevel=0.02, bevel_segments=2)
        F.extrude('arch_band', _tomb(0.3, 0.885, ARCH), 0.022, (0, 0, zf + 0.011), M.amber,
                  holes=[_tomb(0.25, 0.895, ARCH)], bevel=0.005, bevel_segments=1)
        F.extrude('arch_inner', _tomb(0.25, 0.895, ARCH), 0.012, (0, 0, zf + 0.006), M.sunny,
                  holes=[_tomb(0.2, 0.9, ARCH)])
        _slab(F, 'window', _tomb(0.2, 0.9, ARCH), WALL, WALL + 0.04, M.window)
        for k in range(9):                                          # rays from behind the record
            a = math.radians(10 + 20 * k)
            F.box(_name('ray'), 0.012, 0.2, 0.004, (0.14 * math.cos(a), 0.92 + 0.14 * math.sin(a), WALL + 0.042),
                  M.sunny, rot=(0, math.degrees(a) - 90, 0))
        for k in range(3):                                          # the record stack in the window's foot
            F.cyl(_name('stack'), 0.13, 0.008, (0, 0.906 + 0.009 * k, -0.17), M.vinyl, verts=24)
        F.cyl('stack_label', 0.04, 0.03, (0, 0.92, -0.17), M.label, verts=12)
        F.cyl('spindle', 0.006, 0.05, (0, 0.94, -0.17), M.chrome, verts=6)
        # the song cards in a chrome frame, two rows of titles; a row of push buttons under it
        F.extrude('cards_frame', G.rounded_rect(0.48, 0.12, 0.012, corner=2), 0.016, (0, 0.71, zf + 0.008), M.chrome,
                  holes=[G.rounded_rect(0.44, 0.09, 0.006, corner=1)], bevel=0.003, bevel_segments=1)
        F.box('cards', 0.44, 0.09, 0.01, (0, 0.71, zf + 0.004), M.cards)
        for r, y in enumerate((0.73, 0.69)):
            for k in range(5):
                F.box(_name('title'), 0.06, 0.008, 0.002, (-0.17 + 0.085 * k, y, zf + 0.01), (M.coral, M.window)[(k + r) % 2])
        for k, (x, m) in enumerate(((-0.15, M.label), (-0.09, M.coral), (-0.03, M.mint), (0.03, M.sky), (0.09, M.label),
                                    (0.15, M.coral))):
            G.revolve(_name('button'), [(0, 0), (0.016, 0), (0.016, 0.01), (0.012, 0.018), (0, 0.02)],
                      at=F.at(x, 0.625, zf), material=m, segments=10, rot=(90, 0, 0))
            G.torus(_name('button_ring'), 0.018, 0.003, at=F.at(x, 0.625, zf + 0.002), material=M.chrome, rot=(90, 0, 0),
                    major_segments=10, minor_segments=3)
        # the speaker: cloth in a moulded brass frame, chrome bars bursting out from its foot
        G.kit.moulding(F, 'grille_frame', [(-0.26, 0.24), (0.26, 0.24), (0.26, 0.56), (-0.26, 0.56)],
                       [(0.0, 0.0), (0.0, 0.01), (0.008, 0.016), (0.02, 0.016), (0.026, 0.008), (0.026, 0.0)], M.brass,
                       at=zf)
        F.box('grille', 0.48, 0.28, 0.012, (0, 0.4, zf + 0.004), M.grille)
        for k in range(7):
            a = math.radians(30 + 20 * k)
            c, s = math.cos(a), math.sin(a)
            reach = min((0.235 / abs(c)) if abs(c) > 1e-3 else 9, 0.29 / s)
            F.sweep(_name('burst'), [(0.03 * c, 0.255 + 0.03 * s, zf + 0.014), (reach * c, 0.255 + reach * s, zf + 0.014)],
                    None, M.chrome, profile=G.rounded_rect(0.008, 0.01, 0.003, corner=1), up=(0, 0, 1))
        F.cyl('burst_hub', 0.04, 0.012, (0, 0.255, zf + 0.012), M.chrome, rot=(90, 0, 0), verts=12)
        # the bubble tubes at the front corners: turned chrome feet and caps, collars, bubbles rising in them
        for s in (-1, 1):
            x, z = s * 0.33, zf - 0.01
            G.revolve(_name('tube'), [(0, 0), (0.035, 0), (0.035, 0.86), (0, 0.86)], at=F.at(x, 0.12, z),
                      material=M.amber, segments=14)
            G.revolve(_name('tube_foot'), [(0, 0), (0.045, 0), (0.046, 0.02), (0.04, 0.035), (0.042, 0.05), (0, 0.05)],
                      at=F.at(x, 0.095, z), material=M.chrome, segments=14)
            G.revolve(_name('tube_cap'), [(0, 0), (0.042, 0), (0.046, 0.016), (0.04, 0.03), (0.024, 0.046),
                                          (0.012, 0.062), (0.016, 0.072), (0, 0.08)], at=F.at(x, 0.97, z),
                      material=M.chrome, segments=14)
            for y in (0.4, 0.68):
                G.torus(_name('tube_collar'), 0.037, 0.006, at=F.at(x, y, z), material=M.chrome, major_segments=14,
                        minor_segments=3)
            for k in range(5):                                       # bubbles rising in the tubes
                F.sphere(_name('bubble'), 0.012, (x, 0.25 + 0.15 * k + 0.04 * (k % 2), z + 0.032), M.sunny, segments=6,
                         rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The record on top of the stack, lying flat, hinged on its front edge: `record` swings it up into the window.
    zr = -0.17 + 0.13
    with G.new_parts() as rec:
        F.cyl('record', 0.13, 0.008, (0, 0.937, -0.17), M.vinyl, verts=32)
        for r in (0.07, 0.1):
            G.torus(_name('record_groove'), r, 0.003, at=F.at(0, 0.941, -0.17), material=M.vinyl, major_segments=24,
                    minor_segments=3)
    rec = G.hinged(G.join(rec, 'record'), F.at(0, 0.937, zr))
    # Its label, a part inside it: `spin` turns it (and a coral stripe across it, to see it turn) while the record plays.
    with G.new_parts() as lab:
        F.cyl('record_label', 0.045, 0.01, (0, 0.937, -0.17), M.label, verts=16)
        F.box('label_stripe', 0.012, 0.0125, 0.07, (0, 0.937, -0.17), M.coral)
    lab = G.hinged(G.join(lab, 'record_label'), F.at(0, 0.937, -0.17))
    objs = G.settle(centre=False)
    G.attach(lab, rec)
    G.swing(rec, 'record', 90, seconds=0.8, axis='X')
    G.spin(lab, 'spin', turns=-1, seconds=1.8, axis='Z')   # 33 1/3 a minute
    return objs


# --- the spotlight karaoke machine (1x1): a party speaker with a lyrics screen, a mic stand and a spotlight -----
def _tilt(c, ang, dy, dz):
    """A point `dy` up and `dz` out from `c` (game) on a face leaning back by `ang` degrees."""
    a = math.radians(ang)
    return (c[0], c[1] + dy * math.cos(a) + dz * math.sin(a), c[2] - dy * math.sin(a) + dz * math.cos(a))


MIC = (0.18, 0.2)     # the mic stand's foot (x, depth)


def _driver(F, name, y, z, r, segments=24):
    """A speaker driver facing the front at height `y`, its flange on the face at depth `z`: a flange, a rolled
    rubber surround, the cone dished in and a domed dust cap, one turned piece (the surround in the trim colour)."""
    prof = [(r + 0.02, -0.01), (r + 0.02, 0.003), (r + 0.008, 0.005), (r + 0.002, 0.012), (r - 0.008, 0.016),
            (r - 0.017, 0.011), (r - 0.022, 0.003), (r * 0.36, -0.04), (r * 0.33, -0.036), (r * 0.24, -0.028),
            (r * 0.12, -0.023), (0, -0.022)]
    d = G.revolve(name, prof, at=F.at(0, y, z), material=M.cone, segments=segments, rot=(90, 0, 0))
    return _paint(d, M.speaker_trim, lambda c: math.hypot(c.x, c.y) > r - 0.024)


def karaokeMachine():
    F = G.GameFrame()
    w, h, d = 0.5, 0.86, 0.415
    zf = WALL + d
    with G.new_parts() as still:
        F.extrude('cabinet', G.rounded_rect(w, d, 0.05, corner=3), h - 0.03, (0, 0.03 + (h - 0.03) / 2, WALL + d / 2),
                  M.speaker, plane='top', bevel=0.03, bevel_segments=2)
        _around(F, 'band', w + 0.006, d, 0.053, h - 0.12, WALL, zf + 0.003, G.rounded_rect(0.04, 0.008, 0.003, corner=1),
                M.speaker_trim)
        for fx in (-w / 2 + 0.06, w / 2 - 0.06):
            for fz in (WALL + 0.06, zf - 0.06):
                G.revolve(_name('foot'), [(0, 0), (0.022, 0), (0.026, 0.02), (0.022, 0.03), (0, 0.03)],
                          at=F.at(fx, 0, fz), material=M.speaker_trim, segments=10)
        _driver(F, 'woofer', 0.36, zf, 0.17)
        _driver(F, 'tweeter', 0.66, zf, 0.075, segments=16)
        F.extrude('port', G.rounded_rect(0.22, 0.04, 0.02, corner=3), 0.012, (0, 0.11, zf + 0.002), M.cone,
                  holes=[G.rounded_rect(0.2, 0.024, 0.012, corner=3)])
        F.box('port_hole', 0.2, 0.024, 0.004, (0, 0.11, zf - 0.002), M.dark)
        F.box('light_strip', 0.3, 0.012, 0.006, (0, 0.15, zf + 0.004), M.label, bevel=0.003, bevel_segments=1)
        for k in range(3):                                           # the knobs and the volume ring on the top
            F.cyl(_name('top_knob'), 0.014, 0.014, (-0.15 + 0.05 * k, h + 0.007, WALL + 0.36), M.steel, verts=10)
        for s in (-1, 1):                                            # a carrying handle in each side
            G.kit.tube(F, _name('handle'), G.kit.fillet([(s * (w / 2 - 0.004), h - 0.25, WALL + 0.14),
                                                         (s * (w / 2 + 0.014), h - 0.25, WALL + 0.15),
                                                         (s * (w / 2 + 0.014), h - 0.25, WALL + 0.27),
                                                         (s * (w / 2 - 0.004), h - 0.25, WALL + 0.28)], 0.01),
                       0.007, M.speaker_trim, verts=6, round=1)
        # the lyrics screen in a rounded case on a hinged neck, leaning back
        sc = (0, 1.08, -0.2)
        F.sweep('neck', [(0, 0.86, -0.22), (0, 0.92, -0.215), (0, 0.96, -0.21)], None, M.dark,
                profile=G.rounded_rect(0.06, 0.035, 0.012, corner=2), up=(1, 0, 0))
        F.cyl('neck_hinge', 0.016, 0.08, (0, 0.95, -0.21), M.dark, rot=(0, 90, 0), verts=10)
        F.extrude('screen', G.rounded_rect(0.46, 0.34, 0.025, corner=3), 0.04, sc, M.dark, rot=(-10, 0, 0), bevel=0.01,
                  bevel_segments=1)
        F.box('screen_face', 0.42, 0.29, 0.004, _tilt(sc, 10, 0, 0.021), M.lyrics, rot=(-10, 0, 0))
        for dy, wl, m in ((0.06, 0.3, M.white), (0.01, 0.24, M.label), (-0.04, 0.28, M.white), (-0.09, 0.18, M.white)):
            F.box(_name('lyric'), wl, 0.022, 0.003, _tilt(sc, 10, dy, 0.024), m, rot=(-10, 0, 0))
        F.sphere('bouncing_ball', 0.012, _tilt(sc, 10, 0.045, 0.026), M.label, segments=6, rings=4)
        # the round stage rug with its border and star, the mic stand's weighted foot and lower tube
        F.cyl('stage', 0.3, 0.01, (0.02, 0.005, 0.1), M.stage, verts=32)
        G.torus('stage_edge', 0.296, 0.006, at=F.at(0.02, 0.008, 0.1), material=M.star, major_segments=32,
                minor_segments=3).scale = (1, 1, 0.5)
        star = [(0.02 + (0.13 if k % 2 == 0 else 0.055) * math.sin(math.pi * k / 5),
                 0.1 + (0.13 if k % 2 == 0 else 0.055) * math.cos(math.pi * k / 5)) for k in range(10)]
        verts = [(-x, -z, 0.0105) for x, z in star] + [(-x, -z, 0.006) for x, z in star]
        faces = [tuple(range(10)), tuple(range(19, 9, -1))] + [(i, (i + 1) % 10, 10 + (i + 1) % 10, 10 + i) for i in range(10)]
        _obj('star', verts, faces, [M.star])
        mx, mz = MIC
        G.revolve('mic_foot', [(0, 0), (0.13, 0), (0.13, 0.008), (0.122, 0.014), (0.06, 0.022), (0.03, 0.03), (0.02, 0.05),
                               (0, 0.05)], at=F.at(mx, 0.01, mz), material=M.stand, segments=20)
        G.kit.tube(F, 'mic_tube', [(mx, 0.05, mz), (mx, 1.0, mz)], 0.013, M.chrome, verts=10, round=1)
        G.revolve('mic_clutch', [(0, 0), (0.02, 0), (0.022, 0.006), (0.022, 0.034), (0.018, 0.04), (0, 0.04)],
                  at=F.at(mx, 0.98, mz), material=M.stand, segments=10)
        G.kit.tube(F, 'clutch_screw', [(mx, 1.0, mz), (mx + 0.04, 1.0, mz)], 0.005, M.stand, verts=6, round=1)
        # the spotlight on its pole, aimed at the stage: a finned can on a yoke, a lens in its rim
        px, pz = -0.32, 0.3
        G.revolve('spot_foot', [(0, 0), (0.12, 0), (0.12, 0.008), (0.112, 0.014), (0.03, 0.026), (0.02, 0.04), (0, 0.04)],
                  at=F.at(px, 0, pz), material=M.stand, segments=20)
        G.kit.tube(F, 'spot_pole', [(px, 0.03, pz), (px, 1.6, pz)], 0.012, M.stand, verts=8, round=1)
        head, aim = (px, 1.66, pz), (0.1, 1.2, 0.9)
        u = Vector([aim[i] - head[i] for i in range(3)]).normalized()
        d_b = Vector(F.at(*aim)) - Vector(F.at(*head))
        can = G.revolve('spot_can', [(0, -0.1), (0.045, -0.1), (0.058, -0.085), (0.064, -0.06), (0.064, 0.07),
                                     (0.074, 0.082), (0.074, 0.094), (0.066, 0.094), (0.062, 0.09), (0, 0.09)],
                        at=F.at(*head), material=M.dark, segments=16)
        _aim(can, d_b)
        for k in range(3):
            fin = G.torus(_name('spot_fin'), 0.068, 0.004, at=F.at(*[head[i] - u[i] * (0.04 - 0.025 * k) for i in range(3)]),
                          material=M.dark, major_segments=16, minor_segments=3)
            _aim(fin, d_b)
        lens = G.cylinder('spot_lens', 0.06, 0.004, at=F.at(*[head[i] + u[i] * 0.088 for i in range(3)]), material=M.lens,
                          verts=16)
        _aim(lens, d_b)
        side = Vector((u[2], 0, -u[0])).normalized()        # across the can, level (game)
        ends = [tuple(head[i] + side[i] * s * 0.085 for i in range(3)) for s in (-1, 1)]
        G.kit.tube(F, 'spot_yoke', G.kit.fillet([ends[0], (ends[0][0], 1.6, ends[0][2]), (px, 1.6, pz),
                                                 (ends[1][0], 1.6, ends[1][2]), ends[1]], 0.02), 0.008, M.stand, verts=6,
                   round=1)
        for e in ends:
            F.sphere(_name('spot_knob'), 0.014, e, M.stand, segments=8, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The upper tube, boom and microphone, lowered 25 cm into the tube: `mic_up` raises them to singing height.
    mx, mz = MIC
    drop = 0.25
    with G.new_parts() as mic:
        top = 1.35 - drop
        G.kit.tube(F, 'mic_upper', [(mx, 0.75 - drop, mz), (mx, top, mz)], 0.008, M.chrome, verts=8, round=1)
        tip = (mx - 0.05, top + 0.07, mz + 0.08)
        G.kit.tube(F, 'mic_boom', G.kit.fillet([(mx, top - 0.02, mz), (mx, top + 0.02, mz), tip], 0.02), 0.007, M.chrome,
                   verts=6, round=1)
        end = (tip[0] - 0.015, tip[1] + 0.05, tip[2] + 0.07)
        F.sweep('mic_handle', [tip, end], 0.017, M.dark, verts=10, scale=[0.7, 1.0])
        g = (tip[0] - 0.02, tip[1] + 0.068, tip[2] + 0.095)
        F.sphere('mic_grille', 0.03, g, M.steel, segments=10, rings=6)
        F.sweep('mic_band', [end, tuple(end[i] + (g[i] - end[i]) * 0.35 for i in range(3))], 0.024, M.dark, verts=10)
    mic = G.hinged(G.join(mic, 'mic'), F.at(mx, 1.0, mz))
    objs = G.settle(centre=False)
    G.slide(mic, 'mic_up', (0, 0, drop), seconds=0.8)
    return objs


# --- the fizz & sunshine mocktail cart (1x1): a brass bar cart under a striped canopy ---------------------------
TOP = 0.815     # the top shelf's top
SHAKER = (0.06, 0.08)   # the shaker's foot (x, depth), tipping over to the left


def _bottle(F, name, x, y, z, r, h, material, cap):
    """A syrup bottle standing on `y`: a lofted body with a shoulder and neck, a label band, a chrome pourer."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.92), (0.02, 1.0), (0.62, 1.0), (0.76, 0.6), (0.84, 0.38), (0.95, 0.38)]
    b = G.loft(name, [G.ring(r * k, r * k, bz + h * t, 10, at=(bx, by)) for t, k in rows], material=material)
    _paint(b, M.porcelain, lambda c: 0.2 * h < c.z - bz < 0.5 * h)
    G.revolve(f'{name}_pourer', [(0, 0), (r * 0.42, 0), (r * 0.42, 0.012), (r * 0.15, 0.03), (0, 0.032)],
              at=(bx, by, bz + h * 0.95), material=cap, segments=8)


def mocktailCart():
    F = G.GameFrame()
    zb, zf = -0.29, 0.17
    with G.new_parts() as still:
        for y in (0.3, 0.8):
            F.extrude(_name('shelf'), G.rounded_rect(0.78, 0.46, 0.03, corner=2), 0.03, (0, y, -0.06), M.mint,
                      plane='top', bevel=0.008, bevel_segments=1)
            gal = G.rounded_rect(0.78, 0.46, 0.04, corner=2)
            F.sweep(_name('gallery'), [(x, y + 0.06, -0.06 + z) for x, z in gal], 0.006, M.brass, verts=6, closed=True)
            for x in (-0.2, 0.0, 0.2):
                for z in (zb, zf):
                    G.kit.tube(F, _name('spindle'), [(x, y + 0.015, z), (x, y + 0.06, z)], 0.0035, M.brass, verts=4,
                               round=1)
        for x in (-0.39, 0.39):
            for z in (zb, zf):
                y0 = 0.07 if z == zf else 0.0
                G.kit.tube(F, _name('post'), [(x, y0, z), (x, 1.6, z)], 0.012, M.brass, verts=8, round=1)
                G.revolve(_name('finial'), [(0, 0), (0.014, 0), (0.018, 0.008), (0.012, 0.016), (0, 0.022)],
                          at=F.at(x, 1.598, z), material=M.brass, segments=8)
            G.torus(_name('wheel'), 0.055, 0.016, at=F.at(x, 0.071, zf), material=M.tyre, rot=(0, 90, 0),
                    major_segments=16, minor_segments=6)
            F.cyl(_name('hub'), 0.03, 0.04, (x, 0.071, zf), M.brass, rot=(0, 90, 0), verts=10)
            G.revolve(_name('cap'), [(0, 0), (0.018, 0), (0.016, 0.014), (0.012, 0.02), (0, 0.02)], at=F.at(x, 0, zb),
                      material=M.dark, segments=8)
        G.kit.tube(F, 'handle', [(-0.39, 0.95, zb), (-0.39, 0.95, zf)], 0.012, M.brass, verts=8, round=1)
        F.sweep('handle_grip', [(-0.39, 0.95, zb + 0.1), (-0.39, 0.95, zf - 0.1)], 0.017, M.porcelain, verts=8)
        # the striped canopy, arched front to back, with a scalloped valance at the front and sides
        for i in range(6):
            m = M.canopy if i % 2 else M.canopy_white
            x = -0.35 + i * 0.14
            pts = [(x, 1.6 + 0.024 * (1 - ((2 * k / 8) - 1) ** 2) + 0.006, -0.33 + 0.56 * k / 8) for k in range(9)]
            F.sweep(_name('canopy'), pts, None, m, profile=G.rounded_rect(0.012, 0.142, 0.004, corner=1), up=(0, 0, 1))
            pts = [(-0.07, 1.605), (0.07, 1.605)] + [(0.07 * math.cos(math.pi * k / 6), 1.56 - 0.07 * math.sin(math.pi * k / 6))
                                                     for k in range(7)]
            _slab(F, _name('scallop'), [(x + px, py) for px, py in pts], 0.218, 0.23, m)
        for s in (-1, 1):
            for i in range(4):
                z = -0.26 + i * 0.14
                m = M.canopy if i % 2 else M.canopy_white
                pts = [(z - 0.07, 1.605), (z + 0.07, 1.605)] + [(z + 0.07 * math.cos(math.pi * k / 6),
                                                                1.56 - 0.07 * math.sin(math.pi * k / 6)) for k in range(7)]
                verts = [F.at(s * 0.41 + s * 0.006 * j, y, zz) for j in (1, -1) for zz, y in pts]
                nn = len(pts)
                faces = [tuple(range(nn)), tuple(range(2 * nn - 1, nn - 1, -1))]
                faces += [(k, (k + 1) % nn, nn + (k + 1) % nn, nn + k) for k in range(nn)]
                _obj(_name('scallop_side'), verts, faces, [m])
        # the top: a jug of juice, an ice bucket, a pineapple, citrus and three drinks
        jx, jz = -0.25, -0.17
        bx, by, bz = F.at(jx, TOP, jz)
        G.loft('jug', [G.ring(r, r, bz + h, 14, at=(bx, by)) for h, r in ((0, 0.048), (0.01, 0.055), (0.08, 0.06),
                                                                           (0.15, 0.05), (0.19, 0.046), (0.2, 0.05))],
               material=M.porcelain)
        F.cyl('jug_juice', 0.044, 0.002, (jx, TOP + 0.201, jz), M.juice, verts=12)
        F.cone('jug_spout', 0.02, 0.008, 0.03, (jx + 0.05, TOP + 0.195, jz), M.porcelain, rot=(0, -50, 0), verts=8)
        F.sweep('jug_handle', G.kit.fillet([(jx - 0.055, TOP + 0.17, jz), (jx - 0.095, TOP + 0.16, jz),
                                            (jx - 0.09, TOP + 0.06, jz), (jx - 0.058, TOP + 0.04, jz)], 0.02, steps=2),
                None, M.porcelain, profile=G.rounded_rect(0.016, 0.008, 0.003, corner=1), up=(0, 1, 0))
        G.revolve('bucket', [(0, 0), (0.06, 0), (0.08, 0.12), (0.075, 0.12), (0.055, 0.01), (0, 0.01)],
                  at=F.at(0.2, TOP, -0.17), material=M.chrome, segments=16)
        for s in (-1, 1):
            G.torus(_name('bucket_ring'), 0.016, 0.003, at=F.at(0.2 + s * 0.08, TOP + 0.095, -0.17), material=M.chrome,
                    rot=(90, 0, 90), major_segments=8, minor_segments=3)
        for k, (dx, dz) in enumerate(((-0.02, 0.0), (0.02, 0.015), (0.0, -0.025), (0.03, -0.01))):
            F.box(_name('ice'), 0.03, 0.03, 0.03, (0.2 + dx, TOP + 0.11 + 0.008 * (k % 2), -0.17 + dz), M.ice,
                  rot=(10 * k, 20, 15 * k), bevel=0.005, bevel_segments=1)
        # the pineapple: a lumpy body and a crown of long leaves
        G.jitter(F.sphere('pineapple', 0.055, (0.3, TOP + 0.08, 0.06), M.pineapple, scale=(1, 1, 1.45), segments=12,
                          rings=8), 0.004, size=0.02, seed=4.0)
        leaf = G.smooth_outline([(0.0, 0.0), (0.012, 0.03), (0.006, 0.1), (0.0, 0.13), (-0.006, 0.1), (-0.012, 0.03)], 1)
        for k in range(7):
            a = 51 * k
            lean = 18 + 10 * (k % 2)
            F.extrude(_name('crown'), leaf, 0.004, (0.3 + 0.008 * G.cosd(a), TOP + 0.155, 0.06 + 0.008 * G.sind(a)), M.leaf,
                      rot=(lean * G.sind(a), lean * G.cosd(a), -a))
        for x, z, r, m in ((0.15, 0.1, 0.034, M.lemon), (0.21, 0.13, 0.03, M.lime), (0.17, 0.02, 0.038, M.orange)):
            F.sphere(_name('citrus'), r, (x, TOP + r, z), m, scale=(1, 1, 0.95), segments=10, rings=6)
        F.cyl('lemon_half', 0.034, 0.006, (0.24, TOP + 0.003, 0.0), M.porcelain, verts=10)
        F.cyl('lemon_face', 0.03, 0.006, (0.24, TOP + 0.006, 0.0), M.lemon, verts=10)
        _tumbler(F, -0.1, 0.1, M.drink_pink, slice_=M.lemon)
        _tumbler(F, -0.22, 0.06, M.drink_sun, straw=M.sky)
        _tumbler(F, -0.33, 0.1, M.drink_teal, slice_=M.lime)
        # below: a little fridge drawer and three syrup bottles
        F.extrude('fridge', G.rounded_rect(0.3, 0.2, 0.03, corner=2), 0.22, (-0.15, 0.415, -0.1), M.fridge, bevel=0.01,
                  bevel_segments=1)
        F.box('fridge_seam', 0.27, 0.004, 0.004, (-0.15, 0.49, 0.011), M.chrome)
        G.kit.pull(F, 'fridge_handle', -0.15, 0.46, 0.01, 0.12, M.chrome, r=0.005, out=0.02)
        for i, m in enumerate((M.drink_pink, M.drink_teal, M.drink_sun)):
            _bottle(F, _name('syrup'), 0.12 + i * 0.09, 0.315, -0.12, 0.033, 0.24, m, M.chrome)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cocktail shaker, hinged on its foot's left edge: `shake` takes it off the cart (squashed away) while the
    # resident shakes held.py's `tool.shaker` in both hands (review I-kitchen-13).
    sx, sz = SHAKER
    with G.new_parts() as shaker:
        G.revolve('shaker', [(0, 0), (0.034, 0), (0.036, 0.004), (0.042, 0.12), (0.044, 0.124), (0.038, 0.13),
                             (0.036, 0.15), (0.022, 0.172), (0.014, 0.176), (0.016, 0.19), (0.012, 0.205), (0, 0.206)],
                  at=F.at(sx, TOP, sz), material=M.chrome, segments=16)
        G.torus('shaker_band', 0.04, 0.004, at=F.at(sx, TOP + 0.06, sz), material=M.chrome, major_segments=16,
                minor_segments=3)
    shaker = G.hinged(G.join(shaker, 'shaker'), F.at(sx - 0.036, TOP, sz))
    objs = G.settle(centre=False)
    G.squash(shaker, 'shake', 0.001, seconds=0.2)
    return objs


# --- the rainbow hammock (3x1): a striped sling 2 m long between spreader bars, on a wooden stand (2026-10-21: 2x1,
# its sling 1.5 m, grown-ups lying in it with the knees up: M-packs-06, Q-07) ---------------------------------------
SLING_END, SLING_LOW, SLING_W = 0.9, 0.5, 0.62       # the sling's ends' and middle's heights, its width
SLING_X = 1.0                                        # its ends' x, at the spreader bars
HOOK = 1.08                                          # the hooks' height: the sway's pivot
POST_X = 1.38                                        # the posts' tops' x, the hooks on them
PILLOW = 0.75                                        # the pillow's x (towards the left end): `furniture.json` `head`


def _sling_y(u, v):
    """The sling's top at a fraction `u` along it and `v` across it: sagging along, cupped across in the middle."""
    s = 2 * u - 1
    return SLING_LOW + (SLING_END - SLING_LOW) * s * s + 0.07 * (2 * v - 1) ** 2 * (1 - s * s)


def rainbowHammock():
    F = G.GameFrame()
    beam = G.rounded_rect(0.07, 0.09, 0.02, corner=1)
    with G.new_parts() as still:
        # the base beam, its ends rounded, and the two posts curving up and out from it to the hooks
        F.sweep('beam', [(-(POST_X - 0.1), 0.05, 0), (POST_X - 0.1, 0.05, 0)], None, M.wood, profile=beam, up=(0, 0, 1))
        for s in (-1, 1):
            F.sweep(_name('post'), [(s * (POST_X - 0.16), 0.04, 0), (s * (POST_X - 0.1), 0.3, 0),
                                    (s * (POST_X - 0.04), 0.75, 0), (s * POST_X, HOOK + 0.05, 0)], None, M.wood,
                    profile=G.rounded_rect(0.07, 0.07, 0.02, corner=1), up=(0, 0, 1), resolution=3)
            G.revolve(_name('post_top'), [(0, 0), (0.04, 0), (0.042, 0.02), (0.03, 0.034), (0.012, 0.042), (0, 0.042)],
                      at=F.at(s * POST_X, HOOK + 0.05, 0), material=M.wood, segments=12)
            F.extrude(_name('foot'), G.rounded_rect(0.62, 0.045, 0.02, corner=2), 0.09, (s * (POST_X - 0.1), 0.0225, 0),
                      M.wood, plane='side', bevel=0.008, bevel_segments=1)
            for z in (-0.18, 0.18):                                   # braces from the foot to the post
                F.sweep(_name('brace'), [(s * (POST_X - 0.1), 0.04, z * 1.5), (s * (POST_X - 0.11), 0.34, 0)], None, M.wood,
                        profile=G.rounded_rect(0.04, 0.03, 0.01, corner=1), up=(0, 0, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    # The sling, its spreader bars and ropes, the hooks, the pillow and the throw: `sway` rocks them about the hooks.
    with G.new_parts() as sling:
        nu, nv, x0, x1, t = 26, 12, -SLING_X, SLING_X, 0.018
        verts, faces, mats = [], [], []
        for layer in (0, 1):
            for i in range(nu + 1):
                for j in range(nv + 1):
                    u, v = i / nu, j / nv
                    verts.append(F.at(x0 + (x1 - x0) * u, _sling_y(u, v) - t * layer, -SLING_W / 2 + SLING_W * v))
        at = lambda layer, i, j: layer * (nu + 1) * (nv + 1) + i * (nv + 1) + j  # noqa: E731
        stripe = lambda j: min(5, j * 6 // nv)  # noqa: E731
        for i in range(nu):
            for j in range(nv):
                faces.append((at(0, i, j), at(0, i + 1, j), at(0, i + 1, j + 1), at(0, i, j + 1)))
                faces.append((at(1, i, j), at(1, i, j + 1), at(1, i + 1, j + 1), at(1, i + 1, j)))
                mats += [stripe(j)] * 2
        for i in range(nu):                                           # the long edges
            for j in (0, nv):
                faces.append((at(0, i, j), at(1, i, j), at(1, i + 1, j), at(0, i + 1, j)))
                mats.append(stripe(min(j, nv - 1)))
        for j in range(nv):                                           # the ends
            for i in (0, nu):
                faces.append((at(0, i, j), at(0, i, j + 1), at(1, i, j + 1), at(1, i, j)))
                mats.append(stripe(j))
        _obj('sling', verts, faces, [getattr(M, s) for s in STRIPES], face_mats=mats, sharp=50)
        # a corded edge along each long side, and a fringe of tassels hanging from it
        for j, side in ((0, -1), (nv, 1)):
            edge = [(x0 + (x1 - x0) * i / nu, _sling_y(i / nu, j / nv) - t / 2, side * SLING_W / 2) for i in range(nu + 1)]
            F.sweep(_name('edge_cord'), edge, 0.008, M.rope, verts=5)
            for i in range(2, nu - 1, 2):
                x, y, z = edge[i]
                G.kit.tube(F, _name('tassel'), [(x, y, z + side * 0.004), (x, y - 0.09, z + side * 0.012)], 0.005,
                           getattr(M, STRIPES[(i // 2) % 6]), verts=4, round=1, scale=[0.7, 1.2])
        for s in (-1, 1):
            bx = s * (SLING_X + 0.015)
            F.sweep(_name('spreader'), [(bx, SLING_END - 0.01, -(SLING_W / 2 + 0.014)), (bx, SLING_END - 0.01, SLING_W / 2 + 0.014)],
                    0.022, M.spreader, verts=10)
            for z in (-1, 1):
                F.sphere(_name('spreader_end'), 0.026, (bx, SLING_END - 0.01, z * (SLING_W / 2 + 0.014)), M.spreader,
                         segments=10, rings=6)
            hook = (s * (POST_X - 0.02), HOOK, 0)
            G.torus(_name('ring'), 0.03, 0.007, at=F.at(*hook), material=M.lamp_metal, rot=(90, 0, 0), major_segments=12,
                    minor_segments=4)
            G.kit.tube(F, _name('hook'), G.kit.fillet([(s * (POST_X - 0.02), HOOK + 0.05, 0), (s * (POST_X - 0.02), HOOK + 0.032, 0),
                                                       (s * (POST_X - 0.045), HOOK + 0.03, 0),
                                                       (s * (POST_X - 0.048), HOOK + 0.05, 0)], 0.01), 0.005, M.lamp_metal,
                       verts=5, round=1)
            for z in (-0.33, -0.22, -0.11, 0.0, 0.11, 0.22, 0.33):
                G.kit.tube(F, _name('rope'), [(s * (POST_X - 0.04), HOOK - 0.03, 0), (bx, SLING_END + 0.012, z)], 0.005,
                           M.rope, verts=4, round=1)
        # the pillow at the left end, lying on the slope
        u = (-PILLOW - x0) / (x1 - x0)
        slope = math.degrees(math.atan((SLING_END - SLING_LOW) * 4 * (2 * u - 1) / (x1 - x0)))
        F.soft('pillow', 0.36, 0.11, 0.3, (-PILLOW, _sling_y(u, 0.5) + 0.05, 0), M.pillow, rot=(0, slope, 0), round=0.04,
               puff=0.03, flat=2)
    sling = G.hinged(G.join(sling, 'sling'), F.at(0, HOOK, 0))
    objs = G.settle(centre=False)
    G.rock(sling, 'sway', 5, seconds=2.4, axis='X')
    return objs


# --- the sunrise stretch lamp (1x1): a dawn scene in a round frame on a stand, a yoga mat before it -------------
SUN_C = (0.0, 1.25, -0.3)    # the frame's middle (x, height, depth)
SUN_R = 0.33                 # the scene's radius
SUN_RISE = 0.22


def _hills(cy, r, top, waves, amp, phase, n=40):
    """The part of a disc of radius `r` (centre height `cy`) below a wavy horizon at `top` (offsets from the centre)."""
    pts = []
    for k in range(n + 1):                                         # the horizon, left to right
        x = -r + 2 * r * k / n
        y = top + amp * math.sin(waves * math.pi * (x + r) / (2 * r) + phase)
        y = min(y, math.sqrt(max(r * r - x * x, 0.0)))
        pts.append((x, y))
    lo = [(x, -math.sqrt(max(r * r - x * x, 0.0))) for x, _ in pts]
    out = pts + lo[::-1][1:-1]                                     # along the top, back along the circle's foot
    return [(x, cy + y) for x, y in out]


def _spiral(F, name, x, y, z, r0, r1, turns, material, side):
    """A spiral line on the end of a rolled mat at game x `x` (its end face), round (y, z): the layers of the roll."""
    pts = []
    n = int(turns * 14)
    for k in range(n + 1):
        f = k / n
        a = math.tau * turns * f
        r = r0 + (r1 - r0) * f
        pts.append((x + side * 0.002, y + r * math.sin(a), z + r * math.cos(a)))
    return F.sweep(name, pts, 0.0025, material, verts=4)


def sunriseLamp():
    F = G.GameFrame()
    cx, cy, cz = SUN_C
    with G.new_parts() as still:
        G.revolve('base', [(0, 0), (0.13, 0), (0.13, 0.012), (0.122, 0.02), (0.1, 0.028), (0.04, 0.036), (0.03, 0.05),
                           (0, 0.05)], at=F.at(cx, 0, cz), material=M.housing, segments=24)
        G.kit.tube(F, 'stem', [(cx, 0.04, cz - 0.03), (cx, cy - 0.37, cz - 0.03)], 0.016, M.lamp_metal, verts=10, round=1)
        G.revolve('stem_collar', [(0, 0), (0.024, 0), (0.026, 0.01), (0.02, 0.03), (0.018, 0.05), (0, 0.05)],
                  at=F.at(cx, cy - 0.41, cz - 0.03), material=M.lamp_metal, segments=10)
        F.cyl('dimmer', 0.018, 0.014, (cx + 0.07, 0.04, cz + 0.06), M.lamp_metal, verts=10)
        # the case: a shallow round drum, its edge rolled, the ring round the scene's face
        G.revolve('case', [(0, -0.06), (0.37, -0.06), (0.384, -0.05), (0.39, -0.03), (0.386, -0.008), (0.37, 0.0),
                           (0, 0.0)], at=F.at(cx, cy, cz), material=M.housing, segments=40, rot=(90, 0, 0))
        G.torus('ring', 0.355, 0.02, at=F.at(cx, cy, cz + 0.006), material=M.lamp_metal, rot=(90, 0, 0),
                major_segments=40, minor_segments=6)
        F.cyl('dawn', SUN_R + 0.01, 0.006, (cx, cy, cz + 0.003), M.dawn, rot=(90, 0, 0), verts=40)
        for k, (dx, dy, w) in enumerate(((-0.14, 0.17, 0.16), (0.12, 0.22, 0.12), (0.2, 0.08, 0.09))):
            cloud = G.smooth_outline([(-w / 2, 0), (w / 2, 0), (w / 2, 0.012), (w * 0.25, 0.03), (0, 0.022), (-w * 0.2, 0.036),
                                      (-w / 2, 0.012)], 2)
            F.extrude(_name('cloud'), [(cx + dx + u, cy + dy + v) for u, v in cloud], 0.004, (0, 0, cz + 0.024), M.cloud)
        _slab(F, 'hills_far', _hills(cy, SUN_R, -0.06, 3, 0.03, 0.4), cz + 0.032, cz + 0.038, M.hills_far)
        _slab(F, 'hills', _hills(cy, SUN_R, -0.12, 2, 0.035, 2.2), cz + 0.038, cz + 0.046, M.hills)
        # the yoga mat, its far end rolled up (the layers drawn on its ends), and a cork block on it
        F.box('mat', 0.6, 0.008, 0.38, (0, 0.004, 0.13), M.yoga, bevel=0.002, bevel_segments=1)
        F.cyl('mat_roll', 0.06, 0.6, (0, 0.06, 0.38), M.yoga_roll, rot=(0, 90, 0), verts=16)
        for s in (-1, 1):
            _spiral(F, _name('roll_layers'), s * 0.3, 0.06, 0.38, 0.012, 0.054, 2.6, M.yoga, s)
        F.box('block', 0.22, 0.075, 0.14, (-0.2, 0.0455, 0.06), M.cork, rot=(0, 0, 15), bevel=0.008, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sun, set behind the hills: `sunrise` raises it into the sky, which glows at night (`dawn`'s `glow`).
    with G.new_parts() as sun:
        F.cyl('sun', 0.13, 0.004, (cx, cy - 0.19, cz + 0.011), M.sun, rot=(90, 0, 0), verts=32)
        G.torus('sun_halo', 0.11, 0.004, at=F.at(cx, cy - 0.19, cz + 0.014), material=M.cloud, rot=(90, 0, 0),
                major_segments=24, minor_segments=3).scale = (1, 1, 0.4)
    sun = G.hinged(G.join(sun, 'sun'), F.at(cx, cy - 0.19, cz + 0.011))
    objs = G.settle(centre=False)
    G.slide(sun, 'sunrise', (0, 0, SUN_RISE), seconds=1.5)
    return objs


# --- the confetti party booth (2x2): a horseshoe booth for two under a bulb-lit frame, a table with a tablet ----
def _channels(F, name, x0, x1, n, y0, top, z0, z1, lean, mat):
    """A channelled back: `n` stuffed channels side by side from x0 to x1, each rising from the seat at `y0`
    (depth `z0`) to `top`, leaning back to `z1` and rolling over at the top."""
    cw = (x1 - x0) / n
    for i in range(n):
        x = x0 + cw * (i + 0.5)
        F.sweep(_name(name), [(x, y0, z0), (x, (y0 + top) / 2, (z0 + z1) / 2 - 0.01), (x, top - 0.05, z1), (x, top - 0.015, z1 - lean)],
                None, mat, profile=G.rounded_rect(cw * 1.02, 0.17, 0.06, corner=1), up=(1, 0, 0), resolution=2,
                scale=[1.0, 1.04, 1.0, 0.85])


def partyBooth():
    F = G.GameFrame()
    zr, xr, yr = 0.82, 0.88, 1.58
    with G.new_parts() as still:
        F.soft('seat_base', 1.3, 0.36, 0.62, (0, 0.18, -0.05), M.booth, round=0.05, flat=1, round_segments=1)
        F.soft('seat', 1.3, 0.088, 0.58, (0, 0.394, -0.04), M.booth, round=0.035, puff=0.012, piping=0.006,
               piping_material=M.piping, flat=2)
        F.box('kick', 1.26, 0.04, 0.02, (0, 0.02, 0.27), M.panel)
        # the channelled backrest, its rolled top piped in yellow
        _channels(F, 'channel', -0.93, 0.93, 7, 0.38, 1.05, -0.3, -0.42, 0.06, M.booth)
        F.sweep('backrest_piping', [(-0.93, 1.04, -0.475), (0.93, 1.04, -0.475)], 0.012, M.piping, verts=8)
        F.extrude('back_panel', G.rounded_rect(1.9, 1.6, 0.04, corner=2), 0.06, (0, 0.8, WALL2 + 0.03), M.panel,
                  bevel=0.012, bevel_segments=1)
        for s in (-1, 1):
            F.soft(_name('wing'), 0.3, 0.7, 1.0, (s * 0.8, 0.37, 0.4), M.booth, round=0.06, piping=0.008,
                   piping_material=M.piping, flat=1, round_segments=1)
            F.box(_name('wing_base'), 0.28, 0.04, 0.96, (s * 0.8, 0.02, 0.4), M.panel, bevel=0.01, bevel_segments=1)
        # the bulb-lit frame over it: each side one bent brass tube, from the wing's front up and back to the panel
        for s in (-1, 1):
            G.kit.tube(F, _name('frame'), G.kit.fillet([(s * xr, 0.72, zr), (s * xr, yr, zr), (s * xr, yr, WALL2 + 0.06)],
                                                       0.06), 0.016, M.brass, verts=8, round=1)
            G.revolve(_name('frame_foot'), [(0, 0), (0.03, 0), (0.03, 0.01), (0.02, 0.02), (0, 0.02)],
                      at=F.at(s * xr, 0.715, zr), material=M.brass, segments=10)
        F.extrude('marquee', G.rounded_rect(1.76, 0.12, 0.06, corner=4), 0.05, (0, yr, zr), M.panel, bevel=0.01,
                  bevel_segments=1)
        for i in range(9):
            _bulb(F, -0.72 + i * 0.18, yr, zr + 0.03, i, r=0.026)
        # the marquee bulbs and confetti on the back panel
        for i in range(13):
            x = -0.84 + i * 0.14
            _bulb(F, x, 1.53, WALL2 + 0.065, i, r=0.03)
            _bulb(F, x + 0.07, 1.28 + 0.06 * math.sin(i), WALL2 + 0.063, i + 3, r=0.024)
        G.kit.tube(F, 'bulb_wire', [(-0.77 + 0.14 * i, 1.28 + 0.06 * math.sin(i) + 0.02, WALL2 + 0.062) for i in range(13)],
                   0.0025, M.stand, verts=4, round=2)
        G.kit.tube(F, 'bulb_wire_top', [(-0.86, 1.555, WALL2 + 0.062), (0.86, 1.555, WALL2 + 0.062)], 0.0025, M.stand,
                   verts=4, round=1)
        for i, (x, y) in enumerate([(-0.6, 1.08), (-0.3, 1.18), (0.1, 1.1), (0.45, 1.15), (0.7, 1.05), (-0.05, 1.4),
                                    (0.3, 1.38), (-0.45, 1.4), (0.62, 1.42), (-0.75, 1.22)]):
            if i % 2:
                F.box(_name('confetti'), 0.05, 0.05, 0.008, (x, y, WALL2 + 0.064), getattr(M, BULBS[i % 6]), rot=(0, 30 * i, 0))
            else:
                F.cyl(_name('confetti'), 0.026, 0.008, (x, y, WALL2 + 0.064), getattr(M, BULBS[i % 6]), rot=(90, 0, 0), verts=8)
        # the round table with the party tablet, two drinks and a striped bowl of popcorn
        tz = 0.52
        F.cyl('table', 0.31, 0.035, (0, 0.72, tz), M.white, verts=32, bevel=0.01, bevel_segments=1)
        G.revolve('table_stem', [(0.2, 0), (0.2, 0.012), (0.18, 0.022), (0.06, 0.045), (0.035, 0.1), (0.035, 0.68),
                                 (0.06, 0.7), (0, 0.705)], at=F.at(0, 0, tz), material=M.brass, segments=20)
        F.box('tablet_foot', 0.12, 0.012, 0.08, (0, 0.743, 0.62), M.dark, bevel=0.004, bevel_segments=1)
        tc = (0, 0.86, 0.6)
        F.extrude('tablet', G.rounded_rect(0.34, 0.22, 0.02, corner=2), 0.02, tc, M.dark, rot=(12, 0, 0), bevel=0.005,
                  bevel_segments=1)
        F.box('tablet_face', 0.31, 0.19, 0.004, _tilt(tc, -12, 0, -0.011), M.tablet_face, rot=(12, 0, 0))
        _tumbler(F, -0.18, 0.4, M.drink_sun, y=0.7375)
        _tumbler(F, 0.18, 0.4, M.drink_teal, y=0.7375, straw=M.piping)
        bowl = G.revolve('popcorn_bowl', [(0, 0), (0.05, 0), (0.09, 0.05), (0.085, 0.052), (0.045, 0.006), (0, 0.006)],
                         at=F.at(0.14, 0.7375, 0.66), material=M.white, segments=16)
        _paint(bowl, M.bulb_c, lambda c: math.atan2(c.y, c.x) % (math.pi / 4) < math.pi / 8)
        for k in range(7):
            a = 51 * k
            r = 0.04 if k % 3 else 0.015
            F.sphere(_name('popcorn'), 0.022, (0.14 + r * G.cosd(a), 0.79 + 0.008 * (k % 2), 0.66 + r * G.sind(a)),
                     M.popcorn, segments=6, rings=4)
        # the left curtain, gathered at the back
        _pleats(F, 'curtain_left', -xr, 0.77, yr - 0.02, WALL2 + 0.07, WALL2 + 0.45, 4, 0.03, M.curtain)
    G.join(still, 'body', origin=(0, 0, 0))
    # The right curtain, hanging closed along the rail: `curtain` draws it aside to the back.
    with G.new_parts() as cur:
        _pleats(F, 'curtain', xr, 0.77, yr - 0.02, WALL2 + 0.07, zr - 0.04, 9, 0.022, M.curtain)
    cur = G.hinged(G.join(cur, 'curtain'), F.at(xr, yr, WALL2 + 0.07))
    objs = G.settle(centre=False)
    G.squash(cur, 'curtain', (1, 0.25, 1), seconds=0.8)
    return objs


FAMILY = G.Family(
    'pack_cheerful', M,
    [G.Model('jukebox', jukebox, size=(0.75, 1.43, 0.495), key='model.cheerful.jukebox',
             centre=(0, -0.183), motions={'uses': {'dance': 'record'}, 'loops': {'dance': 'spin'}},
             paint=[('cherry', 'Body'), ('amber', 'Trim')]),
     G.Model('karaokeMachine', karaokeMachine, size=(0.76, 1.763, 0.851), key='model.cheerful.karaokeMachine',
             centre=(-0.06, -0.005), motions={'uses': {'sing': 'mic_up'}},
             paint=[('speaker', 'Body'), ('speaker_trim', 'Trim'), ('stage', 'Rug')]),
     G.Model('mocktailCart', mocktailCart, size=(0.84, 1.635, 0.57), key='model.cheerful.mocktailCart',
             centre=(0, -0.045), motions={'uses': {'mix': 'shake'}},
             paint=[('mint', 'Body'), ('canopy', 'Trim'), ('brass', 'Frame')]),
     G.Model('rainbowHammock', rainbowHammock, size=(2.844, 1.172, 0.7), key='model.cheerful.rainbowHammock',
             centre=(0, 0), motions={'loops': {'swing': 'sway', 'doze': 'sway'}, 'ride': 'sway'},
             paint=[('wood', 'Body'), ('spreader', 'Trim'), ('pillow', 'Pillow')]),
     G.Model('sunriseLamp', sunriseLamp, size=(0.784, 1.642, 0.87), key='model.cheerful.sunriseLamp',
             centre=(0, 0.005), motions={'uses': {'stretch': 'sunrise'}},
             paint=[('housing', 'Body'), ('lamp_metal', 'Trim'), ('yoga', 'Mat')]),
     G.Model('partyBooth', partyBooth, size=(1.9, 1.64, 1.83), key='model.cheerful.partyBooth',
             centre=(0, -0.015), motions={'uses': {'party': 'curtain'}, 'hold': 1.5},
             paint=[('booth', 'Body'), ('piping', 'Trim'), ('curtain', 'Curtain')])],
    comment='Blender-built cheerful pack models (tools/art/blender/families/pack_cheerful.py). Listed in '
            'the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. Each has one moving part: the record swings up, the microphone '
            'rises, the shaker is taken off the cart, the hammock sways (a loop its sleeper rides), the sun rises (its sky '
            'glowing at night) and the curtain is drawn.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
