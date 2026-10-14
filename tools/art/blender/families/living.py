# The living room: a fold-down wall bed (`model.wallBed`, content `wallBed` in catalog2/living.json).
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
# Run:  python3 -B tools/art/blender/build.py living [--preview]
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


# The model folded, as the placeholder draws it (the fallback when the file fails to load).
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
             placeholder=PLACEHOLDER)],
    comment='Blender-built living room models (tools/art/blender/families/living.py). New objects with a '
            'Blender model; the placeholder is drawn if a file fails to load, and the materials map onto the '
            'game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
