# The living room: a fold-down wall bed (`model.wallBed`, content `wallBed` in catalog2/living.json), a
# sofa bed (`model.sofaBed`, content `sofaBed`), and the L1 sofas and sectionals (notes: living.md).
#
# A cabinet at the back of a 2 × 3 footprint, with open shelves on both sides. The bed stands folded up
# in it at rest, its underside dressed as two door panels with a handle. It turns down on a hinge at the
# cabinet's foot to fill the footprint in front (`bed_down`) and stays down while the object is open:
# residents fold it down ("Fold down", or going to bed) and up ("Fold up", or after a night's sleep;
# content `when`/`leaves`). The game lays sleepers centred on the footprint, so the mattress is.
#
# Built in game coordinates (`F`: x, height, depth; front +depth), the bed in its down pose, then
# turned up to rest.
#
# The sofa bed is a boxy two-seater against the wall (its placeholder's size), whose mattress, folded
# away inside the base under the seat at rest, pulls out forward over the seat and onto the floor in front
# (`bed_out`), held out while the object is open (dozing on it, "Pull out the bed"; "Fold the bed away").
# Sleepers lie with their heads on the seat and the rest on the mattress (characters.ts BEDS['sofaBed']:
# its top and how far forward).
#
# The sofas and sectionals copy their placeholders (catalog.py, catalog2.py, furniture_prims.py), where
# they stand. Their seat cushions are a part of their own that sinks 2 cm while sat or dozed on (`sink_in`).
#
# Run:  python3 -B tools/art/blender/build.py living [--only sofaBed] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#E9E4DA', 'satin'),       # the cabinet and the bed's underside; the main part the player colours
    inside=('#D8D1C4', 'matte'),     # the cabinet's back and the shelves
    frame=('#B9A58A', 'satin'),      # the bed frame around the mattress
    metal=('#8D8F92', 'metal'),      # handle
    mattress=('#F2EFEA', 'fabric'),
    throw=('#C98B6B', 'fabric'),     # folded at the foot; the game puts a blanket over each sleeper
    pillow=('#F6F3EE', 'fabric'),
    book_a=('#9C4A3C', 'satin'),
    book_b=('#3F5E73', 'satin'),
    book_c=('#C9A65A', 'satin'),
    book_d=('#5D7A54', 'satin'),
    # the sofa bed
    upholstery=('#6F7F8C', 'fabric'),   # its body: arms, back and base; the main part the player colours
    cushion=('#7A8A97', 'fabric'),      # the seat and back cushions
    legs=('#2B2C2E', 'metal'),
    pillow_rose=('#E4B8AE', 'fabric'),
    pillow_cream=('#EFE5D3', 'fabric'),
    bedding=('#F2EFEA', 'fabric'),      # the pulled-out mattress, made up
    # L1, the sofas and sectionals: per model its upholstery (the main part), its cushions, its legs and
    # any show-wood or metal frame, the colours of its placeholder
    linen=('#DCD5CA', 'fabric'),         # the classic camel-back sofa
    linen_cushion=('#DCD5CA', 'fabric'),
    walnut=('#6E4E36', 'wood'),          # its show-wood: the back rail, the apron, the scroll faces
    walnut_legs=('#6E4E36', 'wood'),
    pillow_rust=('#B5653F', 'fabric'),
    pillow_sage=('#9DAF8C', 'fabric'),
    sage=('#9DAF8C', 'fabric'),          # the loveseat
    sage_cushion=('#9DAF8C', 'fabric'),
    oak_legs=('#CDB08A', 'wood'),
    stone=('#C9C6BE', 'fabric'),         # the three-seat sofa
    stone_cushion=('#C9C6BE', 'fabric'),
    oak_dark_legs=('#B08E66', 'wood'),
    throw_sage=('#9DAF8C', 'fabric'),
    teal=('#4E7A7F', 'fabric'),          # the designer sofa
    teal_cushion=('#4E7A7F', 'fabric'),
    brass=('#B8925A', 'metal'),          # its front rail and arm caps
    brass_legs=('#B8925A', 'metal'),
    moss=('#7A8C6E', 'fabric'),          # the three-seater
    moss_cushion=('#7A8C6E', 'fabric'),
    pillow_mustard=('#D6A64A', 'fabric'),
    greige=('#A9A49A', 'fabric'),        # the sectional's modules and the chaise
    greige_cushion=('#A9A49A', 'fabric'),
    espresso_legs=('#3B302A', 'wood'),
    throw_oat=('#D9CFC0', 'fabric'),
)

F = G.GameFrame()

