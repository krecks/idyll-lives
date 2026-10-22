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
from mathutils import Vector, noise

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import LOW, dome, far_of, leaf, low, lump, star  # noqa: E402
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
    mud=('#4A4234', 'matte'),            # the wet margin under the coping stones
    frog=('#6E9A3A', 'satin'),
    iris=('#7A5CC4', 'satin'),           # the flag irises at the front
    # the planter box
    box_trim=('#7A5A40', 'wood'),
    soil=('#4A3A2C', 'matte'),
    bracket=('#3A3D41', 'metal'),        # the planter box's steel corner brackets and screws
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
    lichen=('#C8C29A', 'matte'),
    moss=('#5E7A3A', 'matte'),           # moss at the stepping stones' edges (plain: no foliage draw)         # pale patches on the sundial's steps
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


def _flag(name, r, at, yaw, h, seed, material, sx=1.0, sz=1.0, n=8):
    """A flat stone: an irregular rounded outline `r` across its radius (stretched sx by sz), `h` thick, its
    edges chamfered and its faces a little uneven, its middle at game `at`, turned `yaw` degrees."""
    rng = random.Random(seed)
    pts = []
    for k in range(n):
        a = math.tau * (k + rng.uniform(-0.18, 0.18)) / n
        rr = r * rng.uniform(0.82, 1.08)
        pts.append((rr * sx * math.cos(a), rr * sz * math.sin(a)))
    c, s = G.cosd(yaw), G.sind(yaw)
    pts = [(x * c - z * s, x * s + z * c) for x, z in G.smooth_outline(pts, passes=1)]
    # a slab lofted through its outline: a little in at the foot, full at the waist, chamfered in to the top
    cx, cy, cz = at
    rows = [(0.9, -h / 2), (1.0, -h / 2 + h * 0.5), (0.86, h / 2)]
    stone = G.loft(name, [[F.at(cx + x * k, cy + y, cz + z * k) for x, z in pts] for k, y in rows], material=material)
    return G.jitter(stone, 0.007, size=0.12, seed=seed, up=0.5)


def _koi(x, y, z):
    """The koi, nose to game +x (its node turns it): a lofted white body with an orange saddle and crown, a fanned
    tail, a dorsal fin and two pectoral fins."""
    # (along, half width, half height): nose to the tail's root
    rows = [(0.13, 0.004, 0.004), (0.115, 0.018, 0.016), (0.09, 0.026, 0.026), (0.05, 0.03, 0.034), (0.0, 0.029, 0.033),
            (-0.05, 0.022, 0.025), (-0.09, 0.012, 0.014), (-0.115, 0.006, 0.008)]
    n = 10
    G.loft('koi_body', [[F.at(x + u, y + hh * math.sin(math.tau * k / n), z + hw * math.cos(math.tau * k / n))
                         for k in range(n)] for u, hw, hh in rows], material=M.koi_white)
    # the orange saddle over the back, a hair proud of the white body, and the crown on the head
    for name, part in (('koi_patch', rows[3:7]), ('koi_crown', rows[1:3])):
        G.loft(name, [[F.at(x + u, y + 0.001 + hh * 1.05 * math.sin(math.pi * (k + 0.5) / 7),
                            z + hw * 1.05 * math.cos(math.pi * (k + 0.5) / 7)) for k in range(7)] for u, hw, hh in part],
               material=M.koi)
    F.sweep('koi_tail', [(x - 0.11, y, z), (x - 0.165, y + 0.004, z)], None, M.koi,
            profile=[(-0.0045, -0.0015), (0.0045, -0.0015), (0.0045, 0.0015), (-0.0045, 0.0015)], up=(0, 1, 0),
            scale=[1.0, 7.0])
    F.extrude('koi_fin', [(0.05, 0.0), (-0.03, 0.0), (-0.05, 0.007), (0.0, 0.011)], 0.003, (x, y + 0.024, z), M.koi,
              plane='front')
    for side in (-1, 1):
        F.sweep(f'koi_side_fin{side}', [(x + 0.07, y - 0.012, z + side * 0.026), (x + 0.045, y - 0.02, z + side * 0.058)],
                None, M.koi_white, profile=[(-0.008, -0.001), (0.008, -0.001), (0.008, 0.001), (-0.008, 0.001)],
                up=(0, 1, 0), scale=[0.5, 1.4])


def _frog(x, y, z, turn):
    """A frog squatting on a lily pad at game (x, y, z), looking `turn` degrees round from the front."""
    c, s = G.cosd(turn), G.sind(turn)

    def at(u, v, w):
        return (x + v * s + u * c, y + w, z + v * c - u * s)
    rows = [(-0.03, 0.012, 0.008, 0.012), (-0.018, 0.022, 0.016, 0.018), (0.0, 0.025, 0.019, 0.02),
            (0.018, 0.021, 0.013, 0.019), (0.03, 0.011, 0.006, 0.016)]
    n = 8
    G.loft('frog', [[F.at(*at(hw * math.cos(math.tau * k / n), v, wc + hh * math.sin(math.tau * k / n))) for k in range(n)]
                    for v, hw, hh, wc in rows], material=M.frog)
    for k in (-1, 1):
        F.sphere(f'frog_eye{k}', 0.0065, at(k * 0.012, 0.016, 0.031), M.frog, segments=6, rings=4)
        F.sphere(f'frog_pupil{k}', 0.0035, at(k * 0.0148, 0.0185, 0.0335), M.dark, segments=6, rings=4)
        # the haunches folded at its sides, the front legs straight down
        F.sweep(f'frog_leg{k}', [at(k * 0.02, -0.02, 0.012), at(k * 0.03, -0.002, 0.007), at(k * 0.02, 0.014, 0.003)],
                0.006, M.frog, verts=6, scale=[1.2, 1.0, 0.6])
        F.sweep(f'frog_arm{k}', [at(k * 0.013, 0.014, 0.012), at(k * 0.016, 0.022, 0.001)], 0.003, M.frog, verts=5)


