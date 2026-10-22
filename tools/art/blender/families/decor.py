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
    bronze=('#A47B45', 'metal'),       # a golden cast bronze, bright on its edges (not the plinth's wood)
    bronze_dark=('#55583E', 'metal'),  # the mane and tail: a dark green patina
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
    log_end=('#B08A62', 'satin'),     # a split log's pale cut end
    brick=('#4A2E26', 'matte'),       # the firebox's sooty brick lining
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


def _shape(F, name, pts, z, material, t=0.003, passes=0):
    """A flat shape of any outline (concave too) on a picture: `pts` (game x, height), its back at game depth `z`,
    `t` thick; `passes` rounds its corners (Chaikin)."""
    pts = G.smooth_outline(pts, passes) if passes else pts
    return F.extrude(name, pts, t, (0, 0, z + t / 2), material)


def _ellipse(cx, cy, rx, ry, a0=0.0, a1=360.0, n=20):
    """Points round an ellipse (degrees from the right, anticlockwise), for `_flat`."""
    return [(cx + rx * math.cos(math.radians(a0 + (a1 - a0) * k / n)), cy + ry * math.sin(math.radians(a0 + (a1 - a0) * k / n)))
            for k in range(n + (0 if a1 - a0 >= 360 else 1))]


def _disc(F, name, x, y, z, r, material, t=0.003, verts=24):
    """A round flat disc facing the front, its back at game depth `z`."""
    return F.cyl(name, r, t, (x, y, z + t / 2), material, rot=(90, 0, 0), verts=verts, smooth=False)


def _leaf_outline(length, width, n=6, curl=0.0):
    """A pointed leaf's outline from its stalk at (0, 0) to its tip at (0, length), `width` across, its tip bent
    `curl` to the side."""
    side = [(width / 2 * math.sin(math.pi * t) ** 0.8 + curl * t * t, length * t) for t in (k / n for k in range(1, n))]
    other = [(-u + 2 * curl * v * v / length ** 2, v) for u, v in reversed(side)]
    return [(0.0, 0.0)] + side + [(curl, length)] + other


def _leaf(F, name, x, y, z, w, h, ang, material, t=0.003):
    """A flat pointed leaf `w` across and `h` long on a picture, its stalk at game (x, y), turned `ang` degrees
    from upright (anticlockwise as seen)."""
    a = math.radians(ang)
    pts = [(x + u * math.cos(a) - v * math.sin(a), y + u * math.sin(a) + v * math.cos(a)) for u, v in _leaf_outline(h, w)]
    return _shape(F, name, pts, z, material, t)


# --- frames -----------------------------------------------------------------------------------------------
def _rect(w, h, yc, x=0.0):
    """A w x h rectangle's corners round (x, yc), game (x, height), anticlockwise as seen from the front."""
    return [(x + w / 2, yc - h / 2), (x + w / 2, yc + h / 2), (x - w / 2, yc + h / 2), (x - w / 2, yc - h / 2)]


def _frame(F, w, h, y0, depth, border, material, profile=None, bevel=0.004):
    """A picture frame hanging from `y0`, its back on the wall: one moulding mitred round its four sides
    (`profile` [(inset, out)], from the wall; default a flat face with a rounded outer edge and a lip at the
    sight edge) and a backing. Returns the opening's middle height, width and height, and the game depth of the
    backing's face."""
    yc = y0 + h / 2
    profile = profile or [(0, 0), (0, depth - bevel), (bevel, depth), (border - 0.008, depth), (border - 0.006, depth - 0.005),
                          (border, depth - 0.007), (border, 0)]
    G.kit.moulding(F, 'frame', _rect(w, h, yc), profile, material, at=WALL)
    F.box('backing', w - border, h - border, 0.006, (0, yc, WALL + 0.003), material)
    return yc, w - 2 * border, h - 2 * border, WALL + 0.006


def _mounted(F, yc, iw, ih, z, mount_w, paper):
    """A white mount filling the opening, its window cut with a bevel, and the picture's paper behind it: the
    picture's size and face."""
    pw, ph = iw - 2 * mount_w, ih - 2 * mount_w
    F.box('paper', pw + 0.01, ph + 0.01, 0.002, (0, yc, z + 0.001), paper)
    G.kit.moulding(F, 'mount', _rect(iw, ih, yc), [(0, 0), (0, 0.004), (mount_w - 0.003, 0.004), (mount_w, 0.0005),
                                                   (mount_w, 0)], M.mount, at=z + 0.002)
    return pw, ph, z + 0.003


# --- D1: wall art -----------------------------------------------------------------------------------------
def printFramed():
    """A botanical print: a white moulded frame and a bevel-cut mount, a curving stem on cream paper with paired
    pointed leaves in two greens, a bud at its tip."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.5, 0.71, 1.15, 0.03, 0.03, M.frame_white)
        pw, ph, z = _mounted(F, yc, iw, ih, z, 0.055, M.paper)
        stem = [(0.012 * math.sin(2.4 * t) - 0.006, yc - 0.23 + 0.46 * t) for t in (k / 8 for k in range(9))]
        F.sweep('stem', [(x, y, z + 0.001) for x, y in stem], 0.0028, M.olive, verts=4, resolution=2, scale=[1.3, 1.0, 0.6])
        for i, (t, s) in enumerate(((0.1, -1), (0.2, 1), (0.32, -1), (0.42, 1), (0.54, -1), (0.64, 1), (0.76, -1), (0.85, 1))):
            k = int(t * 8)
            x, y = stem[k][0] + (stem[k + 1][0] - stem[k][0]) * (t * 8 - k), stem[k][1] + (stem[k + 1][1] - stem[k][1]) * (t * 8 - k)
            size = 1 - t * 0.55
            _leaf(F, f'leaf{i}', x, y, z + 0.0005 * (i % 2), 0.034 * size, 0.1 * size, -s * (58 - 20 * t),
                  [M.sage, M.olive][i % 2])
            _leaf(F, f'vein{i}', x, y, z + 0.0025, 0.004 * size, 0.08 * size, -s * (58 - 20 * t), [M.olive, M.sage][i % 2],
                  t=0.001)
        _leaf(F, 'bud', stem[-1][0], stem[-1][1] - 0.01, z, 0.026, 0.05, 4, M.sage)
        _shape(F, 'signature', _rect(0.05, 0.004, yc - ph / 2 + 0.018, pw / 2 - 0.045), z, M.olive, t=0.001)
    G.join_all('printFramed')
    G.settle(centre=False, floor=False)
    return G.meshes()


def photoPrint():
    """A photo print: a thin black frame and a bevel-cut white mount round a mountain lake: two ranges of peaks
    with snow on their ridges, pines along the shore, their reflection and a ripple on the water, a pale sun."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.75, 0.5, 1.25, 0.033, 0.022, M.frame_black,
                               profile=[(0, 0), (0, 0.03), (0.003, 0.033), (0.017, 0.033), (0.022, 0.028), (0.022, 0)])
        pw, ph, z = _mounted(F, yc, iw, ih, z, 0.05, M.sky)
        x0, y0 = -pw / 2, yc - ph / 2
        _shape(F, 'far', [(x0, y0 + 0.08), (x0, y0 + 0.17), (x0 + 0.08, y0 + 0.2), (x0 + 0.15, y0 + 0.17), (x0 + 0.27, y0 + 0.22),
                          (x0 + 0.42, y0 + 0.18), (x0 + 0.52, y0 + 0.21), (x0 + pw, y0 + 0.16), (x0 + pw, y0 + 0.08)], z, M.haze)
        _shape(F, 'peak0', [(x0 + 0.02, y0 + 0.08), (x0 + 0.13, y0 + 0.2), (x0 + 0.16, y0 + 0.215), (x0 + 0.21, y0 + 0.29),
                            (x0 + 0.26, y0 + 0.22), (x0 + 0.29, y0 + 0.2), (x0 + 0.37, y0 + 0.08)], z + 0.001, M.rock)
        _shape(F, 'peak1', [(x0 + 0.24, y0 + 0.08), (x0 + 0.36, y0 + 0.19), (x0 + 0.4, y0 + 0.2), (x0 + 0.44, y0 + 0.25),
                            (x0 + 0.5, y0 + 0.19), (x0 + 0.6, y0 + 0.08)], z + 0.002, M.rock_dark)
        _shape(F, 'snow0', [(x0 + 0.165, y0 + 0.232), (x0 + 0.19, y0 + 0.24), (x0 + 0.205, y0 + 0.226), (x0 + 0.225, y0 + 0.244),
                            (x0 + 0.253, y0 + 0.235), (x0 + 0.21, y0 + 0.29)], z + 0.003, M.snow)
        _shape(F, 'snow1', [(x0 + 0.41, y0 + 0.218), (x0 + 0.43, y0 + 0.222), (x0 + 0.445, y0 + 0.212), (x0 + 0.475, y0 + 0.22),
                            (x0 + 0.44, y0 + 0.25)], z + 0.004, M.snow)
        F.box('lake', pw, 0.075, 0.002, (0, y0 + 0.0425, z + 0.005), M.lake)
        for i, (x, w) in enumerate(((0.12, 0.08), (0.36, 0.05), (0.52, 0.09))):
            F.box(f'ripple{i}', w, 0.0025, 0.001, (x0 + x, y0 + 0.03 + 0.012 * i, z + 0.0065), M.sky)
        F.box('shore', pw, 0.012, 0.002, (0, y0 + 0.081, z + 0.006), M.shore)
        for i in range(9):   # pines along the far shore, and their reflection in the lake
            x, h = x0 + 0.03 + i * 0.06 + 0.015 * (i % 3), 0.03 + 0.012 * ((i * 5) % 3)
            _flat(F, f'pine{i}', [(x - 0.011, y0 + 0.084), (x + 0.011, y0 + 0.084), (x, y0 + 0.084 + h)], z + 0.007, M.shore)
            _flat(F, f'pine_r{i}', [(x - 0.009, y0 + 0.08), (x, y0 + 0.08 - h * 0.7), (x + 0.009, y0 + 0.08)], z + 0.006,
                  M.rock_dark, t=0.001)
        _disc(F, 'sun', x0 + pw - 0.08, y0 + ph - 0.07, z, 0.026, M.sun)
    G.join_all('photoPrint')
    G.settle(centre=False, floor=False)
    return G.meshes()


GILT = [(0, 0), (0, 0.028), (0.005, 0.034), (0.009, 0.042), (0.014, 0.047), (0.019, 0.042), (0.024, 0.036), (0.034, 0.034),
        (0.044, 0.038), (0.052, 0.046), (0.058, 0.05), (0.062, 0.047), (0.066, 0.036), (0.07, 0.03), (0.07, 0)]


def _fan(F, name, x, y, z, r, turn, material, t=0.011):
    """A carved shell on a frame's face, its hinge at game (x, y), fanning `r` out towards `turn` degrees (game x,
    height: 90 is up): seven ribs, scalloped."""
    pts = [(0.0, 0.0)]
    for i in range(15):
        a = math.radians(turn - 90 + 180 * i / 14)
        rr = r * (1 - 0.16 * (i % 2))
        pts.append((rr * math.cos(a), rr * math.sin(a)))
    F.extrude(name, pts, t, (x, y, z + t / 2), material, bevel=0.002, bevel_segments=1)


def paintingClassic():
    """An oil painting in a deep gilt frame: a moulding with a bead, a cove and a raised sight edge, a carved shell
    at each corner and the top's middle, a dark bead round the canvas; a cottage with a chimney and windows by a
    tree, a path and a fence, hills, a field, clouds."""
    F = G.GameFrame()
    with G.new_parts():
        yc, iw, ih, z = _frame(F, 0.84, 0.65, 1.15, 0.05, 0.07, M.gilt, profile=GILT)
        for sx in (-1, 1):
            for sy in (-1, 1):
                _fan(F, f'corner{sx}{sy}', sx * 0.37, yc + sy * 0.275, WALL + 0.039, 0.045,
                     math.degrees(math.atan2(sy, sx)), M.gilt)
        _fan(F, 'crest', 0, yc + 0.282, WALL + 0.039, 0.05, 90, M.gilt)
        G.kit.moulding(F, 'bead', _rect(iw + 0.016, ih + 0.016, yc), [(0, 0), (0, 0.008), (0.004, 0.012), (0.008, 0.008),
                                                                       (0.008, 0)], M.gilt_dark, at=z)
        z += 0.003
        F.box('haze', iw, ih, 0.003, (0, yc, z + 0.0015), M.haze)
        z += 0.003
        y0 = yc - ih / 2
        _shape(F, 'cloud0', _ellipse(-0.1, yc + 0.13, 0.1, 0.022, n=16) , z, M.cloud)
        _shape(F, 'cloud0b', _ellipse(-0.06, yc + 0.145, 0.05, 0.02, n=12), z + 0.0005, M.cloud)
        _shape(F, 'cloud1', _ellipse(0.13, yc + 0.16, 0.075, 0.018, n=16), z, M.cloud)
        rx = iw / 2 + 0.12
        _shape(F, 'hills_far', [(x, y0 + 0.16 + 0.08 * math.sin(3 * x + 1) * math.sqrt(max(0.0, 1 - (x / rx) ** 2)))
                                for x in (iw / 2 - iw * k / 16 for k in range(17))] + [(-iw / 2, y0 + 0.12), (iw / 2, y0 + 0.12)],
               z + 0.0005, M.olive)
        _flat(F, 'hills', [(x, y0 + 0.12 + 0.15 * math.sqrt(max(0.0, 1 - ((x + 0.06) / rx) ** 2)))
                           for x in (iw / 2 - iw * k / 16 for k in range(17))], z + 0.001, M.hills)
        F.box('field', iw, 0.15, 0.002, (0, y0 + 0.075, z + 0.003), M.field)
        _shape(F, 'path', [(0.11, y0 + 0.14), (0.15, y0 + 0.14), (0.06, y0 + 0.07), (0.02, y0), (-0.06, y0), (0.05, y0 + 0.08)],
               z + 0.004, M.cottage, passes=1)
        for i in range(7):
            F.box(f'fence{i}', 0.006, 0.03, 0.001, (0.2 + i * 0.03, y0 + 0.1, z + 0.005), M.trunk)
        F.box('fence_rail', 0.2, 0.004, 0.001, (0.29, y0 + 0.108, z + 0.006), M.trunk)
        F.box('tree_trunk', 0.018, 0.1, 0.002, (-0.17, y0 + 0.17, z + 0.004), M.trunk)
        _shape(F, 'tree', [(-0.17 + 0.075 * (1 + 0.1 * math.sin(5 * a)) * math.cos(a),
                            y0 + 0.27 + 0.085 * (1 + 0.1 * math.sin(5 * a)) * math.sin(a))
                           for a in (math.tau * k / 24 for k in range(24))], z + 0.005, M.tree)
        _shape(F, 'tree_light', _ellipse(-0.19, y0 + 0.3, 0.035, 0.03, n=12), z + 0.006, M.shore, t=0.001)
        F.box('cottage', 0.08, 0.05, 0.002, (0.13, y0 + 0.165, z + 0.004), M.cottage)
        _flat(F, 'roof', [(0.08, y0 + 0.19), (0.18, y0 + 0.19), (0.13, y0 + 0.235)], z + 0.005, M.roof)
        F.box('chimney', 0.012, 0.03, 0.002, (0.158, y0 + 0.22, z + 0.0045), M.roof)
        for i, x in enumerate((0.105, 0.155)):
            F.box(f'window{i}', 0.014, 0.014, 0.001, (x, y0 + 0.17, z + 0.0055), M.trunk)
        F.box('door', 0.012, 0.024, 0.001, (0.13, y0 + 0.152, z + 0.0055), M.roof)
    G.join_all('paintingClassic')
    G.settle(centre=False, floor=False)
    return G.meshes()


