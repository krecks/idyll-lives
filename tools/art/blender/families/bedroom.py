# The bedroom's beds (BR1): the double bed (`model.bed`), the single (`model.bedSingle`), the bunk bed
# (`model.bedBunk`), the futon (`model.bedFuton`), the upholstered king (`model.bedKing`), the oak double
# (`model.bedDouble`), the upholstered queen (`model.bedQueenUpholstered`), the four-poster
# (`model.bedCanopy`) and the daybed (`model.bedDaybed`), over their primitive-part placeholders
# (`furniture.py`, `catalog.py`, `catalog2.py`), which stay the fallback. Notes: bedroom.md.
#
# Each bed's mattress is centred on its footprint, its top at the height the renderer lays sleepers
# at (characters.ts `BEDS`; `BUNKS` for the bunk's two). Its head end stands on the wall's face (depth
# -0.93, `WALL`): headboards, posts and rails stop there, thinner or a few cm forward, so the bed's bounds
# stand a little forward of the middle (the Model's `centre`). The game lays its own quilt over each sleeper on
# the beds in `BLANKET_BEDS`, so a bed's duvet isn't a second cover over them: its top third (`duvet_flap`)
# folds back over the rest toward the foot while a resident sleeps or naps (`turn_down`), flattening as it
# goes so the quilt hides it, and back up as they get up. The futon is a sofa at rest whose back folds down flat into the bed (`unfold`), held open
# like the wall bed (content `leaves` / `when`); the daybed is static (a sofa by day).
# Sleepers lie centred, head about 0.84 m behind the middle and feet 0.86 m in front: headboards keep their
# faces behind -0.87, pillows and the duvet fit around that.
#
# Built in game coordinates (`F`: x, height, depth; front +depth).
#
# Run:  python3 -B tools/art/blender/build.py bedroom [--only bed] [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # shared
    mattress=('#F7F7F5', 'fabric'),
    sheet=('#FFFFFF', 'fabric'),       # the sheet turned down over the duvet's top edge
    pillow=('#F6F4EF', 'fabric'),
    legs_teak=('#B08E66', 'wood'),
    # the double bed (`bed`) and the king: upholstered in linen
    frame_linen=('#D9D4CA', 'fabric'),
    headboard_linen=('#D3CDC1', 'fabric'),
    bedding_cream=('#F1F0EC', 'fabric'),
    throw_sage=('#9DAF8C', 'fabric'),  # the runner across the foot
    # the single, the bunk and the oak double: oak
    frame_oak=('#CDB08A', 'wood'),
    headboard_oak=('#CDB08A', 'wood'),
    headboard_teak=('#B08E66', 'wood'),  # the oak double's slats
    bedding_sage=('#9DAF8C', 'fabric'),
    bedding_teal=('#4E7A7F', 'fabric'),
    bedding_blue=('#9DB7C9', 'fabric'),
    # the futon: pale ash
    frame_ash=('#E3CFA8', 'wood'),
    headboard_ash=('#E3CFA8', 'wood'),
    bedding_futon=('#F4F2EE', 'fabric'),
    quilt_oat=('#E7E0D4', 'fabric'),   # folded on the seat
    # the upholstered queen
    frame_stone=('#B9B2A6', 'fabric'),
    headboard_stone=('#C4BDB1', 'fabric'),
    bedding_oat=('#E9DCC9', 'fabric'),
    # the four-poster: dark walnut, a linen canopy
    frame_walnut=('#4A3426', 'wood'),
    headboard_walnut=('#4A3426', 'wood'),
    bedding_rust=('#B5653F', 'fabric'),
    canopy=('#F2EBDD', 'fabric'),
    # the daybed: painted white
    frame_white=('#F1F0EC', 'satin'),
    back_white=('#F1F0EC', 'satin'),
    cushion_rose=('#E4B8AE', 'fabric'),
)

F = G.GameFrame()

