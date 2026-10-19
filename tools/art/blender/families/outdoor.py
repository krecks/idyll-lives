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
    coal_hot=('#D9542B', 'gloss'),       # the kettle's coals: no glow, which would show through its shut lid
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
    pit=('#8E8A84', 'matte'),            # the fire pit's bowl (Bowl)
    pit_stones=('#A19C94', 'matte'),     # its capstones (Stones)
    log=('#6B4A33', 'bark'),
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
    leaves it where it was built."""
    G.join_all(name)
    G.settle(centre=centred)
    return G.meshes()


# --- the park bench: cast-iron ends, four seat slats and three back slats -------------------------------------
def bench():
    with G.new_parts():
        for s in (-1, 1):
            x = s * 0.78
            _rod(f'leg_front{s}', (x, 0.0, 0.2), (x, 0.45, 0.2), 0.022, M.iron)
            _rod(f'leg_back{s}', (x, 0.0, -0.1), (x, 0.88, -0.224), 0.022, M.iron)
            _rod(f'rail{s}', (x, 0.435, -0.19), (x, 0.435, 0.24), 0.02, M.iron)
            _rod(f'stretcher{s}', (x, 0.1, -0.11), (x, 0.1, 0.2), 0.014, M.iron, verts=8)
            _rod(f'arm_post{s}', (x, 0.44, 0.24), (x, 0.65, 0.25), 0.017, M.iron)
            _board(f'arm{s}', (x, 0.66, -0.17), (x, 0.66, 0.24), 0.06, 0.03, M.iron, bevel=0.012)
            F.sphere(f'arm_end{s}', 0.03, (x, 0.66, 0.245), M.iron, scale=(1, 1, 0.6), segments=12, rings=8)
            G.torus(f'scroll{s}', 0.045, 0.01, at=F.at(x, 0.56, 0.205), material=M.iron, rot=(0, 90, 0),
                    major_segments=16, minor_segments=6)
            for z in (-0.1, 0.2):
                F.box(f'foot{s}{z}', 0.06, 0.02, 0.07, (x, 0.01, z), M.iron, bevel=0.006)
        for k, z in enumerate((-0.1, 0.0, 0.1, 0.2)):
            F.box(f'seat{k}', 1.74, 0.035, 0.085, (0, 0.47, z), M.park_slats, bevel=0.01)
        for k, (y, z) in enumerate(((0.6, -0.2), (0.72, -0.2168), (0.84, -0.2336))):
            F.box(f'back{k}', 1.74, 0.085, 0.03, (0, y, z), M.park_slats, rot=(-8, 0, 0), bevel=0.01)
    return _finish('bench')


# --- the folding bistro set: a slatted table between two folding chairs --------------------------------------
def _bistro_chair(c, facing):
    """A folding chair centred at game depth `c`, its front towards `facing` (+1: +z)."""
    def at(x, y, lz):
        return (x * facing, y, c + lz * facing)
    for k, lz in enumerate((-0.14, -0.045, 0.05, 0.145)):
        F.box(f'chair{c}_seat{k}', 0.42, 0.02, 0.085, at(0, 0.43, lz), M.bistro, bevel=0.006)
    for k, (y, lz) in enumerate(((0.62, -0.204), (0.74, -0.225), (0.86, -0.247))):
        F.box(f'chair{c}_back{k}', 0.4, 0.07, 0.02, at(0, y, lz), M.bistro, rot=(-10 * facing, 0, 0), bevel=0.008)
    for s in (-1, 1):
        x = s * 0.21
        _rod(f'chair{c}_front{s}', at(x, 0.0, 0.2), at(x, 0.92, -0.25), 0.012, M.iron, verts=8)
        _rod(f'chair{c}_back_leg{s}', at(x, 0.0, -0.2), at(x, 0.42, 0.14), 0.012, M.iron, verts=8)
        _rod(f'chair{c}_rail{s}', at(x, 0.415, -0.18), at(x, 0.415, 0.18), 0.011, M.iron, verts=8)
    _rod(f'chair{c}_stretcher', at(-0.21, 0.12, 0.13), at(0.21, 0.12, 0.13), 0.01, M.iron, verts=8)
    _rod(f'chair{c}_seat_bar', at(-0.21, 0.415, 0.18), at(0.21, 0.415, 0.18), 0.01, M.iron, verts=8)


def patioSet():
    with G.new_parts():
        for k, z in enumerate((0.075, 0.225, 0.375, 0.525)):
            F.box(f'top{k}', 0.6, 0.02, 0.135, (0, 0.71, z), M.bistro, bevel=0.006)
        for z in (0.03, 0.57):
            F.box(f'top_rail{z}', 0.58, 0.03, 0.03, (0, 0.69, z), M.iron, bevel=0.006)
        for z in (0.07, 0.53):
            for s in (-1, 1):
                _rod(f'leg{z}{s}', (s * 0.26, 0.0, z), (-s * 0.26, 0.69, z), 0.0125, M.iron, verts=8)
        _rod('table_stretcher', (0, 0.345, 0.07), (0, 0.345, 0.53), 0.01, M.iron, verts=8)
        F.cyl('cup', 0.04, 0.09, (0.12, 0.765, 0.36), M.cup, verts=14)
        G.torus('cup_handle', 0.022, 0.006, at=F.at(0.165, 0.77, 0.36), material=M.cup, rot=(90, 0, 0),
                major_segments=10, minor_segments=5)
        F.cone('jug', 0.05, 0.035, 0.12, (-0.14, 0.78, 0.22), M.jug, verts=14)
        _bistro_chair(-0.62, 1)
        _bistro_chair(0.72, -1)
    return _finish('patioSet')


# --- the picnic table: a planked top on A-frames, a bench each side; a gingham cloth and a basket --------------
def picnicTable():
    with G.new_parts():
        for k, z in enumerate((-0.29, -0.145, 0.0, 0.145, 0.29)):
            F.box(f'top{k}', 2.5, 0.04, 0.135, (0, 0.61, z), M.pine, bevel=0.008)
        for s in (-1, 1):
            for k, z in enumerate((0.6, 0.745)):
                F.box(f'seat{s}{k}', 2.5, 0.04, 0.13, (0, 0.38, s * z), M.pine, bevel=0.008)
        for x in (-0.95, 0.0, 0.95):
            F.box(f'cleat{x}', 0.07, 0.05, 0.72, (x, 0.565, 0), M.pine_dark, bevel=0.008)
        for s in (-1, 1):
            x = s * 0.95
            F.box(f'bearer{s}', 0.07, 0.06, 1.62, (x, 0.33, 0), M.pine_dark, bevel=0.008)
            for e in (-1, 1):
                _board(f'leg{s}{e}', (x, 0.0, e * 0.64), (x, 0.59, e * 0.12), 0.07, 0.05, M.pine_dark,
                       face=(0, 0, e), bevel=0.008)
            _board(f'brace{s}', (s * 0.92, 0.33, 0), (s * 0.04, 0.54, 0), 0.05, 0.05, M.pine_dark, bevel=0.006)
        # the cloth: red, with a white check on it
        F.box('cloth', 0.7, 0.004, 0.6, (-0.3, 0.632, 0), M.cloth, bevel=0.002)
        for i in range(4):
            for j in range(4):
                if (i + j) % 2 == 0:
                    F.box(f'check{i}{j}', 0.175, 0.003, 0.15, (-0.5625 + 0.175 * i, 0.6345, -0.225 + 0.15 * j),
                          M.cloth_check)
        # the basket, its lid and handle
        F.box('basket', 0.4, 0.2, 0.28, (0.55, 0.73, 0.02), M.wicker, bevel=0.03)
        F.box('basket_lid', 0.42, 0.03, 0.3, (0.55, 0.845, 0.02), M.wicker_rim, bevel=0.01)
        G.torus('basket_handle', 0.075, 0.01, at=F.at(0.55, 0.86, 0.02), material=M.handle, rot=(90, 0, 0),
                major_segments=20, minor_segments=6)
        for k, (x, z, m) in enumerate(((-0.45, 0.12, M.cup), (-0.15, -0.1, M.jug))):
            F.cyl(f'cup{k}', 0.04, 0.1, (x, 0.68, z), m, verts=12)
    return _finish('picnicTable')


# --- the garden chair: a white slatted armchair, the back leaning, a cup on the arm ----------------------------
def _back_slats(prefix, xs, lengths, material, fan, bottom=(0.29, -0.123), tilt=22, w=0.085, t=0.022):
    """Back slats up a back leaning `tilt` degrees, from `bottom` (height, depth); `fan` spreads their tops."""
    up = (0, G.cosd(tilt), -G.sind(tilt))
    face = (0, G.sind(tilt), G.cosd(tilt))
    y0, z0 = bottom
    for k, (x, length) in enumerate(zip(xs, lengths)):
        top = (x * fan, y0 + length * up[1], z0 + length * up[2])
        _board(f'{prefix}{k}', (x, y0, z0), top, w, t, material, face=face, bevel=0.01)


def _slatted_chair(prefix, material, fan):
    """The garden and Adirondack chairs' shared frame: a seat sloping back, wide arms on front legs, back legs
    and stringers; the back slats are the caller's."""
    for k, z in enumerate((-0.12, 0.0, 0.12, 0.24)):
        F.box(f'{prefix}seat{k}', 0.56, 0.025, 0.1, (0, 0.4 - z * 0.15, z), material, rot=(-8.5, 0, 0), bevel=0.008)
    _board(f'{prefix}back_rail', (-0.3, 0.56, -0.25), (0.3, 0.56, -0.25), 0.06, 0.03, material,
           face=(0, G.sind(22), G.cosd(22)))
    for s in (-1, 1):
        F.box(f'{prefix}arm{s}', 0.14 if fan > 1 else 0.12, 0.025, 0.7 if fan > 1 else 0.66,
              (s * (0.37 if fan > 1 else 0.36), 0.62, -0.02), material, bevel=0.01)
        F.box(f'{prefix}leg_front{s}', 0.06, 0.61, 0.06, (s * 0.36, 0.305, 0.26), material, bevel=0.01)
        F.box(f'{prefix}leg_back{s}', 0.06, 0.42, 0.06, (s * 0.32, 0.21, -0.24), material, bevel=0.01)
        _board(f'{prefix}stringer{s}', (s * 0.3, 0.42, 0.3), (s * 0.3, 0.32, -0.28), 0.05, 0.08, material,
               face=(1, 0, 0), bevel=0.01)
        _board(f'{prefix}arm_brace{s}', (s * 0.36, 0.45, 0.22), (s * 0.36, 0.6, 0.14), 0.04, 0.04, material,
               face=(1, 0, 0), bevel=0.008)


