# The things residents carry and cook with: the models of content `items` (`item.*`: a bag of shopping, a
# prep bowl, a pot of stew, plates of food, dirty plates and pans) and the props drawn on a station's top
# while a resident cooks there (`tool.*`: the chopping board, the pan, food on the grill). Notes: held.md.
#
# Each is drawn as the old part was (characters.ts `writeCarried`, `writeTool`; items.ts): its origin at
# the middle of its base, front +Z in the game, lifted onto the palms, the top it stands on or the station.
# So each is built where the game places it, in the game's axes (`F`), and stood on the floor
# without moving it sideways (`done`); the few that stand off their middle (the pan's handle, the bag)
# take their measured `centre`.
#
# Small and seen several at once: one draw call each (every finish a plain one), two for the wooden boards
# (the chopping board, the tasting board, the pizza's board: `wood`), and 210 to 510 triangles.
#
# Run:  cargo art held [--only mealPot] [--preview]
#       HELD_MEASURE=1 … --preview   prints where each is built and its size, to copy into its `I(...)` line
import math
import os
import sys

import bmesh
import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # dishes and pans
    china=('#F1EEE6', 'gloss'),
    china_well=('#E4DFD4', 'gloss'),
    bowl=('#E9E4D8', 'gloss'),
    dish=('#E6E0D2', 'gloss'),
    tray=('#A9AEB4', 'metal'),
    iron=('#3A3D42', 'metal'),
    iron_inside=('#7C8188', 'metal'),
    steel=('#C9CED4', 'metal'),
    grip=('#2B2D31', 'satin'),
    board=('#B88A57', 'wood'),
    board_dark=('#9A6A3E', 'wood'),
    peel=('#A8784A', 'wood'),
    kraft=('#C49A6C', 'matte'),
    kraft_fold=('#A9804F', 'matte'),
    kraft_inside=('#6E5236', 'matte'),
    # food
    lettuce=('#5FA33A', 'satin'),
    leaf=('#3F7F2A', 'satin'),
    carrot=('#E8772E', 'satin'),
    tomato=('#D93A2B', 'gloss'),
    potato=('#E8D9A0', 'matte'),
    pea=('#6BA33A', 'satin'),
    stew=('#B0562C', 'satin'),
    bread=('#E9CD98', 'matte'),
    crust=('#B9874E', 'matte'),
    cheese=('#E9C46A', 'satin'),
    cheese_cut=('#F2D88E', 'satin'),
    salami=('#B83A2C', 'satin'),
    grape=('#6E9A3E', 'gloss'),
    cracker=('#D9B26A', 'matte'),
    sausage=('#8A4A2A', 'satin'),
    sausage_b=('#93512E', 'satin'),
    char=('#3E2414', 'matte'),
    patty=('#5E3A24', 'satin'),
    corn=('#F2C230', 'satin'),
    cob=('#E8DDB0', 'matte'),
    dough=('#E2B064', 'matte'),
    dough_baked=('#C98E4A', 'matte'),
    sauce=('#C8452E', 'satin'),
    mozzarella=('#F3E3B0', 'satin'),
    pepperoni=('#9C3024', 'satin'),
    bun=('#D69A55', 'satin'),
    bun_dark=('#B5743A', 'satin'),
    bun_base=('#E8C48A', 'matte'),
    pastry=('#D8A257', 'matte'),
    pastry_baked=('#C08040', 'matte'),
    filling=('#8E2433', 'gloss'),
    cream=('#FBF6EA', 'satin'),
    cookie=('#C98A45', 'matte'),
    chocolate=('#4A2C1C', 'satin'),
    cupcake_case=('#E58FA8', 'satin'),
    icing=('#F8E4EA', 'satin'),
    mixing=('#D9DCD6', 'gloss'),     # the mixing bowl's glaze
    batter=('#EBCB8C', 'satin'),
    spoon=('#C49A62', 'satin'),      # a wooden spoon (plain, so the bowl stays one draw)
    # leftovers
    smear=('#64452B', 'matte'),
    smear_sauce=('#94401F', 'matte'),
    grease=('#6E5338', 'matte'),
    burnt=('#3F3022', 'matte'),
    residue=('#6B4A30', 'matte'),
    crumb=('#D9B98A', 'matte'),
    melt=('#E9C9B4', 'matte'),       # melted ice cream in the bowl
    smear_filling=('#6E2A2E', 'matte'),
    # the quick foods (review M-held-01): ice cream, toast, a smoothie, noodles, takeout
    vanilla=('#F6EBC8', 'satin'),
    strawberry=('#F0A3B2', 'satin'),
    choc_ice=('#7A4E34', 'satin'),
    toast=('#D39A55', 'matte'),
    toast_crust=('#94602F', 'matte'),
    butter=('#F6E08E', 'satin'),
    glass=('#DDE8EA', 'gloss'),      # a tumbler's glass, opaque (plain: one draw)
    smoothie=('#C85A86', 'satin'),   # berry, seen through the glass
    smoothie_top=('#D97AA0', 'satin'),
    straw=('#F2F0EA', 'satin'),
    paper=('#F4F2EC', 'matte'),      # the noodle pail's white card
    paper_red=('#C8402E', 'matte'),
    noodle=('#E7C97E', 'satin'),
    chopstick=('#D8B887', 'satin'),
    fries=('#E9BF55', 'satin'),
)

# Built in the game's axes (x, height, depth; front +depth), as the old parts are.
F = G.GameFrame()


# --- helpers ------------------------------------------------------------------------------------------------
def lathe(name, profile, mats, at=(0, 0, 0), segments=16, scale=(1, 1), rot=(0, 0, 0), sharp=40):
    """A closed surface of revolution about the game's up axis: `profile` [(radius, height)] from the
    bottom's middle (radius 0) round to the top's (radius 0), each end a pole (no doubled points at the
    axis). `mats`: one material, or one per band between profile points. `scale`: (game x, depth) to make
    it oval. Edges sharper than `sharp` degrees are kept sharp (a rim, a lip); the rest is smooth."""
    mats = mats if isinstance(mats, (list, tuple)) else [mats] * (len(profile) - 1)
    assert len(mats) == len(profile) - 1, f'{name}: {len(mats)} materials for {len(profile) - 1} bands'
    bm = bmesh.new()
    rings = []
    for r, h in profile:
        if r < 1e-6:
            rings.append([bm.verts.new((0, 0, h))])
        else:
            rings.append([bm.verts.new((r * math.cos(2 * math.pi * i / segments) * scale[0],
                                        r * math.sin(2 * math.pi * i / segments) * scale[1], h))
                          for i in range(segments)])
    order = []
    for m in mats:
        if m not in order:
            order.append(m)
    for band, (a, b) in enumerate(zip(rings, rings[1:])):
        for i in range(segments):
            j = (i + 1) % segments
            if len(a) == 1:
                quad = (a[0], b[j], b[i])
            elif len(b) == 1:
                quad = (a[i], a[j], b[0])
            else:
                quad = (a[i], a[j], b[j], b[i])
            f = bm.faces.new(quad)
            f.material_index = order.index(mats[band])
            f.smooth = True
    bm.normal_update()
    for e in bm.edges:
        if len(e.link_faces) == 2 and e.link_faces[0].normal.angle(e.link_faces[1].normal, 0) > math.radians(sharp):
            e.smooth = False
    return _object(name, bm, order, F.at(*at), rot)