DUVET = 0.05      # the duvet's top above the mattress top
FLAP = 0.045      # the flap's thickness at rest
# The game's quilt over a sleeper lies 10 cm above the mattress off the body, times the sleeper's size (8.5 cm
# for the smallest), and rounds down to 6 cm over its outer 8 cm. Folded, the flap must stay under it, so the
# clip also flattens it to this much of its thickness (its top, the sheet's, 7.5 cm above the mattress) and,
# where the quilt's edge would cross it, narrows it to 8 cm inside that edge (`_turn_down`).
FOLDED = 0.4
QUILT_EDGE = {1: 0.46, 2: 1.0}  # the quilt's edge from the bed's middle: a one-tile bed's, a two-tile bed's outer one
SECONDS = 1.0     # the duvet turns down this fast, as the resident gets in
WALL = -0.93      # the wall's face in a 2-deep footprint: the head end's back stands on it


def _settle():
    """Stands the bed on the floor where it was built (its mattress centred, its head end on the wall's
    face) and prints the centre of its bounds, the Model's `centre`."""
    lo, hi = G.measure()
    print(f'[bedroom] centre (game x, depth) {-(lo[0] + hi[0]) / 2:.4f}, {-(lo[1] + hi[1]) / 2:.4f}')
    G.settle(centre=False)


def _pillow(name, w, h, d, x, top, z, tilt=8):
    """A pillow on a mattress top `top`, its head end raised a little."""
    F.box(name, w, h, d, (x, top + h / 2 - 0.01, z), M.pillow, rot=(tilt, 0, 0), bevel=min(h, d) * 0.42, bevel_segments=3)


def _duvet(top, half, z_fold, z_foot, drape, material):
    """The duvet's lower two thirds: from the fold to the foot, draped down the sides and the foot end."""
    over = 0.025
    y0, y1 = top - drape, top + DUVET
    F.box('duvet', 2 * (half + over), y1 - y0, z_foot + over - z_fold, (0, (y0 + y1) / 2, (z_fold + z_foot + over) / 2),
          material, bevel=0.024, bevel_segments=3)


def _flap(top, half, z_top, z_fold, material):
    """The duvet's top third, flat on the mattress with the sheet turned down over its head edge; one part
    hinged across its fold on the duvet's top, so turning it 180 degrees lays it over the rest."""
    with G.new_parts() as parts:
        y = top + DUVET - FLAP / 2
        F.box('flap', 2 * (half + 0.012), FLAP, z_fold - z_top, (0, y, (z_top + z_fold) / 2), material,
              bevel=0.02, bevel_segments=3)
        F.box('flap_sheet', 2 * (half + 0.016), FLAP + 0.008, 0.13, (0, y, z_top + 0.055), M.sheet,
              bevel=0.022, bevel_segments=3)
    # A hair above the duvet's top, so the flap folded over doesn't lie in its plane.
    return G.hinged(G.join(parts, 'duvet_flap'), F.at(0, top + DUVET + 0.003, z_fold))


def _bedding(top, half, z_top, z_foot, drape, material, fold=None):
    """A duvet whose top third turns down: the still lower part (left in the scene, joined with the bed)
    and the flap, returned."""
    z_fold = fold if fold is not None else z_top + (z_foot - z_top) / 3
    _duvet(top, half, z_fold, z_foot, drape, material)
    return z_fold


def _turn_down(flap, clip, half, tiles):
    """Keys `clip`: the flap turns 180 degrees over the rest of the duvet and, as it goes, flattens to `FOLDED`
    of its thickness (about its hinge, so it stays on the duvet) and narrows to keep 8 cm inside the edge of the
    game's quilt (`QUILT_EDGE`), so the quilt over a sleeper hides it. `half`: the bedding's half width."""
    narrow = min(1.0, (QUILT_EDGE[tiles] - 0.08) / (half + 0.016))
    G.swing(flap, clip, 180, seconds=SECONDS, axis='X')
    G.squash(flap, clip, (narrow, 1, FOLDED), seconds=SECONDS)


def _finish(name, flap_args, tiles=2):
    """Joins what's in the scene as the still bed, builds the flap, settles, and keys `turn_down`."""
    still = G.meshes()
    G.join(still, name, origin=(0, 0, 0))
    flap = _flap(*flap_args)
    _settle()
    _turn_down(flap, 'turn_down', flap_args[1], tiles)


def _legs_round(xs, zs, h, material, r=0.026):
    for x in xs:
        for z in zs:
            F.cone(f'leg{x}{z}', r * 0.75, r, h, (x, h / 2, z), material, verts=12)


