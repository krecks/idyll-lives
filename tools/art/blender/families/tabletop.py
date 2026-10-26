# The tabletop things: small things that stand on tops (the `surface` layer). Notes: tabletop.md.
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
# Each stands on its base, centred (the renderer lifts it to its host's top), at the old part's size. Most
# have no clips, and every finish is a plain one (wood, fabric and leaves drawn satin or matte),
# so each costs one draw call. The exceptions: the cookie jar, the wine glasses and the snow globe are
# see-through `glass` (two); the kettle's lid, the bread bin's roll-top and the laptop's lid are parts of
# their own, hinged where they open, and open for their uses (two: `lid_open` for tea, `roll_open` for a
# sandwich, the laptop's `lid_open` while it is used); the candlesticks' flames rise out of the candles
# at night and glow (`motions.night`; two); the alarm clock's hour and minute hands tell the game's time
# (`motions.clock`; three).
#
# They are placed many times on every lot, so each keeps to half its size class's target (objects.md): 750
# triangles under 0.35 m, 2,000 for the five larger ones. Turned things are lofted rings (`turned`: few sides,
# rims kept sharp), handles and spouts swept, slabs extruded with their holes, the plants' leaves and flowers
# plants.py's (`leaf`, `rays`, `tulip_head`).
#
# Run:  cargo art tabletop [--only kettle] [--preview]
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import cupped, dome, leaf, rays, ribbed, soil_top, star, tulip_head, twig  # noqa: E402  (leaves, flowers)

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
    crock_band=('#4E6E8A', 'gloss'),
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
    tomato_flesh=('#EE7A63', 'satin'),
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
    flame=('#FFD27A', 'gloss', {'glow': '#FFC46A'}),   # gloss, never emissive; it glows at night (lit)
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
    laptop_screen=('#1F2A36', 'gloss', {'glow': '#A9C8E8', 'screen': True}),   # the laptop's screen: lit while it's used
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
    to right, for one built the other way round from its old part."""
    obj = G.join_all(name)
    if mirror:
        obj.data.transform(Matrix.Scale(-1, 4, Vector((1, 0, 0))))
        obj.data.flip_normals()
    return G.settle([obj])


def turned(name, rows, material, at=(0, 0, 0), verts=16, rot=(0, 0, 0), square=2.0, caps=True, angle=None):
    """A turned (lathed) body through rows of (r, z), or (rx, ry, z) for an oval or (`square`) a rounded square,
    bottom first, lofted with its rims kept sharp (edges turning over `angle`; by default a little more than its
    sides turn, so a body of few sides is still round): a radius of 0 closes it to a point there. Built about z,
    then placed at `at` and turned `rot` (degrees), so it can lie along another axis."""
    if angle is None:
        angle = max(40, 360 / verts + 8) if square <= 2.5 else 40
    secs = []
    for row in rows:
        rx, ry, z = (row[0], row[0], row[1]) if len(row) == 2 else row
        secs.append(G.ring(rx, ry, z, verts, square=square))
    return G.loft(name, secs, material=material, caps=caps, at=at, rot=rot, angle=angle)


def disc(name, r, at, material, verts=12, rx=None):
    """A flat face looking up (a drink's surface, a slice's cut face): one polygon, `verts` corners (seen from
    above only, so no underside)."""
    x, y, z = at
    pts = [(x + (rx or r) * math.cos(math.tau * i / verts), y + r * math.sin(math.tau * i / verts), z)
           for i in range(verts)]
    return G.forms._object(name, pts, [tuple(range(verts))], material, False, 40)


def bar(name, points, profile, material, scale=1.0, up=(0, 0, 1), resolution=1, angle=60):
    """A profile swept through Blender points, its corners as given (round them with `G.kit.fillet` first)."""
    return G.sweep(name, points, None, profile=profile, material=material, scale=scale, up=up, resolution=resolution,
                   angle=angle)


def oval(w, t, verts=8):
    """An oval profile `w` across its `u` axis and `t` across `v` (a flat handle, a spoon's stem)."""
    return [(w / 2 * math.cos(math.tau * i / verts), t / 2 * math.sin(math.tau * i / verts)) for i in range(verts)]


def slab(name, outline, depth, at, material, plane='XZ', rot=(0, 0, 0), bevel=0.0, holes=()):
    """`G.extrude` with one bevel segment (small things: the corners only need catching the light)."""
    return G.extrude(name, outline, depth, at=at, material=material, plane=plane, rot=rot, bevel=bevel,
                     bevel_segments=1, holes=holes)


# --- TT1 kitchenware ------------------------------------------------------------------------------------------
def kettle():
    """A cordless jug kettle on its power base: a body swelling to its shoulder, a beak spout, a D handle in one
    piece with its switch, a water window; the lid a part of its own, hinged on the handle's side: it flips up while
    the kettle is filled for tea (`lid_open`)."""
    with G.new_parts() as body:
        turned('base', [(0.083, 0.0), (0.086, 0.004), (0.086, 0.013), (0.08, 0.018)], M.dark, verts=16)
        turned('body', [(0.074, 0.016), (0.079, 0.026), (0.077, 0.1), (0.068, 0.178), (0.054, 0.203),
                        (0.049, 0.211)], M.kettle, verts=16)
        # the spout: a beak drawn out of the shoulder, flattening and narrowing to its lip
        bar('spout', [(-0.045, 0, 0.172), (-0.078, 0, 0.19), (-0.1, 0, 0.205), (-0.108, 0, 0.212)],
            oval(0.03, 0.044, 8), M.kettle, scale=[1.0, 0.8, 0.55, 0.45], resolution=2)
        # the handle in one piece from the shoulder round to the hip, the switch at its foot
        bar('handle', G.kit.fillet([(0.045, 0, 0.198), (0.1, 0, 0.198), (0.104, 0, 0.075), (0.06, 0, 0.068)], 0.024,
                                   steps=2), oval(0.017, 0.026, 6), M.dark, up=(0, 1, 0))
        box('switch', (0.016, 0.014, 0.008), (0.09, 0, 0.08), M.red)
        slab('window', G.rounded_rect(0.02, 0.1, 0.009, corner=2), 0.006, (0.043, -0.062, 0.1), M.glass_pale,
             rot=(0, 0, 35))
    G.join(body, 'kettle', origin=(0, 0, 0))
    with G.new_parts() as lid:
        turned('lid', [(0.049, 0.209), (0.05, 0.214), (0.046, 0.221), (0.03, 0.225), (0.0, 0.226)], M.dark, verts=14)
        turned('lid_knob', [(0.009, 0.224), (0.011, 0.231), (0.009, 0.236), (0.0, 0.238)], M.dark, at=(-0.014, 0, 0),
               verts=8)
    lid = G.hinged(G.join(lid, 'lid'), (0.046, 0, 0.223))
    G.settle()
    G.swing(lid, 'lid_open', 80, seconds=0.4, axis='Y')   # its spout side up, about the hinge by the handle
    return G.meshes()


def _slice(name, at, material, inner, w=0.13, h=0.075):
    """A slice of bread standing in a slot, facing the front: a crust round a paler crumb face, its top two soft
    shoulders (only the top shows)."""
    pts = [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h * 0.16), (w * 0.47, h * 0.4), (w * 0.33, h / 2),
           (0.0, h * 0.44), (-w * 0.33, h / 2), (-w * 0.47, h * 0.4), (-w / 2, h * 0.16)]
    slab(name, pts, 0.014, at, material)
    slab(f'{name}_crumb', [(u * 0.88, v * 0.88 - h * 0.03) for u, v in pts], 0.0146, at, inner)


def toaster():
    """A two-slice toaster: a rounded body on a dark plinth, two slots with a slice of toast up in each (a crust round
    its crumb), the lever in its guide, a browning dial and a cancel button."""
    turned('plinth', [(0.122, 0.068, 0.0), (0.125, 0.071, 0.004), (0.125, 0.071, 0.013)], M.dark, verts=16, square=6)
    turned('body', [(0.125, 0.07, 0.012), (0.13, 0.075, 0.022), (0.13, 0.075, 0.15), (0.125, 0.07, 0.166),
                    (0.114, 0.06, 0.172)], M.toaster, verts=20, square=5)
    for s in (-1, 1):
        slab(f'slot{s}', G.rounded_rect(0.17, 0.024, 0.01, corner=2), 0.006, (0, s * 0.035, 0.171), M.dark, plane='XY')
        _slice(f'toast{s}', (0, s * 0.035, 0.192), M.bread, M.crumb)
    box('guide', (0.004, 0.012, 0.1), (-0.13, 0, 0.1), M.dark)
    box('lever', (0.022, 0.03, 0.014), (-0.14, 0, 0.125), M.dark, bevel=0.004)
    turned('knob', [(0.016, 0.0), (0.016, 0.006), (0.012, 0.011), (0.0, 0.012)], M.chrome, at=(-0.129, 0.03, 0.055),
           rot=(0, -90, 0), verts=12)
    box('pointer', (0.004, 0.003, 0.012), (-0.1415, 0.03, 0.062), M.dark)
    box('cancel', (0.006, 0.016, 0.01), (-0.132, -0.03, 0.055), M.chrome, bevel=0.002)
    for s in (-1, 1):
        box(f'trim{s}', (0.2, 0.002, 0.008), (0, s * 0.0755, 0.05), M.chrome)
    return done('toaster')


def blender():
    """A jug blender: a tapering base with a dial and two buttons, the collar, a soft-square jug (the smoothie its
    lower half, the jug drawn opaque), its handle in one piece, the lid and its filler cap."""
    turned('base', [(0.068, 0.068, 0.0), (0.07, 0.07, 0.006), (0.066, 0.066, 0.085), (0.06, 0.06, 0.1)], M.blender_base,
           verts=16, square=4)
    turned('dial', [(0.016, 0.0), (0.016, 0.005), (0.012, 0.008), (0.0, 0.009)], M.chrome, at=(0, -0.063, 0.05),
           rot=(90, 0, 0), verts=12)
    for x in (-0.038, 0.038):
        box(f'button{x}', (0.016, 0.008, 0.01), (x, -0.065, 0.05), M.chrome, bevel=0.002)
    turned('collar', [(0.056, 0.098), (0.06, 0.104), (0.06, 0.118), (0.056, 0.122)], M.dark, verts=16)
    turned('smoothie', [(0.05, 0.05, 0.12), (0.058, 0.058, 0.22)], M.smoothie, verts=12, square=3)
    turned('jug', [(0.058, 0.058, 0.22), (0.068, 0.068, 0.318)], M.jug, verts=12, square=3)
    turned('lid', [(0.068, 0.068, 0.316), (0.072, 0.072, 0.322), (0.072, 0.072, 0.334), (0.064, 0.064, 0.34)], M.dark,
           verts=12, square=3)
    turned('cap', [(0.022, 0.338), (0.022, 0.35), (0.018, 0.355), (0.0, 0.356)], M.dark, verts=10)
    bar('handle', G.kit.fillet([(-0.05, 0, 0.302), (-0.098, 0, 0.302), (-0.098, 0, 0.18), (-0.05, 0, 0.18)], 0.022,
                               steps=2), oval(0.018, 0.026, 6), M.dark, up=(0, 1, 0))
    return done('blender')


def standMixer():
    """A tilt-head stand mixer: a rounded base, the column rising into the head (lofted, swelling over the bowl to
    its nose, a chrome band and hub there), the speed lever, the steel bowl with batter in it and the beater's
    shaft."""
    turned('base', [(0.15, 0.085, 0.0), (0.15, 0.085, 0.028), (0.142, 0.078, 0.04)], M.mixer, verts=16, square=5)
    turned('column', [(0.05, 0.056, 0.035), (0.042, 0.05, 0.14), (0.047, 0.056, 0.25)], M.mixer, at=(0.1, 0, 0),
           verts=12, square=3)
    # the head lies along x, its back over the column and its nose over the bowl (rows: up, across, along)
    turned('head', [(0.024, 0.026, 0.0), (0.05, 0.052, 0.012), (0.064, 0.066, 0.05), (0.066, 0.068, 0.14),
                    (0.06, 0.062, 0.21), (0.047, 0.049, 0.255), (0.028, 0.03, 0.282), (0.0, 0.0, 0.29)], M.mixer,
           at=(0.145, 0, 0.29), rot=(0, -90, 0), verts=12)
    turned('band', [(0.0545, 0.0565, 0.226), (0.0505, 0.0525, 0.25)], M.chrome, at=(0.145, 0, 0.29), rot=(0, -90, 0),
           verts=12, caps=False)
    turned('hub', [(0.016, 0.0), (0.016, 0.004), (0.0, 0.006)], M.chrome, at=(-0.143, 0, 0.29), rot=(0, -90, 0), verts=10)
    turned('speed', [(0.009, 0.0), (0.008, 0.012), (0.0, 0.014)], M.chrome, at=(0.1, -0.052, 0.2), rot=(90, 0, 0),
           verts=8)
    turned('bowl', [(0.042, 0.04), (0.06, 0.052), (0.074, 0.075), (0.08, 0.12), (0.082, 0.148), (0.078, 0.152),
                    (0.075, 0.122)], M.steel, at=(-0.07, 0, 0), verts=14)
    turned('batter', [(0.0752, 0.12), (0.05, 0.128), (0.0, 0.132)], M.crumb, at=(-0.07, 0, 0), verts=10)
    G.rod('shaft', (-0.07, 0, 0.232), (-0.07, 0, 0.125), 0.006, material=M.steel, verts=6)
    slab('beater', [(-0.03, 0.03), (0.03, 0.03), (0.022, -0.01), (0.0, -0.03), (-0.022, -0.01)], 0.005,
         (-0.07, 0, 0.11), M.steel, holes=[[(-0.016, 0.022), (0.016, 0.022), (0.01, -0.004), (0.0, -0.016),
                                              (-0.01, -0.004)]])
    return done('standMixer')


