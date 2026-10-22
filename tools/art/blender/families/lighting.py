# The lamps: the ceiling lights (LT1: `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`,
# `ceilingFan`, `trackLights`, `paperLantern`), the wall lights (LT2: `wallSconce`, `wallLampArm`,
# `wallLightOutdoor`), the floor lamps (LT3: `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`,
# `lampTorchiere`) and the table lamps (LT4: `tableLamp`, `lampTableRound`, `lampTableSquare`, `lampBanker`,
# `lampMushroom`, `lampLantern`). Notes: lighting.md.
#
# The light itself is the content's `light`; each lamp's bulb is placed near its `light.height` (asserted
# against the content JSON). The shades, diffusers and bulbs glow at night (`glow`, E8); nothing glows by
# day. Ceiling things hang from 2.8 m and wall things have their back on the wall face, both at their real
# height (`floor=False`). One clip, ambient (`idle`): the ceiling fan's blades turn while it's on. The rest are
# still (the paper lantern's sway went, 2026-10-21: a paper globe indoors hangs still).
#
# Run:  cargo art lighting [--only chandelier] [--preview]
import json
import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

GLOW = {'glow': '#FFD9A0'}
M = G.Palette(
    # fittings
    brass=('#B8925A', 'metal'),
    iron=('#2B2C2E', 'metal'),         # black metal: cords, plates, cages, track heads
    white=('#F2F2F0', 'satin'),        # the fan, the track, the paper lantern's canopy
    bulb=('#FFF1CF', 'gloss', GLOW),   # bulbs and diffusers
    flame=('#FFD27A', 'gloss', {'glow': '#FFC46A'}),
    # the ceiling lights
    dome=('#F2F0EA', 'satin', GLOW),   # the pendant's dome
    panel=('#FBF4E4', 'satin', GLOW),  # the ceiling light's frosted sides
    industrial=('#2F4A3A', 'metal'),   # the industrial pendant's enamel dome
    liner=('#F2F0EA', 'satin', GLOW),  # the white inside of a metal shade
    blade=('#CDB08A', 'wood'),
    paper=('#FBF7EE', 'fabric', GLOW),
    rib=('#EDE6D6', 'fabric'),
    # the wall lights
    shade=('#F4EBD8', 'fabric', GLOW),  # drum and cone shades
    shade_metal=('#2B2C2E', 'metal'),   # the swing-arm lamp's shade
    lens=('#F4EEDC', 'glass', GLOW),    # the bulkhead's lens
    # the floor lamps
    stand=('#3A3530', 'metal'),
    lamp_shade=('#F2E6CC', 'fabric', GLOW),
    trim=('#E3D2B0', 'fabric'),
    marble=('#E7E3DC', 'gloss'),
    arc=('#C9A06A', 'metal'),
    arc_shade=('#C9A06A', 'metal'),
    walnut=('#7A5234', 'wood'),
    linen=('#E7E0D4', 'fabric', GLOW),
    bowl=('#F2EBDD', 'glass', GLOW),
    # the table lamps
    oak=('#CDB08A', 'wood'),
    ceramic=('#E8E2D6', 'gloss'),
    jar=('#3E6E9A', 'gloss'),
    stone=('#E7E3DC', 'matte'),
    square_shade=('#EFE8DA', 'fabric', GLOW),
    banker=('#2F6B4F', 'gloss', {'glow': '#4FB27A'}),   # green glass: lit, it glows green, not cream
    porcelain=('#F2F0EA', 'gloss'),
    cap=('#E9A07B', 'gloss', {'glow': '#FFAA78'}),      # coral glass: lit, it glows coral
    candle=('#F4EBD8', 'satin'),
    glass=('#DCE6EA', 'glass'),
)

WALL = -0.43     # the wall's face, game depth
CEILING = 2.8


def _content_lights():
    """{object id: light.height} from the content: the bulbs are placed against it."""
    root = os.path.join(G.ASSETS, '..', 'content')
    out = {}
    for path in ('base.json', 'furniture.json', os.path.join('catalog2', 'lighting.json')):
        with open(os.path.join(root, path)) as f:
            for o in json.load(f).get('objects', []):
                if 'light' in o:
                    out[o['id']] = o['light']['height']
    return out


LIGHTS = _content_lights()


def _near_light(id, y):
    """Asserts a bulb at height `y` is within 25 cm of where the content puts the lamp's light."""
    want = LIGHTS[id]
    assert abs(y - want) <= 0.25, f'{id}: bulb at {y:.2f} m, the content\'s light at {want:.2f} m'


def _bulb(F, id, r, at, material=None, scale=(1, 1, 1), segments=14, rings=8):
    """The lamp's bulb at a game position (`_near_light`)."""
    _near_light(id, at[1])
    return F.sphere(f'{id}_bulb', r, at, material or M.bulb, scale=scale, segments=segments, rings=rings)


def _pear(F, id, r, at, segments=14):
    """A pear-shaped bulb (globe below, neck above) whose globe's middle is at game `at` (`_near_light`)."""
    _near_light(id, at[1])
    prof = [(0, -r), (r * 0.55, -r * 0.85), (r * 0.9, -r * 0.45), (r, 0), (r * 0.9, r * 0.45), (r * 0.62, r * 0.9),
            (r * 0.45, r * 1.25), (r * 0.45, r * 1.55), (0, r * 1.55)]
    return G.revolve(f'{id}_bulb', prof, at=F.at(*at), material=M.bulb, segments=segments)


def _turned(F, name, profile, at, material, segments=24, rot=(0, 0, 0)):
    """A turned part: `profile` [(radius, up), ...] in metres from its foot at game point `at`, bottom first."""
    return G.revolve(name, profile, at=F.at(*at), material=material, segments=segments, rot=rot)


def _ring(F, name, r, at, t, material, segments=32, minor=6):
    """A level ring (a rolled rim, a wire, a bead) of radius `r` and thickness `t` round game point `at`."""
    return G.torus(name, r, t, at=F.at(*at), material=material, major_segments=segments, minor_segments=minor)


