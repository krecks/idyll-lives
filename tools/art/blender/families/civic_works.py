# The big working things of the town's named places (content/civicWorks.json, the 'community' buy category): the fire
# station's fire engine, fire pole and hose rack, the garage's two-post car lift, tyre rack and tool chest, the
# newspaper's printing press, the server rack, and the builders' yard's timber stack, cement mixer and the town's
# flagpole (`model.<id>`). Notes: civic_works.md.
#
# Built in game axes (`F`: x, height, depth; the front +depth) where the game places them, then stood on the floor
# (`G.settle`). The things against a wall (the hose rack on it; the server rack, the tyre rack and the tool chest
# on the floor) have their backs on the wall's face (-0.43) and a `centre=` on their Model. Each model has its own
# palette, its main part `body` (a red engine, a black cabinet, an orange mixer), over the shared colours
# (`_palette`, as street_furniture.py does). The fire engine, the car lift and the outdoor things have far versions
# for the town overview (built by hand at low detail, `G.far_of`). Three clips:
# - the printing press's two cylinders turn against each other while a resident watches it run (`rollers`, a loop);
# - the server rack's LEDs blink near the camera all the time (`blink`, ambient);
# - the flag sways at the top of its pole near the camera (`flutter`, ambient).
#
# Run:  cargo art civic_works [--only fireEngine] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

F = G.GameFrame()
low, is_low = G.low, G.is_low

# Colours every model may use; each model adds its own 'body' (the part the player colours first) and what is its
# alone (`_palette`).
SHARED = dict(
    steel=('#A9AEB2', 'metal'),       # brushed steel and aluminium: shutters, rails, ladders, beams
    chrome=('#C8CCD0', 'chrome'),     # handles, bezels, trims
    iron=('#2B302E', 'metal'),        # black steel: chassis, brackets, axles
    dark=('#1E2120', 'matte'),        # openings, gaps, grilles, the inside of things
    rubber=('#26282B', 'matte'),      # tyres, mats, feet, seals
    plastic=('#3A3D41', 'satin'),     # black plastic: housings, buttons, castors
    tinted=('#2A3439', 'gloss'),      # dark glass (gloss, not see-through: no draw call of its own)
    lamp=('#F4F1E0', 'gloss'),        # white lenses
    amber=('#E8902E', 'gloss'),       # indicators
    red_lens=('#B8221E', 'gloss'),    # tail lamps
    blue_lens=('#2D62C8', 'gloss'),   # the emergency lights
    white=('#F1F0EC', 'satin'),
    yellow=('#F2C14E', 'gloss'),      # stripes and chevrons
    brass=('#C9A04A', 'metal', {'roughness': 0.2}),   # polished brass: couplings, the fire pole
    paper=('#F2F0EA', 'matte'),
    ink=('#2A2C2E', 'matte'),
)


class _Palettes:
    """The family's palette: the one of the model being built (`Family.run` asks it for the pack's `materials`
    right after each build)."""
    current = None

    def finishes(self, names):
        return self.current.finishes(names)


PALETTES = _Palettes()


def _palette(**own):
    """The palette of the model about to be built: the shared colours and its own (its `body`)."""
    PALETTES.current = G.Palette(**{**SHARED, **own})
    return PALETTES.current


WALL = -0.43   # the wall's face, game depth (one tile deep)


# --- helpers (game coordinates) ------------------------------------------------------------------------------
def _arc(zc, yc, r, y_cut, steps):
    """The points (depth, height) of a wheel arch: the circle of radius `r` round (zc, yc) above the line at height
    `y_cut`, from its rear end over the top to its front end."""
    t0 = math.asin(max(-1.0, min(1.0, (y_cut - yc) / r)))
    return [(zc + r * math.cos(t), yc + r * math.sin(t))
            for t in ((math.pi - t0) - (math.pi - 2 * t0) * i / steps for i in range(steps + 1))]


def _slats(name, x, s, z0, z1, y0, y1, material, pitch=0.06, t=0.008):
    """A roller shutter on a side face at game x `x` (`s` +1 the right side, -1 the left), from depth z0 to z1 and
    height y0 to y1: its slats rounded ridges running along the depth, `t` proud of the face at most. At low detail
    (a far version) a plain panel."""
    if is_low():
        return F.box(name, t, y1 - y0, abs(z1 - z0), (x + s * t / 2, (y0 + y1) / 2, (z0 + z1) / 2), material)
    n = max(1, round((y1 - y0) / pitch))
    p = (y1 - y0) / n
    out = [(x, y0)]
    for i in range(n):   # each slat: a step out at its foot, then sloping back in to the next
        ya = y0 + i * p
        out += [(x + s * t * 0.4, ya), (x + s * t, ya + p * 0.12)]
    out += [(x + s * t * 0.4, y1), (x, y1)]
    return F.extrude(name, out, abs(z1 - z0), (0, 0, (z0 + z1) / 2), material, plane='front')


def _slats_back(name, z, x0, x1, y0, y1, material, pitch=0.06, t=0.008):
    """A roller shutter on a face looking to the back at game depth `z`, from x0 to x1 and height y0 to y1."""
    if is_low():
        return F.box(name, abs(x1 - x0), y1 - y0, t, ((x0 + x1) / 2, (y0 + y1) / 2, z - t / 2), material)
    n = max(1, round((y1 - y0) / pitch))
    p = (y1 - y0) / n
    out = [(z, y0)]
    for i in range(n):
        ya = y0 + i * p
        out += [(z - t * 0.4, ya), (z - t, ya + p * 0.12)]
    out += [(z - t * 0.4, y1), (z, y1)]
    return F.extrude(name, out, abs(x1 - x0), ((x0 + x1) / 2, 0, 0), material, plane='side')


def _tyre(name, x, y, z, r, w, material, grooves=2, segments=None, axis='x'):
    """A tyre round an axle along game x (or 'z'; or 'y', lying flat) through (x, y, z): rounded shoulders, its
    tread's grooves round its crown, its bead's hole open (r * 0.64 round), where a rim goes."""
    hw = w / 2
    sh = min(0.06, r * 0.12)
    if grooves is None:   # plain: one rounded shoulder a side
        prof = [(r * 0.64, -hw + 0.01), (r - sh, -hw), (r, -hw + w * 0.16), (r, hw - w * 0.16), (r - sh, hw),
                (r * 0.64, hw - 0.01)]
        prof.append(prof[0])
        rot = {'x': (0, 90, 0), 'z': (90, 0, 0), 'y': (0, 0, 0)}[axis]
        return G.revolve(name, prof, at=F.at(x, y, z), material=material, rot=rot,
                         segments=segments or low(G.sides(r, 24, 44), 16))
    prof = [(r * 0.64, -hw + 0.01), (r - sh, -hw), (r - sh * 0.3, -hw + w * 0.07), (r, -hw + w * 0.18)]
    if grooves and not is_low():
        for k in range(grooves):
            g = -hw * 0.6 + (w * 0.6) * (k + 0.5) / grooves
            prof += [(r, g - 0.008), (r - 0.01, g - 0.005), (r - 0.01, g + 0.005), (r, g + 0.008)]
    prof += [(r, hw - w * 0.18), (r - sh * 0.3, hw - w * 0.07), (r - sh, hw), (r * 0.64, hw - 0.01)]
    prof.append(prof[0])
    rot = {'x': (0, 90, 0), 'z': (90, 0, 0), 'y': (0, 0, 0)}[axis]
    return G.revolve(name, prof, at=F.at(x, y, z), material=material, rot=rot,
                     segments=segments or low(G.sides(r, 24, 44), 16))


# --- the fire engine: 3x6, a UK fire appliance, its cab to the front -------------------------------------------
# 2.5 m wide over its mirrors, 3.2 m high to its ladder, 5.84 m long: a crew cab over the front axle, a body of
# roller-shuttered lockers, a triple ladder and two hard suction hoses on the roof, blue lights on the cab.
FE_FRONT, FE_BACK = 2.86, -2.88       # the cab's face, the body's back (the bumpers stand a little past them)
FE_CAB = 0.92                         # the cab's back
FE_BODY = 0.86                        # the body's front
FE_FA, FE_RA, FE_R = 2.0, -1.6, 0.5   # the axles' depths, the wheels' radius
FE_SILL = 0.72                        # the cab's and the body's underside
FE_HALF = 1.15                        # the body's half width (the cab's 1.175)
FE_ROOF = 2.85                        # the body's roof (the cab's 2.9)


def _fe_wheel(name, x, z, M, inner=False):
    """A wheel on the axle at depth z: the tyre, a silver rim dished in to the hub, ten nuts. `inner`: the inner
    one of a rear pair, plainer (only its tread shows)."""
    s = 1 if x > 0 else -1
    w = 0.3
    _tyre(f'{name}_tyre', x, FE_R, z, FE_R, w, M.rubber, grooves=None, segments=low(20, 12) if inner else low(40, 16))
    if inner:
        return
    G.revolve(f'{name}_rim', [(0, -w / 2 + 0.04), (0.33, -w / 2 + 0.03), (0.335, w / 2 - 0.012), (0.3, w / 2 - 0.02),
                              (0.13, w / 2 - 0.06), (0.08, w / 2 - 0.03), (0, w / 2 - 0.03)],
              at=F.at(x, FE_R, z), material=M.steel, rot=(0, -90 * s, 0), segments=low(24, 12))
    if is_low():
        return
    for k in range(6):
        a = math.tau * k / 6
        F.cyl(f'{name}_nut{k}', 0.014, 0.03, (x + s * (w / 2 - 0.02), FE_R + 0.1 * math.sin(a), z + 0.1 * math.cos(a)),
              M.chrome, rot=(0, 90, 0), verts=6)
    F.cyl(f'{name}_cap', 0.055, 0.04, (x + s * (w / 2 - 0.02), FE_R, z), M.chrome, rot=(0, 90, 0), verts=16)