def espressoMachineSmall():
    """A small espresso machine: a brushed-steel body rounded over its top, the head over the cup with its buttons
    and light, the group and portafilter (a black handle on a steel basket, two spouts), a slotted grate over the drip
    tray, a pressure gauge in a chrome bezel, the steam knob and a bent wand, the water tank at the back, a rail round
    the cup warmer and a cup upturned on it."""
    turned('body', [(0.11, 0.096, 0.0), (0.11, 0.096, 0.255), (0.104, 0.09, 0.267), (0.094, 0.08, 0.27)], M.espresso,
           at=(0, 0.026, 0), verts=20, square=8)
    slab('head', G.rounded_rect(0.085, 0.07, 0.022, corner=2), 0.2, (0, -0.0925, 0.235), M.espresso, plane='YZ',
         bevel=0.006)
    for i, x in enumerate((-0.06, -0.035)):
        turned(f'button{i}', [(0.008, 0.0), (0.008, 0.004), (0.0, 0.006)], M.chrome, at=(x, -0.134, 0.248),
               rot=(90, 0, 0), verts=10)
    turned('light', [(0.004, 0.0), (0.0, 0.003)], M.red, at=(0.055, -0.134, 0.248), rot=(90, 0, 0), verts=6)
    box('tank', (0.18, 0.05, 0.22), (0, 0.145, 0.12), M.glass_pale, bevel=0.008)
    box('tank_lid', (0.184, 0.054, 0.012), (0, 0.145, 0.236), M.dark, bevel=0.004)
    box('drip_tray', (0.18, 0.13, 0.02), (0, -0.13, 0.01), M.dark, bevel=0.005)
    slots = [G.rounded_rect(0.008, 0.08, 0.003, corner=1) for _ in range(7)]
    slots = [[(u + (i - 3) * 0.02, v) for u, v in s] for i, s in enumerate(slots)]
    slab('grate', G.rounded_rect(0.16, 0.11, 0.008, corner=1), 0.004, (0, -0.13, 0.021), M.steel, plane='XY',
         holes=slots)
    turned('group', [(0.036, 0.17), (0.038, 0.19), (0.034, 0.2)], M.dark, at=(0, -0.105, 0), verts=14)
    turned('basket', [(0.03, 0.15), (0.034, 0.156), (0.034, 0.172), (0.038, 0.174), (0.038, 0.18)], M.steel,
           at=(0, -0.105, 0), verts=14)
    bar('portafilter', [(0, -0.135, 0.172), (0, -0.18, 0.168), (0, -0.222, 0.162)], oval(0.02, 0.024, 8), M.dark,
        scale=[0.8, 1.0, 1.12])
    for x in (-0.009, 0.009):
        G.cone(f'spout{x}', 0.005, 0.003, 0.014, at=(x, -0.105, 0.144), material=M.steel, verts=6)
    # the cup on the grate, its handle to the side
    turned('cup', [(0.018, 0.023), (0.024, 0.026), (0.031, 0.066), (0.032, 0.07), (0.029, 0.07), (0.026, 0.045),
                   (0.0, 0.04)], M.porcelain, at=(0, -0.105, 0), verts=12)
    disc('crema', 0.0285, (0, -0.105, 0.064), M.coffee, verts=12)
    bar('cup_handle', G.kit.fillet([(0.028, -0.105, 0.06), (0.044, -0.105, 0.058), (0.044, -0.105, 0.04),
                                    (0.027, -0.105, 0.036)], 0.008, steps=1), oval(0.005, 0.008, 4), M.porcelain,
        up=(0, 1, 0))
    # the gauge on the body's front, over the tray
    turned('gauge', [(0.026, 0.0), (0.026, 0.006), (0.02, 0.008)], M.chrome, at=(0.062, -0.07, 0.13), rot=(90, 0, 0),
           verts=16)
    turned('gauge_face', [(0.0195, 0.004), (0.0195, 0.0075)], M.paper, at=(0.062, -0.07, 0.13), rot=(90, 0, 0),
           verts=16)
    box('needle', (0.002, 0.002, 0.016), (0.062 - 0.004, -0.0782, 0.136), M.red, rot=(0, -30, 0))
    turned('steam_knob', [(0.016, 0.0), (0.016, 0.006), (0.013, 0.011), (0.0, 0.012)], M.dark, at=(-0.109, -0.03, 0.15),
           rot=(0, -90, 0), verts=10)
    G.sweep('steam_wand', G.kit.fillet([(-0.09, -0.07, 0.215), (-0.1, -0.085, 0.215), (-0.1, -0.09, 0.09)], 0.015,
                                       steps=2), 0.005, material=M.chrome, verts=6)
    G.cone('steam_tip', 0.005, 0.0035, 0.012, at=(-0.1, -0.09, 0.084), material=M.chrome, verts=6)
    # the warmer's rail on four posts, an upturned cup inside it
    rail = [(x, 0.026 + y, 0.288) for x, y in G.rounded_rect(0.17, 0.15, 0.025, corner=2)]
    G.sweep('rail', rail, 0.0035, material=M.chrome, verts=6, closed=True)
    for i, (x, y, _) in enumerate(rail[1::3]):
        G.rod(f'post{i}', (x, y, 0.268), (x, y, 0.288), 0.003, material=M.chrome, verts=6)
    turned('top_cup', [(0.024, 0.27), (0.027, 0.272), (0.021, 0.304), (0.012, 0.31), (0.0, 0.31)], M.porcelain,
           at=(0.04, 0.06, 0), verts=12)
    return done('espressoMachineSmall')


def knifeBlock():
    """A wooden knife block leaning back on its foot, its edges eased; six knives in it, each a riveted black handle
    on a steel bolster, sized to its blade."""
    TILT, ZC = -18, 0.123
    box('foot', (0.12, 0.2, 0.05), (0, 0, 0.025), M.knife_block, bevel=0.006)
    outline = [tilt_x(u, v, TILT) for u, v in G.rounded_rect(0.18, 0.2, 0.014, corner=2)]
    slab('block', [(y, z + ZC) for y, z in outline], 0.12, (0, 0, 0), M.knife_block, plane='YZ', bevel=0.004)
    for i, (x, y) in enumerate(((-0.03, -0.05), (0.0, -0.05), (0.03, -0.05), (-0.022, 0.02), (0.022, 0.02),
                                (0.0, 0.06))):
        w = 0.016 if i < 3 else 0.021
        pts = []
        for z in (0.098, 0.106, 0.15, 0.164):
            dy, dz = tilt_x(y, z, TILT)
            pts.append((x, dy, ZC + dz))
        dy, dz = tilt_x(y, 0.104, TILT)
        box(f'bolster{i}', (w + 0.002, 0.024, 0.008), (x, dy, ZC + dz), M.steel, rot=(TILT, 0, 0))
        bar(f'handle{i}', pts, G.rounded_rect(0.024, w, 0.006, corner=1), M.dark, up=(0, -1, 0),
            scale=[0.95, 1.0, 1.08, 1.0])
    return done('knifeBlock')


def utensilCrock():
    """A glazed crock with a rolled lip and a band, holding a wooden spoon, a slotted spatula, a balloon whisk and a
    ladle."""
    turned('crock', [(0.056, 0.0), (0.06, 0.02), (0.058, 0.11), (0.06, 0.135), (0.062, 0.147),
                     (0.0595, 0.152), (0.054, 0.15), (0.052, 0.12)], M.crock, verts=14)
    turned('band', [(0.0597, 0.072), (0.0592, 0.094)], M.crock_band, verts=14, caps=False)
    # the wooden spoon: one flat stem swelling into its bowl
    bar('spoon', [(0.0, 0.0, 0.12), (0.038, -0.016, 0.24), (0.045, -0.018, 0.256), (0.053, -0.021, 0.284),
                  (0.059, -0.023, 0.31)], oval(0.006, 0.012, 6), M.wood, up=(0, -1, 0),
        scale=[1.0, 1.0, 0.9, 2.6, 0.8], resolution=2, angle=70)
    # the spatula: a black handle into a slotted head
    G.sweep('spatula', [(0.0, 0.0, 0.12), (-0.033, 0.019, 0.25)], 0.0055, material=M.dark, verts=6)
    slab('spatula_head', G.rounded_rect(0.05, 0.065, 0.008, corner=1), 0.004, (-0.04, 0.023, 0.282), M.dark,
         rot=(0, -9, -30), holes=[[(u + dx, v + 0.004) for u, v in ((0.004, 0.02), (-0.004, 0.02), (-0.004, -0.02), (0.004, -0.02))]
                                   for dx in (-0.014, 0.0, 0.014)])
    # the whisk: a steel handle and two wire loops across each other
    G.sweep('whisk', [(0.0, 0.0, 0.12), (0.0, 0.03, 0.23)], 0.0055, material=M.steel, verts=6)
    for k, phi in enumerate((0, 90)):
        c, s = G.cosd(phi), G.sind(phi)
        loop = [(0.0, 0.0), (0.014, 0.016), (0.022, 0.048), (0.014, 0.074), (0.0, 0.081), (-0.014, 0.074),
                (-0.022, 0.048), (-0.014, 0.016)]
        G.sweep(f'wire{k}', [(u * c, 0.031 + u * s + v * 0.15, 0.228 + v) for u, v in loop], 0.0013,
                material=M.steel, verts=4, closed=True)
    # the ladle: a stem with a hook at its top, the bowl at its foot hidden; its bowl turned up at the top
    G.sweep('ladle', [(0.0, 0.0, 0.12), (-0.02, -0.032, 0.245)], 0.005, material=M.wood_dark, verts=6)
    turned('ladle_bowl', [(0.0, 0.0), (0.012, 0.003), (0.019, 0.011), (0.021, 0.02), (0.018, 0.021), (0.015, 0.011),
                          (0.0, 0.007)], M.wood_dark, at=(-0.023, -0.037, 0.242), rot=(-30, -15, 0), verts=8)
    return done('utensilCrock')


def cuttingBoard():
    """A wooden board with a hand hole, eased edges: a scored bloomer loaf, two vine tomatoes with their calyxes, two
    slices cut, and a cook's knife (blade, bolster, black handle)."""
    hole = [(u + 0.135, v) for u, v in G.rounded_rect(0.05, 0.018, 0.009, corner=1)]
    slab('board', G.rounded_rect(0.34, 0.23, 0.03, corner=2), 0.022, (0, 0, 0.011), M.board, plane='XY', bevel=0.003,
         holes=[hole])
    c, s = G.cosd(15), G.sind(15)
    turned('loaf', [(0.018, 0.024, 0.0), (0.033, 0.043, 0.012), (0.04, 0.05, 0.04), (0.041, 0.052, 0.095),
                    (0.04, 0.05, 0.15), (0.033, 0.043, 0.178), (0.018, 0.024, 0.19)], M.bread,
           at=(0.06 - 0.095 * c, 0.02 - 0.095 * s, 0.063), rot=(0, 90, 15), verts=10, square=2.4)
    for i, d in enumerate((-0.042, 0.0, 0.042)):
        box(f'score{i}', (0.007, 0.046, 0.003), (0.06 + d * c, 0.02 + d * s, 0.1018), M.crumb,
            rot=(0, 0, 15 + 35))
    for i, (x, y) in enumerate(((-0.1, 0.05), (-0.125, -0.025))):
        ball(f'tomato{i}', 0.03, (x, y, 0.047), M.red, scale=(1, 1, 0.85), seg=8, rings=5)
        star(f'calyx{i}', (x, y, 0.071), 0.014, 5, M.leaf, inner=0.35, depth=0.003, cup=0.2)
    for i, (x, y) in enumerate(((-0.04, -0.07), (-0.012, -0.082))):
        z = 0.022 + i * 0.006
        turned(f'slice{i}', [(0.027, z), (0.027, z + 0.007)], M.red, verts=10, at=(x, y, 0))
        disc(f'flesh{i}', 0.021, (x, y, z + 0.0072), M.tomato_flesh, verts=8)
    # the knife, lying across the front, its handle to the right
    a = -8
    ca, sa = G.cosd(a), G.sind(a)

    def k(u, v):
        return (0.085 + u * ca - v * sa, -0.082 + u * sa + v * ca)
    blade = [k(0.0, -0.011), k(-0.075, -0.011), k(-0.105, -0.003), k(-0.112, 0.0), k(-0.09, 0.007), k(0.0, 0.012)]
    slab('blade', blade, 0.002, (0, 0, 0.023), M.steel, plane='XY')
    bx, by = k(0.004, 0.0)
    box('bolster', (0.008, 0.026, 0.012), (bx, by, 0.028), M.steel, rot=(0, 0, a))
    bar('knife_handle', [(*k(0.008, 0.0), 0.029), (*k(0.04, 0.0), 0.029), (*k(0.068, 0.0), 0.029)],
        G.rounded_rect(0.013, 0.018, 0.005, corner=1), M.dark, scale=[1.0, 1.08, 1.0])
    return done('cuttingBoard')