def gardenChair():
    with G.new_parts():
        _slatted_chair('', M.garden_white, fan=1.0)
        xs = (-0.24, -0.12, 0.0, 0.12, 0.24)
        _back_slats('back', xs, (0.74, 0.79, 0.81, 0.79, 0.74), M.garden_white, fan=1.0, w=0.1)
        _board('top_rail', (-0.3, 0.98, -0.43), (0.3, 0.98, -0.43), 0.05, 0.025, M.garden_white,
               face=(0, G.sind(22), G.cosd(22)), bevel=0.01)
        F.cyl('cup', 0.05, 0.1, (0.36, 0.683, 0.18), M.cup, verts=14)
    return _finish('gardenChair', centred=True)


def adirondackChair():
    with G.new_parts():
        _slatted_chair('', M.adirondack, fan=1.6)
        xs = (-0.18, -0.09, 0.0, 0.09, 0.18)
        _back_slats('back', xs, (0.78, 0.84, 0.87, 0.84, 0.78), M.adirondack, fan=1.6)
    return _finish('adirondackChair', centred=True)


# --- the round patio table: a mosaic top on an iron pedestal ---------------------------------------------------
def patioTableRound():
    with G.new_parts():
        F.cyl('top', 0.4, 0.04, (0, 0.7, 0), M.mosaic, verts=40)
        G.torus('rim', 0.392, 0.016, at=F.at(0, 0.705, 0), material=M.iron, major_segments=40, minor_segments=6)
        F.cyl('hub', 0.12, 0.04, (0, 0.66, 0), M.iron, verts=20, bevel=0.01)
        F.cyl('pole', 0.035, 0.62, (0, 0.34, 0), M.iron, verts=14)
        G.revolve('foot', [(0, 0), (0.25, 0), (0.25, 0.02), (0.15, 0.05), (0.05, 0.07), (0, 0.07)],
                  at=F.at(0, 0, 0), material=M.iron, segments=28)
        F.cyl('centre_tile', 0.05, 0.006, (0, 0.722, 0), M.tile, verts=16)
        for k in range(10):
            a = 36 * k + 18
            m = M.tile if k % 2 == 0 else M.tile_light
            F.box(f'tile{k}', 0.07, 0.006, 0.07, (0.25 * G.cosd(a), 0.722, 0.25 * G.sind(a)), m, rot=(0, 0, a))
        for k in range(20):
            a = 18 * k
            F.box(f'edge{k}', 0.05, 0.005, 0.03, (0.355 * G.cosd(a), 0.7215, 0.355 * G.sind(a)),
                  M.tile_light if k % 2 else M.tile, rot=(0, 0, a))
    return _finish('patioTableRound')