def canvasAbstract():
    """A big abstract canvas in an oak float frame (an L-section standing off the canvas): the canvas wrapped
    round its stretcher with soft edges, a sage block with a torn edge, a mustard circle, a rust band, a brushed
    dark line and a few small marks."""
    F = G.GameFrame()
    w, h, y0 = 1.5, 0.9, 1.05
    yc = y0 + h / 2
    with G.new_parts():
        G.kit.moulding(F, 'frame', _rect(w, h, yc), [(0, 0), (0, 0.048), (0.003, 0.05), (0.011, 0.05), (0.014, 0.047),
                                                     (0.014, 0.008), (0.03, 0.008), (0.03, 0)], M.oak, at=WALL)
        cw, ch = w - 0.06, h - 0.06
        F.soft('canvas', cw, ch, 0.036, (0, yc, WALL + 0.02), M.canvas, round=0.006, round_segments=2, flat=1)
        z = WALL + 0.038
        bx, by, bw, bh = -cw * 0.17, y0 + 0.03 + ch * 0.46, cw * 0.48, ch * 0.72
        torn = [(bx - bw / 2 + 0.006 * math.sin(i * 1.7), by - bh / 2 + bh * i / 10) for i in range(11)]
        _shape(F, 'block', [(bx + bw / 2, by - bh / 2), (bx + bw / 2, by + bh / 2)] + torn[::-1], z, M.sage, t=0.002)
        _disc(F, 'circle', cw * 0.2, y0 + 0.03 + ch * 0.6, z + 0.001, ch * 0.24, M.mustard, verts=40)
        _shape(F, 'band', [(cw * 0.07, y0 + 0.03 + ch * 0.12), (cw * 0.37, y0 + 0.03 + ch * 0.115), (cw * 0.372, y0 + 0.03 + ch * 0.29),
                           (cw * 0.068, y0 + 0.03 + ch * 0.285)], z + 0.003, M.rust, t=0.002)
        line = [(-cw * 0.02 + 0.01 * math.sin(3 * t), y0 + 0.03 + ch * (0.14 + 0.72 * t)) for t in (k / 6 for k in range(7))]
        F.sweep('line', [(x, y, z + 0.005) for x, y in line], None, M.ink, profile=G.rounded_rect(0.004, 0.018, 0.0015, corner=1),
                up=(0, 1, 0), resolution=2, scale=[0.7, 1.0, 1.1, 1.0, 0.9, 1.0, 0.6])
        for i, (x, y, r) in enumerate(((0.4, 0.86, 0.022), (0.46, 0.8, 0.014), (-0.4, 0.1, 0.018))):
            _disc(F, f'mark{i}', cw * x, y0 + 0.03 + ch * y, z + 0.001, r, [M.ink, M.rust, M.mustard][i], t=0.002, verts=16)
        F.sweep('arc', [(cw * (0.3 + 0.12 * math.cos(a)), y0 + 0.03 + ch * (0.6 + 0.2 * math.sin(a)), z + 0.004)
                        for a in (math.radians(200 + 18 * k) for k in range(8))], None, M.rust,
                profile=G.rounded_rect(0.003, 0.012, 0.001, corner=1), up=(0, 1, 0), resolution=2)
    G.join_all('canvasAbstract')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- D1: the wall clock -----------------------------------------------------------------------------------
CLOCK_Y = 1.9


def wallClock():
    """A round wall clock: a steel case with a rolled bezel, a white face with a minute track (sixty ticks, the
    hours bolder), and shaped hands that tell the game's time (ten past ten at rest, the catalogue's picture)."""
    F = G.GameFrame()
    r = 0.18
    with G.new_parts() as still:
        # (its well's floor behind the face: at 0.056 the case's front hid the face, so "Face" coloured nothing)
        G.revolve('case', [(0, 0), (r - 0.006, 0), (r - 0.001, 0.004), (r, 0.012), (r, 0.052), (r - 0.002, 0.06),
                           (r - 0.007, 0.0655), (r - 0.013, 0.066), (r - 0.018, 0.0625), (r - 0.02, 0.055), (r - 0.021, 0.046),
                           (0, 0.046)],
                  at=F.at(0, CLOCK_Y, WALL), material=M.clock_case, rot=(90, 0, 0), segments=48)
        _disc(F, 'face', 0, CLOCK_Y, WALL + 0.05, r - 0.022, M.face, t=0.004, verts=48)
        z = WALL + 0.054
        for k in range(60):
            a = math.radians(k * 6)
            big = k % 5 == 0
            d = r - 0.045 if big else r - 0.034
            F.box(f'tick{k}', (0.009 if k % 15 == 0 else 0.006) if big else 0.0022, 0.03 if big else 0.008, 0.0015,
                  (math.sin(a) * d, CLOCK_Y + math.cos(a) * d, z + 0.00075), M.ink, rot=(0, -k * 6, 0))
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


def _hand_outline(length, width, tail, spade=False):
    """A clock hand from its tail (below the arbor) to its tip (`length` up), `width` at its widest: a slim
    taper with a round boss on the arbor, or (`spade`) a leaf swelling near the tip."""
    w = width / 2
    boss = [(w * 1.3 * math.cos(a), w * 1.3 * math.sin(a)) for a in (math.radians(200 + 20 * k) for k in range(8))]
    if spade:
        right = [(w * 0.7, 0.0), (w * 0.55, length * 0.55), (w * 1.6, length * 0.7), (w * 1.1, length * 0.86), (0.0, length)]
    else:
        right = [(w * 0.8, 0.0), (w * 0.55, length * 0.75), (w * 0.3, length * 0.95), (0.0, length)]
    tail_pts = [(-w * 0.6, -tail), (w * 0.6, -tail)] if tail else []
    return tail_pts + right + [(-u, v) for u, v in reversed(right[:-1])] + boss


def _clock_hands(F, y, z, hour, minute, x=0.0, material=None):
    """The hour and minute hands, each (name, length, width, lift above `z`, tail), on a dial centred at
    game (`x`, `y`, `z`), pointing at CLOCK_AT; returned hinged on the dial's middle. (Seen from the front, the
    game's +x is on the left, so angles clockwise from twelve go towards -x.)"""
    out = []
    for (name, length, width, lift, tail), angle in ((hour, -(CLOCK_AT / 2) % 360), (minute, -(CLOCK_AT % 60 * 6))):
        with G.new_parts() as parts:
            F.extrude(name, _hand_outline(length, width, tail, spade=name == 'hour'), 0.002, (x, y, z + lift),
                      material or M.ink, rot=(0, -angle, 0))
        out.append(G.hinged(G.join(parts, f'{name}_hand'), F.at(x, y, z)))
    return out


def _clock_clips(hour, minute):
    """Each hand once round clockwise (seen from the front); the game holds them at the time."""
    for hand in (hour, minute):
        G.spin(hand, f'{hand.name}', turns=1, seconds=12, axis='Y')


def _book(F, name, x, y, z, t, h, depth, cover, lean=0.0):
    """A book standing on a shelf top at height `y`, its spine to the front at game x `x`, `t` thick: its boards
    and spine in `cover`, a band at the spine's head and foot, the pages' cream edge showing at the top."""
    with G.new_parts() as parts:
        F.box(f'{name}_pages', t - 0.003, h - 0.006, depth - 0.006, (x, y + h / 2, z - 0.002), M.book_cream)
        for s in (-1, 1):
            F.box(f'{name}_board{s}', 0.0025, h, depth, (x + s * (t / 2 - 0.00125), y + h / 2, z), cover)
        F.box(f'{name}_spine', t, h, 0.004, (x, y + h / 2, z + depth / 2 - 0.002), cover, bevel=0.0012, bevel_segments=1)
        for s in (-1, 1):
            F.box(f'{name}_band{s}', t + 0.0004, 0.004, 0.001, (x, y + h / 2 + s * (h / 2 - 0.018), z + depth / 2 + 0.0002), M.gilt)
    book = G.join(parts, name)
    if lean:
        pivot = F.at(x + t / 2, y, z)
        G.set_origin(book, pivot)
        book.rotation_euler[1] = math.radians(lean)
    return book


def _books(F, x0, x1, y, z, depth, seed):
    """A row of books standing on a shelf top at `y`, from `x0` to `x1`; the last one leaning on them."""
    colours = (M.book_teal, M.book_rust, M.book_mustard, M.book_navy, M.book_cream)
    x, k = x0, 0
    while x < x1 - 0.06:
        t = 0.025 + 0.012 * ((k * 7 + seed) % 3)
        h = 0.17 + 0.025 * ((k * 5 + seed) % 4)
        _book(F, f'book{seed}_{k}', x + t / 2, y, z, t, h, depth - 0.01 * (k % 2), colours[(k + seed) % 5])
        x, k = x + t + 0.002, k + 1
    _book(F, f'book{seed}_lean', x + 0.085, y, z, 0.03, 0.19, depth, colours[(k + seed) % 5], lean=22)


def _heart(F, name, at, size, turn, tilt, material):
    """A heart-shaped leaf `size` long, its stalk at game point `at`, turned `turn` degrees about the up axis and
    tipped `tilt` degrees down from upright (a trailing plant's leaf)."""
    pts = [(0.0, 0.0)] + [(size * 0.5 * math.sin(a) * (1 - 0.3 * math.cos(a)), size * (0.45 - 0.45 * math.cos(a)) + size * 0.1 * math.sin(a) ** 2)
                          for a in (math.pi * k / 8 for k in range(1, 8))] + [(0.0, size)]
    pts = pts + [(-u, v) for u, v in reversed(pts[1:-1])]
    pts = [(u, v - size * 0.08 * math.sin(math.pi * v / size)) for u, v in pts]
    return G.extrude(name, pts, 0.003, at=F.at(*at), material=material, plane='XZ', rot=(tilt, 0, turn))


def _vine(F, name, pts, material, leaves, seed=0):
    """A trailing vine through game points, a heart leaf at every few of its points."""
    F.sweep(name, pts, 0.0028, material, verts=4, resolution=2)
    for i, p in enumerate(pts[1:], 1):
        if i % leaves == 0 or i == len(pts) - 1:
            _heart(F, f'{name}_leaf{i}', p, 0.034 - 0.004 * (i % 3), 40 * ((i * 7 + seed) % 9) - 160, 75 + 5 * (i % 3),
                   M.leaf if (i + seed) % 2 else M.leaf_light)