def breadBin():
    """A roll-top bread bin: a painted body between two D-shaped wooden ends (their edges eased); the roll-top is a
    part of its own, turning about the bin's axis, a wooden bar pull on it: it rolls back into the bin for a sandwich
    (`roll_open`), showing a loaf on the bin's floor."""
    AXIS, R = 0.105, 0.095
    with G.new_parts() as body:
        box('base', (0.31, 0.19, AXIS), (0, 0, AXIS / 2), M.bread_bin, bevel=0.008, segs=1)
        end = [(R + 0.006, 0.0)] + [((R + 0.006) * math.cos(math.pi * i / 10), AXIS + (R + 0.006) * math.sin(math.pi * i / 10))
                                   for i in range(11)] + [(-(R + 0.006), 0.0)]
        for s in (-1, 1):
            slab(f'end{s}', [(-u, v) for u, v in end], 0.014, (s * 0.163, 0, 0), M.wood, plane='YZ', bevel=0.003)
        box('toe', (0.31, 0.012, 0.012), (0, -0.092, 0.006), M.wood, bevel=0.003)
        # a loaf on the bin's floor (the base's top), under the roll-top: seen only when it is rolled back
        G.soft_box('loaf', (0.2, 0.1, 0.07), at=(-0.02, 0.0, AXIS + 0.035), material=M.bread, round=0.028, flat=1,
                   round_segments=1)
    G.join(body, 'bread_bin', origin=(0, 0, 0))
    with G.new_parts() as top:
        arc_shell('roll_top', R, R - 0.008, 0.312, 0, 180, (0, 0, AXIS), M.bread_bin, segments=14)
        F = G.GameFrame()
        a = math.radians(35)
        G.kit.pull(F, 'roll_handle', 0.0, AXIS + R * math.sin(a), R * math.cos(a) - 0.004, 0.1, M.wood, r=0.0065,
                   out=0.02, verts=6)
    top = G.hinged(G.join(top, 'roll_top'), (0, 0, AXIS))
    G.settle()
    # rolled back over the top and down behind it, inside the bin (Blender -X lifts the front)
    G.swing(top, 'roll_open', -100, seconds=0.6, axis='X')
    return G.meshes()


def spiceJars():
    """A wooden spice rack: shaped ends, a back board and a round front rail, five glass jars (each the colour of its
    spice) with paper labels and black screw lids."""
    box('tray', (0.3, 0.12, 0.012), (0, 0, 0.006), M.rack, bevel=0.003, segs=1)
    end = G.smooth_outline([(-0.06, 0.0), (0.06, 0.0), (0.06, 0.07), (0.045, 0.076), (0.0, 0.05), (-0.045, 0.036),
                            (-0.06, 0.03)], passes=1)
    end = [end[0]] + end[1:]
    for s in (-1, 1):
        slab(f'end{s}', [(-u, v) for u, v in end], 0.012, (s * 0.144, 0, 0), M.rack, plane='YZ')
    box('back', (0.276, 0.008, 0.052), (0, 0.054, 0.036), M.rack)
    G.sweep('rail', [(-0.144, -0.05, 0.03), (0.144, -0.05, 0.03)], 0.004, material=M.rack, verts=6)
    for i, colour in enumerate((M.spice_a, M.spice_b, M.spice_c, M.spice_d, M.spice_e)):
        x = -0.11 + i * 0.055
        turned(f'jar{i}', [(0.021, 0.012), (0.022, 0.08), (0.016, 0.092)], colour, at=(x, 0, 0), verts=8)
        turned(f'label{i}', [(0.0226, 0.03), (0.0226, 0.06)], M.paper, at=(x, 0, 0), verts=8, caps=False)
        turned(f'lid{i}', [(0.0175, 0.09), (0.0188, 0.093), (0.0188, 0.112)], M.dark, at=(x, 0, 0), verts=8)
    return done('spiceJars')


def cookieJar():
    """A glass cookie jar with a shoulder and a neck, four cookies stacked inside it, a glazed lid with a turned
    knob."""
    turned('jar', [(0.072, 0.0), (0.079, 0.006), (0.08, 0.02), (0.08, 0.15), (0.068, 0.168), (0.064, 0.173),
                   (0.064, 0.19)], M.glass, verts=16)
    for i in range(4):
        a = i * 72
        ring = 0.022 if i else 0.0
        turned(f'cookie{i}', [(0.033, 0.0), (0.034, 0.005), (0.026, 0.011), (0.0, 0.012)],
               M.cookie if i % 2 else M.cookie_dark, at=(ring * G.cosd(a), ring * G.sind(a), 0.008 + i * 0.03),
               rot=(12 * (-1) ** i, 8, 0), verts=8)
    turned('lid', [(0.062, 0.186), (0.069, 0.19), (0.07, 0.208), (0.066, 0.216), (0.05, 0.221), (0.0, 0.223)],
           M.jar_lid, verts=14)
    turned('knob', [(0.01, 0.22), (0.009, 0.23), (0.02, 0.238), (0.017, 0.246), (0.0, 0.248)], M.jar_lid, verts=8)
    return done('cookieJar')


# --- TT2 tableware --------------------------------------------------------------------------------------------
def _apple(name, at, material, s=1.0, turn=0.0):
    """An apple: round shoulders over a narrower base, a dimple at the top and its stalk."""
    x, y, z = at
    turned(name, [(0.0, 0.006 * s), (0.02 * s, 0.0), (0.037 * s, 0.018 * s), (0.04 * s, 0.038 * s),
                  (0.032 * s, 0.06 * s), (0.014 * s, 0.067 * s), (0.0, 0.061 * s)], material, at=(x, y, z), verts=8,
           rot=(turn, 0, 0))
    G.sweep(f'{name}_stalk', [(x, y, z + 0.058 * s), (x + 0.003, y, z + 0.078 * s)], 0.0022, material=M.stalk, verts=4)


def _banana(name, a, b, lift, material):
    """A banana lying from Blender point `a` to `b`, bowed up `lift` in the middle: five-sided (as a banana's ridges),
    tapering to a dark tip at each end."""
    pts = [Vector(a).lerp(Vector(b), t) + Vector((0, 0, lift * math.sin(math.pi * t))) for t in (0, 0.12, 0.35, 0.65, 0.88, 1)]
    G.sweep(name, [tuple(p) for p in pts], 0.017, material=material, verts=5, scale=[0.25, 0.8, 1.0, 1.0, 0.75, 0.2],
            angle=50)
    for k, (p, q) in enumerate(((pts[0], pts[1]), (pts[-1], pts[-2]))):
        d = (p - q).normalized()
        G.rod(f'{name}_tip{k}', tuple(p - d * 0.004), tuple(p + d * 0.006), 0.0045, material=M.stalk, verts=4)


def fruitBowl():
    """A glazed fruit bowl on a foot ring, with two apples (red and green, stalks), two oranges and two bananas lying
    over them."""
    turned('bowl', [(0.052, 0.0), (0.056, 0.012), (0.1, 0.04), (0.136, 0.084), (0.14, 0.09), (0.132, 0.09),
                    (0.095, 0.046), (0.0, 0.03)], M.fruit_bowl, verts=16)
    _apple('apple_red', (0.05, 0.03, 0.034), M.red)
    _apple('apple_green', (-0.035, 0.065, 0.036), M.apple_green, turn=8)
    for i, (x, y) in enumerate(((-0.055, -0.03), (0.035, -0.058))):
        ball(f'orange{i}', 0.04, (x, y, 0.074), M.orange, scale=(1, 1, 0.94), seg=8, rings=5)
        G.sphere(f'navel{i}', 0.006, at=(x, y, 0.111), material=M.stalk, segments=4, rings=3, scale=(1, 1, 0.4))
    _banana('banana0', (-0.085, -0.03, 0.124), (0.07, 0.05, 0.134), 0.05, M.yellow)
    _banana('banana1', (-0.07, -0.06, 0.114), (0.085, 0.02, 0.146), 0.045, M.yellow)
    return done('fruitBowl')


def _plate(name, r, z, material, verts=24):
    """A plate: a foot ring under it, the well, a rim rising to its rolled edge."""
    k = r / 0.135
    return turned(name, [(0.075 * k, z), (0.08 * k, z + 0.004), (0.128 * k, z + 0.013), (0.135 * k, z + 0.017),
                         (0.132 * k, z + 0.019), (0.09 * k, z + 0.01), (0.0, z + 0.009)], material, verts=verts)


def _rim_band(name, r, z, material, verts=24):
    """The coloured band round a plate's rim (`_plate`'s numbers), a hair proud of it."""
    k = r / 0.135

    def h(rr):
        return z + 0.01 + (rr - 0.09 * k) * (0.009 / (0.042 * k)) + 0.0007
    band = turned(name, [(0.116 * k, h(0.116 * k)), (0.124 * k, h(0.124 * k))], material, verts=verts, caps=False)
    if sum(p.normal.z for p in band.data.polygons) < 0:       # an open band: make it face up, where it's seen
        band.data.flip_normals()
    return band


def _fork(name, at, material):
    """A table fork lying flat, its tines to the back: a handle narrowing to the neck, the head with four tines."""
    c, hw, top, gap = (0.0081, 0.0027, -0.0027, -0.0081), 0.0019, 0.08, 0.052
    pts = [(-0.006, -0.085), (0.006, -0.085), (0.0078, -0.074), (0.0062, -0.02), (0.0032, 0.015), (0.01, 0.04),
           (0.01, top)]
    for i, x in enumerate(c):
        pts.append((x - hw, top))
        if i < 3:
            pts += [(x - hw, gap), (c[i + 1] + hw, gap), (c[i + 1] + hw, top)]
    pts += [(-0.01, top), (-0.01, 0.04), (-0.0032, 0.015), (-0.0062, -0.02), (-0.0078, -0.074)]
    pts = [pts[0]] + [pts[i] for i in range(1, len(pts)) if pts[i] != pts[i - 1]]
    slab(name, pts, 0.0025, at, material, plane='XY')


def plateSet():
    """A place setting: a dinner plate with a coloured band round its rim and a side plate on it, a linen napkin with a
    fork on it, a knife, and a tumbler."""
    _plate('plate', 0.135, 0.0, M.plate)
    _rim_band('rim', 0.135, 0.0, M.plate_rim)
    _plate('side_plate', 0.095, 0.009, M.plate, verts=20)
    _rim_band('side_rim', 0.095, 0.009, M.plate_rim, verts=20)
    G.soft_box('napkin', (0.06, 0.17, 0.005), at=(-0.17, -0.01, 0.0025), material=M.napkin, round=0.002, puff=0.0015,
               round_segments=1, flat=1)
    _fork('fork', (-0.17, -0.005, 0.0062), M.chrome)
    slab('knife', [(-0.0075, -0.1), (0.0075, -0.1), (0.008, -0.012), (0.006, -0.008), (0.009, 0.0), (0.009, 0.075),
                   (0.006, 0.092), (0.0, 0.1), (-0.009, 0.096), (-0.009, -0.006), (-0.0075, -0.012)], 0.003,
         (0.17, 0.0, 0.0015), M.chrome, plane='XY')
    turned('tumbler', [(0.03, 0.0), (0.032, 0.004), (0.037, 0.1), (0.034, 0.1), (0.03, 0.014), (0.0, 0.012)], M.glass_pale,
           at=(0.14, 0.15, 0), verts=14)
    return done('plateSet', mirror=True)


def _drips(obj, z, depth, n=7, seed=0.0):
    """Lowers the vertices of `obj` at height `z` (Blender) into icing drips: `n` round its edge, up to `depth`."""
    for v in obj.data.vertices:
        if abs(v.co.z - z) < 1e-4:
            a = math.atan2(v.co.y, v.co.x)
            v.co.z -= depth * max(0.0, math.sin(n * a + seed)) ** 2
    obj.data.update()
    return obj