# --- the teak patio table: five planks on aprons and square legs -----------------------------------------------
def patioTable4():
    with G.new_parts():
        for k, z in enumerate((-0.36, -0.2, -0.04, 0.12, 0.28)):
            F.box(f'top{k}', 1.6, 0.03, 0.145, (0, 0.725, z), M.teak, bevel=0.008)
        for s in (-1, 1):
            F.box(f'apron{s}', 1.38, 0.06, 0.03, (0, 0.68, -0.04 + s * 0.33), M.teak_dark, bevel=0.006)
            F.box(f'apron_end{s}', 0.03, 0.06, 0.6, (s * 0.7, 0.68, -0.04), M.teak_dark, bevel=0.006)
            for e in (-1, 1):
                F.box(f'leg{s}{e}', 0.06, 0.71, 0.06, (s * 0.72, 0.355, -0.04 + e * 0.33), M.teak_dark, bevel=0.01)
        F.box('stretcher', 1.38, 0.04, 0.04, (0, 0.14, -0.04), M.teak_dark, bevel=0.006)
        for s in (-1, 1):
            F.box(f'foot_rail{s}', 0.04, 0.04, 0.6, (s * 0.72, 0.14, -0.04), M.teak_dark, bevel=0.006)
    return _finish('patioTable4')


# --- the patio chair: a bent-iron frame, slatted back and a seat cushion ---------------------------------------
def patioChair():
    sz = 0.05
    with G.new_parts():
        F.box('seat', 0.46, 0.03, 0.42, (0, 0.45, sz), M.patio_green, bevel=0.01)
        F.box('cushion', 0.42, 0.05, 0.38, (0, 0.49, sz + 0.01), M.cushion, bevel=0.02, bevel_segments=3)
        for s in (-1, 1):
            x = s * 0.2
            _rod(f'leg_front{s}', (x, 0.0, sz + 0.2), (x, 0.44, sz + 0.18), 0.0125, M.patio_green)
            _rod(f'leg_back{s}', (x, 0.0, sz - 0.22), (x * 0.95, 0.92, sz - 0.2), 0.0125, M.patio_green)
            _rod(f'arm_post{s}', (x, 0.44, sz + 0.18), (x, 0.66, sz + 0.12), 0.01, M.patio_green)
            _rod(f'arm{s}', (x, 0.66, sz + 0.12), (x * 0.97, 0.68, sz - 0.2), 0.012, M.patio_green)
            for z in (sz + 0.18, sz - 0.21):
                F.sphere(f'foot{s}{z}', 0.016, (x, 0.008, z + (0.02 if z > sz else -0.0)), M.patio_green,
                         segments=10, rings=6)
        for k, y in enumerate((0.6, 0.72, 0.84)):
            F.box(f'back{k}', 0.38, 0.035, 0.012, (0, y, sz - 0.205 - (y - 0.6) * 0.02), M.patio_green,
                  bevel=0.004)
        _rod('stretcher', (-0.2, 0.15, sz - 0.21), (0.2, 0.15, sz - 0.21), 0.009, M.patio_green, verts=8)
    return _finish('patioChair')


# --- the garden bench: three seat slats and three back slats on three iron frames ------------------------------
def benchGarden3():
    with G.new_parts():
        for k, z in enumerate((-0.08, 0.04, 0.16)):
            F.box(f'seat{k}', 2.7, 0.03, 0.1, (0, 0.45, z), M.bench_slats, bevel=0.008)
        for k, y in enumerate((0.6, 0.72, 0.84)):
            F.box(f'back{k}', 2.7, 0.09, 0.025, (0, y, -0.2 - (y - 0.6) * 0.15), M.bench_slats, rot=(-12, 0, 0),
                  bevel=0.008)
        for x in (-1.3, 0.0, 1.3):
            _rod(f'leg_front{x}', (x, 0.0, 0.18), (x, 0.435, 0.18), 0.022, M.iron)
            _rod(f'leg_back{x}', (x, 0.0, -0.17), (x, 0.9, -0.3), 0.022, M.iron)
            _rod(f'rail{x}', (x, 0.42, -0.23), (x, 0.42, 0.22), 0.02, M.iron)
            _rod(f'stretcher{x}', (x, 0.1, -0.15), (x, 0.1, 0.18), 0.013, M.iron, verts=8)
            for z in (-0.17, 0.18):
                F.box(f'foot{x}{z}', 0.06, 0.02, 0.07, (x, 0.01, z), M.iron, bevel=0.006)
        for s in (-1, 1):
            x = s * 1.3
            _board(f'arm{s}', (x, 0.66, -0.23), (x, 0.66, 0.24), 0.07, 0.035, M.iron, bevel=0.012)
            F.sphere(f'arm_end{s}', 0.035, (x, 0.66, 0.245), M.iron, scale=(1, 1, 0.55), segments=12, rings=8)
            _rod(f'arm_post{s}', (x, 0.43, 0.22), (x, 0.65, 0.225), 0.017, M.iron)
            G.torus(f'scroll{s}', 0.045, 0.01, at=F.at(x, 0.55, 0.18), material=M.iron, rot=(0, 90, 0),
                    major_segments=16, minor_segments=6)
    return _finish('benchGarden3')


# --- the outdoor bar: a slatted teak counter with a back shelf and a foot rail ---------------------------------
def outdoorBar():
    with G.new_parts():
        F.box('body', 2.0, 0.98, 0.5, (0, 0.49, -0.12), M.teak, bevel=0.01)
        F.box('plinth', 1.96, 0.06, 0.46, (0, 0.03, -0.1), M.teak_dark, bevel=0.006)
        F.box('counter', 2.0, 0.06, 0.88, (0, 1.03, -0.04), M.bar_top, bevel=0.012)
        F.box('shelf', 2.0, 0.04, 0.26, (0, 0.92, -0.36), M.bar_top, bevel=0.006)
        for k in range(20):
            F.box(f'slat{k}', 0.088, 0.92, 0.02, (-0.95 + 0.1 * k, 0.47, 0.14), M.teak, bevel=0.004)
        _rod('foot_rail', (-0.95, 0.2, 0.26), (0.95, 0.2, 0.26), 0.02, M.steel, verts=14)
        for x in (-0.88, 0.88):
            _rod(f'bracket{x}', (x, 0.2, 0.15), (x, 0.2, 0.26), 0.014, M.steel, verts=10)
        for x in (-0.95, 0.95):
            F.sphere(f'rail_end{x}', 0.022, (x, 0.2, 0.26), M.steel, segments=12, rings=8)
    return _finish('outdoorBar', centred=True)


