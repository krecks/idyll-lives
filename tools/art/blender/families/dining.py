# The kitchen's dining things. Notes: dining.md.
#
# K7, dining tables: the dining table with its two chairs (`model.diningTable`, base manifest), the bistro table
# and chair (`model.bistroSet`), the glass and farmhouse dining sets (`model.diningGlass`, `model.diningOak`), and
# the chair-less tables of catalog2: the square table for two, the round bistro table, the tables for four, six and
# eight, the round pedestal table, the square table for eight, the farmhouse trestle table and the high café table.
# K8, chairs and stools: the bar, counter and high bar stools, whose seat swivels 20 degrees and back as a resident
# sits (`swivel`, carrying them: `ride`), the dining chair, the wooden, upholstered, moulded, Windsor, bentwood,
# folding and carver chairs and the dining bench.
# K9, dining storage: the walnut sideboard, the painted buffet, the china cabinet and the bar cart, whose cocktail
# shaker is taken off it while a resident mixes a drink (`shake`: squashed away, the resident shakes held.py's
# `tool.shaker`, the same shaker, in both hands; review I-kitchen-13).
#
# Each is built in place where the game places it (`F`, game axes): table tops at the content's `surface`
# heights (`TT` 0.75, `BT` 1.06; 0.75-0.76 for the catalog sets), seats where the game seats residents
# (`characters.ts` SEATS, content `seat`: the seat's top at that height, its middle 5 cm forward for catalog2's
# chairs). The sets' chairs are part of the table, as the old parts drew them, since residents sit on them. The
# storage stands against the wall where the game places it (`centre=`, the old part's middle: its back about
# 5 cm behind the tile's wall face, where the content's `surface` is). The sideboard's, buffet's and china cabinet's
# doors are parts of their own, hinged; one of each (the china cabinet's left glazed door) swings open while a
# resident puts clean things away there (they are `cupboard` stations), the others are still.
#
# The detail is built from the forms (gamekit/forms.py): tops and seat boards lofted with a moulded edge (`_slab`,
# `_round_slab`), turned legs, plates and vases (`_turned`), bent tube and bentwood frames swept in one length
# (`_rail`, `G.kit.tube`), bowed back rails (`_crest`), arched aprons and shaped feet extruded (`_apron`, `F.extrude`),
# stuffed pads and cushions (`F.soft`), the moulded chair's shell as one curved panel (`_surface`).
#
# Run:  cargo art dining [--only barStool] [--preview]
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # table tops and legs (each its own material, so the player colours them apart)
    top_oak=('#CDB08A', 'wood'),          # white oak tops; the dining sets' chair seats
    legs_oak=('#CDB08A', 'wood'),
    legs_oakd=('#B08E66', 'wood'),        # darker oak legs and frames
    top_walnut=('#7A5234', 'wood'),
    legs_walnut=('#7A5234', 'wood'),
    top_banquet=('#8A5A36', 'wood'),
    legs_banquet=('#6B4A33', 'wood'),
    top_birch=('#E3CFA8', 'wood'),
    legs_birch=('#E3CFA8', 'wood'),
    top_farm=('#B48A5E', 'wood'),         # the farmhouse table's planks
    legs_farm=('#8E6A48', 'wood'),
    groove=('#7A5A3A', 'wood'),           # the joints between its planks
    top_white=('#F1F0EC', 'gloss'),       # the round bistro table
    top_white_satin=('#F1F0EC', 'satin'),  # the square table for eight
    top_black=('#2B2C2E', 'satin'),       # the high café table
    legs_black=('#2B2C2E', 'metal'),      # black metal frames
    legs_chrome=('#D0D3D6', 'chrome'),
    glass=('#DCE6E8', 'glass'),           # the glass table's top
    # the sets' chairs and the table settings
    seat_sage=('#9DAF8C', 'fabric'),      # seat pads
    chair_dark=('#3A3C40', 'metal'),      # the glass set's chair frames
    seat_grey=('#C9CBCC', 'satin'),       # and their seats
    runner=('#E8DCC0', 'fabric'),         # a table runner
    porcelain=('#F5F3EE', 'gloss'),       # plates, a cup
    food=('#E7D9BF', 'gloss'),
    tumbler=('#DCE6EA', 'gloss'),         # drinking glasses (gloss, as the old parts')
    cutlery=('#B9BDC1', 'satin'),
    vase=('#D8D2C4', 'gloss'),
    stem=('#7E9A62', 'satin'),
    bloom_red=('#D9483B', 'satin'),
    bloom_yellow=('#F2C14E', 'satin'),
    bloom_white=('#F0EDE6', 'satin'),
    # chairs and stools
    frame_cream=('#E4E0D6', 'satin'),     # the painted dining chair
    frame_honey=('#A8754A', 'wood'),      # the wooden chair
    seat_honey=('#A8754A', 'wood'),
    upholstery=('#B7A48A', 'fabric'),     # the upholstered chair
    shell=('#E8E4DC', 'gloss'),           # the moulded chair
    frame_windsor=('#6B4A33', 'wood'),
    seat_windsor=('#6B4A33', 'wood'),
    frame_bent=('#3B2A20', 'wood'),       # the bentwood chair
    seat_bent=('#8E2A2A', 'fabric'),
    frame_steel=('#C3C7CB', 'metal'),     # the folding chair
    seat_teal=('#4E7A7F', 'satin'),
    cap=('#2B2C2E', 'satin'),             # rubber feet
    frame_oak=('#CDB08A', 'wood'),        # the carver chair, the bar stool
    seat_oak=('#CDB08A', 'wood'),         # the bench's seat
    leather=('#C08A52', 'satin'),         # the high bar stool's seat
    # storage
    body_walnut=('#7A5234', 'wood'),      # the sideboard
    top_sideboard=('#7A5234', 'wood'),
    flute=('#5E3F28', 'wood'),            # the fluting on its doors
    brass=('#B8925A', 'metal'),
    cream=('#EFE5D3', 'gloss'),           # a vase
    body_sage=('#9DAF8C', 'satin'),       # the buffet
    body_china=('#F1EDE4', 'satin'),      # the china cabinet
    top_china=('#F1EDE4', 'satin'),
    back_blue=('#9DB7C9', 'satin'),       # its back, behind the glass
    plinth=('#B9A890', 'satin'),
    plate_blue=('#9DC6CF', 'gloss'),
    # the bar cart
    frame_brass=('#B8925A', 'metal'),
    tray=('#DCE6EA', 'glass'),
    castor=('#2B2C2E', 'satin'),
    bottle_green=('#2F5A3A', 'glass'),
    bottle_red=('#7A2E2E', 'glass'),
    bottle_amber=('#C9A64A', 'glass'),
    shaker=('#D0D3D6', 'chrome'),
    basket=('#C9A979', 'satin'),
    lemon=('#F2C14E', 'satin'),
)

F = G.GameFrame()
TT = 0.75     # a dining table's top (content surface)
BT = 1.06     # a bar table's top (the old `BT`)
SZ = 0.05     # catalog2's chairs: the seat's middle sits 5 cm forward, towards the table


def _v(p):
    """A game position or direction as a Blender vector."""
    return Vector(F.at(*p))


def _board(name, a, b, w, t, material, face=(0, 1, 0), bevel=0.006):
    """A board from game point `a` to `b`, `w` wide and `t` thick, its broad face turned towards game direction
    `face`: slats, rails and legs at any angle."""
    a, b = _v(a), _v(b)
    z = (b - a).normalized()
    x = _v(face).cross(z).normalized()
    y = z.cross(x)
    obj = G.cube(name, (w, t, (b - a).length), material=material, bevel=bevel)
    rot = Matrix((x, y, z)).transposed()
    obj.matrix_world = Matrix.Translation((a + b) / 2) @ rot.to_4x4()
    return obj


def _fillet(pts, bend, steps=3, closed=False):
    """Game points of a polyline whose corners are bent round, about `bend` in radius (a curve through each corner,
    `steps` segments, never past half way along a side): the path of a tube bent from one length."""
    P = [Vector(p) for p in pts]
    n = len(P)
    out = [] if closed else [P[0]]
    for i in range(n) if closed else range(1, n - 1):
        a, p, b = P[i - 1], P[i], P[(i + 1) % n]
        u, w = a - p, b - p
        lu, lw = u.length, w.length
        u.normalize()
        w.normalize()
        ang = u.angle(w)
        if ang > math.pi - 1e-3:
            out.append(p)
            continue
        t = min(bend / math.tan(ang / 2), 0.45 * lu, 0.45 * lw)
        t1, t2 = p + u * t, p + w * t
        for k in range(steps + 1):
            s = k / steps
            out.append(t1 * (1 - s) ** 2 + p * (2 * (1 - s) * s) + t2 * (s * s))
    if not closed:
        out.append(P[-1])
    return [tuple(v) for v in out]


def _rail(name, pts, r, material, bend=0.03, verts=8, steps=3, scale=1.0, closed=False):
    """A round tube of radius `r` through game points `pts`, bent round each corner (`_fillet`): one length of bent
    rod, rail or bentwood. Open ones are `G.kit.tube`; a closed loop is swept round on itself."""
    path = _fillet(pts, bend, steps, closed)
    if closed:
        return F.sweep(name, path, r, material, verts=verts, closed=True)
    return G.kit.tube(F, name, path, r, material, verts=verts, round=1, scale=scale)


def _loop(name, rx, rz, y, minor, material, x=0.0, z=0.0, verts=6, n=28):
    """A flat ring of tube in the floor's plane, `rx` by `rz` about (x, z) at height `y`: a foot ring, a rim."""
    pts = [(x + rx * math.cos(math.tau * i / n), y, z + rz * math.sin(math.tau * i / n)) for i in range(n)]
    return F.sweep(name, pts, minor, material, verts=verts, closed=True)


def _turned(name, profile, at, material, segments=12, angle=35):
    """A turned part (a leg, a finial, a column, a foot, a plate), `profile` [(radius, height), ...] from the
    bottom, standing at game point `at`; edges turning more than `angle` stay crisp."""
    obj = G.revolve(name, profile, at=F.at(*at), material=material, segments=segments)
    obj.data.set_sharp_from_angle(angle=math.radians(angle))
    return obj


def _slab(name, w, d, y_top, t, material, r=0.02, edge=None, corner=3, at=(0.0, 0.0)):
    """A table top or a seat board, `w` by `d` with corners of radius `r`, its top at `y_top`, `t` thick, its edge
    moulded: `edge` [(inset, fraction of the thickness up), ...] from the bottom (by default a rounded nose with
    a chamfer underneath). One piece, lofted round."""
    edge = edge or [(0.006, 0.0), (0.002, 0.3), (0.0, 0.62), (0.0015, 0.88), (0.0045, 1.0)]
    sections = []
    for inset, f in edge:
        y = y_top - t + f * t
        loop = G.rounded_rect(w - 2 * inset, d - 2 * inset, max(r - inset, 0.002), corner=corner)
        sections.append([F.at(at[0] + u, y, at[1] + v) for u, v in loop])
    return G.loft(name, sections, material=material, angle=50)


def _round_slab(name, rad, y_top, t, material, segments=40, edge=None, at=(0.0, 0.0)):
    """A round top `rad` across its radius, its top at `y_top`, `t` thick, moulded as `_slab`."""
    edge = edge or [(0.006, 0.0), (0.002, 0.3), (0.0, 0.62), (0.0015, 0.88), (0.0045, 1.0)]
    prof = [(0.0, 0.0)] + [(rad - i, f * t) for i, f in edge] + [(0.0, t)]
    return _turned(name, prof, (at[0], y_top - t, at[1]), material, segments=segments, angle=50)


def _sq_leg(name, x, z, y0, y1, s_foot, s_top, material, r=0.005, foot=0.0, at_top=None):
    """A square leg tapering from `s_foot` across at `y0` to `s_top` at `y1`, its corners rounded by `r`; `foot`:
    a small chamfer at the floor. `at_top` (x, z): where its top is, for a splayed leg."""
    xt, zt = at_top or (x, z)
    rings = []
    if foot:
        rings.append((s_foot - 2 * foot, y0, x, z))
        rings.append((s_foot, y0 + foot, x, z))
    else:
        rings.append((s_foot, y0, x, z))
    rings.append((s_top, y1, xt, zt))
    sections = []
    for s, y, cx, cz in rings:
        sections.append([F.at(cx + u, y, cz + v) for u, v in G.rounded_rect(s, s, min(r, s * 0.45), corner=1)])
    return G.loft(name, sections, material=material, angle=50)


def _apron(name, length, h, t, at, material, plane='front', arch=0.0, steps=10, bevel=0.003):
    """An apron rail `length` long and `h` high (its top at at[1]), `t` thick, its lower edge arched up by `arch` in
    the middle between the legs: along x (`plane` 'front') or along the depth ('side')."""
    half = length / 2
    lower = [(-half + length * i / steps, -h + arch * math.sin(math.pi * i / steps) ** 1.5) for i in range(steps + 1)]
    outline = [(half, 0.0), (-half, 0.0)] + lower if arch else [(half, 0.0), (-half, 0.0), (-half, -h), (half, -h)]
    return F.extrude(name, outline, t, at, material, plane=plane, bevel=bevel, bevel_segments=1)


def _surface(name, grid, t, material, angle=60):
    """A curved panel `t` thick through a grid of game points (rows of the same length): a moulded shell, a bent
    back, a slatted seat. Its faces are shaded smooth."""
    rows, cols = len(grid), len(grid[0])
    vs = [F.at(*p) for row in grid for p in row]
    faces = [(i * cols + j, i * cols + j + 1, (i + 1) * cols + j + 1, (i + 1) * cols + j)
             for i in range(rows - 1) for j in range(cols - 1)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vs, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    mod = obj.modifiers.new('thick', 'SOLIDIFY')
    mod.thickness = t
    mod.offset = 0.0
    mod.use_even_offset = True
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=mod.name)
    G.shapes._recalc_normals(obj)                  # the grid's winding may face either way: outwards
    obj.data.shade_smooth()
    obj.data.set_sharp_from_angle(angle=math.radians(angle))
    return obj


def _finish(name):
    """Joins the model into one, its feet cut flat on the floor (a leg swept up from it at a slant dips a few mm
    below: cut, it stands where it was built, so its seat stays at its height), and settles it there."""
    obj = G.join_all(name)
    for v in obj.data.vertices:
        v.co.z = max(v.co.z, 0.0)
    obj.data.update()
    G.settle(centre=False)
    return G.meshes()


