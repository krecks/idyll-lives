# The tabletop things: small things that stand on tops (the `surface` layer), over the primitive-part
# placeholders of `catalog2.py`, which stay the fallback. Notes: tabletop.md.
#
#   TT1 kitchenware: kettle, toaster, blender, standMixer, espressoMachineSmall, knifeBlock, utensilCrock,
#       cuttingBoard, breadBin, spiceJars, cookieJar
#   TT2 tableware: fruitBowl, plateSet, cakeStand, teapotSet, mugs, wineGlasses, candlesticks
#   TT3 books and small plants: bookStack, booksUpright, vaseFlowers, vaseSmall, potSucculent, potCactus,
#       potSmall
#   TT4 small ornaments: photoFrame, tableClock, globe, snowGlobe, trophy, sculptureSmall
#   TT5 bath, desk and toys: soapSet, towelStack, perfumeTray, deskOrganiser, laptopClosed, monitor,
#       stuffedToy, toyBlocks
#
# Each stands on its base, centred (the renderer lifts it to its host's top), at its placeholder's size. They
# have no uses, so no clips, and every finish is a plain one (wood, fabric and leaves drawn satin or matte),
# so each costs one draw call. The exceptions: the cookie jar, the wine glasses and the snow globe are
# see-through `glass` (two); the kettle's lid, the bread bin's roll-top and the laptop's lid are parts of
# their own, hinged where they would open, for a use to come (two); the alarm clock's hour and minute hands
# tell the game's time (`motions.clock`; three).
#
# Run:  python3 -B tools/art/blender/build.py tabletop [--only kettle] [--preview]
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
    # shared details
    dark=('#2B2C2E', 'satin'),
    charcoal=('#3A3D41', 'satin'),
    steel=('#C3C7CB', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    brass=('#C9A54A', 'metal'),
    wood=('#9C6B43', 'satin'),         # plain, not `wood`: a textured finish would be a second draw call
    wood_dark=('#7A5234', 'satin'),
    porcelain=('#F5F3EE', 'gloss'),
    paper=('#F7F7F5', 'matte'),
    leaf=('#5E8F4E', 'satin'),         # plain, not `foliage`, for the same reason
    leaf_light=('#7FA35A', 'satin'),
    soil=('#4A3A2C', 'matte'),
    glass=('#DCE6EA', 'glass'),        # see-through: its own draw call (cookie jar, wine glasses, snow globe)
    glass_pale=('#DCE6EA', 'gloss'),   # glass drawn opaque, where seeing through it doesn't matter
    red=('#D9483B', 'gloss'),
    orange=('#F29B38', 'satin'),
    yellow=('#F2C14E', 'satin'),
    apple_green=('#9BC24A', 'satin'),
    stalk=('#5A4632', 'matte'),
    white=('#F2F0EA', 'satin'),
    blush=('#E4B8AE', 'satin'),
    rose=('#E07B7B', 'satin'),
    pink=('#E07BC3', 'satin'),
    bread=('#D9A55B', 'matte'),
    crumb=('#F0DDB5', 'matte'),
    coffee=('#5A3A26', 'gloss'),
    wine=('#6E1E2A', 'gloss'),
    # TT1 kitchenware
    kettle=('#F2F0EA', 'gloss'),
    toaster=('#E3D9C6', 'gloss'),
    blender_base=('#3A3D41', 'satin'),
    jug=('#DCE6EA', 'gloss'),          # opaque, so the smoothie shows as the jug's lower half
    smoothie=('#E9A3B5', 'gloss'),
    mixer=('#C0473A', 'gloss'),
    espresso=('#C3C7CB', 'metal'),
    knife_block=('#9C6B43', 'satin'),
    crock=('#E8E2D6', 'gloss'),
    board=('#C99A68', 'satin'),
    bread_bin=('#E8E2D6', 'gloss'),
    rack=('#9C6B43', 'satin'),
    spice_a=('#C98F4B', 'matte'),
    spice_b=('#7A8C3A', 'matte'),
    spice_c=('#B5452E', 'matte'),
    spice_d=('#E3C66A', 'matte'),
    spice_e=('#5E4B3A', 'matte'),
    jar_lid=('#C25B3E', 'gloss'),
    cookie=('#C98F4B', 'matte'),
    cookie_dark=('#8A5A36', 'matte'),
    # TT2 tableware
    fruit_bowl=('#E8E2D6', 'gloss'),
    plate=('#F5F3EE', 'gloss'),
    plate_rim=('#3E5470', 'gloss'),
    napkin=('#E7E0D4', 'matte'),
    cake_stand=('#F2F0EA', 'gloss'),
    sponge=('#E9C8A0', 'matte'),
    icing=('#F7E7EC', 'gloss'),
    tray=('#9C6B43', 'satin'),
    teapot=('#F2F0EA', 'gloss'),
    teapot_trim=('#3E5470', 'gloss'),
    mug_a=('#3E5470', 'gloss'),
    mug_b=('#E9C46A', 'gloss'),
    mug_c=('#E4B8AE', 'gloss'),
    bottle=('#2F4A33', 'gloss'),
    candle=('#F4EBD8', 'satin'),
    flame=('#FFD27A', 'gloss'),        # gloss, never emissive
    # TT3 books and small plants
    book_a=('#3E5470', 'matte'),
    book_b=('#D6A64A', 'matte'),
    book_c=('#B5653F', 'matte'),
    book_d=('#9DAF8C', 'matte'),
    book_e=('#4E7A7F', 'matte'),
    pages=('#F2F0EA', 'matte'),
    vase=('#9DC6CF', 'gloss'),
    bud_vase=('#C25B3E', 'gloss'),
    pot_pale=('#E8E2D6', 'matte'),
    terracotta=('#C26E4A', 'matte'),
    pot_white=('#F2F0EA', 'gloss'),
    succulent=('#8FB59A', 'satin'),
    succulent_light=('#A9C9A4', 'satin'),
    cactus=('#4F8A55', 'satin'),
    sand=('#D9C7A7', 'matte'),
    # TT4 small ornaments
    frame=('#2B2C2E', 'satin'),
    photo_sky=('#9DB7C9', 'matte'),
    photo_hill=('#8FB27A', 'matte'),
    clock=('#C0473A', 'gloss'),
    clock_face=('#F7F4EC', 'matte'),
    globe=('#4E8FB8', 'gloss'),
    land=('#8EB46A', 'matte'),
    snow=('#F7F7F5', 'satin'),
    fir=('#3F7A4A', 'satin'),
    trophy=('#D6A64A', 'metal'),
    trophy_base=('#2B2C2E', 'gloss'),
    plinth=('#E7E3DC', 'matte'),
    sculpture=('#2B2C2E', 'satin'),
    # TT5 bath, desk and toys
    soap_tray=('#E7E3DC', 'matte'),
    dispenser=('#F2F0EA', 'gloss'),
    tumbler=('#9DC6CF', 'gloss'),
    soap=('#E9DCC4', 'matte'),
    towel_a=('#9DC6CF', 'matte'),
    towel_b=('#F2F0EA', 'matte'),
    towel_c=('#E4B8AE', 'matte'),
    perfume_tray=('#D7E2E6', 'gloss'),
    perfume_a=('#E9A3B5', 'gloss'),
    perfume_b=('#E9C46A', 'gloss'),
    perfume_c=('#9DC6CF', 'gloss'),
    organiser=('#2B2C2E', 'satin'),
    pen_red=('#D9483B', 'gloss'),
    pen_blue=('#3E8EDE', 'gloss'),
    pen_yellow=('#F2C14E', 'gloss'),
    laptop=('#B9BCBF', 'metal'),
    logo=('#E7E7E5', 'gloss'),
    monitor=('#1E1F22', 'satin'),
    screen=('#16202B', 'gloss'),
    keys=('#3A3D41', 'satin'),
    bear=('#B98656', 'matte'),
    bear_light=('#E9D5B5', 'matte'),
    block_a=('#D9483B', 'satin'),
    block_b=('#3E8EDE', 'satin'),
    block_c=('#F2C14E', 'satin'),
    block_d=('#5BAE6A', 'satin'),
    block_e=('#E07BC3', 'satin'),
)


# --- helpers (Blender axes: x across, y to the back, the front at -y; z up; metres) --------------------------
def ball(name, r, at, material, scale=(1, 1, 1), rot=(0, 0, 0), seg=10, rings=7):
    """A low sphere: these are small, so few segments."""
    return G.sphere(name, r, at=at, material=material, scale=scale, rot=rot, segments=seg, rings=rings)


