# The outdoor things. Notes: outdoor.md.
#
# OU1, patio seating and tables: the park bench (`model.bench`), the folding bistro set (`model.patioSet`), the
# picnic table (`model.picnicTable`), the garden chair (`model.gardenChair`), the round mosaic table
# (`model.patioTableRound`), the teak table (`model.patioTable4`), the patio chair (`model.patioChair`), the
# Adirondack chair (`model.adirondackChair`), the three-seat garden bench (`model.benchGarden3`), the outdoor bar
# (`model.outdoorBar`) and the parasol (`model.parasol`), whose canopy stirs in the wind (`stir`, ambient).
#
# OU3, grills and fire: the charcoal grill (`model.grill`, its lid swings up while cooking), the gas grill station
# (`model.grillStation`, its hood swings up) and the fire pit (`model.firePit`, its flames grow while warmed by).
#
# Each is built in place where the game placed it (`F`, game axes), so the seat heights (content `seat`)
# and the tops things stand on (content `surface`) stay where the game has them.
#
# Run:  cargo art outdoor [--only bench] [--preview]
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    iron=('#2A2C30', 'metal'),           # cast and wrought iron frames
    park_slats=('#9A6E4A', 'wood'),      # the park bench's slats
    bistro=('#B5784A', 'wood'),          # the bistro set's slats
    pine=('#9C7A58', 'wood'),            # the picnic table's planks
    pine_dark=('#7E5F42', 'wood'),       # its A-frames
    teak=('#9A6B45', 'wood'),            # the teak table's top, the bar's body
    teak_dark=('#7E5636', 'wood'),       # the teak table's legs and aprons
    bench_slats=('#A9744E', 'wood'),     # the garden bench's slats
    bar_top=('#C9A06A', 'wood'),         # the bar's counter
    steel=('#8E959B', 'metal'),          # the bar's foot rail
    garden_white=('#F2F0EA', 'satin'),   # the garden chair
    adirondack=('#3E7A6E', 'satin'),     # the Adirondack chair
    patio_green=('#2F5A4E', 'metal'),    # the patio chair's frame
    cushion=('#E8E2D6', 'fabric'),       # its seat cushion
    mosaic=('#3E6E73', 'gloss'),         # the round table's top
    tile=('#E9C46A', 'gloss'),           # its mosaic tiles
    tile_light=('#F2EEE4', 'gloss'),
    canopy=('#E7DCC4', 'fabric'),        # the parasol's canopy
    valance=('#C9A979', 'fabric'),       # its scalloped edge
    pole=('#E7E3DC', 'satin'),           # its pole
    base=('#3A3D41', 'satin'),           # its weighted base
    cloth=('#D9483B', 'fabric'),         # the picnic table's gingham cloth
    cloth_check=('#F4EFE6', 'fabric'),
    wicker=('#C9A979', 'fabric'),        # the picnic basket
    wicker_rim=('#B8955F', 'fabric'),
    handle=('#8E6E52', 'wood'),          # the basket's handle
    cup=('#F2F1EE', 'gloss'),            # cups and a jug on the tables
    jug=('#9DC6CF', 'gloss'),
    # OU3, the grills and the fire pit
    kettle=('#2A2C30', 'gloss'),         # the charcoal grill's enamelled bowl (Body)
    kettle_lid=('#2A2C30', 'gloss'),     # its lid (Lid)
    grate=('#3A3D41', 'metal'),          # cooking grates, legs, the kettle's vent and handle brackets
    handle_steel=('#D0D3D6', 'chrome'),  # handles and knobs
    wheel=('#2A2C30', 'satin'),
    coals=('#3A3330', 'matte'),          # a charcoal or ash bed
    ember=('#D9542B', 'gloss', {'glow': '#FF7A2E'}),        # the fire pit's embers: red by day, aglow at night
    coal_hot=('#D9542B', 'gloss', {'glow': '#FF7A2E'}),     # the kettle's coals, aglow at night (not through its lid)
    flame=('#FF9A3C', 'gloss', {'glow': '#FFB050'}),        # the fire pit's flames, grown while warmed by
    flame_core=('#FFD27A', 'gloss', {'glow': '#FFE0A0'}),
    burger=('#6B3A26', 'satin'),         # food on the grates
    sausage=('#8E3F2E', 'satin'),
    corn=('#E8C24A', 'satin'),
    cart=('#C3C7CB', 'metal'),           # the grill station's stainless cart (Cart)
    hood=('#C3C7CB', 'metal'),           # its hood (Hood)
    worktop=('#3A3D41', 'satin'),        # its dark counter
    firebox=('#1F2023', 'matte'),        # inside the grill and under the hood
    gauge=('#F2F1EE', 'gloss'),          # the hood's thermometer face
    board=('#CDB08A', 'wood'),           # the chopping board
    tray=('#E9E6E0', 'gloss'),           # the tray of food
    bottle=('#6FA6D9', 'gloss'),
    bottle_green=('#3E6B48', 'gloss'),   # the bar's bottles and glasses
    bottle_amber=('#8A5A2B', 'gloss'),
    bottle_clear=('#CFE3E6', 'gloss'),
    pit=('#8E8A84', 'matte'),            # the fire pit's bowl (Bowl)
    pit_stones=('#A19C94', 'matte'),     # its capstones (Stones)
    log=('#6B4A33', 'bark'),
    log_end=('#B89A72', 'matte'),        # the logs' sawn ends
    # OU2
    lounger=('#CDB08A', 'wood'),         # the sun lounger's frame
    lounger_pad=('#E8E2D6', 'fabric'),   # its cushions (the back is upholstered all round)
    towel=('#E9853B', 'fabric'),         # a beach towel
    pillow=('#FFFFFF', 'fabric'),
    hammock_frame=('#B08E66', 'wood'),
    sling=('#4E7A7F', 'fabric'),         # the hammock's striped sling
    sling_stripe=('#E8E2D6', 'fabric'),
    porch_frame=('#8E6E52', 'wood'),     # the porch swing's and the swing set's A-frames
    porch_seat=('#EDE6D8', 'satin'),
    porch_cushion=('#7FA3B8', 'fabric'),
    chain=('#8E959B', 'satin'),          # chains: plain, so they share the seat's draw call
    swing_frame=('#B08E66', 'wood'),
    swing_seat=('#D9483B', 'satin'),
    cedar=('#A9744E', 'wood'),           # the hot tub's staves
    cedar_band=('#5E4430', 'wood'),      # its bands and steps' edges
    tub_cover=('#5A5F66', 'satin'),      # its insulated cover
    water=('#5FB4C2', 'gloss'),
    foam=('#E6F4F6', 'gloss'),           # bubbles on the water
)

F = G.GameFrame()


def _v(p):
    """A game position or direction as a Blender vector."""
    return Vector(F.at(*p))


def _board(name, a, b, w, t, material, face=(0, 1, 0), bevel=0.006):
    """A board from game point `a` to `b`, `w` wide and `t` thick, its broad face turned towards game direction
    `face` (up for a seat slat, forwards for a back slat): slats, rails and legs at any angle."""
    a, b = _v(a), _v(b)
    z = (b - a).normalized()
    x = _v(face).cross(z).normalized()
    y = z.cross(x)
    obj = G.cube(name, (w, t, (b - a).length), material=material, bevel=bevel)
    rot = Matrix((x, y, z)).transposed()
    obj.matrix_world = Matrix.Translation((a + b) / 2) @ rot.to_4x4()
    return obj


def _rod(name, a, b, r, material, verts=10):
    """A round bar from game point `a` to `b`."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=material, verts=verts)


def _finish(name, centred=False):
    """Joins the model; `centred`: moves it onto its Model's `centre` (its back onto the wall face), else
    leaves it where it was built. Its feet are cut flat on the floor first: a leg swept up from it at a slant dips
    a few mm below, and settling would lift the whole model (its seat) by that."""
    obj = G.join_all(name)
    for v in obj.data.vertices:
        v.co.z = max(v.co.z, 0.0)
    obj.data.update()
    G.settle(centre=centred)
    return G.meshes()


def _jitter(obj, amount, size=0.3, seed=0.0, up=1.0):
    """`G.jitter`, but the same in every Blender process: mathutils.noise.noise_vector (which G.jitter uses) gives
    different values from one Blender run to the next (Blender 5.2), noise() doesn't; three noise() calls, offset."""
    from mathutils import noise
    bpy.context.view_layer.update()
    for v in obj.data.vertices:
        p = obj.matrix_world @ v.co
        q = Vector((p.x / size + seed, p.y / size - seed, p.z / size + seed * 0.5))
        n = Vector((noise.noise(q), noise.noise(q + Vector((31.4, 0, 0))), noise.noise(q + Vector((0, 47.1, 0)))))
        v.co += Vector((n.x, n.y, n.z * up)) * amount
    obj.data.update()
    return obj


def _bent(name, pts, r, material, bend=0.03, verts=8, steps=3, scale=1.0):
    """One length of round bar or tube through game points, its corners bent round about `bend` (no overshoot)."""
    return G.kit.tube(F, name, G.kit.fillet(pts, bend, steps), r, material, verts=verts, round=1, scale=scale)


def _bar(name, pts, w, h, material, bend=0.03, steps=3, up=(1, 0, 0), r=None, resolution=1, scale=1.0, closed=False):
    """A flat or square bar (`w` across the path's normal, `h` the other way, corners rounded `r`) through game points,
    its corners bent round `bend`: cast-iron frames, flat straps, rails. `up` (Blender) is the normal's direction."""
    path = G.kit.fillet(pts, bend, steps) if bend and not closed else pts
    prof = G.rounded_rect(w, h, r if r is not None else min(w, h) * 0.3, corner=1)
    return F.sweep(name, path, None, material, profile=prof, up=up, resolution=resolution, scale=scale, angle=60,
                   closed=closed)


def _plank(name, a, b, t, material, outline, face=(0, 1, 0), bevel=0.0):
    """A board from game point `a` towards `b`, `t` thick, cut to `outline` [(across, along from a), ...] (a rounded or
    pointed end, an arched top), its broad face towards game direction `face`: shaped slats, stringers, arms."""
    A, B = _v(a), _v(b)
    z = (B - A).normalized()
    x = _v(face).cross(z).normalized()
    y = z.cross(x)
    obj = G.extrude(name, outline, t, material=material, plane='XZ', bevel=bevel, bevel_segments=1)
    obj.matrix_world = Matrix.Translation(A) @ Matrix((x, y, z)).transposed().to_4x4()
    return obj


def _end_round(w, length, r=None, steps=4, both=False):
    """A board's outline `w` across and `length` along, its far end rounded (`r`, a full round by default); `both`:
    its near end too."""
    r = min(r or w / 2, w / 2)
    far = [((w / 2 - r) + r * math.cos(math.pi / 2 * i / steps), length - r + r * math.sin(math.pi / 2 * i / steps))
           for i in range(steps + 1)]
    far += [(-u, v) for u, v in reversed(far)]
    if not both:
        return [(-w / 2, 0.0), (w / 2, 0.0)] + far
    near = [(-u, length - v) for u, v in far]
    return near + far


def _bolts(prefix, points, material, r=0.008, face=(0, 1, 0)):
    """Round bolt heads at game points, domed out towards `face` (up, or forward on a back slat)."""
    for k, p in enumerate(points):
        G.cylinder(f'{prefix}{k}', r, 0.005, at=F.at(*p), material=material, verts=6,
                   rot=(0, 0, 0) if face[1] else (90, 0, 0))


# --- the park bench: cast-iron ends, four seat slats and three back slats -------------------------------------
BENCH_SEAT = 0.45   # the slats' top: `seats.bench` (furniture.json)


def _spiral(y, z, r0, r1, a0, turns, n, s=1):
    """Game (height, depth) points of a scroll's spiral about (y, z), from radius `r0` at angle `a0` (degrees, 0 towards
    +depth, 90 up) in towards `r1`, `turns` round (s=1: up first, then back)."""
    return [(y + (r0 + (r1 - r0) * i / n) * G.sind(a0 + s * 360 * turns * i / n),
             z + (r0 + (r1 - r0) * i / n) * G.cosd(a0 + s * 360 * turns * i / n)) for i in range(n + 1)]


def _cast_end(p, x, back, front, rail, stretcher, arm=None, scroll=None, mat=None):
    """A cast-iron bench end in the side plane at game x, each line one cast piece: the back leg sweeping up into the
    back's support (`back`, (height, depth) points from the foot), the front leg (`front`, from the foot; with an
    `arm`, rising into it and running back to the support), the seat rail (`rail`: height, depth from, to), an arched
    stretcher (`stretcher`: height, depth from, to), a pad under each foot; `arm` (height, depth from, to): a flat
    armrest along the arm's top; `scroll` (height, depth, radius): a scroll in the angle of the leg and the arm."""
    mat = mat or M.iron

    def pts(yz):
        return [(x, y, z) for y, z in yz]
    _bar(f'{p}back', pts(back), 0.03, 0.044, mat, bend=0.12, steps=3, r=0.008)
    _bar(f'{p}front', pts(front), 0.03, 0.04, mat, bend=0.05, steps=3, r=0.008)
    y, z0, z1 = rail
    _bar(f'{p}rail', pts([(y, z0), (y, z1)]), 0.026, 0.04, mat, bend=0, r=0.006)
    y, z0, z1 = stretcher
    _bar(f'{p}stretcher', pts([(y - 0.01, z0), (y + 0.035, (z0 + z1) / 2), (y - 0.01, z1)]), 0.02, 0.026, mat, bend=0,
         r=0.005, resolution=3)
    for fy, fz in (back[0], front[0]):
        F.box(f'{p}foot{fz}', 0.05, 0.016, 0.075, (x, 0.008, fz), mat, bevel=0.006, bevel_segments=1)
    if arm:
        y, z0, z1 = arm
        _plank(f'{p}armrest', (x, y, z0), (x, y, z1), 0.014, mat, _end_round(0.056, z1 - z0, steps=3), bevel=0.004)
    if scroll:
        y, z, r = scroll
        _bar(f'{p}scroll', pts(_spiral(y, z, r, r * 0.25, 0, 1.15, 13)), 0.02, 0.012, mat, bend=0, r=0)


