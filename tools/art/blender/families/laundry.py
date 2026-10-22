# The laundry: the washing machine (`model.laundryWasher`), the tumble dryer (`model.laundryDryer`), the towel
# rail (`model.towelRail`), the towel ladder (`model.towelLadder`), the laundry basket (`model.laundryBasket`)
# and the bath mat (`model.bathMat`). Notes: laundry.md.
#
# The washer and the dryer stand against the wall where the game places them, built in place (`F`, game
# axes). Each has two moving parts: the round door, hinged on its left, which swings open while a resident
# loads or empties it (`door_open`, held 2.5 s), and the drum's face behind the glass, which on the washer
# turns once the door has shut, for the rest of the wash (`drum_spin`, `every` its turn); the dryer's stays
# still while its washing is taken out to fold. The rail hangs on the wall at its real height; the rest are still.
#
# Run:  cargo art laundry [--only laundryWasher] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#F4F4F2', 'gloss'),       # a machine's cabinet; the main part the player colours
    panel=('#E6E6E3', 'gloss'),      # its control strip
    door=('#C9CBCC', 'metal'),       # the washer's door ring, handle and knob: brushed metal
    door_dryer=('#9AA4AB', 'metal'),  # the dryer's, and its vents
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
    towel_hem=('#7FAEB9', 'fabric'),     # the woven band across a towel's foot
    towel_hem2=('#D9D5CA', 'fabric'),
    wicker_band=('#B09A7C', 'satin'),    # the bands round the dryer's basket
)

WALL = -0.43   # the wall's face, game depth
DOOR_Z = 0.152  # the door ring's middle, game depth
DOOR_Y = 0.4    # the door's middle, height
DOOR_R = 0.21   # the ring's radius


def _rbox(F, name, w, h, d, at, mat, r=0.004, segs=1, **kw):
    """A box with its edges chamfered (`segs` 1) or rounded (2 or more)."""
    return F.box(name, w, h, d, at, mat, bevel=r, bevel_segments=segs, **kw)