def pond():
    _own('pond')
    rng = random.Random(101)
    with G.new_parts() as still:
        rx, rz = POND
        # the water: an opaque gloss block to the surface, its sides hidden under the stones; deeper in the middle
        F.cyl('water', 1.0, WATER_Y, (0, WATER_Y / 2, 0), M.water, verts=32, scale=(rx - 0.04, rz - 0.04, 1))
        F.cyl('deep', 1.0, 0.002, (0.05, WATER_Y + 0.001, 0.03), M.water_deep, verts=24, scale=(0.5, 0.42, 1))
        # the margin: a band of dark wet mud at the water's edge, under the coping stones' lips
        G.torus('margin', 1.0, 0.03, at=F.at(0, WATER_Y - 0.012, 0), material=M.mud, major_segments=32,
                minor_segments=4).scale = (rx - 0.12, rz - 0.12, 0.6)
        # the rim: big flat coping stones lapping over the edge, smaller ones set behind in their gaps, pebbles
        n = 15
        for i in range(n):
            a = i / n * math.tau + rng.uniform(-0.05, 0.05)
            r = rng.uniform(0.17, 0.2)
            x, z = (rx - 0.03) * math.cos(a), (rz - 0.03) * math.sin(a)
            h = rng.uniform(0.07, 0.09)
            _flag(f'stone{i}', r, (x, WATER_Y + h / 2 - 0.025, z), -math.degrees(a), h, i * 1.37 + 3, M.body,
                  sx=0.72, sz=1.15, n=6)
        for i in range(n):
            a = (i + 0.5) / n * math.tau + rng.uniform(-0.08, 0.08)
            r = rng.uniform(0.1, 0.13)
            x, z = (rx + 0.06) * math.cos(a), (rz + 0.08) * math.sin(a)
            h = rng.uniform(0.04, 0.055)
            _flag(f'back_stone{i}', r, (x, h / 2, z), -math.degrees(a), h, i * 2.11 + 40, M.body, sx=0.8, sz=1.1, n=5)
        for i in range(12):
            a = (i + 0.3) / 12 * math.tau + rng.uniform(-0.1, 0.1)
            x, z = (rx + 0.1) * math.cos(a), (rz + 0.1) * math.sin(a)
            G.boulder(f'pebble{i}', rng.uniform(0.035, 0.05), at=F.at(x, 0, z), material=M.pebble, scale=(1.2, 1, 0.6),
                      seed=i * 2.9, noise=0.3, subdivisions=1)
        # lily pads, each with a notch, two flowers, a bud and a frog on a pad of its own
        for i, (x, z, r) in enumerate([(-0.3, 0.2, 0.11), (0.28, -0.24, 0.09), (0.42, 0.3, 0.08), (-0.16, -0.36, 0.085),
                                        (-0.02, 0.44, 0.07), (-0.44, -0.08, 0.075), (0.0, 0.3, 0.09)]):
            _pad(f'pad{i}', r, (x, WATER_Y + 0.003, z), rng.uniform(0, 360), M.pad)
        for (x, z), petal in (((-0.27, 0.2), M.lily), ((0.26, -0.22), M.lily_white)):
            for k, rr in enumerate((0.058, 0.042, 0.027)):
                star(f'lily{x}_{k}', F.at(x, WATER_Y + 0.01 + 0.011 * k, z), rr, 8, petal, yaw=22.5 * k, inner=0.4,
                     depth=0.006, cup=0.5 + 0.35 * k)
            F.sphere(f'lily_heart{x}', 0.011, (x, WATER_Y + 0.037, z), M.lily_heart, segments=6, rings=4)
        G.revolve('lily_bud', [(0, 0), (0.016, 0.014), (0.015, 0.036), (0.006, 0.056), (0, 0.06)],
                  at=F.at(0.4, WATER_Y - 0.004, 0.26), material=M.lily, segments=8)
        _frog(0.0, WATER_Y + 0.006, 0.3, 25)
        # reeds and bulrushes at the back left, rising out of the shallows by the stones
        for i in range(11):
            x, z = -0.6 + rng.uniform(-0.12, 0.12), -0.44 + rng.uniform(-0.1, 0.08)
            leaf(f'reed{i}', F.at(x, WATER_Y - 0.01, z), rng.uniform(0, 360), rng.uniform(70, 84), rng.uniform(0.45, 0.7),
                 0.035, M.reed, shape='strap', droop=rng.uniform(0.03, 0.12), thick=0.006, fold=0.3, rows=4, ring=3)
        for i in range(3):
            x, z = -0.62 + i * 0.07, -0.46 + (i % 2) * 0.06
            h = 0.6 + i * 0.06
            F.sweep(f'rush_stem{i}', [(x, WATER_Y - 0.01, z), (x + 0.008, h * 0.6, z), (x + 0.012, h + 0.08, z)], 0.005,
                    M.reed, verts=5, resolution=2, scale=[1.0, 0.9, 0.5])
            bx, by, _ = F.at(x + 0.011, 0, z)
            G.loft(f'rush{i}', [G.ring(r, r, h + dy, 8, at=(bx, by)) for dy, r in
                                ((-0.058, 0.005), (-0.05, 0.015), (0.0, 0.018), (0.05, 0.015), (0.058, 0.005))],
                   material=M.rush)
        # flag irises at the front right: sword leaves in a fan and two flowers
        for i in range(7):
            yaw = 150 + i * 14 + rng.uniform(-6, 6)
            leaf(f'iris{i}', F.at(0.64 + rng.uniform(-0.03, 0.03), WATER_Y - 0.01, 0.36 + rng.uniform(-0.03, 0.03)), yaw,
                 rng.uniform(72, 84), rng.uniform(0.3, 0.4), 0.028, M.reed, shape='strap', droop=0.04, thick=0.005,
                 fold=0.15, rows=3, ring=3)
        for i, (x, z, h) in enumerate(((0.62, 0.36, 0.4), (0.68, 0.33, 0.34))):
            F.sweep(f'iris_stem{i}', [(x, WATER_Y, z), (x, h, z)], 0.004, M.reed, verts=4)
            for k in range(3):
                a = 120 * k + 30 * i
                F.sweep(f'iris_fall{i}{k}', [(x, h, z), (x + 0.028 * G.sind(a), h + 0.008, z + 0.028 * G.cosd(a)),
                                             (x + 0.042 * G.sind(a), h - 0.022, z + 0.042 * G.cosd(a))], None, M.iris,
                        profile=[(-0.009, -0.0015), (0.009, -0.0015), (0.009, 0.0015), (-0.009, 0.0015)], up=(0, 1, 0),
                        scale=[0.4, 1.2, 0.5])
                b = a + 60
                F.sweep(f'iris_std{i}{k}', [(x, h, z), (x + 0.012 * G.sind(b), h + 0.04, z + 0.012 * G.cosd(b))],
                        None, M.iris, profile=[(-0.006, -0.0015), (0.006, -0.0015), (0.006, 0.0015), (-0.006, 0.0015)],
                        up=(0, 1, 0), scale=[0.5, 1.0])
    G.join(still, 'pond', origin=(0, 0, 0))
    # the ripples: three faint rings round where the koi comes up, spreading and settling all the time
    with G.new_parts() as rings:
        for k, r in enumerate((0.1, 0.19, 0.28)):
            G.torus(f'ring{k}', r, 0.006, at=F.at(KOI[0], WATER_Y, KOI[1]), material=M.ripple, major_segments=22,
                    minor_segments=4).scale = (1, 0.9, 0.35)
    ripples = G.hinged(G.join(rings, 'ripples'), F.at(KOI[0], WATER_Y, KOI[1]))
    # the koi, under the surface at rest (the opaque water hides it); it rises until its back is out
    with G.new_parts() as fish:
        x, z = KOI
        y = 0.035
        _koi(x, y, z)
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