def fireEngine():
    M = _palette(body=('#B8221E', 'gloss'),                 # the red paint
                 shutter=('#C3C7CA', 'metal'),              # the lockers' roller shutters
                 stripe=('#F2D33A', 'gloss'),               # the reflective band
                 ladder=('#B9BDC1', 'metal'))               # the ladders
    red = M.body
    hx = FE_HALF + 0.025                                    # the cab's half width
    # the cab: one slab drawn from the side, the front wheel's arch cut out of it, its windscreen raked back
    cab = [(FE_CAB, FE_SILL)] + _arc(FE_FA, FE_R, 0.57, FE_SILL, low(14, 6)) + [
        (FE_FRONT, FE_SILL), (FE_FRONT, 1.62), (FE_FRONT - 0.03, 1.7), (FE_FRONT - 0.15, 2.7), (FE_FRONT - 0.21, 2.84),
        (FE_FRONT - 0.33, 2.9), (FE_CAB + 0.06, 2.9), (FE_CAB, 2.85)]
    F.extrude('cab', cab, 2 * hx, (0, 0, 0), red, plane='side', bevel=low(0.035, 0), bevel_segments=2)
    # the body: the lockers' box, the rear wheel's arch cut out of it
    body = [(FE_BACK, FE_SILL)] + _arc(FE_RA, FE_R, 0.58, FE_SILL, low(14, 6)) + [
        (FE_BODY, FE_SILL), (FE_BODY, FE_ROOF), (FE_BACK, FE_ROOF)]
    F.extrude('body', body, 2 * FE_HALF, (0, 0, 0), red, plane='side', bevel=low(0.02, 0), bevel_segments=2)
    F.box('cab_gap', 2 * FE_HALF - 0.04, FE_ROOF - FE_SILL - 0.1, FE_CAB - FE_BODY + 0.02,
          (0, (FE_ROOF + FE_SILL) / 2, (FE_CAB + FE_BODY) / 2), M.dark)
    # the chassis under it all, the bumpers front and back
    for s in (-1, 1):
        F.box(f'rail{s}', 0.12, 0.26, 5.5, (s * 0.45, 0.6, 0.1), M.iron)
    F.box('bumper', 2.36, 0.36, 0.14, (0, 0.55, FE_FRONT - 0.01), M.plastic, bevel=low(0.03, 0), bevel_segments=2)
    F.box('rear_bumper', 2.2, 0.26, 0.1, (0, 0.57, FE_BACK + 0.01), M.yellow, bevel=low(0.015, 0), bevel_segments=1)
    # the wheels: singles at the front, pairs at the back
    for s in (-1, 1):
        _fe_wheel(f'front{s}', s * 1.0, FE_FA, M)
        _fe_wheel(f'rear{s}', s * 1.0, FE_RA, M)
        _fe_wheel(f'rear_in{s}', s * 0.68, FE_RA, M, inner=True)
        F.box(f'axle_f{s}', 0.6, 0.1, 0.1, (s * 0.55, FE_R, FE_FA), M.iron)
        F.box(f'axle_r{s}', 0.5, 0.14, 0.14, (s * 0.5, FE_R, FE_RA), M.iron)
    # the windscreen, the side windows and the doors' seams
    t = math.atan2(0.12, 1.0)
    F.box('windscreen', 2.12, 0.98, 0.012, (0, 2.19, FE_FRONT - 0.09 + 0.008), M.tinted, rot=(-math.degrees(t), 0, 0))
    for s in (-1, 1):
        xs = s * (hx + 0.004)
        for k, (z0, z1) in enumerate(((1.86, 2.62), (0.98, 1.78))):
            F.extrude(f'window{s}{k}', G.rounded_rect(z1 - z0, 0.72, 0.05, corner=low(3, 1)), 0.01,
                      (xs, 2.3, (z0 + z1) / 2), M.tinted, plane='side')
        if not is_low():
            for z in (0.95, 1.82, 2.66):
                F.box(f'seam{s}{z}', 0.004, 1.9, 0.01, (xs, 1.76, z), M.dark)
            for k, z in enumerate((1.9, 1.02)):
                F.box(f'door_handle{s}{k}', 0.02, 0.03, 0.16, (s * (hx + 0.012), 1.72, z + 0.1), M.chrome)
            for k, z in enumerate((1.76, 0.97)):   # grab rails beside the doors
                G.kit.tube(F, f'grab{s}{k}', [(s * (hx + 0.005), 1.25, z), (s * (hx + 0.04), 1.3, z),
                                              (s * (hx + 0.04), 2.0, z), (s * (hx + 0.005), 2.05, z)], 0.014, M.chrome, round=1)
        # the steps under the crew door, the mirror on its arms
        F.box(f'step{s}', 0.24, 0.05, 0.62, (s * (hx - 0.1), 0.48, 1.18), M.plastic)
        F.box(f'step2{s}', 0.2, 0.04, 0.62, (s * (hx - 0.12), 0.2 + 0.06, 1.18), M.plastic)
        G.kit.tube(F, f'mirror_arm{s}', [(s * hx, 2.2, FE_FRONT - 0.32), (s * 1.24, 2.3, FE_FRONT - 0.32),
                                         (s * 1.24, 2.0, FE_FRONT - 0.32)], 0.015, M.chrome, round=1)
        F.box(f'mirror{s}', 0.05, 0.36, 0.2, (s * 1.225, 2.1, FE_FRONT - 0.3), M.plastic, bevel=low(0.02, 0),
              bevel_segments=1)
        F.box(f'mirror_glass{s}', 0.004, 0.32, 0.16, (s * 1.225, 2.1, FE_FRONT - 0.402), M.tinted)
        # the reflective band along the cab and the body, under the windows and over the lockers
        F.box(f'band_cab{s}', 0.004, 0.08, FE_FRONT - FE_CAB - 0.06, (s * (hx + 0.003), 1.42, (FE_FRONT + FE_CAB) / 2 - 0.02),
              M.stripe)
        F.box(f'band_top{s}', 0.004, 0.06, FE_BODY - FE_BACK - 0.06, (s * (FE_HALF + 0.003), 2.79, (FE_BODY + FE_BACK) / 2),
              M.stripe)
        # the arches' black trims
        for zc, r in ((FE_FA, 0.57), (FE_RA, 0.58)):
            pts = [(s * (hx if zc > 0 else FE_HALF), y, z) for z, y in _arc(zc, FE_R, r + 0.02, FE_SILL + 0.02, low(14, 6))]
            F.sweep(f'arch{s}{zc}', pts, None, M.plastic, profile=G.rounded_rect(0.06, 0.05, 0.0, corner=1),
                    up=(0, 0, 1))
        # the side guard between the wheels: two rails on brackets
        for y in (0.4, 0.62):
            F.box(f'guard{s}{y}', 0.03, 0.06, 2.3, (s * (FE_HALF - 0.04), y, 0.25 - 0.08), M.steel, bevel=low(0.008, 0),
                  bevel_segments=1)
        for z in (-0.82, 0.25, 1.32):
            F.box(f'guard_post{s}{z}', 0.03, 0.34, 0.05, (s * (FE_HALF - 0.06), 0.55, z), M.iron)
    # the lockers: roller shutters along each side and at the back, each with its drop bar
    lockers = ((0.8, 0.04, 0.84), (-0.02, -0.96, 0.84), (-1.04, -2.16, 1.2), (-2.24, -2.82, 0.84))
    for s in (-1, 1):
        for k, (z0, z1, y0) in enumerate(lockers):
            _slats(f'shutter{s}{k}', s * FE_HALF, s, z1, z0, y0, 2.72, M.shutter, pitch=0.07)
            F.box(f'drop_bar{s}{k}', 0.035, 0.04, z0 - z1 - 0.04, (s * (FE_HALF + 0.02), y0 + 0.03, (z0 + z1) / 2),
                  M.chrome)
    _slats_back('shutter_back', FE_BACK, -0.8, 0.8, 0.84, 2.72, M.shutter, pitch=0.07)
    F.box('drop_bar_back', 1.56, 0.04, 0.035, (0, 0.87, FE_BACK - 0.02), M.chrome)
    # the rear: tail lamps on the corners, the chevrons on the bumper
    for s in (-1, 1):
        for k, (y, m) in enumerate(((1.25, M.amber), (1.1, M.red_lens), (0.95, M.red_lens))):
            F.box(f'tail{s}{k}', 0.18, 0.13, 0.03, (s * 0.98, y, FE_BACK - 0.01), m)
        F.box(f'reverse{s}', 0.12, 0.08, 0.03, (s * 0.98, 1.4, FE_BACK - 0.01), M.lamp)
    if not is_low():
        for k in range(-4, 5):
            x0 = k * 0.24
            s = 1 if k >= 0 else -1
            F.extrude(f'chevron{k}', [(x0 - 0.04, 0.44), (x0 + 0.04, 0.44), (x0 + 0.04 + s * 0.12, 0.7), (x0 - 0.04 + s * 0.12, 0.7)],
                      0.004, (0, 0, FE_BACK - 0.041), red, plane='front')
    # the front: grille, headlamps, indicators, blue lights in the grille, a white sign over the windscreen
    F.box('grille', 1.5, 0.5, 0.02, (0, 1.18, FE_FRONT + 0.005), M.dark)
    if not is_low():
        for k in range(5):
            F.box(f'grille_bar{k}', 1.46, 0.025, 0.02, (0, 0.98 + k * 0.1, FE_FRONT + 0.016), M.chrome)
    for s in (-1, 1):
        F.extrude(f'head_bezel{s}', G.rounded_rect(0.36, 0.2, 0.04, corner=low(3, 1)), 0.03, (s * 0.88, 0.93, FE_FRONT + 0.01),
                  M.chrome)
        F.extrude(f'head_lens{s}', G.rounded_rect(0.32, 0.16, 0.03, corner=low(3, 1)), 0.03, (s * 0.88, 0.93, FE_FRONT + 0.018),
                  M.lamp)
        F.box(f'indicator{s}', 0.1, 0.16, 0.03, (s * 1.1, 0.93, FE_FRONT + 0.01), M.amber, bevel=low(0.01, 0), bevel_segments=1)
        F.box(f'grille_blue{s}', 0.16, 0.07, 0.03, (s * 0.5, 0.86, FE_FRONT + 0.012), M.blue_lens, bevel=low(0.01, 0),
              bevel_segments=1)
        F.cyl(f'fog{s}', 0.05, 0.03, (s * 0.85, 0.55, FE_FRONT + 0.045), M.lamp, rot=(90, 0, 0), verts=low(16, 8))
    F.box('sign', 1.5, 0.14, 0.03, (0, 2.79, FE_FRONT - 0.2), M.white, rot=(-math.degrees(t), 0, 0), bevel=low(0.01, 0),
          bevel_segments=1)
    F.box('sign_line', 1.2, 0.035, 0.03, (0, 2.79, FE_FRONT - 0.196), red, rot=(-math.degrees(t), 0, 0))
    # the light bar on the cab's roof: blue lenses either side of a white middle, on a black base
    F.box('lightbar_base', 2.0, 0.05, 0.32, (0, 2.925, FE_FRONT - 0.48), M.plastic, bevel=low(0.015, 0), bevel_segments=1)
    for k in range(6):
        x = -0.83 + k * 0.332
        F.box(f'light{k}', 0.3, 0.1, 0.28, (x, 3.0, FE_FRONT - 0.48), M.lamp if k in (2, 3) else M.blue_lens,
              bevel=low(0.03, 0), bevel_segments=1)
    # the roof: the ladders on two gantries, the hard suction hoses, a walkway, beacons at the back
    F.box('walkway', 2 * FE_HALF - 0.1, 0.01, FE_BODY - FE_BACK - 0.1, (0, FE_ROOF + 0.005, (FE_BODY + FE_BACK) / 2), M.steel)
    for z in (0.55, -2.55):
        F.box(f'gantry{z}', 1.3, 0.04, 0.08, (-0.2, 2.97, z), M.iron)
        for s in (-1, 1):
            F.box(f'gantry_post{z}{s}', 0.05, 0.12, 0.06, (-0.2 + s * 0.6, 2.9, z), M.iron)
    F.box('ladder_rest', 0.7, 0.07, 0.08, (-0.2, 2.94, 2.25), M.iron)
    _ladder('ladder', -0.2, 3.0, 2.45, -2.86, 0.56, M.ladder, M)
    _ladder('ladder2', -0.2, 3.09, 2.25, -2.6, 0.46, M.ladder, M, rungs=False)
    _ladder('roof_ladder', 0.78, 2.9, 0.4, -2.4, 0.34, M.ladder, M)
    for k, x in enumerate((-0.82, -0.98)):
        r = 0.065
        F.sweep(f'suction{k}', [(x, FE_ROOF + r + 0.01, 0.45), (x, FE_ROOF + r + 0.01, -2.7)], r, M.rubber, verts=low(10, 6))
        if not is_low():
            for j in range(6):
                F.cyl(f'suction_rib{k}{j}', r + 0.006, 0.03, (x, FE_ROOF + r + 0.01, 0.1 - j * 0.5), M.rubber,
                      rot=(90, 0, 0), verts=8)
            for z in (0.47, -2.72):
                F.cyl(f'suction_end{k}{z}', r + 0.012, 0.06, (x, FE_ROOF + r + 0.01, z), M.brass, rot=(90, 0, 0), verts=10)
    for s in (-1, 1):
        F.cyl(f'beacon{s}', 0.07, 0.12, (s * 1.0, FE_ROOF + 0.06, FE_BACK + 0.12), M.blue_lens, verts=low(14, 8))
    G.join_all('fireEngine_far' if is_low() else 'fireEngine')
    G.settle()
    return G.meshes()


def _ladder(name, x, y, z0, z1, w, material, M, rungs=True):
    """A ladder lying on the roof along the depth from z0 to z1, its stiles' bottom at height `y`, `w` wide."""
    for s in (-1, 1):
        F.box(f'{name}_stile{s}', 0.04, 0.08, z0 - z1, (x + s * w / 2, y + 0.04, (z0 + z1) / 2), material,
              bevel=low(0.006, 0), bevel_segments=1)
    if rungs:
        n = int((z0 - z1) / low(0.28, 0.56))
        for k in range(n + 1):
            F.cyl(f'{name}_rung{k}', 0.015, w - 0.04, (x, y + 0.04, z1 + 0.1 + k * (z0 - z1 - 0.2) / n), material,
                  rot=(0, 90, 0), verts=low(6, 4))


