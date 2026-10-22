# The hot-headed pack (P8): the heavy punching bag, the plate-smash bin, the thunder drum kit, the cool-down
# meditation cushion, the grudge-to-gratitude writing desk and the glacier cold-plunge tub. Notes: pack_hotHeaded.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the bag swings on its chains while it is pounded (`swing`, a loop), the crate's lid flips up
# (`lid_open`), the crash and ride cymbals rock while the kit is played (`cymbals`, a loop), the zafu sinks
# (`sink_in`), the desk's drawer slides out (`drawer_open`) and the plunge tub's slatted cover rolls back into
# its housing (`cover_off`).
#
# Built with the forms and the kit (2026-10-22): the bag's frame bent from one square tube, chains of links, drums
# with lugs and hoops on tripod stands, a pleated zafu, hairpin desk legs, a jointed lamp, cedar boards in steel
# bands.
#
# Run:  cargo art pack_hotHeaded [--only heavyBag] [--preview]
import math
import os
import random
import sys

from mathutils import Quaternion, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#C49A6C', 'wood'),        # the grudge desk's oak top and its chair; the desk's main part
    bag=('#C8402F', 'satin'),        # the heavy bag's leather
    crate=('#A87547', 'wood'),       # the smash crate and its lid
    shell=('#C8402F', 'gloss'),      # the drum shells
    zafu=('#5F7FA0', 'fabric'),      # the meditation cushion
    cedar=('#C98B58', 'wood'),       # the plunge tub's cladding
    fire=('#C8402F', 'satin'),       # red details: weight plate, gloves, lamp shade
    ember=('#E07A2E', 'satin'),
    flame=('#F2B84B', 'gloss'),      # logos, hazard tape
    steel=('#AEB4BA', 'metal'),
    steel_dark=('#7E858C', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    cymbal=('#C9A24A', 'metal'),
    brass=('#C9A54A', 'metal'),
    dark=('#2F3236', 'satin'),       # straps, castors, the drum throne, pen
    rubber=('#3E4A55', 'satin'),     # floor mats, the drum rug
    rug=('#6E3A2E', 'satin'),
    head=('#F4F1EA', 'satin'),       # drum heads
    porcelain=('#F4F1EA', 'gloss'),  # plates and shards
    plate_blue=('#9CC3DB', 'gloss'),
    plate_gold=('#F2D7A0', 'gloss'),
    plate_green=('#C8E0C0', 'gloss'),
    crate_dark=('#7C5432', 'wood'),  # the crate's posts and floor
    straw=('#D2B872', 'matte'),
    oak=('#B88A5C', 'satin'),        # plain wood: stool, bowl stand, drum sticks, mallet
    calm=('#5F7FA0', 'satin'),       # the chair's cushion
    calm_light=('#A9C2D6', 'fabric'),  # the meditation mat
    calm_deep=('#4E6E8E', 'fabric'),   # the zafu's bands and the mat's edge
    oat=('#E6DCC8', 'fabric'),       # the zabuton
    candle=('#FBF6EC', 'satin'),
    candle_flame=('#FFD27A', 'gloss'),
    leaf=('#5E8F4E', 'satin'),
    pot=('#F4F1EA', 'satin'),
    paper=('#F4EFE2', 'matte'),
    journal=('#7A3A2A', 'satin'),
    drawer=('#E6E3DC', 'satin'),     # the desk's drawer case and the chiller
    cactus=('#6E9E58', 'satin'),
    shell_white=('#F2F2F0', 'satin'),  # the plunge tub's liner and rims
    water=('#BFE6F0', 'gloss'),
    ice=('#F4FAFC', 'gloss'),
    cover=('#5F7FA0', 'satin'),      # the plunge cover's slats
    screen=('#1F3A40', 'gloss'),
    readout=('#7FD1F0', 'gloss'),
    hose=('#5F6368', 'satin'),
    towel=('#F4EFE2', 'satin'),
    amp=('#E4E1DA', 'satin'),
    speaker=('#3A3D41', 'satin'),
    soil=('#4A3A2E', 'matte'),       # the cushion's plant pot and the cactus's
    ash=('#8E8A84', 'matte'),        # the incense's ash
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _turned(F, name, x, y, z, profile, material, segments=16, rot=(0, 0, 0)):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments, rot=rot)


