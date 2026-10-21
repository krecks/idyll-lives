# The decor: wall art and clocks (D1: `model.printFramed`, `photoPrint`, `paintingClassic`, `canvasAbstract`,
# `wallClock`, `wallShelf`), ornaments (D2: `grandfatherClock`, `vaseTall`, `urnPorcelain`, `vasePedestal`,
# `bustMarble`, `statueCat`, `statueHorse`, `coatRack`, `hangingPlant`, `fireplace`) and textiles (D3:
# `curtains`, `curtainsLong`, `rugRound`, `rugLarge`, `rugRunner`, `rugPattern`). Notes: decor.md.
#
# Built where the game places them (`F`, game axes): the wall things at their real height with their back
# on the wall face (`floor=False`), the hanging plant from the ceiling. What moves:
#   - the clocks' hour and minute hands tell the game's time (`hour_hand`, `minute_hand`, `motions.clock`); the
#     grandfather clock's pendulum swings all the time (`pendulum`, `idle`): its case is plain satin, not wood, so
#     it keeps 4 draw calls, and its trunk door stays still (it is wound at the dial);
#   - the hanging plant sways when watered (`sway`);
#   - the fireplace's flames grow on the logs while a resident warms up (`flames`);
#   - the curtains are drawn closed at night (`close`), on cloth tabs over the rail.
#
# Run:  cargo art decor [--only wallClock] [--preview]
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
    # frames and wall things
    frame_white=('#F2F0EA', 'satin'),
    frame_black=('#2A2C30', 'satin'),
    gilt=('#B8925A', 'metal'),
    gilt_dark=('#8E6E3A', 'metal'),
    oak=('#CDB08A', 'wood'),
    mount=('#FBFAF6', 'matte'),
    # the pictures: flat colour, matte
    paper=('#F1EBDD', 'matte'),
    sage=('#9DAF8C', 'matte'),
    olive=('#8C9468', 'matte'),
    sky=('#A9C8DD', 'matte'),
    rock=('#6E7E8A', 'matte'),
    rock_dark=('#56656F', 'matte'),
    snow=('#F4F4F2', 'matte'),
    lake=('#6E95AE', 'matte'),
    shore=('#4E6B4A', 'matte'),
    sun=('#F6E7B0', 'matte'),
    haze=('#C9D3C8', 'matte'),
    cloud=('#EDEBE2', 'matte'),
    hills=('#8E9A7A', 'matte'),
    field=('#B59B5E', 'matte'),
    trunk=('#4A3A2C', 'matte'),
    tree=('#4E5E3A', 'matte'),
    cottage=('#E7E0D4', 'matte'),
    roof=('#8E3F2E', 'matte'),
    canvas=('#F4F2EE', 'matte'),
    mustard=('#D6A64A', 'matte'),
    rust=('#B5653F', 'matte'),
    ink=('#2A2C30', 'satin'),
    # clocks
    clock_case=('#C3C7CB', 'metal'),
    face=('#FBFAF6', 'satin'),
    second=('#D9483B', 'satin'),
    walnut=('#6B4630', 'satin'),     # the grandfather clock's polished case (plain: its pendulum takes the 4th draw)
    walnut_dark=('#4E3222', 'satin'),
    dial=('#F4EBD8', 'gloss'),
    brass=('#B8925A', 'metal'),
    hollow=('#2A1C14', 'matte'),     # the clock's trunk behind the pendulum
    # the shelf's things
    pot=('#E8E6E1', 'gloss'),
    leaf=('#5E8F4E', 'foliage'),
    leaf_light=('#6E9A55', 'foliage'),
    book_teal=('#3E7A7A', 'satin'),
    book_rust=('#B5653F', 'satin'),
    book_mustard=('#D6A64A', 'satin'),
    book_navy=('#3E5470', 'satin'),
    book_cream=('#EFE5D3', 'satin'),
    clay=('#C98F6B', 'matte'),
    # ornaments
    ceramic=('#E3DDD2', 'satin'),
    twig=('#8E6E52', 'satin'),
    plume=('#D9C7A7', 'foliage'),
    porcelain=('#F4F4F0', 'gloss'),
    blue=('#2E4E8E', 'gloss'),
    stone=('#E7E3DC', 'matte'),
    vase_blue=('#3E6E9A', 'gloss'),
    stem=('#6E8B4E', 'foliage'),
    bloom=('#F4EFE6', 'foliage'),
    marble=('#EDEBE6', 'satin'),
    concrete=('#9A9893', 'matte'),
    concrete_dark=('#6E6C68', 'matte'),
    bronze=('#6E4F35', 'metal'),
    bronze_dark=('#5A3F2A', 'metal'),
    coat=('#3E5470', 'fabric'),
    scarf=('#B5653F', 'fabric'),
    hat=('#C9B79C', 'fabric'),
    pot_white=('#F2F0EA', 'satin'),
    cord=('#C9B79C', 'matte'),
    # the fireplace
    surround=('#E7E3DC', 'satin'),
    hearth=('#8E8A84', 'matte'),
    firebox=('#2A2624', 'matte'),
    soot=('#1B1716', 'matte'),
    log=('#6B4A33', 'satin'),
    grate=('#2B2C2E', 'metal'),
    flame=('#FF9A3C', 'gloss'),
    flame_core=('#FFD27A', 'gloss'),
    ember=('#D9542B', 'gloss'),
    picture=('#3E7A7A', 'matte'),
    candle=('#EFE5D3', 'satin'),
    # textiles
    linen=('#E7E0D4', 'fabric'),
    rail=('#2B2C2E', 'metal'),
    rug=('#9DAF8C', 'fabric'),
    rug_border=('#7E9271', 'fabric'),
    rug_ground=('#B5653F', 'fabric'),
    rug_edge=('#3E5470', 'fabric'),
    rug_motif=('#E9C46A', 'fabric'),
    rug_accent=('#F2E6CC', 'fabric'),
)

WALL = -0.43   # the wall's face, game depth


# --- flat shapes ------------------------------------------------------------------------------------------
def _mesh(name, verts, faces, material, smooth=0):
    """A mesh object from Blender points and faces, its normals made outward; the first `smooth` faces
    shaded smooth, the rest flat."""
    bm = bmesh.new()
    vs = [bm.verts.new(v) for v in verts]
    for f in faces:
        bm.faces.new([vs[i] for i in f])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for i, f in enumerate(bm.faces):
        f.smooth = i < smooth
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    return obj