def _loft(F, name, x, z, rings, material, verts=16, caps=True):
    """A body lofted through level rings [(height, half width, half depth, squareness), ...] round game (x, z),
    bottom first (`G.loft`/`G.ring`): a rounded-square canopy, a lantern's roof, a banker's base."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(rx, rz, y, verts, square=sq, at=(bx, by)) for y, rx, rz, sq in rings], material=material,
                  caps=caps)


def _lathe(F, name, w, h, at, profile, material, segments=24, rot=(0, 0, 0), top=False, liner=None):
    """An old part's lathe: `profile` in fractions of `w` across and `h` high (-0.5 … 0.5), centred on game
    `at`, closed if it doesn't end on the axis or where it starts. `top`: `at` is its top middle instead, its
    origin (to tilt it about where it hangs). `liner` (a material): a thin shell of it just inside the profile's inner
    face, the points after the axis (a metal shade's white inside)."""
    pts = [(r * w, (z - (0.5 if top else -0.5)) * h) for r, z in profile]
    if pts[-1][0] > 1e-6 and pts[-1] != pts[0]:
        pts.append(pts[0])
    x, y, z = at
    base = F.at(x, y if top else y - h / 2, z)
    out = [G.revolve(name, pts, at=base, material=material, segments=segments, rot=rot)]
    if liner:
        k = next(i for i in range(len(profile) - 1, 0, -1) if profile[i][0] < 1e-6)
        inner = [(r * w, zz) for (r, _), (_, zz) in zip(profile[k + 1:], pts[k + 1:])]
        band = [(r * 0.97, z) for r, z in inner] + [(r * 0.93, z) for r, z in reversed(inner)]
        band.append(band[0])
        out.append(G.revolve(f'{name}_liner', band, at=base, material=liner, segments=segments, rot=rot))
    return out


def _drum(F, name, r_bottom, r_top, h, at, material, segments=28, t=0.006, rot=(0, 0, 0), rims=None, rim=0.0045,
          spider=None):
    """An open drum or cone shade, `t` thick, its bottom at game `at`. `rims` (a material): a rolled rim round its
    top and bottom edges, as a lampshade's binding is; `spider` (a material): the wire fitter inside its top, three
    spokes to a washer, seen from above."""
    G.revolve(name, [(r_bottom, 0), (r_top, h), (r_top - t, h), (r_bottom - t, 0), (r_bottom, 0)],
              at=F.at(*at), material=material, segments=segments, smooth=segments > 4, rot=rot)
    x, y, z = at
    if rims:
        _ring(F, f'{name}_rim_low', r_bottom - t / 2, (x, y + rim * 0.6, z), rim, rims, segments=segments, minor=5)
        _ring(F, f'{name}_rim_high', r_top - t / 2, (x, y + h - rim * 0.6, z), rim, rims, segments=segments, minor=5)
    if spider:
        top = y + h - 0.012
        for a in (90, 210, 330):
            G.kit.tube(F, f'{name}_spoke{a}', [(x + 0.016 * G.cosd(a), top, z + 0.016 * G.sind(a)),
                                               (x + (r_top - t) * G.cosd(a), top, z + (r_top - t) * G.sind(a))],
                       0.0018, spider, verts=4, round=1)
        _ring(F, f'{name}_washer', 0.016, (x, top, z), 0.003, spider, segments=10, minor=4)


def _trough(name, r, length, t, at, material, steps=12, below=20):
    """A half tube, open below, along Blender x: a banker's lamp shade, `t` thick, coming `below` degrees
    down past the axis on either side."""
    mesh = bpy.data.meshes.new(name)
    verts, faces = [], []
    n = steps + 1
    for e in (-length / 2, length / 2):
        for rr in (r, r - t):
            for i in range(n):
                a = math.radians(-below + (180 + 2 * below) * i / steps)
                verts.append((e, rr * math.cos(a), rr * math.sin(a)))

    def v(e, inner, i):
        return e * 2 * n + inner * n + i

    for i in range(steps):
        faces.append((v(0, 0, i), v(0, 0, i + 1), v(1, 0, i + 1), v(1, 0, i)))      # outside
        faces.append((v(0, 1, i), v(1, 1, i), v(1, 1, i + 1), v(0, 1, i + 1)))      # inside
        for e in (0, 1):
            faces.append((v(e, 0, i), v(e, 1, i), v(e, 1, i + 1), v(e, 0, i + 1)))  # the ends
    for i in (0, steps):
        faces.append((v(0, 0, i), v(1, 0, i), v(1, 1, i), v(0, 1, i)))              # the lower edges
    mesh.from_pydata(verts, [], faces)
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    for p in obj.data.polygons:
        p.use_smooth = True
    obj.data.set_sharp_from_angle(angle=math.radians(40))
    obj.location = at
    return obj


def _canopy(F, name, r, h, material, segments=20, y=CEILING):
    """A ceiling canopy: a shallow turned cup against the ceiling, `r` across and `h` deep, a cord grip below."""
    return _turned(F, name, [(0, 0), (r * 0.22, 0), (r * 0.25, h * 0.2), (r * 0.42, h * 0.32), (r * 0.82, h * 0.5),
                             (r * 0.97, h * 0.72), (r, h * 0.85), (r, h), (0, h)], (0, y - h, 0), material, segments)


def _hang(F, id, cord_bottom, canopy=None, canopy_r=0.06, cord=None):
    """A ceiling canopy (brass, or `canopy`) and the cord down from it to `cord_bottom`, through a cord grip."""
    canopy = canopy or M.brass
    _canopy(F, f'{id}_canopy', canopy_r, 0.03, canopy)
    G.kit.tube(F, f'{id}_cord', [(0, CEILING - 0.03, 0), (0, cord_bottom, 0)], 0.0045, cord or M.iron, verts=6, round=1)


def _socket(F, name, r, h, at, material, segments=16):
    """A lamp socket from its foot at game `at`: a skirt, a body with two turned ribs, a cap."""
    _turned(F, name, [(0, 0), (r * 0.82, 0), (r * 0.9, h * 0.08), (r * 0.9, h * 0.45), (r, h * 0.5), (r, h * 0.58),
                      (r * 0.9, h * 0.62), (r * 0.9, h * 0.72), (r, h * 0.76), (r, h * 0.84), (r * 0.9, h * 0.88),
                      (r * 0.7, h), (0, h)], at, material, segments)


DOME = [[0.5, -0.5], [0.47, 0.0], [0.36, 0.3], [0.14, 0.47], [0, 0.5], [0, 0.42], [0.13, 0.4], [0.33, 0.24],
        [0.44, -0.06], [0.47, -0.5]]


# --- LT1: the ceiling lights -----------------------------------------------------------------------------
# The two pendants residents walk under hang clear of their heads: their lowest point 2.1 m up (a room's
# 2.8 m ceiling, a 0.7 m drop; the reference's least clearance where people walk). The chandelier and the
# industrial pendant hang lower, over a table or a sofa only (content `headHeight`, Q-12).
def ceilingPendant():
    """A white dome on a brass socket and a black cord: a rolled lip round the dome, a brass collar on its crown, a
    pear bulb showing below."""
    F = G.GameFrame()
    with G.new_parts():
        _hang(F, 'ceilingPendant', 2.4)
        _socket(F, 'socket', 0.026, 0.05, (0, 2.35, 0), M.brass)
        _lathe(F, 'dome', 0.43, 0.24, (0, 2.24, 0), DOME, M.dome, segments=40)
        _ring(F, 'dome_lip', 0.209, (0, 2.124, 0), 0.006, M.dome, segments=40)
        _turned(F, 'collar', [(0, 0), (0.064, 0), (0.064, 0.004), (0.052, 0.012), (0.034, 0.018), (0, 0.018)],
                (0, 2.344, 0), M.brass, segments=24)
        _pear(F, 'ceilingPendant', 0.04, (0, 2.162, 0))
    G.join_all('ceilingPendant')
    G.settle(floor=False)
    return G.meshes()


def ceilingLight():
    """A square lantern on a short stem: four frosted sides, each parted in four panes by black glazing bars,
    between black corner posts, under a hipped lid with a knob; a diffuser below."""
    F = G.GameFrame()
    with G.new_parts():
        _loft(F, 'canopy', 0, 0, [(2.772, 0.052, 0.052, 6), (2.785, 0.062, 0.062, 8), (2.8, 0.062, 0.062, 8)], M.iron)
        G.kit.tube(F, 'stem', [(0, 2.772, 0), (0, 2.555, 0)], 0.009, M.iron, verts=8, round=1)
        _turned(F, 'stem_collar', [(0, 0), (0.016, 0), (0.016, 0.012), (0.01, 0.02), (0, 0.02)], (0, 2.548, 0), M.iron, 12)
        # the hipped lid, curving up from its eaves to a knob
        _loft(F, 'lid', 0, 0, [(2.5, 0.18, 0.18, 14), (2.512, 0.18, 0.18, 14), (2.518, 0.168, 0.168, 12),
                               (2.53, 0.12, 0.12, 9), (2.542, 0.065, 0.065, 6), (2.55, 0.03, 0.03, 4)], M.iron, verts=24)
        for s in (-1, 1):
            F.box(f'side_z{s}', 0.32, 0.29, 0.006, (0, 2.355, s * 0.163), M.panel)
            F.box(f'side_x{s}', 0.006, 0.29, 0.32, (s * 0.163, 2.355, 0), M.panel)
            for t in (-1, 1):
                F.box(f'post{s}{t}', 0.016, 0.31, 0.016, (s * 0.17, 2.355, t * 0.17), M.iron, bevel=0.004, bevel_segments=1)
            # the glazing bars across each side: a cross over the frosted pane, and a rail under the lid
            F.box(f'bar_z{s}', 0.32, 0.008, 0.008, (0, 2.355, s * 0.168), M.iron)
            F.box(f'mullion_z{s}', 0.008, 0.29, 0.008, (0, 2.355, s * 0.168), M.iron)
            F.box(f'bar_x{s}', 0.008, 0.008, 0.32, (s * 0.168, 2.355, 0), M.iron)
            F.box(f'mullion_x{s}', 0.008, 0.29, 0.008, (s * 0.168, 2.355, 0), M.iron)
            F.box(f'head_z{s}', 0.34, 0.016, 0.01, (0, 2.494, s * 0.168), M.iron, bevel=0.003, bevel_segments=1)
            F.box(f'head_x{s}', 0.01, 0.016, 0.34, (s * 0.168, 2.494, 0), M.iron, bevel=0.003, bevel_segments=1)
        F.box('rim', 0.36, 0.012, 0.36, (0, 2.206, 0), M.iron, bevel=0.003, bevel_segments=1)
        F.box('diffuser', 0.32, 0.004, 0.32, (0, 2.214, 0), M.bulb)
        _bulb(F, 'ceilingLight', 0.05, (0, 2.32, 0), scale=(1, 1, 1.2))
    G.join_all('ceilingLight')
    G.settle(floor=False)
    return G.meshes()


def chandelier():
    """A turned brass column on a rod, three S-scrolled arms down to a ring of ten globes and three curved spokes to
    a ring of six above, each globe in a bobeche; a drop globe and a finial under the column."""
    F = G.GameFrame()
    with G.new_parts():
        _canopy(F, 'canopy', 0.08, 0.03, M.brass, segments=16)
        G.kit.tube(F, 'rod', [(0, CEILING - 0.03, 0), (0, 2.09, 0)], 0.005, M.brass, verts=8, round=1)
        G.torus('loop', 0.016, 0.004, at=F.at(0, 2.515, 0), material=M.brass, rot=(90, 0, 0), major_segments=12,
                minor_segments=4)
        # the column, turned, its widest bulge the hub the arms spring from
        _turned(F, 'column', [(0, 0), (0.01, 0), (0.02, 0.02), (0.028, 0.05), (0.03, 0.07), (0.026, 0.09), (0.012, 0.12),
                              (0.009, 0.16), (0.014, 0.19), (0.008, 0.22), (0, 0.22)], (0, 2.28, 0), M.brass, 12)
        _turned(F, 'collar', [(0, 0), (0.014, 0), (0.014, 0.012), (0, 0.012)], (0, 2.204, 0), M.brass, 10)
        _turned(F, 'finial', [(0, 0), (0.004, 0), (0.012, 0.012), (0.016, 0.024), (0.008, 0.036), (0.006, 0.05), (0, 0.05)],
                (0, 2.04, 0), M.brass, 10)
        _ring(F, 'ring_low', 0.435, (0, 2.05, 0), 0.013, M.brass, segments=36, minor=4)
        _ring(F, 'ring_high', 0.27, (0, 2.21, 0), 0.011, M.brass, segments=28, minor=4)
        for a in (30, 150, 270):
            c, s = G.cosd(a), G.sind(a)
            G.kit.tube(F, f'arm{a}', [(r * c, y, r * s) for r, y in ((0.02, 2.35), (0.12, 2.39), (0.25, 2.35), (0.36, 2.2),
                                                                   (0.435, 2.062))], 0.006, M.brass, verts=5, round=4)
        for a in (90, 210, 330):
            c, s = G.cosd(a), G.sind(a)
            G.kit.tube(F, f'spoke{a}', [(r * c, y, r * s) for r, y in ((0.012, 2.21), (0.14, 2.235), (0.27, 2.215))],
                       0.0045, M.brass, verts=5, round=3)
        for ring, n, r, y, size in ((0.435, 10, 0.035, 2.1, 0.02), (0.27, 6, 0.03, 2.25, 0.017)):
            for k in range(n):
                a = 360 * k / n + (15 if n == 6 else 0)
                x, z = ring * G.cosd(a), ring * G.sind(a)
                _turned(F, f'cup{y}_{k}', [(0, 0), (size, size * 0.6), (size, size * 0.85), (0, size * 0.85)],
                        (x, y - r - 0.014, z), M.brass, 6)
                F.sphere(f'globe{y}_{k}', r, (x, y, z), M.bulb, segments=8, rings=6)
        _bulb(F, 'chandelier', 0.03, (0, 2.12, 0), segments=10, rings=6)   # a drop under the hub
    G.join_all('chandelier')
    G.settle(floor=False)
    return G.meshes()


def pendantIndustrial():
    """A green enamel dome, white inside, a rolled rim, on a black fitter and a ribbed socket: an Edison bulb shows
    below."""
    F = G.GameFrame()
    with G.new_parts():
        _hang(F, 'pendantIndustrial', 2.24, canopy=M.iron)
        _socket(F, 'socket', 0.026, 0.05, (0, 2.205, 0), M.iron)
        _turned(F, 'fitter', [(0, 0), (0.058, 0), (0.058, 0.006), (0.044, 0.012), (0.032, 0.026), (0.03, 0.03), (0, 0.03)],
                (0, 2.18, 0), M.iron, 24)
        _lathe(F, 'dome', 0.38, 0.24, (0, 2.07, 0), DOME, M.industrial, segments=40, liner=M.liner)
        _ring(F, 'dome_rim', 0.185, (0, 1.954, 0), 0.006, M.industrial, segments=40)
        _pear(F, 'pendantIndustrial', 0.045, (0, 1.975, 0), segments=16)
    G.join_all('pendantIndustrial')
    G.settle(floor=False)
    return G.meshes()


def ceilingFan():
    """A white fan: a canopy and downrod, a motor with a band round its waist, a switch cup and a glass light
    dome below, and five wooden blades with rounded tips on curved irons. The blades turn (`blades_turn`,
    ambient)."""
    F = G.GameFrame()
    with G.new_parts() as still:
        _canopy(F, 'canopy', 0.08, 0.05, M.white)
        G.kit.tube(F, 'downrod', [(0, 2.75, 0), (0, 2.615, 0)], 0.013, M.white, verts=10, round=1)
        _turned(F, 'ball_cover', [(0, 0), (0.016, 0), (0.024, 0.01), (0.025, 0.02), (0.016, 0.03), (0, 0.03)],
                (0, 2.725, 0), M.white, 14)
        _turned(F, 'coupler', [(0, 0), (0.022, 0), (0.024, 0.022), (0.016, 0.03), (0, 0.03)], (0, 2.6, 0), M.white, 14)
        _turned(F, 'motor', [(0, 0), (0.1, 0), (0.124, 0.008), (0.13, 0.025), (0.13, 0.05), (0.134, 0.054),
                             (0.134, 0.066), (0.128, 0.07), (0.114, 0.095), (0.085, 0.112), (0.04, 0.12), (0, 0.12)],
                (0, 2.5, 0), M.white, 32)
        _turned(F, 'switch_cup', [(0, 0), (0.076, 0), (0.094, 0.01), (0.1, 0.022), (0.1, 0.04), (0, 0.04)],
                (0, 2.46, 0), M.white, 28)
        _bulb(F, 'ceilingFan', 0.09, (0, 2.46, 0), scale=(1, 1, 0.5), segments=24, rings=10)
    G.join(still, 'ceilingFan', origin=(0, 0, 0))
    # The blades reach from just outside the motor (0.135 m out) to 0.485 m, their tips' corners under 0.487 m: as
    # they turn they sweep a circle 0.974 m across, inside the 1 x 1 tile with its axis on the tile's middle.
    blade = G.smooth_outline([(-0.175, -0.042), (0.12, -0.05), (0.175, -0.05), (0.175, 0.05), (0.12, 0.05),
                              (-0.175, 0.042)], passes=1)
    blade = [(min(u, 0.175), v) for u, v in blade]
    with G.new_parts() as rotor:
        for k in range(5):
            a = 72 * k
            G.extrude(f'blade{k}', blade, 0.012, at=(0.31 * G.cosd(a), 0.31 * G.sind(a), 2.54), material=M.blade,
                      plane='XY', rot=(10, 0, a), bevel=0.003, bevel_segments=1)
            # the iron: a flat strap curving down from under the motor onto the blade's root
            G.sweep(f'iron{k}', [(r * G.cosd(a), r * G.sind(a), h) for r, h in ((0.1, 2.552), (0.15, 2.546), (0.2, 2.531),
                                                                                 (0.25, 2.528))],
                    profile=G.rounded_rect(0.008, 0.042, 0.003, corner=1), material=M.white, resolution=3)
    blades = G.hinged(G.join(rotor, 'blades'), F.at(0, 2.54, 0))
    # Left where it's built, its axis on the middle: five blades reach farther one way than the other, so
    # centring its box would put the axis off the middle (the Model's `centre` is the box's middle instead).
    G.settle(centre=False, floor=False)
    G.spin(blades, 'blades_turn', turns=1, seconds=2.5, axis='Z')
    return G.meshes()


def _aim(obj, at, direction):
    """Puts `obj` (built along Blender +z) at Blender point `at`, its +z along `direction` (Blender)."""
    obj.location = at
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = Vector(direction).to_track_quat('Z', 'Y')
    return obj


SPOT = [(0, 0), (0.024, 0), (0.034, 0.008), (0.04, 0.02), (0.045, 0.03), (0.045, 0.038), (0.041, 0.042),
        (0.045, 0.046), (0.045, 0.054), (0.041, 0.058), (0.045, 0.062), (0.045, 0.158), (0.043, 0.162), (0.043, 0.17),
        (0.036, 0.17), (0.034, 0.162), (0, 0.162)]


def trackLights():
    """A white track (a channel with a slot underneath, end caps and a live end at its middle) with three black
    spots on adapters, each aimed its own way: a can with cooling fins, a lip round its glowing lens."""
    F = G.GameFrame()
    with G.new_parts():
        slot = [(0.02, -0.025), (0.02, 0.025), (-0.02, 0.025), (-0.02, 0.007), (-0.013, 0.007), (-0.013, -0.007),
                (-0.02, -0.007), (-0.02, -0.025)]
        F.sweep('track', [(-0.462, 2.78, 0), (0.462, 2.78, 0)], None, M.white, profile=slot, up=(0, 0, 1))
        for s in (-1, 1):
            F.box(f'end{s}', 0.016, 0.042, 0.052, (s * 0.462, 2.78, 0), M.white, bevel=0.005, bevel_segments=2)
        F.box('canopy', 0.12, 0.02, 0.12, (0, 2.79, 0), M.white, bevel=0.006, bevel_segments=2)
        F.box('live_end', 0.09, 0.03, 0.054, (0, 2.765, 0), M.white, bevel=0.008, bevel_segments=2)
        for k, (x, aim) in enumerate(((-0.34, (-0.2, -1, 0.34)), (0.0, (0.16, -1, -0.26)), (0.34, (-0.26, -1, 0.42)))):
            n = math.sqrt(sum(c * c for c in aim))
            d = [c / n for c in aim]
            top = (x, 2.68, 0)
            F.box(f'adapter{k}', 0.05, 0.02, 0.034, (x, 2.752, 0), M.iron, bevel=0.005, bevel_segments=1)
            G.kit.tube(F, f'stem{k}', [(x, 2.745, 0), top], 0.008, M.iron, verts=8, round=1)
            F.sphere(f'knuckle{k}', 0.014, top, M.iron, segments=10, rings=6)
            a, b = F.at(*top), F.at(*(top[i] + d[i] for i in range(3)))
            direction = Vector(b) - Vector(a)
            can = G.revolve(f'head{k}', SPOT, material=M.iron, segments=16)
            _aim(can, Vector(a), direction)
            lens = G.cylinder(f'lens{k}', 0.035, 0.004, material=M.bulb, verts=16)
            _aim(lens, Vector(a) + direction * 0.166, direction)
            _near_light('trackLights', top[1] + 0.17 * d[1])
    G.join_all('trackLights')
    G.settle(floor=False)
    return G.meshes()


def paperLantern():
    """A paper globe on wire ribs, the paper sagging a little between them, open at the top and the bottom on wire
    rims; hanging still from a white cord (a paper globe indoors hangs still: no draught), its bottom 2.1 m up."""
    F = G.GameFrame()
    y, R, k = 2.37, 0.27, 0.93   # the globe's middle, its radius, its height over its width
    ribs = (-0.2, -0.12, -0.04, 0.04, 0.12, 0.2)
    with G.new_parts():
        _canopy(F, 'canopy', 0.04, 0.02, M.white, segments=16)
        G.kit.tube(F, 'cord', [(0, CEILING - 0.02, 0), (0, y + 0.255, 0)], 0.004, M.white, verts=6, round=1)
        _turned(F, 'cap', [(0, 0), (0.04, 0), (0.044, 0.004), (0.04, 0.012), (0.012, 0.016), (0, 0.016)],
                (0, y + 0.246, 0), M.white, 16)
        # the paper: a turned skin from the bottom opening to the top one, pinched in at each rib, and back down
        # inside, coarser, so it shows a lining through the openings
        th0 = math.asin(0.06 / R)
        outer = []
        for i in range(37):
            th = th0 + (math.pi - 2 * th0) * i / 36
            dz = -R * k * math.cos(th)
            gap = min(abs(dz - r) for r in ribs)
            sag = 1 - 0.016 * math.sin(math.pi * min(gap, 0.04) / 0.08) ** 2 if abs(dz) < 0.24 else 1.0
            outer.append((R * math.sin(th) * sag, dz))
        inner = [(R * math.sin(th) - 0.004, -R * k * math.cos(th))
                 for th in (math.pi - th0 - (math.pi - 2 * th0) * i / 8 for i in range(9))]
        _near_light('paperLantern', y)
        G.revolve('paperLantern_bulb', outer + inner + [outer[0]], at=F.at(0, y, 0), material=M.paper, segments=24)
        for i, dy in enumerate(ribs):
            r = R * math.sqrt(max(0.0, 1 - (dy / (R * k)) ** 2)) + 0.002
            _ring(F, f'rib{i}', r, (0, y + dy, 0), 0.003, M.rib, segments=24, minor=3)
        for i, (r, dz) in enumerate((outer[0], outer[-1])):
            _ring(F, f'opening{i}', r - 0.001, (0, y + dz, 0), 0.003, M.rib, segments=16, minor=3)
        G.kit.tube(F, 'bulb_holder', [(0, y + 0.25, 0), (0, y + 0.05, 0)], 0.006, M.white, verts=6, round=1)
        F.sphere('lamp_bulb', 0.03, (0, y + 0.02, 0), M.bulb, segments=10, rings=6)
    G.join_all('paperLantern')
    G.settle(floor=False)
    return G.meshes()


# --- LT2: the wall lights ----------------------------------------------------------------------------------
def wallSconce():
    """A turned brass backplate, a gooseneck arm curving up into a candle cup, and a cone shade, rolled at its top
    and bottom edges, on a wire fitter."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'plate', [(0, 0), (0.06, 0), (0.06, 0.006), (0.054, 0.012), (0.03, 0.018), (0.016, 0.022), (0, 0.022)],
                (0, 1.7, WALL), M.brass, 20, rot=(90, 0, 0))
        G.kit.tube(F, 'arm', [(0, 1.7, WALL + 0.02), (0, 1.692, -0.33), (0, 1.7, -0.278), (0, 1.722, -0.26)], 0.0085,
                   M.brass, verts=8, round=3)
        _turned(F, 'cup', [(0, 0), (0.012, 0), (0.024, 0.008), (0.026, 0.018), (0.02, 0.022), (0.02, 0.03), (0, 0.03)],
                (0, 1.72, -0.26), M.brass, 14)
        for a in (90, 210, 330):
            G.kit.tube(F, f'spoke{a}', [(0.02 * G.cosd(a), 1.745, -0.26 + 0.02 * G.sind(a)),
                                        (0.087 * G.cosd(a), 1.745, -0.26 + 0.087 * G.sind(a))], 0.0025, M.brass, verts=4,
                       round=1)
        _bulb(F, 'wallSconce', 0.03, (0, 1.775, -0.26), scale=(1, 1, 1.15), segments=10, rings=6)
        _drum(F, 'shade', 0.1, 0.075, 0.18, (0, 1.71, -0.26), M.shade, segments=24, rims=M.shade, rim=0.004)
    G.join_all('wallSconce')
    G.settle(centre=False, floor=False)
    return G.meshes()


ARM_SHADE = [[0.5, -0.5], [0.5, -0.47], [0.478, -0.44], [0.2, 0.4], [0.1, 0.5], [0, 0.5], [0, 0.42], [0.09, 0.42],
             [0.46, -0.5]]


def wallLampArm():
    """A black swing-arm lamp: a turned wall plate, a lower arm and an upper arm bent to the shade, brass knuckles
    with thumb screws between them, and a tilted cone shade with a rolled rim and a switch on its crown."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'plate', [(0, 0), (0.055, 0), (0.055, 0.006), (0.048, 0.014), (0.022, 0.02), (0, 0.02)],
                (0, 1.55, WALL), M.iron, 22, rot=(90, 0, 0))
        _turned(F, 'pivot', [(0, 0), (0.016, 0), (0.016, 0.03), (0, 0.03)], (0, 1.55, WALL + 0.018), M.brass, 12,
                rot=(90, 0, 0))
        G.kit.tube(F, 'arm_low', [(0, 1.55, WALL + 0.04), (0, 1.58, -0.31), (0, 1.6, -0.25)], 0.0085, M.iron, verts=8)
        F.sphere('knuckle', 0.017, (0, 1.6, -0.25), M.brass, segments=12, rings=8)
        G.kit.tube(F, 'thumb_screw', [(0.017, 1.6, -0.25), (0.034, 1.6, -0.25)], 0.004, M.brass, verts=6, round=1)
        F.cyl('thumb', 0.009, 0.006, (0.036, 1.6, -0.25), M.brass, rot=(0, 90, 0), verts=10)
        G.kit.tube(F, 'arm_high', [(0, 1.6, -0.25), (0, 1.7, -0.235), (0, 1.765, -0.2), (0, 1.775, -0.185)], 0.0075,
                   M.iron, verts=8)
        _lathe(F, 'shade', 0.2, 0.16, (0, 1.78, -0.18), ARM_SHADE, M.shade_metal, segments=28, rot=(-35, 0, 0),
               top=True, liner=M.liner)
        F.cyl('switch', 0.007, 0.01, (0, 1.781, -0.176), M.brass, rot=(-35, 0, 0), verts=10)
        _bulb(F, 'wallLampArm', 0.032, (0, 1.69, -0.125), scale=(1, 1, 1.15), segments=12, rings=6)
    G.join_all('wallLampArm')
    G.settle(centre=False, floor=False)
    return G.meshes()


