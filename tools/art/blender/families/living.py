# The living room: a fold-down wall bed (`model.wallBed`, content `wallBed` in catalog2/living.json), a
# sofa bed (`model.sofaBed`, content `sofaBed`), the L1 sofas and sectionals, the L2 armchairs and small
# seats and the L3 coffee and side tables (notes: living.md).
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
# The armchairs too, but for the recliner (it reclines), the rocking chair (it rocks) and the bean bag.
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
    # L2, the armchairs and small seats
    forest=('#2F5A4E', 'fabric'),        # the wingback
    forest_cushion=('#2F5A4E', 'fabric'),
    walnut_warm=('#7A5234', 'wood'),     # the wingback's, the pouf's and the ottoman's legs
    tan=('#7A5A44', 'fabric'),           # the recliner
    tan_cushion=('#7A5A44', 'fabric'),
    black_satin=('#2B2C2E', 'satin'),    # the designer lounge chair's glides
    leather=('#2B2826', 'satin'),        # the leather armchair's cushions
    teak=('#9A6B45', 'wood'),            # and its open frame
    ply=('#6B4630', 'wood'),             # the designer lounge chair's shells
    hide=('#5A3A2A', 'satin'),           # its cushions
    alu=('#C9CBCC', 'metal'),            # its swivel base
    rocker=('#8A5A36', 'wood'),          # the rocking chair
    rocker_cushion=('#C9B79C', 'fabric'),
    pouf_leather=('#3A2E2A', 'satin'),   # the leather pouf
    pouf_button=('#2A201D', 'satin'),
    navy=('#3E5470', 'fabric'),          # the tufted ottoman
    navy_button=('#2E3F56', 'fabric'),
    mustard=('#D6A64A', 'fabric'),       # the floor cushion
    mustard_piping=('#C9963D', 'fabric'),
    mustard_button=('#B5853A', 'fabric'),
    bag=('#C25B3E', 'fabric'),           # the bean bag
    bag_seat=('#B5523A', 'fabric'),
    # L3, the coffee and side tables (and what stands on them: plain, so it costs no draw call)
    oak=('#CDB08A', 'wood'),
    glass=('#DCE6E8', 'glass'),
    chrome=('#D0D3D6', 'chrome'),
    marble=('#EEECE8', 'gloss'),
    marble_vein=('#C9CBCC', 'gloss'),
    cream_gloss=('#E8E2D6', 'gloss'),
    white_gloss=('#F1F0EC', 'gloss'),
    stone_top=('#E7E3DC', 'gloss'),
    black_metal=('#2B2C2E', 'metal'),
    basket=('#C9A979', 'fabric'),
    magazine_teal=('#4E7A7F', 'matte'),
    magazine_navy=('#3E5470', 'matte'),
    box_sand=('#C9B79C', 'matte'),
    vase=('#E8E6E1', 'gloss'),
    pot_white=('#F2F2F0', 'gloss'),
    plant=('#5E8F4E', 'satin'),
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


def _run(up, cush, legs, w, arms=(), backs=('back',), depth=0.88, z=-0.04, n=None, pillows=(), extra=None):
    """A sofa or a sectional's module `w` wide (catalog2.py `sofa_run`): a base, arms on the `arms` sides
    (-1 at -x), back rests along the back and/or a side ('left' at -x, 'right' at +x), seat and back
    cushions, pillows (x, depth, palette name, turn) and tapered legs; `extra()` adds to the still part.
    Returns the seat."""
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
        if extra:
            extra()
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for i in range(n):
            _cushion(f'seat{i}', cw - 0.02, 0.13, z1 - sz0 - 0.02, (sx0 + cw * (i + 0.5), 0.385, (sz0 + z1) / 2 + 0.005), cush)
        return 0.45
    return _seat(cushions, 0.32, (sz0 + z1) / 2)


def _settled(seat):
    G.settle()  # on its Model's `centre`: built in the placeholder's numbers, its back moved onto the wall face
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


def _box_sofa(up, cush, legs, w, pillows=(), n=2):
    """The loveseat and the lounge chair (catalog.py `sofa`, box arms): a base on tapered legs, box arms, a
    back, `n` seat and back cushions, `w` wide and 0.84 deep; pillows (x, material, turn) against the back."""
    z, depth, back_h = -0.04, 0.84, 0.82
    z0, z1 = z - depth / 2, z + depth / 2
    aw, bt = 0.17, 0.2
    sx0, sx1, sz0 = -w / 2 + aw, w / 2 - aw, z0 + bt
    cw = (sx1 - sx0) / n
    with G.new_parts() as still:
        F.box('base', w, 0.22, depth, (0, 0.21, z), up, bevel=0.04)
        F.box('back', w, back_h - 0.38, bt, (0, (back_h - 0.08 + 0.3) / 2, z0 + bt / 2), up, bevel=0.06, bevel_segments=3)
        for s in (-1, 1):
            F.box(f'arm{s}', aw, 0.52, depth, (s * (w / 2 - aw / 2), 0.36, z), up, bevel=0.07, bevel_segments=3)
        for i in range(n):
            _cushion(f'back_cushion{i}', cw - 0.025, back_h - 0.44, 0.16, (sx0 + cw * (i + 0.5), (back_h + 0.44) / 2 + 0.01, sz0 + 0.07),
                     cush, rot=(-9, 0, 0))
        for i, (px, mat, turn) in enumerate(pillows):
            _pillow(f'pillow{i}', px, 0.64, sz0 + 0.14, mat, turn)
        for lx in (-w / 2 + 0.07, w / 2 - 0.07):
            for lz in (z0 + 0.07, z1 - 0.07):
                _taper_leg(f'leg{lx}{lz}', lx, lz, 0.1, legs)
    G.join(still, 'sofa', origin=(0, 0, 0))

    def cushions():
        for i in range(n):
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


