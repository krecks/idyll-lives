# The slob pack (P16): the late-night microwave cart, the crumb-proof beanbag, the snack-stash nightstand, the
# spritz & go station, the pixel-blaster arcade cabinet and the command-couch gaming recliner.
# Notes: pack_slob.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), with its moving parts: the microwave's door opens and shuts (`door_open`), the beanbag squashes under
# its sitter (`squash`), the nightstand's side drawer of snacks slides out (`drawer_open`), the spray bottle tips
# (`spritz`), the arcade joystick waggles (`joystick`, a loop), and the recliner's back reclines (`recline`) and
# its cup holder swings out (`cup_out`).
#
# Cabinets and appliances are lofted through rounded rectangles; the arcade's sides cut from the classic outline
# (`G.extrude`); cushions, pads, snack bags and the bag itself stuffed (`G.soft_box`) or lofted and wrinkled
# (`G.jitter`); cans, buttons and wheels turned; tubes and handles bent in one piece (`G.kit.tube`, `G.kit.fillet`).
#
# Run:  cargo art pack_slob [--only beanbag] [--preview] [--lit]
import math
import os
import random
import sys

import bpy
from mathutils import Matrix

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
    marquee=('#F2D14E', 'gloss', {'glow': '#FFE08A'}),   # the lit marquee: it glows at night
    screen=('#1B2240', 'gloss'),
    screen_lit=('#3BA3E0', 'gloss', {'glow': '#5CB8F0', 'screen': True}),   # lit while it's played
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


def _m(name):
    return getattr(M, name)


def _box(F, name, w, h, d, at, material, sq=8, verts=24, ease=0.006):
    """A rounded-cornered box lofted upright (a case, a cabinet), its top and bottom edges eased by `ease`."""
    x, y, z = at
    bx, by, bz = F.at(x, y - h / 2, z)
    rings = [(0.0, -ease), (ease, 0.0), (h - ease, 0.0), (h, -ease)]
    return G.loft(name, [G.ring(w / 2 + e, d / 2 + e, bz + hh, verts, square=sq, at=(bx, by)) for hh, e in rings],
                  material=material)


def _bake(objs):
    """Bakes each part's placing into its mesh, so it can be turned about the world's origin."""
    bpy.context.view_layer.update()
    for o in objs:
        o.data.transform(o.matrix_world)
        o.matrix_world = Matrix.Identity(4)
    return objs


def _place(objs, at, yaw=0.0, tilt=0.0, lift=0.0):
    """Turns parts built about the origin (moved up `lift` first) `tilt` degrees about Blender X, then `yaw` about Z,
    and moves them to `at`."""
    m = (Matrix.Translation(at) @ Matrix.Rotation(math.radians(yaw), 4, 'Z') @ Matrix.Rotation(math.radians(tilt), 4, 'X') @
         Matrix.Translation((0, 0, lift)))
    for o in _bake(objs):
        o.data.transform(m)
    return objs


def _can(F, name, x, z, y, material, lying=False, rot=0):
    """A soda can on `y`, standing or lying turned `rot` degrees: a body necked in at both ends, a ring pull."""
    h, r = 0.12, 0.033
    can = G.revolve(name, [(0, 0), (r * 0.8, 0), (r, 0.012), (r, h - 0.012), (r * 0.82, h - 0.002), (r * 0.82, h), (0, h - 0.002)],
                    material=material, segments=12)
    top = G.revolve(f'{name}_top', [(0, h - 0.002), (r * 0.8, h - 0.002), (r * 0.82, h + 0.002), (r * 0.7, h), (0, h)],
                    material=M.chrome, segments=10)
    if lying:                                               # turned about its middle, lying on its side
        _place([can, top], F.at(x, y + r, z), yaw=rot, tilt=90, lift=-h / 2)
    else:
        _place([can, top], F.at(x, y, z), yaw=rot)


def _chip_bag(F, name, x, z, y, material, rot=0, tilt=0, w=0.16, h=0.22):
    """A bag of chips: a puffed pillow, crimped flat at its top and bottom, its label band."""
    bag = F.soft(name, w, h, 0.06, (0, 0, 0), material, round=0.02, puff=0.02, flat=2, round_segments=1)
    seals = [F.box(f'{name}_seal{s}', w, 0.02, 0.008, (0, s * (h / 2 - 0.006), 0), material) for s in (-1, 1)]
    band = F.box(f'{name}_band', w * 0.7, h * 0.3, 0.064, (0, 0.01, 0), M.white)
    _place([bag, band] + seals, F.at(x, y + h / 2, z), yaw=-rot, tilt=tilt)


def _wrapper(F, name, x, z, material, seed=0, y=0.0):
    """A crumpled wrapper on the floor."""
    G.boulder(name, 0.04, at=F.at(x, y, z), material=material, scale=(1.1, 0.9, 0.45), seed=seed, noise=0.4,
              subdivisions=1)


# --- the late-night microwave cart (1x1): a rolling cart, a microwave on top, frozen dinners below ------------
MW = dict(x=-0.02, y=0.815, z0=-0.42, w=0.44, h=0.29, d=0.36)