# The footprint is 2 wide and 3 deep: depth -1.5 to +1.5; the wall's face is 7 cm in (objects.md).
BACK = -1.43
CAB_FRONT = -0.94     # the cabinet's sides end here
HINGE = (0.12, -0.96)  # (height, depth) of the bed's hinge, along x: the folded bed clears the back
LENGTH = 1.98         # the bed, head end to foot end
BED_W = 1.56          # the bed panel's width, inside the cabinet's sides
MATTRESS_TOP = 0.46   # characters.ts BEDS['wallBed']
INNER_TOP = 2.14      # the cabinet's opening: the folded bed reaches HINGE height + LENGTH
SECONDS = 1.0         # the bed turns down this fast, as the use that opens it starts


def _cabinet():
    """The housing and the side shelves: what stays still."""
    depth = CAB_FRONT - BACK
    mid = (BACK + CAB_FRONT) / 2
    F.box('back', 1.66, INNER_TOP, 0.02, (0, INNER_TOP / 2, BACK + 0.01), M.inside)
    for s in (-1, 1):
        F.box(f'side{s}', 0.03, INNER_TOP + 0.03, depth, (s * 0.815, (INNER_TOP + 0.03) / 2, mid), M.body, bevel=0.004)
    F.box('top', 1.66, 0.03, depth, (0, INNER_TOP + 0.015, mid), M.body, bevel=0.004)
    F.box('plinth', 1.6, 0.08, 0.04, (0, 0.04, CAB_FRONT - 0.06), M.body)
    F.box('crown', 1.98, 0.05, depth + 0.03, (0, INNER_TOP + 0.055, mid + 0.015), M.body, bevel=0.008)
    # open shelves on both sides, a few books on each shelf
    for s in (-1, 1):
        x = s * 0.905
        F.box(f'tower_side{s}', 0.02, INNER_TOP + 0.03, depth, (s * 0.98, (INNER_TOP + 0.03) / 2, mid), M.body, bevel=0.004)
        F.box(f'tower_back{s}', 0.13, INNER_TOP, 0.02, (x, INNER_TOP / 2, BACK + 0.01), M.inside)
        for i, y in enumerate((0.0, 0.45, 0.9, 1.35, 1.8)):
            F.box(f'shelf{s}{i}', 0.13, 0.02, depth - 0.02, (x, y + 0.01, mid), M.inside)
            if i < 4:
                _books(f'books{s}{i}', x, y + 0.02, mid, seed=i * 2 + (s > 0))


def _books(name, x, y, z, seed):
    """Three or four books standing on a shelf at height `y`, across the 13 cm wide tower."""
    colours = (M.book_a, M.book_b, M.book_c, M.book_d)
    heights = (0.24, 0.2, 0.27, 0.22, 0.18)
    n = 3 + seed % 2
    for k in range(n):
        h = heights[(seed + k) % len(heights)]
        F.box(f'{name}_{k}', 0.03, h, 0.18, (x - 0.045 + k * 0.03, y + h / 2, z + 0.02 * ((seed + k) % 3 - 1)),
              colours[(seed + k) % 4], bevel=0.003)


def _bed():
    """The bed in its down pose: frame, mattress, duvet and pillows on top, the door panels, handle and
    folding foot underneath. Returns it as one part with its origin on the hinge."""
    h0, z0 = HINGE
    head, foot = z0 - 0.02, z0 - 0.02 + LENGTH
    mid = (head + foot) / 2
    with G.new_parts() as parts:
        F.box('bed_base', BED_W, 0.04, LENGTH, (0, h0 + 0.02, mid), M.body, bevel=0.004)
        for s in (-1, 1):
            F.box(f'rail{s}', 0.03, 0.14, LENGTH, (s * (BED_W / 2 - 0.015), h0 + 0.11, mid), M.frame, bevel=0.004)
        for end, z in (('head', head + 0.015), ('foot', foot - 0.015)):
            F.box(f'rail_{end}', BED_W - 0.06, 0.14, 0.03, (0, h0 + 0.11, z), M.frame, bevel=0.004)
        top = MATTRESS_TOP
        F.box('mattress', BED_W - 0.07, top - (h0 + 0.10), LENGTH - 0.07, (0, (top + h0 + 0.10) / 2, mid),
              M.mattress, bevel=0.03, bevel_segments=3)
        F.box('throw', BED_W - 0.04, 0.06, 0.42, (0, top + 0.025, foot - 0.25), M.throw, bevel=0.025, bevel_segments=3)
        for s in (-1, 1):
            F.box(f'pillow{s}', 0.62, 0.08, 0.34, (s * 0.36, top + 0.04, head + 0.24), M.pillow, bevel=0.035, bevel_segments=3)
        # underneath, facing the room once folded: two raised door panels, a handle, the folding foot
        for k, (z1, z2) in enumerate(((head + 0.08, mid - 0.04), (mid + 0.04, foot - 0.16))):
            F.box(f'door{k}', BED_W - 0.16, 0.015, z2 - z1, (0, h0 - 0.0075, (z1 + z2) / 2), M.body, bevel=0.005)
        handle = head + 1.32  # about 1.45 m up once folded
        F.box('handle_bar', 0.5, 0.018, 0.018, (0, h0 - 0.06, handle), M.metal, bevel=0.006)
        for s in (-1, 1):
            F.box(f'handle_post{s}', 0.016, 0.045, 0.016, (s * 0.23, h0 - 0.03, handle), M.metal)
        F.box('foot_leg', BED_W - 0.2, h0, 0.04, (0, h0 / 2, foot - 0.09), M.body, bevel=0.004)
    return G.hinged(G.join(parts, 'bed'), F.at(0, h0, z0))