# --- the car lift: 3x5, a two-post lift with a hatchback up on its arms, its wheels off -------------------------
# The posts stand either side of the middle (game x +-1.32), the car along the depth, its nose to the front, its
# sills 1.27 m up on the arms' four pads; its wheels lie and lean on the floor round it.
CL_POST = 1.32       # the posts' middles, game x
CL_POST_H = 2.75     # their tops
CL_ARMS = 1.15       # the arms' height (their pads' tops 0.12 higher, where the sills rest)
CAR_Y = 1.1          # the car's own ground: its sills 0.17 over it
CAR_HALF = 0.86      # half its body's width
CAR_FA, CAR_RA, CAR_WR = 1.27, -1.25, 0.31   # its axles' depths and its wheels' radius
CAR_BELT, CAR_ROOF = 0.94, 1.47              # its waist line and roof, over its own ground


def _tapered(objs, y0, y1, k):
    """Draws `objs` in towards the middle the higher they are, from none at game height y0 to `k` of their x at y1: a
    car's glasshouse leaning in to its roof (tumblehome)."""
    for o in objs:
        mw = o.matrix_world.copy()
        inv = mw.inverted()
        for v in o.data.vertices:
            p = mw @ v.co
            f = max(0.0, min(1.0, (p.z - y0) / (y1 - y0)))
            p.x *= 1 - k * f
            v.co = inv @ p
        o.data.update()


def _car(M):
    """The hatchback, lifted: a body drawn from the side, its arches empty, a glasshouse of tinted glass leaning in
    under a roof in the car's colour, pillars, lamps, plates, mirrors, the brakes and springs in its empty arches and
    an exhaust under it."""
    Y = CAR_Y
    paint = M.car
    lower = [(-1.93, Y + 0.17)] + [(z, y + Y) for z, y in _arc(CAR_RA, CAR_WR, 0.36, 0.17, low(12, 6))] + \
        [(z, y + Y) for z, y in _arc(CAR_FA, CAR_WR, 0.36, 0.17, low(12, 6))] + [
        (1.9, Y + 0.17), (1.96, Y + 0.28), (1.98, Y + 0.52), (1.93, Y + 0.65), (1.62, Y + 0.76), (1.0, Y + 0.93),
        (-1.86, Y + CAR_BELT + 0.01), (-1.95, Y + 0.9), (-1.97, Y + 0.6), (-1.95, Y + 0.3)]
    F.extrude('car_body', lower, 2 * CAR_HALF, (0, 0, 0), paint, plane='side', bevel=low(0.05, 0), bevel_segments=low(3, 1))
    with G.new_parts() as house:
        top = [(0.96, Y + 0.92), (0.3, Y + 1.42), (-1.45, Y + 1.44), (-1.8, Y + 1.32), (-1.9, Y + 0.92)]
        F.extrude('car_glass', top, 1.6, (0, 0, 0), M.tinted, plane='side', bevel=low(0.02, 0), bevel_segments=1)
        F.extrude('car_roof', [(0.36, Y + 1.4), (0.3, Y + 1.47), (-1.45, Y + 1.475), (-1.56, Y + 1.44), (-1.5, Y + 1.4)],
                  1.64, (0, 0, 0), paint, plane='side', bevel=low(0.025, 0), bevel_segments=low(2, 1))
        F.box('car_spoiler', 1.3, 0.03, 0.18, (0, Y + 1.47, -1.53), paint, bevel=low(0.012, 0), bevel_segments=1)
        for s in (-1, 1):
            x = s * 0.805
            F.sweep(f'a_pillar{s}', [(x, Y + 0.92, 0.97), (x, Y + 1.43, 0.3)], None, paint,
                    profile=G.rounded_rect(0.07, 0.03, 0.0, corner=1), up=(0, 0, 1))
            F.box(f'b_pillar{s}', 0.02, 0.5, 0.09, (x, Y + 1.17, -0.18), paint)
            F.extrude(f'c_pillar{s}', [(-1.25, Y + 0.93), (-1.42, Y + 1.43), (-1.82, Y + 1.31), (-1.9, Y + 0.93)], 0.014,
                      (x, 0, 0), paint, plane='side')
            F.box(f'sill_trim{s}', 0.014, 0.025, 1.0, (x, Y + 0.935, 0.4), M.chrome)
    _tapered(house, Y + 0.92, Y + 1.48, 0.17)
    for s in (-1, 1):
        xs = s * (CAR_HALF + 0.004)
        if not is_low():
            for z in (0.98, -0.17, -1.22):   # the doors' seams
                F.box(f'car_seam{s}{z}', 0.004, 0.62, 0.008, (xs, Y + 0.6, z), M.dark)
            for z in (0.0, -1.1):
                F.box(f'car_handle{s}{z}', 0.012, 0.025, 0.13, (xs + s * 0.004, Y + 0.84, z), M.chrome)
        F.box(f'car_mirror{s}', 0.11, 0.09, 0.12, (s * 0.88, Y + 1.0, 0.85), paint, bevel=low(0.02, 0), bevel_segments=1)
        # the empty arches: black liners, and the brakes, hubs and springs left on the axles
        for zc in (CAR_FA, CAR_RA):
            pts = [(s * 0.74, y + Y, z) for z, y in _arc(zc, CAR_WR, 0.345, 0.2, low(10, 5))]
            F.sweep(f'liner{s}{zc}', pts, None, M.plastic, profile=[(-0.004, -0.12), (0.004, -0.12), (0.004, 0.12), (-0.004, 0.12)],
                    up=(0, 0, 1))
            F.cyl(f'disc{s}{zc}', 0.14, 0.022, (s * 0.76, Y + CAR_WR, zc), M.steel, rot=(0, 90, 0), verts=low(20, 8))
            F.cyl(f'hub{s}{zc}', 0.065, 0.06, (s * 0.77, Y + CAR_WR, zc), M.iron, rot=(0, 90, 0), verts=low(12, 6))
            F.box(f'caliper{s}{zc}', 0.06, 0.1, 0.09, (s * 0.765, Y + CAR_WR + 0.06, zc - 0.11), M.red_lens, bevel=low(0.015, 0),
                  bevel_segments=1)
            if not is_low():
                for k in range(4):
                    a = math.tau * k / 4 + 0.4
                    F.cyl(f'stud{s}{zc}{k}', 0.009, 0.04, (s * 0.8, Y + CAR_WR + 0.045 * math.sin(a), zc + 0.045 * math.cos(a)),
                          M.chrome, rot=(0, 90, 0), verts=6)
                F.cyl(f'strut{s}{zc}', 0.03, 0.4, (s * 0.64, Y + CAR_WR + 0.24, zc), M.iron, verts=8)
                F.cyl(f'spring{s}{zc}', 0.055, 0.22, (s * 0.64, Y + CAR_WR + 0.26, zc), M.iron, verts=10)
            F.box(f'arm_lower{s}{zc}', 0.32, 0.04, 0.06, (s * 0.58, Y + CAR_WR - 0.02, zc), M.iron)
    # its front and back: lamps, grille, plates (white at the front, yellow at the back), bumpers' lower lips
    for s in (-1, 1):
        F.extrude(f'headlamp{s}', [(-0.2, -0.03), (0.15, -0.05), (0.2, 0.04), (-0.18, 0.05)], 0.03,
                  (s * 0.6, Y + 0.63, 1.94), M.lamp, rot=(-25, 0, 0))
        F.box(f'taillamp{s}', 0.24, 0.12, 0.03, (s * 0.66, Y + 0.84, -1.94), M.red_lens, bevel=low(0.01, 0), bevel_segments=1)
    F.box('car_grille', 0.9, 0.12, 0.02, (0, Y + 0.45, 1.975), M.dark, bevel=low(0.01, 0), bevel_segments=1)
    F.box('plate_front', 0.52, 0.11, 0.012, (0, Y + 0.3, 1.976), M.white)
    F.box('plate_back', 0.52, 0.11, 0.012, (0, Y + 0.5, -1.975), M.yellow)
    if not is_low():
        for k, (y, z) in enumerate(((0.3, 1.983), (0.5, -1.983))):
            F.box(f'plate_text{k}', 0.4, 0.045, 0.004, (0, Y + y, z), M.ink)
    F.box('car_underbody', 1.5, 0.03, 3.3, (0, Y + 0.18, -0.02), M.dark)
    # the exhaust under it, out at the back on the left
    G.kit.tube(F, 'exhaust', [(0.2, Y + 0.17, 1.5), (0.2, Y + 0.15, 0.6), (0.35, Y + 0.15, -0.6), (0.35, Y + 0.15, -1.7),
                              (0.45, Y + 0.2, -1.98)], 0.025, M.steel, round=low(2, 1))
    F.cyl('silencer', 0.1, 0.5, (0.35, Y + 0.18, -1.35), M.steel, rot=(90, 0, 0), verts=low(14, 6))
    F.box('fuel_tank', 0.8, 0.12, 0.5, (-0.2, Y + 0.22, -0.75), M.iron, bevel=low(0.03, 0), bevel_segments=1)


def _alloy(name, M):
    """A car wheel off its car: the tyre on a silver alloy rim, five dark windows between its spokes and a centre
    cap, its face to the front, its middle at the game origin. One object, placed by its caller."""
    with G.new_parts() as parts:
        _tyre(f'{name}_tyre', 0, 0, 0, CAR_WR, 0.2, M.rubber, grooves=None if is_low() else 2, axis='z',
              segments=low(32, 12))
        G.revolve(f'{name}_rim', [(0, 0.06), (0.17, 0.065), (0.2, 0.09), (0.205, 0.095), (0.205, -0.09), (0, -0.07)],
                  at=F.at(0, 0, 0), material=M.alloy, rot=(90, 0, 0), segments=low(24, 10))
        if not is_low():
            for k in range(5):
                a = math.tau * k / 5
                win = [(0.055 * math.cos(a + d), 0.055 * math.sin(a + d)) for d in (-0.32, 0.32)] + \
                      [(0.165 * math.cos(a + d), 0.165 * math.sin(a + d)) for d in (0.36, 0.0, -0.36)]
                F.extrude(f'{name}_window{k}', win, 0.006, (0, 0, 0.0995), M.dark, plane='front')
        F.cyl(f'{name}_cap', 0.04, 0.02, (0, 0, 0.1), M.chrome, rot=(90, 0, 0), verts=low(12, 6))
    return G.join(parts, name, origin=(0, 0, 0))


def _lift_post(s, M):
    """One post at game x s * CL_POST: a channel on a foot plate, its carriage on the inner face at the arms'
    height, the hydraulic ram up its outer side, a cap; the controls on the left one (s -1)."""
    x = s * CL_POST
    F.box(f'post{s}', 0.24, CL_POST_H - 0.015, 0.3, (x, (CL_POST_H + 0.015) / 2, 0), M.body, bevel=low(0.012, 0),
          bevel_segments=1)
    F.box(f'post_slot{s}', 0.004, CL_POST_H - 0.4, 0.08, (x - s * 0.121, CL_POST_H / 2, 0), M.dark)
    F.box(f'foot{s}', 0.3, 0.015, 0.5, (x, 0.0075, 0), M.iron)
    for dz in (-0.2, 0.2):
        for dx in (-0.1, 0.1):
            F.cyl(f'anchor{s}{dz}{dx}', 0.016, 0.012, (x + dx, 0.021, dz), M.chrome, verts=6)
    F.box(f'cap{s}', 0.28, 0.06, 0.34, (x, CL_POST_H - 0.03, 0), M.iron, bevel=low(0.01, 0), bevel_segments=1)
    F.box(f'carriage{s}', 0.1, 0.55, 0.26, (x - s * 0.17, CL_ARMS + 0.12, 0), M.body, bevel=low(0.01, 0), bevel_segments=1)
    F.cyl(f'ram{s}', 0.04, CL_POST_H - 0.3, (x + s * 0.0, (CL_POST_H - 0.3) / 2 + 0.05, -0.19), M.steel, verts=low(12, 6))
    F.cyl(f'ram_rod{s}', 0.022, 0.25, (x, CL_POST_H - 0.15, -0.19), M.chrome, verts=low(10, 6))
    if s < 0:
        F.box('control_box', 0.18, 0.26, 0.12, (x, 1.25, 0.21), M.white, bevel=low(0.01, 0), bevel_segments=1)
        for k, (y, m) in enumerate(((1.31, M.go), (1.21, M.red_lens))):
            F.cyl(f'button{k}', 0.025, 0.02, (x, y, 0.275), m, rot=(90, 0, 0), verts=low(12, 6))
        G.kit.tube(F, 'cable', [(x, 1.12, 0.21), (x + 0.03, 0.8, 0.2), (x + 0.02, 0.5, 0.19), (x, 0.02, 0.18)], 0.01,
                   M.plastic, round=low(3, 1))