def wallLightOutdoor():
    """A bulkhead: an oval glass lens in a black frame on a rounded back plate, under a drip hood, behind a cage of
    bars that arch over the lens, the plate screwed to the wall."""
    F = G.GameFrame()
    with G.new_parts():
        F.extrude('plate', G.rounded_rect(0.2, 0.3, 0.03, corner=3), 0.03, (0, 2.01, WALL + 0.015), M.iron, bevel=0.005,
                  bevel_segments=1)
        for y in (1.885, 2.135):
            F.cyl(f'screw{y}', 0.006, 0.004, (0, y, WALL + 0.031), M.iron, rot=(90, 0, 0), verts=8)
        G.torus('frame', 1.0, 0.11, at=F.at(0, 2.01, WALL + 0.03), material=M.iron, rot=(90, 0, 0),
                major_segments=24, minor_segments=5).scale = (0.074, 0.12, 0.12)
        lens = F.sphere('lens', 1.0, (0, 2.01, WALL + 0.03), M.lens, scale=(0.075, 0.06, 0.125), segments=16, rings=8)
        for v in lens.data.vertices:      # its back half, hidden in the plate, flattened onto the wall face
            if v.co.y > 0:
                v.co.y *= 0.5
        _bulb(F, 'wallLightOutdoor', 0.035, (0, 2.01, -0.38), scale=(1, 0.8, 1.2), segments=10, rings=6)
        # the drip hood: flat on the wall, rounding down over the lens at the front
        hood = G.smooth_outline([(-0.055, 0.0125), (0.035, 0.0125), (0.055, -0.004), (0.055, -0.022), (0.046, -0.022),
                                 (0.044, -0.006), (0.03, -0.0125), (-0.055, -0.0125)], passes=1)
        F.extrude('hood', hood, 0.22, (0, 2.175, WALL + 0.055), M.iron, plane='side')
        # the cage: bars from post to post arching forward over the lens
        for y in (1.94, 2.01, 2.08):
            G.kit.tube(F, f'bar{y}', [(-0.085, y, -0.36), (0, y, -0.327), (0.085, y, -0.36)], 0.006, M.iron, verts=6,
                       round=3)
        G.kit.tube(F, 'bar_v', [(0, 2.12, -0.385), (0, 2.01, -0.327), (0, 1.9, -0.385)], 0.006, M.iron, verts=6, round=3)
        for s in (-1, 1):
            F.box(f'post{s}', 0.012, 0.18, 0.06, (s * 0.085, 2.01, -0.37), M.iron, bevel=0.004, bevel_segments=1)
    G.join_all('wallLightOutdoor')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- LT3: the floor lamps ----------------------------------------------------------------------------------