def wall_bed():
    with G.new_parts() as cabinet:
        _cabinet()
    G.join(cabinet, 'cabinet', origin=(0, 0, 0))
    bed = _bed()
    # Built down; folded up is the rest pose (the clip's first key): its foot up, its mattress to the wall.
    G.turn(bed, -90, 'X')
    G.settle(centre=False)  # it stands at the back of its tiles, where it was built
    G.swing(bed, 'bed_down', 90, seconds=SECONDS, axis='X')


# The sofa bed: 2 x 1 tiles, its back on the wall's face (depth -0.43), 1.86 m wide like its placeholder.
SB_BACK, SB_FRONT, SB_W = -0.43, 0.40, 1.86
SB_SEAT = 0.45          # the seat cushion's top (content seat height 0.44)
SB_MATTRESS = 0.44      # the pulled-out mattress's top: characters.ts BEDS['sofaBed'].h
SB_HEAD, SB_FOOT = -0.26, 1.52  # the mattress out, head end (under the back cushion) to foot end
SB_TUCKED = 0.2         # its share of its length at rest: tucked into the base under the seat, its leg too


def sofa_bed():
    with G.new_parts() as still:
        mid = (SB_BACK + SB_FRONT) / 2
        depth = SB_FRONT - SB_BACK
        F.box('base', SB_W - 0.04, 0.22, depth, (0, 0.21, mid), M.upholstery, bevel=0.03)
        F.box('back', SB_W, 0.5, 0.18, (0, 0.57, SB_BACK + 0.09), M.upholstery, bevel=0.05)
        for s in (-1, 1):
            F.box(f'arm{s}', 0.17, 0.32, depth, (s * (SB_W - 0.17) / 2, 0.46, mid), M.upholstery, bevel=0.06)
        F.box('seat', 1.5, 0.13, 0.64, (0, SB_SEAT - 0.065, SB_FRONT - 0.33), M.cushion, bevel=0.05, bevel_segments=3)
        F.box('seat_seam', 0.006, 0.12, 0.6, (0, SB_SEAT - 0.06, SB_FRONT - 0.33), M.upholstery)
        F.box('back_cushion', 1.49, 0.4, 0.15, (0, 0.63, SB_BACK + 0.24), M.cushion, rot=(-9, 0, 0), bevel=0.06, bevel_segments=3)
        F.box('pillow_l', 0.4, 0.34, 0.11, (-0.55, 0.64, SB_BACK + 0.36), M.pillow_rose, rot=(-12, 0, -10), bevel=0.05, bevel_segments=3)
        F.box('pillow_r', 0.4, 0.34, 0.11, (0.55, 0.64, SB_BACK + 0.36), M.pillow_cream, rot=(-12, 0, 10), bevel=0.05, bevel_segments=3)
        for x in (-0.86, 0.86):
            for z in (SB_BACK + 0.06, SB_FRONT - 0.06):
                F.cyl(f'leg{x}{z}', 0.022, 0.1, (x, 0.05, z), M.legs)
    G.join(still, 'sofa', origin=(0, 0, 0))
    # The mattress, built out: made up, on a thin frame, a folding leg at its foot (upholstered like the
    # sofa: one finish, one draw call); its origin at its head end.
    length = SB_FOOT - SB_HEAD
    with G.new_parts() as bed:
        F.box('mattress', 1.44, 0.1, length, (0, SB_MATTRESS - 0.05, (SB_HEAD + SB_FOOT) / 2), M.bedding, bevel=0.03, bevel_segments=3)
        F.box('bed_frame', 1.48, 0.03, length, (0, SB_MATTRESS - 0.115, (SB_HEAD + SB_FOOT) / 2), M.upholstery)
        F.box('bed_leg', 1.4, SB_MATTRESS - 0.13, 0.025, (0, (SB_MATTRESS - 0.13) / 2, SB_FOOT - 0.06), M.upholstery)
    mattress = G.join(bed, 'bed', origin=F.at(0, SB_MATTRESS - 0.13, SB_HEAD))
    # Folded away at rest: its length shortened into the base, under the seat (Blender y is the depth).
    mattress.scale = (1, SB_TUCKED, 1)
    G.settle(centre=False)  # against the wall, as built
    G.grow(mattress, 'bed_out', seconds=1.2, axis='Y')