def wallShelf():
    """Two floating oak shelves with rounded front edges: a row of bound books (boards, pages, gilt bands), a
    trailing pothos in a lipped pot spilling over the edge, a small framed picture leaning on the wall, a bud vase
    and a clay ball."""
    F = G.GameFrame()
    zc = WALL + 0.12
    with G.new_parts():
        board = G.smooth_outline([(-0.12, -0.02), (0.11, -0.02), (0.12, -0.012), (0.12, 0.012), (0.11, 0.02), (-0.12, 0.02)],
                                 passes=1)
        for y in (1.4, 1.78):
            F.extrude(f'shelf{y}', board, 0.9, (0, y, zc), M.oak, plane='side')
        top0, top1 = 1.42, 1.80
        _books(F, -0.42, -0.06, top0, zc - 0.01, 0.17, 1)
        G.revolve('plant_pot', [(0, 0), (0.042, 0), (0.05, 0.004), (0.056, 0.085), (0.062, 0.088), (0.063, 0.11), (0.056, 0.11),
                                (0.052, 0.09), (0.0, 0.09)], at=F.at(0.25, top0, zc), material=M.pot, segments=24)
        G.revolve('soil', [(0, 0), (0.052, 0), (0, 0.004)], at=F.at(0.25, top0 + 0.088, zc), material=M.clay, segments=12)
        for k, (dx, dy, dz, r) in enumerate(((0, 0.14, 0, 0.06), (-0.04, 0.125, 0.02, 0.045), (0.045, 0.13, -0.015, 0.05))):
            G.jitter(G.sphere(f'plant{k}', r, at=F.at(0.25 + dx, top0 + dy, zc + dz), material=M.leaf if k else M.leaf_light,
                              scale=(1, 0.9, 0.8), segments=10, rings=6), 0.012, size=0.03, seed=k * 1.7)
        # the pothos's vines, out of the pot and down over the shelf's front edge
        _vine(F, 'vine0', [(0.21, top0 + 0.1, zc + 0.03), (0.17, top0 + 0.03, zc + 0.07), (0.13, top0 + 0.008, zc + 0.085),
                           (0.09, top0 + 0.006, zc + 0.09), (0.07, top0 + 0.006, zc + 0.08)], M.leaf, 1, 0)
        _vine(F, 'vine1', [(0.29, top0 + 0.09, zc + 0.03), (0.31, top0 + 0.03, zc + 0.06), (0.33, top0 + 0.006, zc + 0.08),
                           (0.36, top0 + 0.005, zc + 0.085), (0.39, top0 + 0.005, zc + 0.07)], M.leaf_light, 1, 3)
        _vine(F, 'vine2', [(0.3, top0 + 0.11, zc - 0.02), (0.34, top0 + 0.06, zc - 0.04), (0.37, top0 + 0.02, zc - 0.05),
                           (0.39, top0 + 0.006, zc - 0.03)], M.leaf, 1, 5)
        # a small picture leaning on the wall
        with G.new_parts() as pic:
            G.kit.moulding(F, 'picture', _rect(0.18, 0.24, top1 + 0.12, -0.25),
                           [(0, 0), (0, 0.014), (0.004, 0.016), (0.02, 0.016), (0.02, 0)], M.frame_white, at=WALL + 0.036)
            F.box('picture_art', 0.15, 0.21, 0.004, (-0.25, top1 + 0.12, WALL + 0.04), M.clay)
            _shape(F, 'picture_sun', _ellipse(-0.25, top1 + 0.15, 0.03, 0.03, n=12), WALL + 0.042, M.mustard, t=0.001)
        art = G.join(pic, 'picture')
        G.set_origin(art, F.at(-0.25, top1, WALL + 0.044))
        art.rotation_euler[0] = math.radians(-8)
        bx, by, _ = F.at(0.05, 0, zc)
        G.loft('vase', [G.ring(r, r, top1 + y, 18, at=(bx, by)) for y, r in ((0, 0.036), (0.004, 0.04), (0.05, 0.05), (0.1, 0.04),
                                                                            (0.13, 0.022), (0.15, 0.02), (0.158, 0.026),
                                                                            (0.16, 0.024))], material=M.book_cream)
        G.sphere('ball', 0.055, at=F.at(0.3, top1 + 0.055, zc), material=M.clay, segments=16, rings=10)
    G.join_all('wallShelf')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- D2: the grandfather clock ----------------------------------------------------------------------------
GF_Z = WALL + 0.2         # the case's middle, game depth (0.4 deep, back on the wall)
TRUNK_FRONT = WALL + 0.35  # the trunk's front face
DIAL_Y = 1.77


def _box_outline(w, d, z=0.0, x=0.0):
    """A w x d rectangle's corners round game (x, depth z), for a level moulding (`plane='top'`)."""
    return [(x - w / 2, z - d / 2), (x + w / 2, z - d / 2), (x + w / 2, z + d / 2), (x - w / 2, z + d / 2)]


ROMAN = {1: 'I', 2: 'II', 3: 'III', 4: 'IIII', 5: 'V', 6: 'VI', 7: 'VII', 8: 'VIII', 9: 'IX', 10: 'X', 11: 'XI', 12: 'XII'}


def _numerals(F, y, z, r, h, material):
    """Roman hours round a dial centred at game (0, y), their feet towards the middle, `r` out and `h` tall,
    drawn in strokes: I a bar, V and X two bars crossed or splayed."""
    stroke, gap = h * 0.13, h * 0.3
    for hour, text in ROMAN.items():
        a = hour * 30
        widths = [gap * (1.6 if c != 'I' else 1.0) for c in text]
        u = -sum(widths) / 2
        for j, c in enumerate(text):
            mid = u + widths[j] / 2
            u += widths[j]
            bars = [(0, 0)] if c == 'I' else ([(-14, -0.22), (14, 0.22)] if c == 'V' else [(-22, 0), (22, 0)])
            for b, (tilt, off) in enumerate(bars):
                # the stroke's middle on the ring, `mid` along it, turned to the hour and tilted in its letter
                # (seen from the front the game's +x is on the left: three o'clock is at -x)
                ca, sa = math.cos(math.radians(a)), math.sin(math.radians(a))
                x0, y0 = -sa * r - ca * (mid + off * gap), y + ca * r - sa * (mid + off * gap)
                F.box(f'numeral{hour}_{j}_{b}', stroke, h, 0.001, (x0, y0, z), material, rot=(0, a + tilt, 0))


def grandfatherClock():
    """A walnut longcase clock: a moulded plinth and a base with a fielded panel, a waist moulding, a trunk with
    quarter columns and a door whose arched window shows the pendulum, a neck moulding, a hood with turned brass
    columns, a cornice and an arched pediment under three urn finials; a brass dial plate with corner spandrels,
    a cream chapter ring with Roman hours and a minute track, winding holes and shaped hands."""
    F = G.GameFrame()
    with G.new_parts() as still:
        G.kit.moulding(F, 'plinth', _box_outline(0.6, 0.4, GF_Z), [(0, 0), (0, 0.03), (0.006, 0.036), (0.012, 0.046),
                                                                   (0.018, 0.054), (0.022, 0.06), (0.05, 0.06), (0.05, 0)],
                       M.walnut_dark, at=0.0, plane='top')
        F.box('base', 0.56, 0.38, 0.38, (0, 0.25, GF_Z - 0.01), M.walnut, bevel=0.01)
        F.extrude('base_panel', G.rounded_rect(0.36, 0.22, 0.03, corner=3), 0.014, (0, 0.25, GF_Z + 0.185), M.walnut_dark,
                  bevel=0.006, bevel_segments=2)
        G.kit.moulding(F, 'base_frame', _rect(0.4, 0.26, 0.25), [(0, 0), (0, 0.006), (0.006, 0.01), (0.014, 0.006), (0.016, 0)],
                       M.walnut, at=GF_Z + 0.18)
        G.kit.moulding(F, 'waist_moulding', _box_outline(0.58, 0.4, GF_Z), [(0.07, 0), (0, 0), (0, 0.01), (0.006, 0.016),
                                                                            (0.012, 0.028), (0.03, 0.04), (0.07, 0.04)],
                       M.walnut_dark, at=0.44, plane='top')
        F.box('trunk', 0.46, 1.06, 0.3, (0, 1.0, TRUNK_FRONT - 0.15), M.walnut, bevel=0.008)
        F.box('trunk_hollow', 0.32, 0.8, 0.004, (0, 1.0, TRUNK_FRONT + 0.002), M.hollow)
        for s in (-1, 1):   # quarter columns at the trunk's front corners, on brass bases and capitals
            F.cyl(f'quarter{s}', 0.016, 0.96, (s * 0.222, 1.0, TRUNK_FRONT - 0.006), M.walnut_dark, verts=12)
            for y, k in ((0.52, 1), (1.48, -1)):
                _turned_g(F, f'quarter_cap{s}{y}', [(0, 0), (0.022, 0), (0.022, 0.008), (0.017, 0.016), (0, 0.016)],
                          (s * 0.222, y, TRUNK_FRONT - 0.006), M.brass, 12, flip=k < 0)
        G.kit.moulding(F, 'neck_moulding', _box_outline(0.58, 0.4, GF_Z), [(0.07, 0), (0.03, 0), (0.012, 0.012),
                                                                           (0.006, 0.024), (0, 0.03), (0, 0.04), (0.07, 0.04)],
                       M.walnut_dark, at=1.53, plane='top')
        F.box('hood', 0.54, 0.44, 0.38, (0, 1.79, GF_Z - 0.01), M.walnut, bevel=0.01)
        for s in (-1, 1):   # turned brass columns either side of the hood door
            _turned_g(F, f'column{s}', [(0, 0), (0.02, 0), (0.02, 0.012), (0.014, 0.022), (0.016, 0.03), (0.0125, 0.06),
                                        (0.0115, 0.33), (0.0145, 0.37), (0.016, 0.39), (0.02, 0.4), (0.02, 0.412),
                                        (0, 0.412)], (s * 0.24, 1.584, GF_Z + 0.18), M.brass, 12)
        G.kit.moulding(F, 'cornice', _box_outline(0.6, 0.4, GF_Z), [(0.07, 0), (0.03, 0), (0.02, 0.008), (0.012, 0.02),
                                                                    (0.006, 0.034), (0, 0.042), (0, 0.05), (0.07, 0.05)],
                       M.walnut_dark, at=2.01, plane='top')
        F.box('cornice_top', 0.48, 0.006, 0.28, (0, 2.057, GF_Z), M.walnut_dark)
        # the arched pediment over the cornice, its middle finial on the arch's crown
        arch = [(0.29, 2.06), (0.29, 2.068)] + [(0.29 * math.cos(math.radians(a)) * 1.0,
                                                 2.06 + 0.032 * math.sin(math.radians(a)) ** 0.7 + 0.008)
                                                for a in range(10, 171, 20)] + [(-0.29, 2.068), (-0.29, 2.06)]
        F.extrude('pediment', arch, 0.03, (0, 0, GF_Z + 0.155), M.walnut_dark, bevel=0.004, bevel_segments=1)
        for x, y, h in ((-0.25, 2.06, 0.056), (0.25, 2.06, 0.056), (0.0, 2.098, 0.018)):
            _turned_g(F, f'finial{x}', [(0, 0), (0.012, 0), (0.016, 0.008 * h / 0.056), (0.01, 0.016 * h / 0.056),
                                        (0.022 * min(1, h / 0.03), 0.03 * h / 0.056), (0.016, 0.042 * h / 0.056),
                                        (0.004, 0.05 * h / 0.056), (0, h)], (x, y, GF_Z), M.brass, 12)
        # the hood door's frame round the square dial plate
        F.extrude('hood_door', G.rounded_rect(0.44, 0.42, 0.01, corner=1), 0.014, (0, DIAL_Y, GF_Z + 0.178), M.walnut,
                  holes=[G.rounded_rect(0.37, 0.37, 0.006, corner=1)], bevel=0.003, bevel_segments=1)
        dz = GF_Z + 0.18
        F.box('dial_plate', 0.37, 0.37, 0.004, (0, DIAL_Y, dz + 0.002), M.brass)
        for sx in (-1, 1):
            for sy in (-1, 1):
                _fan(F, f'spandrel{sx}{sy}', sx * 0.125, DIAL_Y + sy * 0.125, dz + 0.004, 0.05, math.degrees(math.atan2(sy, sx)),
                     M.brass, t=0.004)
        F.extrude('dial', [(0.16 * math.cos(a), DIAL_Y + 0.16 * math.sin(a)) for a in (math.tau * k / 40 for k in range(40))],
                  0.004, (0, 0, dz + 0.006), M.dial, holes=[[(0.112 * math.cos(a), DIAL_Y + 0.112 * math.sin(a))
                                                             for a in (math.tau * k / 40 for k in range(40))]])
        _disc(F, 'dial_centre', 0, DIAL_Y, dz + 0.004, 0.1, M.dial, t=0.003, verts=32)
        G.torus('dial_ring', 0.165, 0.006, at=F.at(0, DIAL_Y, dz + 0.006), material=M.brass, rot=(90, 0, 0),
                major_segments=40, minor_segments=4)
        _numerals(F, DIAL_Y, dz + 0.0085, 0.134, 0.03, M.ink)
        for k in range(60):
            a = math.radians(k * 6)
            F.box(f'minute{k}', 0.0016, 0.007 if k % 5 else 0.01, 0.001, (math.sin(a) * 0.153, DIAL_Y + math.cos(a) * 0.153,
                                                                          dz + 0.0085), M.ink, rot=(0, -k * 6, 0))
        for s in (-1, 1):
            _disc(F, f'winding{s}', s * 0.05, DIAL_Y - 0.045, dz + 0.007, 0.008, M.hollow, t=0.001, verts=10)
        _disc(F, 'arbor', 0, DIAL_Y, dz + 0.014, 0.01, M.brass, t=0.006, verts=12)
        # The trunk door (a walnut frame round an arched window) is still: the pendulum and the hands take the draws.
        door_z = TRUNK_FRONT + 0.026
        window = [(-0.13, 0.65), (0.13, 0.65)] + [(0.13 * math.cos(math.radians(a)), 1.22 + 0.13 * math.sin(math.radians(a)))
                                                   for a in range(0, 181, 15)] + [(-0.13, 0.65)]
        F.extrude('door', G.rounded_rect(0.36, 0.82, 0.01, corner=1), 0.02, (0, 1.0, door_z), M.walnut,
                  holes=[[(u, v - 1.0) for u, v in _dedupe(window)]], bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'door_bead', [(u, v) for u, v in _dedupe(window)][::-1], [(-0.008, 0), (-0.008, 0.006),
                                                                                    (-0.004, 0.009), (0, 0.006), (0, 0)],
                       M.walnut_dark, at=door_z + 0.01)
        _disc(F, 'escutcheon', 0.15, 1.0, door_z + 0.01, 0.012, M.brass, t=0.003, verts=12)
        F.box('keyhole', 0.004, 0.01, 0.001, (0.15, 1.0, door_z + 0.0135), M.hollow)
    G.join(still, 'body', origin=(0, 0, 0))
    hands = _clock_hands(F, DIAL_Y, dz, ('hour', 0.075, 0.014, 0.009, 0.0), ('minute', 0.115, 0.009, 0.011, 0.0))
    # The pendulum: a brass rod and lens-shaped bob hung from just under the hood, in front of the dark hollow, seen
    # through the door's window; it swings ±4° about its top (`pendulum`, ambient), a beat each 0.8 s, as a pendulum
    # this long does.
    pz = TRUNK_FRONT + 0.008
    with G.new_parts() as pendulum:
        F.box('pendulum_rod', 0.01, 0.58, 0.004, (0, 1.12, pz), M.brass)
        F.box('pendulum_block', 0.03, 0.03, 0.008, (0, 1.39, pz), M.brass, bevel=0.003, bevel_segments=1)
        G.revolve('pendulum_bob', [(0, -0.008), (0.035, -0.006), (0.058, -0.0025), (0.065, 0), (0.058, 0.0025), (0.035, 0.006),
                                   (0, 0.008)], at=F.at(0, 0.8, pz + 0.002), material=M.brass, segments=24, rot=(90, 0, 0))
        F.cyl('rating_nut', 0.008, 0.012, (0, 0.72, pz), M.brass, verts=8)
    pendulum = G.hinged(G.join(pendulum, 'pendulum'), F.at(0, 1.41, pz))
    G.settle(centre=False)
    _clock_clips(*hands)
    G.rock(pendulum, 'pendulum', 4, seconds=1.6, axis='Y')
    return G.meshes()