# --- the parasol: an octagonal canopy on ribs, on a pole in a weighted base ------------------------------------
CANOPY_Y = 1.97   # the canopy's rim, height


def parasol():
    with G.new_parts() as still:
        F.cyl('base', 0.2, 0.06, (0, 0.03, 0), M.base, verts=24, bevel=0.015)
        F.cyl('base_boss', 0.06, 0.06, (0, 0.09, 0), M.base, verts=16, bevel=0.01)
        F.cyl('pole', 0.02, 1.62, (0, 0.93, 0), M.pole, verts=12)
        F.cyl('crank', 0.03, 0.08, (0, 1.3, 0), M.base, verts=12, bevel=0.006)
    G.join(still, 'stand', origin=(0, 0, 0))
    with G.new_parts() as top:
        F.cyl('pole_top', 0.02, 0.7, (0, 2.06, 0), M.pole, verts=12)
        F.cyl('runner', 0.03, 0.07, (0, 1.78, 0), M.pole, verts=12)
        G.revolve('canopy', [(0, 0.39), (1.17, 0.0), (1.19, 0.0), (1.19, 0.012), (0.04, 0.43), (0, 0.43)],
                  at=F.at(0, CANOPY_Y, 0), material=M.canopy, segments=8, rot=(0, 0, 22.5), smooth=False)
        G.revolve('valance', [(1.186, -0.11), (1.194, -0.11), (1.194, 0.004), (1.186, 0.004), (1.186, -0.11)],
                  at=F.at(0, CANOPY_Y, 0), material=M.valance, segments=8, rot=(0, 0, 22.5), smooth=False)
        F.sphere('finial', 0.04, (0, 2.42, 0), M.pole, segments=12, rings=8)
        for k in range(8):
            a = 45 * k + 22.5
            ca, sa = G.cosd(a), G.sind(a)
            _rod(f'rib{k}', (0.04 * ca, 2.35, 0.04 * sa), (1.14 * ca, CANOPY_Y - 0.005, 1.14 * sa), 0.006, M.pole,
                 verts=6)
            _rod(f'strut{k}', (0.03 * ca, 1.78, 0.03 * sa), (0.6 * ca, 2.165, 0.6 * sa), 0.005, M.pole, verts=6)
    canopy = G.hinged(G.join(top, 'canopy'), F.at(0, 1.3, 0))
    G.settle(centre=False)
    # 3°: at 1.2° the rim moved a pixel or two from the game camera, too little to notice
    G.rock(canopy, 'stir', 3, seconds=2.4, axis='X')
    return G.meshes()


# --- OU2: the sun lounger: a slatted platform and cushion; the back reclines flat for sunbathing ---------------
BACK_HINGE = (0.29, -0.36)   # the back's hinge, (height, depth)


def sunLounger():
    with G.new_parts() as still:
        for s in (-1, 1):
            F.box(f'rail{s}', 0.04, 0.07, 1.32, (s * 0.29, 0.255, 0.28), M.lounger, bevel=0.008)
            for z in (-0.3, 0.86):
                F.box(f'leg{s}{z}', 0.05, 0.23, 0.05, (s * 0.28, 0.115, z), M.lounger, bevel=0.008)
        for k in range(9):
            F.box(f'slat{k}', 0.54, 0.02, 0.11, (0, 0.27, -0.3 + 0.145 * k), M.lounger, bevel=0.005)
        F.box('cushion', 0.56, 0.06, 0.9, (0, 0.32, 0.47), M.lounger_pad, bevel=0.025, bevel_segments=3)
        F.box('towel', 0.4, 0.02, 0.3, (0, 0.36, 0.7), M.towel, bevel=0.008)
        F.box('towel_roll', 0.4, 0.04, 0.05, (0, 0.37, 0.56), M.towel, bevel=0.018)
        F.box('seat_end', 0.56, 0.06, 0.32, (0, 0.32, -0.2), M.lounger_pad, bevel=0.025, bevel_segments=3)
    G.join(still, 'lounger', origin=(0, 0, 0))
    y0, z0 = BACK_HINGE
    with G.new_parts() as back:
        a = 35
        along = (0, G.sind(a), -G.cosd(a))       # up the back
        face = (0, G.cosd(a), G.sind(a))
        top = (0, y0 + 0.68 * along[1], z0 + 0.68 * along[2])
        _board('back_pad', (0, y0 + 0.04, z0 - 0.02), top, 0.56, 0.07, M.lounger_pad, face=face, bevel=0.025)
        mid = (0, y0 + 0.61 * along[1] + 0.06 * face[1], z0 + 0.61 * along[2] + 0.06 * face[2])
        _board('pillow', (-0.21, mid[1], mid[2]), (0.21, mid[1], mid[2]), 0.18, 0.08, M.pillow, face=face, bevel=0.035)
    back = G.hinged(G.join(back, 'back'), F.at(0, y0, z0))
    G.settle(centre=False)
    G.swing(back, 'recline', -30, seconds=1.0, axis='X')
    return G.meshes()


# --- the hammock: a striped sling between spreader bars on a wooden stand; it sways while lain in -------------
HOOK_Y = 1.1   # the hooks, height: the sling sways about the line through them