def prism(name, points, y0, y1, top, side, bottom=None, sides=None, at=(0, 0)):
    """A flat-shaded upright prism over a polygon of game (x, depth) points, from height `y0` to `y1`:
    `top` on its top face, `side` (or `sides`, one per edge from point i to i + 1) round it."""
    bm = bmesh.new()
    lo = [bm.verts.new(F.at(x + at[0], y0, z + at[1])) for x, z in points]
    hi = [bm.verts.new(F.at(x + at[0], y1, z + at[1])) for x, z in points]
    mats = [top, bottom or side] + list(sides or [side] * len(points))
    order = []
    for m in mats:
        if m not in order:
            order.append(m)
    bm.faces.new(hi).material_index = order.index(top)
    bm.faces.new(list(reversed(lo))).material_index = order.index(bottom or side)
    n = len(points)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((lo[i], lo[j], hi[j], hi[i])).material_index = order.index(mats[2 + i])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return _object(name, bm, order, (0, 0, 0), (0, 0, 0))


def _object(name, bm, materials, location, rot):
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in materials:
        obj.data.materials.append(m)
    obj.location = location
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


def box(name, w, h, d, at, material, yaw=0.0, tilt=(0, 0), bevel=0.0):
    """A box of game width, height and depth centred at a game point, turned `yaw` degrees about the up
    axis (and tilted (about game x, about game depth))."""
    return G.cube(name, (w, d, h), at=F.at(*at), material=material, rot=(tilt[0], tilt[1], yaw),
                  bevel=bevel, bevel_segments=1)


def disc(name, d, h, at, material, verts=10, depth=None, yaw=0.0, tilt=(0, 0)):
    """An upright disc `d` across (`depth` deep, if oval) and `h` tall, its base at the game point `at`."""
    return G.cylinder(name, d / 2, h, at=F.at(at[0], at[1] + h / 2, at[2]), material=material, verts=verts,
                      scale=(1, (depth or d) / d, 1), rot=(tilt[0], tilt[1], yaw), smooth=False)


def lump(name, r, at, material, scale=(1, 1, 1), seed=0.0, subdivisions=1, noise=0.25):
    """A lumpy bit of food (an icosphere shaken by noise), its base at the game point `at`; `scale` in
    the game's (x, height, depth)."""
    return G.boulder(name, r, at=F.at(*at), material=material, scale=(scale[0], scale[2], scale[1]), seed=seed,
                     subdivisions=subdivisions, noise=noise)


def ball(name, r, at, material, scale=(1, 1, 1), seg=8, rings=5):
    """A low sphere centred at the game point `at`; `scale` in the game's (x, height, depth)."""
    return G.sphere(name, r, at=F.at(*at), material=material, scale=(scale[0], scale[2], scale[1]),
                    segments=seg, rings=rings)


def chunk(name, s, at, material, yaw=0.0, tilt=(0, 0)):
    """A chopped cube of food `s` across, centred at the game point `at`."""
    return box(name, s, s * 0.8, s, at, material, yaw=yaw, tilt=tilt)


def roll(name, r, length, mats, at, yaw=0.0, segments=8, marks=()):
    """A sausage (a capsule `r` thick, `length` long) lying along the game's x axis turned `yaw` degrees,
    centred at the game point `at`. `marks`: the (from, to) stretches along it, from -1 to 1, that take
    the second material (grill marks)."""
    body, mark = mats if isinstance(mats, (list, tuple)) else (mats, mats)
    half = length / 2
    cap = r * 0.75
    stops = sorted({-1.0, 1.0, *[t for m in marks for t in m]})
    profile = [(0, -half), (r * 0.7, -half + r * 0.3)] + [(r, t * (half - cap)) for t in stops] + \
              [(r * 0.7, half - r * 0.3), (0, half)]
    bands = [body, body] + [mark if any(m[0] <= a and b <= m[1] for m in marks) else body
                            for a, b in zip(stops, stops[1:])] + [body, body]
    return lathe(name, profile, bands, at=at, segments=segments, rot=(0, 90, -yaw))


def done(name):
    """A still model: everything joined into one node and stood on the floor, where it was built."""
    obj = G.join_all(name)
    if os.environ.get('HELD_MEASURE'):
        lo, hi = G.measure([obj])
        print(f'[gamekit] {name} built centred on game ({-(lo[0] + hi[0]) / 2:.4f}, {-(lo[1] + hi[1]) / 2:.4f}), '
              f'size {hi[0] - lo[0]:.3f} x {hi[2] - lo[2]:.3f} x {hi[1] - lo[1]:.3f}, bottom {lo[2]:.4f}')
        return G.settle([obj])
    return G.settle([obj], centre=False)


# --- the plate everything is served on ----------------------------------------------------------------------
WELL = 0.009  # the height of the plate's well, where the food sits


def plate():
    """A dinner plate 29 cm across: a foot, a rim rising to 1.7 cm, a well at 0.9 cm in a paler glaze."""
    lathe('plate', [(0, 0), (0.1, 0), (0.144, 0.0155), (0.128, 0.0172), (0.086, WELL), (0, WELL)],
          [M.china, M.china, M.china, M.china, M.china_well], segments=20)


# --- items: carried and set down ------------------------------------------------------------------------------
def ingredients():
    """A paper bag of shopping: a lettuce, a carrot with its tops and a tomato coming out of the top."""
    box('bag', 0.19, 0.2, 0.13, (0, 0.1, 0), M.kraft)
    box('cuff', 0.2, 0.026, 0.14, (0, 0.19, 0), M.kraft_fold)
    box('inside', 0.188, 0.002, 0.128, (0, 0.2025, 0), M.kraft_inside)
    lump('lettuce', 0.058, (0.022, 0.15, -0.005), M.lettuce, scale=(1.0, 0.95, 0.9), seed=1.7, subdivisions=2,
         noise=0.18)
    G.cone('carrot', 0.004, 0.017, 0.17, at=F.at(-0.055, 0.215, 0.02), material=M.carrot, rot=(0, -18, 0), verts=8)
    # its tops, out of its wide end (turned 18 degrees, 8.5 cm up from its middle)
    tx, ty = -0.055 + 0.085 * math.sin(math.radians(18)), 0.215 + 0.085 * math.cos(math.radians(18))
    for i, (lean, turn) in enumerate(((-20, 0), (25, 40), (5, -50))):
        G.cone(f'carrot_top{i}', 0.007, 0.001, 0.045, material=M.leaf, rot=(0, -18 - lean, turn), verts=4,
               at=F.at(tx + 0.02 * math.sin(math.radians(18 + lean)), ty + 0.018, 0.02))
    ball('tomato', 0.035, (0.06, 0.222, 0.035), M.tomato, scale=(1, 0.85, 1), seg=10, rings=6)
    G.cone('tomato_stalk', 0.012, 0.0, 0.008, at=F.at(0.06, 0.255, 0.035), material=M.leaf, verts=5)
    return done('ingredients')