def _dedupe(pts, eps=1e-6):
    out = []
    for p in pts:
        if not out or abs(p[0] - out[-1][0]) + abs(p[1] - out[-1][1]) > eps:
            out.append(p)
    if len(out) > 2 and abs(out[0][0] - out[-1][0]) + abs(out[0][1] - out[-1][1]) <= eps:
        out.pop()
    return out


def _turned_g(F, name, profile, at, material, segments=16, flip=False):
    """A turned part, `profile` [(radius, up)] from its foot at game point `at` (`flip`: hanging down from it)."""
    return G.revolve(name, [(r, -z if flip else z) for r, z in profile], at=F.at(*at), material=material,
                     segments=segments)


# --- D2: vases and statues --------------------------------------------------------------------------------
def vaseTall():
    """A tall floor vase of pale ceramic, thrown with a foot, two grooves and a rolled lip, holding three dried
    twigs curving out, each with a side shoot, under their pampas plumes."""
    with G.new_parts():
        G.revolve('vase', [(0, 0), (0.06, 0), (0.068, 0.004), (0.07, 0.014), (0.064, 0.022), (0.072, 0.05), (0.09, 0.15),
                           (0.098, 0.2), (0.096, 0.205), (0.1, 0.212), (0.1, 0.27), (0.096, 0.33), (0.093, 0.335),
                           (0.092, 0.342), (0.085, 0.42), (0.07, 0.49), (0.052, 0.55), (0.046, 0.59), (0.05, 0.606),
                           (0.058, 0.614), (0.059, 0.622), (0.052, 0.627), (0.044, 0.616), (0.04, 0.59), (0, 0.59)],
                  material=M.ceramic, segments=32)
        for k, (x, y, top) in enumerate(((-0.11, 0.04, 1.0), (0.1, -0.03, 1.06), (0.02, 0.07, 0.94))):
            tip = Vector((-x, y, top + 0.035))   # Blender: the game's x mirrored
            foot = Vector((0, 0, 0.5))
            bend = Vector((-x * 0.35, y * 0.2, 0))
            pts = [foot + (tip - foot) * t + bend * math.sin(math.pi * t) for t in (0, 0.33, 0.66, 1.0)]
            G.sweep(f'twig{k}', [tuple(p) for p in pts], 0.0055, material=M.twig, verts=6, resolution=3,
                    scale=[1.0, 0.85, 0.7, 0.55])
            mid = pts[2]
            shoot = mid + Vector((-x * 0.15, y * 0.3, 0.12))
            G.sweep(f'shoot{k}', [tuple(mid), tuple(mid + (shoot - mid) * 0.5 + Vector((0, 0, 0.02))), tuple(shoot)], 0.0035,
                    material=M.twig, verts=5, resolution=2, scale=[1.0, 0.7])
            for name, base, along, r, n in ((f'plume{k}', tip, tip - foot, 0.045, 3.2), (f'tuft{k}', shoot, shoot - mid, 0.026, 2.4)):
                plume = G.sphere(name, r, at=tuple(base + along.normalized() * r * n * 0.25), material=M.plume,
                                 scale=(1, 1, n), segments=12, rings=10)
                plume.rotation_mode = 'QUATERNION'
                plume.rotation_quaternion = along.to_track_quat('Z', 'Y')   # along its twig
                bpy.context.view_layer.update()
                G.jitter(plume, 0.004, size=0.02, seed=k * 2.3 + r)
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
    G.sweep(f'stem{k}', [p for p, _ in pts], 0.0032, material=M.blue, verts=4, resolution=2, scale=[1.1, 1.0, 0.9, 0.8, 0.7])

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
    """A porcelain ginger jar on a foot ring, white with blue bands and sprigs; its lid on a flange, with a
    turned knob."""
    with G.new_parts():
        G.revolve('jar', [(0, 0.004), (0.098, 0.004), (0.1, 0), (0.108, 0), (0.11, 0.008), (0.112, 0.014), (0.122, 0.024),
                          (0.15, 0.08), (0.17, 0.15), (0.18, 0.27), (0.16, 0.39), (0.13, 0.44), (0.11, 0.46), (0.1, 0.475),
                          (0.1, 0.5), (0, 0.5)], material=M.porcelain, segments=30)
        G.revolve('lid', [(0, 0), (0.104, 0), (0.11, 0.004), (0.112, 0.014), (0.108, 0.022), (0.096, 0.036), (0.08, 0.058),
                          (0.055, 0.075), (0.03, 0.085), (0, 0.087)], at=(0, 0, 0.495), material=M.porcelain, segments=24)
        G.revolve('knob', [(0, 0), (0.016, 0), (0.012, 0.01), (0.022, 0.022), (0.024, 0.032), (0.016, 0.044), (0, 0.048)],
                  at=(0, 0, 0.578), material=M.blue, segments=14)
        for k, (y, r) in enumerate(((0.012, 0.11), (0.39, 0.16), (0.5, 0.106), (0.545, 0.089))):
            G.torus(f'band{k}', r, 0.006, at=(0, 0, y), material=M.blue, major_segments=30, minor_segments=4)
        for k in range(5):
            _sprig(k, math.radians(k * 72 + 20))
    G.join_all('urnPorcelain')
    G.settle()
    return G.meshes()


def _pedestal(h, w, material, top=None, F=None):
    """A square pedestal `h` tall and `w` wide on the footprint's middle: a moulded foot and cap a little wider, the
    shaft between them with a raised fillet framing a sunk panel on each face."""
    F = F or G.GameFrame()
    m = w / 2 + 0.02
    G.kit.moulding(F, 'plinth_foot', _box_outline(2 * m, 2 * m), [(0, 0), (0, 0.022), (0.004, 0.028), (0.01, 0.034),
                                                                  (0.02, 0.04), (0.03, 0.045), (0.04, 0.045), (0.04, 0)],
                   material, at=0.0, plane='top')
    F.box('plinth', w, h - 0.07, w, (0, 0.04 + (h - 0.07) / 2, 0), material, bevel=0.004, bevel_segments=1)
    G.kit.moulding(F, 'plinth_cap', _box_outline(2 * m, 2 * m), [(0.04, 0), (0.02, 0), (0.012, 0.005), (0.006, 0.013),
                                                                 (0, 0.018), (0, 0.03), (0.04, 0.03)],
                   top or material, at=h - 0.03, plane='top')
    F.box('plinth_lid', 2 * m - 0.07, 0.004, 2 * m - 0.07, (0, h - 0.002, 0), top or material)
    fh, fw = h - 0.2, w - 0.08
    frame = G.rounded_rect(fw, fh, 0.004, corner=1)
    hole = G.rounded_rect(fw - 0.024, fh - 0.024, 0.003, corner=1)
    y = 0.04 + (h - 0.07) / 2
    for s in (-1, 1):
        F.extrude(f'fillet_z{s}', frame, 0.008, (0, y, s * (w / 2 + 0.002)), material, holes=[hole])
        F.extrude(f'fillet_x{s}', frame, 0.008, (s * (w / 2 + 0.002), y, 0), material, holes=[hole], plane='side')


def vasePedestal():
    """A blue glazed vase with a foot and a rolled lip, of five white hydrangea heads on curving stems with
    pointed leaves, on a stone pedestal with moulded foot and cap and sunk panels."""
    y = 0.86
    with G.new_parts():
        _pedestal(y, 0.36, M.stone)
        G.revolve('vase', [(0, 0), (0.07, 0), (0.074, 0.006), (0.07, 0.014), (0.1, 0.07), (0.118, 0.13), (0.12, 0.17),
                           (0.108, 0.23), (0.085, 0.28), (0.06, 0.315), (0.06, 0.325), (0.07, 0.334), (0.07, 0.342),
                           (0.062, 0.344), (0.054, 0.335), (0, 0.3)], at=(0, 0, y), material=M.vase_blue, segments=32)
        for k, (x, d, top) in enumerate(((-0.08, 0.02, 1.48), (0.06, -0.04, 1.54), (0.02, 0.07, 1.42), (-0.03, -0.06, 1.36),
                                         (0.1, 0.05, 1.32))):
            tip = Vector((-x, -d, top))
            foot = Vector((0, 0, y + 0.28))
            pts = [foot + (tip - foot) * t + Vector((-x * 0.25, -d * 0.25, 0)) * math.sin(math.pi * t) for t in (0, 0.4, 0.75, 1)]
            G.sweep(f'stem{k}', [tuple(p) for p in pts], 0.0055, material=M.stem, verts=5, resolution=3, scale=[1, 0.9, 0.8, 0.7])
            G.jitter(G.sphere(f'bloom{k}', 0.05, at=(-x, -d, top + 0.02), material=M.bloom, scale=(1, 1, 0.8), segments=14,
                              rings=8), 0.009, size=0.018, seed=k * 1.3)
            # two pointed leaves off the stem, held out and turned along it
            for j, t in enumerate((0.35, 0.6)):
                p = pts[1] if j == 0 else pts[2]
                turn = math.degrees(math.atan2(-d, -x)) + (100 if (j + k) % 2 else -100)
                G.extrude(f'leaf{k}_{j}', _leaf_outline(0.11 - 0.02 * j, 0.045), 0.003, at=tuple(p), material=M.stem,
                          plane='XZ', rot=(-60, 0, turn))
    G.join_all('vasePedestal')
    G.settle()
    return G.meshes()


def bustMarble():
    """A marble bust on a stone pedestal: a turned socle, a chest cut off at the shoulders, a neck, a head lofted
    from chin to crown with a nose, eyelids, lips, a chin and ears, under a cap of curly hair."""
    y = 1.0
    with G.new_parts():
        _pedestal(y, 0.4, M.stone)
        G.revolve('socle', [(0, 0), (0.09, 0), (0.09, 0.012), (0.082, 0.02), (0.06, 0.03), (0.05, 0.05), (0.056, 0.07),
                            (0.056, 0.08), (0, 0.08)], at=(0, 0, y), material=M.marble, segments=28)
        # the chest, cut off flat at the arms, its shoulders sloping to the neck
        G.loft('shoulders', [G.ring(rx, ry, y + z, 24, square=sq, at=(0, cy)) for z, rx, ry, cy, sq in (
            (0.075, 0.055, 0.042, 0.0, 2), (0.088, 0.1, 0.058, 0.0, 2.6), (0.11, 0.14, 0.074, -0.004, 3.4),
            (0.18, 0.152, 0.082, -0.006, 3.6), (0.222, 0.15, 0.08, -0.002, 3.4), (0.24, 0.14, 0.074, 0.002, 3),
            (0.255, 0.115, 0.064, 0.004, 2.6), (0.267, 0.08, 0.052, 0.004, 2.2), (0.276, 0.045, 0.04, 0.004, 2))],
            material=M.marble)
        for s in (-1, 1):   # the collarbones and the chest's divide
            G.sweep(f'collarbone{s}', [(s * 0.012, -0.05, y + 0.255), (s * 0.06, -0.06, y + 0.25), (s * 0.11, -0.05, y + 0.25)],
                    0.008, material=M.marble, verts=6, resolution=2, scale=[0.6, 1.0, 0.5])
            G.sphere(f'chest{s}', 0.5, at=(s * 0.055, -0.07, y + 0.19), material=M.marble, scale=(0.1, 0.03, 0.07),
                     segments=12, rings=6)
        G.loft('neck', [G.ring(r, r * 1.05, y + z, 16, at=(0, cy)) for z, r, cy in ((0.25, 0.046, 0.0), (0.3, 0.04, -0.004),
                                                                                    (0.35, 0.039, -0.008))], material=M.marble)
        G.loft('head', [G.ring(rx, ry, y + z, 20, square=2.2, at=(0, cy)) for z, rx, ry, cy in (
            (0.335, 0.018, 0.016, -0.054), (0.345, 0.036, 0.034, -0.042), (0.36, 0.051, 0.056, -0.024), (0.385, 0.061, 0.073, -0.009),
            (0.42, 0.067, 0.083, 0.0), (0.45, 0.07, 0.086, 0.004), (0.475, 0.067, 0.083, 0.008), (0.495, 0.054, 0.068, 0.01),
            (0.505, 0.03, 0.04, 0.012), (0.508, 0.005, 0.005, 0.012))], material=M.marble)
        G.extrude('nose', [(0.0, 0.0), (0.02, -0.004), (0.026, -0.04), (0.012, -0.048), (-0.004, -0.044)], 0.02,
                  at=(0, -0.078, y + 0.452), material=M.marble, plane='YZ', rot=(0, 0, 180), bevel=0.006, bevel_segments=2)
        for s in (-1, 1):
            G.sphere(f'lid{s}', 0.5, at=(s * 0.028, -0.074, y + 0.437), material=M.marble, scale=(0.026, 0.012, 0.012),
                     segments=10, rings=5)
            G.sphere(f'ear{s}', 0.5, at=(s * 0.07, 0.004, y + 0.425), material=M.marble, scale=(0.018, 0.03, 0.048),
                     rot=(0, 0, s * 20), segments=10, rings=6)
        for k, z in enumerate((0.384, 0.374)):
            G.sphere(f'lip{k}', 0.5, at=(0, -0.074, y + z), material=M.marble, scale=(0.03, 0.012, 0.009), segments=10, rings=5)
        G.sphere('chin', 0.5, at=(0, -0.062, y + 0.352), material=M.marble, scale=(0.036, 0.026, 0.026), segments=10, rings=6)
        # the hair: a curly cap from the brow over the crown to the nape
        G.jitter(G.boulder('hair', 0.085, at=(0, 0.016, y + 0.4), material=M.marble, scale=(0.95, 1.05, 0.8), seed=3.0,
                           noise=0.18, subdivisions=3), 0.005, size=0.012, seed=1.0)
    G.join_all('bustMarble')
    G.settle()
    return G.meshes()


