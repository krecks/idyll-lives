# The laundry: the washing machine (`model.laundryWasher`), the tumble dryer (`model.laundryDryer`), the towel
# rail (`model.towelRail`), the towel ladder (`model.towelLadder`), the laundry basket (`model.laundryBasket`)
# and the bath mat (`model.bathMat`), over their primitive-part placeholders (`catalog.py`, `catalog2.py`),
# which stay the fallback. Notes: laundry.md.
#
# The washer and the dryer stand against the wall where their placeholders do, built in place (`F`, game
# axes). Each has two moving parts: the round door, hinged on its left, which swings open while a resident
# loads it (`door_open`, held 2.5 s), and the drum's face behind the glass, which turns for the whole use
# (`drum_spin`, a loop). The rail hangs on the wall at its real height; the rest are still.
#
# Run:  python3 -B tools/art/blender/build.py laundry [--only laundryWasher] [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#F4F4F2', 'gloss'),       # a machine's cabinet; the main part the player colours
    panel=('#E6E6E3', 'gloss'),      # its control strip
    door=('#C9CBCC', 'satin'),       # the washer's door ring, handle and knob
    door_dryer=('#9AA4AB', 'satin'),  # the dryer's
    glass=('#BFD3DA', 'glass'),      # the door's window (see-through: its own draw call)
    dark=('#3A3D41', 'satin'),       # plinths, displays, the door seal
    drum=('#2B2C2E', 'satin'),       # the drum's dark inside
    paddle=('#9AA0A6', 'metal'),     # the drum's paddles
    led=('#5EC28B', 'gloss'),        # the washer's ready light
    led_dryer=('#E9853B', 'gloss'),  # the dryer's
    # washing: plain, so it costs no draw call of its own on a machine
    wash_a=('#9DC6CF', 'satin'),
    wash_b=('#E4B8AE', 'satin'),
    wash_c=('#F2F0EA', 'satin'),
    wicker_plain=('#C9B79C', 'satin'),  # the basket on the dryer
    # the rail, the ladder, the basket and the mat
    chrome=('#D0D3D6', 'chrome'),
    towel=('#9DC6CF', 'fabric'),
    towel2=('#F2F0EA', 'fabric'),
    towel3=('#9DAF8C', 'fabric'),
    birch=('#E3CFA8', 'wood'),
    wicker=('#C9A979', 'fabric'),
    wicker_rim=('#B8955F', 'fabric'),
    mat=('#B9CCC4', 'fabric'),
    mat_pile=('#C9DAD3', 'fabric'),
)

WALL = -0.43   # the wall's face, game depth
DOOR_Z = 0.152  # the door ring's middle, game depth
DOOR_Y = 0.4    # the door's middle, height
DOOR_R = 0.21   # the ring's radius


# --- the washer and the dryer ---------------------------------------------------------------------------
def _cabinet(F, led, dryer):
    """The still cabinet: the body, a plinth, the control strip with a soap drawer or a filter, a knob, a
    display and a light, and the door seal round the porthole."""
    F.box('cabinet', 0.6, 0.84, 0.6, (0, 0.44, -0.16), M.body, bevel=0.03)
    F.box('plinth', 0.56, 0.02, 0.56, (0, 0.01, -0.16), M.dark, bevel=0.006)
    F.box('strip', 0.6, 0.13, 0.012, (0, 0.78, 0.144), M.panel, bevel=0.006)
    F.box('drawer', 0.14, 0.07, 0.012, (-0.18, 0.78, 0.152), M.body, bevel=0.006)
    F.box('drawer_grip', 0.08, 0.012, 0.008, (-0.18, 0.765, 0.16), M.dark, bevel=0.003)
    F.box('display', 0.12, 0.05, 0.006, (0.02, 0.78, 0.153), M.dark, bevel=0.004)
    F.box('light', 0.03, 0.016, 0.004, (0.02, 0.78, 0.158), led, bevel=0.002)
    F.cyl('knob_skirt', 0.045, 0.012, (0.18, 0.78, 0.154), M.panel, rot=(90, 0, 0), verts=20)
    F.cyl('seal', 0.2, 0.008, (0, DOOR_Y, 0.143), M.dark, rot=(90, 0, 0), verts=36)
    if dryer:
        for k, y in enumerate((0.08, 0.105, 0.13)):
            F.box(f'vent{k}', 0.4, 0.008, 0.01, (0, y, 0.142), M.door_dryer, bevel=0.003)
    else:
        F.box('filter', 0.12, 0.06, 0.008, (0.2, 0.08, 0.142), M.panel, bevel=0.004)