# === K7: dining tables ===============================================================================================
# --- table settings, vases, a runner ---------------------------------------------------------------------------------
KNIFE = [(0, -0.0065), (0.088, -0.006), (0.094, -0.0085), (0.185, -0.0075), (0.2, 0.0), (0.192, 0.0085), (0.094, 0.009),
         (0.088, 0.0065), (0, 0.0065)]
FORK = [(0, -0.0065), (0.1, -0.0045), (0.13, -0.0125), (0.2, -0.0125), (0.2, -0.0082), (0.165, -0.0082),
        (0.165, -0.0058), (0.2, -0.0058), (0.2, -0.0012), (0.165, -0.0012), (0.165, 0.0012), (0.2, 0.0012),
        (0.2, 0.0058), (0.165, 0.0058), (0.165, 0.0082), (0.2, 0.0082), (0.2, 0.0125), (0.13, 0.0125), (0.1, 0.0045),
        (0, 0.0065)]


def _cutlery(name, outline, x, y, z, d):
    """A knife or a fork lying flat on a top at height `y`, its middle at (x, z), its tip towards `d` (±1 along x)."""
    pts = [(d * (u - 0.1), v) for u, v in outline]
    return F.extrude(name, pts, 0.003, (x, y + 0.0015, z), M.cutlery, plane='top')


def _place(prefix, x, z, y, s, napkin=None):
    """A place setting at (x, z) on a top at height `y`, for the diner at the `s` end (-1: the left one, facing
    +x): a rimmed plate with food on it, a knife at their right hand and a fork at their left (on a folded `napkin`),
    a glass beyond the knife."""
    _turned(f'{prefix}plate', [(0, 0), (0.07, 0), (0.075, 0.004), (0.106, 0.011), (0.12, 0.016), (0.118, 0.0185),
                               (0.104, 0.0138), (0.073, 0.0072), (0, 0.0068)], (x, y, z), M.porcelain, segments=18)
    _turned(f'{prefix}food', [(0, 0), (0.066, 0), (0.062, 0.009), (0.045, 0.019), (0.02, 0.025), (0, 0.026)],
            (x, y + 0.006, z), M.food, segments=12)
    right = -s                       # facing +x (s -1), the right hand is towards +z
    _turned(f'{prefix}glass', [(0, 0), (0.025, 0), (0.026, 0.004), (0.034, 0.1), (0.031, 0.1), (0.0235, 0.012),
                               (0, 0.012)], (x - s * 0.1, y, z + right * 0.17), M.tumbler, segments=12)
    _cutlery(f'{prefix}knife', KNIFE, x, y, z + right * 0.15, -s)
    fy = y
    if napkin:
        F.soft(f'{prefix}napkin', 0.15, 0.008, 0.09, (x - s * 0.005, y + 0.004, z - right * 0.155), napkin, round=0.003,
               round_segments=1)
        fy = y + 0.008
    _cutlery(f'{prefix}fork', FORK, x, fy, z - right * 0.155, -s)


def _flowers(prefix, x, z, y, h=0.18, n=5, leaves=True, segments=12):
    """A vase on a top at height `y`, with a lip and a foot, and `n` tulips (up to five) on curving stems, two
    leaves (`leaves`)."""
    _turned(f'{prefix}vase', [(0, 0), (0.032, 0), (0.036, 0.006), (0.034, 0.012), (0.05, 0.05), (0.046, 0.11),
                              (0.03, h - 0.035), (0.027, h - 0.012), (0.033, h - 0.002), (0.031, h), (0.024, h),
                              (0.022, h - 0.02), (0, h - 0.03)], (x, y, z), M.vase, segments=segments)
    blooms = ((-0.03, 0.1, 0.0, M.bloom_red), (0.035, 0.11, -0.02, M.bloom_yellow), (0.0, 0.09, 0.04, M.bloom_white),
              (0.02, 0.06, 0.03, M.bloom_red), (-0.04, 0.07, -0.03, M.bloom_white))
    for k, (dx, dh, dz, m) in enumerate(blooms[:n]):
        base, tip = (x, y + h - 0.04, z), (x + dx, y + h + dh - 0.02, z + dz)
        mid = (x + dx * 0.35, y + h + dh * 0.45, z + dz * 0.35)
        G.kit.tube(F, f'{prefix}stem{k}', [base, mid, tip], 0.0035, M.stem, verts=5, round=2)
        # a tulip: a cup of six petals, closed inside
        _turned(f'{prefix}bloom{k}', [(0, 0), (0.012, 0.004), (0.021, 0.016), (0.024, 0.03), (0.019, 0.042),
                                      (0.015, 0.038), (0, 0.03)], (tip[0], tip[1] - 0.002, tip[2]), m, segments=6)
    for k, (dx, dz) in enumerate(((0.05, 0.01), (-0.045, -0.015)) if leaves else ()):
        F.sweep(f'{prefix}leaf{k}', [(x + dx * 0.2, y + h - 0.03, z + dz * 0.2),
                                     (x + dx * 0.7, y + h + 0.03, z + dz * 0.7),
                                     (x + dx * 1.1, y + h + 0.02, z + dz * 1.1)], None, M.stem,
                profile=G.rounded_rect(0.002, 0.02, 0.001, corner=1), up=(0, 0, 1), resolution=3,
                scale=[0.4, 1.0, 0.15])


def _runner(name, w, y, z, d, material, drop=0.07, x=0.0):
    """A table runner `w` wide across a top `d` deep at height `y` (its top 4 mm up), hanging down over the front
    and back edges by `drop`."""
    e = d / 2 + 0.003
    path = _fillet([(x, y - drop, z - e), (x, y + 0.002, z - e), (x, y + 0.002, z + e), (x, y - drop, z + e)], 0.012,
                   steps=3)
    return F.sweep(name, path, None, material, profile=G.rounded_rect(0.004, w, 0.0015, corner=1), up=(0, 1, 0))


def _ladder_chair(prefix, frame, seat, pad, seat_h=0.46, back_h=0.92, metal=False):
    """A dining set's chair at the origin, its sitter facing +z: a seat board with a rounded edge under a piped pad,
    and a ladder back. Wood: square tapered front legs, back legs bending back into the posts, a bowed crest rail
    and two bowed ladder rails, arched aprons and turned stretchers. Metal (`metal`): the back legs and posts bent
    from one tube over the top, each front leg bent back into the seat's side rail, flat bars for the ladder."""
    _slab(f'{prefix}seat', 0.42, 0.4, seat_h - 0.02, 0.04, seat, r=0.025)
    F.soft(f'{prefix}pad', 0.38, 0.03, 0.36, (0, seat_h - 0.013, 0.01), pad, round=0.012, puff=0.008, round_segments=1,
           flat=2)

    def post_z(y):
        return -0.15 - 0.05 * y / back_h
    if metal:
        _rail(f'{prefix}back_frame', [(-0.18, 0, -0.15), (-0.18, back_h, -0.2), (0.18, back_h, -0.2), (0.18, 0, -0.15)],
              0.012, frame, bend=0.05, verts=8)
        for s in (-1, 1):
            _rail(f'{prefix}front{s}', [(s * 0.18, 0, 0.17), (s * 0.18, seat_h - 0.06, 0.165),
                                        (s * 0.18, seat_h - 0.06, post_z(seat_h))], 0.012, frame, bend=0.03, verts=8)
        G.kit.tube(F, f'{prefix}stretcher', [(-0.18, 0.16, 0.17), (0.18, 0.16, 0.17)], 0.009, frame, verts=8)
        for k, (y, h) in enumerate(((back_h - 0.25, 0.04), (back_h - 0.38, 0.04))):
            _crest(f'{prefix}rail{k + 1}', 0.18, y, post_z(y), h, 0.008, frame, bow=0.015, arch=1.0)
    else:
        for s in (-1, 1):
            _sq_leg(f'{prefix}front{s}', s * 0.18, 0.17, 0.0, seat_h - 0.06, 0.024, 0.034, frame, r=0.006, foot=0.003)
            F.sweep(f'{prefix}back_leg{s}', [(s * 0.18, 0, -0.15), (s * 0.18, 0.03, -0.15),
                                             (s * 0.18, 0.4, post_z(0.4) - 0.006),
                                             (s * 0.18, 0.66, post_z(0.66) - 0.004), (s * 0.18, back_h, -0.2)],
                    None, frame, profile=G.rounded_rect(0.034, 0.032, 0.008, corner=1), up=(-1, 0, 0), resolution=2,
                    scale=[0.92, 0.92, 1.0, 1.0, 0.9], angle=60)
            _stretcher(f'{prefix}side{s}', (s * 0.18, 0.16, 0.17), (s * 0.18, 0.16, -0.152), 0.009, frame, verts=6)
            _apron(f'{prefix}apron_s{s}', 0.3, 0.05, 0.018, (s * 0.175, seat_h - 0.06, 0.01), frame, plane='side',
                   arch=0.012, steps=6)
        _apron(f'{prefix}apron_f', 0.33, 0.05, 0.018, (0, seat_h - 0.06, 0.165), frame, arch=0.014, steps=6)
        _stretcher(f'{prefix}stretcher', (-0.18, 0.2, 0.17), (0.18, 0.2, 0.17), 0.009, frame, verts=6)
        for k, (y, h) in enumerate(((back_h - 0.25, 0.05), (back_h - 0.38, 0.05))):
            _crest(f'{prefix}rail{k + 1}', 0.172, y, post_z(y), h, 0.02, frame, bow=0.015, arch=1.0)
    y = back_h - 0.07
    _crest(f'{prefix}rail0', 0.19 if metal else 0.185, y, post_z(y) - (0.004 if metal else 0.0), 0.1, 0.022, seat,
           bow=0.015, arch=1.1)


SET_X = 0.62    # the sets for two: each chair's seat this far out from the middle, at its end of the top
SET_Z = 0.1     # and this far forward, with the top's middle (content `places` [±0.62, 0.1], facing ±90)


def _end_chair(prefix, x, z, frame, seat, pad, metal=False):
    """A set's chair at the end of its top, seat centred on (x, z), its sitter facing in across the top (+x from
    the left end, -x from the right): built facing the front, then turned a quarter about its seat's middle."""
    with G.new_parts() as parts:
        _ladder_chair(prefix, frame, seat, pad, metal=metal)
    bpy.context.view_layer.update()                     # the last part's matrix is set only on an update
    turn = Matrix.Translation(_v((x, 0, z))) @ Matrix.Rotation(math.radians(-90 if x < 0 else 90), 4, 'Z')
    for o in parts:
        o.matrix_world = turn @ o.matrix_world


def _turned_leg(h, r_top, r_foot, collar=None):
    """A turned table leg's profile `h` tall: a pad foot, a long taper from `r_foot` to `r_top`, a ring under the
    apron, the block that takes the apron; `collar`: a ring that high."""
    p = [(0, 0), (r_foot * 0.92, 0), (r_foot * 1.06, 0.008), (r_foot, 0.022), (r_foot * 0.9, 0.035)]
    if collar:
        c = collar
        rc = r_foot + (r_top - r_foot) * c / h
        p += [(rc * 0.95, c - 0.025), (rc * 1.28, c - 0.01), (rc * 1.3, c), (rc * 1.28, c + 0.01), (rc * 0.95, c + 0.025)]
    p += [(r_top * 0.92, h - 0.14), (r_top * 1.15, h - 0.125), (r_top * 1.15, h - 0.112), (r_top, h - 0.1), (r_top, h),
          (0, h)]
    return p


# --- the dining table (base manifest): an oak top to 0.76 on turned legs, a chair at each end facing in -----------
def diningTable():
    y, z = 0.76, SET_Z
    _slab('top', 0.84, 0.75, y, 0.04, M.top_oak, r=0.02, at=(0, z))
    for s in (-1, 1):
        _apron(f'apron{s}', 0.67, 0.07, 0.022, (0, y - 0.04, z + s * 0.304), M.legs_oakd, arch=0.018)
        _apron(f'apron_end{s}', 0.58, 0.07, 0.022, (s * 0.349, y - 0.04, z), M.legs_oakd, plane='side', arch=0.016)
    for x in (-0.36, 0.36):
        for dz in (-0.315, 0.315):
            _turned(f'leg{x}{dz}', _turned_leg(y - 0.04, 0.026, 0.018), (x, 0, z + dz), M.legs_oakd, segments=12)
    _runner('runner', 0.2, y, z, 0.75, M.runner)
    _flowers('', 0, z - 0.2, y + 0.004, h=0.13)
    for s in (-1, 1):
        _place(f'p{s}', s * 0.25, z, y, s, napkin=M.runner)
        _end_chair(f'c{s}', s * SET_X, z, M.legs_oakd, M.top_oak, M.seat_sage)
    return _finish('diningTable')


# --- the bistro set: a round oak café table and a black metal café chair behind it ----------------------------------
def _cafe_chair(prefix, x, z, frame, pad, seat_h=0.46):
    """A café chair facing +z, its seat centred on (x, z): a round, stuffed seat on a ring, four legs bowed a little
    out with a ring stretcher, back posts curving up into a looped back with a smaller loop inside it."""
    _loop(f'{prefix}seat_ring', 0.2, 0.2, seat_h - 0.045, 0.012, frame, x=x, z=z, verts=5, n=20)
    _turned(f'{prefix}seat', [(0, 0), (0.196, 0), (0.205, 0.008), (0.207, 0.026), (0.2, 0.037), (0.18, 0.043),
                              (0.12, 0.0447), (0, 0.045)], (x, seat_h - 0.045, z), pad, segments=20)
    for k, (sx, sz) in enumerate(((-1, 1), (1, 1), (-1, -1), (1, -1))):
        top = (x + sx * 0.13, seat_h - 0.045, z + sz * 0.13)
        foot = (x + sx * 0.17, 0.006, z + sz * 0.17 + (0.03 if sz < 0 else 0))
        mid = tuple((a + b) / 2 for a, b in zip(top, foot))
        mid = (mid[0] + sx * 0.012, mid[1], mid[2] + sz * 0.012)
        G.kit.tube(F, f'{prefix}leg{k}', [foot, mid, top], 0.011, frame, verts=6, round=2)
        _turned(f'{prefix}foot{k}', [(0, 0), (0.012, 0), (0.014, 0.006), (0.011, 0.012), (0, 0.012)], (foot[0], 0, foot[2]),
                frame, segments=6)
    _loop(f'{prefix}stretcher', 0.165, 0.165, 0.2, 0.008, frame, x=x, z=z + 0.01, verts=5, n=20)
    lean = math.radians(8)

    def hoop(name, rx, ry, minor, n):
        pts = [(x + rx * math.cos(math.tau * i / n), 0.75 + ry * math.sin(math.tau * i / n) * math.cos(lean),
                z - 0.21 - ry * math.sin(math.tau * i / n) * math.sin(lean)) for i in range(n)]
        return F.sweep(name, pts, minor, frame, verts=5, closed=True)
    hoop(f'{prefix}back', 0.16, 0.15, 0.012, 22)
    hoop(f'{prefix}back_in', 0.1, 0.09, 0.008, 16)
    for s in (-1, 1):
        G.kit.tube(F, f'{prefix}post{s}', [(x + s * 0.13, seat_h - 0.045, z - 0.13), (x + s * 0.15, 0.62, z - 0.175),
                                           (x + s * 0.159, 0.75, z - 0.21)], 0.011, frame, verts=8, round=3)
        # the loops tied together at the sides
        G.kit.tube(F, f'{prefix}tie{s}', [(x + s * 0.1, 0.75, z - 0.21), (x + s * 0.16, 0.75, z - 0.21)], 0.006, frame,
                   verts=5)