def cakeStand():
    """A footed cake stand (a turned foot, a knop on its stem, the plate's raised edge), a layer cake on it: the
    sponge, a band of filling, icing over the top running down in drips, six piped rosettes and a cherry."""
    turned('stand', [(0.078, 0.0), (0.08, 0.008), (0.03, 0.022), (0.02, 0.06), (0.026, 0.072), (0.02, 0.084),
                     (0.13, 0.092), (0.14, 0.1), (0.0, 0.102)], M.cake_stand, verts=16)
    turned('sponge', [(0.107, 0.101), (0.107, 0.192)], M.sponge, verts=16)
    turned('filling', [(0.1076, 0.144), (0.1076, 0.152)], M.icing, verts=16, caps=False)
    _drips(turned('icing', [(0.1095, 0.188), (0.1105, 0.198), (0.109, 0.212), (0.096, 0.218), (0.0, 0.219)], M.icing,
                  verts=16), 0.188, 0.026)
    for i in range(6):
        a = i * 60 + 30
        turned(f'rosette{i}', [(0.011, 0.0), (0.012, 0.004), (0.007, 0.011), (0.0, 0.016)], M.icing,
               at=(0.082 * G.cosd(a), 0.082 * G.sind(a), 0.216), verts=5, rot=(0, 0, a))
    ball('cherry', 0.016, (0, 0, 0.234), M.red, seg=8, rings=5)
    G.sweep('cherry_stalk', [(0, 0, 0.247), (0.003, 0.002, 0.255), (0.008, 0.004, 0.262)], 0.0018, material=M.stalk,
            verts=4)
    return done('cakeStand')


def _cup(name, x, y, z, material, turn=180):
    """A teacup on its saucer: the saucer's well, the cup's foot and flaring bowl with tea in it, a loop handle."""
    turned(f'{name}_saucer', [(0.0, z), (0.03, z + 0.001), (0.044, z + 0.006), (0.047, z + 0.01), (0.043, z + 0.01),
                              (0.03, z + 0.005), (0.0, z + 0.004)], material, at=(x, y, 0), verts=12)
    turned(name, [(0.016, z + 0.004), (0.02, z + 0.008), (0.03, z + 0.038), (0.032, z + 0.045), (0.029, z + 0.045),
                  (0.026, z + 0.022), (0.0, z + 0.016)], material, at=(x, y, 0), verts=12)
    disc(f'{name}_tea', 0.0285, (x, y, z + 0.038), M.coffee, verts=10)
    c, s = G.cosd(turn), G.sind(turn)
    loop = G.kit.fillet([(0.026, z + 0.038), (0.042, z + 0.04), (0.042, z + 0.022), (0.024, z + 0.017)], 0.007, steps=1)
    G.sweep(f'{name}_handle', [(x + u * c, y + u * s, v) for u, v in loop], 0.0035, material=material, verts=5)


def teapotSet():
    """A tea tray with a gallery round it and hand holes in its ends, a round teapot (a band round its belly, a
    curving spout, a loop handle, the lid and its knob) and two cups of tea on saucers."""
    slab('tray', G.rounded_rect(0.39, 0.25, 0.02, corner=2), 0.008, (0, 0, 0.004), M.tray, plane='XY', bevel=0.002)
    for s in (-1, 1):
        box(f'tray_side{s}', (0.37, 0.01, 0.026), (0, s * 0.123, 0.013), M.tray, bevel=0.003, segs=1)
        end = G.smooth_outline([(-0.13, 0.0), (0.13, 0.0), (0.13, 0.032), (0.08, 0.05), (-0.08, 0.05), (-0.13, 0.032)],
                               passes=1)
        slab(f'tray_end{s}', end, 0.012, (s * 0.194, 0, 0), M.tray, plane='YZ', bevel=0.002,
             holes=[[(u, v + 0.032) for u, v in G.rounded_rect(0.08, 0.016, 0.008, corner=2)]])
    x, y = 0.07, 0.02
    turned('pot', [(0.045, 0.008), (0.05, 0.014), (0.068, 0.042), (0.073, 0.075), (0.066, 0.11), (0.05, 0.13),
                   (0.04, 0.136)], M.teapot, at=(x, y, 0), verts=14)
    turned('pot_band', [(0.0738, 0.068), (0.0735, 0.08)], M.teapot_trim, at=(x, y, 0), verts=14, caps=False)
    turned('pot_lid', [(0.042, 0.134), (0.045, 0.138), (0.037, 0.147), (0.02, 0.151), (0.0, 0.152)], M.teapot,
           at=(x, y, 0), verts=14)
    turned('pot_knob', [(0.006, 0.15), (0.012, 0.157), (0.01, 0.164), (0.0, 0.167)], M.teapot_trim, at=(x, y, 0), verts=8)
    G.sweep('spout', [(x - 0.05, y, 0.05), (x - 0.085, y, 0.07), (x - 0.105, y, 0.1), (x - 0.122, y, 0.124)], 0.014,
            material=M.teapot, verts=8, scale=[1.5, 1.0, 0.72, 0.58], resolution=2)
    G.sweep('pot_handle', G.kit.fillet([(x + 0.058, y, 0.115), (x + 0.1, y, 0.118), (x + 0.108, y, 0.07),
                                        (x + 0.066, y, 0.038)], 0.02, steps=2), 0.0065, material=M.teapot, verts=6)
    _cup('cup_a', -0.11, -0.06, 0.008, M.teapot, turn=200)
    _cup('cup_b', -0.1, 0.07, 0.008, M.teapot, turn=160)
    return done('teapotSet')


def _mug(name, x, y, turn, material):
    """A mug: a thick rim, coffee in it, a loop handle in one piece."""
    turned(name, [(0.036, 0.0), (0.04, 0.005), (0.04, 0.087), (0.0385, 0.09), (0.035, 0.09), (0.034, 0.076)], material,
           at=(x, y, 0), verts=12)
    disc(f'{name}_coffee', 0.0342, (x, y, 0.078), M.coffee, verts=10)
    c, s = G.cosd(turn), G.sind(turn)
    loop = G.kit.fillet([(0.035, 0.074), (0.066, 0.07), (0.066, 0.026), (0.035, 0.022)], 0.014, steps=2)
    bar(f'{name}_handle', [(x + u * c, y + u * s, v) for u, v in loop], oval(0.009, 0.014, 6), material,
        up=(-s, c, 0))


def mugs():
    """Three mugs of coffee, their handles every way, a tea bag's string and tag hanging over one."""
    _mug('mug_a', 0.068, -0.03, 150, M.mug_a)
    _mug('mug_b', -0.068, -0.04, 20, M.mug_b)
    _mug('mug_c', 0.0, 0.06, 80, M.mug_c)
    a = math.radians(232)
    rx, ry = -0.068 + 0.04 * math.cos(a), -0.04 + 0.04 * math.sin(a)
    G.sweep('string', [(-0.068, -0.04, 0.08), (rx, ry, 0.0885), (rx + 0.0008 * math.cos(a), ry + 0.0008 * math.sin(a), 0.06)],
            0.0008, material=M.paper, verts=3)
    box('tag', (0.012, 0.002, 0.016), (rx + 0.0016 * math.cos(a), ry + 0.0016 * math.sin(a), 0.052), M.yellow,
        rot=(0, 0, 232 - 90))
    return done('mugs')


def _wine_glass(name, x, y):
    """A wine glass, open at its rim (see-through): a foot, a thin stem, the bowl; red wine in it."""
    turned(name, [(0.0, 0.0), (0.03, 0.0), (0.03, 0.003), (0.006, 0.008), (0.0045, 0.07), (0.014, 0.086),
                  (0.028, 0.1), (0.035, 0.125), (0.033, 0.17), (0.031, 0.169)], M.glass, at=(x, y, 0), verts=10,
           caps=False)
    turned(f'{name}_wine', [(0.0, 0.088), (0.02, 0.098), (0.03, 0.114), (0.0318, 0.122)], M.wine, at=(x, y, 0), verts=10)


def wineGlasses():
    """A bottle of red (shoulders, a long neck, a label, the foil capsule) and two glasses poured."""
    turned('bottle', [(0.036, 0.0), (0.04, 0.005), (0.04, 0.19), (0.032, 0.222), (0.016, 0.248), (0.0145, 0.26),
                      (0.0145, 0.3)], M.bottle, at=(0, 0.07, 0), verts=12)
    turned('label', [(0.0406, 0.075), (0.0406, 0.15)], M.paper, at=(0, 0.07, 0), verts=12, caps=False)
    turned('foil', [(0.0152, 0.252), (0.0158, 0.256), (0.0156, 0.299), (0.0, 0.301)], M.wine, at=(0, 0.07, 0), verts=8)
    _wine_glass('glass_a', -0.07, -0.04)
    _wine_glass('glass_b', 0.07, -0.05)
    return done('wineGlasses')


def candlesticks():
    """Two turned brass candlesticks (a stepped foot, a knop on the stem, a drip pan at the top) with tall candles,
    a run of wax down one. By day the wicks are dark; from dusk the flames rise out of the candles' tops (`lit`, a
    `night` clip) and glow (`glow`), the brightest thing on the table."""
    with G.new_parts() as still:
        for i, (x, h) in enumerate(CANDLES):
            turned(f'stick{i}', [(0.0346, 0.0), (0.0346, 0.006), (0.024, 0.016), (0.009, 0.03), (0.008, h * 0.45),
                                 (0.016, h * 0.5), (0.008, h * 0.56), (0.008, h - 0.022), (0.022, h - 0.012),
                                 (0.022, h), (0.0, h)], M.brass, at=(x, 0, 0), verts=10)
            turned(f'candle{i}', [(0.011, h - 0.01), (0.011, h + 0.132), (0.0095, h + 0.139), (0.0, h + 0.14)],
                   M.candle, at=(x, 0, 0), verts=8)
            G.sweep(f'wick{i}', [(x, 0, h + 0.138), (x, 0, h + 0.146), (x + 0.002, 0, h + 0.149)], 0.0012,
                    material=M.dark, verts=3)
        x, h = CANDLES[0]
        G.sweep('drip', [(x - 0.008, -0.007, h + 0.139), (x - 0.0105, -0.0035, h + 0.125), (x - 0.011, -0.003, h + 0.1)],
                0.0028, material=M.candle, verts=4, scale=[1.0, 0.9, 0.7])
    G.join(still, 'candlesticks', origin=(0, 0, 0))
    # the flames, built lit and sunk into the candles at rest (hidden inside the wax, 1.3 cm across to their
    # 2.2): the same rise lifts both, the candles being the same length
    with G.new_parts() as lit:
        for i, (x, h) in enumerate(CANDLES):
            turned(f'flame{i}', [(0.0, 0.0), (0.0055, 0.004), (0.0065, 0.009), (0.0035, 0.018), (0.0, 0.026)], M.flame,
                   at=(x, 0, h + 0.145 - FLAME_RISE), verts=6)
    flames = G.join(lit, 'flames', origin=(0, 0, 0))
    G.settle()
    G.slide(flames, 'lit', (0, 0, FLAME_RISE), seconds=0.6)
    return G.meshes()


CANDLES = ((0.05, 0.18), (-0.05, 0.14))   # (x, the stick's height)
FLAME_RISE = 0.034                         # a flame's 3 cm, from inside the candle's top to over its wick


# --- TT3 books and small plants -------------------------------------------------------------------------------
def _book(name, size, at, cover, turn=0.0, bands=True):
    """A closed hardback lying flat, its rounded spine to the front: the boards a little proud of the page block,
    which shows at the head, tail and fore-edge; two gilt bands across the spine."""
    w, d, t = size
    x, y, z = at
    c, s = G.cosd(turn), G.sind(turn)

    def p(u, v, h=0.0):
        return (x + u * c - v * s, y + u * s + v * c, z + h)
    for k in (-1, 1):
        box(f'{name}_board{k}', (w, d - 0.004, 0.003), p(0, 0.002, k * (t / 2 - 0.0015)), cover, rot=(0, 0, turn))
    G.cylinder(f'{name}_spine', t / 2, w, at=p(0, -d / 2 + t * 0.21), material=cover, verts=8, rot=(0, 90, turn),
               scale=(1, 0.42, 1))
    box(f'{name}_pages', (w - 0.006, d - 0.008, t - 0.005), p(0, 0.0025), M.pages, rot=(0, 0, turn))
    if bands:
        for u in (-0.3, 0.3):
            box(f'{name}_band{u}', (0.005, 0.002, t * 0.7), p(u * w, -d / 2 - 0.0002), M.brass, rot=(0, 0, turn))


def bookStack():
    """Four hardbacks stacked a little askew, their rounded spines to the front with gilt bands, the pages showing
    between the boards, a ribbon marker hanging from the top one."""
    z = 0.0
    for i, (w, d, t, cover, turn, dx) in enumerate(((0.24, 0.17, 0.035, M.book_a, 0, 0.0),
                                                    (0.22, 0.16, 0.03, M.book_b, 10, 0.008),
                                                    (0.2, 0.15, 0.04, M.book_c, -7, -0.008),
                                                    (0.18, 0.13, 0.028, M.book_d, 18, 0.004))):
        _book(f'book{i}', (w, d, t), (dx, 0, z + t / 2), cover, turn)
        z += t
    G.sweep('ribbon', [(0.03, 0.06, z - 0.0005), (0.034, 0.069, z - 0.003), (0.035, 0.07, z - 0.022)], None,
            profile=[(-0.0004, -0.003), (0.0004, -0.003), (0.0004, 0.003), (-0.0004, 0.003)], material=M.red,
            up=(0, 1, 0))
    return done('bookStack')