def bench():
    with G.new_parts():
        for s in (-1, 1):
            x = s * 0.78
            # the back leg splays back at the foot, rises in line with the back slats and curls back at the top
            _cast_end(f'end{s}', x,
                      back=[(0.0, -0.135), (0.08, -0.118), (0.4, -0.15), (0.78, -0.192), (0.85, -0.198),
                            (0.868, -0.218), (0.852, -0.232)],
                      front=[(0.0, 0.205), (0.42, 0.212), (0.6, 0.235), (0.66, 0.256), (0.664, 0.2), (0.664, -0.18)],
                      rail=(0.395, -0.17, 0.236), stretcher=(0.1, -0.125, 0.208), arm=(0.678, -0.175, 0.275),
                      scroll=(0.565, 0.17, 0.048))
            # carriage bolts through the slats into the ends
            _bolts(f'bolt_seat{s}', [(x, BENCH_SEAT + 0.002, z) for z in (-0.1, 0.0, 0.1, 0.2)], M.iron)
            _bolts(f'bolt_back{s}', [(x, y, z + 0.017) for y, z in ((0.6, -0.2), (0.72, -0.2168), (0.84, -0.2336))],
                   M.iron, face=(0, 0, 1))
        for k, z in enumerate((-0.1, 0.0, 0.1, 0.2)):
            F.box(f'seat{k}', 1.74, 0.035, 0.085, (0, BENCH_SEAT - 0.0175, z), M.park_slats, bevel=0.012)
        for k, (y, z) in enumerate(((0.6, -0.2), (0.72, -0.2168), (0.84, -0.2336))):
            F.box(f'back{k}', 1.74, 0.085, 0.03, (0, y, z), M.park_slats, rot=(-8, 0, 0), bevel=0.012)
    return _finish('bench')


# --- the folding bistro set: a slatted table between two folding chairs --------------------------------------
def _bistro_chair(c, facing):
    """A folding chair centred at game depth `c`, its front towards `facing` (+1: +z): the back's frame one bent tube
    from the front feet over the top, the back legs another under the seat's front, a tube round the seat, bowed back
    slats and seat slats with rounded ends."""
    def at(x, y, lz):
        return (x * facing, y, c + lz * facing)
    for k, lz in enumerate((-0.14, -0.045, 0.05, 0.145)):
        _plank(f'chair{c}_seat{k}', at(-0.205, 0.43, lz), at(0.205, 0.43, lz), 0.02, M.bistro,
               _end_round(0.085, 0.41, r=0.025, steps=2, both=True))
    lean = (0, G.cosd(10), -G.sind(10) * facing)          # up the back, leaning 10°
    for k, y in enumerate((0.62, 0.74, 0.86)):
        lz = 0.2 - 0.45 * y / 0.93 - 0.013               # just behind the frame's uprights
        bow = [at(x, y, lz - 0.015 * (1 - (x / 0.205) ** 2)) for x in (-0.205, -0.1, 0.0, 0.1, 0.205)]
        F.sweep(f'chair{c}_back{k}', bow, None, M.bistro, profile=G.rounded_rect(0.07, 0.018, 0.007, corner=1),
                up=tuple(_v(lean)), angle=60)
    x = 0.21
    _bent(f'chair{c}_frame', [at(-x, 0.0, 0.2), at(-x, 0.93, -0.25), at(x, 0.93, -0.25), at(x, 0.0, 0.2)], 0.012,
          M.iron, bend=0.06, verts=6)
    _bent(f'chair{c}_legs', [at(-x + 0.012, 0.0, -0.2), at(-x + 0.012, 0.405, 0.16), at(x - 0.012, 0.405, 0.16),
                             at(x - 0.012, 0.0, -0.2)], 0.012, M.iron, bend=0.03, verts=6)
    ring = [at(u, 0.414, v) for u, v in G.rounded_rect(0.42, 0.37, 0.03, corner=2)]
    F.sweep(f'chair{c}_seat_ring', ring, 0.009, M.iron, verts=6, closed=True)
    _rod(f'chair{c}_stretcher', at(-x, 0.12, 0.13), at(x, 0.12, 0.13), 0.008, M.iron, verts=6)
    for s in (-1, 1):                                      # the pivots where the legs cross the frame, glides under
        G.cylinder(f'chair{c}_pivot{s}', 0.014, 0.012, at=F.at(*at(s * (x + 0.006), 0.2, 0.06)), material=M.iron,
                   rot=(0, 90, 0), verts=8)


# The two chairs face each other across the table, their seats' fronts 4 cm under its edges (content `places`).
BISTRO_CHAIRS = (-0.45, 0.45)


def _cup(name, x, y, z, r, h, material, handle=True, turn=0, segments=12):
    """A cup standing at game (x, y, z): a turned body with a foot and a lip, hollow at the top, a loop handle."""
    bx, by, bz = F.at(x, y, z)
    rings = [(0.0, 0.0), (r * 0.78, 0.0), (r * 0.82, h * 0.08), (r * 0.95, h * 0.5), (r, h), (r * 0.9, h), (r * 0.86, h * 0.55),
             (r * 0.7, h * 0.18), (0.0, h * 0.18)]
    G.revolve(name, rings, at=(bx, by, bz), material=material, segments=segments)
    if handle:
        a = math.radians(turn)
        hx, hz = math.cos(a), math.sin(a)
        pts = [(x + hx * r * 0.9, y + h * 0.78, z + hz * r * 0.9), (x + hx * (r + 0.022), y + h * 0.72, z + hz * (r + 0.022)),
               (x + hx * (r + 0.02), y + h * 0.35, z + hz * (r + 0.02)), (x + hx * r * 0.92, y + h * 0.25, z + hz * r * 0.92)]
        F.sweep(f'{name}_handle', pts, 0.005, material, verts=5, resolution=3)


def _jug(name, x, y, z, r, h, material):
    """A jug standing at game (x, y, z): a belly, a waist, a flared lip with a pinched spout, a handle at the back."""
    bx, by, bz = F.at(x, y, z)
    shape = [(0.0, 0.85), (0.02, 0.92), (0.45, 1.0), (0.75, 0.92), (0.88, 0.7), (0.96, 0.78), (1.0, 0.86)]
    sections = []
    for t, k in shape:
        loop = G.ring(r * k, r * k, bz + h * t, 14, at=(bx, by))
        if t == 1.0:                                       # the spout: the lip pulled out towards -x
            loop = [(px + (0.012 if px > bx and abs(py - by) < r * 0.5 else 0.0), py, pz) for px, py, pz in loop]
        sections.append(loop)
    G.loft(name, sections, material=material)
    pts = [(x + r * 0.8, y + h * 0.9, z), (x + r + 0.035, y + h * 0.82, z), (x + r + 0.03, y + h * 0.4, z),
           (x + r * 0.9, y + h * 0.2, z)]
    F.sweep(f'{name}_handle', pts, None, material, profile=G.rounded_rect(0.008, 0.016, 0.004, corner=1), up=(0, 1, 0),
            resolution=3)


def patioSet():
    with G.new_parts():
        for k, z in enumerate((-0.24, -0.12, 0.0, 0.12, 0.24)):
            _plank(f'top{k}', (-0.3, 0.71, z), (0.3, 0.71, z), 0.02, M.bistro, _end_round(0.11, 0.6, r=0.02, steps=2, both=True),
                   bevel=0.004)
        # the frame under the top, and the folding legs: a U each, crossing at the table's sides, clear of the
        # sitters' knees between them
        ring = [(u, 0.69, v) for u, v in G.rounded_rect(0.56, 0.56, 0.04, corner=2)]
        _bar('top_frame', ring, 0.03, 0.012, M.iron, bend=0, r=0.003, up=(0, 0, 1), closed=True)
        for e in (-1, 1):
            x = 0.27 - (0.014 if e > 0 else 0)
            _bent(f'legs{e}', [(-x, 0.0, e * 0.23), (-x, 0.678, -e * 0.23), (x, 0.678, -e * 0.23), (x, 0.0, e * 0.23)],
                  0.0125, M.iron, bend=0.03, verts=6)
        for s in (-1, 1):
            G.cylinder(f'hinge{s}', 0.016, 0.04, at=F.at(s * 0.263, 0.339, 0), material=M.iron, rot=(0, 90, 0), verts=8)
        _cup('cup', 0.12, 0.72, 0.06, 0.038, 0.085, M.cup, turn=0)
        _jug('jug', -0.14, 0.72, -0.08, 0.05, 0.16, M.jug)
        _bistro_chair(BISTRO_CHAIRS[0], 1)
        _bistro_chair(BISTRO_CHAIRS[1], -1)
    return _finish('patioSet')


# --- the picnic table: a planked top on A-frames, a bench each side; a gingham cloth and a basket --------------
PICNIC_TOP, PICNIC_SEAT = 0.73, 0.44   # the top's and the benches' tops: content `surface`, `seats.picnicTable`


def _side_slab(name, x, outline, t, material, bevel=0.006):
    """A board seen from the side at game x, `t` thick across: its outline [(depth, height), ...] (a leg cut level at
    the foot and the top, a bearer or a cleat with its ends chamfered)."""
    return F.extrude(name, outline, t, (x, 0, 0), material, plane='side', bevel=bevel, bevel_segments=1)


def _basket(x, y, z, w, h, d):
    """A wicker picnic basket standing at game (x, y, z): a tapering body, a rim, two lid flaps, a bent-wood handle
    and a strap."""
    bx, by, bz = F.at(x, y, z)
    rings = [(0.0, 0.9), (0.02, 0.94), (0.7, 1.0), (0.78, 1.0)]
    G.loft('basket', [G.ring(w / 2 * k, d / 2 * k, bz + h * t, 16, square=7, at=(bx, by)) for t, k in rings],
           material=M.wicker)
    rim = [(x + u, y + h * 0.78, z + v) for u, v in G.rounded_rect(w + 0.012, d + 0.012, 0.03, corner=2)]
    F.sweep('basket_rim', rim, 0.009, M.wicker_rim, verts=5, closed=True)
    for s in (-1, 1):                                      # the two lid flaps, hinged along the middle
        F.soft(f'basket_lid{s}', w + 0.004, 0.03, d / 2 - 0.004, (x, y + h * 0.78 + 0.02, z + s * d / 4), M.wicker_rim,
               rot=(s * 3, 0, 0), round=0.012, round_segments=1, flat=1)
    arc = [(x + 0.5 * w * math.cos(math.radians(a)) * 0.82, y + h * 0.8 + 0.156 * math.sin(math.radians(a)), z)
           for a in range(0, 181, 20)]
    F.sweep('basket_handle', arc, None, M.handle, profile=G.rounded_rect(0.012, 0.022, 0.005, corner=1), up=(0, 1, 0))
    F.box('basket_strap', 0.03, h * 0.45, 0.006, (x, y + h * 0.62, z + d / 2 + 0.005), M.handle, bevel=0.002,
          bevel_segments=1)


def picnicTable():
    top, seat = PICNIC_TOP, PICNIC_SEAT
    with G.new_parts():
        for k, z in enumerate((-0.29, -0.145, 0.0, 0.145, 0.29)):
            F.box(f'top{k}', 2.5, 0.04, 0.135, (0, top - 0.02, z), M.pine, bevel=0.01)
        for s in (-1, 1):
            for k, z in enumerate((0.6, 0.745)):
                F.box(f'seat{s}{k}', 2.5, 0.04, 0.13, (0, seat - 0.02, s * z), M.pine, bevel=0.01)
        # the cleats under the top and the bearers under the seats, their ends cut back underneath
        for x in (-0.95, 0.0, 0.95):
            yc = top - 0.065
            _side_slab(f'cleat{x}', x, [(-0.36, yc + 0.025), (0.36, yc + 0.025), (0.32, yc - 0.025), (-0.32, yc - 0.025)],
                       0.07, M.pine_dark)
        hw = 0.035 / G.cosd(38)
        for s in (-1, 1):
            x = s * 0.95
            yb = seat - 0.07
            _side_slab(f'bearer{s}', x, [(-0.81, yb + 0.03), (0.81, yb + 0.03), (0.77, yb - 0.03), (-0.77, yb - 0.03)],
                       0.07, M.pine_dark)
            # the A-frame's legs, cut level at the foot and under the cleat, either side of the cleat and bearer
            for e in (-1, 1):
                zf, zt = e * 0.66, e * 0.12
                _side_slab(f'leg{s}{e}', x + s * 0.06, [(zf - hw, 0.0), (zf + hw, 0.0), (zt + hw, top - 0.04), (zt - hw, top - 0.04)],
                           0.05, M.pine_dark)
                # carriage bolts through leg and bearer, leg and cleat
                for y in (yb, top - 0.065):
                    zl = zf + (zt - zf) * y / (top - 0.04)
                    G.cylinder(f'bolt{s}{e}{y}', 0.011, 0.006, at=F.at(x + s * 0.087, y, zl), material=M.grate,
                               rot=(0, 90, 0), verts=6)
            _board(f'brace{s}', (s * 0.92, seat - 0.07, 0), (s * 0.04, top - 0.07, 0), 0.05, 0.05, M.pine_dark, bevel=0.006)
        # the cloth: red gingham laid across the table where the two sit, hanging over both long edges
        e, hang, ct = 0.362, 0.07, 0.004
        drape = G.kit.fillet([(-0.3, top - hang, -e), (-0.3, top + ct / 2, -e), (-0.3, top + ct / 2, e),
                              (-0.3, top - hang, e)], 0.012, steps=2)
        F.sweep('cloth', drape, None, M.cloth, profile=G.rounded_rect(0.7, ct, 0.0015, corner=1), up=(1, 0, 0))
        cw, cd = 0.7 / 8, 2 * e / 8
        for i in range(8):
            for j in range(8):
                if (i + j) % 2 == 0:
                    F.box(f'check{i}{j}', cw, 0.0012, cd, (-0.65 + cw * (i + 0.5), top + ct + 0.0006, -e + cd * (j + 0.5)),
                          M.cloth_check)
        for s in (-1, 1):                                  # the checks down the hanging edges
            for i in range(8):
                if (i + (0 if s < 0 else 1)) % 2 == 0:
                    F.box(f'check_hang{s}{i}', cw, hang - 0.02, 0.0012, (-0.65 + cw * (i + 0.5), top - hang / 2 - 0.006,
                                                                          s * (e + ct / 2 + 0.0006)), M.cloth_check)
        _basket(0.55, top, 0.02, 0.4, 0.2, 0.28)
        _cup('cup0', -0.45, top + ct, 0.12, 0.038, 0.09, M.cup, turn=200)
        _cup('cup1', -0.15, top + ct, -0.1, 0.035, 0.1, M.jug, handle=False)
    return _finish('picnicTable')


