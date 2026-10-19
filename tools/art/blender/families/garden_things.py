# OU5, the pond, the small garden things and the garden lights: the garden pond (`model.garden.pond`), the planter
# box (`model.planterBox`), the birdhouse (`model.birdhouse`), the bird feeders (`model.birdFeederPole`), the dog
# house (`model.dogHouse`), the sundial (`model.sundial`), the stepping stones (`model.steppingStones`), the garden
# lamp post (`model.lampPost`) and the string lights (`model.stringLights`). Notes: garden_things.md.
#
# Each is built in game axes (`F`) where the game places it. The pond's water is opaque `gloss`, as the
# fountains'; its ripples spread all the time near the camera (`ripple`, ambient) and a koi comes up out of it to
# be fed or watched (`fish`, `surface`). A bird lands on the feeders while a resident watches (`bird`). The planter
# box's leaves are on `foliage`, which the nature shader sways and tints. The lamp post's panes and the string
# lights' bulbs glow at night (E8), each bulb near its content `light.height`; the string sways (`sway`, ambient).
# The planter box and the lamp post have far versions (the town overview). The rest are still.
#
# Run:  cargo art garden_things [--only sundial] [--preview]
import json
import math
import os
import random
import sys

import bpy
from mathutils import Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import LOW, far_of, leaf, low, lump, star  # noqa: E402
from gamekit.motion import _clip, last_frame  # noqa: E402

SPEC = dict(
    # the pond
    water=('#4F8CA6', 'gloss'),          # opaque, as the fountains' and the old part's
    water_deep=('#3E7890', 'gloss'),
    ripple=('#A8D2DE', 'gloss'),
    pebble=('#8F887C', 'matte'),
    pad=('#5E8E3E', 'satin'),            # lily pads
    lily=('#F2A6C8', 'satin'),
    lily_white=('#F6EEF2', 'satin'),
    lily_heart=('#F2C14E', 'satin'),
    reed=('#6E8E3A', 'satin'),           # still (satin, not foliage): it keeps the pond to three draws
    rush=('#6A4A2E', 'matte'),           # the bulrushes' heads
    koi=('#E8742E', 'satin'),
    koi_white=('#F6F2EC', 'satin'),
    # the planter box
    box_trim=('#7A5A40', 'wood'),
    soil=('#4A3A2C', 'matte'),
    leaf=('#4F8A3E', 'foliage'),
    leaf_dark=('#3B7034', 'foliage'),
    leaf_light=('#6FAA52', 'foliage'),
    bloom=('#E07BC3', 'satin'),
    bloom2=('#F2C14E', 'satin'),
    bloom3=('#F2F0EA', 'satin'),
    bloom4=('#D9483B', 'satin'),
    bloom5=('#9B7FD1', 'satin'),
    eye=('#F2C14E', 'satin'),
    # the birdhouse, the feeders and the birds
    post=('#8E6E52', 'wood'),
    roof=('#C0473A', 'satin'),
    dark=('#2B2C2E', 'matte'),           # openings: the birdhouse's hole, the kennel's door
    iron=('#2A2C30', 'metal'),           # the shepherd's hook, the lamp post
    feeder=('#2F5A4E', 'satin'),         # the feeders' caps and trays
    feeder_glass=('#DCE6EA', 'glass'),
    seed=('#C9A979', 'matte'),
    robin=('#7A5A40', 'satin'),
    robin_breast=('#D9603B', 'satin'),
    tit=('#3E6EB0', 'satin'),
    tit_breast=('#F2D14E', 'satin'),
    beak=('#3A3026', 'satin'),
    # the dog house
    plinth=('#6E6A64', 'matte'),
    plate=('#F2F0EA', 'satin'),
    bowl=('#3E8EDE', 'gloss'),
    bowl_water=('#9DC6CF', 'gloss'),
    # the sundial
    dial=('#B87333', 'metal'),           # copper
    dial_mark=('#6E4A2A', 'metal'),
    # the lights
    lantern=('#2F3236', 'metal'),        # the lamp's lantern frame and roof (the player's "Lantern")
    pane=('#F6EBCF', 'satin', {'glow': '#FFD596'}),    # frosted panes, lit at night
    planter=('#A9A49A', 'matte'),        # the string lights' weighted pots
    wire=('#2A2C30', 'matte'),
    socket=('#3A3D41', 'satin'),
    bulb=('#FFF1CF', 'gloss', {'glow': '#FFD9A0'}),
)

# Each model's `body` (the part the player colours) differs in colour and finish: stones, a wooden box, a cream
# birdhouse, a red kennel, a stone plinth, iron posts. `_own` makes the palette's `body` that one for the model
# being built; the pack's `materials` follow (the family asks the palette right after each build).
BODIES = {
    'pond': ('#B4AEA2', 'matte'),
    'planterBox': ('#8A6A4E', 'wood'),
    'birdhouse': ('#E9DCC4', 'satin'),
    'birdFeederPole': ('#2A2C30', 'metal'),
    'dogHouse': ('#B5653F', 'wood'),
    'sundial': ('#C9C2B4', 'matte'),
    'steppingStones': ('#ABA59A', 'matte'),
    'lampPost': ('#2F3236', 'metal'),
    'stringLights': ('#8E6E52', 'wood'),
}