# L2, the armchairs and small seats. Built as the sofas are; the seats sink in (`sink_in`) but for the
# recliner (`recline`), the rocking chair (`rock`), the bean bag (`squash`) and the designer chair (still).
def armchair_bergere():
    """The armchair (furniture_prims.py `armchair_bergere`): an arched back in channels inside a walnut
    frame, padded arms on open walnut supports, cabriole legs; its seat cushion sinks in."""
    zb = -0.445

    def top(u):
        return 0.68 + 0.26 * math.sqrt(max(0.0, 1 - ((u - 0.5) / 0.56) ** 2))
    with G.new_parts() as still:
        F.box('deck', 0.66, 0.16, 0.715, (0, 0.26, -0.058), M.linen, bevel=0.05, bevel_segments=3)
        F.box('apron', 0.68, 0.05, 0.035, (0, 0.17, 0.3), M.walnut, bevel=0.012)
        for i in range(6):
            u = (i + 0.5) / 6
            h = top(u) - 0.3
            F.box(f'channel{i}', 0.11, h, 0.12, (-0.3 + 0.6 * u, 0.3 + h / 2, -0.33), M.linen, rot=(-10, 0, 0), bevel=0.04)
        pts = [(-0.34, 0.42, zb + 0.055)] + [(-0.34 + 0.68 * i / 12, top(i / 12) + 0.025, zb + 0.06) for i in range(13)] \
            + [(0.34, 0.42, zb + 0.055)]
        for i in range(len(pts) - 1):
            G.rod(f'frame{i}', F.at(*pts[i]), F.at(*pts[i + 1]), 0.0175, M.walnut, verts=8)
        for s in (-1, 1):
            ax = s * 0.32
            _cushion(f'arm_pad{s}', 0.1, 0.06, 0.605, (ax, 0.625, -0.022), M.linen)
            F.box(f'arm_rail{s}', 0.06, 0.03, 0.685, (ax, 0.585, -0.003), M.walnut, bevel=0.012)
            F.sphere(f'arm_scroll{s}', 0.032, (ax, 0.59, 0.33), M.walnut, scale=(1.1, 0.95, 0.8), segments=10, rings=6)
            G.rod(f'arm_post{s}', F.at(ax, 0.3, 0.32), F.at(ax, 0.58, 0.3), 0.0225, M.walnut, verts=10)
            F.box(f'side{s}', 0.05, 0.25, 0.545, (ax, 0.45, -0.052), M.linen, bevel=0.02)
        _pillow('pillow', 0, 0.6, -0.245, M.pillow_sage, w=0.34, h=0.3)
        for x in (-0.31, 0.31):
            for z, dz in ((0.3, 1), (-0.385, -1)):
                _cabriole(f'leg{x}{z}', x, z, 0.18, M.walnut_legs, 0.02 * (1 if x > 0 else -1), 0.02 * dz)
    G.join(still, 'chair', origin=(0, 0, 0))

    def cushions():
        _cushion('seat', 0.58, 0.1, 0.575, (0, 0.37, 0.013), M.linen_cushion)
        return 0.42
    _settled(_seat(cushions, 0.32, 0.013))


def armchair_lounge():
    _settled(_box_sofa(M.sage, M.sage_cushion, M.oak_legs, 0.82, n=1))


def armchair_leather():
    """The leather armchair (catalog.py): black leather cushions in an open teak frame, the front legs
    running up into the arms."""
    with G.new_parts() as still:
        _cushion('back_cushion', 0.58, 0.5, 0.12, (0, 0.69, -0.3), M.leather, rot=(-14, 0, 0))
        F.box('front_rail', 0.64, 0.05, 0.04, (0, 0.3, 0.34), M.teak, bevel=0.01)
        F.box('back_rail', 0.6, 0.04, 0.04, (0, 0.3, -0.33), M.teak, bevel=0.01)
        F.box('top_rail', 0.66, 0.05, 0.04, (0, 0.92, -0.435), M.teak, bevel=0.012)
        for s in (-1, 1):
            x = s * 0.33
            G.rod(f'front_leg{s}', F.at(x, 0, 0.37), F.at(x, 0.62, 0.32), 0.0225, M.teak, verts=10)
            G.rod(f'back_leg{s}', F.at(x, 0, -0.38), F.at(x, 0.3, -0.33), 0.0225, M.teak, verts=10)
            G.rod(f'back_post{s}', F.at(x, 0.3, -0.33), F.at(x, 0.94, -0.44), 0.02, M.teak, verts=10)
            F.box(f'arm{s}', 0.07, 0.035, 0.64, (x, 0.635, -0.01), M.teak, bevel=0.012)
            F.box(f'side_rail{s}', 0.04, 0.05, 0.68, (x, 0.3, 0.0), M.teak, bevel=0.01)
    G.join(still, 'chair', origin=(0, 0, 0))

    def cushions():
        _cushion('seat', 0.58, 0.12, 0.6, (0, 0.38, 0.03), M.leather)
        return 0.44
    _settled(_seat(cushions, 0.32, 0.03))