def _lift_arm(s, t, M):
    """An arm from the carriage of post s, to the front (t +1) or back (t -1), its pad under the car's jacking point."""
    x0, z0 = s * (CL_POST - 0.2), t * 0.1
    x1, z1 = s * 0.64, t * 0.86
    dx, dz = x1 - x0, z1 - z0
    L = math.hypot(dx, dz)
    yaw = math.degrees(math.atan2(-dz, -dx))
    G.cube(f'arm{s}{t}', (L * 0.6, 0.12, 0.09), at=F.at(x0 + dx * 0.3, CL_ARMS, z0 + dz * 0.3), material=M.body, rot=(0, 0, yaw),
           bevel=low(0.008, 0), bevel_segments=1)
    G.cube(f'arm_in{s}{t}', (L * 0.55, 0.09, 0.07), at=F.at(x0 + dx * 0.72, CL_ARMS, z0 + dz * 0.72), material=M.steel, rot=(0, 0, yaw))
    F.cyl(f'pivot{s}{t}', 0.05, 0.12, (x0, CL_ARMS, z0), M.iron, verts=low(12, 6))
    F.cyl(f'pad_stem{s}{t}', 0.03, 0.08, (x1, CL_ARMS + 0.07, z1), M.chrome, verts=low(10, 6))
    F.cyl(f'pad{s}{t}', 0.07, 0.035, (x1, CL_ARMS + 0.1, z1), M.rubber, verts=low(16, 8))


def carLift():
    M = _palette(body=('#2F5F9E', 'gloss'),    # the lift: its posts, carriages and arms
                 car=('#C24A3A', 'gloss'),     # the car's paint
                 alloy=('#C9CDD1', 'metal'),   # its wheels' rims
                 go=('#3FAE5A', 'gloss'))      # the lift's up button
    for s in (-1, 1):
        _lift_post(s, M)
        for t in (-1, 1):
            _lift_arm(s, t, M)
    # the floor plate between the posts over the hoses and cables: a low ramp of chequer plate
    F.extrude('floor_plate', [(-0.17, 0), (0.17, 0), (0.12, 0.03), (-0.12, 0.03)], 2 * CL_POST - 0.44, (0, 0, 0), M.steel,
              plane='side')
    _car(M)
    # the wheels: two stacked under its nose, one lying under its tail, one leaning on the right post
    for k, (x, y, z, rx, rz) in enumerate(((-0.45, 0.1, 1.75, 90, 0), (-0.42, 0.3, 1.73, 90, 0), (0.42, 0.1, -1.72, 90, 0),
                                           (CL_POST - 0.24, 0.315, 0.42, -12, 0))):
        w = _alloy(f'wheel{k}', M)
        w.rotation_euler = (math.radians(rx), 0, math.radians(rz))
        w.location = F.at(x, y, z)
    G.join_all('carLift_far' if is_low() else 'carLift')
    G.settle()
    return G.meshes()


# --- the printing press: 2x2, a small web press rolling out the paper ----------------------------------------
# A reel of newsprint behind it; the web runs over a guide roller into the printing unit between two cast side
# frames (ink rollers under a hood, the plate and blanket cylinders, the impression cylinder under them) and out to
# the front, where the cut pages pile on the delivery table. The plate and the blanket cylinders turn against each
# other while a resident watches it run (`rollers`, a loop). Its front to the watcher (+z).
PP_FRAME = 0.66                  # the side frames' inner faces, game x
PP_PLATE = (1.2, 0.0, 0.12)      # the plate cylinder: height, depth, radius
PP_BLANKET = (0.99, 0.13, 0.12)  # the blanket cylinder, against it below and in front
PP_IMPRESSION = (0.79, 0.27, 0.115)
PP_TURN = 1.6                    # seconds a turn


def _press_cylinder(name, y, z, r, surface, marks, M):
    """A cylinder across the press between its frames, its axle's ends in the frames: its surface, the dark gap
    where the plate's (or the blanket's) ends are clamped, and `marks` (the printed blocks it carries) round it.
    Hinged on its axle, to turn."""
    L = 2 * PP_FRAME - 0.02
    with G.new_parts() as parts:
        F.cyl(f'{name}', r, L, (0, y, z), surface, rot=(0, 90, 0), verts=low(G.sides(r, 16, 32), 10))
        F.cyl(f'{name}_axle', 0.035, 2 * PP_FRAME + 0.1, (0, y, z), M.steel, rot=(0, 90, 0), verts=low(10, 6))
        if not is_low():
            for s in (-1, 1):   # its bearers: steel rings at its ends
                F.cyl(f'{name}_bearer{s}', r + 0.004, 0.03, (s * (PP_FRAME - 0.03), y, z), M.steel, rot=(0, 90, 0),
                      verts=G.sides(r, 16, 32))
            F.box(f'{name}_gap', L - 0.08, 0.012, 0.03, (0, y + r - 0.003, z), M.dark)
            F.box(f'{name}_clamp', L - 0.08, 0.01, 0.012, (0, y + r - 0.001, z - 0.021), M.steel)
            for k, (a, w, x) in enumerate(marks):   # print blocks: columns of grey type and a black headline
                t = math.radians(a)
                F.box(f'{name}_mark{k}', w, 0.004, 0.05, (x, y + (r + 0.0005) * math.cos(t), z - (r + 0.0005) * math.sin(t)),
                      M.ink if k % 3 == 0 else M.type, rot=(-a, 0, 0))
    return G.hinged(G.join(parts, name), F.at(0, y, z))


def printingPress():
    M = _palette(body=('#6A8C80', 'satin'),     # the press's machine green
                 plate=('#B9BEC2', 'metal'),    # the plate cylinder's aluminium plate
                 blanket=('#2E3F63', 'matte'),  # the blanket's rubber
                 type=('#8C9196', 'matte'),     # grey type, on the blanket and the pages
                 roller=('#3B3E42', 'satin'),   # the ink rollers' rubber
                 go=('#3FAE5A', 'gloss'))       # the run button
    green = M.body
    with G.new_parts() as still:
        # the base: a cabinet on a dark plinth, panels and vents on its front
        F.box('plinth', 1.44, 0.08, 1.06, (0, 0.04, -0.1), M.dark)
        F.box('base', 1.5, 0.56, 1.12, (0, 0.36, -0.1), green, bevel=low(0.02, 0), bevel_segments=low(2, 1))
        if not is_low():
            for k, x in enumerate((-0.37, 0.37)):
                F.box(f'panel_seam{k}', 0.66, 0.42, 0.004, (x, 0.36, 0.461), M.dark)
                F.box(f'panel{k}', 0.64, 0.4, 0.006, (x, 0.36, 0.463), green, bevel=0.002, bevel_segments=1)
                for j in range(5):
                    F.box(f'vent{k}{j}', 0.3, 0.012, 0.004, (x, 0.24 + j * 0.025, 0.4665), M.dark)
                F.cyl(f'panel_lock{k}', 0.012, 0.01, (x + 0.27, 0.5, 0.468), M.chrome, rot=(90, 0, 0), verts=8)
        F.box('deck', 1.5, 0.03, 1.12, (0, 0.655, -0.1), M.iron, bevel=low(0.006, 0), bevel_segments=1)
        # the side frames: cast plates rising over the cylinders, a hole for each axle
        frame = G.smooth_outline([(-0.62, 0.66), (0.44, 0.66), (0.44, 0.92), (0.3, 1.24), (0.12, 1.42), (-0.22, 1.47),
                                  (-0.5, 1.4), (-0.62, 1.2)], passes=low(2, 1))
        for s in (-1, 1):
            F.extrude(f'frame{s}', frame, 0.07, (s * (PP_FRAME + 0.035), 0, 0), green, plane='side', bevel=low(0.01, 0),
                      bevel_segments=1)
            for k, (y, z, r) in enumerate((PP_PLATE, PP_BLANKET, PP_IMPRESSION)):
                F.cyl(f'boss{s}{k}', 0.06, 0.02, (s * (PP_FRAME + 0.075), y, z), M.iron, rot=(0, 90, 0), verts=low(14, 6))
        # the impression cylinder (steel, smooth: it shows no turning), the ink rollers in a pyramid under a hood,
        # the ink fountain at the back
        y, z, r = PP_IMPRESSION
        F.cyl('impression', r, 2 * PP_FRAME - 0.02, (0, y, z), M.steel, rot=(0, 90, 0), verts=low(G.sides(r, 16, 32), 10))
        for k, (yy, zz, rr) in enumerate(((1.36, -0.08, 0.045), (1.37, 0.03, 0.04), (1.43, -0.17, 0.04), (1.45, -0.05, 0.045),
                                          (1.33, -0.24, 0.04))):
            F.cyl(f'ink_roller{k}', rr, 2 * PP_FRAME - 0.03, (0, yy, zz), M.steel if k % 2 else M.roller, rot=(0, 90, 0),
                  verts=low(14, 6))
        F.extrude('fountain', [(-0.36, 1.3), (-0.22, 1.3), (-0.2, 1.36), (-0.4, 1.42), (-0.44, 1.42)], 2 * PP_FRAME - 0.02,
                  (0, 0, 0), M.iron, plane='side')
        F.box('fountain_ink', 2 * PP_FRAME - 0.06, 0.01, 0.1, (0, 1.39, -0.36), M.ink)
        # the hood over the ink rollers, open to the front where the cylinders show
        steps = [70 + 10 * k for k in range(0, 11, low(1, 5))]
        arc = [(-0.16 + 0.3 * math.cos(math.radians(a)), 1.3 + 0.21 * math.sin(math.radians(a))) for a in steps]
        inner = [(-0.16 + 0.288 * math.cos(math.radians(a)), 1.3 + 0.198 * math.sin(math.radians(a))) for a in reversed(steps)]
        F.extrude('hood', arc + inner, 2 * PP_FRAME - 0.01, (0, 0, 0), green, plane='side')
        # the reel stand behind it: two A legs, the reel of newsprint on its shaft, a brake disc
        RY, RZ, RR = 0.4, -0.7, 0.22
        for s in (-1, 1):
            F.extrude(f'reel_leg{s}', [(-0.88, 0.0), (-0.56, 0.0), (RZ + 0.05, RY + 0.06), (RZ - 0.05, RY + 0.06)], 0.06,
                      (s * 0.6, 0, 0), green, plane='side', bevel=low(0.008, 0), bevel_segments=1)
            F.box(f'reel_foot{s}', 0.12, 0.03, 0.36, (s * 0.6, 0.015, RZ), M.iron)
        F.cyl('reel', RR, 1.0, (0, RY, RZ), M.paper, rot=(0, 90, 0), verts=low(G.sides(RR, 24, 40), 12))
        F.cyl('reel_core', 0.05, 1.04, (0, RY, RZ), M.dark, rot=(0, 90, 0), verts=low(12, 6))
        F.cyl('reel_shaft', 0.025, 1.3, (0, RY, RZ), M.steel, rot=(0, 90, 0), verts=low(10, 6))
        F.cyl('brake', 0.11, 0.025, (-0.66, RY, RZ), M.iron, rot=(0, 90, 0), verts=low(16, 8))
        # the web: up from the reel over the guide roller at the back of the frames, and out of the cylinders' nip to the
        # cutter at the front
        F.cyl('guide', 0.04, 2 * PP_FRAME - 0.02, (0, 1.16, -0.56), M.steel, rot=(0, 90, 0), verts=low(12, 6))
        a0 = (RZ + RR * 0.15, RY + RR * 0.99)
        a1 = (-0.56 - 0.03, 1.16 + 0.03)
        L = math.hypot(a1[0] - a0[0], a1[1] - a0[1])
        ang = math.degrees(math.atan2(a1[1] - a0[1], a1[0] - a0[0]))
        F.box('web_in', 0.96, 0.002, L, (0, (a0[1] + a1[1]) / 2, (a0[0] + a1[0]) / 2), M.paper, rot=(ang, 0, 0))
        F.box('web_out', 0.96, 0.002, 0.2, (0, 0.7, 0.44), M.paper)
        # the cutter at the front and the delivery table with its pile of pages, a few fanned on top
        F.box('cutter', 1.2, 0.14, 0.14, (0, 0.74, 0.5), green, bevel=low(0.015, 0), bevel_segments=1)
        F.box('cutter_slot', 1.0, 0.01, 0.004, (0, 0.72, 0.571), M.dark)
        F.box('table', 1.0, 0.025, 0.36, (0, 0.6, 0.75), M.steel, bevel=low(0.006, 0), bevel_segments=1)
        for s in (-1, 1):
            F.box(f'table_leg{s}', 0.04, 0.6, 0.04, (s * 0.45, 0.3, 0.9), M.iron)
            F.box(f'table_side{s}', 0.015, 0.1, 0.36, (s * 0.49, 0.66, 0.75), M.steel)
        F.box('pile', 0.56, 0.14, 0.32, (0, 0.6125 + 0.07, 0.75), M.paper, bevel=low(0.004, 0), bevel_segments=1)
        for k in range(low(3, 1)):
            yk = 0.7525 + 0.003 * k
            rot = (0, 4 - 5 * k, 0)
            F.box(f'page{k}', 0.56, 0.002, 0.34, (0.01 * k, yk, 0.75 + 0.012 * k), M.paper, rot=rot)
        if not is_low():
            # the top page's front: its headline and columns
            F.box('headline', 0.44, 0.003, 0.05, (0.02, 0.7585, 0.66), M.ink, rot=(0, -6, 0))
            for j in range(4):
                F.box(f'column{j}', 0.1, 0.003, 0.2, (-0.15 + j * 0.12, 0.7585, 0.8), M.type, rot=(0, -6, 0))
            F.box('photo', 0.12, 0.0032, 0.1, (0.15, 0.759, 0.74), M.ink, rot=(0, -6, 0))
        # the control panel on the right frame's front: a sloping desk of buttons, a speed dial, the stop
        F.box('control_post', 0.12, 0.5, 0.12, (-0.78, 0.9, 0.3), green, bevel=low(0.01, 0), bevel_segments=1)
        F.box('control_desk', 0.26, 0.2, 0.08, (-0.78, 1.2, 0.33), M.plastic, rot=(-30, 0, 0), bevel=low(0.01, 0),
              bevel_segments=1)
        for k, (x, m) in enumerate(((-0.71, M.go), (-0.78, M.amber), (-0.85, M.white))):
            F.cyl(f'button{k}', 0.016, 0.02, (x, 1.22, 0.375), m, rot=(60, 0, 0), verts=low(10, 6))
        F.cyl('stop', 0.028, 0.03, (-0.78, 1.14, 0.42), M.red_lens, rot=(60, 0, 0), verts=low(12, 6))
        # the drive's belt guard on the left frame
        F.extrude('belt_guard', G.rounded_rect(0.7, 0.5, 0.18, corner=low(4, 2)), 0.1, (0.76, 1.0, -0.1), M.body,
                  plane='side', bevel=low(0.01, 0), bevel_segments=1)
    G.join(still, 'printingPress_far' if is_low() else 'printingPress', origin=(0, 0, 0))
    if is_low():
        G.settle()
        return G.meshes()
    # the plate and blanket cylinders, each turning on its axle (the far version keeps them still, joined)
    page = [(a, 0.3, x) for a in (30, 75, 200, 250, 300) for x in (-0.3, 0.3)]
    y, z, r = PP_PLATE
    plate = _press_cylinder('plate_cylinder', y, z, r, M.plate, page, M)
    y, z, r = PP_BLANKET
    blanket = _press_cylinder('blanket_cylinder', y, z, r, M.blanket, page, M)
    G.settle()
    G.spin(plate, 'rollers', turns=1, seconds=PP_TURN, axis='X')
    G.spin(blanket, 'rollers', turns=-1, seconds=PP_TURN, axis='X')
    return G.meshes()


