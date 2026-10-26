# The things residents carry and cook with: the models of content `items` (`item.*`: a bag of shopping, a
# prep bowl, a pot of stew, plates of food, dirty plates and pans, the food bought in town), the props drawn on a
# station's top while a resident cooks there (`tool.*`: the chopping board, the pan, food on the grill) and the props
# in a resident's hands while they use a thing (`tool.*`: a book, a cup, cutlery, a phone, a cue; actions.ts
# `PROPS`). Notes: held.md.
#
# Each is drawn as the old part was (characters.ts `writeCarried`, `writeTool`; items.ts): its origin at
# the middle of its base, front +Z in the game, lifted onto the palms, the top it stands on or the station.
# So each is built where the game places it, in the game's axes (`F`), and stood on the floor
# without moving it sideways (`done`); the few that stand off their middle (the pan's handle, the bag)
# take their measured `centre`.
#
# Small and seen several at once: one draw call each (every finish a plain one), two for the wooden boards
# (the chopping board, the tasting board, the pizza's board: `wood`); at most 750 triangles (half the
# budget of a thing under 35 cm), 2,000 for the few 35-40 cm across (the pot, the pan, the boards, trays and
# platters). Turned things are `lathe`s (fluted, lobed or bent where the thing is); crusts, rims, handles,
# straws and piped icing are sweeps; boards, leaves, cutlery and smears are outlines extruded.
#
# Run:  cargo art held [--only mealPot] [--preview --lit]
#       HELD_MEASURE=1 … --preview   prints where each is built and its size, to copy into its `I(...)` line
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
    # the detail pass (2026-10-22)
    carrot_core=('#F2A05A', 'satin'),  # a carrot coin's paler middle
    beef=('#6B3A22', 'satin'),
    broccoli=('#3E7A2C', 'satin'),
    pepper_red=('#C93226', 'gloss'),
    pepper_yellow=('#E9B92E', 'gloss'),
    sesame=('#F1E2B8', 'satin'),
    jam=('#7A2A3A', 'gloss'),          # the tasting board's ramekin of fig jam
    berry=('#8E2446', 'satin'),        # a raspberry on the smoothie
    wafer=('#E2B872', 'matte'),
    napkin=('#F2EFE6', 'matte'),
    cheese_brown=('#C98A3E', 'satin'),  # browned spots on a baked pizza
    straw_stripe=('#D9484A', 'satin'),
    wire=('#9EA4AA', 'metal'),         # the noodle pail's wire handle
    # props in hand (G-70): what residents hold while they use a thing
    mug=('#E8E2D6', 'gloss'),
    mug_band=('#4F7A9A', 'gloss'),
    coffee=('#4A2E1E', 'gloss'),
    crema=('#B88A5A', 'satin'),
    juice=('#F0A23A', 'satin'),        # an orange drink, seen through the glass
    ice=('#E9F1F2', 'gloss'),
    wine=('#6E1A2A', 'gloss'),
    can=('#C8302A', 'gloss'),
    can_band=('#F2F0EA', 'gloss'),
    can_top=('#C9CED4', 'metal'),
    phone=('#2A2C30', 'satin'),
    screen=('#141A22', 'gloss'),
    screen_lit=('#3C5A7A', 'gloss'),
    cover=('#7A2E2A', 'satin'),        # a cloth-bound book's boards
    page=('#F2EEE2', 'matte'),
    page_edge=('#E2DCCB', 'matte'),
    print=('#9A968C', 'matte'),        # the lines of type, a grey band each
    brass=('#C9A54A', 'metal'),
    seed=('#B8964E', 'matte'),
    dart_flight=('#2E6FB8', 'satin'),
    dart_shaft=('#1E1E1E', 'satin'),
    cue=('#D9B98A', 'satin'),
    cue_butt=('#3B2A20', 'satin'),
    cue_wrap=('#141414', 'satin'),
    pearl=('#EDE6D6', 'gloss'),
    cue_tip=('#3E8EDE', 'matte'),      # chalked
    rubber=('#C8322A', 'satin'),       # a ping-pong bat's faces, red over black
    rubber_black=('#1E1E1E', 'satin'),
    blade=('#C9A06A', 'satin'),
    chrome=('#D0D3D6', 'chrome'),
    # the food from town (review M-held-03)
    cup_paper=('#F4F1EA', 'matte'),
    cup_stripe=('#E58FA8', 'matte'),
    paddle=('#D8BE8E', 'matte'),       # a wooden ice cream spoon
    mustard=('#E2B02A', 'satin'),
    ketchup=('#B8281E', 'satin'),
    foil=('#C9CED4', 'metal'),         # a crisp packet's silvered inside
    packet=('#2F5FA8', 'gloss'),
    packet_band=('#F2C230', 'gloss'),
    crisp=('#E8C46A', 'matte'),
    apple=('#B8282A', 'gloss'),
    apple_blush=('#D8A23A', 'gloss'),
    stalk=('#5A3A22', 'matte'),
    bag_white=('#F2EEE4', 'matte'),
)

# Built in the game's axes (x, height, depth; front +depth), as the old parts are.
F = G.GameFrame()
TAU = math.tau


# --- helpers ------------------------------------------------------------------------------------------------
def lathe(name, profile, mats, at=(0, 0, 0), segments=16, scale=(1, 1), rot=(0, 0, 0), sharp=40, wave=None,
          deform=None):
    """A closed surface of revolution about the game's up axis: `profile` [(radius, height)] from the
    bottom's middle (radius 0) round to the top's (radius 0), each end a pole (no doubled points at the
    axis). `mats`: one material, or one per band between profile points. `scale`: (game x, depth) to make
    it oval. Edges sharper than `sharp` degrees are kept sharp (a rim, a lip); the rest is smooth.
    `wave` (count, {profile index: amount}) flutes or lobes those rings, their radius times
    1 + amount * cos(count * angle): a fluted dish, a pleated cake case, a tomato's lobes, an ice cream scoop's
    ragged lip. `deform` moves each point (local x, y, z; z along the axis) after: a sausage bent."""
    mats = mats if isinstance(mats, (list, tuple)) else [mats] * (len(profile) - 1)
    assert len(mats) == len(profile) - 1, f'{name}: {len(mats)} materials for {len(profile) - 1} bands'
    count, amounts = wave if wave else (0, {})
    bm = bmesh.new()
    rings = []
    for k, (r, h) in enumerate(profile):
        if r < 1e-6:
            rings.append([bm.verts.new((0, 0, h))])
        else:
            amp = amounts.get(k, 0.0)
            ring = []
            for i in range(segments):
                a = TAU * i / segments
                rr = r * (1 + amp * math.cos(count * a))
                ring.append(bm.verts.new((rr * math.cos(a) * scale[0], rr * math.sin(a) * scale[1], h)))
            rings.append(ring)
    if deform:
        for v in bm.verts:
            v.co = Vector(deform(v.co.copy()))
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


def lump(name, r, at, material, scale=(1, 1, 1), seed=0.0, subdivisions=1, noise=0.25, rough=0.0):
    """A lumpy bit of food (an icosphere shaken by noise), its base at the game point `at`; `scale` in
    the game's (x, height, depth). `rough` shakes it again in features a third of it across (the boulder's own
    noise is too broad to show on a thing a few centimetres across)."""
    obj = G.boulder(name, r, at=F.at(*at), material=material, scale=(scale[0], scale[2], scale[1]), seed=seed,
                    subdivisions=subdivisions, noise=noise)
    if rough:
        G.jitter(obj, rough * r, size=r * 0.7, seed=seed)
    return obj


def ball(name, r, at, material, scale=(1, 1, 1), seg=8, rings=5):
    """A low sphere centred at the game point `at`; `scale` in the game's (x, height, depth)."""
    return G.sphere(name, r, at=F.at(*at), material=material, scale=(scale[0], scale[2], scale[1]),
                    segments=seg, rings=rings)


def chunk(name, s, at, material, yaw=0.0, tilt=(0, 0)):
    """A chopped cube of food `s` across, centred at the game point `at`."""
    return box(name, s, s * 0.8, s, at, material, yaw=yaw, tilt=tilt)


def dice(name, s, at, material, yaw=0.0, tilt=(0, 0)):
    """A diced piece of vegetable `s` across, its edges softened, centred at the game point `at`."""
    return box(name, s, s * 0.85, s, at, material, yaw=yaw, tilt=tilt, bevel=s * 0.18)


def roll(name, r, length, mats, at, yaw=0.0, segments=8, marks=(), bend=0.0):
    """A sausage (a capsule `r` thick, `length` long) lying along the game's x axis turned `yaw` degrees,
    centred at the game point `at`. `marks`: the (from, to) stretches along it, from -1 to 1, that take
    the second material (grill marks). `bend`: how far its middle bows sideways (towards -depth at yaw 0),
    its ends staying where they were."""
    body, mark = mats if isinstance(mats, (list, tuple)) else (mats, mats)
    half = length / 2
    cap = r * 0.75
    stops = sorted({-1.0, 1.0, *[t for m in marks for t in m]})
    profile = [(0, -half), (r * 0.7, -half + r * 0.3)] + [(r, t * (half - cap)) for t in stops] + \
              [(r * 0.7, half - r * 0.3), (0, half)]
    bands = [body, body] + [mark if any(m[0] <= a and b <= m[1] for m in marks) else body
                            for a, b in zip(stops, stops[1:])] + [body, body]

    def bow(p):
        return (p.x, p.y + bend * (1 - (p.z / half) ** 2), p.z)
    return lathe(name, profile, bands, at=at, segments=segments, rot=(0, 90, -yaw), deform=bow if bend else None)


def coin(name, d, h, at, mats, segments=8, cup=0.35, tilt=(0, 0)):
    """A thin round slice `d` across and `h` thick at its edge, the edge curled up a little (`cup`): pepperoni,
    salami, a carrot coin; its base at the game point `at`. `mats`: one material or (edge, face)."""
    edge, face = mats if isinstance(mats, (list, tuple)) else (mats, mats)
    r = d / 2
    return lathe(name, [(0, 0), (r, 0), (r * 0.96, h * (1 + cup)), (0, h)], [edge, edge, face], at=at,
                 segments=segments, rot=(tilt[0], tilt[1], 0))


def leaf(name, at, length, width, yaw, material, tilt=0.0, t=0.0012, n=4):
    """A flat leaf (basil, parsley, mint, a scrap of greens) `length` long and `width` wide, its middle at the
    game point `at` and its underside there, along game x turned `yaw` degrees, tipped `tilt` about its length."""
    side = [(length * (i / n - 0.5), width / 2 * math.sin(math.pi * i / n) ** 0.8) for i in range(n + 1)]
    outline = side + [(u, -v) for u, v in reversed(side[1:-1])]
    return F.extrude(name, outline, t, (at[0], at[1] + t / 2, at[2]), material, plane='top', rot=(tilt, 0, yaw))


def blob(name, at, rx, rz, h, material, seed=0.0, yaw=0.0, n=8):
    """A thin, irregular smear or puddle `h` thick and about `rx` by `rz`, lying on the game point `at`."""
    pts = []
    for i in range(n):
        a = TAU * i / n
        k = 1 + 0.16 * math.sin(3 * a + seed) + 0.09 * math.sin(5 * a + 2.3 * seed)
        pts.append((rx * k * math.cos(a), rz * k * math.sin(a)))
    return F.extrude(name, G.smooth_outline(pts, passes=1), h, (at[0], at[1] + h / 2, at[2]), material,
                     plane='top', rot=(0, 0, yaw))


def arc(R, A, apex, steps):
    """The (x, depth) points of a wedge's round edge (a slice of something round, its tip at `apex`, pointing
    to the front): `steps` + 1 points from one side to the other."""
    return [(apex[0] + R * math.sin(math.radians(-A + 2 * A * k / steps)),
             apex[1] - R * math.cos(math.radians(-A + 2 * A * k / steps))) for k in range(steps + 1)]


def crust(name, pts, y, r, material, flat=0.8, crimp=0, bulge=0.16, taper=0.65, verts=6, closed=False):
    """A crust or a pastry edge `r` round (squashed to `flat` of that in height) swept along game (x, depth) `pts`
    at height `y` (its middle): its ends tapering unless `closed`, crimped `crimp` times along (a wave in its
    thickness)."""
    prof = [(r * flat * math.cos(TAU * i / verts), r * math.sin(TAU * i / verts)) for i in range(verts)]

    def scale(f):
        s = 1 + bulge * math.cos(TAU * crimp * f) if crimp else 1.0
        if not closed:
            s *= taper + (1 - taper) * min(1.0, min(f, 1 - f) * 6) ** 0.5
        return s
    return F.sweep(name, [(x, y, z) for x, z in pts], None, material, profile=prof, scale=scale, closed=closed,
                   angle=75)


def swirl(name, at, r0, h, material, tube=0.008, turns=2.2, steps=14, star=8):
    """Piped icing or cream: a tube `tube` round with `star` ridges (0: smooth, 6 sides) coiling up from radius
    `r0` round the game point `at` (its base) to a peak `h` above it, thinning as it goes."""
    pts = []
    for i in range(steps + 1):
        f = i / steps
        a = TAU * turns * f
        rr = r0 * (1 - f) ** 0.85
        pts.append((at[0] + rr * math.cos(a), at[1] + tube * 0.8 + (h - tube * 0.8) * f ** 0.9, at[2] + rr * math.sin(a)))
    if star:
        prof = [(math.cos(TAU * i / star) * (1 if i % 2 == 0 else 0.78), math.sin(TAU * i / star) * (1 if i % 2 == 0 else 0.78))
                for i in range(star)]
    else:
        prof = G.circle(1.0, 6)
    return F.sweep(name, pts, None, material, profile=prof, scale=lambda f: tube * (1 - 0.8 * f ** 1.6), angle=80)