def armchair_design():
    """The designer lounge chair (catalog.py): walnut shells with leather cushions on a five-star swivel
    base. Still."""
    F.cyl('column', 0.04, 0.2, (0, 0.17, 0.0), M.alu, verts=16)
    F.box('seat_shell', 0.66, 0.04, 0.6, (0, 0.32, 0.02), M.ply, rot=(-5, 0, 0), bevel=0.015)
    _cushion('seat', 0.62, 0.1, 0.58, (0, 0.39, 0.03), M.hide, rot=(-5, 0, 0))
    F.box('back_shell', 0.66, 0.42, 0.04, (0, 0.6, -0.36), M.ply, rot=(-18, 0, 0), bevel=0.015)
    _cushion('back', 0.6, 0.38, 0.11, (0, 0.62, -0.3), M.hide, rot=(-18, 0, 0))
    F.box('head_shell', 0.6, 0.2, 0.04, (0, 0.86, -0.43), M.ply, rot=(-12, 0, 0), bevel=0.015)
    _cushion('head', 0.56, 0.17, 0.1, (0, 0.86, -0.38), M.hide, rot=(-12, 0, 0))
    for s in (-1, 1):
        _cushion(f'arm_pad{s}', 0.08, 0.05, 0.4, (s * 0.34, 0.57, -0.02), M.hide)
        G.rod(f'arm_front{s}', F.at(s * 0.31, 0.34, 0.08), F.at(s * 0.34, 0.54, 0.02), 0.0125, M.alu, verts=8)
        G.rod(f'arm_back{s}', F.at(s * 0.31, 0.4, -0.26), F.at(s * 0.34, 0.54, -0.18), 0.0125, M.alu, verts=8)
    for i in range(5):
        a = math.radians(i * 72 + 90)
        tip = (math.cos(a) * 0.36, 0.03, math.sin(a) * 0.36)
        G.rod(f'star{i}', F.at(0, 0.08, 0), F.at(*tip), 0.016, M.alu, verts=8)
        F.cyl(f'glide{i}', 0.025, 0.02, (tip[0], 0.01, tip[2]), M.black_satin, verts=10)
    G.join_all('armchairDesign')
    G.settle()  # on its Model's `centre`, its back on the wall face


def armchair_wingback():
    """The wingback armchair (catalog2.py): a sofa_run seat with arms, wings either side of a tall back."""
    def wings():
        for s in (-1, 1):
            F.box(f'wing{s}', 0.16, 0.4, 0.3, (s * 0.34, 0.86, -0.3), M.forest, bevel=0.06, bevel_segments=3)
        F.box('head', 0.84, 0.3, 0.18, (0, 0.95, -0.37), M.forest, bevel=0.07, bevel_segments=3)
    _settled(_run(M.forest, M.forest_cushion, M.walnut_warm, 0.84, arms=(-1, 1), depth=0.8, z=-0.06, extra=wings))


# The recliner: its seat and back tip back 25° about the sitter's hips (`recline`), carrying them (`ride`),
# and its footrest swings up 80° from the front of the base (`footrest_up`), for relax and doze.
# It stands 13 cm off the wall's face, so the back's top stays out of the wall when tipped.
RC_Z0, RC_Z1 = -0.30, 0.40   # the base, back to front
RC_HIPS = (0.44, 0.12)      # (height, depth): content seat 0.44, forward 0.12


def recliner():
    w, aw = 0.92, 0.17
    mid = (RC_Z0 + RC_Z1) / 2
    with G.new_parts() as still:
        F.box('base', w, 0.3, RC_Z1 - RC_Z0, (0, 0.17, mid), M.tan, bevel=0.04)   # skirted to the floor
        F.box('plinth', w - 0.04, 0.02, RC_Z1 - RC_Z0 - 0.04, (0, 0.01, mid), M.tan)
        for s in (-1, 1):
            F.box(f'arm{s}', aw, 0.3, RC_Z1 - RC_Z0, (s * (w / 2 - aw / 2), 0.47, mid), M.tan, bevel=0.07, bevel_segments=3)
    G.join(still, 'chair', origin=(0, 0, 0))
    sw = w - 2 * aw
    with G.new_parts() as body:   # the seat cushion and the back, which tip together
        _cushion('seat', sw - 0.02, 0.13, RC_Z1 - RC_Z0 - 0.22, (0, 0.385, (RC_Z0 + 0.2 + RC_Z1) / 2), M.tan_cushion)
        F.box('back', w - 2 * aw + 0.02, 0.5, 0.18, (0, 0.57, RC_Z0 + 0.1), M.tan, bevel=0.06, bevel_segments=3)
        _cushion('back_cushion', sw - 0.03, 0.42, 0.16, (0, 0.63, RC_Z0 + 0.25), M.tan_cushion, rot=(-9, 0, 0))
    seat = G.join(body, 'seat', origin=F.at(0, *RC_HIPS))
    # the footrest, built up (open), hinged at the front of the base
    hinge = (0.3, RC_Z1 - 0.09)   # folded down, its padded face stands 2 cm proud of the base's front
    with G.new_parts() as foot:
        _cushion('footrest', sw - 0.04, 0.11, 0.26, (0, hinge[0] + 0.03, hinge[1] + 0.14), M.tan_cushion)
    footrest = G.hinged(G.join(foot, 'footrest'), F.at(0, *hinge))
    G.turn(footrest, 90, 'X')   # folded down flat against the front of the base at rest
    G.settle(centre=False)
    G.swing(seat, 'recline', -25, seconds=1.0, axis='X')
    G.swing(footrest, 'footrest_up', -80, seconds=1.0, axis='X')