def _pad(name, r, at, yaw, material, n=11):
    """A lily pad: a thin disc `r` across with a notch cut to its middle, its rim turned up a little, lying at game
    `at`."""
    verts, faces = [], []
    gap = 0.5
    top, bottom = [], []
    for i in range(n + 1):
        a = gap / 2 + (math.tau - gap) * i / n
        lift = 0.006 * (1 + 0.5 * math.sin(3 * a))
        top.append((r * math.cos(a), r * math.sin(a), 0.003 + lift))
        bottom.append((r * math.cos(a), r * math.sin(a), -0.001 + lift))
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
    full = not G.is_low()
    for s in (-1, 1):                  # corner posts, their feet chamfered off the ground
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
    # a batten down the middle of each long side, screwed to the planks
    for e in (-1, 1):
        F.box(f'batten{e}', 0.06, h - 0.1, 0.02, (0, 0.07 + (h - 0.1) / 2, z0 + e * (d / 2 - 0.01)), M.box_trim,
              bevel=low(0.005, 0), bevel_segments=1)
        if full:
            for k in range(3):
                F.cyl(f'screw{e}{k}', 0.006, 0.004, (0, 0.13 + k * 0.125, z0 + e * (d / 2 + 0.001)), M.bracket,
                      rot=(90, 0, 0), verts=6)
    # the cap rail: one moulded frame round the top, its corners mitred, its outer edge rounded over
    hw, hd = w / 2 + 0.02, d / 2 + 0.02
    rail = [(hw, z0 - hd), (hw, z0 + hd), (-hw, z0 + hd), (-hw, z0 - hd)]
    G.kit.moulding(F, 'rim', rail, [(0.0, 0.0), (0.0, low(0.02, 0.03)), (low(0.008, 0.0), 0.03), (0.07, 0.03), (0.07, 0.0)],
                   M.box_trim, at=h, plane='top')
    # steel corner brackets on the front, bolted through
    if full:
        for s in (-1, 1):
            x, zf = s * (w / 2 - 0.03), z0 + d / 2 + 0.002
            for y in (0.12, h - 0.06):
                F.box(f'bracket{s}{y}', 0.075, 0.06, 0.004, (x, y, zf), M.bracket)
                F.box(f'bracket_side{s}{y}', 0.004, 0.06, 0.05, (s * (w / 2 + 0.002), y, zf - 0.025), M.bracket)
                for b in (-1, 1):
                    F.cyl(f'bolt{s}{y}{b}', 0.006, 0.005, (x + b * 0.022, y, zf + 0.003), M.bracket, rot=(90, 0, 0), verts=6)
    soil = F.box('soil', w - 0.08, 0.03, d - 0.08, (0, h - 0.025, z0), M.soil)
    # leafy clumps along the box, flowers over them, and a few leaves trailing over the front
    greens = [M.leaf, M.leaf_dark, M.leaf_light]
    clumps = []
    for i in range(12):                # two staggered rows of leafy clumps, overlapping into one mass
        row = i % 2
        x = -0.36 + i * 0.065 + rng.uniform(-0.015, 0.015)
        zz = z0 + (0.065 if row else -0.06) + rng.uniform(-0.02, 0.02)
        clumps.append((lump(f'clump{i}', 0.075 * rng.uniform(0.9, 1.15), F.at(x, h - 0.04, zz), greens[i % 3],
                            scale=(1.25, 1.15, 1.2), seed=i * 1.3, noise=0.32), Vector(F.at(x, 0, zz))))
    # flowers sitting on the clumps, leaning outwards with the leaves under them, a yellow eye in each
    cols = [M.bloom, M.bloom2, M.bloom3, M.bloom4, M.bloom5]
    for i in range(22):
        clump, c = clumps[(i * 5) % 12]
        a, rr = rng.uniform(0, math.tau), rng.uniform(0.0, 0.06)
        at = c + Vector((rr * math.sin(a), rr * math.cos(a), 0))
        near = [v.co.z for v in clump.data.vertices if (v.co.xy - at.xy).length < 0.03]
        at.z = (max(near) if near else h + 0.15) - 0.006
        star(f'flower{i}', tuple(at), 0.032, 5, cols[i % 5], yaw=math.degrees(a), tilt=rr / 0.06 * 50, inner=0.6,
             cup=0.15)
        if full and i % 2 == 0:
            dome(f'eye{i}', tuple(at + Vector((0, 0, 0.003))), 0.008, 0.005, M.eye, yaw=math.degrees(a),
                 tilt=rr / 0.06 * 50)
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
        # the post, its edges chamfered, on a little collar of earth; a platform on two braces carries the house
        F.box('post', 0.07, 1.52, 0.07, (0, 0.76, 0), M.post, bevel=0.01, bevel_segments=1)
        G.revolve('mound', [(0, 0), (0.1, 0), (0.07, 0.025), (0.04, 0.035), (0, 0.035)], at=F.at(0, 0, 0),
                  material=M.soil, segments=10)
        F.box('bracket', 0.2, 0.025, 0.2, (0, 1.505, 0), M.post, bevel=0.006, bevel_segments=1)
        for e in (-1, 1):
            F.sweep(f'brace{e}', [(0, 1.36, e * 0.035), (0, 1.49, e * 0.09)], None, M.post,
                    profile=[(-0.012, -0.015), (0.012, -0.015), (0.012, 0.015), (-0.012, 0.015)], up=(1, 0, 0))
        # the house: a gabled box, its ridge running front to back, clad in lapped boards down its sides
        _prism('house', [(-0.12, 1.5), (0.12, 1.5), (0.12, 1.76), (0, 1.86), (-0.12, 1.76)], -0.11, 0.11, M.body)
        for s in (-1, 1):
            for k in range(5):
                y = 1.545 + k * 0.05
                F.box(f'board{s}{k}', 0.008, 0.05, 0.215, (s * 0.124, y, 0), M.body, rot=(0, -s * 4, 0))
        # the front: a gable board proud of the wall, the round hole through it into the dark, a ring round it
        gable = [(-0.125, 1.52), (0.125, 1.52), (0.125, 1.765), (0, 1.865), (-0.125, 1.765)]
        hole = [(0.033 * math.cos(math.tau * k / 14), 1.68 + 0.033 * math.sin(math.tau * k / 14)) for k in range(14)]
        F.extrude('front', gable, 0.012, (0, 0, 0.114), M.body, holes=[hole])
        F.cyl('hole', 0.035, 0.002, (0, 1.68, 0.111), M.dark, rot=(90, 0, 0), verts=14)
        G.torus('hole_ring', 0.04, 0.006, at=F.at(0, 1.68, 0.121), material=M.post, rot=(90, 0, 0), major_segments=14,
                minor_segments=4)
        # the perch: a dowel out under the hole, a robin on it looking along the front
        F.sweep('perch', [(0, 1.6, 0.11), (0, 1.6, 0.165)], 0.007, M.post, verts=8)
        G.kit.songbird(F, 'robin', 0.0, 1.607, 0.142, M.robin, M.robin_breast, M.beak, M.dark, turn=90, size=0.85)
        # the roof: two overhanging boards with a fascia under their eaves and a rounded ridge cap
        for s in (-1, 1):
            F.box(f'roof{s}', 0.2, 0.022, 0.29, (s * 0.075, 1.83, 0), M.roof, rot=(0, -s * 40, 0), bevel=0.005,
                  bevel_segments=1)
            for k in range(3):         # shingle courses: each a strip lapped over the one below
                t = 0.042 + k * 0.048
                F.box(f'shingle{s}{k}', 0.06, 0.012, 0.292, (s * (0.161 - t * G.cosd(40)), 1.777 + t * G.sind(40), 0),
                      M.roof, rot=(0, -s * 40, 0), bevel=0.002, bevel_segments=1)
        F.sweep('ridge', [(0, 1.9, -0.15), (0, 1.9, 0.15)], None, M.roof,
                profile=G.rounded_rect(0.03, 0.034, 0.012, corner=2), up=(0, 0, 1))
        F.box('floor', 0.27, 0.02, 0.25, (0, 1.505 + 0.012, 0), M.body, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):              # corner trim
            F.box(f'trim{s}', 0.02, 0.26, 0.02, (s * 0.12, 1.63, 0.105), M.plate, bevel=0.004, bevel_segments=1)
            F.box(f'trim_back{s}', 0.02, 0.26, 0.02, (s * 0.12, 1.63, -0.105), M.plate, bevel=0.004, bevel_segments=1)
    G.join_all('birdhouse')
    G.settle()
    return G.meshes()