def bistroSet():
    y = 0.75
    z = 0.17
    _round_slab('top', 0.31, y, 0.035, M.top_oak, segments=32, at=(0, z),
                edge=[(0.004, 0.0), (0.0, 0.4), (0.0015, 0.85), (0.004, 1.0)])
    _turned('hub', [(0, 0), (0.06, 0), (0.08, 0.012), (0.08, 0.03), (0, 0.03)], (0, y - 0.065, z), M.legs_black,
            segments=16)
    _turned('column', [(0, 0), (0.03, 0), (0.025, 0.03), (0.024, y - 0.15), (0.03, y - 0.13), (0.03, y - 0.12),
                       (0.024, y - 0.11), (0.024, y - 0.08), (0, y - 0.08)], (0, 0.01, z), M.legs_black, segments=14)
    _turned('foot', [(0, 0), (0.205, 0), (0.21, 0.006), (0.205, 0.014), (0.16, 0.024), (0.09, 0.036), (0.05, 0.044),
                     (0.04, 0.056), (0, 0.056)], (0, 0, z), M.legs_black, segments=24)
    # a cup on its saucer and a little vase
    _turned('saucer', [(0, 0), (0.04, 0), (0.045, 0.003), (0.068, 0.008), (0.07, 0.011), (0.066, 0.011),
                       (0.045, 0.007), (0, 0.007)], (0.12, y, 0.24), M.porcelain, segments=14)
    _turned('cup', [(0, 0), (0.026, 0), (0.028, 0.004), (0.033, 0.03), (0.043, 0.07), (0.04, 0.07), (0.03, 0.03),
                    (0.024, 0.012), (0, 0.012)], (0.12, y + 0.007, 0.24), M.porcelain, segments=12)
    _rail('cup_handle', [(0.152, y + 0.065, 0.24), (0.172, y + 0.06, 0.24), (0.172, y + 0.03, 0.24),
                         (0.148, y + 0.025, 0.24)], 0.005, M.porcelain, bend=0.012, verts=5)
    _flowers('', -0.15, 0.1, y, h=0.12, n=3, leaves=False, segments=10)
    _cafe_chair('c', 0, -0.19, M.legs_black, M.seat_sage)        # its back on the wall face, as the old part's
    return _finish('bistroSet')


# --- the glass dining set: a glass top on two chrome sleds along its sides, a dark chair at each end ----------------
def diningGlass():
    y, z = 0.76, SET_Z
    _slab('top', 0.9, 0.8, y, 0.02, M.glass, r=0.03, edge=[(0.003, 0.0), (0.0, 0.3), (0.0, 0.7), (0.003, 1.0)],
          at=(0, z))
    for dz in (-0.3, 0.3):                          # the sleds stand along the sides, the ends open for the knees
        _rail(f'sled{dz}', [(-0.38, 0.018, z + dz), (0.38, 0.018, z + dz), (0.38, y - 0.032, z + dz),
                            (-0.38, y - 0.032, z + dz)], 0.018, M.legs_chrome, bend=0.06, verts=12, closed=True)
        for x in (-0.3, 0.3):
            _turned(f'pad{x}{dz}', [(0, 0), (0.016, 0), (0.02, 0.006), (0.02, 0.012), (0, 0.012)],
                    (x, y - 0.032, z + dz), M.legs_chrome, segments=10)
        for x in (-0.3, 0.3):                       # glides under the runners
            F.box(f'glide{x}{dz}', 0.05, 0.004, 0.03, (x, 0.002, z + dz), M.legs_chrome, bevel=0.0015)
    G.kit.tube(F, 'rail', [(0, y - 0.05, z - 0.3), (0, y - 0.05, z + 0.3)], 0.012, M.legs_chrome, verts=10)
    _flowers('', 0, z - 0.22, y, h=0.18)
    for s in (-1, 1):
        _place(f'p{s}', s * 0.25, z, y, s)
        _end_chair(f'c{s}', s * SET_X, z, M.chair_dark, M.seat_grey, M.seat_grey, metal=True)
    return _finish('diningGlass')


# --- the farmhouse dining set: a thick planked oak top with pegged breadboard ends on square legs, a runner ---------
def diningOak():
    y, z = 0.76, SET_Z
    edge = [(0.003, 0.0), (0.0, 0.25), (0.0, 0.8), (0.002, 0.95), (0.004, 1.0)]
    for k in range(5):                              # five planks between the breadboard ends, rounded where they meet
        _slab(f'plank{k}', 0.78, 0.16, y, 0.05, M.top_oak, r=0.004, edge=edge, at=(0, z - 0.32 + 0.16 * k))
    for s in (-1, 1):
        _slab(f'cleat{s}', 0.06, 0.8, y, 0.05, M.top_oak, r=0.012, at=(s * 0.42, z))   # breadboard ends
        for k in (-1, 0, 1):
            F.cyl(f'peg{s}{k}', 0.007, 0.01, (s * 0.42, y - 0.0045, z + k * 0.28), M.legs_oakd, verts=8)
        _apron(f'apron{s}', 0.66, 0.08, 0.025, (0, y - 0.05, z + s * 0.32), M.legs_oakd, arch=0.018)
        _apron(f'apron_end{s}', 0.6, 0.08, 0.025, (s * 0.34, y - 0.05, z), M.legs_oakd, plane='side', arch=0.018)
    for x in (-0.37, 0.37):
        for dz in (-0.33, 0.33):
            _sq_leg(f'leg{x}{dz}', x, z + dz, 0.0, y - 0.05, 0.06, 0.08, M.legs_oakd, r=0.008, foot=0.004)
    _runner('runner', 0.24, y, z, 0.8, M.runner)
    _flowers('', 0, z - 0.2, y + 0.004, h=0.18)
    for s in (-1, 1):
        _place(f'p{s}', s * 0.27, z, y, s, napkin=M.runner)
        _end_chair(f'c{s}', s * SET_X, z, M.legs_oakd, M.top_oak, M.seat_sage)
    return _finish('diningOak')


# --- the catalog2 tables: a top at 0.75 on four (or six) legs, aprons under it -------------------------------------
def _rect_table(w, d, top, legs, t=0.045, leg=0.06, inset=0.06, turned=False, taper=0.7, apron=True, h=TT, arch=0.02,
                collar=None, r=0.02, end_arch=None):
    """A top `w` by `d`, `t` thick with a moulded edge, on four legs (six past 2.6 m), square tapered or `turned`
    (with a `collar`), joined by aprons arched by `arch` between them."""
    _slab('top', w, d, h, t, top, r=r)
    lx, lz = w / 2 - inset - leg / 2, d / 2 - inset - leg / 2
    if apron:
        for s in (-1, 1):
            _apron(f'apron{s}', 2 * lx - leg * 0.8, 0.08, 0.022, (0, h - t, s * (lz - leg / 2 + 0.011)), legs,
                   arch=arch if 2 * lx < 2.0 else 0.0)
            _apron(f'apron_end{s}', 2 * lz - leg * 0.8, 0.08, 0.022, (s * (lx - leg / 2 + 0.011), h - t, 0), legs,
                   plane='side', arch=arch if end_arch is None else end_arch)
    xs = (-lx, lx) + ((0.0,) if w > 2.6 else ())
    for x in xs:
        for z in (-lz, lz):
            if turned:
                _turned(f'leg{x}{z}', _turned_leg(h - t, leg / 2, leg / 2 * taper, collar), (x, 0, z), legs, segments=12)
            else:
                _sq_leg(f'leg{x}{z}', x, z, 0.0, h - t, leg * taper, leg, legs, r=0.006, foot=0.003)
    if apron and w > 2.6:                             # the middle legs' aprons across
        _apron('apron_mid', 2 * lz - leg * 0.8, 0.08, 0.022, (0, h - t, 0), legs, plane='side', arch=arch)
    return lx, lz


def tableSquare2():
    """Square table for two (catalog2): an oak top with a moulded edge on turned darker legs with collars, arched
    aprons."""
    _rect_table(0.82, 0.82, M.top_oak, M.legs_oakd, turned=True, taper=0.75, collar=0.52)
    return _finish('tableSquare2')


def tableDining4():
    """Dining table for four (catalog2): an oak top on square tapered oak legs, arched aprons, a drawer at each end
    with a turned knob."""
    lx, lz = _rect_table(1.84, 0.88, M.top_oak, M.legs_oak, leg=0.065, taper=0.65, end_arch=0.0)
    for s in (-1, 1):
        x = s * (lx - 0.0325 + 0.022 + 0.004)
        F.extrude(f'drawer{s}', G.rounded_rect(0.4, 0.055, 0.004, corner=1), 0.008, (x, TT - 0.045 - 0.04, 0), M.legs_oak,
                  plane='side', bevel=0.002, bevel_segments=1)
        knob = G.revolve(f'knob{s}', [(0, 0), (0.008, 0), (0.007, 0.01), (0.014, 0.018), (0.012, 0.026), (0, 0.027)],
                         at=F.at(x + s * 0.004, TT - 0.085, 0), material=M.legs_oak, segments=10, rot=(0, -90 * s, 0))
        knob.data.set_sharp_from_angle(angle=math.radians(35))
    return _finish('tableDining4')


def tableDining6():
    """Dining table for six (catalog2): a walnut top on six turned, tapered legs with collars, arched aprons."""
    _rect_table(2.84, 0.88, M.top_walnut, M.legs_walnut, leg=0.07, turned=True, taper=0.7, collar=0.5)
    return _finish('tableDining6')


def tableBanquet8():
    """Banquet table (catalog2): a long top of four planks on six chunky chamfered legs with feet, aprons and an
    H-stretcher pegged through the legs."""
    edge = [(0.004, 0.0), (0.0, 0.25), (0.0, 0.8), (0.0025, 0.95), (0.005, 1.0)]
    for k in range(5):                              # five planks: the middle of the top is on one, not a joint
        _slab(f'plank{k}', 3.84, 0.176, TT, 0.045, M.top_banquet, r=0.006, edge=edge, at=(0, -0.352 + 0.176 * k))
    lx, lz = 3.84 / 2 - 0.06 - 0.04, 0.88 / 2 - 0.06 - 0.04
    for s in (-1, 1):
        _apron(f'apron{s}', 2 * lx, 0.08, 0.022, (0, TT - 0.045, s * (lz - 0.029)), M.legs_banquet)
        _apron(f'apron_end{s}', 2 * lz - 0.06, 0.08, 0.022, (s * (lx - 0.029), TT - 0.045, 0), M.legs_banquet,
               plane='side', arch=0.015)
    for x in (-lx, 0.0, lx):
        for z in (-lz, lz):
            _sq_leg(f'leg{x}{z}', x, z, 0.03, TT - 0.045, 0.066, 0.08, M.legs_banquet, r=0.01)
            _sq_leg(f'foot{x}{z}', x, z, 0.0, 0.03, 0.075, 0.082, M.legs_banquet, r=0.006, foot=0.004)
        _apron(f'cross{x}', 2 * lz, 0.05, 0.04, (x, 0.205, 0), M.legs_banquet, plane='side', arch=0.0)
    F.box('stretcher', 2 * lx, 0.05, 0.04, (0, 0.18, 0), M.legs_banquet, bevel=0.008, bevel_segments=2)
    for x in (-lx, lx):
        s = 1 if x > 0 else -1
        F.box(f'tenon{x}', 0.03, 0.04, 0.03, (x + s * 0.05, 0.18, 0), M.legs_banquet, bevel=0.005)
        F.box(f'wedge{x}', 0.014, 0.06, 0.012, (x + s * 0.058, 0.18, 0), M.legs_banquet, bevel=0.003)
    return _finish('tableBanquet8')


def tableSquare8():
    """Square table for eight (catalog2): a white top with a softened edge on black square-tube legs near its
    corners, a steel frame under the top and a square frame bracing the legs low down, on levelling feet."""
    _slab('top', 1.86, 1.86, TT, 0.03, M.top_white_satin, r=0.015,
          edge=[(0.003, 0.0), (0.0, 0.3), (0.0, 0.7), (0.003, 1.0)])
    for y, h, t in ((TT - 0.06, 0.06, 0.02), (0.1, 0.03, 0.03)):
        frame = G.rounded_rect(1.7, 1.7, 0.03, corner=2)
        F.sweep(f'frame{y}', [(u, y, v) for u, v in frame], None, M.legs_black,
                profile=G.rounded_rect(h, t, 0.004, corner=1), closed=True, angle=60)
    for x in (-0.85, 0.85):
        for z in (-0.85, 0.85):
            _sq_leg(f'leg{x}{z}', x, z, 0.012, TT - 0.03, 0.06, 0.06, M.legs_black, r=0.006)
            _turned(f'foot{x}{z}', [(0, 0), (0.026, 0), (0.028, 0.004), (0.026, 0.007), (0.008, 0.008), (0.008, 0.014),
                                    (0, 0.014)], (x, 0, z), M.legs_black, segments=12)
    return _finish('tableSquare8')