# --- the garden chair: a white slatted armchair, the back leaning, a cup on the arm ----------------------------
def _back_slats(prefix, xs, lengths, material, fan, bottom=(0.29, -0.123), tilt=22, w=0.085, t=0.022, r=0.012):
    """Back slats up a back leaning `tilt` degrees, from `bottom` (height, depth); `fan` spreads their tops; their tops
    rounded by `r` (half their width: a full round). Returns each slat's (bottom, top) game points."""
    up = (0, G.cosd(tilt), -G.sind(tilt))
    face = (0, G.sind(tilt), G.cosd(tilt))
    y0, z0 = bottom
    ends = []
    for k, (x, length) in enumerate(zip(xs, lengths)):
        top = (x * fan, y0 + length * up[1], z0 + length * up[2])
        _plank(f'{prefix}{k}', (x, y0, z0), top, t, material, _end_round(w, length, r=r, steps=3), face=face, bevel=0.005)
        ends.append(((x, y0, z0), top))
    return ends


def _stringer(name, x, front, back, w, t, material):
    """A side board from `front` (height, depth) down to `back`, `w` deep, `t` thick across: its front end cut in a
    round, its back cut level on the floor."""
    (y1, z1), (y0, z0) = front, back
    d = Vector((z1 - z0, y1 - y0))
    n = Vector((-d.y, d.x)).normalized() * (w / 2)
    u = d.normalized()
    out = [(z0 - n.x, y0 - n.y), (z0 + n.x, y0 + n.y)]
    for i in range(7):                                   # the round front end
        a = math.pi / 2 - math.pi * i / 6
        c = Vector((z1, y1)) + u * (w / 2) * math.cos(a) * 0.9 + n * math.sin(a)
        out.append((c.x, c.y))
    return _side_slab(name, x, out, t, material, bevel=0.005)


def _corbel(name, x, y, z, h, d, t, material):
    """A curved bracket under an arm at game (x, y = its top, z = its front), `h` tall and `d` deep, `t` thick."""
    curve = [(z - d + d * (1 - math.cos(math.pi / 2 * i / 5)), y - h * math.sin(math.pi / 2 * i / 5)) for i in range(6)]
    return _side_slab(name, x, [(z - d, y), (z, y)] + [(z, y - h)] + list(reversed(curve))[1:-1], t, material, bevel=0.004)


def _arm_board(name, x, y, z0, z1, back_w, front_w, material, t=0.025):
    """A wide flat arm from depth z0 (the back) to z1, `back_w` wide at the back widening to `front_w`, its front end
    rounded."""
    L, r = z1 - z0, front_w * 0.35
    out = [(-back_w / 2, 0.0), (back_w / 2, 0.0), (front_w / 2, L * 0.55)]
    out += [((front_w / 2 - r) + r * math.cos(math.pi / 2 * i / 3), L - r + r * math.sin(math.pi / 2 * i / 3)) for i in range(4)]
    out += [(-u, v) for u, v in reversed(out[3:])] + [(-front_w / 2, L * 0.55)]
    return _plank(name, (x, y, z0), (x, y, z1), t, material, out, bevel=0.006)


def _slatted_chair(prefix, material, fan):
    """The garden and Adirondack chairs' shared frame: a seat sloping back (its front slat rounded over), wide shaped
    arms on front legs with curved brackets, back legs and stringers rounded at the front; the back slats are the
    caller's."""
    for k, z in enumerate((-0.12, 0.0, 0.12, 0.24)):
        F.box(f'{prefix}seat{k}', 0.56, 0.025, 0.11, (0, 0.4 - z * 0.15, z), material, rot=(-8.5, 0, 0),
              bevel=0.012 if k == 3 else 0.008, bevel_segments=2)
    _board(f'{prefix}back_rail', (-0.3, 0.56, -0.25), (0.3, 0.56, -0.25), 0.06, 0.03, material,
           face=(0, G.sind(22), G.cosd(22)))
    wide = fan > 1
    for s in (-1, 1):
        ax = s * (0.37 if wide else 0.36)
        _arm_board(f'{prefix}arm{s}', ax, 0.62, -0.37 if wide else -0.35, 0.33 if wide else 0.31,
                   0.09, 0.14 if wide else 0.12, material)
        F.box(f'{prefix}leg_front{s}', 0.06, 0.61, 0.06, (s * 0.36, 0.305, 0.26), material, bevel=0.01)
        F.box(f'{prefix}leg_back{s}', 0.06, 0.42, 0.06, (s * 0.32, 0.21, -0.24), material, bevel=0.01)
        _stringer(f'{prefix}stringer{s}', s * 0.3, (0.425, 0.268), (0.32, -0.28), 0.08, 0.05, material)
        _corbel(f'{prefix}arm_brace{s}', s * 0.36, 0.608, 0.22, 0.13, 0.09, 0.04, material)


def gardenChair():
    with G.new_parts():
        _slatted_chair('', M.garden_white, fan=1.0)
        xs = (-0.24, -0.12, 0.0, 0.12, 0.24)
        _back_slats('back', xs, (0.74, 0.79, 0.81, 0.79, 0.74), M.garden_white, fan=1.0, w=0.1, r=0.012)
        # the top rail: one board, its top edge arched, across the slats' fronts, leaning with them
        arch = [(u, -0.03) for u in (-0.3, 0.3)] + [(0.3 - 0.6 * i / 8, 0.018 + 0.016 * math.sin(math.pi * i / 8)) for i in range(9)]
        F.extrude('top_rail', arch, 0.025, (0, 0.95, -0.366), M.garden_white, bevel=0.006, bevel_segments=1, rot=(-22, 0, 0))
        _cup('cup', 0.36, 0.6325, 0.18, 0.045, 0.1, M.cup, turn=60)
    return _finish('gardenChair', centred=True)


def adirondackChair():
    with G.new_parts():
        _slatted_chair('', M.adirondack, fan=1.6)
        xs = (-0.18, -0.09, 0.0, 0.09, 0.18)
        ends = _back_slats('back', xs, (0.78, 0.84, 0.87, 0.84, 0.78), M.adirondack, fan=1.6, r=0.042)
        # the upper back rail, bowed, behind the slats half way up
        face = Vector((0, G.sind(22), G.cosd(22)))
        rail = [tuple(Vector(b) + (Vector(t) - Vector(b)) * 0.52 - face * 0.022) for b, t in ends]
        rail = [(-0.27, rail[0][1], rail[0][2] + 0.012)] + rail + [(0.27, rail[-1][1], rail[-1][2] + 0.012)]
        F.sweep('back_rail_upper', rail, None, M.adirondack, profile=G.rounded_rect(0.06, 0.022, 0.006, corner=1),
                up=tuple(_v((0, G.cosd(22), -G.sind(22)))), resolution=2, angle=60)
    return _finish('adirondackChair', centred=True)


# --- the round patio table: a mosaic top on an iron pedestal ---------------------------------------------------
def _mosaic(name, y, radii, counts, pattern, grout=0.003, raised=0.002):
    """A round mosaic at height `y`: rings of tiles between `radii` (the first a disc in the middle), `counts` tiles
    round each ring, each tile set `raised` proud with a sloping edge `grout` wide, so a V of grout shows between
    them (its shading baked dark); `pattern(ring, k)` gives each tile's material. Open underneath: it lies on a top."""
    import bmesh
    bm = bmesh.new()
    palette = []
    for ring, (r0, r1, n) in enumerate(zip(radii, radii[1:], counts)):
        off = math.pi / n * (ring % 2)
        for k in range(n):
            a0, a1 = math.tau * k / n + off, math.tau * (k + 1) / n + off
            if r0 == 0:
                vs = [bm.verts.new(F.at(r1 * math.cos(a0 + (a1 - a0) * i / 2), y, r1 * math.sin(a0 + (a1 - a0) * i / 2)))
                      for i in range(3)] + [bm.verts.new(F.at(0, y, 0))]
            else:
                vs = [bm.verts.new(F.at(r * math.cos(a), y, r * math.sin(a))) for r, a in ((r0, a0), (r1, a0), (r1, a1), (r0, a1))]
            m = pattern(ring, k)
            if m not in palette:
                palette.append(m)
            bm.faces.new(vs).material_index = palette.index(m)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    bmesh.ops.inset_individual(bm, faces=list(bm.faces), thickness=grout, depth=raised, use_even_offset=True)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in palette:
        obj.data.materials.append(m)
    return obj


def patioTableRound():
    def pattern(ring, k):
        if ring == 0:
            return M.tile
        if ring == 1:
            return M.tile_light if k % 2 else M.tile
        if ring == 3:
            return M.tile if k % 3 == 0 else M.mosaic
        if ring == 5:
            return M.tile_light if k % 2 else M.mosaic
        return M.mosaic
    with G.new_parts():
        # the top: a slab under the mosaic, the tiles, an iron band round its edge
        G.revolve('top', [(0.0, 0.676), (0.37, 0.676), (0.392, 0.69), (0.392, 0.719), (0.0, 0.719)], at=F.at(0, 0, 0),
                  material=M.mosaic, segments=24, smooth=False)
        _mosaic('tiles', 0.72, (0.0, 0.07, 0.13, 0.19, 0.25, 0.31, 0.375), (8, 12, 16, 20, 24, 28), pattern, raised=0.003)
        band = [(0.402 * math.cos(math.tau * i / 28), 0.702, 0.402 * math.sin(math.tau * i / 28)) for i in range(28)]
        F.sweep('rim', band, None, M.iron, profile=G.rounded_rect(0.046, 0.012, 0.005, corner=1), up=(0, 0, 1), closed=True)
        # the pedestal: a hub under the top, a turned column with collars, a domed foot on a plinth
        F.cyl('hub', 0.1, 0.03, (0, 0.661, 0), M.iron, verts=16, bevel=0.008, bevel_segments=1)
        G.revolve('column', [(0.0, 0.07), (0.05, 0.07), (0.05, 0.085), (0.034, 0.1), (0.03, 0.16), (0.042, 0.172),
                             (0.042, 0.184), (0.028, 0.196), (0.024, 0.6), (0.036, 0.612), (0.036, 0.63), (0.05, 0.646),
                             (0.0, 0.646)], at=F.at(0, 0, 0), material=M.iron, segments=12)
        G.revolve('foot', [(0, 0), (0.25, 0), (0.25, 0.012), (0.235, 0.022), (0.17, 0.04), (0.09, 0.062), (0.05, 0.075),
                           (0, 0.075)], at=F.at(0, 0, 0), material=M.iron, segments=20)
    return _finish('patioTableRound')


# --- the teak patio table: a planked top framed by breadboard ends, on arched aprons and tapered legs ----------
def _tapered_leg(name, x, z, y0, y1, foot, top, material, verts=16):
    """A square leg, its corners eased, tapering from `top` across at height y1 to `foot` at y0 (the floor)."""
    bx, by, _ = F.at(x, 0, z)
    rings = [(y0, foot), (y0 + 0.006, foot), (y1, top)]
    return G.loft(name, [G.ring(s / 2, s / 2, y, verts, square=7, at=(bx, by)) for y, s in rings], material=material)


def patioTable4():
    zc, y = -0.04, 0.74
    with G.new_parts():
        for k, z in enumerate((-0.36, -0.2, -0.04, 0.12, 0.28)):
            F.box(f'top{k}', 1.5, 0.03, 0.143, (0, y - 0.015, z), M.teak, bevel=0.007)
        for s in (-1, 1):                                  # the breadboard ends across the planks
            F.box(f'end{s}', 0.05, 0.03, 0.785, (s * 0.775, y - 0.015, zc), M.teak, bevel=0.007)
            for z in (-0.3, -0.04, 0.22):                  # the pegs through them
                F.cyl(f'peg{s}{z}', 0.007, 0.003, (s * 0.775, y + 0.0005, z), M.teak_dark, verts=6)
        # the aprons, their lower edges arched between the legs
        for s in (-1, 1):
            arch = [(0.69, 0.03), (-0.69, 0.03)] + [(-0.69 + 1.38 * i / 12, -0.03 + 0.018 * math.sin(math.pi * i / 12))
                                                     for i in range(13)]
            F.extrude(f'apron{s}', arch, 0.026, (0, 0.68, zc + s * 0.33), M.teak_dark, bevel=0.004, bevel_segments=1)
            arch = [(-0.3, 0.03), (0.3, 0.03)] + [(0.3 - 0.6 * i / 8, -0.03 + 0.014 * math.sin(math.pi * i / 8)) for i in range(9)]
            _side_slab(f'apron_end{s}', s * 0.7, [(zc + u, v + 0.68) for u, v in arch], 0.026, M.teak_dark, bevel=0.004)
            for e in (-1, 1):
                _tapered_leg(f'leg{s}{e}', s * 0.72, zc + e * 0.33, 0.0, y - 0.03, 0.044, 0.06, M.teak_dark)
        # an H stretcher: a foot rail at each end, a rail between their middles
        F.box('stretcher', 1.4, 0.04, 0.035, (0, 0.15, zc), M.teak_dark, bevel=0.008)
        for s in (-1, 1):
            F.box(f'foot_rail{s}', 0.035, 0.045, 0.62, (s * 0.72, 0.14, zc), M.teak_dark, bevel=0.008)
    return _finish('patioTable4')


