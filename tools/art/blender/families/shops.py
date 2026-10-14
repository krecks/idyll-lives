# The town's shop fittings and café furniture (CM4): the shop counter with its till, the bakery case,
# the grocery shelves, the café table and the bentwood café chair. Each replaces the primitive-part
# placeholder of `tools/art/community.py` (content: web/public/content/community.json), which stays the
# fallback if the file fails to load. Notes: shops.md.
#
# They stand on town lots, seen from far and by the dozen, so each has a far version (`far=0.25`, E7).
# The parts are placed in game axes (`F`), where the placeholders stand: the counter, the case and the
# shelves sit back on their 2 x 1 tiles as theirs do (`G.settle(centre=False)` and `centre=` on the
# Model), the café chair's seat 2 cm forward. Two clips: the till's cash drawer slides out to the front
# for `treat` (`till`), and the bakery case's back glass door slides aside for `pastry` (`door`).
#
# Run:  python3 -B tools/art/blender/build.py shops [--preview]
import json
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
import bpy  # noqa: E402

import gamekit as G  # noqa: E402

M = G.Palette(
    # the shop counter
    counter=('#CDB08A', 'wood'),     # the oak carcass and panels; the main part the player colours
    walnut=('#7A5234', 'wood'),      # its walnut worktop, rails, stiles and plinth
    till=('#3A3D41', 'satin'),       # the till's case
    keys=('#C9CCCF', 'satin'),       # its keys and the drawer's tray
    key2=('#D9483B', 'gloss'),       # its total key
    screen=('#7FD1A8', 'gloss'),     # its display
    note=('#8FB98A', 'matte'),       # banknotes in the drawer
    coin=('#C9A54A', 'metal'),       # coins in the drawer
    jar=('#DDEBF0', 'gloss'),        # the sweet jar (opaque gloss: glass would cost a draw call)
    jarLid=('#B8322B', 'gloss'),     # its lid
    sweet1=('#D9483B', 'gloss'), sweet2=('#F2C14E', 'gloss'), sweet3=('#5EC28B', 'gloss'),
    sweet4=('#E9A6B4', 'gloss'), sweet5=('#4E7A7F', 'gloss'),
    mag1=('#3E5470', 'gloss'), mag2=('#D9483B', 'gloss'), mag3=('#D6A64A', 'gloss'), mag4=('#9DAF8C', 'gloss'),
    page=('#F2F0EA', 'matte'),       # a magazine's title strip
    # the bakery case
    case=('#F1E6D0', 'satin'),       # the cream cabinet; the main part the player colours
    rim=('#7A5234', 'satin'),        # its walnut deck, ends, rails and tracks (plain, with the body)
    stripe=('#C98F6B', 'satin'),     # the band along its front
    glass=('#DDEBF0', 'glass'),      # the display's glass and its back doors
    tray=('#ECE8DF', 'gloss'),       # paper doilies and trays
    bun=('#C98F4E', 'satin'), crust=('#B97A40', 'satin'), pastry=('#D9A35F', 'satin'),
    jam=('#C0392B', 'gloss'), lemon=('#F2C14E', 'gloss'), sponge=('#F2E1C1', 'satin'),
    icing=('#E9A6B4', 'gloss'), choc=('#5A3B26', 'gloss'), cream=('#F7F3EA', 'gloss'),
    # the grocery shelves
    shelf=('#D3D4D2', 'metal'),      # the frame and shelves; the main part the player colours
    trim=('#5C6168', 'metal'),       # the kick plate, top cap and shelf lips
    label=('#F7F5EE', 'matte'),      # price tags
    bread=('#E8D2A6', 'matte'),      # sandwich packs
    filling=('#7FA35A', 'matte'),    # the lettuce in them
    good1=('#D9483B', 'gloss'), good2=('#F2C14E', 'gloss'), good3=('#4E7A7F', 'gloss'),
    good4=('#9DAF8C', 'gloss'), good5=('#E8902E', 'gloss'), good6=('#3E5470', 'gloss'),
    good7=('#F2F0EA', 'gloss'), good8=('#8C5A3C', 'gloss'), good9=('#E9A6B4', 'gloss'),
    tinLid=('#B9BDC1', 'metal'),     # tins' and jars' lids
    # the café table and chair
    marble=('#EDEAE4', 'gloss'),     # the table's top; its main part
    iron=('#2B302E', 'metal'),       # its cast-iron foot
    bentwood=('#5A3B26', 'wood'),    # the chair's frame; its main part
    cane=('#D9C7A7', 'fabric'),      # its woven seat
)