def lamp():
    """The classic floor lamp: a dark stepped foot, a pole with turned brass collars, a ribbed socket with its turn
    knob, a brass harp bowing round the bulb to a finial, a drum shade with corded trims on a wire fitter, a pull
    chain."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'foot', [(0, 0), (0.15, 0), (0.15, 0.008), (0.145, 0.016), (0.13, 0.026), (0.1, 0.034), (0.06, 0.05),
                            (0.04, 0.062), (0.03, 0.066), (0.018, 0.07), (0, 0.07)], (0, 0, 0), M.stand, 32)
        G.kit.tube(F, 'pole', [(0, 0.06, 0), (0, 1.44, 0)], 0.012, M.stand, verts=10, round=1)
        for y in (0.5, 1.0):
            _turned(F, f'collar{y}', [(0, 0), (0.016, 0), (0.021, 0.006), (0.021, 0.024), (0.016, 0.03), (0, 0.03)],
                    (0, y - 0.015, 0), M.brass, 14)
        _socket(F, 'socket', 0.025, 0.08, (0, 1.435, 0), M.brass)
        G.kit.tube(F, 'knob_stem', [(0.024, 1.48, 0), (0.042, 1.48, 0)], 0.003, M.brass, verts=6, round=1)
        F.cyl('knob', 0.008, 0.012, (0.046, 1.48, 0), M.brass, rot=(0, 90, 0), verts=10)
        _bulb(F, 'lamp', 0.035, (0, 1.54, 0), scale=(1, 1, 1.25))
        F.box('harp_saddle', 0.06, 0.008, 0.012, (0, 1.448, 0), M.brass, bevel=0.003, bevel_segments=1)
        G.kit.tube(F, 'harp', [(-0.028, 1.45, 0), (-0.07, 1.6, 0), (-0.03, 1.76, 0), (0, 1.776, 0), (0.03, 1.76, 0),
                               (0.07, 1.6, 0), (0.028, 1.45, 0)], 0.0035, M.brass, verts=6, round=3)
        _turned(F, 'finial', [(0, 0), (0.006, 0), (0.012, 0.006), (0.017, 0.016), (0.013, 0.026), (0.005, 0.031),
                              (0, 0.032)], (0, 1.776, 0), M.brass, 14)
        _drum(F, 'shade', 0.22, 0.158, 0.34, (0, 1.43, 0), M.lamp_shade, segments=40)
        G.torus('trim_low', 0.218, 0.007, at=F.at(0, 1.436, 0), material=M.trim, major_segments=40, minor_segments=6)
        G.torus('trim_high', 0.157, 0.007, at=F.at(0, 1.764, 0), material=M.trim, major_segments=32, minor_segments=6)
        G.kit.tube(F, 'chain', [(0.04, 1.47, 0), (0.04, 1.31, 0)], 0.0018, M.brass, verts=4, round=1)
        _turned(F, 'pull', [(0, 0), (0.004, 0), (0.008, 0.008), (0.006, 0.016), (0.002, 0.02), (0, 0.02)],
                (0.04, 1.292, 0), M.brass, 10)
    G.join_all('lamp')
    G.settle()
    return G.meshes()


def floorLampSimple():
    """A square black base with a turned boss, a stem with a coupling and a rocker switch, under a tall square
    shade bound in black at its top and bottom, a diffuser at its foot and a wire fitter at its top."""
    F = G.GameFrame()
    with G.new_parts():
        F.extrude('base', G.rounded_rect(0.28, 0.28, 0.03, corner=3), 0.03, (0, 0.015, 0), M.iron, plane='top',
                  bevel=0.006, bevel_segments=2)
        _turned(F, 'boss', [(0, 0), (0.034, 0), (0.034, 0.005), (0.02, 0.014), (0.014, 0.03), (0, 0.03)], (0, 0.03, 0),
                M.iron, 16)
        G.kit.tube(F, 'stem', [(0, 0.05, 0), (0, 0.87, 0)], 0.011, M.iron, verts=10, round=1)
        _turned(F, 'coupling', [(0, 0), (0.015, 0), (0.015, 0.03), (0, 0.03)], (0, 0.44, 0), M.iron, 12)
        F.box('switch', 0.026, 0.05, 0.018, (0, 0.72, 0.014), M.iron, bevel=0.006, bevel_segments=2)
        _drum(F, 'shade', 0.19, 0.19, 0.74, (0, 0.86, 0), M.shade, segments=4, t=0.008, rot=(0, 0, 45))
        half = 0.19 / math.sqrt(2)
        square = [(-half, -half), (half, -half), (half, half), (-half, half)]
        for name, y in (('rim_low', 0.86), ('rim_high', 1.6 - 0.014)):
            G.kit.moulding(F, name, square, [(-0.002, 0), (0.01, 0), (0.01, 0.014), (-0.002, 0.014)], M.iron, at=y,
                           plane='top')
        F.box('diffuser', 0.255, 0.004, 0.255, (0, 0.866, 0), M.bulb)
        _bulb(F, 'floorLampSimple', 0.045, (0, 1.45, 0), scale=(1, 1, 1.2))
        G.kit.tube(F, 'holder', [(0, 0.87, 0), (0, 1.4, 0)], 0.006, M.iron, verts=6, round=1)
        for a in (0, 90, 180, 270):
            G.kit.tube(F, f'spoke{a}', [(0.012 * G.cosd(a), 1.58, 0.012 * G.sind(a)),
                                        (0.13 * G.cosd(a), 1.58, 0.13 * G.sind(a))], 0.002, M.iron, verts=4, round=1)
        _ring(F, 'washer', 0.014, (0, 1.58, 0), 0.003, M.iron, segments=10, minor=4)
    G.join_all('floorLampSimple')
    G.settle()
    return G.meshes()


def _bezier(p, t):
    u = 1 - t
    return tuple(u ** 3 * p[0][i] + 3 * u * u * t * p[1][i] + 3 * u * t * t * p[2][i] + t ** 3 * p[3][i]
                 for i in range(3))


ARC = ((-0.3, 1.0, -0.3), (-0.3, 1.85, -0.3), (0.22, 2.0, 0.18), (0.22, 1.72, 0.18))
ARC_SHADE = [[0.5, -0.5], [0.42, 0.2], [0.12, 0.5], [0, 0.5], [0, 0.4], [0.1, 0.4], [0.38, 0.15], [0.46, -0.5]]


def lampFloorArc():
    """A turned marble base in the back corner, a brass stem rising into one sweeping arc (a sleeve where the two
    meet, a dimmer on the stem) and a dome shade with a rolled lip on its socket."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'base', [(0, 0), (0.15, 0), (0.15, 0.034), (0.144, 0.047), (0.13, 0.055), (0.07, 0.06), (0, 0.06)],
                (-0.3, 0, -0.3), M.marble, 36)
        _turned(F, 'foot_collar', [(0, 0), (0.032, 0), (0.032, 0.01), (0.022, 0.026), (0.017, 0.04), (0, 0.04)],
                (-0.3, 0.06, -0.3), M.arc, 16)
        arc = [(-0.3, 0.09, -0.3), (-0.3, 0.55, -0.3)] + [_bezier(ARC, k / 14) for k in range(15)]
        G.kit.tube(F, 'arc', arc, 0.0125, M.arc, verts=10, round=2)
        _turned(F, 'sleeve', [(0, 0), (0.017, 0), (0.019, 0.006), (0.019, 0.05), (0.017, 0.056), (0, 0.056)],
                (-0.3, 0.97, -0.3), M.arc, 14)
        F.box('dimmer', 0.03, 0.06, 0.022, (-0.3, 0.82, -0.282), M.arc, bevel=0.008, bevel_segments=2)
        F.cyl('dimmer_knob', 0.007, 0.01, (-0.3, 0.83, -0.268), M.marble, rot=(90, 0, 0), verts=10)
        _socket(F, 'socket', 0.02, 0.04, (0.22, 1.69, 0.18), M.arc, segments=12)
        _lathe(F, 'shade', 0.36, 0.2, (0.22, 1.6, 0.18), ARC_SHADE, M.arc_shade, segments=36, liner=M.liner)
        _ring(F, 'shade_lip', 0.176, (0.22, 1.504, 0.18), 0.005, M.arc_shade, segments=36, minor=5)
        _bulb(F, 'lampFloorArc', 0.05, (0.22, 1.56, 0.18), scale=(1, 1, 0.75))
    G.join_all('lampFloorArc')
    G.settle()
    return G.meshes()