# --- the fire pole: 1x1, a brass pole from floor to ceiling on a round landing mat -------------------------------
CEILING = 2.8        # where the ceiling things hang from: the pole runs up to it, through the watch room's floor


def firePole():
    M = _palette(body=('#C9A04A', 'metal', {'roughness': 0.16}),   # the polished brass pole
                 mat=('#2E2F33', 'matte'))     # the rubber landing mat
    # the landing mat: a thick round of rubber, its edge bevelled, rings moulded in its top, a hole for the pole
    prof = [(0.04, 0.0), (0.4, 0.0), (0.42, 0.012), (0.417, 0.028), (0.4, 0.036)]
    if not is_low():
        for r in (0.35, 0.28, 0.21, 0.14):
            prof += [(r + 0.012, 0.036), (r + 0.006, 0.042), (r - 0.006, 0.042), (r - 0.012, 0.036)]
    prof += [(0.06, 0.036), (0.04, 0.03), (0.04, 0.0)]
    G.revolve('mat', prof, at=F.at(0, 0, 0), material=M.mat, segments=low(44, 16))
    # the pole on its floor flange, through the ceiling's collar
    r = 0.0375
    G.revolve('flange', [(0, 0.03), (0.08, 0.03), (0.08, 0.04), (0.06, 0.05), (0.045, 0.08), (0.0, 0.08)], at=F.at(0, 0, 0),
              material=M.brass, segments=low(24, 10))
    F.cyl('pole', r, CEILING - 0.1, (0, 0.05 + (CEILING - 0.1) / 2, 0), M.body, verts=low(G.sides(r, 16), 8))
    G.revolve('collar', [(0, CEILING - 0.12), (0.045, CEILING - 0.12), (0.06, CEILING - 0.08), (0.06, CEILING - 0.03),
                         (0.13, CEILING - 0.02), (0.13, CEILING), (0, CEILING)], at=F.at(0, 0, 0), material=M.brass,
              segments=low(24, 10))
    G.join_all('firePole_far' if is_low() else 'firePole')
    G.settle()
    return G.meshes()


# --- the hose rack: 1x1 on the wall, two hoses coiled over its saddles -------------------------------------------
def _coil(name, cx, top, rx, ry, turns, r, material, z0, steps=16):
    """A hose coiled over a saddle: `turns` loops hanging from the saddle's top at height `top`, `rx` by `ry` round,
    each a hose's width further from the wall than the last, from depth z0; returns the coil's (start, end) points."""
    pts = []
    n = int(turns * steps)
    for i in range(n + 1):
        a = math.tau * i / steps
        pts.append((cx + rx * math.sin(a), top - ry * (1 - math.cos(a)), z0 + 2.05 * r * i / steps))
    F.sweep(name, pts, r, material, verts=low(6, 4))
    return pts[0], pts[-1]


def hoseRack():
    M = _palette(body=('#B8221E', 'gloss'),    # the rack's red steel
                 hose=('#C9473A', 'satin'),    # the red hose
                 hose2=('#E6DFCC', 'satin'))   # the white one
    z = WALL
    with G.new_parts():
        # the rail on the wall, bolted at its ends, and the two saddles standing off it
        F.box('rail', 0.88, 0.1, 0.025, (0, 1.62, z + 0.0125), M.body, bevel=low(0.006, 0), bevel_segments=1)
        for s in (-1, 1):
            for dy in (-0.025, 0.025):
                F.cyl(f'bolt{s}{dy}', 0.008, 0.008, (s * 0.41, 1.62 + dy, z + 0.029), M.chrome, rot=(90, 0, 0), verts=6)
        tops = {}
        for k, (x, hose, turns) in enumerate(((-0.22, M.hose, 4), (0.22, M.hose2, 3.5))):
            # a saddle: an arm out from the rail, curving up at its end, a curved plate the coil hangs on
            F.box(f'arm{k}', 0.05, 0.04, 0.26, (x, 1.6, z + 0.155), M.body, bevel=low(0.005, 0), bevel_segments=1)
            F.box(f'lip{k}', 0.05, 0.08, 0.02, (x, 1.63, z + 0.29), M.body, bevel=low(0.005, 0), bevel_segments=1)
            arc = [(x + 0.11 * math.cos(math.radians(a)), 1.58 + 0.04 * math.sin(math.radians(a)), z + 0.155)
                   for a in range(0, 181, low(30, 90))]
            F.sweep(f'saddle{k}', arc, None, M.body, profile=[(-0.004, -0.13), (0.004, -0.13), (0.004, 0.13), (-0.004, 0.13)],
                    up=(0, 0, 1))
            r = 0.02
            a, b = _coil(f'coil{k}', x, 1.62 + 0.004 + r, 0.18, 0.21, turns, r, hose, z + 0.05, steps=low(16, 8))
            tops[k] = (a, b, x, hose)
        # the ends: a brass coupling hanging from the back turn, and a branch (the nozzle) from the front one
        for k, (a, b, x, hose) in tops.items():
            end = (b[0], b[1], b[2])
            drop = [end, (end[0] + 0.01, end[1] - 0.12, end[2] + 0.01), (end[0], end[1] - 0.26, end[2] + 0.02)]
            F.sweep(f'tail{k}', drop, 0.02, hose, verts=low(6, 4), resolution=low(2, 1))
            bx, by, bz = drop[-1]
            if k == 0:
                G.revolve(f'branch{k}', [(0, 0.0), (0.03, 0.0), (0.03, -0.04), (0.024, -0.06), (0.024, -0.2), (0.012, -0.26),
                                         (0.012, -0.29), (0, -0.29)], at=F.at(bx, by, bz), material=M.brass, segments=low(12, 6))
                F.box(f'branch_grip{k}', 0.07, 0.1, 0.07, (bx, by - 0.1, bz), M.plastic, bevel=low(0.01, 0), bevel_segments=1)
            else:
                G.revolve(f'coupling{k}', [(0, 0.0), (0.034, 0.0), (0.036, -0.02), (0.036, -0.07), (0.03, -0.08), (0, -0.08)],
                          at=F.at(bx, by, bz), material=M.brass, segments=low(12, 6))
                for j in range(2):   # its lugs
                    F.box(f'lug{k}{j}', 0.1, 0.012, 0.02, (bx, by - 0.04 - 0.02 * j, bz), M.brass)
    G.join_all('hoseRack_far' if is_low() else 'hoseRack')
    G.settle(centre=False, floor=False)
    return G.meshes()


# --- the server rack: 1x1, a tall glass-doored cabinet of servers with blinking lights ----------------------------
SR_W, SR_H, SR_D = 0.6, 2.0, 0.86     # the cabinet: back on the wall's face, door to the front
SR_U = 0.04445                         # a rack unit
SR_FACE = WALL + SR_D - 0.06           # the servers' faces, behind the door