# --- the patio chair: a bent-iron frame, slatted back and a seat cushion ---------------------------------------
def patioChair():
    sz, r = 0.05, 0.0125
    with G.new_parts():
        # the seat: a pan on a tube frame, a piped cushion tied on
        ring = [(u, 0.442, sz + v) for u, v in G.rounded_rect(0.44, 0.4, 0.04, corner=2)]
        F.sweep('seat_frame', ring, 0.011, M.patio_green, verts=6, closed=True)
        F.box('seat', 0.42, 0.012, 0.38, (0, 0.45, sz), M.patio_green, bevel=0.004, bevel_segments=1)
        F.soft('cushion', 0.42, 0.05, 0.38, (0, 0.476, sz + 0.01), M.cushion, round=0.02, puff=0.008, flat=2,
               piping=0.004, round_segments=1)
        # the back: one bent tube from foot to foot over the top; each front leg runs up into an arm and back into it
        _bent('back_frame', [(-0.2, 0.0, sz - 0.22), (-0.19, 0.915, sz - 0.2), (0.19, 0.915, sz - 0.2), (0.2, 0.0, sz - 0.22)],
              r, M.patio_green, bend=0.07, verts=8)
        for s in (-1, 1):
            x = s * 0.2
            _bent(f'arm{s}', [(x, 0.0, sz + 0.204), (x, 0.66, sz + 0.17), (x * 0.985, 0.672, sz - 0.205)], r, M.patio_green,
                  bend=0.05, verts=8)
            for z in (sz + 0.204, sz - 0.22):              # glides under the feet
                F.cyl(f'glide{s}{z}', 0.0125, 0.008, (x, 0.004, z), M.patio_green, verts=8)
        # three back slats, bowed back
        for k, y in enumerate((0.6, 0.72, 0.84)):
            z = sz - 0.207 - (y - 0.6) * 0.02
            bow = [(x, y, z - 0.012 * (1 - (x / 0.19) ** 2)) for x in (-0.19, -0.1, 0.0, 0.1, 0.19)]
            F.sweep(f'back{k}', bow, None, M.patio_green, profile=G.rounded_rect(0.035, 0.01, 0.004, corner=1), up=(0, 0, 1),
                    resolution=2)
        _rod('stretcher', (-0.2, 0.15, sz - 0.21), (0.2, 0.15, sz - 0.21), 0.009, M.patio_green, verts=8)
    return _finish('patioChair')


# --- the garden bench: three seat slats and three back slats on three iron frames ------------------------------
# 1.9 m on its two tiles: the content's three sitters, spread across them, 0.67 m apart.
GARDEN_BENCH_END = 0.92   # its end frames, either side


def benchGarden3():
    with G.new_parts():
        for k, z in enumerate((-0.08, 0.04, 0.16)):
            F.box(f'seat{k}', 1.9, 0.03, 0.1, (0, 0.45, z), M.bench_slats, bevel=0.01)
        backs = [(y, -0.2 - (y - 0.6) * 0.15) for y in (0.6, 0.72, 0.84)]
        for k, (y, z) in enumerate(backs):
            F.box(f'back{k}', 1.9, 0.09, 0.025, (0, y, z), M.bench_slats, rot=(-12, 0, 0), bevel=0.01)
        # the support rises in line with the back slats, just in front of them, and curls back over the top
        back = [(0.0, -0.205), (0.08, -0.17), (0.43, -0.16), (0.8, -0.214), (0.87, -0.234), (0.886, -0.28),
                (0.868, -0.313)]
        for x in (-GARDEN_BENCH_END, 0.0, GARDEN_BENCH_END):
            end = abs(x) > 0
            front = [(0.0, 0.2), (0.44, 0.205), (0.62, 0.235), (0.664, 0.26), (0.664, 0.2), (0.664, -0.21)] if end \
                else [(0.0, 0.19), (0.435, 0.2)]
            _cast_end(f'frame{x}', x, back=back, front=front, rail=(0.42, -0.2, 0.215), stretcher=(0.1, -0.19, 0.196),
                      arm=(0.678, -0.215, 0.28) if end else None, scroll=(0.555, 0.15, 0.05) if end else None)
            _bolts(f'bolt_seat{x}', [(x, 0.467, z) for z in (-0.08, 0.04, 0.16)], M.iron)
            _bolts(f'bolt_back{x}', [(x, y, z + 0.014) for y, z in backs], M.iron, face=(0, 0, 1))
    return _finish('benchGarden3')


# --- the outdoor bar: a slatted teak counter with a back shelf and a foot rail ---------------------------------
def _slab(name, w, d, y_top, t, material, r=0.02, at=(0.0, 0.0), edge=None, corner=3):
    """A top `w` by `d` (corners rounded `r`), its top at `y_top`, `t` thick, its edge moulded: `edge` [(inset,
    fraction of the thickness up), ...] from the bottom (a rounded nose over a chamfer underneath). Lofted in one."""
    edge = edge or [(0.008, 0.0), (0.002, 0.3), (0.0, 0.6), (0.0015, 0.88), (0.005, 1.0)]
    sections = []
    for inset, f in edge:
        loop = G.rounded_rect(w - 2 * inset, d - 2 * inset, max(r - inset, 0.002), corner=corner)
        sections.append([F.at(at[0] + u, y_top - t + f * t, at[1] + v) for u, v in loop])
    return G.loft(name, sections, material=material, angle=50)


def _bottle(name, x, y, z, r, h, material, cap=None, verts=10):
    """A bottle standing at game (x, y, z): a body, a rounded shoulder, a neck, a lip; `cap`: its cap's material."""
    bx, by, bz = F.at(x, y, z)
    rings = [(0.0, 0.92), (0.03, 1.0), (0.6, 1.0), (0.72, 0.8), (0.8, 0.38), (0.94, 0.34), (0.97, 0.4), (1.0, 0.36)]
    G.loft(name, [G.ring(r * k, r * k, bz + h * t, verts, at=(bx, by)) for t, k in rings], material=material)
    if cap:
        F.cyl(f'{name}_cap', r * 0.4, h * 0.05, (x, y + h * 1.02, z), cap, verts=verts)


def outdoorBar():
    zb, zf = -0.37, 0.13                                    # the body's back and front
    with G.new_parts():
        _slab('counter', 2.0, 0.88, 1.06, 0.06, M.bar_top, r=0.025, at=(0, -0.04))
        # the carcass: end panels, a front behind the slats, a middle back with open shelves behind it
        for s in (-1, 1):
            F.box(f'end{s}', 0.03, 0.94, zf - zb, (s * 0.985, 0.53, (zb + zf) / 2), M.teak, bevel=0.006, bevel_segments=1)
            for k in range(4):                              # the end's slats
                F.box(f'end_slat{s}{k}', 0.02, 0.86, 0.1, (s * 0.99, 0.5, zb + 0.07 + 0.12 * k), M.teak, bevel=0.004,
                      bevel_segments=1)
        F.box('front', 1.94, 0.94, 0.02, (0, 0.53, zf - 0.01), M.teak)
        F.box('back', 1.94, 0.94, 0.02, (0, 0.53, -0.12), M.teak_dark)
        for y in (0.07, 0.48):                              # the shelves in the back, and their divider
            F.box(f'shelf{y}', 1.94, 0.025, 0.23, (0, y, -0.245), M.teak, bevel=0.004, bevel_segments=1)
        F.box('divider', 0.025, 0.86, 0.23, (0, 0.5, -0.245), M.teak)
        F.box('plinth', 1.92, 0.06, 0.06, (0, 0.03, zf + 0.0), M.teak_dark)
        for k in range(18):                                 # the front's slats, a gap between each
            F.box(f'slat{k}', 0.096, 0.9, 0.02, (-0.935 + 0.11 * k, 0.51, zf + 0.012), M.teak, bevel=0.004, bevel_segments=1)
        F.box('apron', 2.0, 0.05, 0.03, (0, 0.975, zf + 0.025), M.teak_dark, bevel=0.005, bevel_segments=1)
        # the speed rail under the counter's back edge, bottles and glasses on the shelves
        F.box('rail_shelf', 2.0, 0.04, 0.26, (0, 0.92, -0.36), M.bar_top, bevel=0.006, bevel_segments=1)
        kinds = ((M.bottle_green, 0.034, 0.3), (M.bottle_amber, 0.03, 0.26), (M.bottle, 0.032, 0.28), (M.bottle_clear, 0.03, 0.3))
        for k, x in enumerate((-0.86, -0.78, -0.7, 0.2, 0.3)):
            mat, r, h = kinds[k % 4]
            _bottle(f'bottle{k}', x, 0.4925 if k % 2 else 0.0825, -0.24 + 0.03 * (k % 2), r, h * (0.8 if k % 2 else 1.0),
                    mat, cap=M.steel, verts=8)
        for k, x in enumerate((-0.4, -0.3, 0.65, 0.75)):
            _cup(f'glass{k}', x, 0.4925, -0.25, 0.032, 0.11, M.bottle_clear, handle=False, segments=8)
        # the foot rail: one bent tube out on two brackets, a third in the middle, flanges on the slats
        _bent('foot_rail', [(-0.94, 0.2, zf + 0.02), (-0.94, 0.2, 0.26), (0.94, 0.2, 0.26), (0.94, 0.2, zf + 0.02)], 0.02,
              M.steel, bend=0.06, verts=12)
        _rod('bracket', (0, 0.2, zf + 0.02), (0, 0.2, 0.25), 0.013, M.steel, verts=10)
        for x in (-0.94, 0.0, 0.94):
            G.cylinder(f'flange{x}', 0.035, 0.008, at=F.at(x, 0.2, zf + 0.026), material=M.steel, rot=(90, 0, 0), verts=12)
    return _finish('outdoorBar', centred=True)


# --- the parasol: an octagonal canopy on ribs, on a pole in a weighted base ------------------------------------
CANOPY_Y = 1.97   # the canopy's rim, height
CANOPY_R, CANOPY_H = 1.19, 0.39   # the rib tips' radius, the apex over the rim


def _canopy_point(a, b, j, n, t, drop=0.0, out=0.0):
    """A Blender point on the canopy `t` of the way from its apex to its rim, `j` of `n` steps across the panel between
    the ribs at angles a and b (degrees); the cloth sags between the ribs; `drop` lowers it, `out` pushes it out."""
    r = 0.04 + (CANOPY_R - 0.04) * t + out
    pa = Vector((r * G.cosd(a), r * G.sind(a)))
    pb = Vector((r * G.cosd(b), r * G.sind(b)))
    f = j / n
    p = pa + (pb - pa) * f
    sag = math.sin(math.pi * f)
    p *= 1 - 0.035 * sag * t
    z = CANOPY_Y + CANOPY_H * (1 - t ** 1.3) - 0.045 * sag * t ** 0.8 - drop
    return (p.x, p.y, z)


def _ring_of(t, n, drop=0.0, out=0.0):
    return [_canopy_point(22.5 + 45 * k, 22.5 + 45 * (k + 1), j, n, t, drop, out) for k in range(8) for j in range(n)]