def _lathe(F, name, x, y, z, rings, material, verts=16, square=2.0):
    """A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z)."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(r, r, y + h, verts, square=square, at=(bx, by)) for r, h in rings], material=material)


def _fluted(r, y, verts, at, flutes, depth):
    """A ring of Blender points of radius `r` at height `y`, in and out `depth` (a fraction) `flutes` times round: the
    pleats of a zafu, the ribs of a cactus."""
    return [(at[0] + r * (1 + depth * math.cos(flutes * math.tau * k / verts)) * math.cos(math.tau * k / verts),
             at[1] + r * (1 + depth * math.cos(flutes * math.tau * k / verts)) * math.sin(math.tau * k / verts), y)
            for k in range(verts)]


def _chain(F, name, a, b, material, pitch=0.03, r=0.011, t=0.0028):
    """A chain of oval links from game point `a` to `b`, each turned a quarter to the last."""
    a, b = Vector(F.at(*a)), Vector(F.at(*b))
    d = b - a
    n = max(2, round(d.length / pitch))
    q0 = d.to_track_quat('X', 'Z')
    made = []
    for k in range(n):
        c = a + d * ((k + 0.5) / n)
        link = G.torus(_name(name), r, t, at=tuple(c), material=material, major_segments=8, minor_segments=3)
        link.rotation_mode = 'QUATERNION'
        link.rotation_quaternion = q0 @ Quaternion((1, 0, 0), math.pi / 2 * (k % 2))
        link.scale = (d.length / n / (2 * r) * 1.25, 0.75, 1.0)
        made.append(link)
    return made


# --- the heavy punching bag (1x1): a free-standing frame with a boom, the bag on chains --------------------
HOOK = (0.0, 1.9, 0.1)   # the swivel the bag hangs from (x, height, depth)


def heavyBag():
    F = G.GameFrame()
    hx, hy, hz = HOOK
    sq = G.rounded_rect(0.07, 0.07, 0.014, corner=2)
    with G.new_parts() as still:
        F.box('mat', 0.86, 0.012, 0.86, (0, 0.006, 0), M.rubber, bevel=0.004, bevel_segments=1)
        # the T foot, its ends capped in rubber
        F.sweep('foot_long', [(0, 0.042, -0.42), (0, 0.042, 0.34)], None, M.steel_dark, profile=G.rounded_rect(0.06, 0.08, 0.012, corner=2),
                up=(0, 0, 1))
        F.sweep('foot_cross', [(-0.35, 0.042, -0.37), (0.35, 0.042, -0.37)], None, M.steel_dark,
                profile=G.rounded_rect(0.06, 0.08, 0.012, corner=2), up=(0, 0, 1))
        for x, z in ((0, 0.345), (-0.355, -0.37), (0.355, -0.37)):
            F.box(_name('foot_cap'), 0.084 if x == 0 else 0.012, 0.064, 0.012 if x == 0 else 0.084, (x, 0.042, z), M.dark,
                  bevel=0.004, bevel_segments=1)
        # the upright and the boom bent from one square tube, a gusset in the bend, a round brace to the foot
        F.sweep('frame', G.kit.fillet([(0, 0.07, -0.37), (0, 1.975, -0.37), (0, 1.975, 0.13)], 0.07, steps=4), None, M.steel,
                profile=sq, up=(0, 1, 0))
        for s in (-1, 1):
            F.extrude(_name('gusset'), [(-0.335, 1.94), (-0.2, 1.94), (-0.335, 1.8)], 0.006, (s * 0.038, 0, 0), M.steel_dark,
                      plane='side')
            F.sphere(_name('bolt'), 0.008, (s * 0.041, 1.9, -0.3), M.chrome, scale=(0.5, 1, 1), segments=6, rings=4)
        F.box('boom_cap', 0.074, 0.074, 0.008, (0, 1.975, 0.134), M.dark, bevel=0.003, bevel_segments=1)
        F.sweep('brace', [(0, 0.52, -0.335), (0, 0.075, 0.1)], 0.02, M.steel, verts=10)
        for y, z in ((0.52, -0.335), (0.075, 0.1)):
            F.box(_name('brace_flange'), 0.06, 0.012 if y < 0.1 else 0.06, 0.06 if y < 0.1 else 0.012, (0, y, z), M.steel_dark,
                  bevel=0.003, bevel_segments=1)
        F.cyl('swivel_eye', 0.025, 0.05, (hx, 1.93, hz), M.chrome, verts=12)
        # weight plates on pegs at the foot
        for s, m in ((-1, M.fire), (1, M.dark)):
            G.kit.tube(F, _name('peg'), [(0, 0.072, -0.29), (0, 0.17, -0.29), (s * 0.24, 0.17, -0.29)], 0.015, M.chrome,
                       verts=10, round=1)
            prof = [(0.018, 0.0), (0.13, 0.0), (0.13, 0.045), (0.115, 0.045), (0.105, 0.037), (0.05, 0.037), (0.04, 0.045),
                    (0.018, 0.045), (0.018, 0.0)]
            _turned(F, _name('plate'), s * 0.177 + (s > 0) * 0.045, 0.17, -0.29, prof, m, 22, rot=(0, 90, 0))
        # a hook on the upright with a pair of gloves hanging by their laces
        F.box('glove_hook', 0.2, 0.02, 0.03, (-0.13, 1.32, -0.35), M.steel, bevel=0.006, bevel_segments=1)
        for x in (-0.24, -0.12):
            F.sphere(_name('glove'), 0.07, (x, 1.18, -0.3), M.fire, scale=(0.9, 0.85, 1.1), segments=14, rings=10)
            F.sweep(_name('glove_thumb'), [(x + 0.04, 1.14, -0.27), (x + 0.062, 1.18, -0.265), (x + 0.05, 1.22, -0.27)], 0.022,
                    M.fire, verts=8, resolution=2, scale=[0.9, 1.0, 0.7])
            _turned(F, _name('cuff'), x, 1.04, -0.3, [(0, 0), (0.048, 0), (0.05, 0.03), (0.045, 0.08), (0, 0.08)], M.head, 12)
            F.cyl(_name('strap'), 0.052, 0.026, (x, 1.075, -0.3), M.dark, verts=12)
            G.kit.tube(F, _name('lace'), [(x, 1.1, -0.29), (x + 0.03, 1.24, -0.31), (-0.13, 1.31, -0.34)], 0.004, M.dark,
                       verts=4, round=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bag, its straps, seams and logo, the chains and swivel: one part hung from the swivel, `swing` rocks it
    # back and forth while it is pounded.
    with G.new_parts() as bag:
        _turned(F, 'bag', hx, 0.66, hz, [(0, 0), (0.08, 0.002), (0.14, 0.018), (0.175, 0.05), (0.19, 0.1), (0.19, 0.9),
                                         (0.175, 0.95), (0.14, 0.982), (0.08, 0.998), (0, 1.0)], M.bag, 28)
        for y in (0.86, 1.5):
            _turned(F, _name('strap'), hx, y - 0.025, hz, [(0.189, 0), (0.195, 0.004), (0.195, 0.046), (0.189, 0.05)], M.dark, 28)
        for y in (0.72, 1.6):
            G.torus(_name('seam'), 0.186, 0.003, at=F.at(hx, y, hz), material=M.dark, major_segments=28, minor_segments=3)
        flame = G.smooth_outline([(0.0, -0.1), (0.05, -0.07), (0.055, 0.0), (0.03, 0.05), (0.035, 0.1), (0.0, 0.06),
                                  (-0.02, 0.11), (-0.04, 0.04), (-0.055, -0.01), (-0.045, -0.07)], 2)
        F.extrude('logo', flame, 0.008, (hx, 1.18, hz + 0.188), M.flame)
        _turned(F, 'swivel', hx, 1.84, hz, [(0, 0), (0.012, 0), (0.02, 0.012), (0.02, 0.05), (0.012, 0.06), (0, 0.06)],
                M.chrome, 10)
        for k in range(4):
            a = 45 + 90 * k
            ring = (hx + 0.12 * G.cosd(a), 1.66, hz + 0.12 * G.sind(a))
            G.torus(_name('d_ring'), 0.014, 0.003, at=F.at(*ring), material=M.chrome, major_segments=8, minor_segments=3,
                    rot=(90, 0, -a))
            _chain(F, 'chain', (hx, 1.84, hz), (ring[0], ring[1] + 0.012, ring[2]), M.chrome)
    bag = G.hinged(G.join(bag, 'bag'), F.at(hx, hy, hz))
    objs = G.settle(centre=False)
    G.rock(bag, 'swing', 8, seconds=1.4, axis='X')
    return objs


# --- the plate-smash bin (1x1): a slatted crate with a lid, a stool stacked with old plates ----------------
CRATE = (-0.08, 0.66, 0.6, WALL, 0.56)   # x, width, height, back depth, depth


def _shard(F, name, x, y, z, s, turn, seed, material):
    """A broken piece of plate lying flat: a jagged outline, its glaze side up."""
    rnd = random.Random(seed)
    pts = []
    for k in range(5):
        a = math.tau * k / 5 + rnd.uniform(-0.3, 0.3)
        r = s * rnd.uniform(0.45, 1.0)
        pts.append((r * math.cos(a), r * math.sin(a)))
    o = F.extrude(name, pts, 0.006, (x, y + 0.003, z), material, plane='top')
    o.rotation_euler[2] = math.radians(turn)
    return o


def smashBin():
    F = G.GameFrame()
    rnd = random.Random(4)
    x, w, h, z0, d = CRATE
    z1, zc = z0 + d, z0 + d / 2
    with G.new_parts() as still:
        F.box('mat', 0.84, 0.012, 0.84, (0, 0.006, -0.01), M.rubber, bevel=0.004, bevel_segments=1)
        F.box('floor', w - 0.04, 0.03, d - 0.04, (x, 0.04, zc), M.crate_dark)
        straw = G.boulder('straw', 0.3, at=F.at(x, 0.05, zc), material=M.straw, scale=(1.0, 0.8, 0.5), seed=2.0,
                          noise=0.15, subdivisions=2)
        G.jitter(straw, 0.012, size=0.05, seed=1.0)
        for px in (x - w / 2 + 0.03, x + w / 2 - 0.03):
            for pz in (z0 + 0.03, z1 - 0.03):
                F.box(_name('post'), 0.06, h, 0.06, (px, h / 2, pz), M.crate_dark, bevel=0.006, bevel_segments=1)
        sh = (h - 0.02) / 3
        for k in range(3):
            y = 0.01 + sh * (k + 0.5)
            for zz in (z1 - 0.011, z0 + 0.011):
                F.box(_name('slat'), w - 0.02, sh - 0.025, 0.022, (x, y + rnd.uniform(-0.004, 0.004), zz), M.crate,
                      rot=(0, rnd.uniform(-0.8, 0.8), 0))
            for xx in (x - w / 2 + 0.011, x + w / 2 - 0.011):
                F.box(_name('slat'), 0.022, sh - 0.025, d - 0.02, (xx, y + rnd.uniform(-0.004, 0.004), zc), M.crate,
                      rot=(rnd.uniform(-0.8, 0.8), 0, 0))
        # steel corner brackets at the top, the lid's hinges on the back
        for px in (x - w / 2 + 0.03, x + w / 2 - 0.03):
            for pz in (z0 + 0.03, z1 - 0.03):
                F.box(_name('bracket'), 0.062, 0.06, 0.06, (px + math.copysign(0.001, px - x), h - 0.04, pz), M.steel_dark, bevel=0.002, bevel_segments=1)
        # hazard tape across the front, a stencilled arrow
        F.box('tape', 0.6, 0.05, 0.004, (x, 0.36, z1 + 0.002), M.dark)
        for i in range(5):
            F.box(_name('tape_stripe'), 0.06, 0.05, 0.005, (x - 0.24 + i * 0.12, 0.36, z1 + 0.003), M.flame,
                  rot=(0, 0, 30))
        F.extrude('arrow', [(0.0, 0.05), (0.04, 0.0), (0.015, 0.0), (0.015, -0.05), (-0.015, -0.05), (-0.015, 0.0), (-0.04, 0.0)],
                  0.003, (x, 0.5, z1 + 0.002), M.dark)
        # broken plates on the straw and the mat
        for k, (dx, dz, r) in enumerate(((-0.12, -0.05, 20), (0.1, 0.06, -35), (0.02, -0.12, 70), (-0.05, 0.1, 140))):
            _shard(F, _name('shard'), x + dx, 0.19 + 0.01 * k, zc + dz, 0.05, r, k, M.porcelain if k % 2 else M.plate_blue)
        for k, (sx, sz, r, sr) in enumerate(((-0.3, 0.32, 20, 0.04), (-0.15, 0.36, -40, 0.04), (0.06, 0.408, 70, 0.05),
                                             (-0.38, 0.24, 10, 0.04))):
            _shard(F, _name('shard'), sx, 0.012, sz, sr, r, 10 + k, (M.porcelain, M.plate_gold)[k % 2])
        # the stool and its stack of thrift-store plates
        sx, sz = 0.28, 0.25
        _turned(F, 'stool_top', sx, 0.43, sz, [(0, 0), (0.14, 0), (0.15, 0.008), (0.15, 0.024), (0.14, 0.03), (0, 0.03)], M.oak,
                20)
        for a in (90, 210, 330):
            F.sweep(_name('stool_leg'), [(sx + 0.08 * G.cosd(a), 0.43, sz + 0.08 * G.sind(a)),
                                         (sx + 0.13 * G.cosd(a), 0.0, sz + 0.13 * G.sind(a))], 0.016, M.oak, verts=8,
                    scale=[1.0, 0.75])
        G.torus('stool_ring', 0.108, 0.008, at=F.at(sx, 0.15, sz), material=M.oak, major_segments=16, minor_segments=4)
        y = 0.46
        for i, m in enumerate((M.porcelain, M.plate_blue, M.porcelain, M.plate_gold, M.porcelain, M.plate_green)):
            _turned(F, _name('plate'), sx + rnd.uniform(-0.008, 0.008), y, sz + rnd.uniform(-0.008, 0.008),
                    [(0, 0), (0.065, 0), (0.12, 0.013), (0.112, 0.018), (0.06, 0.006), (0, 0.006)], m, 18)
            y += 0.02
        # safety goggles hung on the crate's front corner post: two lenses in a frame, the strap round the post
        gx, gz = x + w / 2 - 0.03, z1 + 0.035
        F.soft('goggles', 0.16, 0.055, 0.04, (gx + 0.03, 0.5, gz), M.flame, round=0.018, round_segments=1, flat=1)
        for s in (-1, 1):
            F.box(_name('lens'), 0.055, 0.032, 0.004, (gx + 0.03 + s * 0.037, 0.5, gz + 0.021), M.porcelain, bevel=0.01,
                  bevel_segments=1)
        G.kit.tube(F, 'goggle_strap', [(gx + 0.1, 0.505, gz - 0.005), (gx + 0.04, 0.56, gz - 0.04), (gx - 0.035, 0.56, gz - 0.05),
                                       (gx - 0.04, 0.505, gz - 0.005)], 0.006, M.dark, verts=4, round=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The lid: three boards on two battens, a rope grip, hinged on the crate's back top edge: `lid_open` flips it up.
    lz0 = z0 + 0.03
    with G.new_parts() as lid:
        bd = (z1 - lz0) / 3
        for k in range(3):
            F.box(_name('lid_board'), w + 0.02, 0.022, bd - 0.006, (x, h + 0.011, lz0 + bd * (k + 0.5)), M.crate,
                  bevel=0.004, bevel_segments=1, rot=(0, (k - 1) * 0.6, 0))
        for xx in (x - w / 2 + 0.08, x + w / 2 - 0.08):
            F.box(_name('lid_batten'), 0.05, 0.02, z1 - lz0 - 0.04, (xx, h + 0.032, (lz0 + z1) / 2), M.crate_dark,
                  bevel=0.004, bevel_segments=1)
        F.box('lid_grip', 0.16, 0.03, 0.035, (x, h + 0.015, z1 + 0.015), M.crate_dark, bevel=0.01, bevel_segments=1)
    lid = G.hinged(G.join(lid, 'lid'), F.at(x, h, lz0))
    objs = G.settle(centre=False)
    G.swing(lid, 'lid_open', -80, seconds=0.6, axis='X')
    return objs


# --- the thunder drum kit (2x1): the drummer on the throne at the middle, the kit before them --------------
def _drum(F, x, y, z, dia, depth, axis='y', tilt=0.0, lugs=6):
    """A drum shell with chrome hoops, white heads and lugs: axis 'y' a snare or tom (tilted `tilt` about x), 'z'
    the bass drum lying towards the front."""
    r = dia / 2
    rot = (90, 0, 0) if axis == 'z' else (tilt, 0, 0)
    F.cyl(_name('shell'), r, depth, (x, y, z), M.shell, rot=rot, verts=24)
    F.cyl(_name('head'), r * 0.97, depth + 0.006, (x, y, z), M.head, rot=rot, verts=24, smooth=False)

    def along(t, u=0.0, v=0.0):
        """The game point `t` along the drum's axis from its middle, `u` across in x, `v` out along its other side."""
        if axis == 'z':
            return (x + u, y + v, z + t)
        c, s = G.cosd(tilt), G.sind(tilt)          # the shell turned about x: its axis (height, depth) is (c, s)
        return (x + u, y + t * c - v * s, z + t * s + v * c)
    for e in (-1, 1):
        G.torus(_name('hoop'), r + 0.008, 0.009, at=F.at(*along(e * depth / 2)), material=M.chrome, rot=rot, major_segments=20,
                minor_segments=3)
    for k in range(lugs):
        a = math.tau * (k + 0.5) / lugs
        u, v = (r + 0.008) * math.cos(a), (r + 0.008) * math.sin(a)
        G.kit.tube(F, _name('lug'), [along(-depth * 0.2, u, v), along(depth * 0.2, u, v)], 0.006, M.chrome, verts=4, round=1)