# L1, the sofas and sectionals. Each is built where its placeholder stands, its back on the wall's face,
# and settled in place. Two nodes: the sofa (upholstery, frame, back cushions, pillows, legs) and its seat
# cushions (`seat`, origin on their underside), which `sink_in` squashes 2 cm while it's sat or dozed on.
SINK = 0.02
SOFA_USES = {'relax': 'sink_in', 'doze': 'sink_in'}


def _cushion(name, w, h, d, at, mat, rot=(0, 0, 0)):
    F.box(name, w, h, d, at, mat, rot=rot, bevel=min(0.045, 0.4 * min(w, h, d)), bevel_segments=3)


def _seat(cushions, bottom, mid):
    """The seat cushions `cushions()` makes (returning their top), joined into the part `seat`, with the
    clip that sinks them."""
    with G.new_parts() as parts:
        top = cushions()
    seat = G.join(parts, 'seat', origin=F.at(0, bottom, mid))
    return seat, top - bottom


def _sink(seat_and_thickness):
    seat, thick = seat_and_thickness
    G.squash(seat, 'sink_in', (1, 1, (thick - SINK) / thick), seconds=0.6)


def _taper_leg(name, x, z, h, mat, r=0.022):
    F.cone(name, r * 0.75, r, h, (x, h / 2, z), mat, verts=12)


def _pillow(name, x, y, z, mat, turn=0, w=0.4, h=0.36):
    _cushion(name, w, h, 0.12, (x, y, z), mat, rot=(-13, 0, -turn))


def _run(up, cush, legs, w, arms=(), backs=('back',), depth=0.88, z=-0.04, n=None, pillows=()):
    """A sofa or a sectional's module `w` wide (catalog2.py `sofa_run`): a base, arms on the `arms` sides
    (-1 at -x), back rests along the back and/or a side ('left' at -x, 'right' at +x), seat and back
    cushions, pillows (x, depth, palette name, turn) and tapered legs. Returns the seat."""
    z0, z1 = z - depth / 2, z + depth / 2
    sx0, sx1, sz0 = -w / 2, w / 2, z0
    sides = [(-1 if b == 'left' else 1) for b in backs if b in ('left', 'right')]
    if 'back' in backs:
        sz0 = z0 + 0.2
    for s in sides:
        sx0, sx1 = (sx0 + 0.2, sx1) if s < 0 else (sx0, sx1 - 0.2)
    for s in arms:
        sx0, sx1 = (sx0 + 0.17, sx1) if s < 0 else (sx0, sx1 - 0.17)
    n = n or max(1, round((sx1 - sx0) / 0.9))
    cw = (sx1 - sx0) / n
    with G.new_parts() as still:
        F.box('base', w, 0.22, depth, (0, 0.21, z), up, bevel=0.04)
        if 'back' in backs:
            F.box('back', w, 0.5, 0.2, (0, 0.57, z0 + 0.1), up, bevel=0.06, bevel_segments=3)
        for s in sides:
            F.box(f'side_back{s}', 0.2, 0.5, depth, (s * (w / 2 - 0.1), 0.57, z), up, bevel=0.06, bevel_segments=3)
            _cushion(f'side_cushion{s}', 0.16, 0.4, depth - 0.24, (s * (w / 2 - 0.27), 0.62, z + 0.1), cush, rot=(0, -s * 9, 0))
        for s in arms:
            F.box(f'arm{s}', 0.17, 0.3, depth, (s * (w / 2 - 0.085), 0.47, z), up, bevel=0.07, bevel_segments=3)
        if 'back' in backs:
            for i in range(n):
                _cushion(f'back_cushion{i}', cw - 0.03, 0.4, 0.16, (sx0 + cw * (i + 0.5), 0.62, sz0 + 0.07), cush, rot=(-9, 0, 0))
        for i, (px, pz, mat, turn) in enumerate(pillows):
            _pillow(f'pillow{i}', px, 0.62, pz, getattr(M, mat), turn)
        for lx in (-w / 2 + 0.06, w / 2 - 0.06):
            for lz in (z0 + 0.06, z1 - 0.06):
                _taper_leg(f'leg{lx}{lz}', lx, lz, 0.1, legs)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for i in range(n):
            _cushion(f'seat{i}', cw - 0.02, 0.13, z1 - sz0 - 0.02, (sx0 + cw * (i + 0.5), 0.385, (sz0 + z1) / 2 + 0.005), cush)
        return 0.45
    return _seat(cushions, 0.32, (sz0 + z1) / 2)