def prepBowl():
    """A wide glazed bowl of chopped vegetables: diced potato with carrot and green beans over it."""
    lathe('bowl', [(0, 0), (0.095, 0), (0.12, 0.012), (0.162, 0.09), (0.153, 0.0925), (0.146, 0.078), (0, 0.084)],
          [M.bowl, M.bowl, M.bowl, M.bowl, M.bowl, M.potato], segments=18)
    for i, (x, z, m, yaw) in enumerate(((0.03, 0.02, M.carrot, 10), (-0.05, -0.02, M.carrot, 40),
                                        (0.06, -0.06, M.carrot, 70), (-0.02, 0.07, M.carrot, 25),
                                        (-0.04, 0.03, M.pea, 15), (0.0, -0.05, M.pea, 55),
                                        (0.07, 0.04, M.pea, 80), (-0.08, -0.06, M.potato, 30),
                                        (0.01, 0.0, M.potato, 5))):
        chunk(f'bit{i}', 0.024, (x, 0.09, z), m, yaw=yaw, tilt=(12 * (i % 3 - 1), 9 * (i % 2)))
    return done('prepBowl')


def _pot(dirty):
    """The stew pot: dark iron with a rolled rim and a loop handle each side; full of stew, or scraped
    empty with a tidemark and burnt bits."""
    if dirty:
        lathe('pot', [(0, 0), (0.13, 0), (0.135, 0.008), (0.135, 0.15), (0.127, 0.15), (0.127, 0.125),
                      (0.127, 0.113), (0.127, 0.03), (0, 0.034)],
              [M.iron, M.iron, M.iron, M.iron, M.iron_inside, M.residue, M.iron_inside, M.residue], segments=18)
        lump('burnt0', 0.022, (0.04, 0.03, 0.03), M.burnt, scale=(1.2, 0.4, 1), seed=2.0)
        lump('burnt1', 0.016, (-0.05, 0.03, -0.02), M.residue, scale=(1.3, 0.35, 1), seed=4.0)
        lump('burnt2', 0.012, (-0.01, 0.03, 0.07), M.burnt, scale=(1.4, 0.35, 1), seed=6.0)
    else:
        lathe('pot', [(0, 0), (0.13, 0), (0.135, 0.008), (0.135, 0.15), (0.127, 0.15), (0.127, 0.135), (0, 0.139)],
              [M.iron, M.iron, M.iron, M.iron, M.iron_inside, M.stew], segments=18)
        for i, (x, z, m, yaw) in enumerate(((0.04, 0.03, M.carrot, 20), (-0.05, -0.02, M.carrot, 60),
                                            (0.07, -0.05, M.carrot, 5), (-0.02, 0.06, M.potato, 35),
                                            (0.0, -0.06, M.potato, 70), (-0.07, 0.04, M.pea, 15),
                                            (0.03, -0.01, M.pea, 50))):
            chunk(f'bit{i}', 0.022, (x, 0.142, z), m, yaw=yaw, tilt=(10 * (i % 3 - 1), 8 * (i % 2)))
    for s in (-1, 1):
        box(f'lug{s}', 0.012, 0.03, 0.05, (s * 0.141, 0.122, 0), M.iron)
        box(f'loop{s}', 0.012, 0.01, 0.07, (s * 0.189, 0.132, 0), M.iron)
        for t in (-1, 1):
            box(f'arm{s}{t}', 0.05, 0.01, 0.01, (s * 0.165, 0.132, t * 0.03), M.iron)


def mealPot():
    _pot(False)
    return done('mealPot')


def potDirty():
    _pot(True)
    return done('potDirty')


def tastingBoard():
    """A serving board with a tab handle: a wedge of cheese, slices of salami, crackers and grapes."""
    box('board', 0.35, 0.028, 0.24, (-0.025, 0.014, 0), M.board_dark, bevel=0.005)
    box('tab', 0.052, 0.028, 0.08, (0.174, 0.014, 0), M.board_dark, bevel=0.005)
    prism('cheese', [(-0.17, -0.07), (-0.07, -0.035), (-0.165, 0.055)], 0.028, 0.07, M.cheese_cut, M.cheese_cut,
          sides=[M.cheese_cut, M.cheese, M.cheese_cut])
    for i, (x, z, t) in enumerate(((-0.03, -0.05, 0), (0.0, -0.035, 8), (0.03, -0.02, 14), (-0.015, 0.03, 4))):
        disc(f'salami{i}', 0.05, 0.006, (x, 0.028 + 0.004 * (i % 3), z), M.salami, verts=8, tilt=(t, 0))
    for i, (x, z, yaw) in enumerate(((-0.09, 0.07, 10), (-0.06, 0.075, 25), (-0.035, 0.08, 40))):
        box(f'cracker{i}', 0.042, 0.005, 0.042, (x, 0.031 + 0.004 * i, z), M.cracker, yaw=yaw)
    for i, (x, y, z) in enumerate(((0.09, 0.042, -0.02), (0.115, 0.042, -0.035), (0.11, 0.042, 0.0),
                                   (0.135, 0.042, -0.015), (0.105, 0.06, -0.017))):
        ball(f'grape{i}', 0.015, (x, y, z), M.grape, scale=(1, 1, 0.92), seg=6, rings=4)
    G.rod('stalk', F.at(0.1, 0.07, -0.01), F.at(0.135, 0.062, -0.03), 0.003, material=M.leaf, verts=4)
    return done('tastingBoard')


def _platter():
    """An oval serving platter, 40 by 28 cm."""
    lathe('platter', [(0, 0), (0.15, 0), (0.2, 0.017), (0.188, 0.02), (0.15, 0.01), (0, 0.01)],
          [M.china, M.china, M.china, M.china, M.china_well], segments=20, scale=(1, 0.7))


def grillPlatter():
    """The platter from the grill: two sausages with grill marks, a burger and a cob of corn."""
    _platter()
    for i, (z, yaw, m) in enumerate(((-0.04, 12, M.sausage), (0.035, -8, M.sausage_b))):
        roll(f'sausage{i}', 0.025, 0.16, (m, M.char), (-0.08, 0.01 + 0.025, z), yaw=yaw,
             marks=((-0.1, 0.1),))
    disc('patty', 0.1, 0.028, (0.075, 0.01, 0.035), M.patty, verts=12)
    for i, x in enumerate((0.055, 0.09)):
        box(f'mark{i}', 0.008, 0.002, 0.085, (x, 0.039, 0.035), M.char, yaw=20)
    roll('corn', 0.022, 0.13, M.corn, (0.075, 0.01 + 0.022, -0.075), yaw=-12, segments=7)
    return done('grillPlatter')


