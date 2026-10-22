# The play and yard things (docs/objects/outdoor.md OU4). Notes: yard.md.
#
# The sandpit (`model.sandbox`), whose sand castle grows while one is built (`castle`); the bike rack
# (`model.bikeRack`), whose bike lifts its back wheel onto the stand (`wheel_up`) and turns it (`wheel_turn`, a loop)
# for a tune-up; the raised vegetable bed (`model.raisedBed`), whose crop follows its growth stage (`grow`), rustles
# while tended and is picked away for the harvest; the garden shed (`model.shedSmall`) and its door (`door_open`);
# the greenhouse (`model.greenhouse`), whose door and roof vent open (`door_open`); the compost bin
# (`model.compostBin`) and its lid (`lid_open`); the rain barrel (`model.rainBarrel`); the washing line
# (`model.clothesline`), whose washing is pegged out (`washing`) and stays out until taken in (`open`); the log
# store (`model.firewoodStack`).
#
# Each is built in place where the game places it (`F`, game axes), and each has a far version for the town
# overview (`far=`).
#
# Run:  cargo art yard [--only sandbox] [--preview]
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import low  # noqa: E402

M = G.Palette(
    # the sandpit
    frame=('#C9A27A', 'wood'),           # its boards; the main part
    sand=('#E9D5A8', 'matte'),
    sand_damp=('#D8BF8E', 'matte'),      # the castle, moulded from damp sand
    bucket=('#D9483B', 'satin'),
    spade=('#3E8EDE', 'satin'),
    flag=('#F2C14E', 'satin'),
    # the bike rack
    rack=('#5C6168', 'metal'),           # galvanised hoops and the stand
    bike=('#3E8EDE', 'gloss'),           # the bike's frame
    tyre=('#2A2C30', 'satin'),
    rim=('#C9CBCC', 'chrome'),           # rims, hubs, bars, the chain ring
    saddle=('#3A3530', 'satin'),
    # the raised bed
    bed=('#9A7452', 'satin'),            # its boards; the main part
    bed_post=('#7A5A40', 'satin'),
    soil=('#6A4C38', 'matte'),
    lettuce=('#8CC05A', 'foliage'),
    lettuce_dark=('#6FA34A', 'foliage'),
    leaf=('#4F7A3E', 'foliage'),         # tomato plants
    carrot_top=('#5E8F4E', 'foliage'),
    tomato=('#D9483B', 'satin'),
    carrot=('#E07A2E', 'satin'),
    # the shed
    shed=('#8FA79A', 'wood'),            # its weatherboards; the main part
    roof=('#5A4A3E', 'matte'),           # roofing felt
    trim=('#F2F0EA', 'satin'),           # corner boards, barge boards, the door and window frames
    shed_door=('#6F8B7E', 'satin'),      # the painted door
    iron=('#2B2C2E', 'metal'),           # hinges, latch, the log store's rack
    pane=('#3A4A52', 'gloss'),           # the window, dark from outside (opaque)
    inside=('#4A4038', 'matte'),         # the shed's inside, in shadow
    plinth=('#6E6A64', 'matte'),
    box_wood=('#8E6E52', 'satin'),       # the window box, shelves, handles
    flower_pink=('#E07BC3', 'satin'),
    flower_yellow=('#F2C14E', 'satin'),
    flower_red=('#D9483B', 'satin'),
    flower_lilac=('#9B7FD1', 'satin'),
    green=('#5E8F4E', 'satin'),          # plain leaves where a foliage draw would be one too many
    terra=('#C26E4A', 'satin'),          # flower pots
    # the greenhouse
    gh_frame=('#F2F2F0', 'satin'),       # its white frame; the main part
    gh_wall=('#B5A48E', 'satin'),        # the dwarf wall
    gravel=('#B8A88E', 'matte'),
    paving=('#C9C2B4', 'matte'),
    staging=('#9C7A58', 'satin'),        # the slatted benches
    mortar=('#D8D0C2', 'matte'),         # the dwarf wall's courses
    glass=('#DCEBEA', 'glass'),
    glass_bar=('#FFFFFF', 'glass'),      # the door's and the vent's edges: glass too, so they stay one draw each
    # the compost bin
    bin=('#8E6E52', 'wood'),             # its slats and lid; the main part
    compost=('#4A3A2C', 'matte'),
    peel=('#E07A2E', 'satin'),
    greens=('#7FA35A', 'satin'),
    shell=('#EDE6D6', 'satin'),          # eggshells on the heap
    # the rain barrel
    barrel=('#3F6E4A', 'satin'),         # the main part
    barrel_dark=('#35603F', 'satin'),
    brass=('#B8925A', 'metal'),
    can=('#9EA7AD', 'metal'),            # a galvanised watering can
    # the washing line
    posts=('#B9BCBE', 'metal'),
    line=('#F2F2F0', 'satin'),
    basket=('#C9A979', 'fabric'),        # wicker
    washing=('#9DB7C9', 'fabric'),       # the sheet; the washing's main colour
    towel=('#9DC6CF', 'fabric'),
    white=('#F2F0EA', 'fabric'),
    pink=('#E4B8AE', 'fabric'),
    denim=('#3E5470', 'fabric'),
    yellow=('#F2C14E', 'fabric'),
    red=('#D9483B', 'fabric'),
    sage=('#9FB59A', 'fabric'),
    peg=('#C08A52', 'fabric'),           # pegs: fabric too, so the washing stays one draw
    # the log store
    bark=('#8E6A4A', 'bark'),
    bark_dark=('#7A5A3E', 'bark'),
    bark_light=('#9C7A58', 'bark'),
    end_grain=('#D9C3A0', 'matte'),
    end_grain_dark=('#C9AE86', 'matte'),
    axe=('#9A9EA2', 'metal'),
    haft=('#B08E66', 'satin'),
)

F = G.GameFrame()
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _v(p):
    """A game position or direction as a Blender vector."""
    return Vector(F.at(*p))


def _board(name, a, b, w, t, material, face=(0, 1, 0), bevel=0.006):
    """A board from game point `a` to `b`, `w` wide and `t` thick, its broad face turned towards game direction
    `face` (outdoor.py's): rafters, braces, legs at any angle."""
    a, b = _v(a), _v(b)
    z = (b - a).normalized()
    x = _v(face).cross(z).normalized()
    y = z.cross(x)
    obj = G.cube(name, (w, t, (b - a).length), material=material, bevel=bevel)
    obj.matrix_world = Matrix.Translation((a + b) / 2) @ Matrix((x, y, z)).transposed().to_4x4()
    return obj