def _castor(F, name, x, z):
    """A castor: a swivel plate, a fork and a tyred wheel."""
    F.box(f'{name}_plate', 0.04, 0.008, 0.04, (x, 0.074, z), M.chrome, bevel=0.003, bevel_segments=1)
    F.sweep(f'{name}_fork', [(x - 0.018, 0.07, z), (x - 0.018, 0.035, z + 0.008)], None, M.chrome, profile=[(-0.002, -0.008),
            (0.002, -0.008), (0.002, 0.008), (-0.002, 0.008)], up=(1, 0, 0))
    G.revolve(f'{name}_wheel', [(0.0, -0.01), (0.018, -0.01), (0.026, -0.006), (0.027, 0.0), (0.026, 0.006), (0.018, 0.01),
                                (0.0, 0.01)], at=F.at(x, 0.027, z + 0.008), material=M.tyre, segments=12, rot=(0, 90, 0))


def microwaveCart():
    F = G.GameFrame()
    rnd = random.Random(4)
    x, y, z0, w, h, d = (MW[k] for k in ('x', 'y', 'z0', 'w', 'h', 'd'))
    zf, pw = z0 + d, 0.12
    with G.new_parts() as still:
        for sy in (0.8, 0.42, 0.1):                                  # three trays, each with a raised lip
            tw, td = (0.7, 0.46) if sy > 0.5 else (0.66, 0.42)
            bx, by, bz = F.at(0, sy - 0.015, -0.22)
            G.loft(f'tray{sy}', [G.ring(tw / 2 * k, td / 2 * k, bz + hh, 20, square=8, at=(bx, by)) for hh, k in
                                 ((0.0, 0.98), (0.006, 1.0), (0.05, 1.0), (0.054, 0.985), (0.012, 0.975), (0.012, 0.0))],
                   material=M.white)
            F.box(f'tray_band{sy}', tw - 0.04, 0.02, 0.006, (0, sy + 0.012, -0.22 + td / 2 + 0.001), M.body)
        for cx in (-0.33, 0.33):
            for cz in (-0.42, -0.02):
                F.sweep(f'post{cx}{cz}', [(cx, 0.075, cz), (cx, 0.84, cz)], 0.016, M.body, verts=10)
                _castor(F, f'castor{cx}{cz}', cx, cz)
        G.kit.tube(F, 'handle', G.kit.fillet([(0.33, 0.84, -0.4), (0.37, 0.95, -0.4), (0.37, 0.95, -0.04), (0.33, 0.84, -0.04)],
                                             0.03), 0.012, M.chrome, verts=8, round=1)
        F.sweep('handle_grip', [(0.37, 0.95, -0.33), (0.37, 0.95, -0.11)], 0.016, M.tyre, verts=10)
        # the microwave: its case with a cavity cut in (a turntable and a bowl inside), the keypad panel, feet
        case = _box(F, 'mw_case', w, h - 0.012, d - 0.01, (x, y + 0.006 + (h - 0.012) / 2, z0 + (d - 0.01) / 2), M.gloss_white,
                    sq=10, ease=0.01)
        cav = (w - pw - 0.06, h - 0.07, d - 0.08)
        G.cut(case, F.box('mw_cut', cav[0], cav[1], cav[2] + 0.02, (x - pw / 2 - 0.01, y + h / 2, zf - cav[2] / 2), M.dark))
        F.box('mw_cavity', cav[0], cav[1], 0.004, (x - pw / 2 - 0.01, y + h / 2, zf - cav[2] - 0.004), M.panel)
        F.cyl('mw_plate', 0.1, 0.006, (x - pw / 2 - 0.01, y + 0.042, zf - cav[2] / 2), M.panel, verts=16)
        G.revolve('mw_bowl', [(0, 0), (0.04, 0), (0.06, 0.05), (0.055, 0.05), (0.036, 0.006), (0, 0.006)],
                  at=F.at(x - pw / 2 - 0.01, y + 0.045, zf - cav[2] / 2), material=M.pop_orange, segments=12)
        F.cyl('mw_noodles', 0.052, 0.01, (x - pw / 2 - 0.01, y + 0.085, zf - cav[2] / 2), M.cardboard, verts=12)
        F.soft('mw_keypad', pw, h - 0.04, 0.01, (x + w / 2 - pw / 2 - 0.01, y + h / 2, zf - 0.002), M.panel, round=0.004,
               round_segments=1)
        F.box('mw_display', pw - 0.03, 0.04, 0.004, (x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.004), M.display)
        F.box('mw_digits', pw - 0.07, 0.016, 0.003, (x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.006), M.lcd)
        for i in range(3):
            for j in range(3):
                F.soft(f'mw_key{i}{j}', 0.022, 0.016, 0.006, (x + w / 2 - pw / 2 - 0.04 + i * 0.03, y + h - 0.12 - j * 0.03, zf + 0.004),
                       M.white, round=0.003, round_segments=1)
        G.revolve('mw_dial', [(0, 0), (0.024, 0), (0.024, 0.012), (0.02, 0.02), (0, 0.022)],
                  at=F.at(x + w / 2 - pw / 2 - 0.01, y + 0.06, zf + 0.003), material=M.white, segments=12, rot=(90, 0, 0))
        for fx in (x - w / 2 + 0.04, x + w / 2 - 0.04):
            F.cyl(f'mw_foot{fx}', 0.012, 0.008, (fx, y + 0.004, zf - 0.04), M.tyre, verts=8)
        # frozen dinners, a pizza box, the cool box below, cans, a cutlery jar and dropped wrappers
        yy = 0.435
        for i in range(4):
            F.soft(f'dinner{i}', 0.26, 0.045, 0.2, (-0.16 + rnd.uniform(-0.02, 0.02), yy + 0.0225, -0.22), _m(POP[i % 6]),
                   rot=(0, 0, rnd.uniform(-10, 10)), round=0.006, round_segments=1)
            F.box(f'dinner_label{i}', 0.12, 0.046, 0.1, (-0.16, yy + 0.0225, -0.22), M.white)
            yy += 0.045
        F.soft('pizza_box', 0.3, 0.04, 0.3, (0.16, 0.455, -0.22), M.cardboard, round=0.004, round_segments=1)
        F.cyl('pizza_logo', 0.08, 0.002, (0.16, 0.476, -0.22), M.pop_red, verts=14)
        F.soft('cool_box', 0.28, 0.07, 0.22, (0.0, 0.15, -0.22), M.pop_blue, round=0.012, round_segments=1)
        F.soft('cool_lid', 0.29, 0.02, 0.23, (0.0, 0.195, -0.22), M.pop_orange, round=0.008, round_segments=1)
        F.sweep('cool_handle', [(-0.08, 0.205, -0.22), (-0.06, 0.225, -0.22), (0.06, 0.225, -0.22), (0.08, 0.205, -0.22)], 0.007,
                M.pop_orange, verts=5, resolution=2)
        _can(F, 'can_a', 0.26, -0.05, 0.815, M.pop_red)
        _can(F, 'can_b', -0.29, -0.08, 0.815, M.pop_green)
        G.revolve('jar', [(0, 0), (0.03, 0), (0.035, 0.11), (0.037, 0.12), (0.031, 0.12), (0, 0.11)], at=F.at(0.29, 0.815, -0.38),
                  material=M.gloss_white, segments=10)
        for k, dx in enumerate((-0.01, 0.01)):
            F.sweep(f'fork{k}', [(0.29 + dx, 0.85, -0.38), (0.29 + 3 * dx, 0.97, -0.38)], None, M.chrome,
                    profile=[(-0.002, -0.005), (0.002, -0.005), (0.002, 0.005), (-0.002, 0.005)], up=(0, 0, 1),
                    scale=[1.0, 1.6])
        _wrapper(F, 'wrapper_a', 0.3, 0.3, M.bag_yellow, 1)
        _wrapper(F, 'wrapper_b', -0.25, 0.28, M.bag_red, 2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door with its window and handle, hinged on its left edge: `door_open` swings it open to the left.
    dw = w - pw - 0.02
    dx0 = x - w / 2 + 0.005
    with G.new_parts() as door:
        G.kit.shaker(F, 'mw_door', dw, h - 0.05, (dx0 + dw / 2, y + h / 2, zf + 0.006), 0.016, M.gloss_white, rail=0.04,
                     panel=M.window, recess=0.004)
        F.sweep('mw_handle', [(dx0 + dw - 0.02, y + 0.06, zf + 0.012), (dx0 + dw - 0.02, y + 0.07, zf + 0.03),
                              (dx0 + dw - 0.02, y + h - 0.07, zf + 0.03), (dx0 + dw - 0.02, y + h - 0.06, zf + 0.012)], None,
                M.chrome, profile=G.rounded_rect(0.02, 0.012, 0.005, corner=1), up=(1, 0, 0), resolution=2)
    door = G.hinged(G.join(door, 'door'), F.at(dx0, y + h / 2, zf + 0.004))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(door, 'door_open', -100, seconds=0.6)
    return objs


# --- the crumb-proof beanbag (1x1): sat on and napped on in the middle, a snack bag and a controller by it -----
def _controller(F, name, x, y, z, material, turn=0):
    """A game controller lying on `y`: a lofted body with its two grips, two sticks and a button."""
    bx, by, bz = F.at(x, y, z)
    outline = [(0.07, 0.0), (0.06, 0.035), (0.035, 0.03), (0.0, 0.028), (-0.035, 0.03), (-0.06, 0.035), (-0.07, 0.0),
               (-0.065, -0.04), (-0.045, -0.05), (-0.025, -0.02), (0.025, -0.02), (0.045, -0.05), (0.065, -0.04)]
    body = G.loft(name, [[(bx + u * k, by + v * k, bz + hh) for u, v in outline] for hh, k in ((0.0, 0.92), (0.008, 1.0),
                                                                                              (0.026, 1.0), (0.034, 0.9))],
                  material=material)
    made = [body]
    for dx in (-0.03, 0.03):
        made.append(F.cyl(f'{name}_stick{dx}', 0.011, 0.012, (x + dx, y + 0.04, z), M.tyre, verts=8))
    made.append(F.cyl(f'{name}_button', 0.007, 0.006, (x + 0.045, y + 0.036, z + 0.012), M.pop_red, verts=8))
    m = Matrix.Translation((bx, by, 0)) @ Matrix.Rotation(math.radians(turn), 4, 'Z') @ Matrix.Translation((-bx, -by, 0))
    for o in _bake(made):
        o.data.transform(m)


def beanbag():
    F = G.GameFrame()
    with G.new_parts() as still:
        _chip_bag(F, 'chip_bag', 0.318, 0.305, 0.0, M.bag_yellow, rot=30, tilt=80)
        _controller(F, 'controller', -0.34, 0.0, 0.33, M.dark, turn=-25)
        _can(F, 'can', -0.38, -0.32, 0.0, M.pop_blue, lying=True, rot=40)
        _wrapper(F, 'wrapper_a', 0.32, -0.36, M.bag_red, 3)
        _wrapper(F, 'wrapper_b', 0.4, 0.08, M.bag_green, 4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bag: one lofted body, squat and round, rising into a back behind a soft seat (top 0.45), its
    # vinyl wrinkled, a piped seam round its foot and a carry loop: one part with its origin on the floor: `squash`
    # settles it under its sitter.
    with G.new_parts() as bag:
        n = 28
        bx, by, _ = F.at(0, 0, 0)

        def ring(r, h, back=0.0, front=0.0):
            """A ring `r` round at height `h`, its back half raised by up to `back`, its front lowered by `front`."""
            pts = []
            for i in range(n):
                a = math.tau * i / n
                c, s = math.cos(a), math.sin(a)                     # Blender: +y is the back
                lift = back * max(0.0, s) ** 2 - front * max(0.0, -s) ** 2
                pts.append((bx + r * c, by + r * s, h + lift))
            return pts
        # up the sides, over a rim (high at the back, low at the front) and down into the hollow the sitter sinks
        # into (content `seat`: 0.30 m, 5 cm forward of the middle)
        rings = [ring(0.33, 0.0), ring(0.4, 0.04), ring(0.432, 0.13), ring(0.42, 0.25, 0.04), ring(0.37, 0.34, 0.12, 0.02),
                 ring(0.3, 0.41, 0.25, 0.04), ring(0.2, 0.43, 0.22, 0.05), ring(0.12, 0.36, 0.12, 0.02), ring(0.05, 0.315, 0.03),
                 ring(0.0, 0.31)]
        body = G.loft('bag_body', rings, material=M.bag)
        G.jitter(body, 0.008, size=0.16, seed=2.0, up=0.5)
        for v in body.data.vertices:                                # (its foot stays flat on the floor)
            v.co.z = max(v.co.z, 0.0)
        F.sweep('bag_seam', [(0.405 * math.cos(math.tau * i / n), 0.035, 0.405 * math.sin(math.tau * i / n)) for i in range(n)],
                0.01, M.bag_seam, verts=5, closed=True)
        F.sweep('bag_handle', [(-0.06, 0.2, 0.41), (-0.05, 0.24, 0.432), (0.05, 0.24, 0.432), (0.06, 0.2, 0.41)], None,
                M.bag_seam, profile=[(-0.004, -0.012), (0.004, -0.012), (0.004, 0.012), (-0.004, 0.012)], up=(0, 0, 1),
                resolution=2)
    bag = G.join(bag, 'bag', origin=(0, 0, 0))
    objs = G.settle(centre=False)
    G.squash(bag, 'squash', (1.04, 1.04, 0.86), seconds=0.5)
    return objs


# --- the snack-stash nightstand (1x1): sat on (padded top at 0.45), its side drawer full of snacks -------------
def snackNightstand():
    F = G.GameFrame()
    with G.new_parts() as still:
        _box(F, 'cabinet', 0.56, 0.38, 0.46, (0, 0.19, -0.02), M.stand, sq=10, ease=0.01)
        F.soft('pad', 0.56, 0.075, 0.46, (0, 0.4125 - 0.006, -0.02), M.pad, round=0.03, puff=0.006, piping=0.006, flat=2)
        for k in (-1, 1):                                          # two buttons tufting the pad
            F.sphere(f'pad_button{k}', 0.01, (k * 0.12, 0.445, -0.02), M.stand_front, scale=(1, 1, 0.5), segments=6, rings=3)
        F.soft('lower_drawer', 0.012, 0.13, 0.4, (0.284, 0.1, -0.02), M.stand_front, round=0.004, round_segments=1)
        F.sweep('lower_pull', [(0.29, 0.1, -0.06), (0.305, 0.1, -0.05), (0.305, 0.1, 0.01), (0.29, 0.1, 0.02)], 0.006, M.chrome,
                verts=6, resolution=2)
        F.box('drawer_slot', 0.004, 0.13, 0.4, (0.279, 0.27, -0.02), M.dark)
        G.kit.shaker(F, 'front_panel', 0.48, 0.3, (0, 0.2, 0.212), 0.012, M.stand_front, rail=0.05, panel=M.stand)
        F.box('magazine', 0.2, 0.012, 0.26, (-0.1, 0.006, 0.12), M.pop_purple, rot=(0, 0, 12))   # under its front edge
        for fx in (-0.25, 0.25):
            for fz in (-0.22, 0.18):
                F.cyl(f'foot{fx}{fz}', 0.018, 0.012, (fx, 0.006, fz), M.dark, verts=8)
        # floor clutter: cans, wrappers, a knocked-over alarm clock with its bells, a sock
        _can(F, 'can_a', -0.36, 0.26, 0.0, M.pop_blue, lying=True, rot=70)
        _can(F, 'can_b', -0.38, -0.3, 0.0, M.pop_red)
        _can(F, 'can_c', 0.4, 0.3, 0.0, M.pop_green, lying=True, rot=-20)
        _wrapper(F, 'wrapper_a', -0.2, 0.36, M.bag_yellow, 5)
        _wrapper(F, 'wrapper_b', 0.2, 0.34, M.bag_purple, 6)
        clock = [G.revolve('clock', [(0, -0.03), (0.06, -0.03), (0.062, -0.022), (0.062, 0.022), (0.056, 0.03), (0, 0.03)],
                           material=M.gloss_white, segments=16),
                 G.cylinder('clock_face', 0.05, 0.004, at=(0, 0, 0.031), material=M.display, verts=16)]
        for s in (-1, 1):
            clock.append(G.sphere(f'clock_bell{s}', 0.025, at=(s * 0.04, 0.0, 0.062), material=M.chrome, scale=(1, 1, 0.7),
                                  segments=10, rings=5))
        _place(clock, F.at(-0.34, 0.062, -0.06), yaw=-30, tilt=90)
        F.sweep('sock', [(0.374, 0.012, -0.32), (0.414, 0.012, -0.22), (0.424, 0.014, -0.15), (0.434, 0.012, -0.12)], None,
                M.white, profile=G.rounded_rect(0.02, 0.07, 0.01, corner=2), up=(0, 0, 1), resolution=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The top side drawer with its snacks, closed into the right side: `drawer_open` slides it out to the right.
    with G.new_parts() as drawer:
        F.box('drawer_box', 0.24, 0.1, 0.38, (0.16, 0.27, -0.02), M.white, bevel=0.004, bevel_segments=1)
        F.soft('drawer_front', 0.012, 0.13, 0.42, (0.286, 0.27, -0.02), M.stand_front, round=0.005, round_segments=1)
        F.sweep('drawer_pull', [(0.292, 0.29, -0.07), (0.31, 0.29, -0.06), (0.31, 0.29, 0.02), (0.292, 0.29, 0.03)], 0.006,
                M.chrome, verts=6, resolution=2)
        for i, (dz, m) in enumerate(((-0.12, M.bag_red), (0.0, M.bag_orange), (0.12, M.bag_green))):
            _chip_bag(F, f'snack{i}', 0.15, -0.02 + dz, 0.26, m, rot=90, tilt=-15 + i * 12, w=0.16, h=0.14)
        G.revolve('cookie_a', [(0, 0), (0.045, 0), (0.046, 0.012), (0.04, 0.02), (0, 0.021)], at=F.at(0.17, 0.36, 0.12),
                  material=M.cookie, segments=12)
        F.soft('candy_bar', 0.04, 0.02, 0.14, (0.06, 0.335, -0.08), M.pop_purple, round=0.006, round_segments=1)
    drawer = G.hinged(G.join(drawer, 'drawer'), F.at(0.286, 0.27, -0.02))
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (-0.16, 0, 0), seconds=0.6)
    return objs


# --- the spritz & go station (1x1): a narrow shelf unit with a round mirror and an arsenal of sprays ----------
SPRAY = (0.13, 0.935, -0.28)   # the spray bottle's foot (x, height, depth)
BOARD_TOP = 1.62                # the back board's top, over the mirror


def _aerosol(F, name, x, z, y, h, material):
    """A spray can: a body necked into its shoulder, a cap with its nozzle."""
    G.revolve(name, [(0, 0), (0.026, 0), (0.03, 0.008), (0.03, h - 0.03), (0.022, h - 0.01), (0.014, h), (0, h)],
              at=F.at(x, y, z), material=material, segments=12)
    G.revolve(f'{name}_cap', [(0, 0), (0.022, 0), (0.022, 0.03), (0.016, 0.04), (0, 0.04)], at=F.at(x, y + h - 0.004, z),
              material=M.white, segments=10)


def spritzStation():
    F = G.GameFrame()
    with G.new_parts() as still:
        _box(F, 'top', 0.6, 0.03, 0.26, (0, 0.92, WALL + 0.13), M.gloss_white, sq=10, ease=0.006)
        _box(F, 'lower_shelf', 0.56, 0.03, 0.24, (0, 0.42, WALL + 0.13), M.gloss_white, sq=10, ease=0.006)
        for x in (-0.28, 0.28):
            F.soft(f'side{x}', 0.04, 0.92, 0.24, (x, 0.46, WALL + 0.13), M.shelf, round=0.012, round_segments=1)
        # a back board rising from the top carries the round mirror, standing free or on the wall (2026-10-21: the
        # mirror hung on the air over a free-standing station)
        F.box('backboard', 0.6, BOARD_TOP - 0.935, 0.018, (0, (0.935 + BOARD_TOP) / 2, WALL + 0.009), M.shelf, bevel=0.008,
              bevel_segments=1)
        G.revolve('mirror_frame', [(0.19, 0.0), (0.23, 0.0), (0.234, 0.01), (0.226, 0.022), (0.2, 0.024), (0.19, 0.016)],
                  at=F.at(0, 1.35, WALL + 0.018), material=M.shelf, segments=32, rot=(90, 0, 0))
        F.cyl('mirror', 0.2, 0.01, (0, 1.35, WALL + 0.032), M.mirror, rot=(90, 0, 0), verts=32)
        for i, (m, hh) in enumerate((('pop_blue', 0.22), ('pop_red', 0.18), ('pop_yellow', 0.24), ('pop_green', 0.2))):
            _aerosol(F, f'aerosol{i}', -0.24 + i * 0.09, -0.3, 0.935, hh, _m(m))
        for i, m in enumerate(('pop_purple', 'pop_blue', 'pop_red')):
            bx, by, bz = F.at(-0.18 + i * 0.12, 0.435, -0.3)
            G.loft(f'bottle{i}', [G.ring(0.035 * k, 0.03 * k, bz + hh, 12, square=3, at=(bx, by)) for hh, k in
                                  ((0.0, 0.95), (0.01, 1.0), (0.12, 1.0), (0.15, 0.5), (0.18, 0.4))], material=_m(m))
            F.cyl(f'bottle_cap{i}', 0.016, 0.025, (-0.18 + i * 0.12, 0.627, -0.3), M.white, verts=8)
        for k, m in enumerate((M.towel, M.towel_green)):
            F.soft(f'towels{k}', 0.24, 0.04, 0.2, (0.16, 0.457 + 0.042 * k, -0.3), m, round=0.016, puff=0.004, flat=1,
                   round_segments=1)
        # the towel over a bar under the top's front edge (it hung on nothing at the side)
        F.sweep('towel_bar', [(-0.02, 0.88, -0.165), (-0.02, 0.87, -0.155), (0.26, 0.87, -0.155), (0.26, 0.88, -0.165)], 0.007,
                M.chrome, verts=6)
        F.sweep('towel_hung', [(0.12, 0.47, -0.162), (0.12, 0.86, -0.162), (0.12, 0.882, -0.155), (0.12, 0.872, -0.143),
                               (0.12, 0.62, -0.14)], None, M.towel_green, profile=G.rounded_rect(0.008, 0.26, 0.004, corner=1),
                up=(0, 1, 0), resolution=2)
        for k, (x, z, r, m) in enumerate(((-0.112, 0.14, 0.2, M.laundry), (0.08, 0.234, 0.13, M.laundry_red))):
            heap = F.soft(f'laundry{k}', r * 1.75, r * 0.9, r * 1.7, (x, r * 0.45 + 0.03, z), m, round=r * 0.4,
                          puff=(0.02, 0.03, 0.02), flat=3, round_segments=1)
            G.jitter(heap, 0.02, size=0.08, seed=k + 1.0, up=0.5)
            for v in heap.data.vertices:                            # (flat on the floor)
                v.co.z = max(v.co.z, -r * 0.45 - 0.03 + 0.001)
        F.sweep('laundry_sleeve', [(-0.2, 0.12, 0.22), (-0.24, 0.03, 0.3), (-0.26, 0.022, 0.34)], 0.02, M.laundry, verts=6,
                resolution=2)
        _can(F, 'can', 0.32, 0.3, 0.0, M.pop_yellow, lying=True, rot=-30)
    G.join(still, 'body', origin=(0, 0, 0))
    # The trigger spray bottle, hinged on its foot: `spritz` tips it 35 degrees towards the front.
    sx, sy, sz = SPRAY
    with G.new_parts() as spray:
        bx, by, bz = F.at(sx, sy, sz)
        G.loft('spray_bottle', [G.ring(0.036 * k, 0.03 * k, bz + hh, 14, square=3, at=(bx, by)) for hh, k in
                                ((0.0, 0.95), (0.008, 1.0), (0.13, 1.0), (0.16, 0.55), (0.18, 0.4), (0.19, 0.4))],
               material=M.pop_orange)
        F.soft('spray_head', 0.032, 0.045, 0.075, (sx, sy + 0.21, sz + 0.015), M.white, round=0.012, round_segments=1)
        F.sweep('spray_trigger', [(sx, sy + 0.2, sz + 0.03), (sx, sy + 0.17, sz + 0.045), (sx, sy + 0.15, sz + 0.04)], None, M.white,
                profile=[(-0.005, -0.006), (0.005, -0.006), (0.005, 0.006), (-0.005, 0.006)], up=(1, 0, 0), resolution=2)
        F.cyl('spray_nozzle', 0.008, 0.02, (sx, sy + 0.215, sz + 0.055), M.pop_red, rot=(90, 0, 0), verts=8)
    spray = G.hinged(G.join(spray, 'spray'), F.at(sx, sy, sz + 0.035))
    objs = G.settle(centre=False)
    G.swing(spray, 'spritz', 35, seconds=0.4, axis='X')
    return objs


# --- the pixel-blaster arcade cabinet (1x1): an upright cabinet, lit marquee, tilted screen, stick and buttons ---
STICK = (-0.14, 1.04, 0.15)   # the joystick's pivot


# The cabinet's side, in (depth, height): the kick at the foot, the control deck jutting out, the screen's slope
# back, the marquee's hood overhanging, the back straight down.
SIDE = [(WALL + 0.02, 0.0), (0.12, 0.0), (0.12, 0.84), (0.27, 0.96), (0.27, 1.05), (0.07, 1.08), (-0.02, 1.52), (0.13, 1.6),
        (0.13, 1.82), (WALL + 0.12, 1.88), (WALL + 0.02, 1.88)]


def arcadeCabinet():
    F = G.GameFrame()
    with G.new_parts() as still:
        for x in (-0.33, 0.33):
            F.extrude(f'side{x}', G.smooth_outline(SIDE, passes=1), 0.04, (x, 0, 0), M.side, plane='side', bevel=0.008,
                      bevel_segments=1)
            # the side art: two bands following the deck's line
            F.extrude(f'side_stripe{x}', [(-0.4, 0.42), (0.118, 0.42), (0.118, 0.6), (-0.4, 0.6)], 0.044, (x, 0, 0),
                      M.pop_orange, plane='side')
            F.extrude(f'side_stripe_b{x}', [(-0.4, 0.64), (0.118, 0.64), (0.118, 0.69), (-0.4, 0.69)], 0.044, (x, 0, 0), M.pop_red,
                      plane='side')
        F.box('lower_body', 0.62, 0.84, 0.52, (0, 0.42, -0.15), M.cabinet, bevel=0.006, bevel_segments=1)
        F.box('back', 0.62, 1.04, 0.1, (0, 1.36, WALL + 0.07), M.cabinet, bevel=0.006, bevel_segments=1)
        F.box('top', 0.66, 0.06, 0.6, (0, 1.85, -0.13), M.side, bevel=0.01, bevel_segments=2)
        F.box('marquee', 0.62, 0.22, 0.06, (0, 1.71, 0.1), M.marquee, rot=(-10, 0, 0), bevel=0.008, bevel_segments=1)
        F.box('marquee_logo', 0.5, 0.12, 0.004, (0, 1.715, 0.133), M.pop_red, rot=(-10, 0, 0))
        F.box('marquee_hood', 0.62, 0.02, 0.2, (0, 1.82, 0.03), M.cabinet, bevel=0.004, bevel_segments=1)
        F.box('bezel', 0.62, 0.48, 0.03, (0, 1.29, 0.03), M.screen, rot=(-20, 0, 0))
        F.soft('screen', 0.48, 0.38, 0.01, (0, 1.29, 0.045), M.screen_lit, rot=(-20, 0, 0), round=0.004, puff=0.004, flat=2,
               round_segments=1)
        F.box('sprite_a', 0.12, 0.1, 0.004, (-0.1, 1.31, 0.05), M.pop_yellow, rot=(-20, 0, 0))
        F.box('sprite_b', 0.08, 0.06, 0.004, (0.12, 1.21, 0.084), M.pop_red, rot=(-20, 0, 0))
        F.soft('deck', 0.62, 0.08, 0.3, (0, 0.995, 0.13), M.cabinet, rot=(12, 0, 0), round=0.02, round_segments=1)
        F.box('deck_art', 0.58, 0.004, 0.24, (0, 1.037, 0.14), M.pop_orange, rot=(12, 0, 0))
        for i, (dx, dz) in enumerate(((0.06, 0.12), (0.13, 0.11), (0.2, 0.12), (0.08, 0.2), (0.15, 0.19))):
            G.revolve(f'button{i}', [(0, 0), (0.026, 0), (0.026, 0.006), (0.019, 0.008), (0.019, 0.016), (0.014, 0.022),
                                     (0, 0.023)], at=F.at(dx, 1.04 + (0.15 - dz) * 0.2, dz), material=_m(POP[i]), segments=12,
                      rot=(-12, 0, 0))
        F.soft('coin_door', 0.22, 0.26, 0.012, (0, 0.5, 0.126), M.tyre, round=0.006, round_segments=1)
        for k, dx in enumerate((-0.05, 0.05)):
            F.soft(f'coin_slot{k}', 0.032, 0.062, 0.008, (dx, 0.55, 0.134), M.pop_red, round=0.004, round_segments=1)
            F.box(f'coin_slit{k}', 0.004, 0.03, 0.004, (dx, 0.55, 0.139), M.dark)
        F.box('kick', 0.6, 0.05, 0.54, (0, 0.025, -0.15), M.dark)
        _can(F, 'can', 0.38, 0.2, 0.0, M.pop_green)
        _wrapper(F, 'wrapper', -0.3, 0.32, M.bag_yellow, 7)
    G.join(still, 'body', origin=(0, 0, 0))
    # The joystick (shaft and ball), its origin on the pivot in the deck: `joystick` waggles it while played.
    sx, sy, sz = STICK
    with G.new_parts() as stick:
        G.revolve('stick_boot', [(0, 0), (0.03, 0), (0.03, 0.004), (0.016, 0.014), (0, 0.016)], at=F.at(sx, sy, sz),
                  material=M.dark, segments=12)
        F.sweep('stick_shaft', [(sx, sy, sz), (sx, sy + 0.075, sz)], 0.007, M.chrome, verts=8)
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
        F.soft('base', 0.82, 0.3, 0.66, (0, 0.17, -0.06), M.seat, round=0.05, piping=0.006)
        F.soft('seat', 0.6, 0.12, 0.58, (0, 0.38, -0.02), M.stripe, round=0.045, puff=0.012, flat=2)
        for k in (-0.1, 0.1):                                       # the seat's stitched channels
            F.box(f'seat_channel{k}', 0.008, 0.004, 0.5, (k, 0.452, -0.02), M.seat)
        F.soft('footrest', 0.5, 0.1, 0.24, (0, 0.34, 0.32), M.seat, rot=(10, 0, 0), round=0.04, puff=0.01, flat=2)
        for s in (-1, 1):
            F.soft(f'arm{s}', 0.14, 0.32, 0.66, (s * 0.36, 0.45, -0.06), M.seat, round=0.05, puff=(0, 0.01, 0), flat=2,
                   piping=0.005)
            F.box(f'led{s}', 0.008, 0.02, 0.6, (s * 0.432, 0.32, -0.06), M.cyan, bevel=0.003, bevel_segments=1)
        F.box('led_front', 0.8, 0.02, 0.008, (0, 0.04, 0.27), M.cyan)
        # the right: a mini-fridge side table, cans behind its tinted door, a chip bag on top
        fx = 0.7
        _box(F, 'fridge', 0.38, 0.56, 0.42, (fx, 0.28, -0.12), M.gloss_white, sq=10, ease=0.02)
        F.soft('fridge_door', 0.34, 0.5, 0.012, (fx, 0.28, 0.096), M.window, round=0.01, round_segments=1)
        F.sweep('fridge_handle', [(fx + 0.15, 0.48, 0.1), (fx + 0.15, 0.47, 0.112), (fx + 0.15, 0.32, 0.112), (fx + 0.15, 0.31, 0.1)],
                0.006, M.chrome, verts=6, resolution=2)
        F.box('fridge_led', 0.36, 0.02, 0.008, (fx, 0.05, 0.1), M.cyan)
        for i, m in enumerate(('pop_red', 'pop_blue', 'pop_green')):
            _can(F, f'fridge_can{i}', fx - 0.1 + i * 0.1, 0.06, 0.33, _m(m))
        _chip_bag(F, 'chip_bag', fx, -0.12, 0.56, M.bag_yellow, rot=20, tilt=-60)
        # the left: a console dock, a headset on its stand and a controller
        lx = -0.7
        _box(F, 'dock', 0.36, 0.4, 0.36, (lx, 0.2, -0.14), M.seat, sq=10, ease=0.02)
        _box(F, 'console', 0.3, 0.07, 0.24, (lx, 0.435, -0.16), M.gloss_white, sq=6, ease=0.01)
        for k in range(4):
            F.box(f'console_vent{k}', 0.2, 0.003, 0.008, (lx, 0.471, -0.22 + k * 0.03), M.dark)
        F.box('console_led', 0.08, 0.008, 0.002, (lx + 0.05, 0.44, -0.039), M.cyan)
        F.sweep('headset_stand', [(lx, 0.47, -0.24), (lx, 0.83, -0.24)], 0.012, M.chrome, verts=8)
        G.revolve('headset_foot', [(0, 0), (0.05, 0), (0.05, 0.006), (0.02, 0.012), (0, 0.012)], at=F.at(lx, 0.47, -0.24),
                  material=M.chrome, segments=12)
        F.sweep('headset_band', [(lx - 0.1 + 0.2 * k / 8, 0.76 + 0.09 * math.sin(math.pi * k / 8), -0.24) for k in range(9)], None,
                M.stripe, profile=G.rounded_rect(0.012, 0.035, 0.005, corner=1), up=(0, 0, 1))
        for dx in (-0.1, 0.1):
            G.revolve(f'ear_cup{dx}', [(0, -0.025), (0.042, -0.025), (0.048, -0.01), (0.048, 0.015), (0.04, 0.025), (0, 0.025)],
                      at=F.at(lx + dx, 0.74, -0.24), material=M.seat, segments=14, rot=(0, 90, 0))
        _controller(F, 'controller', lx + 0.08, 0.437, 0.0, M.gloss_white, turn=15)
        _wrapper(F, 'wrapper_a', 0.3, 0.38, M.bag_orange, 8)
        _wrapper(F, 'wrapper_b', -0.45, 0.32, M.bag_purple, 9)
        _can(F, 'can', -0.3, 0.38, 0.0, M.pop_blue, lying=True, rot=15)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tall back with its stripe and headrest pillow, hinged at the seat's back: `recline` lays it back 20°.
    hy, hz = HINGE
    with G.new_parts() as back:
        F.soft('back', 0.66, 0.78, 0.16, (0, 0.72, -0.265), M.seat, rot=(-14, 0, 0), round=0.06, puff=0.015, flat=2)
        F.soft('back_stripe', 0.42, 0.62, 0.06, (0, 0.74, -0.18), M.stripe, rot=(-14, 0, 0), round=0.025, puff=0.012, flat=2)
        F.soft('headrest', 0.3, 0.14, 0.1, (0, 1.02, -0.24), M.gloss_white, rot=(-14, 0, 0), round=0.04, puff=0.02, flat=2)
    back = G.hinged(G.join(back, 'back'), F.at(0, hy, hz))
    # The cup holder with a soda in it, tucked under the right armrest's front: `cup_out` swings it out.
    cx, cy, cz = CUP
    with G.new_parts() as cup:
        G.revolve('cup_ring', [(0.034, -0.01), (0.046, -0.01), (0.048, 0.01), (0.036, 0.01)],
                  at=F.at(cx + 0.02, cy, cz - 0.07), material=M.dark, segments=14)
        F.box('cup_arm', 0.03, 0.015, 0.08, (cx, cy - 0.005, cz - 0.03), M.dark, bevel=0.004, bevel_segments=1)
        _can(F, 'soda', cx + 0.02, cz - 0.07, cy - 0.01, M.pop_red)
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
     G.Model('spritzStation', spritzStation, size=(0.692, 1.62, 0.798), key='model.slob.spritzStation',
             centre=(0.0325, -0.031), motions={'uses': {'spritz': 'spritz'}},
             paint=[('shelf', 'Body')]),
     G.Model('arcadeCabinet', arcadeCabinet, size=(0.765, 1.88, 0.784), key='model.slob.arcadeCabinet',
             centre=(0.031, -0.039), motions={'loops': {'chaseHighScore': 'joystick', 'quickRound': 'joystick'}},
             paint=[('cabinet', 'Body'), ('side', 'Sides')]),
     G.Model('gamingRecliner', gamingRecliner, size=(1.77, 1.103, 0.874), key='model.slob.gamingRecliner',
             centre=(0.005, 0.01), motions={'uses': {'marathon': 'recline', 'armrestSoda': 'cup_out'}, 'ride': 'recline'},
             paint=[('seat', 'Body'), ('stripe', 'Stripes')])],
    comment='Blender-built slob pack models (tools/art/blender/families/pack_slob.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves while it is used: the microwave\'s door opens, the beanbag '
            'squashes, the snack drawer slides out, the spray tips, the joystick waggles, and the '
            'recliner reclines and swings its cup holder out.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