# The rocking chair rocks ±6° on its rockers (`rock`, a loop while sat in), the sitter with it (`ride`). It
# turns about the rockers' centre of curvature (1 m radius), so the rockers roll on the floor.
RK_R, RK_Z = 1.0, 0.05


def rocking_chair():
    with G.new_parts() as chair:
        F.box('seat_board', 0.5, 0.04, 0.46, (0, 0.44, 0.04), M.rocker, bevel=0.012)
        _cushion('seat_cushion', 0.44, 0.05, 0.4, (0, 0.48, 0.05), M.rocker_cushion)
        for x in (-0.22, 0.22):
            pts = [(x, RK_R + 0.025 - RK_R * math.cos(math.radians(a)), RK_Z + RK_R * math.sin(math.radians(a)))
                   for a in [-26 + i * 52 / 8 for i in range(9)]]
            for i in range(8):   # a flat bent strip, 3 cm thick and 4.5 cm deep
                (_, h0, d0), (_, h1, d1) = pts[i], pts[i + 1]
                F.box(f'rocker{x}{i}', 0.03, 0.045, math.hypot(d1 - d0, h1 - h0) + 0.006, (x, (h0 + h1) / 2, (d0 + d1) / 2),
                      M.rocker, rot=(math.degrees(math.atan2(-(h1 - h0), d1 - d0)), 0, 0))
            G.rod(f'front_leg{x}', F.at(x, 0.06, 0.26), F.at(x, 0.44, 0.22), 0.0175, M.rocker, verts=8)
            G.rod(f'back_leg{x}', F.at(x, 0.06, -0.18), F.at(x * 1.05, 1.02, -0.3), 0.0175, M.rocker, verts=8)
            F.box(f'arm{x}', 0.05, 0.03, 0.42, (x * 1.08, 0.66, 0.02), M.rocker, bevel=0.01)
            G.rod(f'arm_post{x}', F.at(x * 1.08, 0.44, 0.2), F.at(x * 1.08, 0.66, 0.2), 0.015, M.rocker, verts=8)
        for i in range(6):
            xx = -0.15 + i * 0.06
            G.rod(f'spindle{i}', F.at(xx, 0.46, -0.17), F.at(xx, 0.98, -0.28), 0.01, M.rocker, verts=8)
        F.box('crest', 0.48, 0.07, 0.04, (0, 1.0, -0.29), M.rocker, rot=(-12, 0, 0), bevel=0.015)
    chair = G.join(chair, 'chair', origin=F.at(0, RK_R + 0.025, RK_Z))
    G.settle(centre=False)
    G.rock(chair, 'rock', 6, seconds=1.4, axis='X')


def pouf():
    """The leather pouf (catalog.py): buttoned dark leather on short legs; the buttoned top sinks in."""
    with G.new_parts() as still:
        _cushion('body', 0.72, 0.34, 0.5, (0, 0.26, 0), M.pouf_leather)
        for x in (-0.3, 0.3):
            for z in (-0.19, 0.19):
                _taper_leg(f'leg{x}{z}', x, z, 0.09, M.walnut_warm)
    G.join(still, 'pouf', origin=(0, 0, 0))

    def cushions():
        _cushion('top', 0.7, 0.05, 0.48, (0, 0.435, 0), M.pouf_leather)
        for x in (-0.2, 0.0, 0.2):
            for z in (-0.1, 0.1):
                F.sphere(f'button{x}{z}', 0.015, (x, 0.46, z), M.pouf_button, scale=(1, 1, 0.65), segments=8, rings=6)
        return 0.46
    _settled(_seat(cushions, 0.41, 0))


def ottoman():
    """The tufted ottoman (catalog2.py): a navy drum on little walnut feet, its buttoned top sinking in."""
    with G.new_parts() as still:
        F.cyl('drum', 0.31, 0.36, (0, 0.22, 0), M.navy, verts=32, bevel=0.03, bevel_segments=3)
        for x, z in ((-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2)):
            F.cyl(f'foot{x}{z}', 0.025, 0.04, (x, 0.02, z), M.walnut_warm, verts=12)
    G.join(still, 'ottoman', origin=(0, 0, 0))

    def cushions():
        F.cyl('top', 0.3, 0.06, (0, 0.41, 0), M.navy, verts=32, bevel=0.025, bevel_segments=3)
        for a in range(0, 360, 60):
            r = math.radians(a)
            F.sphere(f'button{a}', 0.0125, (math.cos(r) * 0.15, 0.44, math.sin(r) * 0.15), M.navy_button, scale=(1, 1, 0.8), segments=8, rings=6)
        return 0.44
    _settled(_seat(cushions, 0.38, 0))