def platterDirty():
    """The platter after the barbecue: a smear of sauce and grease, crumbs and a gnawed corn cob."""
    _platter()
    disc('smear', 0.24, 0.002, (0.02, 0.01, 0), M.smear, verts=12, depth=0.15, yaw=8)
    disc('sauce', 0.09, 0.002, (-0.06, 0.0115, 0.03), M.smear_sauce, verts=10, depth=0.04, yaw=-20)
    roll('cob', 0.012, 0.11, M.cob, (0.06, 0.01 + 0.012, -0.04), yaw=20, segments=6)
    for i, (x, z) in enumerate(((-0.02, -0.05), (0.09, 0.04), (-0.1, -0.02), (0.02, 0.06))):
        chunk(f'crumb{i}', 0.008, (x, 0.0135, z), M.crumb, yaw=30 * i)
    return done('platterDirty')


def pizza():
    """A whole pizza on its round board: a golden raised crust, tomato, melted cheese, pepperoni and
    basil."""
    lathe('board', [(0, 0), (0.2, 0), (0.2, 0.02), (0, 0.02)], M.peel, segments=20)
    lathe('pizza', [(0, 0.02), (0.17, 0.02), (0.172, 0.038), (0.16, 0.052), (0.148, 0.042), (0.14, 0.039),
                    (0, 0.0405)],
          [M.dough, M.dough, M.dough_baked, M.dough, M.sauce, M.mozzarella], segments=20)
    for i, (x, z) in enumerate(((0.06, 0.05), (-0.07, 0.03), (0.0, -0.085), (-0.04, -0.04), (0.08, -0.04),
                                (-0.02, 0.095))):
        disc(f'pepperoni{i}', 0.042, 0.004, (x, 0.04, z), M.pepperoni, verts=8)
    for i, (x, z, yaw) in enumerate(((0.02, 0.0, 20), (-0.09, -0.06, 70), (0.1, 0.03, 40), (0.03, -0.1, 10),
                                     (-0.06, 0.09, 60))):
        box(f'basil{i}', 0.022, 0.003, 0.013, (x, 0.0425, z), M.leaf, yaw=yaw)
    return done('pizza')


def _tray():
    """A steel baking tray 38 by 28 cm with a 2.5 cm rim."""
    box('tray', 0.38, 0.006, 0.28, (0, 0.003, 0), M.tray)
    for s in (-1, 1):
        box(f'rim_x{s}', 0.38, 0.025, 0.008, (0, 0.0125, s * 0.136), M.tray)
        box(f'rim_z{s}', 0.008, 0.025, 0.264, (s * 0.186, 0.0125, 0), M.tray)


def bakingTray():
    """A tray of six bread rolls, golden on top, paler round their feet."""
    _tray()
    for i, (x, z) in enumerate(((-0.11, -0.07), (0, -0.07), (0.11, -0.07), (-0.11, 0.07), (0, 0.07), (0.11, 0.07))):
        top = M.bun if i % 2 else M.bun_dark
        lathe(f'roll{i}', [(0, 0), (0.039, 0), (0.038, 0.014), (0.027, 0.034), (0, 0.04)],
              [M.bun_base, M.bun_base, top, top], at=(x, 0.006, z), segments=10, scale=(1, 0.92))
    return done('bakingTray')


def trayDirty():
    """The tray after baking: a greasy stain with burnt patches and crumbs."""
    _tray()
    disc('grease', 0.22, 0.0015, (0.0, 0.006, 0.0), M.grease, verts=12, depth=0.15, yaw=5)
    for i, (x, z, d) in enumerate(((-0.07, -0.025, 0.035), (0.06, 0.03, 0.028), (0.02, -0.045, 0.022))):
        disc(f'burnt{i}', d, 0.0015, (x, 0.0072, z), M.burnt, verts=8, depth=d * 0.7, yaw=30 * i)
    for i, (x, z) in enumerate(((-0.1, 0.06), (0.11, -0.05), (0.04, 0.08), (-0.03, -0.09))):
        chunk(f'crumb{i}', 0.008, (x, 0.0095, z), M.crumb, yaw=25 * i)
    return done('trayDirty')


def pieDish():
    """A cherry pie in a glazed dish: a crimped rim of pastry and a lattice over the filling."""
    lathe('dish', [(0, 0), (0.135, 0), (0.166, 0.057), (0.168, 0.061), (0.158, 0.063), (0.148, 0.072), (0, 0.075)],
          [M.dish, M.dish, M.dish, M.dish, M.pastry, M.filling], segments=20)
    G.torus('rim', 0.148, 0.011, at=F.at(0, 0.073, 0), material=M.pastry_baked, major_segments=16, minor_segments=4)
    for i, o in enumerate((-0.07, 0, 0.07)):
        length = 2 * math.sqrt(0.148 ** 2 - o ** 2)
        box(f'strip_x{i}', length, 0.005, 0.022, (0, 0.0775, o), M.pastry)
        box(f'strip_z{i}', 0.022, 0.005, length, (o, 0.0805, 0), M.pastry)
    return done('pieDish')


def mealPlate():
    """A plate of stew with mashed potato and green beans."""
    plate()
    lump('stew', 0.054, (0.025, WELL - 0.004, 0.0), M.stew, scale=(1.1, 0.42, 0.95), seed=3.1, subdivisions=2,
         noise=0.2)
    lump('mash', 0.034, (-0.055, WELL - 0.003, -0.04), M.potato, scale=(1, 0.8, 1), seed=5.3, subdivisions=2,
         noise=0.2)
    for i, (x, z, m) in enumerate(((0.05, 0.03, M.carrot), (0.0, 0.0, M.carrot), (0.03, -0.03, M.potato))):
        chunk(f'bit{i}', 0.016, (x, 0.04, z), m, yaw=35 * i, tilt=(15, -10 * i))
    for i, (x, z, yaw) in enumerate(((-0.05, 0.05, 15), (-0.04, 0.065, 5), (-0.065, 0.04, 30))):
        box(f'bean{i}', 0.075, 0.01, 0.011, (x, WELL + 0.005 + 0.003 * i, z), M.pea, yaw=yaw)
    return done('mealPlate')


def _wedge(radius, half_angle, apex, steps=3):
    """The (x, depth) points of a wedge (a slice of something round) with its tip at `apex` (x, depth)
    pointing to the front, its round edge to the back."""
    pts = [apex]
    for k in range(steps + 1):
        a = math.radians(-half_angle + 2 * half_angle * k / steps)
        pts.append((apex[0] + radius * math.sin(a), apex[1] - radius * math.cos(a)))
    return pts


def _shrunk(pts, by):
    """`pts` drawn in towards their middle by a share."""
    cx = sum(p[0] for p in pts) / len(pts)
    cz = sum(p[1] for p in pts) / len(pts)
    return [(cx + (x - cx) * (1 - by), cz + (z - cz) * (1 - by)) for x, z in pts]