def parasol():
    with G.new_parts() as still:
        G.revolve('base', [(0.0, 0.0), (0.2, 0.0), (0.2, 0.025), (0.185, 0.045), (0.12, 0.06), (0.06, 0.07), (0.0, 0.07)],
                  at=F.at(0, 0, 0), material=M.base, segments=20)
        G.revolve('base_boss', [(0.0, 0.07), (0.045, 0.07), (0.04, 0.12), (0.03, 0.135), (0.0, 0.135)], at=F.at(0, 0, 0),
                  material=M.base, segments=12)
        _rod('base_screw', (0.03, 0.105, 0), (0.09, 0.105, 0), 0.007, M.base, verts=6)
        F.cyl('base_knob', 0.016, 0.03, (0.095, 0.105, 0), M.base, rot=(0, 0, 90), verts=8)
        F.cyl('pole', 0.02, 1.62, (0, 0.93, 0), M.pole, verts=12)
        # the crank's housing on the pole, its handle turning to the front
        F.box('crank', 0.06, 0.1, 0.06, (0, 1.3, 0), M.base, bevel=0.012, bevel_segments=2)
        _bent('crank_arm', [(0, 1.3, 0.03), (0, 1.3, 0.05), (0.07, 1.27, 0.05)], 0.006, M.base, bend=0.012, verts=6)
        F.cyl('crank_knob', 0.01, 0.035, (0.07, 1.27, 0.065), M.base, rot=(90, 0, 0), verts=8)
    G.join(still, 'stand', origin=(0, 0, 0))
    with G.new_parts() as top:
        F.cyl('pole_top', 0.02, 0.7, (0, 2.06, 0), M.pole, verts=12)
        F.cyl('runner', 0.03, 0.07, (0, 1.78, 0), M.pole, verts=12, bevel=0.006, bevel_segments=1)
        # the canopy: eight panels, sagging between their ribs, in one piece with its thickness; the scalloped valance
        ts = (0.0, 0.15, 0.35, 0.55, 0.75, 0.9, 1.0)
        outer = [_ring_of(t, 4) for t in ts]
        inner = [_ring_of(t, 4, drop=0.01) for t in reversed(ts)]
        G.loft('canopy', outer + inner, material=M.canopy, angle=30)
        val = []
        for drop, out in ((0.004, 0.003), (None, 0.003), (None, -0.004), (0.004, -0.004)):
            loop = []
            for k in range(8):
                for j in range(6):
                    f = j / 6
                    d = drop if drop is not None else 0.075 + 0.045 * math.sin(math.pi * f) ** 0.7
                    loop.append(_canopy_point(22.5 + 45 * k, 22.5 + 45 * (k + 1), j, 6, 1.0, d, out))
            val.append(loop)
        G.loft('valance', val + [val[0]], material=M.valance, caps=False, angle=50)
        # the ribs under it, each with a strut from the runner; a hub and a finial over the apex
        for k in range(8):
            a = 22.5 + 45 * k
            rib = [_canopy_point(a, a, 0, 1, t, drop=0.016) for t in (0.04, 0.3, 0.6, 0.85, 0.99)]
            G.sweep(f'rib{k}', rib, 0.006, material=M.pole, verts=5, resolution=2)
            mid = _canopy_point(a, a, 0, 1, 0.48, drop=0.02)
            G.rod(f'strut{k}', (0.03 * G.cosd(a), 0.03 * G.sind(a), 1.8), mid, 0.005, material=M.pole, verts=4)
            G.sphere(f'tip{k}', 0.011, at=_canopy_point(a, a, 0, 1, 1.0, drop=0.012, out=0.004), material=M.pole,
                     segments=6, rings=4)
        F.cyl('hub', 0.035, 0.05, (0, 2.33, 0), M.pole, verts=10)
        G.revolve('finial', [(0.0, 0.0), (0.04, 0.0), (0.042, 0.012), (0.02, 0.03), (0.026, 0.055), (0.018, 0.075),
                             (0.0, 0.08)], at=F.at(0, 2.355, 0), material=M.pole, segments=10)
    canopy = G.hinged(G.join(top, 'canopy'), F.at(0, 1.3, 0))
    G.settle(centre=False)
    # 3°: at 1.2° the rim moved a pixel or two from the game camera, too little to notice
    G.rock(canopy, 'stir', 3, seconds=2.4, axis='X')
    return G.meshes()


# --- OU2: the sun lounger: a slatted platform and cushion; the back reclines flat for sunbathing ---------------
BACK_HINGE = (0.29, -0.36)   # the back's hinge, (height, depth)


def _rounded_side(z0, z1, y0, y1, r):
    """A side outline (depth, height) from z0 to z1, y0 to y1, its two ends rounded by `r`."""
    out = []
    for cz, a0 in ((z1 - r, -90), (z0 + r, 90)):
        for i in range(5):
            a = math.radians(a0 + 180 * i / 4)
            out.append((cz + r * math.cos(a), (y0 + y1) / 2 + (y1 - y0) / 2 * math.sin(a)))
    return out


def sunLounger():
    with G.new_parts() as still:
        for s in (-1, 1):
            # the side rails, rounded at their ends; tapered legs with a cross rail between each pair
            _side_slab(f'rail{s}', s * 0.29, _rounded_side(-0.38, 0.94, 0.22, 0.29, 0.03), 0.04, M.lounger, bevel=0.005)
            for z in (-0.3, 0.86):
                _tapered_leg(f'leg{s}{z}', s * 0.275, z, 0.0, 0.235, 0.04, 0.05, M.lounger, verts=12)
        for z in (-0.3, 0.86):
            F.box(f'leg_rail{z}', 0.5, 0.04, 0.03, (0, 0.1, z), M.lounger, bevel=0.006, bevel_segments=1)
        for k in range(9):
            F.box(f'slat{k}', 0.54, 0.02, 0.11, (0, 0.27, -0.3 + 0.145 * k), M.lounger, bevel=0.006, bevel_segments=1)
        # the cushions: the long one and the seat's end under the back, piped, their middles at the old tops (0.35)
        F.soft('cushion', 0.56, 0.052, 0.95, (0, 0.32, 0.445), M.lounger_pad, round=0.025, puff=0.008, flat=2,
               piping=0.004, round_segments=1)
        F.soft('seat_end', 0.56, 0.052, 0.32, (0, 0.32, -0.198), M.lounger_pad, round=0.025, puff=0.008, flat=1,
               piping=0.004, round_segments=1)
        # a beach towel laid along the cushion and over its foot, and one rolled up
        drape = G.kit.fillet([(0, 0.355, 0.55), (0, 0.356, 0.915), (0, 0.24, 0.93)], 0.02, steps=2)
        F.sweep('towel', drape, None, M.towel, profile=G.rounded_rect(0.4, 0.008, 0.003, corner=1), up=(1, 0, 0))
        roll = [(x, 0.375, 0.5) for x in (-0.2, 0.2)]
        F.sweep('towel_roll', roll, None, M.towel, profile=[(0.02 * math.cos(math.tau * i / 10) * (1 + 0.08 * (i % 2)),
                                                            0.02 * math.sin(math.tau * i / 10)) for i in range(10)],
                up=(0, 0, 1))
    G.join(still, 'lounger', origin=(0, 0, 0))
    y0, z0 = BACK_HINGE
    with G.new_parts() as back:
        a = 35
        along = (0, G.sind(a), -G.cosd(a))       # up the back
        face = (0, G.cosd(a), G.sind(a))
        c = (0, y0 + 0.345 * along[1] + 0.012 * face[1], z0 + 0.345 * along[2] + 0.012 * face[2])
        F.soft('back_pad', 0.56, 0.68, 0.07, c, M.lounger_pad, rot=(-(90 - a), 0, 0), round=0.028, puff=0.01, flat=2,
               piping=0.004, round_segments=1)
        mid = (0, y0 + 0.585 * along[1] + 0.07 * face[1], z0 + 0.585 * along[2] + 0.07 * face[2])
        F.soft('pillow', 0.42, 0.18, 0.08, mid, M.pillow, rot=(-(90 - a), 0, 0), round=0.035, puff=0.02, flat=2,
               round_segments=1)
    back = G.hinged(G.join(back, 'back'), F.at(0, y0, z0))
    G.settle(centre=False)
    G.swing(back, 'recline', -30, seconds=1.0, axis='X')
    return G.meshes()


# --- the hammock: a striped sling between spreader bars on a wooden stand; it sways while lain in -------------
HOOK_Y = 1.1   # the hooks, height: the sling sways about the line through them


def _sling(name, width, half, low, high, t, materials, bands=8, cols=4, curl=0.05):
    """A sagging sling `width` wide from depth -half to +half, `low` in the middle and `high` at the ends, `t` thick,
    in bands of alternating materials; its sides curl up by `curl` where it sags (flat at the spreader bars), so
    the middle line keeps its heights."""
    n = bands * 3
    xs = [-width / 2 + width * i / cols for i in range(cols + 1)]
    verts, faces, mats = [], [], []

    def y_at(x, z):
        return low + (high - low) * (z / half) ** 2 + curl * (2 * x / width) ** 2 * (1 - (z / half) ** 2)
    for i in range(n + 1):
        z = -half + 2 * half * i / n
        for x in xs:
            y = y_at(x, z)
            verts.append(F.at(x, y, z))
            verts.append(F.at(x, y - t, z))
    row = 2 * (cols + 1)
    for i in range(n):
        m = (i // 3) % len(materials)
        a, b = i * row, (i + 1) * row
        for c in range(cols):
            p, q = 2 * c, 2 * (c + 1)
            for f in ((a + p, b + p, b + q, a + q), (a + p + 1, a + q + 1, b + q + 1, b + p + 1)):
                faces.append(f)
                mats.append(m)
        for p in (0, 2 * cols):                            # the two side edges
            faces.append((a + p, a + p + 1, b + p + 1, b + p))
            mats.append(m)
    for a in (0, n * row):                                 # the two ends
        for c in range(cols):
            p, q = 2 * c, 2 * (c + 1)
            faces.append((a + p, a + q, a + q + 1, a + p + 1))
            mats.append(0)
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in materials:
        obj.data.materials.append(m)
    for p, m in zip(obj.data.polygons, mats):
        p.material_index = m
        p.use_smooth = True
    G.shapes._recalc_normals(obj)
    obj.data.update()
    return obj


# A stand hammock on its 1 x 3 tiles: a 2.8 m stand, a 1.96 m bed between the spreader bars, long enough to lie in
# flat (`beds.hammock` `sling`: `low`, `end`, `half`, and the pillow at `head`).
SLING_LOW, SLING_END, SLING_HALF, SLING_HEAD = 0.52, 0.84, 0.98, 0.78
HAMMOCK_HOOK = 1.245  # the hooks on the posts' inner faces, either side of the middle


def hammock():
    with G.new_parts() as stand:
        _side_slab('base', 0, [(-1.4, 0.0), (1.4, 0.0), (1.37, 0.06), (-1.37, 0.06)], 0.1, M.hammock_frame)
        for s in (-1, 1):
            # the feet across, rounded at their ends; the posts leaning in, cut level at the foot; braces
            _plank(f'foot{s}', (-0.5, 0.02, s * 1.36), (0.5, 0.02, s * 1.36), 0.04, M.hammock_frame,
                   _end_round(0.08, 1.0, r=0.03, steps=2, both=True), face=(0, 1, 0), bevel=0.006)
            hw = 0.04 / G.cosd(2.9)
            _side_slab(f'post{s}', 0, [(s * 1.36 - hw, 0.0), (s * 1.36 + hw, 0.0), (s * 1.3 + hw, 1.2), (s * 1.3 - hw, 1.2)],
                       0.08, M.hammock_frame)
            _board(f'brace{s}', (0, 0.06, s * 1.0), (0, 0.5, s * 1.34), 0.05, 0.05, M.hammock_frame, face=(1, 0, 0),
                   bevel=0.008)
            F.cyl(f'cap{s}', 0.045, 0.02, (0, 1.19, s * 1.3), M.hammock_frame, verts=10, bevel=0.006, bevel_segments=1)
            # the hook: an eye screwed into the post and a ring hanging from it
            G.torus(f'hook{s}', 0.018, 0.005, at=F.at(0, HOOK_Y, s * HAMMOCK_HOOK), material=M.hammock_frame,
                    rot=(0, 90, 0), major_segments=10, minor_segments=4)
    G.join(stand, 'stand', origin=(0, 0, 0))
    with G.new_parts() as bed:
        _sling('sling', 0.74, SLING_HALF, SLING_LOW, SLING_END, 0.02, [M.sling, M.sling_stripe], bands=10)
        bar = SLING_HALF + 0.01
        for s in (-1, 1):
            G.revolve(f'spreader{s}', [(0.0, -0.4), (0.012, -0.4), (0.016, -0.39), (0.016, 0.39), (0.012, 0.4), (0.0, 0.4)],
                      at=F.at(0, SLING_END + 0.005, s * bar), material=M.hammock_frame, segments=8, rot=(0, 90, 0))
            # the cords fanning from the bar to a ring under the hook, and the rope up to it
            knot = (0, HOOK_Y - 0.09, s * (HAMMOCK_HOOK - 0.06))
            for x in (-0.36, -0.18, 0.0, 0.18, 0.36):
                F.sweep(f'cord{s}{x}', [(x, SLING_END + 0.01, s * bar), knot], 0.0035, M.sling_stripe, verts=4)
            G.torus(f'ring{s}', 0.02, 0.006, at=F.at(*knot), material=M.sling_stripe, rot=(90, 0, 0), major_segments=10,
                    minor_segments=4)
            F.sweep(f'rope{s}', [knot, (0, HOOK_Y, s * HAMMOCK_HOOK)], 0.006, M.sling_stripe, verts=5)
            # the sling's edges whipped round the bar
            F.sweep(f'hem{s}', [(x, SLING_END - 0.002, s * (bar - 0.012)) for x in (-0.37, 0.37)], 0.012, M.sling, verts=6)
        head = SLING_LOW + (SLING_END - SLING_LOW) * (SLING_HEAD / SLING_HALF) ** 2
        F.soft('pillow', 0.4, 0.08, 0.26, (0, head + 0.03, -SLING_HEAD), M.pillow, rot=(12, 0, 0), round=0.035, puff=0.02,
               flat=2, round_segments=1)
    bed = G.hinged(G.join(bed, 'sling'), F.at(0, HOOK_Y, 0))
    G.settle(centre=False)
    G.rock(bed, 'sway', 8, seconds=2.0, axis='Y')
    return G.meshes()


# --- the porch swing: a slatted swing seat hung on chains from an A-frame; it swings while sat in --------------
def _a_frame(prefix, beam_y, material, round_legs=False, spread=0.42):
    """Two A-frame ends at x ±0.92, their legs `spread` m out at the foot, and the beam across them: square legs cut
    level at the foot and meeting under the beam, or round timbers tapering up; ties across; a beam with its ends
    cut back underneath."""
    tie = spread * (1 - 0.7 / beam_y)
    for s in (-1, 1):
        x = s * 0.92
        for e in (-1, 1):
            if round_legs:
                F.sweep(f'{prefix}leg{s}{e}', [(x, 0.0, e * spread), (x, beam_y * 0.5, e * spread * 0.5),
                                               (x, beam_y - 0.02, e * 0.02)], 0.04, material, verts=10,
                        scale=[1.0, 0.93, 0.86])
            else:
                hw = 0.04 * math.hypot(spread, beam_y) / beam_y
                _side_slab(f'{prefix}leg{s}{e}', x, [(e * spread - hw, 0.0), (e * spread + hw, 0.0), (e * hw * 0.2, beam_y),
                                                     (-e * hw * 1.8, beam_y)] if e > 0 else
                           [(e * spread - hw, 0.0), (e * spread + hw, 0.0), (hw * 1.8, beam_y), (-hw * 0.2, beam_y)],
                           0.08, material, bevel=0.006)
        _board(f'{prefix}tie{s}', (x, 0.7, -tie - 0.03), (x, 0.7, tie + 0.03), 0.06, 0.04, material, bevel=0.008)
        if not round_legs:                                 # a gusset plate where the legs meet the beam
            _side_slab(f'{prefix}gusset{s}', x - s * 0.046, [(-0.14, beam_y - 0.05), (0.14, beam_y - 0.05), (0.06, beam_y - 0.2),
                                                             (-0.06, beam_y - 0.2)], 0.012, material, bevel=0.003)
    if round_legs:
        F.sweep(f'{prefix}beam', [(-0.96, beam_y, 0), (0.96, beam_y, 0)], 0.045, material, verts=14)
        for s in (-1, 1):
            F.cyl(f'{prefix}beam_end{s}', 0.047, 0.012, (s * 0.955, beam_y, 0), material, rot=(0, 90, 0), verts=14)
    else:
        F.extrude(f'{prefix}beam', [(-0.96, 0.05), (0.96, 0.05), (0.96, 0.0), (0.92, -0.05), (-0.92, -0.05), (-0.96, 0.0)],
                  0.1, (0, beam_y, 0), material, bevel=0.008, bevel_segments=1)


# The seat hung 5 cm lower than it was: the cushion's top at 0.515 m (content `seat` 0.50, sunk in it).
PORCH_DROP = 0.05


def porchSwing():
    with G.new_parts() as frame:
        _a_frame('', 2.0, M.porch_frame)
    G.join(frame, 'frame', origin=(0, 0, 0))
    d = PORCH_DROP
    with G.new_parts() as seat:
        for k in range(4):
            F.box(f'seat{k}', 1.3, 0.03, 0.105, (0, 0.48 - d, -0.13 + 0.115 * k), M.porch_seat, bevel=0.008, bevel_segments=1)
        for k in range(4):
            F.box(f'back{k}', 1.3, 0.08, 0.025, (0, 0.6 - d + 0.1 * k, -0.19 - 0.021 * k), M.porch_seat,
                  rot=(-12, 0, 0), bevel=0.008, bevel_segments=1)
        # the back's top rail, its top edge arched; the seat's front rail
        arch = [(-0.7, -0.03), (0.7, -0.03)] + [(0.7 - 1.4 * i / 10, 0.03 + 0.025 * math.sin(math.pi * i / 10)) for i in range(11)]
        F.extrude('top_rail', arch, 0.03, (0, 0.98 - d, -0.275), M.porch_seat, rot=(-12, 0, 0), bevel=0.006, bevel_segments=1)
        F.box('front_rail', 1.4, 0.06, 0.03, (0, 0.44 - d, 0.235), M.porch_seat, bevel=0.008, bevel_segments=1)
        for s in (-1, 1):
            _arm_board(f'arm{s}', s * 0.68, 0.7 - d, -0.21, 0.25, 0.06, 0.08, M.porch_seat, t=0.03)
            G.revolve(f'arm_post{s}', [(0.0, 0.0), (0.024, 0.0), (0.024, 0.03), (0.017, 0.05), (0.02, 0.12), (0.014, 0.18),
                                       (0.022, 0.2), (0.022, 0.22), (0.0, 0.22)], at=F.at(s * 0.68, 0.47 - d, 0.21),
                      material=M.porch_seat, segments=10)
            F.box(f'side{s}', 0.05, 0.06, 0.48, (s * 0.68, 0.45 - d, 0.03), M.porch_seat, bevel=0.008, bevel_segments=1)
            # two chains from the seat's corners to a ring, one chain up to the hook under the beam
            ring = (s * 0.67, 1.42, 0.02)
            for z in (-0.18, 0.24):
                F.sweep(f'chain{s}{z}', [(s * 0.66, 0.5 - d, z), ring], 0.006, M.chain, verts=5)
                F.cyl(f'eye{s}{z}', 0.012, 0.03, (s * 0.66, 0.5 - d, z), M.chain, verts=6)
            G.torus(f'ring{s}', 0.022, 0.006, at=F.at(*ring), material=M.chain, rot=(0, 90, 0), major_segments=10,
                    minor_segments=4)
            F.sweep(f'chain_top{s}', [ring, (s * 0.68, 1.93, 0.0)], 0.006, M.chain, verts=5)
            G.torus(f'hook{s}', 0.02, 0.005, at=F.at(s * 0.68, 1.95, 0.0), material=M.chain, rot=(0, 90, 0),
                    major_segments=10, minor_segments=4)
        F.soft('cushion', 1.28, 0.065, 0.42, (0, 0.5225 - d, 0.05), M.porch_cushion, round=0.03, puff=0.01, flat=2,
               piping=0.005, round_segments=1)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 1.98, 0))
    G.settle(centre=False)
    G.rock(seat, 'swing', 10, seconds=2.0, axis='X')
    return G.meshes()