def _server(name, y, units, M, kind, leds, blink):
    """One unit in the rack at height y (its bottom), `units` high: a bezel, what its kind shows on its face (drive
    bays, ports, a screen), and its LEDs: kept on `leds`, the ones that blink on `blink` (lists of parts)."""
    h = units * SR_U - 0.003
    w = 0.482
    yc = y + h / 2
    F.box(name, w, h, 0.03, (0, yc, SR_FACE - 0.015), M.unit if kind != 'blank' else M.dark,
          bevel=0.002 if kind != 'blank' and not is_low() else 0, bevel_segments=1)
    if kind == 'blank' or is_low():
        return
    face = SR_FACE
    if kind == 'server':
        bays = 8 if units == 2 else 4
        for k in range(bays):
            F.box(f'{name}_bay{k}', 0.046, h - 0.016, 0.004, (-0.2 + k * 0.05, yc, face + 0.001), M.bay)
        F.box(f'{name}_tag', 0.03, h - 0.02, 0.004, (0.215, yc, face + 0.001), M.white)
        for k in range(3):
            led = M.led_green if k < 2 else M.led_blue
            (blink if (k + units) % 2 else leds).append(
                F.box(f'{name}_led{k}', 0.01, 0.008, 0.004, (0.19, y + 0.012 + 0.012 * k, face + 0.002), led))
    elif kind == 'switch':
        for k in range(12):
            F.box(f'{name}_port{k}', 0.014, 0.012, 0.004, (-0.19 + k * 0.022, yc, face + 0.001), M.dark)
            if k % 2 == 0:
                (blink if k % 4 else leds).append(
                    F.box(f'{name}_led{k}', 0.008, 0.006, 0.004, (-0.19 + k * 0.022, yc + 0.013, face + 0.002),
                          M.led_amber if k % 3 else M.led_green))
    elif kind == 'patch':
        cables = (M.cable_blue, M.cable_yellow, M.cable_blue, M.cable_red, M.cable_blue, M.cable_yellow)
        for k in range(12):
            F.box(f'{name}_port{k}', 0.014, 0.012, 0.004, (-0.19 + k * 0.022, yc, face + 0.001), M.dark)
        for k, c in enumerate(cables):   # patch leads looping down and off to the side
            x = -0.19 + k * 0.044
            G.kit.tube(F, f'{name}_lead{k}', [(x, yc, face + 0.01), (x, yc - 0.01, face + 0.035), (x + 0.02, yc - 0.06, face + 0.035),
                                              (0.23, yc - 0.05 - 0.004 * k, face + 0.02)], 0.0035, c, verts=4, round=2)
    elif kind == 'ups':
        F.box(f'{name}_screen', 0.08, 0.04, 0.004, (0.12, yc + 0.02, face + 0.001), M.screen)
        for k in range(4):
            F.box(f'{name}_vent{k}', 0.25, 0.006, 0.004, (-0.08, yc - 0.03 + k * 0.02, face + 0.001), M.dark)
        leds.append(F.box(f'{name}_led', 0.008, 0.008, 0.004, (0.2, yc - 0.03, face + 0.002), M.led_green))


def serverRack():
    M = _palette(body=('#222528', 'satin'),     # the cabinet's black steel
                 unit=('#7A8088', 'satin'),     # the servers' grey bezels
                 bay=('#2E3236', 'satin'),      # their drive bays
                 glass=('#B9C8CE', 'glass'),    # the door's glass
                 screen=('#4FA8D8', 'gloss', {'glow': '#7FC8FF'}),
                 led_green=('#4BE07A', 'gloss', {'glow': '#5CFF8A'}),
                 led_blue=('#4F8CFF', 'gloss', {'glow': '#6FA4FF'}),
                 led_amber=('#FFB23A', 'gloss', {'glow': '#FFC25A'}),
                 cable_blue=('#3E7BC8', 'satin'), cable_yellow=('#E8C23A', 'satin'), cable_red=('#C83A3A', 'satin'))
    z0, z1 = WALL, WALL + SR_D
    zc = (z0 + z1) / 2
    black = M.body
    leds, blink = [], []
    with G.new_parts() as still:
        # the cabinet: a plinth, its sides, roof and back, the rails the servers hang on
        F.box('plinth', SR_W - 0.02, 0.06, SR_D - 0.04, (0, 0.03, zc), M.dark)
        for s in (-1, 1):
            F.box(f'side{s}', 0.02, SR_H - 0.06, SR_D, (s * (SR_W / 2 - 0.01), (SR_H + 0.06) / 2, zc), black,
                  bevel=low(0.004, 0), bevel_segments=1)
            F.box(f'rail{s}', 0.02, SR_H - 0.14, 0.02, (s * 0.25, SR_H / 2, SR_FACE - 0.01), M.steel)
            for k in range(low(40, 0)):
                F.box(f'rail_hole{s}{k}', 0.006, 0.008, 0.002, (s * 0.25, 0.1 + k * SR_U, SR_FACE + 0.0005), M.dark)
        F.box('roof', SR_W, 0.03, SR_D, (0, SR_H - 0.015, zc), black, bevel=low(0.004, 0), bevel_segments=1)
        F.box('back', SR_W - 0.04, SR_H - 0.09, 0.02, (0, SR_H / 2, z0 + 0.01), black)
        F.box('inside', SR_W - 0.04, SR_H - 0.1, 0.004, (0, SR_H / 2, z0 + 0.3), M.dark)
        if not is_low():
            for k in range(6):   # the roof's vents and its cable brush
                F.box(f'roof_vent{k}', 0.4, 0.004, 0.012, (0, SR_H + 0.0005, zc + 0.2 - k * 0.03), M.dark)
            F.box('brush', 0.3, 0.004, 0.06, (0, SR_H + 0.0005, z0 + 0.1), M.plastic)
        # the units, from the floor up: a UPS, storage, servers, a switch and a patch panel, blanks between
        y = 0.08
        for k, (units, kind) in enumerate(((3, 'ups'), (2, 'blank'), (4, 'server'), (2, 'server'), (2, 'server'), (1, 'blank'),
                                          (2, 'server'), (2, 'server'), (2, 'server'), (1, 'blank'), (1, 'server'),
                                          (1, 'server'), (1, 'server'), (2, 'blank'), (1, 'switch'), (1, 'patch'),
                                          (1, 'switch'), (1, 'patch'), (4, 'blank'), (2, 'server'), (1, 'blank'))):
            _server(f'unit{k}', y, units, M, kind, leds, blink)
            y += units * SR_U
        # the door: a frame round its glass, hinged on the left, a swing handle on the right
        DZ = z1 - 0.012
        frame = G.rounded_rect(SR_W, SR_H - 0.07, 0.01, corner=1)
        hole = [(u, v) for u, v in G.rounded_rect(SR_W - 0.1, SR_H - 0.26, 0.02, corner=low(2, 1))]
        F.extrude('door', [(u, v + SR_H / 2 + 0.035) for u, v in frame], 0.024, (0, 0, DZ), black,
                  holes=[[(u, v + SR_H / 2 + 0.06) for u, v in hole]], bevel=low(0.004, 0), bevel_segments=1)
        F.box('door_glass', SR_W - 0.09, SR_H - 0.25, 0.006, (0, SR_H / 2 + 0.06, DZ), M.glass)
        F.box('handle_plate', 0.03, 0.2, 0.01, (-0.255, 1.05, z1 + 0.005), M.plastic, bevel=low(0.004, 0), bevel_segments=1)
        F.box('handle', 0.022, 0.15, 0.022, (-0.255, 1.05, z1 + 0.018), M.chrome, bevel=low(0.006, 0), bevel_segments=1)
        for k, y in enumerate((0.25, 1.75)):
            F.box(f'hinge{k}', 0.012, 0.07, 0.014, (0.293, y, z1 - 0.01), M.steel)
        if not is_low():
            F.box('label', 0.12, 0.03, 0.002, (0, SR_H - 0.06, z1 + 0.001), M.white)
    G.join([o for o in still if o not in blink] if not is_low() else still, 'serverRack_far' if is_low() else 'serverRack',
           origin=(0, 0, 0))
    if is_low():
        G.settle(centre=False)
        return G.meshes()
    lights = G.join(blink, 'leds', origin=F.at(0, 1.0, SR_FACE))
    G.settle(centre=False)
    # the blinking LEDs: back into their bezels for a moment, now and then
    rest = lights.location[1]
    d = 0.006
    G.keys(lights, 'blink', 'location', 1, [(0, rest, 'linear'), (0.5, rest, 'linear'), (0.55, rest + d, 'linear'),
                                            (0.85, rest + d, 'linear'), (0.9, rest, 'linear'), (1.3, rest, 'linear'),
                                            (1.35, rest + d, 'linear'), (1.5, rest + d, 'linear'), (1.55, rest, 'linear'),
                                            (2.2, rest, 'linear')])
    return G.meshes()


# --- the tyre rack: 2x1 against the wall, steel shelves of new tyres ---------------------------------------------
TR_D = 0.66          # the frames' depth, a tyre's height: back on the wall's face
TR_LEVELS = (0.12, 0.76, 1.4)


def tyreRack():
    M = _palette(body=('#E07A1F', 'satin'),     # the frames' orange uprights
                 beam=('#5C6168', 'metal'),     # the beams
                 label=('#F2C14E', 'gloss'), label2=('#3E7BC8', 'gloss'), label3=('#F1F0EC', 'gloss'))
    z0 = WALL
    zc = z0 + TR_D / 2
    with G.new_parts():
        for s in (-1, 1):
            x = s * 0.93
            for k, z in enumerate((z0 + 0.035, z0 + TR_D - 0.035)):
                F.box(f'upright{s}{k}', 0.06, 2.0, 0.07, (x, 1.0, z), M.body, bevel=low(0.004, 0), bevel_segments=1)
                F.box(f'footplate{s}{k}', 0.1, 0.012, 0.1, (x, 0.006, z + (0.015 if k == 0 else -0.015)), M.iron)
                if not is_low():
                    for j in range(24):
                        F.box(f'slot{s}{k}{j}', 0.004, 0.025, 0.012, (x - s * 0.031, 0.2 + j * 0.075, z), M.dark)
            for j, y in enumerate((0.3, 0.9, 1.5, 1.95)):   # the frame's bracing
                F.box(f'brace{s}{j}', 0.03, 0.03, TR_D - 0.12, (x, y, zc), M.body)
            for j, (ya, yb) in enumerate(((0.3, 0.9), (0.9, 1.5))):
                L = math.hypot(TR_D - 0.12, yb - ya)
                F.box(f'diag{s}{j}', 0.025, L, 0.025, (x, (ya + yb) / 2, zc), M.body,
                      rot=((1 if j % 2 else -1) * math.degrees(math.atan2(TR_D - 0.12, yb - ya)), 0, 0))
        labels = (M.label, M.label2, M.label3)
        for lv, y in enumerate(TR_LEVELS):
            # a level: two step beams, front and back, the tyres standing in a row in the gap between them
            for k, z in enumerate((z0 + 0.06, z0 + TR_D - 0.06)):
                F.box(f'beam{lv}{k}', 1.8, 0.1, 0.05, (0, y, z), M.beam, bevel=low(0.004, 0), bevel_segments=1)
            if not is_low():
                for t in range(3):
                    F.box(f'tag{lv}{t}', 0.08, 0.04, 0.003, (-0.6 + t * 0.6, y, z0 + TR_D - 0.034), M.label3)
            r, w = 0.31, 0.205
            n = 8
            for t in range(n):
                x = -0.79 + t * (1.58 / (n - 1))
                ty = y + 0.05 - (r - math.sqrt(r * r - (TR_D / 2 - 0.085) ** 2)) + r   # down between the beams
                _tyre(f'tyre{lv}{t}', x, ty, zc, r, w, M.rubber, grooves=None, segments=low(20, 10))
                if not is_low():
                    # the tread's label, on its crown to the front
                    F.box(f'label{lv}{t}', 0.12, 0.16, 0.004, (x, ty, zc + r + 0.001), labels[(t + lv) % 3])
    G.join_all('tyreRack_far' if is_low() else 'tyreRack')
    G.settle(centre=False)
    return G.meshes()


# --- the tool chest: 1x1 against the wall, a red roller cabinet with a top box ------------------------------------
TC_W, TC_D = 0.68, 0.46


def _drawer_front(name, y0, h, z, M, out=0.0):
    """A drawer's front from height y0, `h` high, its face at depth z (`out` further, pulled out): the red front, the
    aluminium pull along its top edge."""
    F.box(name, TC_W - 0.04, h - 0.006, 0.018, (0, y0 + h / 2, z + out + 0.009), M.body, bevel=low(0.003, 0), bevel_segments=1)
    F.box(f'{name}_pull', TC_W - 0.08, 0.014, 0.022, (0, y0 + h - 0.018, z + out + 0.029), M.steel, bevel=low(0.004, 0),
          bevel_segments=1)