def _sling(name, width, half, low, high, t, materials, bands=8):
    """A sagging sling `width` wide from depth -half to +half, `low` in the middle and `high` at the ends,
    `t` thick, in bands of alternating materials."""
    import bpy
    n = bands * 3
    verts, faces, mats = [], [], []
    for i in range(n + 1):
        z = -half + 2 * half * i / n
        y = low + (high - low) * (z / half) ** 2
        for x in (-width / 2, width / 2):
            verts.append(F.at(x, y, z))
            verts.append(F.at(x, y - t, z))
    for i in range(n):
        a, b = 4 * i, 4 * (i + 1)
        m = (i // 3) % len(materials)
        for f in ((a, b, b + 2, a + 2), (a + 1, a + 3, b + 3, b + 1), (a, a + 1, b + 1, b), (a + 2, b + 2, b + 3, a + 3)):
            faces.append(f)
            mats.append(m)
    a, b = 0, 4 * n
    faces += [(a, a + 2, a + 3, a + 1), (b, b + 1, b + 3, b + 2)]
    mats += [0, 0]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in materials:
        obj.data.materials.append(m)
    for p, m in zip(obj.data.polygons, mats):
        p.material_index = m
    obj.data.update()
    return obj


def hammock():
    with G.new_parts() as stand:
        F.box('base', 0.1, 0.06, 1.86, (0, 0.03, 0), M.hammock_frame, bevel=0.01)
        for s in (-1, 1):
            F.box(f'foot{s}', 0.7, 0.04, 0.08, (0, 0.02, s * 0.86), M.hammock_frame, bevel=0.01)
            _board(f'post{s}', (0, 0.0, s * 0.89), (0, 1.15, s * 0.85), 0.08, 0.08, M.hammock_frame, face=(0, 0, s),
                   bevel=0.012)
            _board(f'brace{s}', (0, 0.06, s * 0.6), (0, 0.42, s * 0.87), 0.05, 0.05, M.hammock_frame, face=(1, 0, 0),
                   bevel=0.008)
            F.sphere(f'hook{s}', 0.02, (0, HOOK_Y, s * 0.82), M.hammock_frame, segments=8, rings=6)
    G.join(stand, 'stand', origin=(0, 0, 0))
    with G.new_parts() as bed:
        _sling('sling', 0.62, 0.66, 0.6, 0.86, 0.025, [M.sling, M.sling_stripe])
        for s in (-1, 1):
            _rod(f'spreader{s}', (-0.34, 0.865, s * 0.67), (0.34, 0.865, s * 0.67), 0.016, M.hammock_frame)
            for x in (-0.31, 0.31):
                _rod(f'rope{s}{x}', (x, 0.87, s * 0.67), (0, HOOK_Y, s * 0.82), 0.007, M.sling_stripe, verts=6)
        F.box('pillow', 0.4, 0.08, 0.26, (0, 0.73, -0.5), M.pillow, bevel=0.034, rot=(14, 0, 0))
    bed = G.hinged(G.join(bed, 'sling'), F.at(0, HOOK_Y, 0))
    G.settle(centre=False)
    G.rock(bed, 'sway', 8, seconds=2.0, axis='Y')
    return G.meshes()


# --- the porch swing: a slatted swing seat hung on chains from an A-frame; it swings while sat in --------------
def _a_frame(prefix, beam_y, material, round_legs=False):
    """Two A-frame ends at x ±0.92 and the beam across them."""
    for s in (-1, 1):
        for e in (-1, 1):
            a, b = (s * 0.92, 0.0, e * 0.42), (s * 0.92, beam_y, 0.0)
            if round_legs:
                _rod(f'{prefix}leg{s}{e}', a, b, 0.035, material, verts=12)
            else:
                _board(f'{prefix}leg{s}{e}', a, b, 0.08, 0.08, material, face=(0, 0, e), bevel=0.01)
        _board(f'{prefix}tie{s}', (s * 0.92, 0.7, -0.27), (s * 0.92, 0.7, 0.27), 0.06, 0.04, material, bevel=0.008)
    if round_legs:
        _rod(f'{prefix}beam', (-0.96, beam_y, 0), (0.96, beam_y, 0), 0.045, material, verts=14)
    else:
        F.box(f'{prefix}beam', 1.92, 0.1, 0.1, (0, beam_y, 0), material, bevel=0.01)


def porchSwing():
    with G.new_parts() as frame:
        _a_frame('', 2.0, M.porch_frame)
    G.join(frame, 'frame', origin=(0, 0, 0))
    with G.new_parts() as seat:
        for k in range(4):
            F.box(f'seat{k}', 1.4, 0.03, 0.105, (0, 0.48, -0.13 + 0.115 * k), M.porch_seat, bevel=0.006)
        for k in range(4):
            F.box(f'back{k}', 1.4, 0.08, 0.025, (0, 0.6 + 0.1 * k, -0.19 - 0.021 * k), M.porch_seat, rot=(-12, 0, 0),
                  bevel=0.006)
        for s in (-1, 1):
            F.box(f'arm{s}', 0.05, 0.03, 0.46, (s * 0.68, 0.7, 0.02), M.porch_seat, bevel=0.008)
            F.box(f'arm_post{s}', 0.05, 0.22, 0.05, (s * 0.68, 0.59, 0.22), M.porch_seat, bevel=0.008)
            F.box(f'side{s}', 0.05, 0.06, 0.48, (s * 0.68, 0.45, 0.03), M.porch_seat, bevel=0.008)
            for z in (-0.18, 0.24):
                _rod(f'chain{s}{z}', (s * 0.66, 0.5, z), (s * 0.68, 1.96, 0.0), 0.008, M.chain, verts=6)
        F.box('cushion', 1.3, 0.07, 0.42, (0, 0.53, 0.05), M.porch_cushion, bevel=0.03, bevel_segments=3)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 1.98, 0))
    G.settle(centre=False)
    G.rock(seat, 'swing', 10, seconds=2.0, axis='X')
    return G.meshes()


# --- the swing set: two seats on chains from a round-timber A-frame; they swing while played on ---------------
def swingSet():
    with G.new_parts() as frame:
        _a_frame('', 1.92, M.swing_frame, round_legs=True)
    G.join(frame, 'frame', origin=(0, 0, 0))
    with G.new_parts() as seats:
        for x in (-0.48, 0.48):
            F.box(f'seat{x}', 0.42, 0.035, 0.2, (x, 0.48, 0), M.swing_seat, bevel=0.015)
            for dx in (-0.17, 0.17):
                _rod(f'chain{x}{dx}', (x + dx, 0.495, 0), (x + dx, 1.88, 0), 0.008, M.chain, verts=6)
                F.sphere(f'clip{x}{dx}', 0.016, (x + dx, 1.88, 0), M.chain, segments=8, rings=6)
    seats = G.hinged(G.join(seats, 'seats'), F.at(0, 1.92, 0))
    G.settle(centre=False)
    G.rock(seats, 'swing', 20, seconds=1.6, axis='X')
    return G.meshes()


# --- the cedar hot tub: banded staves, a step, water; the cover folds back for a soak and the water bubbles ----
TUB_R = 0.92