TILL_SECONDS = 0.8   # the cash drawer pops out this fast
TILL_HOLD = 1.5      # and shuts again this long after the use starts
DOOR_SECONDS = 1.0   # the bakery case's back door slides aside this long
DOOR_HOLD = 2.0      # and shuts again this long after the use starts

# Heights the content names (community.json): kept here and checked against it.
TABLE_TOP = 0.745    # cafeTable surface.height
SEAT = 0.47          # cafeChair seat.height
SEAT_FORWARD = 0.02  # cafeChair seat.forward


def _content_facts():
    path = os.path.join(G.ASSETS, '..', 'content', 'community.json')
    with open(path) as f:
        objs = {o['id']: o for o in json.load(f)['objects']}
    assert objs['cafeTable']['surface']['height'] == TABLE_TOP, 'cafeTable: surface height moved'
    assert objs['cafeChair']['seat'] == {'height': SEAT, 'forward': SEAT_FORWARD}, 'cafeChair: seat moved'


def _prism(name, profile, a0, a1, material, axis='x', shift=0.0):
    """A slab from a0 to a1 along game x (its side the convex polygon `profile` [(depth, height)], in
    order: the bakery case's end panels) or along the depth (axis='z', its face [(x, height)]: a
    sandwich pack). `shift` moves it back as a GameFrame's does."""
    bm = bmesh.new()
    if axis == 'x':
        a = [bm.verts.new((-a0, -z - shift, y)) for z, y in profile]
        b = [bm.verts.new((-a1, -z - shift, y)) for z, y in profile]
    else:
        a = [bm.verts.new((-x, -a0, y)) for x, y in profile]
        b = [bm.verts.new((-x, -a1, y)) for x, y in profile]
    bm.faces.new(a)
    bm.faces.new(list(reversed(b)))
    n = len(profile)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    return obj