def _front_disc(F, name, profile, x, y, z, mat, segments=32):
    """A turned part facing the front from game depth `z`: `profile` [(radius, out), ...] revolved about the depth axis."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=mat, segments=segments, rot=(90, 0, 0))


def _lump(F, name, x, y, z, r, mat, scale=(1, 0.5, 0.8), seed=0.0, turn=0.0):
    """A crumple of washing: a squashed ball, lumpy (Blender scale)."""
    obj = G.sphere(name, r, at=F.at(x, y, z), material=mat, scale=scale, rot=(0, turn, 0), segments=10, rings=6)
    return G.jitter(obj, r * 0.18, size=r * 0.9, seed=seed)


# --- the washer and the dryer ---------------------------------------------------------------------------
def _cabinet(F, led, dryer, ring):
    """The still cabinet: the body under a top panel, a recessed plinth, the control strip with a soap drawer or a
    filter, a program knob in its ring of settings, a display and buttons, the rubber bellows round the porthole."""
    F.soft('cabinet', 0.6, 0.84, 0.6, (0, 0.44, -0.16), M.body, round=0.03, round_segments=2)
    _rbox(F, 'top', 0.6, 0.014, 0.6, (0, 0.853, -0.16), M.body, r=0.006)
    _rbox(F, 'plinth', 0.56, 0.02, 0.56, (0, 0.01, -0.16), M.dark, r=0.006)
    F.box('kick', 0.5, 0.012, 0.002, (0, 0.035, 0.1405), M.dark)
    _rbox(F, 'strip', 0.6, 0.13, 0.012, (0, 0.78, 0.144), M.panel, r=0.006)
    # the soap drawer (washer) or the water tank (dryer): a front with a scoop grip, its compartments' marks
    _rbox(F, 'drawer', 0.14, 0.07, 0.012, (-0.18, 0.78, 0.152), M.body, r=0.005)
    _rbox(F, 'drawer_grip', 0.08, 0.012, 0.006, (-0.18, 0.759, 0.159), M.dark, r=0.003)
    for k in range(3):
        F.box(f'drawer_mark{k}', 0.012, 0.004, 0.001, (-0.225 + 0.03 * k, 0.8, 0.1585), M.dark)
    # the display: the time left, the light by it, three buttons under
    _rbox(F, 'display', 0.12, 0.05, 0.006, (0.02, 0.785, 0.153), M.dark, r=0.004)
    for k, w in enumerate((0.012, 0.004, 0.012, 0.012)):
        F.box(f'time{k}', w, 0.018, 0.001, (-0.005 + 0.016 * k, 0.79, 0.1562), led)
    F.box('light', 0.03, 0.006, 0.002, (0.02, 0.769, 0.157), led)
    for k in range(3):
        _front_disc(F, f'button{k}', [(0, 0.0), (0.008, 0.0), (0.008, 0.004), (0.006, 0.006), (0, 0.006)],
                    -0.02 + 0.04 * k, 0.738, 0.15, M.body, segments=8)
    # the program knob's ring of settings
    _front_disc(F, 'knob_skirt', [(0, 0.0), (0.046, 0.0), (0.046, 0.004), (0.04, 0.006), (0, 0.006)], 0.18, 0.78, 0.15,
                M.panel, segments=24)
    for k in range(10):
        a = math.radians(-150 + k * 300 / 9)
        F.box(f'setting{k}', 0.004, 0.004, 0.001, (0.18 + 0.056 * math.sin(a), 0.78 + 0.056 * math.cos(a), 0.1505),
              M.dark if k % 3 else led)
    # the bellows: a rubber lip round the opening, behind the door
    _front_disc(F, 'seal', [(0.205, -0.004), (0.205, 0.0), (0.19, 0.004), (0.172, 0.0), (0.168, -0.01), (0.15, -0.012),
                            (0.205, -0.004)],
                0, DOOR_Y, 0.146, M.dark, segments=24)
    if dryer:
        # the vents along the foot, the condenser's hatch beside them
        for k, y in enumerate((0.075, 0.1, 0.125)):
            _rbox(F, f'vent{k}', 0.4, 0.008, 0.01, (0, y, 0.142), M.door_dryer, r=0.003)
    else:
        _rbox(F, 'filter', 0.12, 0.06, 0.008, (0.2, 0.08, 0.142), M.panel, r=0.004)
        F.box('filter_slot', 0.04, 0.006, 0.002, (0.2, 0.1, 0.1465), M.dark)


def _door(F, ring):
    """The door: a turned ring with its window domed in it and a grip on the right, hinged on its left edge."""
    with G.new_parts() as parts:
        _front_disc(F, 'door_ring', [(DOOR_R - 0.035, DOOR_Z - 0.006), (DOOR_R + 0.012, DOOR_Z - 0.006),
                                     (DOOR_R + 0.012, DOOR_Z + 0.006), (DOOR_R + 0.004, DOOR_Z + 0.022),
                                     (DOOR_R - 0.012, DOOR_Z + 0.03), (DOOR_R - 0.026, DOOR_Z + 0.026),
                                     (DOOR_R - 0.035, DOOR_Z + 0.014), (DOOR_R - 0.035, DOOR_Z - 0.006)], 0, DOOR_Y, 0, ring,
                    segments=32)
        # the grip: a bar standing out of the ring's right side
        F.sweep('handle', G.kit.fillet([(DOOR_R - 0.005, DOOR_Y - 0.045, DOOR_Z + 0.012),
                                        (DOOR_R + 0.012, DOOR_Y - 0.045, DOOR_Z + 0.03),
                                        (DOOR_R + 0.012, DOOR_Y + 0.045, DOOR_Z + 0.03),
                                        (DOOR_R - 0.005, DOOR_Y + 0.045, DOOR_Z + 0.012)], 0.012),
                None, ring, profile=G.rounded_rect(0.012, 0.022, 0.005, corner=1), up=(0, 1, 0), angle=50)
        _front_disc(F, 'window', [(0, DOOR_Z + 0.028), (DOOR_R - 0.08, DOOR_Z + 0.024), (DOOR_R - 0.03, DOOR_Z + 0.012),
                                  (DOOR_R - 0.025, DOOR_Z), (DOOR_R - 0.04, DOOR_Z - 0.004), (0, DOOR_Z - 0.004)],
                    0, DOOR_Y, 0, M.glass, segments=32)
    door = G.join(parts, 'door')
    return G.hinged(door, F.at(-DOOR_R - 0.02, DOOR_Y, DOOR_Z))


def _drum(F, washing, seed):
    """The drum's face behind the window: its dark inside with a ring of holes, three lifters and the washing,
    joined round the middle so it turns about its axis."""
    with G.new_parts() as parts:
        F.cyl('drum_face', 0.19, 0.006, (0, DOOR_Y, 0.146), M.drum, rot=(90, 0, 0), verts=32)
        for k in range(12):
            a = math.tau * k / 12
            F.box(f'hole{k}', 0.008, 0.008, 0.001, (0.165 * math.cos(a), DOOR_Y + 0.165 * math.sin(a), 0.1492), M.paddle,
                  rot=(0, math.degrees(a), 0))
        for k in range(3):
            a = 120 * k
            x, y = 0.14 * G.sind(a), DOOR_Y + 0.14 * G.cosd(a)
            F.extrude(f'paddle{k}', G.smooth_outline([(-0.016, -0.04), (0.016, -0.04), (0.01, 0.04), (-0.01, 0.04)],
                                                     passes=1), 0.012, (x, y, 0.153), M.paddle, rot=(0, -a, 0),
                      bevel=0.003, bevel_segments=1)
        for k, (m, dx, dy, r) in enumerate(washing):
            _lump(F, f'wash{k}', dx, DOOR_Y + dy, 0.153, r, m, scale=(1, 0.18, 0.8), seed=seed + k, turn=15 * k)
    drum = G.join(parts, 'drum')
    return G.hinged(drum, F.at(0, DOOR_Y, 0.146))


def _folded(F, name, x, y, z, w, d, h, mat, turn=0.0):
    """A folded towel lying on `y`: a soft slab, its folded edge rolled to the front."""
    _rbox(F, name, w, h, d, (x, y + h / 2, z), mat, r=min(0.02, h * 0.45), segs=2, rot=(0, 0, turn))


def _machine(dryer):
    F = G.GameFrame(shift=0.03)      # built 3 cm into the wall (as the old part was); moved onto its face
    led, ring = (M.led_dryer, M.door_dryer) if dryer else (M.led, M.door)
    with G.new_parts() as still:
        _cabinet(F, led, dryer, ring)
        _front_disc(F, 'knob', [(0, 0.0), (0.033, 0.0), (0.034, 0.014), (0.03, 0.026), (0, 0.028)], 0.18, 0.78, 0.154,
                    ring, segments=18)
        _rbox(F, 'knob_pointer', 0.006, 0.026, 0.006, (0.18, 0.795, 0.181), M.dark, r=0.002)
        if dryer:
            # a wicker basket of dry washing on the top, its weave in bands, handles at its ends
            G.revolve('basket', [(0, 0), (0.185, 0), (0.21, 0.18), (0.195, 0.18), (0.17, 0.02), (0, 0.02)],
                      at=F.at(-0.06, 0.86, -0.14), material=M.wicker_plain, segments=24, scale=(1, 0.76, 1))
            for k, (yy, r) in enumerate(((0.05, 0.193), (0.1, 0.199), (0.15, 0.205))):
                loop = [(-0.06 + r * math.cos(math.tau * i / 24), 0.86 + yy, -0.14 + 0.76 * r * math.sin(math.tau * i / 24))
                        for i in range(24)]
                F.sweep(f'basket_band{k}', loop, 0.004, M.wicker_band, verts=4, closed=True)
            G.sphere('pile', 0.15, at=F.at(-0.06, 1.0, -0.14), material=M.wash_c, scale=(1, 0.75, 0.38),
                     segments=16, rings=8)
            _lump(F, 'pile_top', 0.02, 1.05, -0.1, 0.07, M.wash_a, scale=(1.1, 0.9, 0.5), seed=4)
            _lump(F, 'pile_sock', -0.15, 1.04, -0.08, 0.04, M.wash_b, scale=(1.2, 0.6, 0.5), seed=7)
        else:
            # folded towels on the top
            _folded(F, 'fold0', 0.08, 0.86, -0.1, 0.3, 0.22, 0.06, M.wash_c)
            _folded(F, 'fold1', 0.08, 0.92, -0.1, 0.28, 0.2, 0.056, M.wash_b, turn=4)
    G.join(still, 'body', origin=(0, 0, 0))
    washing = ([(M.wash_c, -0.05, -0.06, 0.08), (M.wash_b, 0.06, -0.08, 0.07), (M.wash_a, 0.0, 0.07, 0.06)]
               if dryer else
               [(M.wash_a, -0.04, -0.07, 0.08), (M.wash_b, 0.07, -0.05, 0.06), (M.wash_c, 0.02, 0.06, 0.07)])
    drum = _drum(F, washing, 1 if dryer else 0)
    door = _door(F, ring)
    G.settle(centre=False)
    G.swing(door, 'door_open', 110, seconds=1.0)
    if not dryer:
        G.spin(drum, 'drum_spin', turns=1, seconds=1.2, axis='Y')
    return G.meshes()


def laundryWasher():
    return _machine(dryer=False)


def laundryDryer():
    return _machine(dryer=True)


# --- the towel rail: two chrome bars on the wall, towels over them ----------------------------------------
def _drape(F, name, x, w, bar_y, bar_z, drop_front, drop_back, material, t=0.012, r=0.016, hem=None, sway=0.0):
    """A towel hung over a bar at (bar_y, bar_z) in one piece: down the back, round the bar and down the front, its
    flaps hanging a little apart below the fold; `hem` a band across the front flap near its foot."""
    R = r + t / 2
    pts = [(x, bar_y - drop_back + 0.01, bar_z - R - 0.004), (x, bar_y - drop_back * 0.5, bar_z - R - 0.002)]
    pts += [(x, bar_y + R * math.sin(math.radians(a)), bar_z + R * math.cos(math.radians(a))) for a in range(180, -1, -30)]
    pts += [(x, bar_y - drop_front * 0.5, bar_z + R + 0.002 + sway), (x, bar_y - drop_front + 0.01, bar_z + R + 0.004 + sway)]
    F.sweep(name, pts, None, material, profile=G.rounded_rect(w, t, t * 0.45, corner=2), up=(1, 0, 0), angle=60)
    if hem:
        y = bar_y - drop_front + 0.06
        F.box(f'{name}_hem', w + 0.002, 0.03, t + 0.002, (x, y, bar_z + R + 0.0035 + sway * 0.9), hem)


def towelRail():
    F = G.GameFrame()
    z = -0.36
    with G.new_parts() as parts:
        for y in (1.2, 0.95):
            F.sweep(f'bar{y}', G.kit.fillet([(-0.34, y, WALL + 0.012), (-0.34, y, z), (0.34, y, z), (0.34, y, WALL + 0.012)],
                                            0.03, 3), 0.0125, M.chrome, verts=12)
        for s in (-1, 1):
            F.extrude(f'plate{s}', G.rounded_rect(0.05, 0.36, 0.025, corner=3), 0.012, (s * 0.34, 1.075, WALL + 0.006),
                      M.chrome, bevel=0.004, bevel_segments=1)
            for y in (1.2, 0.95):
                G.revolve(f'rose{s}{y}', [(0, 0), (0.02, 0), (0.019, 0.006), (0.014, 0.012), (0, 0.013)],
                          at=F.at(s * 0.34, y, WALL + 0.011), material=M.chrome, segments=14, rot=(90, 0, 0))
        _drape(F, 'bath_towel', -0.07, 0.42, 1.2, z, 0.5, 0.38, M.towel, r=0.0125, hem=M.towel_hem)
        _drape(F, 'hand_towel', 0.17, 0.26, 0.95, z, 0.3, 0.22, M.towel2, r=0.0125, hem=M.towel_hem2)
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
            # a squared rail, its edges and its top rounded off
            a, b = (s * 0.24, 0.012, LADDER_FOOT - 0.02), (s * 0.24, top - 0.012, _on_ladder(top - 0.012) - 0.02)
            F.sweep(f'rail{s}', [a, b], None, M.birch, profile=G.rounded_rect(0.04, 0.04, 0.01, corner=2),
                    up=(1, 0, 0), angle=50)
            G.sphere(f'rail_cap{s}', 0.02, at=F.at(s * 0.24, top - 0.012, _on_ladder(top - 0.012) - 0.02), material=M.birch,
                     segments=10, rings=6)
        for k, y in enumerate((0.3, 0.63, 0.96, 1.29, 1.62)):
            zz = _on_ladder(y) - 0.02
            F.sweep(f'rung{k}', [(-0.24, y, zz), (0.24, y, zz)], 0.0125, M.birch, verts=10)
        for k, (y, m, hem) in enumerate(((0.63, M.towel3, None), (0.96, M.towel, M.towel_hem), (1.29, M.towel2, M.towel_hem2))):
            _drape(F, f'towel{k}', 0.01 * (k - 1), 0.4, y, _on_ladder(y) - 0.02, 0.3, 0.26, m, r=0.0125, hem=hem)
    G.join_all('towelLadder')
    G.settle(centre=False)
    return G.meshes()


# --- the laundry basket: a wicker tub of clean washing -----------------------------------------------------
def laundryBasket():
    F = G.GameFrame()
    z = -0.2
    bx, by, _ = F.at(0, 0, z)
    with G.new_parts() as parts:
        # the tub: woven rows (each a ring of over-and-under ripples, the next row's turned half a step), lofted up the
        # outside to the rim and down the inside to the floor
        n, rows = 40, 10
        secs = []
        for i in range(2 * rows + 1):
            t = i / (2 * rows)
            r = 0.23 + 0.025 * t
            row = i // 2
            ring = []
            for k in range(n):
                a = math.tau * k / n
                # a row's middle bulges where its strand passes over a stake, sinks where it passes under; the next row
                # the other way round, so the weave checks; the rows' seams sink a little
                rr = r * (1 + 0.032 * math.cos(10 * a + math.pi * row) if i % 2 else 1 - 0.01 * (0 < i < 2 * rows))
                ring.append((bx + rr * math.cos(a), by + 0.78 * rr * math.sin(a), 0.33 * t))
            secs.append(ring)
        for y, r in ((0.33, 0.236), (0.03, 0.222)):
            secs.append([(bx + r * math.cos(math.tau * k / n), by + 0.78 * r * math.sin(math.tau * k / n), y) for k in range(n)])
        # (the lining inside, plain: only its top shows)
        G.loft('basket', secs, material=M.wicker, angle=25)
        rim = [(0.25 * math.cos(math.tau * k / 40), 0.335, z - 0.78 * 0.248 * math.sin(math.tau * k / 40)) for k in range(40)]
        F.sweep('rim', rim, None, M.wicker_rim, profile=G.rounded_rect(0.034, 0.034, 0.012, corner=2), closed=True,
                up=(0, 0, 1), angle=60)
        # a band of plaited rim at the foot
        foot = [(0.232 * math.cos(math.tau * k / 32), 0.012, z - 0.78 * 0.232 * math.sin(math.tau * k / 32)) for k in range(32)]
        F.sweep('foot_band', foot, 0.012, M.wicker_rim, verts=5, closed=True)
        for s in (-1, 1):
            # a loop handle on each end
            pts = [(s * 0.256, 0.24, z - 0.06), (s * 0.263, 0.29, z - 0.035), (s * 0.264, 0.305, z),
                   (s * 0.263, 0.29, z + 0.035), (s * 0.256, 0.24, z + 0.06)]
            F.sweep(f'handle{s}', pts, 0.008, M.wicker_rim, verts=6, resolution=2)
        # clean washing heaped in it: folded towels and a crumpled shirt
        _folded(F, 'wash0', -0.06, 0.3, z - 0.02, 0.3, 0.24, 0.06, M.towel, turn=8)
        _folded(F, 'wash1', 0.07, 0.33, z + 0.03, 0.26, 0.2, 0.05, M.towel2, turn=-14)
        _lump(F, 'wash2', 0.0, 0.345, z - 0.07, 0.12, M.wash_b, scale=(1.1, 0.8, 0.35), seed=3)
    G.join_all('laundryBasket')
    G.settle(centre=False)
    return G.meshes()


# --- the bath mat: a soft rounded mat with a raised pile -----------------------------------------------------
def bathMat():
    with G.new_parts() as parts:
        G.soft_box('mat', (0.76, 0.5, 0.016), at=(0, 0, 0.008), material=M.mat, round=0.008, round_segments=2)
        G.soft_box('pile', (0.66, 0.4, 0.012), at=(0, 0, 0.0165), material=M.mat_pile, round=0.006, puff=0.003,
                   round_segments=2, flat=3)
        for s in (-1, 1):
            G.cube(f'stripe{s}', (0.6, 0.025, 0.004), at=(0, s * 0.15, 0.0225), material=M.mat, bevel=0.0015,
                   bevel_segments=1)
    # a stitched seam round the mat's edge
    seam = [(u, v, 0.0162) for u, v in G.rounded_rect(0.72, 0.46, 0.02, corner=2)]
    G.sweep('seam', seam, 0.0016, material=M.mat_pile, verts=3, closed=True)
    G.join_all('bathMat')
    G.settle()
    return G.meshes()


# The washer's door opens for the washing to go in and shuts (`hold`; `door_open` takes 1.0 s each way), and
# only then its drum goes round, a turn each 1.2 s (`every` a hair under it, so no pause between turns) till the
# wash ends. Folding at the dryer opens its
# door to take the washing out: the drum stays still.
WASH = {'uses': {'laundry': ['door_open', {'clip': 'drum_spin', 'delay': 3.5, 'every': 1.1}]}, 'hold': 2.5}
FOLD = {'uses': {'laundry': 'door_open'}, 'hold': 2.5}

FAMILY = G.Family(
    'laundry', M,
    [G.Model('laundryWasher', laundryWasher, size=(0.6, 0.98, 0.65), key='model.laundryWasher', centre=(0, -0.105),
             motions=WASH, paint=[('body', 'Body'), ('door', 'Door'), ('panel', 'Panel')]),
     G.Model('laundryDryer', laundryDryer, size=(0.6, 1.08, 0.65), key='model.laundryDryer', centre=(0, -0.105),
             motions=FOLD, paint=[('body', 'Body'), ('door_dryer', 'Door'), ('panel', 'Panel')]),
     G.Model('towelRail', towelRail, size=(0.73, 0.595, 0.098), key='model.towelRail', centre=(0, -0.381), floor=False,
             paint=[('chrome', 'Rail'), ('towel', 'Towels'), ('towel2', 'Hand towel')]),
     G.Model('towelLadder', towelLadder, size=(0.52, 1.707, 0.359), key='model.towelLadder', centre=(0, -0.25),
             paint=[('birch', 'Ladder'), ('towel2', 'Top towel'), ('towel', 'Middle towel'), ('towel3', 'Bottom towel')]),
     G.Model('laundryBasket', laundryBasket, size=(0.54, 0.39, 0.421), key='model.laundryBasket', centre=(0, -0.2),
             paint=[('wicker', 'Basket'), ('wicker_rim', 'Rim & weave')]),
     G.Model('bathMat', bathMat, size=(0.76, 0.025, 0.5), key='model.bathMat',
             paint=[('mat', 'Mat'), ('mat_pile', 'Pile')])],
    comment='Blender-built laundry models (tools/art/blender/families/laundry.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. The washer and the dryer open their door (`door_open`) while a resident loads '
            'or empties them, and the washer turns its drum (`drum_spin`) once the door has shut.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