def _legs_square(xs, zs, h, material, w=0.06):
    for x in xs:
        for z in zs:
            F.box(f'leg{x}{z}', w, h, w, (x, h / 2, z), material, bevel=0.006)


# ---------------------------------------------------------------- the double bed (`bed`)

def bed():
    top = 0.58  # BEDS['bed']
    _legs_round((-0.84, 0.84), (-0.82, 0.88), 0.12, M.legs_teak)
    F.box('base', 1.82, 0.28, 1.83, (0, 0.26, 0.025), M.frame_linen, bevel=0.04, bevel_segments=3)
    # headboard: a padded back with four channels, its back on the wall's face
    F.box('headboard', 1.9, 1.03, 0.035, (0, 0.615, WALL + 0.0175), M.headboard_linen, bevel=0.015, bevel_segments=3)
    for i, x in enumerate((-0.66, -0.22, 0.22, 0.66)):
        F.box(f'channel{i}', 0.42, 0.74, 0.04, (x, 0.72, -0.89), M.headboard_linen, bevel=0.018, bevel_segments=3)
    F.box('mattress', 1.72, 0.2, 1.78, (0, top - 0.1, 0.03), M.mattress, bevel=0.04, bevel_segments=3)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.68, 0.15, 0.4, s * 0.42, top, -0.66)
    z_top, z_foot = -0.42, 0.92
    z_fold = _bedding(top, 0.86, z_top, z_foot, 0.17, M.bedding_cream)
    # the runner across the foot, over the duvet and down its sides
    F.box('runner', 1.83, 0.22, 0.38, (0, top + DUVET + 0.015 - 0.11, 0.72), M.throw_sage, bevel=0.012, bevel_segments=2)
    _finish('bed', (top, 0.86, z_top, z_fold, M.bedding_cream))


# ---------------------------------------------------------------- the single bed

def bedSingle():
    top = 0.5  # BEDS['bedSingle']
    head = WALL + 0.03  # the head end's posts and boards, their backs on the wall's face
    for s in (-1, 1):
        x = s * 0.45
        F.box(f'post_head{s}', 0.06, 0.95, 0.06, (x, 0.475, head), M.frame_oak, bevel=0.008)
        F.box(f'post_foot{s}', 0.06, 0.62, 0.06, (x, 0.31, 0.94), M.frame_oak, bevel=0.008)
        F.box(f'rail{s}', 0.04, 0.18, 1.82, (x, 0.25, 0), M.frame_oak, bevel=0.006)
    F.box('rail_head', 0.84, 0.18, 0.04, (0, 0.25, head), M.frame_oak, bevel=0.006)
    F.box('rail_foot', 0.84, 0.18, 0.04, (0, 0.25, 0.94), M.frame_oak, bevel=0.006)
    # headboard: a top rail over five slats; a low footboard
    F.box('head_top', 0.9, 0.08, 0.05, (0, 0.9, head), M.headboard_oak, bevel=0.012)
    F.box('head_low', 0.84, 0.05, 0.035, (0, 0.47, head), M.headboard_oak, bevel=0.006)
    for i in range(5):
        F.box(f'slat{i}', 0.08, 0.38, 0.022, (-0.28 + i * 0.14, 0.67, head), M.headboard_oak, bevel=0.005)
    F.box('footboard', 0.84, 0.24, 0.03, (0, 0.47, 0.94), M.headboard_oak, bevel=0.008)
    F.box('foot_top', 0.9, 0.04, 0.05, (0, 0.6, 0.94), M.headboard_oak, bevel=0.01)
    F.box('mattress', 0.84, 0.18, 1.8, (0, top - 0.09, 0), M.mattress, bevel=0.035, bevel_segments=3)
    _pillow('pillow', 0.62, 0.13, 0.34, 0, top, -0.71)
    z_top, z_foot = -0.3, 0.885
    z_fold = _bedding(top, 0.42, z_top, z_foot, 0.14, M.bedding_sage)
    _finish('bedSingle', (top, 0.42, z_top, z_fold, M.bedding_sage), tiles=1)


# ---------------------------------------------------------------- the bunk bed

