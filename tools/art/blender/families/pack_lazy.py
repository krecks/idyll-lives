# The lazy pack (P10): the snoozecloud adjustable bed, the daydream nap pod, the heated quilt table, the doorstep
# delivery locker, the effortless vibration plate and the smart-home command hub.
# Notes: pack_lazy.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the bed's head section rises (`head_up`), the pod's hood swings half over (`hood`), the quilt's edge
# lifts (`quilt`), a locker door opens (`door_open`), the plate's deck shakes (`shake`, a loop) and the tablet
# tilts up on its dock (`screen_up`).
#
# Built with the forms and the kit (2026-10-22): soft mattresses, pillows and cushions with piping, a cloud
# headboard cut from one outline, the pod's egg and a piped hood, a quilt hanging in folds to the floor, lockers with
# vents and recessed pulls, a handlebar bent from one tube, a mid-century side table on splayed legs.
#
# Run:  cargo art pack_lazy [--only napPod] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit.shapes import _recalc_normals  # noqa: E402

M = G.Palette(
    body=('#E9E4F4', 'satin'),        # the bed's puffy base and cloud headboard; the main part
    upholstery=('#F4F2FA', 'fabric'),  # the bed's mattress
    pillow=('#FFFFFF', 'fabric'),
    throw=('#CFC6EA', 'fabric'),
    blush=('#F2C6CF', 'fabric'),
    lilac=('#CFC6EA', 'fabric'),
    cloud=('#F7F7FA', 'satin'),
    birch=('#E3CFA8', 'satin'),
    shell=('#F4F4F2', 'gloss'),       # the nap pod's egg
    seat=('#7E95B8', 'fabric'),       # the nap pod's cushions and hood
    hood=('#8FA6C6', 'fabric'),
    shag=('#E6DCCB', 'fabric'),
    chrome=('#D0D3D6', 'chrome'),
    oak=('#CDB08A', 'satin'),         # the quilt table's top, plain (the quilt takes the fabric draw)
    quilt=('#C96A5B', 'fabric'),      # the kotatsu quilt
    quilt_band=('#E8B07A', 'fabric'),
    pouf=('#E8D7B8', 'fabric'),
    pouf_top=('#F3E7CF', 'fabric'),
    sage=('#B9C7A8', 'fabric'),
    heater=('#5A2A24', 'satin'),      # under the quilt
    porcelain=('#F2F2F0', 'gloss'),
    mandarin=('#F28C28', 'gloss'),
    biscuit=('#E8C27A', 'satin'),
    locker=('#EEF2F0', 'satin'),      # the delivery locker's cabinet
    door=('#CFE7DC', 'satin'),        # its doors
    accent=('#F28C28', 'satin'),      # its roof band; the plate's grips
    steel=('#8A9096', 'metal'),
    bezel=('#2B3440', 'satin'),
    screen=('#2E7DB8', 'gloss'),
    screen_light=('#3C8DBF', 'gloss'),
    white=('#F2F2F0', 'gloss'),
    led_green=('#7FB800', 'gloss'),
    cubby=('#3A3F44', 'satin'),
    kraft=('#C8A27A', 'matte'),
    tape=('#E9DFC8', 'matte'),
    bag=('#F2EEE6', 'matte'),
    bag_logo=('#D7263D', 'matte'),
    plate=('#F2F2F0', 'gloss'),       # the vibration plate's base and column
    lime=('#9BD45A', 'satin'),
    pad=('#3D4A57', 'matte'),
    pad_inner=('#4C5B6A', 'matte'),
    readout=('#6CC3F0', 'gloss'),
    bottle=('#7FC4E8', 'gloss'),
    wood=('#B07A4E', 'wood'),         # the smart hub's side table
    speaker=('#BFC3C7', 'satin'),
    ring=('#4FD1C5', 'gloss'),
    app_yellow=('#F2C14E', 'gloss'),
    leaf=('#6E9E58', 'satin'),        # the mandarins' leaves
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _turned(F, name, x, y, z, profile, material, segments=16, rot=(0, 0, 0)):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments, rot=rot)


def _lathe(F, name, x, y, z, rings, material, verts=16, square=2.0, sy=1.0):
    """A body lofted through rings [(radius, height up), ...] standing at game (x, y, z); `sy` squashes them in depth."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(r, r * sy, y + h, verts, square=square, at=(bx, by)) for r, h in rings], material=material)


def _mug(F, name, x, y, z, r, h, material, turn=0.0):
    """A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the front)."""
    _turned(F, name, x, y, z, [(0.0, 0.0), (0.88 * r, 0.0), (r, 0.06 * h), (r, h), (0.86 * r, h), (0.86 * r, 0.15 * h),
                               (0.0, 0.15 * h)], material, 14)
    c, s = G.cosd(turn), G.sind(turn)
    F.sweep(f'{name}_handle', [(x + c * d, y + t * h, z + s * d) for d, t in ((r - 0.003, 0.8), (r + 0.016, 0.82),
                                                                              (r + 0.024, 0.55), (r + 0.015, 0.27),
                                                                              (r - 0.003, 0.25))], 0.0045, material,
            verts=5, resolution=2)


def _sphere_band(name, F, centre, r_in, r_out, sides, pitch, material, n_side=14, n_pitch=10):
    """A closed shell of a sphere about `centre` (game): points at side angle `sides` (from the middle towards
    ±x) and pitch `pitch` (from the top, negative towards the back), between radii `r_in` and `r_out`. A
    pram-hood shape; turning it about the centre's x axis keeps it on its sphere."""
    a0, a1 = (math.radians(a) for a in sides)
    p0, p1 = (math.radians(p) for p in pitch)
    cx, cy, cz = centre
    verts, faces = [], []

    def grid(r):
        base = len(verts)
        for i in range(n_side + 1):
            a = a0 + (a1 - a0) * i / n_side
            for j in range(n_pitch + 1):
                p = p0 + (p1 - p0) * j / n_pitch
                verts.append(F.at(cx + r * math.sin(a), cy + r * math.cos(a) * math.cos(p), cz + r * math.cos(a) * math.sin(p)))
        return base

    def idx(base, i, j):
        return base + i * (n_pitch + 1) + j

    o, n = grid(r_out), grid(r_in)
    for i in range(n_side):
        for j in range(n_pitch):
            faces.append((idx(o, i, j), idx(o, i + 1, j), idx(o, i + 1, j + 1), idx(o, i, j + 1)))
            faces.append((idx(n, i, j + 1), idx(n, i + 1, j + 1), idx(n, i + 1, j), idx(n, i, j)))
    for i in range(n_side):                       # the two pitch edges
        for j in (0, n_pitch):
            faces.append((idx(o, i, j), idx(n, i, j), idx(n, i + 1, j), idx(o, i + 1, j)))
    for j in range(n_pitch):                      # the two side edges
        for i in (0, n_side):
            faces.append((idx(o, i, j), idx(o, i, j + 1), idx(n, i, j + 1), idx(n, i, j)))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    _recalc_normals(obj)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return obj