def _arc_rods(name, radius, half_angle, apex, y, r, material, steps=3):
    pts = _wedge(radius, half_angle, apex, steps)[1:]
    for k, (a, b) in enumerate(zip(pts, pts[1:])):
        G.rod(f'{name}{k}', F.at(a[0], y, a[1]), F.at(b[0], y, b[1]), r, material=material, verts=6)
        if k:
            ball(f'{name}_joint{k}', r, (a[0], y, a[1]), material, seg=6, rings=4)


def pizzaSlice():
    """A plate with a slice of pizza: the base, tomato showing at its cut sides, cheese, pepperoni, and its
    crust along the back."""
    plate()
    apex, R, A = (0.0, 0.075), 0.15, 21
    wedge = _wedge(R, A, apex)
    prism('base', wedge, WELL, WELL + 0.006, M.dough, M.dough)
    prism('topping', _shrunk(wedge, 0.04), WELL + 0.006, WELL + 0.012, M.mozzarella, M.sauce)
    _arc_rods('crust', R - 0.006, A, apex, WELL + 0.011, 0.011, M.dough_baked)
    for i, (x, z) in enumerate(((0.0, 0.0), (-0.025, -0.045), (0.028, -0.04))):
        disc(f'pepperoni{i}', 0.03, 0.003, (x, WELL + 0.012, z), M.pepperoni, verts=8)
    box('basil', 0.016, 0.002, 0.01, (0.005, WELL + 0.0135, 0.035), M.leaf, yaw=30)
    return done('pizzaSlice')


def treatPlate():
    """A plate of treats: chocolate-chip cookies, one leaning on the others, and an iced cupcake with a
    cherry."""
    plate()
    cookies = (((-0.042, WELL, 0.01), (0, 0)), ((0.04, WELL, 0.025), (0, 0)), ((0.0, WELL + 0.012, -0.012), (9, -6)))
    for i, ((x, y, z), tilt) in enumerate(cookies):
        disc(f'cookie{i}', 0.076, 0.013, (x, y, z), M.cookie, verts=10, tilt=tilt)
        for k, (dx, dz) in enumerate(((-0.015, -0.008), (0.012, 0.012))):
            lift = (dz * math.tan(math.radians(tilt[0])) - dx * math.tan(math.radians(tilt[1]))) if i == 2 else 0
            chunk(f'chip{i}{k}', 0.009, (x + dx, y + 0.013 + lift, z + dz), M.chocolate, yaw=20 * k)
    lathe('cupcake', [(0, 0), (0.022, 0), (0.028, 0.025), (0.031, 0.027), (0.027, 0.038), (0.016, 0.047), (0, 0.05)],
          [M.cupcake_case, M.cupcake_case, M.icing, M.icing, M.icing, M.icing], at=(0.058, WELL, -0.066), segments=10)
    ball('cherry', 0.0075, (0.058, WELL + 0.056, -0.066), M.tomato, seg=6, rings=4)
    return done('treatPlate')


def pieSlice():
    """A plate with a slice of cherry pie: pastry under and over the red filling, its crimped edge along the
    back, and a spoonful of cream beside it."""
    plate()
    apex, R, A = (-0.015, 0.06), 0.125, 24
    wedge = _wedge(R, A, apex)
    prism('pastry_base', wedge, WELL, WELL + 0.008, M.pastry, M.pastry_baked)
    prism('filling', _shrunk(wedge, 0.03), WELL + 0.008, WELL + 0.036, M.filling, M.filling)
    prism('pastry_top', wedge, WELL + 0.036, WELL + 0.043, M.pastry, M.pastry,
          sides=[M.pastry_baked] + [M.pastry] * (len(wedge) - 1))
    _arc_rods('edge', R - 0.008, A, apex, WELL + 0.039, 0.011, M.pastry_baked)
    for i, (x, z) in enumerate(((-0.015, -0.01), (-0.015, -0.04))):
        box(f'slit{i}', 0.022, 0.002, 0.004, (x, WELL + 0.0435, z), M.filling)
    lump('cream', 0.022, (0.07, WELL - 0.002, 0.035), M.cream, scale=(1, 0.9, 1), seed=7.0, subdivisions=2,
         noise=0.15)
    return done('pieSlice')


def sandwich():
    """A sandwich cut corner to corner: two halves of bread, lettuce, cheese and tomato, the crust round
    the outside and the crumb at the cut."""
    plate()
    a = [(-0.058, -0.052), (0.058, -0.052), (-0.058, 0.052)]
    b = [(0.058, -0.052), (0.058, 0.052), (-0.058, 0.052)]
    # each half drawn back from the cut a little; the cut is the side from point 1 to 2 (a) or 2 to 0 (b)
    nx, nz = 0.052, 0.058
    n = math.hypot(nx, nz)
    for half, pts, cut, sign in (('a', a, 1, -1), ('b', b, 2, 1)):
        off = (sign * nx / n * 0.008, sign * nz / n * 0.008)
        y = WELL
        for layer, h, m, grow in (('bread_lo', 0.019, M.bread, 0), ('lettuce', 0.006, M.lettuce, 0.08),
                                  ('cheese', 0.004, M.cheese, 0.05), ('tomato', 0.007, M.tomato, -0.04),
                                  ('bread_hi', 0.019, M.bread, 0)):
            shape = _shrunk(pts, -grow)
            if m is M.bread:
                sides = [M.bread if k == cut else M.crust for k in range(3)]
                prism(f'{layer}_{half}', shape, y, y + h, M.crust if layer == 'bread_hi' else M.bread, M.crust,
                      sides=sides, at=off)
            else:
                prism(f'{layer}_{half}', shape, y, y + h, m, m, at=off)
            y += h
    return done('sandwich')


def plateDirty():
    """A plate after the meal: a smear of gravy, a streak of sauce, crumbs, and the fork left on it."""
    plate()
    disc('smear', 0.13, 0.002, (0.02, WELL, 0.02), M.smear, verts=12, depth=0.09, yaw=-15)
    disc('sauce', 0.07, 0.002, (-0.04, WELL + 0.0005, 0.045), M.smear_sauce, verts=10, depth=0.022, yaw=25)
    for i, (x, z) in enumerate(((-0.05, -0.02), (0.06, 0.05), (0.0, 0.07))):
        chunk(f'crumb{i}', 0.008, (x, WELL + 0.004, z), M.crumb, yaw=40 * i)
    with G.new_parts() as fork:
        G.cube('fork_handle', (0.085, 0.012, 0.004), at=(0.03, 0, 0.002), material=M.steel)
        G.cube('fork_neck', (0.025, 0.007, 0.003), at=(-0.022, 0, 0.0015), material=M.steel)
        G.cube('fork_head', (0.016, 0.02, 0.003), at=(-0.042, 0, 0.0015), material=M.steel)
        for k in (-1, 0, 1):
            G.cube(f'fork_tine{k}', (0.028, 0.0032, 0.0025), at=(-0.064, k * 0.0075, 0.00125), material=M.steel)
    f = G.join(fork, 'fork', origin=(0, 0, 0))
    f.location = F.at(0.0, WELL + 0.0025, -0.028)
    f.rotation_euler = (0, 0, math.radians(25))
    return done('plateDirty')