def cyl(name, r, h, at, material, verts=16, rot=(0, 0, 0), bevel=0.0):
    return G.cylinder(name, r, h, at=at, material=material, verts=verts, rot=rot, bevel=bevel, bevel_segments=1)


def box(name, size, at, material, bevel=0.0, rot=(0, 0, 0), segs=1):
    return G.cube(name, size, at=at, material=material, bevel=bevel, rot=rot, bevel_segments=segs)


def lathe(name, profile, material, at=(0, 0, 0), segments=16):
    """A surface of revolution about z from `profile` [(r, z)], bottom on the axis first, top on the axis last."""
    return G.revolve(name, profile, at=at, material=material, segments=segments)


def tilt_x(y, z, degrees):
    """(y, z) turned about the x axis by `degrees` (negative leans the top back, to +y)."""
    c, s = G.cosd(degrees), G.sind(degrees)
    return y * c - z * s, y * s + z * c


def arc_shell(name, r_out, r_in, length, a0, a1, at, material, segments=12):
    """Part of a tube along x, closed: the ring between `r_in` and `r_out` from angle `a0` to `a1` (degrees;
    0 is to the front, -y, and 90 straight up), `length` long, centred on `at`."""
    bm = bmesh.new()
    rings = []
    for i in range(segments + 1):
        a = math.radians(a0 + (a1 - a0) * i / segments)
        ring = []
        for x in (-length / 2, length / 2):
            for r in (r_out, r_in):
                ring.append(bm.verts.new((x, -math.cos(a) * r, math.sin(a) * r)))
        rings.append(ring)       # [x0 out, x0 in, x1 out, x1 in]
    for a, b in zip(rings, rings[1:]):
        bm.faces.new((a[0], b[0], b[2], a[2]))   # outside
        bm.faces.new((a[1], a[3], b[3], b[1]))   # inside
        bm.faces.new((a[0], a[1], b[1], b[0]))   # the end at -x
        bm.faces.new((a[2], b[2], b[3], a[3]))   # the end at +x
    for ring in (rings[0], rings[-1]):
        bm.faces.new((ring[0], ring[2], ring[3], ring[1]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    obj.location = at
    return obj


def done(name, mirror=False):
    """A still model: everything joined into one node, stood on the floor and centred. `mirror` flips it left
    to right, for one built the other way round from its placeholder."""
    obj = G.join_all(name)
    if mirror:
        obj.data.transform(Matrix.Scale(-1, 4, Vector((1, 0, 0))))
        obj.data.flip_normals()
    return G.settle([obj])


# --- TT1 kitchenware ------------------------------------------------------------------------------------------
def kettle():
    with G.new_parts() as body:
        cyl('base', 0.086, 0.02, (0, 0, 0.01), M.dark, verts=18)
        lathe('body', [(0, 0.02), (0.078, 0.02), (0.08, 0.035), (0.073, 0.15), (0.062, 0.2), (0.046, 0.212),
                       (0, 0.212)], M.kettle, segments=18)
        cyl('window', 0.0125, 0.1, (0.04, -0.066, 0.1), M.glass_pale, verts=8, rot=(0, 0, 0))
        G.torus('handle', 0.038, 0.011, at=(0.066, 0, 0.13), material=M.dark, rot=(90, 0, 0),
                major_segments=14, minor_segments=6)
        G.cone('spout', 0.022, 0.01, 0.07, at=(-0.08, 0, 0.165), material=M.kettle, rot=(0, -50, 0), verts=10)
    G.join(body, 'kettle', origin=(0, 0, 0))
    # the lid: a part of its own, hinged at its back edge, for a use to come (it lifts)
    with G.new_parts() as lid:
        cyl('lid', 0.044, 0.012, (0, 0, 0.217), M.dark, verts=16)
        ball('lid_knob', 0.013, (0, -0.012, 0.228), M.dark, scale=(1.2, 1, 0.8))
    G.hinged(G.join(lid, 'lid'), (0, 0.044, 0.223))
    return G.settle()


def toaster():
    box('plinth', (0.24, 0.14, 0.012), (0, 0, 0.006), M.dark, bevel=0.004)
    box('body', (0.26, 0.15, 0.16), (0, 0, 0.092), M.toaster, bevel=0.035, segs=3)
    for s in (-1, 1):
        box(f'slot{s}', (0.17, 0.024, 0.004), (0, s * 0.035, 0.1715), M.dark)
        box(f'toast{s}', (0.13, 0.014, 0.075), (0, s * 0.035, 0.192), M.bread, bevel=0.012, segs=2)
        box(f'crumb{s}', (0.116, 0.0145, 0.06), (0, s * 0.035, 0.192), M.crumb)
    box('lever', (0.02, 0.03, 0.014), (-0.14, 0, 0.125), M.dark, bevel=0.004)
    cyl('knob', 0.013, 0.012, (-0.136, 0.0, 0.06), M.chrome, verts=12, rot=(0, 90, 0))
    return done('toaster')


def blender():
    box('base', (0.14, 0.14, 0.1), (0, 0, 0.05), M.blender_base, bevel=0.02, segs=2)
    for i, x in enumerate((-0.03, 0, 0.03)):
        cyl(f'button{i}', 0.009, 0.008, (x, -0.071, 0.05), M.chrome, verts=10, rot=(90, 0, 0))
    cyl('collar', 0.06, 0.02, (0, 0, 0.11), M.dark, verts=16)
    lathe('smoothie', [(0, 0.12), (0.05, 0.12), (0.058, 0.22), (0, 0.22)], M.smoothie)
    lathe('jug', [(0, 0.22), (0.058, 0.22), (0.068, 0.32), (0, 0.32)], M.jug)
    cyl('lid', 0.071, 0.02, (0, 0, 0.33), M.dark, verts=16)
    cyl('cap', 0.022, 0.016, (0, 0, 0.348), M.dark, verts=10)
    G.rod('handle', (-0.098, 0, 0.17), (-0.098, 0, 0.3), 0.011, material=M.dark, verts=8)
    G.rod('handle_low', (-0.05, 0, 0.18), (-0.1, 0, 0.18), 0.011, material=M.dark, verts=8)
    G.rod('handle_high', (-0.06, 0, 0.29), (-0.1, 0, 0.29), 0.011, material=M.dark, verts=8)
    return done('blender')


def standMixer():
    box('base', (0.3, 0.17, 0.04), (0, 0, 0.02), M.mixer, bevel=0.015, segs=2)
    box('column', (0.09, 0.12, 0.23), (0.1, 0, 0.15), M.mixer, bevel=0.035, segs=3)
    ball('head', 0.066, (0.0, 0, 0.29), M.mixer, scale=(2.2, 1, 1), seg=16, rings=10)
    cyl('band', 0.03, 0.006, (-0.13, 0, 0.29), M.chrome, verts=14, rot=(0, 90, 0))
    cyl('speed', 0.012, 0.014, (0.1, -0.064, 0.2), M.chrome, verts=10, rot=(90, 0, 0))
    lathe('bowl', [(0, 0.04), (0.05, 0.04), (0.075, 0.075), (0.08, 0.15), (0.074, 0.15), (0.068, 0.08),
                   (0.045, 0.05), (0, 0.05)], M.steel, at=(-0.07, 0, 0), segments=18)
    lathe('batter', [(0, 0.05), (0.06, 0.05), (0.069, 0.11), (0, 0.11)], M.crumb, at=(-0.07, 0, 0), segments=18)
    G.rod('shaft', (-0.07, 0, 0.23), (-0.07, 0, 0.13), 0.008, material=M.steel, verts=8)
    ball('beater', 0.022, (-0.07, 0, 0.125), M.steel, scale=(1, 0.5, 1.4))
    return done('standMixer')


def espressoMachineSmall():
    box('body', (0.22, 0.24, 0.27), (0, 0.05, 0.135), M.espresso, bevel=0.018, segs=2)
    box('hood', (0.22, 0.07, 0.06), (0, -0.1, 0.24), M.espresso, bevel=0.012, segs=2)
    box('drip_tray', (0.18, 0.13, 0.02), (0, -0.13, 0.01), M.dark, bevel=0.005)
    box('grate', (0.16, 0.11, 0.004), (0, -0.13, 0.021), M.steel)
    cyl('group', 0.034, 0.03, (0, -0.11, 0.195), M.dark, verts=14)
    cyl('basket', 0.034, 0.022, (0, -0.11, 0.17), M.steel, verts=14)
    G.rod('portafilter', (0, -0.13, 0.17), (0, -0.22, 0.165), 0.012, material=M.dark, verts=8)
    lathe('cup', [(0, 0.023), (0.025, 0.023), (0.032, 0.07), (0.028, 0.07), (0.022, 0.03), (0, 0.03)],
          M.porcelain, at=(0, -0.12, 0), segments=14)
    cyl('crema', 0.028, 0.004, (0, -0.12, 0.062), M.coffee, verts=12)
    cyl('gauge', 0.024, 0.008, (0.06, -0.074, 0.12), M.chrome, verts=14, rot=(90, 0, 0))
    cyl('gauge_face', 0.019, 0.004, (0.06, -0.079, 0.12), M.paper, verts=14, rot=(90, 0, 0))
    cyl('knob', 0.014, 0.022, (-0.11, -0.03, 0.15), M.dark, verts=10, rot=(0, 90, 0))
    G.rod('steam_wand', (-0.09, -0.07, 0.21), (-0.1, -0.1, 0.08), 0.005, material=M.chrome, verts=6)
    lathe('top_cup', [(0, 0.27), (0.025, 0.27), (0.025, 0.272), (0.02, 0.31), (0, 0.31)], M.porcelain,
          at=(0.05, 0.1, 0), segments=12)
    return done('espressoMachineSmall')


def knifeBlock():
    box('foot', (0.12, 0.2, 0.05), (0, 0, 0.025), M.knife_block, bevel=0.006)
    TILT, ZC = -18, 0.123
    box('block', (0.12, 0.18, 0.2), (0, 0, ZC), M.knife_block, bevel=0.01, rot=(TILT, 0, 0))
    for i, (x, y) in enumerate(((-0.03, -0.05), (0.0, -0.05), (0.03, -0.05), (-0.022, 0.02), (0.022, 0.02),
                                (0.0, 0.06))):
        dy, dz = tilt_x(y, 0.1 + 0.032, TILT)
        box(f'handle{i}', (0.018 if i < 3 else 0.022, 0.026, 0.064), (x, dy, ZC + dz), M.dark, bevel=0.006,
            rot=(TILT, 0, 0))
    return done('knifeBlock')


def utensilCrock():
    lathe('crock', [(0, 0), (0.055, 0), (0.062, 0.02), (0.062, 0.135), (0.058, 0.15), (0.052, 0.15),
                    (0.053, 0.03), (0, 0.03)], M.crock)
    # a wooden spoon, a spatula, a whisk and a ladle, leaning out of the crock
    G.rod('spoon', (0.0, 0.0, 0.04), (0.035, -0.02, 0.27), 0.006, material=M.wood, verts=6)
    ball('spoon_bowl', 0.02, (0.039, -0.022, 0.285), M.wood, scale=(1, 0.45, 1.5), rot=(0, 8, 0))
    G.rod('spatula', (0.0, 0.0, 0.04), (-0.035, 0.02, 0.25), 0.006, material=M.dark, verts=6)
    box('spatula_head', (0.05, 0.006, 0.065), (-0.041, 0.024, 0.28), M.dark, bevel=0.004, rot=(0, -9, 0))
    G.rod('whisk', (0.0, 0.0, 0.04), (0.0, 0.035, 0.24), 0.006, material=M.steel, verts=6)
    ball('whisk_head', 0.022, (0.0, 0.04, 0.27), M.steel, scale=(1, 1, 1.7), seg=8, rings=6)
    G.rod('ladle', (0.0, 0.0, 0.04), (-0.02, -0.035, 0.26), 0.006, material=M.wood_dark, verts=6)
    ball('ladle_bowl', 0.019, (-0.022, -0.039, 0.275), M.wood_dark, scale=(1.1, 0.6, 1.2))
    return done('utensilCrock')


def cuttingBoard():
    box('board', (0.34, 0.23, 0.022), (0, 0, 0.011), M.board, bevel=0.008, segs=2)
    ball('loaf', 0.052, (0.06, 0.02, 0.06), M.bread, scale=(1.9, 1.0, 0.85), rot=(0, 0, 15), seg=14, rings=9)
    for i, d in enumerate((-0.04, 0.0, 0.04)):
        top = 0.06 + 0.052 * 0.85 * math.sqrt(1 - (d / 0.099) ** 2)
        box(f'score{i}', (0.007, 0.06, 0.008), (0.06 + d * G.cosd(15), 0.02 + d * G.sind(15), top - 0.002), M.crumb,
            rot=(0, 0, 15))
    for i, (x, y) in enumerate(((-0.1, 0.05), (-0.12, -0.03))):
        ball(f'tomato{i}', 0.03, (x, y, 0.05), M.red, scale=(1, 1, 0.85))
        cyl(f'stalk{i}', 0.008, 0.006, (x, y, 0.077), M.leaf, verts=6)
    for i, (x, y) in enumerate(((-0.04, -0.07), (-0.01, -0.08))):
        cyl(f'slice{i}', 0.027, 0.008, (x, y, 0.026 + i * 0.004), M.red, verts=12)
    box('blade', (0.11, 0.024, 0.003), (0.03, -0.075, 0.024), M.steel, rot=(0, 0, -8))
    box('knife_handle', (0.06, 0.018, 0.013), (0.11, -0.086, 0.03), M.dark, bevel=0.004, rot=(0, 0, -8))
    return done('cuttingBoard')


def breadBin():
    AXIS, R = 0.105, 0.095
    with G.new_parts() as body:
        box('base', (0.31, 0.19, AXIS), (0, 0, AXIS / 2), M.bread_bin, bevel=0.008)
        for s in (-1, 1):
            # the D-shaped wooden ends
            # flat-shaded: smooth shading rounds a cylinder's flat face into a dome
            G.cylinder(f'end_round{s}', R + 0.006, 0.014, at=(s * 0.163, 0, AXIS), material=M.wood, verts=20,
                       rot=(0, 90, 0), smooth=False)
            # a hair thinner than the round end, so their faces aren't one plane (they flickered)
            box(f'end_foot{s}', (0.012, 2 * R + 0.012, AXIS), (s * 0.163, 0, AXIS / 2), M.wood)
    G.join(body, 'bread_bin', origin=(0, 0, 0))
    # the roll-top: a part of its own, turning about the bin's axis, for a use to come (it rolls open)
    with G.new_parts() as top:
        arc_shell('roll_top', R, R - 0.008, 0.312, 0, 180, (0, 0, AXIS), M.bread_bin, segments=14)
        box('roll_handle', (0.1, 0.016, 0.016), (0, -R * math.cos(math.radians(35)) - 0.006,
                                                AXIS + R * math.sin(math.radians(35))), M.wood, bevel=0.005)
    G.hinged(G.join(top, 'roll_top'), (0, 0, AXIS))
    return G.settle()


def spiceJars():
    box('tray', (0.3, 0.12, 0.014), (0, 0, 0.007), M.rack, bevel=0.004)
    box('rail', (0.3, 0.01, 0.026), (0, -0.055, 0.02), M.rack, bevel=0.003)
    box('back', (0.3, 0.01, 0.06), (0, 0.055, 0.037), M.rack, bevel=0.003)
    for i, colour in enumerate((M.spice_a, M.spice_b, M.spice_c, M.spice_d, M.spice_e)):
        x = -0.11 + i * 0.055
        cyl(f'jar{i}', 0.022, 0.08, (x, 0, 0.054), colour, verts=12)
        cyl(f'label{i}', 0.0225, 0.026, (x, 0, 0.05), M.paper, verts=12)
        cyl(f'lid{i}', 0.0235, 0.018, (x, 0, 0.103), M.dark, verts=12)
    return done('spiceJars')


def cookieJar():
    lathe('jar', [(0, 0), (0.072, 0), (0.08, 0.012), (0.08, 0.15), (0.064, 0.172), (0.062, 0.19), (0, 0.19)],
          M.glass, segments=18)
    for i in range(5):
        a = i * 72
        ring = 0.03 if i else 0.0
        cyl(f'cookie{i}', 0.034, 0.012, (ring * G.cosd(a), ring * G.sind(a), 0.02 + i * 0.026),
            M.cookie if i % 2 else M.cookie_dark, verts=12, rot=(12 * (-1) ** i, 8, 0))
    cyl('lid', 0.068, 0.03, (0, 0, 0.205), M.jar_lid, verts=18, bevel=0.006)
    ball('knob', 0.022, (0, 0, 0.226), M.jar_lid)
    return done('cookieJar')


# --- TT2 tableware --------------------------------------------------------------------------------------------
def fruitBowl():
    lathe('bowl', [(0, 0), (0.055, 0), (0.06, 0.012), (0.1, 0.03), (0.14, 0.09), (0.133, 0.093), (0.095, 0.042),
                   (0.05, 0.03), (0, 0.03)], M.fruit_bowl, segments=20)
    for i, (x, y, z, colour) in enumerate(((0.05, 0.02, 0.07, M.red), (-0.05, -0.03, 0.07, M.orange),
                                           (-0.03, 0.07, 0.07, M.apple_green), (0.04, -0.05, 0.065, M.orange),
                                           (0.0, 0.01, 0.14, M.red))):
        ball(f'fruit{i}', 0.04, (x, y, z), colour, scale=(1, 1, 0.95))
        if colour is not M.orange:
            cyl(f'stem{i}', 0.003, 0.016, (x, y, z + 0.044), M.stalk, verts=5)
    ball('banana', 0.027, (-0.025, -0.035, 0.168), M.yellow, scale=(3.0, 0.75, 0.75), rot=(0, -12, 30), seg=12,
         rings=6)
    return done('fruitBowl')


def _plate(name, r, z, material):
    return lathe(name, [(0, z), (r * 0.6, z), (r * 0.64, z + 0.005), (r, z + 0.016), (r - 0.003, z + 0.02),
                        (r * 0.62, z + 0.011), (0, z + 0.011)], material, segments=24)


def plateSet():
    _plate('plate', 0.135, 0, M.plate)
    G.torus('rim', 0.128, 0.0035, at=(0, 0, 0.019), material=M.plate_rim, major_segments=24, minor_segments=4)
    _plate('side_plate', 0.095, 0.011, M.plate)
    box('napkin', (0.06, 0.17, 0.005), (-0.17, -0.01, 0.0025), M.napkin, bevel=0.002)
    box('fork', (0.014, 0.13, 0.003), (-0.17, -0.02, 0.0065), M.chrome)
    box('fork_head', (0.022, 0.04, 0.003), (-0.17, 0.06, 0.0065), M.chrome)
    box('knife', (0.016, 0.2, 0.003), (0.17, 0, 0.0015), M.chrome)
    lathe('tumbler', [(0, 0), (0.032, 0), (0.037, 0.1), (0.033, 0.1), (0.029, 0.008), (0, 0.008)], M.glass_pale,
          at=(0.14, 0.15, 0), segments=14)
    return done('plateSet', mirror=True)


def cakeStand():
    lathe('stand', [(0, 0), (0.08, 0), (0.08, 0.01), (0.03, 0.026), (0.022, 0.085), (0.14, 0.094),
                    (0.14, 0.103), (0, 0.103)], M.cake_stand, segments=24)
    cyl('sponge', 0.108, 0.096, (0, 0, 0.151), M.sponge, verts=24)
    cyl('filling', 0.109, 0.01, (0, 0, 0.151), M.icing, verts=24)
    cyl('icing', 0.112, 0.022, (0, 0, 0.207), M.icing, verts=24, bevel=0.008)
    for i in range(8):
        a = i * 45 + 22.5
        ball(f'swirl{i}', 0.012, (0.085 * G.cosd(a), 0.085 * G.sind(a), 0.222), M.icing, seg=8, rings=5)
    ball('cherry', 0.016, (0, 0, 0.234), M.red)
    G.rod('cherry_stalk', (0, 0, 0.248), (0.006, 0.004, 0.262), 0.002, material=M.stalk, verts=4)
    return done('cakeStand')


def _cup(name, x, y, z, material):
    lathe(f'{name}_saucer', [(0, z), (0.045, z + 0.002), (0.047, z + 0.009), (0.044, z + 0.01), (0, z + 0.006)],
          material, at=(x, y, 0), segments=12)
    lathe(name, [(0, z + 0.006), (0.02, z + 0.006), (0.031, z + 0.045), (0.027, z + 0.045), (0.018, z + 0.014),
                 (0, z + 0.014)], material, at=(x, y, 0), segments=12)
    cyl(f'{name}_tea', 0.027, 0.004, (x, y, z + 0.038), M.coffee, verts=12)
    G.torus(f'{name}_handle', 0.013, 0.004, at=(x - 0.032, y, z + 0.03), material=material, rot=(90, 0, 0),
            major_segments=10, minor_segments=4)


def teapotSet():
    box('tray', (0.4, 0.26, 0.012), (0, 0, 0.006), M.tray, bevel=0.004)
    for s in (-1, 1):
        box(f'tray_side{s}', (0.012, 0.26, 0.03), (s * 0.194, 0, 0.015), M.tray, bevel=0.003)
        box(f'tray_end{s}', (0.376, 0.012, 0.03), (0, s * 0.124, 0.015), M.tray, bevel=0.003)
    x, y = 0.07, 0.02
    ball('pot', 0.07, (x, y, 0.08), M.teapot, scale=(1, 1, 0.86), seg=14, rings=8)
    G.torus('pot_band', 0.07, 0.003, at=(x, y, 0.08), material=M.teapot_trim, major_segments=16, minor_segments=4)
    cyl('pot_lid', 0.036, 0.014, (x, y, 0.141), M.teapot, verts=14)
    ball('pot_knob', 0.012, (x, y, 0.155), M.teapot_trim, seg=8, rings=5)
    G.cone('spout', 0.016, 0.008, 0.08, at=(x - 0.083, y, 0.108), material=M.teapot, rot=(0, -50, 0), verts=10)
    G.torus('pot_handle', 0.034, 0.009, at=(x + 0.07, y, 0.093), material=M.teapot, rot=(90, 0, 0),
            major_segments=12, minor_segments=5)
    _cup('cup_a', -0.11, -0.06, 0.012, M.teapot)
    _cup('cup_b', -0.1, 0.07, 0.012, M.teapot)
    return done('teapotSet')


def _mug(name, x, y, turn, material):
    lathe(name, [(0, 0), (0.038, 0), (0.04, 0.004), (0.04, 0.09), (0.035, 0.09), (0.035, 0.012), (0, 0.012)],
          material, at=(x, y, 0), segments=16)
    cyl(f'{name}_coffee', 0.0355, 0.004, (x, y, 0.078), M.coffee, verts=14)
    G.torus(f'{name}_handle', 0.024, 0.007, at=(x + 0.042 * G.cosd(turn), y + 0.042 * G.sind(turn), 0.048),
            material=material, rot=(90, 0, turn), major_segments=12, minor_segments=5)


def mugs():
    _mug('mug_a', 0.068, -0.03, 150, M.mug_a)
    _mug('mug_b', -0.068, -0.04, 20, M.mug_b)
    _mug('mug_c', 0.0, 0.06, 80, M.mug_c)
    return done('mugs')


def _wine_glass(name, x, y):
    lathe(name, [(0, 0), (0.03, 0), (0.03, 0.004), (0.005, 0.008), (0.004, 0.075), (0.026, 0.095), (0.037, 0.125),
                 (0.033, 0.17), (0, 0.17)], M.glass, at=(x, y, 0), segments=14)
    lathe(f'{name}_wine', [(0, 0.083), (0.022, 0.093), (0.031, 0.115), (0.032, 0.122), (0, 0.122)], M.wine,
          at=(x, y, 0), segments=12)


def wineGlasses():
    lathe('bottle', [(0, 0), (0.038, 0), (0.04, 0.006), (0.04, 0.19), (0.03, 0.225), (0.014, 0.25),
                     (0.014, 0.3), (0, 0.3)], M.bottle, at=(0, 0.07, 0), segments=16)
    cyl('label', 0.0405, 0.07, (0, 0.07, 0.11), M.paper, verts=16)
    cyl('foil', 0.0155, 0.045, (0, 0.07, 0.278), M.wine, verts=10)
    _wine_glass('glass_a', -0.07, -0.04)
    _wine_glass('glass_b', 0.07, -0.05)
    return done('wineGlasses')


def candlesticks():
    for i, (x, h) in enumerate(((0.05, 0.18), (-0.05, 0.14))):
        lathe(f'stick{i}', [(0, 0), (0.034, 0), (0.034, 0.006), (0.024, 0.016), (0.009, 0.03), (0.008, h * 0.45),
                            (0.016, h * 0.5), (0.008, h * 0.56), (0.008, h - 0.022), (0.022, h - 0.012),
                            (0.022, h), (0, h)], M.brass, at=(x, 0, 0), segments=14)
        cyl(f'candle{i}', 0.011, 0.14, (x, 0, h + 0.07), M.candle, verts=10)
        G.rod(f'wick{i}', (x, 0, h + 0.14), (x, 0, h + 0.148), 0.0015, material=M.dark, verts=4)
        ball(f'flame{i}', 0.0075, (x, 0, h + 0.158), M.flame, scale=(1, 1, 2.0), seg=8, rings=5)
    return done('candlesticks')


# --- TT3 books and small plants -------------------------------------------------------------------------------
def _book(name, size, at, cover, turn=0.0):
    """A closed book lying flat, its spine to -x: the boards above and below and the spine, the pages
    between them showing on the other three sides."""
    w, d, t = size
    x, y, z = at
    c, s = G.cosd(turn), G.sind(turn)
    for k in (-1, 1):
        box(f'{name}_board{k}', (w, d, 0.003), (x, y, z + k * (t / 2 - 0.0015)), cover, rot=(0, 0, turn))
    box(f'{name}_spine', (0.004, d, t), (x - (w / 2 - 0.002) * c, y - (w / 2 - 0.002) * s, z), cover,
        bevel=0.0015, rot=(0, 0, turn))
    box(f'{name}_pages', (w - 0.008, d - 0.006, t - 0.006), (x - 0.002 * c, y - 0.002 * s, z), M.pages,
        rot=(0, 0, turn))


def bookStack():
    z = 0.0
    for i, (w, d, t, cover, turn, dx) in enumerate(((0.24, 0.17, 0.035, M.book_a, 0, 0.0),
                                                    (0.22, 0.16, 0.03, M.book_b, 10, 0.008),
                                                    (0.2, 0.15, 0.04, M.book_c, -7, -0.008),
                                                    (0.18, 0.13, 0.028, M.book_d, 18, 0.004))):
        _book(f'book{i}', (w, d, t), (dx, 0, z + t / 2), cover, turn)
        z += t
    box('ribbon', (0.006, 0.002, 0.03), (0.03, -0.067, z - 0.018), M.red, rot=(0, 0, 18))
    box('ribbon_top', (0.006, 0.02, 0.0015), (0.034, -0.057, z - 0.004), M.red, rot=(0, 0, 18))
    return done('bookStack')


def booksUpright():
    for s in (-1, 1):
        box(f'bookend{s}', (0.012, 0.12, 0.16), (s * 0.154, 0, 0.08), M.brass, bevel=0.003)
        box(f'bookend_foot{s}', (0.06, 0.12, 0.006), (s * 0.13, 0, 0.003), M.brass, bevel=0.002)
    covers = (M.book_a, M.book_c, M.book_b, M.book_e, M.book_d, M.book_c, M.book_a, M.book_b, M.book_e)
    x, k = -0.146, 6
    for i, cover in enumerate(covers):
        k = (k * 7 + 3) % 11
        w = 0.022 + (k % 4) * 0.007
        h = 0.2 * (0.72 + (k % 5) * 0.07)
        if x + w > 0.146:
            break
        box(f'book{i}', (w, 0.125, h), (x + w / 2, 0, 0.006 + h / 2), cover, bevel=0.002)
        box(f'book{i}_band', (w + 0.0005, 0.002, 0.008), (x + w / 2, -0.0625, 0.006 + h * 0.8), M.paper)
        x += w + 0.002
    return done('booksUpright')


def _flower(name, at, petal, centre, r=0.022, petals=5):
    x, y, z = at
    for i in range(petals):
        a = i * 360 / petals
        ball(f'{name}_petal{i}', r * 0.6, (x + r * 0.7 * G.cosd(a), y + r * 0.7 * G.sind(a), z), petal,
             scale=(1.1, 0.8, 0.45), rot=(0, 0, a), seg=6, rings=3)
    ball(f'{name}_centre', r * 0.4, (x, y, z + 0.003), centre, scale=(1, 1, 0.7), seg=6, rings=4)


def vaseFlowers():
    lathe('vase', [(0, 0), (0.042, 0), (0.06, 0.05), (0.065, 0.11), (0.052, 0.18), (0.032, 0.22), (0.034, 0.24),
                   (0.028, 0.24), (0, 0.23)], M.vase, segments=18)
    heads = ((0.0, 0.0, 0.40, M.white), (-0.06, 0.03, 0.35, M.blush), (0.06, -0.02, 0.36, M.yellow),
             (0.03, 0.06, 0.38, M.rose), (-0.04, -0.05, 0.34, M.white), (0.05, 0.04, 0.33, M.blush))
    for i, (x, y, z, petal) in enumerate(heads):
        G.rod(f'stem{i}', (x * 0.25, y * 0.25, 0.2), (x, y, z), 0.004, material=M.leaf, verts=5)
        _flower(f'flower{i}', (x, y, z), petal, M.yellow if petal is not M.yellow else M.orange, r=0.026)
    for i, a in enumerate((20, 140, 260)):
        ball(f'leaf{i}', 0.035, (0.04 * G.cosd(a), 0.04 * G.sind(a), 0.27), M.leaf, scale=(1.2, 0.45, 0.2),
             rot=(0, -35, a), seg=6, rings=4)
    return done('vaseFlowers')


def vaseSmall():
    lathe('vase', [(0, 0), (0.026, 0), (0.038, 0.035), (0.04, 0.06), (0.025, 0.12), (0.014, 0.15), (0.016, 0.16),
                   (0.011, 0.16), (0, 0.155)], M.bud_vase, segments=16)
    G.rod('stem', (0, 0, 0.14), (0.012, 0.004, 0.265), 0.003, material=M.leaf, verts=5)
    ball('leaf', 0.02, (0.012, 0.0, 0.2), M.leaf, scale=(1.2, 0.45, 0.25), rot=(0, -35, 0), seg=8, rings=5)
    _flower('flower', (0.012, 0.004, 0.268), M.white, M.yellow, r=0.022)
    return done('vaseSmall')


def potSucculent():
    lathe('pot', [(0, 0), (0.06, 0), (0.07, 0.07), (0.064, 0.07), (0.058, 0.06), (0, 0.06)], M.pot_pale,
          segments=18)
    for ring, (n, rad, length, lean, z, colour) in enumerate(((8, 0.05, 0.07, 60, 0.088, M.succulent),
                                                               (6, 0.016, 0.05, 30, 0.1, M.succulent_light))):
        for i in range(n):
            a = i * 360 / n + ring * 20
            x, y = rad * G.cosd(a), rad * G.sind(a)
            G.cone(f'leaf{ring}_{i}', 0.016, 0.004, length, at=(x, y, z + length * 0.3), material=colour,
                   rot=(0, lean, a), verts=6)
    ball('heart', 0.016, (0, 0, 0.115), M.succulent_light, scale=(1, 1, 1.2), seg=8, rings=5)
    return done('potSucculent')


def potCactus():
    lathe('pot', [(0, 0), (0.05, 0), (0.062, 0.09), (0.066, 0.1), (0.058, 0.1), (0.056, 0.09), (0, 0.09)],
          M.terracotta, segments=16)
    cyl('sand', 0.058, 0.008, (0, 0, 0.094), M.sand, verts=16)
    lathe('cactus', [(0, 0.09), (0.034, 0.09), (0.036, 0.2), (0.03, 0.28), (0.016, 0.305), (0, 0.31)], M.cactus,
          segments=12)
    for s, (h, z) in ((1, (0.06, 0.17)), (-1, (0.05, 0.2))):
        G.rod(f'arm{s}', (0, 0, z), (s * 0.076, 0, z), 0.016, material=M.cactus, verts=8)
        G.rod(f'arm_up{s}', (s * 0.076, 0, z), (s * 0.076, 0, z + h), 0.016, material=M.cactus, verts=8)
        ball(f'arm_tip{s}', 0.016, (s * 0.076, 0, z + h), M.cactus, seg=8, rings=5)
        ball(f'elbow{s}', 0.016, (s * 0.076, 0, z), M.cactus, seg=8, rings=5)
    _flower('flower', (0, 0, 0.316), M.pink, M.yellow, r=0.018)
    return done('potCactus')


def potSmall():
    lathe('pot', [(0, 0), (0.055, 0), (0.066, 0.11), (0.06, 0.11), (0.056, 0.1), (0, 0.1)], M.pot_white,
          segments=18)
    cyl('soil', 0.058, 0.008, (0, 0, 0.104), M.soil, verts=16)
    for i in range(9):
        a = i * 40
        up = 45 + (i % 3) * 15
        half = 0.046 + (i % 3) * 0.006
        out = 0.024 + half * G.cosd(up)
        ball(f'leaf{i}', half, (out * G.cosd(a), out * G.sind(a), 0.105 + half * G.sind(up) + (i % 3) * 0.012),
             M.leaf if i % 2 else M.leaf_light, scale=(1.0, 0.5, 0.12), rot=(0, -up, a), seg=8, rings=5)
    for i in range(3):
        a = i * 120 + 60
        ball(f'top{i}', 0.04, (0.012 * G.cosd(a), 0.012 * G.sind(a), 0.225), M.leaf_light,
             scale=(1.0, 0.4, 0.12), rot=(0, -70, a), seg=8, rings=5)
    return done('potSmall')


# --- TT4 small ornaments --------------------------------------------------------------------------------------
def photoFrame():
    TILT, ZC = -10, 0.116

    def leaned(name, size, y, z, material):
        dy, dz = tilt_x(y, z, TILT)
        box(name, size, (0, dy, ZC + dz), material, rot=(TILT, 0, 0))

    leaned('frame', (0.18, 0.02, 0.23), 0, 0, M.frame)
    leaned('mount', (0.15, 0.003, 0.2), -0.0105, 0, M.paper)
    leaned('photo_sky', (0.12, 0.003, 0.16), -0.012, 0.0, M.photo_sky)
    leaned('photo_hill', (0.12, 0.003, 0.05), -0.0125, -0.055, M.photo_hill)
    dy, dz = tilt_x(-0.0135, 0.01, TILT)
    ball('photo_head', 0.017, (0.0, dy, ZC + dz), M.blush, scale=(1, 0.15, 1.15), rot=(TILT, 0, 0), seg=10, rings=6)
    dy, dz = tilt_x(-0.0135, -0.045, TILT)
    ball('photo_body', 0.03, (0.0, dy, ZC + dz), M.teapot_trim, scale=(1.3, 0.12, 0.9), rot=(TILT, 0, 0), seg=10, rings=6)
    box('stand', (0.03, 0.012, 0.16), (0, 0.045, 0.075), M.frame, rot=(25, 0, 0))
    return done('photoFrame')


def tableClock():
    Z, R = 0.112, 0.074
    with G.new_parts() as body:
        cyl('case', R, 0.05, (0, 0, Z), M.clock, verts=24, rot=(90, 0, 0), bevel=0.008)
        cyl('face', R - 0.01, 0.004, (0, -0.025, Z), M.clock_face, verts=24, rot=(90, 0, 0))
        for i in range(12):
            a = i * 30
            box(f'tick{i}', (0.003, 0.002, 0.01 if i % 3 else 0.014),
                (0.054 * G.sind(a), -0.0275, Z + 0.054 * G.cosd(a)), M.dark, rot=(0, a, 0))
        cyl('hub', 0.004, 0.003, (0, -0.031, Z), M.red, verts=10, rot=(90, 0, 0))
        for s in (-1, 1):
            ball(f'bell{s}', 0.032, (s * 0.048, 0.0, Z + 0.083), M.brass, scale=(1, 1, 0.75), rot=(0, s * 30, 0))
            G.rod(f'leg{s}', (s * 0.045, 0.0, Z - 0.06), (s * 0.06, -0.008, 0.0), 0.005, material=M.brass, verts=6)
            ball(f'foot{s}', 0.008, (s * 0.06, -0.008, 0.004), M.brass, seg=8, rings=5)
        G.rod('bell_post', (0, 0.0, Z + R - 0.004), (0, 0.0, Z + 0.104), 0.0035, material=M.brass, verts=6)
        ball('hammer', 0.007, (0, 0.0, Z + 0.105), M.brass, seg=8, rings=5)
    G.join(body, 'clock', origin=(0, 0, 0))
    # the hour and minute hands: parts of their own round the face's middle, telling the game's time
    # (`motions.clock`), at rest ten past ten
    hands = []
    for name, (w, depth, length), y, angle in (('hour', (0.005, 0.002, 0.032), -0.029, 305), ('minute', (0.004, 0.002, 0.05), -0.03, 60)):
        with G.new_parts() as hand:
            box(f'{name}_hand', (w, depth, length), (G.sind(angle) * length / 2, y, Z + G.cosd(angle) * length / 2), M.dark,
                rot=(0, angle, 0))
        hands.append(G.hinged(G.join(hand, f'{name}_hand'), (0, y, Z)))
    G.settle()
    for hand in hands:
        G.spin(hand, hand.name, 1, seconds=12, axis='Y')   # once round clockwise; the game holds it at the time
    return G.meshes()


def globe():
    cyl('base', 0.068, 0.016, (0, 0, 0.008), M.wood_dark, verts=18, bevel=0.004)
    G.cone('column', 0.016, 0.01, 0.05, at=(0, 0, 0.04), material=M.wood_dark, verts=10)
    Z, R = 0.19, 0.105
    ball('sea', R, (0, 0, Z), M.globe, rot=(0, 23, 0), seg=20, rings=12)
    for i, (lon, lat, size, seed) in enumerate(((-10, 20, 0.042, 0.3), (40, -15, 0.035, 1.4), (100, 30, 0.045, 2.2),
                                                (-80, 35, 0.04, 3.1), (-60, -20, 0.032, 4.5))):
        x = R * G.cosd(lat) * G.sind(lon)
        y = -R * G.cosd(lat) * G.cosd(lon)
        z = R * G.sind(lat)
        G.boulder(f'land{i}', size, at=(0, 0, 0), material=M.land, scale=(1, 1, 0.12), seed=seed, noise=0.4,
                  subdivisions=1)
        land = G.meshes()[-1]
        land.rotation_euler = (math.radians(90 - lat), 0, math.radians(lon))
        land.location = (x * 0.93, y * 0.93, Z + z * 0.93)
    G.torus('meridian', 0.116, 0.004, at=(0, 0, Z), material=M.brass, rot=(90, 0, 90 + 0), major_segments=28,
            minor_segments=4)
    meridian = G.meshes()[-1]
    meridian.rotation_euler = (math.radians(90), math.radians(23), math.radians(90))
    return done('globe')


def snowGlobe():
    lathe('base', [(0, 0), (0.064, 0), (0.064, 0.008), (0.056, 0.05), (0, 0.05)], M.wood_dark, segments=18)
    cyl('band', 0.06, 0.008, (0, 0, 0.03), M.brass, verts=18)
    Z = 0.112
    ball('dome', 0.064, (0, 0, Z), M.glass, seg=18, rings=10)
    cyl('ground', 0.044, 0.012, (0, 0, 0.07), M.snow, verts=16)
    G.cone('tree', 0.026, 0.0, 0.07, at=(0, 0, 0.108), material=M.fir, verts=8)
    G.cone('tree_top', 0.018, 0.0, 0.04, at=(0, 0, 0.13), material=M.fir, verts=8)
    for i, (x, y, z) in enumerate(((0.03, -0.02, 0.13), (-0.035, 0.0, 0.15), (0.01, 0.03, 0.16), (-0.02, -0.03, 0.1),
                                   (0.035, 0.02, 0.095))):
        ball(f'flake{i}', 0.004, (x, y, z), M.snow, seg=6, rings=4)
    return done('snowGlobe')


def trophy():
    box('base', (0.1, 0.1, 0.04), (0, 0, 0.02), M.trophy_base, bevel=0.005)
    box('plaque', (0.06, 0.002, 0.018), (0, -0.0505, 0.02), M.trophy)
    lathe('stem', [(0, 0.04), (0.03, 0.04), (0.03, 0.046), (0.01, 0.06), (0.009, 0.09), (0.018, 0.1),
                   (0.012, 0.11), (0, 0.11)], M.trophy, segments=14)
    lathe('cup', [(0, 0.105), (0.02, 0.108), (0.05, 0.14), (0.062, 0.2), (0.064, 0.226), (0.059, 0.226),
                  (0.056, 0.2), (0.045, 0.15), (0, 0.135)], M.trophy, segments=18)
    for s in (-1, 1):
        G.torus(f'handle{s}', 0.016, 0.005, at=(s * 0.053, 0, 0.175), material=M.trophy, rot=(90, 0, 0),
                major_segments=12, minor_segments=5)
    return done('trophy')


def sculptureSmall():
    box('plinth', (0.12, 0.12, 0.05), (0, 0, 0.025), M.plinth, bevel=0.004)
    G.torus('ring', 0.07, 0.017, at=(0, 0, 0.05 + 0.07 + 0.012), material=M.sculpture, rot=(90, 0, 30),
            major_segments=24, minor_segments=8)
    ball('ball', 0.03, (0.012, -0.01, 0.08), M.brass, seg=14, rings=9)
    return done('sculptureSmall')


# --- TT5 bath, desk and toys ----------------------------------------------------------------------------------
def soapSet():
    box('tray', (0.26, 0.12, 0.012), (0, 0, 0.006), M.soap_tray, bevel=0.004)
    lathe('dispenser', [(0, 0.012), (0.03, 0.012), (0.031, 0.02), (0.031, 0.11), (0.024, 0.124), (0, 0.124)],
          M.dispenser, at=(0.08, 0, 0), segments=14)
    cyl('pump_collar', 0.014, 0.01, (0.08, 0, 0.128), M.dark, verts=10)
    cyl('pump', 0.005, 0.02, (0.08, 0, 0.142), M.dark, verts=8)
    box('nozzle', (0.012, 0.03, 0.01), (0.08, -0.012, 0.155), M.dark, bevel=0.003)
    lathe('tumbler', [(0, 0.012), (0.027, 0.012), (0.031, 0.09), (0.027, 0.09), (0.024, 0.018), (0, 0.018)],
          M.tumbler, at=(-0.02, 0, 0), segments=14)
    for i, (colour, top) in enumerate(((M.blush, (-0.0, 0.01, 0.165)), (M.leaf_light, (-0.04, -0.012, 0.16)))):
        G.rod(f'brush{i}', (-0.02, 0, 0.03), top, 0.005, material=colour, verts=6)
        box(f'bristles{i}', (0.008, 0.01, 0.022), (top[0], top[1] - 0.006, top[2] - 0.01), M.white, bevel=0.002)
    box('soap', (0.07, 0.05, 0.024), (-0.09, 0, 0.024), M.soap, bevel=0.01, segs=2)
    return done('soapSet')


def towelStack():
    for i, (colour, dy, turn) in enumerate(((M.towel_a, 0.0, 0), (M.towel_b, -0.006, 3), (M.towel_c, 0.004, -2))):
        z = 0.025 + i * 0.05
        box(f'towel{i}', (0.26, 0.18, 0.05), (0, dy, z), colour, bevel=0.018, segs=2, rot=(0, 0, turn))
        box(f'fold{i}', (0.25, 0.006, 0.012), (0, dy - 0.0905, z + 0.006), colour, bevel=0.003, rot=(0, 0, turn))
    return done('towelStack')


def perfumeTray():
    box('tray', (0.26, 0.16, 0.01), (0, 0, 0.005), M.perfume_tray)
    for s in (-1, 1):
        box(f'rim_side{s}', (0.008, 0.168, 0.016), (s * 0.134, 0, 0.008), M.brass, bevel=0.002)
        box(f'rim_end{s}', (0.26, 0.008, 0.016), (0, s * 0.084, 0.008), M.brass, bevel=0.002)
    box('bottle_a', (0.06, 0.04, 0.09), (0.07, 0.0, 0.055), M.perfume_a, bevel=0.008, segs=2)
    cyl('neck_a', 0.008, 0.012, (0.07, 0, 0.106), M.brass, verts=8)
    ball('cap_a', 0.017, (0.07, 0, 0.126), M.brass, seg=10, rings=6)
    lathe('bottle_b', [(0, 0.01), (0.03, 0.01), (0.03, 0.06), (0.012, 0.075), (0.01, 0.09), (0, 0.09)], M.perfume_b,
          at=(-0.02, -0.02, 0), segments=14)
    cyl('cap_b', 0.013, 0.02, (-0.02, -0.02, 0.098), M.dark, verts=10)
    cyl('bottle_c', 0.025, 0.07, (-0.08, 0.03, 0.045), M.perfume_c, verts=14)
    cyl('cap_c', 0.015, 0.024, (-0.08, 0.03, 0.092), M.dark, verts=10)
    return done('perfumeTray')


def deskOrganiser():
    lathe('pen_pot', [(0, 0), (0.035, 0), (0.035, 0.1), (0.031, 0.1), (0.031, 0.006), (0, 0.006)], M.organiser,
          at=(0.09, 0.02, 0), segments=14)
    for i, (dx, dy, colour) in enumerate(((0.015, 0.0, M.pen_red), (-0.012, 0.01, M.pen_blue),
                                          (0.0, -0.015, M.pen_yellow))):
        G.rod(f'pen{i}', (0.09, 0.02, 0.01), (0.09 + dx, 0.02 + dy, 0.16), 0.0045, material=colour, verts=6)
    for tier in range(2):
        z = tier * 0.05
        box(f'shelf{tier}', (0.16, 0.2, 0.008), (-0.06, 0, z + 0.004), M.organiser)
        box(f'paper{tier}', (0.14, 0.19, 0.008), (-0.06, 0.003, z + 0.012), M.paper)
        for s in (-1, 1):
            box(f'side{tier}{s}', (0.006, 0.2, 0.03), (-0.06 + s * 0.077, 0, z + 0.019), M.organiser)
        box(f'back{tier}', (0.16, 0.006, 0.03), (-0.06, 0.097, z + 0.019), M.organiser)
    for s in (-1, 1):
        box(f'post{s}', (0.006, 0.006, 0.05), (-0.06 + s * 0.077, -0.097, 0.045), M.organiser)
    return done('deskOrganiser')


def laptopClosed():
    with G.new_parts() as body:
        box('base', (0.32, 0.22, 0.011), (0, 0, 0.0055), M.laptop, bevel=0.005, segs=2)
        box('hinge', (0.26, 0.008, 0.008), (0, 0.108, 0.013), M.dark, bevel=0.003)
        box('notch', (0.04, 0.003, 0.003), (0, -0.1095, 0.0115), M.dark)
    G.join(body, 'laptop', origin=(0, 0, 0))
    # the lid: a part of its own, hinged at its back edge, for a use to come (it opens)
    with G.new_parts() as lid:
        box('lid', (0.318, 0.218, 0.008), (0, 0, 0.0155), M.laptop, bevel=0.004, segs=2)
        cyl('logo', 0.015, 0.001, (0, 0.01, 0.02), M.logo, verts=14)
    G.hinged(G.join(lid, 'lid'), (0, 0.109, 0.0115))
    return G.settle()


def monitor():
    box('foot', (0.2, 0.14, 0.012), (0, 0.08, 0.006), M.charcoal, bevel=0.005)
    box('neck', (0.05, 0.025, 0.2), (0, 0.1, 0.11), M.charcoal, bevel=0.008)
    box('bezel', (0.54, 0.025, 0.32), (0, 0.075, 0.3), M.monitor, bevel=0.006)
    box('screen', (0.52, 0.002, 0.29), (0, 0.0615, 0.305), M.screen)
    box('chin_logo', (0.03, 0.002, 0.006), (0, 0.0615, 0.152), M.chrome)
    box('keyboard', (0.38, 0.12, 0.014), (0, -0.12, 0.007), M.monitor, bevel=0.004)
    for row in range(4):
        box(f'keys{row}', (0.35, 0.02, 0.004), (0, -0.155 + row * 0.024, 0.016), M.keys)
    box('space', (0.13, 0.016, 0.004), (0, -0.177, 0.016), M.keys)
    ball('mouse', 0.028, (0.25, -0.12, 0.0154), M.monitor, scale=(0.8, 1.3, 0.55), seg=12, rings=7)
    return done('monitor', mirror=True)


def stuffedToy():
    ball('body', 0.075, (0, 0, 0.08), M.bear, scale=(1, 0.9, 1.05), seg=12, rings=8)
    ball('tummy', 0.048, (0, -0.05, 0.075), M.bear_light, scale=(1, 0.45, 1.1), seg=8, rings=6)
    ball('head', 0.062, (0, -0.01, 0.205), M.bear, seg=12, rings=8)
    ball('muzzle', 0.026, (0, -0.064, 0.193), M.bear_light, scale=(1.1, 0.8, 0.8), seg=8, rings=6)
    ball('nose', 0.009, (0, -0.085, 0.2), M.dark, scale=(1.3, 1, 1), seg=6, rings=4)
    for s in (-1, 1):
        ball(f'eye{s}', 0.007, (s * 0.024, -0.064, 0.222), M.dark, seg=6, rings=4)
        ball(f'ear{s}', 0.024, (s * 0.047, 0.0, 0.258), M.bear, scale=(1, 0.55, 1), seg=8, rings=6)
        ball(f'ear_in{s}', 0.014, (s * 0.047, -0.01, 0.256), M.bear_light, scale=(1, 0.4, 1), seg=6, rings=4)
        ball(f'arm{s}', 0.026, (s * 0.07, -0.035, 0.1), M.bear, scale=(1, 1, 1.9), rot=(-30, -s * 20, 0), seg=8, rings=6)
        ball(f'leg{s}', 0.03, (s * 0.048, -0.065, 0.03), M.bear, scale=(1, 1.9, 1), rot=(0, 0, s * 15), seg=8, rings=6)
        ball(f'pad{s}', 0.02, (s * 0.056, -0.118, 0.032), M.bear_light, scale=(1, 0.35, 1.1), seg=6, rings=4)
        ball(f'bow{s}', 0.016, (s * 0.018, -0.054, 0.15), M.red, scale=(1.3, 0.5, 0.9), seg=6, rings=4)
    return done('stuffedToy')


def toyBlocks():
    B = 0.065
    for i, (x, y, z, colour, turn) in enumerate(((-0.05, 0.02, 0, M.block_a, 10), (0.03, 0.03, 0, M.block_b, -15),
                                                 (-0.01, 0.02, 1, M.block_c, 30), (0.08, -0.06, 0, M.block_d, 20),
                                                 (-0.07, -0.07, 0, M.block_e, 0))):
        box(f'block{i}', (B, B, B), (x, y, B / 2 + z * B), colour, bevel=0.006, rot=(0, 0, turn))
    # a roof on the top block
    G.cone('roof', B / math.sqrt(2), 0.0, 0.07, at=(-0.01, 0.02, 2 * B + 0.035), material=M.block_d,
           rot=(0, 0, 45 + 30), verts=4, smooth=False)
    return done('toyBlocks')


def T(id, build, size, paint, **kw):
    return G.Model(id, build, size=size, key=f'model.{id}', paint=paint, **kw)


BODY = 'Body'
FAMILY = G.Family(
    'tabletop', M,
    [  # TT1 kitchenware
        T('kettle', kettle, (0.228, 0.238, 0.172), [('kettle', BODY), ('dark', 'Base, handle & lid')]),
        T('toaster', toaster, (0.28, 0.23, 0.15), [('toaster', BODY)]),
        T('blender', blender, (0.18, 0.356, 0.146), [('blender_base', BODY), ('smoothie', 'Smoothie')]),
        T('standMixer', standMixer, (0.3, 0.356, 0.17), [('mixer', BODY)]),
        T('espressoMachineSmall', espressoMachineSmall, (0.231, 0.31, 0.391), [('espresso', BODY)]),
        T('knifeBlock', knifeBlock, (0.12, 0.297, 0.232), [('knife_block', BODY), ('dark', 'Handles')]),
        T('utensilCrock', utensilCrock, (0.132, 0.316, 0.124), [('crock', BODY)]),
        T('cuttingBoard', cuttingBoard, (0.34, 0.106, 0.23), [('board', BODY)]),
        T('breadBin', breadBin, (0.34, 0.206, 0.202), [('bread_bin', BODY), ('wood', 'Ends & handle')]),
        T('spiceJars', spiceJars, (0.3, 0.112, 0.12), [('rack', BODY)]),
        T('cookieJar', cookieJar, (0.16, 0.248, 0.158), [('jar_lid', 'Lid')]),
        # TT2 tableware
        T('fruitBowl', fruitBowl, (0.28, 0.194, 0.28), [('fruit_bowl', BODY)]),
        T('plateSet', plateSet, (0.378, 0.1, 0.321), [('plate', BODY), ('plate_rim', 'Rim'), ('napkin', 'Napkin')]),
        T('cakeStand', cakeStand, (0.28, 0.263, 0.28), [('cake_stand', BODY), ('icing', 'Icing')]),
        T('teapotSet', teapotSet, (0.4, 0.167, 0.26), [('teapot', BODY), ('tray', 'Tray')]),
        T('mugs', mugs, (0.216, 0.09, 0.212), [('mug_a', 'Mug 1'), ('mug_b', 'Mug 2'), ('mug_c', 'Mug 3')]),
        T('wineGlasses', wineGlasses, (0.214, 0.301, 0.197), [('bottle', BODY)]),
        T('candlesticks', candlesticks, (0.168, 0.353, 0.066), [('brass', BODY), ('candle', 'Candles')]),
        # TT3 books and small plants
        T('bookStack', bookStack, (0.25, 0.133, 0.196),
          [('book_a', 'Book 1'), ('book_b', 'Book 2'), ('book_c', 'Book 3'), ('book_d', 'Book 4')]),
        # each cover colour, left to right as they first show (eight books: Book 1-3 colour two each)
        T('booksUpright', booksUpright, (0.32, 0.206, 0.126),
          [('brass', 'Bookends'), ('book_a', 'Book 1'), ('book_c', 'Book 2'), ('book_b', 'Book 3'),
           ('book_e', 'Book 4'), ('book_d', 'Book 5')]),
        T('vaseFlowers', vaseFlowers, (0.179, 0.41, 0.172), [('vase', BODY)]),
        T('vaseSmall', vaseSmall, (0.08, 0.277, 0.08), [('bud_vase', BODY)]),
        T('potSucculent', potSucculent, (0.164, 0.136, 0.164), [('pot_pale', BODY)]),
        T('potCactus', potCactus, (0.184, 0.324, 0.132), [('terracotta', BODY)]),
        T('potSmall', potSmall, (0.16, 0.261, 0.154), [('pot_white', BODY)]),
        # TT4 small ornaments
        T('photoFrame', photoFrame, (0.18, 0.231, 0.114), [('frame', BODY)]),
        T('tableClock', tableClock, (0.153, 0.228, 0.063), [('clock', BODY), ('brass', 'Bells & legs')],
          motions={'clock': {'hour': 'hour_hand', 'minute': 'minute_hand', 'at': 10 * 60 + 10}}),
        T('globe', globe, (0.212, 0.31, 0.24), [('globe', BODY), ('wood_dark', 'Stand')]),
        T('snowGlobe', snowGlobe, (0.128, 0.176, 0.128), [('wood_dark', BODY)]),
        T('trophy', trophy, (0.148, 0.226, 0.126), [('trophy', BODY)]),
        T('sculptureSmall', sculptureSmall, (0.154, 0.219, 0.12), [('sculpture', BODY), ('brass', 'Ball')]),
        # TT5 bath, desk and toys
        T('soapSet', soapSet, (0.26, 0.166, 0.12), [('soap_tray', BODY), ('dispenser', 'Dispenser')]),
        T('towelStack', towelStack, (0.267, 0.15, 0.204), [('towel_a', 'Towel 1'), ('towel_b', 'Towel 2'),
                                                       ('towel_c', 'Towel 3')]),
        T('perfumeTray', perfumeTray, (0.276, 0.143, 0.176), [('perfume_tray', BODY)]),
        T('deskOrganiser', deskOrganiser, (0.265, 0.16, 0.2), [('organiser', BODY)]),
        T('laptopClosed', laptopClosed, (0.32, 0.0205, 0.223), [('laptop', BODY)]),
        T('monitor', monitor, (0.542, 0.46, 0.335), [('monitor', BODY)]),
        T('stuffedToy', stuffedToy, (0.197, 0.282, 0.192), [('bear', BODY)]),
        T('toyBlocks', toyBlocks, (0.222, 0.2, 0.171), [('block_a', 'Block 1'), ('block_b', 'Block 2'),
                                                       ('block_c', 'Block 3')]),
    ],
    comment='Blender-built tabletop things (tools/art/blender/families/tabletop.py). Replaces the primitive-part '
            'placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a '
            'file fails to load, and the materials map onto the game\'s finishes, so each costs one draw call (the '
            'see-through ones and the parts kept for a use to come one more, the alarm clock\'s two hands two more).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
