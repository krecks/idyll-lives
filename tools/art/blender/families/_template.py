# A family to copy: a toy chest whose lid swings open and whose drawer slides out. Copy this file to
# families/<family>.py, rename the family, models and key, and build:
#
#   python3 -B tools/art/blender/build.py _template --preview     this template: previews only
#   python3 -B tools/art/blender/build.py <family> --preview      yours: look at previews/<id>.png
#   python3 -B tools/art/blender/build.py <family>                export, register, check
#
# The rules (size, budgets, finishes, moving parts) are docs/design/objects.md; the steps are README.md.
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

# 1. The palette: one material per colour, named for what it is, with the finish the game draws it
#    with. The names are the material names in the file and the keys of the pack's `materials`.
M = G.Palette(
    body=('#7FA7C9', 'satin'),     # the painted box; the main part the player colours (`paint` below)
    trim=('#F1E9DA', 'satin'),     # plinth and lid edge
    inside=('#C9B79A', 'matte'),   # seen with the lid open
    brass=('#C9A54A', 'metal'),    # knob and handle
)

# Sizes in metres. Blender axes: x to the right as seen from the front, y to the BACK (the front faces
# -y), z up. The origin is the middle of the footprint, on the floor (G.settle puts it there).
W, D, H = 0.8, 0.45, 0.5
LID = 0.04        # lid thickness
DRAWER = 0.12     # drawer height, above a 4 cm plinth
WALL = 0.02


def _box_walls():
    """The chest above the drawer: a floor and four walls, so the open lid shows an inside."""
    z0, z1 = 0.04 + DRAWER, H - LID
    h = z1 - z0
    G.cube('floor', (W, D, WALL), at=(0, 0, z0 + WALL / 2), material=M.inside)
    G.cube('wall_front', (W, WALL, h), at=(0, -D / 2 + WALL / 2, z0 + h / 2), material=M.body, bevel=0.004)
    G.cube('wall_back', (W, WALL, h), at=(0, D / 2 - WALL / 2, z0 + h / 2), material=M.body, bevel=0.004)
    for s in (-1, 1):
        G.cube(f'wall_{s}', (WALL, D - 2 * WALL, h), at=(s * (W / 2 - WALL / 2), 0, z0 + h / 2),
               material=M.body, bevel=0.004)
    # the inside lining, a little in from the walls
    G.cube('lining', (W - 2 * WALL - 0.004, D - 2 * WALL - 0.004, 0.004), at=(0, 0, z0 + WALL + 0.002),
           material=M.inside)


def chest():
    # 2. The part that stays still: build it from primitives, then join it into one object. `new_parts`
    #    collects what is made inside the block.
    with G.new_parts() as body:
        G.cube('plinth', (W - 0.04, D - 0.04, 0.04), at=(0, 0, 0.02), material=M.trim, bevel=0.004)
        G.cube('drawer_case', (W, D, DRAWER), at=(0, 0, 0.04 + DRAWER / 2), material=M.body, bevel=0.004)
        _box_walls()
    G.join(body, 'body', origin=(0, 0, 0))

    # 3. A moving part: its own object, joined from its pieces, its origin on the hinge. The lid hinges
    #    on its back edge (Blender +y), at the top of the walls.
    with G.new_parts() as lid:
        G.cube('lid', (W + 0.02, D + 0.02, LID), at=(0, 0, H - LID / 2), material=M.trim, bevel=0.008)
        G.cube('lid_knob', (0.08, 0.02, 0.02), at=(0, -D / 2 - 0.02, H - LID / 2), material=M.brass, bevel=0.004)
    lid = G.hinged(G.join(lid, 'lid'), (0, D / 2 + 0.01, H - LID))

    # A drawer slides from where it is: its origin can stay where it is.
    with G.new_parts() as drawer:
        G.cube('drawer', (W - 0.06, 0.02, DRAWER - 0.02), at=(0, -D / 2 - 0.01, 0.04 + DRAWER / 2),
               material=M.body, bevel=0.004)
        G.cylinder('drawer_handle', 0.012, 0.14, at=(0, -D / 2 - 0.035, 0.04 + DRAWER / 2),
                   material=M.brass, rot=(0, 90, 0), verts=10)
        # the tray behind the front, hidden in the case until the drawer is out
        G.cube('drawer_tray', (W - 0.1, D - 0.08, DRAWER - 0.04), at=(0, 0.02, 0.04 + DRAWER / 2),
               material=M.inside)
    drawer = G.join(drawer, 'drawer')

    # 4. On the floor, centred: it moves everything, hinges included, and never scales. Before the clips,
    #    which key the rest pose where it is.
    objs = G.settle()

    # 5. One clip per moving part, named for the pack's `motions`. The first key is the rest pose
    #    (closed); the clip opens it. Angles turn about the part's own axis: about x, a negative angle
    #    lifts the front. Offsets are Blender metres: -y pulls towards the front.
    G.swing(lid, 'lid_open', -100, seconds=1.0, axis='X')
    G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=0.8)
    return objs


FAMILY = G.Family(
    # The models' folder: models/example/<id>.glb, keys model.example.<id>.
    'example', M,
    [G.Model('chest', chest,
             # 6. The size it must come out at (game width, height, depth; knob and handle included), within 2 %.
             size=(0.82, 0.5, 0.51),
             # Which use plays which clip (an interaction's id or its animation tag), and for how long
             # after the use starts (seconds of the residents' time; absent: until the use ends).
             motions={'uses': {'fetch': 'lid_open', 'putAway': 'drawer_open'}, 'hold': 2.5},
             # The materials the player's colour goes on.
             paint=[('body', 'Body')],
             )],
    comment='Blender-built example models (tools/art/blender/families/<family>.py): what the family is.',
)

if __name__ == '__main__':
    if '--preview' not in sys.argv:
        sys.exit('the template only previews: copy it to families/<family>.py to export')
    FAMILY.run(sys.argv)