def _door(F, ring):
    """The door: a ring, its window and a handle on the right, hinged on its left edge."""
    with G.new_parts() as parts:
        F.cyl('door_back', DOOR_R + 0.012, 0.012, (0, DOOR_Y, 0.148), ring, rot=(90, 0, 0), verts=36)
        G.torus('door_ring', DOOR_R - 0.012, 0.024, at=F.at(0, DOOR_Y, DOOR_Z + 0.006), material=ring,
                rot=(90, 0, 0), major_segments=36, minor_segments=10)
        F.cyl('window', DOOR_R - 0.025, 0.01, (0, DOOR_Y, DOOR_Z + 0.012), M.glass, rot=(90, 0, 0), verts=36)
        F.box('handle', 0.03, 0.08, 0.03, (DOOR_R + 0.01, DOOR_Y, DOOR_Z + 0.02), ring, bevel=0.01)
    door = G.join(parts, 'door')
    return G.hinged(door, F.at(-DOOR_R - 0.02, DOOR_Y, DOOR_Z))


def _drum(F, washing):
    """The drum's face behind the window: its dark inside, three paddles and the washing, joined round the
    middle so it turns about its axis."""
    with G.new_parts() as parts:
        F.cyl('drum_face', 0.19, 0.006, (0, DOOR_Y, 0.146), M.drum, rot=(90, 0, 0), verts=32)
        for k in range(3):
            a = 120 * k
            x, y = 0.14 * G.sind(a), DOOR_Y + 0.14 * G.cosd(a)
            F.box(f'paddle{k}', 0.025, 0.07, 0.008, (x, y, 0.151), M.paddle, rot=(0, 0, -a), bevel=0.003)
        for k, (m, dx, dy, r) in enumerate(washing):
            G.sphere(f'wash{k}', r, at=F.at(dx, DOOR_Y + dy, 0.151), material=m, scale=(1, 0.18, 0.8),
                     rot=(0, 15 * k, 0), segments=14, rings=8)
    drum = G.join(parts, 'drum')
    return G.hinged(drum, F.at(0, DOOR_Y, 0.146))


def _machine(dryer):
    F = G.GameFrame(shift=0.03)      # built 3 cm into the wall (as the placeholder was); moved onto its face
    led, ring = (M.led_dryer, M.door_dryer) if dryer else (M.led, M.door)
    with G.new_parts() as still:
        _cabinet(F, led, dryer)
        F.cyl('knob', 0.033, 0.03, (0.18, 0.78, 0.165), ring, rot=(90, 0, 0), verts=18, bevel=0.006)
        if dryer:
            # a wicker basket of dry washing on the top
            G.revolve('basket', [(0, 0), (0.185, 0), (0.21, 0.18), (0.195, 0.18), (0.17, 0.02), (0, 0.02)],
                      at=F.at(-0.06, 0.86, -0.14), material=M.wicker_plain, segments=24, scale=(1, 0.76, 1))
            G.sphere('pile', 0.15, at=F.at(-0.06, 1.0, -0.14), material=M.wash_c, scale=(1, 0.75, 0.38),
                     segments=16, rings=8)
            G.sphere('pile_top', 0.07, at=F.at(0.02, 1.05, -0.1), material=M.wash_a, scale=(1.1, 0.9, 0.5),
                     segments=12, rings=8)
        else:
            # folded towels on the top
            F.box('fold0', 0.3, 0.08, 0.22, (0.08, 0.9, -0.1), M.wash_c, bevel=0.02)
            F.box('fold1', 0.28, 0.04, 0.2, (0.08, 0.96, -0.1), M.wash_b, bevel=0.015)
    G.join(still, 'body', origin=(0, 0, 0))
    washing = ([(M.wash_c, -0.05, -0.06, 0.08), (M.wash_b, 0.06, -0.08, 0.07), (M.wash_a, 0.0, 0.07, 0.06)]
               if dryer else
               [(M.wash_a, -0.04, -0.07, 0.08), (M.wash_b, 0.07, -0.05, 0.06), (M.wash_c, 0.02, 0.06, 0.07)])
    drum = _drum(F, washing)
    door = _door(F, ring)
    G.settle(centre=False)
    G.swing(door, 'door_open', 110, seconds=1.0)
    G.spin(drum, 'drum_spin', turns=1, seconds=1.2, axis='Y')
    return G.meshes()