def tableFarmhouse6():
    """Farmhouse table (catalog2): a thick top of three planks on two trestles, each a chamfered post on a foot
    shaped underneath and under a shaped bearer, braced, joined by a stretcher wedged through the posts."""
    edge = [(0.004, 0.0), (0.0, 0.25), (0.0, 0.8), (0.0025, 0.95), (0.005, 1.0)]
    for k in range(3):
        _slab(f'plank{k}', 2.86, 0.2925, TT, 0.06, M.top_farm, r=0.008, edge=edge, at=(0, (k - 1) * 0.2935))
    for x in (-1.15, 1.15):
        _sq_leg(f'post{x}', x, 0, 0.07, TT - 0.12, 0.1, 0.1, M.legs_farm, r=0.012)
        # the foot: arched up underneath between its pads, chamfered at its ends
        foot = [(-0.4, 0.0), (-0.28, 0.0), (-0.26, 0.012)] + \
            [(-0.24 + 0.48 * i / 8, 0.012 + 0.012 * math.sin(math.pi * i / 8)) for i in range(9)] + \
            [(0.26, 0.0), (0.28, 0.0), (0.4, 0.0), (0.4, 0.035), (0.33, 0.07), (-0.33, 0.07), (-0.4, 0.035)]
        F.extrude(f'foot{x}', foot, 0.1, (x, 0, 0), M.legs_farm, plane='side', bevel=0.006, bevel_segments=1)
        bearer = [(-0.39, 0.06), (0.39, 0.06), (0.39, 0.03), (0.3, 0.0), (-0.3, 0.0), (-0.39, 0.03)]
        F.extrude(f'cleat{x}', bearer, 0.1, (x, TT - 0.12, 0), M.legs_farm, plane='side', bevel=0.006,
                  bevel_segments=1)
        for s in (-1, 1):
            _board(f'brace{x}{s}', (x, 0.07, s * 0.3), (x, 0.32, s * 0.05), 0.05, 0.06, M.legs_farm, face=(1, 0, 0),
                   bevel=0.008)
            _board(f'brace_top{x}{s}', (x, TT - 0.12, s * 0.3), (x, 0.5, s * 0.05), 0.04, 0.05, M.legs_farm,
                   face=(1, 0, 0), bevel=0.008)
    F.box('stretcher', 2.5, 0.08, 0.06, (0, 0.32, 0), M.legs_farm, bevel=0.012, bevel_segments=2)
    for x in (-1.225, 1.225):
        F.box(f'wedge{x}', 0.025, 0.1, 0.03, (x, 0.32, 0), M.groove, bevel=0.005)
    return _finish('tableFarmhouse6')


def _pedestal(h, top_r, top, base, hub_r, col_r, foot_r, foot_h=0.05, t=0.04, verts=40):
    """A round top on a column: a hub under the top, the column with a ring near its top, a domed cast foot."""
    _round_slab('top', top_r, h, t, top, segments=verts)
    _turned('hub', [(0, 0), (hub_r * 0.6, 0), (hub_r, 0.014), (hub_r, 0.03), (0, 0.03)], (0, h - t - 0.03, 0), base,
            segments=18)
    ch = h - t - 0.03 - foot_h + 0.01
    _turned('column', [(0, 0), (col_r * 1.4, 0), (col_r, 0.04), (col_r, ch - 0.08), (col_r * 1.35, ch - 0.06),
                       (col_r * 1.35, ch - 0.045), (col_r, ch - 0.03), (col_r, ch), (0, ch)], (0, foot_h - 0.01, 0), base,
            segments=14)
    _turned('foot', [(0, 0), (foot_r - 0.004, 0), (foot_r, 0.005), (foot_r - 0.003, 0.013), (foot_r * 0.6, foot_h * 0.5),
                     (col_r * 2.2, foot_h * 0.9), (col_r * 1.6, foot_h), (0, foot_h)], (0, 0, 0), base, segments=32)


def tableBistroRound():
    """Round bistro table (catalog2): a white gloss top with a black edge band, on a black column and cast foot."""
    _pedestal(TT, 0.355, M.top_white, M.legs_black, 0.1, 0.03, 0.25)
    _turned('edge', [(0.35, 0), (0.358, 0.0), (0.36, 0.004), (0.36, 0.026), (0.358, 0.03), (0.35, 0.03)],
            (0, TT - 0.034, 0), M.legs_black, segments=48)
    return _finish('tableBistroRound')


def tableBarHigh():
    """High café table (catalog2): a dark top at bar height on a column, a foot ring on four spokes for the stools."""
    _pedestal(BT, 0.35, M.top_black, M.legs_black, 0.1, 0.035, 0.25)
    _loop('foot_ring', 0.185, 0.185, 0.3, 0.013, M.legs_black, verts=8, n=32)
    _turned('ring_hub', [(0, 0), (0.05, 0), (0.05, 0.03), (0, 0.03)], (0, 0.285, 0), M.legs_black, segments=14)
    for k in range(4):
        a = 45 + 90 * k
        c, s = G.cosd(a), G.sind(a)
        G.kit.tube(F, f'spoke{k}', [(c * 0.05, 0.3, s * 0.05), (c * 0.18, 0.3, s * 0.18)], 0.009, M.legs_black, verts=8)
    return _finish('tableBarHigh')


def tableRound4():
    """Round pedestal table (catalog2, 2x2): a birch top with a moulded edge on a turned column, four feet curving
    out from it to turned toes."""
    _round_slab('top', 0.76, TT, 0.04, M.top_birch, segments=64)
    _turned('apron', [(0, 0), (0.28, 0), (0.3, 0.01), (0.3, 0.05), (0.285, 0.06), (0, 0.06)], (0, TT - 0.1, 0),
            M.legs_birch, segments=32)
    _turned('column', [(0, 0.08), (0.13, 0.08), (0.13, 0.12), (0.1, 0.16), (0.075, 0.3), (0.09, 0.42), (0.08, 0.5),
                       (0.065, 0.58), (0.072, 0.6), (0.065, 0.62), (0.09, 0.64), (0.09, 0.67), (0, 0.67)], (0, 0, 0),
            M.legs_birch, segments=24)
    for k in range(4):
        a = 45 + 90 * k
        c, s = G.cosd(a), G.sind(a)
        F.sweep(f'foot{k}', [(c * 0.06, 0.1, s * 0.06), (c * 0.17, 0.085, s * 0.17), (c * 0.28, 0.05, s * 0.28),
                             (c * 0.36, 0.03, s * 0.36)], None, M.legs_birch,
                profile=G.rounded_rect(0.07, 0.06, 0.015, corner=2), up=(0, 0, 1), resolution=3,
                scale=[1.1, 1.0, 0.82, 0.7], angle=60)
        _turned(f'toe{k}', [(0, 0), (0.026, 0), (0.03, 0.008), (0.026, 0.02), (0.02, 0.03), (0, 0.03)],
                (c * 0.36, 0, s * 0.36), M.legs_birch, segments=12)
    return _finish('tableRound4')


# === K8: chairs and stools ===========================================================================================
def _stool(seat_h, z, frame, seat, legs_r=0.22, back=False, cushion_h=0.04, verts=24, cap=None, metal=False):
    """A stool on four splayed legs in shoes and a foot ring, its seat (and back) a part of its own turning on a hub:
    a seat board with a rounded edge under a stuffed cushion with a piped seam, its top at `seat_h`. `cap`: the
    feet and the swivel plate (the frame's own material on a wood stool, so it draws in one group)."""
    cap = cap or frame
    with G.new_parts() as base:
        for k in range(4):
            a = 45 + 90 * k
            c, s = G.cosd(a), G.sind(a)
            foot, top = (c * legs_r, 0.008, z + s * legs_r), (c * 0.12, seat_h - 0.06, z + s * 0.12)
            if metal:
                G.kit.tube(F, f'leg{k}', [foot, top], 0.016, frame, verts=10)
            else:                                    # turned: a long taper with a ring at the foot rest and a neck
                pts = [tuple(f + (t - f) * w for f, t in zip(foot, top)) for w in (0, 0.25, 0.33, 0.38, 0.43, 0.9, 1)]
                G.kit.tube(F, f'leg{k}', pts, 0.017, frame, verts=10, round=2,
                           scale=[0.78, 0.86, 0.92, 1.18, 0.92, 1.05, 0.95])
            _turned(f'shoe{k}', [(0, 0), (0.017, 0), (0.019, 0.004), (0.018, 0.014), (0.015, 0.018), (0, 0.018)],
                    (c * legs_r, 0.0, z + s * legs_r), cap, segments=10)
        ring_y = seat_h * 0.36                       # the foot rest: where the renderer's `rest` puts the feet
        k_r = legs_r - (legs_r - 0.12) * ring_y / (seat_h - 0.06)
        _loop('foot_ring', k_r, k_r, ring_y, 0.012, frame, z=z, verts=8, n=32)
        if metal:                                    # a kick plate on the ring's front, where the shoes rest
            F.box('kick', 0.12, 0.004, 0.03, (0, ring_y + 0.012, z + k_r - 0.006), frame, bevel=0.0015)
        _turned('hub', [(0, 0), (0.135, 0), (0.15, 0.006), (0.148, 0.016), (0.11, 0.02), (0, 0.02)],
                (0, seat_h - 0.075, z), frame, segments=24)
    G.join(base, 'base', origin=(0, 0, 0))
    with G.new_parts() as top:
        _turned('swivel', [(0, 0), (0.07, 0), (0.07, 0.006), (0.05, 0.012), (0, 0.012)], (0, seat_h - 0.056, z), cap,
                segments=16)
        y0 = seat_h - cushion_h - 0.025
        _turned('seat_board', [(0, 0), (0.18, 0), (0.192, 0.005), (0.197, 0.017), (0.194, 0.029), (0.186, 0.035),
                               (0, 0.035)], (0, y0, z), frame, segments=verts)
        ch = cushion_h
        _turned('cushion', [(0, 0), (0.176, 0), (0.185, 0.004), (0.188, ch * 0.45), (0.185, ch - 0.01),
                            (0.175, ch - 0.004), (0.14, ch - 0.0012), (0.07, ch - 0.0002), (0.03, ch - 0.0015),
                            (0.012, ch - 0.005), (0, ch - 0.0055)], (0, seat_h - ch, z), seat, segments=verts)
        _turned('button', [(0, 0), (0.011, 0), (0.01, 0.003), (0.005, 0.0045), (0, 0.0048)], (0, seat_h - 0.005, z),
                seat, segments=8)
        _loop('piping', 0.181, 0.181, seat_h - 0.007, 0.004, seat, z=z, verts=5, n=verts)
        if back:
            # the back's frame bent from one tube up behind the seat, and a curved, padded back board on it
            for s in (-1, 1):
                _rail(f'back_post{s}', [(s * 0.12, seat_h - 0.04, z - 0.135), (s * 0.14, seat_h + 0.1, z - 0.17),
                                        (s * 0.14, seat_h + 0.27, z - 0.19)], 0.012, frame, bend=0.08, verts=8)
            tilt = math.radians(8)
            F.sweep('back', [(-0.18, seat_h + 0.24, z - 0.152), (0, seat_h + 0.24, z - 0.178),
                             (0.18, seat_h + 0.24, z - 0.152)], None, seat,
                    profile=G.rounded_rect(0.14, 0.03, 0.013, corner=2), up=(0, math.sin(tilt), math.cos(tilt)),
                    resolution=5, angle=60)
    seat_part = G.hinged(G.join(top, 'seat'), F.at(0, seat_h - 0.056, z))
    G.settle(centre=False)
    G.swing(seat_part, 'swivel', 20, seconds=0.5)
    return G.meshes()


def barStool():
    """Bar stool (catalog): an oak stool with a sage seat, top at 0.74 (sitters at 0.72), centred."""
    return _stool(0.74, 0.0, M.frame_oak, M.seat_sage, legs_r=0.21, cushion_h=0.05)


def stoolCounter():
    """Counter stool (catalog2): an oak stool, seat top at 0.685, 5 cm forward."""
    return _stool(0.685, SZ, M.frame_oak, M.seat_sage, legs_r=0.21)


def stoolBarHigh():
    """High bar stool (catalog2): black metal with a cognac seat and a low back, seat top at 0.805."""
    return _stool(0.805, SZ, M.legs_black, M.leather, legs_r=0.21, back=True, cushion_h=0.05, cap=M.cap, metal=True)


def _stretcher(name, a, b, r, material, swell=1.3, verts=8):
    """A turned stretcher from game point `a` to `b`, swelling to `swell` times `r` in the middle."""
    m = tuple((p + q) / 2 for p, q in zip(a, b))
    return G.kit.tube(F, name, [a, m, b], r, material, verts=verts, round=3, scale=[0.8, swell, 0.8])


def _crest(name, x, y, z, h, t, material, bow=0.025, arch=1.12, up=(0, 0, 1), resolution=3):
    """A back rail from -x to x at height `y`, bowed back by `bow` in the middle (round the sitter's back), `h` high
    and `t` thick, taller by `arch` in the middle."""
    return F.sweep(name, [(-x, y, z), (0, y, z - bow), (x, y, z)], None, material,
                   profile=G.rounded_rect(h, t, min(t * 0.45, 0.01), corner=1), up=up, resolution=resolution,
                   scale=[1.0, arch, 1.0], angle=60)


def _bowed(x, xm, bow):
    """How far back a rail bowed by `bow` (its ends at ±xm) is at x."""
    return bow * (1 - (x / xm) ** 2)


def diningChair():
    """Dining chair (catalog): a painted ladder-back chair on turned legs with a tie-on piped pad, seat top 0.46:
    turned front legs and back posts with finials, three bowed slats, arched aprons, turned stretchers."""
    z = 0.02
    _slab('seat', 0.44, 0.42, 0.46, 0.04, M.frame_cream, r=0.03, at=(0, z))
    F.soft('pad', 0.38, 0.025, 0.36, (0, 0.4665, z + 0.01), M.seat_sage, round=0.012, puff=0.006, piping=0.0035)
    leg = [(0, 0), (0.013, 0), (0.0145, 0.014), (0.0145, 0.11), (0.019, 0.125), (0.019, 0.14), (0.0145, 0.155),
           (0.016, 0.27), (0.021, 0.29), (0.021, 0.305), (0.017, 0.32), (0.019, 0.42), (0, 0.42)]
    for x in (-0.19, 0.19):
        _turned(f'front{x}', leg, (x, 0, 0.19), M.frame_cream, segments=10)
        G.kit.tube(F, f'back{x}', [(x, 0, -0.17), (x, 0.42, -0.178), (x * 0.97, 0.7, -0.193), (x * 0.95, 0.955, -0.21)],
                   0.017, M.frame_cream, verts=10, round=3, scale=[0.9, 1.0, 0.95, 0.88])
        _turned(f'finial{x}', [(0, 0), (0.015, 0), (0.019, 0.01), (0.012, 0.018), (0.021, 0.03), (0.017, 0.041),
                               (0.006, 0.046), (0, 0.047)], (x * 0.95, 0.955, -0.21), M.frame_cream, segments=10)
        # the pad's ties, knotted round the back post, their tails hanging
        _loop(f'tie{x}', 0.021, 0.021, 0.468, 0.004, M.seat_sage, x=x, z=-0.172, verts=4, n=10)
        for k, dx in enumerate((-0.012, 0.01)):
            F.sweep(f'tail{x}{k}', [(x + dx, 0.468, -0.195), (x + dx * 1.4, 0.44, -0.2), (x + dx * 1.8, 0.405, -0.196)],
                    None, M.seat_sage, profile=G.rounded_rect(0.012, 0.003, 0.0012, corner=1), up=(0, 1, 0),
                    resolution=2)
    # three slats between the posts, bowed back round the sitter, the top one widest
    for k, (y, h) in enumerate(((0.885, 0.075), (0.75, 0.062), (0.615, 0.055))):
        zz = -0.17 - 0.04 * y / 0.96
        _crest(f'slat{k}', 0.178, y, zz, h, 0.016, M.frame_cream, bow=0.022, arch=1.12 if k == 0 else 1.0)
    # aprons under the seat, arched between the legs; turned stretchers, the sides higher than the front and back
    _apron('apron_f', 0.36, 0.05, 0.018, (0, 0.42, 0.185), M.frame_cream, arch=0.016)
    _apron('apron_b', 0.36, 0.05, 0.018, (0, 0.42, -0.165), M.frame_cream)
    for x in (-0.185, 0.185):
        _apron(f'apron_s{x}', 0.34, 0.05, 0.018, (x, 0.42, 0.01), M.frame_cream, plane='side', arch=0.012)
        _stretcher(f'side{x}', (x, 0.24, 0.19), (x, 0.24, -0.172), 0.009, M.frame_cream)
    for zz in (-0.172, 0.19):
        _stretcher(f'stretcher{zz}', (-0.19, 0.16, zz), (0.19, 0.16, zz), 0.009, M.frame_cream)
    return _finish('diningChair')