# --- the swing set: two seats on chains from a round-timber A-frame; they swing while played on ---------------
def swingSet():
    with G.new_parts() as frame:
        _a_frame('', 1.92, M.swing_frame, round_legs=True, spread=0.72)   # splayed for seats swinging 20°
    G.join(frame, 'frame', origin=(0, 0, 0))
    with G.new_parts() as seats:
        for x in (-0.48, 0.48):
            # a belt seat: a strap sagging between its chains, edges rolled
            belt = [(x + u, 0.48 + 0.035 * (u / 0.2) ** 2, 0) for u in (-0.21, -0.14, -0.07, 0.0, 0.07, 0.14, 0.21)]
            F.sweep(f'seat{x}', belt, None, M.swing_seat, profile=G.rounded_rect(0.012, 0.2, 0.005, corner=1),
                    up=(0, 0, 1), resolution=2)
            for dx in (-0.2, 0.2):
                F.sweep(f'chain{x}{dx}', [(x + dx, 0.53, 0), (x + dx, 1.86, 0)], 0.006, M.chain, verts=5)
                F.box(f'clip{x}{dx}', 0.03, 0.04, 0.03, (x + dx, 0.515, 0), M.chain, bevel=0.008, bevel_segments=1)
                # the hanger round the beam: a ring the chain hangs from (it turns on its own axis as it swings)
                G.torus(f'hanger{x}{dx}', 0.052, 0.008, at=F.at(x + dx, 1.92, 0), material=M.chain, rot=(0, 90, 0),
                        major_segments=12, minor_segments=4)
    seats = G.hinged(G.join(seats, 'seats'), F.at(0, 1.92, 0))
    G.settle(centre=False)
    G.rock(seats, 'swing', 20, seconds=1.6, axis='X')
    return G.meshes()


# --- the cedar hot tub: banded staves, a step, water; the cover folds back for a soak and the water bubbles ----
TUB_R = 0.92
TUB_SEAT, TUB_BENCH = 0.35, 0.44   # the bench's top (`seats.hotTubCedar`) and its inner edge's radius