# --- the quick foods, each its own (review M-held-01: they borrowed the plates and the prep bowl) -----------------
def _dessert_bowl():
    """A glazed dessert bowl 12.5 cm across and 5 cm deep, its inside floor at 1.2 cm."""
    lathe('bowl', [(0, 0), (0.035, 0), (0.04, 0.006), (0.062, 0.05), (0.058, 0.052), (0.054, 0.044), (0, 0.012)],
          [M.bowl, M.bowl, M.bowl, M.bowl, M.bowl, M.china_well], segments=16)


def _spoon(at, yaw, tilt, name='spoon'):
    """A steel dessert spoon, its bowl at the game point `at`, the handle out along `yaw` (degrees from +x)
    rising at `tilt` degrees."""
    x, y, z = at
    a, t = math.radians(yaw), math.radians(tilt)
    end = (x + 0.11 * math.cos(a) * math.cos(t), y + 0.11 * math.sin(t), z - 0.11 * math.sin(a) * math.cos(t))
    G.rod(f'{name}_handle', F.at(x, y, z), F.at(*end), 0.003, material=M.steel, verts=5)
    ball(f'{name}_bowl', 0.012, (x, y, z), M.steel, scale=(1, 0.35, 1.4), seg=8, rings=4)


def iceCream():
    """A dessert bowl of ice cream from the freezer: three scoops (vanilla, strawberry, chocolate) and a spoon."""
    _dessert_bowl()
    for i, (x, z, m, s) in enumerate(((-0.018, 0.012, M.vanilla, 1.0), (0.02, 0.008, M.strawberry, 0.95),
                                      (0.0, -0.022, M.choc_ice, 0.9))):
        lump(f'scoop{i}', 0.03 * s, (x, 0.03, z), m, scale=(1, 0.85, 1), seed=1.3 + i, subdivisions=2, noise=0.12)
    _spoon((0.025, 0.07, 0.03), 35, 15)
    return done('iceCream')


def bowlDirty():
    """The dessert bowl eaten from: a film of melted ice cream in the bottom and the spoon left in it."""
    _dessert_bowl()
    disc('melt', 0.07, 0.002, (0.004, 0.012, 0.0), M.melt, verts=10, depth=0.06, yaw=15)
    disc('melt_choc', 0.03, 0.002, (-0.012, 0.0135, -0.01), M.choc_ice, verts=8, depth=0.02, yaw=-30)
    _spoon((0.012, 0.022, 0.012), 30, 32)
    return done('bowlDirty')


def toast():
    """A plate of toast: a slice cut corner to corner, toasted golden on top with darker crusts, a pat of butter
    melting on each half."""
    plate()
    a = [(-0.055, -0.05), (0.055, -0.05), (-0.055, 0.05)]
    b = [(0.055, -0.05), (0.055, 0.05), (-0.055, 0.05)]
    nx, nz = 0.05, 0.055
    n = math.hypot(nx, nz)
    for half, pts, cut, sign in (('a', a, 1, -1), ('b', b, 2, 1)):
        off = (sign * nx / n * 0.01, sign * nz / n * 0.01)
        sides = [M.bread if k == cut else M.toast_crust for k in range(3)]
        prism(f'slice_{half}', pts, WELL, WELL + 0.013, M.toast, M.toast_crust, sides=sides, at=off)
        cx = sum(p[0] for p in pts) / 3 + off[0]
        cz = sum(p[1] for p in pts) / 3 + off[1]
        box(f'butter_{half}', 0.018, 0.005, 0.016, (cx, WELL + 0.0155, cz), M.butter, yaw=20 if sign > 0 else -10,
            bevel=0.002)
    return done('toast')


def smoothie():
    """A tall tumbler of berry smoothie, 8 cm across and 14 cm high (about 0.4 L), with a paper straw."""
    lathe('glass', [(0, 0), (0.03, 0), (0.031, 0.008), (0.039, 0.12), (0.04, 0.14), (0.037, 0.14), (0.036, 0.12),
                    (0, 0.12)],
          [M.glass, M.glass, M.smoothie, M.glass, M.glass, M.glass, M.smoothie_top], segments=16)
    G.rod('straw', F.at(0.008, 0.1, 0.004), F.at(0.026, 0.195, 0.016), 0.0035, material=M.straw, verts=6)
    return done('smoothie')


def noodles():
    """A takeaway noodle pail of white card with a red band, noodles and spring onion heaped in its top and a
    pair of chopsticks standing in it."""
    # a square pail, 7 cm across at its foot and 10 at its mouth, 10 cm high (corners at r = side / sqrt 2)
    r0, r1, h = 0.0495, 0.0707, 0.1
    r = lambda y: r0 + (r1 - r0) * y / h  # noqa: E731
    lathe('pail', [(0, 0), (r0, 0), (r(0.05), 0.05), (r(0.068), 0.068), (r1, h), (r1 - 0.004, h), (0, 0.086)],
          [M.paper, M.paper, M.paper_red, M.paper, M.paper, M.noodle], segments=4, rot=(0, 0, 45))
    for i, (x, z, yaw) in enumerate(((-0.02, 0.01, 10), (0.015, -0.012, 70), (0.0, 0.022, 130), (0.022, 0.018, 40))):
        roll(f'noodle{i}', 0.004, 0.06, M.noodle, (x, 0.092, z), yaw=yaw, segments=5)
    for i, (x, z) in enumerate(((-0.01, -0.02), (0.02, 0.0), (-0.025, 0.02))):
        chunk(f'onion{i}', 0.008, (x, 0.098, z), M.lettuce, yaw=30 * i)
    for i, dx in enumerate((-0.006, 0.006)):
        G.rod(f'chopstick{i}', F.at(0.01 + dx, 0.06, -0.005), F.at(0.035 + dx * 2, 0.27, -0.03), 0.0035,
              material=M.chopstick, verts=5)
    return done('noodles')


