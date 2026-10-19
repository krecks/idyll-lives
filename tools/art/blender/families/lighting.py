# The lamps: the ceiling lights (LT1: `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`,
# `ceilingFan`, `trackLights`, `paperLantern`), the wall lights (LT2: `wallSconce`, `wallLampArm`,
# `wallLightOutdoor`), the floor lamps (LT3: `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`,
# `lampTorchiere`) and the table lamps (LT4: `tableLamp`, `lampTableRound`, `lampTableSquare`, `lampBanker`,
# `lampMushroom`, `lampLantern`). Notes: lighting.md.
#
# The light itself is the content's `light`; each lamp's bulb is placed near its `light.height` (asserted
# against the content JSON). The shades, diffusers and bulbs glow at night (`glow`, E8); nothing glows by
# day. Ceiling things hang from 2.8 m and wall things have their back on the wall face, both at their real
# height (`floor=False`). Two clips, both ambient (`idle`): the ceiling fan's blades turn, the paper lantern
# sways. The rest are still.
#
# Run:  cargo art lighting [--only chandelier] [--preview]
import json
import math
import os
import sys

import bmesh
import bpy

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
    banker=('#2F6B4F', 'gloss'),
    porcelain=('#F2F0EA', 'gloss'),
    cap=('#E9A07B', 'gloss', GLOW),
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


def _drum(F, name, r_bottom, r_top, h, at, material, segments=28, t=0.006, rot=(0, 0, 0)):
    """An open drum or cone shade, `t` thick, its bottom at game `at`."""
    G.revolve(name, [(r_bottom, 0), (r_top, h), (r_top - t, h), (r_bottom - t, 0), (r_bottom, 0)],
              at=F.at(*at), material=material, segments=segments, smooth=segments > 4, rot=rot)


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
    obj.location = at
    return obj


def _hang(F, id, cord_bottom, canopy=None, canopy_r=0.06):
    """A ceiling canopy (brass, or `canopy`) and the black cord down from it to `cord_bottom`."""
    canopy = canopy or M.brass
    F.cyl(f'{id}_canopy', canopy_r, 0.03, (0, CEILING - 0.015, 0), canopy, verts=20, bevel=0.006)
    G.rod(f'{id}_cord', F.at(0, CEILING - 0.03, 0), F.at(0, cord_bottom, 0), 0.005, material=M.iron, verts=6)


DOME = [[0.5, -0.5], [0.47, 0.0], [0.36, 0.3], [0.14, 0.47], [0, 0.5], [0, 0.42], [0.13, 0.4], [0.33, 0.24],
        [0.44, -0.06], [0.47, -0.5]]


# --- LT1: the ceiling lights -----------------------------------------------------------------------------
def ceilingPendant():
    F = G.GameFrame()
    with G.new_parts():
        _hang(F, 'ceilingPendant', 2.13)
        F.cyl('socket', 0.03, 0.05, (0, 2.11, 0), M.brass, verts=14)
        _lathe(F, 'dome', 0.43, 0.24, (0, 1.98, 0), DOME, M.dome, segments=32)
        _bulb(F, 'ceilingPendant', 0.04, (0, 1.9, 0), scale=(1, 1, 1.2))
    G.join_all('ceilingPendant')
    G.settle(floor=False)
    return G.meshes()