def _stand(F, x, z, h, spread=0.17, top=0.011):
    """A chrome tripod stand: three legs braced to a collar, a tube up to height `h` with a clamp halfway."""
    G.kit.tube(F, _name('stand_pole'), [(x, 0.1, z), (x, h * 0.55, z)], 0.011, M.chrome, verts=8, round=1)
    G.kit.tube(F, _name('stand_upper'), [(x, h * 0.55, z), (x, h - 0.02, z)], 0.008, M.chrome, verts=8, round=1)
    F.cyl(_name('stand_clamp'), 0.015, 0.024, (x, h * 0.55, z), M.chrome, verts=6)
    F.cyl(_name('stand_hub'), 0.017, 0.03, (x, 0.22, z), M.chrome, verts=6)
    for k in range(3):
        a = 90 + 120 * k
        c, s = G.cosd(a), G.sind(a)
        G.kit.tube(F, _name('stand_leg'), [(x, 0.22, z), (x + spread * c, 0.012, z + spread * s)], 0.0065, M.chrome, verts=6,
                   round=1)
        G.kit.tube(F, _name('stand_brace'), [(x, 0.13, z), (x + spread * 0.5 * c, 0.115, z + spread * 0.5 * s)], 0.004,
                   M.chrome, verts=4, round=1)
        F.cyl(_name('stand_foot'), 0.011, 0.014, (x + spread * c, 0.007, z + spread * s), M.dark, verts=5)