def lampFloorTripod():
    """Three tapered walnut legs on brass ferrules, meeting at a turned brass hub under a linen drum shade bound at
    its edges, a diffuser and a wire fitter inside."""
    F = G.GameFrame()
    with G.new_parts():
        for a in (90, 210, 330):
            c, s = G.cosd(a), G.sind(a)
            G.kit.tube(F, f'leg{a}', [(r * c, y, r * s) for r, y in ((0.287, 0.03), (0.17, 0.62), (0.022, 1.27))],
                       0.0175, M.walnut, verts=10, round=3, scale=[0.75, 1.0, 0.8])
            G.kit.tube(F, f'ferrule{a}', [(0.3 * c, 0.0, 0.3 * s), (0.285 * c, 0.04, 0.285 * s)], 0.0125, M.brass, verts=10,
                       round=1)
        _turned(F, 'hub', [(0, 0), (0.02, 0), (0.031, 0.012), (0.031, 0.058), (0.026, 0.066), (0.026, 0.076), (0.014, 0.08),
                           (0, 0.08)], (0, 1.24, 0), M.brass, 16)
        _socket(F, 'socket', 0.02, 0.05, (0, 1.32, 0), M.brass, segments=12)
        _bulb(F, 'lampFloorTripod', 0.04, (0, 1.4, 0), scale=(1, 1, 1.2))
        _drum(F, 'shade', 0.24, 0.24, 0.32, (0, 1.29, 0), M.linen, segments=40, rims=M.linen, spider=M.brass)
        F.cyl('diffuser', 0.234, 0.004, (0, 1.3, 0), M.bulb, verts=32)
    G.join_all('lampFloorTripod')
    G.settle()
    return G.meshes()