def _settled(seat):
    G.settle(centre=False)  # against the wall, as built
    _sink(seat)


def sofa_classic():
    """The classic sofa (furniture_prims.py `sofa_classic`): a camel back in channels under a walnut
    rail, scroll arms with carved faces, an apron rail with a shell, cabriole legs; two seat cushions."""
    with G.new_parts() as still:
        F.box('deck', 1.7, 0.18, 0.775, (0, 0.27, -0.048), M.linen, bevel=0.05, bevel_segments=3)
        F.box('apron', 1.72, 0.05, 0.035, (0, 0.17, 0.33), M.walnut, bevel=0.012)
        F.sphere('shell', 0.04, (0, 0.17, 0.35), M.walnut, scale=(1, 0.4, 0.75), segments=12, rings=8)
        F.box('back_board', 1.7, 0.42, 0.1, (0, 0.53, -0.385), M.linen, bevel=0.04, bevel_segments=3)
        for i in range(7):
            u = (i + 0.5) / 7
            x = -0.84 + 1.68 * u
            h = _hump(u) - 0.32
            _cushion(f'channel{i}', 0.25, h, 0.17, (x, 0.32 + h / 2, -0.325), M.linen, rot=(-8, 0, 0))
        pts = [F.at(-0.84 + 1.68 * i / 14, _hump(i / 14) + 0.015, -0.375) for i in range(15)]
        for i in range(14):
            G.rod(f'rail{i}', pts[i], pts[i + 1], 0.02, M.walnut, verts=10)
        for i in range(15):
            F.sphere(f'rail_joint{i}', 0.02, (-0.84 + 1.68 * i / 14, _hump(i / 14) + 0.015, -0.375), M.walnut, segments=10, rings=6)
        for s in (-1, 1):
            ax = s * 0.84
            F.box(f'arm{s}', 0.16, 0.36, 0.735, (s * 0.83, 0.42, -0.048), M.linen, bevel=0.06, bevel_segments=3)
            F.cyl(f'scroll{s}', 0.1, 0.695, (ax, 0.6, -0.028), M.linen, rot=(90, 0, 0), verts=20)
            F.cyl(f'scroll_face{s}', 0.08, 0.03, (ax, 0.6, 0.335), M.walnut, rot=(90, 0, 0), verts=20, bevel=0.008)
            F.box(f'arm_post{s}', 0.05, 0.28, 0.035, (ax, 0.38, 0.335), M.walnut, bevel=0.015)
        _pillow('pillow_l', -0.6, 0.6, -0.215, M.pillow_rust, turn=-18, w=0.36, h=0.34)
        _pillow('pillow_r', 0.6, 0.6, -0.215, M.pillow_sage, turn=18, w=0.34, h=0.32)
        for x in (-0.82, 0.82):
            for z, dz in ((0.33, 1), (-0.38, -1)):
                _cabriole(f'leg{x}{z}', x, z, 0.18, M.walnut_legs, 0.025 * (1 if x > 0 else -1), 0.02 * dz)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for s in (-1, 1):
            _cushion(f'seat{s}', 0.78, 0.1, 0.595, (s * 0.4, 0.39, 0.033), M.linen_cushion)
        return 0.44
    _settled(_seat(cushions, 0.34, 0.033))


def _hump(u):
    """The camel back's top at `u` across (0 to 1)."""
    return 0.74 + 0.16 * math.exp(-((u - 0.5) / 0.22) ** 2)


def _cabriole(name, x, z, h, mat, sx, sz):
    """A cabriole leg from the frame at height `h` down to a ball foot: the knee out by (sx, sz), the
    ankle back in."""
    knee, ankle = F.at(x + sx, h * 0.62, z + sz), F.at(x + sx * 0.4, 0.05, z + sz * 0.4)
    G.rod(f'{name}_thigh', F.at(x, h, z), knee, 0.024, mat, verts=10)
    G.rod(f'{name}_shin', knee, ankle, 0.018, mat, verts=10)
    F.sphere(f'{name}_knee', 0.024, (x + sx, h * 0.62, z + sz), mat, segments=10, rings=6)
    F.sphere(f'{name}_foot', 0.026, (x + sx * 0.4, 0.026, z + sz * 0.4), mat, segments=10, rings=6)