def hotTubCedar():
    with G.new_parts() as tub:
        G.revolve('staves', [(0, 0.02), (TUB_R, 0.02), (TUB_R, 0.8), (TUB_R - 0.06, 0.8), (TUB_R - 0.06, 0.12),
                             (0, 0.12)], at=F.at(0, 0, 0), material=M.cedar, segments=32, smooth=False)
        G.torus('rim', TUB_R - 0.03, 0.04, at=F.at(0, 0.8, 0), material=M.cedar, major_segments=32,
                minor_segments=8)
        for y in (0.2, 0.6):
            G.torus(f'band{y}', TUB_R + 0.005, 0.018, at=F.at(0, y, 0), material=M.cedar_band, major_segments=32,
                    minor_segments=6)
        for k in range(16):
            a = 22.5 * k
            F.box(f'groove{k}', 0.012, 0.78, 0.006, (TUB_R * G.cosd(a), 0.41, TUB_R * G.sind(a)), M.cedar_band,
                  rot=(0, 0, 90 - a))
        F.box('step', 0.6, 0.2, 0.3, (0, 0.1, 0.8), M.cedar, bevel=0.01)
        F.box('step_edge', 0.6, 0.02, 0.04, (0, 0.2, 0.94), M.cedar_band, bevel=0.005)
    G.join(tub, 'tub', origin=(0, 0, 0))
    with G.new_parts() as water:
        F.cyl('water', TUB_R - 0.07, 0.02, (0, 0.68, 0), M.water, verts=32)
        for k in range(14):
            a = 360 / 14 * k + (k % 3) * 9
            r = 0.2 + 0.5 * ((k * 7) % 5) / 5
            F.sphere(f'bubble{k}', 0.025 + 0.01 * (k % 3), (r * G.cosd(a), 0.69, r * G.sind(a)), M.foam,
                     scale=(1, 1, 0.4), segments=8, rings=4)
    water = G.hinged(G.join(water, 'water'), F.at(0, 0.68, 0))
    with G.new_parts() as cover:
        F.cyl('cover', TUB_R + 0.02, 0.07, (0, 0.875, 0), M.tub_cover, verts=32, bevel=0.02)
        F.box('cover_seam', 1.86, 0.01, 0.03, (0, 0.912, 0), M.tub_cover, bevel=0.004)
        for x in (-0.3, 0.3):
            F.box(f'strap{x}', 0.05, 0.012, 0.1, (x, 0.9, TUB_R - 0.02), M.tub_cover, bevel=0.004)
    cover = G.hinged(G.join(cover, 'cover'), F.at(0, 0.84, -TUB_R - 0.02))
    G.settle(centre=False)
    G.swing(cover, 'cover_off', -105, seconds=1.2, axis='X')
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


KETTLE_R, KETTLE_RIM = 0.3, 0.99     # the charcoal grill's bowl: its radius and its rim's height (old part)


# The charcoal grill: a kettle on three legs, wheels on the back two, an ash catcher under the bowl; inside, a bed of
# red coals and a grate with burgers and sausages. The lid is hinged at the back of the rim
# and swings up (`lid_open`) for the whole of a cook.
def grill():
    with G.new_parts() as still:
        y0, top = 0.64, KETTLE_RIM
        G.revolve('bowl', [(0, y0), (0.15, y0 + 0.005), (0.24, y0 + 0.04), (0.29, 0.76), (KETTLE_R, 0.86),
                           (KETTLE_R, top), (KETTLE_R - 0.014, top), (KETTLE_R - 0.014, 0.86), (0.276, 0.77),
                           (0.226, y0 + 0.055), (0.14, y0 + 0.02), (0, y0 + 0.02)],
                  at=F.at(0, 0, 0), material=M.kettle, segments=32)
        G.torus('rim', KETTLE_R - 0.004, 0.007, at=F.at(0, top, 0), material=M.kettle, major_segments=32, minor_segments=6)
        for s in (-1, 1):                                           # the bowl's side handles
            F.box(f'side_handle{s}', 0.012, 0.025, 0.12, (s * (KETTLE_R + 0.015), 0.93, 0), M.grate, bevel=0.004)
        # the coals, glowing, and the cooking grate over them with food
        F.cyl('coal_bed', 0.24, 0.03, (0, 0.85, 0), M.coals, verts=20)
        _embers('coal', 0.21, 0.875, 14, 20, M.coal_hot)
        G.torus('grate_ring', KETTLE_R - 0.025, 0.006, at=F.at(0, 0.965, 0), material=M.grate, major_segments=28, minor_segments=5)
        for k in range(-5, 6):
            x = k * 0.05
            half = math.sqrt(max((KETTLE_R - 0.025) ** 2 - x * x, 0))
            _rod(f'grate_bar{k}', (x, 0.965, -half), (x, 0.965, half), 0.004, M.grate, verts=6)
        for k, (x, z) in enumerate(((-0.12, 0.06), (0.02, -0.12), (0.13, 0.08))):
            F.cyl(f'burger{k}', 0.048, 0.022, (x, 0.982, z), M.burger, verts=14, bevel=0.006)
        for k, (x, z) in enumerate(((-0.06, -0.09), (0.06, -0.02))):
            _rod(f'sausage{k}', (x - 0.07, 0.982, z), (x + 0.07, 0.982, z + 0.02), 0.016, M.sausage, verts=8)
        # the legs, the ash catcher and the wheels
        for k, a in enumerate((90, 210, 330)):
            c, s_ = G.cosd(a), G.sind(a)
            foot_y = 0.075 if a != 90 else 0.0
            _rod(f'leg{k}', (c * 0.13, 0.68, s_ * 0.13), (c * 0.31, foot_y, s_ * 0.31), 0.014, M.grate, verts=8)
        F.cyl('foot_cap', 0.02, 0.02, (0, 0.01, 0.31), M.wheel, verts=10)
        G.torus('rack', 0.2, 0.008, at=F.at(0, 0.3, 0), material=M.grate, major_segments=24, minor_segments=5)
        F.cyl('ash_catcher', 0.12, 0.05, (0, 0.45, 0), M.grate, verts=20, bevel=0.01)
        _rod('axle', (-0.29, 0.075, -0.155), (0.29, 0.075, -0.155), 0.008, M.grate, verts=6)
        for s in (-1, 1):
            G.cylinder(f'wheel{s}', 0.075, 0.03, at=F.at(s * 0.29, 0.075, -0.155), material=M.wheel, rot=(0, 90, 0), verts=18)
    G.join(still, 'kettle', origin=(0, 0, 0))
    with G.new_parts() as parts:
        G.revolve('lid', [(0, 1.215), (0.16, 1.19), (0.25, 1.125), (KETTLE_R - 0.012, 1.04), (KETTLE_R - 0.012, KETTLE_RIM + 0.002),
                          (KETTLE_R + 0.004, KETTLE_RIM + 0.002), (KETTLE_R + 0.004, 1.045), (0.26, 1.135), (0.165, 1.205),
                          (0, 1.23)],
                  at=F.at(0, 0, 0), material=M.kettle_lid, segments=32)
        F.cyl('vent', 0.04, 0.012, (0, 1.232, 0), M.grate, verts=16)
        for s in (-1, 1):                                           # the handle, on two brackets at the lid's front
            _rod(f'handle_post{s}', (s * 0.06, 1.14, 0.235), (s * 0.06, 1.165, 0.275), 0.007, M.grate, verts=6)
        _rod('handle', (-0.08, 1.165, 0.277), (0.08, 1.165, 0.277), 0.013, M.handle_steel, verts=10)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, KETTLE_RIM, -KETTLE_R))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -95, seconds=0.9, axis='X')
    return G.meshes()