def floor_cushion():
    """The floor cushion (catalog2.py): a round mustard cushion with piping and a button; all of it sinks."""
    def cushions():
        F.cyl('cushion', 0.31, 0.12, (0, 0.06, 0), M.mustard, verts=32, bevel=0.04, bevel_segments=3)
        G.torus('piping', 0.285, 0.025, at=F.at(0, 0.1, 0), material=M.mustard_piping, major_segments=32, minor_segments=8)
        F.sphere('button', 0.02, (0, 0.125, 0), M.mustard_button, scale=(1, 1, 0.7), segments=10, rings=6)
        return 0.145
    _settled(_seat(cushions, 0.0, 0))


def bean_bag():
    """The bean bag (catalog2.py): a slumped bag with a raised back and a dent to sit in; it flattens 15 %
    when flopped into (`squash`)."""
    with G.new_parts() as parts:
        F.sphere('bag', 0.42, (0, 0.23, 0.02), M.bag, scale=(1, 1, 0.46 / 0.84), segments=28, rings=16)
        F.sphere('back', 0.36, (0, 0.42, -0.2), M.bag, scale=(1, 0.4 / 0.72, 0.56 / 0.72), rot=(-12, 0, 0), segments=24, rings=14)
        F.sphere('dent', 0.25, (0, 0.42, 0.05), M.bag_seat, scale=(1, 0.4 / 0.5, 0.12 / 0.5), segments=20, rings=10)
    bag = G.join(parts, 'bag', origin=F.at(0, 0, 0.02))
    G.settle(centre=False)
    G.squash(bag, 'squash', (1.06, 1.06, 0.85), seconds=0.6)


# L3, the coffee and side tables: still, but for the coffee table's drawer (`drawer_open`, a magazine in it,
# for the magazine use). Their tops stay at the content `surface` heights (catalog2.py `TOPS`).
def _legs4(xs, zs, h, t, mat, y0=0.0):
    for x in xs:
        for z in zs:
            F.box(f'leg{x}{z}', t, h, t, (x, y0 + h / 2, z), mat, bevel=min(0.006, t / 4))


def coffee_table():
    """The coffee table (catalog.py `coffee_table`, classic): an oak top at 0.42 on tapered legs, aprons and
    a slatted shelf; a drawer in the front apron."""
    t, h = 0.04, 0.38
    with G.new_parts() as still:
        F.box('top', 1.3, t, 0.74, (0, 0.42 - t / 2, 0), M.oak, bevel=0.012)
        F.box('apron_back', 1.12, 0.06, 0.02, (0, h - 0.03, -0.28), M.oak, bevel=0.005)
        for s in (-1, 1):
            F.box(f'apron_front{s}', 0.3, 0.06, 0.02, (s * 0.41, h - 0.03, 0.28), M.oak, bevel=0.005)
            F.box(f'runner{s}', 0.02, 0.06, 0.56, (s * 0.26, h - 0.03, 0.0), M.oak)
        for x in (-0.58, 0.58):
            for z in (-0.3, 0.3):
                F.cone(f'leg{x}{z}', 0.016, 0.025, h, (x, h / 2, z), M.oak, verts=12)
        for z in (-0.2, -0.07, 0.07, 0.2):
            F.box(f'slat{z}', 1.1, 0.02, 0.09, (0, 0.12, z), M.oak, bevel=0.004)
    G.join(still, 'table', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.box('drawer_front', 0.5, 0.055, 0.02, (0, h - 0.03, 0.28), M.oak, bevel=0.004)
        F.sphere('knob', 0.014, (0, h - 0.03, 0.296), M.brass, segments=10, rings=6)
        F.box('drawer_bottom', 0.48, 0.01, 0.5, (0, h - 0.055, 0.02), M.oak)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.01, 0.045, 0.5, (s * 0.235, h - 0.03, 0.02), M.oak)
        F.box('drawer_back', 0.48, 0.045, 0.01, (0, h - 0.03, -0.225), M.oak)
        F.box('magazine', 0.3, 0.012, 0.22, (0.05, h - 0.044, 0.08), M.magazine_teal, rot=(0, 0, 8), bevel=0.002)
    drawer = G.join(parts, 'drawer')
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=0.8)


def coffee_table_glass():
    """The glass coffee table (catalog.py `glass_table`): a glass top and shelf in a chrome frame, a plant and
    a vase."""
    F.box('top', 1.3, 0.02, 0.74, (0, 0.41, 0), M.glass, bevel=0.008)
    F.box('shelf', 1.2, 0.012, 0.64, (0, 0.12, 0), M.glass, bevel=0.005)
    for x in (-0.6, 0.6):
        for z in (-0.32, 0.32):
            F.cyl(f'post{x}{z}', 0.015, 0.4, (x, 0.2, z), M.chrome, verts=12)
    for y in (0.105, 0.39):
        for s in (-1, 1):
            F.box(f'rail_long{y}{s}', 1.2, 0.022, 0.022, (0, y, s * 0.32), M.chrome, bevel=0.006)
            F.box(f'rail_short{y}{s}', 0.022, 0.022, 0.64, (s * 0.6, y, 0), M.chrome, bevel=0.006)
    F.cyl('pot', 0.07, 0.08, (-0.3, 0.46, 0.0), M.pot_white, verts=20, bevel=0.01)
    F.sphere('plant', 0.1, (-0.3, 0.53, 0.0), M.plant, scale=(1, 1, 0.7), segments=14, rings=8)
    F.cone('vase', 0.05, 0.03, 0.18, (0.36, 0.22, 0.05), M.vase, verts=20)
    G.join_all('coffeeTableGlass')
    G.settle(centre=False)