def _box_sofa(up, cush, legs, w, pillows=()):
    """The loveseat (catalog.py `sofa`, box arms): a base on tapered legs, box arms, a back, two seat and
    two back cushions, `w` wide and 0.84 deep; pillows (x, material, turn) against the back."""
    z, depth, back_h = -0.04, 0.84, 0.82
    z0, z1 = z - depth / 2, z + depth / 2
    aw, bt = 0.17, 0.2
    sx0, sx1, sz0 = -w / 2 + aw, w / 2 - aw, z0 + bt
    cw = (sx1 - sx0) / 2
    with G.new_parts() as still:
        F.box('base', w, 0.22, depth, (0, 0.21, z), up, bevel=0.04)
        F.box('back', w, back_h - 0.38, bt, (0, (back_h - 0.08 + 0.3) / 2, z0 + bt / 2), up, bevel=0.06, bevel_segments=3)
        for s in (-1, 1):
            F.box(f'arm{s}', aw, 0.52, depth, (s * (w / 2 - aw / 2), 0.36, z), up, bevel=0.07, bevel_segments=3)
        for i in range(2):
            _cushion(f'back_cushion{i}', cw - 0.025, back_h - 0.44, 0.16, (sx0 + cw * (i + 0.5), (back_h + 0.44) / 2 + 0.01, sz0 + 0.07),
                     cush, rot=(-9, 0, 0))
        for i, (px, mat, turn) in enumerate(pillows):
            _pillow(f'pillow{i}', px, 0.64, sz0 + 0.14, mat, turn)
        for lx in (-w / 2 + 0.07, w / 2 - 0.07):
            for lz in (z0 + 0.07, z1 - 0.07):
                _taper_leg(f'leg{lx}{lz}', lx, lz, 0.1, legs)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for i in range(2):
            _cushion(f'seat{i}', cw - 0.015, 0.13, z1 - sz0 - 0.02, (sx0 + cw * (i + 0.5), 0.385, (sz0 + z1) / 2 + 0.005), cush)
        return 0.45
    return _seat(cushions, 0.32, (sz0 + z1) / 2)


def loveseat():
    _settled(_box_sofa(M.sage, M.sage_cushion, M.oak_legs, 1.84, pillows=[(-0.55, M.pillow_cream, 10)]))


def sofa_three():
    """The three-seat sofa (catalog.py `sofa3`): a deep base, three seat and three back cushions, two
    pillows and a throw folded over the right-hand seat."""
    with G.new_parts() as still:
        F.box('base', 2.82, 0.2, 0.86, (0, 0.22, -0.03), M.stone, bevel=0.05)
        F.box('back', 2.82, 0.5, 0.22, (0, 0.62, -0.35), M.stone, bevel=0.08, bevel_segments=3)
        for s in (-1, 1):
            F.box(f'arm{s}', 0.2, 0.32, 0.86, (s * 1.31, 0.48, -0.03), M.stone, bevel=0.08, bevel_segments=3)
        for i, x in enumerate((-0.8, 0.0, 0.8)):
            _cushion(f'back_cushion{i}', 0.74, 0.4, 0.16, (x, 0.62, -0.2), M.stone_cushion, rot=(-10, 0, 0))
        _pillow('pillow_l', -0.98, 0.65, -0.12, M.pillow_sage, turn=10, h=0.34)
        _pillow('pillow_r', 0.98, 0.65, -0.12, M.pillow_sage, turn=-10, h=0.34)
        for x in (-1.32, 1.32):
            for z in (-0.38, 0.32):
                _taper_leg(f'leg{x}{z}', x, z, 0.12, M.oak_dark_legs, r=0.025)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for i, x in enumerate((-0.8, 0.0, 0.8)):
            _cushion(f'seat{i}', 0.78, 0.12, 0.62, (x, 0.38, 0.06), M.stone_cushion)
        F.box('throw', 0.62, 0.025, 0.5, (0.86, 0.452, 0.04), M.throw_sage, rot=(0, 0, 4), bevel=0.01)
        return 0.44
    _settled(_seat(cushions, 0.32, 0.06))