def ceilingLight():
    """A square lantern on a short stem: four frosted sides between black corner posts, a diffuser below."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('canopy', 0.12, 0.03, 0.12, (0, CEILING - 0.015, 0), M.iron, bevel=0.006)
        F.cyl('stem', 0.01, 0.28, (0, 2.645, 0), M.iron, verts=8)
        F.box('lid', 0.36, 0.014, 0.36, (0, 2.507, 0), M.iron, bevel=0.004)
        for s in (-1, 1):
            F.box(f'side_z{s}', 0.32, 0.29, 0.006, (0, 2.355, s * 0.163), M.panel)
            F.box(f'side_x{s}', 0.006, 0.29, 0.32, (s * 0.163, 2.355, 0), M.panel)
            for t in (-1, 1):
                F.box(f'post{s}{t}', 0.016, 0.31, 0.016, (s * 0.17, 2.355, t * 0.17), M.iron, bevel=0.004)
        F.box('rim', 0.36, 0.012, 0.36, (0, 2.206, 0), M.iron, bevel=0.003)
        F.box('diffuser', 0.32, 0.004, 0.32, (0, 2.214, 0), M.bulb)
        _bulb(F, 'ceilingLight', 0.05, (0, 2.32, 0), scale=(1, 1, 1.2))
    G.join_all('ceilingLight')
    G.settle(floor=False)
    return G.meshes()


def chandelier():
    """Two brass rings on three arms from a central rod, ten globes on the lower ring and six on the upper."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('canopy', 0.08, 0.03, (0, CEILING - 0.015, 0), M.brass, verts=20, bevel=0.006)
        G.rod('rod', F.at(0, CEILING - 0.03, 0), F.at(0, 2.08, 0), 0.007, material=M.brass, verts=8)
        F.sphere('hub', 0.025, (0, 2.35, 0), M.brass, segments=12, rings=8)
        F.sphere('finial', 0.022, (0, 2.06, 0), M.brass, segments=12, rings=8)
        G.torus('ring_low', 0.435, 0.013, at=F.at(0, 2.05, 0), material=M.brass, major_segments=36,
                minor_segments=6)
        G.torus('ring_high', 0.27, 0.011, at=F.at(0, 2.21, 0), material=M.brass, major_segments=28,
                minor_segments=6)
        for a in (30, 150, 270):
            G.rod(f'arm{a}', F.at(0, 2.35, 0), F.at(0.435 * G.cosd(a), 2.05, 0.435 * G.sind(a)), 0.006,
                  material=M.brass, verts=8)
        for a in (90, 210, 330):
            G.rod(f'spoke{a}', F.at(0, 2.21, 0), F.at(0.27 * G.cosd(a), 2.21, 0.27 * G.sind(a)), 0.005,
                  material=M.brass, verts=6)
        for ring, n, r, y, size in ((0.435, 10, 0.035, 2.1, 0.02), (0.27, 6, 0.03, 2.25, 0.017)):
            for k in range(n):
                a = 360 * k / n + (15 if n == 6 else 0)
                x, z = ring * G.cosd(a), ring * G.sind(a)
                F.cyl(f'cup{y}_{k}', size, 0.02, (x, y - r - 0.002, z), M.brass, verts=10)
                F.sphere(f'globe{y}_{k}', r, (x, y, z), M.bulb, segments=10, rings=6)
        _bulb(F, 'chandelier', 0.03, (0, 2.12, 0), segments=10, rings=6)   # a drop under the hub
    G.join_all('chandelier')
    G.settle(floor=False)
    return G.meshes()


def pendantIndustrial():
    """A green enamel dome, white inside, on a black cord."""
    F = G.GameFrame()
    with G.new_parts():
        _hang(F, 'pendantIndustrial', 2.22, canopy=M.iron)
        F.cyl('socket', 0.03, 0.05, (0, 2.2, 0), M.iron, verts=14)
        _lathe(F, 'dome', 0.38, 0.24, (0, 2.07, 0), DOME, M.industrial, segments=32, liner=M.liner)
        _bulb(F, 'pendantIndustrial', 0.045, (0, 1.98, 0), scale=(1, 1, 1.2))
    G.join_all('pendantIndustrial')
    G.settle(floor=False)
    return G.meshes()