LOWER, UPPER = 0.44, 1.38  # characters.ts BUNKS['bedBunk']
BUNK_ENDS = {-1: WALL + 0.03, 1: 0.93}  # the depths of the posts and end boards: the head end's backs on the wall's face


def _bunk(name, top, guard):
    """One bunk: its rails, mattress, pillow and end boards; the upper one with a guard rail."""
    y = top - 0.16 - 0.06
    for s in (-1, 1):
        F.box(f'{name}_rail{s}', 0.04, 0.12, 1.8, (s * 0.45, y, 0), M.frame_oak, bevel=0.006)
        F.box(f'{name}_end{s}', 0.86, 0.12, 0.04, (0, y, BUNK_ENDS[s]), M.frame_oak, bevel=0.006)
        h = 0.36 if guard else 0.3
        F.box(f'{name}_board{s}', 0.84, h, 0.025, (0, top - 0.13 + h / 2, BUNK_ENDS[s]), M.headboard_oak, bevel=0.006)
    F.box(f'{name}_mattress', 0.84, 0.16, 1.8, (0, top - 0.08, 0), M.mattress, bevel=0.03, bevel_segments=3)
    _pillow(f'{name}_pillow', 0.56, 0.11, 0.3, 0, top, -0.71)
    if guard:
        F.box('guard_back', 0.04, 0.05, 1.82, (-0.45, 1.6, 0), M.frame_oak, bevel=0.008)
        F.box('guard_front', 0.04, 0.05, 1.28, (0.45, 1.6, -0.26), M.frame_oak, bevel=0.008)
        F.box('guard_post', 0.035, 0.38, 0.035, (0.45, 1.41, 0.38), M.frame_oak, bevel=0.006)


def bedBunk():
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'post{sx}{sz}', 0.06, 1.9, 0.06, (sx * 0.45, 0.95, BUNK_ENDS[sz]), M.frame_oak, bevel=0.008)
    _bunk('lower', LOWER, False)
    _bunk('upper', UPPER, True)
    # the ladder on the right, at the foot end: two rails and four rungs
    for z in (0.42, 0.86):
        F.box(f'ladder{z}', 0.03, 1.62, 0.045, (0.465, 0.81, z), M.frame_oak, bevel=0.006)
    for i, y in enumerate((0.32, 0.62, 0.92, 1.22)):
        G.rod(f'rung{i}', F.at(0.465, y, 0.42), F.at(0.465, y, 0.86), 0.016, M.frame_oak, verts=8)
    z_top, z_foot = -0.28, 0.875
    # Each bunk's duvet turns down for the one who sleeps in it (`slot`: the lower bunk is 0, the upper 1).
    z_fold_upper = _bedding(UPPER, 0.42, z_top, z_foot, 0.12, M.bedding_teal)
    z_fold = _bedding(LOWER, 0.42, z_top, z_foot, 0.12, M.bedding_teal)
    G.join(G.meshes(), 'bedBunk', origin=(0, 0, 0))
    lower = _flap(LOWER, 0.42, z_top, z_fold, M.bedding_teal)
    upper = _flap(UPPER, 0.42, z_top, z_fold_upper, M.bedding_teal)
    upper.name = 'duvet_flap_upper'
    _settle()
    _turn_down(lower, 'turn_down', 0.42, 1)
    _turn_down(upper, 'turn_down_upper', 0.42, 1)


# ---------------------------------------------------------------- the futon

FUTON_TOP = 0.4      # BEDS['bedFuton']
FUTON_HINGE = -0.2   # the depth where the back folds up from the seat
FUTON_BACK = 66      # the back's lean at rest, degrees up from flat


