# The living room's media (L4, notes: media.md): the television on its media unit, the flat-screen TV on the
# wall, the portable TV, the low TV bench with a TV lift, the home cinema, the stereo, the boombox and the
# electric fireplace.
#
# Each is built where the game places it (`F`, game axes: x, height, depth; front +depth), its back on the
# wall face (-0.43) where the old part's reached past it, then stood on the floor (`G.settle(centre=False)`,
# its `centre=` on the Model). Every screen faces the front (+depth), where the game stands its watchers, and
# glows at night (`glow`); the home cinema's projector screen is rolled away at rest, so it glows only while it
# is down, while watched.
#
# What moves:
#   tvRetro            the rabbit ears turn 20° and extend (`antenna`, for watch);
#   tvStandLow         the TV rises 60 cm out of the bench's lift (`tv_up`, for watch, back down at the end);
#   homeCinema         the projector screen unrolls from its case on the wall (`screen_down`, watch and film);
#   stereo             the record turns on the turntable (`record`, a loop) and the woofers pulse (`pulse`, `every`);
#   boombox            the cassette door tips open and shuts (`cassette`, hold 1.2 s);
#   fireplaceElectric  the flames grow from the log bed (`flames`, for relax; gloss, glowing at night).
#
# Run:  cargo art media [--only tvRetro] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

SCREEN_GLOW = '#8DB4E2'   # an LCD's cool light at night

M = G.Palette(
    # shared
    screen=('#16202B', 'gloss', {'glow': SCREEN_GLOW}),   # every flat screen, dark by day
    black_metal=('#2B2C2E', 'metal'),                    # feet, pulls, brackets
    steel=('#C3C7CB', 'metal'),                          # knobs, tonearm, ears
    grille=('#2E3034', 'matte'),                         # speaker cloth
    led=('#5EC28B', 'gloss'),
    # the television on its media unit (the old `tv_set`)
    tv_cabinet=('#B08E66', 'wood'),     # the unit; Body
    tv_oak=('#CDB08A', 'wood'),         # its top board and doors; Trim
    tv_frame=('#2A2C30', 'satin'),      # the set's casing and stand
    soundbar=('#3A3D41', 'satin'),
    bowl=('#F2F2F0', 'gloss'),
    pebble=('#E9E6E0', 'gloss'),
    # the flat-screen TV on the wall
    wall_frame=('#2A2C30', 'satin'),    # Body
    wall_bar=('#3A3D41', 'satin'),      # the soundbar; Trim
    # the portable TV on its cabinet
    retro_wood=('#A8754A', 'wood'),     # Body
    retro_bezel=('#2A2C30', 'gloss'),   # the screen's surround and the ears' foot; Trim
    crt=('#22303E', 'gloss', {'glow': '#A8C8E0'}),
    # the low TV bench
    bench=('#F1F0EC', 'satin'),         # Bench (and the lift's lid, on the TV)
    bench_oak=('#CDB08A', 'wood'),      # top and legs
    bench_tv=('#1E1F22', 'satin'),      # TV
    plant=('#5E8F4E', 'satin'),
    pot=('#E8E6E1', 'gloss'),
    # the home cinema
    cinema_unit=('#B08E66', 'wood'),    # Body
    cinema_oak=('#CDB08A', 'wood'),     # top and door fronts; Trim
    tower=('#3A3D41', 'satin'),         # the tower speakers, the soundbar, the screen's case
    cone=('#22252A', 'satin'),
    projector=('#E6E6E3', 'satin'),
    lens=('#1A1C20', 'gloss'),
    projection=('#F4F4F1', 'matte', {'glow': '#DCE6F5'}),   # the pulled-down screen, lit while watched at night
    screen_border=('#1E1F22', 'matte'),
    # the stereo
    stereo_wood=('#CDB08A', 'wood'),    # Body
    stereo_dark=('#3A3D41', 'satin'),   # the amplifier, the CD player, the baffles; Trim
    stereo_inside=('#2A2620', 'matte'),
    display=('#7FC8E8', 'gloss'),
    disc=('#D9DDE2', 'chrome'),
    vinyl=('#141414', 'satin'),
    label=('#B5653F', 'satin'),
    rec_rust=('#B5653F', 'satin'),
    rec_mustard=('#D6A64A', 'satin'),
    rec_teal=('#4E7A7F', 'satin'),
    rec_cream=('#EFE5D3', 'satin'),
    rec_navy=('#3E5470', 'satin'),
    rec_clay=('#C98F6B', 'satin'),
    rec_black=('#2A2C30', 'satin'),
    # the boombox on its crate
    boombox_body=('#3A3D41', 'satin'),  # Body
    boombox_trim=('#C9CBCC', 'metal'),  # Trim: the top, rings, handle and aerial
    crate=('#A8754A', 'wood'),
    crate_dark=('#9E6E43', 'wood'),
    cassette=('#1D2228', 'gloss'),
    window=('#3A4450', 'gloss'),
    # the electric fireplace
    surround=('#F4F3EF', 'satin'),      # Surround
    firebox=('#16181B', 'gloss'),       # the firebox's back and its black frame; Firebox
    mantel=('#CDB08A', 'wood'),
    logs=('#4A3B30', 'matte'),
    ember=('#E0672E', 'gloss', {'glow': '#FF8A3C'}),
    flame=('#FF9A3C', 'gloss', {'glow': '#FFB25C'}),
    flame_core=('#FFD27A', 'gloss', {'glow': '#FFE0A0'}),
    candle=('#EFE5D3', 'gloss'),
    hearth_cushion=('#B5653F', 'matte'),   # the floor cushion residents warm up on (characters.ts SEATS)
    hearth_piping=('#9A5433', 'matte'),
    picture=('#2B2C2E', 'satin'),
    art_print=('#C9B79C', 'matte'),
    print_sky=('#9DC6CF', 'matte'),
    print_hill=('#7FA86A', 'matte'),
    candle_wax=('#F4EBDC', 'satin'),
    firebox_line=('#24272B', 'gloss'),   # the flutes on the firebox's back
    grille_dark=('#1F2124', 'matte'),    # ribs on speaker cloth
    led_red=('#D9483B', 'gloss'),
    steel_dark=('#55595E', 'metal'),     # the amplifier's brushed face
    glass_lens=('#2E4A66', 'gloss'),
)

F = G.GameFrame()
WALL = -0.43


def _front_cyl(name, r, depth, x, y, z, mat, verts=20, **kw):
    """A disc facing the front (its axis along the game's depth), its face at depth `z` + depth/2."""
    return F.cyl(name, r, depth, (x, y, z), mat, rot=(90, 0, 0), verts=verts, **kw)


def _rod(name, a, b, r, mat, verts=8):
    """A rod between two game points."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=mat, verts=verts)


def _axle(gx, gz):
    """The rot for a cylinder whose axis runs along the game direction (gx, 0, gz)."""
    return (0, 90, math.degrees(math.atan2(-gz, -gx)))


def _rbox(name, w, h, d, at, mat, r=0.004, segs=1, **kw):
    """A box with its edges chamfered (`segs` 1) or rounded (2 or more)."""
    return F.box(name, w, h, d, at, mat, bevel=r, bevel_segments=segs, **kw)


def _capsule(name, x0, x1, y, z, h, d, mat, r=None):
    """A bar along x from x0 to x1, its section an h x d rounded rectangle (a soundbar, a rail, a case)."""
    return F.sweep(name, [(x0, y, z), (x1, y, z)], None, mat, profile=G.rounded_rect(h, d, r or min(h, d) * 0.45, corner=3),
                   up=(0, 0, 1), angle=50)


def _grille(name, x, y, z, w, h, mat, rows=None, face=1):
    """A speaker's cloth face: a thin panel on the front at depth `z`, with fine horizontal ribs."""
    _rbox(name, w, h, 0.003, (x, y, z), mat, r=0.001)
    for k in range(rows or 0):
        F.box(f'{name}_rib{k}', w - 0.006, 0.0015, 0.001, (x, y - h / 2 + h * (k + 0.5) / rows, z + face * 0.002), M.grille_dark)