def chairWood():
    """Wooden chair (catalog2): honey wood, a spindle back between two bowed rails, seat top 0.47: square tapered
    front legs, back legs bending back into the posts, swelled spindles, arched aprons."""
    x, zf, zb = 0.19, 0.23, -0.095
    _slab('seat', 0.44, 0.42, 0.47, 0.04, M.seat_honey, r=0.025, at=(0, SZ))
    for s in (-1, 1):
        _sq_leg(f'front{s}', s * x, zf, 0.0, 0.43, 0.026, 0.036, M.frame_honey, r=0.006, foot=0.003)
        F.sweep(f'back{s}', [(s * x, 0, zb), (s * x, 0.03, zb), (s * x, 0.43, zb - 0.025), (s * x, 0.66, zb - 0.045),
                             (s * x, 0.892, zb - 0.074)],
                None, M.frame_honey, profile=G.rounded_rect(0.034, 0.03, 0.008, corner=1), up=(-1, 0, 0), resolution=3,
                scale=[0.92, 0.92, 1.0, 1.0, 0.92], angle=60)
        _turned(f'cap{s}', [(0, 0), (0.018, 0), (0.018, 0.006), (0.012, 0.012), (0, 0.013)], (s * x, 0.889, zb - 0.074),
                M.frame_honey, segments=8)
    # the rails, bowed back; five swelled spindles between them, following the bow
    bow = 0.012
    z_top, z_low = zb - 0.068, zb - 0.04
    _crest('rail0', x - 0.008, 0.835, z_top, 0.1, 0.026, M.frame_honey, bow=bow, arch=1.15)
    _crest('rail1', x - 0.008, 0.6, z_low, 0.05, 0.026, M.frame_honey, bow=bow)
    for k, sx in enumerate((-0.1, -0.05, 0.0, 0.05, 0.1)):
        a = (sx, 0.615, z_low - _bowed(sx, x, bow))
        b = (sx, 0.79, z_top - _bowed(sx, x, bow))
        _stretcher(f'spindle{k}', a, b, 0.008, M.frame_honey, swell=1.45)
    # aprons under the seat, arched; turned side and cross stretchers
    _apron('apron_f', 2 * x - 0.03, 0.06, 0.02, (0, 0.45, zf - 0.005), M.frame_honey, arch=0.02)
    _apron('apron_b', 2 * x - 0.03, 0.06, 0.02, (0, 0.45, zb - 0.02), M.frame_honey)
    for s in (-1, 1):
        _apron(f'apron_s{s}', zf - zb - 0.03, 0.06, 0.02, (s * (x - 0.004), 0.45, (zf + zb) / 2), M.frame_honey,
               plane='side', arch=0.015)
        _stretcher(f'side{s}', (s * x, 0.24, zf), (s * x, 0.24, zb - 0.01), 0.011, M.frame_honey)
        _stretcher(f'cross{s}', (-x, 0.18, zf if s > 0 else zb - 0.008), (x, 0.18, zf if s > 0 else zb - 0.008), 0.011,
                   M.frame_honey)
    return _finish('chairWood')


def chairUpholstered():
    """Upholstered chair (catalog2): a stuffed, piped seat and back in oatmeal fabric on walnut legs, seat at 0.48:
    tapered front legs, sabre back legs, the back buttoned."""
    F.soft('seat', 0.46, 0.085, 0.44, (0, 0.4275, SZ + 0.01), M.upholstery, round=0.03, puff=0.01, piping=0.005)
    back = F.soft('back', 0.44, 0.5, 0.075, (0, 0.74, SZ - 0.19), M.upholstery, rot=(-7, 0, 0), round=0.03,
                  puff=0.012, piping=0.005)
    # three buttons in a row across the back's front, pulled in a little
    bpy.context.view_layer.update()
    for k, bx in enumerate((-0.11, 0.0, 0.11)):
        p = back.matrix_world @ Vector((-bx, -0.045, 0.06))
        G.sphere(f'button{k}', 0.011, at=p, material=M.upholstery, scale=(1, 0.5, 1), segments=8, rings=5)
    for x in (-0.19, 0.19):
        G.kit.tube(F, f'front{x}', [(x, 0.0, SZ + 0.17), (x, 0.4, SZ + 0.165)], 0.02, M.legs_walnut, verts=10,
                   scale=[0.6, 1.0])
        G.kit.tube(F, f'back_leg{x}', [(x, 0.0, SZ - 0.252), (x, 0.2, SZ - 0.212), (x, 0.4, SZ - 0.172)], 0.017,
                   M.legs_walnut, verts=10, round=3, scale=[0.72, 0.92, 1.05])
    return _finish('chairUpholstered')


def chairModern():
    """Moulded chair (catalog2): a gloss shell in one piece, its seat rolling over at the front and curving up into
    a back that wraps round the sitter, on a black sled frame of two bent tubes."""
    # the shell's side section (height, depth, half width, how far its sides wrap forward), front lip to top of back
    keys = [(0.434, SZ + 0.225, 0.2, 0.0), (0.458, SZ + 0.209, 0.214, 0.0), (0.464, SZ + 0.15, 0.218, 0.0),
            (0.461, SZ + 0.02, 0.215, 0.0), (0.466, SZ - 0.11, 0.21, 0.012), (0.5, SZ - 0.175, 0.205, 0.035),
            (0.58, SZ - 0.2, 0.208, 0.055), (0.69, SZ - 0.228, 0.214, 0.065), (0.787, SZ - 0.262, 0.208, 0.06)]
    rows, _ = G.forms._catmull([Vector(k) for k in keys], 2, False)
    grid = []
    for y, zz, hw, wrap in rows:
        grid.append([(u * hw, y, zz + wrap * u * u) for u in (i / 6 - 1 for i in range(13))])
    _surface('shell', grid, 0.012, M.shell)
    # the sleds: each side one tube bent round, a runner on the floor, the legs and a rail under the seat; a cross
    # rail between them and two mounting pads
    for s in (-1, 1):
        x = s * 0.19
        _rail(f'sled{s}', [(x, 0.011, SZ - 0.17), (x, 0.011, SZ + 0.17), (x, 0.444, SZ + 0.155), (x, 0.444, SZ - 0.13)],
              0.011, M.legs_black, bend=0.035, verts=8, closed=True)
    G.kit.tube(F, 'cross', [(-0.19, 0.444, SZ), (0.19, 0.444, SZ)], 0.009, M.legs_black, verts=8)
    return _finish('chairModern')


def chairWindsor():
    """Windsor chair (catalog2): a carved saddle seat on splayed baluster-turned legs and a turned H-stretcher, a
    bow bent from one tapering length over seven spindles."""
    # the saddle: dished, the rim highest at the back and sides, a pommel at the front
    grid = []
    n = 12
    for i in range(n + 1):
        v = i / n * 2 - 1
        row = []
        for j in range(n + 1):
            u = j / n * 2 - 1
            px, pz = u * math.sqrt(max(0.0, 1 - v * v / 2)), v * math.sqrt(max(0.0, 1 - u * u / 2))
            r2 = px * px + pz * pz
            pommel = 0.004 * math.exp(-(px / 0.15) ** 2) * max(0.0, pz)          # never above the rim
            dish = max(0.0, 0.012 * (1 - r2) * (1 - 0.6 * max(0.0, pz) ** 2) - pommel)
            row.append((px * 0.235, 0.4475 - dish, SZ + 0.002 + pz * 0.225 - 0.02 * (1 - px * px) * max(0.0, -pz)))
        grid.append(row)
    _surface('seat', grid, 0.045, M.seat_windsor, angle=50)
    turn = [0.8, 1.0, 1.3, 0.85, 0.95, 0.75]
    for sx in (-1, 1):
        for sz in (-1, 1):
            foot, top = (sx * 0.22, 0.0, SZ + sz * 0.21), (sx * 0.15, 0.43, SZ + sz * 0.13)
            pts = [tuple(f + (t - f) * w for f, t in zip(foot, top)) for w in (0, 0.2, 0.42, 0.62, 0.82, 1)]
            pts[0] = (foot[0], 0.0, foot[2])
            G.kit.tube(F, f'leg{sx}{sz}', pts, 0.018, M.frame_windsor, verts=10, round=2, scale=turn)
    # the H-stretcher, where the splayed legs pass 16 cm up
    y = 0.16
    lx, lz = 0.22 - 0.07 * y / 0.43, 0.21 - 0.08 * y / 0.43
    for sx in (-1, 1):
        _stretcher(f'side{sx}', (sx * lx, y, SZ + lz), (sx * lx, y, SZ - lz), 0.011, M.frame_windsor, swell=1.4)
    _stretcher('cross', (-lx, y, SZ), (lx, y, SZ), 0.011, M.frame_windsor, swell=1.4)
    # the bow: a half ellipse leaning back, 0.21 across and up to 0.97, bent from one length tapering to its top
    y0, ry, z0, back, rx = 0.455, 0.5, SZ - 0.13, 0.08, 0.175
    pts = []
    for i in range(17):
        t = math.pi * i / 16
        yy = y0 + ry * math.sin(t)
        pts.append((rx * math.cos(t), yy, z0 - back * (yy - y0) / ry))
    pts = [(pts[0][0], 0.43, z0 + 0.005)] + pts + [(pts[-1][0], 0.43, z0 + 0.005)]
    G.kit.tube(F, 'bow', pts, 0.016, M.frame_windsor, verts=8, round=2,
               scale=[1.05] + [1.0 - 0.18 * math.sin(math.pi * i / 16) for i in range(17)] + [1.05])
    for k, sx in enumerate((-0.12, -0.08, -0.04, 0.0, 0.04, 0.08, 0.12)):
        yy = y0 + ry * math.sqrt(max(0.0, 1 - (sx / rx) ** 2))
        G.kit.tube(F, f'spindle{k}', [(sx * 0.9, 0.455, SZ - 0.115), (sx, yy - 0.005, z0 - back * (yy - y0) / ry)],
                   0.008, M.frame_windsor, verts=6, scale=[1.15, 0.75])
    return _finish('chairWindsor')


def chairBentwood():
    """Bentwood chair (catalog2): dark steam-bent beech, a round seat ring with a red cane insert, the back legs and
    the back's loop bent from one length, a smaller loop inside it, a leg ring and knee braces."""
    _turned('seat', [(0.0, 0.0), (0.205, 0.0), (0.216, 0.01), (0.217, 0.022), (0.21, 0.033), (0.197, 0.035),
                     (0.176, 0.035), (0.172, 0.03), (0.172, 0.02), (0.0, 0.02)], (0, 0.44, SZ), M.frame_bent,
            segments=28)
    _turned('cane', [(0, 0), (0.174, 0), (0.174, 0.03), (0.16, 0.036), (0.08, 0.0395), (0, 0.04)], (0, 0.44, SZ),
            M.seat_bent, segments=24)
    _loop('seat_rim', 0.203, 0.203, 0.432, 0.011, M.frame_bent, z=SZ, verts=6, n=28)
    _loop('leg_ring', 0.166, 0.166, 0.2, 0.011, M.frame_bent, z=SZ, verts=6, n=24)
    for sx in (-1, 1):
        G.kit.tube(F, f'front{sx}', [(sx * 0.18, 0.0, SZ + 0.2), (sx * 0.155, 0.22, SZ + 0.162),
                                     (sx * 0.14, 0.432, SZ + 0.13)], 0.016, M.frame_bent, verts=8, round=2,
                   scale=[0.78, 0.95, 1.05])
        # a knee brace from the leg up under the seat rim
        G.kit.tube(F, f'brace{sx}', [(sx * 0.15, 0.33, SZ + 0.145), (sx * 0.12, 0.4, SZ + 0.1),
                                     (sx * 0.06, 0.425, SZ + 0.13)], 0.007, M.frame_bent, verts=6, round=3)
    # the back legs and the loop of the back in one length: up from the floor past the seat, round the top, down
    y0, ry, z0, back, rx = 0.44, 0.51, SZ - 0.17, 0.05, 0.155
    arc = []
    for i in range(1, 18):
        t = math.pi * i / 18
        yy = y0 + ry * math.sin(t)
        arc.append((rx * math.cos(t), yy, z0 - back * (yy - y0) / ry))
    pts = [(0.17, 0.0, SZ - 0.2), (0.16, 0.22, SZ - 0.183), (0.155, 0.44, z0)] + arc + \
        [(-0.155, 0.44, z0), (-0.16, 0.22, SZ - 0.183), (-0.17, 0.0, SZ - 0.2)]
    taper = [0.8, 0.95, 1.05] + [1.0 - 0.12 * math.sin(math.pi * i / 18) for i in range(1, 18)] + [1.05, 0.95, 0.8]
    G.kit.tube(F, 'back', pts, 0.015, M.frame_bent, verts=8, round=2, scale=taper)
    # the inner loop, leaning with the back, tied to it each side
    lean = math.radians(6)
    inner = [(0.085 * math.cos(math.tau * i / 20), 0.71 + 0.11 * math.sin(math.tau * i / 20) * math.cos(lean),
              SZ - 0.195 - 0.11 * math.sin(math.tau * i / 20) * math.sin(lean)) for i in range(20)]
    F.sweep('back_in', inner, 0.01, M.frame_bent, verts=6, closed=True)
    for s in (-1, 1):
        G.kit.tube(F, f'tie{s}', [(s * 0.084, 0.71, SZ - 0.195), (s * 0.154, 0.7, SZ - 0.196)], 0.007, M.frame_bent,
                   verts=6)
    return _finish('chairBentwood')