def ceilingFan():
    """A white fan with five wooden blades on irons and a light dome below. The blades turn (`blades_turn`,
    ambient)."""
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('canopy', 0.08, 0.05, (0, CEILING - 0.025, 0), M.white, verts=20, bevel=0.01)
        F.cyl('downrod', 0.015, 0.14, (0, 2.69, 0), M.white, verts=10)
        F.cone('motor', 0.13, 0.0975, 0.12, (0, 2.56, 0), M.white, verts=28, bevel=0.01)
        F.cyl('switch_cup', 0.1, 0.04, (0, 2.48, 0), M.white, verts=24, bevel=0.008)
        _bulb(F, 'ceilingFan', 0.09, (0, 2.46, 0), scale=(1, 1, 0.5), segments=20, rings=10)
    G.join(still, 'ceilingFan', origin=(0, 0, 0))
    # The blades reach from just outside the motor (0.135 m out) to 0.485 m, their tips' corners 0.487 m: as
    # they turn they sweep a circle 0.974 m across, inside the 1 x 1 tile with its axis on the tile's middle.
    with G.new_parts() as rotor:
        for k in range(5):
            a = 72 * k
            G.cube(f'blade{k}', (0.35, 0.10, 0.012), at=(0.31 * G.cosd(a), 0.31 * G.sind(a), 2.54),
                   material=M.blade, rot=(10, 0, a), bevel=0.004)
            G.cube(f'iron{k}', (0.125, 0.035, 0.015), at=(0.1375 * G.cosd(a), 0.1375 * G.sind(a), 2.548),
                   material=M.white, rot=(0, 0, a), bevel=0.005)
    blades = G.hinged(G.join(rotor, 'blades'), F.at(0, 2.54, 0))
    # Left where it's built, its axis on the middle: five blades reach farther one way than the other, so
    # centring its box would put the axis 3.3 cm off (the Model's `centre` is the box's middle instead).
    G.settle(centre=False, floor=False)
    G.spin(blades, 'blades_turn', turns=1, seconds=2.5, axis='Z')
    return G.meshes()


def trackLights():
    """A white track with three black spots, each aimed its own way."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('track', 0.94, 0.04, 0.05, (0, 2.78, 0), M.white, bevel=0.01)
        F.box('canopy', 0.12, 0.02, 0.12, (0, 2.79, 0), M.white, bevel=0.006)
        for k, (x, aim) in enumerate(((-0.34, (-0.2, -1, 0.34)), (0.0, (0.16, -1, -0.26)), (0.34, (-0.26, -1, 0.42)))):
            n = math.sqrt(sum(c * c for c in aim))
            d = [c / n for c in aim]
            top = (x, 2.68, 0)
            G.rod(f'stem{k}', F.at(x, 2.76, 0), F.at(*top), 0.01, material=M.iron, verts=8)
            F.sphere(f'knuckle{k}', 0.016, top, M.iron, segments=10, rings=6)
            pt = lambda s: F.at(*(top[i] + s * d[i] for i in range(3)))  # noqa: E731
            G.rod(f'head{k}', pt(0.01), pt(0.17), 0.045, material=M.iron, verts=16)
            G.rod(f'lens{k}', pt(0.168), pt(0.172), 0.036, material=M.bulb, verts=14)
            _near_light('trackLights', top[1] + 0.17 * d[1])
    G.join_all('trackLights')
    G.settle(floor=False)
    return G.meshes()


def paperLantern():
    """A paper globe on wire ribs, hanging from a white cord. It sways (`sway`, ambient)."""
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('canopy', 0.04, 0.02, (0, CEILING - 0.01, 0), M.white, verts=14, bevel=0.004)
    G.join(still, 'paperLantern', origin=(0, 0, 0))
    with G.new_parts() as hung:
        G.rod('cord', F.at(0, CEILING - 0.02, 0), F.at(0, 2.29, 0), 0.004, material=M.white, verts=6)
        F.cyl('cap', 0.04, 0.012, (0, 2.305, 0), M.white, verts=14)
        _bulb(F, 'paperLantern', 0.27, (0, 2.05, 0), material=M.paper, scale=(1, 1, 0.93), segments=24, rings=14)
        for k, y in enumerate((1.85, 1.93, 2.01, 2.09, 2.17, 2.25)):
            r = 0.27 * math.sqrt(max(0.0, 1 - ((y - 2.05) / (0.27 * 0.93)) ** 2)) + 0.002
            G.torus(f'rib{k}', r, 0.003, at=F.at(0, y, 0), material=M.rib, major_segments=28, minor_segments=4)
    lantern = G.hinged(G.join(hung, 'lantern'), F.at(0, CEILING - 0.02, 0))
    G.settle(floor=False)
    G.rock(lantern, 'sway', 2.5, seconds=2.0, axis='X')
    return G.meshes()


# --- LT2: the wall lights ----------------------------------------------------------------------------------
def wallSconce():
    """A brass wall plate and arm holding a cone shade, open below and above."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('plate', 0.06, 0.02, (0, 1.7, WALL + 0.01), M.brass, rot=(90, 0, 0), verts=20, bevel=0.005)
        G.rod('arm', F.at(0, 1.7, WALL + 0.02), F.at(0, 1.735, -0.26), 0.01, material=M.brass, verts=8)
        F.cyl('cup', 0.025, 0.03, (0, 1.735, -0.26), M.brass, verts=12)
        for a in (90, 210, 330):
            G.rod(f'spoke{a}', F.at(0, 1.745, -0.26), F.at(0.087 * G.cosd(a), 1.745, -0.26 + 0.087 * G.sind(a)),
                  0.003, material=M.brass, verts=4)
        _bulb(F, 'wallSconce', 0.03, (0, 1.775, -0.26), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.1, 0.075, 0.18, (0, 1.71, -0.26), M.shade)
    G.join_all('wallSconce')
    G.settle(centre=False, floor=False)
    return G.meshes()