def _spanner(name, x, y, z, length, M, turn=0.0):
    """A combination spanner lying flat at (x, y, z), along the depth turned `turn` degrees: a bar, an open jaw at one
    end, a ring at the other."""
    hw = length * 0.05
    out = [(-length / 2 + hw * 1.6, -hw * 0.5), (length / 2 - hw * 1.6, -hw * 0.5), (length / 2 - hw * 1.6, hw * 0.5),
           (-length / 2 + hw * 1.6, hw * 0.5)]
    with G.new_parts() as parts:
        F.extrude(f'{name}_bar', out, 0.006, (x, y, z), M.chrome, plane='top')
        F.extrude(f'{name}_ring', [(math.cos(a) * hw * 1.6 - length / 2 + hw * 1.6, math.sin(a) * hw * 1.6) for a in
                                   (math.tau * k / 10 for k in range(10))], 0.008, (x, y, z), M.chrome, plane='top',
                  holes=[[(math.cos(a) * hw * 0.9 - length / 2 + hw * 1.6, math.sin(a) * hw * 0.9) for a in
                          (math.tau * k / 8 for k in range(8))]])
        jaw = [(length / 2 - hw * 2.2, -hw * 1.4), (length / 2, -hw * 1.6), (length / 2, -hw * 0.6),
               (length / 2 - hw * 1.2, -hw * 0.6), (length / 2 - hw * 1.2, hw * 0.6), (length / 2, hw * 0.6),
               (length / 2, hw * 1.6), (length / 2 - hw * 2.2, hw * 1.4)]
        F.extrude(f'{name}_jaw', jaw, 0.008, (x, y, z), M.chrome, plane='top')
    for p in parts:
        p.rotation_euler[2] += math.radians(turn)
    return parts


def toolChest():
    M = _palette(body=('#C0201C', 'gloss'),     # the chest's red
                 foam=('#2B2C2E', 'matte'))     # the drawers' foam liners
    z0 = WALL
    front = z0 + TC_D
    zc = z0 + TC_D / 2
    with G.new_parts():
        # the roller cabinet: a carcass on four castors, its drawers, a push handle on its right side
        F.box('cabinet', TC_W, 0.84, TC_D, (0, 0.13 + 0.42, zc), M.body, bevel=low(0.008, 0), bevel_segments=1)
        F.box('cabinet_top', TC_W + 0.01, 0.02, TC_D + 0.005, (0, 0.98, zc + 0.0025), M.rubber, bevel=low(0.004, 0), bevel_segments=1)
        F.box('gaps', TC_W - 0.03, 0.78, 0.002, (0, 0.55, front + 0.0005), M.dark)
        y = 0.16
        for k, h in enumerate((0.2, 0.12, 0.12, 0.075, 0.075, 0.075, 0.075)):
            if k == 5:
                open_at = (y, h)
            else:
                _drawer_front(f'drawer{k}', y, h, front + 0.001, M)
            y += h
        # the third drawer pulled out: its sides and its foam liner of spanners showing
        y2, h2 = open_at
        out = 0.16
        _drawer_front('drawer_open', y2, h2, front + 0.001, M, out=out)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.012, h2 - 0.02, out + 0.02, (s * (TC_W / 2 - 0.035), y2 + (h2 - 0.02) / 2 + 0.004,
                                                                    front + out / 2 - 0.01), M.steel)
        F.box('drawer_foam', TC_W - 0.08, 0.012, out + 0.02, (0, y2 + 0.04, front + out / 2 - 0.01), M.foam)
        if not is_low():
            for k, (L, t) in enumerate(((0.12, 0), (0.14, 0), (0.16, 0), (0.18, 0), (0.2, 0), (0.22, 0))):
                _spanner(f'spanner{k}', -0.23 + k * 0.09, y2 + 0.049, front + out / 2 - 0.02, L, M, turn=90 + t)
        # castors under it: a swivel plate, a fork and a wheel each, brakes on the front pair
        for sx in (-1, 1):
            for sz in (-1, 1):
                cx, cz = sx * (TC_W / 2 - 0.06), zc + sz * (TC_D / 2 - 0.07)
                F.box(f'castor_plate{sx}{sz}', 0.08, 0.008, 0.08, (cx, 0.126, cz), M.steel)
                F.box(f'castor_fork{sx}{sz}', 0.05, 0.05, 0.06, (cx, 0.1, cz - 0.01), M.steel)
                F.cyl(f'castor{sx}{sz}', 0.045, 0.035, (cx, 0.045, cz - 0.02), M.plastic, rot=(0, 90, 0), verts=low(14, 6))
                if sz > 0:
                    F.box(f'brake{sx}', 0.03, 0.012, 0.04, (cx, 0.11, cz + 0.045), M.red_lens)
        G.kit.tube(F, 'handle', [(-TC_W / 2, 0.86, zc - 0.15), (-TC_W / 2 - 0.05, 0.86, zc - 0.15),
                                 (-TC_W / 2 - 0.05, 0.86, zc + 0.15), (-TC_W / 2, 0.86, zc + 0.15)], 0.012, M.chrome,
                   round=low(2, 1))
        # the top box: drawers under a lid, its back to the wall, a hasp and handles on its ends
        TB = 0.34
        tz = z0 + TB / 2
        F.box('topbox', TC_W, 0.36, TB, (0, 0.99 + 0.18, tz), M.body, bevel=low(0.008, 0), bevel_segments=1)
        F.box('top_gaps', TC_W - 0.03, 0.28, 0.002, (0, 1.13, z0 + TB + 0.0005), M.dark)
        y = 0.995
        for k, h in enumerate((0.06, 0.06, 0.06, 0.1)):
            _drawer_front(f'top_drawer{k}', y, h, z0 + TB + 0.001, M)
            y += h
        F.box('lid', TC_W + 0.006, 0.08, TB + 0.003, (0, 1.39, tz + 0.0015), M.body, bevel=low(0.012, 0), bevel_segments=low(2, 1))
        F.box('lid_trim', TC_W - 0.1, 0.02, 0.012, (0, 1.39, z0 + TB + 0.008), M.steel, bevel=low(0.004, 0), bevel_segments=1)
        F.box('hasp', 0.04, 0.05, 0.012, (0, 1.36, z0 + TB + 0.01), M.chrome)
        for s in (-1, 1):
            G.kit.tube(F, f'end_handle{s}', [(s * TC_W / 2, 1.27, tz - 0.08), (s * (TC_W / 2 + 0.03), 1.27, tz - 0.08),
                                            (s * (TC_W / 2 + 0.03), 1.27, tz + 0.08), (s * TC_W / 2, 1.27, tz + 0.08)], 0.008,
                       M.chrome, round=low(2, 1))
        # on the cabinet's top in front of the box: a socket set's case and a spanner left out
        F.box('socket_case', 0.3, 0.05, 0.1, (0.12, 1.015, front - 0.06), M.plastic, bevel=low(0.01, 0), bevel_segments=1)
        if not is_low():
            _spanner('loose', -0.18, 0.993, front - 0.06, 0.2, M, turn=20)
    G.join_all('toolChest_far' if is_low() else 'toolChest')
    G.settle(centre=False)
    return G.meshes()


# --- the timber stack: 2x1 outdoors, planks and posts on bearers under a roped tarpaulin, an old tyre on it ---------
TS_L = 1.82      # the planks' length, across


def timberStack():
    M = _palette(body=('#3C6E9E', 'fabric'),     # the tarpaulin
                 timber=('#C9A56E', 'wood'),     # sawn planks and posts
                 bearer=('#8A6A48', 'wood'),     # the weathered bearers and stickers
                 rope=('#D9C28A', 'matte'))
    with G.new_parts():
        for k, x in enumerate((-0.7, 0.0, 0.7)):
            F.box(f'bearer{k}', 0.1, 0.08, 0.86, (x, 0.04, 0), M.bearer, bevel=low(0.006, 0), bevel_segments=1)
        y = 0.08
        # layers of planks, stickers across between them; posts on top
        for lv in range(4):
            for k in range(5):
                z = -0.34 + k * 0.17
                F.box(f'plank{lv}{k}', TS_L - 0.02 * ((lv + k) % 3), 0.032, 0.15,
                      (0.01 * ((lv * 3 + k) % 5 - 2), y + 0.016, z), M.timber, bevel=low(0.003, 0), bevel_segments=1)
            y += 0.032
            for j, x in enumerate((-0.7, 0.0, 0.7)):
                F.box(f'sticker{lv}{j}', 0.04, 0.02, 0.84, (x, y + 0.01, 0), M.bearer)
            y += 0.02
        for k in range(4):
            F.box(f'post{k}', TS_L - 0.04 * (k % 2), 0.1, 0.1, (0.015 * (k % 3 - 1), y + 0.05, -0.27 + k * 0.18), M.timber,
                  bevel=low(0.005, 0), bevel_segments=1)
        y += 0.1
        top = y
        # the tarpaulin over the middle of it, hanging down its front and back, the ends of the timber showing
        drape = 0.26
        F.soft('tarp', 1.5, drape + 0.03, 0.9, (0, top - drape / 2 + 0.03, 0), M.body, round=0.04, puff=0.006,
               pinch=0.0, rumple=low(0.02, 0.0), rumple_size=0.3, seed=3.0, resting=True)
        # two ropes over it, down to the bearers
        for k, x in enumerate((-0.5, 0.5)):
            pts = [(x, 0.06, 0.44), (x + 0.02, top - drape + 0.02, 0.458), (x + 0.02, top + 0.03, 0.43), (x, top + 0.035, 0.0),
                   (x - 0.02, top + 0.03, -0.43), (x - 0.02, top - drape + 0.02, -0.458), (x, 0.06, -0.44)]
            G.kit.tube(F, f'rope{k}', G.kit.fillet(pts, 0.03, steps=low(2, 1)), 0.008, M.rope, verts=low(6, 4), round=1)
        # an old tyre on top, keeping the tarp down
        _tyre('old_tyre', 0.32, top + 0.045 + 0.08, -0.1, 0.3, 0.17, M.rubber, grooves=None, segments=low(24, 10), axis='y')
    G.join_all('timberStack_far' if is_low() else 'timberStack')
    G.settle()
    return G.meshes()


# --- the cement mixer: 1x1 outdoors, a little orange mixer, its drum crusted grey -------------------------------
CM_TILT = 55     # the drum's axis, degrees from upright, its mouth up to the front