def booksUpright():
    """Eight books standing between brass bookends (each an arched plate with a round cut-out on a foot): the boards
    proud of the page block at the top, rounded spines to the front, paper bands on some."""
    for s in (-1, 1):
        slab(f'bookend{s}', [(u, v + 0.08) for u, v in G.rounded_rect(0.12, 0.16, 0.035, corner=2)], 0.008,
             (s * 0.156, 0, 0), M.brass, plane='YZ',
             holes=[[(u, v + 0.118) for u, v in G.rounded_rect(0.036, 0.036, 0.018, corner=2)]])
        box(f'bookend_foot{s}', (0.06, 0.12, 0.006), (s * 0.13, 0, 0.003), M.brass)
    covers = (M.book_a, M.book_c, M.book_b, M.book_e, M.book_d, M.book_c, M.book_a, M.book_b, M.book_e)
    x, k = -0.146, 6
    for i, cover in enumerate(covers):
        k = (k * 7 + 3) % 11
        w = 0.022 + (k % 4) * 0.007
        h = 0.2 * (0.72 + (k % 5) * 0.07)
        if x + w > 0.146:
            break
        cx = x + w / 2
        box(f'book{i}_pages', (w - 0.003, 0.12, h - 0.004), (cx, 0.0025, 0.006 + (h - 0.004) / 2), M.pages)
        for e in (-1, 1):
            box(f'book{i}_board{e}', (0.0016, 0.124, h), (cx + e * (w / 2 - 0.0008), 0.0005, 0.006 + h / 2), cover)
        G.cylinder(f'book{i}_spine', w / 2, h, at=(cx, -0.0625 + w * 0.2, 0.006 + h / 2), material=cover, verts=6,
                   scale=(1, 0.4, 1))
        if i % 3 != 1:
            box(f'book{i}_band', (w * 0.8, 0.002, 0.008), (cx, -0.0626, 0.006 + h * 0.8), M.paper)
        x += w + 0.002
    return done('booksUpright')


def _facing(x, y):
    """The yaw (plants.py: degrees round from the back) that faces out from the middle towards (x, y)."""
    return 90 - math.degrees(math.atan2(y, x)) if (x or y) else 0.0


def vaseFlowers():
    """A glazed vase swelling at its belly with a flared lip, a mixed bunch on curving stems: white and yellow daisies
    (ray petals round a raised eye), two tulips and a pink cosmos, leaves at the neck."""
    turned('vase', [(0.04, 0.0), (0.044, 0.004), (0.06, 0.05), (0.065, 0.11), (0.052, 0.18), (0.032, 0.215),
                    (0.036, 0.24), (0.03, 0.24), (0.027, 0.225)], M.vase, verts=16)
    heads = ((0.0, 0.0, 0.392, M.white, 'daisy'), (-0.066, 0.03, 0.345, M.blush, 'tulip'),
             (0.064, -0.02, 0.355, M.yellow, 'daisy'), (0.03, 0.06, 0.37, M.rose, 'tulip'),
             (-0.042, -0.05, 0.335, M.white, 'daisy'), (0.05, 0.042, 0.325, M.blush, 'cosmos'),
             (-0.022, 0.05, 0.372, M.white, 'small'), (0.022, -0.052, 0.352, M.rose, 'tulip'),
             (-0.068, -0.008, 0.31, M.yellow, 'small'), (0.066, 0.01, 0.3, M.white, 'small'))
    for i, (x, y, z, petal, kind) in enumerate(heads):
        yaw, tilt = _facing(x, y), 25 if (x or y) else 0
        twig(f'stem{i}', [(x * 0.15, y * 0.15, 0.2), (x * 0.55, y * 0.55, (0.24 + z) / 2), (x, y, z - 0.004)], 0.0032,
             0.0026, M.leaf, verts=4, res=1)
        if kind == 'tulip':
            tulip_head(f'flower{i}', (x, y, z - 0.012), 0.017, 0.042, petal, yaw, tilt * 0.6)
        else:
            n, r, width = {'daisy': (12, 0.03, 0.7), 'small': (10, 0.021, 0.7), 'cosmos': (8, 0.028, 0.95)}[kind]
            rays(f'flower{i}', (x, y, z), r, n, petal, yaw, tilt, inner=0.25, cup=0.12, width=width, thick=0.004)
            dome(f'eye{i}', (x, y, z), r * 0.3, 0.006, M.orange if petal is M.yellow else M.yellow, yaw, tilt)
        if i % 2:                              # a leaf part way up the stem
            mx, my = x * 0.55, y * 0.55
            leaf(f'stem_leaf{i}', (mx, my, (0.24 + z) / 2 - 0.01), _facing(x, y) + 40, 35, 0.05, 0.018, M.leaf_light,
                 shape='lance', droop=0.3, thick=0.003, rows=3, ring=4)
    for i, a in enumerate((20, 140, 260)):
        bx, by = 0.022 * G.cosd(a), 0.022 * G.sind(a)
        leaf(f'leaf{i}', (bx, by, 0.232), _facing(bx, by), 40, 0.075, 0.03, M.leaf, shape='lance', droop=0.3, thick=0.004,
             rows=4, ring=4)
    return done('vaseFlowers')


def vaseSmall():
    """A glazed bud vase with a slim neck and a flared lip, one rose on a curving stem: a cup of outer petals round a
    furled bud, green sepals under it, a leaf on the stem."""
    turned('vase', [(0.026, 0.0), (0.03, 0.004), (0.039, 0.035), (0.04, 0.06), (0.025, 0.12), (0.013, 0.148),
                    (0.016, 0.16), (0.011, 0.16), (0.009, 0.15)], M.bud_vase, verts=16)
    twig('stem', [(0.0, 0.0, 0.14), (0.004, 0.002, 0.2), (0.012, 0.004, 0.25)], 0.003, 0.0026, M.leaf, verts=4, res=1)
    leaf('leaf', (0.004, 0.002, 0.205), 250, 35, 0.04, 0.022, M.leaf, shape='oval', droop=0.25, thick=0.003, rows=3,
         ring=4)
    star('sepals', (0.012, 0.004, 0.249), 0.013, 5, M.leaf, inner=0.3, depth=0.002, cup=-0.3)
    cupped('rose', [(0.0, 0.0), (0.011, 0.004), (0.017, 0.013), (0.0165, 0.022), (0.014, 0.0215)], (0.012, 0.004, 0.25),
           M.white, segments=8)
    cupped('rose_inner', [(0.0, 0.004), (0.009, 0.008), (0.0125, 0.018), (0.011, 0.025), (0.009, 0.0245)],
           (0.012, 0.004, 0.25), M.white, yaw=22, segments=7)
    tulip_head('rose_bud', (0.012, 0.004, 0.253), 0.0105, 0.024, M.white)
    return done('vaseSmall')


def potSucculent():
    """An echeveria in a pale pot with a rim, on grit: three tiers of fat, upturned leaves round a tight heart."""
    turned('pot', [(0.054, 0.0), (0.058, 0.004), (0.066, 0.056), (0.07, 0.06), (0.07, 0.07), (0.064, 0.07),
                   (0.062, 0.058)], M.pot_pale, verts=16)
    soil_top(None, 0.062, 0.066, segments=12, mound=0.006, seed=1.0, material=M.sand)
    for tier, (n, base, length, width, tilt, z, colour, rows) in enumerate((
            (8, 0.014, 0.072, 0.044, 20, 0.074, M.succulent, 3), (6, 0.009, 0.054, 0.036, 44, 0.084, M.succulent, 2),
            (5, 0.004, 0.044, 0.028, 72, 0.096, M.succulent_light, 2))):
        for i in range(n):
            a = i * 360 / n + tier * 22
            bx, by = base * G.cosd(a), base * G.sind(a)
            leaf(f'leaf{tier}_{i}', (bx, by, z), 90 - a, tilt, length, width, colour, shape='oval', thick=0.016, fold=0.18,
                 rows=rows, ring=4, droop=-0.12)
    dome('heart', (0, 0, 0.098), 0.012, 0.018, M.succulent_light, lift=0.0)
    return done('potSucculent')


def potCactus():
    """A ribbed column cactus in a terracotta pot with a rolled rim, on sand: two ribbed arms bent up from its sides,
    a pink flower on top."""
    turned('pot', [(0.048, 0.0), (0.052, 0.005), (0.06, 0.075), (0.066, 0.078), (0.067, 0.1), (0.058, 0.1),
                   (0.056, 0.092)], M.terracotta, verts=14)
    disc('sand', 0.0565, (0, 0, 0.094), M.sand, verts=14)
    secs = [[(u, v, z) for u, v in ribbed(r, 7, 0.16, verts=21)] for z, r in
            ((0.09, 0.034), (0.2, 0.036), (0.27, 0.031), (0.298, 0.02), (0.31, 0.006))]
    G.loft('cactus', secs, material=M.cactus)
    for s, (h, z) in ((1, (0.06, 0.16)), (-1, (0.05, 0.195))):
        pts = G.kit.fillet([(s * 0.02, 0, z), (s * 0.077, 0, z + 0.004), (s * 0.081, 0, z + h)], 0.022, steps=2)
        G.sweep(f'arm{s}', pts, None, profile=ribbed(0.016, 5, 0.2), material=M.cactus, angle=60,
                scale=lambda w: 1 - 0.55 * max(0.0, (w - 0.82) / 0.18) ** 1.5)
    rays('flower', (0, 0, 0.308), 0.022, 10, M.pink, inner=0.3, cup=0.45, width=0.9, centre=0.003)
    dome('flower_eye', (0, 0, 0.308), 0.006, 0.007, M.yellow)
    return done('potCactus')


def potSmall():
    """A peace lily in a glossy pot with a rolled rim: arching, pointed leaves on their stalks and two white spathes
    on tall stems."""
    turned('pot', [(0.05, 0.0), (0.054, 0.004), (0.062, 0.09), (0.068, 0.096), (0.068, 0.11), (0.06, 0.11),
                   (0.058, 0.1)], M.pot_white, verts=16)
    soil_top(None, 0.058, 0.106, segments=12, mound=0.008, seed=2.0, material=M.soil)
    for i in range(7):
        a = i * 360 / 7 + 10
        r0, lift = 0.012, 0.03 + (i % 3) * 0.012
        bx, by = r0 * G.cosd(a), r0 * G.sind(a)
        tx, ty = (r0 + 0.012) * G.cosd(a), (r0 + 0.012) * G.sind(a)
        twig(f'stalk{i}', [(bx * 0.5, by * 0.5, 0.104), (tx, ty, 0.104 + lift)], 0.0028, 0.0022, M.leaf, verts=3, res=1)
        reach = 1 + 0.19 * G.cosd(2 * a)           # longer to the sides than to the front and back (its footprint)
        leaf(f'leaf{i}', (tx, ty, 0.104 + lift), 90 - a, 70 - (i % 3) * 7, (0.088 + (i % 2) * 0.012) * reach, 0.04,
             M.leaf if i % 2 else M.leaf_light, shape='lance', droop=0.22, thick=0.004, fold=0.2, rows=4, ring=4)
    for i, (x, y, top, yaw) in enumerate(((0.012, -0.01, 0.261, 200), (-0.014, 0.012, 0.235, 30))):
        twig(f'spathe_stem{i}', [(x * 0.3, y * 0.3, 0.104), (x, y, top - 0.06), (x, y, top - 0.052)], 0.0026, 0.0022,
             M.leaf, verts=3, res=1)
        leaf(f'spathe{i}', (x, y, top - 0.055), yaw, 78, 0.055, 0.032, M.white, shape='oval', thick=0.003, fold=0.7,
             rows=3, ring=4, droop=-0.05)
        G.rod(f'spadix{i}', (x, y, top - 0.054), (x, y, top - 0.026), 0.0035, material=M.crumb, verts=5)
    return done('potSmall')


# --- TT4 small ornaments --------------------------------------------------------------------------------------
def _place(objs, matrix):
    """Moves made parts by `matrix` (a part built upright at the origin, then leaned and lifted into place)."""
    bpy.context.view_layer.update()            # each part's matrix_world as placed (the last one made isn't yet)
    for o in objs:
        o.matrix_world = matrix @ o.matrix_world