# The gas grill station: a stainless cart on a plinth, two doors and the control panel's knobs below a dark
# counter (its top at 0.9, the content's `surface`, which stands things on the right); a chopping board with a
# tray of food on the left, a bottle and a pepper mill at the back right. The grill in the middle (food on its
# grate) has a barrel hood hinged at the back that swings up (`hood_up`) for cook and feast.
HOOD = (-0.39, 0.11, 0.905, 1.0)      # the hood's back and front depth, its bottom and where its barrel starts


def grillStation():
    back, front, bottom, spring = HOOD
    with G.new_parts() as still:
        F.box('plinth', 1.56, 0.06, 0.54, (0, 0.03, -0.15), M.firebox, bevel=0.005)
        F.box('cart', 1.6, 0.8, 0.58, (0, 0.46, -0.15), M.cart, bevel=0.01)
        F.box('top', 1.66, 0.04, 0.66, (0, 0.88, -0.125), M.worktop, bevel=0.008)
        F.box('panel', 0.96, 0.1, 0.012, (0, 0.8, 0.146), M.firebox, bevel=0.003)
        for k, x in enumerate((-0.3, -0.1, 0.1, 0.3)):
            G.cylinder(f'knob{k}', 0.026, 0.03, at=F.at(x, 0.8, 0.165), material=M.handle_steel, rot=(90, 0, 0), verts=16, bevel=0.004)
        for s in (-1, 1):
            F.box(f'door{s}', 0.74, 0.6, 0.014, (s * 0.39, 0.4, 0.147), M.cart, bevel=0.004)
            _rod(f'door_handle{s}', (s * 0.39 - 0.22, 0.66, 0.175), (s * 0.39 + 0.22, 0.66, 0.175), 0.009, M.handle_steel, verts=8)
            for e in (-1, 1):
                _rod(f'door_post{s}{e}', (s * 0.39 + e * 0.2, 0.66, 0.154), (s * 0.39 + e * 0.2, 0.66, 0.172), 0.006, M.handle_steel, verts=6)
        # the grill: a dark firebox let into the counter, its grate with food on it
        F.box('firebox', 0.86, 0.012, 0.46, (0, 0.898, -0.14), M.firebox)
        for k in range(-8, 9):
            _rod(f'grate_bar{k}', (k * 0.05, 0.909, -0.36), (k * 0.05, 0.909, 0.08), 0.005, M.grate, verts=6)
        for k, (x, z) in enumerate(((-0.25, -0.05), (-0.1, -0.2), (0.06, -0.06))):
            F.cyl(f'burger{k}', 0.05, 0.022, (x, 0.925, z), M.burger, verts=14, bevel=0.006)
        for k, z in enumerate((-0.25, -0.18)):
            _rod(f'sausage{k}', (0.12, 0.927, z), (0.3, 0.927, z + 0.02), 0.017, M.sausage, verts=8)
        _rod('corn', (0.2, 0.93, 0.0), (0.36, 0.93, -0.04), 0.024, M.corn, verts=10)
        # on the counter
        F.box('board', 0.34, 0.02, 0.36, (-0.62, 0.91, -0.12), M.board, bevel=0.006)
        F.box('tray', 0.24, 0.03, 0.16, (-0.62, 0.935, -0.12), M.tray, bevel=0.01)
        for k, x in enumerate((-0.68, -0.6)):
            F.cyl(f'patty{k}', 0.035, 0.016, (x, 0.958, -0.12), M.burger, verts=12, bevel=0.005)
        G.revolve('bottle', [(0, 0), (0.035, 0), (0.035, 0.15), (0.015, 0.2), (0.012, 0.24), (0, 0.24)],
                  at=F.at(0.72, 0.9, -0.33), material=M.bottle, segments=16)
        F.cyl('pepper_mill', 0.022, 0.14, (0.62, 0.97, -0.36), M.board, verts=12, bevel=0.01)
    G.join(still, 'cart', origin=(0, 0, 0))
    with G.new_parts() as parts:
        # the barrel: an ellipse over its upright front and back
        mid, rz, ry = (back + front) / 2, (front - back) / 2, 0.2
        arc = [(mid + rz * math.cos(math.radians(a)), spring + ry * math.sin(math.radians(a))) for a in range(0, 181, 12)]
        _across('hood', [(front, bottom)] + arc + [(back, bottom)], -0.45, 0.45, M.hood)
        F.box('hood_inside', 0.86, 0.004, rz * 2 - 0.04, (0, bottom - 0.002, mid), M.firebox)
        for s in (-1, 1):
            _rod(f'hood_post{s}', (s * 0.3, 1.0, front - 0.005), (s * 0.3, 1.0, front + 0.045), 0.008, M.handle_steel, verts=6)
        _rod('hood_handle', (-0.34, 1.0, front + 0.05), (0.34, 1.0, front + 0.05), 0.013, M.handle_steel, verts=10)
        G.cylinder('gauge_bezel', 0.038, 0.016, at=F.at(0, 1.1, front + 0.035), material=M.handle_steel, rot=(90, 0, 0), verts=20)
        G.cylinder('gauge', 0.03, 0.004, at=F.at(0, 1.1, front + 0.044), material=M.gauge, rot=(90, 0, 0), verts=20)
    hood = G.hinged(G.join(parts, 'hood'), F.at(0, spring, back))
    G.settle(centre=False)
    G.swing(hood, 'hood_up', -75, seconds=0.9, axis='X')
    return G.meshes()


# The fire pit: a stone bowl with a ring of capstones on its rim, an ash bed with embers that glow at night and a
# teepee of logs. The flames grow on the logs (`flames`) while a resident warms up by it; the light is the
# content's (`light`, at 0.5 m).
PIT_R, PIT_TOP = 0.43, 0.3