BOWL = [[0, -0.5], [0.04, -0.5], [0.3, -0.1], [0.5, 0.5], [0.46, 0.5], [0.27, -0.06], [0, -0.4]]


def lampTorchiere():
    """An uplighter: a stepped black base, a brass pole with turned collars and a dimmer, a frosted glass bowl with
    a petalled rim on a cup."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'base', [(0, 0), (0.15, 0), (0.15, 0.012), (0.142, 0.024), (0.11, 0.034), (0.05, 0.04), (0.03, 0.05),
                            (0, 0.05)], (0, 0, 0), M.iron, 36)
        G.kit.tube(F, 'pole', [(0, 0.05, 0), (0, 1.7, 0)], 0.0125, M.brass, verts=10, round=1)
        for y in (0.6, 1.15):
            _turned(F, f'collar{y}', [(0, 0), (0.017, 0), (0.021, 0.006), (0.021, 0.022), (0.017, 0.028), (0, 0.028)],
                    (0, y, 0), M.brass, 14)
        F.box('dimmer', 0.034, 0.07, 0.026, (0, 0.95, 0.016), M.iron, bevel=0.008, bevel_segments=2)
        F.cyl('dimmer_knob', 0.009, 0.012, (0, 0.96, 0.034), M.brass, rot=(90, 0, 0), verts=12)
        _turned(F, 'cup', [(0, 0), (0.018, 0), (0.026, 0.016), (0.03, 0.03), (0.044, 0.04), (0, 0.04)], (0, 1.69, 0),
                M.brass, 16)
        # the bowl: lofted from its foot to a rim of eight soft petals, and back down inside
        bx, by, _ = F.at(0, 0, 0)

        def petals(r, y, amp, n=48):
            return [(bx + r * (1 + amp * math.cos(8 * a)) * math.cos(a), by + r * (1 + amp * math.cos(8 * a)) * math.sin(a), y)
                    for a in (math.tau * i / n for i in range(n))]
        outer = [(0.016, 1.71, 0), (0.06, 1.725, 0.01), (0.11, 1.755, 0.025), (0.16, 1.805, 0.04), (0.19, 1.85, 0.05)]
        inner = [(0.172, 1.85, 0.05), (0.145, 1.81, 0.04), (0.1, 1.77, 0.025), (0.05, 1.746, 0.01), (0.006, 1.74, 0.0)]
        G.loft('bowl', [petals(r, y, amp) for r, y, amp in outer + inner], material=M.bowl)
        _bulb(F, 'lampTorchiere', 0.035, (0, 1.77, 0))
    G.join_all('lampTorchiere')
    G.settle()
    return G.meshes()


# --- LT4: the table lamps --------------------------------------------------------------------------------
GOURD = [[0, -0.5], [0.32, -0.5], [0.46, -0.3], [0.5, -0.05], [0.4, 0.25], [0.24, 0.42], [0.26, 0.5], [0.2, 0.5],
         [0.18, 0.4], [0, 0.38]]
JAR = [[0, -0.5], [0.3, -0.5], [0.48, -0.25], [0.5, 0.05], [0.38, 0.38], [0.15, 0.45], [0.12, 0.5], [0, 0.5]]


def tableLamp():
    """An oak side table (a top with rounded corners, an apron, tapered legs, a shelf) with a ceramic gourd lamp on a
    brass foot ring, a socket, and a drum shade bound at its edges on a wire fitter."""
    F = G.GameFrame()
    with G.new_parts():
        F.extrude('top', G.rounded_rect(0.66, 0.42, 0.03, corner=3), 0.03, (0, 0.485, 0), M.oak, plane='top', bevel=0.008,
                  bevel_segments=2)
        for s in (-1, 1):
            F.box(f'apron_z{s}', 0.52, 0.06, 0.018, (0, 0.44, s * 0.155), M.oak, bevel=0.003, bevel_segments=1)
            F.box(f'apron_x{s}', 0.018, 0.06, 0.28, (s * 0.275, 0.44, 0), M.oak, bevel=0.003, bevel_segments=1)
        F.extrude('shelf', G.rounded_rect(0.58, 0.34, 0.02, corner=2), 0.02, (0, 0.14, 0), M.oak, plane='top', bevel=0.005,
                  bevel_segments=1)
        for x in (-0.28, 0.28):
            for z in (-0.16, 0.16):
                _loft(F, f'leg{x}{z}', x, z, [(0.0, 0.013, 0.013, 8), (0.47, 0.021, 0.021, 8)], M.oak, verts=12)
        _turned(F, 'foot', [(0, 0), (0.052, 0), (0.054, 0.005), (0.05, 0.01), (0, 0.01)], (-0.12, 0.5, 0), M.brass, 24)
        _lathe(F, 'base', 0.17, 0.19, (-0.12, 0.605, 0), GOURD, M.ceramic, segments=28)
        _turned(F, 'neck', [(0, 0), (0.024, 0), (0.024, 0.008), (0.014, 0.014), (0, 0.014)], (-0.12, 0.695, 0), M.brass, 14)
        _socket(F, 'socket', 0.02, 0.05, (-0.12, 0.71, 0), M.brass, segments=12)
        _bulb(F, 'tableLamp', 0.035, (-0.12, 0.81, 0), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.15, 0.1275, 0.2, (-0.12, 0.72, 0), M.shade, segments=36, rims=M.shade, spider=M.brass)
        F.cyl('diffuser', 0.143, 0.004, (-0.12, 0.723, 0), M.bulb, verts=24)
    G.join_all('tableLamp')
    G.settle()
    return G.meshes()


def lampTableRound():
    """A blue ginger jar on a turned brass plinth, a brass cap on its neck, a stem and socket, a harp and finial
    over a drum shade bound at its edges."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'plinth', [(0, 0), (0.07, 0), (0.072, 0.006), (0.068, 0.012), (0.058, 0.018), (0, 0.018)], (0, 0, 0),
                M.brass, 28)
        _lathe(F, 'jar', 0.2, 0.245, (0, 0.1405, 0), JAR, M.jar, segments=32)
        for k, (y, r) in enumerate(((0.06, 0.084), (0.2, 0.084))):
            _ring(F, f'jar_band{k}', r, (0, y, 0), 0.003, M.jar, segments=32, minor=4)
        _turned(F, 'cap', [(0, 0), (0.03, 0), (0.03, 0.008), (0.022, 0.016), (0.012, 0.02), (0, 0.02)], (0, 0.258, 0),
                M.brass, 16)
        G.kit.tube(F, 'stem', [(0, 0.27, 0), (0, 0.35, 0)], 0.008, M.brass, verts=8, round=1)
        _socket(F, 'socket', 0.018, 0.045, (0, 0.335, 0), M.brass, segments=12)
        _bulb(F, 'lampTableRound', 0.03, (0, 0.4, 0), scale=(1, 1, 1.15))
        G.kit.tube(F, 'harp', [(-0.02, 0.34, 0), (-0.05, 0.44, 0), (-0.02, 0.545, 0), (0, 0.55, 0), (0.02, 0.545, 0),
                               (0.05, 0.44, 0), (0.02, 0.34, 0)], 0.0028, M.brass, verts=6, round=3)
        _turned(F, 'finial', [(0, 0), (0.005, 0), (0.01, 0.006), (0.013, 0.014), (0.01, 0.021), (0.004, 0.026), (0, 0.027)],
                (0, 0.55, 0), M.brass, 12)
        _drum(F, 'shade', 0.17, 0.1445, 0.22, (0, 0.33, 0), M.shade, segments=36, rims=M.shade)
        F.cyl('diffuser', 0.162, 0.004, (0, 0.34, 0), M.bulb, verts=24)
    G.join_all('lampTableRound')
    G.settle()
    return G.meshes()


