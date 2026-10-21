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
#   stereo             the CD tray slides out and back (`tray`, hold 1.2 s), the woofers pulse (`pulse`, E1 loop);
#   boombox            the cassette door tips open and shuts (`cassette`, hold 1.2 s);
#   fireplaceElectric  the flames grow from the log bed (`flames`, for relax; gloss, glowing at night).
#
# Run:  cargo art media [--only tvRetro] [--preview]
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
)

F = G.GameFrame()
WALL = -0.43


def _front_cyl(name, r, depth, x, y, z, mat, verts=20, **kw):
    """A disc facing the front (its axis along the game's depth), its face at depth `z` + depth/2."""
    return F.cyl(name, r, depth, (x, y, z), mat, rot=(90, 0, 0), verts=verts, **kw)


def _rod(name, a, b, r, mat, verts=8):
    """A rod between two game points."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=mat, verts=verts)


# --- the television (1x1): a flat set on its stand on a low oak media unit (the old `tv_set`) -------------
def tv():
    F.box('unit', 0.9, 0.4, 0.42, (0, 0.24, -0.22), M.tv_cabinet, bevel=0.012)
    F.box('top', 0.91, 0.03, 0.43, (0, 0.455, -0.215), M.tv_oak, bevel=0.008)
    for s in (-1, 1):
        F.box(f'door{s}', 0.43, 0.3, 0.012, (s * 0.225, 0.24, -0.004), M.tv_oak, bevel=0.005)
        F.box(f'pull{s}', 0.012, 0.12, 0.012, (s * 0.02, 0.24, 0.008), M.black_metal, bevel=0.003)
        for z in (-0.4, -0.04):
            F.box(f'foot{s}{z}', 0.03, 0.04, 0.03, (s * 0.42, 0.02, z), M.black_metal, bevel=0.006)
    # the set: a slim panel with a deeper back, on a neck and a foot
    F.box('panel', 0.86, 0.5, 0.025, (0, 0.82, -0.29), M.tv_frame, bevel=0.008)
    F.box('back', 0.6, 0.32, 0.03, (0, 0.8, -0.317), M.tv_frame, bevel=0.01)
    F.box('screen', 0.835, 0.475, 0.003, (0, 0.82, -0.2765), M.screen)
    F.box('neck', 0.06, 0.13, 0.025, (0, 0.53, -0.32), M.tv_frame, bevel=0.006)
    F.box('foot', 0.26, 0.012, 0.16, (0, 0.476, -0.3), M.tv_frame, bevel=0.005)
    # a soundbar in front of it, a bowl of pebbles beside
    F.box('soundbar', 0.5, 0.05, 0.07, (0, 0.495, -0.12), M.soundbar, bevel=0.02)
    F.box('soundbar_grille', 0.46, 0.03, 0.004, (0, 0.495, -0.0835), M.grille)
    G.revolve('bowl', [(0, 0), (0.05, 0), (0.065, 0.02), (0.06, 0.035), (0.05, 0.02), (0, 0.012)],
              at=F.at(0.33, 0.47, -0.1), material=M.bowl, segments=20)
    for i, (x, z) in enumerate(((0.32, -0.1), (0.345, -0.09), (0.33, -0.12))):
        F.sphere(f'pebble{i}', 0.022, (x, 0.5, z), M.pebble, scale=(1, 1, 0.6), segments=10, rings=6)
    G.join_all('tv')
    G.settle(centre=False)


# --- the flat-screen TV (2x1, wall): a thin panel on a bracket, a soundbar under it (the old tv_wall) ------
def tv_wall():
    y = 1.28
    F.box('bracket', 0.3, 0.2, 0.008, (0, y - 0.02, WALL + 0.004), M.black_metal, bevel=0.002)
    F.box('back', 0.8, 0.45, 0.022, (0, y - 0.02, WALL + 0.019), M.wall_frame, bevel=0.008)
    F.box('panel', 1.24, 0.72, 0.022, (0, y, WALL + 0.041), M.wall_frame, bevel=0.006)
    F.box('screen', 1.215, 0.695, 0.003, (0, y, WALL + 0.0525), M.screen)
    F.box('logo', 0.05, 0.008, 0.003, (0, y - 0.352, WALL + 0.0525), M.steel)
    # the soundbar on its own two brackets
    for s in (-1, 1):
        F.box(f'bar_bracket{s}', 0.03, 0.05, 0.03, (s * 0.3, 0.8, WALL + 0.015), M.black_metal)
    F.box('soundbar', 0.9, 0.06, 0.07, (0, 0.8, WALL + 0.065), M.wall_bar, bevel=0.022)
    F.box('soundbar_grille', 0.84, 0.036, 0.004, (0, 0.8, WALL + 0.1), M.grille)
    G.join_all('tvWall')
    G.settle(centre=False, floor=False)


# --- the portable TV (1x1): a bulging CRT in a honey-wood cabinet on splayed legs, rabbit ears on top --------
RETRO_Z = -0.205   # the cabinet's middle depth: its back on the wall face
EARS = (0.24, 0.875, RETRO_Z - 0.06)


def tv_retro():
    z = RETRO_Z
    with G.new_parts() as still:
        F.box('cabinet', 0.92, 0.62, 0.44, (0, 0.5, z), M.retro_wood, bevel=0.03)
        F.box('top', 0.94, 0.03, 0.46, (0, 0.825, z), M.retro_wood, bevel=0.01)
        # the screen: a bezel and a rounded, bulging tube
        F.box('bezel', 0.56, 0.46, 0.02, (-0.14, 0.52, z + 0.225), M.retro_bezel, bevel=0.03, bevel_segments=3)
        F.box('tube', 0.48, 0.37, 0.03, (-0.14, 0.52, z + 0.24), M.crt, bevel=0.012, bevel_segments=3)
        F.sphere('bulge', 0.22, (-0.14, 0.52, z + 0.25), M.crt, scale=(1.05, 0.06, 0.8), segments=24, rings=12)
        # the speaker: cloth behind wooden slats, two knobs above it
        F.box('cloth', 0.26, 0.3, 0.01, (0.3, 0.44, z + 0.222), M.grille)
        for i in range(7):
            F.box(f'slat{i}', 0.26, 0.014, 0.012, (0.3, 0.315 + i * 0.042, z + 0.229), M.retro_wood)
        for x in (0.24, 0.36):
            _front_cyl(f'knob{x}', 0.028, 0.03, x, 0.7, z + 0.235, M.steel, verts=14)
            F.box(f'knob_mark{x}', 0.006, 0.03, 0.004, (x, 0.71, z + 0.251), M.retro_bezel)
        for x in (-1, 1):
            for s in (-1, 1):
                _rod(f'leg{x}{s}', (x * 0.42, 0.0, z + s * 0.17), (x * 0.38, 0.2, z + s * 0.15), 0.02, M.retro_wood)
        # the ears' foot
        F.sphere('ears_foot', 0.05, (EARS[0], 0.86, EARS[2]), M.retro_bezel, scale=(1, 1, 0.5), segments=14, rings=8)
    G.join(still, 'tvRetro', origin=(0, 0, 0))
    # the rabbit ears: two telescopic rods from the foot, a part of their own turning and extending about it
    with G.new_parts() as ears:
        for name, tip in (('ear_left', (0.1, 1.12, z - 0.12)), ('ear_right', (0.4, 1.1, z - 0.1))):
            mid = tuple(a + (b - a) * 0.55 for a, b in zip(EARS, tip))
            _rod(name, EARS, mid, 0.006, M.steel)
            _rod(name + '_tip', mid, tip, 0.0042, M.steel)
            F.sphere(name + '_ball', 0.008, tip, M.steel, segments=8, rings=5)
    ears = G.join(ears, 'ears', origin=F.at(*EARS))
    G.settle(centre=False)
    G.swing(ears, 'antenna', 20, seconds=0.9)
    G.squash(ears, 'antenna', (1, 1, 1.6), seconds=0.9)


# --- the low TV bench (2x1): a white bench on oak legs, a TV lift in a hood at its back -----------------------
HOOD_W, HOOD_Z0, HOOD_Z1, HOOD_TOP = 1.3, WALL, -0.31, 0.8
LIFT = 0.6


def tv_stand_low():
    with G.new_parts() as still:
        for x in (-0.84, 0.84):
            for z in (-0.4, -0.04):
                F.box(f'leg{x}{z}', 0.04, 0.06, 0.04, (x, 0.03, z), M.bench_oak, bevel=0.006)
        F.box('carcass', 1.8, 0.4, 0.43, (0, 0.26, -0.215), M.bench, bevel=0.01)
        # the top: in front of the hood and either side of it
        F.box('top', 1.82, 0.03, 0.34, (0, 0.475, (HOOD_Z1 + 0.03) / 2), M.bench_oak, bevel=0.008)
        side = (1.82 - HOOD_W) / 2
        for s in (-1, 1):
            F.box(f'top_back{s}', side, 0.03, HOOD_Z1 - WALL, (s * (HOOD_W + side) / 2, 0.475, (WALL + HOOD_Z1) / 2),
                  M.bench_oak, bevel=0.006)
        # the hood the TV sinks into: four walls, the TV's lid closing its top
        h = HOOD_TOP - 0.46
        yc = 0.46 + h / 2
        F.box('hood_back', HOOD_W, h, 0.012, (0, yc, WALL + 0.006), M.bench, bevel=0.003)
        F.box('hood_front', HOOD_W, h, 0.012, (0, yc, HOOD_Z1 - 0.006), M.bench, bevel=0.003)
        for s in (-1, 1):
            F.box(f'hood_side{s}', 0.012, h, HOOD_Z1 - WALL, (s * (HOOD_W / 2 - 0.006), yc, (WALL + HOOD_Z1) / 2), M.bench,
                  bevel=0.003)
        # three doors with black pulls
        for x in (-0.6, 0.0, 0.6):
            F.box(f'door{x}', 0.58, 0.34, 0.02, (x, 0.26, 0.01), M.bench, bevel=0.005)
            F.box(f'pull{x}', 0.12, 0.012, 0.014, (x, 0.4, 0.027), M.black_metal, bevel=0.003)
        # a plant on the left, a remote on the right
        F.cyl('pot', 0.06, 0.1, (-0.78, 0.54, -0.1), M.pot, verts=18, bevel=0.008)
        F.sphere('plant', 0.09, (-0.78, 0.63, -0.1), M.plant, scale=(1, 1, 0.8), segments=14, rings=8)
        F.box('remote', 0.04, 0.015, 0.16, (0.72, 0.4975, -0.1), M.bench_tv, rot=(0, 0, -12), bevel=0.005)
    G.join(still, 'tvStandLow', origin=(0, 0, 0))
    # the TV, down in the bench at rest, its lid flush with the hood's top
    zc = (WALL + HOOD_Z1) / 2
    with G.new_parts() as tv:
        th, y0 = 0.7, HOOD_TOP - 0.02 - 0.7
        F.box('tv_panel', 1.22, th, 0.035, (0, y0 + th / 2, zc), M.bench_tv, bevel=0.006)
        F.box('tv_screen', 1.19, th - 0.03, 0.003, (0, y0 + th / 2 + 0.005, zc + 0.019), M.screen)
        F.box('tv_lid', HOOD_W - 0.03, 0.02, HOOD_Z1 - WALL - 0.026, (0, HOOD_TOP - 0.01, zc), M.bench, bevel=0.004)
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
            F.box(f'sled{s}', 0.04, 0.04, 0.4, (s * 0.86, 0.02, -0.22), M.black_metal, bevel=0.008)
        F.box('unit', 1.86, 0.42, 0.43, (0, 0.25, -0.215), M.cinema_unit, bevel=0.012)
        F.box('top', 1.88, 0.03, 0.44, (0, 0.475, -0.21), M.cinema_oak, bevel=0.008)
        for x in (-0.62, 0.0, 0.62):
            F.box(f'front{x}', 0.58, 0.3, 0.012, (x, 0.25, 0.006), M.cinema_oak, bevel=0.005)
            F.box(f'grip{x}', 0.16, 0.012, 0.012, (x, 0.385, 0.012), M.cinema_unit)
        # the soundbar at the front, the short-throw projector behind it, a player on the right
        F.box('soundbar', 1.0, 0.07, 0.09, (0, 0.525, -0.07), M.tower, bevel=0.025)
        F.box('soundbar_grille', 0.94, 0.044, 0.004, (0, 0.525, -0.0235), M.grille)
        F.box('projector', 0.5, 0.1, 0.3, (0, 0.54, -0.27), M.projector, bevel=0.025)
        F.box('lens', 0.32, 0.006, 0.07, (0, 0.591, -0.37), M.lens, bevel=0.002)
        F.box('player', 0.34, 0.05, 0.24, (0.55, 0.515, -0.25), M.tower, bevel=0.01)
        F.box('player_led', 0.02, 0.008, 0.004, (0.66, 0.515, -0.128), M.led)
        # the towers, two woofers and a tweeter each
        for s in (-1, 1):
            x = s * 0.86
            F.box(f'tower{s}', 0.15, 0.9, 0.2, (x, 0.94, -0.3), M.tower, bevel=0.02)
            for y in (0.78, 1.0):
                _front_cyl(f'woofer{s}{y}', 0.055, 0.012, x, y, -0.198, M.cone, verts=20)
            _front_cyl(f'tweeter{s}', 0.025, 0.012, x, 1.22, -0.198, M.cone, verts=14)
        # the screen's case, on the wall above, with its end caps and brackets
        F.box('case', SCREEN_W + 0.08, 0.09, 0.09, (0, CASE_Y, WALL + 0.045), M.tower, bevel=0.03, bevel_segments=3)
        for s in (-1, 1):
            F.box(f'cap{s}', 0.02, 0.1, 0.1, (s * (SCREEN_W / 2 + 0.05), CASE_Y, WALL + 0.05), M.black_metal, bevel=0.006)
    G.join(still, 'homeCinema', origin=(0, 0, 0))
    # the screen, its origin in the case, rolled up to nothing at rest: `screen_down` pulls it down
    top = CASE_Y - 0.045
    z = WALL + 0.045
    with G.new_parts() as screen:
        F.box('projection', SCREEN_W - 0.06, SCREEN_H - 0.1, 0.004, (0, top - 0.06 - (SCREEN_H - 0.1) / 2 - 0.02, z),
              M.projection)
        F.box('screen_back', SCREEN_W, SCREEN_H, 0.003, (0, top - SCREEN_H / 2, z - 0.0035), M.screen_border)
        F.box('screen_bar', SCREEN_W + 0.02, 0.02, 0.016, (0, top - SCREEN_H, z), M.black_metal, bevel=0.005)
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
        F.box('cab_top', 0.92, 0.025, d, (0, 0.5875, z), M.stereo_wood, bevel=0.006)
        F.box('cab_bottom', 0.92, 0.025, d, (0, 0.1525, z), M.stereo_wood, bevel=0.006)
        for s in (-1, 1):
            F.box(f'cab_side{s}', 0.025, 0.46, d, (s * 0.4475, 0.37, z), M.stereo_wood, bevel=0.006)
        F.box('cab_mid', 0.02, 0.41, d - 0.02, (-0.01, 0.37, z - 0.01), M.stereo_wood)
        F.box('cab_back', 0.87, 0.41, 0.012, (0, 0.37, z - d / 2 + 0.006), M.stereo_inside)
        F.box('cab_shelf', 0.43, 0.018, d - 0.02, (-0.225, 0.37, z - 0.01), M.stereo_wood)
        for x in (-0.4, 0.4):
            for s in (-1, 1):
                F.cone(f'leg{x}{s}', 0.014, 0.02, 0.14, (x, 0.07, z + s * 0.16), M.stereo_wood, verts=10)
        # the amplifier on the shelf: a display and two knobs
        F.box('amp', 0.4, 0.1, 0.3, (-0.225, 0.43, z), M.stereo_dark, bevel=0.008)
        F.box('amp_display', 0.12, 0.03, 0.004, (-0.3, 0.44, z + 0.152), M.display)
        for x in (-0.15, -0.08):
            _front_cyl(f'amp_knob{x}', 0.017, 0.016, x, 0.44, z + 0.156, M.steel, verts=12)
        # the CD player under it, its tray's slot
        F.box('cd', 0.4, 0.07, 0.3, (-0.225, 0.2, z), M.stereo_dark, bevel=0.008)
        F.box('cd_slot', 0.16, 0.014, 0.004, (-0.26, 0.21, z + 0.15), M.stereo_inside)
        F.box('cd_led', 0.02, 0.008, 0.004, (-0.1, 0.2, z + 0.151), M.led)
        # records on end in the right bay
        for i, rec in enumerate(RECORDS):
            F.box(f'record{i}', 0.012, 0.3, 0.31, (0.04 + i * 0.036, 0.315, z - 0.02), getattr(M, rec),
                  rot=(0, 3 if i == len(RECORDS) - 1 else 0, 0), bevel=0.002)
        # the turntable on top
        F.box('plinth', 0.4, 0.07, 0.32, (0, 0.635, z), M.stereo_wood, bevel=0.01)
        F.cyl('platter', 0.15, 0.02, (-0.03, 0.68, z), M.stereo_dark, verts=32)
        F.cyl('vinyl', 0.145, 0.006, (-0.03, 0.693, z), M.vinyl, verts=32)
        F.cyl('vinyl_label', 0.045, 0.008, (-0.03, 0.694, z), M.label, verts=16)
        F.cyl('arm_base', 0.02, 0.02, (0.15, 0.68, z - 0.1), M.steel, verts=12)
        _rod('tonearm', (0.15, 0.7, z - 0.1), (0.06, 0.7, z + 0.04), 0.006, M.steel)
        # the speakers: oak boxes, dark baffles, a tweeter (the woofers pulse)
        for s in (-1, 1):
            x = s * 0.37
            F.box(f'speaker{s}', 0.16, 0.24, 0.2, (x, 0.72, z - 0.02), M.stereo_wood, bevel=0.012)
            F.box(f'baffle{s}', 0.14, 0.22, 0.004, (x, 0.72, z + 0.081), M.stereo_dark)
            _front_cyl(f'tweeter{s}', 0.022, 0.008, x, 0.79, z + 0.085, M.grille, verts=12)
            _front_cyl(f'surround{s}', 0.058, 0.004, x, 0.685, z + 0.084, M.grille, verts=24)
    G.join(still, 'stereo', origin=(0, 0, 0))
    # the CD tray, in the player at rest, with a disc on it
    with G.new_parts() as tray:
        F.box('tray', 0.15, 0.012, 0.15, (-0.26, 0.206, z + 0.075), M.stereo_dark, bevel=0.003)
        F.box('tray_lip', 0.16, 0.016, 0.004, (-0.26, 0.21, z + 0.152), M.stereo_dark)
        F.cyl('cd_disc', 0.058, 0.002, (-0.26, 0.2135, z + 0.075), M.disc, verts=24)
    tray = G.join(tray, 'tray')
    # the woofers' cones, one part, its origin on the baffles' face: they pulse forward and back
    with G.new_parts() as cones:
        for s in (-1, 1):
            x = s * 0.37
            G.cone(f'cone{s}', 0.054, 0.02, 0.016, at=F.at(x, 0.685, z + 0.089), material=M.cone, rot=(-90, 0, 0), verts=24)
            F.sphere(f'cap{s}', 0.02, (x, 0.685, z + 0.097), M.cone, scale=(1, 0.5, 1), segments=12, rings=6)
    cones = G.join(cones, 'cones', origin=F.at(0, 0.685, z + 0.082))
    G.settle(centre=False)
    G.slide(tray, 'tray', (0, -0.11, 0), seconds=0.6)
    G.squash(cones, 'pulse', (1, 1.6, 1), seconds=0.5, back=True)


# --- the boombox (1x1): a twin-speaker cassette radio on a record crate turned on its side ---------------------
BB_Y = 0.42   # the boombox's foot: the crate's top
BB_Z = -0.08


def boombox():
    y0, z = BB_Y, BB_Z
    zf = z + 0.07    # the boombox's front
    with G.new_parts() as still:
        # the crate on its side, open to the front: slatted sides, top and back, records inside
        F.box('crate_bottom', 0.6, 0.02, 0.44, (0, 0.01, 0), M.crate, bevel=0.003)
        for s in (-1, 1):
            for y in (0.1, 0.3):
                F.box(f'crate_side{s}{y}', 0.018, 0.13, 0.44, (s * 0.291, y, 0), M.crate, bevel=0.003)
            for zz in (-0.205, 0.205):
                F.box(f'crate_post{s}{zz}', 0.03, 0.42, 0.03, (s * 0.285, 0.21, zz), M.crate_dark, bevel=0.003)
        for x in (-0.15, 0.15):
            F.box(f'crate_top{x}', 0.27, 0.018, 0.44, (x, 0.411, 0), M.crate, bevel=0.003)
        for y in (0.1, 0.3):
            F.box(f'crate_back{y}', 0.6, 0.13, 0.018, (0, y, -0.211), M.crate, bevel=0.003)
        for i in range(11):
            rec = RECORDS[i % len(RECORDS)]
            F.box(f'record{i}', 0.01, 0.3, 0.3, (-0.24 + i * 0.033, 0.17, -0.04), getattr(M, rec),
                  rot=(0, -6 if i == 10 else 0, 0))
        for i, (x, zz, turn) in enumerate(((0.1, 0.1, 12), (0.13, 0.12, -20))):
            F.box(f'tape_case{i}', 0.11, 0.017, 0.07, (x, 0.4285 + i * 0.017, zz), M.cassette if i else M.rec_cream,
                  rot=(0, 0, turn), bevel=0.003)
        # the boombox: a rounded case, a chrome top, two speakers either side of the cassette deck
        F.box('case', 0.52, 0.26, 0.14, (0, y0 + 0.13, z), M.boombox_body, bevel=0.03, bevel_segments=3)
        F.box('top_strip', 0.5, 0.02, 0.12, (0, y0 + 0.265, z), M.boombox_trim, bevel=0.008)
        for s in (-1, 1):
            x = s * 0.17
            _front_cyl(f'speaker{s}', 0.075, 0.012, x, y0 + 0.12, zf, M.grille, verts=24)
            G.torus(f'ring{s}', 0.078, 0.007, at=F.at(x, y0 + 0.12, zf + 0.004), material=M.boombox_trim,
                    rot=(90, 0, 0), major_segments=20, minor_segments=6)
            _front_cyl(f'cap{s}', 0.022, 0.016, x, y0 + 0.12, zf + 0.004, M.cone, verts=12)
            _rod(f'post{s}', (x * 1.1, y0 + 0.27, z), (x * 1.1, y0 + 0.33, z), 0.012, M.boombox_trim)
        _rod('handle', (-0.2, y0 + 0.33, z), (0.2, y0 + 0.33, z), 0.016, M.boombox_trim, verts=10)
        _rod('aerial', (0.22, y0 + 0.275, z - 0.05), (0.28, y0 + 0.52, z - 0.08), 0.005, M.boombox_trim, verts=6)
        F.sphere('aerial_tip', 0.008, (0.28, y0 + 0.52, z - 0.08), M.boombox_trim, segments=8, rings=5)
        # the deck: a recess, the cassette in it, a display above and buttons on top
        F.box('deck', 0.13, 0.09, 0.004, (0, y0 + 0.125, zf + 0.001), M.stereo_inside)
        F.box('tape', 0.1, 0.064, 0.012, (0, y0 + 0.125, zf - 0.004), M.cassette, bevel=0.003)
        for x in (-0.025, 0.025):
            _front_cyl(f'reel{x}', 0.011, 0.004, x, y0 + 0.13, zf + 0.003, M.rec_cream, verts=10)
        F.box('display', 0.1, 0.022, 0.004, (0, y0 + 0.21, zf + 0.001), M.display)
        for i in range(4):
            F.box(f'button{i}', 0.03, 0.014, 0.02, (-0.06 + i * 0.04, y0 + 0.28, z + 0.03), M.cassette, bevel=0.003)
    G.join(still, 'boombox', origin=(0, 0, 0))
    # the cassette door, hinged along its foot: `cassette` tips it open towards the front
    hy = y0 + 0.125 - 0.045
    with G.new_parts() as door:
        F.box('door_frame', 0.13, 0.09, 0.008, (0, y0 + 0.125, zf + 0.006), M.cassette, bevel=0.003)
        F.box('door_window', 0.1, 0.05, 0.003, (0, y0 + 0.13, zf + 0.011), M.window)
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


def fireplace_electric():
    zc = (FP_Z0 + FP_Z1) / 2
    d = FP_Z1 - FP_Z0
    with G.new_parts() as still:
        # the surround around the firebox: two piers, a lintel, a hearth and the back
        pier = (1.6 - 2 * BOX_X) / 2
        for s in (-1, 1):
            F.box(f'pier{s}', pier, 0.98, d, (s * (BOX_X + pier / 2), 0.49, zc), M.surround, bevel=0.01)
        F.box('lintel', 2 * BOX_X + 0.02, 0.98 - BOX_Y1, d, (0, (BOX_Y1 + 0.98) / 2, zc), M.surround, bevel=0.008)
        F.box('hearth', 2 * BOX_X + 0.02, BOX_Y0, d, (0, BOX_Y0 / 2, zc), M.surround, bevel=0.008)
        F.box('plinth', 1.64, 0.05, d + 0.02, (0, 0.025, zc + 0.01), M.surround, bevel=0.008)
        # the firebox: a black frame round the opening, its glossy back and floor
        F.box('box_back', 2 * BOX_X, BOX_Y1 - BOX_Y0, 0.01, (0, (BOX_Y0 + BOX_Y1) / 2, BOX_BACK), M.firebox)
        F.box('box_floor', 2 * BOX_X, 0.01, FP_Z1 - BOX_BACK, (0, BOX_Y0 + 0.005, (BOX_BACK + FP_Z1) / 2), M.firebox)
        for s in (-1, 1):
            F.box(f'box_side{s}', 0.01, BOX_Y1 - BOX_Y0, FP_Z1 - BOX_BACK, (s * (BOX_X - 0.005), (BOX_Y0 + BOX_Y1) / 2,
                  (BOX_BACK + FP_Z1) / 2), M.firebox)
        F.box('box_roof', 2 * BOX_X, 0.01, FP_Z1 - BOX_BACK, (0, BOX_Y1 - 0.005, (BOX_BACK + FP_Z1) / 2), M.firebox)
        fw = 0.03
        for s in (-1, 1):
            F.box(f'frame_side{s}', fw, BOX_Y1 - BOX_Y0 + 2 * fw, 0.012, (s * (BOX_X + fw / 2), (BOX_Y0 + BOX_Y1) / 2, FP_Z1 + 0.006),
                  M.firebox, bevel=0.003)
        for y in (BOX_Y0 - fw / 2, BOX_Y1 + fw / 2):
            F.box(f'frame{y}', 2 * BOX_X, fw, 0.012, (0, y, FP_Z1 + 0.006), M.firebox, bevel=0.003)
        # the log bed: three logs on a strip of embers, which glows at night as the fire's light comes on
        F.box('embers', 0.84, 0.03, 0.08, (0, BOX_Y0 + 0.02, LOG_Z + 0.01), M.ember, bevel=0.01)
        for i, (x, length, turn) in enumerate(((-0.2, 0.42, 4), (0.2, 0.4, -5), (0.0, 0.36, 0))):
            y = LOG_Y + (0.03 if i == 2 else 0.0)
            F.cyl(f'log{i}', 0.04, length, (x, y, LOG_Z - (0.03 if i == 2 else 0.0)), M.logs, rot=(0, 90, turn), verts=12)
        # the floor cushion in front, where residents sit cross-legged facing the fire (furniture.json seats: 0.14 high, the hips 0.36 out)
        F.box('cushion', 0.56, 0.12, 0.4, (0, 0.07, HEARTH_Z), M.hearth_cushion, bevel=0.05, bevel_segments=4)
        F.box('cushion_piping', 0.5, 0.014, 0.34, (0, 0.13, HEARTH_Z), M.hearth_piping, bevel=0.006)
        F.cyl('cushion_button', 0.016, 0.012, (0, 0.137, HEARTH_Z), M.hearth_piping, verts=10)
        # the mantel, a candle vase and a picture on it
        F.box('mantel', 1.76, 0.06, 0.37, (0, 1.0, WALL + 0.185), M.mantel, bevel=0.012)
        F.cone('vase', 0.06, 0.04, 0.26, (-0.6, 1.16, -0.32), M.candle, verts=20)
        F.box('frame_pic', 0.22, 0.28, 0.025, (0.55, 1.17, -0.37), M.picture, rot=(-8, 0, 0), bevel=0.006)
        F.box('print', 0.17, 0.22, 0.004, (0.55, 1.172, -0.354), M.art_print, rot=(-8, 0, 0))
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
             motions={'uses': {'listen': 'tray', 'dance': 'tray'}, 'hold': 1.2,
                      'loops': {'listen': 'pulse', 'dance': 'pulse'}},
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