def statueCat():
    """A concrete cat sitting up: its rump with a thigh bulging at either side, the hind feet tucked forward by the
    front legs and paws (toes marked), a chest, a head with a muzzle, whisker pads, a chin, a nose and lidded eyes,
    pointed ears with hollows, and its tail curling round its front paws."""
    with G.new_parts():
        G.sphere('rump', 0.5, at=(0, 0.07, 0.12), material=M.concrete, scale=(0.22, 0.26, 0.23), segments=20, rings=12)
        for s in (-1, 1):
            G.sphere(f'thigh{s}', 0.5, at=(s * 0.085, 0.03, 0.1), material=M.concrete, scale=(0.11, 0.22, 0.17),
                     rot=(-20, 0, 0), segments=16, rings=10)
            G.sphere(f'hind_foot{s}', 0.5, at=(s * 0.1, -0.08, 0.018), material=M.concrete, scale=(0.055, 0.1, 0.036),
                     segments=10, rings=6)
        G.sphere('chest', 0.5, at=(0, -0.04, 0.26), material=M.concrete, scale=(0.19, 0.18, 0.32), rot=(-12, 0, 0), segments=18, rings=12)
        G.sphere('head', 0.5, at=(0, -0.07, 0.46), material=M.concrete, scale=(0.19, 0.17, 0.16), segments=18, rings=12)
        G.sphere('muzzle', 0.5, at=(0, -0.145, 0.43), material=M.concrete, scale=(0.08, 0.05, 0.05), segments=12, rings=8)
        G.sphere('chin', 0.5, at=(0, -0.14, 0.408), material=M.concrete, scale=(0.04, 0.03, 0.025), segments=8, rings=6)
        G.sphere('nose', 0.5, at=(0, -0.168, 0.445), material=M.concrete_dark, scale=(0.02, 0.012, 0.012), segments=8, rings=4)
        for s in (-1, 1):
            G.sphere(f'pad{s}', 0.5, at=(s * 0.018, -0.162, 0.428), material=M.concrete, scale=(0.034, 0.026, 0.026),
                     segments=8, rings=6)
            G.sphere(f'eye{s}', 0.5, at=(s * 0.04, -0.152, 0.475), material=M.concrete_dark, scale=(0.026, 0.01, 0.016),
                     rot=(0, s * 12, 0), segments=8, rings=4)
            G.sphere(f'lid{s}', 0.5, at=(s * 0.04, -0.15, 0.481), material=M.concrete, scale=(0.03, 0.012, 0.008),
                     rot=(0, s * 12, 0), segments=8, rings=4)
            # the ear: a pointed cone, flattened front to back, its front hollowed
            G.loft(f'ear{s}', [G.ring(rx, ry, z, 8, at=(s * (0.052 + 0.012 * t), -0.062)) for t, z, rx, ry in (
                (0, 0.53, 0.036, 0.02), (0.5, 0.56, 0.022, 0.013), (1, 0.6, 0.003, 0.003))], material=M.concrete)
            G.loft(f'ear_hollow{s}', [G.ring(rx, ry, z, 8, at=(s * (0.052 + 0.012 * t), -0.077)) for t, z, rx, ry in (
                (0, 0.535, 0.024, 0.004), (1, 0.585, 0.003, 0.002))], material=M.concrete_dark)
            G.sweep(f'leg{s}', [(s * 0.045, -0.12, 0.23), (s * 0.044, -0.13, 0.14), (s * 0.045, -0.14, 0.05),
                                (s * 0.045, -0.145, 0.025)], 0.024, material=M.concrete, verts=10, resolution=2,
                    scale=[1.15, 1.0, 0.9, 0.95])
            G.sphere(f'paw{s}', 0.5, at=(s * 0.045, -0.15, 0.017), material=M.concrete, scale=(0.06, 0.07, 0.035), segments=10, rings=6)
            for t in (-1, 1):
                G.cube(f'toe{s}{t}', (0.002, 0.03, 0.012), at=(s * 0.045 + t * 0.01, -0.172, 0.02), material=M.concrete_dark)
        # the tail: one tapering sweep from the rump round the side to the front paws
        G.sweep('tail', [(-0.08, 0.17, 0.035), (-0.13, 0.1, 0.028), (-0.148, 0.0, 0.026), (-0.128, -0.11, 0.026),
                         (-0.08, -0.18, 0.024), (-0.03, -0.195, 0.024)], 0.022, material=M.concrete, verts=10,
                resolution=3, scale=[1.0, 1.0, 0.95, 0.9, 0.85, 0.7])
    G.join_all('statueCat')
    G.settle()
    return G.meshes()


def _ellipse_profile(a, b, n=10):
    return [(a * math.cos(math.tau * k / n), b * math.sin(math.tau * k / n)) for k in range(n)]


def statueHorse():
    """A bronze horse standing on a bronze base plate on an oak pedestal: a lofted barrel dipping at the back, a
    crested neck and a tapering head with a jowl and muzzle, ears, a flowing mane and forelock, legs with knees,
    hocks and hooves, and a full tail, in a dark patina."""
    y = 0.92
    with G.new_parts():
        _pedestal(y, 0.4, M.oak)
        G.extrude('base', G.rounded_rect(0.34, 0.16, 0.03, corner=3), 0.02, at=(0, 0, y + 0.01), material=M.bronze,
                  plane='XY', bevel=0.004, bevel_segments=1)
        # the barrel along the length (Blender x, its head to -x): rings across it, lofted, turned to lie along x
        rings = [(0.0, 0.03, 0.025, 0.012), (0.02, 0.062, 0.042, 0.012), (0.05, 0.072, 0.052, 0.01), (0.1, 0.066, 0.056, 0.0),
                 (0.16, 0.064, 0.055, -0.004), (0.22, 0.068, 0.054, 0.004), (0.26, 0.064, 0.05, 0.008), (0.29, 0.05, 0.042, 0.004),
                 (0.305, 0.022, 0.024, 0.0)]
        G.loft('barrel', [G.ring(rv, rw, z, 16, at=(dz, 0)) for z, rv, rw, dz in rings], material=M.bronze,
               at=(0.152, 0, y + 0.272), rot=(0, -90, 0))
        G.sweep('neck', [(-0.11, 0, y + 0.29), (-0.135, 0, y + 0.36), (-0.155, 0, y + 0.42), (-0.162, 0, y + 0.445)], None,
                material=M.bronze, profile=_ellipse_profile(0.038, 0.026), up=(1, 0, 0), resolution=2,
                scale=[1.45, 1.1, 0.85, 0.75])
        G.sweep('head', [(-0.15, 0, y + 0.445), (-0.18, 0, y + 0.415), (-0.205, 0, y + 0.38), (-0.225, 0, y + 0.355)], None,
                material=M.bronze, profile=_ellipse_profile(0.026, 0.02), up=(-1, 0, 1), resolution=2, scale=[1.0, 1.05, 0.85, 0.8])
        G.sphere('jowl', 0.5, at=(-0.17, 0, y + 0.415), material=M.bronze, scale=(0.05, 0.042, 0.045), segments=10, rings=6)
        G.sphere('muzzle', 0.023, at=(-0.222, 0, y + 0.352), material=M.bronze, scale=(1.1, 0.95, 1), segments=10, rings=6)
        for s in (-1, 1):
            G.cone(f'ear{s}', 0.011, 0.002, 0.04, at=(-0.14, s * 0.012, y + 0.47), material=M.bronze, rot=(s * -10, -12, 0),
                   verts=6)
        # the mane along the crest, falling to one side, and a forelock between the ears
        G.sweep('mane', [(-0.098, 0.004, y + 0.33), (-0.112, 0.008, y + 0.37), (-0.128, 0.01, y + 0.41), (-0.14, 0.008, y + 0.445)],
                None, material=M.bronze_dark, profile=G.rounded_rect(0.036, 0.012, 0.005, corner=1), up=(1, 0, 1), resolution=2,
                scale=[0.7, 1.0, 1.0, 0.7])
        G.sweep('forelock', [(-0.15, 0, y + 0.462), (-0.168, 0, y + 0.445), (-0.178, 0, y + 0.428)], 0.008,
                material=M.bronze_dark, verts=5, resolution=2, scale=[1.0, 0.8, 0.4])
        G.sweep('tail', [(0.145, 0, y + 0.31), (0.175, 0, y + 0.3), (0.19, 0.004, y + 0.24), (0.188, 0.008, y + 0.17),
                         (0.19, 0.004, y + 0.12)], 0.015, material=M.bronze_dark, verts=8, resolution=2,
                scale=[0.7, 1.0, 1.15, 1.0, 0.6])
        # legs: (hip or shoulder, knee or hock, fetlock, hoof), each tapering, a hoof under it
        for x, joints in ((-0.1, ((-0.1, 0.25), (-0.104, 0.135), (-0.108, 0.055), (-0.112, 0.03))),
                          (0.1, ((0.1, 0.26), (0.128, 0.13), (0.112, 0.055), (0.11, 0.03)))):
            for d in (-0.035, 0.035):
                G.sweep(f'leg{x}{d}', [(jx, d, y + jy) for jx, jy in joints], 0.0135, material=M.bronze, verts=8, resolution=2,
                        scale=[2.0 if x > 0 else 1.6, 1.05, 0.95, 0.85])
                G.cone(f'hoof{x}{d}', 0.017, 0.013, 0.022, at=(joints[-1][0], d, y + 0.031), material=M.bronze, verts=10)
    G.join_all('statueHorse')
    G.settle()
    return G.meshes()


COAT_AT, COAT_HEM, COAT_TURN = (0.1, 0.07), 0.97, 30   # where the coat hangs (game x, depth), its hem's height, its turn


def _coat(F):
    """A coat hung by its collar: a body flaring a little to the hem, lapels and three buttons down its front, pocket
    flaps, sleeves hanging at its sides with turned-back cuffs, a collar rolled round its neck."""
    c, turn = F.at(COAT_AT[0], 0, COAT_AT[1]), math.radians(COAT_TURN)

    def at(x, y, h):
        """A point in the coat's own frame (Blender x across it, y through it, h up from the hem)."""
        return (c[0] + x * math.cos(turn) - y * math.sin(turn), c[1] + x * math.sin(turn) + y * math.cos(turn), COAT_HEM + h)

    G.revolve('coat', [(0, 0), (0.125, 0), (0.13, 0.015), (0.12, 0.1), (0.108, 0.3), (0.112, 0.45), (0.105, 0.53),
                       (0.075, 0.585), (0.035, 0.6), (0, 0.6)],
              at=at(0, 0, 0), material=M.coat, segments=20, scale=(1, 0.45, 1), rot=(0, 0, COAT_TURN))
    G.sweep('coat_collar', [at(0.07 * math.sin(a), 0.035 * math.cos(a), 0.585 + 0.01 * math.cos(a))
                            for a in (math.radians(-120 + 24 * k) for k in range(11))], 0.014, material=M.coat, verts=6,
            resolution=2)
    for s in (-1, 1):
        G.extrude(f'lapel{s}', [(0, 0), (s * 0.05, 0.11), (s * 0.03, 0.135), (s * 0.006, 0.13)], 0.006,
                  at=at(s * 0.006, -0.05, 0.43), material=M.coat, plane='XZ', rot=(-6, 0, COAT_TURN))
        G.extrude(f'pocket{s}', G.rounded_rect(0.06, 0.022, 0.004, corner=1), 0.006, at=at(s * 0.065, -0.048, 0.2),
                  material=M.coat, plane='XZ', rot=(0, 0, COAT_TURN + s * 18))
        G.sweep(f'sleeve{s}', [at(s * 0.1, -0.005, 0.53), at(s * 0.118, -0.012, 0.36), at(s * 0.122, -0.022, 0.15)], 0.027,
                material=M.coat, verts=10, resolution=2, scale=[1.0, 0.95, 0.9])
        G.torus(f'cuff{s}', 0.024, 0.006, at=at(s * 0.122, -0.022, 0.16), material=M.coat, major_segments=10,
                minor_segments=4, rot=(0, 0, COAT_TURN))
    for k in range(3):
        G.sphere(f'button{k}', 0.007, at=at(0, -0.054 + 0.004 * k, 0.36 - 0.09 * k), material=M.brass, segments=8, rings=4)