def sofa_design():
    """The designer sofa (catalog.py `design_sofa`): a slim deck on splayed brass legs, a tilted back with
    buttons, slim arms with brass caps and a brass rail along the front."""
    z = -0.04
    with G.new_parts() as still:
        F.box('deck', 1.86, 0.1, 0.8, (0, 0.27, z), M.teal, bevel=0.03)
        F.box('back', 1.86, 0.4, 0.14, (0, 0.52, z - 0.33), M.teal, rot=(-8, 0, 0), bevel=0.05, bevel_segments=3)
        F.box('front_rail', 1.8, 0.03, 0.03, (0, 0.22, z + 0.38), M.brass, bevel=0.008)
        for s in (-1, 1):
            F.box(f'arm{s}', 0.1, 0.32, 0.8, (s * 0.88, 0.38, z), M.teal, bevel=0.04, bevel_segments=3)
            F.box(f'arm_cap{s}', 0.11, 0.025, 0.7, (s * 0.88, 0.555, z + 0.02), M.brass, bevel=0.008)
            for lz in (z - 0.3, z + 0.3):
                foot = lz + (0.03 if lz > z else -0.03)
                G.rod(f'leg{s}{lz}', F.at(s * 0.8, 0.0, foot), F.at(s * 0.78, 0.23, lz), 0.018, M.brass_legs, verts=10)
        for x in (-0.42, 0.42):
            for bx in (-0.2, 0.0, 0.2):
                F.sphere(f'button{x}{bx}', 0.014, (x + bx, 0.58, z - 0.255), M.teal, scale=(1, 0.6, 1), segments=8, rings=6)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for x in (-0.42, 0.42):
            _cushion(f'seat{x}', 0.82, 0.11, 0.66, (x, 0.37, z + 0.06), M.teal_cushion)
        return 0.425
    _settled(_seat(cushions, 0.315, z + 0.06))


def sofa_three_seat():
    _settled(_run(M.moss, M.moss_cushion, M.oak_legs, 2.86, arms=(-1, 1), n=3,
                  pillows=[(-1.0, -0.08, 'pillow_cream', 14), (1.0, -0.08, 'pillow_mustard', -14)]))


def sectional(arms=(), backs=('back',), **kw):
    return lambda: _settled(_run(M.greige, M.greige_cushion, M.espresso_legs, 1.0, arms=arms, backs=backs, **kw))


def chaise():
    """The sectional's chaise (1 x 2): a long seat with a back at the wall and an arm along +x, a throw
    over its foot."""
    with G.new_parts() as still:
        F.box('base', 0.96, 0.22, 1.9, (0, 0.21, 0.0), M.greige, bevel=0.04)
        F.box('back', 0.96, 0.5, 0.2, (0, 0.57, -0.85), M.greige, bevel=0.06, bevel_segments=3)
        F.box('arm', 0.17, 0.3, 1.9, (0.395, 0.47, 0.0), M.greige, bevel=0.07, bevel_segments=3)
        _cushion('back_cushion', 0.74, 0.4, 0.16, (-0.08, 0.62, -0.68), M.greige_cushion, rot=(-9, 0, 0))
        for lx in (-0.42, 0.42):
            for lz in (-0.88, 0.88):
                _taper_leg(f'leg{lx}{lz}', lx, lz, 0.1, M.espresso_legs)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        _cushion('seat', 0.76, 0.13, 1.66, (-0.08, 0.385, 0.1), M.greige_cushion)
        F.box('throw', 0.5, 0.03, 0.6, (-0.06, 0.464, 0.55), M.throw_oat, bevel=0.012)
        return 0.45
    _settled(_seat(cushions, 0.32, 0.1))


# The wall bed folded, as the placeholder draws it (the fallback when the file fails to load).
_D, _MID = CAB_FRONT - BACK, (BACK + CAB_FRONT) / 2
PLACEHOLDER = [
    {'shape': 'box', 'size': [1.66, 2.17, _D], 'at': [0, 1.085, _MID], 'color': '#E9E4DA', 'material': 'material.finish.satin'},
    {'shape': 'box', 'size': [1.98, 0.05, _D + 0.03], 'at': [0, 2.195, _MID + 0.015], 'color': '#E9E4DA', 'material': 'material.finish.satin'},
    {'shape': 'box', 'size': [0.15, 2.17, _D], 'at': [-0.905, 1.085, _MID], 'color': '#D8D1C4', 'material': 'material.finish.matte'},
    {'shape': 'box', 'size': [0.15, 2.17, _D], 'at': [0.905, 1.085, _MID], 'color': '#D8D1C4', 'material': 'material.finish.matte'},
    {'shape': 'box', 'size': [1.36, 0.12, 0.12], 'at': [0, 2.0, CAB_FRONT + 0.04], 'color': '#E9E4DA', 'material': 'material.finish.satin'},
    {'shape': 'box', 'size': [0.5, 0.03, 0.04], 'at': [0, 1.45, CAB_FRONT + 0.04], 'color': '#8D8F92', 'material': 'material.finish.metal'},
]

SECTIONAL = [('greige', 'Upholstery'), ('greige_cushion', 'Cushions'), ('espresso_legs', 'Legs')]