def hotTubCedar():
    with G.new_parts() as tub:
        # the staves: one wall, a V between each of 32, the inside round; a floor; the rim round their tops
        out = []
        n = 32
        for k in range(n):
            for f, r in ((0.0, TUB_R - 0.006), (0.12, TUB_R), (0.88, TUB_R)):
                a = math.tau * (k + f) / n
                out.append((r * math.cos(a), r * math.sin(a)))
        hole = [((TUB_R - 0.06) * math.cos(math.tau * k / n), (TUB_R - 0.06) * math.sin(math.tau * k / n)) for k in range(n)]
        F.extrude('staves', out, 0.78, (0, 0.41, 0), M.cedar, plane='top', holes=[hole])
        G.revolve('floor', [(0.0, 0.0), (TUB_R - 0.05, 0.0), (TUB_R - 0.05, 0.12), (0.0, 0.12)], at=F.at(0, 0, 0),
                  material=M.cedar, segments=24, smooth=False)
        G.revolve('rim', [(TUB_R - 0.075, 0.79), (TUB_R + 0.005, 0.79), (TUB_R + 0.01, 0.81), (TUB_R + 0.002, 0.826),
                          (TUB_R - 0.07, 0.826), (TUB_R - 0.078, 0.81)], at=F.at(0, 0, 0), material=M.cedar, segments=32)
        for y in (0.2, 0.6):                               # the bands, a lug where each is drawn tight
            band = [((TUB_R + 0.004) * math.cos(math.tau * k / 32), y, (TUB_R + 0.004) * math.sin(math.tau * k / 32))
                    for k in range(32)]
            F.sweep(f'band{y}', band, None, M.cedar_band, profile=G.rounded_rect(0.04, 0.008, 0.003, corner=1), up=(0, 1, 0),
                    closed=True)
            F.box(f'lug{y}', 0.05, 0.05, 0.03, (0.62, y, 0.68), M.cedar_band, rot=(0, 0, 0), bevel=0.006, bevel_segments=1)
        # the bench round the inside wall, under the water: the soakers' seat (`seats.hotTubCedar`)
        G.revolve('bench', [(TUB_BENCH, 0.12), (TUB_BENCH, TUB_SEAT - 0.01), (TUB_BENCH + 0.01, TUB_SEAT),
                            (TUB_R - 0.06, TUB_SEAT)], at=F.at(0, 0, 0), material=M.cedar, segments=32, smooth=False)
        # the step: a tread with a dark nosing on two shaped sides and a riser
        F.box('tread', 0.6, 0.03, 0.29, (0, 0.185, 0.81), M.cedar, bevel=0.006, bevel_segments=1)
        F.box('tread_edge', 0.6, 0.022, 0.03, (0, 0.188, 0.944), M.cedar_band, bevel=0.005, bevel_segments=1)
        F.box('riser', 0.54, 0.14, 0.02, (0, 0.09, 0.93), M.cedar, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            _side_slab(f'step_side{s}', s * 0.285, [(0.66, 0.0), (0.955, 0.0), (0.955, 0.17), (0.66, 0.17)], 0.03, M.cedar,
                       bevel=0.004)
    G.join(tub, 'tub', origin=(0, 0, 0))
    with G.new_parts() as water:
        G.revolve('water', [(0.0, 0.69), (0.3, 0.688), (0.6, 0.692), (TUB_R - 0.06, 0.69), (TUB_R - 0.06, 0.67), (0.0, 0.67)],
                  at=F.at(0, 0, 0), material=M.water, segments=32)
        for k in range(14):
            a = 360 / 14 * k + (k % 3) * 9
            r = 0.2 + 0.5 * ((k * 7) % 5) / 5
            F.sphere(f'bubble{k}', 0.025 + 0.01 * (k % 3), (r * G.cosd(a), 0.69, r * G.sind(a)), M.foam,
                     scale=(1, 1, 0.4), segments=8, rings=4)
        for k in range(4):                                 # the jets' foam against the wall
            a = 90 * k + 30
            F.sphere(f'jet{k}', 0.06, ((TUB_R - 0.11) * G.cosd(a), 0.69, (TUB_R - 0.11) * G.sind(a)), M.foam,
                     scale=(1, 0.7, 0.3), segments=8, rings=4)
    water = G.hinged(G.join(water, 'water'), F.at(0, 0.68, 0))
    # The cover in two halves meeting at the seam across its middle (a fourth draw, as the shower's door, Q-05): one
    # lofted disc, each half cut from a copy of it, a hair apart at the seam, where a vinyl hinge strip joins them.
    rings = [(TUB_R + 0.0, 0.84), (TUB_R + 0.02, 0.85), (TUB_R + 0.02, 0.895), (TUB_R + 0.005, 0.91), (TUB_R - 0.03, 0.912)]

    def half(name, s):
        """The half of the cover on side `s` of the seam (1: the front, game +z; -1: the back), cut from the disc."""
        disc = G.loft(name, [G.ring(r, r, y, 32) for r, y in rings], material=M.tub_cover, angle=30)
        cutter = F.box(f'{name}_cutter', 2.2, 0.2, 1.2, (0, 0.876, -s * 0.597), M.tub_cover)
        return G.cut(disc, cutter)

    with G.new_parts() as back:
        half('cover', -1)
        F.box('cover_seam', 1.84, 0.008, 0.02, (0, 0.912, -0.003), M.tub_cover, bevel=0.003, bevel_segments=1)
    with G.new_parts() as front:
        half('cover_front', 1)
        for x in (-0.3, 0.3):                              # the straps down to their clips, and the handles
            F.box(f'strap{x}', 0.05, 0.06, 0.008, (x, 0.865, TUB_R + 0.024), M.tub_cover, bevel=0.002, bevel_segments=1)
            F.box(f'clip{x}', 0.04, 0.025, 0.012, (x, 0.83, TUB_R + 0.028), M.tub_cover,
                  bevel=0.003, bevel_segments=1)
            F.sweep(f'handle{x}', [(x - 0.05, 0.91, 0.6), (x - 0.03, 0.919, 0.6), (x + 0.03, 0.919, 0.6), (x + 0.05, 0.91, 0.6)],
                    0.006, M.tub_cover, verts=5, resolution=2)
    # the back half's origin on its back edge, at the bottom; the front half's on the seam, at the top, about which
    # it folds over onto the back half
    cover = G.hinged(G.join(back, 'cover'), F.at(0, 0.84, -TUB_R - 0.02))
    flap = G.hinged(G.join(front, 'cover_front'), F.at(0, 0.912, 0))
    G.settle(centre=False)
    G.attach(flap, cover)
    # Off for a soak: the front half folds back over the back half on the seam, then the two together are lifted,
    # turned down behind the tub and lowered to hang there on their folded edge, as on a cover lifter, the round
    # edge just off the ground (Blender offsets: +y is back).
    G.swing(flap, 'cover_off', -180, seconds=0.5, axis='X')
    G.path(cover, 'cover_off', [(0, (0, 0, 0), (0, 0, 0)), (0.5, (0, 0, 0), (0, 0, 0)), (0.8, (0, 0, 0.25), (0, 0, 0)),
                                (1.1, (0, 0.16, 1.16), (90, 0, 0)), (1.5, (0, 0.16, 0.11), (90, 0, 0))])
    G.motion._peak(cover, 'cover_off', G.motion.last_frame(1.5))   # previewed hung up, at its end
    G.spin(water, 'bubbles', turns=1, seconds=2.0)
    return G.meshes()


# --- OU3: the grills and the fire pit -------------------------------------------------------------------------
def _across(name, outline, x0, x1, material):
    """A convex outline of game (depth, height) points, run across from game x0 to x1: a hood's barrel."""
    n = len(outline)
    verts = [F.at(x0, y, z) for z, y in outline] + [F.at(x1, y, z) for z, y in outline]
    faces = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))] + [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    G.shapes._recalc_normals(obj)
    return obj


def _embers(prefix, r, y, n, seed, material):
    """`n` glowing lumps of coal scattered over a disc of radius `r` at height `y`."""
    for k in range(n):
        a = (k * 137.5 + seed) % 360
        d = r * math.sqrt((k + 0.5) / n)
        F.sphere(f'{prefix}{k}', 0.022 + 0.008 * (k % 3), (d * G.cosd(a), y, d * G.sind(a)), material,
                 scale=(1, 1, 0.6), segments=6, rings=4)


def _burger(name, x, y, z, r, material):
    """A patty lying at game (x, y = its underside, z): a rounded disc, its edge a little ragged."""
    G.revolve(name, [(0.0, 0.0), (r * 0.92, 0.0), (r, 0.006), (r * 0.98, 0.016), (r * 0.85, 0.022), (0.0, 0.023)],
              at=F.at(x, y, z), material=material, segments=10)


def _sausage(name, a, b, r, material):
    """A sausage from game point a to b, rounded at both ends, a little bent."""
    mid = tuple((p + q) / 2 + (0.006 if i == 2 else 0.0) for i, (p, q) in enumerate(zip(a, b)))
    F.sweep(name, [a, mid, b], r, material, verts=6, resolution=3, scale=lambda t: max(0.35, min(1.0, 4 * t, 4 * (1 - t))))


KETTLE_R, KETTLE_RIM = 0.3, 0.79     # the charcoal grill's bowl: its radius and its rim's height
UP = KETTLE_RIM - 0.99               # its parts' heights, drawn for a rim at 0.99 m, moved down to it


def _h(y):
    return y + UP


# The charcoal grill: a kettle on three legs, wheels on the back two, an ash catcher under the bowl; inside, a bed of
# red coals (aglow at night) and a grate with burgers and sausages, at a 57 cm kettle's working height. The lid is
# hinged at the back of the rim and swings up (`lid_open`) for the whole of a cook.
def grill():
    with G.new_parts() as still:
        y0, top = _h(0.64), KETTLE_RIM
        G.revolve('bowl', [(0, y0), (0.15, y0 + 0.005), (0.24, y0 + 0.04), (0.29, _h(0.76)), (KETTLE_R, _h(0.86)),
                           (KETTLE_R, top - 0.006), (KETTLE_R + 0.006, top - 0.002), (KETTLE_R + 0.004, top + 0.003),
                           (KETTLE_R - 0.012, top + 0.003), (KETTLE_R - 0.014, _h(0.86)), (0.276, _h(0.77)),
                           (0.226, y0 + 0.055), (0.14, y0 + 0.02), (0, y0 + 0.02)],
                  at=F.at(0, 0, 0), material=M.kettle, segments=32)
        for k, a in enumerate((90, 210, 330)):              # the bowl's dampers, under it
            c, s_ = G.cosd(a + 60), G.sind(a + 60)
            F.cyl(f'damper{k}', 0.035, 0.006, (0.17 * c, y0 + 0.01, 0.17 * s_), M.grate, verts=10)
        for s in (-1, 1):                                   # the bowl's side handles, on their brackets
            _bent(f'side_handle{s}', [(s * (KETTLE_R - 0.005), _h(0.93), -0.06), (s * (KETTLE_R + 0.014), _h(0.93), -0.05),
                                      (s * (KETTLE_R + 0.014), _h(0.93), 0.05), (s * (KETTLE_R - 0.005), _h(0.93), 0.06)],
                  0.007, M.grate, bend=0.012, verts=6)
        # the coals, glowing, under the cooking grate (a ring and its bars) with food
        F.cyl('coal_bed', 0.24, 0.03, (0, _h(0.85), 0), M.coals, verts=16)
        _embers('coal', 0.21, _h(0.875), 14, 20, M.coal_hot)
        ring = [((KETTLE_R - 0.025) * math.cos(math.tau * i / 24), _h(0.965), (KETTLE_R - 0.025) * math.sin(math.tau * i / 24))
                for i in range(24)]
        F.sweep('grate_ring', ring, 0.006, M.grate, verts=5, closed=True)
        for k in range(-5, 6):
            x = k * 0.05
            half = math.sqrt(max((KETTLE_R - 0.025) ** 2 - x * x, 0))
            _rod(f'grate_bar{k}', (x, _h(0.965), -half), (x, _h(0.965), half), 0.004, M.grate, verts=5)
        for k, (x, z) in enumerate(((-0.12, 0.06), (0.02, -0.12), (0.13, 0.08))):
            _burger(f'burger{k}', x, _h(0.97), z, 0.05, M.burger)
        for k, (x, z) in enumerate(((-0.06, -0.09), (0.06, -0.02))):
            _sausage(f'sausage{k}', (x - 0.07, _h(0.985), z), (x + 0.07, _h(0.985), z + 0.02), 0.016, M.sausage)
        # the legs into sockets under the bowl, a wire rack between them, the ash pan, the wheels on their axle
        feet = []
        for k, a in enumerate((90, 210, 330)):
            c, s_ = G.cosd(a), G.sind(a)
            foot_y = 0.075 if a != 90 else 0.0
            F.box(f'socket{k}', 0.04, 0.05, 0.04, (c * 0.13, _h(0.665), s_ * 0.13), M.grate, rot=(0, 0, a), bevel=0.006,
                  bevel_segments=1)
            _rod(f'leg{k}', (c * 0.13, _h(0.665), s_ * 0.13), (c * 0.31, foot_y, s_ * 0.31), 0.014, M.grate, verts=8)
            t = (0.2 - foot_y) / (_h(0.665) - foot_y)
            feet.append((c * (0.31 + (0.13 - 0.31) * t), 0.2, s_ * (0.31 + (0.13 - 0.31) * t)))
        F.sweep('rack', feet, 0.006, M.grate, verts=5, closed=True)
        G.revolve('ash_catcher', [(0.0, 0.0), (0.11, 0.0), (0.125, 0.04), (0.13, 0.045), (0.115, 0.045), (0.105, 0.008),
                                  (0.0, 0.008)], at=F.at(0, 0.29, 0), material=M.grate, segments=16)
        F.cyl('foot_cap', 0.02, 0.02, (0, 0.01, 0.31), M.wheel, verts=10)
        _rod('axle', (-0.3, 0.075, -0.155), (0.3, 0.075, -0.155), 0.008, M.grate, verts=6)
        for s in (-1, 1):                                   # a tyre, a hub cap
            G.revolve(f'wheel{s}', [(0.0, -0.015), (0.06, -0.015), (0.072, -0.012), (0.075, 0.0), (0.072, 0.012), (0.06, 0.015),
                                    (0.03, 0.017), (0.0, 0.017)], at=F.at(s * 0.29, 0.075, -0.155), material=M.wheel,
                      segments=16, rot=(0, -90 * s, 0))
            F.cyl(f'hub{s}', 0.025, 0.006, (s * 0.307, 0.075, -0.155), M.grate, rot=(0, 90, 0), verts=10)
    G.join(still, 'kettle', origin=(0, 0, 0))
    with G.new_parts() as parts:
        G.revolve('lid', [(0, _h(1.215)), (0.16, _h(1.19)), (0.25, _h(1.125)), (KETTLE_R - 0.012, _h(1.04)),
                          (KETTLE_R - 0.012, KETTLE_RIM + 0.004), (KETTLE_R + 0.004, KETTLE_RIM + 0.004),
                          (KETTLE_R + 0.004, _h(1.045)), (0.26, _h(1.135)), (0.165, _h(1.205)), (0, _h(1.23))],
                  at=F.at(0, 0, 0), material=M.kettle_lid, segments=32)
        # the vent on top with its damper's tab; the thermometer on the front; the handle on two brackets under a shield
        F.cyl('vent', 0.045, 0.008, (0, _h(1.232), 0), M.grate, verts=12)
        F.box('vent_tab', 0.05, 0.006, 0.015, (0.05, _h(1.236), 0), M.grate)
        G.cylinder('gauge_bezel', 0.024, 0.01, at=F.at(0, _h(1.172), 0.19), material=M.handle_steel, rot=(50, 0, 0),
                   verts=12)
        G.cylinder('gauge', 0.019, 0.004, at=F.at(0, _h(1.176), 0.194), material=M.gauge, rot=(50, 0, 0), verts=12)
        for s in (-1, 1):
            _rod(f'handle_post{s}', (s * 0.06, _h(1.12), 0.24), (s * 0.06, _h(1.165), 0.275), 0.007, M.grate, verts=6)
        F.box('handle_shield', 0.17, 0.004, 0.05, (0, _h(1.152), 0.255), M.grate, rot=(-35, 0, 0))
        _rod('handle', (-0.08, _h(1.165), 0.277), (0.08, _h(1.165), 0.277), 0.013, M.handle_steel, verts=10)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, KETTLE_RIM, -KETTLE_R))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -95, seconds=0.9, axis='X')
    return G.meshes()


# The gas grill station: a stainless cart on a plinth, two doors with bar handles and a control panel's knobs below a
# dark counter (its top at 0.9, the content's `surface`, which stands things on the right); a chopping board with a
# tray of food on the left, a bottle and a pepper mill at the back right. The grill in the middle (food on its
# grate) has a barrel hood hinged at the back that swings up (`hood_up`) for cook and feast.
HOOD = (-0.39, 0.11, 0.905, 1.0)      # the hood's back and front depth, its bottom and where its barrel starts


def grillStation():
    back, front, bottom, spring = HOOD
    with G.new_parts() as still:
        F.box('plinth', 1.52, 0.06, 0.5, (0, 0.03, -0.16), M.firebox, bevel=0.005, bevel_segments=1)
        F.box('cart', 1.6, 0.8, 0.58, (0, 0.46, -0.15), M.cart, bevel=0.01, bevel_segments=2)
        _slab('top', 1.66, 0.66, 0.9, 0.04, M.worktop, r=0.02, at=(0, -0.125))
        # the control panel, tilted back, its knobs and an igniter
        F.box('panel', 0.96, 0.1, 0.014, (0, 0.8, 0.147), M.firebox, rot=(-12, 0, 0), bevel=0.003, bevel_segments=1)
        for k, x in enumerate((-0.3, -0.1, 0.1, 0.3)):
            G.kit.knob(F, f'knob{k}', x, 0.8, 0.154, 0.018, M.handle_steel, segments=10)
        F.cyl('igniter', 0.012, 0.012, (0.42, 0.8, 0.158), M.burger, rot=(90, 0, 0), verts=8)
        # two stainless doors, framed, with bar pulls; the hinges' knuckles at their outer edges
        for s in (-1, 1):
            G.kit.shaker(F, f'door{s}', 0.74, 0.6, (s * 0.39, 0.4, 0.147), 0.014, M.cart, rail=0.05, recess=0.003)
            G.kit.pull(F, f'door_handle{s}', s * 0.39, 0.64, 0.154, 0.44, M.handle_steel, r=0.008, out=0.026)
            for y in (0.17, 0.63):
                F.cyl(f'knuckle{s}{y}', 0.006, 0.05, (s * 0.765, y, 0.15), M.handle_steel, verts=6)
        # a towel bar along the right end
        _bent('towel_bar', [(0.79, 0.72, -0.35), (0.818, 0.72, -0.35), (0.818, 0.72, 0.05), (0.79, 0.72, 0.05)], 0.008,
              M.handle_steel, bend=0.015, verts=6)
        # the grill: a dark firebox let into the counter, its grate with food on it, a warming rack at the back
        F.box('firebox', 0.86, 0.012, 0.46, (0, 0.898, -0.14), M.firebox)
        for k in range(-8, 9):
            _rod(f'grate_bar{k}', (k * 0.05, 0.909, -0.36), (k * 0.05, 0.909, 0.08), 0.005, M.grate, verts=5)
        for z in (-0.36, 0.08):
            _rod(f'grate_frame{z}', (-0.42, 0.909, z), (0.42, 0.909, z), 0.006, M.grate, verts=5)
        for k, (x, z) in enumerate(((-0.25, -0.05), (-0.1, -0.2), (0.06, -0.06))):
            _burger(f'burger{k}', x, 0.913, z, 0.05, M.burger)
        for k, z in enumerate((-0.25, -0.18)):
            _sausage(f'sausage{k}', (0.12, 0.927, z), (0.3, 0.927, z + 0.02), 0.017, M.sausage)
        F.sweep('corn', [(0.2, 0.93, 0.0), (0.36, 0.93, -0.04)], 0.024, M.corn, verts=8, resolution=1,
                scale=[1.0, 0.75])
        # on the counter: a board, a tray of patties, a bottle and a pepper mill
        F.box('board', 0.34, 0.02, 0.36, (-0.62, 0.91, -0.12), M.board, bevel=0.006, bevel_segments=1)
        F.box('tray', 0.24, 0.025, 0.16, (-0.62, 0.9325, -0.12), M.tray, bevel=0.008, bevel_segments=1)
        for k, x in enumerate((-0.68, -0.6)):
            _burger(f'patty{k}', x, 0.945, -0.12, 0.035, M.burger)
        _bottle('bottle', 0.72, 0.9, -0.33, 0.035, 0.24, M.bottle, cap=M.handle_steel, verts=10)
        G.revolve('pepper_mill', [(0.0, 0.0), (0.022, 0.0), (0.024, 0.02), (0.016, 0.06), (0.02, 0.1), (0.022, 0.12),
                                  (0.012, 0.14), (0.004, 0.15), (0.0, 0.15)], at=F.at(0.62, 0.9, -0.36), material=M.board,
                  segments=10)
    G.join(still, 'cart', origin=(0, 0, 0))
    with G.new_parts() as parts:
        # the barrel: an ellipse over its upright front and back, dark end caps, a bar handle, a thermometer
        mid, rz, ry = (back + front) / 2, (front - back) / 2, 0.2
        arc = [(mid + rz * math.cos(math.radians(a)), spring + ry * math.sin(math.radians(a))) for a in range(0, 181, 12)]
        outline = [(front, bottom)] + arc + [(back, bottom)]
        _across('hood', outline, -0.45, 0.45, M.hood)
        for s in (-1, 1):
            _across(f'hood_end{s}', [(z + (0.006 if z > mid else -0.006), y + 0.006) for z, y in outline[1:-1]] +
                    [(back - 0.006, bottom), (front + 0.006, bottom)], s * 0.45, s * 0.462, M.firebox)
        F.box('hood_inside', 0.86, 0.004, rz * 2 - 0.04, (0, bottom - 0.002, mid), M.firebox)
        G.kit.pull(F, 'hood_handle', 0, 1.0, front, 0.68, M.handle_steel, r=0.012, out=0.055)
        G.cylinder('gauge_bezel', 0.038, 0.016, at=F.at(0, 1.1, front + 0.035), material=M.handle_steel, rot=(70, 0, 0),
                   verts=16)
        G.cylinder('gauge', 0.03, 0.004, at=F.at(0, 1.103, front + 0.043), material=M.gauge, rot=(70, 0, 0), verts=16)
    hood = G.hinged(G.join(parts, 'hood'), F.at(0, spring, back))
    G.settle(centre=False)
    G.swing(hood, 'hood_up', -75, seconds=0.9, axis='X')
    return G.meshes()