def takeout():
    """An open kraft takeaway box, its lid standing up at the back: a burger and a heap of fries."""
    box('tray', 0.19, 0.045, 0.14, (0, 0.0225, 0), M.kraft, bevel=0.003)
    box('tray_inside', 0.18, 0.002, 0.13, (0, 0.0455, 0), M.kraft_inside)
    # the lid, hinged on the tray's back edge and opened past upright (110 degrees), leaning back
    lean = math.radians(20)
    box('lid', 0.19, 0.14, 0.006, (0, 0.045 + 0.07 * math.cos(lean), -0.07 - 0.07 * math.sin(lean)), M.kraft_fold,
        tilt=(-20, 0), bevel=0.002)
    # the burger
    bx, bz, y = -0.04, 0.005, 0.0465
    disc('bun_base', 0.085, 0.016, (bx, y, bz), M.bun_base, verts=12)
    disc('patty', 0.09, 0.016, (bx, y + 0.016, bz), M.patty, verts=12)
    box('cheese', 0.072, 0.003, 0.072, (bx, y + 0.0335, bz), M.cheese, yaw=45)
    lump('lettuce', 0.045, (bx, y + 0.033, bz), M.lettuce, scale=(1.1, 0.18, 1.1), seed=2.2, noise=0.25)
    lathe('bun_top', [(0, 0), (0.043, 0), (0.044, 0.008), (0.035, 0.026), (0.018, 0.034), (0, 0.036)],
          [M.bun_base, M.bun, M.bun, M.bun, M.bun], at=(bx, y + 0.04, bz), segments=12)
    # the fries, heaped beside it
    for i in range(9):
        a = 37 * i
        box(f'fry{i}', 0.07, 0.008, 0.008, (0.05 + 0.012 * math.cos(math.radians(a * 3)), y + 0.004 + 0.007 * (i % 3),
                                            0.01 + 0.03 * math.sin(math.radians(a * 2))), M.fries, yaw=a, tilt=(0, 8 * (i % 2)))
    return done('takeout')


# --- the dishes emptied, each the dish that went in (review M-held-02) ----------------------------------------------
def _serving_board():
    """The tasting board's board: 35 by 24 cm with a tab handle."""
    box('board', 0.35, 0.028, 0.24, (-0.025, 0.014, 0), M.board_dark, bevel=0.005)
    box('tab', 0.052, 0.028, 0.08, (0.174, 0.014, 0), M.board_dark, bevel=0.005)


def boardDirty():
    """The tasting board or cheese board cleared: a cheese rind, a bare grape stalk, cracker crumbs and a smear."""
    _serving_board()
    disc('smear', 0.14, 0.0015, (-0.06, 0.028, 0.0), M.grease, verts=10, depth=0.08, yaw=12)
    prism('rind', [(-0.14, -0.05), (-0.11, -0.04), (-0.135, -0.02)], 0.028, 0.04, M.cheese, M.cheese)
    G.rod('stalk', F.at(0.08, 0.034, -0.02), F.at(0.12, 0.034, -0.035), 0.003, material=M.leaf, verts=4)
    G.rod('stalk_twig', F.at(0.1, 0.034, -0.027), F.at(0.11, 0.034, -0.005), 0.002, material=M.leaf, verts=4)
    for i, (x, z) in enumerate(((-0.02, 0.05), (0.03, 0.07), (-0.09, 0.06), (0.05, -0.06), (0.0, -0.03))):
        chunk(f'crumb{i}', 0.008, (x, 0.031, z), M.cracker, yaw=33 * i)
    return done('boardDirty')


def pizzaBoardDirty():
    """The pizza's round board after it: crust ends left at the edge, a grease smear and crumbs."""
    lathe('board', [(0, 0), (0.2, 0), (0.2, 0.02), (0, 0.02)], M.peel, segments=20)
    disc('grease', 0.15, 0.0015, (-0.01, 0.02, 0.01), M.grease, verts=12, depth=0.11, yaw=10)
    disc('sauce', 0.06, 0.0015, (0.05, 0.0213, -0.04), M.smear_sauce, verts=8, depth=0.03, yaw=-25)
    for i, (a, length) in enumerate(((30, 0.06), (150, 0.05), (250, 0.055))):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        roll(f'crust{i}', 0.009, length, M.dough_baked, (0.13 * c, 0.029, 0.13 * s), yaw=a + 90, segments=6)
    for i, (x, z) in enumerate(((-0.05, 0.04), (0.03, 0.08), (0.08, 0.02), (-0.02, -0.07))):
        chunk(f'crumb{i}', 0.008, (x, 0.0235, z), M.crumb, yaw=40 * i)
    return done('pizzaBoardDirty')


def pieDishDirty():
    """The pie dish emptied: a red smear of filling over the bottom, pastry crumbs against its sides."""
    lathe('dish', [(0, 0), (0.135, 0), (0.166, 0.057), (0.168, 0.061), (0.158, 0.063), (0.128, 0.01), (0, 0.01)],
          [M.dish, M.dish, M.dish, M.dish, M.dish, M.residue], segments=20)
    disc('filling', 0.16, 0.002, (0.01, 0.01, 0.0), M.smear_filling, verts=12, depth=0.12, yaw=20)
    for i, a in enumerate((20, 95, 170, 230, 300)):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        chunk(f'pastry{i}', 0.014, (0.115 * c, 0.016, 0.115 * s), M.pastry_baked, yaw=a, tilt=(15, 10))
    return done('pieDishDirty')


# --- tools: on the station while a resident cooks ---------------------------------------------------------------
def board():
    """The chopping board: a carrot half chopped into rounds, chopped greens and the knife."""
    box('board', 0.34, 0.025, 0.24, (0, 0.0125, 0), M.board, bevel=0.005)
    G.cone('carrot', 0.007, 0.015, 0.09, at=F.at(-0.1, 0.04, 0.035), material=M.carrot, rot=(0, -90, 15), verts=8)
    for i, (x, z, t) in enumerate(((-0.04, 0.04, 0), (-0.025, 0.06, 0), (-0.035, 0.015, 0), (-0.01, 0.035, 0),
                                   (-0.05, 0.075, 0))):
        disc(f'round{i}', 0.028, 0.006, (x, 0.025, z), M.carrot, verts=8)
    for i, (x, z, yaw) in enumerate(((0.02, 0.07, 10), (0.04, 0.05, 40), (0.03, 0.09, 70), (0.055, 0.075, 20))):
        chunk(f'green{i}', 0.018, (x, 0.032, z), M.pea, yaw=yaw)
    with G.new_parts() as knife:
        G.cube('blade', (0.115, 0.03, 0.003), at=(0.0, 0.0, 0.0), material=M.steel)
        G.cone('tip', 0.015, 0.0, 0.03, at=(-0.0725, 0.0, 0.0), material=M.steel, rot=(0, -90, 0), verts=4,
               scale=(0.1, 1, 1))
        G.cube('handle', (0.07, 0.016, 0.018), at=(0.092, -0.004, 0.004), material=M.grip)
    k = G.join(knife, 'knife', origin=(0, 0, 0))
    k.location = F.at(0.05, 0.027, -0.07)
    k.rotation_euler = (0, 0, math.radians(20))
    return done('board')