def aim(d):
    """The Blender rotation (degrees) that turns Blender +x along the game direction `d`, rolling it no more than
    that takes: the long axis of a spoon's bowl laid along its handle."""
    bx, by, bz = -d[0], -d[2], d[1]
    n = math.sqrt(bx * bx + by * by + bz * bz)
    return (0.0, -math.degrees(math.asin(bz / n)), math.degrees(math.atan2(by, bx)))


def rrect(hw, hd, n=16, square=8.0):
    """A rounded rectangle's (x, depth) points, `hw` by `hd` from its middle (G.ring's superellipse)."""
    e = 2.0 / square
    out = []
    for i in range(n):
        a = TAU * i / n
        c, s = math.cos(a), math.sin(a)
        out.append((hw * math.copysign(abs(c) ** e, c), hd * math.copysign(abs(s) ** e, s)))
    return out


def loft(name, sections, material, angle=40):
    """A body lofted through `sections`, each [(x, height, depth)] in game points with the same count, bottom
    first (G.loft): a paper bag, a tray turned up at its edges, a box's lid."""
    return G.loft(name, [[F.at(*p) for p in s] for s in sections], material=material, angle=angle)


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
    """A dinner plate 29 cm across: a flat foot, a rim rising to 1.7 cm with a rolled edge, a well at 0.9 cm in a
    paler glaze."""
    lathe('plate', [(0, 0), (0.094, 0), (0.1438, 0.0156), (0.1408, 0.0172), (0.124, 0.0162), (0.088, WELL),
                    (0, WELL)],
          [M.china, M.china, M.china, M.china, M.china, M.china_well], segments=22)


def _spoon(at, yaw, tilt, name='spoon', reach=0.11):
    """A steel dessert spoon, its bowl at the game point `at`, the handle out along `yaw` (degrees from +x)
    rising at `tilt` degrees: a flat handle widening to its end, an oval bowl along it."""
    x, y, z = at
    a, t = math.radians(yaw), math.radians(tilt)
    d = (math.cos(a) * math.cos(t), math.sin(t), -math.sin(a) * math.cos(t))

    def along(k):
        return (x + d[0] * k, y + d[1] * k, z + d[2] * k)
    F.sweep(f'{name}_handle', [along(0.012), along(0.05), along(reach)], None, M.steel,
            profile=G.rounded_rect(0.0024, 0.0075, 0.001, corner=1), scale=[0.55, 0.8, 1.25])
    G.sphere(f'{name}_bowl', 1.0, at=F.at(x, y, z), material=M.steel, scale=(0.017, 0.012, 0.0042),
             rot=aim(d), segments=8, rings=5)


# --- items: carried and set down ------------------------------------------------------------------------------
def _bag_ring(y, hw, hd, pinch):
    """A ring of the paper bag at height `y`: its front and back flat, each side folded in by `pinch` at its
    middle (the gussets)."""
    pts = [(-hw, hd), (-hw / 3, hd), (hw / 3, hd), (hw, hd), (hw - pinch, 0), (hw, -hd), (hw / 3, -hd),
           (-hw / 3, -hd), (-hw, -hd), (-hw + pinch, 0)]
    return [(px, y, pz) for px, pz in reversed(pts)]


def ingredients():
    """A kraft paper bag of shopping, its sides folded in at the gussets and its top turned over in a cuff: a
    baguette, a lettuce, a carrot with its feathery tops and a tomato with its green calyx."""
    bag = loft('bag', [_bag_ring(y, 0.094, 0.064, p) for y, p in ((0, 0.006), (0.06, 0.016), (0.13, 0.018),
                                                                  (0.19, 0.014))], M.kraft, angle=25)
    G.jitter(bag, 0.0014, size=0.05, seed=2.0, up=0.0)
    F.extrude('cuff', G.rounded_rect(0.2, 0.14, 0.004, corner=1), 0.024, (0, 0.191, 0), M.kraft_fold, plane='top',
              holes=[G.rounded_rect(0.192, 0.132, 0.003, corner=1)])
    box('inside', 0.19, 0.002, 0.13, (0, 0.1905, 0), M.kraft_inside)
    # the baguette, leaning at the back, three slashes across its crust
    b0, b1 = Vector((0.052, 0.09, -0.03)), Vector((0.074, 0.305, -0.044))
    dirn = (b1 - b0).normalized()
    F.sweep('baguette', [tuple(b0 + (b1 - b0) * k) for k in (0, 0.1, 0.5, 0.9, 1)], 0.021, M.bun, verts=8,
            scale=[0.5, 0.92, 1.0, 0.92, 0.45])
    front = Vector((0, 0, 1)) - dirn * dirn.z
    front.normalize()
    side = dirn.cross(front)
    for i, k in enumerate((0.55, 0.72, 0.88)):
        c = b0 + (b1 - b0) * k
        pts = [tuple(c + dirn * (0.012 * s) + (front * math.cos(0.55 * s) + side * math.sin(0.55 * s)) * 0.0212)
               for s in (-1, 0, 1)]
        up = (-front.x, -front.z, front.y)
        F.sweep(f'slash{i}', pts, None, M.bun_base, profile=[(-0.0008, -0.003), (0.0008, -0.003), (0.0008, 0.003),
                                                             (-0.0008, 0.003)], up=up, scale=[0.5, 1, 0.5])
    lettuce = lump('lettuce', 0.054, (0.012, 0.15, 0.0), M.lettuce, scale=(1.0, 0.95, 0.9), seed=1.7, subdivisions=2,
                   noise=0.18)
    G.jitter(lettuce, 0.004, size=0.018, seed=1.7)
    # the carrot, out of the bag at the front left, tapering to its root, a rounded shoulder at its top
    c0, c1 = (-0.0813, 0.134, 0.02), (-0.0287, 0.296, 0.02)
    path = [tuple(c0[i] + (c1[i] - c0[i]) * k for i in range(3)) for k in (0, 0.2, 0.45, 0.7, 0.9, 0.97, 1.0)]
    carrot = F.sweep('carrot', path, 1.0, M.carrot, verts=7, scale=[0.002, 0.007, 0.011, 0.0145, 0.0165, 0.0155, 0.009])
    G.jitter(carrot, 0.0008, size=0.01, seed=4.0)
    tx, ty = c1[0], c1[1]
    for i, (dx, dz, top) in enumerate(((-0.022, 0.006, 0.334), (0.004, -0.012, 0.336), (0.02, 0.01, 0.328))):
        F.sweep(f'carrot_top{i}', [(tx, ty - 0.004, 0.02), (tx + dx * 0.3, ty + 0.012, 0.02 + dz * 0.3),
                                  (tx + dx * 0.75, ty + 0.03, 0.02 + dz * 0.75), (tx + dx, top, 0.02 + dz)], None, M.leaf,
                profile=[(-0.0012, -0.004), (0.0012, -0.004), (0.0012, 0.004), (-0.0012, 0.004)],
                scale=[0.6, 0.8, 1.2, 0.5])
    # the tomato: lobed, a dimple at its top, a star of a calyx and a stalk
    lathe('tomato', [(0, 0.004), (0.02, 0.0), (0.033, 0.013), (0.035, 0.03), (0.027, 0.052), (0.009, 0.06),
                     (0, 0.056)], M.tomato, at=(0.06, 0.192, 0.035), segments=12,
          wave=(6, {2: 0.04, 3: 0.05, 4: 0.05}), sharp=80)
    star = [((0.014 if i % 2 == 0 else 0.005) * math.cos(TAU * i / 10), (0.014 if i % 2 == 0 else 0.005) * math.sin(TAU * i / 10))
            for i in range(10)]
    F.extrude('calyx', star, 0.0015, (0.06, 0.2495, 0.035), M.leaf, plane='top', rot=(0, 0, 10))
    disc('tomato_stalk', 0.004, 0.009, (0.06, 0.25, 0.035), M.leaf, verts=5)
    return done('ingredients')


def prepBowl():
    """A wide glazed bowl of chopped vegetables, its rim rolled: diced potato heaped with carrot, green beans and
    peas over it."""
    lathe('bowl', [(0, 0), (0.095, 0), (0.12, 0.012), (0.1605, 0.087), (0.162, 0.0915), (0.157, 0.093),
                   (0.152, 0.0905), (0.146, 0.078), (0.09, 0.083), (0, 0.085)],
          [M.bowl] * 7 + [M.potato, M.potato], segments=20)
    for i, (x, z, m, yaw) in enumerate(((0.03, 0.02, M.carrot, 10), (-0.05, -0.02, M.carrot, 40),
                                        (0.06, -0.06, M.carrot, 70), (-0.02, 0.07, M.carrot, 25),
                                        (-0.08, -0.06, M.potato, 30), (0.01, 0.0, M.potato, 5),
                                        (0.09, 0.03, M.potato, 50), (-0.06, 0.05, M.potato, 65),
                                        (0.04, -0.09, M.carrot, 20), (-0.1, 0.0, M.carrot, 80))):
        y = 0.093 - 0.00004 * (abs(x) + abs(z)) * 100
        box(f'dice{i}', 0.02, 0.017, 0.02, (x, y, z), m, yaw=yaw, tilt=(12 * (i % 3 - 1), 9 * (i % 2)))
    for i, (x, z, yaw) in enumerate(((-0.04, 0.025, 15), (0.02, -0.045, 75), (0.075, 0.07, 120))):
        a = math.radians(yaw)
        F.sweep(f'bean{i}', [(x - 0.016 * math.cos(a), 0.094, z + 0.016 * math.sin(a)),
                             (x + 0.016 * math.cos(a), 0.094, z - 0.016 * math.sin(a))], 0.0045, M.pea, verts=5)
    for i, (x, z) in enumerate(((0.0, 0.04), (-0.03, -0.05), (0.05, 0.0), (0.07, -0.03), (-0.07, 0.03), (0.03, 0.07))):
        ball(f'pea{i}', 0.0055, (x, 0.0945, z), M.pea, seg=6, rings=3)
    return done('prepBowl')


def _pot(dirty):
    """The stew pot: dark iron, a rolled rim and a loop handle each side, riveted on; full of stew (beef, carrot
    coins, potato and peas in gravy, parsley), or scraped empty with a tidemark and burnt bits."""
    body = [(0, 0), (0.126, 0), (0.132, 0.006), (0.132, 0.141), (0.1348, 0.144), (0.134, 0.1495),
            (0.1295, 0.1495), (0.127, 0.145)]
    if dirty:
        lathe('pot', body + [(0.127, 0.125), (0.127, 0.113), (0.127, 0.03), (0, 0.034)],
              [M.iron] * 6 + [M.iron_inside, M.iron_inside, M.residue, M.iron_inside, M.residue], segments=24)
        for i, (x, z, r, sx, m) in enumerate(((0.04, 0.03, 0.022, 1.2, M.burnt), (-0.05, -0.02, 0.016, 1.3, M.residue),
                                              (-0.01, 0.07, 0.012, 1.4, M.burnt), (0.07, -0.06, 0.01, 1.2, M.residue))):
            lump(f'burnt{i}', r, (x, 0.031, z), m, scale=(sx, 0.35, 1), seed=2.0 * i + 2, rough=0.15)
        blob('film', (0.0, 0.0335, 0.0), 0.085, 0.075, 0.0012, M.residue, seed=1.1, n=9)
    else:
        lathe('pot', body + [(0.127, 0.135), (0.07, 0.1385), (0, 0.139)],
              [M.iron] * 6 + [M.iron_inside, M.iron_inside, M.stew, M.stew], segments=24)
        for i, (x, z) in enumerate(((0.045, 0.03), (-0.05, -0.025), (0.0, -0.065), (-0.03, 0.06), (0.075, -0.035))):
            dice(f'beef{i}', 0.021, (x, 0.1405, z), M.beef, yaw=25 * i, tilt=(14 * (i % 3 - 1), 10))
        for i, (x, z, t) in enumerate(((0.01, 0.0, 8), (-0.07, 0.01, -10), (0.06, 0.06, 12), (0.03, -0.075, -6))):
            coin(f'carrot{i}', 0.024, 0.005, (x, 0.1395, z), (M.carrot, M.carrot_core), tilt=(t, t * 0.6))
        for i, (x, z, yaw) in enumerate(((-0.02, -0.04, 20), (0.08, 0.02, 60), (-0.075, -0.06, 35))):
            dice(f'potato{i}', 0.019, (x, 0.1445, z), M.potato, yaw=yaw, tilt=(10, 6 * i))
        for i, (x, z) in enumerate(((-0.045, 0.03), (0.035, 0.07), (0.09, -0.07), (-0.095, -0.02))):
            ball(f'pea{i}', 0.006, (x, 0.1405, z), M.pea, seg=6, rings=3)
        for i, (x, z, yaw) in enumerate(((0.0, 0.035, 30), (0.012, 0.045, 100))):
            leaf(f'parsley{i}', (x, 0.1435, z), 0.018, 0.011, yaw, M.leaf, tilt=8)
    # the handles: a loop of iron each side, its ends riveted to the wall under the rim
    for s in (-1, 1):
        y = 0.124
        G.kit.tube(F, f'handle{s}', G.kit.fillet([(s * 0.129, y, -0.034), (s * 0.189, y, -0.03), (s * 0.189, y, 0.03),
                                                 (s * 0.129, y, 0.034)], 0.014), 0.0058, M.iron, verts=6, round=1)
        for t in (-1, 1):
            G.cylinder(f'rivet{s}{t}', 0.0045, 0.004, at=F.at(s * 0.133, y, t * 0.034), material=M.iron,
                       rot=(0, 90, 0), verts=6)