def lampTableSquare():
    """A square stone block with softened edges under a black cap plate, a stem with a switch and a socket, and a
    tapered square shade bound in black at its edges."""
    F = G.GameFrame()
    with G.new_parts():
        F.extrude('base', G.rounded_rect(0.12, 0.12, 0.01, corner=2), 0.24, (0, 0.12, 0), M.stone, plane='top', bevel=0.005,
                  bevel_segments=2)
        F.extrude('base_cap', G.rounded_rect(0.1, 0.1, 0.008, corner=2), 0.016, (0, 0.248, 0), M.iron, plane='top',
                  bevel=0.003, bevel_segments=1)
        G.kit.tube(F, 'stem', [(0, 0.255, 0), (0, 0.345, 0)], 0.009, M.iron, verts=8, round=1)
        F.box('switch', 0.022, 0.03, 0.016, (0, 0.29, 0.012), M.iron, bevel=0.005, bevel_segments=1)
        _socket(F, 'socket', 0.018, 0.045, (0, 0.335, 0), M.iron, segments=12)
        _bulb(F, 'lampTableSquare', 0.03, (0, 0.4, 0), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.15, 0.12, 0.2, (0, 0.34, 0), M.square_shade, segments=4, rot=(0, 0, 45))
        for name, y, r in (('rim_low', 0.34, 0.15), ('rim_high', 0.53, 0.12)):
            h = r / math.sqrt(2)
            G.kit.moulding(F, name, [(-h, -h), (h, -h), (h, h), (-h, h)],
                           [(-0.0015, 0), (0.006, 0), (0.006, 0.01), (-0.0015, 0.01)], M.iron, at=y, plane='top')
        F.box('diffuser', 0.2, 0.004, 0.2, (0, 0.35, 0), M.bulb)
    G.join_all('lampTableSquare')
    G.settle()
    return G.meshes()


def lampBanker():
    """A banker's lamp: a stepped brass base with rounded ends, an upright curving forward into the shade's cradle,
    a green half-round shade (white inside) held by a brass band and bound at its ends, a beaded pull chain."""
    F = G.GameFrame()
    with G.new_parts():
        _loft(F, 'base', 0, 0, [(0.0, 0.11, 0.07, 5), (0.008, 0.11, 0.07, 5), (0.013, 0.104, 0.064, 5),
                                (0.018, 0.092, 0.054, 5), (0.022, 0.088, 0.05, 5)], M.brass, verts=24)
        _turned(F, 'foot', [(0, 0), (0.024, 0), (0.024, 0.006), (0.016, 0.014), (0.012, 0.02), (0, 0.02)],
                (-0.06, 0.02, 0), M.brass, 14)
        G.kit.tube(F, 'upright', [(-0.06, 0.03, 0), (-0.06, 0.28, 0), (-0.06, 0.3, 0.006), (-0.06, 0.305, 0.022)], 0.008,
                   M.brass, verts=8, round=3)
        _trough('shade', 0.06, 0.3, 0.004, F.at(0, 0.33, 0.03), M.banker)
        _trough('shade_liner', 0.0555, 0.292, 0.003, F.at(0, 0.33, 0.03), M.liner)
        # the band over the shade where the upright holds it, and a bound edge at each end
        for name, x, w in (('band', -0.06, 0.012), ('edge_l', -0.149, 0.004), ('edge_r', 0.149, 0.004)):
            arc = [(x, 0.33 + 0.0605 * math.sin(math.radians(a)), 0.03 + 0.0605 * math.cos(math.radians(a)))
                   for a in range(-22, 204, 15)]
            F.sweep(name, arc, None, M.brass, profile=G.rounded_rect(w, 0.003, 0.0012, corner=1), up=(1, 0, 0),
                    resolution=2)
        G.rod('lampBanker_bulb', F.at(-0.13, 0.327, 0.03), F.at(0.13, 0.327, 0.03), 0.009, material=M.bulb, verts=10)
        _near_light('lampBanker', 0.327)
        G.kit.tube(F, 'chain', [(0.08, 0.33, 0.07), (0.08, 0.245, 0.07)], 0.0012, M.brass, verts=4, round=1)
        for k in range(5):
            F.sphere(f'bead{k}', 0.0025, (0.08, 0.32 - k * 0.016, 0.07), M.brass, segments=6, rings=4)
        _turned(F, 'pull', [(0, 0), (0.003, 0), (0.007, 0.006), (0.006, 0.014), (0.002, 0.018), (0, 0.018)],
                (0.08, 0.228, 0.07), M.brass, 10)
    G.join_all('lampBanker')
    G.settle()
    return G.meshes()