def laundryWasher():
    return _machine(dryer=False)


def laundryDryer():
    return _machine(dryer=True)


# --- the towel rail: two chrome bars on the wall, towels over them ----------------------------------------
def _towel_over(F, name, x, w, bar_y, bar_z, drop_front, drop_back, material, t=0.012):
    """A towel folded over a bar at (bar_y, bar_z): a rounded top and two flaps hanging down either side."""
    F.box(f'{name}_top', w, 0.03, 0.04, (x, bar_y + 0.016, bar_z), material, bevel=0.012)
    F.box(f'{name}_front', w, drop_front, t, (x, bar_y + 0.01 - drop_front / 2, bar_z + 0.022), material, bevel=0.005)
    F.box(f'{name}_back', w, drop_back, t, (x, bar_y + 0.01 - drop_back / 2, bar_z - 0.022), material, bevel=0.005)


def towelRail():
    F = G.GameFrame()
    z = -0.36
    with G.new_parts() as parts:
        for y in (1.2, 0.95):
            G.rod(f'bar{y}', F.at(-0.34, y, z), F.at(0.34, y, z), 0.0125, material=M.chrome, verts=12)
        for s in (-1, 1):
            F.box(f'plate{s}', 0.05, 0.36, 0.012, (s * 0.34, 1.075, WALL + 0.006), M.chrome, bevel=0.005)
            F.box(f'post{s}', 0.025, 0.31, 0.025, (s * 0.34, 1.075, z), M.chrome, bevel=0.008)
            for y in (1.2, 0.95):
                G.rod(f'arm{s}{y}', F.at(s * 0.34, y, WALL + 0.012), F.at(s * 0.34, y, z), 0.011,
                      material=M.chrome, verts=10)
        _towel_over(F, 'bath_towel', -0.07, 0.42, 1.2, z, 0.5, 0.38, M.towel)
        _towel_over(F, 'hand_towel', 0.17, 0.26, 0.95, z, 0.3, 0.22, M.towel2)
    G.join_all('towelRail')
    G.settle(centre=False, floor=False)   # on the wall, at its real height
    return G.meshes()


# --- the towel ladder: a birch ladder leaning on the wall, towels over three rungs --------------------------
LADDER_TILT = 10.66      # degrees back from upright
LADDER_FOOT = -0.1       # game depth of the rails' feet


def _on_ladder(y):
    """The game depth of the ladder's front face at height `y`."""
    return LADDER_FOOT - y * G.sind(LADDER_TILT) / G.cosd(LADDER_TILT)


def towelLadder():
    F = G.GameFrame(shift=0.03)      # its top rail on the wall face, not 3 cm into it
    top = 1.7
    with G.new_parts() as parts:
        for s in (-1, 1):
            G.rod(f'rail{s}', F.at(s * 0.24, 0.0, LADDER_FOOT - 0.02), F.at(s * 0.24, top, _on_ladder(top) - 0.02),
                  0.02, material=M.birch, verts=12)
        for k, y in enumerate((0.3, 0.63, 0.96, 1.29, 1.62)):
            z = _on_ladder(y) - 0.02
            G.rod(f'rung{k}', F.at(-0.24, y, z), F.at(0.24, y, z), 0.0125, material=M.birch, verts=10)
        for k, (y, m) in enumerate(((0.63, M.towel3), (0.96, M.towel), (1.29, M.towel2))):
            _towel_over(F, f'towel{k}', 0, 0.4, y, _on_ladder(y) - 0.02, 0.3, 0.26, m)
    G.join_all('towelLadder')
    G.settle(centre=False)
    return G.meshes()