def mealPot():
    _pot(False)
    return done('mealPot')


def potDirty():
    _pot(True)
    return done('potDirty')


def _paddle(name, material, y=0.0, t=0.028):
    """A serving board 35 by 24 cm, its corners rounded, with a rounded tab handle at its right end and a hole to
    hang it by: one outline, extruded `t` thick from height `y`."""
    def quarter(cx, cz, r, a0, n=3):
        return [(cx + r * math.cos(math.radians(a0 + 90 * i / n)), cz + r * math.sin(math.radians(a0 + 90 * i / n)))
                for i in range(n + 1)]
    r = 0.015
    pts = quarter(-0.185, -0.105, r, 180) + quarter(0.135, -0.105, r, 270)
    pts += [(0.15, -0.04), (0.16, -0.04)]
    pts += [(0.16 + 0.04 * math.cos(math.radians(a)), 0.04 * math.sin(math.radians(a))) for a in (-60, -30, 0, 30, 60)]
    pts += [(0.16, 0.04), (0.15, 0.04)] + quarter(0.135, 0.105, r, 0) + quarter(-0.185, 0.105, r, 90)
    hole = [(0.173 + 0.0095 * math.cos(TAU * i / 8), 0.0095 * math.sin(TAU * i / 8)) for i in range(8)]
    return F.extrude(name, pts, t, (0, y + t / 2, 0), material, plane='top', holes=[hole], bevel=0.0035,
                     bevel_segments=1)


def tastingBoard():
    """A serving board with a tab handle: a wedge of cheese and cubes cut from it, a fan of salami, crackers, a
    ramekin of fig jam and a bunch of grapes on its stalk."""
    _paddle('board', M.board_dark)
    prism('cheese', [(-0.17, -0.07), (-0.07, -0.035), (-0.165, 0.055)], 0.028, 0.07, M.cheese_cut, M.cheese_cut,
          sides=[M.cheese_cut, M.cheese, M.cheese_cut])
    for i, (x, z, yaw) in enumerate(((-0.075, 0.0, 20), (-0.095, 0.03, 55))):
        dice(f'cube{i}', 0.018, (x, 0.0368, z), M.cheese_cut, yaw=yaw)
    for i in range(6):
        x, z = -0.045 + 0.016 * i, -0.06 + 0.012 * i
        coin(f'salami{i}', 0.048, 0.004, (x, 0.028 + 0.0045 * (i % 2), z), M.salami, segments=10, cup=0.25,
             tilt=(10 + 4 * (i % 2), -14))
    for i, (x, z, yaw) in enumerate(((-0.125, 0.075, 10), (-0.095, 0.08, 25), (-0.065, 0.083, 40))):
        box(f'cracker{i}', 0.042, 0.005, 0.042, (x, 0.031 + 0.005 * i, z), M.cracker, yaw=yaw, bevel=0.0012)
    lathe('ramekin', [(0, 0), (0.026, 0), (0.0285, 0.004), (0.0285, 0.033), (0.026, 0.034), (0.024, 0.028),
                      (0, 0.0295)], [M.dish] * 5 + [M.jam], at=(0.03, 0.028, 0.065), segments=14)
    for i, (x, y, z) in enumerate(((0.09, 0.042, -0.02), (0.115, 0.042, -0.035), (0.11, 0.042, 0.0),
                                   (0.135, 0.042, -0.015), (0.105, 0.06, -0.017), (0.07, 0.042, -0.045),
                                   (0.095, 0.042, -0.06), (0.125, 0.059, -0.04), (0.085, 0.058, -0.04))):
        ball(f'grape{i}', 0.015, (x, y, z), M.grape, scale=(1, 1, 0.92), seg=7, rings=5)
    G.kit.tube(F, 'stalk', [(0.15, 0.064, -0.002), (0.13, 0.073, -0.022), (0.1, 0.073, -0.04), (0.075, 0.064, -0.055)],
               0.0018, M.leaf, verts=4, round=2)
    return done('tastingBoard')


def _platter():
    """An oval serving platter, 40 by 28 cm, its rim rolled."""
    lathe('platter', [(0, 0), (0.15, 0), (0.198, 0.0162), (0.2, 0.0186), (0.196, 0.02), (0.188, 0.0192),
                      (0.15, 0.01), (0, 0.01)],
          [M.china] * 6 + [M.china_well], segments=28, scale=(1, 0.7))


def _patty(name, at, d=0.1, h=0.028, marks=True):
    """A burger off the grill: a thick round patty, its edge rounded and rough, three bar marks across its top."""
    x, y, z = at
    r = d / 2
    p = lathe(name, [(0, 0), (r * 0.94, 0.0), (r, h * 0.35), (r * 0.95, h * 0.85), (r * 0.7, h), (0, h * 1.02)],
              M.patty, at=at, segments=14)
    G.jitter(p, 0.0012, size=0.012, seed=x * 31, up=0.0)
    for i, dx in enumerate((-0.02, 0.0, 0.02) if marks else ()):
        box(f'{name}_mark{i}', 0.0065, 0.0016, d * 0.8 - abs(dx) * 1.2, (x + dx, y + h + 0.0002, z), M.char, yaw=20)


def _corn(name, r, length, at, yaw, kernels=True, gnawed=False):
    """A cob of corn lying along game x turned `yaw` degrees: rows of kernels (ridged along and round it), its
    rounded ends; `gnawed`: eaten to the cob in the middle, kernels left at its ends."""
    half = length / 2
    stops = [-1, -0.8, -0.6, -0.4, -0.2, 0, 0.2, 0.4, 0.6, 0.8, 1]
    prof = [(0, -half - r * 0.2), (r * 0.7, -half)]
    for i, t in enumerate(stops):
        bump = (1.0 if i % 2 == 0 else 0.88)
        if gnawed:
            bump = 1.0 if abs(t) > 0.55 else 0.6
        prof.append((r * bump, t * (half - r * 0.3)))
    prof += [(r * 0.7, half), (0, half + r * 0.2)]
    mats = [M.corn if kernels else M.cob] * (len(prof) - 1)
    if gnawed:
        mats = [M.corn if abs(stops[min(max(i - 1, 0), len(stops) - 1)]) > 0.55 else M.cob for i in range(len(prof) - 1)]
    wave = (7, {k: 0.1 for k in range(2, len(prof) - 2)}) if kernels else None
    return lathe(name, prof, mats, at=at, segments=14, rot=(0, 90, -yaw), wave=wave)


def grillPlatter():
    """The platter from the grill: two sausages, bent and marked by the bars, a marked burger, a cob of corn with
    a pat of butter melting on it, and a sprig of parsley."""
    _platter()
    for i, (z, yaw, m, b) in enumerate(((-0.04, 12, M.sausage, -0.008), (0.035, -8, M.sausage_b, 0.008))):
        roll(f'sausage{i}', 0.025, 0.16, (m, M.char), (-0.08, 0.01 + 0.025, z), yaw=yaw, segments=10,
             marks=((-0.55, -0.4), (-0.05, 0.1), (0.45, 0.6)), bend=b)
    _patty('patty', (0.075, 0.01, 0.035))
    _corn('corn', 0.022, 0.13, (0.075, 0.01 + 0.022, -0.075), yaw=-12)
    box('butter', 0.016, 0.006, 0.014, (0.07, 0.054, -0.075), M.butter, yaw=20, bevel=0.002)
    for i, (x, z, yaw) in enumerate(((0.145, 0.0, 20), (0.155, -0.02, 80))):
        leaf(f'parsley{i}', (x, 0.0105, z), 0.022, 0.014, yaw, M.leaf, tilt=6)
    return done('grillPlatter')


def platterDirty():
    """The platter after the barbecue: a smear of grease and a streak of sauce, crumbs, a gnawed cob and a paper
    napkin screwed up on its rim."""
    _platter()
    blob('smear', (0.02, 0.01, 0), 0.12, 0.075, 0.0016, M.smear, seed=0.7, yaw=8, n=10)
    blob('sauce', (-0.06, 0.0112, 0.03), 0.045, 0.02, 0.0016, M.smear_sauce, seed=2.1, yaw=-20)
    _corn('cob', 0.012, 0.11, (0.06, 0.01 + 0.012, -0.04), yaw=20, gnawed=True)
    for i, (x, z) in enumerate(((-0.02, -0.05), (0.09, 0.04), (-0.1, -0.02), (0.02, 0.06), (-0.04, 0.07))):
        chunk(f'crumb{i}', 0.008, (x, 0.0135, z), M.crumb, yaw=30 * i)
    lump('napkin', 0.028, (-0.12, 0.009, -0.05), M.napkin, scale=(1.25, 0.42, 1.0), seed=3.3, subdivisions=2,
         noise=0.3, rough=0.22)
    return done('platterDirty')


def pizza():
    """A whole pizza on its round board: a puffed, golden crust, tomato showing inside it, melted cheese browned
    in spots, pepperoni curled at their edges, basil, and the cuts between eight slices."""
    lathe('board', [(0, 0), (0.197, 0), (0.2, 0.004), (0.2, 0.016), (0.197, 0.02), (0, 0.02)], M.peel, segments=28)
    top = lathe('pizza', [(0, 0.02), (0.166, 0.02), (0.166, 0.028), (0.147, 0.0345), (0.138, 0.0375), (0.07, 0.0395),
                          (0, 0.04)],
                [M.dough, M.dough, M.sauce, M.sauce, M.mozzarella, M.mozzarella], segments=28)
    G.jitter(top, 0.0008, size=0.03, seed=0.4, up=0.6)
    ring = [(0.156 * math.cos(TAU * i / 24), 0.156 * math.sin(TAU * i / 24)) for i in range(24)]
    c = crust('crust', ring, 0.0405, 0.0115, M.dough_baked, flat=0.95, crimp=5, bulge=0.06, verts=6, closed=True)
    G.jitter(c, 0.0012, size=0.02, seed=1.9, up=0.4)
    for i, (x, z) in enumerate(((0.06, 0.05), (-0.07, 0.03), (0.0, -0.085), (-0.04, -0.04), (0.08, -0.04),
                                (-0.02, 0.095), (0.1, 0.03))):
        coin(f'pepperoni{i}', 0.042, 0.003, (x, 0.039, z), M.pepperoni, segments=10)
    for i, (x, z, yaw) in enumerate(((0.02, 0.0, 20), (-0.09, -0.06, 70), (0.1, -0.08, 40), (0.03, -0.1, 10),
                                     (-0.06, 0.09, 60))):
        leaf(f'basil{i}', (x, 0.0415, z), 0.026, 0.016, yaw, M.leaf, tilt=5)
    for i, (x, z, s) in enumerate(((-0.045, 0.065, 0.012), (0.045, -0.02, 0.01), (-0.1, -0.02, 0.009),
                                   (0.03, 0.09, 0.008))):
        blob(f'brown{i}', (x, 0.0398, z), s, s * 0.7, 0.0006, M.cheese_brown, seed=i, yaw=40 * i, n=6)
    for i in range(4):
        box(f'cut{i}', 0.29, 0.0008, 0.0016, (0, 0.0405, 0), M.sauce, yaw=45 * i)
    return done('pizza')


def _tray():
    """A steel baking tray 38 by 28 cm: its floor and sides pressed in one, the edge rolled round a wire."""
    def ring(y, hw, hd):
        return [(x, y, z) for x, z in rrect(hw, hd, n=24, square=12)]
    loft('tray', [ring(0, 0.18, 0.13), ring(0.022, 0.1865, 0.1365), ring(0.022, 0.1835, 0.1335),
                  ring(0.006, 0.177, 0.127)], M.tray, angle=30)
    rim = G.rounded_rect(0.374, 0.274, 0.012, corner=3)
    F.sweep('rim', [(x, 0.022, z) for x, z in rim], 0.003, M.tray, verts=6, closed=True)


def bakingTray():
    """A tray of six bread rolls, golden on top, paler round their feet, each slashed across its top."""
    _tray()
    for i, (x, z) in enumerate(((-0.11, -0.07), (0, -0.07), (0.11, -0.07), (-0.11, 0.07), (0, 0.07), (0.11, 0.07))):
        top = M.bun if i % 2 else M.bun_dark
        r = lathe(f'roll{i}', [(0, 0), (0.038, 0), (0.0395, 0.008), (0.036, 0.02), (0.026, 0.033), (0.012, 0.039),
                               (0, 0.04)],
                  [M.bun_base, M.bun_base, M.bun_base, top, top, top], at=(x, 0.006, z), segments=14, scale=(1, 0.92))
        G.jitter(r, 0.001, size=0.02, seed=i * 1.7)
        a = math.radians(25 + 40 * (i % 3))
        ca, sa = math.cos(a), math.sin(a)
        F.sweep(f'slash{i}', [(x - 0.016 * ca, 0.006 + 0.0372, z + 0.016 * sa), (x, 0.006 + 0.0403, z),
                              (x + 0.016 * ca, 0.006 + 0.0372, z - 0.016 * sa)], None, M.bun_base,
                profile=[(-0.0006, -0.0025), (0.0006, -0.0025), (0.0006, 0.0025), (-0.0006, 0.0025)],
                scale=[0.4, 1, 0.4], resolution=2)
    return done('bakingTray')