def _cymbal(F, name, x, y, z, d):
    """A cymbal centred on (x, y, z): a domed bell rising out of a bow that thins to its edge, a felt and its nut."""
    r = d / 2
    G.revolve(name, [(0, 0.03), (0.035, 0.027), (0.048, 0.014), (r * 0.6, 0.005), (r, 0.0), (r, -0.003), (r * 0.6, 0.002),
                     (0.048, 0.01), (0.035, 0.022), (0, 0.025)], at=F.at(x, y, z), material=M.cymbal, segments=20)
    F.cyl(_name('felt'), 0.014, 0.022, (x, y + 0.042, z), M.dark, verts=8)
    F.cyl(_name('wingnut'), 0.006, 0.014, (x, y + 0.058, z), M.chrome, verts=6)
    F.box(_name('wing'), 0.03, 0.01, 0.003, (x, y + 0.062, z), M.chrome)


def _pedal(F, x, z, length=0.2):
    """A foot pedal along the depth, its heel at the back: a heel plate, a footboard tilting up, its frame and cam."""
    z0, z1 = z - length / 2, z + length / 2
    F.box(_name('pedal_heel'), 0.075, 0.012, 0.06, (x, 0.018, z0 + 0.03), M.chrome, bevel=0.003, bevel_segments=1)
    F.sweep(_name('pedal_board'), [(x, 0.026, z0 + 0.05), (x, 0.06, z1 - 0.03)], None, M.dark,
            profile=G.rounded_rect(0.008, 0.065, 0.003, corner=1), scale=[0.85, 1.1])
    F.box(_name('pedal_base'), 0.11, 0.01, 0.06, (x, 0.017, z1), M.chrome, bevel=0.003, bevel_segments=1)
    for s in (-1, 1):
        G.kit.tube(F, _name('pedal_post'), [(x + s * 0.05, 0.02, z1), (x + s * 0.05, 0.13, z1)], 0.006, M.chrome, verts=6,
                   round=1)
    F.cyl(_name('pedal_cam'), 0.02, 0.1, (x, 0.12, z1), M.chrome, rot=(0, 90, 0), verts=10)


CRASH = (-0.5, 1.22, 0.22, 0.4)   # x, height, depth, diameter
RIDE = (0.66, 1.12, 0.18, 0.44)


def thunderDrums():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('rug', 1.88, 0.012, 0.9, (0, 0.006, 0), M.rug, bevel=0.004, bevel_segments=1)
        F.box('rug_band', 1.8, 0.0135, 0.82, (0, 0.00675, 0), M.ember)
        F.box('rug_field', 1.74, 0.015, 0.76, (0, 0.0075, 0), M.rug)
        # the bass drum, its port and logo, spurs; the pedal and beater on its batter head
        _drum(F, 0.0, 0.29, 0.27, 0.52, 0.34, axis='z', lugs=8)
        F.cyl('bass_logo', 0.1, 0.004, (0.0, 0.29, 0.443), M.flame, rot=(90, 0, 0), verts=20, smooth=False)
        F.cyl('bass_port', 0.045, 0.004, (0.12, 0.2, 0.443), M.speaker, rot=(90, 0, 0), verts=14, smooth=False)
        for s in (-1, 1):
            G.kit.tube(F, _name('spur'), G.kit.fillet([(s * 0.22, 0.16, 0.36), (s * 0.27, 0.1, 0.38), (s * 0.3, 0.012, 0.4)], 0.03),
                       0.008, M.chrome, verts=6, round=1)
            F.cyl(_name('spur_foot'), 0.011, 0.014, (s * 0.3, 0.007, 0.4), M.dark, verts=6)
        _pedal(F, 0.0, 0.02)
        G.kit.tube(F, 'beater', [(0.0, 0.12, 0.12), (0.0, 0.27, 0.11)], 0.004, M.chrome, verts=5, round=1)
        F.cyl('beater_head', 0.022, 0.03, (0.0, 0.28, 0.108), M.head, rot=(90, 0, 0), verts=10)
        # the rack toms on their mount, the floor tom on its legs, the snare on its stand
        _drum(F, -0.15, 0.68, 0.24, 0.26, 0.18, tilt=-18)
        _drum(F, 0.15, 0.68, 0.24, 0.28, 0.2, tilt=-18)
        F.box('tom_clamp', 0.05, 0.03, 0.05, (0.0, 0.555, 0.27), M.chrome, bevel=0.006, bevel_segments=1)
        G.kit.tube(F, 'tom_mount', [(0.0, 0.55, 0.27), (0.0, 0.64, 0.26)], 0.013, M.chrome, verts=8, round=1)
        for s in (-1, 1):
            G.kit.tube(F, _name('tom_arm'), G.kit.fillet([(0.0, 0.64, 0.26), (s * 0.09, 0.64, 0.26), (s * 0.09, 0.66, 0.255)], 0.02),
                       0.008, M.chrome, verts=6, round=1)
        _drum(F, 0.42, 0.43, 0.12, 0.38, 0.34, lugs=8)
        for a in (30, 150, 270):
            G.kit.tube(F, _name('tom_leg'), G.kit.fillet([(0.42 + 0.205 * G.cosd(a), 0.46, 0.12 + 0.205 * G.sind(a)),
                                                          (0.42 + 0.23 * G.cosd(a), 0.4, 0.12 + 0.23 * G.sind(a)),
                                                          (0.42 + 0.24 * G.cosd(a), 0.012, 0.12 + 0.24 * G.sind(a))], 0.02),
                       0.007, M.chrome, verts=6, round=1)
            F.cyl(_name('tom_foot'), 0.011, 0.014, (0.42 + 0.24 * G.cosd(a), 0.007, 0.12 + 0.24 * G.sind(a)), M.dark, verts=6)
        _drum(F, -0.32, 0.6, 0.14, 0.34, 0.14, tilt=-6, lugs=8)
        _stand(F, -0.32, 0.14, 0.53, spread=0.16)
        F.box('snare_strainer', 0.02, 0.045, 0.025, (-0.32 + 0.18, 0.6, 0.14), M.chrome, bevel=0.004, bevel_segments=1)
        # the hi-hat, a pair of cymbals on its stand with a pedal
        _stand(F, -0.66, 0.06, 0.9)
        _cymbal(F, 'hihat_low', -0.66, 0.82, 0.06, 0.32)
        _cymbal(F, 'hihat_top', -0.66, 0.855, 0.06, 0.32)
        _pedal(F, -0.66, -0.06, length=0.18)
        # the crash's and the ride's stands
        _stand(F, CRASH[0], CRASH[2], CRASH[1])
        _stand(F, RIDE[0], RIDE[2], RIDE[1])
        # the throne, top 0.47: a round padded seat on a tripod; a pair of sticks on it
        _lathe(F, 'throne_seat', 0, 0.4, 0.0, [(0.15, 0.0), (0.175, 0.012), (0.182, 0.035), (0.178, 0.055), (0.16, 0.066),
                                              (0.1, 0.07), (0.0, 0.07)], M.dark, verts=24)
        F.cyl('throne_base', 0.06, 0.02, (0, 0.39, 0.0), M.chrome, verts=12)
        _stand(F, 0.0, 0.0, 0.4, spread=0.2)
        for dz, r in ((0.03, 70), (0.0, 80)):              # across the floor tom's head
            a = math.radians(r)
            F.sweep(_name('stick'), [(0.42 - 0.2 * math.sin(a), 0.611, 0.12 + dz - 0.2 * math.cos(a)),
                                     (0.42 + 0.2 * math.sin(a), 0.611, 0.12 + dz + 0.2 * math.cos(a))], 0.007, M.oak,
                    verts=6, scale=[1.0, 0.55])
        # the monitor amp at the back: a cabinet with corner caps, its grille cloth in a frame, the speaker, knobs and a
        # handle on top
        ax, ay, az = 0.78, 0.3, -0.28
        F.box('amp', 0.34, 0.6, 0.3, (ax, ay, az), M.amp, bevel=0.02, bevel_segments=2)
        F.box('amp_grille', 0.28, 0.4, 0.008, (ax, 0.27, az + 0.149), M.speaker, bevel=0.006, bevel_segments=1)
        _turned(F, 'amp_cone', ax, 0.3, az + 0.153, [(0.11, 0), (0.1, 0.004), (0.05, 0.012), (0.03, 0.012), (0, 0.02)],
                M.dark, 20, rot=(-90, 0, 0))
        F.sphere('amp_dust_cap', 0.035, (ax, 0.3, az + 0.163), M.fire, scale=(1, 0.5, 1), segments=12, rings=6)
        F.box('amp_panel', 0.3, 0.07, 0.006, (ax, 0.53, az + 0.15), M.speaker, bevel=0.003, bevel_segments=1)
        for k in range(4):
            F.cyl(_name('amp_knob'), 0.011, 0.016, (ax - 0.09 + 0.06 * k, 0.53, az + 0.158), M.chrome, rot=(90, 0, 0), verts=8)
        F.sphere('amp_light', 0.006, (ax + 0.13, 0.53, az + 0.155), M.fire, segments=6, rings=4)
        G.kit.tube(F, 'amp_handle', G.kit.fillet([(ax - 0.08, 0.6, az), (ax - 0.08, 0.63, az), (ax + 0.08, 0.63, az),
                                                  (ax + 0.08, 0.6, az)], 0.015), 0.008, M.dark, verts=6, round=1)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(_name('amp_corner'), 0.04, 0.04, 0.04, (ax + sx * 0.155, 0.02, az + sz * 0.135), M.dark)
    G.join(still, 'body', origin=(0, 0, 0))
    # The crash and the ride, each tilted on its stand's tip: `cymbals` rocks both while the kit is played.
    moving = []
    for name, (x, y, z, dia), tilt in (('crash', CRASH, 14), ('ride', RIDE, 10)):
        with G.new_parts() as c:
            _cymbal(F, name, x, 0.0, 0.0, dia)
        part = G.join(c, name, origin=F.at(x, 0.0, 0.0))
        part.location = F.at(x, y, z)
        part.rotation_euler = (math.radians(tilt), 0, 0)   # tipped towards the room
        moving.append(part)
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.rock(moving[0], 'cymbals', 9, seconds=0.8, axis='X')
    G.rock(moving[1], 'cymbals', -6, seconds=0.8, axis='X')
    return objs