def coatRack():
    """A turned oak coat stand: four legs curving down from a hub to round pads, a pole with beads under each ring
    of brass S-hooks and a ball finial; a coat, a fringed scarf over a hook and a hat with a band."""
    F = G.GameFrame()
    with G.new_parts():
        prof = [(0, 0), (0.03, 0), (0.042, 0.02), (0.044, 0.05), (0.036, 0.07), (0.026, 0.085), (0.023, 0.12), (0.021, 1.2),
                (0.026, 1.215), (0.026, 1.235), (0.02, 1.25), (0.019, 1.39), (0.024, 1.405), (0.024, 1.425), (0.019, 1.44),
                (0.017, 1.54), (0.013, 1.565), (0, 1.57)]
        G.revolve('pole', prof, at=F.at(0, 0.17, 0), material=M.oak, segments=14)
        F.sphere('top', 0.038, (0, 1.75, 0), M.oak, segments=14, rings=8)
        for k in range(4):
            a = math.radians(45 + k * 90)
            c, s = math.cos(a), math.sin(a)
            G.kit.tube(F, f'leg{k}', [(r * c, h, r * s) for r, h in ((0.02, 0.24), (0.08, 0.205), (0.15, 0.1), (0.19, 0.03))],
                       0.016, M.oak, verts=8, round=3, scale=[1.25, 1.05, 0.85, 0.75])
            G.revolve(f'foot{k}', [(0, 0), (0.022, 0), (0.024, 0.008), (0.016, 0.022), (0, 0.024)],
                      at=F.at(0.2 * c, 0.0, 0.2 * s), material=M.oak, segments=10)
        for k, y in enumerate((1.62, 1.62, 1.62, 1.62, 1.44, 1.44)):
            a = math.radians(k * 90 + (45 if k > 3 else 0) + (180 if k == 5 else 0))
            c, s = math.cos(a), math.sin(a)
            G.kit.tube(F, f'hook{k}', [(r * c, y + h, r * s) for r, h in ((0.015, 0.0), (0.06, 0.004), (0.1, 0.03), (0.117, 0.06))],
                       0.0055, M.brass, verts=6, round=3)
            F.sphere(f'hook_tip{k}', 0.011, (0.12 * c, y + 0.07, 0.12 * s), M.brass, segments=8, rings=6)
        # a coat hanging from a hook on the right-front, a scarf over the lower left one, a hat on the top hook
        _coat(F)
        sx, sy, sz = 0.085, 1.5, -0.085
        for side, d in (('front', 1), ('back', -1)):
            F.sweep(f'scarf_{side}', [(sx, sy + 0.012, sz), (sx + 0.02 * d, sy - 0.02, sz + 0.028 * d),
                                  (sx + 0.035 * d, sy - 0.2, sz + 0.035 * d), (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d)],
                None, M.scarf, profile=G.rounded_rect(0.07, 0.012, 0.005, corner=1), up=(1, 0, -1), resolution=3)
            end = (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d)
            for i in range(5):
                G.kit.tube(F, f'fringe_{side}{i}', [(end[0] + (i - 2) * 0.012 * 0.7, end[1], end[2] + (i - 2) * 0.012 * 0.7),
                                                     (end[0] + (i - 2) * 0.012 * 0.7, end[1] - 0.035, end[2] + (i - 2) * 0.012 * 0.7)],
                           0.0025, M.scarf, verts=4, round=1)
        G.revolve('hat', [(0, 0), (0.12, 0), (0.122, 0.006), (0.11, 0.012), (0.072, 0.012), (0.068, 0.03), (0.066, 0.07),
                          (0.058, 0.088), (0.03, 0.092), (0.012, 0.084), (0, 0.086)],
                  at=F.at(-0.09, 1.684, -0.06), material=M.hat, segments=24)
        G.torus('hat_band', 0.069, 0.007, at=F.at(-0.09, 1.703, -0.06), material=M.scarf, major_segments=24, minor_segments=4)
    G.join_all('coatRack')
    G.settle()
    return G.meshes()


# --- D2: the hanging plant --------------------------------------------------------------------------------
CEILING = 2.8


def hangingPlant():
    """A pot with a rolled rim in a macramé hanger: a cord from the ceiling hook to a ring, a gathering knot, three
    knotted strands that cross into a net round the pot and gather again in a tasselled knot below it; leafy, its
    vines trailing with heart leaves. It sways when watered: the whole hanger swings about the hook."""
    F = G.GameFrame()
    with G.new_parts() as still:
        G.revolve('hook_plate', [(0, 0), (0.008, 0), (0.012, 0.004), (0.03, 0.008), (0.03, 0.012), (0, 0.012)],
                  at=F.at(0, CEILING - 0.012, 0), material=M.cord, segments=14)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as parts:
        G.kit.tube(F, 'cord', [(0, CEILING - 0.01, 0), (0, 1.84, 0)], 0.005, M.cord, verts=6, round=1)
        G.torus('ring', 0.016, 0.004, at=F.at(0, 1.825, 0), material=M.cord, rot=(90, 0, 0), major_segments=10, minor_segments=4)
        F.sphere('gather_knot', 0.016, (0, 1.79, 0), M.cord, scale=(1, 1, 1.4), segments=8, rings=6)
        corners = [(math.cos(math.radians(90 + k * 120)), math.sin(math.radians(90 + k * 120))) for k in range(3)]
        for k, (c, s) in enumerate(corners):
            G.kit.tube(F, f'hanger{k}', [(0.008 * c, 1.78, 0.008 * s), (0.1 * c, 1.6, 0.1 * s), (0.162 * c, 1.44, 0.162 * s)],
                       0.0045, M.cord, verts=5, round=2)
            for i, h in enumerate((1.62, 1.5)):
                r = 0.008 + (0.162 - 0.008) * (1.78 - h) / 0.34
                F.sphere(f'knot{k}_{i}', 0.009, (r * c, h, r * s), M.cord, scale=(1, 1, 1.3), segments=6, rings=4)
            # the strand splits: each half crosses to meet its neighbour's under the pot's belly
            c2, s2 = corners[(k + 1) % 3]
            mc, ms = (c + c2) / 2, (s + s2) / 2
            ml = math.hypot(mc, ms)
            mc, ms = mc / ml, ms / ml
            for j, (ec, es) in enumerate(((c, s), (c2, s2))):
                dc, ds = ec + mc, es + ms
                dl = math.hypot(dc, ds)
                G.kit.tube(F, f'net{k}_{j}', [(0.162 * ec, 1.44, 0.162 * es), (0.157 * dc / dl, 1.37, 0.157 * ds / dl),
                                              (0.146 * mc, 1.3, 0.146 * ms)], 0.0035, M.cord, verts=4, round=2)
            F.sphere(f'net_knot{k}', 0.008, (0.146 * mc, 1.3, 0.146 * ms), M.cord, segments=6, rings=4)
            G.kit.tube(F, f'strand{k}', [(0.146 * mc, 1.3, 0.146 * ms), (0.125 * mc, 1.235, 0.125 * ms), (0.07 * mc, 1.19, 0.07 * ms),
                                         (0.01 * mc, 1.165, 0.01 * ms)], 0.004, M.cord, verts=4, round=2)
        F.sphere('base_knot', 0.02, (0, 1.155, 0), M.cord, scale=(1, 1, 1.3), segments=8, rings=6)
        for i in range(6):
            a = math.radians(i * 60)
            G.kit.tube(F, f'tassel{i}', [(0.006 * math.cos(a), 1.14, 0.006 * math.sin(a)),
                                         (0.016 * math.cos(a), 1.06, 0.016 * math.sin(a))], 0.0035, M.cord, verts=4, round=1)
        G.revolve('pot', [(0, 0), (0.1, 0), (0.104, 0.006), (0.142, 0.12), (0.15, 0.2), (0.16, 0.205), (0.163, 0.218),
                          (0.16, 0.24), (0.148, 0.24), (0.14, 0.215), (0.13, 0.13), (0, 0.13)],
                  at=F.at(0, 1.2, 0), material=M.pot_white, segments=28)
        G.torus('pot_band', 0.148, 0.007, at=F.at(0, 1.33, 0), material=M.cord, major_segments=28, minor_segments=4)
        for k, (dx, dy, dz, r) in enumerate(((0, 1.5, 0, 0.17), (-0.12, 1.47, 0.06, 0.11), (0.12, 1.46, -0.05, 0.11),
                                             (0.04, 1.47, 0.13, 0.1), (-0.04, 1.46, -0.13, 0.1))):
            G.jitter(G.sphere(f'leaves{k}', r, at=F.at(dx, dy, dz), material=M.leaf, scale=(1, 1, 0.75), segments=14, rings=8),
                     0.02, size=0.05, seed=k * 0.9)
        for k, (x, z, length) in enumerate(((0.18, 0.08, 0.45), (-0.17, -0.06, 0.38), (0.05, -0.17, 0.3), (-0.06, 0.16, 0.34))):
            pts = [(x * (0.8 + 0.2 * t) + 0.012 * math.sin(7 * t + k), 1.42 - length * t, z * (0.8 + 0.2 * t))
                   for t in (i / 6 for i in range(7))]
            pts[-1] = (pts[-1][0], 1.42 - length + 0.042, pts[-1][2])
            F.sweep(f'vine{k}', pts, 0.004, M.leaf_light, verts=4, resolution=2)
            for i, p in enumerate(pts[1:], 1):
                _heart(F, f'vine{k}_leaf{i}', p, 0.05 - 0.004 * i, 50 * ((i * 3 + k) % 7) - 150, 150 + 10 * (i % 2),
                       M.leaf if (i + k) % 2 else M.leaf_light)
    plant = G.hinged(G.join(parts, 'plant'), F.at(0, CEILING, 0))
    G.settle(floor=False)
    # one sway as it's watered: out (eased) and, played back after the hold, home; a rock (out, past rest and
    # home) played back would swing it twice
    G.swing(plant, 'sway', 5, seconds=0.8, axis='X')
    return G.meshes()


# --- D2: the fireplace ------------------------------------------------------------------------------------
HEARTH = 0.42   # the raised hearth's top: the content's `seat` height, where residents warming up sit
MANTEL_Z = WALL + 0.38   # the mantel's things stand here, in front of the flue (its front WALL + 0.28)
HEARTH_FRONT = 0.45   # its front edge, game depth: 0.46 m in front of the firebox's opening (at the tile's middle),
                      # so a resident sitting on its edge (content `seat.forward` 0.36) has the fire behind her


def _log(F, name, a, b, r, seed):
    """A split log from game point `a` to `b`: a barked, slightly crooked round, its cut ends pale."""
    va, vb = Vector(F.at(*a)), Vector(F.at(*b))
    mid = (va + vb) / 2 + Vector((0, 0, r * 0.15))
    log = G.sweep(name, [tuple(va), tuple(mid), tuple(vb)], r, material=M.log, verts=10, resolution=2,
                  scale=[1.0, 1.06, 0.94])
    G.jitter(log, r * 0.12, size=0.03, seed=seed)
    d = (vb - va).normalized()
    for k, (p, s) in enumerate(((va, -1), (vb, 1))):
        end = G.cylinder(f'{name}_end{k}', r * 0.86, 0.004, material=M.log_end, verts=10)
        end.location = p + d * s * 0.001
        end.rotation_mode = 'QUATERNION'
        end.rotation_quaternion = d.to_track_quat('Z', 'Y')