def trayDirty():
    """The tray after baking: a greasy stain with burnt patches, crumbs and a burnt crumb or two."""
    _tray()
    blob('grease', (0.0, 0.006, 0.0), 0.11, 0.075, 0.0012, M.grease, seed=0.3, yaw=5, n=10)
    for i, (x, z, d) in enumerate(((-0.07, -0.025, 0.02), (0.06, 0.03, 0.016), (0.02, -0.045, 0.013),
                                   (-0.11, 0.07, 0.012))):
        blob(f'burnt{i}', (x, 0.0072, z), d, d * 0.7, 0.0012, M.burnt, seed=i * 1.3, yaw=30 * i, n=6)
    for i, (x, z) in enumerate(((-0.1, 0.06), (0.11, -0.05), (0.04, 0.08), (-0.03, -0.09), (0.13, 0.09), (-0.14, -0.08))):
        chunk(f'crumb{i}', 0.008, (x, 0.0095, z), M.crumb, yaw=25 * i)
    lump('bit', 0.007, (0.08, 0.0065, -0.06), M.burnt, scale=(1.3, 0.6, 1), seed=5.0)
    return done('trayDirty')


def _pie_dish(inside_mats, inside):
    """The glazed pie dish, 33.6 cm across, its outside fluted: `inside` the profile from its rim's inner edge."""
    lathe('dish', [(0, 0), (0.135, 0), (0.166, 0.057), (0.168, 0.061), (0.158, 0.063)] + inside,
          [M.dish, M.dish, M.dish, M.dish] + inside_mats, segments=24, wave=(12, {1: 0.012, 2: 0.01}))


def pieDish():
    """A cherry pie in a fluted glazed dish: a crimped rim of pastry and a lattice woven over the red filling."""
    _pie_dish([M.pastry, M.filling], [(0.148, 0.072), (0, 0.075)])
    c = crust('rim', [(0.147 * math.cos(TAU * i / 28), 0.147 * math.sin(TAU * i / 28)) for i in range(28)], 0.0725,
              0.0105, M.pastry_baked, flat=0.95, crimp=14, bulge=0.12, verts=5, closed=True)
    G.jitter(c, 0.0006, size=0.02, seed=0.8)
    prof = [(-0.002, -0.011), (0.002, -0.011), (0.002, 0.011), (-0.002, 0.011)]
    for i, o in enumerate((-0.07, 0, 0.07)):
        h = math.sqrt(0.146 ** 2 - o ** 2)
        lo, hi = 0.0775, 0.0805
        for axis in ('x', 'z'):
            ys = [(lo if (j + i + (axis == 'z')) % 2 else hi) for j in range(3)]
            pts = [(-h, 0.0765), (-0.07, ys[0]), (0, ys[1]), (0.07, ys[2]), (h, 0.0765)]
            pts = [(u, y, o) if axis == 'x' else (o, y, u) for u, y in pts]
            F.sweep(f'strip_{axis}{i}', pts, None, M.pastry, profile=prof, up=(0, 0, 1))
    return done('pieDish')


def mealPlate():
    """A plate of stew: chunks of beef and carrot coins in gravy, a dollop of mashed potato swirled to a peak,
    and green beans."""
    plate()
    blob('gravy', (0.025, WELL, 0.0), 0.07, 0.058, 0.003, M.stew, seed=0.9, yaw=10)
    for i, (x, z) in enumerate(((0.035, 0.02), (0.0, -0.02), (0.06, -0.025), (0.01, 0.04))):
        chunk(f'beef{i}', 0.018, (x, WELL + 0.0095, z), M.beef, yaw=30 * i, tilt=(12 * (i % 3 - 1), 9))
    for i, (x, z, t) in enumerate(((0.06, 0.03, 10), (0.03, -0.045, -12), (-0.005, 0.012, 6))):
        coin(f'carrot{i}', 0.022, 0.005, (x, WELL + 0.003, z), (M.carrot, M.carrot_core), tilt=(t, -t * 0.5))
    chunk('potato', 0.016, (0.075, WELL + 0.008, -0.002), M.potato, yaw=30, tilt=(10, 5))
    mash = lathe('mash', [(0, 0), (0.036, 0), (0.037, 0.01), (0.031, 0.028), (0.018, 0.044), (0.006, 0.051),
                          (0, 0.052)], M.potato, at=(-0.055, WELL, -0.04), segments=14,
                 wave=(7, {2: 0.05, 3: 0.08, 4: 0.1, 5: 0.1}), rot=(0, 0, 15))
    G.jitter(mash, 0.0012, size=0.012, seed=5.3)
    for i, (x, z, yaw, b) in enumerate(((-0.05, 0.05, 15, 0.004), (-0.04, 0.066, 5, -0.003), (-0.066, 0.04, 30, 0.003),
                                        (-0.03, 0.04, 40, -0.004))):
        a = math.radians(yaw)
        ca, sa = math.cos(a), math.sin(a)
        y = WELL + 0.0045 + 0.004 * (i % 2)
        G.kit.tube(F, f'bean{i}', [(x - 0.036 * ca, y, z + 0.036 * sa), (x + b * sa, y + 0.002, z + b * ca),
                                   (x + 0.036 * ca, y, z - 0.036 * sa)], 0.0048, M.pea, verts=5, round=1)
    leaf('parsley', (0.03, WELL + 0.016, 0.0), 0.016, 0.01, 40, M.leaf, tilt=10)
    return done('mealPlate')


def _shrunk(pts, by):
    """`pts` drawn in towards their middle by a share."""
    cx = sum(p[0] for p in pts) / len(pts)
    cz = sum(p[1] for p in pts) / len(pts)
    return [(cx + (x - cx) * (1 - by), cz + (z - cz) * (1 - by)) for x, z in pts]


def pizzaSlice():
    """A plate with a slice of pizza: the base, tomato showing at its cut sides, cheese browned in a spot,
    pepperoni and basil, and its puffed crust along the back."""
    plate()
    apex, R, A = (0.0, 0.075), 0.15, 21
    wedge = [apex] + arc(R, A, apex, 3)
    prism('base', wedge, WELL, WELL + 0.006, M.dough, M.dough)
    prism('topping', _shrunk(wedge, 0.04), WELL + 0.006, WELL + 0.011, M.mozzarella, M.sauce)
    c = crust('crust', arc(R - 0.009, A + 0.5, apex, 8), WELL + 0.012, 0.012, M.dough_baked, flat=0.83, taper=0.8)
    G.jitter(c, 0.0008, size=0.015, seed=2.4, up=0.3)
    for i, (x, z) in enumerate(((0.0, 0.0), (-0.025, -0.045), (0.028, -0.04))):
        coin(f'pepperoni{i}', 0.03, 0.0025, (x, WELL + 0.0108, z), M.pepperoni, segments=10)
    leaf('basil', (0.005, WELL + 0.0122, 0.035), 0.022, 0.013, 30, M.leaf, tilt=4)
    blob('brown', (-0.012, WELL + 0.0112, 0.02), 0.007, 0.005, 0.0005, M.cheese_brown, seed=1.0, n=6)
    return done('pizzaSlice')


def treatPlate():
    """A plate of treats: chocolate-chip cookies, one leaning on the others, and a cupcake in a pleated case,
    its icing piped in a swirl to a peak, a cherry on top."""
    plate()
    cookies = (((-0.042, WELL, 0.01), (0, 0)), ((0.04, WELL, 0.025), (0, 0)), ((0.0, WELL + 0.012, -0.012), (9, -6)))
    for i, ((x, y, z), tilt) in enumerate(cookies):
        ck = lathe(f'cookie{i}', [(0, 0), (0.036, 0), (0.038, 0.006), (0.033, 0.012), (0, 0.013)], M.cookie,
                   at=(x, y, z), segments=10, rot=(tilt[0], tilt[1], 0))
        G.jitter(ck, 0.0009, size=0.01, seed=i + 0.5, up=0.5)
        for k, (dx, dz) in enumerate(((-0.015, -0.008), (0.012, 0.012))[:2 if i == 2 else 1]):
            lift = (dz * math.tan(math.radians(tilt[0])) - dx * math.tan(math.radians(tilt[1]))) if i == 2 else 0
            chunk(f'chip{i}{k}', 0.008, (x + dx, y + 0.0125 + lift, z + dz), M.chocolate, yaw=20 * k + 10 * i,
                  tilt=(15, 10))
    cx, cz = 0.058, -0.066
    lathe('cupcake', [(0, 0), (0.022, 0), (0.0285, 0.025), (0.0262, 0.0262), (0, 0.0262)],
          [M.cupcake_case, M.cupcake_case, M.cupcake_case, M.icing], at=(cx, WELL, cz), segments=16,
          wave=(8, {2: 0.07, 3: 0.07}))
    swirl('icing', (cx, WELL + 0.024, cz), 0.019, 0.025, M.icing, tube=0.0095, turns=2.0, steps=10, star=6)
    ball('cherry', 0.0075, (cx, WELL + 0.0555, cz), M.tomato, seg=6, rings=5)
    G.kit.tube(F, 'cherry_stalk', [(cx, WELL + 0.0605, cz), (cx + 0.005, WELL + 0.0632, cz - 0.002),
                                   (cx + 0.011, WELL + 0.0625, cz - 0.004)], 0.0008, M.leaf, verts=4, round=1)
    return done('treatPlate')


def pieSlice():
    """A plate with a slice of cherry pie: pastry under and over the red filling, its crimped edge along the
    back, filling run out at its tip, and a piped rosette of cream with a mint leaf."""
    plate()
    apex, R, A = (-0.015, 0.06), 0.125, 24
    wedge = [apex] + arc(R, A, apex, 3)
    prism('pastry_base', wedge, WELL, WELL + 0.008, M.pastry, M.pastry_baked)
    prism('filling', _shrunk(wedge, 0.03), WELL + 0.008, WELL + 0.036, M.filling, M.filling)
    prism('pastry_top', wedge, WELL + 0.036, WELL + 0.043, M.pastry, M.pastry,
          sides=[M.pastry_baked] + [M.pastry] * (len(wedge) - 1))
    c = crust('edge', arc(R - 0.008, A + 0.5, apex, 10), WELL + 0.039, 0.011, M.pastry_baked, flat=0.85, crimp=5,
              bulge=0.18, taper=0.85)
    G.jitter(c, 0.0005, size=0.015, seed=3.0)
    for i, (x, z) in enumerate(((-0.015, -0.01), (-0.015, -0.04))):
        box(f'slit{i}', 0.022, 0.002, 0.004, (x, WELL + 0.0435, z), M.filling)
    blob('ooze', (-0.015, WELL, 0.068), 0.014, 0.01, 0.0014, M.filling, seed=0.6, n=7)
    swirl('cream', (0.07, WELL, 0.035), 0.014, 0.03, M.cream, tube=0.008, turns=1.8, steps=11)
    leaf('mint', (0.066, WELL + 0.03, 0.033), 0.016, 0.009, 50, M.leaf, tilt=20)
    return done('pieSlice')


# The sandwich and toast halves: a slice cut corner to corner, its loaf corner rounded. Each half's outline,
# the index of its cut side, and which way it is drawn back from the cut.
def _halves(s, rc=0.012):
    hw, hd = s
    k = rc * (1 - math.sqrt(0.5))
    a = [(-hw, -hd + rc), (-hw + k, -hd + k), (-hw + rc, -hd), (hw, -hd), (-hw, hd)]
    b = [(hw, -hd), (hw, hd - rc), (hw - k, hd - k), (hw - rc, hd), (-hw, hd)]
    return (('a', a, 3, -1), ('b', b, 4, 1))


def _frill(pts, grow, amp, n=4):
    """A lettuce leaf's outline: `pts` grown by a share, each side waved out and in `n` times."""
    g = _shrunk(pts, -grow)
    cx = sum(p[0] for p in g) / len(g)
    cz = sum(p[1] for p in g) / len(g)
    out = []
    for p, q in zip(g, g[1:] + g[:1]):
        for k in range(n):
            f = k / n
            x, z = p[0] + (q[0] - p[0]) * f, p[1] + (q[1] - p[1]) * f
            d = math.hypot(x - cx, z - cz) or 1
            w = amp * (1 if k % 2 else -0.4)
            out.append((x + (x - cx) / d * w, z + (z - cz) / d * w))
    return out


def sandwich():
    """A sandwich cut corner to corner: two halves of bread, their crusts round the outside and the crumb at the
    cut and on top, lettuce frilled out past the bread, cheese and tomato."""
    plate()
    nx, nz = 0.052, 0.058
    n = math.hypot(nx, nz)
    for half, pts, cut, sign in _halves((0.058, 0.052)):
        off = (sign * nx / n * 0.008, sign * nz / n * 0.008)
        y = WELL
        for layer, h, m, grow in (('bread_lo', 0.019, M.bread, 0), ('lettuce', 0.006, M.lettuce, 0.08),
                                  ('cheese', 0.004, M.cheese, 0.05), ('tomato', 0.007, M.tomato, -0.04),
                                  ('bread_hi', 0.019, M.bread, 0)):
            if m is M.bread:
                sides = [M.bread if k == cut else M.crust for k in range(len(pts))]
                prism(f'{layer}_{half}', pts, y, y + h, M.bread, M.crust, sides=sides, at=off)
            elif m is M.lettuce:
                lt = F.extrude(f'lettuce_{half}', [(x + off[0], z + off[1]) for x, z in _frill(pts, 0.07, 0.004)], 0.004,
                               (0, y + 0.003, 0), M.lettuce, plane='top')
                G.jitter(lt, 0.0016, size=0.01, seed=3.0 if sign > 0 else 7.0, up=1.0)
            else:
                prism(f'{layer}_{half}', _shrunk(pts, -grow), y, y + h, m, m, at=off)
            y += h
    return done('sandwich')