def pan():
    """The frying pan on the stove, its handle to the back (+depth: away from the cook, as the old part's,
    out of the way of the hands): a heap of stir-fry, beef with carrot and greens."""
    lathe('pan', [(0, 0), (0.115, 0), (0.14, 0.045), (0.135, 0.05), (0.113, 0.009), (0, 0.009)],
          [M.iron, M.iron, M.iron, M.iron, M.iron], segments=20)
    box('neck', 0.022, 0.012, 0.04, (0, 0.04, 0.15), M.iron)
    box('handle', 0.026, 0.018, 0.09, (0, 0.044, 0.21), M.grip, bevel=0.004)
    lump('heap', 0.075, (0.0, 0.006, 0.0), M.stew, scale=(1, 0.32, 1), seed=8.5, noise=0.15)
    for i, (x, z, m, yaw) in enumerate(((0.02, -0.01, M.stew, 10), (-0.04, 0.02, M.stew, 40), (0.05, 0.04, M.stew, 70),
                                        (-0.02, -0.05, M.stew, 25), (0.05, -0.035, M.carrot, 15),
                                        (-0.055, -0.02, M.carrot, 55), (0.0, 0.055, M.carrot, 80),
                                        (-0.045, 0.05, M.pea, 30), (0.025, 0.025, M.pea, 5), (0.065, 0.005, M.pea, 60))):
        # on the heap: higher in its middle
        y = 0.012 + 0.04 * math.sqrt(max(0.0, 1 - (math.hypot(x, z) / 0.08) ** 2))
        chunk(f'bit{i}', 0.022 if m is M.stew else 0.017, (x, y, z), m, yaw=yaw, tilt=(10 * (i % 3 - 1), 0))
    return done('pan')


def grillFood():
    """On the grill: two sausages and a burger, marked by the bars."""
    for i, (z, m) in enumerate(((-0.05, M.sausage), (0.04, M.sausage_b))):
        roll(f'sausage{i}', 0.0225, 0.16, (m, M.char), (-0.08, 0.0225, z), marks=((-0.55, -0.4), (-0.05, 0.1),
                                                                                (0.45, 0.6)))
    disc('patty', 0.1, 0.03, (0.08, 0, 0.0), M.patty, verts=12)
    for i, x in enumerate((0.06, 0.08, 0.1)):
        box(f'mark{i}', 0.007, 0.002, 0.088, (x, 0.0305, 0.0), M.char)
    return done('grillFood')


def mixingBowl():
    """On the counter while batter is mixed or pastry made (`mix`): a deep glazed bowl of batter, a wooden
    spoon standing in it, leaning on the far rim (+depth: away from the cook, as the pan's handle)."""
    lathe('bowl', [(0, 0), (0.07, 0), (0.085, 0.01), (0.13, 0.11), (0.124, 0.113), (0.117, 0.1), (0, 0.075)],
          [M.mixing, M.mixing, M.mixing, M.mixing, M.mixing, M.batter], segments=20)
    lump('batter_swirl', 0.05, (0.015, 0.07, -0.01), M.batter, scale=(1, 0.25, 1), seed=3.5, noise=0.15)
    G.rod('spoon_handle', F.at(0.0, 0.07, 0.02), F.at(0.03, 0.27, 0.13), 0.007, material=M.spoon, verts=8)
    ball('spoon_bowl', 0.022, (-0.002, 0.07, 0.012), M.spoon, scale=(1, 0.35, 1.4))
    return done('mixingBowl')


def I(id, build, size, key, centre=(0.0, 0.0), code=False):
    return G.Model(id, build, size=size, key=key, centre=centre, code=code,
                   tolerance=3 if os.environ.get('HELD_MEASURE') else 0.02)


FAMILY = G.Family(
    'held', M,
    [
        I('ingredients', ingredients, (0.2, 0.336, 0.14), 'item.ingredients'),
        I('prepBowl', prepBowl, (0.324, 0.104, 0.319), 'item.prepBowl'),
        I('mealPot', mealPot, (0.39, 0.154, 0.266), 'item.mealPot'),
        I('potDirty', potDirty, (0.39, 0.15, 0.266), 'item.potDirty'),
        I('tastingBoard', tastingBoard, (0.4, 0.075, 0.24), 'item.tastingBoard'),
        I('grillPlatter', grillPlatter, (0.4, 0.06, 0.28), 'item.grillPlatter'),
        I('platterDirty', platterDirty, (0.4, 0.034, 0.28), 'item.platterDirty'),
        I('pizza', pizza, (0.4, 0.052, 0.4), 'item.pizza'),
        I('bakingTray', bakingTray, (0.38, 0.046, 0.28), 'item.bakingTray'),
        I('trayDirty', trayDirty, (0.38, 0.025, 0.28), 'item.trayDirty'),
        I('pieDish', pieDish, (0.336, 0.084, 0.336), 'item.pieDish'),
        I('mealPlate', mealPlate, (0.288, 0.061, 0.288), 'item.mealPlate'),
        I('pizzaSlice', pizzaSlice, (0.288, 0.031, 0.288), 'item.pizzaSlice'),
        I('treatPlate', treatPlate, (0.288, 0.072, 0.288), 'item.treatPlate'),
        I('pieSlice', pieSlice, (0.288, 0.059, 0.288), 'item.pieSlice'),
        I('sandwich', sandwich, (0.288, 0.064, 0.288), 'item.sandwich'),
        I('plateDirty', plateDirty, (0.288, 0.017, 0.288), 'item.plateDirty'),
        I('iceCream', iceCream, (0.176, 0.101, 0.124), 'item.iceCream', centre=(0.0258, 0.0)),
        I('bowlDirty', bowlDirty, (0.157, 0.083, 0.124), 'item.bowlDirty', centre=(0.0164, 0.0)),
        I('toast', toast, (0.288, 0.027, 0.288), 'item.toast'),
        I('smoothie', smoothie, (0.08, 0.196, 0.08), 'item.smoothie'),
        I('noodles', noodles, (0.1, 0.271, 0.1), 'item.noodles'),
        I('takeout', takeout, (0.192, 0.177, 0.19), 'item.takeout', centre=(0.001, -0.025)),
        I('boardDirty', boardDirty, (0.4, 0.04, 0.24), 'item.boardDirty'),
        I('pizzaBoardDirty', pizzaBoardDirty, (0.4, 0.038, 0.4), 'item.pizzaBoardDirty'),
        I('pieDishDirty', pieDishDirty, (0.336, 0.063, 0.336), 'item.pieDishDirty'),
        I('board', board, (0.34, 0.055, 0.24), 'tool.board'),
        # the handle reaches 2 cm further back than the old part's crossbar: the pan stays on its middle
        I('pan', pan, (0.28, 0.061, 0.395), 'tool.pan', centre=(0.0, 0.0575)),
        I('grillFood', grillFood, (0.29, 0.045, 0.135), 'tool.grillFood', centre=(-0.015, -0.005)),
        I('mixingBowl', mixingBowl, (0.26, 0.273, 0.266), 'tool.mixingBowl', centre=(0.0, 0.0029), code=True),
    ],
    comment='Blender-built things residents carry and cook with (tools/art/blender/families/held.py): the '
            'models of content `items` and the props drawn on a station while cooking. Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes, so each costs one draw call '
            '(the wooden boards two).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