def firePit():
    with G.new_parts() as still:
        G.revolve('bowl', [(0, 0), (PIT_R - 0.03, 0), (PIT_R, 0.03), (PIT_R, PIT_TOP), (0.33, PIT_TOP), (0.31, 0.2), (0, 0.2)],
                  at=F.at(0, 0, 0), material=M.pit, segments=32, smooth=False)
        F.cyl('ash', 0.315, 0.1, (0, 0.255, 0), M.coals, verts=24)
        _embers('ember', 0.2, 0.31, 12, 40, M.ember)
        for i in range(10):
            a = i * 36 + (7 if i % 2 else -4)
            F.box(f'stone{i}', 0.24, 0.075, 0.14, (0.38 * G.cosd(a), 0.335, 0.38 * G.sind(a)), M.pit_stones,
                  rot=(0, 0, a - 90 + (i % 3 - 1) * 5), bevel=0.03, bevel_segments=2)
        for k, a in enumerate((20, 110, 200, 290)):
            c, s_ = G.cosd(a), G.sind(a)
            _rod(f'log{k}', (0.22 * c, 0.31, 0.22 * s_), (0.03 * c, 0.6, 0.03 * s_), 0.035, M.log, verts=8)
        _rod('log_across', (-0.18, 0.33, -0.08), (0.18, 0.33, 0.1), 0.04, M.log, verts=8)
    G.join(still, 'pit', origin=(0, 0, 0))
    # the flames: tongues round the logs and a bright core, grown from the ash
    base = 0.298   # under the ash's top
    with G.new_parts() as parts:
        for k, (a, d, h, r) in enumerate(((0, 0.09, 0.3, 0.08), (51, 0.11, 0.22, 0.07), (103, 0.08, 0.34, 0.08),
                                          (154, 0.11, 0.24, 0.07), (206, 0.09, 0.32, 0.08), (257, 0.11, 0.2, 0.06),
                                          (309, 0.09, 0.28, 0.075))):
            # tongues leaning out a little from the middle, each flattened across the way it leans
            F.cone(f'flame{k}', r, 0.004, h, (d * G.cosd(a), base + h / 2, d * G.sind(a)), M.flame, verts=10,
                   scale=(1, 0.6, 1), rot=(9 * G.sind(a), -9 * G.cosd(a), -a))
        for k, (x, z, h, r) in enumerate(((0.0, 0.0, 0.46, 0.085), (0.04, -0.03, 0.32, 0.06), (-0.04, 0.03, 0.36, 0.06))):
            F.cone(f'flame_core{k}', r, 0.003, h, (x, base + h / 2, z), M.flame_core, verts=10, scale=(1, 0.7, 1), rot=(0, 0, 40 * k))
    flames = G.join(parts, 'flames', origin=F.at(0, base, 0))
    # Shrunk to a point at rest, not just flattened: the night glow's copy of the faces, drawn a hair in front,
    # would show a flattened disc through the ash.
    flames.scale = (0.001, 0.001, 0.001)
    G.settle(centre=False)
    for axis in 'XYZ':
        G.grow(flames, 'flames', seconds=0.8, axis=axis)
    return G.meshes()


SLATS = [('iron', 'Frame'), ('park_slats', 'Slats')]

FAMILY = G.Family(
    'outdoor', M,
    [G.Model('bench', bench, size=(1.74, 0.886, 0.528), centre=(0, 0.011), key='model.bench', paint=SLATS),
     G.Model('patioSet', patioSet, size=(0.6, 0.933, 1.863), centre=(0, 0.05), key='model.patioSet',
             paint=[('iron', 'Frame'), ('bistro', 'Slats')]),
     G.Model('picnicTable', picnicTable, size=(2.5, 0.958, 1.62), key='model.picnicTable',
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
     G.Model('benchGarden3', benchGarden3, size=(2.7, 0.906, 0.602), centre=(0, -0.021), key='model.benchGarden3',
             paint=[('iron', 'Frame'), ('bench_slats', 'Slats')]),
     G.Model('outdoorBar', outdoorBar, size=(2.0, 1.06, 0.89), centre=(0, 0.015), key='model.outdoorBar',
             paint=[('teak', 'Body'), ('bar_top', 'Counter'), ('steel', 'Foot rail')]),
     G.Model('parasol', parasol, size=(2.2, 2.46, 2.2), key='model.parasol', motions={'idle': 'stir'},
             paint=[('canopy', 'Canopy'), ('pole', 'Pole'), ('valance', 'Edge')]),
     G.Model('sunLounger', sunLounger, size=(0.62, 0.759, 1.866), centre=(0, 0.007), key='model.sunLounger',
             motions={'uses': {'sunbathe': 'recline', 'doze': 'recline'}},
             paint=[('lounger', 'Frame'), ('lounger_pad', 'Cushions'), ('towel', 'Towel')]),
     G.Model('hammock', hammock, size=(0.7, 1.15, 1.86), key='model.hammock',
             motions={'loops': {'swing': 'sway', 'doze': 'sway'}, 'ride': 'sway'},
             paint=[('hammock_frame', 'Frame'), ('sling', 'Fabric'), ('sling_stripe', 'Stripes')]),
     G.Model('porchSwing', porchSwing, size=(1.92, 2.05, 0.92), key='model.porchSwing',
             motions={'loops': {'swing': 'swing'}, 'ride': 'swing'},
             paint=[('porch_seat', 'Seat'), ('porch_frame', 'Frame'), ('chain', 'Chains'), ('porch_cushion', 'Cushion')]),
     G.Model('swingSet', swingSet, size=(1.92, 1.972, 0.908), key='model.swingSet',
             motions={'loops': {'swing': 'swing'}, 'ride': 'swing'},
             paint=[('swing_frame', 'Frame'), ('swing_seat', 'Seats')]),
     G.Model('hotTubCedar', hotTubCedar, size=(1.886, 0.917, 1.903), centre=(0, 0.008), key='model.hotTubCedar',
             motions={'uses': {'soak': 'cover_off'}, 'open': 'cover_off', 'loops': {'soak': 'bubbles'}},
             paint=[('cedar', 'Cedar'), ('tub_cover', 'Cover')]),
     # OU3: the grills and the fire pit
     G.Model('grill', grill, size=(0.642, 1.242, 0.634), centre=(0, 0.013), key='model.grill', motions={'uses': {'cook': 'lid_open'}},
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