# --- the snoozecloud adjustable bed (2x2): two sleepers side by side, mattress top 0.49, head at the back ----
HINGE_Z = -0.42     # the head section's front edge (depth), where it rises from
MAT_TOP = 0.49


def _cloud(w, h, bumps, base=0.0):
    """A cloud's outline `w` across and `h` high above `base`: a row of round bumps (centre x, height, radius) along its
    top, flat along its foot."""
    top = []
    for k in range(49):
        x = -w / 2 + w * k / 48
        y = max([cy + math.sqrt(r * r - (x - cx) ** 2) for cx, cy, r in bumps if abs(x - cx) < r] + [base + 0.02])
        top.append((x, y))
    return [(-w / 2, base)] + top + [(w / 2, base)]


def snoozecloudBed():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the base: one puffy upholstered block, piped round its top, on four turned birch feet
        F.soft('base', 1.86, 0.26, 1.86, (0, 0.15, 0), M.body, round=0.1, puff=(0.0, 0.0, 0.0), piping=0.008, flat=1)
        for sx in (-1, 1):
            for sz in (-1, 1):
                _turned(F, _name('foot'), sx * 0.8, 0, sz * 0.8, [(0, 0), (0.045, 0), (0.05, 0.012), (0.06, 0.035), (0.062, 0.05),
                                                                  (0, 0.05)], M.birch, 12)
        # the cloud headboard: a big cloud cut from one outline, a smaller one in front of it in the other white
        bumps = [(-0.66, 0.62, 0.24), (-0.36, 0.78, 0.3), (0.0, 0.86, 0.32), (0.36, 0.78, 0.3), (0.66, 0.62, 0.24),
                 (-0.5, 0.98, 0.2), (0.48, 0.98, 0.22), (0.0, 1.08, 0.21)]
        out = G.smooth_outline(_cloud(1.82, 1.29, [(x, y, r) for x, y, r in bumps], base=0.3), 1)
        F.extrude('cloud_back', out, 0.12, (0, 0, -0.86), M.body, bevel=0.045, bevel_segments=3, smooth=True, angle=60)
        front = [(-0.42, 0.62, 0.2), (-0.12, 0.76, 0.24), (0.22, 0.72, 0.22), (0.5, 0.6, 0.18)]
        out2 = G.smooth_outline(_cloud(1.2, 0.0, front, base=0.5), 1)
        F.extrude('cloud_front', out2, 0.05, (0, 0, -0.776), M.cloud, bevel=0.022, bevel_segments=2, smooth=True, angle=60)
        # the foot of the mattress, quilted, a throw folded over its end, a remote on the side
        d = 0.93 - HINGE_Z
        F.soft('mattress_foot', 1.74, 0.2 - 0.016, d - 0.01, (0, MAT_TOP - 0.1 - 0.008, HINGE_Z + d / 2 + 0.005), M.upholstery,
               round=0.05, puff=0.016, piping=0.006, flat=3)
        throw = F.soft('throw', 1.78, 0.036, 0.42, (0, MAT_TOP + 0.018, 0.66), M.throw, round=0.016, puff=0.006, flat=2)
        G.jitter(throw, 0.004, size=0.15, seed=2.0, up=0.6)
        F.sweep('throw_fold', [(-0.89, MAT_TOP + 0.02, 0.87), (0.89, MAT_TOP + 0.02, 0.87)], 0.02, M.throw, verts=10)
        F.box('remote', 0.06, 0.02, 0.14, (0.78, MAT_TOP + 0.016, 0.3), M.porcelain, bevel=0.008, bevel_segments=2)
        for k in range(3):
            F.cyl(_name('remote_button'), 0.008, 0.004, (0.78, MAT_TOP + 0.027, 0.26 + 0.025 * k), M.screen, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The head section of the mattress and its pillows, hinged on its front edge: `head_up` raises it 30°.
    with G.new_parts() as head:
        hd = HINGE_Z + 0.77
        F.soft('mattress_head', 1.74, 0.2 - 0.016, hd, (0, MAT_TOP - 0.1 - 0.008, HINGE_Z - hd / 2), M.upholstery, round=0.05,
               puff=0.016, piping=0.006, flat=3)
        for s in (-1, 1):
            F.soft(_name('pillow'), 0.68, 0.14, 0.36, (s * 0.42, MAT_TOP + 0.06, -0.6), M.pillow, round=0.06, puff=0.03, flat=3)
            F.soft(_name('cushion'), 0.22, 0.26, 0.1, (s * 0.77, MAT_TOP + 0.15, -0.68), M.blush if s < 0 else M.lilac,
                   rot=(18, 0, 0), round=0.045, puff=0.025, flat=2)
    head = G.hinged(G.join(head, 'head_section'), F.at(0, MAT_TOP - 0.2, HINGE_Z))
    objs = G.settle(centre=False)
    G.swing(head, 'head_up', 30, seconds=1.2, axis='X')
    return objs


# --- the daydream nap pod (1x1): an egg chair open to the front, seat top 0.45, a hood that swings over -----
POD = (0.0, 0.82, 0.09)    # the egg's centre (x, height, depth)
POD_R, POD_H = 0.44, 0.66  # its radius across and its half height (a grown-up sitter's crown reaches about 1.4)
HOOD = (0.0, 1.05, 0.09)   # the hood's centre: a sphere about the egg's upper half, clear of a grown-up's head
HOOD_R = 0.5      # 2026-10-21: 0.48 about 0.95, the hood closing onto the head (A-packs-04)


def napPod():
    F = G.GameFrame()
    cx, cy, cz = POD
    with G.new_parts() as still:
        # the egg: an ellipsoid open at the front, revolved about the depth axis and stretched upwards, its lip rounded
        prof = []
        for k in range(19):
            t = math.radians(100 * k / 18)
            prof.append((POD_R * math.sin(t), -POD_R * math.cos(t)))
        ri = POD_R - 0.035
        prof.append(((POD_R - 0.0175) * math.sin(math.radians(102)), -(POD_R - 0.0175) * math.cos(math.radians(102))))
        for k in range(18, -1, -1):
            t = math.radians(100 * k / 18)
            prof.append((ri * math.sin(t), -ri * math.cos(t)))
        G.revolve('egg', prof, at=F.at(cx, cy, cz), material=M.shell, segments=36, rot=(90, 0, 0),
                  scale=(1, POD_H / POD_R, 1))
        # its stem and swivel base
        _turned(F, 'stem', 0, 0.035, cz, [(0, 0), (0.07, 0), (0.06, 0.04), (0.045, 0.2), (0.06, 0.24), (0, 0.24)], M.chrome, 20)
        _turned(F, 'base_disc', 0, 0, cz, [(0, 0), (0.27, 0), (0.28, 0.01), (0.27, 0.03), (0.2, 0.04), (0, 0.04)], M.chrome, 36)
        # the cushions: a round seat (top 0.45), a tufted back cushion in the egg's curve, a little pillow
        _lathe(F, 'seat', 0, 0.35, 0.1, [(0.27, 0.0), (0.29, 0.015), (0.3, 0.05), (0.295, 0.08), (0.27, 0.096), (0.18, 0.1),
                                         (0.0, 0.1)], M.seat, verts=32)
        G.torus('seat_piping', 0.293, 0.006, at=F.at(0, 0.43, 0.1), material=M.seat, major_segments=32, minor_segments=4)
        F.sphere('back_cushion', 0.3, (0, 0.84, -0.21), M.seat, scale=(1, 0.3, 1.3), segments=20, rings=12)
        for i, (u, v) in enumerate(((0, 0.18), (-0.12, 0.05), (0.12, 0.05), (0, -0.08), (-0.12, -0.2), (0.12, -0.2), (0, 0.32))):
            depth = 0.09 * math.sqrt(max(0.0, 1 - (u / 0.3) ** 2 - (v / 0.39) ** 2))
            F.sphere(_name('button'), 0.012, (u, 0.84 + v, -0.21 + depth - 0.004), M.hood, scale=(1, 0.5, 1), segments=6,
                     rings=4)
        F.soft('pillow', 0.26, 0.2, 0.12, (0.17, 0.56, -0.01), M.blush, rot=(-20, 0, 20), round=0.05, puff=0.035, flat=2)
        # a shaggy rug in front, its pile jittered
        rug = F.cyl('rug', 0.27, 0.016, (0, 0.008, 0.33), M.shag, verts=32, scale=(1, 0.55, 1), bevel=0.006, bevel_segments=1)
        G.jitter(rug, 0.003, size=0.03, seed=1.0, up=0.6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The hood: a band of a sphere about the egg's upper half, folded back over the egg's top and turning about
    # that sphere's centre, piped along its front edge; `hood` swings it 60° forward, half over the opening.
    with G.new_parts() as hood:
        _sphere_band('hood', F, HOOD, HOOD_R, HOOD_R + 0.02, (-62, 62), (-108, -20), M.hood, n_side=18, n_pitch=12)
        hx, hy, hz = HOOD
        r = HOOD_R + 0.01
        edge = []
        for i in range(13):
            a = math.radians(-62 + 124 * i / 12)
            p = math.radians(-20)
            edge.append((hx + r * math.sin(a), hy + r * math.cos(a) * math.cos(p), hz + r * math.cos(a) * math.sin(p)))
        F.sweep('hood_piping', edge, 0.014, M.seat, verts=8)
        for k in range(1, 4):                                  # the ribs' seams across the hood
            p = math.radians(-20 - 22 * k)
            seam = [(hx + (r + 0.011) * math.sin(math.radians(a)), hy + (r + 0.011) * math.cos(math.radians(a)) * math.cos(p),
                     hz + (r + 0.011) * math.cos(math.radians(a)) * math.sin(p)) for a in range(-60, 61, 12)]
            F.sweep(_name('hood_seam'), seam, 0.004, M.seat, verts=4)
    hood = G.hinged(G.join(hood, 'hood'), F.at(*HOOD))
    objs = G.settle(centre=False)   # built in place: the seat under the sitter, the hood over their head
    G.swing(hood, 'hood', 60, seconds=1.0, axis='X')
    return objs


# --- the heated quilt table (2x1): a kotatsu in front, two poufs (tops 0.45) at x = ±0.5 behind it ---------
TOP_Y = 0.6        # the underside of the table top
TZ0, TZ1 = -0.03, 0.47   # the top's back and front edges (depth): 0.5 deep, so the sitters' legs and feet go under it
FLAP_Z = TZ0 - 0.012     # the quilt's back edge, hung over the sitters' laps
SEAT_Z = -0.2      # the poufs' middles (depth): the hips there (`furniture.json` `lazy.quiltTable`)


def _drape(F, name, a, b, y0, y1, out, material, folds=6, amp=0.012, t=0.022):
    """A curtain of quilt from height y1 down to y0 along the line from game (x, depth) `a` to `b`, hanging `out` past
    it (outwards, to its left going from a to b), in `folds` soft folds."""
    (ax, az), (bx, bz) = a, b
    L = math.hypot(bx - ax, bz - az)
    ux, uz = (bx - ax) / L, (bz - az) / L
    nx, nz = -uz, ux                      # outwards
    n = folds * 4
    pts = []
    for k in range(n + 1):
        f = k / n
        w = out + amp * (0.5 - 0.5 * math.cos(math.tau * folds * f)) * min(1.0, 4 * f, 4 * (1 - f))
        pts.append((ax + ux * L * f + nx * w, (y0 + y1) / 2, az + uz * L * f + nz * w))
    prof = G.rounded_rect(y1 - y0, t, t * 0.45, corner=1)
    return F.sweep(name, pts, None, material, profile=prof, up=(0, 0, 1))


def quiltTable():
    F = G.GameFrame()
    tz, td = (TZ0 + TZ1) / 2, TZ1 - TZ0
    with G.new_parts() as still:
        F.box('top', 1.74, 0.035, td, (0, TOP_Y + 0.0175, tz), M.oak, bevel=0.012, bevel_segments=2)
        for x in (-0.82, 0.82):                                       # the legs, inside the quilt
            for z in (TZ0 + 0.05, TZ1 - 0.05):
                F.box(_name('leg'), 0.05, TOP_Y, 0.05, (x, TOP_Y / 2, z), M.oak)
        F.box('heater', 0.36, 0.05, 0.3, (0, TOP_Y - 0.025, tz), M.heater, bevel=0.01, bevel_segments=1)   # under the top
        # the quilt, puffed over the top and hanging in folds down its front and ends to the floor: hollow, the legs go
        # in from the back under its flap (2026-10-21: a solid block 0.4 deep, the feet out of its front: I-packs-01)
        F.soft('quilt', 1.8, 0.03, td + 0.03, (0, TOP_Y + 0.05, tz), M.quilt, round=0.015, puff=0.008, flat=2)
        for i in range(1, 6):                                          # its quilting, stitched in rows
            F.box(_name('quilt_seam'), 0.004, 0.004, td + 0.02, (-0.9 + 0.3 * i, TOP_Y + 0.069, tz), M.quilt_band)
        _drape(F, 'quilt_front', (-0.879, TZ1 - 0.006), (0.879, TZ1 - 0.006), 0.01, TOP_Y + 0.065, 0.008, M.quilt, folds=7)
        for s in (-1, 1):
            a, b = ((-0.879, TZ0 - 0.005), (-0.879, TZ1 - 0.006)) if s < 0 else ((0.879, TZ1 - 0.006), (0.879, TZ0 - 0.005))
            _drape(F, _name('quilt_end'), a, b, 0.01, TOP_Y + 0.065, 0.008, M.quilt, folds=3)
        _drape(F, 'quilt_band', (-0.879, TZ1 - 0.006), (0.879, TZ1 - 0.006), 0.015, 0.045, 0.01, M.quilt_band, folds=7, t=0.026)
        # a bowl of mandarins, two mugs, a plate of biscuits on the quilt's top
        ty = TOP_Y + 0.067
        _turned(F, 'bowl', 0, ty, 0.25, [(0, 0), (0.05, 0), (0.06, 0.008), (0.12, 0.055), (0.13, 0.065), (0.122, 0.067),
                                         (0.055, 0.016), (0, 0.016)], M.porcelain, 22)
        for dx, dz, dy in ((-0.05, 0.22, 0), (0.05, 0.27, 0), (0.0, 0.31, 0), (-0.02, 0.2, 0.04), (0.04, 0.2, 0.0)):
            F.sphere(_name('mandarin'), 0.035, (dx, ty + 0.035 + dy, dz), M.mandarin, scale=(1, 1, 0.85), segments=12, rings=8)
        F.sphere('mandarin_leaf', 0.016, (-0.02, ty + 0.1, 0.21), M.leaf, scale=(1.6, 0.6, 0.2), rot=(0, 15, 30), segments=6,
                 rings=4)
        for s in (-1, 1):
            x = s * 0.5
            _mug(F, _name('mug'), x + s * 0.12, ty, 0.12, 0.042, 0.09, M.porcelain, turn=90 - 70 * s)
            # the pouf, its seat cushion (top 0.45) and a back cushion against the wall
            F.soft(_name('pouf'), 0.5, 0.38, 0.44, (x, 0.2, SEAT_Z), M.pouf, round=0.12, puff=(0.03, 0.0, 0.008), flat=3)
            F.soft(_name('pouf_top'), 0.46, 0.05, 0.42, (x, 0.42, SEAT_Z), M.pouf_top, round=0.022, puff=0.008, piping=0.005,
                   flat=2)
            F.soft(_name('back_cushion'), 0.44, 0.3, 0.064, (x, 0.6, WALL + 0.05), M.sage if s < 0 else M.lilac,
                   rot=(5, 0, 0), round=0.03, puff=0.012, flat=2)
        _turned(F, 'snack_plate', 0.6, ty, 0.32, [(0, 0), (0.06, 0), (0.1, 0.01), (0.105, 0.013), (0.06, 0.006), (0, 0.006)],
                M.porcelain, 18)
        for dx, dz in ((-0.02, -0.01), (0.03, 0.02), (0.0, 0.035)):
            _turned(F, _name('biscuit'), 0.6 + dx, ty + 0.006, 0.32 + dz, [(0, 0), (0.026, 0.002), (0.022, 0.012), (0, 0.012)],
                    M.biscuit, 10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The quilt's back edge, hung from the table top: `quilt` lifts it over the sitters' laps.
    with G.new_parts() as flap:
        F.soft('quilt_flap', 1.62, 0.29, 0.025, (0, TOP_Y - 0.1, FLAP_Z), M.quilt, round=0.01, puff=0.006, flat=2)
        F.sweep('flap_band', [(-0.81, TOP_Y - 0.235, FLAP_Z), (0.81, TOP_Y - 0.235, FLAP_Z)], None, M.quilt_band,
                profile=G.rounded_rect(0.03, 0.03, 0.008, corner=1), up=(0, 0, 1))
        F.sweep('flap_hem', [(-0.81, TOP_Y + 0.03, FLAP_Z), (0.81, TOP_Y + 0.03, FLAP_Z)], None, M.quilt_band,
                profile=G.rounded_rect(0.03, 0.028, 0.008, corner=1), up=(0, 0, 1))
    flap = G.hinged(G.join(flap, 'quilt_flap'), F.at(0, TOP_Y + 0.05, FLAP_Z))
    objs = G.settle(centre=False)   # built in place: the poufs under the sitters (`furniture.json`), the back on the wall
    G.swing(flap, 'quilt', 70, seconds=0.8, axis='X')
    return objs


# --- the doorstep delivery locker (1x1): a bank of parcel lockers with a touch screen, an order waiting -----
COLS = (-0.28, 0.0, 0.28)
ROWS = ((0.34, 0.5), (0.88, 0.5), (1.42, 0.5))   # (middle, height)
OPEN = (0, 1)                                    # the door that opens: left column, middle row
FRONT = 0.0


def _locker_door(F, name, x, y, h, number, led):
    """A locker door, its face pressed with a vent of slots, a recessed pull at its right, a status light and a number
    plate. Returns nothing; all in the current parts."""
    slots = [[(u, v + 0.06 * k - 0.03) for u, v in G.rounded_rect(0.14, 0.012, 0.006, corner=2)] for k in range(2)]
    vent = [[(u, v + h / 2 - 0.1) for u, v in s] for s in slots]
    F.extrude(name, G.rounded_rect(0.26, h - 0.03, 0.01, corner=2), 0.02, (x, y, FRONT + 0.01), M.door, holes=vent,
              bevel=0.004, bevel_segments=1)
    F.box(f'{name}_vent_back', 0.15, 0.08, 0.004, (x, y + h / 2 - 0.1, FRONT + 0.002), M.cubby)
    F.box(f'{name}_pull', 0.022, 0.1, 0.012, (x + 0.1, y, FRONT + 0.024), M.steel, bevel=0.005, bevel_segments=1)
    F.box(f'{name}_pull_well', 0.014, 0.08, 0.004, (x + 0.1, y, FRONT + 0.0305), M.cubby)
    F.sphere(f'{name}_led', 0.008, (x - 0.09, y + h / 2 - 0.07, FRONT + 0.021), led, scale=(1, 0.5, 1), segments=8, rings=4)
    F.box(f'{name}_plate', 0.05, 0.026, 0.003, (x - 0.07, y - h / 2 + 0.06, FRONT + 0.0215), M.white, bevel=0.001,
          bevel_segments=1)
    F.box(f'{name}_number', 0.012 + 0.004 * (number % 3), 0.014, 0.0015, (x - 0.07, y - h / 2 + 0.06, FRONT + 0.0235), M.bezel)


def deliveryLocker():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('cabinet', 0.86, 1.86, -WALL, (0, 0.93, WALL / 2), M.locker, bevel=0.02, bevel_segments=2)
        F.box('plinth', 0.82, 0.06, 0.02, (0, 0.03, FRONT + 0.008), M.bezel)
        for c, x in enumerate(COLS):
            for r, (y, h) in enumerate(ROWS):
                if (c, r) == (1, 1):
                    continue
                if (c, r) == OPEN:
                    # the open locker: a lined cubby, a shelf, the takeout bag waiting in it
                    F.box('cubby', 0.24, h - 0.05, 0.004, (x, y, FRONT + 0.002), M.cubby)
                    F.box('cubby_bag', 0.12, 0.16, 0.004, (x - 0.02, y - 0.1, FRONT + 0.005), M.bag)
                    F.box('cubby_bag_logo', 0.06, 0.04, 0.002, (x - 0.02, y - 0.09, FRONT + 0.008), M.bag_logo)
                    continue
                _locker_door(F, _name('door'), x, y, h, 3 * c + r, M.led_green if (r + c) % 3 else M.accent)
        # the console: a bezel with the touch screen, its banner and button, a keypad and a camera
        F.box('bezel', 0.26, 0.47, 0.03, (0, 0.88, FRONT + 0.008), M.bezel, bevel=0.012, bevel_segments=2)
        F.box('touch_screen', 0.22, 0.3, 0.004, (0, 0.94, FRONT + 0.025), M.screen, bevel=0.004, bevel_segments=1)
        F.box('screen_banner', 0.18, 0.05, 0.003, (0, 1.0, FRONT + 0.0275), M.white)
        F.box('screen_button', 0.08, 0.08, 0.003, (0, 0.87, FRONT + 0.0275), M.accent, bevel=0.008, bevel_segments=1)
        F.box('keypad', 0.13, 0.08, 0.006, (0, 0.72, FRONT + 0.024), M.steel, bevel=0.003, bevel_segments=1)
        for i in range(3):
            for j in range(2):
                F.box(_name('key'), 0.03, 0.022, 0.004, (-0.04 + 0.04 * i, 0.705 + 0.032 * j, FRONT + 0.028), M.bezel)
        F.sphere('camera', 0.01, (0, 1.12, FRONT + 0.024), M.screen_light, scale=(1, 0.5, 1), segments=8, rings=4)
        # the canopy over the top: an orange band with the logo, its underside lit
        F.box('roof', 0.86, 0.1, -WALL + 0.02, (0, 1.91, WALL / 2 + 0.01), M.accent, bevel=0.02, bevel_segments=2)
        F.box('logo', 0.4, 0.06, 0.004, (0, 1.91, FRONT + 0.032), M.white, bevel=0.002, bevel_segments=1)
        F.extrude('logo_box', [(-0.02, -0.018), (0.02, -0.018), (0.02, 0.012), (0.0, 0.022), (-0.02, 0.012)], 0.003,
                  (-0.17, 1.91, FRONT + 0.035), M.accent)
        # the order waiting on the doorstep: a taped parcel with its label, a takeout bag with its top folded and handles
        F.box('parcel', 0.32, 0.22, 0.24, (0.2, 0.11, 0.24), M.kraft, rot=(0, 0, -8), bevel=0.006, bevel_segments=1)
        F.box('parcel_tape', 0.33, 0.02, 0.05, (0.2, 0.221, 0.24), M.tape, rot=(0, 0, -8))
        F.box('parcel_tape_side', 0.004, 0.2, 0.05, (0.2 + 0.161 * G.cosd(-8), 0.12, 0.24 + 0.161 * G.sind(-8)), M.tape,
              rot=(0, 0, -8))
        F.box('parcel_label', 0.1, 0.002, 0.07, (0.25, 0.221, 0.31), M.white, rot=(0, 0, -8))
        F.soft('takeout_bag', 0.22, 0.24, 0.14, (-0.22, 0.12, 0.22), M.bag, rot=(0, 0, 6), round=0.012, puff=0.006, flat=2)
        F.box('takeout_fold', 0.222, 0.03, 0.12, (-0.22, 0.255, 0.22), M.bag, rot=(0, 0, 6), bevel=0.008, bevel_segments=1)
        F.box('takeout_logo', 0.12, 0.08, 0.004, (-0.215, 0.14, 0.292), M.bag_logo, rot=(0, 0, 6))
        for dx in (-0.05, 0.05):
            G.kit.tube(F, _name('bag_handle'), [(-0.22 + dx, 0.268, 0.2), (-0.22 + dx * 0.9, 0.32, 0.215), (-0.22 + dx * 0.8, 0.268, 0.24)],
                       0.005, M.tape, verts=4, round=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # One locker's door, hinged on its left edge: `door_open` swings it open for a takeout.
    x, (y, h) = COLS[OPEN[0]], ROWS[OPEN[1]]
    with G.new_parts() as door:
        _locker_door(F, 'locker_door', x, y, h, 1, M.accent)
    door = G.hinged(G.join(door, 'locker_door'), F.at(x - 0.13, y, FRONT + 0.01))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 105, seconds=0.7)
    return objs


# --- the effortless vibration plate (1x1): a deck at 0.125 the resident stands on, a column with a console --
DECK = (0.0, 0.105, 0.12)   # the deck's middle, at its underside
BAR_Y = 1.1                 # the handlebar's height: a grown-up's waist on the deck


def vibePlate():
    F = G.GameFrame()
    dx, dy, dz = DECK
    with G.new_parts() as still:
        F.soft('base', 0.62, dy - 0.012, 0.6, (0, dy / 2 + 0.006, dz - 0.02), M.plate, round=0.04, round_segments=2, flat=1)
        F.sweep('band', [(u, 0.026, v + dz - 0.02) for u, v in G.rounded_rect(0.622, 0.6, 0.04, corner=3)], None, M.lime,
                profile=G.rounded_rect(0.02, 0.008, 0.003, corner=1), up=(0, 0, 1), closed=True)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.cyl(_name('foot'), 0.03, 0.012, (sx * 0.25, 0.006, dz - 0.02 + sz * 0.24), M.pad, verts=10)
        # the column at the plate's far end, which the stander faces (they step on from the tile in front and face
        # in), curving up to the console; the handlebar bent from one tube, its arms reaching back to either side of
        # them at waist height, the grips by their hands (2026-10-21: a short bar at the hip, 30 cm ahead: I-packs-04)
        F.sweep('column', [(0, 0.09, -0.235), (0, 0.5, -0.27), (0, BAR_Y - 0.02, -0.32)], None, M.plate,
                profile=G.rounded_rect(0.07, 0.09, 0.03, corner=3), up=(0, 1, 0), resolution=2, scale=[1.15, 1.0, 0.9])
        F.cyl('column_collar', 0.06, 0.02, (0, 0.1, -0.236), M.lime, verts=16)
        bar = G.kit.fillet([(-0.26, BAR_Y - 0.02, -0.07), (-0.26, BAR_Y, -0.32), (0.26, BAR_Y, -0.32), (0.26, BAR_Y - 0.02, -0.07)],
                           0.07, steps=4)
        G.kit.tube(F, 'handlebar', bar, 0.02, M.plate, verts=10, round=1)
        for s in (-1, 1):
            F.sweep(_name('grip'), [(s * 0.26, BAR_Y - 0.0185, -0.2), (s * 0.26, BAR_Y - 0.02, -0.06)], 0.029, M.lime, verts=12,
                    scale=[1.0, 1.0])
            F.sphere(_name('grip_end'), 0.029, (s * 0.26, BAR_Y - 0.02, -0.06), M.lime, scale=(1, 1, 1), segments=12, rings=6)
        # the console, tilted towards the stander: its screen and three buttons
        F.box('console', 0.22, 0.14, 0.04, (0, BAR_Y + 0.09, -0.32), M.bezel, rot=(30, 0, 0), bevel=0.015, bevel_segments=2)
        F.box('console_screen', 0.17, 0.08, 0.004, (0, BAR_Y + 0.1, -0.298), M.readout, rot=(30, 0, 0), bevel=0.003,
              bevel_segments=1)
        for k in range(3):
            F.cyl(_name('console_button'), 0.009, 0.006, (-0.05 + 0.05 * k, BAR_Y + 0.045, -0.29), M.lime if k == 1 else M.white,
                  rot=(120, 0, 0), verts=8)
        _lathe(F, 'bottle', 0.32, 0, -0.32, [(0.032, 0), (0.035, 0.006), (0.035, 0.08), (0.03, 0.1), (0.035, 0.12), (0.035, 0.165),
                                             (0.022, 0.195), (0.018, 0.2), (0, 0.2)], M.bottle, verts=14)
        _turned(F, 'bottle_cap', 0.32, 0.2, -0.32, [(0, 0), (0.019, 0), (0.019, 0.016), (0.012, 0.022), (0, 0.022)], M.white, 12)
    G.join(still, 'body', origin=(0, 0, 0))
    # The deck the resident stands on: `shake` rocks it to and fro, quickly, while they shake. A ribbed rubber pad
    # with two lime footprints.
    with G.new_parts() as deck:
        F.box('deck', 0.54, 0.012, 0.48, (dx, dy + 0.006, dz), M.pad, bevel=0.01, bevel_segments=2)
        F.box('deck_pad', 0.5, 0.008, 0.44, (dx, dy + 0.016, dz), M.pad_inner, bevel=0.006, bevel_segments=1)
        for k in range(9):
            F.box(_name('rib'), 0.47, 0.003, 0.012, (dx, dy + 0.0205, dz - 0.2 + 0.05 * k), M.pad, bevel=0.0012, bevel_segments=1)
        foot = G.smooth_outline([(0.0, -0.11), (0.035, -0.09), (0.045, 0.0), (0.05, 0.08), (0.03, 0.11), (-0.01, 0.11),
                                 (-0.04, 0.07), (-0.035, 0.0), (-0.03, -0.09)], 2)
        for s in (-1, 1):
            F.extrude(_name('foot_mark'), [(s * u, v) for u, v in foot], 0.004, (dx + s * 0.13, dy + 0.0215, dz), M.lime,
                      plane='top')
    deck = G.hinged(G.join(deck, 'deck'), F.at(dx, dy, dz))
    objs = G.settle(centre=False)
    G.rock(deck, 'shake', 3.0, seconds=0.25, axis='Y')
    return objs


# --- the smart-home command hub (1x1): a side table with a smart speaker, a tablet on its dock, a robot ------
TABLE = (0.6, 0.6, 0.44)   # width, top height, depth


def smartHub():
    F = G.GameFrame()
    w, h, d = TABLE
    z = WALL + d / 2
    with G.new_parts() as still:
        F.box('table_top', w, 0.03, d, (0, h - 0.015, z), M.wood, bevel=0.01, bevel_segments=2)
        F.box('apron', w - 0.06, 0.06, d - 0.06, (0, h - 0.06, z), M.wood, bevel=0.004, bevel_segments=1)
        F.box('shelf', w - 0.07, 0.02, d - 0.07, (0, 0.14, z), M.wood, bevel=0.005, bevel_segments=1)
        for lx in (-w / 2 + 0.05, w / 2 - 0.05):
            for lz in (z - d / 2 + 0.05, z + d / 2 - 0.05):
                F.sweep(_name('leg'), [(lx, h - 0.03, lz), (lx + math.copysign(0.025, lx), 0.0, lz + math.copysign(0.02, lz - z))],
                        None, M.wood, profile=G.rounded_rect(0.034, 0.034, 0.012, corner=2), up=(0, 0, 1), scale=[1.0, 0.6])
        # the smart speaker: a rounded fabric-covered cylinder, its light ring and buttons on top
        _lathe(F, 'speaker', -0.14, h, z - 0.02, [(0.055, 0.0), (0.063, 0.008), (0.066, 0.03), (0.066, 0.18), (0.062, 0.196),
                                                  (0.055, 0.2), (0.0, 0.2)], M.speaker, verts=24)
        G.torus('speaker_ring', 0.056, 0.005, at=F.at(-0.14, h + 0.2, z - 0.02), material=M.ring, major_segments=24,
                minor_segments=4)
        for k in range(4):
            a = 45 + 90 * k
            F.cyl(_name('speaker_button'), 0.007, 0.003, (-0.14 + 0.03 * G.cosd(a), h + 0.2015, z - 0.02 + 0.03 * G.sind(a)), M.bezel,
                  verts=8)
        # the tablet's dock, a few books on the shelf, a robot vacuum on the floor in front
        F.box('dock', 0.2, 0.025, 0.1, (0.12, h + 0.0125, z + 0.02), M.white, bevel=0.01, bevel_segments=2)
        F.box('dock_slot', 0.18, 0.004, 0.016, (0.12, h + 0.025, z + 0.02), M.bezel)
        for k, (bh, m, r) in enumerate(((0.04, M.screen_light, 0), (0.035, M.app_yellow, 6))):
            y = 0.15 + 0.04 * k + bh / 2
            F.box(_name('book'), 0.22 - 0.02 * k, bh, 0.16 - 0.01 * k, (-0.1, y, z), m, rot=(0, 0, r), bevel=0.003, bevel_segments=1)
            F.box(_name('book_pages'), 0.21 - 0.02 * k, bh - 0.008, 0.164 - 0.01 * k, (-0.096, y, z), M.white, rot=(0, 0, r))
        _turned(F, 'robot', 0, 0, 0.26, [(0, 0.008), (0.14, 0.008), (0.15, 0.02), (0.15, 0.05), (0.14, 0.065), (0, 0.065)], M.white, 32)
        F.cyl('robot_band', 0.152, 0.014, (0, 0.03, 0.26), M.bezel, verts=32)
        F.cyl('robot_turret', 0.04, 0.012, (0, 0.071, 0.2), M.bezel, verts=14)
        F.cyl('robot_button', 0.025, 0.004, (0, 0.067, 0.29), M.ring, verts=12)
        for s in (-1, 1):
            F.cyl(_name('robot_wheel'), 0.018, 0.012, (s * 0.1, 0.018, 0.26), M.bezel, rot=(0, 90, 0), verts=10)
        for k in range(3):
            a = 30 + 120 * k
            F.box(_name('brush'), 0.05, 0.003, 0.006, (0.11 + 0.025 * G.cosd(a), 0.006, 0.36 + 0.025 * G.sind(a)), M.bezel,
                  rot=(0, 0, a))
    G.join(still, 'body', origin=(0, 0, 0))
    # The tablet, built standing up on its hinge in the dock, then laid back to rest: `screen_up` tilts it up.
    hy, hz = h + 0.02, z + 0.02
    with G.new_parts() as tab:
        F.box('tablet', 0.26, 0.18, 0.012, (0.12, hy + 0.09, hz), M.bezel, bevel=0.006, bevel_segments=2)
        F.box('tablet_screen', 0.23, 0.15, 0.003, (0.12, hy + 0.09, hz + 0.006), M.screen_light, bevel=0.002, bevel_segments=1)
        for i, (m, ax, ay) in enumerate(((M.app_yellow, 0.06, 0.13), (M.led_green, 0.105, 0.13), (M.ring, 0.15, 0.13),
                                         (M.white, 0.06, 0.085), (M.app_yellow, 0.105, 0.085))):
            F.box(_name('app'), 0.032, 0.032, 0.002, (ax, hy + ay, hz + 0.008), m, bevel=0.006, bevel_segments=1)
        F.box('app_bar', 0.2, 0.025, 0.002, (0.12, hy + 0.035, hz + 0.008), M.white, bevel=0.004, bevel_segments=1)
        F.sphere('tablet_camera', 0.003, (0.12, hy + 0.172, hz + 0.0065), M.screen, segments=6, rings=3)
    tab = G.hinged(G.join(tab, 'tablet'), F.at(0.12, hy, hz))
    G.turn(tab, -72, 'X')
    objs = G.settle(centre=False)
    G.swing(tab, 'screen_up', 57, seconds=0.8, axis='X')
    return objs


FAMILY = G.Family(
    'pack_lazy', M,
    [G.Model('snoozecloudBed', snoozecloudBed, size=(1.86, 1.292, 1.86), key='model.lazy.snoozecloudBed',
             motions={'uses': {'sleepIn': 'head_up'}, 'hold': 2},
             paint=[('body', 'Body'), ('upholstery', 'Upholstery'), ('throw', 'Throw')]),
     G.Model('napPod', napPod, size=(0.918, 1.539, 0.908), key='model.lazy.napPod', centre=(0, 0.0243),
             motions={'uses': {'powerNap': 'hood', 'curlUp': 'hood'}},
             paint=[('shell', 'Body'), ('seat', 'Upholstery'), ('hood', 'Hood')]),
     G.Model('quiltTable', quiltTable, size=(1.81, 0.77, 0.929), key='model.lazy.quiltTable', centre=(0, 0.0343),
             motions={'uses': {'tuckIn': 'quilt', 'doze': 'quilt'}},
             paint=[('oak', 'Body'), ('quilt', 'Upholstery'), ('pouf', 'Poufs')]),
     G.Model('deliveryLocker', deliveryLocker, size=(0.86, 1.96, 0.81), key='model.lazy.deliveryLocker', centre=(0, -0.025),
             motions={'uses': {'takeout': 'door_open', 'noodles': 'door_open'}, 'hold': 2},
             paint=[('locker', 'Body'), ('door', 'Doors'), ('accent', 'Trim')]),
     G.Model('vibePlate', vibePlate, size=(0.67, 1.255, 0.771), key='model.lazy.vibePlate', centre=(0.02, 0.0193),
             motions={'loops': {'shake': 'shake'}},
             paint=[('plate', 'Body'), ('lime', 'Trim'), ('pad', 'Deck')]),
     G.Model('smartHub', smartHub, size=(0.6, 0.806, 0.842), key='model.lazy.smartHub', centre=(0, -0.009),
             motions={'uses': {'program': 'screen_up', 'story': 'screen_up'}},
             paint=[('wood', 'Body'), ('speaker', 'Speaker')])],
    comment='Blender-built lazy pack models (tools/art/blender/families/pack_lazy.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves while it is used: the bed\'s head rises, the pod\'s hood swings '
            'over, the quilt lifts, a locker door opens, the plate shakes and the tablet tilts up.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