def coffee_table_marble():
    """The marble side table (catalog.py): a round marble top on a brass tulip foot, a vase and a magazine."""
    F.cyl('top', 0.44, 0.04, (0, 0.4, 0), M.marble, verts=48, bevel=0.006)
    G.revolve('foot', [(0, -0.5), (0.5, -0.5), (0.5, -0.43), (0.2, -0.25), (0.11, 0.2), (0.18, 0.5), (0, 0.5)],
              at=F.at(0, 0.19, 0), material=M.brass, segments=32, scale=(0.56, 0.56, 0.38))
    for i, (x, z, l, turn) in enumerate(((-0.05, 0.06, 0.5, 32), (0.16, -0.12, 0.3, -20))):
        F.box(f'vein{i}', l, 0.003, 0.012, (x, 0.4215, z), M.marble_vein, rot=(0, 0, -turn))
    F.cone('vase', 0.05, 0.03, 0.16, (0.12, 0.5, 0.05), M.vase, verts=20)
    F.box('magazine', 0.24, 0.03, 0.18, (-0.12, 0.435, -0.08), M.magazine_navy, rot=(0, 0, -14), bevel=0.003)
    G.join_all('coffeeTableMarble')
    G.settle(centre=False)


def side_table():
    """The side table (catalog.py `side_table`, classic, which the catalog pack draws over the base one): an oak
    top at 0.55 over a drawer box with a brass knob, a lower shelf, tapered legs."""
    F.box('top', 0.55, 0.03, 0.45, (0, 0.535, 0), M.oak, bevel=0.01)
    F.box('drawer_box', 0.46, 0.08, 0.36, (0, 0.48, 0), M.oak, bevel=0.006)
    F.box('drawer_front', 0.3, 0.06, 0.012, (0, 0.48, 0.185), M.oak, bevel=0.004)
    F.sphere('knob', 0.014, (0, 0.48, 0.196), M.brass, segments=10, rings=6)
    F.box('shelf', 0.46, 0.02, 0.36, (0, 0.16, 0), M.oak, bevel=0.004)
    for x in (-0.23, 0.23):
        for z in (-0.18, 0.18):
            F.cone(f'leg{x}{z}', 0.014, 0.02, 0.52, (x, 0.26, z), M.oak, verts=12)
    G.join_all('sideTable')
    G.settle(centre=False)


def coffee_table_round():
    """The round coffee table (catalog2.py): a cream top at 0.42 on an oak drum."""
    F.cyl('top', 0.42, 0.04, (0, 0.4, 0), M.cream_gloss, verts=48, bevel=0.008)
    F.cone('drum', 0.3, 0.255, 0.36, (0, 0.2, 0), M.oak, verts=40)
    F.cyl('plinth', 0.35, 0.02, (0, 0.01, 0), M.oak, verts=40)
    G.join_all('coffeeTableRound')
    G.settle(centre=False)


def coffee_table_long():
    """The long coffee table (catalog2.py): a walnut top at 0.42 and shelf on black legs, a box and a book."""
    F.box('top', 1.66, 0.04, 0.72, (0, 0.4, 0), M.walnut_warm, bevel=0.012)
    F.box('shelf', 1.5, 0.025, 0.58, (0, 0.14, 0), M.walnut_warm, bevel=0.008)
    _legs4((-0.76, 0.76), (-0.3, 0.3), 0.38, 0.05, M.black_metal)
    F.box('box', 0.32, 0.05, 0.24, (0.4, 0.178, 0.0), M.box_sand, bevel=0.008)
    F.box('book', 0.28, 0.04, 0.2, (0.4, 0.22, 0.0), M.magazine_teal, bevel=0.006)
    G.join_all('coffeeTableLong')
    G.settle(centre=False)


def nesting_tables():
    """The nesting tables (catalog2.py): a white table at 0.51 on black legs, a smaller oak one under it (a
    part of its own, for a clip pulling it out when it gets a use)."""
    with G.new_parts() as big:
        F.box('top', 0.56, 0.025, 0.46, (0, 0.5, -0.02), M.white_gloss, bevel=0.008)
        for x in (-0.26, 0.26):
            for z in (-0.23, 0.19):
                F.box(f'leg{x}{z}', 0.02, 0.5, 0.02, (x, 0.25, z), M.black_metal, bevel=0.004)
    G.join(big, 'table', origin=(0, 0, 0))
    with G.new_parts() as small:
        F.box('small_top', 0.42, 0.025, 0.36, (0.08, 0.38, 0.12), M.oak, bevel=0.008)
        for x in (-0.11, 0.27):
            for z in (-0.04, 0.28):
                F.box(f'small_leg{x}{z}', 0.018, 0.37, 0.018, (x, 0.185, z), M.black_metal, bevel=0.004)
    G.join(small, 'small_table')
    G.settle(centre=False)