def _flat(F, name, pts, z, material, t=0.003):
    """A flat convex shape on a picture: `pts` (game x, height) at game depth `z`, `t` thick, facing the front."""
    n = len(pts)
    verts = [F.at(x, y, z) for x, y in pts] + [F.at(x, y, z + t) for x, y in pts]
    faces = [list(range(n)), list(range(2 * n - 1, n - 1, -1))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return _mesh(name, verts, faces, material)


def _ellipse(cx, cy, rx, ry, a0=0.0, a1=360.0, n=20):
    """Points round an ellipse (degrees from the right, anticlockwise), for `_flat`."""
    return [(cx + rx * math.cos(math.radians(a0 + (a1 - a0) * k / n)), cy + ry * math.sin(math.radians(a0 + (a1 - a0) * k / n)))
            for k in range(n + (0 if a1 - a0 >= 360 else 1))]


def _disc(F, name, x, y, z, r, material, t=0.003, verts=24):
    """A round flat disc facing the front, its back at game depth `z`."""
    return F.cyl(name, r, t, (x, y, z + t / 2), material, rot=(90, 0, 0), verts=verts, smooth=False)


def _leaf(F, name, x, y, z, w, h, ang, material, t=0.004):
    """A flat leaf (an ellipse `w` by `h`) on a picture, turned `ang` degrees in its plane."""
    return G.sphere(name, 0.5, at=F.at(x, y, z + t / 2), material=material, scale=(w, t, h), rot=(0, ang, 0),
                    segments=12, rings=6)


# --- frames -----------------------------------------------------------------------------------------------
def _frame(F, w, h, y0, depth, border, material, bevel=0.004):
    """A picture frame hanging from `y0`, its back on the wall: four mouldings round a backing. Returns the
    opening's middle height, width and height, and the game depth of the backing's face."""
    yc = y0 + h / 2
    z = WALL + depth / 2
    for s in (-1, 1):
        F.box(f'rail{s}', w, border, depth, (0, yc + s * (h - border) / 2, z), material, bevel=bevel)
        F.box(f'stile{s}', border, h - 2 * border + 0.002, depth, (s * (w - border) / 2, yc, z), material, bevel=bevel)
    F.box('backing', w - border, h - border, 0.006, (0, yc, WALL + 0.003), material)
    return yc, w - 2 * border, h - 2 * border, WALL + 0.006


def _mounted(F, yc, iw, ih, z, mount_w, paper):
    """A white mount filling the opening and the picture's paper in its window: the picture's size and face."""
    F.box('mount', iw, ih, 0.004, (0, yc, z + 0.002), M.mount)
    pw, ph = iw - 2 * mount_w, ih - 2 * mount_w
    F.box('paper', pw, ph, 0.002, (0, yc, z + 0.005), paper)
    return pw, ph, z + 0.006


# --- D1: wall art -----------------------------------------------------------------------------------------
def printFramed():
    """A botanical print: a white frame and mount, a leafy stem on cream paper."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.5, 0.71, 1.15, 0.03, 0.03, M.frame_white)
        pw, ph, z = _mounted(F, yc, iw, ih, z, 0.055, M.paper)
        F.box('stem', 0.006, ph * 0.72, 0.003, (0.0, yc - 0.02, z + 0.0015), M.olive, rot=(0, 3, 0))
        for i, (dy, s) in enumerate(((-0.16, -1), (-0.1, 1), (-0.04, -1), (0.02, 1), (0.08, -1), (0.13, 1), (0.17, -1))):
            size = 1 - i * 0.07
            _leaf(F, f'leaf{i}', s * 0.042 * size, yc + dy, z + 0.001 * (i % 2), 0.09 * size, 0.036 * size, s * 32,
                  [M.sage, M.olive][i % 2])
        _leaf(F, 'tip', 0.004, yc + 0.215, z, 0.03, 0.06, 0, M.sage)
    G.join_all('printFramed')
    G.settle(centre=False, floor=False)
    return G.meshes()


def photoPrint():
    """A photo print: a thin black frame and white mount round a mountain lake."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.75, 0.5, 1.25, 0.033, 0.022, M.frame_black, bevel=0.003)
        pw, ph, z = _mounted(F, yc, iw, ih, z, 0.05, M.sky)
        x0, y0 = -pw / 2, yc - ph / 2
        _flat(F, 'peak0', [(x0 + 0.02, y0 + 0.08), (x0 + 0.37, y0 + 0.08), (x0 + 0.21, y0 + 0.29)], z, M.rock)
        _flat(F, 'peak1', [(x0 + 0.24, y0 + 0.08), (x0 + 0.6, y0 + 0.08), (x0 + 0.44, y0 + 0.25)], z + 0.001, M.rock_dark)
        _flat(F, 'snow0', [(x0 + 0.165, y0 + 0.232), (x0 + 0.253, y0 + 0.235), (x0 + 0.21, y0 + 0.29)], z + 0.002, M.snow)
        _flat(F, 'snow1', [(x0 + 0.41, y0 + 0.218), (x0 + 0.475, y0 + 0.22), (x0 + 0.44, y0 + 0.25)], z + 0.002, M.snow)
        F.box('lake', pw, 0.07, 0.002, (0, y0 + 0.065, z + 0.004), M.lake)
        F.box('shore', pw, 0.03, 0.002, (0, y0 + 0.015, z + 0.005), M.shore)
        _disc(F, 'sun', x0 + pw - 0.08, y0 + ph - 0.07, z, 0.026, M.sun)
    G.join_all('photoPrint')
    G.settle(centre=False, floor=False)
    return G.meshes()


def paintingClassic():
    """An oil painting in a deep gilt frame with an inner bead: a cottage by a tree, hills, a field."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.84, 0.65, 1.15, 0.05, 0.07, M.gilt, bevel=0.012)
        for s in (-1, 1):   # the bead round the opening, standing proud of the backing
            F.box(f'bead_rail{s}', iw + 0.02, 0.012, 0.012, (0, yc + s * (ih / 2 + 0.002), z + 0.022), M.gilt_dark, bevel=0.004)
            F.box(f'bead_stile{s}', 0.012, ih + 0.02, 0.012, (s * (iw / 2 + 0.002), yc, z + 0.022), M.gilt_dark, bevel=0.004)
        z += 0.012
        F.box('haze', iw, ih, 0.003, (0, yc, z + 0.0015), M.haze)
        z += 0.003
        y0 = yc - ih / 2
        _flat(F, 'cloud0', _ellipse(-0.1, yc + 0.13, 0.1, 0.028), z, M.cloud)
        _flat(F, 'cloud1', _ellipse(0.13, yc + 0.16, 0.075, 0.022), z, M.cloud)
        rx = iw / 2 + 0.12
        _flat(F, 'hills', [(x, y0 + 0.12 + 0.15 * math.sqrt(max(0.0, 1 - ((x + 0.06) / rx) ** 2)))
                           for x in (iw / 2 - iw * k / 16 for k in range(17))], z + 0.001, M.hills)
        F.box('field', iw, 0.15, 0.002, (0, y0 + 0.075, z + 0.003), M.field)
        F.box('tree_trunk', 0.018, 0.1, 0.002, (-0.17, y0 + 0.17, z + 0.004), M.trunk)
        _flat(F, 'tree', _ellipse(-0.17, y0 + 0.27, 0.075, 0.085), z + 0.005, M.tree)
        F.box('cottage', 0.08, 0.05, 0.002, (0.13, y0 + 0.165, z + 0.004), M.cottage)
        _flat(F, 'roof', [(0.08, y0 + 0.19), (0.18, y0 + 0.19), (0.13, y0 + 0.235)], z + 0.005, M.roof)
    G.join_all('paintingClassic')
    G.settle(centre=False, floor=False)
    return G.meshes()


def canvasAbstract():
    """A big abstract canvas in an oak float frame: colour blocks, a circle and a dark line."""
    F = G.GameFrame()
    w, h, y0 = 1.5, 0.9, 1.05
    yc = y0 + h / 2
    with G.new_parts():
        for s in (-1, 1):   # the float frame: a thin oak edge standing a little off the canvas
            F.box(f'frame_rail{s}', w, 0.014, 0.05, (0, yc + s * (h - 0.014) / 2, WALL + 0.025), M.oak, bevel=0.003)
            F.box(f'frame_stile{s}', 0.014, h - 0.026, 0.05, (s * (w - 0.014) / 2, yc, WALL + 0.025), M.oak, bevel=0.003)
        cw, ch = w - 0.06, h - 0.06
        F.box('canvas', cw, ch, 0.036, (0, yc, WALL + 0.02), M.canvas, bevel=0.004)
        z = WALL + 0.038
        F.box('block', cw * 0.48, ch * 0.72, 0.002, (-cw * 0.17, y0 + 0.03 + ch * 0.46, z + 0.001), M.sage)
        _disc(F, 'circle', cw * 0.2, y0 + 0.03 + ch * 0.6, z + 0.001, ch * 0.24, M.mustard, verts=36)
        F.box('band', cw * 0.3, ch * 0.17, 0.002, (cw * 0.22, y0 + 0.03 + ch * 0.2, z + 0.003), M.rust)
        F.box('line', 0.018, ch * 0.72, 0.002, (-cw * 0.02, y0 + 0.03 + ch * 0.5, z + 0.005), M.ink)
    G.join_all('canvasAbstract')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- D1: the wall clock -----------------------------------------------------------------------------------
CLOCK_Y = 1.9


def wallClock():
    """A round wall clock: a steel case, a white face with ticks, and hands that tell the game's time (ten
    past ten at rest, the catalogue's picture)."""
    F = G.GameFrame()
    r = 0.18
    with G.new_parts() as still:
        # (its well's floor behind the face: at 0.056 the case's front hid the face, so "Face" coloured nothing)
        G.revolve('case', [(0, 0), (r - 0.004, 0), (r, 0.006), (r, 0.058), (r - 0.008, 0.066), (r - 0.018, 0.066),
                           (r - 0.02, 0.046), (0, 0.046)],
                  at=F.at(0, CLOCK_Y, WALL), material=M.clock_case, rot=(90, 0, 0), segments=40)
        _disc(F, 'face', 0, CLOCK_Y, WALL + 0.05, r - 0.022, M.face, t=0.004, verts=40)
        z = WALL + 0.054
        for k in range(12):
            a = math.radians(k * 30)
            big = k % 3 == 0
            d = r - 0.045 if big else r - 0.04
            F.box(f'tick{k}', 0.01 if big else 0.005, 0.03 if big else 0.016, 0.002,
                  (math.sin(a) * d, CLOCK_Y + math.cos(a) * d, z + 0.001), M.ink, rot=(0, -k * 30, 0))
        _disc(F, 'cap', 0, CLOCK_Y, z + 0.007, 0.009, M.second, t=0.004, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    hands = _clock_hands(F, CLOCK_Y, z, ('hour', 0.075, 0.012, 0.003, 0.015), ('minute', 0.115, 0.008, 0.005, 0.018))
    G.settle(centre=False, floor=False)
    _clock_clips(*hands)
    return G.meshes()


# The clocks' hands show the game's time (`motions.clock`, CLOCK): each hand its own part on the dial's
# middle, one turn clockwise from rest a clip, and at rest ten past ten (CLOCK_AT, minutes past twelve).
CLOCK_AT = 10 * 60 + 10
CLOCK = {'clock': {'hour': 'hour_hand', 'minute': 'minute_hand', 'at': CLOCK_AT}}


def _clock_hands(F, y, z, hour, minute, x=0.0, material=None):
    """The hour and minute hands, each (name, length, width, lift above `z`, tail), on a dial centred at
    game (`x`, `y`, `z`), pointing at CLOCK_AT; returned hinged on the dial's middle. (Seen from the front, the
    game's +x is on the left, so angles clockwise from twelve go towards -x.)"""
    out = []
    for (name, length, width, lift, tail), angle in ((hour, -(CLOCK_AT / 2) % 360), (minute, -(CLOCK_AT % 60 * 6))):
        with G.new_parts() as parts:
            a = math.radians(angle)
            mid = (length - tail) / 2
            F.box(name, width, length + tail, 0.002, (x + math.sin(a) * mid, y + math.cos(a) * mid, z + lift),
                  material or M.ink, rot=(0, -angle, 0))
        out.append(G.hinged(G.join(parts, f'{name}_hand'), F.at(x, y, z)))
    return out


def _clock_clips(hour, minute):
    """Each hand once round clockwise (seen from the front); the game holds them at the time."""
    for hand in (hour, minute):
        G.spin(hand, f'{hand.name}', turns=1, seconds=12, axis='Y')


def _books(F, x0, x1, y, z, depth, seed):
    """A row of books standing on a shelf top at `y`, from `x0` to `x1`; the last one leaning."""
    colours = (M.book_teal, M.book_rust, M.book_mustard, M.book_navy, M.book_cream)
    x, k = x0, 0
    while x < x1 - 0.06:
        t = 0.025 + 0.012 * ((k * 7 + seed) % 3)
        h = 0.17 + 0.025 * ((k * 5 + seed) % 4)
        F.box(f'book{seed}_{k}', t, h, depth - 0.01 * (k % 2), (x + t / 2, y + h / 2, z), colours[(k + seed) % 5], bevel=0.003)
        x, k = x + t + 0.002, k + 1
    F.box(f'book{seed}_lean', 0.03, 0.19, depth, (x + 0.05, y + 0.088, z), colours[(k + seed) % 5], rot=(0, -22, 0), bevel=0.003)


def wallShelf():
    """Two floating oak shelves: books, a potted plant, a leaning picture, a vase and a clay ball."""
    F = G.GameFrame()
    zc = WALL + 0.12
    with G.new_parts():
        for y in (1.4, 1.78):
            F.box(f'shelf{y}', 0.9, 0.04, 0.24, (0, y, zc), M.oak, bevel=0.006)
        top0, top1 = 1.42, 1.80
        _books(F, -0.42, -0.06, top0, zc - 0.01, 0.17, 1)
        G.revolve('plant_pot', [(0, 0), (0.045, 0), (0.06, 0.11), (0.055, 0.11), (0.042, 0.01), (0, 0.01)],
                  at=F.at(0.25, top0, zc), material=M.pot, segments=20)
        for k, (dx, dy, dz, r) in enumerate(((0, 0.17, 0, 0.08), (-0.05, 0.14, 0.02, 0.055), (0.05, 0.15, -0.02, 0.06))):
            G.sphere(f'plant{k}', r, at=F.at(0.25 + dx, top0 + dy, zc + dz), material=M.leaf if k else M.leaf_light,
                     scale=(1, 0.9, 0.85), segments=10, rings=6)
        F.box('picture', 0.18, 0.24, 0.016, (-0.25, top1 + 0.12, WALL + 0.035), M.frame_white, rot=(-8, 0, 0), bevel=0.004)
        F.box('picture_art', 0.13, 0.18, 0.004, (-0.25, top1 + 0.122, WALL + 0.045), M.clay, rot=(-8, 0, 0))
        G.revolve('vase', [(0, 0), (0.04, 0), (0.05, 0.05), (0.035, 0.13), (0.022, 0.15), (0.026, 0.16), (0, 0.16)],
                  at=F.at(0.05, top1, zc), material=M.book_cream, segments=18)
        G.sphere('ball', 0.055, at=F.at(0.3, top1 + 0.055, zc), material=M.clay, segments=14, rings=8)
    G.join_all('wallShelf')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- D2: the grandfather clock ----------------------------------------------------------------------------
GF_Z = WALL + 0.2         # the case's middle, game depth (0.4 deep, back on the wall)
TRUNK_FRONT = WALL + 0.35  # the trunk's front face
DIAL_Y = 1.77


def grandfatherClock():
    """A walnut longcase clock: a plinth, a trunk with a glazed door, a pendulum swinging behind it, and a
    hood with a cream dial and brass finials."""
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('plinth', 0.6, 0.06, 0.4, (0, 0.03, GF_Z), M.walnut_dark, bevel=0.008)
        F.box('base', 0.56, 0.38, 0.38, (0, 0.25, GF_Z - 0.01), M.walnut, bevel=0.01)
        F.box('base_panel', 0.4, 0.24, 0.012, (0, 0.25, GF_Z + 0.185), M.walnut_dark, bevel=0.004)
        F.box('waist_moulding', 0.58, 0.04, 0.4, (0, 0.46, GF_Z), M.walnut_dark, bevel=0.01)
        F.box('trunk', 0.46, 1.06, 0.3, (0, 1.0, TRUNK_FRONT - 0.15), M.walnut, bevel=0.008)
        F.box('trunk_hollow', 0.32, 0.8, 0.004, (0, 1.0, TRUNK_FRONT + 0.002), M.hollow)
        F.box('neck_moulding', 0.58, 0.04, 0.4, (0, 1.55, GF_Z), M.walnut_dark, bevel=0.01)
        F.box('hood', 0.54, 0.44, 0.38, (0, 1.79, GF_Z - 0.01), M.walnut, bevel=0.01)
        for s in (-1, 1):
            F.cyl(f'column{s}', 0.016, 0.4, (s * 0.24, 1.79, GF_Z + 0.18), M.brass, verts=10)
        F.box('cornice', 0.6, 0.05, 0.4, (0, 2.035, GF_Z), M.walnut_dark, bevel=0.012)
        for x in (-0.25, 0.0, 0.25):
            G.sphere(f'finial{x}', 0.028, at=F.at(x, 2.088, GF_Z), material=M.brass, segments=10, rings=6)
            F.cyl(f'finial_stem{x}', 0.008, 0.03, (x, 2.07, GF_Z), M.brass, verts=8)
        # the dial in the hood
        dz = GF_Z + 0.18
        _disc(F, 'dial', 0, DIAL_Y, dz, 0.16, M.dial, t=0.006, verts=36)
        G.torus('dial_ring', 0.165, 0.01, at=F.at(0, DIAL_Y, dz + 0.004), material=M.brass, rot=(90, 0, 0),
                major_segments=36, minor_segments=6)
        for k in range(12):
            a = math.radians(k * 30)
            F.box(f'numeral{k}', 0.01, 0.028, 0.002, (math.sin(a) * 0.13, DIAL_Y + math.cos(a) * 0.13, dz + 0.007),
                  M.ink, rot=(0, -k * 30, 0))
        _disc(F, 'arbor', 0, DIAL_Y, dz + 0.014, 0.012, M.brass, t=0.006, verts=10)
        # The trunk door (a walnut frame round an open window) is still: the pendulum and the hands take the draws.
        door_z = TRUNK_FRONT + 0.026
        for s in (-1, 1):
            F.box(f'door_rail{s}', 0.36, 0.06, 0.02, (0, 1.0 + s * 0.41, door_z), M.walnut, bevel=0.005)
            F.box(f'door_stile{s}', 0.05, 0.78, 0.02, (s * 0.155, 1.0, door_z), M.walnut, bevel=0.005)
        F.box('door_knob', 0.016, 0.03, 0.016, (0.15, 1.0, door_z + 0.016), M.brass, bevel=0.005)
    G.join(still, 'body', origin=(0, 0, 0))
    hands = _clock_hands(F, DIAL_Y, dz, ('hour', 0.07, 0.012, 0.008, 0.0), ('minute', 0.11, 0.008, 0.010, 0.0))
    # The pendulum: a brass rod and bob hung from just under the hood, in front of the dark hollow, seen through the
    # door's window; it swings ±4° about its top (`pendulum`, ambient), a beat each 0.8 s, as a pendulum this long does.
    pz = TRUNK_FRONT + 0.008
    with G.new_parts() as pendulum:
        F.box('pendulum_rod', 0.012, 0.58, 0.006, (0, 1.12, pz), M.brass)
        _disc(F, 'pendulum_bob', 0, 0.8, pz - 0.004, 0.065, M.brass, t=0.012, verts=24)
    pendulum = G.hinged(G.join(pendulum, 'pendulum'), F.at(0, 1.41, pz))
    G.settle(centre=False)
    _clock_clips(*hands)
    G.rock(pendulum, 'pendulum', 4, seconds=1.6, axis='Y')
    return G.meshes()


# --- D2: vases and statues --------------------------------------------------------------------------------
def vaseTall():
    """A tall floor vase of pale ceramic with three dried twigs and their pampas plumes."""
    with G.new_parts():
        G.revolve('vase', [(0, 0), (0.07, 0), (0.085, 0.05), (0.1, 0.26), (0.085, 0.45), (0.05, 0.56), (0.048, 0.6),
                           (0.058, 0.62), (0.05, 0.625), (0.04, 0.6), (0, 0.6)], material=M.ceramic, segments=28)
        for k, (x, y, top) in enumerate(((-0.11, 0.04, 1.0), (0.1, -0.03, 1.06), (0.02, 0.07, 0.94))):
            tip = (-x, y, top)   # Blender: the game's x mirrored
            G.rod(f'twig{k}', (0, 0, 0.55), tip, 0.006, material=M.twig, verts=6)
            plume = G.sphere(f'plume{k}', 0.045, at=(-x * 1.1, y * 1.1, top + 0.07), material=M.plume, scale=(1, 1, 3.2),
                             segments=10, rings=8)
            plume.rotation_mode = 'QUATERNION'
            plume.rotation_quaternion = (Vector(tip) - Vector((0, 0, 0.55))).to_track_quat('Z', 'Y')   # along its twig
    G.join_all('vaseTall')
    G.settle()
    return G.meshes()


JAR_SIDE = [(0.15, 0.17), (0.27, 0.18), (0.39, 0.16)]   # the ginger jar's radius at heights (its profile)


def _jar_radius(y):
    for (y0, r0), (y1, r1) in zip(JAR_SIDE, JAR_SIDE[1:]):
        if y <= y1:
            return r0 + (r1 - r0) * (y - y0) / (y1 - y0)
    return JAR_SIDE[-1][1]


def _sprig(k, a0):
    """A sprig of blossom painted on the jar's side: a curving stem, three leaves along it and a five-petalled
    flower at its tip, each a thin shape lying on the porcelain."""
    pts = []
    for i in range(5):
        t = i / 4
        y = 0.17 + 0.15 * t
        a = a0 + 0.3 * t + 0.08 * math.sin(math.pi * t)
        r = _jar_radius(y) + 0.002
        pts.append(((math.cos(a) * r, math.sin(a) * r, y), a))
    for i in range(4):
        G.rod(f'stem{k}_{i}', pts[i][0], pts[i + 1][0], 0.0035, material=M.blue, verts=4)

    def on_side(p, a, along, up):
        """A point `along` the jar's turn and `up` from `p`, on its side."""
        return (p[0] - math.sin(a) * along, p[1] + math.cos(a) * along, p[2] + up)

    for i, side in ((1, 1), (2, -1), (3, 1)):   # each leaf from the stem, tipped 50 degrees off it
        p, a = pts[i]
        tilt = math.radians(side * 50)
        G.sphere(f'leaf{k}_{i}', 0.5, at=on_side(p, a, -math.sin(tilt) * 0.02, math.cos(tilt) * 0.02), material=M.blue,
                 scale=(0.006, 0.014, 0.04), rot=(side * 50, 0, math.degrees(a)), segments=6, rings=4)
    p, a = pts[4]
    for j in range(5):
        b = math.radians(j * 72 + 90)
        G.sphere(f'petal{k}_{j}', 0.5, at=on_side(p, a, math.cos(b) * 0.011, math.sin(b) * 0.011), material=M.blue,
                 scale=(0.006, 0.013, 0.013), rot=(0, 0, math.degrees(a)), segments=6, rings=3)


def urnPorcelain():
    """A porcelain ginger jar, white with blue bands and sprigs, and its lid."""
    with G.new_parts():
        G.revolve('jar', [(0, 0), (0.105, 0), (0.12, 0.02), (0.17, 0.15), (0.18, 0.27), (0.16, 0.39), (0.11, 0.46),
                          (0.1, 0.5), (0, 0.5)], material=M.porcelain, segments=32)
        G.revolve('lid', [(0, 0), (0.108, 0), (0.106, 0.02), (0.08, 0.06), (0.03, 0.085), (0, 0.085)],
                  at=(0, 0, 0.495), material=M.porcelain, segments=28)
        G.sphere('knob', 0.025, at=(0, 0, 0.595), material=M.blue, segments=12, rings=8)
        for k, (y, r) in enumerate(((0.012, 0.11), (0.39, 0.16), (0.5, 0.104))):
            G.torus(f'band{k}', r, 0.007, at=(0, 0, y), material=M.blue, major_segments=32, minor_segments=6)
        for k in range(5):
            _sprig(k, math.radians(k * 72 + 20))
    G.join_all('urnPorcelain')
    G.settle()
    return G.meshes()


def _plinth(h, w, material, top=None):
    """A square plinth `h` tall and `w` wide, with a skirting and a cap a little wider."""
    G.cube('plinth', (w, w, h), at=(0, 0, h / 2), material=material, bevel=0.008)
    G.cube('plinth_foot', (w + 0.04, w + 0.04, 0.04), at=(0, 0, 0.02), material=material, bevel=0.008)
    G.cube('plinth_cap', (w + 0.04, w + 0.04, 0.03), at=(0, 0, h - 0.015), material=top or material, bevel=0.008)


def vasePedestal():
    """A blue glazed vase of white blooms on a stone pedestal."""
    y = 0.86
    with G.new_parts():
        _plinth(y, 0.36, M.stone)
        G.revolve('vase', [(0, 0), (0.07, 0), (0.11, 0.08), (0.12, 0.17), (0.09, 0.27), (0.06, 0.32), (0.07, 0.34),
                           (0.055, 0.34), (0, 0.3)], at=(0, 0, y), material=M.vase_blue, segments=28)
        for k, (x, d, top) in enumerate(((-0.08, 0.02, 1.48), (0.06, -0.04, 1.54), (0.02, 0.07, 1.42), (-0.03, -0.06, 1.36),
                                         (0.1, 0.05, 1.32))):
            tip = (-x, -d, top)
            G.rod(f'stem{k}', (0, 0, y + 0.28), tip, 0.006, material=M.stem, verts=6)
            G.sphere(f'bloom{k}', 0.05, at=(-x, -d, top + 0.02), material=M.bloom, scale=(1, 1, 0.8), segments=10, rings=6)
            G.sphere(f'leaf{k}', 0.04, at=(-x * 0.6, -d * 0.6, y + 0.3 + (top - y - 0.3) * 0.45), material=M.stem,
                     scale=(1.4, 0.5, 0.35), rot=(0, 20, k * 70), segments=8, rings=4)
    G.join_all('vasePedestal')
    G.settle()
    return G.meshes()


def bustMarble():
    """A marble bust on a stone plinth: a turned socle, shoulders, neck, head, hair, nose and ears."""
    y = 1.0
    with G.new_parts():
        _plinth(y, 0.4, M.stone)
        G.revolve('socle', [(0, 0), (0.09, 0), (0.09, 0.025), (0.055, 0.05), (0.045, 0.08), (0, 0.08)],
                  at=(0, 0, y), material=M.marble, segments=24)
        G.sphere('shoulders', 0.5, at=(0, 0, y + 0.17), material=M.marble, scale=(0.3, 0.17, 0.2), segments=24, rings=12)
        G.cylinder('neck', 0.04, 0.12, at=(0, 0, y + 0.3), material=M.marble, verts=14)
        G.sphere('head', 0.5, at=(0, -0.01, y + 0.42), material=M.marble, scale=(0.15, 0.17, 0.19), segments=20, rings=12)
        G.boulder('hair', 0.085, at=(0, 0.012, y + 0.4), material=M.marble, scale=(0.95, 1.05, 0.8), seed=3.0, noise=0.18,
                  subdivisions=2)
        G.cube('nose', (0.022, 0.03, 0.045), at=(0, -0.093, y + 0.42), material=M.marble, rot=(-15, 0, 0), bevel=0.008)
        for s in (-1, 1):
            G.sphere(f'ear{s}', 0.5, at=(s * 0.075, 0, y + 0.42), material=M.marble, scale=(0.025, 0.035, 0.05), segments=8, rings=6)
    G.join_all('bustMarble')
    G.settle()
    return G.meshes()


def statueCat():
    """A concrete cat sitting up, its tail curled round its front paws."""
    with G.new_parts():
        # sitting: the rump, a thigh bulging at either side and the hind feet tucked forward by the front paws
        G.sphere('rump', 0.5, at=(0, 0.07, 0.12), material=M.concrete, scale=(0.22, 0.26, 0.23), segments=20, rings=12)
        for s in (-1, 1):
            G.sphere(f'thigh{s}', 0.5, at=(s * 0.085, 0.03, 0.1), material=M.concrete, scale=(0.11, 0.22, 0.17),
                     rot=(-20, 0, 0), segments=16, rings=10)
            G.sphere(f'hind_foot{s}', 0.5, at=(s * 0.1, -0.08, 0.018), material=M.concrete, scale=(0.055, 0.1, 0.036),
                     segments=10, rings=6)
        G.sphere('chest', 0.5, at=(0, -0.04, 0.26), material=M.concrete, scale=(0.19, 0.18, 0.32), rot=(-12, 0, 0), segments=18, rings=12)
        G.sphere('head', 0.5, at=(0, -0.07, 0.46), material=M.concrete, scale=(0.19, 0.17, 0.16), segments=18, rings=12)
        G.sphere('muzzle', 0.5, at=(0, -0.145, 0.43), material=M.concrete, scale=(0.08, 0.05, 0.05), segments=12, rings=8)
        G.sphere('nose', 0.5, at=(0, -0.168, 0.445), material=M.concrete_dark, scale=(0.02, 0.012, 0.012), segments=8, rings=4)
        for s in (-1, 1):
            G.sphere(f'eye{s}', 0.5, at=(s * 0.04, -0.152, 0.475), material=M.concrete_dark, scale=(0.025, 0.01, 0.018), segments=8, rings=4)
            G.cone(f'ear{s}', 0.035, 0.004, 0.075, at=(s * 0.055, -0.065, 0.56), material=M.concrete, rot=(0, s * 14, 0), verts=8)
            G.cylinder(f'leg{s}', 0.024, 0.22, at=(s * 0.045, -0.12, 0.11), material=M.concrete, verts=10)
            G.sphere(f'paw{s}', 0.5, at=(s * 0.045, -0.15, 0.017), material=M.concrete, scale=(0.06, 0.07, 0.035), segments=10, rings=6)
        G.rod('tail0', (-0.1, 0.16, 0.03), (-0.15, -0.03, 0.03), 0.022, material=M.concrete, verts=10)
        G.rod('tail1', (-0.15, -0.03, 0.03), (-0.07, -0.19, 0.03), 0.02, material=M.concrete, verts=10)
        G.sphere('tail_tip', 0.02, at=(-0.07, -0.19, 0.03), material=M.concrete, segments=10, rings=6)
        G.sphere('tail_bend', 0.022, at=(-0.15, -0.03, 0.03), material=M.concrete, segments=10, rings=6)
    G.join_all('statueCat')
    G.settle()
    return G.meshes()


def statueHorse():
    """A bronze horse on an oak plinth, standing, its head up."""
    y = 0.92
    with G.new_parts():
        _plinth(y, 0.4, M.oak)
        G.cube('base', (0.34, 0.16, 0.02), at=(0, 0, y + 0.01), material=M.bronze, bevel=0.006)
        G.sphere('barrel', 0.5, at=(0, 0, y + 0.27), material=M.bronze, scale=(0.3, 0.11, 0.12), segments=18, rings=10)
        G.rod('neck', (-0.1, 0, y + 0.28), (-0.155, 0, y + 0.42), 0.035, material=M.bronze, verts=12)
        G.rod('head', (-0.14, 0, y + 0.43), (-0.215, 0, y + 0.36), 0.026, material=M.bronze, verts=12)
        G.sphere('muzzle', 0.026, at=(-0.215, 0, y + 0.36), material=M.bronze, segments=10, rings=6)
        G.cube('mane', (0.02, 0.012, 0.14), at=(-0.115, 0, y + 0.39), material=M.bronze_dark, rot=(0, 25, 0), bevel=0.004)
        for s in (-1, 1):
            G.cone(f'ear{s}', 0.012, 0.002, 0.04, at=(-0.14, s * 0.014, y + 0.47), material=M.bronze, verts=6)
        G.rod('tail', (0.14, 0, y + 0.3), (0.19, 0, y + 0.13), 0.015, material=M.bronze_dark, verts=8)
        for x, dx in ((-0.1, -0.02), (0.1, 0.01)):
            for d in (-0.035, 0.035):
                G.rod(f'leg{x}{d}', (x, d, y + 0.24), (x + dx, d, y + 0.02), 0.014, material=M.bronze, verts=8)
    G.join_all('statueHorse')
    G.settle()
    return G.meshes()


COAT_AT, COAT_HEM, COAT_TURN = (0.1, 0.07), 0.97, 30   # where the coat hangs (game x, depth), its hem's height, its turn


def _coat(F):
    """A coat hung by its collar: a body flaring a little to the hem, sleeves hanging at its sides, a collar."""
    c, turn = F.at(COAT_AT[0], 0, COAT_AT[1]), math.radians(COAT_TURN)

    def at(x, y, h):
        """A point in the coat's own frame (Blender x across it, y through it, h up from the hem)."""
        return (c[0] + x * math.cos(turn) - y * math.sin(turn), c[1] + x * math.sin(turn) + y * math.cos(turn), COAT_HEM + h)

    G.revolve('coat', [(0, 0), (0.125, 0), (0.13, 0.015), (0.12, 0.1), (0.108, 0.3), (0.112, 0.45), (0.105, 0.53),
                       (0.075, 0.585), (0.035, 0.6), (0, 0.6)],
              at=at(0, 0, 0), material=M.coat, segments=20, scale=(1, 0.45, 1), rot=(0, 0, COAT_TURN))
    G.sphere('coat_collar', 0.5, at=at(0, 0, 0.585), material=M.coat, scale=(0.13, 0.075, 0.05), rot=(0, 0, COAT_TURN),
             segments=12, rings=6)
    for s in (-1, 1):
        G.rod(f'sleeve{s}', at(s * 0.1, -0.005, 0.53), at(s * 0.12, -0.02, 0.14), 0.026, material=M.coat, verts=10)
        G.sphere(f'cuff{s}', 0.028, at=at(s * 0.12, -0.02, 0.14), material=M.coat, scale=(1, 1, 0.6), segments=10, rings=6)


def coatRack():
    """A turned oak coat stand on four splayed feet, brass hooks, a coat, a scarf and a hat."""
    F = G.GameFrame()
    with G.new_parts():
        F.cone('pole', 0.025, 0.019, 1.6, (0, 0.95, 0), M.oak, verts=14)
        F.sphere('top', 0.038, (0, 1.75, 0), M.oak, segments=14, rings=8)
        F.cyl('hub', 0.04, 0.08, (0, 0.2, 0), M.oak, verts=14, bevel=0.01)
        for k in range(4):
            a = math.radians(45 + k * 90)
            foot = F.at(math.cos(a) * 0.2, 0.015, math.sin(a) * 0.2)
            G.rod(f'leg{k}', F.at(0, 0.22, 0), foot, 0.016, material=M.oak, verts=8)
            G.sphere(f'foot{k}', 0.022, at=foot, material=M.oak, scale=(1, 1, 0.7), segments=10, rings=6)
        for k, y in enumerate((1.62, 1.62, 1.62, 1.62, 1.44, 1.44)):
            a = math.radians(k * 90 + (45 if k > 3 else 0) + (180 if k == 5 else 0))
            tip = F.at(math.cos(a) * 0.12, y + 0.07, math.sin(a) * 0.12)
            G.rod(f'hook{k}', F.at(0, y, 0), tip, 0.006, material=M.brass, verts=6)
            G.sphere(f'hook_tip{k}', 0.012, at=tip, material=M.brass, segments=8, rings=4)
        # a coat hanging from a hook on the right-front, a scarf on the left, a hat on the top hook
        _coat(F)
        F.box('scarf', 0.07, 0.56, 0.018, (-0.1, 1.27, -0.07), M.scarf, rot=(4, 0, -40), bevel=0.006)
        F.cyl('hat_brim', 0.12, 0.012, (-0.09, 1.69, -0.06), M.hat, verts=20)
        F.cone('hat_crown', 0.075, 0.062, 0.09, (-0.09, 1.74, -0.06), M.hat, verts=18)
    G.join_all('coatRack')
    G.settle()
    return G.meshes()


# --- D2: the hanging plant --------------------------------------------------------------------------------
CEILING = 2.8


def hangingPlant():
    """A pot in a macramé hanger on a cord from the ceiling, leafy, its vines trailing. It sways when
    watered: the whole hanger swings about the hook."""
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('hook_plate', 0.03, 0.012, (0, CEILING - 0.006, 0), M.cord, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.cyl('cord', 0.006, 0.98, (0, CEILING - 0.5, 0), M.cord, verts=6)
        G.torus('ring', 0.016, 0.004, at=F.at(0, 1.81, 0), material=M.cord, rot=(90, 0, 0), major_segments=10, minor_segments=4)
        for k in range(3):
            a = math.radians(90 + k * 120)
            G.rod(f'hanger{k}', F.at(0, 1.8, 0), F.at(math.cos(a) * 0.165, 1.42, math.sin(a) * 0.165), 0.005, material=M.cord, verts=6)
            G.rod(f'hanger_low{k}', F.at(math.cos(a) * 0.165, 1.42, math.sin(a) * 0.165),
                  F.at(math.cos(a) * 0.105, 1.2, math.sin(a) * 0.105), 0.005, material=M.cord, verts=6)
        G.revolve('pot', [(0, 0), (0.1, 0), (0.15, 0.13), (0.16, 0.24), (0.145, 0.24), (0.135, 0.13), (0, 0.13)],
                  at=F.at(0, 1.2, 0), material=M.pot_white, segments=24)
        G.torus('pot_band', 0.152, 0.008, at=F.at(0, 1.38, 0), material=M.cord, major_segments=24, minor_segments=4)
        for k, (dx, dy, dz, r) in enumerate(((0, 1.5, 0, 0.17), (-0.12, 1.47, 0.06, 0.11), (0.12, 1.46, -0.05, 0.11),
                                             (0.04, 1.47, 0.13, 0.1), (-0.04, 1.46, -0.13, 0.1))):
            G.sphere(f'leaves{k}', r, at=F.at(dx, dy, dz), material=M.leaf, scale=(1, 1, 0.75), segments=12, rings=8)
        for k, (x, z, length) in enumerate(((0.18, 0.08, 0.45), (-0.17, -0.06, 0.38), (0.05, -0.17, 0.3), (-0.06, 0.16, 0.34))):
            G.sphere(f'vine{k}', 0.5, at=F.at(x, 1.42 - length / 2, z), material=M.leaf_light,
                     scale=(0.05, 0.05, length), segments=8, rings=8)
    plant = G.hinged(G.join(parts, 'plant'), F.at(0, CEILING, 0))
    G.settle(floor=False)
    # one sway as it's watered: out (eased) and, played back after the hold, home; a rock (out, past rest and
    # home) played back would swing it twice
    G.swing(plant, 'sway', 5, seconds=0.8, axis='X')
    return G.meshes()


# --- D2: the fireplace ------------------------------------------------------------------------------------
HEARTH = 0.42   # the raised hearth's top: the content's `seat` height, where residents warming up sit


def fireplace():
    """A stone chimney breast with a mantel shelf, a raised hearth bench, a dark firebox above it with logs
    on a grate, and things on the mantel. The flames grow on the logs while someone warms up by it."""
    F = G.GameFrame()
    with G.new_parts() as still:
        # the chimney breast, built round the firebox's opening (x ±0.44, from the hearth bench up 0.66 m)
        top = HEARTH + 0.66
        for side in (-1, 1):
            F.box(f'breast{side}', 0.48, 1.25, 0.4, (side * 0.68, 0.625, WALL + 0.2), M.surround, bevel=0.012)
        F.box('breast_top', 0.9, 1.25 - top, 0.4, (0, (1.25 + top) / 2, WALL + 0.2), M.surround, bevel=0.012)
        F.box('firebox_back', 0.9, top - HEARTH + 0.02, 0.1, (0, (top + HEARTH) / 2, WALL + 0.05), M.soot)
        for side in (-1, 1):
            F.box(f'firebox_side{side}', 0.012, top - HEARTH, 0.3, (side * 0.434, (top + HEARTH) / 2, WALL + 0.25), M.firebox)
        F.box('firebox_top', 0.88, 0.012, 0.3, (0, top - 0.006, WALL + 0.25), M.firebox)
        F.box('flue', 1.6, 1.55, 0.28, (0, 2.025, WALL + 0.14), M.surround, bevel=0.012)
        F.box('mantel', 1.96, 0.07, 0.48, (0, 1.285, WALL + 0.24), M.oak, bevel=0.01)
        for side in (-1, 1):
            F.box(f'corbel{side}', 0.12, 0.08, 0.06, (side * 0.8, 1.21, WALL + 0.43), M.oak, bevel=0.008)
        # the raised hearth: a stone bench along the front, the firebox's floor; residents warming up sit on it
        F.box('hearth', 1.96, HEARTH, 0.57, (0, HEARTH / 2, WALL + 0.285), M.hearth, bevel=0.01)
        F.box('surround_lintel', 1.08, 0.1, 0.03, (0, top + 0.05, WALL + 0.415), M.surround, bevel=0.008)
        for side in (-1, 1):
            F.box(f'surround_jamb{side}', 0.1, top - HEARTH, 0.03, (side * 0.49, (top + HEARTH) / 2, WALL + 0.415), M.surround, bevel=0.008)
        # the grate and the logs
        y = HEARTH
        for s in (-1, 1):
            F.box(f'grate_bar{s}', 0.6, 0.012, 0.012, (0, y + 0.05, WALL + 0.25 + s * 0.08), M.grate, bevel=0.003)
            F.box(f'grate_leg{s}', 0.012, 0.04, 0.2, (s * 0.28, y + 0.03, WALL + 0.25), M.grate, bevel=0.003)
        G.rod('log0', F.at(-0.25, y + 0.09, WALL + 0.21), F.at(0.2, y + 0.09, WALL + 0.29), 0.045, material=M.log, verts=10)
        G.rod('log1', F.at(-0.18, y + 0.09, WALL + 0.31), F.at(0.26, y + 0.09, WALL + 0.2), 0.042, material=M.log, verts=10)
        G.rod('log2', F.at(-0.12, y + 0.16, WALL + 0.25), F.at(0.14, y + 0.17, WALL + 0.26), 0.038, material=M.log, verts=10)
        # on the mantel
        top = 1.32
        G.revolve('vase', [(0, 0), (0.04, 0), (0.055, 0.07), (0.04, 0.18), (0.028, 0.22), (0.034, 0.24), (0, 0.24)],
                  at=F.at(-0.65, top, WALL + 0.2), material=M.pot, segments=18)
        for k, (x, h) in enumerate(((-0.4, 0.12), (-0.3, 0.18))):
            F.cyl(f'candle{k}', 0.028, h, (x, top + h / 2, WALL + 0.2), M.candle, verts=12)
            F.cyl(f'candle_holder{k}', 0.04, 0.015, (x, top + 0.0075, WALL + 0.2), M.brass, verts=12)
        F.box('picture_frame', 0.3, 0.38, 0.025, (0.55, top + 0.19, WALL + 0.06), M.frame_white, rot=(-6, 0, 0), bevel=0.005)
        F.box('picture_art', 0.22, 0.3, 0.004, (0.55, top + 0.192, WALL + 0.074), M.picture, rot=(-6, 0, 0))
    G.join(still, 'body', origin=(0, 0, 0))

    # the flames: an ember bed and tongues of fire, joined at the logs' top, grown from nothing
    base = HEARTH + 0.12
    with G.new_parts() as parts:
        F.box('embers', 0.5, 0.02, 0.16, (0, base, WALL + 0.25), M.ember, bevel=0.008)
        for k, (x, h, r, z) in enumerate(((-0.14, 0.26, 0.07, 0.24), (0.0, 0.36, 0.09, 0.26), (0.13, 0.28, 0.07, 0.25),
                                          (-0.06, 0.2, 0.05, 0.3), (0.07, 0.22, 0.05, 0.21))):
            F.cone(f'flame{k}', r, 0.004, h, (x, base + h / 2, WALL + z), M.flame, verts=10, scale=(1, 0.45, 1))
        for k, (x, h, r) in enumerate(((-0.07, 0.17, 0.045), (0.06, 0.21, 0.05))):
            F.cone(f'flame_core{k}', r, 0.003, h, (x, base + h / 2, WALL + 0.24), M.flame_core, verts=10, scale=(1, 0.45, 1))
    flames = G.join(parts, 'flames', origin=F.at(0, base, WALL + 0.25))
    flames.scale[2] = 0.001
    G.settle(centre=False)
    G.grow(flames, 'flames', seconds=0.8)
    return G.meshes()


# --- D3: curtains -----------------------------------------------------------------------------------------
def _pleats(name, x_out, x_in, y0, y1, z, material, waves=4, amp=0.018, t=0.008, F=None):
    """A gathered curtain panel from game x `x_out` to `x_in` and height `y0` to `y1`, its pleats a sine
    `amp` deep round game depth `z`, `t` thick."""
    n = waves * 8
    xs = [x_out + (x_in - x_out) * k / n for k in range(n + 1)]
    front = [(x, z + t / 2 + amp * math.sin(2 * math.pi * waves * k / n)) for k, x in enumerate(xs)]
    back = [(x, d - t) for x, d in front]
    ring = front + back[::-1]
    m = len(ring)
    verts = [F.at(x, y0, d) for x, d in ring] + [F.at(x, y1, d) for x, d in ring]
    faces = [(i, (i + 1) % m, m + (i + 1) % m, m + i) for i in range(m)]
    for base in (0, m):   # the bottom and top edges: a strip of quads across the cloth's thickness
        faces += [(base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k) for k in range(n)]
    return _mesh(name, verts, faces, material, smooth=m)   # the cloth smooth, its hems flat


def _curtains(w, top, bottom, gathered):
    """Two pleated panels on a rail, gathered open at the sides (rest). At night `close` draws them across:
    each widens from its outer end until they meet in the middle."""
    F = G.GameFrame()
    rail_y, z = top + 0.05, WALL + 0.08
    with G.new_parts() as still:
        F.cyl('rail', 0.012, w + 0.06, (0, rail_y, z), M.rail, rot=(0, 90, 0), verts=12)
        for s in (-1, 1):
            F.sphere(f'finial{s}', 0.024, (s * (w / 2 + 0.04), rail_y, z), M.rail, segments=12, rings=8)
            F.box(f'bracket{s}', 0.016, 0.016, z - WALL, (s * (w / 2 - 0.02), rail_y, (z + WALL) / 2), M.rail, bevel=0.004)
            F.cyl(f'bracket_plate{s}', 0.022, 0.008, (s * (w / 2 - 0.02), rail_y, WALL + 0.004), M.rail, rot=(90, 0, 0), verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    panels = []
    for s in (-1, 1):
        x_out = s * w / 2
        with G.new_parts() as parts:
            _pleats(f'panel{s}', x_out, x_out - s * gathered, bottom, top + 0.02, z, M.linen, waves=max(3, round(gathered / 0.06)), F=F)
            for k in range(4):
                # cloth tabs over the rail (the panel's own fabric: no draw call of their own)
                G.torus(f'tab{s}_{k}', 0.018, 0.006, at=F.at(x_out - s * (0.02 + k * (gathered - 0.04) / 3), rail_y, z),
                        material=M.linen, rot=(0, 90, 0), major_segments=10, minor_segments=4)
        panels.append(G.hinged(G.join(parts, 'panel_left' if s < 0 else 'panel_right'), F.at(x_out, rail_y, z)))
    G.settle(centre=False, floor=False)
    for panel in panels:
        G.squash(panel, 'close', ((w / 2 + 0.01) / gathered, 1, 1), seconds=1.2)
    return G.meshes()


def curtains():
    return _curtains(0.86, 2.45, 1.0, 0.2)


def curtainsLong():
    return _curtains(1.8, 2.55, 0.04, 0.34)


# --- D3: rugs ---------------------------------------------------------------------------------------------
def _rug_rect(name, sx, sz, field, border, line):
    """A flat rug `sx` wide and `sz` deep: a border, a raised field and a line round it, 2 cm high."""
    G.cube(f'{name}_border', (sx, sz, 0.012), at=(0, 0, 0.006), material=border, bevel=0.005, bevel_segments=3)
    G.cube(f'{name}_field', (sx - 0.24, sz - 0.24, 0.006), at=(0, 0, 0.013), material=field, bevel=0.002)
    lw = 0.03
    for s in (-1, 1):
        G.cube(f'{name}_line_x{s}', (sx - 0.44, lw, 0.004), at=(0, s * (sz - 0.44 - lw) / 2, 0.017), material=line, bevel=0.0015)
        G.cube(f'{name}_line_z{s}', (lw, sz - 0.44 - 2 * lw, 0.004), at=(s * (sx - 0.44 - lw) / 2, 0, 0.017), material=line, bevel=0.0015)


def rugRound():
    """A round rug: a darker border, a field and a thin ring."""
    d = 1.9
    with G.new_parts():
        G.cylinder('border', d / 2, 0.012, at=(0, 0, 0.006), material=M.rug_border, verts=64, bevel=0.005, bevel_segments=3)
        G.cylinder('field', d / 2 - 0.12, 0.006, at=(0, 0, 0.013), material=M.rug, verts=64, bevel=0.002)
        G.revolve('line', [(d / 2 - 0.25, 0.015), (d / 2 - 0.22, 0.015), (d / 2 - 0.22, 0.019), (d / 2 - 0.25, 0.019),
                           (d / 2 - 0.25, 0.015)], material=M.rug_border, segments=64, smooth=False)
    G.join_all('rugRound')
    G.settle()
    return G.meshes()


def rugLarge():
    with G.new_parts():
        _rug_rect('rug', 2.85, 1.85, M.rug, M.rug_border, M.rug_border)
    G.join_all('rugLarge')
    G.settle()
    return G.meshes()


def rugRunner():
    """A hall runner: border, field and line, and a fringe on each short end."""
    with G.new_parts():
        _rug_rect('rug', 0.8, 2.68, M.rug, M.rug_border, M.rug_border)
        for s in (-1, 1):
            for i in range(9):
                G.cube(f'tassel{s}_{i}', (0.022, 0.06, 0.005), at=(-0.32 + i * 0.08, s * 1.37, 0.0025), material=M.rug_border)
    G.join_all('rugRunner')
    G.settle()
    return G.meshes()


def rugPattern():
    """A patterned rug: a navy border, a terracotta ground, a cream line inside, a large diamond medallion
    with two smaller ones, and a fringe on each short end."""
    sx, sz = 2.7, 1.86
    with G.new_parts():
        G.cube('border', (sx, sz, 0.012), at=(0, 0, 0.006), material=M.rug_edge, bevel=0.005, bevel_segments=3)
        G.cube('ground', (sx - 0.26, sz - 0.26, 0.006), at=(0, 0, 0.013), material=M.rug_ground, bevel=0.002)
        for s in (-1, 1):
            G.cube(f'line_x{s}', (sx - 0.46, 0.04, 0.004), at=(0, s * (sz - 0.5) / 2, 0.017), material=M.rug_accent, bevel=0.0015)
            G.cube(f'line_z{s}', (0.04, sz - 0.54, 0.004), at=(s * (sx - 0.5) / 2, 0, 0.017), material=M.rug_accent, bevel=0.0015)
        G.cube('medallion', (0.66, 0.66, 0.004), at=(0, 0, 0.017), material=M.rug_motif, rot=(0, 0, 45), bevel=0.0015)
        G.cube('medallion_core', (0.38, 0.38, 0.004), at=(0, 0, 0.0195), material=M.rug_accent, rot=(0, 0, 45), bevel=0.0015)
        G.cube('medallion_eye', (0.14, 0.14, 0.004), at=(0, 0, 0.022), material=M.rug_edge, rot=(0, 0, 45), bevel=0.0015)
        for s in (-1, 1):
            G.cube(f'side{s}', (0.34, 0.34, 0.004), at=(s * 0.82, 0, 0.017), material=M.rug_motif, rot=(0, 0, 45), bevel=0.0015)
            G.cube(f'side_core{s}', (0.14, 0.14, 0.004), at=(s * 0.82, 0, 0.0195), material=M.rug_accent, rot=(0, 0, 45), bevel=0.0015)
            for i in range(18):
                G.cube(f'tassel{s}_{i}', (0.07, 0.022, 0.005), at=(s * (sx / 2 + 0.035), -0.85 + i * 0.1, 0.0025), material=M.rug_accent)
    G.join_all('rugPattern')
    G.settle()
    return G.meshes()


# --- the family -------------------------------------------------------------------------------------------
FAMILY = G.Family(
    'decor', M,
    [  # D1. Wall art and clocks
     G.Model('printFramed', printFramed, size=(0.5, 0.71, 0.03), key='model.printFramed',
             centre=(0, -0.415), floor=False, paint=[('frame_white', 'Frame')]),
     G.Model('photoPrint', photoPrint, size=(0.75, 0.5, 0.033), key='model.photoPrint',
             centre=(0, -0.4135), floor=False, paint=[('frame_black', 'Frame')]),
     G.Model('paintingClassic', paintingClassic, size=(0.84, 0.65, 0.05), key='model.paintingClassic',
             centre=(0, -0.405), floor=False, paint=[('gilt', 'Frame')]),
     G.Model('canvasAbstract', canvasAbstract, size=(1.5, 0.9, 0.05), key='model.canvasAbstract',
             centre=(0, -0.405), floor=False, paint=[('oak', 'Frame')]),
     G.Model('wallClock', wallClock, size=(0.36, 0.36, 0.066), key='model.wallClock',
             centre=(0, -0.397), floor=False, motions=CLOCK, paint=[('clock_case', 'Case'), ('face', 'Face')]),
     G.Model('wallShelf', wallShelf, size=(0.9, 0.659, 0.24), key='model.wallShelf',
             centre=(0, -0.31), floor=False, paint=[('oak', 'Shelves')]),
     # D2. Ornaments
     G.Model('grandfatherClock', grandfatherClock, size=(0.6, 2.116, 0.4), key='model.grandfatherClock',
             centre=(0, -0.23), motions={**CLOCK, 'idle': 'pendulum'},
             paint=[('walnut', 'Case'), ('dial', 'Face')]),
     G.Model('vaseTall', vaseTall, size=(0.329, 1.27, 0.224), key='model.vaseTall', paint=[('ceramic', 'Body')]),
     G.Model('urnPorcelain', urnPorcelain, size=(0.36, 0.62, 0.363), key='model.urnPorcelain',
             paint=[('porcelain', 'Body')]),
     G.Model('vasePedestal', vasePedestal, size=(0.4, 1.6, 0.4), key='model.vasePedestal',
             paint=[('vase_blue', 'Vase'), ('stone', 'Pedestal')]),
     G.Model('bustMarble', bustMarble, size=(0.44, 1.539, 0.44), key='model.bustMarble',
             paint=[('marble', 'Bust'), ('stone', 'Plinth')]),
     G.Model('statueCat', statueCat, size=(0.311, 0.598, 0.41), key='model.statueCat', paint=[('concrete', 'Body')]),
     G.Model('statueHorse', statueHorse, size=(0.46, 1.41, 0.44), key='model.statueHorse',
             paint=[('bronze', 'Body')]),
     G.Model('coatRack', coatRack, size=(0.431, 1.788, 0.355), key='model.coatRack', paint=[('oak', 'Body')]),
     G.Model('hangingPlant', hangingPlant, size=(0.46, 1.83, 0.46), key='model.hangingPlant', floor=False,
             motions={'uses': {'water': 'sway'}, 'hold': 0.8}, paint=[('pot_white', 'Pot'), ('cord', 'Cord')]),
     G.Model('fireplace', fireplace, size=(1.96, 2.8, 0.57), key='model.fireplace', centre=(0, -0.145),
             motions={'uses': {'relax': 'flames'}}, paint=[('surround', 'Surround'), ('oak', 'Mantel')]),
     # D3. Textiles
     G.Model('curtains', curtains, size=(0.988, 1.524, 0.104), key='model.curtains', centre=(0, -0.378),
             floor=False, motions={'night': 'close'}, paint=[('linen', 'Fabric'), ('rail', 'Rail')]),
     G.Model('curtainsLong', curtainsLong, size=(1.928, 2.584, 0.104), key='model.curtainsLong', centre=(0, -0.378),
             floor=False, motions={'night': 'close'}, paint=[('linen', 'Fabric'), ('rail', 'Rail')]),
     G.Model('rugRound', rugRound, size=(1.9, 0.019, 1.9), key='model.rugRound', paint=[('rug', 'Rug')]),
     G.Model('rugLarge', rugLarge, size=(2.85, 0.019, 1.85), key='model.rugLarge', paint=[('rug', 'Rug')]),
     G.Model('rugRunner', rugRunner, size=(0.8, 0.019, 2.8), key='model.rugRunner', paint=[('rug', 'Rug')]),
     G.Model('rugPattern', rugPattern, size=(2.84, 0.024, 1.86), key='model.rugPattern',
             paint=[('rug_ground', 'Rug')])],
    comment='Blender-built decor (tools/art/blender/families/decor.py): wall art and clocks, ornaments, '
            'curtains and rugs. Listed in the base manifest\'s packs. The materials map onto the game\'s '
            'finishes so a model costs one draw call per finish group. The clocks\' hands tell the game\'s '
            'time (`clock`); the hanging plant sways when watered, the fire burns while a resident warms '
            'up by it, and the curtains are drawn at night.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
