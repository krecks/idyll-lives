# The slob pack (P16): the late-night microwave cart, the crumb-proof beanbag, the snack-stash nightstand, the
# spritz & go station, the pixel-blaster arcade cabinet and the command-couch gaming recliner, over their
# primitive-part placeholders (`tools/art/pack_slob.py`, `packs/slob/manifest.json`), which stay the fallback.
# Notes: pack_slob.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), with its moving parts: the microwave's door opens and shuts (`door_open`), the beanbag squashes under
# its sitter (`squash`), the nightstand's side drawer of snacks slides out (`drawer_open`), the spray bottle tips
# (`spritz`), the arcade joystick waggles (`joystick`, a loop), and the recliner's back reclines (`recline`) and
# its cup holder swings out (`cup_out`).
#
# Run:  python3 -B tools/art/blender/build.py pack_slob [--only beanbag] [--preview]
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#D9533F', 'metal'),       # the microwave cart's frame; see the per-model notes for each body
    bag=('#E07A2E', 'satin'),        # the beanbag's wipe-clean vinyl
    bag_light=('#EB8C40', 'satin'),
    bag_seam=('#C8642A', 'satin'),
    stand=('#6E9FC0', 'satin'),      # the nightstand
    stand_front=('#5A8BAE', 'satin'),
    pad=('#F2D14E', 'fabric'),       # the nightstand's padded top
    shelf=('#9B59B6', 'satin'),      # the spritz station's sides and mirror frame
    towel=('#F7F7F7', 'fabric'),
    towel_green=('#6CC27A', 'fabric'),
    laundry=('#6E9FC0', 'fabric'),
    laundry_red=('#E8473B', 'fabric'),
    cabinet=('#4660C2', 'satin'),    # the arcade cabinet's body
    side=('#6A4BB5', 'gloss'),
    marquee=('#F2D14E', 'gloss'),
    screen=('#1B2240', 'gloss'),
    screen_lit=('#3BA3E0', 'gloss'),
    seat=('#5A6680', 'satin'),       # the recliner's leatherette
    stripe=('#E8473B', 'satin'),
    cyan=('#3FD0E0', 'gloss'),       # LED trim
    white=('#E9E4DA', 'satin'),
    gloss_white=('#F2EDE2', 'gloss'),
    panel=('#C9CBCC', 'satin'),
    display=('#2E3A46', 'gloss'),
    lcd=('#7FE0A0', 'gloss'),
    window=('#3A4652', 'gloss'),
    dark=('#2B2C2E', 'satin'),
    tyre=('#3A3D41', 'satin'),
    chrome=('#D0D3D6', 'chrome'),
    mirror=('#DCE6EA', 'chrome'),
    cardboard=('#E9D8B4', 'matte'),
    cookie=('#8E5A3A', 'matte'),
    pop_red=('#E8473B', 'gloss'),
    pop_orange=('#F2A541', 'gloss'),
    pop_blue=('#3E8EDE', 'gloss'),
    pop_green=('#6CC27A', 'gloss'),
    pop_purple=('#9B59B6', 'gloss'),
    pop_yellow=('#F2D14E', 'gloss'),
    bag_red=('#E8473B', 'satin'),    # chip bags and wrappers, plain
    bag_orange=('#F2A541', 'satin'),
    bag_green=('#6CC27A', 'satin'),
    bag_yellow=('#F2D14E', 'satin'),
    bag_purple=('#9B59B6', 'satin'),
)

POP = ('pop_red', 'pop_orange', 'pop_blue', 'pop_green', 'pop_purple', 'pop_yellow')   # by name (made anew per reset)
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _m(name):
    return getattr(M, name)


def _can(F, x, z, y, material, lying=False, rot=0):
    """A soda can on `y`, standing or lying turned `rot` degrees."""
    if lying:
        F.cyl(_name('can'), 0.033, 0.12, (x, y + 0.033, z), material, rot=(90, 0, rot), verts=12)
    else:
        F.cyl(_name('can'), 0.033, 0.12, (x, y + 0.06, z), material, verts=12)
        F.cyl(_name('can_top'), 0.028, 0.006, (x, y + 0.123, z), M.chrome, verts=10)