def photoFrame():
    """A photo frame leaning back on its strut: a moulded black frame (mitred), a white mount round a photo of a
    sunny hillside with someone in it, a back board."""
    TILT, ZC = -10, 0.116
    F = G.GameFrame()
    with G.new_parts() as upright:
        G.kit.moulding(F, 'frame', [(0.09, -0.115), (0.09, 0.115), (-0.09, 0.115), (-0.09, -0.115)],
                       [(0.0, -0.008), (0.0, 0.004), (0.004, 0.009), (0.012, 0.011), (0.018, 0.007), (0.022, 0.003),
                        (0.022, -0.008)], M.frame)
        box('back_board', (0.17, 0.004, 0.22), (0, 0.007, 0), M.frame)
        box('mount', (0.142, 0.003, 0.192), (0, 0.0035, 0), M.paper)
        box('photo_sky', (0.1, 0.002, 0.14), (0, 0.0015, 0.01), M.photo_sky)
        hills = [(-0.05, -0.06), (0.05, -0.06), (0.05, -0.012), (0.03, -0.005), (0.012, -0.016), (-0.01, -0.006),
                 (-0.032, -0.022), (-0.05, -0.015)]
        slab('photo_hill', hills, 0.002, (0, 0.0005, 0), M.photo_hill)
        turned('photo_sun', [(0.009, 0.0), (0.009, 0.001)], M.yellow, at=(0.03, 0.0004, 0.048), rot=(90, 0, 0), verts=8)
        ball('photo_head', 0.012, (-0.008, -0.0003, 0.012), M.blush, scale=(1, 0.12, 1.15), seg=8, rings=5)
        ball('photo_body', 0.022, (-0.008, -0.0003, -0.022), M.teapot_trim, scale=(1.2, 0.1, 0.95), seg=8, rings=5)
    _place(upright, Matrix.Translation((0, 0, ZC)) @ Matrix.Rotation(math.radians(TILT), 4, 'X'))
    slab('strut', [(-0.012, -0.08), (0.012, -0.08), (0.016, 0.08), (-0.016, 0.08)], 0.008, (0, 0.048, 0.075), M.frame,
         rot=(25, 0, 0))
    return done('photoFrame')


def _ticks(name, r, y, z, material, S):
    """The twelve hour marks round a clock's face (radius `r`, its middle at height `z`), flat on it at depth `y`, the
    quarters longer: one mesh of flat bars facing the front."""
    vs, faces = [], []
    for i in range(12):
        a = math.radians(i * 30)
        length, w = (0.014 if i % 3 == 0 else 0.009) * S, 0.0028
        cx, cz = r * math.sin(a), z + r * math.cos(a)
        dx, dz, tx, tz = math.sin(a), math.cos(a), math.cos(a), -math.sin(a)
        k = len(vs)
        for p, q in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            vs.append((cx + dx * length / 2 * p + tx * w / 2 * q, y, cz + dz * length / 2 * p + tz * w / 2 * q))
        faces.append((k, k + 1, k + 2, k + 3))
    obj = G.forms._object(name, vs, faces, material, False, 40)
    if sum(p.normal.y for p in obj.data.polygons) > 0:     # flat bars: face the front, where they're seen
        obj.data.flip_normals()
    return obj


def tableClock():
    """A twin-bell alarm clock: a red drum case rolled at the back with a raised bezel, a cream face with its hour marks,
    two brass bells on their posts with the hammer between them, a carrying loop, splayed legs on bun feet. 16.5 cm tall
    in all (a real one 12-18 cm), built at 0.72 of its first size (`S`), which was half again too tall."""
    S = 0.72
    Z, R = 0.112 * S, 0.074 * S
    with G.new_parts() as body:
        turned('case', [(R * 0.9, -0.025 * S), (R, -0.017 * S), (R, 0.016 * S), (R * 1.01, 0.024 * S),
                        (R * 0.87, 0.027 * S), (R * 0.86, 0.021 * S)], M.clock, at=(0, 0, Z), rot=(90, 0, 0), verts=20)
        face = disc('face', R * 0.862, (0, 0, 0), M.clock_face, verts=20)
        face.rotation_euler = (math.radians(90), 0, 0)
        face.location = (0, -0.0235 * S, Z)
        _ticks('ticks', 0.054 * S, -0.0235 * S - 0.0004, Z, M.dark, S)
        cyl('hub', 0.004, 0.003, (0, -0.025 * S - 0.006, Z), M.red, verts=8, rot=(90, 0, 0))
        for s in (-1, 1):
            turned(f'bell{s}', [(0.032 * S, 0.0), (0.031 * S, 0.01 * S), (0.022 * S, 0.024 * S), (0.008 * S, 0.03 * S),
                                (0.0, 0.031 * S)], M.brass, at=(s * 0.0445 * S, 0.0, Z + 0.06 * S), rot=(0, s * 30, 0),
                   verts=10)
            G.sweep(f'leg{s}', [(s * 0.04 * S, 0.0, Z - 0.05 * S), (s * 0.056 * S, -0.004 * S, 0.02 * S),
                                (s * 0.06 * S, -0.008 * S, 0.008 * S)], 0.005 * S, material=M.brass, verts=5)
            turned(f'foot{s}', [(0.009 * S, 0.0), (0.009 * S, 0.004 * S), (0.005 * S, 0.009 * S), (0.0, 0.0105 * S)],
                   M.brass, at=(s * 0.06 * S, -0.008 * S, 0), verts=6)
        G.rod('bell_post', (0, 0.0, Z + R - 0.004 * S), (0, 0.0, Z + 0.1 * S), 0.0035 * S, material=M.brass, verts=6)
        ball('hammer', 0.007 * S, (0, 0.0, Z + 0.103 * S), M.brass, seg=6, rings=4)
        G.sweep('loop', G.kit.fillet([(-0.026 * S, 0.012 * S, Z + R * 0.9), (-0.026 * S, 0.012 * S, Z + 0.112 * S),
                                      (0.026 * S, 0.012 * S, Z + 0.112 * S), (0.026 * S, 0.012 * S, Z + R * 0.9)],
                                     0.012 * S, steps=2), 0.0028 * S, material=M.brass, verts=4)
    G.join(body, 'clock', origin=(0, 0, 0))
    # the hour and minute hands: parts of their own round the face's middle, telling the game's time
    # (`motions.clock`), at rest ten past ten
    hands = []
    for name, (w, depth, length), y, angle in (('hour', (0.005, 0.002, 0.032 * S), -0.025 * S - 0.004, 305),
                                               ('minute', (0.004, 0.002, 0.05 * S), -0.025 * S - 0.005, 60)):
        with G.new_parts() as hand:
            box(f'{name}_hand', (w, depth, length), (G.sind(angle) * length / 2, y, Z + G.cosd(angle) * length / 2), M.dark,
                rot=(0, angle, 0))
        hands.append(G.hinged(G.join(hand, f'{name}_hand'), (0, y, Z)))
    G.settle()
    for hand in hands:
        G.spin(hand, hand.name, 1, seconds=12, axis='Y')   # once round clockwise; the game holds it at the time
    return G.meshes()


# the continents, roughly: (longitude, latitude, reach in degrees)
CONTINENTS = ((-100, 48, 24), (-80, 30, 14), (-60, -15, 20), (-68, -38, 10), (12, 50, 14), (35, 58, 18), (20, 8, 22),
              (25, -20, 14), (85, 50, 30), (110, 28, 16), (78, 20, 10), (135, -25, 13), (-42, 72, 9))


def globe():
    """A desk globe: the blue sea with its continents (faces of the sphere coloured land), tilted on its axis in a brass
    meridian ring with pins at the poles, on a turned wooden column and foot."""
    turned('base', [(0.068, 0.0), (0.068, 0.008), (0.062, 0.014), (0.03, 0.022), (0.02, 0.026)], M.wood_dark, verts=12)
    turned('column', [(0.016, 0.024), (0.011, 0.045), (0.015, 0.056), (0.008, 0.066), (0.006, 0.077)], M.wood_dark,
           verts=8)
    Z, R = 0.19, 0.105
    sea = ball('sea', R / G.cosd(11.25), (0, 0, Z), M.globe, rot=(23, 0, 0), seg=16, rings=12)   # (its sides between corners)
    sea.data.materials.append(M.land)
    for p in sea.data.polygons:
        c = p.center.normalized()
        lat, lon = math.degrees(math.asin(max(-1.0, min(1.0, c.z)))), math.degrees(math.atan2(c.y, c.x))
        for clon, clat, reach in CONTINENTS:
            d = math.degrees(math.acos(max(-1.0, min(1.0, math.sin(math.radians(lat)) * math.sin(math.radians(clat)) +
                                                     math.cos(math.radians(lat)) * math.cos(math.radians(clat)) *
                                                     math.cos(math.radians(lon - clon))))))
            if d < reach:
                p.material_index = 1
                break
    ring = [(0, 0.117 * math.cos(math.tau * i / 18), Z + 0.117 * math.sin(math.tau * i / 18)) for i in range(18)]
    G.sweep('meridian', ring, None, profile=G.rounded_rect(0.004, 0.008, 0.001, corner=1)[::2], material=M.brass,
            closed=True, up=(0, 1, 0))
    a = math.radians(23)
    for k in (-1, 1):
        G.rod(f'pin{k}', (0, k * -R * 0.98 * math.sin(a), Z + k * R * 0.98 * math.cos(a)),
              (0, k * -0.118 * math.sin(a), Z + k * 0.118 * math.cos(a)), 0.003, material=M.brass, verts=6)
    return done('globe')


def snowGlobe():
    """A snow globe: a turned wooden base with a brass band, the glass dome, inside it a snowy mound with a fir tree in
    three tiers, a little red cabin with snow on its roof, a snowman and snowflakes in the water."""
    turned('base', [(0.064, 0.0), (0.064, 0.008), (0.06, 0.03), (0.054, 0.05), (0.046, 0.052), (0.045, 0.062),
                    (0.0, 0.063)], M.wood_dark, verts=16)
    turned('band', [(0.0634, 0.017), (0.0634, 0.019), (0.0617, 0.027)], M.brass, verts=16)
    Z = 0.112
    ball('dome', 0.064, (0, 0, Z), M.glass, seg=14, rings=9)
    turned('ground', [(0.04, 0.06), (0.039, 0.066), (0.028, 0.073), (0.0, 0.077)], M.snow, verts=10)
    G.rod('trunk', (-0.008, 0.008, 0.07), (-0.008, 0.008, 0.082), 0.004, material=M.wood_dark, verts=4)
    for i, (z, r, h) in enumerate(((0.079, 0.021, 0.03), (0.097, 0.016, 0.026), (0.113, 0.011, 0.022))):
        G.cone(f'tree{i}', r, 0.0, h, at=(-0.008, 0.008, z + h / 2), material=M.fir, verts=7)
    box('cabin', (0.02, 0.016, 0.015), (0.02, 0.012, 0.074), M.red)
    slab('roof', [(-0.014, 0.0), (0.014, 0.0), (0.0, 0.011)], 0.02, (0.02, 0.012, 0.081), M.snow)
    ball('snowman_base', 0.008, (0.01, -0.022, 0.077), M.snow, seg=6, rings=4)
    ball('snowman_head', 0.0055, (0.01, -0.022, 0.089), M.snow, seg=6, rings=4)
    for i, (x, y, z) in enumerate(((0.03, -0.02, 0.13), (-0.035, 0.0, 0.15), (0.01, 0.03, 0.16), (-0.02, -0.03, 0.1),
                                   (0.035, 0.02, 0.095), (-0.03, 0.025, 0.125))):
        ball(f'flake{i}', 0.0035, (x, y, z), M.snow, seg=4, rings=2)
    return done('snowGlobe')


def trophy():
    """A cup trophy: a two-step black plinth with a brass plaque, a turned stem with a knop, a flaring cup with a
    rolled rim and two scrolled handles."""
    box('base', (0.1, 0.1, 0.03), (0, 0, 0.015), M.trophy_base, bevel=0.004, segs=1)
    box('base_top', (0.08, 0.08, 0.012), (0, 0, 0.036), M.trophy_base, bevel=0.002, segs=1)
    box('plaque', (0.06, 0.002, 0.018), (0, -0.0505, 0.015), M.trophy)
    turned('stem', [(0.03, 0.042), (0.03, 0.047), (0.012, 0.058), (0.009, 0.085), (0.018, 0.095), (0.009, 0.104),
                    (0.014, 0.11)], M.trophy, verts=12)
    turned('cup', [(0.014, 0.108), (0.03, 0.118), (0.05, 0.145), (0.06, 0.19), (0.063, 0.218), (0.065, 0.226),
                   (0.06, 0.226), (0.057, 0.205), (0.0, 0.2)], M.trophy, verts=14)
    for s in (-1, 1):
        G.sweep(f'handle{s}', [(s * 0.056, 0, 0.205), (s * 0.066, 0, 0.202), (s * 0.0695, 0, 0.18), (s * 0.062, 0, 0.158),
                               (s * 0.05, 0, 0.142)], 0.0045, material=M.trophy, verts=6, resolution=2,
                scale=[1.0, 1.1, 1.0, 0.9, 0.8])
    return done('trophy')