def cementMixer():
    M = _palette(body=('#E8742A', 'gloss'),     # the orange drum and motor cover
                 crust=('#A39F96', 'matte'),    # dried cement
                 frame=('#3A3D41', 'metal'))    # the stand
    DY, DZ = 0.86, 0.0                          # the drum's middle
    with G.new_parts() as drum:
        # the drum: a cone up from its gear end to the belly, a shoulder in to the mouth, the inside dark and crusted
        n = low(32, 12)
        G.revolve('drum', [(0, -0.32), (0.13, -0.32), (0.24, -0.24), (0.3, -0.1), (0.305, -0.02), (0.3, 0.04),
                           (0.24, 0.16), (0.16, 0.26), (0.15, 0.3), (0.13, 0.3), (0.13, 0.26), (0.22, 0.12), (0.0, 0.06)],
                  at=(0, 0, 0), material=M.body, segments=n)
        # the cement dried on its inside: a thin shell just within it
        G.revolve('drum_crust', [(0, 0.064), (0.214, 0.122), (0.126, 0.258), (0.126, 0.296), (0.116, 0.296), (0.116, 0.255),
                                 (0.2, 0.13), (0, 0.075)], at=(0, 0, 0), material=M.crust, segments=n)
        G.torus('lip_crust', 0.142, 0.016, at=(0, 0, 0.3), material=M.crust, major_segments=n, minor_segments=low(6, 4))
        G.torus('gear', 0.302, 0.018, at=(0, 0, -0.06), material=M.iron, major_segments=n, minor_segments=low(6, 4))
        if not is_low():
            for k in range(8):   # crusts and dribbles down the front of the drum from its mouth
                a = math.tau * k / 8 + 0.4 * (k % 3)
                t = 0.15 + 0.2 * ((k * 0.618) % 1.0)          # how far down the cone from the mouth
                r, h = 0.16 + (0.3 - 0.16) * t * 1.6, 0.26 - (0.26 - 0.04) * t * 1.6
                G.sphere(f'crust{k}', 0.035 + 0.012 * (k % 3), at=(r * math.cos(a), r * math.sin(a), h), material=M.crust,
                         scale=(1.6, 0.9, 0.22), segments=8, rings=5, rot=(0, 63.4, math.degrees(a)))
    drum = G.join(drum, 'drum')
    drum.rotation_euler[0] = math.radians(CM_TILT)
    drum.location = F.at(0, DY, DZ)
    with G.new_parts():
        # the yoke the drum turns in, on a post, the gearbox under the drum, the tilting wheel on its right
        F.box('gearbox', 0.24, 0.16, 0.22, (0, DY - 0.2, DZ - 0.14), M.frame, bevel=low(0.02, 0), bevel_segments=1, rot=(CM_TILT, 0, 0))
        for s in (-1, 1):
            F.box(f'yoke{s}', 0.04, 0.38, 0.06, (s * 0.33, DY - 0.17, DZ - 0.1), M.frame, bevel=low(0.006, 0), bevel_segments=1)
            F.cyl(f'pivot{s}', 0.035, 0.05, (s * 0.33, DY - 0.08, DZ - 0.1), M.steel, rot=(0, 90, 0), verts=low(12, 6))
        F.box('yoke_base', 0.7, 0.05, 0.08, (0, DY - 0.36, DZ - 0.1), M.frame, bevel=low(0.006, 0), bevel_segments=1)
        F.cyl('post', 0.04, 0.3, (0, 0.38, -0.1), M.frame, verts=low(12, 6))
        G.torus('handwheel', 0.12, 0.012, at=F.at(-0.38, DY - 0.08, DZ - 0.1), material=M.frame, rot=(0, 90, 0),
                major_segments=low(16, 8), minor_segments=4)
        for k in range(low(3, 0)):
            a = math.tau * k / 3
            F.box(f'spoke{k}', 0.012, 0.24, 0.012, (-0.38, DY - 0.08, DZ - 0.1), M.frame, rot=(math.degrees(a), 0, 0))
        F.cyl('wheel_knob', 0.016, 0.07, (-0.41, DY - 0.08 + 0.12, DZ - 0.1), M.plastic, rot=(0, 90, 0), verts=6)
        # the motor under its cover, behind the post, its vents
        F.box('motor', 0.34, 0.24, 0.26, (0, 0.42, -0.3), M.body, bevel=low(0.03, 0), bevel_segments=low(2, 1))
        if not is_low():
            for k in range(4):
                F.box(f'vent{k}', 0.004, 0.012, 0.16, (-0.172, 0.36 + k * 0.03, -0.3), M.dark)
            F.box('switch', 0.06, 0.08, 0.04, (0.12, 0.47, -0.16), M.plastic)
            F.box('switch_red', 0.03, 0.02, 0.012, (0.12, 0.49, -0.135), M.red_lens)
            G.kit.tube(F, 'lead', [(0.12, 0.43, -0.16), (0.2, 0.3, -0.25), (0.25, 0.1, -0.35), (0.3, 0.012, -0.4)], 0.007,
                       M.plastic, verts=4, round=2)
        # the stand: a tube frame, two wheels at the back, two legs at the front, a handle to wheel it by
        for s in (-1, 1):
            G.kit.tube(F, f'frame{s}', [(s * 0.22, 0.03, 0.4), (s * 0.22, 0.25, 0.1), (s * 0.22, 0.25, -0.38)], 0.018, M.frame,
                       round=low(2, 1))
            F.box(f'foot{s}', 0.07, 0.03, 0.09, (s * 0.22, 0.015, 0.41), M.rubber)
            F.box(f'axle_bracket{s}', 0.03, 0.12, 0.05, (s * 0.22, 0.2, -0.36), M.frame)
            _tyre(f'wheel{s}', s * 0.29, 0.15, -0.36, 0.15, 0.07, M.rubber, grooves=None, segments=low(20, 10))
            F.cyl(f'hub{s}', 0.095, 0.06, (s * 0.29, 0.15, -0.36), M.yellow, rot=(0, 90, 0), verts=low(14, 6))
        F.cyl('axle', 0.016, 0.62, (0, 0.15, -0.36), M.steel, rot=(0, 90, 0), verts=6)
        F.box('cross', 0.44, 0.04, 0.04, (0, 0.25, -0.1), M.frame)
        G.kit.tube(F, 'tow', [(0.22, 0.25, -0.38), (0.15, 0.45, -0.46), (-0.15, 0.45, -0.46), (-0.22, 0.25, -0.38)], 0.016,
                   M.frame, round=low(2, 1))
    G.join_all('cementMixer_far' if is_low() else 'cementMixer')
    G.settle()
    return G.meshes()


# --- the flagpole: 1x1 outdoors, a white pole with the town's flag at the top ---------------------------------
FP_H = 6.0        # the pole's top, its truck and ball above it
FP_FLAG = (1.5, 0.9)   # the flag: its fly (length) and hoist (height)


def _flag_path(n):
    """The flag's middle line from the pole out to its fly: rippling in the depth, falling a little."""
    L = FP_FLAG[0]
    return [(0.05 + L * i / n, FP_H - 0.12 - FP_FLAG[1] / 2 - 0.04 * (i / n) ** 2,
             0.07 * math.sin(math.tau * 1.3 * i / n) * (i / n) ** 0.7) for i in range(n + 1)]


def flagPole():
    M = _palette(body=('#F1F0EC', 'gloss'),     # the white pole
                 flag=('#2E5FA8', 'fabric'),    # the flag's field
                 flag2=('#F2C14E', 'fabric'),   # its gold band
                 flag3=('#F7F5EE', 'fabric'),   # its white canton
                 concrete=('#A9A59C', 'matte'), rope=('#E6E2D6', 'matte'))
    with G.new_parts() as still:
        # the base: a concrete pad, a steel flange bolted to it, the pole tapering up to its truck and gilt ball
        F.box('pad', 0.6, 0.06, 0.6, (0, 0.03, 0), M.concrete, bevel=low(0.01, 0), bevel_segments=1)
        G.revolve('flange', [(0, 0.06), (0.12, 0.06), (0.12, 0.075), (0.07, 0.09), (0.07, 0.2), (0.0, 0.2)], at=F.at(0, 0, 0),
                  material=M.body, segments=low(24, 8))
        if not is_low():
            for k in range(4):
                a = math.tau * (k + 0.5) / 4
                F.cyl(f'bolt{k}', 0.012, 0.024, (0.095 * math.cos(a), 0.08, 0.095 * math.sin(a)), M.steel, verts=6)
        F.cone('pole', 0.055, 0.03, FP_H - 0.2, (0, 0.2 + (FP_H - 0.2) / 2, 0), M.body, verts=low(16, 8))
        F.cyl('truck', 0.045, 0.06, (0, FP_H + 0.03, 0), M.body, verts=low(16, 8))
        F.sphere('ball', 0.06, (0, FP_H + 0.11, 0), M.brass, segments=low(16, 8), rings=low(10, 5))
        # the halyard: down from the truck's pulley to a cleat at hand height, its two strands
        F.box('cleat', 0.025, 0.14, 0.03, (0.0, 1.3, -0.065), M.steel, bevel=low(0.004, 0), bevel_segments=1)
        for k, x in enumerate((-0.012, 0.012)):
            F.cyl(f'halyard{k}', 0.0035, FP_H - 1.3, (x, 1.3 + (FP_H - 1.3) / 2, -0.06), M.rope, verts=4)
        for j, y in enumerate((FP_H - 0.12, FP_H - 0.12 - FP_FLAG[1])):   # the flag's clips on the halyard
            F.box(f'clip{j}', 0.04, 0.03, 0.02, (0.03, y, -0.0), M.steel)
    G.join(still, 'flagPole_far' if is_low() else 'flagPole', origin=(0, 0, 0))
    with G.new_parts() as flag:
        n = low(24, 8)
        path = _flag_path(n)
        hh, t = FP_FLAG[1] / 2, 0.006
        F.sweep('flag', path, None, M.flag, profile=[(-hh, -t / 2), (hh, -t / 2), (hh, t / 2), (-hh, t / 2)], up=(0, 0, 1))
        F.sweep('flag_band', path, None, M.flag2, profile=[(-0.09, -t / 2 - 0.002), (0.09, -t / 2 - 0.002),
                                                           (0.09, t / 2 + 0.002), (-0.09, t / 2 + 0.002)], up=(0, 0, 1))
        k = n * 4 // 10
        F.sweep('flag_canton', path[:k + 1], None, M.flag3, profile=[(0.13, -t / 2 - 0.003), (hh - 0.05, -t / 2 - 0.003),
                                                                     (hh - 0.05, t / 2 + 0.003), (0.13, t / 2 + 0.003)],
                up=(0, 0, 1))
    if is_low():
        G.join_all('flagPole_far')
        G.settle(centre=False)
        return G.meshes()
    flag_part = G.join(flag, 'flag', origin=F.at(0, FP_H - 0.6, 0))
    G.settle(centre=False)
    G.rock(flag_part, 'flutter', 7, seconds=3.4, axis='Z')
    return G.meshes()


FAMILY = G.Family(
    'civic_works', PALETTES,
    [G.Model('fireEngine', fireEngine, size=(2.51, 3.17, 5.843), key='model.fireEngine', far=G.far_of(fireEngine),
             paint=[('body', 'Paint'), ('shutter', 'Lockers'), ('stripe', 'Stripes'), ('ladder', 'Ladders')]),
     G.Model('carLift', carLift, size=(2.94, 2.76, 4.09), key='model.carLift', far=G.far_of(carLift),
             paint=[('body', 'Lift'), ('car', 'Car'), ('alloy', 'Wheels')]),
     G.Model('printingPress', printingPress, size=(1.72, 1.51, 1.864), key='model.printingPress',
             motions={'loops': {'watch': 'rollers'}},
             paint=[('body', 'Press')]),
     G.Model('firePole', firePole, size=(0.84, 2.8, 0.84), key='model.firePole', paint=[('body', 'Pole'), ('mat', 'Mat')]),
     G.Model('hoseRack', hoseRack, size=(0.88, 0.786, 0.3), key='model.hoseRack', centre=(0, -0.28), floor=False,
             paint=[('body', 'Rack'), ('hose', 'Hose'), ('hose2', 'Second hose')]),
     G.Model('serverRack', serverRack, size=(0.6, 2.003, 0.889), key='model.serverRack', centre=(0, 0.0145),
             motions={'idle': 'blink'}, paint=[('body', 'Cabinet'), ('unit', 'Servers')]),
     G.Model('tyreRack', tyreRack, size=(1.96, 2.0, 0.66), key='model.tyreRack', centre=(0, -0.1),
             paint=[('body', 'Frame'), ('beam', 'Beams')]),
     G.Model('toolChest', toolChest, size=(0.79, 1.429, 0.661), key='model.toolChest', centre=(-0.0132, -0.0995),
             paint=[('body', 'Chest')]),
     G.Model('timberStack', timberStack, size=(1.86, 0.598, 0.931), key='model.timberStack',
             far=G.far_of(timberStack), paint=[('body', 'Tarpaulin'), ('timber', 'Timber')]),
     G.Model('cementMixer', cementMixer, size=(0.8, 1.163, 0.965), key='model.cementMixer',
             far=G.far_of(cementMixer), paint=[('body', 'Mixer'), ('frame', 'Stand')]),
     G.Model('flagPole', flagPole, size=(1.874, 6.17, 0.6), key='model.flagPole', centre=(0.6368, 0),
             far=G.far_of(flagPole), motions={'idle': 'flutter'}, paint=[('body', 'Pole'), ('flag', 'Flag')])],
    comment='Blender-built big working things for the town\'s named places (tools/art/blender/families/civic_works.py; '
            'content/civicWorks.json): the fire engine, fire pole and hose rack, the two-post car lift, tyre rack and '
            'tool chest, the printing press, the server rack, the timber stack, cement mixer and flagpole. Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The fire engine, the car lift and the outdoor things have far versions for the town '
            'overview. The press\'s plate and blanket cylinders turn while a resident watches it run (`rollers`); the '
            'server rack\'s LEDs blink (`blink`) and the flag sways at the top of its pole (`flutter`), both ambient; '
            'the server rack\'s LEDs and its UPS\'s screen glow at night.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