def _chip_bag(F, x, z, y, material, rot=0, tilt=0):
    F.box(_name('chip_bag'), 0.16, 0.22, 0.06, (x, y + 0.11, z), material, rot=(tilt, 0, rot), bevel=0.025, bevel_segments=2)


def _wrapper(F, x, z, material, seed=0):
    G.boulder(_name('wrapper'), 0.04, at=F.at(x, 0, z), material=material, scale=(1.1, 0.9, 0.5), seed=seed, noise=0.35,
              subdivisions=1)


# --- the late-night microwave cart (1x1): a rolling cart, a microwave on top, frozen dinners below ------------
MW = dict(x=-0.02, y=0.815, z0=-0.42, w=0.44, h=0.29, d=0.36)


def microwaveCart():
    F = G.GameFrame()
    rnd = random.Random(4)
    x, y, z0, w, h, d = (MW[k] for k in ('x', 'y', 'z0', 'w', 'h', 'd'))
    zf, pw = z0 + d, 0.12
    with G.new_parts() as still:
        for sy in (0.8, 0.42, 0.1):
            F.box(_name('cart_shelf'), 0.7 if sy > 0.5 else 0.66, 0.03, 0.46 if sy > 0.5 else 0.42, (0, sy, -0.22), M.white,
                  bevel=0.008)
            F.box(_name('shelf_lip'), 0.66, 0.025, 0.012, (0, sy + 0.025, -0.01), M.body)
        for cx in (-0.33, 0.33):
            for cz in (-0.42, -0.02):
                F.cyl(_name('cart_post'), 0.016, 0.76, (cx, 0.46, cz), M.body, verts=10)
                G.torus(_name('castor'), 0.024, 0.011, at=F.at(cx, 0.035, cz), material=M.tyre, rot=(90, 0, 90),
                        major_segments=12, minor_segments=6)
                F.cyl(_name('fork'), 0.008, 0.04, (cx, 0.075, cz), M.chrome, verts=6)
        G.rod('cart_handle', F.at(0.36, 0.95, -0.4), F.at(0.36, 0.95, -0.04), 0.012, material=M.chrome, verts=8)
        for cz in (-0.4, -0.04):
            G.rod(_name('handle_post'), F.at(0.33, 0.84, cz), F.at(0.36, 0.95, cz), 0.01, material=M.chrome, verts=8)
        # the microwave: its case, the keypad panel on the right, feet
        F.box('mw_case', w, h - 0.012, d - 0.01, (x, y + 0.006 + (h - 0.012) / 2, z0 + (d - 0.01) / 2), M.gloss_white,
              bevel=0.015)
        F.box('mw_keypad', pw, h - 0.04, 0.01, (x + w / 2 - pw / 2 - 0.01, y + h / 2, zf - 0.004), M.panel, bevel=0.004)
        F.box('mw_display', pw - 0.03, 0.04, 0.004, (x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.002), M.display)
        F.box('mw_digits', pw - 0.07, 0.016, 0.003, (x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.004), M.lcd)
        for i in range(3):
            for j in range(3):
                F.box(_name('mw_key'), 0.022, 0.016, 0.006, (x + w / 2 - pw / 2 - 0.04 + i * 0.03, y + h - 0.12 - j * 0.03,
                                                             zf + 0.002), M.white)
        F.cyl('mw_dial', 0.022, 0.02, (x + w / 2 - pw / 2 - 0.01, y + 0.06, zf + 0.004), M.white, rot=(90, 0, 0), verts=12)
        F.box('mw_cavity', w - pw - 0.04, h - 0.06, 0.004, (x - pw / 2 - 0.005, y + h / 2, zf - 0.012), M.dark)
        # frozen dinners, a pizza box, the cool box below, cans, a cutlery jar and dropped wrappers
        yy = 0.435
        for i in range(4):
            F.box(_name('dinner'), 0.26, 0.045, 0.2, (-0.16 + rnd.uniform(-0.02, 0.02), yy + 0.0225, -0.22),
                  _m(POP[i % 6]), rot=(0, 0, rnd.uniform(-10, 10)), bevel=0.006)
            yy += 0.045
        F.box('pizza_box', 0.3, 0.04, 0.3, (0.16, 0.455, -0.22), M.cardboard, bevel=0.004)
        F.box('pizza_logo', 0.2, 0.002, 0.08, (0.16, 0.476, -0.22), M.pop_red)
        F.box('cool_box', 0.28, 0.07, 0.22, (0.0, 0.15, -0.22), M.pop_blue, bevel=0.01)
        F.box('cool_lid', 0.29, 0.02, 0.23, (0.0, 0.195, -0.22), M.pop_orange, bevel=0.006)
        _can(F, 0.26, -0.05, 0.815, M.pop_red)
        _can(F, -0.29, -0.08, 0.815, M.pop_green)
        G.revolve('jar', [(0, 0), (0.03, 0), (0.035, 0.12), (0, 0.11)], at=F.at(0.29, 0.815, -0.38), material=M.gloss_white,
                  segments=10)
        for dx in (-0.01, 0.01):
            G.rod(_name('fork'), F.at(0.29 + dx, 0.85, -0.38), F.at(0.29 + 3 * dx, 0.97, -0.38), 0.005, material=M.chrome,
                  verts=6)
        _wrapper(F, 0.3, 0.3, M.bag_yellow, 1)
        _wrapper(F, -0.25, 0.28, M.bag_red, 2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door with its window and handle, hinged on its left edge: `door_open` swings it open to the left.
    dw = w - pw - 0.02
    dx0 = x - w / 2 + 0.005
    with G.new_parts() as door:
        F.box('mw_door', dw, h - 0.05, 0.016, (dx0 + dw / 2, y + h / 2, zf + 0.004), M.gloss_white, bevel=0.008)
        F.box('mw_window', dw - 0.08, h - 0.11, 0.004, (dx0 + dw / 2 - 0.015, y + h / 2, zf + 0.013), M.window)
        F.box('mw_handle', 0.02, h - 0.1, 0.02, (dx0 + dw - 0.025, y + h / 2, zf + 0.022), M.chrome, bevel=0.006)
    door = G.hinged(G.join(door, 'door'), F.at(dx0, y + h / 2, zf + 0.004))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(door, 'door_open', -100, seconds=0.6)
    return objs


# --- the crumb-proof beanbag (1x1): sat on and napped on in the middle, a snack bag and a controller by it -----
def beanbag():
    F = G.GameFrame()
    with G.new_parts() as still:
        _chip_bag(F, 0.33, 0.32, 0.0, M.bag_yellow, rot=30, tilt=80)
        F.box('controller', 0.14, 0.035, 0.09, (-0.34, 0.018, 0.33), M.dark, rot=(0, 0, -25), bevel=0.015)
        for dx in (-0.03, 0.03):
            F.sphere(_name('stick'), 0.012, (-0.34 + dx, 0.04, 0.33), M.tyre, segments=8, rings=4)
        F.sphere('button', 0.01, (-0.3, 0.04, 0.35), M.pop_red, segments=8, rings=4)
        _can(F, -0.38, -0.32, 0.0, M.pop_blue, lying=True, rot=40)
        _wrapper(F, 0.32, -0.36, M.bag_red, 3)
        _wrapper(F, 0.4, 0.08, M.bag_green, 4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bag: a squat round body, a raised back and a dimple to sit in (top 0.46), one part with its origin on
    # the floor: `squash` settles it under its sitter.
    with G.new_parts() as bag:
        G.revolve('bag_body', [(0, 0), (0.33, 0), (0.41, 0.06), (0.43, 0.16), (0.39, 0.3), (0.28, 0.38), (0.12, 0.41),
                               (0, 0.41)], at=F.at(0, 0, 0.0), material=M.bag, segments=28)
        F.sphere('bag_back', 0.4, (0, 0.42, -0.22), M.bag, scale=(1.0, 0.45, 0.62), rot=(-15, 0, 0), segments=24, rings=12)
        F.sphere('bag_seat', 0.28, (0, 0.4, 0.05), M.bag_light, scale=(1, 0.9, 0.2), segments=22, rings=8)
        G.torus('bag_seam', 0.33, 0.012, at=F.at(0, 0.02, 0), material=M.bag_seam, major_segments=28, minor_segments=6)
        F.box('bag_handle', 0.12, 0.03, 0.02, (0, 0.22, 0.425), M.bag_seam, rot=(-20, 0, 0), bevel=0.008)
    bag = G.join(bag, 'bag', origin=(0, 0, 0))
    objs = G.settle(centre=False)
    G.squash(bag, 'squash', (1.04, 1.04, 0.86), seconds=0.5)
    return objs


# --- the snack-stash nightstand (1x1): sat on (padded top at 0.45), its side drawer full of snacks -------------
def snackNightstand():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('cabinet', 0.56, 0.38, 0.46, (0, 0.19, -0.02), M.stand, bevel=0.015)
        F.box('pad', 0.56, 0.08, 0.46, (0, 0.41, -0.02), M.pad, bevel=0.03, bevel_segments=3)
        F.box('lower_drawer', 0.012, 0.13, 0.4, (0.284, 0.1, -0.02), M.stand_front, bevel=0.004)
        F.box('lower_pull', 0.02, 0.03, 0.08, (0.296, 0.1, -0.02), M.chrome, bevel=0.006)
        F.box('drawer_slot', 0.004, 0.13, 0.4, (0.279, 0.27, -0.02), M.dark)
        # floor clutter: cans, wrappers, a knocked-over alarm clock, a sock
        _can(F, -0.36, 0.26, 0.0, M.pop_blue, lying=True, rot=70)
        _can(F, -0.38, -0.3, 0.0, M.pop_red)
        _can(F, 0.4, 0.3, 0.0, M.pop_green, lying=True, rot=-20)
        _wrapper(F, -0.2, 0.36, M.bag_yellow, 5)
        _wrapper(F, 0.2, 0.34, M.bag_purple, 6)
        F.cyl('clock', 0.06, 0.06, (-0.34, 0.06, -0.06), M.gloss_white, rot=(90, 0, 30), verts=16)
        F.cyl('clock_face', 0.045, 0.004, (-0.355, 0.06, -0.035), M.display, rot=(90, 0, 30), verts=16)
        F.box('sock', 0.08, 0.02, 0.2, (0.4, 0.01, -0.22), M.white, rot=(0, 0, 20), bevel=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # The top side drawer with its snacks, closed into the right side: `drawer_open` slides it out to the right.
    with G.new_parts() as drawer:
        F.box('drawer_box', 0.24, 0.1, 0.38, (0.16, 0.27, -0.02), M.white, bevel=0.006)
        F.box('drawer_front', 0.012, 0.13, 0.42, (0.286, 0.27, -0.02), M.stand_front, bevel=0.005)
        F.box('drawer_pull', 0.02, 0.02, 0.1, (0.3, 0.29, -0.02), M.chrome, bevel=0.006)
        for i, (dz, m) in enumerate(((-0.12, M.bag_red), (0.0, M.bag_orange), (0.12, M.bag_green))):
            F.box(_name('snack'), 0.16, 0.14, 0.05, (0.15, 0.33, -0.02 + dz), m, rot=(-15 + i * 12, 0, 0), bevel=0.02,
                  bevel_segments=2)
        F.cyl('cookie_a', 0.045, 0.02, (0.17, 0.37, 0.12), M.cookie, verts=12)
    drawer = G.hinged(G.join(drawer, 'drawer'), F.at(0.286, 0.27, -0.02))
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (-0.16, 0, 0), seconds=0.6)
    return objs


# --- the spritz & go station (1x1): a narrow shelf unit with a round mirror and an arsenal of sprays ----------
SPRAY = (0.13, 0.935, -0.28)   # the spray bottle's foot (x, height, depth)


def spritzStation():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.6, 0.03, 0.26, (0, 0.92, WALL + 0.13), M.gloss_white, bevel=0.008)
        F.box('lower_shelf', 0.56, 0.03, 0.24, (0, 0.42, WALL + 0.13), M.gloss_white, bevel=0.008)
        for x in (-0.28, 0.28):
            F.box(_name('side'), 0.04, 0.92, 0.24, (x, 0.46, WALL + 0.13), M.shelf, bevel=0.01)
        F.cyl('mirror_frame', 0.23, 0.03, (0, 1.35, WALL + 0.015), M.shelf, rot=(90, 0, 0), verts=32)
        F.cyl('mirror', 0.2, 0.01, (0, 1.35, WALL + 0.032), M.mirror, rot=(90, 0, 0), verts=32)
        F.box('mirror_post', 0.04, 0.28, 0.03, (0, 1.06, WALL + 0.015), M.shelf, bevel=0.008)
        for i, (m, hh) in enumerate((('pop_blue', 0.22), ('pop_red', 0.18), ('pop_yellow', 0.24), ('pop_green', 0.2))):
            x = -0.24 + i * 0.09
            F.cyl(_name('can'), 0.03, hh, (x, 0.935 + hh / 2, -0.3), _m(m), verts=12)
            F.cyl(_name('cap'), 0.022, 0.04, (x, 0.955 + hh, -0.3), M.white, verts=10)
        for i, m in enumerate(('pop_purple', 'pop_blue', 'pop_red')):
            F.cyl(_name('bottle'), 0.035, 0.16, (-0.18 + i * 0.12, 0.515, -0.3), _m(m), verts=12)
        F.box('towels', 0.24, 0.04, 0.2, (0.16, 0.455, -0.3), M.towel, bevel=0.015)
        F.box('towel_hung', 0.36, 0.5, 0.03, (0.24, 0.74, -0.15), M.towel_green, rot=(0, -8, 0), bevel=0.012)
        G.boulder('laundry', 0.2, at=F.at(-0.14, 0, 0.14), material=M.laundry, scale=(1.05, 0.85, 0.5), seed=1.0,
                  noise=0.3, subdivisions=2)
        G.boulder('laundry_red', 0.13, at=F.at(0.08, 0, 0.24), material=M.laundry_red, scale=(1.0, 0.85, 0.55), seed=2.0,
                  noise=0.3, subdivisions=2)
        _can(F, 0.32, 0.3, 0.0, M.pop_yellow, lying=True, rot=-30)
    G.join(still, 'body', origin=(0, 0, 0))
    # The trigger spray bottle, hinged on its foot: `spritz` tips it 35 degrees towards the front.
    sx, sy, sz = SPRAY
    with G.new_parts() as spray:
        G.revolve('spray_bottle', [(0, 0), (0.035, 0), (0.037, 0.14), (0.025, 0.17), (0.014, 0.19), (0, 0.19)],
                  at=F.at(sx, sy, sz), material=M.pop_orange, segments=14)
        F.box('spray_head', 0.03, 0.04, 0.07, (sx, sy + 0.21, sz + 0.015), M.white, bevel=0.008)
        F.box('spray_trigger', 0.012, 0.05, 0.015, (sx, sy + 0.18, sz + 0.035), M.white, rot=(15, 0, 0))
        F.cyl('spray_nozzle', 0.008, 0.02, (sx, sy + 0.215, sz + 0.055), M.pop_red, rot=(90, 0, 0), verts=8)
    spray = G.hinged(G.join(spray, 'spray'), F.at(sx, sy, sz + 0.035))
    objs = G.settle(centre=False)
    G.swing(spray, 'spritz', 35, seconds=0.4, axis='X')
    return objs


# --- the pixel-blaster arcade cabinet (1x1): an upright cabinet, lit marquee, tilted screen, stick and buttons ---
STICK = (-0.14, 1.04, 0.15)   # the joystick's pivot


def arcadeCabinet():
    F = G.GameFrame()
    with G.new_parts() as still:
        for x in (-0.33, 0.33):
            F.box(_name('side'), 0.04, 1.84, 0.6, (x, 0.92, -0.13), M.side, bevel=0.012)
            F.box(_name('side_stripe'), 0.044, 0.2, 0.602, (x, 0.5, -0.13), M.pop_orange)
            F.box(_name('side_stripe'), 0.044, 0.06, 0.602, (x, 0.66, -0.13), M.pop_red)
        F.box('lower_body', 0.62, 0.84, 0.56, (0, 0.42, -0.15), M.cabinet, bevel=0.008)
        F.box('back', 0.62, 1.0, 0.1, (0, 1.34, WALL + 0.05), M.cabinet, bevel=0.008)
        F.box('top', 0.66, 0.06, 0.6, (0, 1.85, -0.13), M.side, bevel=0.01)
        F.box('marquee', 0.62, 0.24, 0.1, (0, 1.7, 0.08), M.marquee, rot=(-10, 0, 0), bevel=0.01)
        F.box('marquee_logo', 0.5, 0.12, 0.004, (0, 1.71, 0.135), M.pop_red, rot=(-10, 0, 0))
        F.box('bezel', 0.6, 0.5, 0.06, (0, 1.28, -0.04), M.screen, rot=(-18, 0, 0), bevel=0.01)
        F.box('screen', 0.48, 0.38, 0.004, (0, 1.28, -0.006), M.screen_lit, rot=(-18, 0, 0))
        F.box('sprite_a', 0.12, 0.1, 0.004, (-0.1, 1.3, -0.001), M.pop_yellow, rot=(-18, 0, 0))
        F.box('sprite_b', 0.08, 0.06, 0.004, (0.12, 1.2, -0.022), M.pop_red, rot=(-18, 0, 0))
        F.box('deck', 0.62, 0.08, 0.3, (0, 0.98, 0.15), M.cabinet, rot=(12, 0, 0), bevel=0.01)
        F.box('deck_art', 0.58, 0.004, 0.24, (0, 1.02, 0.16), M.pop_orange, rot=(12, 0, 0))
        for i, (dx, dz) in enumerate(((0.06, 0.12), (0.13, 0.11), (0.2, 0.12), (0.08, 0.2), (0.15, 0.19))):
            F.cyl(_name('button'), 0.02, 0.02, (dx, 1.035 + (0.15 - dz) * 0.2, dz), _m(POP[i]), rot=(12, 0, 0), verts=12)
        F.box('coin_door', 0.22, 0.26, 0.012, (0, 0.5, 0.13), M.tyre, bevel=0.006)
        for dx in (-0.05, 0.05):
            F.box(_name('coin_slot'), 0.03, 0.06, 0.004, (dx, 0.55, 0.138), M.pop_red)
        F.box('kick', 0.6, 0.05, 0.54, (0, 0.025, -0.15), M.dark)
        _can(F, 0.38, 0.2, 0.0, M.pop_green)
        _wrapper(F, -0.3, 0.32, M.bag_yellow, 7)
    G.join(still, 'body', origin=(0, 0, 0))
    # The joystick (shaft and ball), its origin on the pivot in the deck: `joystick` waggles it while played.
    sx, sy, sz = STICK
    with G.new_parts() as stick:
        F.cyl('stick_boot', 0.025, 0.015, (sx, sy + 0.005, sz), M.dark, verts=12)
        F.cyl('stick_shaft', 0.008, 0.08, (sx, sy + 0.04, sz), M.tyre, verts=8)
        F.sphere('stick_ball', 0.03, (sx, sy + 0.09, sz), M.pop_red, segments=12, rings=8)
    stick = G.hinged(G.join(stick, 'joystick'), F.at(sx, sy, sz))
    objs = G.settle(centre=False)
    G.rock(stick, 'joystick', 18, seconds=0.6, axis='Y')
    return objs


# --- the command-couch gaming recliner (2x1): one seat in the middle, a mini-fridge table and a console dock ----
HINGE = (0.42, -0.3)    # the back's hinge (height, depth)
CUP = (0.37, 0.6, 0.12)  # the cup holder's pivot (x, height, depth), at the armrest's front


def gamingRecliner():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 0.82, 0.3, 0.66, (0, 0.17, -0.06), M.seat, bevel=0.05, bevel_segments=3)
        F.box('seat', 0.6, 0.12, 0.58, (0, 0.39, -0.02), M.stripe, bevel=0.05, bevel_segments=3)
        F.box('footrest', 0.5, 0.1, 0.24, (0, 0.34, 0.32), M.seat, rot=(10, 0, 0), bevel=0.04, bevel_segments=2)
        for s in (-1, 1):
            F.box(_name('arm'), 0.14, 0.32, 0.66, (s * 0.36, 0.45, -0.06), M.seat, bevel=0.05, bevel_segments=3)
            F.box(_name('led'), 0.008, 0.02, 0.6, (s * 0.432, 0.32, -0.06), M.cyan)
        F.box('led_front', 0.8, 0.02, 0.008, (0, 0.04, 0.27), M.cyan)
        # the right: a mini-fridge side table with cans behind its door, a chip bag on top
        fx = 0.7
        F.box('fridge', 0.38, 0.56, 0.42, (fx, 0.28, -0.12), M.gloss_white, bevel=0.025)
        F.box('fridge_door', 0.34, 0.5, 0.012, (fx, 0.28, 0.096), M.window, bevel=0.01)
        F.box('fridge_led', 0.36, 0.02, 0.008, (fx, 0.05, 0.1), M.cyan)
        for i, m in enumerate(('pop_red', 'pop_blue', 'pop_green')):
            _can(F, fx - 0.1 + i * 0.1, 0.06, 0.33, _m(m))
        _chip_bag(F, fx, -0.12, 0.56, M.bag_yellow, rot=20, tilt=-60)
        # the left: a console dock with a headset on its stand and a controller
        lx = -0.7
        F.box('dock', 0.36, 0.4, 0.36, (lx, 0.2, -0.14), M.seat, bevel=0.025)
        F.box('console', 0.3, 0.07, 0.24, (lx, 0.435, -0.16), M.gloss_white, bevel=0.012)
        F.box('console_led', 0.08, 0.008, 0.002, (lx + 0.05, 0.44, -0.039), M.cyan)
        F.cyl('headset_stand', 0.012, 0.36, (lx, 0.65, -0.24), M.chrome, verts=8)
        G.torus('headset_band', 0.1, 0.014, at=F.at(lx, 0.8, -0.24), material=M.stripe, rot=(90, 0, 0),
                major_segments=16, minor_segments=6)
        for dx in (-0.1, 0.1):
            F.cyl(_name('ear_cup'), 0.045, 0.05, (lx + dx, 0.76, -0.24), M.seat, rot=(0, 90, 0), verts=14)
        F.box('controller', 0.14, 0.03, 0.08, (lx + 0.08, 0.455, 0.0), M.gloss_white, bevel=0.012)
        _wrapper(F, 0.3, 0.38, M.bag_orange, 8)
        _wrapper(F, -0.45, 0.32, M.bag_purple, 9)
        _can(F, -0.3, 0.38, 0.0, M.pop_blue, lying=True, rot=15)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tall back with its stripe and headrest pillow, hinged at the seat's back: `recline` lays it back 20°.
    hy, hz = HINGE
    with G.new_parts() as back:
        F.box('back', 0.66, 0.78, 0.16, (0, 0.72, -0.27), M.seat, rot=(-14, 0, 0), bevel=0.06, bevel_segments=3)
        F.box('back_stripe', 0.42, 0.62, 0.06, (0, 0.74, -0.18), M.stripe, rot=(-14, 0, 0), bevel=0.025, bevel_segments=2)
        F.box('headrest', 0.3, 0.14, 0.1, (0, 1.02, -0.24), M.gloss_white, rot=(-14, 0, 0), bevel=0.04, bevel_segments=2)
    back = G.hinged(G.join(back, 'back'), F.at(0, hy, hz))
    # The cup holder with a soda in it, tucked under the right armrest's front: `cup_out` swings it out.
    cx, cy, cz = CUP
    with G.new_parts() as cup:
        F.cyl('cup_ring', 0.045, 0.02, (cx + 0.02, cy, cz - 0.07), M.dark, verts=14)
        F.box('cup_arm', 0.03, 0.015, 0.08, (cx, cy - 0.005, cz - 0.03), M.dark)
        F.cyl('soda', 0.033, 0.12, (cx + 0.02, cy + 0.05, cz - 0.07), M.pop_red, verts=12)
        F.cyl('soda_top', 0.028, 0.006, (cx + 0.02, cy + 0.113, cz - 0.07), M.chrome, verts=10)
    cup = G.hinged(G.join(cup, 'cup_holder'), F.at(cx, cy, cz))
    objs = G.settle(centre=False)
    G.swing(back, 'recline', -20, seconds=1.0, axis='X')
    G.swing(cup, 'cup_out', 110, seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_slob', M,
    [G.Model('microwaveCart', microwaveCart, size=(0.722, 1.099, 0.783), key='model.slob.microwaveCart',
             centre=(0.011, -0.0386), motions={'uses': {'nuke': 'door_open', 'invent': 'door_open'}, 'hold': 1.5},
             paint=[('body', 'Body')]),
     G.Model('beanbag', beanbag, size=(0.888, 0.663, 0.875), key='model.slob.beanbag',
             centre=(0, 0.007), motions={'uses': {'flop': 'squash', 'snooze': 'squash'}},
             paint=[('bag', 'Body')]),
     G.Model('snackNightstand', snackNightstand, size=(0.897, 0.45, 0.728), key='model.slob.snackNightstand',
             centre=(0.021, 0.031), motions={'uses': {'midnightSnack': 'drawer_open'}},
             paint=[('stand', 'Body'), ('pad', 'Top')]),
     G.Model('spritzStation', spritzStation, size=(0.765, 1.58, 0.798), key='model.slob.spritzStation',
             centre=(0.069, -0.031), motions={'uses': {'spritz': 'spritz'}},
             paint=[('shelf', 'Body')]),
     G.Model('arcadeCabinet', arcadeCabinet, size=(0.765, 1.88, 0.784), key='model.slob.arcadeCabinet',
             centre=(0.031, -0.039), motions={'loops': {'chaseHighScore': 'joystick', 'quickRound': 'joystick'}},
             paint=[('cabinet', 'Body'), ('side', 'Sides')]),
     G.Model('gamingRecliner', gamingRecliner, size=(1.77, 1.103, 0.874), key='model.slob.gamingRecliner',
             centre=(0.005, 0.01), motions={'uses': {'marathon': 'recline', 'armrestSoda': 'cup_out'}, 'ride': 'recline'},
             paint=[('seat', 'Body'), ('stripe', 'Stripes')])],
    comment='Blender-built slob pack models (tools/art/blender/families/pack_slob.py). Replaces the primitive-part '
            'placeholders of packs/slob by key; the placeholder is kept as a fallback if a file fails to load, and the '
            'materials map onto the game\'s finishes so a model costs one draw call per finish group. Each moves while '
            'it is used: the microwave\'s door opens, the beanbag squashes, the snack drawer slides out, the spray '
            'tips, the joystick waggles, and the recliner reclines and swings its cup holder out.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