def _fork(name, at, yaw, pitch=0.0):
    """A steel table fork lying with its tines at the game point `at` (its underside there), handle out along
    `yaw` (degrees): handle, neck, a head of four tines; `pitch` raises the handle's end (degrees)."""
    o = [(0.15, -0.0055), (0.15, 0.0055), (0.08, 0.0042), (0.046, 0.0026), (0.03, 0.0095), (0.021, 0.00975),
         (0.0, 0.00975), (0.0, 0.00675), (0.017, 0.00675), (0.017, 0.00425), (0.0, 0.00425), (0.0, 0.00125),
         (0.017, 0.00125), (0.017, -0.00125), (0.0, -0.00125), (0.0, -0.00425), (0.017, -0.00425),
         (0.017, -0.00675), (0.0, -0.00675), (0.0, -0.00975), (0.021, -0.00975), (0.03, -0.0095), (0.046, -0.0026),
         (0.08, -0.0042)]
    return F.extrude(name, list(reversed(o)), 0.0022, (at[0], at[1] + 0.0011, at[2]), M.steel, plane='top',
                     rot=(0, pitch, yaw))


def _knife(name, at, yaw, pitch=0.0):
    """A table knife lying with its tip at the game point `at`: a rounded blade and a handle."""
    o = [(0.0, 0.0), (0.006, -0.0075), (0.09, -0.0085), (0.098, -0.0055), (0.2, -0.0065), (0.205, -0.004),
         (0.205, 0.004), (0.2, 0.0065), (0.098, 0.0055), (0.09, 0.0035), (0.02, 0.0045)]
    return F.extrude(name, o, 0.0022, (at[0], at[1] + 0.0011, at[2]), M.steel, plane='top', rot=(0, pitch, yaw))


def plateDirty():
    """A plate after the meal: a smear of gravy, a streak of sauce, crumbs, a pea left over, and the knife and
    fork laid side by side."""
    plate()
    blob('smear', (0.02, WELL, 0.02), 0.065, 0.045, 0.0016, M.smear, seed=1.4, yaw=-15)
    blob('sauce', (-0.04, WELL + 0.0004, 0.045), 0.035, 0.011, 0.0016, M.smear_sauce, seed=0.2, yaw=25, n=7)
    for i, (x, z) in enumerate(((-0.05, -0.02), (0.06, 0.05), (0.0, 0.07), (-0.07, 0.02))):
        chunk(f'crumb{i}', 0.008, (x, WELL + 0.004, z), M.crumb, yaw=40 * i)
    ball('pea', 0.0042, (0.045, WELL + 0.0036, -0.03), M.pea, seg=6, rings=4)
    # side by side at twenty past four, the knife's handle resting on the rim
    _fork('fork', (-0.0642, WELL + 0.0003, -0.0301), 35)
    _knife('knife', (-0.067, WELL + 0.0003, -0.061), 35, pitch=1.5)
    return done('plateDirty')


# --- the quick foods, each its own (review M-held-01: they borrowed the plates and the prep bowl) -----------------
def _dessert_bowl():
    """A glazed dessert bowl 12.5 cm across and 5 cm deep, its rim rolled, its inside floor at 1.2 cm."""
    lathe('bowl', [(0, 0), (0.035, 0), (0.04, 0.006), (0.0612, 0.047), (0.0615, 0.051), (0.0565, 0.051),
                   (0.053, 0.044), (0, 0.012)],
          [M.bowl] * 6 + [M.china_well], segments=20)


def iceCream():
    """A dessert bowl of ice cream from the freezer: three scoops (vanilla, strawberry, chocolate), each with the
    ragged lip a scoop leaves, chocolate sauce trickled over them, a fan wafer and a spoon."""
    _dessert_bowl()
    for i, (x, z, m, s) in enumerate(((-0.018, 0.012, M.vanilla, 1.0), (0.02, 0.008, M.strawberry, 0.95),
                                      (0.0, -0.022, M.choc_ice, 0.9))):
        sc = lathe(f'scoop{i}', [(0, 0), (0.027 * s, 0), (0.031 * s, 0.006 * s), (0.03 * s, 0.022 * s),
                                 (0.02 * s, 0.039 * s), (0, 0.044 * s)], m,
                   at=(x, 0.026, z), segments=10, wave=(5, {2: 0.12}), rot=(0, 0, 25 * i))
        G.jitter(sc, 0.0012, size=0.01, seed=1.3 + i)
    G.kit.tube(F, 'sauce', [(-0.035, 0.05, 0.02), (-0.02, 0.0695, 0.012), (-0.005, 0.062, 0.0), (0.008, 0.063, -0.018),
                            (0.02, 0.066, 0.004), (0.035, 0.054, 0.012)], 0.0022, M.chocolate, verts=4, round=1)
    box('wafer', 0.036, 0.004, 0.026, (-0.004, 0.077, -0.004), M.wafer, yaw=30, tilt=(55, 10), bevel=0.0008)
    _spoon((0.025, 0.07, 0.03), 35, 16)
    return done('iceCream')


def bowlDirty():
    """The dessert bowl eaten from: a film of melted ice cream in the bottom, a streak of chocolate, a drip down the
    side and the spoon left in it."""
    _dessert_bowl()
    blob('melt', (0.004, 0.012, 0.0), 0.034, 0.028, 0.0016, M.melt, seed=0.5, yaw=15)
    blob('melt_choc', (-0.012, 0.0132, -0.01), 0.014, 0.008, 0.0012, M.choc_ice, seed=1.5, yaw=-30, n=6)
    blob('drip', (0.02, 0.0134, 0.012), 0.009, 0.006, 0.001, M.strawberry, seed=2.5, n=6)
    _spoon((0.012, 0.022, 0.012), 30, 33)
    return done('bowlDirty')


def toast():
    """A plate of toast: a slice cut corner to corner, its loaf corner rounded, toasted golden with darker crusts
    and a scorch or two, a pat of butter melting into a pool on each half."""
    plate()
    nx, nz = 0.05, 0.055
    n = math.hypot(nx, nz)
    for half, pts, cut, sign in _halves((0.055, 0.05)):
        off = (sign * nx / n * 0.01, sign * nz / n * 0.01)
        sides = [M.bread if k == cut else M.toast_crust for k in range(len(pts))]
        prism(f'slice_{half}', pts, WELL, WELL + 0.013, M.toast, M.toast_crust, sides=sides, at=off)
        cx = sum(p[0] for p in pts) / len(pts) + off[0]
        cz = sum(p[1] for p in pts) / len(pts) + off[1]
        blob(f'scorch_{half}', (cx - 0.015 * sign, WELL + 0.013, cz + 0.012 * sign), 0.01, 0.006, 0.0004,
             M.toast_crust, seed=sign, n=6)
        blob(f'pool_{half}', (cx, WELL + 0.013, cz), 0.015, 0.012, 0.0006, M.butter, seed=sign * 2, n=7)
        box(f'butter_{half}', 0.018, 0.005, 0.016, (cx, WELL + 0.0154, cz), M.butter, yaw=20 if sign > 0 else -10,
            bevel=0.0018)
    return done('toast')


def smoothie():
    """A tall tumbler of berry smoothie, 8 cm across and 14 cm high (about 0.4 L): a thick glass foot, the
    smoothie seen through the glass up to a frothy top, a striped bendy straw, a raspberry and two mint leaves."""
    lathe('glass', [(0, 0), (0.03, 0), (0.031, 0.004), (0.0315, 0.013), (0.039, 0.12), (0.0398, 0.134), (0.04, 0.1395),
                    (0.0378, 0.1402), (0.0366, 0.126), (0.03, 0.1265), (0, 0.129)],
          [M.glass, M.glass, M.glass, M.smoothie, M.glass, M.glass, M.glass, M.glass, M.smoothie_top, M.smoothie_top],
          segments=20)
    path = G.kit.fillet([(0.006, 0.1, 0.004), (0.011, 0.172, 0.007), (0.031, 0.19, 0.016)], 0.012, steps=3)
    G.kit.tube(F, 'straw', path, 0.0035, M.straw, verts=6, round=1)
    for i, y in enumerate((0.142, 0.154, 0.166)):
        k = (y - 0.1) / 0.072
        G.kit.tube(F, f'stripe{i}', [(0.006 + 0.005 * k, y, 0.004 + 0.003 * k),
                                     (0.006 + 0.005 * (k + 0.06), y + 0.0045, 0.004 + 0.003 * (k + 0.06))],
                   0.0037, M.straw_stripe, verts=6, round=1)
    b = ball('raspberry', 0.0085, (-0.015, 0.1335, -0.008), M.berry, seg=7, rings=5)
    G.jitter(b, 0.0009, size=0.003, seed=2.0)
    for i, (x, z, yaw) in enumerate(((-0.004, -0.012, 30), (-0.022, 0.004, 110))):
        leaf(f'mint{i}', (x, 0.1305, z), 0.018, 0.01, yaw, M.leaf, tilt=12)
    return done('smoothie')


def noodles():
    """A takeaway noodle pail of white card with a red band: its four flaps standing open, a wire handle, noodles
    and spring onion heaped in its top and a pair of chopsticks standing in it."""
    def ring(y, hw):
        return [(x, y, z) for x, z in rrect(hw, hw, n=16, square=9)]

    def side(y):
        return 0.035 + 0.015 * y / 0.1
    loft('pail', [ring(0, side(0)), ring(0.05, side(0.05))], M.paper)
    loft('band', [ring(0.05, side(0.05) + 0.0004), ring(0.068, side(0.068) + 0.0004)], M.paper_red)
    loft('pail_top', [ring(0.068, side(0.068)), ring(0.1, side(0.1)), ring(0.1, side(0.1) - 0.0025),
                      ring(0.088, side(0.088) - 0.0025)], M.paper, angle=30)
    # the flaps: a trapezoid of card standing up from the front and back edges, leaning out a little (the side
    # ones are tucked in, where the wire handle hooks in)
    tab = [(-0.038, 0.0), (0.038, 0.0), (0.022, 0.026), (-0.022, 0.026)]
    for i, yaw in enumerate((0, 180)):
        z = 0.0488 if yaw == 0 else -0.0488
        F.extrude(f'flap{i}', tab, 0.0012, (0, 0.0995, z), M.paper, plane='front', rot=(2, 0, yaw))
    mound = lathe('noodles', [(0, 0), (0.044, 0), (0.035, 0.008), (0.018, 0.014), (0, 0.016)], M.noodle,
                  at=(0, 0.086, 0), segments=10)
    G.jitter(mound, 0.0025, size=0.01, seed=0.8)
    for i, (pts) in enumerate(([(-0.03, 0.093, 0.0), (-0.018, 0.1, 0.012), (0.0, 0.097, 0.02), (0.02, 0.102, 0.008)],
                               [(0.03, 0.094, -0.02), (0.01, 0.101, -0.012), (-0.008, 0.098, -0.026), (-0.028, 0.096, -0.018)])):
        G.kit.tube(F, f'noodle{i}', pts, 0.0026, M.noodle, verts=4, round=2)
    for i, (x, z) in enumerate(((-0.01, -0.02), (0.02, 0.0), (-0.025, 0.02))):
        G.cylinder(f'onion{i}', 0.0045, 0.0025, at=F.at(x, 0.103, z), material=M.lettuce, verts=6, rot=(20, 10 * i, 0))
    # the wire handle, from side to side over the top, hooked into the card below the band
    pts = [(-0.0488 * math.cos(math.radians(a)), 0.088 + 0.07 * math.sin(math.radians(a)), 0.0)
           for a in range(0, 181, 15)]
    G.kit.tube(F, 'wire', pts, 0.0011, M.wire, verts=4, round=1)
    for i, dx in enumerate((-0.006, 0.006)):
        F.sweep(f'chopstick{i}', [(0.01 + dx, 0.06, -0.005), (0.033 + dx * 2, 0.268, -0.03)], None, M.chopstick,
                profile=[(-0.0035, -0.0035), (0.0035, -0.0035), (0.0035, 0.0035), (-0.0035, 0.0035)], scale=[0.5, 1.0])
    return done('noodles')


def _hinged(points, angle=110.0, hinge=(0.045, -0.07)):
    """Game points of the takeaway box's lid as if shut (x, height above the rim, depth from the hinge forward),
    turned open about the hinge along the box's back edge by `angle` degrees."""
    c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
    return [(x, hinge[0] + h * c + d * s, hinge[1] - h * s + d * c) for x, h, d in points]