FAMILY = G.Family(
    'living', M,
    [G.Model('wallBed', wall_bed, key='model.wallBed', size=(1.98, 2.22, 0.6), centre=(0, -1.135),
             # Down while the bed is open (the sim's `open`), from the moment a use that opens it
             # starts (`uses`, so it doesn't wait for the house to be redrawn).
             motions={'uses': {'sleep': 'bed_down', 'nap': 'bed_down', 'foldDown': 'bed_down'}, 'open': 'bed_down'},
             # The books keep their own colours.
             paint=[('body', 'Cabinet'), ('frame', 'Bed frame'), ('mattress', 'Mattress'), ('pillow', 'Pillows'),
                    ('throw', 'Throw'), ('inside', 'Inside & shelves'), ('metal', 'Handle')],
             placeholder=PLACEHOLDER),
     G.Model('sofaBed', sofa_bed, key='model.sofaBed', size=(1.86, 0.83, 0.83), centre=(0, -0.015),
             # Out while the sofa bed is open, from the moment a use that opens it starts.
             motions={'uses': {'doze': 'bed_out', 'pullOut': 'bed_out'}, 'open': 'bed_out'},
             paint=[('upholstery', 'Upholstery'), ('cushion', 'Cushions'), ('legs', 'Legs'), ('bedding', 'Bedding'),
                    ('pillow_rose', 'Pillow'), ('pillow_cream', 'Second pillow')]),
     # L1: the sofas and sectionals, their seat cushions sinking while sat or dozed on
     G.Model('sofa', sofa_classic, key='model.sofa', size=(1.88, 0.935, 0.817), centre=(0, -0.035),
             motions={'uses': {'relax': 'sink_in'}},
             paint=[('linen', 'Upholstery'), ('linen_cushion', 'Cushions'), ('walnut', 'Frame'), ('walnut_legs', 'Legs'),
                    ('pillow_rust', 'Pillow'), ('pillow_sage', 'Second pillow')]),
     G.Model('loveseat', loveseat, key='model.loveseat', size=(1.84, 0.833, 0.84), centre=(0, -0.04),
             motions={'uses': SOFA_USES},
             paint=[('sage', 'Upholstery'), ('sage_cushion', 'Cushions'), ('oak_legs', 'Legs'), ('pillow_cream', 'Pillow')]),
     G.Model('sofaThree', sofa_three, key='model.sofaThree', size=(2.82, 0.87, 0.86), centre=(0, -0.03),
             motions={'uses': SOFA_USES},
             paint=[('stone', 'Upholstery'), ('stone_cushion', 'Cushions'), ('oak_dark_legs', 'Legs'), ('pillow_sage', 'Pillows'),
                    ('throw_sage', 'Throw')]),
     G.Model('sofaDesign', sofa_design, key='model.sofaDesign', size=(1.87, 0.72, 0.82), centre=(0, -0.05),
             motions={'uses': SOFA_USES},
             paint=[('teal', 'Upholstery'), ('teal_cushion', 'Cushions'), ('brass', 'Frame'), ('brass_legs', 'Legs')]),
     G.Model('sofaThreeSeat', sofa_three_seat, key='model.sofaThreeSeat', size=(2.86, 0.823, 0.88), centre=(0, -0.04),
             motions={'uses': SOFA_USES},
             paint=[('moss', 'Upholstery'), ('moss_cushion', 'Cushions'), ('oak_legs', 'Legs'), ('pillow_cream', 'Pillow'),
                    ('pillow_mustard', 'Second pillow')]),
     G.Model('sectionalMiddle', sectional(), key='model.sectionalMiddle', size=(1.0, 0.823, 0.88), centre=(0, -0.04),
             motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalArmLeft', sectional(arms=(1,)), key='model.sectionalArmLeft', size=(1.0, 0.823, 0.88), centre=(0, -0.04),
             motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalArmRight', sectional(arms=(-1,)), key='model.sectionalArmRight', size=(1.0, 0.823, 0.88),
             centre=(0, -0.04), motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalCorner', sectional(backs=('back', 'right'), depth=0.96, z=-0.02, pillows=[(0.1, -0.1, 'pillow_mustard', -35)]),
             key='model.sectionalCorner', size=(1.0, 0.823, 0.96), centre=(0, -0.02),
             motions={'uses': SOFA_USES}, paint=SECTIONAL + [('pillow_mustard', 'Pillow')]),
     G.Model('chaise', chaise, key='model.chaise', size=(0.96, 0.823, 1.9), centre=(0, 0.0),
             motions={'uses': SOFA_USES}, paint=SECTIONAL + [('throw_oat', 'Throw')])],
    comment='Blender-built living room models (tools/art/blender/families/living.py). New objects with a '
            'Blender model; the placeholder is drawn if a file fails to load, and the materials map onto the '
            'game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