MUSHROOM = [[0.5, -0.5], [0.46, 0.15], [0.3, 0.42], [0, 0.5], [0, 0.4], [0.28, 0.32], [0.44, 0.1], [0.47, -0.5]]


def lampMushroom():
    """A white porcelain foot rising into a slim stem that flares under a coral glass cap with a rolled lip."""
    F = G.GameFrame()
    with G.new_parts():
        _turned(F, 'base', [(0, 0), (0.07, 0), (0.072, 0.006), (0.068, 0.014), (0.05, 0.02), (0.024, 0.026), (0, 0.026)],
                (0, 0, 0), M.porcelain, 32)
        _turned(F, 'stem', [(0, 0), (0.02, 0), (0.0145, 0.03), (0.0105, 0.09), (0.009, 0.15), (0.013, 0.18),
                            (0.022, 0.19), (0, 0.19)], (0, 0.02, 0), M.porcelain, 16)
        _lathe(F, 'cap', 0.3, 0.16, (0, 0.28, 0), MUSHROOM, M.cap, segments=36)
        _ring(F, 'cap_lip', 0.146, (0, 0.202, 0), 0.004, M.cap, segments=36, minor=5)
        _bulb(F, 'lampMushroom', 0.05, (0, 0.22, 0), scale=(1, 1, 0.5))
    G.join_all('lampMushroom')
    G.settle()
    return G.meshes()


def lampLantern():
    """A candle lantern: a black base, four posts and rails round the glass, a roof curving up to a knob and a
    carrying ring, a latch; a candle with a drip down its side, burning inside."""
    F = G.GameFrame()
    with G.new_parts():
        F.extrude('base', G.rounded_rect(0.16, 0.16, 0.012, corner=2), 0.02, (0, 0.01, 0), M.iron, plane='top',
                  bevel=0.004, bevel_segments=1)
        F.box('glass', 0.128, 0.22, 0.128, (0, 0.13, 0), M.glass)
        for x in (-0.07, 0.07):
            for z in (-0.07, 0.07):
                F.box(f'post{x}{z}', 0.016, 0.22, 0.016, (x, 0.13, z), M.iron, bevel=0.003, bevel_segments=1)
        frame = [(-0.075, -0.075), (0.075, -0.075), (0.075, 0.075), (-0.075, 0.075)]
        rail = [(0, 0), (0.012, 0), (0.012, 0.01), (0, 0.01)]
        G.kit.moulding(F, 'rail_low', frame, rail, M.iron, at=0.02, plane='top')
        G.kit.moulding(F, 'rim', [(x * 1.07, z * 1.07) for x, z in frame], [(0, 0), (0.018, 0), (0.018, 0.012), (0, 0.012)],
                       M.iron, at=0.24, plane='top')
        _loft(F, 'roof', 0, 0, [(0.252, 0.074, 0.074, 12), (0.262, 0.066, 0.066, 10), (0.28, 0.046, 0.046, 8),
                                (0.3, 0.026, 0.026, 6), (0.318, 0.012, 0.012, 4)], M.iron, verts=16)
        _turned(F, 'knob', [(0, 0), (0.014, 0), (0.014, 0.006), (0.008, 0.014), (0, 0.016)], (0, 0.316, 0), M.iron, 10)
        G.torus('ring', 0.025, 0.004, at=F.at(0, 0.345, 0), material=M.iron, rot=(90, 0, 0), major_segments=14,
                minor_segments=4)
        F.box('latch', 0.014, 0.024, 0.006, (0.064, 0.15, 0.076), M.iron, bevel=0.002, bevel_segments=1)
        _turned(F, 'candle', [(0, 0), (0.025, 0), (0.025, 0.094), (0.022, 0.1), (0.004, 0.102), (0, 0.102)], (0, 0.02, 0),
                M.candle, 16)
        G.kit.tube(F, 'drip', [(0.025, 0.118, 0.004), (0.026, 0.09, 0.006), (0.0255, 0.06, 0.004)], 0.004, M.candle, verts=5,
                   round=2, scale=[1.0, 0.8, 0.6])
        G.kit.tube(F, 'wick', [(0, 0.12, 0), (0, 0.13, 0)], 0.0012, M.iron, verts=4, round=1)
        _bulb(F, 'lampLantern', 0.011, (0, 0.14, 0), material=M.flame, scale=(1, 1, 1.7), segments=10, rings=6)
    G.join_all('lampLantern')
    G.settle()
    return G.meshes()


IDLE_FAN = {'idle': 'blades_turn'}


def _m(id, build, size, paint, **kw):
    return G.Model(id, build, size=size, key=f'model.{id}', paint=paint, **kw)


FAMILY = G.Family(
    'lighting', M,
    [_m('ceilingPendant', ceilingPendant, (0.43, 0.688, 0.43), [('brass', 'Body'), ('dome', 'Shade')], floor=False),
     _m('ceilingLight', ceilingLight, (0.36, 0.6, 0.36), [('iron', 'Body'), ('panel', 'Shade')], floor=False),
     _m('chandelier', chandelier, (0.94, 0.763, 0.897), [('brass', 'Body'), ('bulb', 'Globes')], floor=False),
     _m('pendantIndustrial', pendantIndustrial, (0.38, 0.874, 0.38), [('iron', 'Body'), ('industrial', 'Shade')],
        floor=False),
     _m('ceilingFan', ceilingFan, (0.905, 0.385, 0.951), [('white', 'Body'), ('blade', 'Blades')], floor=False,
        centre=(-0.0374, 0), motions=IDLE_FAN),   # its box's middle; the blades' axis is on the tile's
     _m('trackLights', trackLights, (0.94, 0.296, 0.19), [('white', 'Body'), ('iron', 'Shade')], floor=False),
     _m('paperLantern', paperLantern, (0.543, 0.681, 0.543), [('paper', 'Paper'), ('rib', 'Ribs')], floor=False),
     _m('wallSconce', wallSconce, (0.2, 0.25, 0.27), [('brass', 'Body'), ('shade', 'Shade')], floor=False,
        centre=(0, -0.295)),
     _m('wallLampArm', wallLampArm, (0.2, 0.296, 0.424), [('iron', 'Body'), ('shade_metal', 'Shade')], floor=False,
        centre=(0, -0.218)),
     _m('wallLightOutdoor', wallLightOutdoor, (0.22, 0.327, 0.11), [('iron', 'Body'), ('lens', 'Shade')], floor=False,
        centre=(0, -0.375)),
     _m('lamp', lamp, (0.45, 1.808, 0.45), [('stand', 'Stand'), ('lamp_shade', 'Shade'), ('brass', 'Fittings')]),
     _m('floorLampSimple', floorLampSimple, (0.28, 1.6, 0.28), [('iron', 'Stand'), ('shade', 'Shade')]),
     _m('lampFloorArc', lampFloorArc, (0.85, 1.861, 0.81), [('arc', 'Stand'), ('arc_shade', 'Shade'), ('marble', 'Base')],
        centre=(0, -0.025)),   # its base's back on the wall face
     _m('lampFloorTripod', lampFloorTripod, (0.554, 1.614, 0.557), [('walnut', 'Stand'), ('linen', 'Shade')]),
     _m('lampTorchiere', lampTorchiere, (0.4, 1.85, 0.4), [('brass', 'Stand'), ('bowl', 'Shade'), ('iron', 'Base')]),
     _m('tableLamp', tableLamp, (0.66, 0.92, 0.42), [('ceramic', 'Base'), ('shade', 'Shade'), ('oak', 'Table')]),
     _m('lampTableRound', lampTableRound, (0.34, 0.577, 0.34), [('jar', 'Base'), ('shade', 'Shade')]),
     _m('lampTableSquare', lampTableSquare, (0.212, 0.54, 0.212), [('stone', 'Base'), ('square_shade', 'Shade')]),
     _m('lampBanker', lampBanker, (0.3, 0.39, 0.16), [('brass', 'Base'), ('banker', 'Shade')]),
     _m('lampMushroom', lampMushroom, (0.3, 0.36, 0.3), [('porcelain', 'Base'), ('cap', 'Shade')]),
     _m('lampLantern', lampLantern, (0.16, 0.373, 0.16), [('iron', 'Base'), ('glass', 'Glass')])],
    comment='Blender-built lamps (tools/art/blender/families/lighting.py). Listed in the base manifest\'s '
            'packs. The materials map onto the game\'s finishes so a model costs one draw call per finish '
            'group. Shades, diffusers and bulbs glow at night (`glow`); the ceiling fan\'s blades turn while it is '
            'on, near the camera (`idle`).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