def _catmull(points, steps=4):
    """A smooth path through `points` (game (x, height, depth)), `steps` samples a span."""
    pts = [points[0]] + list(points) + [points[-1]]
    out = []
    for i in range(1, len(pts) - 2):
        p0, p1, p2, p3 = pts[i - 1], pts[i], pts[i + 1], pts[i + 2]
        for s in range(steps):
            t = s / steps
            out.append(tuple(0.5 * (2 * p1[k] + (-p0[k] + p2[k]) * t + (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * t * t
                                     + (-p0[k] + 3 * p1[k] - 3 * p2[k] + p3[k]) * t ** 3) for k in range(3)))
    out.append(points[-1])
    return out


def _tube(F, name, points, radius, material, steps=4):
    """A round bar bent through `points` (game axes), capped: bentwood and cast-iron curves."""
    cu = bpy.data.curves.new(name, 'CURVE')
    cu.dimensions = '3D'
    cu.bevel_depth = radius
    cu.bevel_resolution = 1
    cu.use_fill_caps = True
    path = _catmull(points, steps)
    sp = cu.splines.new('POLY')
    sp.points.add(len(path) - 1)
    for p, q in zip(sp.points, path):
        p.co = (*F.at(*q), 1.0)
    obj = bpy.data.objects.new(name, cu)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.shade_smooth()
    return bpy.context.view_layer.objects.active


# --- the shop counter: 2x1, an oak counter with a till, a sweet jar and a magazine rack -------------
def _tilted(cx, cy, cz, deg, dz, up):
    """A point on a plane through (cx, cy, cz) tilted `deg` back (its back edge higher): `dz` towards
    the front along it, `up` off it."""
    t = math.radians(deg)
    return (cx, cy - dz * math.sin(t) + up * math.cos(t), cz + dz * math.cos(t) + up * math.sin(t))


def shopCounter():
    F = G.GameFrame(shift=0.04)   # 4 cm forward: the worktop's back on the wall face (-0.43)
    Z, FRONT, TOP = -0.08, 0.27, 0.97          # the counter's depth centre, front face, worktop
    TX, TZ = 0.5, -0.2                          # the till on the worktop
    with G.new_parts() as still:
        F.box('plinth', 1.84, 0.08, 0.64, (0, 0.04, Z - 0.01), M.walnut, bevel=0.004)
        F.box('carcass', 1.9, 0.84, 0.7, (0, 0.5, Z), M.counter, bevel=0.008)
        F.box('worktop', 1.96, 0.05, 0.78, (0, 0.945, Z), M.walnut, bevel=0.01)
        # the front: walnut rails and stiles framing three raised oak panels
        F.box('rail_top', 1.9, 0.06, 0.02, (0, 0.87, FRONT + 0.01), M.walnut, bevel=0.004)
        F.box('rail_bottom', 1.9, 0.08, 0.02, (0, 0.12, FRONT + 0.01), M.walnut, bevel=0.004)
        for i, x in enumerate((-0.925, -0.31, 0.31, 0.925)):
            F.box(f'stile{i}', 0.05, 0.68, 0.02, (x, 0.5, FRONT + 0.01), M.walnut, bevel=0.004)
        for i, x in enumerate((-0.6175, 0.0, 0.6175)):
            F.box(f'field{i}', 0.42, 0.5, 0.024, (x, 0.5, FRONT + 0.012), M.counter, bevel=0.008)
        # the back, the shopkeeper's side: two cupboard doors with knobs
        for i, x in enumerate((-0.47, 0.47)):
            F.box(f'back_door{i}', 0.86, 0.7, 0.016, (x, 0.5, Z - 0.355), M.counter, bevel=0.005)
            F.sphere(f'back_knob{i}', 0.016, (x - 0.36 * (1 if x > 0 else -1), 0.74, Z - 0.37), M.walnut,
                     segments=8, rings=5)
        # the till: a case holding the drawer, a keypad sloping up to the back and a display on a post
        F.box('till_case', 0.38, 0.09, 0.34, (TX, TOP + 0.045, TZ), M.till, bevel=0.008)
        kc = (TX, TOP + 0.115, TZ - 0.04)
        F.box('till_keypad', 0.34, 0.05, 0.2, kc, M.till, bevel=0.008, rot=(12, 0, 0))
        for r in range(3):
            for c in range(4):
                at = _tilted(TX - 0.105 + c * 0.07, kc[1], kc[2], 12, 0.06 - r * 0.05, 0.03)
                F.box(f'key{r}{c}', 0.05, 0.012, 0.035, (at[0], at[1], at[2]),
                      M.key2 if (r, c) == (0, 3) else M.keys, bevel=0.003, rot=(12, 0, 0))
        F.cyl('till_post', 0.015, 0.1, (TX, TOP + 0.17, TZ - 0.13), M.till, verts=8)
        F.box('till_display', 0.22, 0.1, 0.03, (TX, TOP + 0.255, TZ - 0.13), M.till, bevel=0.006, rot=(-15, 0, 0))
        # its screen on the display's face, which leans back 15 degrees
        F.box('till_screen', 0.18, 0.07, 0.006, (TX, TOP + 0.255 + 0.0047, TZ - 0.13 + 0.0174), M.screen, rot=(-15, 0, 0))
        # the sweet jar: a gloss jar with sweets pressed to its sides, as if seen through it
        JX, JZ = -0.2, 0.05
        F.cyl('jar', 0.085, 0.2, (JX, TOP + 0.1, JZ), M.jar, verts=20)
        F.cone('jar_shoulder', 0.085, 0.06, 0.03, (JX, TOP + 0.215, JZ), M.jar, verts=20)
        F.cyl('jar_lid', 0.068, 0.035, (JX, TOP + 0.247, JZ), M.jarLid, verts=20, bevel=0.006)
        sweets = (M.sweet1, M.sweet2, M.sweet3, M.sweet4, M.sweet5)
        k = 0
        for row, y in enumerate((0.03, 0.075, 0.12, 0.165)):
            for j in range(7):
                a = math.radians(j * 360 / 7 + row * 25)
                F.sphere(f'sweet{row}{j}', 0.021, (JX + 0.075 * math.sin(a), TOP + y, JZ + 0.075 * math.cos(a)),
                         sweets[k % 5], segments=6, rings=4)
                k += 3
        # the magazine rack: a walnut stand with four magazines leaning back
        RX, RZ = -0.66, -0.24
        F.box('rack_base', 0.52, 0.03, 0.18, (RX, TOP + 0.015, RZ), M.walnut, bevel=0.004)
        F.box('rack_back', 0.52, 0.22, 0.015, (RX, TOP + 0.13, RZ - 0.09), M.walnut, bevel=0.004, rot=(-15, 0, 0))
        for i, mag in enumerate((M.mag1, M.mag2, M.mag3, M.mag4)):
            x, z = RX - 0.18 + i * 0.12, RZ - 0.045 + (i % 2) * 0.03
            F.box(f'magazine{i}', 0.16, 0.23, 0.01, (x, TOP + 0.145, z), mag, rot=(-15, 0, 0))
            up, t = 0.09, math.radians(15)   # the title strip near the top of its leaning cover
            F.box(f'masthead{i}', 0.13, 0.03, 0.004, (x, TOP + 0.145 + up * math.cos(t) + 0.0018,
                                                       z - up * math.sin(t) + 0.0068), M.page, rot=(-15, 0, 0))
    G.join(still, 'counter', origin=(0, 0, 0))

    # the cash drawer: a front under the keypad and a tray of notes and coins hidden in the case at rest
    DZ = TZ + 0.17   # the case's front face
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.35, 0.065, 0.014, (TX, TOP + 0.042, DZ + 0.007), M.till, bevel=0.004)
        F.box('drawer_lip', 0.12, 0.012, 0.012, (TX, TOP + 0.06, DZ + 0.018), M.keys, bevel=0.003)
        F.box('drawer_tray', 0.33, 0.045, 0.3, (TX, TOP + 0.03, DZ - 0.15), M.keys, bevel=0.003)
        for i in range(3):
            F.box(f'divider{i}', 0.006, 0.012, 0.28, (TX - 0.09 + i * 0.09, TOP + 0.058, DZ - 0.15), M.till)
        for i in range(4):
            F.box(f'notes{i}', 0.07, 0.008, 0.13, (TX - 0.135 + i * 0.09, TOP + 0.056, DZ - 0.22), M.note)
            for j in range(2):
                F.cyl(f'coin{i}{j}', 0.012, 0.006, (TX - 0.15 + i * 0.09 + j * 0.03, TOP + 0.055, DZ - 0.07),
                      M.coin, verts=8)
    drawer = G.join(drawer, 'cash_drawer', origin=F.at(TX, TOP + 0.03, DZ))

    G.settle(centre=False)  # back on its tiles, where the placeholder stands
    G.slide(drawer, 'till', (0, -0.2, 0), seconds=TILL_SECONDS)