def fireplace():
    """A painted chimney breast with fielded panels and a crown moulding, an oak mantel shelf with a moulded edge on
    scroll corbels, a moulded surround with corner blocks and a keystone round a firebox lined in sooty brick, a
    raised hearth bench of stone slabs, an iron grate with finials holding split logs, and things on the mantel.
    The flames grow on the logs while someone warms up by it."""
    F = G.GameFrame()
    with G.new_parts() as still:
        # the chimney breast, built round the firebox's opening (x ±0.44, from the hearth bench up 0.66 m)
        top = HEARTH + 0.66
        for side in (-1, 1):
            F.box(f'breast{side}', 0.48, 1.25, 0.4, (side * 0.68, 0.625, WALL + 0.2), M.surround, bevel=0.012)
        F.box('breast_top', 0.9, 1.25 - top, 0.4, (0, (1.25 + top) / 2, WALL + 0.2), M.surround, bevel=0.012)
        F.box('firebox_back', 0.9, top - HEARTH + 0.02, 0.1, (0, (top + HEARTH) / 2, WALL + 0.05), M.soot)
        # sooty brick lining the back, in stretcher bond
        for row in range(10):
            y = HEARTH + 0.03 + row * 0.065
            off = 0.1 if row % 2 else 0.0
            for i in range(-3, 4):
                x0, x1 = max(-0.43, i * 0.2 + off - 0.096), min(0.43, i * 0.2 + off + 0.096)
                if x1 - x0 > 0.03 and y < top - 0.02:
                    F.box(f'brick{row}_{i}', x1 - x0, 0.058, 0.01, ((x0 + x1) / 2, y, WALL + 0.105), M.brick)
        for side in (-1, 1):
            F.box(f'firebox_side{side}', 0.012, top - HEARTH, 0.3, (side * 0.434, (top + HEARTH) / 2, WALL + 0.25), M.firebox)
        F.box('firebox_top', 0.88, 0.012, 0.3, (0, top - 0.006, WALL + 0.25), M.firebox)
        F.box('flue', 1.6, 1.55, 0.28, (0, 2.025, WALL + 0.14), M.surround, bevel=0.012)
        G.kit.moulding(F, 'crown', [(-0.8, WALL + 0.02), (0.8, WALL + 0.02), (0.8, WALL + 0.28), (-0.8, WALL + 0.28)],
                       [(0.03, 0), (0.0, 0.0), (-0.006, 0.01), (-0.014, 0.022), (-0.018, 0.04), (0.03, 0.04)], M.surround,
                       at=2.7, plane='top')
        for side in (-1, 1):   # fielded panels on the breast either side of the opening
            F.extrude(f'panel{side}', G.rounded_rect(0.3, 0.64, 0.02, corner=2), 0.012, (side * 0.7, 0.8, WALL + 0.402),
                      M.surround, bevel=0.005, bevel_segments=1)
        F.extrude('panel_flue', G.rounded_rect(1.1, 1.0, 0.03, corner=2), 0.012, (0, 2.0, WALL + 0.282), M.surround,
                  bevel=0.005, bevel_segments=1)
        # the mantel: an oak shelf with a rounded nosing and a cove under it, on two scroll corbels
        shelf = G.smooth_outline([(-0.24, -0.035), (0.19, -0.035), (0.205, -0.028), (0.215, -0.012), (0.232, -0.002),
                                  (0.24, 0.012), (0.24, 0.028), (0.233, 0.035), (-0.24, 0.035)], passes=1)
        F.extrude('mantel', shelf, 1.96, (0, 1.285, WALL + 0.24), M.oak, plane='side')
        for side in (-1, 1):
            corbel = G.smooth_outline([(-0.05, 0.04), (0.03, 0.04), (0.035, 0.0), (0.012, -0.02), (0.02, -0.045),
                                       (0.0, -0.06), (-0.02, -0.05), (-0.05, -0.06)], passes=1)
            F.extrude(f'corbel{side}', corbel, 0.1, (side * 0.8, 1.21, WALL + 0.41), M.oak, plane='side')
        # the hearth: a stone bench from the wall out to 0.46 m in front of the opening, the firebox's floor, its top
        # three slabs with a nosing; residents warming up sit on its front edge, their backs to the fire (a real raised
        # hearth runs 0.4-0.5 m out)
        F.box('hearth', 1.94, HEARTH - 0.04, HEARTH_FRONT - WALL - 0.01, (0, (HEARTH - 0.04) / 2, (HEARTH_FRONT + WALL) / 2 - 0.005),
              M.hearth, bevel=0.01)
        for i, (x0, x1) in enumerate(((-0.98, -0.33), (-0.33, 0.33), (0.33, 0.98))):
            F.box(f'slab{i}', x1 - x0 - 0.004, 0.04, HEARTH_FRONT - WALL, ((x0 + x1) / 2, HEARTH - 0.02, (HEARTH_FRONT + WALL) / 2),
                  M.hearth, bevel=0.008, bevel_segments=2)
        # the surround: moulded jambs and lintel, square corner blocks with rosettes, a keystone
        jamb = G.smooth_outline([(-0.05, 0), (0.05, 0), (0.05, 0.02), (0.035, 0.03), (-0.035, 0.03), (-0.05, 0.02)], passes=1)
        for side in (-1, 1):
            F.sweep(f'surround_jamb{side}', [(side * 0.49, HEARTH, WALL + 0.4), (side * 0.49, top, WALL + 0.4)], None,
                    M.surround, profile=[(v, u) for u, v in jamb], up=(0, -1, 0))
            F.box(f'corner_block{side}', 0.12, 0.12, 0.034, (side * 0.49, top + 0.05, WALL + 0.417), M.surround,
                  bevel=0.006, bevel_segments=1)
            _disc(F, f'rosette{side}', side * 0.49, top + 0.05, WALL + 0.434, 0.035, M.surround, t=0.008, verts=16)
        F.sweep('surround_lintel', [(-0.43, top + 0.05, WALL + 0.4), (0.43, top + 0.05, WALL + 0.4)], None, M.surround,
                profile=[(v, u) for u, v in jamb], up=(0, -1, 0))
        F.extrude('keystone', [(-0.035, -0.06), (0.035, -0.06), (0.045, 0.06), (-0.045, 0.06)], 0.04,
                  (0, top + 0.05, WALL + 0.42), M.surround, bevel=0.005, bevel_segments=1)
        # the grate and the logs
        y = HEARTH
        for s in (-1, 1):
            G.kit.tube(F, f'grate_bar{s}', [(-0.3, y + 0.05, WALL + 0.25 + s * 0.08), (0.3, y + 0.05, WALL + 0.25 + s * 0.08)],
                       0.007, M.grate, verts=6, round=1)
            G.kit.tube(F, f'grate_leg{s}', [(s * 0.28, y, WALL + 0.16), (s * 0.28, y + 0.05, WALL + 0.17), (s * 0.28, y + 0.05, WALL + 0.33),
                                            (s * 0.28, y + 0.12, WALL + 0.37)], 0.007, M.grate, verts=6, round=2)
            G.revolve(f'grate_finial{s}', [(0, 0), (0.012, 0), (0.016, 0.012), (0.01, 0.024), (0.014, 0.034), (0, 0.044)],
                      at=F.at(s * 0.28, y + 0.12, WALL + 0.37), material=M.grate, segments=10)
        for k in range(5):
            G.kit.tube(F, f'grate_slat{k}', [(-0.22 + 0.11 * k, y + 0.05, WALL + 0.17), (-0.22 + 0.11 * k, y + 0.05, WALL + 0.33)],
                       0.005, M.grate, verts=5, round=1)
        _log(F, 'log0', (-0.25, y + 0.09, WALL + 0.21), (0.2, y + 0.09, WALL + 0.29), 0.045, 0.3)
        _log(F, 'log1', (-0.18, y + 0.09, WALL + 0.31), (0.26, y + 0.09, WALL + 0.2), 0.042, 1.7)
        _log(F, 'log2', (-0.12, y + 0.16, WALL + 0.25), (0.14, y + 0.17, WALL + 0.26), 0.038, 2.9)
        # on the mantel
        top = 1.32
        G.revolve('vase', [(0, 0), (0.036, 0), (0.04, 0.006), (0.055, 0.07), (0.05, 0.12), (0.034, 0.18), (0.026, 0.215),
                           (0.03, 0.232), (0.034, 0.24), (0.026, 0.24), (0, 0.22)], at=F.at(-0.65, top, MANTEL_Z), material=M.pot,
                  segments=20)
        for k, (x, h) in enumerate(((-0.4, 0.12), (-0.3, 0.18))):
            G.revolve(f'candle_holder{k}', [(0, 0), (0.04, 0), (0.04, 0.006), (0.016, 0.014), (0.012, 0.03), (0.018, 0.04),
                                            (0.03, 0.044), (0.03, 0.05), (0, 0.05)], at=F.at(x, top, MANTEL_Z), material=M.brass,
                      segments=14)
            G.revolve(f'candle{k}', [(0, 0), (0.024, 0), (0.024, h - 0.006), (0.02, h), (0.004, h + 0.002), (0, h + 0.002)],
                      at=F.at(x, top + 0.05, MANTEL_Z), material=M.candle, segments=12)
            G.kit.tube(F, f'wick{k}', [(x, top + 0.05 + h, MANTEL_Z), (x, top + 0.062 + h, MANTEL_Z)], 0.0015, M.soot, verts=4,
                       round=1)
        with G.new_parts() as pic:
            G.kit.moulding(F, 'picture_frame', _rect(0.3, 0.38, top + 0.19, 0.55), [(0, 0), (0, 0.02), (0.005, 0.025),
                                                                                    (0.03, 0.025), (0.035, 0.018), (0.035, 0)],
                           M.frame_white, at=WALL + 0.3)
            F.box('picture_art', 0.24, 0.32, 0.004, (0.55, top + 0.19, WALL + 0.304), M.picture)
            _shape(F, 'picture_hill', [(0.43, top + 0.04), (0.67, top + 0.04), (0.67, top + 0.12), (0.6, top + 0.16),
                                       (0.5, top + 0.11), (0.43, top + 0.14)], WALL + 0.306, M.olive, t=0.001)
        art = G.join(pic, 'picture')
        G.set_origin(art, F.at(0.55, top, WALL + 0.31))
        art.rotation_euler[0] = math.radians(-3)
    G.join(still, 'body', origin=(0, 0, 0))

    # the flames: an ember bed and tongues of fire, each lofted narrowing and leaning as it rises, joined at the
    # logs' top, grown from nothing
    base = HEARTH + 0.12
    with G.new_parts() as parts:
        F.box('embers', 0.5, 0.02, 0.16, (0, base, WALL + 0.25), M.ember, bevel=0.008)
        for k, (x, h, r, z, mat) in enumerate(((-0.14, 0.26, 0.07, 0.24, M.flame), (0.0, 0.36, 0.09, 0.26, M.flame),
                                               (0.13, 0.28, 0.07, 0.25, M.flame), (-0.06, 0.2, 0.05, 0.3, M.flame),
                                               (0.07, 0.22, 0.05, 0.21, M.flame), (-0.07, 0.17, 0.045, 0.24, M.flame_core),
                                               (0.06, 0.21, 0.05, 0.24, M.flame_core))):
            bx, by, _ = F.at(x, 0, WALL + z)
            lean = 0.02 * (1 if k % 2 else -1)
            G.loft(f'flame{k}', [G.ring(r * w, r * w * 0.45, base + h * t, 10,
                                        at=(bx + lean * math.sin(math.pi * t) * (1 + t), by))
                                 for t, w in ((0, 1.0), (0.2, 0.95), (0.45, 0.7), (0.7, 0.4), (0.9, 0.14), (1.0, 0.02))],
                   material=mat)
    flames = G.join(parts, 'flames', origin=F.at(0, base, WALL + 0.25))
    flames.scale[2] = 0.001
    G.settle(centre=False)
    G.grow(flames, 'flames', seconds=0.8)
    return G.meshes()


# --- D3: curtains -----------------------------------------------------------------------------------------
def _pleats(name, x_out, x_in, y0, y1, z, material, waves=4, amp=0.018, t=0.008, F=None):
    """A gathered curtain panel from game x `x_out` to `x_in` and height `y0` to `y1`, its pleats a sine round game
    depth `z`, `t` thick: deepest at the hem, where a turned-up hem thickens it, easing up the drop and drawn flat
    into the heading tape at the top; each fold drifting a little as it falls."""
    n = waves * 8
    xs = [x_out + (x_in - x_out) * k / n for k in range(n + 1)]
    h = y1 - y0
    rows = [(y0, 1.0, 1.4, 0.0), (y0 + 0.012, 1.0, 1.4, 0.02), (y0 + 0.05, 1.0, 1.0, 0.05), (y0 + h * 0.35, 0.94, 1.0, 0.25),
            (y0 + h * 0.7, 0.84, 1.0, 0.4), (y1 - 0.075, 0.74, 1.0, 0.45), (y1 - 0.065, 0.42, 1.3, 0.45), (y1, 0.38, 1.3, 0.45)]
    verts, ring_len = [], 0
    for y, af, tf, phase in rows:
        front = [(x, z + t * tf / 2 + amp * af * math.sin(2 * math.pi * waves * k / n + phase)) for k, x in enumerate(xs)]
        back = [(x, d - t * tf) for x, d in front]
        ring = front + back[::-1]
        ring_len = len(ring)
        verts += [F.at(x, y, d) for x, d in ring]
    m = ring_len
    faces = []
    for r in range(len(rows) - 1):
        a, b = r * m, (r + 1) * m
        faces += [(a + i, a + (i + 1) % m, b + (i + 1) % m, b + i) for i in range(m)]
    smooth = len(faces)
    for base in (0, (len(rows) - 1) * m):   # the bottom and top edges: a strip of quads across the cloth's thickness
        faces += [(base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k) for k in range(n)]
    return _mesh(name, verts, faces, material, smooth=smooth)   # the cloth smooth, its hems flat


def _curtains(w, top, bottom, gathered):
    """Two pleated panels on a rail, gathered open at the sides (rest). At night `close` draws them across:
    each widens from its outer end until they meet in the middle. The rail ends in turned finials, on brackets
    from round wall plates."""
    F = G.GameFrame()
    rail_y, z = top + 0.05, WALL + 0.08
    with G.new_parts() as still:
        G.kit.tube(F, 'rail', [(-(w / 2 + 0.03), rail_y, z), (w / 2 + 0.03, rail_y, z)], 0.012, M.rail, verts=12, round=1)
        for s in (-1, 1):
            G.revolve(f'finial{s}', [(0, 0), (0.014, 0), (0.016, 0.006), (0.011, 0.012), (0.018, 0.022), (0.024, 0.038),
                                     (0.018, 0.054), (0.006, 0.062), (0, 0.064)],
                      at=F.at(s * (w / 2 + 0.008), rail_y, z), material=M.rail, segments=14, rot=(0, -s * 90, 0))
            G.kit.tube(F, f'bracket{s}', [(s * (w / 2 - 0.02), rail_y - 0.035, WALL + 0.008), (s * (w / 2 - 0.02), rail_y - 0.03, z - 0.03),
                                          (s * (w / 2 - 0.02), rail_y - 0.012, z - 0.004), (s * (w / 2 - 0.02), rail_y, z)],
                       0.006, M.rail, verts=8, round=2)
            G.revolve(f'bracket_plate{s}', [(0, 0), (0.022, 0), (0.022, 0.004), (0.016, 0.008), (0.008, 0.01), (0, 0.01)],
                      at=F.at(s * (w / 2 - 0.02), rail_y - 0.035, WALL), material=M.rail, segments=14, rot=(90, 0, 0))
    G.join(still, 'body', origin=(0, 0, 0))
    panels = []
    for s in (-1, 1):
        x_out = s * w / 2
        with G.new_parts() as parts:
            _pleats(f'panel{s}', x_out, x_out - s * gathered, bottom, top + 0.02, z, M.linen, waves=max(3, round(gathered / 0.06)), F=F)
            for k in range(4):
                # cloth tabs over the rail (the panel's own fabric: no draw call of their own)
                G.torus(f'tab{s}_{k}', 0.018, 0.006, at=F.at(x_out - s * (0.02 + k * (gathered - 0.04) / 3), rail_y, z),
                        material=M.linen, rot=(0, 90, 0), major_segments=12, minor_segments=5)
        panels.append(G.hinged(G.join(parts, 'panel_left' if s < 0 else 'panel_right'), F.at(x_out, rail_y, z)))
    G.settle(centre=False, floor=False)
    for panel in panels:
        G.squash(panel, 'close', ((w / 2 + 0.01) / gathered, 1, 1), seconds=1.2)
    return G.meshes()