def bedFuton():
    top = FUTON_TOP
    # a low ash platform: a frame round slats, on a recessed plinth
    F.box('plinth', 1.8, 0.04, 1.8, (0, 0.02, 0), M.frame_ash)
    # the frame's head end stands on the wall's face
    for s in (-1, 1):
        F.box(f'side{s}', 0.06, 0.16, 0.98 - WALL, (s * 0.95, 0.12, (WALL + 0.98) / 2), M.frame_ash, bevel=0.008)
        F.box(f'end{s}', 1.84, 0.16, 0.06, (0, 0.12, 0.95 if s > 0 else WALL + 0.03), M.frame_ash, bevel=0.008)
    for i in range(14):
        F.box(f'slat{i}', 1.84, 0.025, 0.08, (0, 0.1875, -0.86 + i * 0.132), M.frame_ash, bevel=0.004)
    F.box('headboard', 1.96, 0.46, 0.04, (0, 0.43, WALL + 0.02), M.headboard_ash, bevel=0.01)
    F.box('headboard_cap', 1.96, 0.035, 0.05, (0, 0.665, WALL + 0.025), M.headboard_ash, bevel=0.01)
    # the seat: the front of the futon
    F.box('seat', 1.86, top - 0.2, 0.93 - FUTON_HINGE, (0, (top + 0.2) / 2, (FUTON_HINGE + 0.93) / 2), M.bedding_futon,
          bevel=0.06, bevel_segments=3)
    F.box('quilt', 1.5, 0.06, 0.32, (0, top + 0.02, 0.7), M.quilt_oat, bevel=0.025, bevel_segments=3)
    G.join(G.meshes(), 'futon', origin=(0, 0, 0))
    # the back, built flat (the bed's head end) with the pillows on it, hinged on the seat's back edge
    with G.new_parts() as parts:
        z0 = WALL + 0.04  # in front of the headboard
        F.box('back', 1.86, top - 0.2, FUTON_HINGE - z0 - 0.01, (0, (top + 0.2) / 2, (z0 + FUTON_HINGE - 0.01) / 2),
              M.bedding_futon, bevel=0.06, bevel_segments=3)
        for s in (-1, 1):
            _pillow(f'pillow{s}', 0.66, 0.13, 0.34, s * 0.45, top, -0.71, tilt=4)
    back = G.hinged(G.join(parts, 'futon_back'), F.at(0, 0.3, FUTON_HINGE))
    # Built flat; leaning up is the rest pose (the clip's first key): its head end up, its top to the front.
    G.turn(back, FUTON_BACK, 'X')
    _settle()
    G.swing(back, 'unfold', -FUTON_BACK, seconds=SECONDS, axis='X')


# ---------------------------------------------------------------- the upholstered king

def bedKing():
    top = 0.59  # BEDS['bedKing']
    _legs_square((-0.9, 0.9), (-0.8, 0.9), 0.1, M.legs_teak, w=0.055)
    F.box('base', 1.96, 0.3, 1.82, (0, 0.25, 0.06), M.frame_linen, bevel=0.05, bevel_segments=3)
    # a tall padded headboard with five channels
    F.box('headboard', 1.98, 1.36, 0.04, (0, 0.72, WALL + 0.02), M.headboard_linen, bevel=0.018, bevel_segments=3)
    for i in range(5):
        F.box(f'channel{i}', 0.37, 0.88, 0.04, (-0.74 + i * 0.37, 0.89, -0.89), M.headboard_linen, bevel=0.018, bevel_segments=3)
    # wings each side, turned in a little round the pillows, their backs on the wall's face
    for s in (-1, 1):
        F.box(f'wing{s}', 0.08, 1.3, 0.2, (s * 0.935, 0.69, -0.823), M.headboard_linen, rot=(0, 0, s * 14),
              bevel=0.035, bevel_segments=3)
    F.box('mattress', 1.86, 0.22, 1.8, (0, top - 0.11, 0.04), M.mattress, bevel=0.045, bevel_segments=3)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.74, 0.16, 0.4, s * 0.45, top, -0.66)
    z_top, z_foot = -0.4, 0.945
    z_fold = _bedding(top, 0.93, z_top, z_foot, 0.18, M.bedding_cream)
    # (just round the duvet's sides: its ends stay under the edges of the quilts over sleepers)
    F.box('runner', 1.93, 0.23, 0.38, (0, top + DUVET + 0.015 - 0.115, 0.75), M.throw_sage, bevel=0.012, bevel_segments=2)
    _finish('bedKing', (top, 0.93, z_top, z_fold, M.bedding_cream))


# ---------------------------------------------------------------- the oak double

DOUBLE_TOP = 0.52  # its placeholder's mattress top (characters.ts BEDS['bedDouble'])