# --- the laundry basket: a wicker tub of clean washing -----------------------------------------------------
def laundryBasket():
    F = G.GameFrame()
    z = -0.2
    with G.new_parts() as parts:
        G.revolve('basket', [(0, 0), (0.23, 0), (0.25, 0.02), (0.255, 0.33), (0.24, 0.33), (0.225, 0.03), (0, 0.03)],
                  at=F.at(0, 0, z), material=M.wicker, segments=32, scale=(1, 0.78, 1))
        G.torus('rim', 0.25, 0.02, at=F.at(0, 0.335, z), material=M.wicker_rim, major_segments=32,
                minor_segments=8).scale = (1, 0.78, 1)
        for k in range(6):
            G.torus(f'weave{k}', 0.243 + 0.0018 * k, 0.006, at=F.at(0, 0.05 + 0.05 * k, z), material=M.wicker_rim,
                    major_segments=32, minor_segments=5).scale = (1, 0.78, 1)
        for s in (-1, 1):
            G.torus(f'handle{s}', 0.05, 0.012, at=F.at(s * 0.25, 0.29, z), material=M.wicker_rim, rot=(90, 0, 90),
                    major_segments=16, minor_segments=6)
        for k, (x, dz, m) in enumerate(((-0.08, 0.0, M.towel), (0.1, 0.03, M.towel2), (0.0, -0.06, M.wash_b))):
            G.sphere(f'wash{k}', 0.15, at=F.at(x, 0.33, z + dz), material=m, scale=(1, 0.8, 0.4), segments=16, rings=8)
    G.join_all('laundryBasket')
    G.settle(centre=False)
    return G.meshes()


# --- the bath mat: a soft rounded mat with a raised pile -----------------------------------------------------
def bathMat():
    with G.new_parts() as parts:
        G.cube('mat', (0.76, 0.5, 0.016), at=(0, 0, 0.008), material=M.mat, bevel=0.007, bevel_segments=3)
        G.cube('pile', (0.66, 0.4, 0.012), at=(0, 0, 0.016), material=M.mat_pile, bevel=0.005, bevel_segments=3)
        for s in (-1, 1):
            G.cube(f'stripe{s}', (0.66, 0.025, 0.004), at=(0, s * 0.15, 0.0225), material=M.mat, bevel=0.0015)
    G.join_all('bathMat')
    G.settle()
    return G.meshes()


LAUNDRY = {'uses': {'laundry': 'door_open'}, 'hold': 2.5, 'loops': {'laundry': 'drum_spin'}}

FAMILY = G.Family(
    'laundry', M,
    [G.Model('laundryWasher', laundryWasher, size=(0.6, 0.98, 0.65), key='model.laundryWasher', centre=(0, -0.105),
             motions=LAUNDRY, paint=[('body', 'Body'), ('door', 'Door'), ('panel', 'Panel')]),
     G.Model('laundryDryer', laundryDryer, size=(0.6, 1.08, 0.65), key='model.laundryDryer', centre=(0, -0.105),
             motions=LAUNDRY, paint=[('body', 'Body'), ('door_dryer', 'Door'), ('panel', 'Panel')]),
     G.Model('towelRail', towelRail, size=(0.73, 0.595, 0.098), key='model.towelRail', centre=(0, -0.381), floor=False,
             paint=[('chrome', 'Rail'), ('towel', 'Towels'), ('towel2', 'Hand towel')]),
     G.Model('towelLadder', towelLadder, size=(0.52, 1.707, 0.359), key='model.towelLadder', centre=(0, -0.25),
             paint=[('birch', 'Ladder'), ('towel', 'Towels'), ('towel2', 'Second towel'), ('towel3', 'Third towel')]),
     G.Model('laundryBasket', laundryBasket, size=(0.54, 0.39, 0.421), key='model.laundryBasket', centre=(0, -0.2),
             paint=[('wicker', 'Basket'), ('wicker_rim', 'Rim & weave')]),
     G.Model('bathMat', bathMat, size=(0.76, 0.025, 0.5), key='model.bathMat',
             paint=[('mat', 'Mat'), ('mat_pile', 'Pile')])],
    comment='Blender-built laundry models (tools/art/blender/families/laundry.py). Replaces the primitive-part '
            'placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a '
            'file fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The washer and the dryer open their door (`door_open`) and turn their drum (`drum_spin`) '
            'while a resident does the laundry.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