def takeout():
    """An open kraft takeaway box, its sides flared, its lid standing open at the back: a burger (sesame bun,
    patty, cheese, lettuce, tomato) and a heap of fries."""
    def ring(y, hw, hd, n=12):
        return [(x, y, z) for x, z in rrect(hw, hd, n=n, square=8)]
    loft('tray', [ring(0, 0.085, 0.06), ring(0.045, 0.095, 0.07), ring(0.045, 0.091, 0.066), ring(0.004, 0.081, 0.056)],
         M.kraft, angle=30)
    # the lid, shallow, its inside to the front
    lid = []
    for h, hw, hd in ((0, 0.095, 0.07), (0.005, 0.089, 0.064), (0.0035, 0.085, 0.06), (0, 0.091, 0.066)):
        lid.append(_hinged([(x, h, 0.07 + z) for x, z in rrect(hw, hd, n=12, square=8)]))
    loft('lid', lid, M.kraft_fold, angle=30)
    # the burger
    bx, bz, y = -0.04, 0.005, 0.0045
    lathe('bun_base', [(0, 0), (0.04, 0), (0.0425, 0.008), (0.04, 0.016), (0, 0.016)], M.bun_base, at=(bx, y, bz),
          segments=10)
    _patty('patty', (bx, y + 0.016, bz), d=0.09, h=0.015, marks=False)
    box('cheese', 0.07, 0.002, 0.07, (bx, y + 0.032, bz), M.cheese, yaw=45, tilt=(2, -2))
    lt = F.extrude('lettuce', [(0.048 * (1 + 0.1 * (k % 2)) * math.cos(TAU * k / 14), 0.048 * (1 + 0.1 * (k % 2)) *
                                math.sin(TAU * k / 14)) for k in range(14)], 0.003, (bx, y + 0.0345, bz), M.lettuce,
                   plane='top')
    G.jitter(lt, 0.0015, size=0.01, seed=2.2)
    coin('tomato', 0.07, 0.004, (bx, y + 0.036, bz), M.tomato, segments=10, cup=0.0)
    lathe('bun_top', [(0, 0), (0.043, 0), (0.044, 0.008), (0.035, 0.026), (0.018, 0.034), (0, 0.036)],
          [M.bun_base, M.bun, M.bun, M.bun, M.bun], at=(bx, y + 0.04, bz), segments=12)
    for i, (dx, dz) in enumerate(((0.0, 0.0), (0.016, 0.01), (-0.014, 0.012), (0.01, -0.017))):
        h = 0.036 - 0.01 * (dx * dx + dz * dz) / 0.0004
        box(f'sesame{i}', 0.004, 0.0015, 0.0025, (bx + dx, y + 0.04 + h, bz + dz), M.sesame, yaw=37 * i,
            tilt=(-dz * 900, dx * 900))
    # the fries, heaped beside it
    for i in range(8):
        a = 37 * i
        box(f'fry{i}', 0.056 - 0.004 * (i % 3), 0.008, 0.008,
            (0.045 + 0.01 * math.cos(math.radians(a * 3)), y + 0.016 + 0.009 * (i % 3),
             0.01 + 0.03 * math.sin(math.radians(a * 2))), M.fries, yaw=a, tilt=(0, 18 + 9 * (i % 3)))
    return done('takeout')


# --- the dishes emptied, each the dish that went in (review M-held-02) ----------------------------------------------
def boardDirty():
    """The tasting board or cheese board cleared: a cheese rind, a bare grape stalk, a broken cracker, crumbs and a
    smear of jam."""
    _paddle('board', M.board_dark)
    blob('smear', (-0.06, 0.028, 0.0), 0.07, 0.04, 0.0014, M.grease, seed=0.8, yaw=12)
    blob('jam', (0.03, 0.028, 0.065), 0.02, 0.012, 0.0014, M.jam, seed=1.6, yaw=-20, n=7)
    prism('rind', [(-0.14, -0.05), (-0.11, -0.04), (-0.135, -0.02)], 0.028, 0.04, M.cheese, M.cheese)
    G.kit.tube(F, 'stalk', [(0.13, 0.031, -0.03), (0.11, 0.0325, -0.03), (0.08, 0.031, -0.02)], 0.0018, M.leaf,
               verts=4, round=2)
    for i, (x, z, dx, dz) in enumerate(((0.11, -0.03, -0.004, 0.02), (0.095, -0.026, 0.006, 0.018), (0.09, -0.023, -0.01, -0.015))):
        G.kit.tube(F, f'twig{i}', [(x, 0.0315, z), (x + dx, 0.0305, z + dz)], 0.0011, M.leaf, verts=4, round=1)
    box('cracker', 0.03, 0.005, 0.022, (-0.03, 0.0305, 0.07), M.cracker, yaw=25, bevel=0.0012)
    for i, (x, z) in enumerate(((-0.02, 0.05), (0.03, 0.03), (-0.09, 0.06), (0.05, -0.06), (0.0, -0.03), (-0.12, 0.02))):
        chunk(f'crumb{i}', 0.007, (x, 0.031, z), M.cracker, yaw=33 * i)
    return done('boardDirty')


def pizzaBoardDirty():
    """The pizza's round board after it: crust ends left at the edge, a grease smear and a smudge of sauce, crumbs
    and the pizza wheel."""
    lathe('board', [(0, 0), (0.197, 0), (0.2, 0.004), (0.2, 0.016), (0.197, 0.02), (0, 0.02)], M.peel, segments=28)
    blob('grease', (-0.01, 0.02, 0.01), 0.075, 0.055, 0.0012, M.grease, seed=0.9, yaw=10, n=10)
    blob('sauce', (0.05, 0.0208, -0.04), 0.03, 0.015, 0.0012, M.smear_sauce, seed=1.9, yaw=-25, n=7)
    for i, (a, length) in enumerate(((30, 0.06), (150, 0.05), (250, 0.055))):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        cr = roll(f'crust{i}', 0.009, length, M.dough_baked, (0.13 * c, 0.029, 0.13 * s), yaw=a + 90, segments=7,
                  bend=-0.006)
        G.jitter(cr, 0.0008, size=0.01, seed=i + 0.3, up=0.3)
    for i, (x, z) in enumerate(((-0.05, 0.04), (0.03, 0.08), (0.08, 0.02), (-0.02, -0.07), (-0.08, -0.02))):
        chunk(f'crumb{i}', 0.008, (x, 0.0235, z), M.crumb, yaw=40 * i)
    # the pizza wheel, lying on its side: the blade, its guard and hub, and a handle
    wx, wz = -0.06, -0.06
    disc('wheel', 0.064, 0.0016, (wx, 0.0205, wz), M.steel, verts=16)
    disc('hub', 0.016, 0.006, (wx, 0.0205, wz), M.steel, verts=8)
    F.sweep('handle', [(wx - 0.006, 0.026, wz), (wx - 0.03, 0.0275, wz - 0.004), (wx - 0.085, 0.0275, wz - 0.014)],
            None, M.grip, profile=G.rounded_rect(0.014, 0.018, 0.006, corner=1), scale=[0.7, 1.0, 1.05])
    return done('pizzaBoardDirty')


def pieDishDirty():
    """The pie dish emptied: a red smear of filling over the bottom, pastry crumbs against its sides and the pie
    server left in it."""
    _pie_dish([M.dish, M.residue], [(0.128, 0.01), (0, 0.01)])
    blob('filling', (0.01, 0.01, 0.0), 0.08, 0.06, 0.0016, M.smear_filling, seed=0.4, yaw=20, n=10)
    for i, a in enumerate((20, 95, 170, 230, 300, 340)):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        chunk(f'pastry{i}', 0.012 + 0.002 * (i % 2), (0.112 * c, 0.016, 0.112 * s), M.pastry_baked, yaw=a, tilt=(15, 10))
    blade = [(0.0, 0.0), (0.06, -0.028), (0.066, -0.026), (0.07, -0.012), (0.07, 0.012), (0.066, 0.026), (0.06, 0.028)]
    F.extrude('server', [(u - 0.08, v) for u, v in blade], 0.0016, (0.0, 0.0118, -0.02), M.steel, plane='top',
              rot=(0, 0, -20))
    F.sweep('server_handle', [(-0.0094, 0.0124, -0.0166), (0.0235, 0.014, -0.0286), (0.094, 0.016, -0.0542)],
            None, M.steel, profile=G.rounded_rect(0.003, 0.009, 0.0012, corner=1), scale=[0.6, 0.9, 1.1])
    return done('pieDishDirty')


# --- tools: on the station while a resident cooks ---------------------------------------------------------------
def board():
    """The chopping board: rounded corners, a juice groove round its face and a slot to hang it by; a carrot half
    chopped into coins, chopped parsley and the chef's knife."""
    hole = [(0.152 + u, v) for u, v in G.rounded_rect(0.016, 0.07, 0.008, corner=2)]
    F.extrude('board', G.rounded_rect(0.34, 0.24, 0.02, corner=2), 0.025, (0, 0.0125, 0), M.board, plane='top',
              holes=[hole], bevel=0.004, bevel_segments=1)
    groove = G.rounded_rect(0.27, 0.205, 0.018, corner=2)
    F.sweep('groove', [(x - 0.01, 0.0251, z) for x, z in groove], None, M.board_dark,
            profile=[(-0.0004, -0.0025), (0.0004, -0.0025), (0.0004, 0.0025), (-0.0004, 0.0025)], closed=True)
    # the carrot: tapering to its root, cut flat where the coins came off
    path = [(-0.145, 0.04, 0.035), (-0.13, 0.04, 0.034), (-0.1, 0.04, 0.032), (-0.06, 0.04, 0.03)]
    car = F.sweep('carrot', path, 1.0, M.carrot, verts=8, scale=[0.003, 0.008, 0.013, 0.015])
    G.jitter(car, 0.0006, size=0.008, seed=1.0)
    for i, (x, z, t) in enumerate(((-0.04, 0.04, 0), (-0.025, 0.062, 8), (-0.035, 0.015, -6), (-0.012, 0.036, 4))):
        coin(f'coin{i}', 0.028, 0.005, (x, 0.025, z), (M.carrot, M.carrot_core), tilt=(t, 0))
    for i, (x, z, yaw) in enumerate(((0.02, 0.07, 10), (0.04, 0.05, 40), (0.03, 0.09, 70), (0.055, 0.075, 20),
                                     (0.065, 0.055, 120))):
        leaf(f'parsley{i}', (x, 0.0252, z), 0.014, 0.009, yaw, M.pea, tilt=10 * (i % 2), n=3)
    with G.new_parts() as knife:
        o = [(-0.0575, 0.01), (-0.035, -0.004), (0.0, -0.012), (0.045, -0.015), (0.06, -0.015), (0.06, 0.015),
             (-0.045, 0.015)]
        G.extrude('blade', o, 0.002, at=(0.0, 0.0, 0.0), material=M.steel, plane='XY')
        G.cube('bolster', (0.01, 0.02, 0.012), at=(0.064, 0.005, 0.005), material=M.steel)
        G.cube('handle', (0.07, 0.016, 0.018), at=(0.103, 0.006, 0.007), material=M.grip, bevel=0.005,
               bevel_segments=1)
        for k, u in enumerate((0.085, 0.12)):
            G.cylinder(f'rivet{k}', 0.0022, 0.0186, at=(u, 0.006, 0.007), material=M.steel, verts=6)
    k = G.join(knife, 'knife', origin=(0, 0, 0))
    k.location = F.at(0.05, 0.027, -0.07)
    k.rotation_euler = (0, 0, math.radians(15))
    return done('board')


def pan():
    """The frying pan on the stove, its handle to the back (+depth: away from the cook, as the old part's,
    out of the way of the hands): a heavy rolled rim, the handle riveted on through a bracket and ending in a
    rounded grip; a heap of stir-fry, strips of beef, broccoli, peppers and carrot."""
    lathe('pan', [(0, 0), (0.112, 0), (0.118, 0.003), (0.1385, 0.044), (0.14, 0.0482), (0.1372, 0.0502),
                  (0.1342, 0.0472), (0.113, 0.0115), (0.105, 0.009), (0, 0.009)], M.iron, segments=24)
    G.kit.tube(F, 'bracket', [(0, 0.03, 0.128), (0, 0.04, 0.152), (0, 0.044, 0.172)], 0.0075, M.iron, verts=6, round=1)
    for t in (-1, 1):
        G.cylinder(f'rivet{t}', 0.004, 0.004, at=F.at(t * 0.008, 0.03, 0.129), material=M.iron, rot=(90, 0, 0), verts=6)
    F.sweep('handle', [(0, 0.043, 0.166), (0, 0.045, 0.2), (0, 0.047, 0.235), (0, 0.048, 0.25), (0, 0.048, 0.2545)],
            None, M.grip, profile=G.rounded_rect(0.018, 0.026, 0.008, corner=2), up=(0, 0, 1),
            scale=[0.85, 0.92, 1.0, 0.88, 0.55])
    heap = lump('heap', 0.075, (0.0, 0.006, 0.0), M.stew, scale=(1, 0.32, 1), seed=8.5, noise=0.15, subdivisions=2)
    G.jitter(heap, 0.003, size=0.025, seed=8.5)

    def top(x, z):
        return 0.012 + 0.04 * math.sqrt(max(0.0, 1 - (math.hypot(x, z) / 0.08) ** 2))
    for i, (x, z, yaw) in enumerate(((0.02, -0.01, 10), (-0.04, 0.02, 40), (0.05, 0.04, 70), (-0.02, -0.05, 25),
                                     (0.0, 0.03, 120))):
        box(f'beef{i}', 0.034, 0.008, 0.012, (x, top(x, z) + 0.002, z), M.beef, yaw=yaw, tilt=(10 * (i % 3 - 1), 6),
            bevel=0.0025)
    for i, (x, z) in enumerate(((0.05, -0.035), (-0.055, -0.02), (-0.03, 0.055), (0.07, 0.01))):
        y = top(x, z) - 0.006
        G.cylinder(f'stem{i}', 0.004, 0.012, at=F.at(x, y + 0.006, z), material=M.pea, verts=5)
        lump(f'floret{i}', 0.0115, (x, y + 0.009, z), M.broccoli, scale=(1, 0.8, 1), seed=i * 2.3, rough=0.25)
    for i, (x, z, yaw, m) in enumerate(((-0.01, -0.025, 30, M.pepper_red), (0.035, 0.015, 100, M.pepper_yellow),
                                        (-0.05, 0.035, 150, M.pepper_red), (0.02, 0.06, 60, M.pepper_yellow))):
        a = math.radians(yaw)
        ca, sa = math.cos(a), math.sin(a)
        y = top(x, z) + 0.002
        F.sweep(f'pepper{i}', [(x - 0.017 * ca, y - 0.002, z + 0.017 * sa), (x + 0.004 * sa, y + 0.001, z + 0.004 * ca),
                               (x + 0.017 * ca, y - 0.002, z - 0.017 * sa)], None, m,
                profile=[(-0.002, -0.0045), (0.002, -0.0045), (0.002, 0.0045), (-0.002, 0.0045)], resolution=2)
    for i, (x, z, t) in enumerate(((0.0, -0.06, 14), (-0.065, 0.0, -10), (0.04, -0.06, 8), (0.06, 0.06, -12))):
        coin(f'carrot{i}', 0.02, 0.004, (x, top(x, z) - 0.004, z), (M.carrot, M.carrot_core), tilt=(t, t * 0.5))
    return done('pan')