def bedDouble():
    top = DOUBLE_TOP
    _legs_square((-0.75, 0.75), (-0.84, 0.9), 0.1, M.frame_oak)
    F.box('frame', 1.62, 0.22, 1.84, (0, 0.21, 0.04), M.frame_oak, bevel=0.012)
    # headboard: oak posts and rails round teak slats, their backs on the wall's face
    head = WALL + 0.035
    for s in (-1, 1):
        F.box(f'post{s}', 0.07, 0.95, 0.07, (s * 0.775, 0.475, head), M.frame_oak, bevel=0.01)
    F.box('head_top', 1.62, 0.08, 0.07, (0, 0.91, head), M.frame_oak, bevel=0.012)
    F.box('head_low', 1.48, 0.06, 0.05, (0, 0.35, head), M.frame_oak, bevel=0.008)
    for i in range(8):
        F.box(f'slat{i}', 0.1, 0.5, 0.025, (-0.69 + i * 0.197, 0.62, head), M.headboard_teak, bevel=0.006)
    F.box('mattress', 1.56, 0.2, 1.79, (0, top - 0.1, 0.025), M.mattress, bevel=0.04, bevel_segments=3)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.69, 0.14, 0.34, s * 0.405, top, -0.68)
    z_top, z_foot = -0.36, 0.92
    z_fold = _bedding(top, 0.78, z_top, z_foot, 0.2, M.bedding_blue)
    _finish('bedDouble', (top, 0.78, z_top, z_fold, M.bedding_blue))


# ---------------------------------------------------------------- the upholstered queen

def bedQueenUpholstered():
    top = DOUBLE_TOP
    _legs_square((-0.8, 0.8), (-0.84, 0.9), 0.08, M.legs_teak, w=0.05)
    F.box('base', 1.72, 0.24, 1.82, (0, 0.2, 0.05), M.frame_stone, bevel=0.03, bevel_segments=3)
    # a tall headboard of five rounded channels
    F.box('headboard', 1.72, 1.28, 0.04, (0, 0.66, WALL + 0.02), M.headboard_stone, bevel=0.018, bevel_segments=3)
    for i in range(5):
        F.box(f'channel{i}', 0.3, 0.86, 0.045, (-0.64 + i * 0.32, 0.85, -0.8925), M.headboard_stone, bevel=0.02, bevel_segments=3)
    F.box('mattress', 1.66, 0.2, 1.78, (0, top - 0.1, 0.03), M.mattress, bevel=0.04, bevel_segments=3)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.74, 0.15, 0.34, s * 0.43, top, -0.68)
    z_top, z_foot = -0.36, 0.935
    z_fold = _bedding(top, 0.855, z_top, z_foot, 0.2, M.bedding_oat)
    _finish('bedQueenUpholstered', (top, 0.855, z_top, z_fold, M.bedding_oat))


# ---------------------------------------------------------------- the four-poster

def _drape(name, x, z, inward):
    """A curtain gathered at a post, hanging down the side: three tapering folds, flat across the side,
    and a tie-back. `inward`: the depth direction along the side, away from the post."""
    for k, dz in enumerate((-0.045, 0.0, 0.04)):
        r = 0.07 if k != 1 else 0.06
        F.cone(f'{name}_{k}', r, 0.032, 1.9, (x, 1.05, z + inward * dz), M.canopy, scale=(0.4, 1, 1), verts=12)
    F.cyl(f'{name}_tie', 0.056, 0.035, (x, 1.0, z), M.canopy, scale=(0.5, 1, 1), verts=12)