def chairFolding():
    """Folding chair (catalog2): a steel frame of two bent tubes crossing at rivets, a padded teal seat on a pan and
    a curved back panel, rubber feet."""
    xf, xr = 0.215, 0.2
    # the front legs rise into the back posts and bend across the top: one tube
    _rail('frame_front', [(-xf, 0.012, SZ + 0.22), (-xf, 0.88, SZ - 0.2), (xf, 0.88, SZ - 0.2), (xf, 0.012, SZ + 0.22)],
          0.011, M.frame_steel, bend=0.05, verts=8)
    # the rear legs rise under the seat's front and bend across it: the other tube
    _rail('frame_rear', [(-xr, 0.012, SZ - 0.2), (-xr, 0.43, SZ + 0.13), (xr, 0.43, SZ + 0.13), (xr, 0.012, SZ - 0.2)],
          0.011, M.frame_steel, bend=0.04, verts=8)
    G.kit.tube(F, 'stretcher', [(-xf, 0.1, SZ + 0.15), (xf, 0.1, SZ + 0.15)], 0.009, M.frame_steel, verts=8)
    G.kit.tube(F, 'stretcher_rear', [(-xr, 0.1, SZ - 0.15), (xr, 0.1, SZ - 0.15)], 0.009, M.frame_steel, verts=8)
    for s in (-1, 1):
        F.cyl(f'pivot{s}', 0.012, 0.012, (s * (xf + 0.011), 0.29, SZ - 0.02), M.frame_steel, rot=(0, 90, 0), verts=10,
              bevel=0.003, bevel_segments=1)
        # the seat's hinge brackets at the back, on the back posts
        F.box(f'bracket{s}', 0.006, 0.05, 0.04, (s * (xf - 0.013), 0.43, SZ - 0.13), M.frame_steel, bevel=0.002)
        # rubber feet on the four tube ends
        for zz, xx in ((SZ + 0.22, s * xf), (SZ - 0.2, s * xr)):
            F.box(f'foot{s}{zz}', 0.03, 0.022, 0.055, (xx, 0.011, zz), M.cap, bevel=0.008, bevel_segments=2)
    F.box('pan', 0.41, 0.012, 0.37, (0, 0.437, SZ + 0.02), M.frame_steel, bevel=0.005, bevel_segments=1)
    F.soft('seat', 0.42, 0.03, 0.38, (0, 0.445, SZ + 0.02), M.seat_teal, round=0.012, puff=0.005)
    tilt = math.radians(25)
    F.sweep('back', [(-0.205, 0.77, SZ - 0.165), (0, 0.77, SZ - 0.18), (0.205, 0.77, SZ - 0.165)], None, M.seat_teal,
            profile=G.rounded_rect(0.2, 0.022, 0.01, corner=2), up=(0, math.sin(tilt), math.cos(tilt)), resolution=5,
            angle=60)
    return _finish('chairFolding')


def chairArm():
    """Carver chair (catalog2): an oak armchair, padded and piped sage seat and back, the cushion's top at 0.475 (sat
    on at 0.47, a dining chair's height, so it goes at the head of the same table): arms that sweep forward and
    roll down into their posts, a bowed crest rail, back legs bending back into the posts."""
    x, zf, zb = 0.24, 0.23, -0.105
    _slab('seat', 0.54, 0.44, 0.425, 0.05, M.frame_oak, r=0.025, at=(0, SZ))
    F.soft('cushion', 0.46, 0.06, 0.4, (0, 0.437, SZ + 0.01), M.seat_sage, round=0.022, puff=0.008, piping=0.005)
    F.soft('back_pad', 0.42, 0.3, 0.05, (0, 0.70, SZ - 0.17), M.seat_sage, rot=(-6, 0, 0), round=0.02, puff=0.01)
    for s in (-1, 1):
        _sq_leg(f'front{s}', s * x, zf, 0.0, 0.4, 0.03, 0.042, M.frame_oak, r=0.007, foot=0.003)
        F.sweep(f'back{s}', [(s * x, 0, zb), (s * x, 0.03, zb), (s * x, 0.4, zb - 0.025), (s * x, 0.66, zb - 0.045),
                             (s * x, 0.918, zb - 0.07)],
                None, M.frame_oak, profile=G.rounded_rect(0.04, 0.04, 0.009, corner=1), up=(-1, 0, 0), resolution=3,
                scale=[0.92, 0.92, 1.0, 1.0, 0.95], angle=60)
        # the arm: from the back post forward, rolling down over its front into the post, one shaped piece
        ax = s * 0.255
        F.sweep(f'arm{s}', [(ax, 0.655, zb - 0.03), (ax, 0.662, 0.05), (ax, 0.665, SZ + 0.15), (ax, 0.64, SZ + 0.2),
                            (ax, 0.56, SZ + 0.192), (ax, 0.42, SZ + 0.18)],
                None, M.frame_oak, profile=G.rounded_rect(0.032, 0.058, 0.012, corner=2), up=(0, 0, 1),
                resolution=3, scale=[0.9, 1.0, 1.1, 1.05, 0.8, 0.85], angle=60)
        _stretcher(f'stretcher{s}', (s * x, 0.18, zf), (s * x, 0.18, zb - 0.01), 0.012, M.frame_oak)
        _apron(f'apron_s{s}', zf - zb - 0.03, 0.05, 0.02, (s * (x - 0.004), 0.4, (zf + zb) / 2), M.frame_oak,
               plane='side', arch=0.012)
    _crest('top_rail', x + 0.01, 0.88, zb - 0.064, 0.075, 0.035, M.frame_oak, bow=0.012, arch=1.12)
    _apron('apron_f', 2 * x - 0.03, 0.05, 0.02, (0, 0.4, zf - 0.005), M.frame_oak, arch=0.016)
    _stretcher('stretcher_front', (-x, 0.2, zf), (x, 0.2, zf), 0.012, M.frame_oak)
    return _finish('chairArm')


def benchDining():
    """Dining bench (catalog2, 2x1, two sitters): an oak plank with a rounded edge on splayed darker legs, through a
    shaped cleat at each end, an arched rail under it and pegged stretchers."""
    for k, dz in enumerate((-0.12, 0.0, 0.12)):                  # three planks, a rounded groove between each
        _slab(f'seat{k}', 1.6, 0.12, 0.465, 0.05, M.seat_oak, r=0.01, at=(0, SZ + dz),
              edge=[(0.004, 0.0), (0.0, 0.25), (0.0, 0.8), (0.0025, 0.95), (0.005, 1.0)])
    _apron('rail', 1.36, 0.06, 0.03, (0, 0.415, SZ), M.legs_oakd, arch=0.022, steps=14)
    for x in (-0.68, 0.68):
        s = 1 if x > 0 else -1
        F.extrude(f'cleat{x}', [(-0.15, 0.0175), (0.15, 0.0175), (0.15, 0.0), (0.12, -0.0175), (-0.12, -0.0175),
                                (-0.15, 0.0)], 0.07, (x, 0.3975, SZ), M.legs_oakd, plane='side', bevel=0.004,
                  bevel_segments=1)
        for dz in (-0.12, 0.12):
            _sq_leg(f'leg{x}{dz}', x + s * 0.05, SZ + dz * 1.25, 0.0, 0.415, 0.04, 0.046, M.legs_oakd, r=0.008,
                    foot=0.003, at_top=(x, SZ + dz))
        G.kit.tube(F, f'stretcher{x}', [(x + s * 0.025, 0.15, SZ - 0.142), (x + s * 0.025, 0.15, SZ + 0.142)], 0.014,
                   M.legs_oakd, verts=8)
        for dz in (-0.1, 0.1):
            F.cyl(f'peg{x}{dz}', 0.006, 0.05, (x + s * 0.025, 0.15, SZ + dz * 1.42), M.seat_oak, rot=(0, 90, 0), verts=6)
    return _finish('benchDining')


# === K9: dining storage ===============================================================================================
WALL = -0.43   # where the storage is built from (game depth); `settle()` then moves it to the old part's centre
LEG_H = 0.16   # a sideboard's legs
DOOR_SECONDS = 0.9   # a storage door swings open in this long, while clean things are put away (a `cupboard`)
PUT_AWAY = {'uses': {'fetch': 'door_open'}, 'hold': 1.5}


def _cabinet(w, h, d, body, top, legs, top_t=0.03, top_over=0.01, hollow=None, ferrule=None):
    """A cabinet on four turned tapered legs (in brass shoes: `ferrule`), its back on the wall face, a top with a
    moulded edge over it, a bead along the carcass's foot; returns the front's depth. `hollow`: (x0, x1, y0, y1,
    depth), a cupboard open at the front behind a door that opens."""
    z = WALL + d / 2
    yb, yt = LEG_H, h - top_t
    if hollow is None:
        F.box('carcass', w, yt - yb, d, (0, (yb + yt) / 2, z), body, bevel=0.01)
    else:
        # solid either side (their sides one piece), the cupboard between: a back, a floor and a roof
        x0, x1, y0, y1, depth = hollow
        for name, a, b in (('carcass_l', -w / 2, x0), ('carcass_r', x1, w / 2)):
            F.box(name, b - a, yt - yb, d, ((a + b) / 2, (yb + yt) / 2, z), body, bevel=0.01)
        F.box('carcass_back', x1 - x0, yt - yb, d - depth, ((x0 + x1) / 2, (yb + yt) / 2, WALL + (d - depth) / 2), body)
        front = WALL + d - depth / 2
        F.box('carcass_floor', x1 - x0, y0 - yb, depth, ((x0 + x1) / 2, (yb + y0) / 2, front), body, bevel=0.003)
        F.box('carcass_roof', x1 - x0, yt - y1, depth, ((x0 + x1) / 2, (y1 + yt) / 2, front), body, bevel=0.003)
    _slab('top', w + 2 * top_over, d + top_over, h, top_t, top, r=0.012, at=(0, z + top_over / 2))
    # a bead along the front's foot and round its sides
    F.sweep('bead', [(-w / 2 - 0.004, yb + 0.006, WALL + 0.02), (-w / 2 - 0.004, yb + 0.006, WALL + d + 0.004),
                     (w / 2 + 0.004, yb + 0.006, WALL + d + 0.004), (w / 2 + 0.004, yb + 0.006, WALL + 0.02)], None, body,
            profile=G.rounded_rect(0.014, 0.012, 0.005, corner=1), up=(0, 0, 1), angle=50)
    leg = [(0, 0), (0.0135, 0), (0.0145, 0.03), (0.019, LEG_H - 0.03), (0.022, LEG_H - 0.022), (0.022, LEG_H), (0, LEG_H)]
    for x in (-w / 2 + 0.08, w / 2 - 0.08):
        for zz in (WALL + 0.06, WALL + d - 0.06):
            _turned(f'leg{x}{zz}', leg, (x, 0, zz), legs, segments=12)
            if ferrule:
                _turned(f'shoe{x}{zz}', [(0, 0), (0.0138, 0), (0.0148, 0.022), (0.0146, 0.03), (0, 0.03)], (x, 0, zz),
                        ferrule, segments=12)
    return WALL + d