class _Palettes:
    """The family's palette: the one of the model being built (`Family.run` asks it for the pack's `materials`
    right after each build)."""
    current = None

    def finishes(self, names):
        return self.current.finishes(names)


PALETTES = _Palettes()
M = None


def _own(id):
    """The palette for model `id`: the family's colours and its own `body`."""
    global M
    M = PALETTES.current = G.Palette(**SPEC, body=BODIES[id])
    return M


F = G.GameFrame()
_count = [0]


def _name(stem):
    _count[0] += 1
    return f'{stem}{_count[0]}'


def _content_lights():
    """{object id: light.height} from the content: the bulbs are placed against it."""
    root = os.path.join(G.ASSETS, '..', 'content')
    out = {}
    for path in ('furniture.json', os.path.join('catalog2', 'outdoor.json')):
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


def _mesh(name, verts, faces, material, smooth=False):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    for p in mesh.polygons:
        p.use_smooth = smooth
    return obj


def _prism(name, outline, z0, z1, material):
    """A flat shape `outline` [(game x, height)], counter-clockwise seen from the front, standing from game depth
    `z0` back to `z1` (a gable, a gnomon)."""
    n = len(outline)
    verts = [F.at(x, y, z1) for x, y in outline] + [F.at(x, y, z0) for x, y in outline]
    faces = [tuple(range(n))[::-1], tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        faces.append((i, j, n + j, n + i))
    obj = _mesh(name, verts, faces, material)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')
    return obj


def _bird(x, y, z, body, breast, turn=0.0):
    """A little bird perched at height `y`, looking to the front (or `turn` degrees round towards +x)."""
    c, s = G.cosd(turn), G.sind(turn)
    F.sphere(_name('bird_body'), 0.5, (x, y + 0.03, z), body, scale=(0.055, 0.085, 0.055), rot=(0, 0, -turn),
             segments=12, rings=8)
    F.sphere(_name('bird_head'), 0.023, (x + 0.034 * s, y + 0.072, z + 0.034 * c), body, segments=10, rings=7)
    F.sphere(_name('bird_breast'), 0.5, (x + 0.013 * s, y + 0.026, z + 0.016 * c), breast, scale=(0.045, 0.05, 0.05),
             segments=10, rings=6)
    G.rod(_name('beak'), F.at(x + 0.05 * s, y + 0.074, z + 0.05 * c), F.at(x + 0.068 * s, y + 0.071, z + 0.068 * c),
          0.006, material=M.beak, verts=5)
    G.rod(_name('tail'), F.at(x - 0.035 * s, y + 0.035, z - 0.035 * c), F.at(x - 0.078 * s, y + 0.052, z - 0.078 * c),
          0.012, material=body, verts=5)


# --- the garden pond (2x2): a stone-edged pool with lilies and reeds, ripples, and a koi that comes up ---------
POND = (0.86, 0.76)      # the rim stones' ring (game x, depth radii)
WATER_Y = 0.07           # the surface: the koi lies hidden under it at rest
KOI = (0.18, 0.12)       # where the koi comes up (game x, depth)


def pond():
    _own('pond')
    rng = random.Random(101)
    with G.new_parts() as still:
        rx, rz = POND
        # the water: an opaque gloss block to the surface, its sides hidden under the stones
        F.cyl('water', 1.0, WATER_Y, (0, WATER_Y / 2, 0), M.water, verts=32, scale=(rx - 0.04, rz - 0.04, 1))
        F.cyl('deep', 1.0, 0.002, (0.05, WATER_Y + 0.001, 0.03), M.water_deep, verts=24, scale=(0.5, 0.42, 1))
        # the rim: flat stones round the edge, a few smaller ones and pebbles between them
        n = 18
        for i in range(n):
            a = i / n * math.tau + rng.uniform(-0.05, 0.05)
            r = rng.uniform(0.15, 0.19)
            x, z = rx * math.cos(a), rz * math.sin(a)
            G.boulder(f'stone{i}', r, at=F.at(x, 0, z), material=M.body, scale=(1.15, 0.8, 0.36),
                      rot=(0, 0, math.degrees(a) + 90), seed=i * 1.37, noise=0.25, subdivisions=2)
        for i in range(10):
            a = (i + 0.5) / 10 * math.tau + rng.uniform(-0.1, 0.1)
            x, z = (rx + 0.1) * math.cos(a), (rz + 0.1) * math.sin(a)
            G.boulder(f'pebble{i}', rng.uniform(0.05, 0.07), at=F.at(x, 0, z), material=M.pebble, scale=(1.2, 1, 0.6),
                      seed=i * 2.9, noise=0.3, subdivisions=1)
        # lily pads, each with a notch, and two flowers
        for i, (x, z, r) in enumerate([(-0.3, 0.2, 0.11), (0.28, -0.24, 0.09), (0.42, 0.3, 0.08), (-0.16, -0.36, 0.085),
                                        (-0.02, 0.44, 0.07), (-0.44, -0.08, 0.075)]):
            _pad(f'pad{i}', r, (x, WATER_Y + 0.003, z), rng.uniform(0, 360), M.pad)
        for (x, z), petal in (((-0.27, 0.2), M.lily), ((0.26, -0.22), M.lily_white)):
            for k, (rr, tilt) in enumerate(((0.055, 0), (0.04, 0))):
                star(f'lily{x}_{k}', F.at(x, WATER_Y + 0.012 + 0.012 * k, z), rr, 8, petal, yaw=22.5 * k, inner=0.45,
                     depth=0.008, cup=0.55 + 0.3 * k)
            F.sphere(f'lily_heart{x}', 0.014, (x, WATER_Y + 0.03, z), M.lily_heart, segments=8, rings=5)
        # reeds and bulrushes at the back left, rising out of the shallows by the stones
        for i in range(11):
            x, z = -0.6 + rng.uniform(-0.12, 0.12), -0.44 + rng.uniform(-0.1, 0.08)
            leaf(f'reed{i}', F.at(x, WATER_Y - 0.01, z), rng.uniform(0, 360), rng.uniform(70, 84), rng.uniform(0.45, 0.7),
                 0.035, M.reed, shape='strap', droop=rng.uniform(0.03, 0.12), thick=0.006, fold=0.3, rows=4, ring=3)
        for i in range(3):
            x, z = -0.62 + i * 0.07, -0.46 + (i % 2) * 0.06
            h = 0.6 + i * 0.06
            G.rod(f'rush_stem{i}', F.at(x, WATER_Y - 0.01, z), F.at(x, h + 0.08, z), 0.005, material=M.reed, verts=5)
            F.cyl(f'rush{i}', 0.016, 0.11, (x, h, z), M.rush, verts=8)
    G.join(still, 'pond', origin=(0, 0, 0))
    # the ripples: three faint rings round where the koi comes up, spreading and settling all the time
    with G.new_parts() as rings:
        for k, r in enumerate((0.1, 0.19, 0.28)):
            G.torus(f'ring{k}', r, 0.006, at=F.at(KOI[0], WATER_Y, KOI[1]), material=M.ripple, major_segments=28,
                    minor_segments=4).scale = (1, 0.9, 0.35)
    ripples = G.hinged(G.join(rings, 'ripples'), F.at(KOI[0], WATER_Y, KOI[1]))
    # the koi, under the surface at rest (the opaque water hides it); it rises until its back is out
    with G.new_parts() as fish:
        x, z = KOI
        y = 0.035
        F.sphere('koi_body', 0.5, (x, y, z), M.koi, scale=(0.26, 0.075, 0.05), segments=14, rings=8)
        F.sphere('koi_patch', 0.5, (x - 0.03, y + 0.004, z), M.koi_white, scale=(0.1, 0.07, 0.048), segments=10, rings=6)
        F.sphere('koi_head', 0.5, (x + 0.1, y + 0.002, z), M.koi_white, scale=(0.07, 0.06, 0.042), segments=10, rings=6)
        F.cone('koi_tail', 0.035, 0.002, 0.08, (x - 0.15, y, z), M.koi, rot=(0, 90, 0), verts=6,
               scale=(0.4, 1, 1))
        F.box('koi_fin', 0.08, 0.02, 0.006, (x - 0.01, y + 0.022, z), M.koi, rot=(0, 0, 0), bevel=0.003)
        for s in (-1, 1):
            F.box(f'koi_side_fin{s}', 0.03, 0.004, 0.035, (x + 0.05, y - 0.012, z + s * 0.045), M.koi, bevel=0.0015)
    koi = G.hinged(G.join(fish, 'koi'), F.at(x, y, z))
    koi.rotation_euler[2] = math.radians(25)        # heading a little to the front (a node turn, not baked)
    G.settle()
    G.squash(ripples, 'ripple', (1.3, 1.3, 1), seconds=3.0, back=True)
    G.slide(koi, 'fish', (0, 0, 0.042), seconds=1.2)
    G.swing(koi, 'fish', 30, seconds=1.2)
    # for watching: up and down again every so often (`every`), from rest to rest
    end = last_frame(2.4)
    _clip(koi, 'surface', 'location', 2, [(1, koi.location[2]), (end // 2, koi.location[2] + 0.04),
                                                     (end, koi.location[2])])
    return G.meshes()


def _pad(name, r, at, yaw, material, n=14):
    """A lily pad: a thin disc `r` across with a notch cut to its middle, lying at game `at`."""
    verts, faces = [], []
    gap = 0.5
    top, bottom = [], []
    for i in range(n + 1):
        a = gap / 2 + (math.tau - gap) * i / n
        top.append((r * math.cos(a), r * math.sin(a), 0.003))
        bottom.append((r * math.cos(a), r * math.sin(a), -0.003))
    verts = top + bottom + [(0, 0, 0.003), (0, 0, -0.003)]
    m = n + 1
    ct, cb = 2 * m, 2 * m + 1
    for i in range(n):
        faces += [(ct, i, i + 1), (cb, m + i + 1, m + i), (i, m + i, m + i + 1, i + 1)]
    faces += [(ct, cb, m, 0), (ct, n, m + n, cb)]
    obj = _mesh(name, verts, faces, material)
    obj.location = F.at(*at)
    obj.rotation_euler = (0, 0, math.radians(yaw))
    return obj


# --- the planter box (1x1): a plank trough on feet, full of leaves and flowers ---------------------------------
PLANTER_C = -0.09      # the depth it is centred on: the trough 10 cm back (as the old part's), leaves over its front


def planterBox():
    _own('planterBox')
    rng = random.Random(58)
    z0 = -0.1                          # the old part's trough stands 10 cm back on its tile
    w, d, h = 0.9, 0.42, 0.44
    for s in (-1, 1):
        for e in (-1, 1):
            F.box(f'leg{s}{e}', 0.06, h, 0.06, (s * (w / 2 - 0.03), h / 2, z0 + e * (d / 2 - 0.03)), M.box_trim,
                  bevel=low(0.008, 0), bevel_segments=1)
    for k in range(3):                 # three planks a side, a little gap between
        y = 0.07 + k * 0.125 + 0.06
        for e in (-1, 1):
            F.box(f'plank{k}{e}', w - 0.1, 0.115, 0.03, (0, y, z0 + e * (d / 2 - 0.035)), M.body, bevel=low(0.006, 0), bevel_segments=1)
        for s in (-1, 1):
            F.box(f'end{k}{s}', 0.03, 0.115, d - 0.1, (s * (w / 2 - 0.035), y, z0), M.body, bevel=low(0.006, 0), bevel_segments=1)
    F.box('floor', w - 0.08, 0.02, d - 0.08, (0, 0.08, z0), M.body)
    for e in (-1, 1):                  # the rim capping the planks
        F.box(f'rim{e}', w + 0.04, 0.03, 0.06, (0, h + 0.015, z0 + e * (d / 2 - 0.01)), M.box_trim, bevel=low(0.008, 0), bevel_segments=1)
    for s in (-1, 1):
        F.box(f'rim_end{s}', 0.06, 0.03, d - 0.08, (s * (w / 2 - 0.01), h + 0.015, z0), M.box_trim, bevel=low(0.008, 0), bevel_segments=1)
    F.box('soil', w - 0.08, 0.03, d - 0.08, (0, h - 0.025, z0), M.soil)
    # leafy clumps along the box, flowers over them, and a few leaves trailing over the front
    greens = [M.leaf, M.leaf_dark, M.leaf_light]
    clumps = []
    for i in range(12):                # two staggered rows of leafy clumps, overlapping into one mass
        row = i % 2
        x = -0.36 + i * 0.065 + rng.uniform(-0.015, 0.015)
        zz = z0 + (0.065 if row else -0.06) + rng.uniform(-0.02, 0.02)
        clumps.append((lump(f'clump{i}', 0.075 * rng.uniform(0.9, 1.15), F.at(x, h - 0.04, zz), greens[i % 3],
                            scale=(1.25, 1.15, 1.2), seed=i * 1.3, noise=0.32), Vector(F.at(x, 0, zz))))
    # flowers sitting on the clumps, leaning outwards with the leaves under them
    cols = [M.bloom, M.bloom2, M.bloom3, M.bloom4, M.bloom5]
    for i in range(22):
        clump, c = clumps[(i * 5) % 12]
        a, rr = rng.uniform(0, math.tau), rng.uniform(0.0, 0.06)
        at = c + Vector((rr * math.sin(a), rr * math.cos(a), 0))
        near = [v.co.z for v in clump.data.vertices if (v.co.xy - at.xy).length < 0.03]
        at.z = (max(near) if near else h + 0.15) - 0.006
        star(f'flower{i}', tuple(at), 0.032, 5, cols[i % 5], yaw=math.degrees(a), tilt=rr / 0.06 * 50, inner=0.6,
             cup=0.15)
        if not LOW[0]:
            G.sphere(f'eye{i}', 0.009, at=tuple(at + Vector((0, 0, 0.006))), material=M.eye, segments=6, rings=3)
    for i, x in enumerate((-0.3, -0.1, 0.12, 0.3)):
        for k in range(2):
            size = rng.uniform(0.12, 0.16)
            if LOW[0] and k:
                continue
            leaf(f'trail{i}_{k}', F.at(x + k * 0.04, h + 0.03, z0 + d / 2 - 0.035), 180 + (k * 2 - 1) * 20, 30, size, 0.05,
                 [M.leaf_light, M.leaf][k], shape='lance', droop=1.1, fold=0.2, rows=5, ring=3)
    box = G.join_all('planterBox')
    lo, hi = G.measure([box])        # centred on PLANTER_C deep, the far version too (its leaves are coarser)
    box.location.y += -PLANTER_C - (lo[1] + hi[1]) / 2
    return G.settle([box], centre=False)


# --- the birdhouse (1x1): a little cream house with a red roof on a post --------------------------------------
def birdhouse():
    _own('birdhouse')
    with G.new_parts():
        F.box('post', 0.07, 1.52, 0.07, (0, 0.76, 0), M.post, bevel=0.01)
        F.box('bracket', 0.2, 0.025, 0.2, (0, 1.505, 0), M.post, bevel=0.006)
        # the house: a gabled box, its ridge running front to back
        _prism('house', [(-0.12, 1.5), (0.12, 1.5), (0.12, 1.76), (0, 1.86), (-0.12, 1.76)], -0.11, 0.11, M.body)
        for s in (-1, 1):
            F.box(f'roof{s}', 0.2, 0.022, 0.29, (s * 0.075, 1.83, 0), M.roof, rot=(0, -s * 40, 0), bevel=0.005)
        F.box('ridge', 0.03, 0.03, 0.3, (0, 1.9, 0), M.roof, bevel=0.008)
        F.cyl('hole', 0.034, 0.008, (0, 1.68, 0.108), M.dark, rot=(90, 0, 0), verts=16)
        F.cyl('hole_ring', 0.042, 0.006, (0, 1.68, 0.11), M.post, rot=(90, 0, 0), verts=16)
        G.rod('perch', F.at(0, 1.6, 0.11), F.at(0, 1.6, 0.17), 0.007, material=M.post, verts=8)
        F.box('floor', 0.27, 0.02, 0.25, (0, 1.505 + 0.012, 0), M.body, bevel=0.004)
        for s in (-1, 1):              # corner trim
            F.box(f'trim{s}', 0.02, 0.26, 0.02, (s * 0.12, 1.63, 0.105), M.plate, bevel=0.004)
    G.join_all('birdhouse')
    G.settle()
    return G.meshes()


# --- the bird feeders (1x1): two seed tubes on a shepherd's hook; a robin on one, a blue tit lands for watching -
FEED_Z = -0.2      # the hook's plane (the old part's)


def _feeder(x):
    """A seed tube hanging from the hook's end at `x`: glass round the seed, a green cap and tray, a perch ring."""
    z = FEED_Z
    G.rod(_name('hanger'), F.at(x, 1.78, z), F.at(x, 1.705, z), 0.003, material=M.iron, verts=4)
    F.cone(_name('cap'), 0.075, 0.02, 0.06, (x, 1.68, z), M.feeder, verts=16)
    F.cyl(_name('cap_rim'), 0.072, 0.012, (x, 1.651, z), M.feeder, verts=16)
    F.cyl(_name('tube'), 0.055, 0.2, (x, 1.545, z), M.feeder_glass, verts=14)
    F.cyl(_name('tube_seed'), 0.048, 0.17, (x, 1.53, z), M.seed, verts=12)
    F.cyl(_name('tray'), 0.09, 0.014, (x, 1.44, z), M.feeder, verts=18)
    F.cyl(_name('base'), 0.06, 0.02, (x, 1.455, z), M.feeder, verts=14)
    G.torus(_name('perch'), 0.1, 0.005, at=F.at(x, 1.47, z), material=M.iron, major_segments=20, minor_segments=4)
    for a in (0, 90, 180, 270):
        G.rod(_name('spoke'), F.at(x + 0.06 * G.cosd(a), 1.452, z + 0.06 * G.sind(a)),
              F.at(x + 0.1 * G.cosd(a), 1.47, z + 0.1 * G.sind(a)), 0.003, material=M.iron, verts=4)


def birdFeederPole():
    _own('birdFeederPole')
    pole = M.body
    with G.new_parts() as still:
        z = FEED_Z
        G.rod('pole', F.at(0, 0, z), F.at(0, 1.88, z), 0.013, material=pole, verts=10)
        F.sphere('finial', 0.022, (0, 1.89, z), pole, segments=10, rings=6)
        F.box('foot_bar', 0.16, 0.012, 0.025, (0, 0.18, z), pole, bevel=0.004)      # the bar it's trodden in by
        for s in (-1, 1):
            # each arm curls out from the pole and down into a hook
            pts = [(0, 1.74), (s * 0.12, 1.82), (s * 0.24, 1.85), (s * 0.3, 1.83), (s * 0.32, 1.78)]
            for a, b in zip(pts, pts[1:]):
                G.rod(_name('arm'), F.at(a[0], a[1], z), F.at(b[0], b[1], z), 0.009, material=pole, verts=8)
                F.sphere(_name('arm_joint'), 0.009, (b[0], b[1], z), pole, segments=8, rings=5)
            _feeder(s * 0.32)
        _bird(0.32, 1.47, FEED_Z + 0.1, M.robin, M.robin_breast, turn=-20)            # the robin, always there
    G.join(still, 'feeders', origin=(0, 0, 0))
    # the blue tit that comes to the other feeder's perch, built full size and scaled to nothing
    with G.new_parts() as bird:
        _bird(-0.32, 1.47, FEED_Z + 0.1, M.tit, M.tit_breast, turn=25)
    bird = G.join(bird, 'bird', origin=F.at(-0.32, 1.47, FEED_Z + 0.1))
    bird.scale = (0.001, 0.001, 0.001)
    G.settle(centre=False)
    G.squash(bird, 'bird', 1000, seconds=0.5)
    return G.meshes()


# --- the dog house (1x1): a red kennel with a shingled roof, a name over the arched door and a water bowl -----
def _letter(ch, x, y, z, size, material):
    """A letter of the dog's name in strokes, `size` tall, its middle at game (x, y), on the face at depth z. Game x
    runs to the viewer's left seen from the front (Blender's front view), so the strokes and the letters' order are
    mirrored to read right."""
    s, t = size, 0.008
    strokes = {
        'R': [((-0.5, -0.5), (-0.5, 0.5)), ((-0.5, 0.5), (0.3, 0.5)), ((0.3, 0.5), (0.4, 0.25)), ((0.4, 0.25), (0.3, 0)),
              ((0.3, 0), (-0.5, 0)), ((-0.1, 0), (0.4, -0.5))],
        'E': [((-0.4, -0.5), (-0.4, 0.5)), ((-0.4, 0.5), (0.4, 0.5)), ((-0.4, 0), (0.25, 0)), ((-0.4, -0.5), (0.4, -0.5))],
        'X': [((-0.45, -0.5), (0.45, 0.5)), ((-0.45, 0.5), (0.45, -0.5))],
    }[ch]
    for (ax, ay), (bx, by) in strokes:
        a, b = Vector(F.at(x - ax * s * 0.8, y + ay * s, z)), Vector(F.at(x - bx * s * 0.8, y + by * s, z))
        G.rod(_name('stroke'), a, b, t / 2, material=material, verts=4)


def dogHouse():
    _own('dogHouse')
    zc, d = -0.03, 0.84                # the old part's body: 0.8 wide, centred 3-4 cm back
    front = zc + d / 2
    with G.new_parts():
        F.box('base', 0.84, 0.04, 0.9, (0, 0.02, zc), M.plinth, bevel=0.008)
        _prism('house', [(-0.4, 0.04), (0.4, 0.04), (0.4, 0.62), (0, 0.9), (-0.4, 0.62)], zc - d / 2, front, M.body)
        for s in (-1, 1):              # corner boards
            for e in (-1, 1):
                F.box(f'corner{s}{e}', 0.04, 0.58, 0.04, (s * 0.395, 0.33, zc + e * (d / 2 - 0.005)), M.plate, bevel=0.006)
        # the roof: two slopes of four overlapping shingle rows, and a ridge board
        slope = math.degrees(math.atan2(0.28, 0.4))
        for s in (-1, 1):
            for k in range(4):
                t = (k + 0.5) / 4
                x = s * (0.02 + t * 0.46)
                y = 0.935 - t * 0.46 * math.tan(math.radians(slope)) - 0.012 * k
                F.box(f'shingle{s}{k}', 0.15, 0.035, 0.92, (x, y, zc), M.roof, rot=(0, -s * slope, 0), bevel=0.006)
        F.box('ridge', 0.08, 0.05, 0.94, (0, 0.95, zc), M.roof, bevel=0.012)
        # the arched door and its frame
        F.box('door', 0.3, 0.26, 0.02, (0, 0.04 + 0.13, front), M.dark)
        F.cyl('door_arch', 0.15, 0.02, (0, 0.3, front), M.dark, rot=(90, 0, 0), verts=20)
        F.box('sill', 0.36, 0.025, 0.04, (0, 0.05, front + 0.01), M.plate, bevel=0.006)
        # the name plate in the gable
        F.box('plate', 0.26, 0.075, 0.015, (0, 0.6, front + 0.005), M.plate, bevel=0.006)
        for i, ch in enumerate('REX'):
            _letter(ch, (1 - i) * 0.06, 0.6, front + 0.015, 0.042, M.dark)
        # the water bowl by the door
        G.revolve('bowl', [(0, 0), (0.055, 0), (0.065, 0.04), (0.058, 0.045), (0.05, 0.012), (0, 0.012)],
                  at=F.at(0.29, 0, 0.43), material=M.bowl, segments=18)
        F.cyl('bowl_water', 0.052, 0.004, (0.29, 0.034, 0.43), M.bowl_water, verts=18)
    G.join_all('dogHouse')
    G.settle(centre=False)
    return G.meshes()


# --- the sundial (1x1): a copper dial with its gnomon on a stone baluster ---------------------------------------
def sundial():
    _own('sundial')
    with G.new_parts():
        F.box('step', 0.36, 0.06, 0.36, (0, 0.03, 0), M.body, bevel=0.01)
        G.revolve('pedestal', [(0, 0.06), (0.15, 0.06), (0.15, 0.1), (0.13, 0.13), (0.1, 0.16), (0.085, 0.3),
                               (0.075, 0.5), (0.085, 0.66), (0.1, 0.7), (0.12, 0.73), (0.16, 0.78), (0.17, 0.81),
                               (0.17, 0.85), (0, 0.85)], at=F.at(0, 0, 0), material=M.body, segments=24)
        F.cyl('dial', 0.15, 0.012, (0, 0.856, 0), M.dial, verts=28)
        G.torus('dial_rim', 0.148, 0.006, at=F.at(0, 0.862, 0), material=M.dial, major_segments=28, minor_segments=5)
        for a in range(0, 360, 30):    # the hour marks, longer at the quarters
            r = 0.11 if a % 90 else 0.105
            F.box(f'mark{a}', 0.008, 0.003, 0.03 if a % 90 else 0.04, (r * G.sind(a), 0.8635, r * G.cosd(a)),
                  M.dial_mark, rot=(0, 0, -a))
        F.cyl('boss', 0.018, 0.006, (0, 0.865, 0), M.dial_mark, verts=12)
        # the gnomon: a triangle along the noon line, its edge at the latitude's angle
        verts = []
        for sx in (-0.004, 0.004):
            verts += [F.at(sx, 0.862, 0.11), F.at(sx, 0.862, -0.1), F.at(sx, 0.862 + 0.1, -0.1)]
        faces = [(0, 1, 2), (5, 4, 3), (0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0)]
        g = _mesh('gnomon', verts, faces, M.dial)
        bpy.context.view_layer.objects.active = g
        bpy.ops.object.select_all(action='DESELECT')
        g.select_set(True)
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.mesh.normals_make_consistent(inside=False)
        bpy.ops.object.mode_set(mode='OBJECT')
    G.join_all('sundial')
    G.settle()
    return G.meshes()


# --- the stepping stones (1x3, rug layer): four flat stones set in the lawn -----------------------------------
STONES = [(0.05, -1.05, 0.52, 10), (-0.08, -0.35, 0.48, 40), (0.06, 0.35, 0.54, 70), (-0.04, 1.05, 0.5, 15)]
STONE_H = 0.04


def steppingStones():
    _own('steppingStones')
    with G.new_parts():
        for i, (x, z, d, r) in enumerate(STONES):
            s = G.boulder(f'stone{i}', d / 2, at=F.at(x, 0, z), material=M.body, rot=(0, 0, r), seed=i * 3.1 + 0.4,
                          noise=0.24, subdivisions=2)
            # flatten it into a slab: a level top with rounded edges, its bottom on the ground
            bpy.context.view_layer.update()
            top = max(v.co.z for v in s.data.vertices)
            for v in s.data.vertices:
                k = v.co.z / top
                v.co.z = STONE_H * min(1.0, k * 2.2) ** 0.6
                v.co.y *= 0.86
            s.data.update()
    G.join_all('steppingStones')
    G.settle()
    return G.meshes()


# --- the garden lamp post (1x1): a lantern of frosted panes on an iron post; the panes glow at night -----------
LAMP_Y = 2.3       # the content's `light.height`: the panes' middle


def lampPost():
    _own('lampPost')
    post, frame = M.body, M.lantern
    sq = 2 ** 0.5
    with G.new_parts():
        G.revolve('base', [(0, 0), (0.15, 0), (0.15, 0.03), (0.125, 0.05), (0.1, 0.12), (0.07, 0.16), (0.05, 0.3),
                           (0.045, 0.34), (0, 0.34)], at=F.at(0, 0, 0), material=post, segments=low(16, 8))
        F.cyl('ring', 0.055, 0.03, (0, 0.33, 0), post, verts=12)
        F.cyl('pole', 0.035, 1.72, (0, 0.34 + 0.86, 0), post, verts=12)
        F.cyl('collar', 0.05, 0.04, (0, 2.06, 0), post, verts=12)
        F.cone('cup', 0.045, 0.1, 0.07, (0, 2.115, 0), post, verts=12)
        for k in range(4):             # brackets from the post to the lantern's floor corners
            a = 45 + 90 * k
            G.rod(f'bracket{k}', F.at(0.03 * G.cosd(a), 2.03, 0.03 * G.sind(a)),
                  F.at(0.1 * G.cosd(a), 2.14, 0.1 * G.sind(a)), 0.007, material=post, verts=5)
        # the lantern: four frosted panes, wider at the top, in a frame under a pyramid roof
        F.cone('panes', 0.09 * sq, 0.125 * sq, 0.32, (0, LAMP_Y, 0), M.pane, verts=4, rot=(0, 0, 45), smooth=False)
        F.cone('lantern_floor', 0.11 * sq, 0.11 * sq, 0.025, (0, LAMP_Y - 0.165, 0), frame, verts=4, rot=(0, 0, 45),
               smooth=False)
        F.cone('lantern_top', 0.135 * sq, 0.135 * sq, 0.03, (0, LAMP_Y + 0.172, 0), frame, verts=4, rot=(0, 0, 45),
               smooth=False)
        for k in range(4):
            a = 45 + 90 * k
            lo, hi = 0.094 * sq, 0.129 * sq
            G.rod(f'frame{k}', F.at(lo * G.cosd(a), LAMP_Y - 0.16, lo * G.sind(a)),
                  F.at(hi * G.cosd(a), LAMP_Y + 0.16, hi * G.sind(a)), 0.009, material=frame, verts=5)
        F.cone('roof', 0.16 * sq, 0.025, 0.15, (0, LAMP_Y + 0.267, 0), frame, verts=4, rot=(0, 0, 45), smooth=False)
        F.cyl('vent', 0.035, 0.04, (0, LAMP_Y + 0.365, 0), frame, verts=10)
        F.sphere('finial', 0.03, (0, LAMP_Y + 0.405, 0), frame, segments=10, rings=6)
        _near_light('lampPost', LAMP_Y)
    G.join_all('lampPost')
    G.settle()
    return G.meshes()


# --- the string lights (2x1): festoon bulbs on a sagging wire between two posts in weighted pots ---------------
POST_X = 0.86
WIRE_Y, SAG = 2.27, 0.28     # the wire's height at the posts, and how far it sags in the middle
BULBS = 9


def _wire_y(x):
    return WIRE_Y - SAG * (1 - (x / (POST_X - 0.04)) ** 2)


def stringLights():
    _own('stringLights')
    with G.new_parts() as still:
        for s in (-1, 1):
            x = s * POST_X
            G.revolve(f'pot{s}', [(0, 0), (0.1, 0), (0.12, 0.27), (0.125, 0.3), (0.105, 0.3), (0.1, 0.28), (0, 0.28)],
                      at=F.at(x, 0, 0), material=M.planter, segments=18)
            F.cyl(f'pot_soil{s}', 0.1, 0.01, (x, 0.275, 0), M.soil, verts=14)
            F.box(f'post{s}', 0.07, 2.12, 0.07, (x, 0.2 + 1.06, 0), M.body, bevel=0.008)
            F.cone(f'cap{s}', 0.055, 0.01, 0.05, (x, 2.345, 0), M.body, verts=4, rot=(0, 0, 45), smooth=False)
            G.torus(f'eye{s}', 0.018, 0.004, at=F.at(x - s * 0.045, WIRE_Y, 0), material=M.wire, rot=(90, 0, 0),
                    major_segments=10, minor_segments=4)
    G.join(still, 'posts', origin=(0, 0, 0))
    with G.new_parts() as string:
        x0 = POST_X - 0.04
        n = 16
        pts = [(-x0 + 2 * x0 * i / n, _wire_y(-x0 + 2 * x0 * i / n)) for i in range(n + 1)]
        for (ax, ay), (bx, by) in zip(pts, pts[1:]):
            G.rod(_name('wire'), F.at(ax, ay, 0), F.at(bx, by, 0), 0.005, material=M.wire, verts=5)
        heights = []
        for i in range(BULBS):
            x = -x0 + 2 * x0 * (i + 0.5) / BULBS
            y = _wire_y(x)
            F.cyl(_name('socket'), 0.012, 0.035, (x, y - 0.02, 0), M.socket, verts=8)
            F.sphere(_name('bulb'), 0.035, (x, y - 0.07, 0), M.bulb, scale=(1, 1, 1.3), segments=10, rings=7)
            heights.append(y - 0.07)
        _near_light('stringLights', sum(heights) / len(heights))
    string = G.hinged(G.join(string, 'string'), F.at(0, WIRE_Y, 0))
    G.settle(centre=False)
    G.rock(string, 'sway', 4, seconds=3.2, axis='X')
    return G.meshes()


FAMILY = G.Family(
    'garden_things', PALETTES,
    [G.Model('pond', pond, size=(1.986, 0.8, 1.823), key='model.garden.pond',
             motions={'idle': 'ripple', 'uses': {'feedFish': 'fish', 'fish': {'clip': 'surface', 'delay': 1.5,
                                                                                'every': 7}}},
             paint=[('body', 'Stones')]),
     G.Model('planterBox', planterBox, size=(0.94, 0.623, 0.504), centre=(0, PLANTER_C), key='model.planterBox',
             far=far_of(planterBox), paint=[('body', 'Box')]),
     G.Model('birdhouse', birdhouse, size=(0.313, 1.915, 0.32), key='model.birdhouse',
             paint=[('body', 'House'), ('roof', 'Roof')]),
     G.Model('birdFeederPole', birdFeederPole, size=(0.85, 1.912, 0.271), centre=(0, -0.1694), key='model.birdFeederPole',
             motions={'uses': {'birdwatch': 'bird'}},
             paint=[('body', 'Pole'), ('feeder', 'Feeders')]),
     G.Model('dogHouse', dogHouse, size=(0.983, 0.975, 0.994), centre=(0, -0.003), key='model.dogHouse',
             paint=[('body', 'House'), ('roof', 'Roof')]),
     G.Model('sundial', sundial, size=(0.36, 0.962, 0.36), key='model.sundial',
             paint=[('body', 'Plinth'), ('dial', 'Dial')]),
     G.Model('steppingStones', steppingStones, size=(0.592, 0.04, 2.537), key='model.steppingStones',
             paint=[('body', 'Stones')]),
     G.Model('lampPost', lampPost, size=(0.32, 2.735, 0.32), key='model.lampPost', far=0.25,
             paint=[('body', 'Post'), ('lantern', 'Lantern')]),
     G.Model('stringLights', stringLights, size=(1.97, 2.37, 0.246), key='model.stringLights',
             motions={'idle': 'sway'}, paint=[('body', 'Posts'), ('bulb', 'Bulbs')])],
    pack='garden_things',
    comment='Blender-built garden things and lights (tools/art/blender/families/garden_things.py): the '
            'garden pond, the planter box, the birdhouse, the bird feeders, the dog house, the sundial, '
            'the stepping stones, the garden lamp post and the string lights. Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. The pond\'s ripples spread near the camera (`ripple`, ambient) and a koi '
            'comes up for `feedFish` (`fish`) and now and then while watched (`surface`); a blue tit '
            'lands on the feeders for `birdwatch` (`bird`); the lamp\'s panes and the bulbs glow at night, '
            'and the string sways near the camera (`sway`, ambient). The planter box and the lamp post '
            'have far versions.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