def bedCanopy():
    top = DOUBLE_TOP
    # the posts' depths: the head end's feet, crest and valance stand on the wall's face
    posts = {-1: WALL + 0.045, 1: 0.92}
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'post{sx}{sz}', 0.07, 2.11, 0.07, (sx * 0.82, 1.055, posts[sz]), M.frame_walnut, bevel=0.01)
            F.box(f'foot{sx}{sz}', 0.09, 0.12, 0.09, (sx * 0.82, 0.06, posts[sz]), M.frame_walnut, bevel=0.012)
            F.box(f'finial{sx}{sz}', 0.05, 0.05, 0.05, (sx * 0.82, 2.11, posts[sz]), M.frame_walnut, bevel=0.02, bevel_segments=3)
        F.box(f'rail_top{sx}', 0.06, 0.06, 1.77, (sx * 0.82, 2.06, 0), M.frame_walnut, bevel=0.008)
        F.box(f'rail_end{sx}', 1.57, 0.06, 0.06, (0, 2.06, posts[sx]), M.frame_walnut, bevel=0.008)
    F.box('frame', 1.6, 0.22, 1.78, (0, 0.21, 0), M.frame_walnut, bevel=0.012)
    # a panelled headboard under a crest rail
    F.box('headboard', 1.57, 0.6, 0.04, (0, 0.68, WALL + 0.02), M.headboard_walnut, bevel=0.008)
    F.box('crest', 1.64, 0.07, 0.07, (0, 1.0, WALL + 0.035), M.headboard_walnut, bevel=0.012)
    for i, x in enumerate((-0.52, 0, 0.52)):
        F.box(f'panel{i}', 0.42, 0.4, 0.016, (x, 0.7, WALL + 0.043), M.headboard_walnut, bevel=0.006)
    # the canopy: a linen valance round the open top (sleepers stay in view from above), a curtain
    # gathered at each post
    for s in (-1, 1):
        F.box(f'valance_side{s}', 0.012, 0.2, 1.77, (s * 0.85, 1.99, 0), M.canopy, bevel=0.004)
        F.box(f'valance_end{s}', 1.57, 0.2, 0.012, (0, 1.99, 0.95 if s > 0 else WALL + 0.006), M.canopy, bevel=0.004)
    for sx in (-1, 1):
        for sz in (-1, 1):
            _drape(f'drape{sx}{sz}', sx * 0.83, sz * 0.8, -sz)
    F.box('mattress', 1.56, 0.2, 1.76, (0, top - 0.1, 0), M.mattress, bevel=0.04, bevel_segments=3)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.69, 0.14, 0.34, s * 0.405, top, -0.68)
    # (the duvet's foot 1.5 cm past the mattress's end, not in its plane)
    z_top, z_foot = -0.36, 0.87
    z_fold = _bedding(top, 0.775, z_top, z_foot, 0.2, M.bedding_rust)
    _finish('bedCanopy', (top, 0.775, z_top, z_fold, M.bedding_rust))


# ---------------------------------------------------------------- the daybed

DAYBED_TOP = 0.51  # its placeholder's mattress top (characters.ts BEDS['bedDaybed'])


def bedDaybed():
    top = DAYBED_TOP
    # the ends' depths: the head end's posts and arm stand on the wall's face, the base and the mattress
    # between the arms
    ends = {-1: WALL + 0.03, 1: 0.95}
    z0, z1 = ends[-1] + 0.03, 0.92
    # four posts, the back's taller; a base between them
    for sx in (-1, 1):
        h = 0.84 if sx < 0 else 0.74
        for sz in (-1, 1):
            F.box(f'post{sx}{sz}', 0.06, h, 0.06, (sx * 0.45, h / 2, ends[sz]), M.frame_white, bevel=0.008)
    F.box('base', 0.9, 0.23, z1 - z0, (0, 0.215, (z0 + z1) / 2), M.frame_white, bevel=0.012)
    # the back along the left side: a top rail over spindles (the headboard)
    F.box('back_rail', 0.06, 0.05, 0.93 - ends[-1], (-0.45, 0.805, (ends[-1] + 0.93) / 2), M.back_white, bevel=0.01)
    for i in range(15):
        F.box(f'back_spindle{i}', 0.03, 0.47, 0.03, (-0.45, 0.565, -0.84 + i * 0.12), M.back_white, bevel=0.004)
    # an arm at each end, the same
    for s in (-1, 1):
        F.box(f'arm_rail{s}', 0.86, 0.05, 0.06, (0, 0.705, ends[s]), M.frame_white, bevel=0.01)
        F.box(f'arm_low{s}', 0.86, 0.06, 0.05, (0, 0.3, ends[s]), M.frame_white, bevel=0.006)
        for i in range(5):
            F.box(f'arm_spindle{s}{i}', 0.03, 0.37, 0.03, (-0.3 + i * 0.15, 0.495, ends[s]), M.frame_white, bevel=0.004)
    # the mattress in a fitted cover (the bedding), two back cushions clear of a sleeper, a pillow
    F.box('mattress', 0.86, top - 0.33, z1 - z0, (0.02, (top + 0.33) / 2, (z0 + z1) / 2), M.bedding_sage, bevel=0.035, bevel_segments=3)
    # (thin and close to the back, leaning on it, so a sleeper's shoulder and hip stay clear of them)
    for i, z in enumerate((-0.32, 0.3)):
        F.box(f'cushion{i}', 0.11, 0.34, 0.56, (-0.385, top + 0.16, z), M.cushion_rose, rot=(0, -8, 0),
              bevel=0.045, bevel_segments=3)
    _pillow('pillow', 0.56, 0.13, 0.3, 0.06, top, -0.72)
    G.join_all('bedDaybed')
    _settle()