# A window's opening is its edges less 0.12 m at each end (house.ts WIN_INSET): 0.76 m over one edge, 1.76 m over
# two. A rail runs past it, so the open curtains stack on the wall beside the glass, not over it: each pair's
# `w` is the opening plus its two gathered panels (rails 1.22 and 2.38 m, 0.1-0.2 m past the footprint each side).
# The window-height pairs hang from above the head (2.1 m) to just under the sill (0.9 m).
def curtains():
    return _curtains(1.16, 2.45, 0.85, 0.2)


def curtainsWide():
    return _curtains(2.32, 2.45, 0.85, 0.3)


def curtainsLong():
    return _curtains(2.32, 2.55, 0.04, 0.3)


# --- D3: rugs ---------------------------------------------------------------------------------------------
# Rugs lie flat (Blender axes, centred): a bound border 1.2 cm, the field in it to 1.6 cm and what is woven into it
# raised to at most 1.9 cm (the size's height), so the house's floor, drawn 1 cm up, hides none of it.
def _flat_shape(name, pts, z0, z1, material):
    """A motif woven into a rug: the outline `pts` (Blender x, y) from height `z0` to `z1`."""
    return G.extrude(name, pts, z1 - z0, at=(0, 0, (z0 + z1) / 2), material=material, plane='XY')


def _diamond(cx, cy, rx, ry, steps=0):
    """A diamond's outline round (cx, cy), `rx` and `ry` to its points; `steps` > 0 cuts its edges into that many
    stairs, as a knotted rug's diagonals are."""
    pts = [(cx + rx, cy), (cx, cy + ry), (cx - rx, cy), (cx, cy - ry)]
    if not steps:
        return pts
    out = []
    for (px, py), (qx, qy) in zip(pts, pts[1:] + pts[:1]):
        dx, dy = (qx - px) / steps, (qy - py) / steps
        for i in range(steps):
            ax, ay = px + dx * i, py + dy * i
            out.append((ax, ay))
            # the stair's corner on the outside of the diagonal
            c1, c2 = (ax, ay + dy), (ax + dx, ay)
            far = lambda c: abs(c[0] - cx) / rx + abs(c[1] - cy) / ry  # noqa: E731
            out.append(c1 if far(c1) > far(c2) else c2)
    return out


def _fringe(name, xs, y_edge, length, material, side=1, along='y'):
    """A fringe of knotted tassels lying on the floor, one at each of `xs` across the rug's end at `y_edge`, `length`
    out from it (`along` the end's normal: Blender 'y' or 'x'), each a wavy cord with a knot where it leaves the
    rug."""
    for i, x in enumerate(xs):
        wob = 0.006 * math.sin(i * 2.3)
        pts = [(x, y_edge - side * 0.004), (x + wob * 0.4, y_edge + side * length * 0.45), (x + wob, y_edge + side * length)]
        if along == 'x':
            pts = [(b, a) for a, b in pts]
        G.sweep(f'{name}{i}', [(px, py, 0.0034) for px, py in pts], 0.0026, material=material, verts=4, resolution=2,
                scale=[1.3, 1.0, 0.8])


def _rug_rect(name, sx, sz, field, border, line):
    """A flat rug `sx` wide and `sz` deep: a border, a raised field and a line round it, 2 cm high."""
    G.cube(f'{name}_border', (sx, sz, 0.012), at=(0, 0, 0.006), material=border, bevel=0.005, bevel_segments=3)
    G.cube(f'{name}_field', (sx - 0.24, sz - 0.24, 0.006), at=(0, 0, 0.013), material=field, bevel=0.002)
    lw = 0.03
    for s in (-1, 1):
        G.cube(f'{name}_line_x{s}', (sx - 0.44, lw, 0.004), at=(0, s * (sz - 0.44 - lw) / 2, 0.017), material=line, bevel=0.0015)
        G.cube(f'{name}_line_z{s}', (lw, sz - 0.44 - 2 * lw, 0.004), at=(s * (sx - 0.44 - lw) / 2, 0, 0.017), material=line, bevel=0.0015)


def rugRound():
    """A round rug: a darker bound border, a field with a thin ring, a garland of sixteen cream leaves round a
    stepped star medallion with a field-coloured heart."""
    d = 1.9
    with G.new_parts():
        G.cylinder('border', d / 2, 0.012, at=(0, 0, 0.006), material=M.rug_border, verts=72, bevel=0.005, bevel_segments=3)
        G.cylinder('field', d / 2 - 0.12, 0.006, at=(0, 0, 0.013), material=M.rug, verts=72, bevel=0.002)
        G.revolve('line', [(d / 2 - 0.25, 0.015), (d / 2 - 0.22, 0.015), (d / 2 - 0.22, 0.019), (d / 2 - 0.25, 0.019),
                           (d / 2 - 0.25, 0.015)], material=M.rug_border, segments=72, smooth=False)
        for k in range(16):
            a = math.tau * k / 16
            leaf = [(0.5 * 0.13 * (2 * t - 1), 0.03 * math.sin(math.pi * t)) for t in (i / 6 for i in range(7))]
            leaf = leaf + [(u, -v) for u, v in reversed(leaf[1:-1])]
            r = 0.52
            pts = [(r * math.cos(a) + u * -math.sin(a) + v * math.cos(a), r * math.sin(a) + u * math.cos(a) + v * math.sin(a))
                   for u, v in leaf]
            _flat_shape(f'leaf{k}', pts, 0.0155, 0.019, M.rug_accent)
        star = []
        for k in range(16):
            a = math.tau * k / 16
            rr = 0.3 if k % 2 == 0 else 0.2
            star.append((rr * math.cos(a), rr * math.sin(a)))
        _flat_shape('medallion', star, 0.0155, 0.0175, M.rug_border)
        _flat_shape('medallion_heart', [(0.11 * math.cos(math.tau * k / 8 + math.pi / 8), 0.11 * math.sin(math.tau * k / 8 + math.pi / 8))
                                        for k in range(8)], 0.0165, 0.019, M.rug_accent)
        _flat_shape('medallion_eye', _diamond(0, 0, 0.05, 0.05), 0.0175, 0.019, M.rug)
    G.join_all('rugRound')
    G.settle()
    return G.meshes()


def rugLarge():
    """A large plain rug: a bound border and a field with a line round it, a stepped diamond medallion and stepped
    corner pieces in the border's colour, and a knotted fringe across each short end."""
    sx, sz = 2.75, 1.85
    with G.new_parts():
        _rug_rect('rug', sx, sz, M.rug, M.rug_border, M.rug_border)
        _flat_shape('medallion', _diamond(0, 0, 0.42, 0.3, steps=5), 0.0155, 0.0175, M.rug_border)
        _flat_shape('medallion_core', _diamond(0, 0, 0.2, 0.14, steps=3), 0.0165, 0.019, M.rug_accent)
        _flat_shape('medallion_eye', _diamond(0, 0, 0.07, 0.05), 0.0175, 0.019, M.rug)
        for qx in (-1, 1):
            for qy in (-1, 1):
                cx, cy = qx * (sx / 2 - 0.25), qy * (sz / 2 - 0.25)
                corner = [(cx, cy), (cx - qx * 0.26, cy), (cx - qx * 0.26, cy - qy * 0.04), (cx - qx * 0.14, cy - qy * 0.04),
                          (cx - qx * 0.14, cy - qy * 0.1), (cx - qx * 0.04, cy - qy * 0.1), (cx - qx * 0.04, cy - qy * 0.18),
                          (cx, cy - qy * 0.18)]
                _flat_shape(f'corner{qx}{qy}', corner, 0.0155, 0.019, M.rug_border)
        for s in (-1, 1):
            _fringe(f'fringe{s}_', [-0.85 + 0.1 * i for i in range(18)], s * sx / 2, 0.05, M.rug_accent, side=s, along='x')
    G.join_all('rugLarge')
    G.settle()
    return G.meshes()


def rugRunner():
    """A hall runner: a bound border, a field with a line round it, three stepped medallions down its length with
    cream hearts, and a knotted fringe on each short end."""
    with G.new_parts():
        _rug_rect('rug', 0.8, 2.68, M.rug, M.rug_border, M.rug_border)
        for k, y in enumerate((-0.75, 0.0, 0.75)):
            _flat_shape(f'medallion{k}', _diamond(0, y, 0.16, 0.26, steps=4), 0.0155, 0.0175, M.rug_border)
            _flat_shape(f'heart{k}', _diamond(0, y, 0.06, 0.1), 0.0165, 0.019, M.rug_accent)
        for k, y in enumerate((-0.375, 0.375)):
            _flat_shape(f'link{k}', _diamond(0, y, 0.05, 0.05), 0.0155, 0.0175, M.rug_border)
        for s in (-1, 1):
            _fringe(f'fringe{s}_', [-0.32 + 0.08 * i for i in range(9)], s * 1.34, 0.06, M.rug_border, side=s)
    G.join_all('rugRunner')
    G.settle()
    return G.meshes()


def rugPattern():
    """A patterned rug: a navy border with a cream zigzag running round it, a terracotta ground, a large stepped
    medallion in three tiers with two smaller ones beside it, stepped corner pieces, and a knotted fringe on each
    short end."""
    sx, sz = 2.7, 1.86
    with G.new_parts():
        G.cube('border', (sx, sz, 0.012), at=(0, 0, 0.006), material=M.rug_edge, bevel=0.005, bevel_segments=3)
        G.cube('ground', (sx - 0.26, sz - 0.26, 0.006), at=(0, 0, 0.013), material=M.rug_ground, bevel=0.002)
        # the zigzag round the border: a band of teeth along each side
        for s in (-1, 1):
            for along, length, at_edge in (('x', sx - 0.12, sz / 2 - 0.065), ('y', sz - 0.12, sx / 2 - 0.065)):
                n = int(length / 0.08)
                pts = [(-length / 2 + length * i / n, 0.018 * (1 if i % 2 else -1)) for i in range(n + 1)]
                band = [(u, v + 0.008) for u, v in pts] + [(u, v - 0.008) for u, v in reversed(pts)]
                if along == 'x':
                    band = [(u, s * at_edge + v) for u, v in band]
                else:
                    band = [(s * at_edge + v, u) for u, v in band]
                _flat_shape(f'zigzag_{along}{s}', band, 0.0115, 0.0135, M.rug_accent)
            G.cube(f'line_x{s}', (sx - 0.46, 0.04, 0.004), at=(0, s * (sz - 0.5) / 2, 0.017), material=M.rug_accent, bevel=0.0015)
            G.cube(f'line_z{s}', (0.04, sz - 0.54, 0.004), at=(s * (sx - 0.5) / 2, 0, 0.017), material=M.rug_accent, bevel=0.0015)
        _flat_shape('medallion', _diamond(0, 0, 0.5, 0.42, steps=6), 0.0155, 0.0185, M.rug_motif)
        _flat_shape('medallion_core', _diamond(0, 0, 0.3, 0.25, steps=4), 0.0175, 0.0205, M.rug_accent)
        _flat_shape('medallion_eye', _diamond(0, 0, 0.12, 0.1, steps=2), 0.0195, 0.0225, M.rug_edge)
        _flat_shape('medallion_dot', _diamond(0, 0, 0.04, 0.035), 0.0215, 0.024, M.rug_motif)
        for s in (-1, 1):
            _flat_shape(f'side{s}', _diamond(s * 0.85, 0, 0.17, 0.17, steps=3), 0.0155, 0.0185, M.rug_motif)
            _flat_shape(f'side_core{s}', _diamond(s * 0.85, 0, 0.07, 0.07), 0.0175, 0.0205, M.rug_accent)
            for qy in (-1, 1):
                cx, cy = s * (sx / 2 - 0.27), qy * (sz / 2 - 0.27)
                corner = [(cx, cy), (cx - s * 0.24, cy), (cx - s * 0.24, cy - qy * 0.05), (cx - s * 0.12, cy - qy * 0.05),
                          (cx - s * 0.12, cy - qy * 0.12), (cx - s * 0.05, cy - qy * 0.12), (cx - s * 0.05, cy - qy * 0.2),
                          (cx, cy - qy * 0.2)]
                _flat_shape(f'corner{s}{qy}', corner, 0.0155, 0.0185, M.rug_motif)
            _fringe(f'fringe{s}_', [-0.85 + 0.1 * i for i in range(18)], s * sx / 2, 0.07, M.rug_accent, side=s, along='x')
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
     G.Model('fireplace', fireplace, size=(1.96, 2.8, 0.88), key='model.fireplace', centre=(0, 0.01),
             motions={'uses': {'relax': 'flames'}}, paint=[('surround', 'Surround'), ('oak', 'Mantel')]),
     # D3. Textiles
     G.Model('curtains', curtains, size=(1.288, 1.674, 0.104), key='model.curtains', centre=(0, -0.378),
             floor=False, motions={'night': 'close'}, paint=[('linen', 'Fabric'), ('rail', 'Rail')]),
     G.Model('curtainsWide', curtainsWide, size=(2.448, 1.674, 0.104), key='model.curtainsWide', centre=(0, -0.378),
             floor=False, motions={'night': 'close'}, paint=[('linen', 'Fabric'), ('rail', 'Rail')]),
     G.Model('curtainsLong', curtainsLong, size=(2.448, 2.584, 0.104), key='model.curtainsLong', centre=(0, -0.378),
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