# --- the bakery case: 2x1, a cream cabinet under a glass display, its back doors sliding ------------
def bakeryCase():
    F = G.GameFrame(shift=0.04)   # 4 cm forward: the worktop's back on the wall face (-0.43)
    Z, FRONT, DECK = -0.04, 0.36, 0.78
    BACK = -0.43      # the display's back face, before the frame's 4 cm shift forward
    with G.new_parts() as still:
        F.box('plinth', 1.86, 0.06, 0.76, (0, 0.03, Z), M.rim, bevel=0.004)
        F.box('cabinet', 1.9, 0.68, 0.8, (0, 0.4, Z), M.case, bevel=0.01)
        F.box('deck', 1.94, 0.04, 0.86, (0, 0.76, Z), M.rim, bevel=0.008)
        # the front: a band of colour over three panels
        F.box('band', 1.9, 0.08, 0.016, (0, 0.62, FRONT + 0.008), M.stripe, bevel=0.003)
        for i, x in enumerate((-0.62, 0.0, 0.62)):
            F.box(f'panel{i}', 0.54, 0.38, 0.016, (x, 0.33, FRONT + 0.008), M.case, bevel=0.006)
            F.box(f'moulding{i}', 0.46, 0.3, 0.012, (x, 0.33, FRONT + 0.018), M.case, bevel=0.005)
        # the display: walnut ends, sloping front glass, glass top and rails
        for s in (-1, 1):
            x = s * 0.935
            _prism(f'end{s}', [(-0.45, DECK), (0.38, DECK), (0.2, 1.29), (-0.45, 1.29)], x - 0.015, x + 0.015, M.rim, shift=F.shift)
        lean = math.degrees(math.atan2(0.19, 0.48))
        F.box('glass_front', 1.84, math.hypot(0.19, 0.48), 0.01, (0, DECK + 0.24, 0.27), M.glass, rot=(-lean, 0, 0))
        F.box('glass_top', 1.84, 0.01, 0.6, (0, 1.265, -0.125), M.glass)
        F.box('rail_front', 1.9, 0.03, 0.05, (0, 1.275, 0.18), M.rim, bevel=0.004)
        F.box('rail_back', 1.9, 0.03, 0.05, (0, 1.275, BACK), M.rim, bevel=0.004)
        F.box('track', 1.84, 0.02, 0.05, (0, DECK + 0.01, BACK), M.rim, bevel=0.003)
        # the fixed back pane, in the rear track on the left
        F.box('glass_back', 0.93, 0.44, 0.008, (-0.455, 1.02, BACK - 0.01), M.glass)
        # a glass shelf over the back row, on two brackets
        F.box('glass_shelf', 1.84, 0.01, 0.28, (0, 1.03, -0.25), M.glass)
        for s in (-1, 1):
            F.box(f'bracket{s}', 0.02, 0.025, 0.28, (s * 0.905, 1.015, -0.25), M.rim)
        # the bakes: a front row of buns, tarts and loaves on doilies, cakes and biscuits behind, and
        # croissants and cupcakes on the shelf
        for i in range(6):
            x = -0.75 + i * 0.3
            F.cyl(f'doily{i}', 0.11, 0.006, (x, DECK + 0.003, 0.14), M.tray, verts=16)
            F.box(f'tray{i}', 0.26, 0.012, 0.2, (x, DECK + 0.006, -0.2), M.tray, bevel=0.003)
            k = i % 3
            if k == 0:
                for j, (dx, dz) in enumerate(((-0.045, -0.03), (0.045, -0.03), (0.0, 0.04))):
                    F.sphere(f'bun{i}{j}', 0.045, (x + dx, DECK + 0.03, 0.14 + dz), M.bun, scale=(1, 1, 0.7),
                             segments=10, rings=6)
            elif k == 1:
                F.cyl(f'tart{i}', 0.085, 0.03, (x, DECK + 0.021, 0.14), M.pastry, verts=16, bevel=0.008)
                F.cyl(f'tart_fill{i}', 0.07, 0.008, (x, DECK + 0.037, 0.14), M.jam if i < 3 else M.lemon, verts=16)
            else:
                F.box(f'loaf{i}', 0.19, 0.08, 0.1, (x, DECK + 0.046, 0.14), M.crust, bevel=0.03, bevel_segments=2)
                for j in range(3):
                    F.box(f'score{i}{j}', 0.012, 0.004, 0.08, (x - 0.05 + j * 0.05, DECK + 0.087, 0.14), M.pastry)
            if i % 2 == 0:
                cake = M.icing if i != 2 else M.choc
                F.cyl(f'cake{i}', 0.09, 0.07, (x, DECK + 0.047, -0.2), M.sponge, verts=16)
                F.cyl(f'cake_top{i}', 0.088, 0.012, (x, DECK + 0.087, -0.2), cake, verts=16, bevel=0.005)
                for j in range(3):
                    a = math.radians(j * 120 + 30)
                    F.sphere(f'cherry{i}{j}', 0.013, (x + 0.05 * math.sin(a), DECK + 0.1, -0.2 + 0.05 * math.cos(a)),
                             M.jam, segments=6, rings=4)
            else:
                for j in range(4):
                    F.cyl(f'biscuit{i}{j}', 0.04, 0.012, (x - 0.055 + (j % 2) * 0.11, DECK + 0.02 + (j // 2) * 0.012,
                                                          -0.2 + (j % 2) * 0.02), M.pastry if j % 2 else M.choc, verts=10)
        for i in range(5):
            x = -0.72 + i * 0.36
            if i % 2 == 0:
                for j, dx in enumerate((-0.06, 0.06)):
                    F.sphere(f'croissant{i}{j}', 0.05, (x + dx, 1.06, -0.24), M.crust, scale=(1.5, 0.8, 0.55),
                             segments=10, rings=6, rot=(0, 0, 25 if j else -25))
            else:
                for j, dx in enumerate((-0.065, 0.0, 0.065)):
                    F.cone(f'cupcake{i}{j}', 0.022, 0.03, 0.04, (x + dx, 1.055, -0.24), M.tray, verts=10)
                    F.sphere(f'frosting{i}{j}', 0.032, (x + dx, 1.085, -0.24), M.cream if j != 1 else M.icing,
                             scale=(1, 1, 0.75), segments=8, rings=5)
    G.join(still, 'case', origin=(0, 0, 0))

    # the back door: the right-hand pane, in the front track, sliding left in front of the fixed one
    with G.new_parts() as door:
        F.box('door_glass', 0.93, 0.44, 0.008, (0.455, 1.02, BACK + 0.008), M.glass)
    door = G.join(door, 'back_door', origin=F.at(0.455, 1.02, BACK + 0.008))

    G.settle(centre=False)
    G.slide(door, 'door', (0.84, 0, 0), seconds=DOOR_SECONDS)   # Blender +x is the game's -x: to the left


# --- the grocery shelves: 2x1, a metal gondola of tins, packets and jars, sandwiches at the bottom ---
def groceryShelf():
    F = G.GameFrame()
    goods = (M.good1, M.good2, M.good3, M.good4, M.good5, M.good6, M.good7, M.good8, M.good9)
    with G.new_parts() as still:
        for s in (-1, 1):
            F.box(f'side{s}', 0.04, 1.72, 0.47, (s * 0.94, 0.86, -0.195), M.shelf, bevel=0.008)
        F.box('back', 1.84, 1.62, 0.025, (0, 0.89, -0.4175), M.shelf, bevel=0.004)
        F.box('kick', 1.84, 0.1, 0.44, (0, 0.05, -0.19), M.trim, bevel=0.004)
        F.box('cap', 1.92, 0.04, 0.48, (0, 1.70, -0.19), M.trim, bevel=0.006)
        k = 1
        for row, y in enumerate((0.115, 0.52, 0.92, 1.32)):
            F.box(f'shelf{row}', 1.84, 0.025, 0.44, (0, y, -0.19), M.shelf, bevel=0.003)
            F.box(f'lip{row}', 1.84, 0.045, 0.012, (0, y - 0.004, 0.036), M.trim, bevel=0.003)
            for t in range(4):
                F.box(f'tag{row}{t}', 0.06, 0.028, 0.004, (-0.66 + t * 0.44, y - 0.004, 0.044), M.label)
            x, top = -0.89, y + 0.0125
            n = 0
            while x < 0.84:
                k = (k * 7 + 5) % 23
                c = goods[k % len(goods)]
                name = f'g{row}_{n}'
                n += 1
                if row == 0:     # sandwich packs: triangles standing on their long side, two deep
                    w, cx = 0.15, x + 0.075
                    for j, z in enumerate((-0.23, -0.06)):
                        _prism(f'{name}_{j}', [(cx - 0.07, top), (cx + 0.07, top), (cx, top + 0.12)],
                               z - 0.035, z + 0.035, M.bread, axis='z')
                    F.box(f'{name}_fill', 0.075, 0.014, 0.072, (cx, top + 0.05, -0.06), M.filling)
                    F.box(f'{name}_sticker', 0.05, 0.03, 0.074, (cx, top + 0.022, -0.06), c)
                elif k % 3 == 0:  # tins, two deep
                    w = 0.1
                    for j, z in enumerate((-0.28, -0.15, -0.02)):
                        F.cyl(f'{name}_{j}', 0.045, 0.12, (x + w / 2, top + 0.06, z), c, verts=10)
                        F.cyl(f'{name}_lid{j}', 0.04, 0.006, (x + w / 2, top + 0.123, z), M.tinLid, verts=10)
                elif k % 5 == 0:  # jars
                    w = 0.11
                    for j, z in enumerate((-0.2, -0.02)):
                        F.cyl(f'{name}_{j}', 0.045, 0.14, (x + w / 2, top + 0.07, z), c, verts=10)
                        F.cyl(f'{name}_lid{j}', 0.047, 0.03, (x + w / 2, top + 0.155, z), M.tinLid, verts=10)
                else:             # packets
                    w = min(0.13 + (k % 3) * 0.03, 0.92 - x)
                    h = 0.18 + (k % 4) * 0.04
                    F.box(name, w - 0.02, h, 0.3, (x + w / 2, top + h / 2, -0.15), c, bevel=0.006, bevel_segments=1)
                    F.box(f'{name}_label', w - 0.06, h * 0.3, 0.004, (x + w / 2, top + h * 0.55, 0.002), M.label)
                x += w + 0.015
    G.join(still, 'shelves', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the café table: 1x1, a round marble top on a cast-iron tripod foot -----------------------------
def cafeTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('top', 0.31, 0.03, (0, TABLE_TOP - 0.015, 0), M.marble, verts=36, bevel=0.008)
        F.cyl('collar', 0.07, 0.03, (0, TABLE_TOP - 0.045, 0), M.iron, verts=16, bevel=0.004)
        for i in range(3):
            a = math.radians(i * 120 + 60)
            G.rod(f'spider{i}', F.at(0.05 * math.sin(a), TABLE_TOP - 0.05, 0.05 * math.cos(a)),
                  F.at(0.2 * math.sin(a), TABLE_TOP - 0.032, 0.2 * math.cos(a)), 0.01, M.iron, verts=8)
        F.cyl('column', 0.024, 0.56, (0, 0.40, 0), M.iron, verts=12)
        F.sphere('knop', 0.04, (0, 0.34, 0), M.iron, scale=(1, 1, 1.3), segments=12, rings=8)
        F.cone('hub', 0.07, 0.035, 0.1, (0, 0.13, 0), M.iron, verts=16)
        for i in range(3):
            a = math.radians(i * 120)
            pts = [(r * math.sin(a), y, r * math.cos(a)) for r, y in
                   ((0.04, 0.16), (0.1, 0.13), (0.17, 0.07), (0.23, 0.03), (0.265, 0.018))]
            _tube(F, f'leg{i}', pts, 0.018, M.iron)
            F.cyl(f'foot{i}', 0.03, 0.018, (0.27 * math.sin(a), 0.009, 0.27 * math.cos(a)), M.iron, verts=12)
    G.join(still, 'table', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the café chair: 1x1, a bentwood chair with a cane seat, the sitter facing the front ------------
def cafeChair():
    F = G.GameFrame()
    C = SEAT_FORWARD   # the seat's centre, 2 cm forward
    with G.new_parts() as still:
        F.cyl('seat', 0.205, 0.035, (0, SEAT - 0.04, C), M.bentwood, verts=28)
        F.cyl('cane', 0.188, 0.03, (0, SEAT - 0.017, C), M.cane, verts=28)
        G.torus('seat_rim', 0.196, 0.012, F.at(0, SEAT - 0.012, C), M.bentwood, major_segments=28, minor_segments=6)
        # the front legs, splaying a little, and the ring that braces all four
        for s in (-1, 1):
            G.rod(f'leg{s}', F.at(s * 0.13, SEAT - 0.05, C + 0.13), F.at(s * 0.145, 0.0, C + 0.16), 0.016,
                  M.bentwood, verts=10)
        G.torus('leg_ring', 0.212, 0.011, F.at(0, 0.22, -0.0124), M.bentwood, major_segments=28, minor_segments=6)
        # one bent piece: the back legs rising past the seat into the bow of the back
        side = [(0.152, 0.0, -0.192), (0.15, 0.22, -0.178), (0.152, 0.44, -0.165), (0.148, 0.66, -0.19),
                (0.14, 0.82, -0.215), (0.09, 0.915, -0.226)]
        bow = [(-x, y, z) for x, y, z in side] + [(0.0, 0.935, -0.23)] + list(reversed(side))
        _tube(F, 'bow', bow, 0.016, M.bentwood)
        # the inner loop of the back
        side = [(0.075, SEAT - 0.01, -0.17), (0.08, 0.6, -0.186), (0.065, 0.71, -0.2)]
        loop = [(-x, y, z) for x, y, z in side] + [(0.0, 0.75, -0.206)] + list(reversed(side))
        _tube(F, 'loop', loop, 0.012, M.bentwood)
    G.join(still, 'chair', origin=(0, 0, 0))
    G.settle(centre=False)


_FAR = 0.25   # placed on town lots, seen from afar: a quarter of the triangles (E7)

FAMILY = G.Family(
    'shops', M,
    [G.Model('shopCounter', shopCounter, size=(1.96, 1.276, 0.78), key='model.shopCounter', centre=(0, -0.04),
             # buying a little something pops the till's drawer
             motions={'uses': {'treat': 'till'}, 'hold': TILL_HOLD},
             paint=[('counter', 'Counter'), ('walnut', 'Trim')], far=_FAR),
     G.Model('bakeryCase', bakeryCase, size=(1.94, 1.29, 0.86), key='model.bakeryCase', centre=(0, 0.0),
             # buying a pastry slides the back door aside
             motions={'uses': {'pastry': 'door'}, 'hold': DOOR_HOLD},
             paint=[('case', 'Counter'), ('rim', 'Trim')], far=_FAR),
     G.Model('groceryShelf', groceryShelf, size=(1.92, 1.72, 0.48), key='model.groceryShelf', centre=(0, -0.19),
             paint=[('shelf', 'Shelves'), ('trim', 'Trim')], far=_FAR),
     G.Model('cafeTable', cafeTable, size=(0.62, 0.745, 0.62), key='model.cafeTable',
             paint=[('marble', 'Top'), ('iron', 'Frame')], far=_FAR),
     G.Model('cafeChair', cafeChair, size=(0.446, 0.952, 0.472), key='model.cafeChair', centre=(0, -0.008),
             paint=[('bentwood', 'Frame'), ('cane', 'Seat')], far=_FAR)],
    comment='Blender-built shop fittings and café furniture for the town (tools/art/blender/families/shops.py). '
            'Replaces the primitive-part placeholders of tools/art/community.py by key; listed in the base '
            'manifest\'s packs. The placeholder is kept as a fallback if a file fails to load. Each has a far '
            'version for the town overview. The till\'s cash drawer slides out (`till`) while a resident buys '
            'a treat; the bakery case\'s back glass door slides aside (`door`) for a pastry.',
)

if __name__ == '__main__':
    _content_facts()
    FAMILY.run(sys.argv)
