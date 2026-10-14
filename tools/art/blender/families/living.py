# The living room: a fold-down wall bed (`model.wallBed`, content `wallBed` in catalog2/living.json) and a
# sofa bed (`model.sofaBed`, content `sofaBed`).
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
# Run:  python3 -B tools/art/blender/build.py living [--only sofaBed] [--preview]
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
                    ('pillow_rose', 'Pillow'), ('pillow_cream', 'Second pillow')])],
    comment='Blender-built living room models (tools/art/blender/families/living.py). New objects with a '
            'Blender model; the placeholder is drawn if a file fails to load, and the materials map onto the '
            'game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