# --- the bird feeders (1x1): two seed tubes on a shepherd's hook; a robin on one, a blue tit lands for watching -
FEED_Z = -0.2      # the hook's plane (the old part's)


def _feeder(x):
    """A seed tube hanging from the hook's end at `x`: a wire bail, a domed green lid, the glass tube full of seed
    with two feeding ports and their pegs, a base and a seed tray with a lip, a perch ring on four spokes."""
    z = FEED_Z
    bx, by, _ = F.at(x, 0, z)
    # the bail: a loop over the hook's tip, down to the lid's knob
    F.sweep(_name('hanger'), [(x - 0.012, 1.705, z), (x - 0.012, 1.765, z), (x, 1.782, z), (x + 0.012, 1.765, z),
                              (x + 0.012, 1.705, z)], 0.0025, M.iron, verts=4, resolution=2)
    G.revolve(_name('cap'), [(0.0, 1.645), (0.074, 1.645), (0.076, 1.655), (0.068, 1.668), (0.048, 1.69),
                             (0.022, 1.703), (0.012, 1.712), (0.0, 1.716)], at=(bx, by, 0), material=M.feeder,
              segments=14)
    F.cyl(_name('tube'), 0.055, 0.2, (x, 1.545, z), M.feeder_glass, verts=14)
    F.cyl(_name('tube_seed'), 0.048, 0.17, (x, 1.53, z), M.seed, verts=12)
    for y, a in ((1.6, 0), (1.5, 90)):           # feeding ports, each with a peg to stand on
        c, s = G.cosd(a), G.sind(a)
        for k in (-1, 1):
            F.sweep(_name('port'), [(x + k * 0.05 * c, y, z + k * 0.05 * s), (x + k * 0.062 * c, y, z + k * 0.062 * s)],
                    0.012, M.feeder, verts=8)
            F.sweep(_name('peg'), [(x + k * 0.05 * c, y - 0.022, z + k * 0.05 * s),
                                   (x + k * 0.1 * c, y - 0.022, z + k * 0.1 * s)], 0.0025, M.iron, verts=4)
    G.revolve(_name('tray'), [(0.0, 1.433), (0.09, 1.433), (0.094, 1.447), (0.088, 1.452), (0.083, 1.441),
                              (0.06, 1.441), (0.06, 1.465), (0.0, 1.465)], at=(bx, by, 0), material=M.feeder, segments=16)
    G.torus(_name('perch'), 0.1, 0.005, at=F.at(x, 1.47, z), material=M.iron, major_segments=20, minor_segments=4)
    for a in (0, 90, 180, 270):
        F.sweep(_name('spoke'), [(x + 0.06 * G.cosd(a), 1.452, z + 0.06 * G.sind(a)),
                                 (x + 0.1 * G.cosd(a), 1.47, z + 0.1 * G.sind(a))], 0.003, M.iron, verts=4)