def _driver(name, x, y, z, r, cone=None, ring=None, verts=20):
    """A loudspeaker driver facing the front from depth `z`: a rim, a cone sunk into it and a dust cap."""
    G.revolve(name, [(0, -0.002), (r, -0.002), (r, 0.003), (r * 0.9, 0.004), (r * 0.82, 0.0), (r * 0.3, -r * 0.35),
                     (r * 0.28, -r * 0.3), (r * 0.22, -r * 0.18), (0, -r * 0.14)],
              at=F.at(x, y, z), material=cone or M.cone, segments=verts, rot=(90, 0, 0))
    if ring:
        F.sweep(f'{name}_ring', [(x + (r + 0.003) * G.cosd(a), y + (r + 0.003) * G.sind(a), z + 0.003)
                                 for a in range(0, 360, 360 // verts)], 0.0035, ring, verts=5, closed=True)


def _flat_screen(prefix, x, y, z, w, h, frame, depth=0.025, back=None, chin=0.0):
    """A flat panel facing the front at depth `z` (its middle): a slim rounded bezel, the screen set a hair in, a deeper
    rounded hump behind (`back`: its w, h), a logo under the screen."""
    F.soft(f'{prefix}panel', w, h, depth, (x, y, z), frame, round=0.006, round_segments=2)
    F.box(f'{prefix}screen', w - 0.022, h - 0.022 - chin, 0.002, (x, y + chin / 2, z + depth / 2 + 0.0005), M.screen)
    F.box(f'{prefix}logo', 0.04, 0.005, 0.001, (x, y - h / 2 + 0.006 + chin / 3, z + depth / 2 + 0.0004), M.steel)
    if back:
        _rbox(f'{prefix}back', back[0], back[1], 0.03, (x, y - 0.02, z - depth / 2 - 0.015 + 0.004), frame, r=0.012, segs=2)
        for k in range(5):   # vents across the hump's back
            F.box(f'{prefix}vent{k}', back[0] * 0.7, 0.003, 0.002, (x, y - 0.02 + back[1] / 2 - 0.03 - 0.012 * k,
                                                                    z - depth / 2 - 0.0305 + 0.004), M.grille)


# --- the television (1x1): a flat set on its stand on a low oak media unit (the old `tv_set`) -------------
def tv():
    # the unit: a carcass on four turned feet, an oak top board, two doors with slim pulls, a reveal round them
    F.soft('unit', 0.9, 0.4, 0.42, (0, 0.24, -0.22), M.tv_cabinet, round=0.012, round_segments=2)
    F.soft('top', 0.91, 0.03, 0.43, (0, 0.455, -0.215), M.tv_oak, round=0.009, round_segments=2)
    F.box('reveal', 0.87, 0.31, 0.002, (0, 0.24, -0.0095), M.grille)
    for s in (-1, 1):
        G.kit.shaker(F, f'door{s}', 0.43, 0.3, (s * 0.2195, 0.24, -0.004), 0.012, M.tv_oak, rail=0.045, recess=0.003)
        G.kit.pull(F, f'pull{s}', s * 0.03, 0.24, 0.002, 0.12, M.black_metal, r=0.004, out=0.006, upright=True)
        for z in (-0.4, -0.04):
            G.revolve(f'foot{s}{z}', [(0, 0), (0.011, 0), (0.013, 0.004), (0.016, 0.034), (0.019, 0.04), (0, 0.04)],
                      at=F.at(s * 0.42, 0, z), material=M.black_metal, segments=10)
    # the set: a slim panel with a deeper back, on a swept neck and an oval foot
    _flat_screen('', 0, 0.82, -0.29, 0.86, 0.5, M.tv_frame, back=(0.6, 0.32))
    F.sweep('neck', [(0, 0.66, -0.318), (0, 0.55, -0.325), (0, 0.484, -0.31)], None, M.tv_frame,
            profile=G.rounded_rect(0.018, 0.06, 0.007, corner=1), up=(0, 1, 0), resolution=3, angle=60)
    F.extrude('foot', G.rounded_rect(0.26, 0.16, 0.07, corner=4), 0.012, (0, 0.476, -0.3), M.tv_frame, plane='top',
              bevel=0.004, bevel_segments=1)
    # a soundbar in front of it, a bowl of pebbles on two books beside, the remote
    _capsule('soundbar', -0.25, 0.25, 0.495, -0.12, 0.05, 0.07, M.soundbar, r=0.01)
    _grille('soundbar_grille', 0, 0.495, -0.0855, 0.46, 0.03, M.grille, rows=4)
    F.box('soundbar_led', 0.012, 0.003, 0.002, (0.2, 0.476, -0.0855), M.led)
    for k, (m, t, turn) in enumerate(((M.retro_bezel, 0.022, 4), (M.label, 0.018, -6))):
        _rbox(f'book{k}', 0.2, t, 0.15, (0.33, 0.47 + 0.022 * k + t / 2, -0.115), m, r=0.002, rot=(0, 0, turn))
    G.revolve('bowl', [(0, 0), (0.05, 0), (0.065, 0.02), (0.06, 0.035), (0.05, 0.02), (0, 0.012)],
              at=F.at(0.33, 0.51, -0.11), material=M.bowl, segments=20)
    for i, (x, z) in enumerate(((0.32, -0.1), (0.345, -0.09), (0.33, -0.125), (0.31, -0.118))):
        p = F.sphere(f'pebble{i}', 0.02, (x, 0.536 + 0.004 * (i % 2), z), M.pebble, scale=(1.1, 0.9, 0.55), segments=10,
                     rings=6)
        G.jitter(p, 0.003, size=0.02, seed=i)
    _rbox('remote', 0.04, 0.014, 0.15, (-0.33, 0.477, -0.08), M.tv_frame, r=0.006, segs=2, rot=(0, 0, 14))
    for k in range(3):
        dz = -0.04 + 0.025 * k
        F.box(f'remote_key{k}', 0.02, 0.002, 0.012, (-0.33 - dz * G.sind(14), 0.4845, -0.08 + dz * G.cosd(14)),
              M.steel if k else M.led_red, rot=(0, 0, 14))
    G.join_all('tv')
    G.settle(centre=False)


# --- the flat-screen TV (2x1, wall): a thin panel on a bracket, a soundbar under it (the old tv_wall) ------
def tv_wall():
    y = 1.28
    # the wall bracket: a plate on the wall and two arms out to the set's back
    _rbox('bracket', 0.3, 0.2, 0.008, (0, y - 0.02, WALL + 0.004), M.black_metal, r=0.002)
    for s in (-1, 1):
        _rbox(f'bracket_arm{s}', 0.025, 0.36, 0.012, (s * 0.1, y - 0.02, WALL + 0.012), M.black_metal, r=0.003)
    _rbox('back', 0.8, 0.45, 0.022, (0, y - 0.02, WALL + 0.019), M.wall_frame, r=0.008, segs=2)
    F.soft('panel', 1.24, 0.72, 0.022, (0, y, WALL + 0.041), M.wall_frame, round=0.005, round_segments=2)
    F.box('screen', 1.215, 0.695, 0.002, (0, y, WALL + 0.0525), M.screen)
    F.box('logo', 0.05, 0.008, 0.002, (0, y - 0.352, WALL + 0.0525), M.steel)
    F.box('led', 0.008, 0.003, 0.002, (0.5, y - 0.352, WALL + 0.0525), M.led_red)
    # the cable down the wall to the soundbar, in a slim cover
    _rbox('cable_cover', 0.03, 0.12, 0.012, (0.22, 0.88, WALL + 0.006), M.wall_frame, r=0.003)
    # the soundbar on its own two brackets, its cloth front ribbed
    for s in (-1, 1):
        _rbox(f'bar_bracket{s}', 0.03, 0.05, 0.03, (s * 0.3, 0.8, WALL + 0.015), M.black_metal, r=0.003)
    _capsule('soundbar', -0.45, 0.45, 0.8, WALL + 0.065, 0.06, 0.07, M.wall_bar, r=0.012)
    _grille('soundbar_grille', 0, 0.8, WALL + 0.0995, 0.84, 0.036, M.grille, rows=5)
    F.box('soundbar_led', 0.01, 0.003, 0.0015, (0.38, 0.775, WALL + 0.1), M.led)
    G.join_all('tvWall')
    G.settle(centre=False, floor=False)


# --- the portable TV (1x1): a bulging CRT in a honey-wood cabinet on splayed legs, rabbit ears on top --------
RETRO_Z = -0.205   # the cabinet's middle depth: its back on the wall face
EARS = (0.24, 0.875, RETRO_Z - 0.06)


def tv_retro():
    z = RETRO_Z
    with G.new_parts() as still:
        F.soft('cabinet', 0.92, 0.62, 0.44, (0, 0.5, z), M.retro_wood, round=0.035, round_segments=3)
        F.soft('top', 0.94, 0.03, 0.46, (0, 0.825, z), M.retro_wood, round=0.012, round_segments=2)
        # the screen: a rounded bezel round a domed tube
        hole = G.rounded_rect(0.47, 0.36, 0.05, corner=4)
        F.extrude('bezel', G.rounded_rect(0.56, 0.46, 0.07, corner=4), 0.02, (-0.14, 0.52, z + 0.225), M.retro_bezel,
                  holes=[hole], bevel=0.006, bevel_segments=2)
        bx, by, bz = F.at(-0.14, 0.52, z + 0.215)
        secs = []
        for t, k in ((0.0, 1.0), (0.015, 1.0), (0.028, 0.96), (0.038, 0.86), (0.044, 0.68), (0.048, 0.3)):
            secs.append([(bx + p[0], by - t, bz + p[1]) for p in
                         ((x * k, yv * k) for x, yv, _ in G.ring(0.238, 0.183, 0, 28, square=4.0))])
        G.loft('tube', secs, material=M.crt)
        # the speaker: cloth behind wooden slats, a channel dial and a volume knob above it
        F.box('cloth', 0.26, 0.3, 0.01, (0.3, 0.44, z + 0.222), M.grille)
        for i in range(7):
            _rbox(f'slat{i}', 0.26, 0.014, 0.012, (0.3, 0.315 + i * 0.042, z + 0.229), M.retro_wood, r=0.003)
        for x, n in ((0.24, 12), (0.36, 0)):
            G.revolve(f'knob{x}', [(0, 0), (0.03, 0), (0.03, 0.006), (0.026, 0.012), (0.024, 0.028), (0.02, 0.032),
                                   (0, 0.033)], at=F.at(x, 0.7, z + 0.22), material=M.steel, segments=16, rot=(90, 0, 0))
            F.box(f'knob_mark{x}', 0.005, 0.022, 0.003, (x, 0.71, z + 0.254), M.retro_bezel)
            for k in range(n):   # the channel numbers round the dial
                a = math.tau * k / n
                F.box(f'dial{x}{k}', 0.004, 0.004, 0.001, (x + 0.042 * math.cos(a), 0.7 + 0.042 * math.sin(a), z + 0.2205),
                      M.retro_bezel)
        F.box('brand', 0.08, 0.012, 0.002, (0.3, 0.61, z + 0.2205), M.steel)
        for x in (-1, 1):
            for s in (-1, 1):
                # a tapered splayed leg in a brass ferrule
                a, b = (x * 0.39, 0.205, z + s * 0.15), (x * 0.42, 0.0, z + s * 0.17)
                F.sweep(f'leg{x}{s}', [a, b], 0.02, M.retro_wood, verts=10, scale=[1.0, 0.6])
                F.sweep(f'ferrule{x}{s}', [tuple(p + (q - p) * 0.82 for p, q in zip(a, b)), b], 0.0125, M.steel, verts=10,
                        scale=[1.0, 0.95])
        # the ears' foot
        F.sphere('ears_foot', 0.05, (EARS[0], 0.86, EARS[2]), M.retro_bezel, scale=(1, 1, 0.5), segments=16, rings=8)
    G.join(still, 'tvRetro', origin=(0, 0, 0))
    # the rabbit ears: two telescopic rods from the foot, a part of their own turning and extending about it
    with G.new_parts() as ears:
        for name, tip in (('ear_left', (0.1, 1.12, z - 0.12)), ('ear_right', (0.4, 1.1, z - 0.1))):
            pts = [tuple(a + (b - a) * t for a, b in zip(EARS, tip)) for t in (0.0, 0.4, 0.401, 0.7, 0.701, 1.0)]
            F.sweep(name, pts, 0.006, M.steel, verts=8, scale=[1.0, 1.0, 0.8, 0.8, 0.6, 0.6])
            F.sphere(name + '_ball', 0.008, tip, M.steel, segments=8, rings=5)
        F.cyl('ears_pivot', 0.012, 0.02, (EARS[0], EARS[1] + 0.006, EARS[2]), M.steel, verts=12)
    ears = G.join(ears, 'ears', origin=F.at(*EARS))
    G.settle(centre=False)
    G.swing(ears, 'antenna', 20, seconds=0.9)
    G.squash(ears, 'antenna', (1, 1, 1.6), seconds=0.9)


# --- the low TV bench (2x1): a white bench on oak legs, a TV lift in a hood at its back -----------------------
HOOD_W, HOOD_Z0, HOOD_Z1, HOOD_TOP = 1.3, WALL, -0.31, 0.8
LIFT = 0.6


def _plant(name, x, y, z, r, h, pot, leaf, n=9, length=0.12):
    """A pot of leaves: a rimmed pot `r` round and `h` high on `y`, leaves arching out of it."""
    G.revolve(f'{name}_pot', [(0, 0), (r * 0.78, 0), (r * 0.95, h * 0.85), (r * 1.06, h * 0.87), (r * 1.06, h),
                              (r * 0.92, h), (r * 0.9, h * 0.9), (0, h * 0.9)], at=F.at(x, y, z), material=pot, segments=18)
    for k in range(n):
        a = math.tau * k / n + 0.35 * math.sin(k * 2.3)
        c, s = math.cos(a), math.sin(a)
        L = length * (0.75 + 0.25 * math.sin(k * 1.7) ** 2)
        lift = 0.8 + 0.2 * math.cos(k * 1.3)
        y0 = y + h * 0.9
        pts = [(x + c * 0.01, y0, z + s * 0.01), (x + c * L * 0.3, y0 + L * lift * 0.75, z + s * L * 0.3),
               (x + c * L * 0.6, y0 + L * lift, z + s * L * 0.6), (x + c * L * 0.9, y0 + L * lift * 0.8, z + s * L * 0.9)]
        F.sweep(f'{name}_leaf{k}', pts, None, leaf, profile=[(0.0, -0.018), (0.003, 0.0), (0.0, 0.018), (-0.0015, 0.0)],
                up=(0, 0, 1), resolution=2, scale=[0.4, 1.0, 0.8, 0.15])


def tv_stand_low():
    with G.new_parts() as still:
        for x in (-0.84, 0.84):
            for z in (-0.4, -0.04):
                F.sweep(f'leg{x}{z}', [(x, 0.06, z), (x + math.copysign(0.006, x), 0.0, z + math.copysign(0.006, z + 0.22))],
                        0.02, M.bench_oak, verts=10, scale=[1.0, 0.75])
        F.soft('carcass', 1.8, 0.4, 0.43, (0, 0.26, -0.215), M.bench, round=0.01, round_segments=2)
        F.box('reveal', 1.78, 0.35, 0.002, (0, 0.26, 0.0005), M.grille)
        # the top: in front of the hood and either side of it
        F.soft('top', 1.82, 0.03, 0.34, (0, 0.475, (HOOD_Z1 + 0.03) / 2), M.bench_oak, round=0.008, round_segments=2)
        side = (1.82 - HOOD_W) / 2
        for s in (-1, 1):
            F.soft(f'top_back{s}', side, 0.03, HOOD_Z1 - WALL, (s * (HOOD_W + side) / 2, 0.475, (WALL + HOOD_Z1) / 2),
                   M.bench_oak, round=0.006, round_segments=2)
        # the hood the TV sinks into: four walls, the TV's lid closing its top
        h = HOOD_TOP - 0.46
        yc = 0.46 + h / 2
        _rbox('hood_back', HOOD_W, h, 0.012, (0, yc, WALL + 0.006), M.bench, r=0.003)
        _rbox('hood_front', HOOD_W, h, 0.012, (0, yc, HOOD_Z1 - 0.006), M.bench, r=0.003)
        for s in (-1, 1):
            _rbox(f'hood_side{s}', 0.012, h, HOOD_Z1 - WALL, (s * (HOOD_W / 2 - 0.006), yc, (WALL + HOOD_Z1) / 2), M.bench,
                  r=0.003)
        # three doors, each framed round a panel, with black bar pulls
        for x in (-0.6, 0.0, 0.6):
            G.kit.shaker(F, f'door{x}', 0.58, 0.34, (x, 0.26, 0.01), 0.02, M.bench, rail=0.05, recess=0.004)
            G.kit.pull(F, f'pull{x}', x, 0.4, 0.02, 0.12, M.black_metal, r=0.005, out=0.009)
        # a plant on the left, a remote and a game controller on the right
        _plant('plant', -0.78, 0.49, -0.1, 0.06, 0.1, M.pot, M.plant, n=10, length=0.13)
        _rbox('remote', 0.04, 0.015, 0.16, (0.72, 0.4975, -0.1), M.bench_tv, r=0.006, segs=2, rot=(0, 0, -12))
        for s in (-1, 1):
            F.sphere(f'pad_grip{s}', 0.03, (0.5 + s * 0.045, 0.505, -0.14), M.bench_tv, scale=(1, 1.2, 0.6), segments=10,
                     rings=6)
        _rbox('pad', 0.1, 0.022, 0.06, (0.5, 0.501, -0.15), M.bench_tv, r=0.01, segs=2)
        for k in range(4):
            F.cyl(f'pad_button{k}', 0.004, 0.004, (0.52 + 0.009 * (k % 2), 0.512, -0.155 + 0.009 * (k // 2)),
                  (M.led, M.led_red, M.steel, M.steel)[k], verts=8)
    G.join(still, 'tvStandLow', origin=(0, 0, 0))
    # the TV, down in the bench at rest, its lid flush with the hood's top
    zc = (WALL + HOOD_Z1) / 2
    with G.new_parts() as tv:
        th, y0 = 0.7, HOOD_TOP - 0.02 - 0.7
        F.soft('tv_panel', 1.22, th, 0.035, (0, y0 + th / 2, zc), M.bench_tv, round=0.006, round_segments=2)
        F.box('tv_screen', 1.19, th - 0.03, 0.002, (0, y0 + th / 2 + 0.005, zc + 0.018), M.screen)
        F.box('tv_logo', 0.05, 0.006, 0.001, (0, y0 + 0.008, zc + 0.018), M.steel)
        _rbox('tv_lid', HOOD_W - 0.03, 0.02, HOOD_Z1 - WALL - 0.026, (0, HOOD_TOP - 0.01, zc), M.bench, r=0.004, segs=2)
        F.box('tv_lid_grip', 0.2, 0.002, 0.03, (0, HOOD_TOP + 0.0005, HOOD_Z1 - 0.03), M.bench_tv)
    tv = G.join(tv, 'tv')
    G.settle(centre=False)
    G.slide(tv, 'tv_up', (0, 0, LIFT), seconds=1.6)


# --- the home cinema (2x1): an oak media unit, tower speakers, a soundbar and a short-throw projector on it,
# and a projector screen rolled up in its case on the wall above ------------------------------------------------
CASE_Y = 1.6
SCREEN_W, SCREEN_H = 1.54, 0.92


def home_cinema():
    with G.new_parts() as still:
        for s in (-1, 1):
            F.sweep(f'sled{s}', G.kit.fillet([(s * 0.86, 0.04, -0.405), (s * 0.86, 0.02, -0.38), (s * 0.86, 0.02, -0.06),
                                              (s * 0.86, 0.04, -0.035)], 0.02, 2),
                    None, M.black_metal, profile=G.rounded_rect(0.03, 0.04, 0.008, corner=1), up=(1, 0, 0), angle=50)
        F.soft('unit', 1.86, 0.42, 0.43, (0, 0.25, -0.215), M.cinema_unit, round=0.012, round_segments=2)
        F.soft('top', 1.88, 0.03, 0.44, (0, 0.475, -0.21), M.cinema_oak, round=0.009, round_segments=2)
        F.box('reveal', 1.82, 0.31, 0.002, (0, 0.25, 0.0005), M.grille)
        for x in (-0.62, 0.0, 0.62):
            # a slab front with a finger grip routed along its top
            F.soft(f'front{x}', 0.58, 0.3, 0.012, (x, 0.25, 0.006), M.cinema_oak, round=0.004, round_segments=2)
            _capsule(f'grip{x}', x - 0.08, x + 0.08, 0.385, 0.012, 0.012, 0.012, M.cinema_unit, r=0.004)
        # the soundbar at the front, the short-throw projector behind it, a player on the right
        _capsule('soundbar', -0.5, 0.5, 0.525, -0.07, 0.07, 0.09, M.tower, r=0.015)
        _grille('soundbar_grille', 0, 0.525, -0.0255, 0.94, 0.04, M.grille, rows=5)
        _rbox('projector', 0.5, 0.1, 0.3, (0, 0.54, -0.27), M.projector, r=0.025, segs=3)
        _rbox('lens', 0.32, 0.006, 0.07, (0, 0.591, -0.37), M.lens, r=0.002)
        F.box('lens_glass', 0.28, 0.002, 0.04, (0, 0.5945, -0.37), M.glass_lens)
        for k in range(6):
            F.box(f'projector_vent{k}', 0.004, 0.06, 0.002, (-0.12 + 0.012 * k, 0.53, -0.1195), M.grille)
        _rbox('player', 0.34, 0.05, 0.24, (0.55, 0.515, -0.25), M.tower, r=0.01, segs=2)
        F.box('player_slot', 0.16, 0.005, 0.002, (0.52, 0.522, -0.1295), M.cone)
        F.box('player_led', 0.02, 0.008, 0.004, (0.66, 0.515, -0.128), M.led)
        # the towers: rounded cabinets on plinths, two woofers and a tweeter each behind rings
        for s in (-1, 1):
            x = s * 0.86
            F.soft(f'tower{s}', 0.15, 0.88, 0.2, (x, 0.95, -0.3), M.tower, round=0.018, round_segments=2)
            _rbox(f'tower_plinth{s}', 0.155, 0.02, 0.205, (x, 0.5, -0.3), M.black_metal, r=0.004)
            for y in (0.78, 1.0):
                _driver(f'woofer{s}{y}', x, y, -0.2, 0.055, ring=M.black_metal)
            _driver(f'tweeter{s}', x, 1.22, -0.2, 0.025, verts=14)
            F.box(f'tower_badge{s}', 0.03, 0.006, 0.002, (x, 0.56, -0.199), M.steel)
        # the screen's case, on the wall above, with its end caps and brackets
        _capsule('case', -SCREEN_W / 2 - 0.04, SCREEN_W / 2 + 0.04, CASE_Y, WALL + 0.045, 0.09, 0.09, M.tower, r=0.035)
        for s in (-1, 1):
            _rbox(f'cap{s}', 0.02, 0.1, 0.1, (s * (SCREEN_W / 2 + 0.05), CASE_Y, WALL + 0.05), M.black_metal, r=0.008,
                  segs=2)
    G.join(still, 'homeCinema', origin=(0, 0, 0))
    # the screen, its origin in the case, rolled up to nothing at rest: `screen_down` pulls it down
    top = CASE_Y - 0.045
    z = WALL + 0.045
    with G.new_parts() as screen:
        F.box('projection', SCREEN_W - 0.06, SCREEN_H - 0.1, 0.004, (0, top - 0.06 - (SCREEN_H - 0.1) / 2 - 0.02, z),
              M.projection)
        F.box('screen_back', SCREEN_W, SCREEN_H, 0.003, (0, top - SCREEN_H / 2, z - 0.0035), M.screen_border)
        _capsule('screen_bar', -SCREEN_W / 2 - 0.01, SCREEN_W / 2 + 0.01, top - SCREEN_H, z, 0.02, 0.016, M.black_metal,
                 r=0.007)
        F.sweep('screen_pull', G.kit.fillet([(-0.03, top - SCREEN_H - 0.008, z), (-0.03, top - SCREEN_H - 0.04, z),
                                             (0.03, top - SCREEN_H - 0.04, z), (0.03, top - SCREEN_H - 0.008, z)], 0.01),
                0.003, M.black_metal, verts=5)
    screen = G.join(screen, 'screen', origin=F.at(0, CASE_Y - 0.01, z))
    screen.scale[2] = 0.001
    G.settle(centre=False)
    G.grow(screen, 'screen_down', seconds=1.4)


# --- the stereo (1x1): an oak hi-fi cabinet with an amplifier, a CD player and records in its open front, a
# turntable between two speakers on top ------------------------------------------------------------------------
ST_Z = -0.22   # the cabinet's middle depth: its back on the wall face
RECORDS = ('rec_rust', 'rec_black', 'rec_mustard', 'rec_teal', 'rec_cream', 'rec_black', 'rec_navy', 'rec_clay',
           'rec_teal', 'rec_rust')


def stereo():
    z = ST_Z
    d = 0.42
    with G.new_parts() as still:
        # the open cabinet: a shell around two bays
        _rbox('cab_top', 0.92, 0.025, d, (0, 0.5875, z), M.stereo_wood, r=0.006)
        _rbox('cab_bottom', 0.92, 0.025, d, (0, 0.1525, z), M.stereo_wood, r=0.006)
        for s in (-1, 1):
            _rbox(f'cab_side{s}', 0.025, 0.46, d, (s * 0.4475, 0.37, z), M.stereo_wood, r=0.006)
        F.box('cab_mid', 0.02, 0.41, d - 0.02, (-0.01, 0.37, z - 0.01), M.stereo_wood)
        F.box('cab_back', 0.87, 0.41, 0.012, (0, 0.37, z - d / 2 + 0.006), M.stereo_inside)
        F.box('cab_shelf', 0.43, 0.018, d - 0.02, (-0.225, 0.37, z - 0.01), M.stereo_wood)
        for x in (-0.4, 0.4):
            for s in (-1, 1):
                # tapered legs, splayed a little, in brass shoes
                F.sweep(f'leg{x}{s}', [(x, 0.14, z + s * 0.16), (x * 1.04, 0.0, z + s * 0.17)], 0.02, M.stereo_wood, verts=8,
                        scale=[1.0, 0.65])
        # the amplifier on the shelf: a brushed front, a display, two knobs and a power button
        _rbox('amp', 0.4, 0.1, 0.3, (-0.225, 0.43, z), M.stereo_dark, r=0.006)
        F.box('amp_face', 0.39, 0.09, 0.002, (-0.225, 0.43, z + 0.1505), M.steel_dark)
        F.box('amp_display', 0.12, 0.03, 0.002, (-0.3, 0.44, z + 0.152), M.display)
        for x in (-0.15, -0.08):
            G.revolve(f'amp_knob{x}', [(0, 0), (0.019, 0), (0.019, 0.004), (0.016, 0.016), (0, 0.017)],
                      at=F.at(x, 0.44, z + 0.151), material=M.steel, segments=14, rot=(90, 0, 0))
        F.cyl('amp_power', 0.006, 0.004, (-0.39, 0.41, z + 0.152), M.led, rot=(90, 0, 0), verts=8)
        # the CD player under it, its tray's slot
        _rbox('cd', 0.4, 0.07, 0.3, (-0.225, 0.2, z), M.stereo_dark, r=0.006)
        F.box('cd_slot', 0.16, 0.012, 0.002, (-0.26, 0.21, z + 0.1505), M.stereo_inside)
        F.box('cd_led', 0.02, 0.008, 0.002, (-0.1, 0.2, z + 0.151), M.led)
        # records on end in the right bay
        for i, rec in enumerate(RECORDS):
            F.box(f'record{i}', 0.012, 0.3, 0.31, (0.04 + i * 0.036, 0.315, z - 0.02), getattr(M, rec),
                  rot=(0, 3 if i == len(RECORDS) - 1 else 0, 0))
        # the turntable on top: a plinth, the arm on its pivot with a counterweight and a headshell, the speed knob
        _rbox('plinth', 0.4, 0.07, 0.32, (0, 0.635, z), M.stereo_wood, r=0.01)
        G.revolve('arm_base', [(0, 0), (0.022, 0), (0.022, 0.012), (0.012, 0.016), (0.01, 0.03), (0, 0.03)],
                  at=F.at(0.15, 0.67, z - 0.1), material=M.steel, segments=12)
        F.sweep('tonearm', [(0.175, 0.698, z - 0.135), (0.15, 0.698, z - 0.1), (0.1, 0.699, z - 0.0), (0.075, 0.699, z + 0.05)],
                0.004, M.steel, verts=6, resolution=2)
        _rbox('headshell', 0.016, 0.006, 0.03, (0.072, 0.697, z + 0.06), M.stereo_dark, r=0.002, rot=(0, 0, 25))
        F.cyl('counterweight', 0.012, 0.022, (0.178, 0.698, z - 0.14), M.steel, rot=_axle(0.58, -0.81), verts=10)
        F.cyl('speed_knob', 0.012, 0.01, (-0.16, 0.675, z + 0.12), M.steel, verts=10)
        # the speakers: oak boxes, dark baffles, a tweeter (the woofers pulse)
        for s in (-1, 1):
            x = s * 0.37
            _rbox(f'speaker{s}', 0.16, 0.24, 0.2, (x, 0.72, z - 0.02), M.stereo_wood, r=0.01)
            F.box(f'baffle{s}', 0.14, 0.22, 0.004, (x, 0.72, z + 0.081), M.stereo_dark)
            _driver(f'tweeter{s}', x, 0.79, z + 0.083, 0.022, cone=M.grille, verts=12)
            F.cyl(f'surround{s}', 0.058, 0.004, (x, 0.685, z + 0.084), M.grille, rot=(90, 0, 0), verts=24)
            F.cyl(f'port{s}', 0.01, 0.003, (x + 0.045, 0.62, z + 0.0835), M.stereo_inside, rot=(90, 0, 0), verts=10)
    G.join(still, 'stereo', origin=(0, 0, 0))
    # the record on the turntable: the platter, the vinyl and its label, with a stripe on the label so its turn
    # shows; it goes round at 33 rpm while the music plays (`record`, a loop)
    px, py = -0.03, 0.68
    with G.new_parts() as record:
        G.revolve('platter', [(0, -0.01), (0.15, -0.01), (0.152, -0.006), (0.152, 0.006), (0.148, 0.01), (0, 0.01)],
                  at=F.at(px, py, z), material=M.stereo_dark, segments=32)
        # the vinyl: a disc with grooved rings, a raised label and the spindle
        G.revolve('vinyl', [(0.0, 0.0), (0.145, 0.0), (0.145, 0.0025), (0.12, 0.0025), (0.119, 0.002), (0.09, 0.002),
                            (0.089, 0.0025), (0.06, 0.0025), (0.059, 0.002), (0.048, 0.002), (0.046, 0.003), (0.0, 0.003)],
                  at=F.at(px, 0.69, z), material=M.vinyl, segments=32)
        F.cyl('vinyl_label', 0.045, 0.002, (px, 0.6935, z), M.label, verts=16)
        F.cyl('spindle', 0.0035, 0.012, (px, 0.697, z), M.steel, verts=8)
        F.box('label_stripe', 0.05, 0.002, 0.012, (px + 0.012, 0.6945, z), M.vinyl)
        F.box('vinyl_sheen', 0.07, 0.001, 0.008, (px - 0.085, 0.6927, z + 0.03), M.disc, rot=(0, 20, 0))
    record = G.join(record, 'record', origin=F.at(px, py, z))
    # the woofers' cones, one part, its origin on the baffles' face: they pulse forward and back
    with G.new_parts() as cones:
        for s in (-1, 1):
            x = s * 0.37
            G.revolve(f'cone{s}', [(0, 0.0), (0.054, 0.016), (0.054, 0.022), (0.05, 0.02), (0.024, 0.012), (0.02, 0.006),
                                   (0, 0.004)],
                      at=F.at(x, 0.685, z + 0.075), material=M.cone, segments=20, rot=(90, 0, 0))
            F.sphere(f'cap{s}', 0.02, (x, 0.685, z + 0.093), M.cone, scale=(1, 0.5, 1), segments=12, rings=6)
    cones = G.join(cones, 'cones', origin=F.at(0, 0.685, z + 0.082))
    G.settle(centre=False)
    G.spin(record, 'record', turns=1, seconds=1.8)
    G.squash(cones, 'pulse', (1, 1.6, 1), seconds=0.5, back=True)


# --- the boombox (1x1): a twin-speaker cassette radio on a record crate turned on its side ---------------------
BB_Y = 0.42   # the boombox's foot: the crate's top
BB_Z = -0.08


def boombox():
    y0, z = BB_Y, BB_Z
    zf = z + 0.07    # the boombox's front
    with G.new_parts() as still:
        # the crate on its side, open to the front: slatted sides, top and back, corner posts, records inside
        _rbox('crate_bottom', 0.6, 0.02, 0.44, (0, 0.01, 0), M.crate, r=0.003)
        for s in (-1, 1):
            for y in (0.1, 0.3):
                _rbox(f'crate_side{s}{y}', 0.018, 0.13, 0.44, (s * 0.291, y, 0), M.crate, r=0.003)
            for zz in (-0.205, 0.205):
                _rbox(f'crate_post{s}{zz}', 0.03, 0.42, 0.03, (s * 0.285, 0.21, zz), M.crate_dark, r=0.003)
        for x in (-0.15, 0.15):
            _rbox(f'crate_top{x}', 0.27, 0.018, 0.44, (x, 0.411, 0), M.crate, r=0.003)
        for y in (0.1, 0.3):
            _rbox(f'crate_back{y}', 0.6, 0.13, 0.018, (0, y, -0.211), M.crate, r=0.003)
        for i in range(11):
            rec = RECORDS[i % len(RECORDS)]
            F.box(f'record{i}', 0.01, 0.3, 0.3, (-0.24 + i * 0.033, 0.17, -0.04), getattr(M, rec),
                  rot=(0, -6 if i == 10 else 0, 0))
        for i, (x, zz, turn) in enumerate(((0.1, 0.1, 12), (0.13, 0.12, -20))):
            _rbox(f'tape_case{i}', 0.11, 0.017, 0.07, (x, 0.4285 + i * 0.017, zz), M.cassette if i else M.rec_cream,
                  r=0.003, rot=(0, 0, turn))
        # the boombox: a rounded case, a metal top, two speakers either side of the cassette deck
        F.soft('case', 0.52, 0.26, 0.14, (0, y0 + 0.13, z), M.boombox_body, round=0.03, round_segments=2)
        _capsule('top_strip', -0.25, 0.25, y0 + 0.264, z, 0.02, 0.12, M.boombox_trim, r=0.008)
        for s in (-1, 1):
            x = s * 0.17
            F.cyl(f'speaker{s}', 0.075, 0.006, (x, y0 + 0.12, zf + 0.003), M.grille, rot=(90, 0, 0), verts=20)
            for k in range(5):   # the grille's ribs
                F.box(f'speaker_rib{s}{k}', 0.13 - 0.03 * abs(k - 2) ** 1.4, 0.004, 0.002,
                      (x, y0 + 0.12 + 0.025 * (k - 2), zf + 0.007), M.grille_dark)
            F.sweep(f'ring{s}', [(x + 0.078 * G.cosd(a), y0 + 0.12 + 0.078 * G.sind(a), zf + 0.006) for a in range(0, 360, 18)],
                    0.007, M.boombox_trim, verts=5, closed=True)
            F.cyl(f'cap{s}', 0.022, 0.012, (x, y0 + 0.12, zf + 0.008), M.cone, rot=(90, 0, 0), verts=12)
            F.cyl(f'tweeter{s}', 0.016, 0.006, (x, y0 + 0.225, zf + 0.002), M.boombox_trim, rot=(90, 0, 0), verts=10)
        # the handle: one bent bar over the top on two posts, a rubber grip in the middle
        F.sweep('handle', G.kit.fillet([(-0.19, y0 + 0.265, z), (-0.19, y0 + 0.33, z), (0.19, y0 + 0.33, z),
                                        (0.19, y0 + 0.265, z)], 0.035, 3), 0.012, M.boombox_trim, verts=8)
        _capsule('handle_grip', -0.1, 0.1, y0 + 0.33, z, 0.03, 0.03, M.cassette, r=0.014)
        F.sweep('aerial', [(0.22, y0 + 0.275, z - 0.05), (0.25, y0 + 0.4, z - 0.065), (0.28, y0 + 0.52, z - 0.08)], 0.005,
                M.boombox_trim, verts=6, scale=[1.0, 0.8, 0.6])
        F.cyl('aerial_hinge', 0.01, 0.016, (0.22, y0 + 0.27, z - 0.05), M.boombox_trim, rot=(0, 90, 0), verts=8)
        F.sphere('aerial_tip', 0.007, (0.28, y0 + 0.52, z - 0.08), M.boombox_trim, segments=8, rings=5)
        # the deck: a recess, the cassette in it, a tuning display above and piano keys on top
        F.box('deck', 0.13, 0.09, 0.004, (0, y0 + 0.125, zf + 0.001), M.stereo_inside)
        _rbox('tape', 0.1, 0.064, 0.012, (0, y0 + 0.125, zf - 0.004), M.cassette, r=0.003)
        F.box('tape_label', 0.08, 0.02, 0.001, (0, y0 + 0.142, zf + 0.002), M.rec_cream)
        for x in (-0.025, 0.025):
            F.cyl(f'reel{x}', 0.011, 0.004, (x, y0 + 0.125, zf + 0.003), M.rec_cream, rot=(90, 0, 0), verts=10)
        F.box('display', 0.1, 0.022, 0.003, (0, y0 + 0.21, zf + 0.001), M.display)
        F.box('display_needle', 0.002, 0.018, 0.001, (0.02, y0 + 0.21, zf + 0.003), M.led_red)
        for i in range(5):
            _rbox(f'button{i}', 0.028, 0.012, 0.02, (-0.075 + i * 0.035, y0 + 0.275, z + 0.035), (M.cassette, M.cassette,
                  M.led_red, M.cassette, M.cassette)[i], r=0.003)
        for x in (-0.22, 0.22):
            F.cyl(f'dial{x}', 0.014, 0.012, (x, y0 + 0.23, zf + 0.003), M.boombox_trim, rot=(90, 0, 0), verts=12)
    G.join(still, 'boombox', origin=(0, 0, 0))
    # the cassette door, hinged along its foot: `cassette` tips it open towards the front
    hy = y0 + 0.125 - 0.045
    with G.new_parts() as door:
        _rbox('door_frame', 0.13, 0.09, 0.008, (0, y0 + 0.125, zf + 0.006), M.cassette, r=0.003)
        F.box('door_window', 0.1, 0.05, 0.003, (0, y0 + 0.13, zf + 0.011), M.window)
        F.box('door_tab', 0.04, 0.008, 0.004, (0, y0 + 0.165, zf + 0.011), M.boombox_trim)
    door = G.hinged(G.join(door, 'door'), F.at(0, hy, zf + 0.006))
    G.settle(centre=False)
    G.swing(door, 'cassette', 32, seconds=0.5, axis='X')


# --- the electric fireplace (2x1): a white surround with an oak mantel, a log bed glowing in its firebox -------
FP_Z0, FP_Z1 = WALL, -0.11      # the surround's back and front
BOX_Y0, BOX_Y1, BOX_X = 0.16, 0.68, 0.48
BOX_BACK = -0.24
LOG_Y, LOG_Z = 0.2, -0.165
HEARTH_Z = 0.29   # the floor cushion's middle
# a tongue of flame, (radius, height) of a unit one, round at the foot and drawn to a point
TONGUE = [(0, 0), (0.3, 0.06), (0.48, 0.2), (0.46, 0.38), (0.34, 0.58), (0.18, 0.8), (0, 1.0)]


def _log(name, a, b, r, seed):
    """A split log from game point `a` to `b`: a lumpy barked round, its sawn ends showing."""
    log = F.sweep(name, [a, tuple((p + q) / 2 for p, q in zip(a, b)), b], r, M.logs, verts=9, scale=[0.95, 1.05, 0.9])
    G.jitter(log, r * 0.12, size=0.04, seed=seed)
    return log


def fireplace_electric():
    zc = (FP_Z0 + FP_Z1) / 2
    d = FP_Z1 - FP_Z0
    with G.new_parts() as still:
        # the surround: two piers, each panelled, a lintel with a frieze panel, the hearth and a plinth
        pier = (1.6 - 2 * BOX_X) / 2
        for s in (-1, 1):
            px = s * (BOX_X + pier / 2)
            _rbox(f'pier{s}', pier, 0.98, d, (px, 0.49, zc), M.surround, r=0.008, segs=2)
            G.kit.moulding(F, f'pier_bead{s}', [(px + pier / 2 - 0.05, 0.12), (px + pier / 2 - 0.05, 0.86),
                                                (px - pier / 2 + 0.05, 0.86), (px - pier / 2 + 0.05, 0.12)],
                           [(0.0, 0.0), (0.0, 0.004), (0.006, 0.008), (0.012, 0.004), (0.012, 0.0)], M.surround,
                           at=FP_Z1)
        _rbox('lintel', 2 * BOX_X + 0.02, 0.98 - BOX_Y1, d, (0, (BOX_Y1 + 0.98) / 2, zc), M.surround, r=0.008, segs=2)
        G.kit.moulding(F, 'frieze', [(BOX_X - 0.04, BOX_Y1 + 0.07), (BOX_X - 0.04, 0.92), (-BOX_X + 0.04, 0.92),
                                     (-BOX_X + 0.04, BOX_Y1 + 0.07)],
                       [(0.0, 0.0), (0.0, 0.004), (0.006, 0.008), (0.012, 0.004), (0.012, 0.0)], M.surround, at=FP_Z1)
        _rbox('hearth', 2 * BOX_X + 0.02, BOX_Y0, d, (0, BOX_Y0 / 2, zc), M.surround, r=0.008, segs=2)
        _rbox('plinth', 1.64, 0.05, d + 0.02, (0, 0.025, zc + 0.01), M.surround, r=0.008, segs=2)
        # the firebox: a black frame round the opening, its glossy back and floor, a fluted back panel
        F.box('box_back', 2 * BOX_X, BOX_Y1 - BOX_Y0, 0.01, (0, (BOX_Y0 + BOX_Y1) / 2, BOX_BACK), M.firebox)
        for k in range(9):
            F.box(f'box_flute{k}', 0.006, BOX_Y1 - BOX_Y0 - 0.04, 0.004, (-0.4 + 0.1 * k, (BOX_Y0 + BOX_Y1) / 2,
                                                                         BOX_BACK + 0.007), M.firebox_line)
        F.box('box_floor', 2 * BOX_X, 0.01, FP_Z1 - BOX_BACK, (0, BOX_Y0 + 0.005, (BOX_BACK + FP_Z1) / 2), M.firebox)
        for s in (-1, 1):
            F.box(f'box_side{s}', 0.01, BOX_Y1 - BOX_Y0, FP_Z1 - BOX_BACK, (s * (BOX_X - 0.005), (BOX_Y0 + BOX_Y1) / 2,
                  (BOX_BACK + FP_Z1) / 2), M.firebox)
        F.box('box_roof', 2 * BOX_X, 0.01, FP_Z1 - BOX_BACK, (0, BOX_Y1 - 0.005, (BOX_BACK + FP_Z1) / 2), M.firebox)
        fw = 0.03
        G.kit.moulding(F, 'box_frame', [(BOX_X + fw, BOX_Y0 - fw), (BOX_X + fw, BOX_Y1 + fw), (-BOX_X - fw, BOX_Y1 + fw),
                                        (-BOX_X - fw, BOX_Y0 - fw)],
                       [(0.0, 0.0), (0.0, 0.01), (0.006, 0.012), (fw, 0.012), (fw, 0.0)], M.firebox, at=FP_Z1)
        F.box('switch', 0.03, 0.012, 0.004, (BOX_X - 0.05, BOX_Y0 - 0.015, FP_Z1 + 0.014), M.steel)
        # the log bed: logs on a strip of embers and stones, which glows at night as the fire's light comes on
        _rbox('embers', 0.84, 0.03, 0.08, (0, BOX_Y0 + 0.02, LOG_Z + 0.01), M.ember, r=0.01, segs=2)
        for i in range(9):
            st = F.sphere(f'ember_stone{i}', 0.018, (-0.36 + 0.09 * i, BOX_Y0 + 0.032, LOG_Z + 0.04 - 0.02 * (i % 2)),
                          M.logs if i % 3 else M.ember, scale=(1.3, 1.0, 0.6), segments=8, rings=5)
            G.jitter(st, 0.004, size=0.02, seed=i)
        for i, (x, length, turn) in enumerate(((-0.2, 0.42, 4), (0.2, 0.4, -5), (0.0, 0.36, 0))):
            y = LOG_Y + (0.03 if i == 2 else 0.0)
            zz = LOG_Z - (0.03 if i == 2 else 0.0)
            c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
            _log(f'log{i}', (x - c * length / 2, y, zz + s * length / 2), (x + c * length / 2, y, zz - s * length / 2), 0.04, i)
        # the floor cushion in front, where residents sit cross-legged facing the fire (furniture.json seats: 0.14 high,
        # the hips 0.36 out): plump, piped, a button tufted into its middle
        F.soft('cushion', 0.56, 0.12, 0.4, (0, 0.06, HEARTH_Z), M.hearth_cushion, round=0.045, puff=0.012, flat=3,
               piping=0.008, piping_material=M.hearth_piping)
        F.cyl('cushion_button', 0.016, 0.012, (0, 0.13, HEARTH_Z), M.hearth_piping, verts=10, bevel=0.004,
              bevel_segments=1)
        # the mantel: a moulded shelf on two corbels, a candle vase and a framed picture on it
        F.soft('mantel', 1.76, 0.06, 0.37, (0, 1.0, WALL + 0.185), M.mantel, round=0.012, round_segments=2)
        for s in (-1, 1):
            F.extrude(f'corbel{s}', G.smooth_outline([(0.0, 0.0), (0.045, 0.0), (0.045, -0.03), (0.012, -0.12), (0.0, -0.12)],
                                                     passes=1),
                      0.06, (s * 0.7, 0.97, FP_Z1), M.mantel, plane='side', bevel=0.004, bevel_segments=1)
        G.loft('vase', [G.ring(r, r, 1.03 + h, 20, at=F.at(-0.6, 0, -0.32)[:2]) for h, r in
                        ((0.0, 0.05), (0.01, 0.058), (0.08, 0.062), (0.18, 0.05), (0.24, 0.04), (0.26, 0.042))],
               material=M.candle)
        F.cyl('candle', 0.018, 0.06, (-0.6, 1.25, -0.32), M.candle_wax, verts=10)
        G.kit.moulding(F, 'frame_pic', [(0.66, 1.03), (0.66, 1.31), (0.44, 1.31), (0.44, 1.03)],
                       [(0.0, 0.0), (0.0, 0.02), (0.012, 0.025), (0.03, 0.02), (0.03, 0.0)], M.picture, at=-0.383)
        F.box('print', 0.17, 0.22, 0.004, (0.55, 1.17, -0.381), M.art_print)
        F.box('print_sky', 0.15, 0.09, 0.002, (0.55, 1.22, -0.378), M.print_sky)
        F.box('print_hill', 0.15, 0.06, 0.002, (0.55, 1.12, -0.3775), M.print_hill)
    G.join(still, 'fireplaceElectric', origin=(0, 0, 0))
    # the flames: tongues on the logs, their origin inside the log bed, flattened into it at rest
    with G.new_parts() as flames:
        tongues = ((-0.3, 0.2, 0.17), (-0.15, 0.27, 0.15), (0.0, 0.23, 0.18), (0.15, 0.29, 0.15), (0.3, 0.2, 0.16))
        for i, (x, h, w) in enumerate(tongues):
            lean = (-6, 4, -3, 7, -5)[i]
            G.revolve(f'flame{i}', TONGUE, at=F.at(x, LOG_Y - 0.02, LOG_Z), material=M.flame, segments=14,
                      scale=(w, 0.04, h + 0.02), rot=(0, lean, 0))
            G.revolve(f'core{i}', TONGUE, at=F.at(x, LOG_Y - 0.01, LOG_Z + 0.012), material=M.flame_core, segments=12,
                      scale=(w * 0.55, 0.03, h * 0.6), rot=(0, lean, 0))
    flames = G.join(flames, 'flames', origin=F.at(0, LOG_Y, LOG_Z))
    flames.scale[2] = 0.001
    G.settle(centre=False)
    G.grow(flames, 'flames', seconds=0.8)


FAMILY = G.Family(
    'media', M,
    [G.Model('tv', tv, key='model.tv', size=(0.91, 1.07, 0.445), centre=(0, -0.21),
             paint=[('tv_cabinet', 'Body'), ('tv_oak', 'Trim')]),
     G.Model('tvWall', tv_wall, key='model.tvWall', size=(1.24, 0.87, 0.102), centre=(0, -0.379), floor=False,
             paint=[('wall_frame', 'Body'), ('wall_bar', 'Trim')]),
     G.Model('tvRetro', tv_retro, key='model.tvRetro', size=(0.94, 1.132, 0.493), centre=(0, -0.1884),
             motions={'uses': {'watch': 'antenna'}},
             paint=[('retro_wood', 'Body'), ('retro_bezel', 'Trim')]),
     G.Model('tvStandLow', tv_stand_low, key='model.tvStandLow', size=(1.82, 0.8, 0.464), centre=(0, -0.198),
             motions={'uses': {'watch': 'tv_up'}},
             paint=[('bench', 'Bench'), ('bench_tv', 'TV')]),
     G.Model('homeCinema', home_cinema, key='model.homeCinema', size=(1.88, 1.65, 0.448), centre=(0, -0.206),
             motions={'uses': {'watch': 'screen_down', 'film': 'screen_down'}},
             paint=[('cinema_unit', 'Body'), ('cinema_oak', 'Trim')]),
     G.Model('stereo', stereo, key='model.stereo', size=(0.92, 0.84, 0.42), centre=(0, -0.22),
             # while the music plays the record turns (a loop) and the woofers pulse (`every` a hair under their beat)
             motions={'uses': {use: {'clip': 'pulse', 'every': 0.45} for use in ('listen', 'dance')},
                      'loops': {'listen': 'record', 'dance': 'record'}},
             paint=[('stereo_wood', 'Body'), ('stereo_dark', 'Trim')]),
     G.Model('boombox', boombox, key='model.boombox', size=(0.6, 0.95, 0.44),
             motions={'uses': {'listen': 'cassette'}, 'hold': 1.2},
             paint=[('boombox_body', 'Body'), ('boombox_trim', 'Trim')]),
     G.Model('fireplaceElectric', fireplace_electric, key='model.fireplaceElectric', size=(1.76, 1.31, 0.92),
             centre=(0, 0.03), motions={'uses': {'relax': 'flames'}},
             paint=[('surround', 'Surround'), ('firebox', 'Firebox')])],
    comment='Blender-built living room media (tools/art/blender/families/media.py): the TVs, the home '
            'cinema, the stereo, the boombox and the electric fireplace. The materials map onto the '
            'game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