def grillFood():
    """On the grill: two sausages, bent and marked by the bars, and a burger with its rounded, rough edge and
    the bars' marks across it."""
    for i, (z, m, b) in enumerate(((-0.0526, M.sausage, -0.007), (0.033, M.sausage_b, -0.007))):
        roll(f'sausage{i}', 0.0225, 0.16, (m, M.char), (-0.08, 0.0225, z), segments=12, bend=b,
             marks=((-0.55, -0.4), (-0.05, 0.1), (0.45, 0.6)))
    _patty('patty', (0.08, 0, 0.0), h=0.03)
    return done('grillFood')


def mixingBowl():
    """On the counter while batter is mixed or pastry made (`mix`): a deep glazed bowl, its rim rolled, of batter
    stirred into a swirl, a wooden spoon standing in it, leaning on the far rim (+depth: away from the cook, as
    the pan's handle)."""
    lathe('bowl', [(0, 0), (0.07, 0), (0.085, 0.01), (0.128, 0.104), (0.13, 0.109), (0.1265, 0.113), (0.1215, 0.1115),
                   (0.117, 0.1), (0.06, 0.0745), (0, 0.075)],
          [M.mixing] * 7 + [M.batter, M.batter], segments=20)
    pts = []
    for i in range(17):
        f = i / 16
        a = TAU * 1.6 * f
        rr = 0.09 * (1 - f) + 0.01
        pts.append((0.008 + rr * math.cos(a), 0.0752 + 0.002 * f, -0.006 + rr * math.sin(a)))
    F.sweep('ridge', pts, None, M.batter, profile=[(-0.002, -0.004), (0.0008, -0.0025), (0.0008, 0.0025),
                                                   (-0.002, 0.004)], scale=lambda f: 1 - 0.5 * f)
    # the spoon: a handle tapering down to the bowl, buried in the batter
    s0, s1 = Vector((0.0, 0.07, 0.02)), Vector((0.03, 0.27, 0.13))
    d = (s1 - s0).normalized()
    F.sweep('spoon_handle', [tuple(s0 + d * 0.012), tuple(s0 + (s1 - s0) * 0.5), tuple(s1 - d * 0.004), tuple(s1)],
            0.007, M.spoon, verts=8, scale=[0.6, 0.85, 1.0, 0.75])
    G.sphere('spoon_bowl', 1.0, at=F.at(*(s0 - d * 0.004)), material=M.spoon, scale=(0.028, 0.02, 0.008),
             rot=aim(tuple(d)), segments=8, rings=5)
    return done('mixingBowl')


# --- props in hand (G-70): drawn in a resident's hands while they use a thing ---------------------------------------
# Each is built as it lies or stands on a table, in the game's axes, its origin where characters.ts holds it from
# (actions.ts `HAND_PROPS`): the middle of its base for the cups and the bag, the butt of a cue, the middle of a
# grip for the rest. What is held along its length (cutlery, a dart, a cue, a bat) points to the front (+depth).
def mug():
    """A glazed mug 8 cm across and 9.5 cm high, a blue band round it, its handle to the right (+x, the side of the
    hand that holds it), coffee in it with a little crema."""
    lathe('mug', [(0, 0), (0.037, 0), (0.0405, 0.004), (0.041, 0.06), (0.041, 0.072), (0.041, 0.092), (0.0395, 0.095),
                  (0.0365, 0.0935), (0.036, 0.082), (0.03, 0.0822), (0, 0.0824)],
          [M.mug, M.mug, M.mug, M.mug_band, M.mug, M.mug, M.mug, M.mug, M.coffee, M.crema], segments=18)
    path = G.kit.fillet([(0.038, 0.078, 0.0), (0.066, 0.074, 0.0), (0.066, 0.03, 0.0), (0.038, 0.024, 0.0)], 0.012,
                        steps=3)
    F.sweep('handle', path, None, M.mug, profile=G.rounded_rect(0.007, 0.012, 0.003, corner=1))
    return done('mug')


def glass():
    """A tumbler 7 cm across and 11 cm high of something orange and cold, two cubes of ice at the top."""
    lathe('glass', [(0, 0), (0.03, 0), (0.0315, 0.006), (0.0325, 0.012), (0.035, 0.09), (0.0355, 0.11),
                    (0.0335, 0.1105), (0.033, 0.092), (0, 0.092)],
          [M.glass, M.glass, M.glass, M.juice, M.glass, M.glass, M.glass, M.juice], segments=18)
    for i, (x, z, yaw) in enumerate(((-0.01, 0.006, 20), (0.012, -0.008, 65))):
        box(f'ice{i}', 0.016, 0.012, 0.016, (x, 0.094, z), M.ice, yaw=yaw, tilt=(8, -6), bevel=0.002)
    return done('glass')


def wineGlass():
    """A stemmed glass of red wine, 19 cm high: a round foot, a stem, a bowl filled to its widest (the wine seen
    through the glass)."""
    lathe('glass', [(0, 0), (0.034, 0), (0.0345, 0.003), (0.008, 0.007), (0.004, 0.02), (0.004, 0.078), (0.012, 0.088),
                    (0.034, 0.112), (0.0415, 0.14), (0.039, 0.172), (0.036, 0.19), (0.0345, 0.19), (0.0375, 0.172),
                    (0.0395, 0.142), (0, 0.142)],
          [M.glass] * 6 + [M.wine, M.wine, M.glass, M.glass, M.glass, M.glass, M.glass, M.wine], segments=18)
    return done('wineGlass')


def can():
    """A 330 ml can of soda, 6.6 cm across and 12 cm high: red with a white band, its top and bottom pressed in,
    a ring pull."""
    lathe('can', [(0, 0.003), (0.024, 0.0), (0.031, 0.004), (0.033, 0.012), (0.033, 0.05), (0.033, 0.075),
                  (0.033, 0.108), (0.0285, 0.118), (0.027, 0.1205), (0.0255, 0.1195), (0.0245, 0.117), (0, 0.117)],
          [M.can_top, M.can_top, M.can, M.can, M.can_band, M.can, M.can_top, M.can_top, M.can_top, M.can_top,
           M.can_top], segments=18)
    F.extrude('pull', [(-0.006, -0.004), (0.006, -0.004), (0.008, 0.008), (0.0, 0.014), (-0.008, 0.008)], 0.0012,
              (0.0, 0.1178, 0.006), M.can_top, plane='top', holes=[[(-0.003, 0.004), (0.003, 0.004), (0.0, 0.009)]])
    return done('can')


def _tines(n, half, length, step, neck):
    """A fork's head outline (u along it, v across) from its neck (u 0, `neck` each side) to the tips (u `length`),
    `n` tines across `half` each side, split down to `step`."""
    gap = 2 * half / (2 * n - 1)
    pts = [(0.0, -neck), (0.014, -half)]
    for k in range(n):
        a = -half + 2 * k * gap
        pts += [(length, a), (length, a + gap)]
        if k < n - 1:
            pts += [(step, a + gap), (step, a + 2 * gap)]
    return pts + [(0.014, half), (0.0, neck)]


def _along(name, outline, t, y, shift, material):
    """A flat piece lying along the depth (u to the front, v across), `t` thick with its underside at `y`, moved
    `shift` along the depth."""
    return F.extrude(name, [(v, u + shift) for u, v in outline], t, (0.0, y + t / 2, 0.0), material, plane='top')


def fork():
    """A steel table fork, 18 cm long, lying flat, its tines to the front (+depth) and held at the origin, 6 cm
    from the handle's end: a handle widening to its end, a neck, four tines."""
    handle = [(-0.125, 0.0), (-0.12, -0.0058), (-0.04, -0.0042), (0.0, -0.0026), (0.0, 0.0026), (-0.04, 0.0042),
              (-0.12, 0.0058)]
    _along('handle', handle, 0.0024, 0.0, 0.065, M.steel)
    _along('head', _tines(4, 0.0115, 0.046, 0.026, 0.0026), 0.0022, 0.0002, 0.065, M.steel)
    return done('fork')


def spoon():
    """A steel dessert spoon, 17 cm long, lying flat, its bowl to the front (+depth) and held at the origin, 6 cm
    from the handle's end."""
    handle = [(-0.12, 0.0), (-0.115, -0.0062), (-0.05, -0.0045), (0.0, -0.0028), (0.0, 0.0028), (-0.05, 0.0045),
              (-0.115, 0.0062)]
    _along('handle', handle, 0.0024, 0.0, 0.06, M.steel)
    G.sphere('bowl', 1.0, at=F.at(0.0, 0.0055, 0.082), material=M.steel, scale=(0.017, 0.025, 0.0055), segments=10,
             rings=6)
    return done('spoon')


def chopsticks():
    """A pair of tapering wooden chopsticks, 23 cm long, side by side, their tips to the front (+depth), held from
    the origin."""
    for i, dx in enumerate((-0.006, 0.006)):
        F.sweep(f'chopstick{i}', [(dx * 1.6, 0.0035, -0.06), (dx * 0.5, 0.0035, 0.17)], None, M.chopstick,
                profile=[(-0.0035, -0.0035), (0.0035, -0.0035), (0.0035, 0.0035), (-0.0035, 0.0035)], scale=[1.0, 0.5])
    return done('chopsticks')


def phone():
    """A smartphone 7.2 by 14.8 cm and 8 mm thick, lying on its back, its screen up and its top to the front
    (+depth): a dark rounded body, the glass, a camera at the back."""
    F.extrude('body', G.rounded_rect(0.072, 0.148, 0.009, corner=3), 0.0075, (0, 0.00375, 0), M.phone, plane='top',
              bevel=0.002, bevel_segments=1)
    F.extrude('screen', G.rounded_rect(0.066, 0.14, 0.007, corner=3), 0.0008, (0, 0.0078, 0), M.screen, plane='top')
    F.extrude('glow', G.rounded_rect(0.058, 0.118, 0.003, corner=2), 0.0004, (0, 0.0083, -0.002), M.screen_lit,
              plane='top')
    F.extrude('camera', G.rounded_rect(0.022, 0.022, 0.005, corner=2), 0.0016, (-0.018, -0.0006, 0.054), M.phone,
              plane='top')
    return done('phone')


def book():
    """An open cloth-bound book, 30 cm across and 22 cm from the foot of its pages to their head (+depth), lying on
    its boards: the pages rising from the spine and curving down to the fore-edges, lines of type in grey bands."""
    for s in (-1, 1):
        box(f'board{s}', 0.152, 0.0032, 0.224, (s * 0.076, 0.0016, 0.0), M.cover, tilt=(0, -s * 3), bevel=0.0008)
        prof = [(0.0, 0.004), (0.002, 0.022), (0.03, 0.027), (0.08, 0.026), (0.125, 0.021), (0.146, 0.014),
                (0.148, 0.006), (0.146, 0.004)]
        F.extrude(f'pages{s}', [(s * u, v) for u, v in prof], 0.214, (0, 0, 0), M.page_edge, plane='front')
        # the open page's face: a thin sheet over the block, the type on it
        top = [(s * u, v + 0.0006) for u, v in prof[1:6]]
        F.sweep(f'sheet{s}', [(x, y, 0.0) for x, y in top], None, M.page,
                profile=[(-0.0003, -0.107), (0.0003, -0.107), (0.0003, 0.107), (-0.0003, 0.107)], up=(0, 0, 1))
        for k in range(7):
            z = -0.075 + 0.025 * k
            F.sweep(f'type{s}{k}', [(x, y + 0.0005, z) for x, y in top[1:4]], None, M.print,
                    profile=[(-0.0003, -0.006), (0.0003, -0.006), (0.0003, 0.006), (-0.0003, 0.006)], up=(0, 0, 1))
    return done('book')


def fountainCoin():
    """A coin for the fountain, 2.3 cm across: a milled rim and a raised face."""
    lathe('coin', [(0, 0), (0.0115, 0), (0.0117, 0.0011), (0.0115, 0.0022), (0.0098, 0.0022), (0.0096, 0.0018),
                   (0, 0.0019)], M.brass, segments=14)
    return done('coin')


def seedBag():
    """A small paper bag of bird seed, 8 cm wide and 12 cm high, its top rolled open over the seed."""
    rings = [_bag_ring(0.0, 0.04, 0.025, 0.012), _bag_ring(0.085, 0.041, 0.026, 0.012),
             _bag_ring(0.1, 0.043, 0.028, 0.006), _bag_ring(0.108, 0.04, 0.025, 0.004)]
    G.loft('bag', [[F.at(*p) for p in r] for r in rings], material=M.kraft, angle=30)
    mound = lathe('seed', [(0, 0), (0.038, 0), (0.028, 0.007), (0, 0.011)], M.seed, at=(0, 0.1075, 0), segments=10,
                  scale=(1, 0.6))
    G.jitter(mound, 0.002, size=0.006, seed=3.1)
    return done('seedBag')