PAINT = ('Frame', 'Headboard', 'Bedding')


def _paint(frame, headboard, bedding):
    return list(zip((frame, headboard, bedding), PAINT))


TURN_DOWN = {'uses': {'sleep': 'turn_down', 'nap': 'turn_down'}}
# The bunk turns down the duvet of the bunk slept in: the lower one's for slot 0, the upper one's for slot 1.
BUNKS = [{'clip': 'turn_down', 'slot': 0}, {'clip': 'turn_down_upper', 'slot': 1}]
BUNK_TURN_DOWN = {'uses': {'sleep': BUNKS, 'nap': BUNKS}}

FAMILY = G.Family(
    'bedroom', M,
    [G.Model('bed', bed, size=(1.9, 1.13, 1.878), key='model.bed', centre=(0.0, 0.009), motions=TURN_DOWN,
             paint=_paint('frame_linen', 'headboard_linen', 'bedding_cream')),
     G.Model('bedSingle', bedSingle, size=(0.96, 0.95, 1.9), key='model.bedSingle', centre=(0.0, 0.02), motions=TURN_DOWN,
             paint=_paint('frame_oak', 'headboard_oak', 'bedding_sage')),
     G.Model('bedBunk', bedBunk, size=(0.96, 1.9, 1.89), key='model.bedBunk', centre=(0.0, 0.015), motions=BUNK_TURN_DOWN,
             paint=_paint('frame_oak', 'headboard_oak', 'bedding_teal')),
     # Down while the futon is open (the sim's `open`), from the moment a use that opens it starts.
     G.Model('bedFuton', bedFuton, size=(1.96, 0.995, 1.91), key='model.bedFuton', centre=(0.0, 0.025),
             motions={'uses': {'sleep': 'unfold', 'nap': 'unfold', 'unfold': 'unfold'}, 'open': 'unfold'},
             paint=_paint('frame_ash', 'headboard_ash', 'bedding_futon')),
     G.Model('bedKing', bedKing, size=(1.98, 1.4, 1.9), key='model.bedKing', centre=(0.0, 0.02), motions=TURN_DOWN,
             paint=_paint('frame_linen', 'headboard_linen', 'bedding_cream')),
     G.Model('bedDouble', bedDouble, size=(1.62, 0.95, 1.888), key='model.bedDouble', centre=(0.0, 0.014), motions=TURN_DOWN,
             paint=_paint('frame_oak', 'headboard_teak', 'bedding_blue')),
     G.Model('bedQueenUpholstered', bedQueenUpholstered, size=(1.76, 1.3, 1.89), key='model.bedQueenUpholstered', centre=(0.0, 0.015),
             motions=TURN_DOWN, paint=_paint('frame_stone', 'headboard_stone', 'bedding_oat')),
     G.Model('bedCanopy', bedCanopy, size=(1.73, 2.135, 1.895), key='model.bedCanopy', centre=(0.0, 0.0175), motions=TURN_DOWN,
             paint=_paint('frame_walnut', 'headboard_walnut', 'bedding_rust')),
     G.Model('bedDaybed', bedDaybed, size=(0.96, 0.84, 1.91), key='model.bedDaybed', centre=(0.0, 0.025),
             paint=_paint('frame_white', 'back_white', 'bedding_sage'))],
    comment='Blender-built bedroom models (tools/art/blender/families/bedroom.py). Replaces the primitive-part '
            'placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a '
            'file fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The beds turn their duvet down (`turn_down`) while a resident sleeps or naps; the '
            'futon unfolds into a bed (`unfold`) and stays open.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