def birdFeederPole():
    _own('birdFeederPole')
    pole = M.body
    with G.new_parts() as still:
        z = FEED_Z
        # the pole: a spike into the lawn, a step bar to tread it in, an acorn finial
        F.sweep('pole', [(0, 0, z), (0, 0.06, z), (0, 1.86, z)], 0.013, pole, verts=10, scale=[0.4, 1.0, 1.0])
        G.revolve('finial', [(0.0, 0.0), (0.016, 0.0), (0.016, 0.008), (0.022, 0.016), (0.02, 0.034), (0.011, 0.05),
                             (0.0, 0.058)], at=F.at(0, 1.855, z), material=pole, segments=10)
        F.sweep('foot_bar', G.kit.fillet([(-0.085, 0.14, z), (-0.085, 0.18, z), (0.085, 0.18, z), (0.085, 0.14, z)], 0.015),
                0.007, pole, verts=6)
        F.cyl('collar', 0.02, 0.03, (0, 1.735, z), pole, verts=10, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            # each arm in one piece: out from the pole's collar, curling over and down into a hook
            pts = [(0, 1.735, z), (s * 0.06, 1.79, z), (s * 0.15, 1.835, z), (s * 0.24, 1.85, z), (s * 0.3, 1.835, z),
                   (s * 0.322, 1.8, z), (s * 0.32, 1.775, z), (s * 0.305, 1.765, z)]
            F.sweep(f'arm{s}', pts, 0.009, pole, verts=6, resolution=3, scale=[1.2, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.8])
            # a scroll under it, stiffening the bend
            F.sweep(f'scroll{s}', [(0, 1.68, z), (s * 0.05, 1.7, z), (s * 0.1, 1.76, z), (s * 0.085, 1.79, z),
                                   (s * 0.07, 1.775, z)], 0.005, pole, verts=5, resolution=2)
            _feeder(s * 0.32)
        G.kit.songbird(F, 'robin', 0.32, 1.475, FEED_Z + 0.1, M.robin, M.robin_breast, M.beak, M.dark, turn=-20,
                       size=0.8)                                                      # the robin, always there
    G.join(still, 'feeders', origin=(0, 0, 0))
    # the blue tit that comes to the other feeder's perch, built full size and scaled to nothing
    with G.new_parts() as bird:
        G.kit.songbird(F, 'tit', -0.32, 1.475, FEED_Z + 0.1, M.tit, M.tit_breast, M.beak, M.dark, turn=25, size=0.75)
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
    slope = math.degrees(math.atan2(0.28, 0.4))
    cs, sn = G.cosd(slope), G.sind(slope)
    rng = random.Random(7)
    with G.new_parts():
        F.box('base', 0.84, 0.04, 0.9, (0, 0.02, zc), M.plinth, bevel=0.008, bevel_segments=1)
        _prism('house', [(-0.4, 0.04), (0.4, 0.04), (0.4, 0.62), (0, 0.9), (-0.4, 0.62)], zc - d / 2, front, M.body)
        # lapped boards down the sides and the back: each a strip proud of the wall, its lower edge out a little
        for k in range(7):
            y = 0.08 + k * 0.08
            for s in (-1, 1):
                F.box(f'side_board{s}{k}', 0.008, 0.082, d - 0.04, (s * 0.403, y, zc), M.body, rot=(0, s * 3, 0))
            F.box(f'back_board{k}', 0.76, 0.082, 0.008, (0, y, zc - d / 2 - 0.003), M.body, rot=(3, 0, 0))
        # the front: a gable board with the arched doorway through it, its boards' laps either side of the door
        gable = [(-0.4, 0.04), (0.4, 0.04), (0.4, 0.62), (0, 0.9), (-0.4, 0.62)]
        arch = [(0.15, 0.04), (0.15, 0.3)] + [(0.15 * G.cosd(a), 0.3 + 0.15 * G.sind(a)) for a in range(15, 180, 15)] + \
            [(-0.15, 0.3), (-0.15, 0.04)]
        hole = [(0.15, 0.045)] + arch[1:-1] + [(-0.15, 0.045)]
        F.extrude('front', gable, 0.012, (0, 0, front + 0.006), M.body, holes=[hole])
        for k in range(1, 9):
            y = 0.04 + k * 0.08
            half = 0.4 if y <= 0.62 else 0.4 * (0.9 - y) / 0.28
            if half < 0.05:
                continue
            spans = [(-half, half)] if y > 0.46 else [(-half, -0.17), (0.17, half)]
            if 0.3 < y <= 0.46:
                gap = (0.15 ** 2 - (y - 0.3) ** 2) ** 0.5 + 0.02
                spans = [(-half, -gap), (gap, half)]
            for i, (a, b) in enumerate(spans):
                F.box(f'lap{k}{i}', b - a, 0.006, 0.004, ((a + b) / 2, y, front + 0.013), M.body)
        # the dark inside, seen through the doorway, and a trim round the arch
        F.box('door', 0.3, 0.42, 0.01, (0, 0.25, front - 0.002), M.dark)
        F.sweep('door_trim', [(x, y, front + 0.016) for x, y in arch], None, M.plate,
                profile=[(-0.006, -0.018), (0.006, -0.018), (0.006, 0.018), (-0.006, 0.018)], up=(0, -1, 0))
        F.box('sill', 0.36, 0.025, 0.05, (0, 0.05, front + 0.015), M.plate, bevel=0.006, bevel_segments=1)
        for s in (-1, 1):              # corner boards
            for e in (-1, 1):
                F.box(f'corner{s}{e}', 0.04, 0.58, 0.04, (s * 0.395, 0.33, zc + e * (d / 2 - 0.005)), M.plate, bevel=0.006,
                      bevel_segments=1)
            # the bargeboards up the front gable's edges
            F.sweep(f'barge{s}', [(s * 0.47, 0.885 - 0.47 * 0.7, front + 0.035), (0, 0.885, front + 0.035)], None, M.plate,
                    profile=[(-0.01, -0.03), (0.01, -0.03), (0.01, 0.03), (-0.01, 0.03)], up=(0, -1, 0))
        # the roof: an underlay board a slope, four courses of shingles lapped over it, staggered, and a ridge cap
        T = 0.47 / cs                  # down each slope, from the ridge to the eave
        for s in (-1, 1):
            def on(t, out):
                """The game (x, height) `t` down the slope from the ridge, `out` off the walls' line."""
                return (s * (t * cs + out * sn), 0.9 - t * sn + out * cs)
            x, y = on(T / 2, 0.015)
            F.box(f'underlay{s}', T, 0.03, 0.94, (x, y, zc), M.roof, rot=(0, -s * slope, 0))
            for k in range(4):
                t0 = T - k * 0.135
                x, y = on(t0 - 0.085, 0.038 + 0.009 * k)
                width, off = 0.94 / 6, (0.94 / 12 if k % 2 else 0)
                edges = sorted({max(-0.47, min(0.47, -0.47 + off + i * width)) for i in range(-1, 8)})
                for i, (a, b) in enumerate(zip(edges, edges[1:])):
                    if b - a < 0.02:
                        continue
                    tilt = rng.uniform(-1.2, 1.2)
                    F.box(f'shingle{s}{k}{i}', 0.17, 0.016, b - a - 0.006, (x, y, zc + (a + b) / 2), M.roof,
                          rot=(tilt, -s * (slope + 2), 0))
        F.sweep('ridge', [(0, 0.948, zc - 0.47), (0, 0.948, zc + 0.47)], None, M.roof,
                profile=G.rounded_rect(0.05, 0.09, 0.022, corner=2), up=(0, 0, 1))
        # the name plate in the gable
        F.box('plate', 0.26, 0.075, 0.015, (0, 0.6, front + 0.016), M.plate, bevel=0.006, bevel_segments=1)
        for i, ch in enumerate('REX'):
            _letter(ch, (1 - i) * 0.06, 0.6, front + 0.026, 0.042, M.dark)
        # the water bowl by the door, and a chew bone on the plinth
        G.revolve('bowl', [(0, 0), (0.055, 0), (0.065, 0.04), (0.058, 0.045), (0.05, 0.012), (0, 0.012)],
                  at=F.at(0.29, 0, 0.43), material=M.bowl, segments=18)
        F.cyl('bowl_water', 0.052, 0.004, (0.29, 0.034, 0.43), M.bowl_water, verts=18)
        F.sweep('bone', [(-0.3, 0.05, 0.4), (-0.22, 0.05, 0.43)], 0.009, M.plate, verts=6)
        for k, (bx, bz) in enumerate(((-0.3, 0.4), (-0.22, 0.43))):
            for j in (-1, 1):
                F.sphere(f'bone_knob{k}{j}', 0.013, (bx + j * 0.006, 0.052, bz - j * 0.012), M.plate, segments=8, rings=5)
    G.join_all('dogHouse')
    G.settle(centre=False)
    return G.meshes()


# --- the sundial (1x1): a copper dial with its gnomon on a stone baluster ---------------------------------------
def sundial():
    _own('sundial')
    with G.new_parts():
        # two square steps, their top edges chamfered, a few patches of lichen on them
        F.box('step', 0.36, 0.045, 0.36, (0, 0.0225, 0), M.body, bevel=0.008, bevel_segments=1)
        F.box('step2', 0.3, 0.03, 0.3, (0, 0.06, 0), M.body, bevel=0.006, bevel_segments=1)
        for i, (x, z, r) in enumerate(((0.13, 0.12, 0.02), (-0.11, 0.15, 0.014), (0.12, -0.14, 0.017))):
            F.cyl(f'lichen{i}', r, 0.002, (x, 0.045, z), M.lichen, verts=7, scale=(1.3, 1, 1))
        # the baluster: a torus and a scotia at its foot, a swelling vase, a ringed neck and a spreading capital
        G.revolve('pedestal', [(0, 0.075), (0.125, 0.075), (0.125, 0.095), (0.135, 0.105), (0.13, 0.12), (0.11, 0.128),
                               (0.095, 0.14), (0.1, 0.155), (0.09, 0.17), (0.085, 0.22), (0.098, 0.32), (0.1, 0.4),
                               (0.088, 0.5), (0.068, 0.6), (0.06, 0.64), (0.072, 0.652), (0.072, 0.668), (0.06, 0.678),
                               (0.066, 0.7), (0.09, 0.73), (0.125, 0.765), (0.14, 0.78), (0.14, 0.79), (0, 0.79)],
                  at=F.at(0, 0, 0), material=M.body, segments=20)
        # a square cap stone, chamfered, the dial on it
        F.box('cap', 0.32, 0.05, 0.32, (0, 0.815, 0), M.body, bevel=0.012, bevel_segments=1)
        F.box('cap_top', 0.29, 0.012, 0.29, (0, 0.846, 0), M.body, bevel=0.004, bevel_segments=1)
        F.cyl('dial', 0.15, 0.012, (0, 0.856, 0), M.dial, verts=28, bevel=0.002, bevel_segments=1)
        G.torus('dial_rim', 0.146, 0.006, at=F.at(0, 0.862, 0), material=M.dial, major_segments=28, minor_segments=4)
        G.torus('chapter_ring', 0.088, 0.0025, at=F.at(0, 0.862, 0), material=M.dial_mark, major_segments=24,
                minor_segments=3)
        for a in range(0, 360, 30):    # the hour marks, longer at the quarters, a dot at each half hour
            r = 0.115 if a % 90 else 0.11
            F.box(f'mark{a}', 0.008, 0.003, 0.03 if a % 90 else 0.04, (r * G.sind(a), 0.8635, r * G.cosd(a)),
                  M.dial_mark, rot=(0, 0, -a))
            F.cyl(f'dot{a}', 0.004, 0.003, (0.118 * G.sind(a + 15), 0.8635, 0.118 * G.cosd(a + 15)), M.dial_mark, verts=5)
        # a compass rose in the middle: an eight-pointed star
        rose = [((0.06 if k % 4 == 0 else 0.035 if k % 2 == 0 else 0.012) * G.cosd(22.5 * k),
                 (0.06 if k % 4 == 0 else 0.035 if k % 2 == 0 else 0.012) * G.sind(22.5 * k)) for k in range(16)]
        F.extrude('rose', rose, 0.002, (0, 0.8625, 0), M.dial_mark, plane='top')
        F.cyl('boss', 0.014, 0.008, (0, 0.866, 0), M.dial_mark, verts=10, bevel=0.002, bevel_segments=1)
        # the gnomon: a plate along the noon line, its edge at the latitude's angle, pierced
        hole = [(-0.055 + 0.018 * G.cosd(a), 0.03 + 0.018 * G.sind(a)) for a in range(0, 360, 36)]
        F.extrude('gnomon', [(0.11, 0.0), (-0.1, 0.0), (-0.1, 0.1), (-0.085, 0.1)], 0.008, (0, 0.862, 0), M.dial,
                  plane='side', holes=[hole])
    G.join_all('sundial')
    G.settle()
    return G.meshes()


# --- the stepping stones (1x3, rug layer): four flat stones set in the lawn -----------------------------------
STONES = [(0.05, -1.05, 0.52, 10), (-0.08, -0.35, 0.48, 40), (0.06, 0.35, 0.54, 70), (-0.04, 1.05, 0.5, 15)]
STONE_H = 0.04


def steppingStones():
    _own('steppingStones')
    rng = random.Random(31)
    with G.new_parts():
        for i, (x, z, d, r) in enumerate(STONES):
            s = G.boulder(f'stone{i}', d / 2, at=F.at(x, 0, z), material=M.body, rot=(0, 0, r), seed=i * 3.1 + 0.4,
                          noise=0.24, subdivisions=3)
            # flatten it into a slab: a worn, faintly uneven top with rounded edges, its bottom on the ground
            bpy.context.view_layer.update()
            top = max(v.co.z for v in s.data.vertices)
            for v in s.data.vertices:
                k = v.co.z / top
                edge = min(1.0, k * 2.2)
                relief = noise.noise(Vector((v.co.x * 9 + i, v.co.y * 9 - i, 0.5)))
                v.co.z = (STONE_H - 0.0045) * edge ** 0.6 + (0.0025 + 0.002 * relief) * edge ** 4
                v.co.y *= 0.86
            s.data.update()
        # moss creeping in at the stones' edges, a scatter of gravel between them
        for i, (x, z, d, r) in enumerate(STONES):
            for k in range(3):
                a = rng.uniform(0, math.tau)
                rr = d / 2 * rng.uniform(0.75, 0.9)
                mx = max(-0.2, min(0.2, x + rr * math.cos(a)))
                G.boulder(f'moss{i}{k}', rng.uniform(0.035, 0.045), at=F.at(mx, 0.026, z + 0.86 * rr * math.sin(a)),
                          material=M.moss, scale=(1.5, 1.0, 0.12), rot=(0, 0, rng.uniform(0, 180)), seed=i * 5 + k, noise=0.4,
                          subdivisions=1)
        for i in range(14):
            x, z = rng.uniform(-0.25, 0.25), rng.uniform(-1.2, 1.2)
            if any(abs(x - sx) < d * 0.45 and abs(z - sz) < d * 0.4 for sx, sz, d, _ in STONES):
                continue
            G.boulder(f'gravel{i}', rng.uniform(0.014, 0.022), at=F.at(x, 0, z), material=M.pebble, scale=(1.3, 1.0, 0.6),
                      seed=i * 1.9, noise=0.35, subdivisions=1)
    G.join_all('steppingStones')
    G.settle()
    return G.meshes()


# --- the garden lamp post (1x1): a lantern of frosted panes on an iron post; the panes glow at night -----------
LAMP_Y = 2.3       # the content's `light.height`: the panes' middle


def lampPost():
    _own('lampPost')
    post, frame = M.body, M.lantern
    sq = 2 ** 0.5

    def square(h, y, r):
        """A square ring of half side `r` at game height `y`, turned to face the front: a loft section."""
        return [F.at(r * x, y, r * z) for x, z in ((1, 1), (-1, 1), (-1, -1), (1, -1))]
    with G.new_parts():
        # the base: a plinth, a torus, a bell and a waisted neck, turned in one piece
        G.revolve('base', [(0, 0), (0.15, 0), (0.15, 0.022), (0.14, 0.03), (0.135, 0.045), (0.142, 0.06), (0.128, 0.075),
                           (0.105, 0.09), (0.098, 0.13), (0.08, 0.18), (0.062, 0.24), (0.052, 0.3), (0.058, 0.315),
                           (0.058, 0.335), (0.045, 0.345), (0, 0.345)], at=F.at(0, 0, 0), material=post, segments=low(16, 8))
        # the shaft: fluted, a plain neck and collars over it
        flutes = [((0.035 if k % 2 == 0 else 0.03) * math.cos(math.pi * k / 10), (0.035 if k % 2 == 0 else 0.03)
                   * math.sin(math.pi * k / 10)) for k in range(20)]
        F.sweep('pole', [(0, 0.34, 0), (0, 1.75, 0)], None, post, profile=flutes, up=(0, 0, 1), angle=60,
                scale=[1.08, 0.94])
        F.cyl('pole_neck', 0.031, 0.32, (0, 1.9, 0), post, verts=12)
        for y, r in ((0.36, 0.05), (1.76, 0.042), (2.06, 0.05)):
            G.revolve(f'collar{y}', [(0, -0.02), (r * 0.8, -0.02), (r, -0.01), (r, 0.01), (r * 0.8, 0.02), (0, 0.02)],
                      at=F.at(0, y, 0), material=post, segments=12)
        # the ladder bar a lamplighter leant his ladder on, its ends turned into balls
        F.sweep('ladder_bar', [(-0.13, 1.95, 0), (0.13, 1.95, 0)], 0.009, post, verts=8)
        for s in (-1, 1):
            F.sphere(f'bar_ball{s}', 0.017, (s * 0.135, 1.95, 0), post, segments=10, rings=6)
        F.cone('cup', 0.045, 0.1, 0.07, (0, 2.115, 0), post, verts=12)
        # four scroll brackets from the collar out under the lantern's corners, each curling back in
        for k in range(4):
            a = 45 + 90 * k
            c, s = G.cosd(a), G.sind(a)
            pts = [(0.035, 2.0), (0.07, 2.02), (0.105, 2.07), (0.11, 2.12), (0.095, 2.145), (0.08, 2.13), (0.085, 2.115)]
            F.sweep(f'bracket{k}', [(r * c, y, r * s) for r, y in pts], 0.0065, post, verts=5, resolution=2,
                    scale=[1.2, 1.0, 1.0, 1.0, 0.9, 0.8, 0.7])
        # the lantern: four frosted panes, wider at the top, in a frame of corner bars and bands under a flared roof
        F.cone('panes', 0.09 * sq, 0.125 * sq, 0.32, (0, LAMP_Y, 0), M.pane, verts=4, rot=(0, 0, 45), smooth=False)
        G.loft('lantern_floor', [square(0, LAMP_Y - 0.19, 0.08), square(0, LAMP_Y - 0.17, 0.112),
                                 square(0, LAMP_Y - 0.155, 0.112), square(0, LAMP_Y - 0.15, 0.1)], material=frame, smooth=False)
        for k in range(4):
            a = 45 + 90 * k
            lo, hi = 0.094 * sq, 0.129 * sq
            F.sweep(f'frame{k}', [(lo * G.cosd(a), LAMP_Y - 0.16, lo * G.sind(a)), (hi * G.cosd(a), LAMP_Y + 0.16, hi * G.sind(a))],
                    None, frame, profile=[(-0.008, -0.008), (0.008, -0.008), (0.008, 0.008), (-0.008, 0.008)], up=(0, 0, 1))
        for y, r in ((LAMP_Y + 0.07, 0.117),):     # a band round the panes, two thirds up
            G.loft('band', [square(0, y - 0.008, r), square(0, y + 0.008, r + 0.003)], material=frame, smooth=False)
        # the roof: an eave lip, a concave pyramid flaring out over the panes, a vent drum, a crown and a spike
        G.loft('roof', [square(0, LAMP_Y + 0.155, 0.13), square(0, LAMP_Y + 0.168, 0.16), square(0, LAMP_Y + 0.185, 0.16),
                        square(0, LAMP_Y + 0.215, 0.12), square(0, LAMP_Y + 0.26, 0.075), square(0, LAMP_Y + 0.31, 0.045),
                        square(0, LAMP_Y + 0.335, 0.03)], material=frame, smooth=False)
        G.revolve('crown', [(0, 0.0), (0.04, 0.0), (0.042, 0.035), (0.032, 0.045), (0.022, 0.05), (0.024, 0.07),
                            (0.014, 0.085), (0.006, 0.12), (0, 0.135)], at=F.at(0, LAMP_Y + 0.3, 0), material=frame,
                  segments=low(12, 6))
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
    rng = random.Random(23)
    with G.new_parts() as still:
        for s in (-1, 1):
            x = s * POST_X
            # a tapered pot with a rolled rim, gravel over its soil, a few leaves of ivy at the post's foot
            G.revolve(f'pot{s}', [(0, 0), (0.095, 0), (0.1, 0.012), (0.117, 0.25), (0.125, 0.262), (0.127, 0.29),
                                  (0.121, 0.3), (0.108, 0.3), (0.104, 0.28), (0, 0.28)],
                      at=F.at(x, 0, 0), material=M.planter, segments=18)
            F.cyl(f'pot_soil{s}', 0.104, 0.01, (x, 0.275, 0), M.soil, verts=14)
            for k in range(5):
                a = rng.uniform(0, math.tau)
                r = rng.uniform(0.05, 0.085)
                G.boulder(f'gravel{s}{k}', 0.014, at=F.at(x + r * math.cos(a), 0.279, r * math.sin(a)), material=M.pebble,
                          scale=(1.2, 1, 0.6), seed=k * 1.7 + s, noise=0.3, subdivisions=1)
            for k in range(4):
                a = 90 * k + 45 + rng.uniform(-20, 20)
                leaf(f'ivy{s}{k}', F.at(x + 0.04 * G.cosd(a), 0.28, 0.04 * G.sind(a)), -a + 90, rng.uniform(25, 45),
                     0.07, 0.045, M.reed, shape='oval', droop=0.25, thick=0.004, fold=0.15, rows=3, ring=3)
            # the post: chamfered, a pyramid cap on a collar, a steel eye screwed in for the wire, a hook below it
            F.box(f'post{s}', 0.07, 2.12, 0.07, (x, 0.2 + 1.06, 0), M.body, bevel=0.008, bevel_segments=1)
            F.box(f'collar{s}', 0.082, 0.02, 0.082, (x, 2.3, 0), M.body, bevel=0.004, bevel_segments=1)
            F.cone(f'cap{s}', 0.058, 0.008, 0.06, (x, 2.34, 0), M.body, verts=4, rot=(0, 0, 45), smooth=False)
            G.torus(f'eye{s}', 0.018, 0.004, at=F.at(x - s * 0.045, WIRE_Y, 0), material=M.wire, rot=(90, 0, 0),
                    major_segments=10, minor_segments=4)
            F.sweep(f'eye_shank{s}', [(x - s * 0.035, WIRE_Y - 0.018, 0), (x - s * 0.02, WIRE_Y - 0.018, 0)], 0.003, M.wire,
                    verts=4)
            F.sweep(f'hook{s}', G.kit.fillet([(x - s * 0.035, 1.2, 0), (x - s * 0.06, 1.2, 0), (x - s * 0.06, 1.24, 0)], 0.01),
                    0.004, M.wire, verts=5)
    G.join(still, 'posts', origin=(0, 0, 0))
    with G.new_parts() as string:
        x0 = POST_X - 0.04
        n = 24
        # the wire in one piece, sagging between the eyes, a drop to each socket
        F.sweep('wire', [(-x0 + 2 * x0 * i / n, _wire_y(-x0 + 2 * x0 * i / n), 0) for i in range(n + 1)], 0.0045, M.wire,
                verts=5)
        heights = []
        for i in range(BULBS):
            x = -x0 + 2 * x0 * (i + 0.5) / BULBS
            y = _wire_y(x)
            # the socket: a clip round the wire and a ribbed holder; the bulb a festoon globe on a short neck
            bx, by, bz = F.at(x, 0, 0)
            G.loft(_name('socket'), [G.ring(r, r, y + dy, 6, at=(bx, by)) for dy, r in
                                     ((-0.05, 0.011), (-0.042, 0.014), (-0.03, 0.014), (-0.026, 0.012), (-0.016, 0.012),
                                      (-0.008, 0.008), (0.002, 0.006))], material=M.socket)
            G.loft(_name('bulb'), [G.ring(r, r, y + dy, 8, at=(bx, by)) for dy, r in
                                   ((-0.05, 0.009), (-0.058, 0.012), (-0.068, 0.026), (-0.085, 0.035), (-0.105, 0.034),
                                    (-0.122, 0.024), (-0.13, 0.01), (-0.133, 0.003))], material=M.bulb)
            heights.append(y - 0.09)
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