def dart():
    """A steel dart, 15 cm long, lying along the depth, its point to the front (+depth), held by the barrel at the
    origin: a knurled barrel, a black shaft and three blue flights."""
    lathe('barrel', [(0, -0.02), (0.0035, -0.019), (0.0042, -0.012), (0.0046, 0.0), (0.0042, 0.014), (0.0032, 0.022),
                     (0.0011, 0.026), (0.0006, 0.06), (0, 0.064)],
          [M.steel] * 8, at=(0, 0.0046, 0), segments=8, rot=(90, 0, 0), wave=(8, {3: 0.06}))
    F.sweep('shaft', [(0, 0.0046, -0.02), (0, 0.0046, -0.065)], 0.0021, M.dart_shaft, verts=6)
    for k in range(3):
        a = math.radians(90 + 120 * k)
        F.extrude(f'flight{k}', [(0.0, 0.0), (0.0, 0.004), (0.017, 0.016), (0.03, 0.016), (0.036, 0.0)], 0.0008,
                  (0.0, 0.0046, -0.09), M.dart_flight, plane='side', rot=(0, math.degrees(a), 0))
    return done('dart')


CUE = 1.47


def cue():
    """A pool cue, 1.47 m, lying along the depth from its butt at the origin to its tip at the front: a wrapped
    butt, a joint, a tapering maple shaft, a white ferrule and a chalked tip (as the pool table's was)."""
    def along(t):
        return (0.0, 0.014, CUE * t)
    F.sweep('cue_butt', [along(0), along(0.3)], 0.014, M.cue_butt, verts=8, scale=[1.0, 0.86])
    F.sweep('cue_wrap', [along(0.06), along(0.2)], 0.0142, M.cue_wrap, verts=8, scale=[1.0, 0.95])
    F.sweep('cue_joint', [along(0.3), along(0.32)], 0.0122, M.pearl, verts=8)
    F.sweep('cue_shaft', [(0.0, 0.014, CUE * 0.32), (0.0, 0.014, CUE * 0.99)], 0.0118, M.cue, verts=8,
            scale=[1.0, 0.55])
    F.sweep('cue_ferrule', [(0.0, 0.014, CUE * 0.99), (0.0, 0.014, CUE * 0.997)], 0.0066, M.pearl, verts=8)
    F.sphere('cue_tip', 0.0064, (0.0, 0.014, CUE * 0.997), M.cue_tip, segments=8, rings=4)
    return done('cue')


def bat():
    """A ping-pong bat lying on its back, its blade to the front (+depth) and its handle at the origin, where it is
    held: a wooden blade 15 by 16 cm, red rubber on top and black underneath, a flared handle."""
    for name, y, h, m in (('rubber_back', 0.0, 0.002, M.rubber_black), ('blade', 0.002, 0.006, M.blade),
                          ('rubber', 0.008, 0.002, M.rubber)):
        disc(name, 0.15, h, (0.0, y, 0.122), m, verts=20, depth=0.158)
    F.sweep('handle', [(0.0, 0.005, 0.05), (0.0, 0.005, 0.0), (0.0, 0.005, -0.048)], None, M.blade,
            profile=G.rounded_rect(0.024, 0.016, 0.007, corner=2), up=(0, 0, 1), scale=[0.85, 1.0, 1.2])
    return done('bat')


def shaker():
    """A three-piece cocktail shaker, 22 cm high, the bar cart's (dining.py `barCart`, hidden while it is in a
    hand): a tapering chrome tin, the strainer and its cap."""
    lathe('shaker', [(0, 0), (0.034, 0), (0.036, 0.004), (0.04, 0.12), (0.042, 0.124), (0.037, 0.13), (0.03, 0.17),
                     (0.032, 0.174), (0.018, 0.2), (0.012, 0.212), (0.014, 0.215), (0, 0.22)], M.chrome, segments=16)
    return done('shaker')


# --- the food from town (review M-held-03): each in what it is sold in ------------------------------------------------
def iceCreamCup():
    """A paper cup of ice cream from the cart, 9 cm across at its top: pink-striped card, two scoops (strawberry on
    vanilla), each with a scoop's ragged lip, a flat wooden spoon stuck in."""
    lathe('cup', [(0, 0), (0.031, 0), (0.0315, 0.003), (0.045, 0.058), (0.0465, 0.06), (0.0435, 0.06), (0, 0.05)],
          [M.cup_paper, M.cup_paper, M.cup_stripe, M.cup_paper, M.cup_paper, M.vanilla], segments=18,
          wave=(9, {3: 0.02}))
    for i, (x, y, z, m, s) in enumerate(((-0.006, 0.05, 0.004, M.vanilla, 1.0), (0.008, 0.074, -0.004, M.strawberry,
                                                                                   0.85))):
        sc = lathe(f'scoop{i}', [(0, 0), (0.03 * s, 0), (0.034 * s, 0.006 * s), (0.032 * s, 0.022 * s),
                                 (0.02 * s, 0.038 * s), (0, 0.043 * s)], m,
                   at=(x, y, z), segments=10, wave=(5, {2: 0.12}), rot=(0, 0, 30 * i))
        G.jitter(sc, 0.0012, size=0.01, seed=2.3 + i)
    F.extrude('paddle', [(-0.004, 0.0), (0.004, 0.0), (0.0055, 0.06), (0.004, 0.075), (-0.004, 0.075),
                         (-0.0055, 0.06)], 0.0018, (0.024, 0.062, 0.014), M.paddle, plane='front', rot=(-12, 30, -28))
    return done('iceCreamCup')


def hotDog():
    """A hot dog in its bun on a paper boat, 19 cm long: the bun split along its top, the sausage in it longer than
    the bun, mustard piped along it in a zigzag and a line of ketchup."""
    def ring(y, hw, hd):
        return [(x, y, z) for x, z in rrect(hw, hd, n=12, square=6)]
    loft('boat', [ring(0.0, 0.092, 0.032), ring(0.022, 0.1, 0.04), ring(0.022, 0.097, 0.037), ring(0.004, 0.089,
                                                                                                    0.029)],
         M.cup_paper, angle=30)
    for s in (-1, 1):
        half = lathe(f'bun{s}', [(0, -0.075), (0.012, -0.072), (0.018, -0.055), (0.019, 0.0), (0.018, 0.055),
                                 (0.012, 0.072), (0, 0.075)], [M.bun] * 6, at=(0, 0.0, 0), segments=10,
                     rot=(0, 90, 0), scale=(0.9, 1.25))
        half.location = F.at(0.0, 0.022, s * 0.015)
        half.rotation_euler = (math.radians(s * 14), math.radians(90), 0)
    roll('sausage', 0.0115, 0.19, M.sausage, (0.0, 0.034, 0.0), segments=10, bend=-0.004)
    pts = [(-0.075 + 0.015 * k, 0.047, 0.006 * (1 if k % 2 else -1)) for k in range(11)]
    F.sweep('mustard', pts, 0.0028, M.mustard, verts=5, resolution=2)
    F.sweep('ketchup', [(-0.07, 0.0455, 0.0045), (0.0, 0.046, 0.0055), (0.07, 0.0455, 0.0045)], 0.0022, M.ketchup,
            verts=5, resolution=2)
    return done('hotDog')


def crisps():
    """A packet of crisps from the machine, 13 by 18 cm, lying on its back, its top torn open (+depth) and the
    silvered inside showing, a few crisps spilt out in front: blue with a yellow band, crimped seals."""
    F.soft('packet', 0.13, 0.026, 0.16, (0, 0.013, -0.01), M.packet, round=0.01, puff=0.008, flat=2, round_segments=1)
    F.soft('band', 0.132, 0.03, 0.04, (0, 0.013, -0.02), M.packet_band, round=0.01, puff=0.0085, flat=2,
           round_segments=1)
    for z in (-0.095, 0.075):
        box(f'seal{z}', 0.128, 0.004, 0.014, (0, 0.016, z), M.packet if z < 0 else M.foil, bevel=0.001)
    lathe('mouth', [(0, 0), (0.06, 0), (0.06, 0.006), (0, 0.006)], M.foil, at=(0, 0.024, 0.068), segments=12,
          scale=(1, 0.22), rot=(-70, 0, 0))
    for i, (x, z, a) in enumerate(((-0.03, 0.095, 10), (0.01, 0.1, 50), (0.035, 0.09, 80), (0.0, 0.08, 130),
                                   (-0.012, 0.106, 170))):
        coin(f'crisp{i}', 0.03, 0.0015, (x, 0.004 + 0.012 * (i % 2), z), M.crisp, segments=8, cup=1.6,
             tilt=(a / 6, a / 8))
    return done('crisps')


def apple():
    """An apple from the stall, 7.5 cm across: red with a yellow blush, a dimple at the stalk and the eye, its
    stalk and a leaf."""
    a = lathe('apple', [(0, 0.006), (0.014, 0.0), (0.03, 0.01), (0.037, 0.03), (0.0365, 0.045), (0.031, 0.06),
                        (0.016, 0.068), (0.006, 0.063), (0, 0.058)],
              [M.apple_blush, M.apple, M.apple, M.apple, M.apple, M.apple, M.apple_blush, M.apple_blush], segments=14,
              wave=(5, {2: 0.025, 3: 0.02}))
    G.jitter(a, 0.0008, size=0.02, seed=1.7)
    F.sweep('stalk', [(0.0, 0.06, 0.0), (0.002, 0.072, 0.001), (0.005, 0.08, 0.003)], 0.0017, M.stalk, verts=5)
    leaf('leaf', (0.012, 0.072, 0.004), 0.034, 0.016, 30, M.leaf, tilt=-20)
    return done('apple')


def pastryBag():
    """A pastry from the bakery in a white paper bag, 12 cm wide and 17 cm high, its top folded open and a croissant
    poking out of it."""
    rings = [_bag_ring(0.0, 0.06, 0.03, 0.016), _bag_ring(0.13, 0.061, 0.031, 0.016),
             _bag_ring(0.155, 0.064, 0.036, 0.008), _bag_ring(0.165, 0.06, 0.03, 0.004)]
    G.loft('bag', [[F.at(*p) for p in r] for r in rings], material=M.bag_white, angle=30)
    # the croissant: a crescent of lobes standing up out of the bag, its tips curling down to the front
    for i, t in enumerate((-1.0, -0.5, 0.0, 0.5, 1.0)):
        r = 0.022 * (1 - 0.35 * abs(t))
        lump(f'lobe{i}', r, (0.032 * t, 0.12 + 0.03 * (1 - t * t), 0.008 * t * t), M.pastry_baked,
             scale=(0.8, 1.1, 0.95), seed=4.0 + i, noise=0.12)
    return done('pastryBag')


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
        # props in hand (G-70: actions.ts `HAND_PROPS`)
        I('mug', mug, (0.11, 0.095, 0.081), 'tool.mug', centre=(0.0142, 0.0), code=True),
        I('glass', glass, (0.071, 0.111, 0.07), 'tool.glass', code=True),
        I('wineGlass', wineGlass, (0.083, 0.19, 0.082), 'tool.wineGlass', code=True),
        I('can', can, (0.066, 0.12, 0.065), 'tool.can', code=True),
        I('fork', fork, (0.023, 0.0024, 0.171), 'tool.fork', centre=(0.0, 0.0255), code=True),
        I('spoon', spoon, (0.032, 0.011, 0.167), 'tool.spoon', centre=(0.0, 0.0235), code=True),
        I('chopsticks', chopsticks, (0.026, 0.007, 0.23), 'tool.chopsticks', centre=(0.0, 0.055), code=True),
        I('phone', phone, (0.072, 0.0099, 0.148), 'tool.phone', code=True),
        I('book', book, (0.304, 0.032, 0.224), 'tool.book', code=True),
        I('coin', fountainCoin, (0.023, 0.0022, 0.023), 'tool.coin', code=True),
        I('seedBag', seedBag, (0.086, 0.119, 0.056), 'tool.seedBag', code=True),
        I('dart', dart, (0.024, 0.028, 0.154), 'tool.dart', centre=(-0.0038, -0.013), code=True),
        I('cue', cue, (0.028, 0.028, 1.472), 'tool.cue', centre=(0.0, 0.736), code=True),
        I('bat', bat, (0.15, 0.029, 0.249), 'tool.bat', centre=(0.0, 0.0765), code=True),
        I('shaker', shaker, (0.084, 0.22, 0.084), 'tool.shaker', code=True),
        # the food from town (review M-held-03)
        I('iceCreamCup', iceCreamCup, (0.093, 0.128, 0.092), 'item.iceCreamCup'),
        I('hotDog', hotDog, (0.2, 0.05, 0.098), 'item.hotDog'),
        I('crisps', crisps, (0.132, 0.049, 0.223), 'item.crisps', centre=(0.0, 0.0093)),
        I('apple', apple, (0.074, 0.081, 0.073), 'item.apple'),
        I('pastryBag', pastryBag, (0.128, 0.198, 0.072), 'item.pastryBag'),
    ],
    comment='Blender-built things residents carry and cook with (tools/art/blender/families/held.py): the '
            'models of content `items`, the props drawn on a station while cooking and those in the hands '
            'while a thing is used. Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes, so each costs one draw call '
            '(the wooden boards two).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