# The fire pit: a stone bowl with a ring of capstones on its rim, an ash bed with embers that glow at night and a
# teepee of logs. The flames grow on the logs (`flames`) while a resident warms up by it; the light is the
# content's (`light`, at 0.5 m).
PIT_R, PIT_TOP = 0.43, 0.3


def _log(name, a, b, r, material, end, seed):
    """A split log from game point a to b: rough bark, its sawn ends pale."""
    pts = [a, tuple((p + q) / 2 for p, q in zip(a, b)), b]
    _jitter(F.sweep(name, pts, r, material, verts=7, scale=[1.0, 0.95, 0.9]), 0.004, size=0.06, seed=seed)
    d = (_v(b) - _v(a)).normalized()
    for k, (p, rr) in enumerate(((a, r), (b, r * 0.9))):
        cap = G.cylinder(f'{name}_end{k}', rr * 0.85, 0.004, at=tuple(_v(p) + d * (0.001 if k else -0.001)), material=end,
                         verts=7)
        cap.rotation_mode = 'QUATERNION'
        cap.rotation_quaternion = d.to_track_quat('Z', 'Y')


def firePit():
    with G.new_parts() as still:
        # the bowl: two courses of stones round a dark wall, the ash bed inside
        G.revolve('bowl', [(0.0, 0.0), (0.36, 0.0), (0.36, PIT_TOP - 0.02), (0.31, PIT_TOP - 0.02), (0.31, 0.2), (0.0, 0.2)],
                  at=F.at(0, 0, 0), material=M.coals, segments=20, smooth=False)
        for row, (y, h, n, off) in enumerate(((0.07, 0.14, 11, 0.0), (0.21, 0.13, 11, 0.5))):
            for k in range(n):
                a = 360 * (k + off) / n
                w = math.tau * 0.39 / n - 0.012
                st = F.box(f'stone{row}_{k}', w, h - 0.008, 0.1, (0.39 * G.cosd(a), y, 0.39 * G.sind(a)), M.pit,
                           rot=(0, 0, a - 90), bevel=0.018, bevel_segments=1)
                _jitter(st, 0.008, size=0.07, seed=row * 11 + k)
        F.cyl('ash', 0.315, 0.1, (0, 0.255, 0), M.coals, verts=20)
        _embers('ember', 0.2, 0.31, 12, 40, M.ember)
        # the capstones: ten wedges round the rim, each a little askew (three pushed out a little: the old one's bounds)
        for i in range(10):
            a = i * 36 + (7 if i % 2 else -4)
            o = {7: 0.021, 0: 0.014, 5: 0.017}.get(i, 0.0)
            wedge = [(0.31 + o, -0.085), (0.435 + o, -0.112), (0.435 + o, 0.112), (0.31 + o, 0.085)]
            cap = F.extrude(f'cap{i}', wedge, 0.07, (0, 0.335, 0), M.pit_stones, plane='top', bevel=0.022, bevel_segments=1,
                            rot=(0, 0, a + (i % 3 - 1) * 5))
            _jitter(cap, 0.006, size=0.08, seed=50 + i, up=0.4)
        # the teepee of logs, and one across under it
        for k, a in enumerate((20, 110, 200, 290)):
            c, s_ = G.cosd(a), G.sind(a)
            _log(f'log{k}', (0.22 * c, 0.31, 0.22 * s_), (0.03 * c, 0.6, 0.03 * s_), 0.035, M.log, M.log_end, k * 3.1)
        _log('log_across', (-0.18, 0.33, -0.08), (0.18, 0.33, 0.1), 0.04, M.log, M.log_end, 9.0)
    G.join(still, 'pit', origin=(0, 0, 0))
    # the flames: tongues round the logs and a bright core, grown from the ash
    base = 0.298   # under the ash's top
    with G.new_parts() as parts:
        for k, (a, d, h, r) in enumerate(((0, 0.09, 0.3, 0.08), (51, 0.11, 0.22, 0.07), (103, 0.08, 0.34, 0.08),
                                          (154, 0.11, 0.24, 0.07), (206, 0.09, 0.32, 0.08), (257, 0.11, 0.2, 0.06),
                                          (309, 0.09, 0.28, 0.075))):
            _flame(f'flame{k}', d * G.cosd(a), base, d * G.sind(a), r, h, a, M.flame, lean=0.04)
        for k, (x, z, h, r) in enumerate(((0.0, 0.0, 0.46, 0.085), (0.04, -0.03, 0.32, 0.06), (-0.04, 0.03, 0.36, 0.06))):
            _flame(f'flame_core{k}', x, base, z, r, h, 40 * k, M.flame_core, lean=0.015)
    flames = G.join(parts, 'flames', origin=F.at(0, base, 0))
    # Shrunk to a point at rest, not just flattened: the night glow's copy of the faces, drawn a hair in front,
    # would show a flattened disc through the ash.
    flames.scale = (0.001, 0.001, 0.001)
    G.settle(centre=False)
    for axis in 'XYZ':
        G.grow(flames, 'flames', seconds=0.8, axis=axis)
    return G.meshes()


def _flame(name, x, y, z, r, h, a, material, lean=0.03):
    """A tongue of flame from game (x, y, z), `r` across at its foot and `h` tall, flattened across the way it leans
    (out along `a` degrees), swelling a little above its foot, twisting as it narrows to its tip."""
    bx, by, bz = F.at(x, y, z)
    out = Vector((-G.cosd(a), -G.sind(a)))               # game (cos, sin) along x, depth as Blender
    sections = []
    for t, k in ((0.0, 0.8), (0.18, 1.0), (0.45, 0.78), (0.72, 0.42), (1.0, 0.0)):
        c = Vector((bx, by)) + out * lean * t * t
        tw = math.radians(a + 50 * t)
        loop = []
        for i in range(6):
            p = math.tau * i / 6
            u, v = r * k * math.cos(p), r * k * 0.6 * math.sin(p)
            loop.append((c.x + u * math.cos(tw) - v * math.sin(tw), c.y + u * math.sin(tw) + v * math.cos(tw), bz + h * t))
        sections.append(loop)
    return G.loft(name, sections, material=material, angle=80)


SLATS = [('iron', 'Frame'), ('park_slats', 'Slats')]

FAMILY = G.Family(
    'outdoor', M,
    [G.Model('bench', bench, size=(1.74, 0.886, 0.528), centre=(0, 0.011), key='model.bench', paint=SLATS),
     G.Model('patioSet', patioSet, size=(0.6, 0.933, 1.423), key='model.patioSet',
             paint=[('iron', 'Frame'), ('bistro', 'Slats')]),
     G.Model('picnicTable', picnicTable, size=(2.5, 1.058, 1.62), key='model.picnicTable',
             paint=[('pine_dark', 'Frame'), ('pine', 'Planks'), ('cloth', 'Cloth')]),
     G.Model('gardenChair', gardenChair, size=(0.84, 1.041, 0.757), centre=(0, -0.052), key='model.gardenChair',
             paint=[('garden_white', 'Chair')]),
     G.Model('patioTableRound', patioTableRound, size=(0.816, 0.725, 0.816), key='model.patioTableRound',
             paint=[('iron', 'Frame'), ('mosaic', 'Top'), ('tile', 'Tiles')]),
     G.Model('patioTable4', patioTable4, size=(1.6, 0.74, 0.785), centre=(0, -0.04), key='model.patioTable4',
             paint=[('teak_dark', 'Frame'), ('teak', 'Top')]),
     G.Model('patioChair', patioChair, size=(0.46, 0.928, 0.448), centre=(0, 0.042), key='model.patioChair',
             paint=[('patio_green', 'Frame'), ('cushion', 'Cushion')]),
     G.Model('adirondackChair', adirondackChair, size=(0.88, 1.097, 0.785), centre=(0, -0.038), key='model.adirondackChair',
             paint=[('adirondack', 'Chair')]),
     G.Model('benchGarden3', benchGarden3, size=(1.91, 0.906, 0.602), centre=(0, -0.021), key='model.benchGarden3',
             paint=[('iron', 'Frame'), ('bench_slats', 'Slats')]),
     G.Model('outdoorBar', outdoorBar, size=(2.0, 1.06, 0.89), centre=(0, 0.015), key='model.outdoorBar',
             paint=[('teak', 'Body'), ('bar_top', 'Counter'), ('steel', 'Foot rail')]),
     G.Model('parasol', parasol, size=(2.2, 2.46, 2.2), key='model.parasol', motions={'idle': 'stir'},
             paint=[('canopy', 'Canopy'), ('pole', 'Pole'), ('valance', 'Edge')]),
     G.Model('sunLounger', sunLounger, size=(0.62, 0.759, 1.866), centre=(0, 0.007), key='model.sunLounger',
             motions={'uses': {'sunbathe': 'recline', 'doze': 'recline'}},
             paint=[('lounger', 'Frame'), ('lounger_pad', 'Cushions'), ('towel', 'Towel')]),
     G.Model('hammock', hammock, size=(1.0, 1.2, 2.8), key='model.hammock',
             motions={'loops': {'swing': 'sway', 'doze': 'sway'}, 'ride': 'sway'},
             paint=[('hammock_frame', 'Frame'), ('sling', 'Fabric'), ('sling_stripe', 'Stripes')]),
     G.Model('porchSwing', porchSwing, size=(1.92, 2.05, 0.92), key='model.porchSwing',
             motions={'loops': {'swing': 'swing'}, 'ride': 'swing'},
             paint=[('porch_seat', 'Seat'), ('porch_frame', 'Frame'), ('chain', 'Chains'), ('porch_cushion', 'Cushion')]),
     G.Model('swingSet', swingSet, size=(1.92, 1.972, 1.51), key='model.swingSet',
             motions={'loops': {'swing': 'swing'}, 'ride': 'swing'},
             paint=[('swing_frame', 'Frame'), ('swing_seat', 'Seats')]),
     G.Model('hotTubCedar', hotTubCedar, size=(1.886, 0.917, 1.903), centre=(0, 0.008), key='model.hotTubCedar',
             motions={'uses': {'soak': 'cover_off'}, 'open': 'cover_off', 'loops': {'soak': 'bubbles'}},
             paint=[('cedar', 'Cedar'), ('tub_cover', 'Cover')]),
     # OU3: the grills and the fire pit
     G.Model('grill', grill, size=(0.642, 1.042, 0.634), centre=(0, 0.013), key='model.grill', motions={'uses': {'cook': 'lid_open'}},
             paint=[('kettle', 'Body'), ('kettle_lid', 'Lid')]),
     G.Model('grillStation', grillStation, size=(1.66, 1.199, 0.66), centre=(0, -0.125), key='model.grillStation',
             motions={'uses': {'cook': 'hood_up', 'feast': 'hood_up'}}, paint=[('cart', 'Cart'), ('hood', 'Hood')]),
     G.Model('firePit', firePit, size=(0.927, 0.619, 0.91), centre=(-0.001, -0.009), key='model.firePit', motions={'uses': {'relax': 'flames'}},
             paint=[('pit', 'Bowl'), ('pit_stones', 'Stones')])],
    comment='Blender-built outdoor things (tools/art/blender/families/outdoor.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. The parasol\'s canopy stirs in the wind (`stir`, ambient); the lounger '
            'reclines, the hammock and the swings sway with whoever is on them, and the hot tub\'s cover '
            'folds back for a soak.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