def _mobius(name, r, w, t, n, material):
    """A flat band `w` wide and `t` thick round a circle of radius `r` in the x-z plane (Blender), turning half over on
    its way round (a Mobius band), `n` sections."""
    prof = [(-w / 2, -t / 2), (w / 2, -t / 2), (w / 2, t / 2), (-w / 2, t / 2)]
    vs, faces = [], []
    for i in range(n):
        phi, tw = math.tau * i / n, math.pi * i / n
        c, rad = Vector((r * math.cos(phi), 0, r * math.sin(phi))), Vector((math.cos(phi), 0, math.sin(phi)))
        for u, v in prof:
            uu, vv = u * math.cos(tw) - v * math.sin(tw), u * math.sin(tw) + v * math.cos(tw)
            vs.append(c + rad * vv + Vector((0, uu, 0)))
    for i in range(n):
        for k in range(4):
            k2 = (k + 1) % 4
            if i < n - 1:
                faces.append((i * 4 + k, i * 4 + k2, (i + 1) * 4 + k2, (i + 1) * 4 + k))
            else:                                  # round to the start, half turned: corner k meets corner k + 2
                faces.append((i * 4 + k, i * 4 + k2, (k2 + 2) % 4, (k + 2) % 4))
    return G.forms._object(name, vs, faces, material, True, 40)


def sculptureSmall():
    """An abstract sculpture: a black band standing on edge and turning half over on its way round (a Mobius band), a
    brass ball resting inside it, on a pale plinth with a brass name plate."""
    box('plinth', (0.12, 0.12, 0.05), (0, 0, 0.025), M.plinth, bevel=0.004, segs=1)
    box('plate', (0.05, 0.002, 0.012), (0, -0.0605, 0.03), M.brass)
    band = _mobius('ring', 0.071, 0.036, 0.008, 28, M.sculpture)
    low = min(v.co.z for v in band.data.vertices)
    band.location = (0, 0, 0.048 - low)
    band.rotation_euler = (0, 0, math.radians(30))
    ball('ball', 0.03, (0.012, -0.01, 0.08), M.brass, seg=10, rings=7)
    return done('sculptureSmall')


# --- TT5 bath, desk and toys ----------------------------------------------------------------------------------
def soapSet():
    """A bathroom set on a stone tray: a pump dispenser (its collar, the pump head and spout in one piece), a tumbler
    with two toothbrushes (bristles on their heads), a bar of soap on drainer ridges."""
    slab('tray', G.rounded_rect(0.26, 0.12, 0.02, corner=2), 0.012, (0, 0, 0.006), M.soap_tray, plane='XY', bevel=0.002)
    for i in range(3):
        box(f'ridge{i}', (0.006, 0.07, 0.004), (-0.11 + i * 0.02, 0, 0.013), M.soap_tray)
    turned('dispenser', [(0.028, 0.012), (0.031, 0.018), (0.031, 0.105), (0.026, 0.118), (0.016, 0.124)], M.dispenser,
           at=(0.08, 0, 0), verts=12)
    turned('pump_collar', [(0.016, 0.122), (0.016, 0.132), (0.011, 0.135)], M.dark, at=(0.08, 0, 0), verts=10)
    G.rod('pump', (0.08, 0, 0.134), (0.08, 0, 0.148), 0.005, material=M.dark, verts=6)
    bar('pump_head', G.kit.fillet([(0.08, 0.008, 0.154), (0.08, -0.008, 0.156), (0.08, -0.034, 0.154),
                                   (0.08, -0.036, 0.146)], 0.006, steps=1), G.rounded_rect(0.012, 0.01, 0.004, corner=1),
        M.dark, up=(1, 0, 0))
    turned('tumbler', [(0.026, 0.012), (0.03, 0.09), (0.027, 0.09), (0.024, 0.02), (0.0, 0.018)], M.tumbler,
           at=(-0.02, 0, 0), verts=12)
    for i, (colour, top) in enumerate(((M.blush, (-0.004, 0.012, 0.165)), (M.leaf_light, (-0.042, -0.012, 0.158)))):
        base = Vector((-0.02, 0, 0.03))
        d = (Vector(top) - base).normalized()
        bar(f'brush{i}', [tuple(base), tuple(base + d * 0.09), tuple(base + d * 0.105), tuple(Vector(top))],
            oval(0.007, 0.01, 6), colour, up=(0, -1, 0), scale=[1.0, 0.75, 1.0, 1.0])
        bt = Vector(top) - d * 0.012 + Vector((0, -0.007, 0))
        box(f'bristles{i}', (0.008, 0.008, 0.02), tuple(bt), M.white, bevel=0.002, segs=1)
    box('soap', (0.07, 0.05, 0.022), (-0.09, 0, 0.026), M.soap, bevel=0.008, segs=1)
    return done('soapSet')


def towelStack():
    """Three folded towels stacked a little askew, each folded over on itself (two soft layers, the fold showing at the
    front), with a woven band across its front."""
    for i, (colour, band, dy, turn) in enumerate(((M.towel_a, M.towel_b, 0.0, 0), (M.towel_b, M.towel_a, -0.006, 3),
                                                  (M.towel_c, M.towel_b, 0.004, -2))):
        z = i * 0.05
        with G.new_parts() as towel:
            for k in range(2):
                G.soft_box(f'towel{i}_{k}', (0.26 - k * 0.004, 0.18 - k * 0.006, 0.0245), at=(0, k * 0.003, 0.0125 + k * 0.024),
                           material=colour, round=0.011, puff=0.002, round_segments=1, flat=1)
            box(f'band{i}', (0.2, 0.002, 0.007), (0, -0.0905, 0.03), band)
        _place(towel, Matrix.Translation((0, dy, z)) @ Matrix.Rotation(math.radians(turn), 4, 'Z'))
    return done('towelStack')


def perfumeTray():
    """A mirrored vanity tray with a brass gallery round it, three scent bottles: a faceted pink one with a brass
    stopper, a round flacon on its edge with an atomizer bulb, and a tall blue one with a black cap."""
    slab('tray', G.rounded_rect(0.268, 0.168, 0.004, corner=1), 0.008, (0, 0, 0.004), M.perfume_tray, plane='XY')
    G.kit.moulding(G.GameFrame(), 'gallery', [(0.138, 0.088), (-0.138, 0.088), (-0.138, -0.088), (0.138, -0.088)],
                   [(0.0, 0.0), (0.0, 0.016), (0.003, 0.0175), (0.006, 0.016), (0.006, 0.0)], M.brass, at=0.0, plane='top')
    turned('bottle_a', [(0.03, 0.02, 0.008), (0.031, 0.021, 0.012), (0.031, 0.021, 0.094), (0.024, 0.017, 0.1),
                        (0.01, 0.01, 0.103)], M.perfume_a, at=(0.07, 0, 0), verts=16, square=6, angle=30)
    G.rod('neck_a', (0.07, 0, 0.1), (0.07, 0, 0.112), 0.008, material=M.brass, verts=8)
    ball('cap_a', 0.017, (0.07, 0, 0.126), M.brass, scale=(1, 0.75, 1), seg=8, rings=6)
    turned('bottle_b', [(0.03, -0.012), (0.034, -0.007), (0.034, 0.007), (0.03, 0.012)], M.perfume_b,
           at=(-0.02, -0.02, 0.044), rot=(90, 0, 0), verts=14)
    G.rod('neck_b', (-0.02, -0.02, 0.076), (-0.02, -0.02, 0.09), 0.007, material=M.brass, verts=6)
    G.sweep('tube_b', [(-0.02, -0.02, 0.088), (-0.03, -0.03, 0.096), (-0.045, -0.04, 0.084), (-0.05, -0.044, 0.07)],
            0.0018, material=M.dark, verts=4, resolution=2)
    ball('bulb_b', 0.011, (-0.052, -0.046, 0.062), M.dark, scale=(1, 1, 1.25), seg=6, rings=5)
    turned('bottle_c', [(0.025, 0.008), (0.025, 0.074), (0.022, 0.08)], M.perfume_c, at=(-0.08, 0.03, 0), verts=12)
    turned('cap_c', [(0.015, 0.078), (0.015, 0.1), (0.012, 0.104)], M.dark, at=(-0.08, 0.03, 0), verts=10)
    return done('perfumeTray')


def _pen(name, a, b, material, cap=None, pencil=False):
    """A pen standing in a pot from Blender point `a` up to `b`: its barrel, a cap with a clip; or a hexagonal pencil
    sharpened to a point."""
    a, b = Vector(a), Vector(b)
    d = (b - a).normalized()
    if pencil:
        G.sweep(name, [tuple(a), tuple(b - d * 0.014)], 0.0045, material=material, verts=6)
        G.sweep(f'{name}_tip', [tuple(b - d * 0.014), tuple(b)], 0.0045, material=M.crumb, verts=6, scale=[1.0, 0.15])
        return
    G.sweep(name, [tuple(a), tuple(b - d * 0.03)], 0.0042, material=material, verts=6)
    G.sweep(f'{name}_cap', [tuple(b - d * 0.032), tuple(b)], 0.0048, material=cap or M.dark, verts=6, scale=[1.0, 0.8])
    side = d.cross(Vector((0, 0, 1))).normalized() if abs(d.z) < 0.999 else Vector((1, 0, 0))
    G.rod(f'{name}_clip', tuple(b - d * 0.026 + side.cross(d) * 0.005), tuple(b - d * 0.004 + side.cross(d) * 0.005),
          0.0012, material=M.chrome, verts=3)


def deskOrganiser():
    """A desk organiser: a two-tier letter tray (shaped sides scooped at the front, a back, posts) with paper in each
    tier, and a pen pot holding two pens with clipped caps and a sharpened pencil."""
    side = G.smooth_outline([(-0.1, 0.0), (0.1, 0.0), (0.1, 0.03), (-0.035, 0.03), (-0.07, 0.013), (-0.1, 0.011)],
                            passes=1)
    for tier in range(2):
        z = tier * 0.05
        box(f'shelf{tier}', (0.16, 0.2, 0.006), (-0.06, 0, z + 0.003), M.organiser)
        box(f'paper{tier}', (0.14, 0.19, 0.008), (-0.06, 0.003, z + 0.01), M.paper)
        box(f'sheet{tier}', (0.138, 0.186, 0.0015), (-0.058, 0.0, z + 0.0148), M.paper, rot=(0, 0, 3 - tier * 5))
        for s in (-1, 1):
            slab(f'side{tier}{s}', [(-u, v) for u, v in side], 0.006, (-0.06 + s * 0.077, 0, z + 0.003), M.organiser,
                 plane='YZ')
        box(f'back{tier}', (0.16, 0.006, 0.03), (-0.06, 0.097, z + 0.018), M.organiser)
    for s in (-1, 1):
        box(f'post{s}', (0.006, 0.006, 0.05), (-0.06 + s * 0.077, -0.094, 0.04), M.organiser)
    turned('pen_pot', [(0.035, 0.0), (0.035, 0.1), (0.031, 0.1), (0.031, 0.09)], M.organiser, at=(0.09, 0.02, 0), verts=12)
    _pen('pen0', (0.09, 0.02, 0.01), (0.105, 0.02, 0.16), M.pen_red)
    _pen('pen1', (0.09, 0.02, 0.01), (0.078, 0.03, 0.158), M.pen_blue)
    _pen('pencil', (0.09, 0.02, 0.01), (0.09, 0.005, 0.16), M.pen_yellow, pencil=True)
    return done('deskOrganiser')


def laptopClosed():
    """A closed laptop: an aluminium base with eased edges, a thumb notch at the front, ports on its side and a hinge
    barrel at the back, a keyboard and a touchpad on its top; the lid a part of its own, hinged at its back edge, the
    logo on it and the screen under it: it opens while the laptop is used (`lid_open`)."""
    with G.new_parts() as body:
        slab('base', G.rounded_rect(0.32, 0.22, 0.012, corner=2), 0.011, (0, 0, 0.0055), M.laptop, plane='XY',
             bevel=0.0025)
        G.cylinder('hinge', 0.0055, 0.26, at=(0, 0.106, 0.012), material=M.dark, verts=8, rot=(0, 90, 0))
        box('notch', (0.05, 0.003, 0.003), (0, -0.1095, 0.0115), M.dark)
        for i, (y, w) in enumerate(((-0.03, 0.012), (-0.005, 0.009), (0.02, 0.009))):
            box(f'port{i}', (0.002, w, 0.004), (0.1601, y, 0.006), M.dark)
        # the keyboard's well and the touchpad, hidden under the lid until it opens
        box('keyboard', (0.27, 0.105, 0.0006), (0, 0.025, 0.0109), M.dark)
        box('touchpad', (0.1, 0.06, 0.0004), (0, -0.068, 0.0109), M.charcoal)
    G.join(body, 'laptop', origin=(0, 0, 0))
    with G.new_parts() as lid:
        slab('lid', G.rounded_rect(0.318, 0.218, 0.012, corner=2), 0.008, (0, 0, 0.0155), M.laptop, plane='XY',
             bevel=0.002)
        turned('logo', [(0.015, 0.0195), (0.015, 0.0201)], M.logo, at=(0, 0.01, 0), verts=12)
        box('screen', (0.29, 0.18, 0.0005), (0, 0.006, 0.01175), M.laptop_screen)   # under the lid, facing the keyboard
    lid = G.hinged(G.join(lid, 'lid'), (0, 0.109, 0.0115))
    G.settle()
    G.swing(lid, 'lid_open', -105, seconds=0.8, axis='X')   # its front up and over, the screen to the front
    return G.meshes()