def side_table_round():
    """The round side table (catalog2.py): a stone top at 0.56 on a brass stem and disc foot."""
    F.cyl('top', 0.26, 0.03, (0, 0.545, 0), M.stone_top, verts=40, bevel=0.006)
    F.cyl('stem', 0.03, 0.5, (0, 0.28, 0), M.brass, verts=16)
    F.cyl('foot', 0.18, 0.03, (0, 0.015, 0), M.brass, verts=36, bevel=0.006)
    G.join_all('sideTableRound')
    G.settle(centre=False)


def console_table():
    """The console table (catalog2.py): a narrow oak top at 0.8 against the wall, a low shelf with two baskets,
    on black legs."""
    F.box('top', 1.8, 0.035, 0.56, (0, 0.78, -0.18), M.oak, bevel=0.01)
    F.box('shelf', 1.7, 0.025, 0.48, (0, 0.22, -0.18), M.oak, bevel=0.006)
    _legs4((-0.86, 0.86), (-0.42, 0.06), 0.77, 0.04, M.black_metal)
    for x in (-0.5, 0.0):
        F.box(f'basket{x}', 0.36, 0.14, 0.3, (x, 0.31, -0.18), M.basket, bevel=0.03, bevel_segments=3)
    G.join_all('consoleTable')
    G.settle()  # on its Model's `centre`, its back on the wall face


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
     G.Model('sofa', sofa_classic, key='model.sofa', size=(1.88, 0.935, 0.817), centre=(0, -0.0215),
             motions={'uses': {'relax': 'sink_in'}},
             paint=[('linen', 'Upholstery'), ('linen_cushion', 'Cushions'), ('walnut', 'Frame'), ('walnut_legs', 'Legs'),
                    ('pillow_rust', 'Pillow'), ('pillow_sage', 'Second pillow')]),
     G.Model('loveseat', loveseat, key='model.loveseat', size=(1.84, 0.833, 0.84), centre=(0, -0.01),
             motions={'uses': SOFA_USES},
             paint=[('sage', 'Upholstery'), ('sage_cushion', 'Cushions'), ('oak_legs', 'Legs'), ('pillow_cream', 'Pillow')]),
     G.Model('sofaThree', sofa_three, key='model.sofaThree', size=(2.82, 0.87, 0.86), centre=(0, 0.0),
             motions={'uses': SOFA_USES},
             paint=[('stone', 'Upholstery'), ('stone_cushion', 'Cushions'), ('oak_dark_legs', 'Legs'), ('pillow_sage', 'Pillows'),
                    ('throw_sage', 'Throw')]),
     G.Model('sofaDesign', sofa_design, key='model.sofaDesign', size=(1.87, 0.72, 0.82), centre=(0, -0.02),
             motions={'uses': SOFA_USES},
             paint=[('teal', 'Upholstery'), ('teal_cushion', 'Cushions'), ('brass', 'Frame'), ('brass_legs', 'Legs')]),
     G.Model('sofaThreeSeat', sofa_three_seat, key='model.sofaThreeSeat', size=(2.86, 0.823, 0.88), centre=(0, 0.01),
             motions={'uses': SOFA_USES},
             paint=[('moss', 'Upholstery'), ('moss_cushion', 'Cushions'), ('oak_legs', 'Legs'), ('pillow_cream', 'Pillow'),
                    ('pillow_mustard', 'Second pillow')]),
     G.Model('sectionalMiddle', sectional(), key='model.sectionalMiddle', size=(1.0, 0.823, 0.88), centre=(0, 0.01),
             motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalArmLeft', sectional(arms=(1,)), key='model.sectionalArmLeft', size=(1.0, 0.823, 0.88), centre=(0, 0.01),
             motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalArmRight', sectional(arms=(-1,)), key='model.sectionalArmRight', size=(1.0, 0.823, 0.88),
             centre=(0, 0.01), motions={'uses': SOFA_USES}, paint=SECTIONAL),
     G.Model('sectionalCorner', sectional(backs=('back', 'right'), depth=0.89, z=0.015, pillows=[(0.1, -0.03, 'pillow_mustard', -35)]),
             key='model.sectionalCorner', size=(1.0, 0.823, 0.89), centre=(0, 0.015),
             motions={'uses': SOFA_USES}, paint=SECTIONAL + [('pillow_mustard', 'Pillow')]),
     G.Model('chaise', chaise, key='model.chaise', size=(0.96, 0.823, 1.9), centre=(0, 0.02),
             motions={'uses': SOFA_USES}, paint=SECTIONAL + [('throw_oat', 'Throw')]),
     # L2: the armchairs and small seats
     G.Model('armchair', armchair_bergere, key='model.armchair', size=(0.74, 0.982, 0.797), centre=(0, -0.0315),
             motions={'uses': {'sit': 'sink_in'}},
             paint=[('linen', 'Upholstery'), ('linen_cushion', 'Cushion'), ('walnut', 'Frame'), ('walnut_legs', 'Legs'),
                    ('pillow_sage', 'Pillow')]),
     G.Model('armchairLounge', armchair_lounge, key='model.armchairLounge', size=(0.82, 0.83, 0.84), centre=(0, -0.01),
             motions={'uses': {'sit': 'sink_in'}},
             paint=[('sage', 'Upholstery'), ('sage_cushion', 'Cushions'), ('oak_legs', 'Legs')]),
     G.Model('armchairLeather', armchair_leather, key='model.armchairLeather', size=(0.73, 0.949, 0.852), centre=(0, -0.004),
             motions={'uses': {'sit': 'sink_in', 'read': 'sink_in'}},
             paint=[('teak', 'Frame'), ('leather', 'Upholstery')]),
     G.Model('armchairDesign', armchair_design, key='model.armchairDesign', size=(0.76, 0.959, 0.852), centre=(0, -0.004),
             paint=[('ply', 'Frame'), ('hide', 'Upholstery'), ('alu', 'Base')]),
     G.Model('armchairWingback', armchair_wingback, key='model.armchairWingback', size=(0.84, 1.1, 0.8), centre=(0, -0.03),
             motions={'uses': {'sit': 'sink_in', 'read': 'sink_in'}},
             paint=[('forest', 'Upholstery'), ('forest_cushion', 'Cushion'), ('walnut_warm', 'Legs')]),
     G.Model('recliner', recliner, key='model.recliner', size=(0.92, 0.843, 0.7), centre=(0, 0.05),
             motions={'uses': {'relax': ['recline', 'footrest_up'], 'doze': ['recline', 'footrest_up']}, 'ride': 'recline'},
             paint=[('tan', 'Upholstery'), ('tan_cushion', 'Cushions & footrest')]),
     G.Model('rockingChair', rocking_chair, key='model.rockingChair', size=(0.525, 1.033, 0.9), centre=(0, 0.05),
             motions={'loops': {'relax': 'rock', 'read': 'rock'}, 'ride': 'rock'},
             paint=[('rocker', 'Wood'), ('rocker_cushion', 'Cushion')]),
     G.Model('pouf', pouf, key='model.pouf', size=(0.72, 0.47, 0.5), motions={'uses': {'sit': 'sink_in'}},
             paint=[('pouf_leather', 'Leather'), ('pouf_button', 'Buttons'), ('walnut_warm', 'Legs')]),
     G.Model('ottoman', ottoman, key='model.ottoman', size=(0.62, 0.45, 0.62), motions={'uses': {'sit': 'sink_in'}},
             paint=[('navy', 'Upholstery'), ('navy_button', 'Buttons'), ('walnut_warm', 'Feet')]),
     G.Model('floorCushion', floor_cushion, key='model.floorCushion', size=(0.62, 0.139, 0.62), motions={'uses': {'sit': 'sink_in'}},
             paint=[('mustard', 'Cushion'), ('mustard_piping', 'Piping'), ('mustard_button', 'Button')]),
     G.Model('beanBag', bean_bag, key='model.beanBag', size=(0.84, 0.696, 0.844), centre=(0, 0.018), motions={'uses': {'flop': 'squash'}},
             paint=[('bag', 'Bag'), ('bag_seat', 'Seat')]),
     # L3: the coffee and side tables
     G.Model('coffeeTable', coffee_table, key='model.coffeeTable', size=(1.3, 0.42, 0.74),
             motions={'uses': {'magazine': 'drawer_open'}, 'hold': 2.5}, paint=[('oak', 'Wood'), ('brass', 'Knob')]),
     G.Model('coffeeTableGlass', coffee_table_glass, key='model.coffeeTableGlass', size=(1.3, 0.6, 0.74),
             paint=[('chrome', 'Frame'), ('glass', 'Glass')]),
     G.Model('coffeeTableMarble', coffee_table_marble, key='model.coffeeTableMarble', size=(0.88, 0.58, 0.88),
             paint=[('marble', 'Top'), ('brass', 'Base')]),
     G.Model('sideTable', side_table, key='model.sideTable', size=(0.55, 0.55, 0.45), paint=[('oak', 'Wood'), ('brass', 'Knob')]),
     G.Model('coffeeTableRound', coffee_table_round, key='model.coffeeTableRound', size=(0.84, 0.42, 0.84),
             paint=[('cream_gloss', 'Top'), ('oak', 'Base')]),
     G.Model('coffeeTableLong', coffee_table_long, key='model.coffeeTableLong', size=(1.66, 0.42, 0.72),
             paint=[('walnut_warm', 'Top'), ('black_metal', 'Legs')]),
     G.Model('nestingTables', nesting_tables, key='model.nestingTables', size=(0.57, 0.512, 0.55), centre=(0.005, 0.025),
             paint=[('white_gloss', 'Top'), ('oak', 'Small table'), ('black_metal', 'Legs')]),
     G.Model('sideTableRound', side_table_round, key='model.sideTableRound', size=(0.52, 0.56, 0.52),
             paint=[('stone_top', 'Top'), ('brass', 'Base')]),
     G.Model('consoleTable', console_table, key='model.consoleTable', size=(1.8, 0.7975, 0.56), centre=(0, -0.15),
             paint=[('oak', 'Top'), ('black_metal', 'Legs'), ('basket', 'Baskets')])],
    comment='Blender-built living room models (tools/art/blender/families/living.py). New objects with a '
            'Blender model; the placeholder is drawn if a file fails to load, and the materials map onto the '
            'game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