def _rod(name, a, b, r, material, verts=10):
    """A round bar from game point `a` to `b`."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=material, verts=verts)


def _arc_points(centre, r, a0, a1, n, plane='xy'):
    """Game points round an arc of radius `r` about game point `centre`, from `a0` to `a1` degrees, in the game's
    x-height plane ('xy', 0° along +x, 90° up) or its x-depth plane ('xz')."""
    cx, cy, cz = centre
    out = []
    for k in range(n + 1):
        a = a0 + (a1 - a0) * k / n
        c, s = G.cosd(a), G.sind(a)
        out.append((cx + r * c, cy + r * s, cz) if plane == 'xy' else (cx + r * c, cy, cz + r * s))
    return out


def _prism(name, pts, z0, z1, material):
    """A slab: the polygon `pts` [(game x, height)] between game depths `z0` and `z1` (gables, a roof's ends)."""
    n = len(pts)
    verts = [F.at(x, y, z1) for x, y in pts] + [F.at(x, y, z0) for x, y in pts]
    faces = [tuple(range(n)), tuple(range(n, 2 * n))] + [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    return obj


def _blob(x, y, z, r, material, seed, scale=(1, 1, 1), noise=0.4):
    """A lumpy clump standing on height `y` (a lettuce, a bush, a heap)."""
    return G.boulder(_name('blob'), r, at=F.at(x, y, z), material=material, seed=seed, noise=noise,
                     subdivisions=low(2, 1), scale=scale)


def _pot(x, y, z, r, h, material=None):
    """A flower pot standing on height `y`, `r` across the rim, with soil in it."""
    G.revolve(_name('pot'), [(r * 0.72, 0), (r, h * 0.86), (r * 1.06, h * 0.88), (r * 1.06, h), (r * 0.9, h),
                             (r * 0.88, h * 0.86), (0, h * 0.86)],
              at=F.at(x, y, z), material=material or M.terra, segments=low(12, 6))


def _bent(name, pts, r, material, bend=0.03, verts=8, steps=3):
    """One length of round bar or tube through game points, its corners bent round about `bend` (kit.fillet: no
    overshoot): hoops, handles, frames."""
    return G.kit.tube(F, name, G.kit.fillet(pts, bend, steps), r, material, verts=verts, round=1)


def _side_slab(name, x, outline, t, material, bevel=0.0):
    """A board seen from the side at game x, `t` thick across: its outline [(depth, height), ...]."""
    return F.extrude(name, outline, t, (x, 0, 0), material, plane='side', bevel=bevel, bevel_segments=1)


def _ground(name, w, d, y, material, n=(10, 10), bump=0.01, seed=0.0, at=(0.0, 0.0), edge=0.0):
    """A ground surface `w` by `d` at height `y` (sand, soil): a grid raised in soft lumps up to `bump` (noise, the
    same every run), `edge` metres in from its rim falling to `y`; closed underneath, 3 cm deep."""
    from mathutils import noise
    nx, nz = n
    top = []
    for i in range(nx + 1):
        for j in range(nz + 1):
            u, v = -w / 2 + w * i / nx, -d / 2 + d * j / nz
            k = 1.0
            if edge:
                k = min(1.0, (w / 2 - abs(u)) / edge, (d / 2 - abs(v)) / edge)
            h = bump * k * (0.5 + 0.5 * noise.noise(Vector((u * 4 + seed, v * 4 - seed, seed))))
            top.append(F.at(at[0] + u, y + h, at[1] + v))
    bot = [(p[0], p[1], y - 0.03) for p in top]
    vs = top + bot
    m = (nz + 1)
    faces = []
    off = len(top)
    for i in range(nx):
        for j in range(nz):
            a, b, c, e = i * m + j, (i + 1) * m + j, (i + 1) * m + j + 1, i * m + j + 1
            faces.append((a, b, c, e))
            faces.append((off + e, off + c, off + b, off + a))
    rim = [i * m for i in range(nx + 1)] + [nx * m + j for j in range(1, nz + 1)] + \
        [i * m + nz for i in range(nx - 1, -1, -1)] + [j for j in range(nz - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        faces.append((a, b, off + b, off + a))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(p) for p in vs], [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    G.shapes._recalc_normals(obj)
    for p in obj.data.polygons:
        p.use_smooth = True
    return obj


# --- the sandpit (2x2): a box of boards round the sand, two corner seats, a bucket and spade; a castle grows ---
SAND = 0.18          # the sand's top
CASTLE = (-0.32, 0.18)   # where the castle is built (x, depth)


def _star(r0, r1, n=5, turn=90):
    """A star's outline: `n` points out to r1, in to r0 between them."""
    return [((r1 if k % 2 == 0 else r0) * G.cosd(turn + 180 * k / n), (r1 if k % 2 == 0 else r0) * G.sind(turn + 180 * k / n))
            for k in range(2 * n)]


def sandbox():
    with G.new_parts():
        # two boards high each side, lapped at the corners; a mitred capping frame round the top to sit on
        for y in (0.065, 0.195):
            for z in (-0.92, 0.92):
                F.box(_name('board'), 1.92, 0.125, 0.08, (0, y, z), M.frame, bevel=low(0.01, 0), bevel_segments=1)
            for x in (-0.92, 0.92):
                F.box(_name('board'), 0.08, 0.125, 1.76, (x, y, 0), M.frame, bevel=low(0.01, 0), bevel_segments=1)
        G.kit.moulding(F, 'cap', [(0.99, 0.99), (-0.99, 0.99), (-0.99, -0.99), (0.99, -0.99)],
                       [(0.0, 0.0), (0.0, 0.024), (0.006, 0.03), (0.124, 0.03), (0.13, 0.024), (0.13, 0.0)], M.frame,
                       at=0.26, plane='top')
        for sx in (-1, 1):                                 # corner seats at the back: triangles across the corners
            tri = [(sx * 0.86, -0.86), (sx * 0.86 - sx * 0.36, -0.86), (sx * 0.86, -0.86 + 0.36)]
            F.extrude(_name('seat'), tri, 0.03, (0, 0.3, 0), M.frame, plane='top', bevel=low(0.008, 0), bevel_segments=1)
        _ground('sand', 1.76, 1.76, SAND - 0.012, M.sand, n=low((12, 12), (3, 3)), bump=0.014, seed=2.0, edge=0.12)
        _blob(0.42, SAND - 0.03, -0.38, 0.2, M.sand, 3.0, scale=(1.2, 1, 0.4), noise=0.3)     # a heap
        _blob(-0.55, SAND - 0.02, -0.5, 0.12, M.sand, 5.0, scale=(1, 1, 0.35), noise=0.3)
        # the bucket, full of sand, its handle down; the spade lying beside it
        bx, bz = 0.3, 0.32
        px, pz, _ = F.at(bx, 0, bz)
        rings = [(0.0, 0.075), (0.15, 0.093), (0.15, 0.104), (0.17, 0.104), (0.17, 0.097), (0.16, 0.096)]
        G.loft('bucket', [G.ring(r, r, SAND + h, low(16, 8), at=(px, pz)) for h, r in rings], material=M.bucket)
        F.cyl('bucket_sand', 0.096, 0.01, (bx, SAND + 0.163, bz), M.sand, verts=low(16, 8))
        G.revolve('bucket_mound', [(0.0, 0.0), (0.09, 0.0), (0.06, 0.008), (0.0, 0.011)], at=F.at(bx, SAND + 0.165, bz),
                  material=M.sand, segments=low(10, 5))
        if not G.is_low():
            arc = [(bx + 0.1 * G.cosd(a), SAND + 0.165 - 0.06 * G.sind(a) * 0.3, bz + 0.1 * G.sind(a) * 0.95) for a in range(0, 181, 30)]
            F.sweep('bucket_handle', arc, 0.004, M.spade, verts=4)
        # the spade: a T grip, its shaft and a blade with a rounded tip, lying on the sand
        th = 18
        d = (G.sind(th), G.cosd(th))
        g0 = (0.5, 0.0)
        g1 = (g0[0] + 0.25 * d[0], g0[1] + 0.25 * d[1])
        blade = [(-0.055, 0.0), (0.055, 0.0), (0.055, 0.08)] + [(0.055 * G.cosd(a), 0.08 + 0.05 * G.sind(a)) for a in range(30, 151, 30)] \
            + [(-0.055, 0.08)]
        F.extrude('spade_blade', blade, 0.008, (g1[0], SAND + 0.006, g1[1]), M.spade, plane='top', rot=(0, 0, -th))
        F.box('spade_handle', 0.025, 0.02, 0.25, ((g0[0] + g1[0]) / 2, SAND + 0.012, (g0[1] + g1[1]) / 2), M.spade,
              rot=(0, 0, -th), bevel=0.006, bevel_segments=1)
        F.box('spade_grip', 0.09, 0.022, 0.025, (g0[0], SAND + 0.012, g0[1]), M.spade, rot=(0, 0, -th), bevel=0.008,
              bevel_segments=1)
        # a star mould turned out, and the mould lying on its side
        F.extrude('mould_turned', _star(0.03, 0.055), 0.045, (-0.62, SAND + 0.0225, 0.55), M.sand_damp, plane='top',
                  bevel=low(0.006, 0), bevel_segments=1)
        F.extrude('mould', _star(0.032, 0.058), 0.05, (-0.4, SAND + 0.025, 0.62), M.bucket, plane='top', rot=(0, 0, 15),
                  bevel=low(0.004, 0), bevel_segments=1)
    if G.is_low():
        return G.settle([G.join_all('sandbox')], centre=False)
    G.join(G.meshes(), 'sandbox', origin=(0, 0, 0))
    # The castle: a keep and four turrets turned out of the bucket, walls between them, a flag. Its origin is 2 cm
    # down in the sand, so flattened at rest it is out of sight; `castle` raises it.
    cx, cz = CASTLE
    with G.new_parts() as castle:
        F.cone('keep', 0.1, 0.075, 0.17, (cx, SAND + 0.085, cz), M.sand_damp, verts=16)
        for k in range(8):
            a = 45 * k
            F.box(_name('merlon'), 0.03, 0.03, 0.025, (cx + 0.062 * G.cosd(a), SAND + 0.183, cz + 0.062 * G.sind(a)),
                  M.sand_damp, rot=(0, 0, a))
        F.cyl('keep_top', 0.06, 0.012, (cx, SAND + 0.172, cz), M.sand_damp, verts=12)
        for k in range(3):                                 # the bucket's ridges, pressed into the keep
            F.cyl(_name('keep_ring'), 0.1 - 0.025 * (k + 0.5) / 3 * 1.0 + 0.003, 0.008, (cx, SAND + 0.03 + 0.05 * k, cz), M.sand_damp,
                  verts=16)
        F.box('keep_door', 0.04, 0.05, 0.02, (cx, SAND + 0.025, cz + 0.092), M.sand, bevel=0.006, bevel_segments=1)
        for sx in (-1, 1):
            for sz in (-1, 1):
                x, z = cx + sx * 0.15, cz + sz * 0.15
                F.cone(_name('turret'), 0.055, 0.042, 0.11, (x, SAND + 0.055, z), M.sand_damp, verts=12)
                F.cone(_name('turret_top'), 0.046, 0.0, 0.06, (x, SAND + 0.14, z), M.sand_damp, verts=12)
        for s in (-1, 1):
            F.box(_name('wall'), 0.24, 0.06, 0.03, (cx, SAND + 0.03, cz + s * 0.15), M.sand_damp, bevel=0.006)
            F.box(_name('wall'), 0.03, 0.06, 0.24, (cx + s * 0.15, SAND + 0.03, cz), M.sand_damp, bevel=0.006)
        F.box('gate', 0.07, 0.045, 0.01, (cx, SAND + 0.023, cz + 0.166), M.sand, bevel=0.003)
        _rod('flag_pole', (cx, SAND + 0.17, cz), (cx, SAND + 0.31, cz), 0.004, M.haft, verts=5)
        _prism('flag', [(cx, SAND + 0.3), (cx, SAND + 0.25), (cx + 0.08, SAND + 0.275)], cz - 0.002, cz + 0.002, M.flag)
    castle = G.join(castle, 'castle', origin=F.at(cx, SAND - 0.02, cz))
    castle.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(castle, 'castle', seconds=2.5)
    return objs


# --- the bike rack (2x1): four galvanised hoops, a bike parked in front; on a tune-up its back wheel lifts ------
WHEEL_R = 0.31
REAR, FRONT = -0.52, 0.5      # the axles, game x
BZ = 0.0                      # the bike's middle, depth
RACK_Z = -0.3
LIFT = 8.5                    # degrees the bike tips up about the front tyre to rest its back axle on the stand


def _wheel(x, z):
    """A wheel at axle (x, WHEEL_R, z): a treaded tyre, a rim, a hub with its flanges and laced spokes, in the game's
    x-height plane."""
    y = WHEEL_R
    G.torus(_name('tyre'), WHEEL_R - 0.02, 0.02, at=F.at(x, y, z), material=M.tyre, rot=(90, 0, 0),
            major_segments=low(32, 10), minor_segments=low(6, 3))
    if not G.is_low():
        G.torus(_name('rim'), WHEEL_R - 0.043, 0.009, at=F.at(x, y, z), material=M.rim, rot=(90, 0, 0),
                major_segments=32, minor_segments=4)
        G.revolve(_name('hub'), [(0.0, -0.05), (0.008, -0.05), (0.008, -0.035), (0.024, -0.033), (0.024, -0.026),
                                 (0.013, -0.022), (0.013, 0.022), (0.024, 0.026), (0.024, 0.033), (0.008, 0.035),
                                 (0.008, 0.05), (0.0, 0.05)], at=F.at(x, y, z), material=M.rim, segments=8, rot=(90, 0, 0))
        for k in range(16):                                # laced: each spoke leaves its flange tangentially
            a = 22.5 * k
            side = 0.03 if k % 2 else -0.03
            lead = 12 if k % 2 else -12
            _rod(_name('spoke'), (x + 0.022 * G.cosd(a + lead), y + 0.022 * G.sind(a + lead), z + side),
                 (x + (WHEEL_R - 0.05) * G.cosd(a), y + (WHEEL_R - 0.05) * G.sind(a), z), 0.0022, M.rim, verts=3)
    else:
        F.cyl(_name('hub'), 0.022, 0.1, (x, y, z), M.rim, rot=(90, 0, 0), verts=5)


def _bike():
    """The bike, parked along x: its frame, front wheel, bars, saddle, cranks (the back wheel is the caller's)."""
    ra, fa = (REAR, WHEEL_R), (FRONT, WHEEL_R)
    bb, sc, ht, hb = (-0.1, 0.27), (-0.24, 0.66), (0.3, 0.68), (0.34, 0.56)

    def at(p, dz=0.0):
        return (p[0], p[1], BZ + dz)
    # the main triangle, the tubes tapering a little; a head tube; the stays in pairs
    for k, (a, b, r) in enumerate(((bb, sc, 0.018), (sc, ht, 0.016), (bb, hb, 0.02))):
        F.sweep(_name('tube'), [at(a), at(b)], r, M.bike, verts=low(10, 5), scale=[1.0, 0.85])
    F.sweep('head_tube', [at((0.345, 0.55)), at((0.295, 0.7))], 0.022, M.bike, verts=low(10, 5))
    for dz in (-0.035, 0.035):
        F.sweep(_name('chainstay'), [at(ra, dz), at(bb, dz * 0.4)], 0.011, M.bike, verts=low(6, 4), scale=[0.8, 1.0])
        F.sweep(_name('seatstay'), [at(ra, dz), at(((ra[0] + sc[0]) / 2, (ra[1] + sc[1]) / 2 + 0.01), dz * 0.6), at(sc, dz * 0.25)],
                0.009, M.bike, verts=low(6, 4), resolution=low(2, 1))
        # the fork's blades, raked forward to the axle
        F.sweep(_name('fork'), [at(hb, dz * 0.5), at((0.4, 0.42), dz * 0.9), at(fa, dz)], 0.011, M.bike, verts=low(6, 4),
                resolution=low(3, 1), scale=[1.1, 1.0, 0.8])
    _wheel(FRONT, BZ)
    # mudguards: a C-section arc over each wheel, on stays
    for x, a0, a1 in ((FRONT, 20, 120), (REAR, 60, 160)):
        pts = _arc_points((x, WHEEL_R, BZ), WHEEL_R + 0.03, a0, a1, low(10, 4))
        F.sweep(_name('guard'), pts, None, M.bike, profile=[(-0.025, 0.0), (-0.02, 0.008), (0.0, 0.01), (0.02, 0.008), (0.025, 0.0),
                                                           (0.0, 0.004)], up=(0, 1, 0))
    # seat post and a saddle; stem and swept-back bars with grips; a bell
    _rod('seat_post', at(sc), at((-0.27, 0.78)), 0.012, M.rim, verts=8)
    sad = [(0.0, 0.03, 0.02), (0.03, 0.08, 0.035), (0.08, 0.075, 0.035), (0.14, 0.035, 0.03), (0.2, 0.015, 0.022),
           (0.23, 0.008, 0.012)]
    G.loft('saddle', [[F.at(-0.38 + u, 0.785 + th * (0.5 + 0.5 * G.sind(a)), BZ + hw * G.cosd(a))
                       for a in range(0, 360, low(45, 90))] for u, hw, th in sad], material=M.saddle)
    _rod('stem', at(ht), at((0.27, 0.8)), 0.013, M.rim, verts=8)
    _bent('bars', [(0.21, 0.81, BZ - 0.2), (0.27, 0.8, BZ - 0.13), (0.27, 0.8, BZ + 0.13), (0.21, 0.81, BZ + 0.2)], 0.011, M.rim,
          bend=0.05, verts=low(8, 5))
    for s in (-1, 1):
        _rod(_name('grip'), (0.222, 0.808, BZ + s * 0.185), (0.206, 0.81, BZ + s * 0.211), 0.016, M.saddle, verts=low(8, 5))
    F.cyl('bell', 0.02, 0.015, (0.27, 0.82, BZ + 0.12), M.rim, verts=8)
    # chain ring, cranks and pedals on the drive side (the front), the chain to the back cog
    G.torus('chainring', 0.075, 0.006, at=F.at(bb[0], bb[1], BZ + 0.05), material=M.rim, rot=(90, 0, 0),
            major_segments=low(24, 8), minor_segments=4)
    F.cyl('cog', 0.035, 0.008, (REAR, WHEEL_R, BZ + 0.05), M.rim, rot=(90, 0, 0), verts=low(10, 5))
    for s in (-1, 1):
        a = (0.1 * s, -0.13 * s)
        dz = 0.06 * s
        _rod(_name('crank'), at(bb, dz), (bb[0] + a[0], bb[1] + a[1], BZ + dz), 0.009, M.rim, verts=6)
        F.box(_name('pedal'), 0.08, 0.02, 0.06, (bb[0] + a[0], bb[1] + a[1], BZ + dz + 0.04 * s), M.tyre, bevel=0.005)
    for dy in (0.075, -0.075)[:low(2, 0)]:
        _rod(_name('chain'), (bb[0], bb[1] + dy, BZ + 0.05), (REAR, WHEEL_R + dy * 0.47, BZ + 0.05), 0.004, M.tyre,
             verts=4)
    # a kickstand, folded up along the chainstay
    _rod('kickstand', (-0.14, 0.25, BZ - 0.05), (-0.42, 0.24, BZ - 0.06), 0.008, M.rim, verts=6)


def _rear_axle_lifted():
    """Where the back axle is once the bike tips up by LIFT about the front tyre's foot (game x, height)."""
    dx, dy = REAR - FRONT, WHEEL_R
    c, s = G.cosd(LIFT), G.sind(LIFT)
    return FRONT + dx * c + dy * s, -dx * s + dy * c


def bikeRack():
    with G.new_parts() as still:
        F.box('rail', 1.8, 0.05, 0.12, (0, 0.025, RACK_Z), M.rack, bevel=0.01, bevel_segments=1)
        for x in (-0.6, -0.2, 0.2, 0.6):                   # inverted-U hoops on the rail, each one bent tube, on its feet
            _bent(_name('hoop'), [(x - 0.15, 0.04, RACK_Z), (x - 0.15, 0.57, RACK_Z), (x + 0.15, 0.57, RACK_Z),
                                  (x + 0.15, 0.04, RACK_Z)], 0.02, M.rack, bend=0.15, verts=low(10, 6), steps=low(5, 2))
            for e in (-1, 1):
                F.box(_name('hoop_foot'), 0.07, 0.008, 0.1, (x + e * 0.15, 0.054, RACK_Z), M.rack, bevel=low(0.003, 0),
                      bevel_segments=1)
        # the repair stand behind the back wheel: two legs either side of it, a cradle at the lifted axle
        ax, ay = _rear_axle_lifted()
        for dz in (-0.075, 0.075):
            z = BZ + dz
            for e in (-1, 1):
                _rod(_name('stand_leg'), (ax + e * 0.16, 0.0, z + dz * 0.8), (ax, ay - 0.03, z), 0.012, M.rack, verts=6)
            F.sweep(_name('cradle'), _arc_points((ax, ay, z), 0.022, 180, 360, 4), 0.008, M.rack, verts=4)
            for e in (-1, 1):
                F.cyl(_name('stand_foot'), 0.018, 0.012, (ax + e * 0.16, 0.006, z + dz * 0.8), M.tyre, verts=6)
    if not G.is_low():
        G.join(still, 'rack', origin=(0, 0, 0))
    with G.new_parts() as bike:
        _bike()
    with G.new_parts() as wheel:
        _wheel(REAR, BZ)
    if G.is_low():
        return G.settle([G.join_all('bikeRack')], centre=False)
    bike = G.hinged(G.join(bike, 'bike'), F.at(FRONT, 0, BZ))
    wheel = G.hinged(G.join(wheel, 'wheel_rear'), F.at(REAR, WHEEL_R, BZ))
    objs = G.settle(centre=False)
    G.attach(wheel, bike)
    G.swing(bike, 'wheel_up', -LIFT, seconds=0.8, axis='Y')
    G.spin(wheel, 'wheel_turn', turns=1, seconds=1.2, axis='Y')
    return objs


# --- the raised vegetable bed (2x1): boards round the soil, lettuces, carrots and tomato plants --------------
BED_TOP = 0.32       # the soil's top
# The growth stages (`grow`, G.stages, as plants.py's edibles): ripe at rest; picked, the plants pulled and the
# crop gone; growing, the plants lower, no crop; planted, seedlings. The plants flatten and sink into the soil
# (shares of their height), so each seedling comes up where its plant stands.
PLANTS = dict(picked=(1, 1, 0.3), growing=(1, 1, 0.75), planted=(1, 1, 0.4))
SINK = (0.32, 0.03, 0.1)
CROP = dict(picked=0.001, growing=0.001, planted=0.001)


def _lettuce(name, x, y, z, r, h, material, seed):
    """A lettuce standing on height `y`: its outer leaves ruffled round a cupped heart, lofted in one."""
    bx, by, bz = F.at(x, y, z)
    n = low(14, 6)
    rows = [(0.0, 0.6, 0.05), (0.12, 1.0, 0.16), (0.38, 0.92, 0.12), (0.66, 0.62, 0.07), (0.8, 0.32, 0.02), (0.7, 0.0, 0.0)]
    secs = []
    for t, k, wave in rows:
        loop = []
        for i in range(n):
            a = math.tau * i / n
            w = 1 + wave * math.sin(5 * a + seed)
            loop.append((bx + r * k * w * math.cos(a), bz + r * k * w * math.sin(a), by + h * t + h * wave * 0.4 * math.cos(3 * a + seed)))
        secs.append(loop)
    return G.loft(name, secs, material=material, angle=70)


def _frond(name, x, y, z, a, h, material, lean=0.22):
    """A carrot's frond from game (x, y, z): a feathery leaf `h` tall, leaning out along `a` degrees."""
    out = [(0.0, 0.0)]
    steps = low(3, 1)
    for i in range(1, steps + 1):
        v = h * i / steps
        w = 0.016 * math.sin(math.pi * i / (steps + 1))
        out += [(w, v - h / steps * 0.5), (w * 0.4, v)]
    out += [(-u, v) for u, v in reversed(out[1:])]
    leaf = F.extrude(name, out, 0.003, (x, y, z), material)
    leaf.matrix_world = Matrix.Translation(F.at(x, y, z)) @ Matrix.Rotation(math.radians(a - 90), 4, 'Z') @ \
        Matrix.Rotation(lean, 4, 'X')
    return leaf


def raisedBed():
    rnd = random.Random(7)
    with G.new_parts() as still:
        for z in (-0.4, 0.4):
            for y in (0.085, 0.255):
                F.box(_name('board'), 1.84, 0.162, 0.06, (0, y, z), M.bed, bevel=low(0.01, 0), bevel_segments=1)
        for x in (-0.89, 0.89):
            for y in (0.085, 0.255):
                F.box(_name('board'), 0.06, 0.162, 0.74, (x, y, 0), M.bed, bevel=low(0.01, 0), bevel_segments=1)
            for z in (-0.395, 0.395):
                F.box(_name('post'), 0.09, 0.36, 0.09, (x * 0.995, 0.18, z), M.bed_post, bevel=low(0.01, 0), bevel_segments=1)
                if not G.is_low():                         # a pyramid cap on each post
                    G.cone(_name('post_cap'), 0.064, 0.0, 0.024, at=F.at(x * 0.995, 0.372, z), material=M.bed_post, verts=4,
                           rot=(0, 0, 45))
        _ground('soil', 1.74, 0.76, BED_TOP - 0.005, M.soil, n=low((14, 6), (2, 1)), bump=0.012, seed=5.0)
        F.box('label', 0.07, 0.05, 0.006, (-0.8, BED_TOP + 0.1, 0.34), M.trim, bevel=0.002, bevel_segments=1)
        F.box('label_stick', 0.012, 0.12, 0.004, (-0.8, BED_TOP + 0.02, 0.34), M.trim)
        for k, x in enumerate((-0.45, 0.0, 0.45)):         # the tomatoes' canes
            _rod(_name('cane'), (x + 0.14, BED_TOP - 0.02, 0.22), (x + 0.15, BED_TOP + 0.2, 0.22), 0.005, M.haft, verts=4)
    if not G.is_low():
        G.join(still, 'bed', origin=(0, 0, 0))
    xs = [-0.7 + i * 0.28 for i in range(6)]
    jit = [(rnd.uniform(-0.03, 0.03), rnd.uniform(-0.03, 0.03)) for _ in range(18)]
    with G.new_parts() as plants:
        for i, x in enumerate(xs):                          # lettuces, at the back
            dx, dz = jit[i]
            _lettuce(_name('lettuce'), x + dx, BED_TOP - 0.01, -0.24 + dz, 0.11, 0.15, (M.lettuce, M.lettuce_dark)[i % 2], i * 1.7)
        for i, x in enumerate(xs):                          # carrot tops, in the middle
            dx, dz = jit[6 + i]
            for k in range(low(4, 2)):
                _frond(_name('frond'), x + dx, BED_TOP, dz, k * 90 + i * 25, 0.17 + 0.02 * (k % 2), M.carrot_top)
        for i, x in enumerate(xs[::2]):                     # bushy tomato plants up their canes, at the front
            dx, dz = jit[12 + i]
            for k in range(3):
                a = k * 120 + i * 40
                _blob(x + 0.14 + dx + 0.06 * G.cosd(a), BED_TOP - 0.01 + k * 0.04, 0.22 + dz + 0.06 * G.sind(a),
                      0.09 - 0.012 * k, M.leaf, 20 + i * 3 + k, scale=(1, 1, 0.9), noise=0.45)
    with G.new_parts() as crop:
        for i, x in enumerate(xs):                          # carrot shoulders
            dx, dz = jit[6 + i]
            F.cone(_name('carrot'), 0.028, 0.02, 0.035, (x + dx, BED_TOP + 0.012, dz), M.carrot, verts=low(8, 5))
        for i, x in enumerate(xs[::2]):                     # tomatoes on the bushes, each with its green calyx
            dx, dz = jit[12 + i]
            for k in range(4):
                a = k * 90 + 20 + i * 30
                p = (x + 0.14 + dx + 0.12 * G.cosd(a), BED_TOP + 0.08 + 0.04 * (k % 2), 0.22 + dz + 0.12 * G.sind(a))
                F.sphere(_name('tomato'), 0.035, p, M.tomato, scale=(1, 1, 0.85), segments=low(8, 6), rings=low(5, 4))
                if not G.is_low():
                    F.cone(_name('calyx'), 0.016, 0.004, 0.012, (p[0], p[1] + 0.03, p[2]), M.green, verts=5)
    if G.is_low():                                          # a far version: all one still part, ripe
        return G.settle([G.join_all('raisedBed')], centre=False)
    origin = F.at(0, BED_TOP, 0)
    plants = G.join(plants, 'plants', origin=origin)
    crop = G.join(crop, 'crop', origin=origin)
    objs = G.settle(centre=False)
    height = G.measure([plants])[1][2] - plants.location[2]   # the plants' top over the soil
    G.stages(plants, 'grow', **PLANTS, sink=[s * height for s in SINK])
    G.stages(crop, 'grow', **CROP)
    G.squash(crop, 'harvest', 0.001, seconds=1.0)           # the ripe crop picked away (and back after, if not picked)
    # Tending: the plants and their crop rustle, 3° back and forth about the soil line through the bed's middle, on
    # top of the growth stage (objectMotion.ts plays a use clip after it).
    for part in (plants, crop):
        G.rock(part, 'rustle', 3, seconds=0.8, axis='X')
    return objs


# --- the garden shed (2x2): sage weatherboards, a felt roof, a window with a window box; the door opens ------
SW, SD, SZ = 1.8, 1.6, -0.08    # the walls' width and depth, and their middle (depth)
EAVE, RIDGE = 1.72, 2.3         # the walls' top and the roof's ridge
DOOR_X, DOOR_W, DOOR_H = -0.3, 0.68, 1.48   # the doorway: its middle, width, top (from 0.08)
FRONT_Z = SZ + SD / 2           # the front wall's face
OVER = 0.08                     # the roof's overhang at the eaves
LAP = 0.16                      # the weatherboards' pitch


def _laps(y0, y1, t, out, base=0.08):
    """A weatherboarded wall's section from height y0 to y1, `t` thick: each board's foot `out` proud of the one under
    it (a saw tooth outside, flat inside), the boards on the walls' one grid (from `base`, LAP apart), as (out from the
    inside face, height) points round it."""
    cuts = [base + k * LAP for k in range(1, 40) if y0 + 1e-6 < base + k * LAP < y1 - 1e-6]
    first = base + max(0, math.floor((y0 - base) / LAP + 1e-6)) * LAP
    # how far the board the section starts in has thinned by y0
    f0 = (y0 - first) / LAP
    pts = [(0.0, y0), (t + out * (1 - f0), y0)]
    for c in cuts:
        pts += [(t, c), (t + out, c)]
    pts += [(t, y1), (0.0, y1)]
    return pts


def _lap_wall(name, along, a0, a1, at, y0, y1, side, t=0.04, out=0.02):
    """A weatherboarded wall from a0 to a1 along game x (`along` 'x', its inside face at depth `at`, facing `side` ±1 in
    depth) or along depth ('z', its inside face at x `at`, facing `side` ±1 in x), from height y0 to y1: one sweep
    of the lapped section."""
    prof = _laps(y0, y1, t, out)
    if along == 'x':
        return _wall_sweep(name, [(a0, 0.0, at), (a1, 0.0, at)], prof, (0, 0, side))
    return _wall_sweep(name, [(at, 0.0, a0), (at, 0.0, a1)], prof, (side, 0, 0))


def _wall_sweep(name, path, prof, out_dir, step=0.3):
    """Sweeps a wall's section [(out, height)] along a straight game path at floor level: `out` along game direction
    `out_dir`, height up; cut every `step` along, so the baked shading has corners along the wall, not only at its
    buried ends ("Shading by corners")."""
    a, b = Vector(path[0]), Vector(path[1])
    o = Vector(out_dir)
    n = max(1, round((b - a).length / step)) if not G.is_low() else 1
    vs, faces = [], []
    for i in range(n + 1):
        p = a + (b - a) * (i / n)
        for u, v in prof:
            q = p + o * u
            vs.append(F.at(q.x, v, q.z))
    m = len(prof)
    for i in range(n):
        for k in range(m):
            k2 = (k + 1) % m
            faces.append((i * m + k, i * m + k2, (i + 1) * m + k2, (i + 1) * m + k))
    faces += [tuple(range(m - 1, -1, -1)), tuple(range(n * m, (n + 1) * m))]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vs, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(M.shed)
    G.shapes._recalc_normals(obj)
    return obj


def _shed_walls():
    t = 0.04
    base = 0.08
    h = EAVE - base
    yc = base + h / 2
    F.box('plinth', SW + 0.06, base, SD + 0.06, (0, base / 2, SZ), M.plinth, bevel=0.01, bevel_segments=1)
    zb, zf = SZ - SD / 2, FRONT_Z
    if G.is_low():
        F.box('wall_back', SW, h, t, (0, yc, zb + t / 2), M.shed)
        for s in (-1, 1):
            F.box(_name('wall_side'), t, h, SD - 2 * t, (s * (SW / 2 - t / 2), yc, SZ), M.shed)
    else:
        # weatherboards: each wall one sweep of the lapped section, the boards' feet proud of the ones under them
        _lap_wall('wall_back', 'x', -SW / 2, SW / 2, zb + t, base, EAVE, -1, t)
        for s in (-1, 1):
            _lap_wall(_name('wall_side'), 'z', zb + t, zf - t, s * (SW / 2 - t), base, EAVE, s, t)
    # the front, round the doorway
    fz = FRONT_Z - t / 2
    x0, x1 = DOOR_X - DOOR_W / 2, DOOR_X + DOOR_W / 2
    if G.is_low():
        left = x0 + SW / 2
        right = SW / 2 - x1
        F.box('wall_front_l', left, h, t, (-SW / 2 + left / 2, yc, fz), M.shed)
        F.box('wall_front_r', right, h, t, (SW / 2 - right / 2, yc, fz), M.shed)
        F.box('wall_front_top', DOOR_W, EAVE - (base + DOOR_H), t, (DOOR_X, (EAVE + base + DOOR_H) / 2, fz), M.shed)
    else:
        _lap_wall('wall_front_l', 'x', -SW / 2, x0, zf - t, base, EAVE, 1, t)
        _lap_wall('wall_front_r', 'x', x1, SW / 2, zf - t, base, EAVE, 1, t)
        _lap_wall('wall_front_top', 'x', x0, x1, zf - t, base + DOOR_H, EAVE, 1, t)
    # the gables
    for z, z0 in ((FRONT_Z - t, FRONT_Z), (SZ - SD / 2, SZ - SD / 2 + t)):
        _prism(_name('gable'), [(-SW / 2, EAVE), (SW / 2, EAVE), (0, RIDGE)], min(z, z0), max(z, z0), M.shed)
    # corner boards and the door and window frames
    for s in (-1, 1):
        for z in (FRONT_Z + 0.012, SZ - SD / 2 - 0.012):
            F.box(_name('corner'), 0.07, h, 0.024, (s * (SW / 2 - 0.025), yc, z), M.trim, bevel=low(0.004, 0), bevel_segments=1)
            F.box(_name('corner_side'), 0.024, h, 0.07, (s * (SW / 2 + 0.012), yc, z - 0.023 * (1 if z > SZ else -1)), M.trim,
                  bevel=low(0.004, 0), bevel_segments=1)
    for s in (-1, 1):
        F.box(_name('door_trim'), 0.06, DOOR_H, 0.03, (DOOR_X + s * (DOOR_W / 2 + 0.03), base + DOOR_H / 2, FRONT_Z + 0.01),
              M.trim, bevel=0.005)
    F.box('door_head', DOOR_W + 0.18, 0.07, 0.035, (DOOR_X, base + DOOR_H + 0.035, FRONT_Z + 0.012), M.trim, bevel=0.005)
    if not G.is_low():                                     # a drip cap over the door head
        F.box('door_drip', DOOR_W + 0.2, 0.015, 0.05, (DOOR_X, base + DOOR_H + 0.077, FRONT_Z + 0.02), M.trim, bevel=0.004,
              bevel_segments=1)


def _shed_roof():
    run = math.hypot(SW / 2 + OVER, (RIDGE - EAVE) * (SW / 2 + OVER) / (SW / 2))
    slope = math.degrees(math.atan2(RIDGE - EAVE, SW / 2))
    for s in (-1, 1):
        x = s * (SW / 2 + OVER) / 2
        y = RIDGE - (RIDGE - EAVE) * ((SW / 2 + OVER) / 2) / (SW / 2) + 0.03
        F.box(_name('roof'), run, 0.04, SD + 0.24, (x, y, SZ), M.roof, rot=(0, -s * slope, 0), bevel=low(0.008, 0))
        if not G.is_low():
            # the felt's laps: a strip proud at every course down the slope, a drip edge along the eave
            for k in range(1, 4):
                f = k / 4
                cx, cy = s * (SW / 2 + OVER) * f, RIDGE - (RIDGE - EAVE) * (SW / 2 + OVER) * f / (SW / 2) + 0.052
                F.box(_name('felt_lap'), 0.05, 0.006, SD + 0.24, (cx, cy, SZ), M.roof, rot=(0, -s * slope, 0))
            ex, ey = s * (SW / 2 + OVER), RIDGE - (RIDGE - EAVE) * (SW / 2 + OVER) / (SW / 2) + 0.01
            F.box(_name('drip'), 0.03, 0.05, SD + 0.24, (ex - s * 0.004, ey, SZ), M.roof, bevel=0.004, bevel_segments=1)
        for z in (FRONT_Z + 0.12, SZ - SD / 2 - 0.12):     # barge boards on the gables' edges
            F.box(_name('barge'), run - 0.04, 0.1, 0.025, (x, y - 0.045, z), M.trim, rot=(0, -s * slope, 0))
    F.box('ridge_cap', 0.14, 0.04, SD + 0.26, (0, RIDGE + 0.05, SZ), M.roof, bevel=low(0.012, 0), bevel_segments=1)
    if not G.is_low():                                     # a finial where the barge boards meet
        for z in (FRONT_Z + 0.115, SZ - SD / 2 - 0.115):
            F.box(_name('finial'), 0.05, 0.16, 0.035, (0, RIDGE - 0.02, z), M.trim, bevel=0.006, bevel_segments=1)


def _shed_inside():
    """In the shadow behind the doorway: dark lining, a shelf of pots, a fork and a rake on the wall."""
    t = 0.04
    lz0, lz1 = SZ - SD / 2 + t, FRONT_Z - t
    F.box('in_back', SW - 2 * t, EAVE - 0.08, 0.005, (0, (EAVE + 0.08) / 2, lz0 + 0.003), M.inside)
    for s in (-1, 1):
        F.box(_name('in_side'), 0.005, EAVE - 0.08, lz1 - lz0, (s * (SW / 2 - t - 0.003), (EAVE + 0.08) / 2, SZ), M.inside)
    F.box('in_floor', SW - 2 * t, 0.005, lz1 - lz0, (0, 0.083, SZ), M.inside)
    F.box('in_ceiling', SW - 2 * t, 0.005, lz1 - lz0 + 0.04, (0, EAVE - 0.003, SZ), M.inside)
    F.box('in_front', SW - 2 * t, EAVE - 0.08, 0.005, (0, (EAVE + 0.08) / 2, lz1 - 0.003), M.inside)
    F.box('shelf', 1.0, 0.03, 0.3, (-0.3, 1.0, lz0 + 0.15), M.box_wood)
    F.box('bench', 1.0, 0.04, 0.5, (-0.3, 0.8, lz0 + 0.25), M.box_wood)
    for k, x in enumerate((-0.65, -0.45, -0.25)):
        _pot(x, 1.015, lz0 + 0.14, 0.06, 0.1)
    for x, head in ((0.0, 'fork'), (-0.75, 'rake')):
        _rod(_name('tool'), (x, 0.1, lz0 + 0.3), (x + 0.04, 1.25, lz0 + 0.06), 0.012, M.haft, verts=6)
        F.box(_name(head), 0.14 if head == 'fork' else 0.3, 0.12 if head == 'fork' else 0.03, 0.02,
              (x, 0.15, lz0 + 0.29), M.axe)


def _t_hinge(name, x, y, z, length, material):
    """A black T-hinge on a door at game (x = its knuckle, y, z = the door's face): a strap tapering to a rounded end
    across the door, a plate on the frame, and three bolt heads."""
    strap = [(0.0, -0.022), (length - 0.02, -0.01), (length, -0.006), (length + 0.006, 0.0), (length, 0.006),
             (length - 0.02, 0.01), (0.0, 0.022), (-0.03, 0.04), (-0.05, 0.04), (-0.05, -0.04), (-0.03, -0.04)]
    F.extrude(name, [(x + u, y + v) for u, v in strap], 0.005, (0, 0, z), material, bevel=0.0015, bevel_segments=1)
    for k, u in enumerate((0.08, length * 0.55, length - 0.03)):
        F.cyl(f'{name}_bolt{k}', 0.007, 0.006, (x + u, y, z + 0.004), material, rot=(90, 0, 0), verts=6)


def shedSmall():
    with G.new_parts():
        _shed_walls()
        _shed_roof()
        if not G.is_low():
            _shed_inside()
        # the window, right of the door: a moulded white frame mitred round four dark panes and their bars; a sill
        wx, wy = 0.42, 1.12
        G.kit.moulding(F, 'window_frame', [(wx + 0.28, wy - 0.23), (wx + 0.28, wy + 0.23), (wx - 0.28, wy + 0.23), (wx - 0.28, wy - 0.23)],
                       [(0.0, 0.0), (0.0, 0.03), (0.012, 0.04), (0.045, 0.04), (0.055, 0.03), (0.055, 0.0)] if not G.is_low()
                       else [(0.0, 0.0), (0.0, 0.04), (0.055, 0.04), (0.055, 0.0)], M.trim, at=FRONT_Z - 0.002)
        F.box('window_pane', 0.46, 0.36, 0.02, (wx, wy, FRONT_Z + 0.002), M.pane)
        F.box('window_bar_v', 0.022, 0.36, 0.02, (wx, wy, FRONT_Z + 0.014), M.trim)
        F.box('window_bar_h', 0.46, 0.022, 0.02, (wx, wy, FRONT_Z + 0.014), M.trim)
        F.extrude('window_sill', [(-0.33, 0.0), (0.33, 0.0), (0.33, 0.035), (-0.33, 0.035)], 0.08, (wx, wy - 0.262, FRONT_Z + 0.04),
                  M.trim, bevel=low(0.008, 0), bevel_segments=1)
        # the window box on brackets, in flower
        bx, by, bz = F.at(wx, 0, FRONT_Z + 0.095)
        box = [(-0.3, -0.085), (0.3, -0.085), (0.3, 0.085), (-0.3, 0.085)]
        G.loft('window_box', [[(bx - u * k, by - v * k, wy + dy) for u, v in box]
                              for dy, k in ((-0.43, 0.9), (-0.29, 1.0), (-0.285, 0.96))], material=M.box_wood)
        for s in (-1, 1):
            F.extrude(_name('bracket'), [(0.0, 0.0), (0.0, 0.14), (0.15, 0.14), (0.15, 0.12)], 0.03, (wx + s * 0.22, wy - 0.57, FRONT_Z),
                      M.box_wood, plane='side', bevel=low(0.004, 0), bevel_segments=1)
        flowers = (M.flower_pink, M.flower_yellow, M.flower_red, M.flower_lilac)
        for i in range(4):
            x = wx - 0.21 + i * 0.14
            _blob(x, wy - 0.3, FRONT_Z + 0.1, 0.07, M.green, 30 + i, scale=(1, 1, 0.8), noise=0.35)
            for k in range(3):
                F.sphere(_name('flower'), 0.025, (x + (k - 1) * 0.035, wy - 0.2 + 0.01 * (k % 2), FRONT_Z + 0.09 + 0.03 * (k - 1)),
                         flowers[(i + k) % 4], segments=low(8, 4), rings=low(5, 3))
    if not G.is_low():
        G.join(G.meshes(), 'shed', origin=(0, 0, 0))
    # The door: ledged and braced boards, black T-hinges on the left, a ring latch on the right; it swings out.
    with G.new_parts() as door:
        y0, x0 = 0.09, DOOR_X - DOOR_W / 2
        dz = FRONT_Z + 0.015
        for k in range(5):
            w = (DOOR_W - 0.02) / 5
            F.box(_name('door_board'), w - 0.006, DOOR_H - 0.02, 0.03, (x0 + 0.01 + w * (k + 0.5), y0 + (DOOR_H - 0.02) / 2, dz),
                  M.shed_door, bevel=low(0.004, 0), bevel_segments=1)
        for y in (0.3, 1.27):
            F.box(_name('ledge'), DOOR_W - 0.08, 0.1, 0.02, (DOOR_X, y, dz + 0.024), M.shed_door, bevel=low(0.005, 0),
                  bevel_segments=1)
            if G.is_low():
                F.box(_name('hinge'), 0.3, 0.03, 0.008, (x0 + 0.15, y, dz + 0.038), M.iron)
            else:
                _t_hinge(_name('hinge'), x0 + 0.02, y, dz + 0.0365, 0.32, M.iron)
        _board('brace', (x0 + 0.08, 0.36, dz + 0.024), (x0 + DOOR_W - 0.08, 1.21, dz + 0.024), 0.09, 0.02, M.shed_door,
               face=(0, 0, 1), bevel=low(0.005, 0))
        lx = x0 + DOOR_W - 0.08
        F.extrude('latch_plate', G.rounded_rect(0.06, 0.15, 0.012, corner=low(2, 1)), 0.006, (lx, 0.86, dz + 0.018), M.iron)
        if not G.is_low():
            G.torus('latch_ring', 0.03, 0.005, at=F.at(lx, 0.83, dz + 0.026), material=M.iron, rot=(90, 0, 0),
                    major_segments=12, minor_segments=4)
            F.cyl('latch_boss', 0.012, 0.012, (lx, 0.86, dz + 0.026), M.iron, rot=(90, 0, 0), verts=8)
            F.box('latch_bar', 0.12, 0.012, 0.012, (lx + 0.05, 0.9, dz + 0.024), M.iron, bevel=0.003, bevel_segments=1)
    if G.is_low():
        return G.settle([G.join_all('shedSmall')], centre=False)
    door = G.hinged(G.join(door, 'door'), F.at(DOOR_X - DOOR_W / 2, 0.8, FRONT_Z + 0.015))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=1.0)
    return objs


# --- the greenhouse (2x3): a white frame on a dwarf wall, glazed; staging and tomatoes inside; door and vent ----
GHW, GHD = 1.8, 2.8              # width, depth
WALL_TOP, GH_EAVE, GH_RIDGE = 0.42, 1.8, 2.36
GH_DOOR = 0.62
VENT_Z = (-0.75, 0.15)           # the roof vent, depths (left slope)


def greenhouse():
    hx, hz = GHW / 2, GHD / 2
    slope = math.degrees(math.atan2(GH_RIDGE - GH_EAVE, hx))
    run = math.hypot(hx, GH_RIDGE - GH_EAVE)

    def roof_at(t):
        """A point down the slope `t` of the way from the ridge to the eave (x on the +x side, height)."""
        return hx * t, GH_RIDGE - (GH_RIDGE - GH_EAVE) * t

    with G.new_parts():
        # the dwarf wall, open at the door; a gravel floor and a paved path
        t = 0.1
        F.box('wall_back', GHW, WALL_TOP, t, (0, WALL_TOP / 2, -hz + t / 2), M.gh_wall, bevel=low(0.01, 0))
        for s in (-1, 1):
            F.box(_name('wall_side'), t, WALL_TOP, GHD - 2 * t, (s * (hx - t / 2), WALL_TOP / 2, 0), M.gh_wall,
                  bevel=low(0.01, 0))
            w = (GHW - GH_DOOR) / 2
            F.box(_name('wall_front'), w, WALL_TOP, t, (s * (GH_DOOR / 2 + w / 2), WALL_TOP / 2, hz - t / 2), M.gh_wall,
                  bevel=low(0.01, 0))
        if not G.is_low():                                 # the brick courses' mortar lines, and a coping
            for k in range(1, 5):
                y = k * WALL_TOP / 5
                F.box(_name('course'), GHW + 0.002, 0.006, t + 0.002, (0, y, -hz + t / 2), M.mortar)
                for s in (-1, 1):
                    F.box(_name('course'), t + 0.002, 0.006, GHD - 2 * t, (s * (hx - t / 2), y, 0), M.mortar)
                    w = (GHW - GH_DOOR) / 2
                    F.box(_name('course'), w + 0.002, 0.006, t + 0.002, (s * (GH_DOOR / 2 + w / 2), y, hz - t / 2), M.mortar)
        F.box('gravel', GHW - 2 * t, 0.02, GHD - t, (0, 0.01, 0.05), M.gravel)
        F.box('path', 0.5, 0.025, GHD - t, (0, 0.0125, 0.05), M.paving)
        if not G.is_low():                                 # the path's slabs
            for k in range(1, 6):
                F.box(_name('joint'), 0.5, 0.003, 0.008, (0, 0.025, -hz + t + k * (GHD - t) / 6), M.gravel)
        # the frame: sills, corner posts, glazing bars, eaves, rafters, the ridge
        for s in (-1, 1):
            F.box(_name('sill'), 0.06, 0.04, GHD, (s * (hx - 0.03), WALL_TOP + 0.02, 0), M.gh_frame)
            F.box(_name('eave'), 0.06, 0.05, GHD, (s * (hx - 0.03), GH_EAVE, 0), M.gh_frame)
        F.box('sill_back', GHW, 0.04, 0.06, (0, WALL_TOP + 0.02, -hz + 0.03), M.gh_frame)
        for z in (-hz + 0.025, hz - 0.025):
            F.box(_name('eave_end'), GHW, 0.04, 0.05, (0, GH_EAVE, z), M.gh_frame)
        bars = [-hz + 0.025 + k * (GHD - 0.05) / 4 for k in range(5)]
        for z in bars:
            for s in (-1, 1):
                F.box(_name('bar'), 0.04, GH_EAVE - WALL_TOP, 0.04, (s * (hx - 0.025), (GH_EAVE + WALL_TOP) / 2, z),
                      M.gh_frame)
                _board(_name('rafter'), (s * 0.03, GH_RIDGE, z), (s * (hx - 0.02), GH_EAVE + 0.01, z), 0.04, 0.04,
                       M.gh_frame, face=(0, 0, 1), bevel=0)
        F.box('ridge', 0.06, 0.06, GHD + 0.04, (0, GH_RIDGE, 0), M.gh_frame)
        for z in (-hz + 0.01, hz - 0.01):                  # a finial at each end of the ridge
            G.revolve(_name('finial'), [(0.0, 0.0), (0.018, 0.0), (0.012, 0.015), (0.016, 0.024), (0.0, 0.036)],
                      at=F.at(0, GH_RIDGE + 0.03, z), material=M.gh_frame, segments=low(8, 4))
        if not G.is_low():
            for z in (-hz + 0.025 + (GHD - 0.05) * k / 4 for k in range(4)):
                for s in (-1, 1):                          # a glazing bar across each side's panes
                    F.box(_name('bar_h'), 0.03, 0.025, (GHD - 0.05) / 4 - 0.04, (s * (hx - 0.022), 1.1, z + (GHD - 0.05) / 8),
                          M.gh_frame)
        for x in (-0.45, 0.45):                            # the back's and the front's glazing bars
            F.box(_name('bar_back'), 0.04, GH_EAVE - WALL_TOP, 0.04, (x, (GH_EAVE + WALL_TOP) / 2, -hz + 0.025), M.gh_frame)
        for s in (-1, 1):
            F.box(_name('door_post'), 0.05, GH_EAVE, 0.05, (s * (GH_DOOR / 2 + 0.025), GH_EAVE / 2, hz - 0.025), M.gh_frame)
            F.box(_name('sill_front'), (GHW - GH_DOOR) / 2, 0.04, 0.06, (s * (GHW + GH_DOOR) / 4, WALL_TOP + 0.02, hz - 0.03),
                  M.gh_frame)
        F.box('transom', GH_DOOR, 0.05, 0.05, (0, GH_EAVE - 0.04, hz - 0.025), M.gh_frame)
        # the vent's frame on the left slope (the vent's glass is its own part)
        vz0, vz1 = VENT_Z
        for z in (vz0, vz1):
            _board(_name('vent_rafter'), (-0.03, GH_RIDGE, z), (-(hx - 0.02), GH_EAVE + 0.01, z), 0.03, 0.04, M.gh_frame,
                   face=(0, 0, 1), bevel=0)
        # the glass: sides above the wall, back, front either side of the door, gables, the roof round the vent
        gh = GH_EAVE - WALL_TOP - 0.04
        gy = WALL_TOP + 0.04 + gh / 2
        for s in (-1, 1):
            F.box(_name('glass_side'), 0.01, gh, GHD - 0.06, (s * (hx - 0.025), gy, 0), M.glass)
            F.box(_name('glass_front'), (GHW - GH_DOOR) / 2 - 0.08, gh, 0.01, (s * (GHW + GH_DOOR) / 4, gy, hz - 0.025),
                  M.glass)
        F.box('glass_back', GHW - 0.06, gh, 0.01, (0, gy, -hz + 0.025), M.glass)
        for z in (-hz + 0.025, hz - 0.025):
            _prism(_name('glass_gable'), [(-hx + 0.03, GH_EAVE + 0.02), (hx - 0.03, GH_EAVE + 0.02), (0, GH_RIDGE - 0.03)],
                   z - 0.005, z + 0.005, M.glass)
        # the roof: the right slope whole; the left round its vent
        mx, my = roof_at(0.5)
        F.box('glass_roof_r', run, 0.01, GHD - 0.04, (mx, my + 0.025, 0), M.glass, rot=(0, -slope, 0))
        for z0, z1 in ((-hz + 0.02, vz0 - 0.015), (vz1 + 0.015, hz - 0.02)):
            F.box(_name('glass_roof_l'), run, 0.01, z1 - z0, (-mx, my + 0.025, (z0 + z1) / 2), M.glass, rot=(0, slope, 0))
        # the staging: a slatted bench down each side, pots of seedlings; tomatoes in grow bags at the back
        for s in (-1, 1):
            x = s * 0.56
            for k in range(low(5, 2)):
                F.box(_name('slat'), 0.09 if not G.is_low() else 0.5, 0.025, 1.9,
                      (x - 0.2 + k * 0.1 if not G.is_low() else x, 0.78, 0.25), M.staging)
            for z in (-0.65, 1.15):
                for e in (-1, 1):
                    F.box(_name('leg'), 0.04, 0.76, 0.04, (x + e * 0.2, 0.38, z), M.staging)
            for k, z in enumerate((-0.5, -0.1, 0.3, 0.7, 1.05)):
                _pot(x + (0.08 if k % 2 else -0.08), 0.79, z, 0.07, 0.11)
                _blob(x + (0.08 if k % 2 else -0.08), 0.88, z, 0.08, M.green, 40 + k + 5 * s, scale=(1, 1, 0.9), noise=0.4)
        for k, x in enumerate((-0.45, 0.0, 0.45)):         # tomato plants on canes along the back
            F.box(_name('grow_bag'), 0.36, 0.1, 0.24, (x, 0.07, -1.18), M.compost, bevel=low(0.03, 0))
            _rod(_name('cane'), (x, 0.1, -1.18), (x, 1.55, -1.2), 0.008, M.haft, verts=4)
            for j in range(low(5, 3)):
                _blob(x + (0.05 if j % 2 else -0.05), 0.15 + j * 0.27, -1.16, 0.12 - 0.01 * j, M.green, 60 + 7 * k + j,
                      scale=(1, 0.7, 1.4), noise=0.45)
            for j in range(4):
                F.sphere(_name('tomato'), 0.032, (x + 0.08 * (1 if j % 2 else -1), 0.5 + j * 0.22, -1.1), M.tomato,
                         segments=low(10, 5), rings=low(6, 3))
    if not G.is_low():
        G.join(G.meshes(), 'greenhouse', origin=(0, 0, 0))
    # The door: a glass leaf hinged on the left post, swinging out; the vent: a glass panel hinged at the ridge,
    # lifting. Their edges are white glass (one draw each: see the notes); `door_open` opens both.
    with G.new_parts() as door:
        dz = hz - 0.025
        dh = GH_EAVE - 0.07
        F.box('door_glass', GH_DOOR - 0.04, dh - 0.06, 0.01, (0, 0.03 + dh / 2, dz), M.glass)
        for s in (-1, 1):
            F.box(_name('door_stile'), 0.03, dh, 0.02, (s * (GH_DOOR / 2 - 0.035), 0.03 + dh / 2, dz), M.glass_bar)
        for y in (0.045, 0.03 + dh - 0.015, 0.95):
            F.box(_name('door_rail'), GH_DOOR - 0.07, 0.03, 0.02, (0, y, dz), M.glass_bar)
        F.box('door_kick', GH_DOOR - 0.07, 0.24, 0.014, (0, 0.18, dz), M.glass_bar)
    with G.new_parts() as vent:
        mx, my = roof_at(0.5)
        F.box('vent_glass', run - 0.04, 0.012, vz1 - vz0 - 0.02, (-mx, my + 0.032, (vz0 + vz1) / 2), M.glass,
              rot=(0, slope, 0))
        for z in (vz0 + 0.02, vz1 - 0.02):
            F.box(_name('vent_edge'), run - 0.04, 0.02, 0.025, (-mx, my + 0.04, z), M.glass_bar, rot=(0, slope, 0))
    if G.is_low():
        return G.settle([G.join_all('greenhouse')], centre=False)
    door = G.hinged(G.join(door, 'door'), F.at(-GH_DOOR / 2 + 0.02, 1.0, hz - 0.025))
    vent = G.hinged(G.join(vent, 'vent'), F.at(-0.02, GH_RIDGE + 0.01, 0))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 70, seconds=0.9)
    G.swing(vent, 'door_open', -25, seconds=0.9, axis='Y')
    return objs


# --- the compost bin (1x1): a slatted box on corner posts, the heap inside; the lid lifts ---------------------
CB = (0.8, 0.76, -0.08)     # width, depth, middle (depth)
CB_TOP = 0.72


def compostBin():
    w, d, cz = CB
    z0, z1 = cz - d / 2, cz + d / 2
    with G.new_parts():
        for x in (-w / 2 + 0.03, w / 2 - 0.03):
            for z in (z0 + 0.03, z1 - 0.03):
                F.box(_name('post'), 0.06, CB_TOP - 0.012, 0.06, (x, (CB_TOP - 0.012) / 2, z), M.bin, bevel=low(0.008, 0),
                      bevel_segments=1)
                if not G.is_low():                         # each post's top cut to a shallow point
                    G.cone(_name('post_cap'), 0.042, 0.0, 0.012, at=F.at(x, CB_TOP - 0.006, z), material=M.bin, verts=4,
                           rot=(0, 0, 45))
        for k, y in enumerate((0.08, 0.25, 0.42, 0.59)):
            F.box(_name('slat_back'), w - 0.04, 0.12, 0.025, (0, y + 0.02, z0 + 0.01), M.bin, bevel=low(0.006, 0),
                  bevel_segments=1)
            for s in (-1, 1):
                F.box(_name('slat_side'), 0.025, 0.12, d - 0.12, (s * (w / 2 - 0.01), y + 0.02, cz), M.bin,
                      bevel=low(0.006, 0), bevel_segments=1)
            if k < 3:                                      # the front boards: the top one lifted out
                F.box(_name('slat_front'), w - 0.12, 0.12, 0.025, (0, y + 0.02, z1 - 0.01), M.bin, bevel=low(0.006, 0),
                      bevel_segments=1)
        if not G.is_low():                                 # galvanised nails in the slats' ends
            for k, y in enumerate((0.08, 0.25, 0.42, 0.59)):
                for s in (-1, 1):
                    for z in (z0 + 0.03, z1 - 0.03):
                        F.cyl(_name('nail'), 0.005, 0.004, (s * (w / 2 + 0.001), y + 0.02, z), M.can, rot=(0, 90, 0),
                              verts=5)
        for s in (-1, 1):                                  # the front boards' guide battens
            F.box(_name('guide'), 0.03, CB_TOP - 0.06, 0.03, (s * (w / 2 - 0.075), (CB_TOP - 0.06) / 2, z1 - 0.035), M.bin,
                  bevel=low(0.004, 0), bevel_segments=1)
        # the heap: a mound of compost, peelings and greens on top, a few lumps
        _blob(0, 0.02, cz, 0.36, M.compost, 2.0, scale=(1, 1, 0.82), noise=0.25)
        for k, (x, z, m) in enumerate(((0.12, cz + 0.06, M.greens), (-0.15, cz - 0.08, M.peel), (0.02, cz - 0.14, M.greens),
                                       (-0.05, cz + 0.12, M.peel), (0.18, cz - 0.1, M.compost), (-0.2, cz + 0.08, M.greens))):
            _blob(x, 0.56 - 0.02 * (k // 4), z, 0.05, m, 9 + k, scale=(1.4, 1, 0.4), noise=0.5)
        if not G.is_low():
            for k, (x, z) in enumerate(((0.06, cz - 0.02), (-0.1, cz + 0.02), (0.12, cz + 0.14))):
                F.cone(_name('shell'), 0.025, 0.018, 0.014, (x, 0.585, z), M.shell, verts=6, rot=(10 * k, 0, 0))
    if not G.is_low():
        G.join(G.meshes(), 'bin', origin=(0, 0, 0))
    # The lid: four boards on two battens, a wooden grip, hinged along the back.
    with G.new_parts() as lid:
        for k in range(4):
            bw = (d + 0.04) / 4
            F.box(_name('lid_board'), w + 0.04, 0.025, bw - 0.008, (0, CB_TOP + 0.0125, z0 - 0.02 + bw * (k + 0.5)), M.bin,
                  bevel=low(0.006, 0), bevel_segments=1)
        for s in (-1, 1):
            F.box(_name('batten'), 0.05, 0.03, d - 0.04, (s * (w / 2 - 0.1), CB_TOP - 0.012, cz), M.bin, bevel=low(0.004, 0),
                  bevel_segments=1)
        F.extrude('grip', [(-0.1, 0.0), (0.1, 0.0), (0.085, 0.03), (0.06, 0.035), (-0.06, 0.035), (-0.085, 0.03)], 0.04,
                  (0, CB_TOP + 0.025, z1 - 0.03), M.bin, bevel=low(0.006, 0), bevel_segments=1)
    if G.is_low():
        return G.settle([G.join_all('compostBin')], centre=False)
    lid = G.hinged(G.join(lid, 'lid'), F.at(0, CB_TOP, z0 - 0.02))
    objs = G.settle(centre=False)
    G.swing(lid, 'lid_open', -75, seconds=0.8, axis='X')
    return objs


# --- the rain barrel (1x1): a ribbed green barrel with a lid and its inlet, a brass tap ------------------------
# Free-standing, so no downpipe (Q-06): one ended in mid-air wherever the barrel stood. It stands in the middle of its
# tile (it stood 0.15 m back, the pipe against the wall).
RB_Z = 0.0


def rainBarrel():
    with G.new_parts():
        # the barrel: moulded in one, a foot ring, two hoop bands, ribs running up between them, a rolled top edge
        bx, by, _ = F.at(0, 0, RB_Z)
        n = low(32, 12)
        rows = [(0.0, 0.292, 0), (0.008, 0.302, 0), (0.05, 0.302, 0), (0.07, 0.28, 0), (0.25, 0.285, 1), (0.285, 0.296, 0),
                (0.315, 0.296, 0), (0.34, 0.286, 1), (0.62, 0.283, 1), (0.645, 0.293, 0), (0.675, 0.293, 0),
                (0.7, 0.282, 1), (0.88, 0.272, 1), (0.9, 0.28, 0), (0.93, 0.28, 0), (0.935, 0.26, 0)]
        if G.is_low():
            rows = [rows[i] for i in (0, 2, 3, 5, 6, 9, 10, 12, 13, 14, 15)]
        secs = [G.ring(r, r, y, n, at=(bx, by)) for y, r, rib in rows]
        if not G.is_low():                                 # the ribs: every other point a little proud where `rib`
            secs = [[(bx + (p[0] - bx) * (1 + (0.025 if rib and i % 2 else 0)), by + (p[1] - by) * (1 + (0.025 if rib and i % 2 else 0)), p[2])
                     for i, p in enumerate(sec)] for sec, (_, _, rib) in zip(secs, rows)]
        G.loft('barrel', secs, material=M.barrel, angle=35)
        # the lid: a domed disc with a raised rim, a mesh inlet at the back and its ring
        G.revolve('lid', [(0.0, 0.0), (0.262, 0.0), (0.27, 0.012), (0.262, 0.024), (0.24, 0.026), (0.12, 0.034), (0.0, 0.036)],
                  at=F.at(0, 0.925, RB_Z), material=M.barrel_dark, segments=low(24, 10))
        F.cyl('lid_inlet', 0.065, 0.016, (0, 0.962, RB_Z - 0.1), M.barrel_dark, verts=low(12, 6))
        if not G.is_low():
            F.cyl('inlet_mesh', 0.05, 0.004, (0, 0.969, RB_Z - 0.1), M.tyre, verts=12)
            for k in range(3):
                F.box(_name('lid_grip'), 0.08, 0.008, 0.012, (0.13 * G.cosd(120 * k + 30), 0.958, RB_Z + 0.13 * G.sind(120 * k + 30)),
                      M.barrel_dark, rot=(0, 0, 120 * k + 30), bevel=0.003, bevel_segments=1)
        # the brass tap at the front: a flange, a body, a spout curving down, a lever on top
        F.cyl('tap_flange', 0.032, 0.012, (0, 0.17, RB_Z + 0.29), M.brass, rot=(90, 0, 0), verts=low(10, 6))
        G.kit.tube(F, 'tap_spout', G.kit.fillet([(0, 0.17, RB_Z + 0.29), (0, 0.17, RB_Z + 0.355), (0, 0.12, RB_Z + 0.367)], 0.03, 3),
                   0.016, M.brass, verts=low(10, 6), round=1)
        F.cyl('tap_body', 0.024, 0.05, (0, 0.2, RB_Z + 0.33), M.brass, verts=low(10, 6))
        F.sweep('tap_lever', [(0, 0.23, RB_Z + 0.33), (0, 0.235, RB_Z + 0.37)], None, M.brass,
                profile=G.rounded_rect(0.012, 0.03, 0.004, corner=1), up=(1, 0, 0), scale=[1.0, 1.2])
    return G.settle([G.join_all('rainBarrel')], centre=False)


# --- the washing line (3x1): two T-posts, three lines; the washing is pegged out and stays out --------------
LINE_Y = 1.76
LINES = (-0.26, 0.0, 0.26)


def _hang(name, outline, x, z, m, t=0.006):
    """A piece of washing cut to `outline` [(across, down from the line), ...], hung at x on the line at depth z: a slab
    `t` thick (thinner than the line: flattened at rest, the washing is inside it)."""
    return F.extrude(name, [(x + u, LINE_Y - v) for u, v in outline], t, (0, 0, z), m)


def _garment(kind, x, z, w, h, m):
    """A thing hung on the line at x, depth z, `w` wide and `h` down, in material `m`, with two pegs: a T-shirt with
    its sleeves and neck, trousers with their legs apart, a pair of socks, a towel with a woven band, a sheet with a
    soft hem, a pillowcase."""
    hw = w / 2
    if kind == 'shirt':
        body = w * 0.31
        out = [(-body, 0.0), (-body * 0.45, 0.0), (-body * 0.3, 0.04), (body * 0.3, 0.04), (body * 0.45, 0.0), (body, 0.0),
               (hw, h * 0.12), (hw - 0.02, h * 0.42), (body, h * 0.32), (body, h), (-body, h), (-body, h * 0.32),
               (-hw + 0.02, h * 0.42), (-hw, h * 0.12)]
        _hang(_name('shirt'), out, x, z, m)
        pegs = (x - body * 0.8, x + body * 0.8)
    elif kind == 'trousers':
        out = [(-hw, 0.0), (hw, 0.0), (hw, h * 0.2), (hw * 0.96, h), (hw * 0.12, h), (0.0, h * 0.34), (-hw * 0.12, h),
               (-hw * 0.96, h), (-hw, h * 0.2)]
        _hang(_name('trousers'), out, x, z, m)
        pegs = (x - w * 0.4, x + w * 0.4)
    elif kind == 'socks':
        for s in (-1, 1):
            cx = x + s * w * 0.25
            sock = [(-0.04, 0.0), (0.04, 0.0), (0.04, h * 0.68), (0.095, h * 0.76), (0.1, h * 0.9), (0.085, h), (-0.03, h),
                    (-0.04, h * 0.92)]
            sock = sock if s > 0 else [(-u, v) for u, v in sock]
            _hang(_name('sock'), sock, cx, z, m)
            F.box(_name('sock_cuff'), 0.08, 0.04, 0.008, (cx, LINE_Y - 0.025, z), M.white)
        pegs = (x - w * 0.25, x + w * 0.25)
    elif kind == 'sheet':
        hem = [(hw - w * i / 12, h + 0.012 * math.sin(math.pi * i / 3)) for i in range(13)]
        _hang(_name('sheet'), [(-hw, 0.0), (hw, 0.0)] + hem, x, z, m)
        F.box(_name('sheet_hem'), w, 0.025, 0.008, (x, LINE_Y - h + 0.03, z), M.white)
        pegs = (x - hw + 0.05, x, x + hw - 0.05)
    else:                                                  # a towel, a pillowcase
        _hang(_name(kind), [(-hw, 0.0), (hw, 0.0), (hw, h), (-hw, h)], x, z, m)
        band = M.white if m is not M.white else M.sage
        for y in ((h * 0.82,) if kind == 'towel' else (h * 0.15,)):
            F.box(_name('band'), w, 0.045 if kind == 'towel' else 0.02, 0.008, (x, LINE_Y - y, z), band)
        pegs = (x - hw + 0.05, x + hw - 0.05)
    for px in pegs:                                        # a wooden peg: two legs split at the foot, astride the line
        _hang(_name('peg'), [(-0.008, -0.014), (0.008, -0.014), (0.009, 0.02), (0.0045, 0.05), (0.0, 0.036), (-0.0045, 0.05),
                             (-0.009, 0.02)], px, z, M.peg, t=0.012)


def clothesline():
    with G.new_parts():
        for x in (-1.32, 1.32):
            # the post, a cap on its top; the arm across, capped; two braces bent up into it; a flange at its foot
            F.sweep(_name('post'), [(x, 0.0, 0), (x, LINE_Y + 0.045, 0)], 0.03, M.posts, verts=low(12, 6))
            F.cyl(_name('post_cap'), 0.034, 0.012, (x, LINE_Y + 0.051, 0), M.posts, verts=low(12, 6), bevel=low(0.004, 0),
                  bevel_segments=1)
            F.sweep(_name('arm'), [(x, LINE_Y + 0.02, -0.343), (x, LINE_Y + 0.02, 0.343)], 0.022, M.posts, verts=low(10, 5))
            for s in (-1, 1):
                F.cyl(_name('arm_cap'), 0.025, 0.012, (x, LINE_Y + 0.02, s * 0.345), M.posts, rot=(90, 0, 0), verts=low(10, 5))
                _bent(_name('brace'), [(x, LINE_Y - 0.28, 0.02 * s), (x, LINE_Y - 0.1, 0.12 * s), (x, LINE_Y, s * 0.25)], 0.012,
                      M.posts, bend=0.06, verts=low(6, 4))
            G.revolve(_name('foot'), [(0.0, 0.0), (0.06, 0.0), (0.056, 0.01), (0.04, 0.03), (0.035, 0.06), (0.0, 0.06)],
                      at=F.at(x, 0, 0), material=M.posts, segments=low(12, 6))
        for z in LINES:
            F.sweep(_name('line'), [(-1.34, LINE_Y, z), (1.34, LINE_Y, z)], 0.008, M.line, verts=low(8, 4))
            if not G.is_low():                             # a tensioner on each line by the right post
                F.box(_name('tensioner'), 0.06, 0.02, 0.022, (1.2, LINE_Y, z), M.posts, bevel=0.005, bevel_segments=1)
        # the basket by the left post, woven, a rim and two handles, the rest of the washing folded in it
        bx, bz = -1.0, 0.24
        px, pz, _ = F.at(bx, 0, bz)
        rings = [(0.0, 0.22, 0.125), (0.02, 0.24, 0.142), (0.22, 0.25, 0.152)]
        G.loft('basket', [G.ring(rx, rz, y, low(20, 8), square=4, at=(px, pz)) for y, rx, rz in rings], material=M.basket)
        F.sweep('basket_rim', [(bx + u, 0.235, bz + v) for u, v in G.rounded_rect(0.52, 0.322, 0.08, corner=low(3, 1))], 0.014,
                M.basket, verts=low(6, 4), closed=True)
        for s in (-1, 1):
            if not G.is_low():
                G.torus(_name('basket_handle'), 0.035, 0.009, at=F.at(bx + s * 0.262, 0.23, bz), material=M.basket,
                        rot=(90, 0, 90), major_segments=8, minor_segments=4)
        for k, (m, y) in enumerate(((M.white, 0.25), (M.pink, 0.278), (M.sage, 0.302))):
            F.soft(_name('folded'), 0.34 - 0.04 * k, 0.026, 0.22 - 0.02 * k, (bx + 0.01 * k, y, bz), m, round=0.012,
                   puff=0.004, flat=1, round_segments=1)
    if G.is_low():                                          # from afar: at rest, the washing in
        return G.settle([G.join_all('clothesline')], centre=False)
    G.join(G.meshes(), 'posts', origin=(0, 0, 0))
    # The washing, pegged on the lines: its origin is on the line, so at rest it is flattened into it, and
    # `washing` lets it down.
    with G.new_parts() as washing:
        hung = (('towel', -0.9, 0, 0.5, 0.6, M.towel), ('pillowcase', -0.3, 0, 0.42, 0.5, M.white),
                ('shirt', 0.42, 0, 0.62, 0.48, M.pink),
                ('trousers', -0.62, 1, 0.38, 0.74, M.denim), ('shirt', 0.18, 1, 0.5, 0.4, M.yellow),
                ('towel', 0.92, 1, 0.32, 0.56, M.white),
                ('socks', -1.0, 2, 0.26, 0.3, M.red), ('sheet', 0.0, 2, 0.9, 0.66, M.washing),
                ('towel', 0.86, 2, 0.4, 0.46, M.sage))
        for kind, x, row, w, h, m in hung:
            _garment(kind, x, LINES[row], w, h, m)
    washing = G.join(washing, 'washing', origin=F.at(0, LINE_Y, 0))
    washing.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(washing, 'washing', seconds=1.5)
    return objs


# --- the log store (1x1): a black iron rack under a little roof, stacked with logs, ends to the front --------
LS_Z = -0.24


def _log_section(kind, r, rnd, n):
    """A log's cross-section (x, height about its middle): round (its bark lumpy), split in half or in a quarter."""
    if kind == 'round':
        return [(r * rnd.uniform(0.93, 1.05) * math.cos(math.tau * i / n), r * rnd.uniform(0.93, 1.05) * math.sin(math.tau * i / n))
                for i in range(n)]
    if kind == 'half':
        pts = [(r * math.cos(math.pi * i / (n - 2)), r * 0.15 + r * 0.85 * math.sin(math.pi * i / (n - 2))) for i in range(n - 1)]
        return [(u, v - r * 0.45) for u, v in pts]
    pts = [(0.0, 0.0)] + [(r * 1.25 * math.cos(math.pi / 2 * i / (n - 2)), r * 1.25 * math.sin(math.pi / 2 * i / (n - 2)))
                          for i in range(n - 1)]
    return [(u - r * 0.5, v - r * 0.5) for u, v in pts]


def firewoodStack():
    rnd = random.Random(11)
    with G.new_parts():
        # the rack: a base frame on four feet, two ends each one bent tube, rails along the top
        F.box('rack_base', 0.9, 0.04, 0.36, (0, 0.06, LS_Z), M.iron, bevel=low(0.006, 0), bevel_segments=1)
        for x in (-0.44, 0.44):
            _bent(_name('rack_end'), [(x, 0.0, LS_Z - 0.16), (x, 1.0, LS_Z - 0.16), (x, 1.0, LS_Z + 0.16), (x, 0.0, LS_Z + 0.16)],
                  0.0175, M.iron, bend=0.03, verts=low(8, 4))
            F.box(_name('rack_side'), 0.035, 0.035, 0.36, (x, 0.08, LS_Z), M.iron)
            for z in (LS_Z - 0.16, LS_Z + 0.16):
                F.box(_name('rack_foot'), 0.06, 0.008, 0.05, (x, 0.004, z), M.iron, bevel=low(0.002, 0), bevel_segments=1)
        for z in (LS_Z - 0.16, LS_Z + 0.16):
            F.sweep(_name('rack_top'), [(-0.46, 1.0, z), (0.46, 1.0, z)], 0.0175, M.iron, verts=low(8, 4))
        # the shelter: a pent roof of corrugated iron, sloping to the back, on two bearers
        for x in (-0.44, 0.44):
            _board(_name('bearer'), (x, 1.06, LS_Z + 0.17), (x, 1.0, LS_Z - 0.17), 0.035, 0.035, M.iron, face=(1, 0, 0),
                   bevel=0)
        n = low(14, 1)
        if G.is_low():
            _board('roof_sheet', (0, 1.088, LS_Z + 0.185), (0, 1.03, LS_Z - 0.185), 1.0, 0.012, M.iron, bevel=0)
        else:
            wave = [(-0.5 + i / (4 * n), 0.008 * math.sin(math.pi / 2 * i)) for i in range(4 * n + 1)]
            prof = [(u, v + 0.003) for u, v in wave] + [(u, v - 0.003) for u, v in reversed(wave)]
            F.sweep('roof_sheet', [(0, 1.088, LS_Z + 0.185), (0, 1.03, LS_Z - 0.185)], None, M.iron,
                    profile=[(-u, v) for u, v in prof], up=(1, 0, 0), angle=60)
        F.box('roof_drip', 1.0, 0.04, 0.012, (0, 1.075, LS_Z + 0.185), M.iron)
        # the logs: six rows in brick bond, ends to the front, round and split, bark lumpy; their sawn ends pale with a
        # darker heart
        barks = (M.bark, M.bark_dark, M.bark_light)
        ends = (M.end_grain, M.end_grain_dark)
        for row in range(6):
            for col in range(6 - (row % 2)):
                x = -0.36 + col * 0.145 + (row % 2) * 0.0725
                y = 0.15 + row * 0.13
                r = 0.066 * rnd.uniform(0.92, 1.05)
                spin = rnd.uniform(0, 360)
                kind = ('round', 'round', 'half', 'round', 'quarter')[(row * 5 + col * 3) % 5]
                nv = low(8, 5)
                sec = _log_section(kind, r, random.Random(row * 10 + col), nv)   # its own numbers: the far version draws fewer
                c, s_ = G.cosd(spin), G.sind(spin)
                sec = [(u * c - v * s_, u * s_ + v * c) for u, v in sec]
                F.sweep(_name('log'), [(x, y, LS_Z - 0.17), (x, y, LS_Z + 0.169)], None, barks[(row + col) % 3],
                        profile=[(-u, v) for u, v in sec], up=(0, 0, 1), smooth=False)
                F.extrude(_name('log_end'), [(x + u * 0.94, y + v * 0.94) for u, v in sec], 0.004, (0, 0, LS_Z + 0.171),
                          ends[(row * 7 + col) % 2])
                if kind == 'round' and not G.is_low():
                    F.cyl(_name('log_heart'), r * 0.45, 0.002, (x, y, LS_Z + 0.1735), ends[(row * 7 + col + 1) % 2], rot=(90, 0, 0),
                          verts=6)
    return G.settle([G.join_all('firewoodStack')], centre=False)


FAMILY = G.Family(
    'yard', M,
    [G.Model('sandbox', sandbox, size=(1.98, 0.355, 1.98), key='model.sandbox', far=G.far_of(sandbox),
             motions={'uses': {'sandcastle': 'castle'}}, paint=[('frame', 'Frame')]),
     G.Model('bikeRack', bikeRack, size=(1.8, 0.832, 0.58), centre=(0, -0.07), key='model.bikeRack', far=G.far_of(bikeRack),
             motions={'uses': {'tuneUp': 'wheel_up'}, 'loops': {'tuneUp': 'wheel_turn'}},
             paint=[('rack', 'Rack'), ('bike', 'Bike')]),
     G.Model('raisedBed', raisedBed, size=(1.861, 0.536, 0.88), key='model.raisedBed', far=G.far_of(raisedBed),
             motions={'growth': 'grow', 'uses': {'harvest': {'clip': 'harvest', 'over': 1},
                                                 'tend': {'clip': 'rustle', 'delay': 0.6, 'every': 2.4}}},
             paint=[('bed', 'Bed')]),
     G.Model('shedSmall', shedSmall, size=(1.981, 2.37, 1.913), centre=(0, -0.056), key='model.shedSmall', far=G.far_of(shedSmall),
             motions={'uses': {'potter': 'door_open'}},
             paint=[('shed', 'Wood'), ('roof', 'Roof'), ('shed_door', 'Door')]),
     G.Model('greenhouse', greenhouse, size=(1.805, 2.398, 2.84), key='model.greenhouse', far=G.far_of(greenhouse),
             motions={'uses': {'tend': 'door_open', 'harvest': 'door_open'}},
             paint=[('gh_frame', 'Frame'), ('gh_wall', 'Wall')]),
     G.Model('compostBin', compostBin, size=(0.84, 0.775, 0.792), centre=(0, -0.08), key='model.compostBin', far=G.far_of(compostBin),
             motions={'uses': {'compost': 'lid_open'}, 'hold': 2},
             paint=[('bin', 'Bin')]),
     G.Model('rainBarrel', rainBarrel, size=(0.606, 0.97, 0.686), centre=(0, 0.04), key='model.rainBarrel', far=G.far_of(rainBarrel),
             paint=[('barrel', 'Barrel')]),
     G.Model('clothesline', clothesline, size=(2.76, 1.805, 0.763), centre=(0, 0.031), key='model.clothesline', far=G.far_of(clothesline),
             motions={'uses': {'washing': {'clip': 'washing', 'over': 1}, 'takeIn': {'clip': 'washing', 'over': -1}},
                      'open': 'washing'},
             paint=[('posts', 'Posts'), ('washing', 'Washing')]),
     G.Model('firewoodStack', firewoodStack, size=(1.0, 1.095, 0.377), centre=(0, -0.238), key='model.firewoodStack', far=G.far_of(firewoodStack),
             paint=[('iron', 'Shelter')])],
    comment='Blender-built play and yard things (tools/art/blender/families/yard.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. A sand castle grows in the sandpit, the bike lifts its back wheel and '
            'turns it, the raised bed\'s crop grows and is picked, the shed\'s and the greenhouse\'s doors '
            'open (and the vent), the compost bin\'s lid lifts and the washing goes out on the line and '
            'stays out.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