def monitor():
    """A desk monitor on a stand (a rounded foot, a curved neck into the back of the housing), a slim bezel round the
    dark screen with a chrome logo and a power light; a keyboard of separate keys on its rounded base, a mouse."""
    slab('foot', G.rounded_rect(0.2, 0.14, 0.04, corner=2), 0.012, (0, 0.08, 0.006), M.charcoal, plane='XY', bevel=0.003)
    bar('neck', G.kit.fillet([(0, 0.098, 0.01), (0, 0.112, 0.11), (0, 0.104, 0.22)], 0.04, steps=2),
        G.rounded_rect(0.05, 0.022, 0.006, corner=1), M.charcoal, up=(1, 0, 0))
    G.soft_box('housing', (0.46, 0.03, 0.25), at=(0, 0.098, 0.3), material=M.monitor, round=0.012, puff=(0, 0.006, 0),
               round_segments=1, flat=2)
    hole = [(u, v + 0.006) for u, v in G.rounded_rect(0.52, 0.29, 0.002, corner=1)]
    slab('bezel', G.rounded_rect(0.54, 0.32, 0.008, corner=2), 0.022, (0, 0.075, 0.3), M.monitor, bevel=0.002,
         holes=[hole])
    box('screen', (0.522, 0.004, 0.292), (0, 0.067, 0.306), M.screen)
    box('chin_logo', (0.03, 0.002, 0.005), (0, 0.0635, 0.149), M.chrome)
    box('power_light', (0.004, 0.002, 0.002), (0.24, 0.0635, 0.149), M.white)
    slab('keyboard', G.rounded_rect(0.38, 0.12, 0.008, corner=2), 0.012, (0, -0.12, 0.006), M.monitor, plane='XY',
         bevel=0.003)
    for row in range(4):
        for k in range(13):
            box(f'key{row}_{k}', (0.021, 0.02, 0.006), (-0.153 + k * 0.0255, -0.152 + row * 0.024, 0.015), M.keys)
    for x, w in ((-0.153, 0.021), (-0.122, 0.03), (0.0, 0.15), (0.122, 0.03), (0.153, 0.021)):
        box(f'key4_{x}', (w, 0.018, 0.006), (x, -0.176, 0.015), M.keys)
    turned('mouse', [(0.019, 0.032, 0.0), (0.0225, 0.0365, 0.008), (0.021, 0.034, 0.02), (0.013, 0.022, 0.028),
                     (0.0, 0.0, 0.031)], M.monitor, at=(0.25, -0.12, 0), verts=12)
    box('wheel', (0.004, 0.01, 0.004), (0.25, -0.139, 0.029), M.keys)
    return done('monitor', mirror=True)


def stuffedToy():
    """A sitting teddy bear: a pear-shaped body with a pale tummy, a round head with a pale muzzle, nose and eyes,
    cupped ears, arms hanging forward with round paws, legs out in front with pale foot pads, a red bow at the neck."""
    turned('body', [(0.045, 0.04, 0.0), (0.068, 0.062, 0.025), (0.075, 0.068, 0.07), (0.066, 0.06, 0.12),
                    (0.042, 0.04, 0.152), (0.02, 0.02, 0.162)], M.bear, verts=10)
    ball('tummy', 0.044, (0, -0.058, 0.072), M.bear_light, scale=(1, 0.32, 1.1), seg=7, rings=4)
    turned('head', [(0.03, 0.03, 0.0), (0.055, 0.05, 0.015), (0.062, 0.058, 0.05), (0.055, 0.052, 0.085),
                    (0.034, 0.032, 0.106), (0.0, 0.0, 0.114)], M.bear, at=(0, -0.008, 0.152), verts=10)
    ball('muzzle', 0.026, (0, -0.062, 0.188), M.bear_light, scale=(1.1, 0.8, 0.8), seg=8, rings=5)
    ball('nose', 0.009, (0, -0.083, 0.196), M.dark, scale=(1.3, 1, 0.9), seg=5, rings=3)
    for s in (-1, 1):
        ball(f'eye{s}', 0.0065, (s * 0.023, -0.06, 0.216), M.dark, seg=4, rings=3)
        turned(f'ear{s}', [(0.023, 0.0), (0.02, 0.011), (0.0, 0.016)], M.bear,
               at=(s * 0.046, -0.002, 0.256), rot=(-90, 0, s * -15), verts=8)
        disc(f'ear_in{s}', 0.014, (0, 0, 0), M.bear_light, verts=6).matrix_world = (
            Matrix.Translation((s * 0.046, -0.0026, 0.256)) @ Matrix.Rotation(math.radians(-s * 15), 4, 'Z')
            @ Matrix.Rotation(math.radians(90), 4, 'X'))
        G.sweep(f'arm{s}', [(s * 0.05, -0.008, 0.135), (s * 0.07, -0.03, 0.105), (s * 0.074, -0.055, 0.075)], 0.024,
                material=M.bear, verts=6, scale=[0.8, 1.0, 0.95])
        ball(f'paw{s}', 0.023, (s * 0.074, -0.057, 0.072), M.bear, seg=6, rings=4)
        G.sweep(f'leg{s}', [(s * 0.045, -0.02, 0.032), (s * 0.05, -0.08, 0.03), (s * 0.052, -0.126, 0.03)], 0.03,
                material=M.bear, verts=7, scale=[0.9, 1.0, 1.0])
        pad = disc(f'pad{s}', 0.02, (0, 0, 0), M.bear_light, verts=7, rx=0.017)
        pad.matrix_world = Matrix.Translation((s * 0.052, -0.1265, 0.031)) @ Matrix.Rotation(math.radians(90), 4, 'X')
        ball(f'bow{s}', 0.016, (s * 0.018, -0.052, 0.152), M.red, scale=(1.3, 0.5, 0.9), rot=(0, s * 15, 0), seg=5, rings=3)
    ball('knot', 0.007, (0, -0.058, 0.152), M.red, seg=5, rings=3)
    return done('stuffedToy')


LETTERS = {
    'A': ([(-0.018, -0.02), (-0.009, -0.02), (-0.006, -0.012), (0.006, -0.012), (0.009, -0.02), (0.018, -0.02),
           (0.004, 0.02), (-0.004, 0.02)], [[(-0.0035, -0.004), (0.0035, -0.004), (0.0, 0.008)]]),
    'L': ([(-0.014, -0.02), (0.014, -0.02), (0.014, -0.012), (-0.005, -0.012), (-0.005, 0.02), (-0.014, 0.02)], []),
    'T': ([(-0.004, -0.02), (0.004, -0.02), (0.004, 0.012), (0.016, 0.012), (0.016, 0.02), (-0.016, 0.02),
           (-0.016, 0.012), (-0.004, 0.012)], []),
    'H': ([(-0.016, -0.02), (-0.008, -0.02), (-0.008, -0.004), (0.008, -0.004), (0.008, -0.02), (0.016, -0.02),
           (0.016, 0.02), (0.008, 0.02), (0.008, 0.004), (-0.008, 0.004), (-0.008, 0.02), (-0.016, 0.02)], []),
    'O': (G.rounded_rect(0.032, 0.04, 0.013, corner=2), [G.rounded_rect(0.014, 0.022, 0.006, corner=2)]),
}


def toyBlocks():
    """Wooden toy blocks, their edges rounded, a raised letter on two faces of each, and a triangular roof block on top
    of the stack."""
    B = 0.065
    for i, (x, y, z, colour, turn, letters) in enumerate(((-0.05, 0.02, 0, M.block_a, 10, 'AL'),
                                                          (0.03, 0.03, 0, M.block_b, -15, 'TH'),
                                                          (-0.01, 0.02, 1, M.block_c, 30, 'OA'),
                                                          (0.08, -0.06, 0, M.block_d, 20, 'HO'),
                                                          (-0.07, -0.07, 0, M.block_e, 0, 'LT'))):
        with G.new_parts() as block:
            box(f'block{i}', (B, B, B), (0, 0, B / 2), colour, bevel=0.006, segs=1)
            outline, holes = LETTERS[letters[0]]
            slab(f'letter{i}a', outline, 0.003, (0, -B / 2 - 0.0008, B / 2), M.white, holes=holes)
            outline, holes = LETTERS[letters[1]]
            slab(f'letter{i}b', [(-u, v) for u, v in outline], 0.003, (B / 2 + 0.0008, 0, B / 2), M.white, plane='YZ',
                 holes=[[(-u, v) for u, v in h] for h in holes])
        _place(block, Matrix.Translation((x, y, z * B)) @ Matrix.Rotation(math.radians(turn), 4, 'Z'))
    # a roof on the top block: a triangular prism
    roof = slab('roof', [(-B / 2 - 0.003, 0.0), (B / 2 + 0.003, 0.0), (0.0, 0.07)], B, (0, 0, 0), M.block_d)
    roof.matrix_world = Matrix.Translation((-0.01, 0.02, 2 * B)) @ Matrix.Rotation(math.radians(30), 4, 'Z')
    return done('toyBlocks')


def T(id, build, size, paint, **kw):
    return G.Model(id, build, size=size, key=f'model.{id}', paint=paint, **kw)


BODY = 'Body'
FAMILY = G.Family(
    'tabletop', M,
    [  # TT1 kitchenware
        T('kettle', kettle, (0.228, 0.238, 0.172), [('kettle', BODY), ('dark', 'Base, handle & lid')],
          motions={'uses': {'tea': {'clip': 'lid_open', 'hold': 1.6}}}),
        T('toaster', toaster, (0.28, 0.23, 0.15), [('toaster', BODY)]),
        T('blender', blender, (0.18, 0.356, 0.146), [('blender_base', BODY), ('smoothie', 'Smoothie')]),
        T('standMixer', standMixer, (0.3, 0.356, 0.17), [('mixer', BODY)]),
        T('espressoMachineSmall', espressoMachineSmall, (0.231, 0.31, 0.391), [('espresso', BODY)]),
        T('knifeBlock', knifeBlock, (0.12, 0.297, 0.232), [('knife_block', BODY), ('dark', 'Handles')]),
        T('utensilCrock', utensilCrock, (0.132, 0.316, 0.124), [('crock', BODY)]),
        T('cuttingBoard', cuttingBoard, (0.34, 0.106, 0.23), [('board', BODY)]),
        T('breadBin', breadBin, (0.34, 0.206, 0.202), [('bread_bin', BODY), ('wood', 'Ends & handle')],
          motions={'uses': {'sandwich': {'clip': 'roll_open', 'hold': 2.4}}}),
        T('spiceJars', spiceJars, (0.3, 0.112, 0.12), [('rack', BODY)]),
        T('cookieJar', cookieJar, (0.16, 0.248, 0.158), [('jar_lid', 'Lid')]),
        # TT2 tableware
        T('fruitBowl', fruitBowl, (0.28, 0.194, 0.28), [('fruit_bowl', BODY)]),
        T('plateSet', plateSet, (0.378, 0.1, 0.321), [('plate', BODY), ('plate_rim', 'Rim'), ('napkin', 'Napkin')]),
        T('cakeStand', cakeStand, (0.28, 0.263, 0.28), [('cake_stand', BODY), ('icing', 'Icing')]),
        T('teapotSet', teapotSet, (0.4, 0.167, 0.26), [('teapot', BODY), ('tray', 'Tray')]),
        T('mugs', mugs, (0.216, 0.09, 0.212), [('mug_a', 'Mug 1'), ('mug_b', 'Mug 2'), ('mug_c', 'Mug 3')]),
        T('wineGlasses', wineGlasses, (0.214, 0.301, 0.197), [('bottle', BODY)]),
        T('candlesticks', candlesticks, (0.168, 0.328, 0.066), [('brass', BODY), ('candle', 'Candles')],
          motions={'night': 'lit'}),
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
        T('tableClock', tableClock, (0.11, 0.164, 0.048), [('clock', BODY), ('brass', 'Bells & legs')],
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
        T('laptopClosed', laptopClosed, (0.32, 0.0205, 0.223), [('laptop', BODY)],
          motions={'uses': {'type': 'lid_open'}}),   # open while it is used (browse, code, write, homework)
        T('monitor', monitor, (0.542, 0.46, 0.335), [('monitor', BODY)]),
        T('stuffedToy', stuffedToy, (0.197, 0.282, 0.192), [('bear', BODY)]),
        T('toyBlocks', toyBlocks, (0.222, 0.2, 0.171), [('block_a', 'Block 1'), ('block_b', 'Block 2'),
                                                       ('block_c', 'Block 3')]),
    ],
    comment='Blender-built tabletop things (tools/art/blender/families/tabletop.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes, so each costs one draw call '
            '(the see-through ones and the parts kept for a use to come one more, the alarm clock\'s two '
            'hands two more).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