def _door(name, x0, x1, y0, y1, zf, material, hinge_left, build=None, frame=0.0):
    """A door from x0 to x1 (game), `y0` to `y1`, its back on the front face `zf`, its edges rounded: a part of its own,
    hinged on its outer edge (one of them swings open while clean things are put away: `door_open`; the others
    are still). `frame`: only a frame that wide (a glazed door), its inner edge moulded."""
    with G.new_parts() as parts:
        xm, ym, w, h = (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0
        outline = G.rounded_rect(w, h, 0.006, corner=1)
        if frame:
            hole = G.rounded_rect(w - 2 * frame, h - 2 * frame, 0.004, corner=1)
            F.extrude(name, outline, 0.02, (xm, ym, zf + 0.01), material, holes=[hole], bevel=0.004, bevel_segments=1)
        else:
            F.extrude(name, outline, 0.02, (xm, ym, zf + 0.01), material, bevel=0.005, bevel_segments=1)
        if build:
            build(x0, x1, y0, y1, zf + 0.02)
    door = G.join(parts, name)
    return G.hinged(door, F.at(x0 if hinge_left else x1, (y0 + y1) / 2, zf + 0.01))


def _knob(name, x, y, zf, material, r=0.016, l=None):
    """A mushroom knob on a face at depth `zf`, `l` long: a neck flaring into a domed head."""
    k = (l or r * 1.85) / 1.85
    return G.revolve(name, [(0, 0), (r * 0.6, 0), (r * 0.42, k * 0.5), (r * 0.5, k * 0.9), (r, k * 1.25), (r * 0.9, k * 1.6),
                            (r * 0.5, k * 1.8), (0, k * 1.85)], at=F.at(x, y, zf), material=material, segments=10,
                     rot=(90, 0, 0))


def _plate(name, x, y, z, r, material, segments=16, rot=None):
    """A plate lying on a top at (x, y, z), or standing (`rot`, Blender degrees): a well and a rim."""
    prof = [(0, 0), (r * 0.6, 0), (r * 0.64, r * 0.04), (r * 0.9, r * 0.1), (r, r * 0.14), (r * 0.98, r * 0.16),
            (r * 0.88, r * 0.12), (r * 0.62, r * 0.065), (0, r * 0.06)]
    obj = G.revolve(name, prof, at=F.at(x, y, z), material=material, segments=segments, rot=rot or (0, 0, 0))
    obj.data.set_sharp_from_angle(angle=math.radians(35))
    return obj


def _crockery(cup, zf, shelf):
    """A shelf half way up a cupboard (`_cabinet`'s `hollow`) and stacks of rimmed plates and bowls on its floor
    and on the shelf; `zf` is the cabinet's front."""
    x0, x1, y0, y1, depth = cup
    xm, z = (x0 + x1) / 2, zf - depth / 2
    ym = (y0 + y1) / 2
    F.box('cup_shelf', x1 - x0, 0.018, depth - 0.02, (xm, ym, z - 0.01), shelf, bevel=0.003)
    for k, base in enumerate((y0, ym + 0.009)):
        # a stack of plates, only the top one's well and rim showing; a stack of bowls, the top one open
        for i in range(5):
            F.cyl(f'cup_plate{k}{i}', 0.1, 0.012, (xm - 0.06, base + 0.006 + 0.014 * i, z),
                  M.porcelain if i % 3 else M.plate_blue, verts=14)
        _plate(f'cup_plate{k}5', xm - 0.06, base + 0.07, z, 0.1, M.porcelain, segments=14)
        for i in range(2):
            F.cone(f'cup_bowl{k}{i}', 0.04, 0.066, 0.045, (xm + 0.11, base + 0.0225 + 0.02 * i, z + 0.04), M.porcelain,
                   verts=12)
        _turned(f'cup_bowl{k}2', [(0, 0), (0.04, 0), (0.06, 0.03), (0.066, 0.045), (0.06, 0.045), (0.054, 0.03),
                                  (0.036, 0.008), (0, 0.008)], (xm + 0.11, base + 0.04, z + 0.04), M.porcelain,
                segments=12)


def sideboard():
    """Walnut sideboard (catalog2, 2x1): four fluted doors with brass knobs on turned legs in brass shoes, a top with
    a moulded edge; a vase and a bowl of lemons on top."""
    w, h, d = 1.8, 0.8, 0.62
    n, y0, y1 = 4, LEG_H + 0.02, h - 0.05
    dw = (w - 0.04) / n
    # the second door's cupboard (the door from x0 to x1), with plates and bowls in it
    cx0 = -w / 2 + 0.02 + dw + 0.006
    cup = (cx0 + 0.02, cx0 + dw - 0.032, y0 + 0.02, y1 - 0.02, 0.45)
    with G.new_parts() as still:
        zf = _cabinet(w, h, d, M.body_walnut, M.top_sideboard, M.body_walnut, hollow=cup, ferrule=M.brass)
        _crockery(cup, zf, M.body_walnut)
        _turned('vase', [(0, 0), (0.05, 0), (0.052, 0.01), (0.068, 0.07), (0.066, 0.12), (0.048, 0.2), (0.033, 0.26),
                         (0.031, 0.28), (0.04, 0.295), (0.038, 0.3), (0.03, 0.3), (0.026, 0.28), (0, 0.27)],
                (-0.6, h, -0.3), M.cream, segments=16)
        _turned('bowl', [(0, 0), (0.05, 0), (0.055, 0.008), (0.1, 0.045), (0.11, 0.06), (0.104, 0.062), (0.092, 0.05),
                         (0.045, 0.016), (0, 0.016)], (0.45, h, -0.2), M.cream, segments=18)
        for i, (dx, dz, dy) in enumerate(((-0.04, 0.0, 0.0), (0.035, -0.02, 0.0), (0.0, 0.04, 0.0), (0.0, 0.0, 0.035))):
            F.sphere(f'lemon{i}', 0.03, (0.45 + dx, h + 0.045 + dy, -0.2 + dz), M.lemon, scale=(1, 1.25, 1),
                     rot=(0, 25 * i, 40 * i), segments=10, rings=7)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []

    def flutes(x0, x1, y0, y1, zf):
        # half-round reeds standing proud of the door, stopped short of its top and bottom
        half = [(0.006 * math.sin(math.pi * i / 6), 0.006 * math.cos(math.pi * i / 6)) for i in range(7)]
        for k in range(-3, 4):
            x = (x0 + x1) / 2 + k * dw / 8
            F.sweep(f'flute{x0}{k}', [(x, y0 + 0.035, zf - 0.001), (x, y1 - 0.035, zf - 0.001)], None, M.flute,
                    profile=half, up=(0, -1, 0), angle=60)

    for i in range(n):
        x0 = -w / 2 + 0.02 + dw * i + 0.006
        x1 = x0 + dw - 0.012
        left = i % 2 == 0

        def knob(a, b, c, e, zf, left=left):
            _knob(f'knob{a}', (b - 0.05) if left else (a + 0.05), (c + e) / 2, zf, M.brass, r=0.014, l=0.018)
            flutes(a, b, c, e, zf)
        doors.append(_door(f'door{i}', x0, x1, y0, y1, zf, M.body_walnut, left, knob))
    G.settle()                       # where the game places it, against the wall (its Model's centre)
    # the second door (hinged on its right) swings open while clean things are put away; one door only, as
    # each costs draws of its own
    G.swing(doors[1], 'door_open', -100, seconds=DOOR_SECONDS)
    return G.meshes()


def _pull(name, x, y, zf, w, material):
    """A bar pull `w` long across a face at depth `zf`: a rounded bar on two posts, bent from one piece."""
    return _rail(name, [(x - w / 2, y, zf), (x - w / 2, y, zf + 0.022), (x + w / 2, y, zf + 0.022), (x + w / 2, y, zf)],
                 0.005, material, bend=0.008, verts=6)


def buffet():
    """Painted buffet (catalog2, 2x1): a sage cabinet, three drawers with brass bar pulls over five panelled doors
    with brass knobs, an oak top with a moulded edge, an arched apron between turned oak legs."""
    w, h, d = 1.84, 0.9, 0.62
    ym = h - 0.035 - 0.15
    # the wide middle door's cupboard, with plates and bowls in it
    cup = (-0.265, 0.265, LEG_H + 0.04, ym - 0.03, 0.45)
    with G.new_parts() as still:
        zf = _cabinet(w, h, d, M.body_sage, M.top_oak, M.legs_oak, top_t=0.035, top_over=0.02, hollow=cup)
        _crockery(cup, zf, M.body_sage)
        _apron('apron', w - 0.24, 0.035, 0.012, (0, LEG_H + 0.002, zf - 0.008), M.body_sage, arch=0.012, steps=12)
        for x in (-0.6, 0.0, 0.6):
            F.extrude(f'drawer{x}', G.rounded_rect(0.56, 0.13, 0.006, corner=1), 0.02, (x, ym + 0.075, zf + 0.01),
                      M.body_sage, bevel=0.005, bevel_segments=1)
            F.extrude(f'drawer_field{x}', G.rounded_rect(0.5, 0.08, 0.004, corner=1), 0.006, (x, ym + 0.075, zf + 0.022),
                      M.body_sage, bevel=0.0025, bevel_segments=1)
            _pull(f'pull{x}', x, ym + 0.075, zf + 0.025, 0.12, M.brass)
        for x in (-0.3, 0.3):
            F.box(f'stile{x}', 0.03, ym - LEG_H - 0.02, 0.01, (x, (ym + LEG_H) / 2, zf + 0.005), M.body_sage)
    G.join(still, 'body', origin=(0, 0, 0))

    def panel(x0, x1, y0, y1, zf, left):
        # a raised field: a bevelled panel on the door's face, a fillet round it
        F.extrude(f'panel{x0}', G.rounded_rect(x1 - x0 - 0.07, y1 - y0 - 0.08, 0.005, corner=1), 0.01,
                  ((x0 + x1) / 2, (y0 + y1) / 2, zf + 0.002), M.body_sage, bevel=0.0045, bevel_segments=1)
        _knob(f'knob{x0}', (x1 - 0.04) if left else (x0 + 0.04), y1 - 0.09, zf + 0.002, M.brass)
    y0, y1 = LEG_H + 0.02, ym - 0.01
    doors = [(-0.89, -0.605, True), (-0.595, -0.31, False), (-0.285, 0.285, True), (0.31, 0.595, True),
             (0.605, 0.89, False)]
    parts = [_door(f'door{i}', x0, x1, y0, y1, zf, M.body_sage, left,
                   lambda a, b, c, e, z, left=left: panel(a, b, c, e, z, left))
             for i, (x0, x1, left) in enumerate(doors)]
    G.settle()                       # where the game places it, against the wall (its Model's centre)
    # the wide middle door (hinged on its left) swings open while clean things are put away
    G.swing(parts[2], 'door_open', 100, seconds=DOOR_SECONDS)
    return G.meshes()


def _crown(name, w, d, y, z_back, material, depth=0.06):
    """A cove moulding along a cabinet's front and sides at height `y` (its foot), `w` across, its front `d` in
    front of `z_back`, `depth` high: a bead and a cove flaring out under the cornice's cap, swept round the corners."""
    prof = [(0.0, 0.0), (0.0, 0.008), (0.008, 0.011), (0.018, 0.019), (0.026, 0.03), (0.03, depth), (0.0, depth)]
    zf = z_back + d
    path = [(-w / 2, y, z_back), (-w / 2, y, zf), (w / 2, y, zf), (w / 2, y, z_back)]
    # profile (u, v): u along the frame's normal, as near Blender +Z as it goes (up), v across (outward on the front)
    return F.sweep(name, _fillet(path, 0.004, steps=1), None, material, profile=[(v, -u) for u, v in prof],
                   up=(0, 0, 1), angle=40)


def chinaCabinet():
    """China cabinet (catalog2): a cream two-door base with raised panels on a shaped plinth, a moulded worktop, a
    glazed hutch with plates on show behind doors with glazing bars, a cornice of mouldings on top."""
    z_base, z_hutch = WALL + 0.24, WALL + 0.18
    with G.new_parts() as still:
        kick = [(-0.47, 0.06), (0.47, 0.06), (0.47, 0.0), (0.4, 0.0), (0.38, 0.02)] + \
            [(0.34 - 0.68 * i / 10, 0.02 + 0.012 * math.sin(math.pi * i / 10)) for i in range(11)] + \
            [(-0.38, 0.02), (-0.4, 0.0), (-0.47, 0.0)]
        F.extrude('plinth', kick, 0.46, (0, 0, z_base - 0.01), M.plinth, bevel=0.004, bevel_segments=1)
        F.box('base', 0.94, 0.8, 0.48, (0, 0.46, z_base), M.body_china, bevel=0.012)
        _slab('worktop', 0.98, 0.51, 0.9, 0.04, M.top_china, r=0.01, at=(0, z_base + 0.015))
        for s in (-1, 1):                                                  # two base doors with brass knobs
            F.extrude(f'door{s}', G.rounded_rect(0.45, 0.7, 0.006, corner=1), 0.02, (s * 0.235, 0.46, z_base + 0.25),
                      M.body_china, bevel=0.005, bevel_segments=1)
            F.extrude(f'door_panel{s}', G.rounded_rect(0.35, 0.58, 0.005, corner=1), 0.012,
                      (s * 0.235, 0.46, z_base + 0.262), M.body_china, bevel=0.005, bevel_segments=2)
            _knob(f'knob{s}', s * 0.035, 0.6, z_base + 0.26, M.brass)
        # the hutch: sides, top, back, shelves of plates
        hy0, hy1 = 0.9, 2.04
        # the back: blue behind the plates, the body's colour outside (seen when it stands free)
        F.box('hutch_back', 0.88, hy1 - hy0, 0.01, (0, (hy0 + hy1) / 2, WALL + 0.015), M.back_blue)
        F.box('hutch_back_out', 0.88, hy1 - hy0, 0.01, (0, (hy0 + hy1) / 2, WALL + 0.005), M.body_china)
        for s in (-1, 1):
            F.box(f'hutch_side{s}', 0.03, hy1 - hy0, 0.36, (s * 0.445, (hy0 + hy1) / 2, z_hutch), M.body_china,
                  bevel=0.006)
        # the cornice: a cap with a moulded edge over a cove moulding round the front and sides
        _slab('crown', 0.98, 0.42, 2.1, 0.06, M.top_china, r=0.006, at=(0, z_hutch + 0.01),
              edge=[(0.004, 0.0), (0.0, 0.2), (0.0, 0.45), (0.008, 0.6), (0.008, 0.75), (0.0, 0.85), (0.003, 1.0)])
        _crown('cove', 0.89, 0.4, 1.995, WALL - 0.02, M.top_china, depth=0.045)
        for k, y in enumerate((1.04, 1.39, 1.73)):
            F.box(f'shelf{k}', 0.86, 0.02, 0.32, (0, y, z_hutch - 0.01), M.body_china, bevel=0.004)
            for j, x in enumerate((-0.3, -0.1, 0.1, 0.3)):
                m = M.porcelain if (j + k) % 2 == 0 else M.plate_blue
                _plate(f'plate{k}{j}', x, y + 0.095, WALL + 0.06, 0.085, m, segments=14, rot=(78, 0, 0))
            F.box(f'rail{k}', 0.86, 0.012, 0.01, (0, y + 0.04, WALL + 0.11), M.body_china)
        # cups on the lowest shelf, before the plates
        for j, x in enumerate((-0.2, 0.0, 0.2)):
            _turned(f'cup{j}', [(0, 0), (0.022, 0), (0.024, 0.004), (0.034, 0.05), (0.031, 0.05), (0.02, 0.008), (0, 0.008)],
                    (x, 1.05, WALL + 0.21), M.porcelain if j % 2 else M.plate_blue, segments=12)
    G.join(still, 'body', origin=(0, 0, 0))
    zf = z_hutch + 0.18

    def glazing(x0, x1, y0, y1, zf):
        xm = (x0 + x1) / 2
        F.box(f'pane{x0}', x1 - x0 - 0.06, y1 - y0 - 0.06, 0.006, (xm, (y0 + y1) / 2, zf - 0.01), M.glass)
        # glazing bars: one upright and two across, dividing the glass into six lights
        F.box(f'bar_v{x0}', 0.016, y1 - y0 - 0.08, 0.012, (xm, (y0 + y1) / 2, zf - 0.01), M.body_china, bevel=0.003)
        for dy in (-0.115, 0.17):
            F.box(f'bar{x0}{dy}', x1 - x0 - 0.06, 0.016, 0.012, (xm, (y0 + y1) / 2 + dy, zf - 0.01), M.body_china,
                  bevel=0.003)
    glazed_doors = []
    for s, (x0, x1) in ((-1, (-0.43, -0.005)), (1, (0.005, 0.43))):
        def glazed(a, b, c, e, z, s=s):
            glazing(a, b, c, e, z)
            _knob(f'knob_g{s}', s * 0.04, 1.45, z - 0.002, M.brass, r=0.013)
        glazed_doors.append(_door(f'glass_door{s}', x0, x1, 0.93, 2.0, zf, M.body_china, s < 0, glazed, frame=0.045))
    G.settle()                       # where the game places it, against the wall (its Model's centre)
    # the left glazed door (hinged on its left) swings open onto the plates while clean things are put away
    G.swing(glazed_doors[0], 'door_open', 100, seconds=DOOR_SECONDS)
    return G.meshes()


def _bottle(name, x, y, z, r, h, material, cap, label=None, verts=12):
    """A bottle standing at (x, y, z): a round body, a shoulder, a long neck and a cork; `label`: a paper band."""
    bx, by, bz = F.at(x, y, z)
    rings = [(0.0, 0.85), (0.02, 1.0), (0.55, 1.0), (0.66, 0.78), (0.74, 0.4), (0.93, 0.36), (0.95, 0.42),
             (0.97, 0.36)]
    G.loft(name, [G.ring(r * k, r * k, bz + h * t, verts, at=(bx, by)) for t, k in rings], material=material)
    F.cyl(f'{name}_cork', r * 0.34, 0.03, (x, y + h + 0.01, z), cap, verts=8)
    if label:
        F.cyl(f'{name}_label', r * 1.02, h * 0.22, (x, y + h * 0.3, z), label, verts=verts)


def barCart():
    """Bar cart (catalog2): two glass tiers in brass frames on castors, a handle bent from one tube, bottles, glasses,
    an ice bucket and a basket; the shaker is taken off it while a resident mixes a drink."""
    with G.new_parts() as still:
        for k, y in enumerate((0.3, 0.76)):
            _slab(f'tray{k}', 0.68, 0.38, y + 0.006, 0.012, M.tray, r=0.02,
                  edge=[(0.002, 0.0), (0.0, 0.3), (0.0, 0.7), (0.002, 1.0)])
            # the gallery: a rounded rectangle of brass tube round the glass, standing a little above it
            _rail(f'rim{k}', [(-0.35, y + 0.012, -0.2), (0.35, y + 0.012, -0.2), (0.35, y + 0.012, 0.2),
                              (-0.35, y + 0.012, 0.2)], 0.011, M.frame_brass, bend=0.03, verts=8, closed=True)
        for x in (-0.35, 0.35):
            for z in (-0.2, 0.2):
                G.kit.tube(F, f'post{x}{z}', [(x, 0.06, z), (x, 0.8, z)], 0.011, M.frame_brass, verts=10)
                _turned(f'finial{x}{z}', [(0, 0), (0.012, 0), (0.017, 0.008), (0.018, 0.016), (0.012, 0.026),
                                          (0.006, 0.03), (0, 0.031)], (x, 0.792, z), M.frame_brass, segments=10)
                # the castor: a swivel fork in brass, a black wheel on an axle
                _turned(f'socket{x}{z}', [(0, 0), (0.016, 0), (0.018, 0.006), (0.014, 0.02), (0, 0.02)], (x, 0.048, z),
                        M.frame_brass, segments=10)
                F.box(f'fork{x}{z}', 0.03, 0.022, 0.04, (x, 0.04, z), M.frame_brass, bevel=0.006, bevel_segments=1)
                G.revolve(f'castor{x}{z}', [(0, -0.008), (0.018, -0.009), (0.024, -0.007), (0.025, 0.0), (0.024, 0.007),
                                            (0.018, 0.009), (0, 0.008)], at=F.at(x, 0.025, z), material=M.castor,
                          segments=14, rot=(0, 90, 0))
        # a gallery rail round the lower tier on short balusters, so nothing rolls off
        _rail('gallery', [(-0.35, 0.345, -0.2), (0.35, 0.345, -0.2), (0.35, 0.345, 0.2), (-0.35, 0.345, 0.2)], 0.006,
              M.frame_brass, bend=0.03, verts=6, closed=True)
        for x in (-0.12, 0.12):
            for z in (-0.2, 0.2):
                G.kit.tube(F, f'baluster{x}{z}', [(x, 0.312, z), (x, 0.345, z)], 0.005, M.frame_brass, verts=6)
        # the handle: bent from one tube, out from the posts and across
        _rail('handle', [(-0.35, 0.78, -0.2), (-0.395, 0.84, -0.2), (-0.395, 0.84, 0.2), (-0.35, 0.78, 0.2)], 0.011,
              M.frame_brass, bend=0.03, verts=8)
        # bottles on the top tier, labelled
        for i, (x, m) in enumerate(((-0.24, M.bottle_green), (-0.15, M.bottle_red), (-0.06, M.bottle_amber))):
            _bottle(f'bottle{i}', x, 0.766, -0.08, 0.035, 0.3, m, M.basket, label=M.porcelain)
        # two wine glasses
        for i, (x, z) in enumerate(((0.2, 0.1), (0.28, 0.0))):
            _turned(f'wine_glass{i}', [(0, 0), (0.03, 0), (0.03, 0.004), (0.006, 0.01), (0.004, 0.07), (0.012, 0.08),
                                       (0.033, 0.11), (0.04, 0.16), (0.036, 0.16), (0.03, 0.12), (0.012, 0.09), (0, 0.088)],
                    (x, 0.766, z), M.glass, segments=12)
        # a lemon dish on the top tier; an ice bucket with tongs and a woven basket on the lower one
        _turned('dish', [(0, 0), (0.035, 0), (0.055, 0.012), (0.062, 0.016), (0.056, 0.016), (0.035, 0.006), (0, 0.006)],
                (0.1, 0.766, -0.1), M.porcelain, segments=16)
        for i, (dx, dz) in enumerate(((0, 0), (0.035, 0.025))):
            F.sphere(f'lemon{i}', 0.025, (0.1 + dx, 0.8, -0.1 + dz), M.lemon, scale=(1, 1, 0.85), segments=10,
                     rings=6)
        _turned('ice_bucket', [(0, 0), (0.065, 0), (0.068, 0.008), (0.08, 0.14), (0.084, 0.15), (0.075, 0.15),
                               (0.064, 0.02), (0, 0.02)], (0.0, 0.306, 0.0), M.shaker, segments=18)
        for s in (-1, 1):
            F.cyl(f'bucket_ear{s}', 0.014, 0.008, (s * 0.083, 0.43, 0.0), M.shaker, rot=(0, 90, 0), verts=8)
        for i in range(5):                              # ice in the bucket
            F.box(f'ice{i}', 0.03, 0.03, 0.03, (-0.03 + 0.015 * i, 0.43 + 0.01 * (i % 2), -0.02 + 0.01 * (i % 3)), M.tray,
                  rot=(20 * i, 10 * i, 30 * i), bevel=0.006, bevel_segments=1)
        F.soft('basket', 0.18, 0.1, 0.16, (0.2, 0.356, 0.05), M.basket, round=0.02, puff=(0.006, 0.0, 0.006))
        _rail('basket_rim', [(0.11, 0.405, -0.03), (0.29, 0.405, -0.03), (0.29, 0.405, 0.13), (0.11, 0.405, 0.13)], 0.006,
              M.basket, bend=0.02, verts=5, closed=True)
        _rail('basket_handle', [(0.2, 0.405, -0.03), (0.2, 0.46, -0.03), (0.2, 0.46, 0.13), (0.2, 0.405, 0.13)], 0.005,
              M.basket, bend=0.04, verts=5)
    G.join(still, 'cart', origin=(0, 0, 0))
    # the cocktail shaker, standing at the front of the top tier, in the middle, before the resident who
    # mixes from the front: its own part, its origin at its foot, squashed away while it is in their hands
    sx, sz = 0.0, 0.1
    with G.new_parts() as shake:
        _turned('shaker', [(0, 0), (0.034, 0), (0.036, 0.004), (0.04, 0.12), (0.042, 0.124), (0.037, 0.13),
                           (0.03, 0.17), (0.032, 0.174), (0.018, 0.2), (0.012, 0.212), (0.014, 0.215), (0, 0.22)],
                (sx, 0.766, sz), M.shaker, segments=16)
    shaker = G.hinged(G.join(shake, 'shaker'), F.at(sx, 0.766, sz))
    G.settle(centre=False)
    G.squash(shaker, 'shake', 0.001, seconds=0.2)
    return G.meshes()


DINING_TOP = [('top_oak', 'Top'), ('legs_oakd', 'Legs')]
SEAT = [('frame_oak', 'Frame'), ('seat_sage', 'Seat')]
STOOL = {'uses': {'sit': 'swivel', 'drink': 'swivel'}, 'hold': 0.6, 'ride': 'swivel'}

FAMILY = G.Family(
    'dining', M,
    [  # K7: dining tables
     G.Model('diningTable', diningTable, size=(1.674, 1.027, 0.75), centre=(0, 0.1), key='model.diningTable',
             paint=DINING_TOP),
     G.Model('bistroSet', bistroSet, size=(0.62, 1.004, 0.912), key='model.bistroSet', centre=(0, 0.024),
             paint=[('top_oak', 'Top'), ('legs_black', 'Frame'), ('seat_sage', 'Seats')]),
     G.Model('diningGlass', diningGlass, size=(1.666, 1.073, 0.8), centre=(0, 0.1), key='model.diningGlass',
             paint=[('glass', 'Top'), ('legs_chrome', 'Legs')]),
     G.Model('diningOak', diningOak, size=(1.674, 1.077, 0.8), centre=(0, 0.1), key='model.diningOak',
             paint=DINING_TOP),
     G.Model('tableSquare2', tableSquare2, size=(0.82, 0.75, 0.82), key='model.tableSquare2', paint=DINING_TOP),
     G.Model('tableBistroRound', tableBistroRound, size=(0.72, 0.75, 0.72), key='model.tableBistroRound',
             paint=[('top_white', 'Top'), ('legs_black', 'Legs')]),
     G.Model('tableDining4', tableDining4, size=(1.84, 0.75, 0.88), key='model.tableDining4',
             paint=[('top_oak', 'Top'), ('legs_oak', 'Legs')]),
     G.Model('tableDining6', tableDining6, size=(2.84, 0.75, 0.88), key='model.tableDining6',
             paint=[('top_walnut', 'Top'), ('legs_walnut', 'Legs')]),
     G.Model('tableBanquet8', tableBanquet8, size=(3.84, 0.751, 0.88), key='model.tableBanquet8',
             paint=[('top_banquet', 'Top'), ('legs_banquet', 'Legs')]),
     G.Model('tableRound4', tableRound4, size=(1.52, 0.75, 1.52), key='model.tableRound4',
             paint=[('top_birch', 'Top'), ('legs_birch', 'Legs')]),
     G.Model('tableSquare8', tableSquare8, size=(1.86, 0.75, 1.86), key='model.tableSquare8',
             paint=[('top_white_satin', 'Top'), ('legs_black', 'Legs')]),
     G.Model('tableFarmhouse6', tableFarmhouse6, size=(2.86, 0.751, 0.88), key='model.tableFarmhouse6',
             paint=[('top_farm', 'Top'), ('legs_farm', 'Legs')]),
     G.Model('tableBarHigh', tableBarHigh, size=(0.7, 1.06, 0.7), key='model.tableBarHigh',
             paint=[('top_black', 'Top'), ('legs_black', 'Legs')]),
     # K8: chairs and stools
     G.Model('barStool', barStool, size=(0.39, 0.742, 0.39), key='model.barStool', motions=STOOL, paint=SEAT),
     G.Model('stoolCounter', stoolCounter, size=(0.39, 0.687, 0.39), centre=(0, 0.05), key='model.stoolCounter',
             motions=STOOL, paint=SEAT),
     G.Model('stoolBarHigh', stoolBarHigh, size=(0.39, 1.117, 0.398), centre=(0, 0.046), key='model.stoolBarHigh',
             motions=STOOL, paint=[('legs_black', 'Frame'), ('leather', 'Seat')]),
     G.Model('diningChair', diningChair, size=(0.44, 1.002, 0.463), centre=(0, -0.002), key='model.diningChair',
             paint=[('frame_cream', 'Frame'), ('seat_sage', 'Seat')]),
     G.Model('chairWood', chairWood, size=(0.44, 0.902, 0.45), centre=(0, 0.035), key='model.chairWood',
             paint=[('frame_honey', 'Frame'), ('seat_honey', 'Seat')]),
     G.Model('chairUpholstered', chairUpholstered, size=(0.46, 0.992, 0.496), centre=(0, 0.032),
             key='model.chairUpholstered', paint=[('legs_walnut', 'Frame'), ('upholstery', 'Upholstery')]),
     G.Model('chairModern', chairModern, size=(0.44, 0.789, 0.496), centre=(0, 0.032), key='model.chairModern',
             paint=[('legs_black', 'Frame'), ('shell', 'Seat')]),
     G.Model('chairWindsor', chairWindsor, size=(0.475, 0.975, 0.474), centre=(0, 0.041), key='model.chairWindsor',
             paint=[('frame_windsor', 'Frame'), ('seat_windsor', 'Seat')]),
     G.Model('chairBentwood', chairBentwood, size=(0.434, 0.968, 0.452), centre=(0, 0.041), key='model.chairBentwood',
             paint=[('frame_bent', 'Frame'), ('seat_bent', 'Seat')]),
     G.Model('chairFolding', chairFolding, size=(0.46, 0.892, 0.48), centre=(0, 0.06), key='model.chairFolding',
             paint=[('frame_steel', 'Frame'), ('seat_teal', 'Seat')]),
     G.Model('chairArm', chairArm, size=(0.57, 0.922, 0.47), centre=(0, 0.035), key='model.chairArm', paint=SEAT),
     G.Model('benchDining', benchDining, size=(1.6, 0.468, 0.36), centre=(0, 0.05), key='model.benchDining',
             paint=[('legs_oakd', 'Frame'), ('seat_oak', 'Seat')]),
     # K9: dining storage
     # putting clean dishes away (a `cupboard` station, cooking putAwayAt, anim fetch) opens a door
     G.Model('sideboard', sideboard, size=(1.82, 1.1, 0.668), centre=(0, -0.096), key='model.sideboard',
             motions=PUT_AWAY, paint=[('body_walnut', 'Body'), ('top_sideboard', 'Top'), ('brass', 'Handles')]),
     G.Model('buffet', buffet, size=(1.88, 0.9, 0.668), centre=(0, -0.096), key='model.buffet',
             motions=PUT_AWAY, paint=[('body_sage', 'Body'), ('top_oak', 'Top'), ('brass', 'Handles')]),
     G.Model('chinaCabinet', chinaCabinet, size=(0.98, 2.1, 0.548), centre=(0, -0.156), key='model.chinaCabinet',
             motions=PUT_AWAY, paint=[('body_china', 'Body'), ('top_china', 'Top'), ('brass', 'Handles')]),
     G.Model('barCart', barCart, size=(0.769, 1.09, 0.45), centre=(-0.017, 0), key='model.barCart',
             motions={'uses': {'mix': 'shake'}},
             paint=[('frame_brass', 'Frame'), ('tray', 'Trays')])],
    comment='Blender-built dining tables, chairs, stools and storage '
            '(tools/art/blender/families/dining.py). Listed in the base manifest\'s packs. The materials '
            'map onto the game\'s finishes so a model costs one draw call per finish group. Table tops '
            'stand at the content\'s surface heights and seats where the game seats residents; the stools\' '
            'seats swivel 20 degrees as a resident sits (`swivel`, carrying them), and the bar cart\'s '
            'cocktail shaker is taken off it while a drink is mixed (`shake`).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