# --- the cool-down meditation cushion (1x1): a round mat, a zabuton and a tall zafu, a singing bowl ---------
def coolDownCushion():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('mat', 0.45, 0.012, (0, 0.006, 0), M.calm_light, verts=48)
        G.torus('mat_edge', 0.44, 0.009, at=F.at(0, 0.012, 0), material=M.calm_deep, major_segments=48, minor_segments=3)
        # the zabuton: a plump mat, tufted at four points, piped round
        F.soft('zabuton', 0.62, 0.062, 0.58, (0, 0.044, -0.02), M.oat, round=0.03, puff=0.008, flat=2)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.sphere(_name('tuft'), 0.012, (sx * 0.2, 0.082, -0.02 + sz * 0.19), M.oat, scale=(1, 1, 0.4), segments=6,
                         rings=4)
        # the singing bowl on its ring cushion and stand, the mallet beside it
        _turned(F, 'bowl_stand', 0.32, 0.012, 0.3, [(0, 0), (0.1, 0), (0.11, 0.008), (0.105, 0.022), (0.09, 0.026), (0, 0.026)],
                M.oak, 16)
        G.torus('bowl_ring', 0.055, 0.014, at=F.at(0.32, 0.045, 0.3), material=M.fire, major_segments=14, minor_segments=6)
        _turned(F, 'singing_bowl', 0.32, 0.045, 0.3, [(0, 0), (0.05, 0), (0.08, 0.02), (0.1, 0.06), (0.102, 0.08), (0.096, 0.082),
                                                      (0.092, 0.064), (0.075, 0.03), (0.045, 0.012), (0, 0.012)], M.brass, 20)
        F.sweep('mallet', [(0.2, 0.026, 0.3), (0.07, 0.04, 0.385)], 0.01, M.oak, verts=8, scale=[0.8, 1.1])
        F.cyl('mallet_felt', 0.017, 0.06, (0.09, 0.037, 0.372), M.calm, rot=(90, 0, 57), verts=10, bevel=0.005,
              bevel_segments=1)
        # a candle on its dish, a potted plant and an incense stick on a leaf-shaped holder
        _turned(F, 'candle_dish', -0.32, 0.012, 0.3, [(0, 0), (0.06, 0), (0.066, 0.008), (0.062, 0.012), (0.046, 0.006), (0, 0.006)],
                M.brass, 16)
        _turned(F, 'candle', -0.32, 0.018, 0.3, [(0, 0), (0.04, 0), (0.04, 0.064), (0.037, 0.07), (0.022, 0.064), (0, 0.062)],
                M.candle, 14)
        F.sweep('wick', [(-0.32, 0.08, 0.3), (-0.319, 0.094, 0.3)], 0.0018, M.dark, verts=4)
        F.sphere('candle_flame', 0.011, (-0.319, 0.108, 0.3), M.candle_flame, scale=(1, 1, 2.0), segments=8, rings=5)
        _turned(F, 'plant_pot', -0.3, 0.0, -0.32, [(0, 0), (0.045, 0), (0.052, 0.07), (0.062, 0.075), (0.062, 0.1), (0.054, 0.1),
                                                   (0.05, 0.092), (0, 0.092)], M.pot, 16)
        for k in range(7):
            a = k * 51 + 10
            c, s = G.cosd(a), G.sind(a)
            L = 0.07 + 0.039 * ((k * 5) % 3) / 2
            F.sweep(_name('leaf'), [(-0.3 + 0.01 * c, 0.09, -0.32 + 0.01 * s), (-0.3 + 0.35 * L * c, 0.09 + L * 1.1, -0.32 + 0.35 * L * s),
                                    (-0.3 + 0.8 * L * c, 0.09 + L * 1.25 + 0.01, -0.32 + 0.8 * L * s)], None, M.leaf,
                    profile=G.rounded_rect(0.003, 0.034, 0.001, corner=1), up=(0, 0, 1), resolution=2, scale=[0.3, 1.0, 0.15])
        F.extrude('incense_holder', G.smooth_outline([(-0.06, 0.0), (0.0, 0.022), (0.06, 0.0), (0.0, -0.022)], 2), 0.008,
                  (0.3, 0.016, -0.3), M.brass, plane='top', rot=(0, 0, 40))
        F.sweep('incense', [(0.3, 0.02, -0.3), (0.33, 0.21, -0.33)], 0.0028, M.ember, verts=4)
        F.sweep('incense_ash', [(0.328, 0.2, -0.328), (0.332, 0.225, -0.332)], 0.0032, M.ash, verts=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The zafu, a real one's size (0.38 across, 10 cm on the 8 cm zabuton): its side gathered in pleats into the
    # buttoned middle of its top, a carrying loop, origin on its underside: `sink_in` presses it to the seat height
    # (0.17), sat on cross-legged (a seat under 0.2 m: the floor clips). 2026-10-21: it was a 0.45 m pouf (M-packs-12).
    bx, by, _ = F.at(0, 0, 0)
    with G.new_parts() as zafu:
        y0 = 0.08
        rings = [G.ring(0.15, 0.15, y0, 40, at=(bx, by)), _fluted(0.18, y0 + 0.01, 40, (bx, by), 10, 0.03),
                 _fluted(0.19, y0 + 0.035, 40, (bx, by), 20, 0.035), _fluted(0.192, y0 + 0.06, 40, (bx, by), 20, 0.035),
                 _fluted(0.186, y0 + 0.085, 40, (bx, by), 20, 0.03), _fluted(0.15, y0 + 0.1, 40, (bx, by), 10, 0.05),
                 _fluted(0.08, y0 + 0.104, 40, (bx, by), 10, 0.08), G.ring(0.02, 0.02, y0 + 0.098, 40, at=(bx, by))]
        G.loft('zafu', rings, material=M.zafu)
        G.torus('zafu_piping', 0.18, 0.006, at=F.at(0, y0 + 0.095, 0), material=M.calm_deep, major_segments=24, minor_segments=3)
        F.sphere('zafu_button', 0.02, (0, y0 + 0.1, 0), M.calm_deep, scale=(1, 1, 0.45), segments=10, rings=5)
        G.kit.tube(F, 'zafu_loop', [(-0.06, y0 + 0.07, 0.19), (0.0, y0 + 0.055, 0.205), (0.06, y0 + 0.07, 0.19)], 0.006,
                   M.calm_deep, verts=5, round=2)
    zafu = G.hinged(G.join(zafu, 'zafu'), F.at(0, 0.08, 0))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.squash(zafu, 'sink_in', (1.03, 1.03, 0.86), seconds=0.6)
    return objs


# --- the grudge-to-gratitude writing desk (1x1): the writer sits in the middle, the desk before them -------
def _hairpin(F, name, x, z0, z1, top, material, splay=0.03):
    """A hairpin leg: one steel rod from under the top at depth z0, down to a point on the floor splayed `splay` out
    in x, and back up to z1."""
    sx = math.copysign(splay, x)
    zm = (z0 + z1) / 2
    pts = [(x, top, z0), (x + sx, 0.01, zm - 0.012), (x + sx, 0.004, zm), (x + sx, 0.01, zm + 0.012), (x, top, z1)]
    return F.sweep(name, G.kit.fillet(pts, 0.012, steps=2), 0.0065, material, verts=6)


def grudgeJournal():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.84, 0.03, 0.3, (0, 0.73, 0.3), M.body, bevel=0.008, bevel_segments=2)
        for x in (-0.37, 0.37):
            _hairpin(F, _name('leg'), x, 0.18, 0.42, 0.715, M.steel_dark)
            F.box(_name('leg_plate'), 0.05, 0.006, 0.27, (x, 0.712, 0.3), M.steel_dark)
        # the drawer's case, hung under the top, open towards the writer
        F.box('case_top', 0.36, 0.012, 0.28, (0.22, 0.709, 0.3), M.drawer)
        F.box('case_floor', 0.36, 0.012, 0.28, (0.22, 0.561, 0.3), M.drawer, bevel=0.003, bevel_segments=1)
        for x in (0.045, 0.395):
            F.box(_name('case_side'), 0.012, 0.16, 0.28, (x, 0.635, 0.3), M.drawer, bevel=0.003, bevel_segments=1)
        F.box('case_back', 0.36, 0.16, 0.012, (0.22, 0.635, 0.434), M.drawer)
        # the open journal with its ribbon, the pen, crumpled pages, a waste bin, a ribbed cactus and a red desk lamp
        F.box('journal_cover', 0.36, 0.008, 0.24, (-0.02, 0.749, 0.28), M.journal, bevel=0.003, bevel_segments=1)
        for s in (-1, 1):
            pts = [(-0.02 + s * 0.165 * k / 6, 0.756 + 0.009 * math.sin(math.pi / 2 * min(1.0, k / 3)) ** 0.6 - 0.002 * (k == 6),
                    0.28) for k in range(7)]
            prof = G.rounded_rect(0.008, 0.214, 0.003, corner=1)
            F.sweep(_name('page'), [(px, py, pz) for px, py, pz in pts], None, M.paper, profile=prof, up=(0, 0, 1))
        F.box('ribbon', 0.008, 0.002, 0.2, (-0.015, 0.765, 0.25), M.fire, rot=(0, 0, 4))
        F.sweep('pen', [(0.13, 0.757, 0.21), (0.21, 0.757, 0.27)], 0.006, M.dark, verts=8, scale=[0.5, 1.0])
        F.box('pen_clip', 0.03, 0.002, 0.004, (0.19, 0.763, 0.256), M.steel, rot=(0, 0, -37))
        for k, (x, y, z, r) in enumerate(((-0.3, 0.78, 0.22, 0.035), (-0.25, 0.775, 0.36, 0.03), (0.3, 0.035, 0.3, 0.035),
                                          (-0.36, 0.02, 0.08, 0.03))):
            c = G.boulder(_name('crumple'), r, at=F.at(x, y - r, z), material=M.paper, seed=x * 10, noise=0.35, subdivisions=1)
            G.jitter(c, r * 0.18, size=r * 0.6, seed=k)
        _turned(F, 'bin', -0.3, 0, 0.3, [(0, 0), (0.085, 0), (0.1, 0.25), (0.106, 0.26), (0.094, 0.26), (0.08, 0.008), (0, 0.008)],
                M.steel, 20)
        G.boulder('bin_paper', 0.05, at=F.at(-0.31, 0.21, 0.3), material=M.paper, seed=3.0, noise=0.35, subdivisions=1)
        _turned(F, 'cactus_pot', 0.3, 0.745, 0.38, [(0, 0), (0.04, 0), (0.05, 0.07), (0.056, 0.072), (0.056, 0.085), (0.048, 0.085),
                                                    (0, 0.08)], M.ember, 14)
        cbx, cby, _ = F.at(0.3, 0, 0.38)
        G.loft('cactus', [_fluted(0.03, 0.82, 32, (cbx, cby), 8, 0.12), _fluted(0.037, 0.86, 32, (cbx, cby), 8, 0.12),
                          _fluted(0.036, 0.91, 32, (cbx, cby), 8, 0.12), _fluted(0.026, 0.945, 32, (cbx, cby), 8, 0.12),
                          _fluted(0.01, 0.962, 32, (cbx, cby), 8, 0.1)], material=M.cactus)
        F.sphere('cactus_flower', 0.011, (0.3, 0.965, 0.38), M.ember, scale=(1, 1, 0.6), segments=8, rings=4)
        # the lamp: a weighted foot, two arms jointed with knobs, a deep conical shade
        lx, lz = 0.33, 0.18
        _turned(F, 'lamp_foot', lx, 0.745, lz, [(0, 0), (0.055, 0), (0.058, 0.008), (0.05, 0.016), (0.012, 0.02), (0, 0.02)], M.dark, 16)
        G.kit.tube(F, 'lamp_arm', [(lx, 0.76, lz), (0.3, 0.975, 0.24)], 0.007, M.dark, verts=6, round=1)
        G.kit.tube(F, 'lamp_arm2', [(0.3, 0.975, 0.24), (0.225, 0.96, 0.2)], 0.007, M.dark, verts=6, round=1)
        for (jx, jy, jz) in ((lx, 0.765, lz), (0.3, 0.975, 0.24), (0.225, 0.96, 0.2)):
            F.cyl(_name('lamp_joint'), 0.012, 0.024, (jx, jy, jz), M.steel_dark, rot=(0, 90, 0), verts=8)
        G.kit.tube(F, 'lamp_spring', [(lx + 0.012, 0.8, lz + 0.01), (0.312, 0.94, 0.232)], 0.003, M.steel, verts=4, round=1)
        _turned(F, 'lamp_shade', 0.2, 0.905, 0.19, [(0.062, 0), (0.058, 0.004), (0.05, 0.022), (0.026, 0.078), (0.02, 0.088), (0, 0.09)],
                M.fire, 18, rot=(25, 0, -20))
        # the chair, the writer's seat at 0.45: a padded seat, a curved back rail, tapered legs and stretchers
        F.box('chair_seat', 0.42, 0.04, 0.4, (0, 0.405, 0.0), M.body, bevel=0.01, bevel_segments=2)
        F.soft('chair_pad', 0.38, 0.03, 0.36, (0, 0.438, 0.01), M.calm, round=0.014, puff=0.004, flat=2)
        for dx in (-0.18, 0.18):
            F.sweep(_name('front_leg'), [(dx, 0.39, 0.16), (dx, 0.0, 0.165)], None, M.body,
                    profile=G.rounded_rect(0.034, 0.034, 0.008, corner=1), up=(0, 0, 1), scale=[1.0, 0.72])
            F.sweep(_name('back_leg'), [(dx, 0.0, -0.165), (dx, 0.4, -0.18), (dx, 0.9, -0.2)], None, M.body,
                    profile=G.rounded_rect(0.034, 0.034, 0.008, corner=1), up=(0, 0, 1), resolution=2, scale=[0.75, 1.0, 0.9])
            G.kit.tube(F, _name('stretcher'), [(dx, 0.14, 0.16), (dx, 0.14, -0.168)], 0.008, M.body, verts=6, round=1)
        rail = [(x, 0.74 + 0.012 * (1 - (x / 0.2) ** 2), -0.185 - 0.02 * (1 - (x / 0.2) ** 2)) for x in
                (-0.2, -0.13, -0.065, 0.0, 0.065, 0.13, 0.2)]
        F.sweep('chair_back', rail, None, M.calm, profile=G.rounded_rect(0.26, 0.04, 0.016, corner=2), up=(0, 0, 1))
        F.sweep('chair_back_rail', [(x, y - 0.155, z) for x, y, z in rail], None, M.body,
                profile=G.rounded_rect(0.03, 0.022, 0.008, corner=1), up=(0, 0, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    # The drawer: its front and pull towards the writer, the tray and the letters in it; `drawer_open` slides it out
    # towards the writer.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.34, 0.13, 0.016, (0.22, 0.635, 0.162), M.drawer, bevel=0.004, bevel_segments=1)
        F.box('drawer_inset', 0.3, 0.09, 0.004, (0.22, 0.635, 0.153), M.drawer, bevel=0.002, bevel_segments=1)
        pts = [(0.16, 0.66, 0.153), (0.16, 0.66, 0.13), (0.28, 0.66, 0.13), (0.28, 0.66, 0.153)]   # a bar pull towards the writer
        F.sweep('drawer_pull', G.kit.fillet(pts, 0.009), 0.005, M.steel, verts=6)
        F.box('drawer_tray', 0.31, 0.01, 0.25, (0.22, 0.577, 0.29), M.drawer)
        for x in (0.07, 0.37):
            F.box(_name('drawer_side'), 0.008, 0.08, 0.25, (x, 0.615, 0.29), M.drawer)
        F.box('drawer_back', 0.31, 0.08, 0.008, (0.22, 0.615, 0.41), M.drawer)
        for k, (dx, dz, r) in enumerate(((-0.05, 0.25, 5), (0.04, 0.3, -8), (0.0, 0.34, 3))):
            F.box(_name('letter'), 0.16, 0.012, 0.11, (0.22 + dx, 0.588 + 0.012 * k, dz), M.paper, rot=(0, 0, r))
            F.box(_name('letter_flap'), 0.15, 0.002, 0.04, (0.22 + dx, 0.595 + 0.012 * k, dz - 0.03), M.journal, rot=(0, 0, r))
    drawer = G.join(drawer, 'drawer')
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, 0.17, 0), seconds=0.6)   # game -z: out towards the writer
    return objs