ARM_SHADE = [[0.5, -0.5], [0.2, 0.4], [0.1, 0.5], [0, 0.5], [0, 0.42], [0.09, 0.42], [0.46, -0.5]]


def wallLampArm():
    """A black swing-arm lamp: a wall plate, two arms on a brass knuckle and a tilted cone shade."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('plate', 0.055, 0.02, (0, 1.55, WALL + 0.01), M.iron, rot=(90, 0, 0), verts=18, bevel=0.005)
        G.rod('arm_low', F.at(0, 1.55, WALL + 0.02), F.at(0, 1.6, -0.25), 0.009, material=M.iron, verts=8)
        F.sphere('knuckle', 0.0175, (0, 1.6, -0.25), M.brass, segments=10, rings=6)
        G.rod('arm_high', F.at(0, 1.6, -0.25), F.at(0, 1.775, -0.185), 0.008, material=M.iron, verts=8)
        _lathe(F, 'shade', 0.2, 0.16, (0, 1.78, -0.18), ARM_SHADE, M.shade_metal, segments=24, rot=(-35, 0, 0),
               top=True, liner=M.liner)
        _bulb(F, 'wallLampArm', 0.032, (0, 1.69, -0.125), scale=(1, 1, 1.15))
    G.join_all('wallLampArm')
    G.settle(centre=False, floor=False)
    return G.meshes()


def wallLightOutdoor():
    """A bulkhead: an oval glass lens in a black frame, under a hood, behind a cage of three bars."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('plate', 0.2, 0.3, 0.03, (0, 2.01, WALL + 0.015), M.iron, bevel=0.012)
        G.torus('frame', 1.0, 0.11, at=F.at(0, 2.01, WALL + 0.03), material=M.iron, rot=(90, 0, 0),
                major_segments=28, minor_segments=5).scale = (0.074, 0.12, 0.12)
        lens = F.sphere('lens', 1.0, (0, 2.01, WALL + 0.03), M.lens, scale=(0.075, 0.06, 0.125), segments=20, rings=10)
        for v in lens.data.vertices:      # its back half, hidden in the plate, flattened onto the wall face
            if v.co.y > 0:
                v.co.y *= 0.5
        _bulb(F, 'wallLightOutdoor', 0.035, (0, 2.01, -0.38), scale=(1, 0.8, 1.2), segments=10, rings=6)
        F.box('hood', 0.22, 0.025, 0.11, (0, 2.175, WALL + 0.055), M.iron, bevel=0.008)
        # the cage, just clear of the lens: a vertical bar and three cross bars on two side posts
        F.box('bar_v', 0.012, 0.26, 0.012, (0, 2.01, -0.328), M.iron, bevel=0.004, bevel_segments=1)
        for y in (1.94, 2.01, 2.08):
            F.box(f'bar{y}', 0.182, 0.012, 0.012, (0, y, -0.328), M.iron, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            F.box(f'post{s}', 0.012, 0.16, 0.072, (s * 0.085, 2.01, -0.364), M.iron, bevel=0.004, bevel_segments=1)
    G.join_all('wallLightOutdoor')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- LT3: the floor lamps ----------------------------------------------------------------------------------
def lamp():
    """The classic floor lamp: a dark stand with brass collars, a drum shade on a harp, a pull chain."""
    F = G.GameFrame()
    with G.new_parts():
        F.cone('foot', 0.15, 0.138, 0.03, (0, 0.015, 0), M.stand, verts=28, bevel=0.006)
        F.cone('foot_top', 0.1, 0.05, 0.03, (0, 0.045, 0), M.stand, verts=24)
        F.cyl('pole', 0.012, 1.38, (0, 0.75, 0), M.stand, verts=10)
        for y in (0.5, 1.0):
            F.cyl(f'collar{y}', 0.02, 0.03, (0, y, 0), M.brass, verts=12, bevel=0.004)
        F.cyl('socket', 0.025, 0.08, (0, 1.47, 0), M.brass, verts=12)
        _bulb(F, 'lamp', 0.035, (0, 1.54, 0), scale=(1, 1, 1.25))
        for s in (-1, 1):
            G.rod(f'harp{s}', F.at(s * 0.02, 1.45, 0), F.at(s * 0.005, 1.775, 0), 0.004, material=M.brass, verts=6)
        F.sphere('finial', 0.0175, (0, 1.79, 0), M.brass, segments=10, rings=6)
        _drum(F, 'shade', 0.22, 0.158, 0.34, (0, 1.43, 0), M.lamp_shade, segments=32)
        G.torus('trim_low', 0.218, 0.007, at=F.at(0, 1.436, 0), material=M.trim, major_segments=32, minor_segments=4)
        G.torus('trim_high', 0.157, 0.007, at=F.at(0, 1.764, 0), material=M.trim, major_segments=28, minor_segments=4)
        G.rod('chain', F.at(0.04, 1.43, 0), F.at(0.04, 1.31, 0), 0.002, material=M.brass, verts=4)
        F.sphere('pull', 0.008, (0.04, 1.3, 0), M.brass, scale=(1, 1, 1.3), segments=8, rings=6)
    G.join_all('lamp')
    G.settle()
    return G.meshes()


def floorLampSimple():
    """A square black base and stem under a tall square shade, a diffuser at its foot."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('base', 0.28, 0.03, 0.28, (0, 0.015, 0), M.iron, bevel=0.008)
        F.cyl('stem', 0.011, 0.84, (0, 0.45, 0), M.iron, verts=8)
        _drum(F, 'shade', 0.19, 0.19, 0.74, (0, 0.86, 0), M.shade, segments=4, t=0.008, rot=(0, 0, 45))
        F.box('diffuser', 0.255, 0.004, 0.255, (0, 0.866, 0), M.bulb)
        _bulb(F, 'floorLampSimple', 0.045, (0, 1.45, 0), scale=(1, 1, 1.2))
        G.rod('holder', F.at(0, 0.87, 0), F.at(0, 1.4, 0), 0.006, material=M.iron, verts=6)
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
    """A marble base in the back corner and a brass arc reaching over to a dome shade."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('base', 0.15, 0.06, (-0.3, 0.03, -0.3), M.marble, verts=28, bevel=0.012)
        G.rod('stem', F.at(-0.3, 0.06, -0.3), F.at(*ARC[0]), 0.0125, material=M.arc, verts=10)
        pts = [_bezier(ARC, k / 14) for k in range(15)]
        for k in range(14):
            G.rod(f'arc{k}', F.at(*pts[k]), F.at(*pts[k + 1]), 0.0125, material=M.arc, verts=10)
            F.sphere(f'joint{k}', 0.0125, pts[k + 1], M.arc, segments=10, rings=6)
        F.cyl('socket', 0.02, 0.04, (0.22, 1.71, 0.18), M.arc, verts=12)
        _lathe(F, 'shade', 0.36, 0.2, (0.22, 1.6, 0.18), ARC_SHADE, M.arc_shade, segments=28, liner=M.liner)
        _bulb(F, 'lampFloorArc', 0.05, (0.22, 1.56, 0.18), scale=(1, 1, 0.75))
    G.join_all('lampFloorArc')
    G.settle()
    return G.meshes()


def lampFloorTripod():
    """Three walnut legs meeting at a brass hub under a linen drum shade."""
    F = G.GameFrame()
    with G.new_parts():
        for a in (90, 210, 330):
            c, s = G.cosd(a), G.sind(a)
            G.rod(f'leg{a}', F.at(0.3 * c, 0.0, 0.3 * s), F.at(0.018 * c, 1.27, 0.018 * s), 0.0175,
                  material=M.walnut, verts=10)
        F.cyl('hub', 0.03, 0.08, (0, 1.28, 0), M.brass, verts=14, bevel=0.005)
        _bulb(F, 'lampFloorTripod', 0.04, (0, 1.4, 0), scale=(1, 1, 1.2))
        _drum(F, 'shade', 0.24, 0.24, 0.32, (0, 1.29, 0), M.linen, segments=32)
        F.cyl('diffuser', 0.234, 0.004, (0, 1.3, 0), M.bulb, verts=32)
    G.join_all('lampFloorTripod')
    G.settle()
    return G.meshes()


BOWL = [[0, -0.5], [0.04, -0.5], [0.3, -0.1], [0.5, 0.5], [0.46, 0.5], [0.27, -0.06], [0, -0.4]]


def lampTorchiere():
    """An uplighter: a brass pole on a black disc, a frosted glass bowl at the top."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('base', 0.15, 0.04, (0, 0.02, 0), M.iron, verts=28, bevel=0.01)
        F.cyl('pole', 0.0125, 1.68, (0, 0.88, 0), M.brass, verts=10)
        F.cyl('collar', 0.025, 0.04, (0, 1.72, 0), M.brass, verts=14, bevel=0.005)
        _lathe(F, 'bowl', 0.4, 0.14, (0, 1.78, 0), BOWL, M.bowl, segments=32)
        _bulb(F, 'lampTorchiere', 0.035, (0, 1.77, 0))
    G.join_all('lampTorchiere')
    G.settle()
    return G.meshes()


# --- LT4: the table lamps --------------------------------------------------------------------------------
GOURD = [[0, -0.5], [0.32, -0.5], [0.46, -0.3], [0.5, -0.05], [0.4, 0.25], [0.24, 0.42], [0.26, 0.5], [0.2, 0.5],
         [0.18, 0.4], [0, 0.38]]
JAR = [[0, -0.5], [0.3, -0.5], [0.48, -0.25], [0.5, 0.05], [0.38, 0.38], [0.15, 0.45], [0.12, 0.5], [0, 0.5]]


def tableLamp():
    """An oak side table with a shelf, a ceramic gourd lamp with a drum shade on it."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('top', 0.66, 0.03, 0.42, (0, 0.485, 0), M.oak, bevel=0.01)
        F.box('shelf', 0.58, 0.02, 0.34, (0, 0.14, 0), M.oak, bevel=0.006)
        for x in (-0.28, 0.28):
            for z in (-0.16, 0.16):
                F.cone(f'leg{x}{z}', 0.015, 0.0225, 0.47, (x, 0.235, z), M.oak, verts=12)
        _lathe(F, 'base', 0.17, 0.2, (-0.12, 0.6, 0), GOURD, M.ceramic, segments=20)
        F.cyl('stem', 0.008, 0.1, (-0.12, 0.74, 0), M.brass, verts=8)
        _bulb(F, 'tableLamp', 0.035, (-0.12, 0.81, 0), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.15, 0.1275, 0.2, (-0.12, 0.72, 0), M.shade)
        F.cyl('diffuser', 0.143, 0.004, (-0.12, 0.723, 0), M.bulb, verts=24)
    G.join_all('tableLamp')
    G.settle()
    return G.meshes()


def lampTableRound():
    """A blue ginger jar with a brass neck under a drum shade."""
    F = G.GameFrame()
    with G.new_parts():
        _lathe(F, 'jar', 0.2, 0.26, (0, 0.13, 0), JAR, M.jar, segments=24)
        F.cyl('neck', 0.022, 0.02, (0, 0.265, 0), M.brass, verts=14, bevel=0.004)
        F.cyl('stem', 0.01, 0.1, (0, 0.31, 0), M.brass, verts=8)
        _bulb(F, 'lampTableRound', 0.03, (0, 0.4, 0), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.17, 0.1445, 0.22, (0, 0.33, 0), M.shade)
        F.cyl('diffuser', 0.162, 0.004, (0, 0.34, 0), M.bulb, verts=24)
        F.sphere('finial', 0.012, (0, 0.565, 0), M.brass, segments=10, rings=6)
        G.rod('finial_stem', F.at(0, 0.45, 0), F.at(0, 0.56, 0), 0.003, material=M.brass, verts=4)
    G.join_all('lampTableRound')
    G.settle()
    return G.meshes()


def lampTableSquare():
    """A square stone block, a black stem and a tapered square shade."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('base', 0.12, 0.26, 0.12, (0, 0.13, 0), M.stone, bevel=0.01)
        F.cyl('stem', 0.009, 0.1, (0, 0.31, 0), M.iron, verts=8)
        _bulb(F, 'lampTableSquare', 0.03, (0, 0.4, 0), scale=(1, 1, 1.15))
        _drum(F, 'shade', 0.15, 0.12, 0.2, (0, 0.34, 0), M.square_shade, segments=4, rot=(0, 0, 45))
        F.box('diffuser', 0.2, 0.004, 0.2, (0, 0.35, 0), M.bulb)
    G.join_all('lampTableSquare')
    G.settle()
    return G.meshes()


def lampBanker():
    """A banker's lamp: a brass base and upright, a green half-round shade, white inside, a pull chain."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('base', 0.22, 0.02, 0.14, (0, 0.01, 0), M.brass, bevel=0.008)
        F.cyl('foot', 0.02, 0.012, (-0.06, 0.026, 0), M.brass, verts=12)
        F.cyl('upright', 0.009, 0.29, (-0.06, 0.165, 0), M.brass, verts=8)
        G.rod('arm', F.at(-0.06, 0.305, 0), F.at(-0.02, 0.325, 0.03), 0.008, material=M.brass, verts=8)
        _trough('shade', 0.06, 0.3, 0.004, F.at(0, 0.33, 0.03), M.banker)
        _trough('shade_liner', 0.0555, 0.292, 0.003, F.at(0, 0.33, 0.03), M.liner)
        G.rod('lampBanker_bulb', F.at(-0.13, 0.327, 0.03), F.at(0.13, 0.327, 0.03), 0.009, material=M.bulb, verts=10)
        _near_light('lampBanker', 0.327)
        G.rod('chain', F.at(0.08, 0.33, 0.07), F.at(0.08, 0.24, 0.07), 0.0015, material=M.brass, verts=4)
        F.sphere('pull', 0.007, (0.08, 0.235, 0.07), M.brass, scale=(1, 1, 1.3), segments=8, rings=6)
    G.join_all('lampBanker')
    G.settle()
    return G.meshes()


MUSHROOM = [[0.5, -0.5], [0.46, 0.15], [0.3, 0.42], [0, 0.5], [0, 0.4], [0.28, 0.32], [0.44, 0.1], [0.47, -0.5]]


def lampMushroom():
    """A white stem on a disc under a coral glass cap."""
    F = G.GameFrame()
    with G.new_parts():
        F.cyl('base', 0.07, 0.02, (0, 0.01, 0), M.porcelain, verts=24, bevel=0.006)
        F.cone('stem', 0.015, 0.0075, 0.2, (0, 0.11, 0), M.porcelain, verts=12)
        _lathe(F, 'cap', 0.3, 0.16, (0, 0.28, 0), MUSHROOM, M.cap, segments=28)
        _bulb(F, 'lampMushroom', 0.05, (0, 0.22, 0), scale=(1, 1, 0.5))
    G.join_all('lampMushroom')
    G.settle()
    return G.meshes()


def lampLantern():
    """A candle lantern: a black frame round glass, a pyramid roof and a ring, a candle burning inside."""
    F = G.GameFrame()
    with G.new_parts():
        F.box('base', 0.16, 0.02, 0.16, (0, 0.01, 0), M.iron, bevel=0.004)
        F.box('glass', 0.128, 0.22, 0.128, (0, 0.13, 0), M.glass)
        for x in (-0.07, 0.07):
            for z in (-0.07, 0.07):
                F.box(f'post{x}{z}', 0.016, 0.22, 0.016, (x, 0.13, z), M.iron, bevel=0.003)
        F.box('rim', 0.16, 0.012, 0.16, (0, 0.246, 0), M.iron, bevel=0.003)
        F.cone('roof', 0.105, 0.012, 0.07, (0, 0.287, 0), M.iron, verts=4, rot=(0, 0, 45), smooth=False)
        G.torus('ring', 0.025, 0.004, at=F.at(0, 0.345, 0), material=M.iron, rot=(90, 0, 0), major_segments=14,
                minor_segments=4)
        F.cyl('candle', 0.025, 0.1, (0, 0.07, 0), M.candle, verts=14)
        _bulb(F, 'lampLantern', 0.011, (0, 0.14, 0), material=M.flame, scale=(1, 1, 1.7), segments=10, rings=6)
    G.join_all('lampLantern')
    G.settle()
    return G.meshes()


IDLE_FAN = {'idle': 'blades_turn'}
IDLE_SWAY = {'idle': 'sway'}


def _m(id, build, size, paint, **kw):
    return G.Model(id, build, size=size, key=f'model.{id}', paint=paint, **kw)


FAMILY = G.Family(
    'lighting', M,
    [_m('ceilingPendant', ceilingPendant, (0.43, 0.948, 0.43), [('brass', 'Body'), ('dome', 'Shade')], floor=False),
     _m('ceilingLight', ceilingLight, (0.36, 0.6, 0.36), [('iron', 'Body'), ('panel', 'Shade')], floor=False),
     _m('chandelier', chandelier, (0.94, 0.763, 0.897), [('brass', 'Body'), ('bulb', 'Globes')], floor=False),
     _m('pendantIndustrial', pendantIndustrial, (0.38, 0.874, 0.38), [('iron', 'Body'), ('industrial', 'Shade')],
        floor=False),
     _m('ceilingFan', ceilingFan, (0.905, 0.385, 0.951), [('white', 'Body'), ('blade', 'Blades')], floor=False,
        centre=(-0.0326, 0), motions=IDLE_FAN),   # its box's middle; the blades' axis is on the tile's
     _m('trackLights', trackLights, (0.94, 0.296, 0.19), [('white', 'Body'), ('iron', 'Shade')], floor=False),
     _m('paperLantern', paperLantern, (0.543, 1.001, 0.543), [('paper', 'Paper'), ('rib', 'Ribs')], floor=False,
        motions=IDLE_SWAY),
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
            'group. Shades, diffusers and bulbs glow at night (`glow`); the ceiling fan\'s blades turn and '
            'the paper lantern sways all the time near the camera (`idle`).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