# --- the glacier cold-plunge tub (1x2): the bather sits in it at the middle; the chiller at the back --------
TUB = (-0.55, 0.45, 0.62)   # back depth, front depth, rim height


def coldPlunge():
    F = G.GameFrame()
    zb, zf, h = TUB
    zc, d = (zb + zf) / 2, zf - zb
    rnd = random.Random(7)
    with G.new_parts() as still:
        F.box('plinth', 0.86, 0.06, d + 0.02, (0, 0.03, zc), M.hose, bevel=0.01, bevel_segments=1)
        # the liner: walls round the water, so the bather sits down into it
        F.box('liner_floor', 0.72, 0.04, d - 0.12, (0, 0.08, zc), M.shell_white)
        for x in (-0.395, 0.395):
            F.box(_name('wall'), 0.05, h - 0.06, d - 0.02, (x, 0.06 + (h - 0.06) / 2, zc), M.shell_white)
        for z in (zb + 0.025, zf - 0.025):
            F.box(_name('wall'), 0.74, h - 0.06, 0.05, (0, 0.06 + (h - 0.06) / 2, z), M.shell_white)
        # cedar cladding, eight boards round the outside, each a hair proud or shy of the next; corner posts
        for i in range(8):
            y = 0.1 + i * 0.065
            j = rnd.uniform(-0.0015, 0.0015)
            F.box(_name('board_side'), 0.022, 0.058, d - 0.03, (-0.43 + 0.011 + j, y, zc), M.cedar, bevel=0.003, bevel_segments=1)
            F.box(_name('board_side'), 0.022, 0.058, d - 0.03, (0.43 - 0.011 - j, y, zc), M.cedar, bevel=0.003, bevel_segments=1)
            F.box(_name('board_front'), 0.81, 0.058, 0.022, (0, y, zf + 0.001 + j), M.cedar, bevel=0.003, bevel_segments=1)
            F.box(_name('board_back'), 0.81, 0.058, 0.022, (0, y, zb - 0.001 - j), M.cedar, bevel=0.003, bevel_segments=1)
        for x in (-0.409, 0.409):
            for z in (zb + 0.009, zf - 0.009):
                F.box(_name('corner'), 0.042, 0.53, 0.042, (x, 0.335, z), M.cedar, bevel=0.004, bevel_segments=1)
        # two steel bands round the cladding
        for y in (0.2, 0.5):
            loop = [(u, v + zc) for u, v in G.rounded_rect(0.865, 2 * (zf + 0.0145 - zc), 0.03, corner=2)]
            F.sweep(_name('band'), [(u, y, v) for u, v in loop], None, M.steel, profile=G.rounded_rect(0.03, 0.004, 0.001, corner=1),
                    up=(0, 0, 1), closed=True)
        # the rim, one moulded piece round the top
        loop = [(u, v + zc) for u, v in G.rounded_rect(0.8, d - 0.05, 0.04, corner=3)]
        F.sweep('rim', [(u, h + 0.005, v) for u, v in loop], None, M.shell_white,
                profile=G.rounded_rect(0.03, 0.07, 0.01, corner=1), up=(0, 0, 1), closed=True)
        # the icy water and floating ice
        F.box('water', 0.72, 0.012, d - 0.1, (0, h - 0.03, zc), M.water)
        for x, z, r in ((-0.2, 0.15, 10), (0.15, -0.3, 40), (0.22, 0.25, -20), (-0.1, -0.2, 70), (0.05, 0.05, 15),
                        (-0.25, -0.38, 30)):
            F.box(_name('ice'), 0.06, 0.045, 0.06, (x, h - 0.022, z), M.ice, rot=(8, 5, r), bevel=0.012, bevel_segments=1)
        # the cover's roller housing on the back rim, end caps, a slot for the cover
        F.box('housing', 0.86, 0.1, 0.12, (0, h + 0.05, zb + 0.05), M.cedar, bevel=0.012, bevel_segments=2)
        for x in (-0.424, 0.424):
            F.cyl(_name('housing_cap'), 0.048, 0.012, (x, h + 0.05, zb + 0.05), M.hose, rot=(0, 90, 0), verts=14)
        F.box('housing_lip', 0.8, 0.02, 0.01, (0, h + 0.03, zb + 0.112), M.dark)
        # the chiller behind: a slotted grille, a display with buttons, hoses arching into the tub; a towel and a bottle
        F.box('chiller', 0.7, 0.6, 0.29, (0, 0.3, -0.78), M.drawer, bevel=0.025, bevel_segments=2)
        slots = [G.rounded_rect(0.52, 0.018, 0.009, corner=2) for _ in range(6)]
        slots = [[(u, v - 0.13 + 0.052 * k) for u, v in s] for k, s in enumerate(slots)]
        F.extrude('grille', G.rounded_rect(0.6, 0.36, 0.012, corner=2), 0.008, (0, 0.28, -0.634), M.steel, holes=slots)
        F.box('grille_back', 0.58, 0.34, 0.004, (0, 0.28, -0.639), M.dark)
        F.box('display', 0.16, 0.07, 0.008, (0.2, 0.52, -0.632), M.screen, bevel=0.006, bevel_segments=1)
        F.box('readout', 0.1, 0.025, 0.004, (0.2, 0.52, -0.627), M.readout)
        for k in range(3):
            F.cyl(_name('button'), 0.009, 0.006, (-0.04 + 0.03 * k, 0.52, -0.634), M.readout if k == 0 else M.dark,
                  rot=(90, 0, 0), verts=8)
        for x in (-0.22, 0.0):
            G.kit.tube(F, _name('hose'), G.kit.fillet([(x, 0.5, -0.64), (x, 0.66, -0.62), (x, 0.66, zb - 0.03),
                                                       (x, 0.56, zb - 0.01)], 0.04), 0.022, M.hose, verts=10, round=1)
            F.cyl(_name('hose_clamp'), 0.027, 0.014, (x, 0.5, -0.648), M.steel, rot=(90, 0, 0), verts=10)
        for sx in (-1, 1):
            F.box(_name('chiller_foot'), 0.06, 0.012, 0.25, (sx * 0.3, 0.006, -0.78), M.dark)
        F.soft('towel', 0.36, 0.04, 0.2, (0.17, 0.62, -0.79), M.towel, round=0.016, puff=0.006, flat=2)
        F.box('towel_band', 0.364, 0.01, 0.03, (0.17, 0.62, -0.72), M.cover)
        _lathe(F, 'bottle', -0.22, 0.6, -0.79, [(0.045, 0), (0.048, 0.006), (0.048, 0.17), (0.04, 0.195), (0.026, 0.21),
                                               (0.022, 0.24), (0.0, 0.24)], M.calm, verts=14)
        F.cyl('bottle_cap', 0.024, 0.02, (-0.22, 0.85, -0.79), M.dark, verts=10, bevel=0.004, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The slatted cover, rolled out over the water from its housing, origin at the housing: `cover_off`
    # rolls it back in.
    cz0 = zb + 0.11
    with G.new_parts() as cover:
        n = 14
        sl = (zf - 0.03 - cz0) / n
        for k in range(n):
            F.box(_name('slat'), 0.8, 0.024, sl - 0.004, (0, h + 0.03, cz0 + sl * (k + 0.5)), M.cover, bevel=0.006,
                  bevel_segments=1)
        F.box('cover_pull', 0.24, 0.025, 0.03, (0, h + 0.05, zf - 0.04), M.dark, bevel=0.01, bevel_segments=1)
    cover = G.join(cover, 'cover', origin=F.at(0, h + 0.03, cz0))
    objs = G.settle(centre=False)
    G.squash(cover, 'cover_off', (1, 0.04, 1), seconds=0.9)
    return objs


FAMILY = G.Family(
    'pack_hotHeaded', M,
    [G.Model('heavyBag', heavyBag, size=(0.86, 2.01, 0.86), key='model.hotHeaded.heavyBag', centre=(0, 0.0),
             motions={'loops': {'pound': 'swing'}},
             paint=[('bag', 'Body')]),
     G.Model('smashBin', smashBin, size=(0.854, 0.643, 0.876), key='model.hotHeaded.smashBin', centre=(0.003, 0.008),
             # the lid stands open while plates are thrown in, and shuts when they're done (2026-10-21: after 1 s)
             motions={'uses': {'smash': 'lid_open'}},
             paint=[('crate', 'Body')]),
     G.Model('thunderDrums', thunderDrums, size=(1.89, 1.279, 0.9), key='model.hotHeaded.thunderDrums', centre=(0.005, 0.02),
             motions={'loops': {'drum': 'cymbals'}},
             paint=[('shell', 'Body')]),
     G.Model('coolDownCushion', coolDownCushion, size=(0.9, 0.236, 0.9), centre=(0, 0.02), key='model.hotHeaded.coolDownCushion',
             motions={'uses': {'countToTen': 'sink_in', 'breathe': 'sink_in'}},
             paint=[('zafu', 'Body')]),
     G.Model('grudgeJournal', grudgeJournal, size=(0.84, 1.003, 0.677), key='model.hotHeaded.grudgeJournal', centre=(0, 0.111),
             motions={'uses': {'writeItOut': 'drawer_open'}},
             paint=[('body', 'Body')]),
     G.Model('coldPlunge', coldPlunge, size=(0.86, 0.86, 1.39), key='model.hotHeaded.coldPlunge', centre=(0, -0.23),
             motions={'uses': {'plunge': 'cover_off'}},
             paint=[('cedar', 'Body')])],
    comment='Blender-built hot-headed pack models (tools/art/blender/families/pack_hotHeaded.py). Listed '
            'in the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs '
            'one draw call per finish group. Each has its moving part: the bag swings while pounded, the '
            'crate\'s lid flips up, the cymbals rock while the kit is played, the zafu sinks, the desk '
            'drawer slides out and the plunge tub\'s slatted cover rolls back.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
