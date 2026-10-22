# The bedroom's beds (BR1): the double bed (`model.bed`), the single (`model.bedSingle`), the bunk bed
# (`model.bedBunk`), the futon (`model.bedFuton`), the upholstered king (`model.bedKing`), the oak double
# (`model.bedDouble`), the upholstered queen (`model.bedQueenUpholstered`), the four-poster
# (`model.bedCanopy`) and the daybed (`model.bedDaybed`). Notes: bedroom.md.
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
# Run:  cargo art bedroom [--only bed] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402
import gamekit as G  # noqa: E402
from mathutils import Vector, noise  # noqa: E402

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


def _pillow(name, w, h, d, x, top, z, tilt=8, material=None, seed=0.0):
    """A pillow on a mattress top `top`, `h` thick in the middle: plump, thinning to its seams, its head end
    raised a little and a little crumpled."""
    p = F.soft(name, w, h * 0.42, d, (x, top + h / 2 - 0.01, z), material or M.pillow, rot=(tilt, 0, 0),
               round=min(h * 0.2, 0.03), puff=h * 0.29, flat=2)
    return G.jitter(p, 0.004, size=0.12, seed=seed + x * 7, up=0.4)


def _mattress(name, w, h, d, top, z, x=0.0, material=None, puff=0.006, piping=0.007, round=0.035, flat=3):
    """A mattress `w` x `d`, `h` deep, its top (the middle of it, a hair puffed) at `top`, where sleepers lie:
    rounded, with a piped seam round its top and bottom."""
    return F.soft(name, w, h - 2 * puff, d, (x, top - h / 2, z), material or M.mattress, round=round, puff=puff,
                  flat=flat, piping=piping)


def _duvet(top, half, z_fold, z_foot, drape, material, seed=0.0, flat=5):
    """The duvet's lower two thirds: from the fold to the foot, draped down the sides and the foot end, puffed on
    top (its middle at `DUVET`) and a little rumpled."""
    over, puff = 0.028, 0.01
    y0, y1 = top - drape, top + DUVET
    d = F.soft('duvet', 2 * (half + over), y1 - y0 - 2 * puff, z_foot + over - z_fold,
               (0, (y0 + y1) / 2, (z_fold + z_foot + over) / 2), material, round=0.035, puff=puff, flat=flat)
    _rumple(d, 0.004, top, seed)
    return d


def _rumple(obj, amount, above, seed, size=0.2):
    """Rumples a cover's top: its vertices above height `above` moved up or down by smooth noise, up to `amount`,
    never sideways, so its bounds (the size and centre checks) don't move. (G.jitter reads the object's world matrix
    before the scene has updated it, so where its noise lands depends on what ran before.)"""
    bpy.context.view_layer.update()
    mw = obj.matrix_world
    for v in obj.data.vertices:
        p = mw @ v.co
        if p.z > above:
            v.co.z += noise.noise(Vector((p.x / size + seed, p.y / size - seed, seed))) * amount
    obj.data.update()


def _flap(top, half, z_top, z_fold, material):
    """The duvet's top third, flat on the mattress with the sheet turned down over its head edge; one part
    hinged across its fold on the duvet's top, so turning it 180 degrees lays it over the rest."""
    with G.new_parts() as parts:
        y = top + DUVET - FLAP / 2
        F.soft('flap', 2 * (half + 0.012), FLAP - 0.01, z_fold - z_top, (0, y, (z_top + z_fold) / 2), material,
               round=0.018, puff=0.005, flat=4)
        F.soft('flap_sheet', 2 * (half + 0.016), FLAP + 0.008, 0.13, (0, y, z_top + 0.055), M.sheet,
               round=0.022)
    # A hair above the duvet's top, so the flap folded over doesn't lie in its plane.
    return G.hinged(G.join(parts, 'duvet_flap'), F.at(0, top + DUVET + 0.003, z_fold))


def _bedding(top, half, z_top, z_foot, drape, material, fold=None, seed=0.0, flat=5):
    """A duvet whose top third turns down: the still lower part (left in the scene, joined with the bed)
    and the flap, returned."""
    z_fold = fold if fold is not None else z_top + (z_foot - z_top) / 3
    _duvet(top, half, z_fold, z_foot, drape, material, seed, flat)
    return z_fold


def _runner(name, x, y_top, z, depth, hang, material):
    """A runner laid across the foot of the bed over the duvet, `depth` wide, hanging `hang` down each side to
    game x = ±`x`: one cloth swept over the bed, its top `y_top`."""
    t, rr = 0.012, 0.035
    yc = y_top - t / 2
    pts = [(-x, yc - hang, z)]
    pts += [(-x + rr - rr * math.cos(math.radians(a)), yc - rr + rr * math.sin(math.radians(a)), z) for a in range(0, 91, 15)]
    pts += [(-x * k, yc, z) for k in (0.5, 0.0, -0.5)]
    pts += [(x - rr + rr * math.sin(math.radians(a)), yc - rr + rr * math.cos(math.radians(a)), z) for a in range(0, 91, 15)]
    pts += [(x, yc - hang, z)]
    r = F.sweep(name, pts, None, material, profile=G.rounded_rect(t, depth, 0.005, corner=1), up=(1, 0, 0))
    return G.jitter(r, 0.003, size=0.1, seed=z, up=0.2)


def _channel(name, x, w, y0, y1, z, t, material, dome=False, verts=16, rows=4):
    """An upholstered channel on a headboard: a padded column `w` wide from height y0 to y1, its back at game
    depth `z` and its front `t` in front of it, plump across and tucked in at its seams; `dome` rounds its top to a
    half circle, else its top corners are softly rounded."""
    bx, by = -x, -(z + t / 2)
    hw = w / 2
    r = hw if dome else min(0.06, hw * 0.5)
    secs = [G.ring(hw * 0.92, t / 2 * 0.5, y0, verts, square=3.0, at=(bx, by))]
    for k in range(rows + 1):
        secs.append(G.ring(hw, t / 2, y0 + 0.015 + (y1 - r - y0 - 0.015) * k / rows, verts, square=3.0, at=(bx, by)))
    for a in (25, 50, 70, 84):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        rx = hw * c if dome else hw - r + r * c
        secs.append(G.ring(max(rx, 0.004), t / 2 * (0.55 + 0.45 * c), y1 - r + r * s, verts, square=3.0, at=(bx, by)))
    secs.append(G.ring(0.004 if dome else hw - r, t / 2 * 0.4, y1, verts, square=3.0, at=(bx, by)))
    return G.loft(name, secs, material)


def _turn_down(flap, clip, half, tiles):
    """Keys `clip`: the flap turns 180 degrees over the rest of the duvet and, as it goes, flattens to `FOLDED`
    of its thickness (about its hinge, so it stays on the duvet) and narrows to keep 8 cm inside the edge of the
    game's quilt (`QUILT_EDGE`), so the quilt over a sleeper hides it. `half`: the bedding's half width."""
    narrow = min(1.0, (QUILT_EDGE[tiles] - 0.08) / (half + 0.016))
    G.swing(flap, clip, 180, seconds=SECONDS, axis='X')
    G.squash(flap, clip, (narrow, 1, FOLDED), seconds=SECONDS)
    G.squash(flap, clip, (1, ROLL, 1), seconds=SECONDS, back=True)


# Half way over, the flap is gathered to this much of its length along the bed (it springs out again as it lies
# down), so a duvet turned back rolls over its fold as cloth does, a low bulge, never a board standing up.
ROLL = 0.2


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
    """Square legs tapering to their feet, `w` across at the top."""
    r = w / math.sqrt(2)
    for x in xs:
        for z in zs:
            F.cone(f'leg{x}{z}', r * 0.72, r, h, (x, h / 2, z), material, verts=4, rot=(0, 0, 45), smooth=False)


def _post(name, x, z, h, w, material, cap='ball', y0=0.0):
    """A square post from y0 up to `h`, `w` across: its shaft, a collar under the top and a cap: a ball, a pyramid
    or a flat moulded block."""
    ch = {'ball': w * 0.8, 'pyramid': w * 0.55, 'flat': w * 0.3}[cap]
    shaft = h - ch - 0.018
    F.box(f'{name}_shaft', w, shaft - y0, w, (x, (y0 + shaft) / 2, z), material, bevel=0.006, bevel_segments=1)
    F.box(f'{name}_collar', w + 0.008, 0.018, w + 0.008, (x, shaft + 0.009, z), material, bevel=0.004, bevel_segments=1)
    if cap == 'ball':
        F.cyl(f'{name}_neck', w * 0.28, ch * 0.3, (x, shaft + 0.018 + ch * 0.15, z), material, verts=10)
        F.sphere(f'{name}_ball', ch * 0.38, (x, h - ch * 0.38, z), material, segments=12, rings=7)
    elif cap == 'pyramid':
        F.cone(f'{name}_cap', (w + 0.008) / math.sqrt(2), 0.004, ch, (x, h - ch / 2, z), material, verts=4,
               rot=(0, 0, 45), smooth=False)
    else:
        F.box(f'{name}_cap', w + 0.004, ch, w + 0.004, (x, h - ch / 2, z), material, bevel=0.008, bevel_segments=1)


def _board(name, w, y0, y1, z, t, material, arch=0.0, holes=(), x=0.0, bevel=0.006, n=14):
    """A board `w` wide from y0 up to y1 at game depth `z`, `t` thick, its top arched up `arch` in the middle (its
    ends at y1 - arch); `holes`: (x, y, w, h) rounded slots cut through it."""
    top = [(w / 2 - w * i / n, y1 - arch + arch * math.cos(math.pi * (0.5 - i / n))) for i in range(n + 1)]
    outline = [(-w / 2, y0), (w / 2, y0)] + top
    loops = [[(hx + u, hy + v) for u, v in G.rounded_rect(hw, hh, min(hw, hh) * 0.5, corner=3)] for hx, hy, hw, hh in holes]
    return F.extrude(name, [(u, v) for u, v in outline], t, (x, 0, z), material, holes=loops, bevel=bevel,
                     bevel_segments=1)


def _tufts(name, xs, zs, y, material, r=0.014):
    """Tufting buttons across a mattress or cushion top at height `y`, a hair proud of it."""
    for i, x in enumerate(xs):
        for k, z in enumerate(zs):
            F.sphere(f'{name}{i}_{k}', r, (x, y, z), material, scale=(1, 1, 0.4), segments=8, rings=5)


# ---------------------------------------------------------------- the double bed (`bed`)

def bed():
    top = 0.58  # BEDS['bed']
    _legs_round((-0.84, 0.84), (-0.82, 0.88), 0.12, M.legs_teak)
    # the upholstered base, piped round its top and bottom
    F.soft('base', 1.82, 0.28, 1.83, (0, 0.26, 0.025), M.frame_linen, round=0.04, piping=0.008)
    # headboard: a piped padded back with four channels, its back on the wall's face
    F.soft('headboard', 1.9, 1.03, 0.035, (0, 0.615, WALL + 0.0175), M.headboard_linen, round=0.015, piping=0.006)
    for i, x in enumerate((-0.66, -0.22, 0.22, 0.66)):
        _channel(f'channel{i}', x, 0.41, 0.35, 1.09, -0.905, 0.035, M.headboard_linen)
    _mattress('mattress', 1.72, 0.2, 1.78, top, 0.03)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.68, 0.15, 0.4, s * 0.42, top, -0.66)
    z_top, z_foot = -0.42, 0.92
    z_fold = _bedding(top, 0.86, z_top, z_foot, 0.17, M.bedding_cream)
    # the runner across the foot, over the duvet and down its sides
    _runner('runner', 0.909, top + DUVET + 0.015, 0.72, 0.38, 0.2, M.throw_sage)
    _finish('bed', (top, 0.86, z_top, z_fold, M.bedding_cream))


# ---------------------------------------------------------------- the single bed

def bedSingle():
    top = 0.5  # BEDS['bedSingle']
    head = WALL + 0.03  # the head end's posts and boards, their backs on the wall's face
    for s in (-1, 1):
        x = s * 0.45
        _post(f'post_head{s}', x, head, 0.95, 0.06, M.frame_oak)
        _post(f'post_foot{s}', x, 0.94, 0.62, 0.06, M.frame_oak)
        F.box(f'rail{s}', 0.04, 0.18, 1.82, (x, 0.25, 0), M.frame_oak, bevel=0.008)
    F.box('rail_head', 0.84, 0.18, 0.04, (0, 0.25, head), M.frame_oak, bevel=0.008)
    F.box('rail_foot', 0.84, 0.18, 0.04, (0, 0.25, 0.94), M.frame_oak, bevel=0.008)
    # headboard: an arched top rail over five slats rounded at their ends; a framed, panelled footboard
    _board('head_top', 0.86, 0.86, 0.94, head, 0.05, M.headboard_oak, arch=0.035, bevel=0.01)
    F.box('head_low', 0.84, 0.05, 0.035, (0, 0.47, head), M.headboard_oak, bevel=0.008)
    for i in range(5):
        x = -0.28 + i * 0.14
        y1 = 0.885 + 0.035 * math.cos(math.pi * x / 0.86)
        F.soft(f'slat{i}', 0.08, y1 - 0.48, 0.022, (x, (0.48 + y1) / 2, head), M.headboard_oak, round=0.008)
    frame = G.rounded_rect(0.84, 0.24, 0.01, corner=1)
    F.extrude('footboard', frame, 0.03, (0, 0.47, 0.94), M.headboard_oak,
              holes=[G.rounded_rect(0.74, 0.15, 0.012, corner=2)], bevel=0.005, bevel_segments=1)
    F.box('foot_panel', 0.738, 0.148, 0.024, (0, 0.47, 0.94), M.headboard_oak, bevel=0.008, bevel_segments=1)
    F.soft('foot_top', 0.9, 0.04, 0.05, (0, 0.6, 0.94), M.headboard_oak, round=0.012)
    _mattress('mattress', 0.84, 0.18, 1.8, top, 0)
    _pillow('pillow', 0.62, 0.13, 0.34, 0, top, -0.71)
    z_top, z_foot = -0.3, 0.885
    z_fold = _bedding(top, 0.42, z_top, z_foot, 0.14, M.bedding_sage, flat=3)
    _finish('bedSingle', (top, 0.42, z_top, z_fold, M.bedding_sage), tiles=1)


# ---------------------------------------------------------------- the bunk bed

LOWER, UPPER = 0.44, 1.38  # characters.ts BUNKS['bedBunk']
BUNK_ENDS = {-1: WALL + 0.03, 1: 0.93}  # the depths of the posts and end boards: the head end's backs on the wall's face


def _bunk(name, top, guard):
    """One bunk: its rails, mattress, pillow and end boards (arched; the upper bunk's with a hand hold); the upper
    one with a guard rail."""
    y = top - 0.16 - 0.06
    for s in (-1, 1):
        F.box(f'{name}_rail{s}', 0.04, 0.12, 1.8, (s * 0.45, y, 0), M.frame_oak, bevel=0.008, bevel_segments=1)
        F.box(f'{name}_end{s}', 0.86, 0.12, 0.04, (0, y, BUNK_ENDS[s]), M.frame_oak, bevel=0.008, bevel_segments=1)
        h = 0.36 if guard else 0.3
        y0 = top - 0.13
        holes = [(0, y0 + h - 0.075, 0.18, 0.05)] if guard else ()
        _board(f'{name}_board{s}', 0.84, y0, y0 + h, BUNK_ENDS[s], 0.025, M.headboard_oak, arch=0.03, holes=holes)
    _mattress(f'{name}_mattress', 0.84, 0.16, 1.8, top, 0, round=0.03, flat=2)
    _pillow(f'{name}_pillow', 0.56, 0.11, 0.3, 0, top, -0.71, seed=top)
    if guard:
        F.box('guard_back', 0.04, 0.05, 1.82, (-0.45, 1.6, 0), M.frame_oak, bevel=0.012)
        F.box('guard_front', 0.04, 0.05, 1.28, (0.45, 1.6, -0.26), M.frame_oak, bevel=0.012)
        F.box('guard_post', 0.035, 0.38, 0.035, (0.45, 1.41, 0.38), M.frame_oak, bevel=0.008)


def bedBunk():
    for sx in (-1, 1):
        for sz in (-1, 1):
            _post(f'post{sx}{sz}', sx * 0.45, BUNK_ENDS[sz], 1.9, 0.06, M.frame_oak, cap='flat')
    _bunk('lower', LOWER, False)
    _bunk('upper', UPPER, True)
    # the ladder on the right, at the foot end: two rails, rounded, hooked over the upper bunk's rail, and four
    # round rungs
    for z in (0.42, 0.86):
        F.box(f'ladder{z}', 0.03, 1.62, 0.045, (0.465, 0.81, z), M.frame_oak, bevel=0.01)
        F.sweep(f'ladder_hook{z}', [(0.465, 1.6, z), (0.465, 1.66, z), (0.45, 1.69, z), (0.43, 1.66, z)], None,
                M.frame_oak, profile=G.rounded_rect(0.02, 0.04, 0.008, corner=1), up=(0, 0, 1), resolution=2)
    for i, y in enumerate((0.32, 0.62, 0.92, 1.22)):
        G.rod(f'rung{i}', F.at(0.465, y, 0.42), F.at(0.465, y, 0.86), 0.016, M.frame_oak, verts=10)
    z_top, z_foot = -0.28, 0.875
    # Each bunk's duvet turns down for the one who sleeps in it (`slot`: the lower bunk is 0, the upper 1).
    z_fold_upper = _bedding(UPPER, 0.42, z_top, z_foot, 0.12, M.bedding_teal, seed=1, flat=3)
    z_fold = _bedding(LOWER, 0.42, z_top, z_foot, 0.12, M.bedding_teal, seed=2, flat=3)
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
FUTON_TUFTS = (-0.66, -0.22, 0.22, 0.66)


def bedFuton():
    top = FUTON_TOP
    # a low ash platform: a frame round slats, on a recessed plinth
    F.box('plinth', 1.8, 0.04, 1.8, (0, 0.02, 0), M.frame_ash)
    # the frame's head end stands on the wall's face; rounded solid ash, the ends' tenons showing through the sides
    for s in (-1, 1):
        F.box(f'side{s}', 0.06, 0.16, 0.98 - WALL, (s * 0.95, 0.12, (WALL + 0.98) / 2), M.frame_ash, bevel=0.014,
              bevel_segments=2)
        F.box(f'end{s}', 1.84, 0.16, 0.06, (0, 0.12, 0.95 if s > 0 else WALL + 0.03), M.frame_ash, bevel=0.014,
              bevel_segments=2)
        for z in (0.95, WALL + 0.03):
            F.box(f'tenon{s}{z}', 0.012, 0.09, 0.035, (s * 0.983, 0.12, z), M.frame_ash, bevel=0.003)
    for i in range(14):
        F.box(f'slat{i}', 1.84, 0.025, 0.08, (0, 0.1875, -0.86 + i * 0.132), M.frame_ash, bevel=0.006)
    # a low headboard with four slots through it, under a rounded cap rail
    holes = [(x, 0.43, 0.06, 0.26) for x in (-0.6, -0.2, 0.2, 0.6)]
    F.extrude('headboard', G.rounded_rect(1.96, 0.46, 0.02, corner=2), 0.04, (0, 0.43, WALL + 0.02), M.headboard_ash,
              holes=[[(hx + u, hy - 0.43 + v) for u, v in G.rounded_rect(hw, hh, 0.03, corner=3)] for hx, hy, hw, hh in holes],
              bevel=0.006, bevel_segments=1)
    F.soft('headboard_cap', 1.96, 0.035, 0.05, (0, 0.665, WALL + 0.025), M.headboard_ash, round=0.014)
    # the seat: the front of the futon, a tufted mattress
    _mattress('seat', 1.86, top - 0.2, 0.93 - FUTON_HINGE, top, (FUTON_HINGE + 0.93) / 2, material=M.bedding_futon,
              puff=0.008, piping=0.008, round=0.06)
    _tufts('tuft', FUTON_TUFTS, (0.05, 0.42, 0.79), top - 0.004, M.bedding_futon)
    q = F.soft('quilt', 1.5, 0.05, 0.32, (0, top + 0.02, 0.7), M.quilt_oat, round=0.022, puff=0.008, flat=3)
    G.jitter(q, 0.004, size=0.12, seed=3, up=0.3)
    G.join(G.meshes(), 'futon', origin=(0, 0, 0))
    # the back, built flat (the bed's head end) with the pillows on it, hinged on the seat's back edge
    with G.new_parts() as parts:
        z0 = WALL + 0.04  # in front of the headboard
        _mattress('back', 1.86, top - 0.2, FUTON_HINGE - z0 - 0.01, top, (z0 + FUTON_HINGE - 0.01) / 2,
                  material=M.bedding_futon, puff=0.008, piping=0.008, round=0.06)
        _tufts('back_tuft', FUTON_TUFTS, (-0.69, -0.42), top - 0.004, M.bedding_futon)
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
    F.soft('base', 1.96, 0.3, 1.82, (0, 0.25, 0.06), M.frame_linen, round=0.05, piping=0.008)
    # a tall padded headboard, piped, with five channels
    F.soft('headboard', 1.98, 1.36, 0.04, (0, 0.72, WALL + 0.02), M.headboard_linen, round=0.018, piping=0.006)
    for i in range(5):
        _channel(f'channel{i}', -0.74 + i * 0.37, 0.36, 0.45, 1.33, -0.91, 0.04, M.headboard_linen)
    # wings each side, turned in a little round the pillows, their backs on the wall's face: piped too
    for s in (-1, 1):
        F.soft(f'wing{s}', 0.08, 1.3, 0.2, (s * 0.935, 0.69, -0.823), M.headboard_linen, rot=(0, 0, s * 14),
               round=0.03, puff=0.008, flat=2, piping=0.006)
    _mattress('mattress', 1.86, 0.22, 1.8, top, 0.04, round=0.045)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.74, 0.16, 0.4, s * 0.45, top, -0.66)
    z_top, z_foot = -0.4, 0.945
    z_fold = _bedding(top, 0.93, z_top, z_foot, 0.18, M.bedding_cream)
    # (just round the duvet's sides: its ends stay under the edges of the quilts over sleepers)
    _runner('runner', 0.959, top + DUVET + 0.015, 0.75, 0.38, 0.21, M.throw_sage)
    _finish('bedKing', (top, 0.93, z_top, z_fold, M.bedding_cream))


# ---------------------------------------------------------------- the oak double

DOUBLE_TOP = 0.52  # the old part's mattress top (characters.ts BEDS['bedDouble'])


def bedDouble():
    top = DOUBLE_TOP
    _legs_square((-0.75, 0.75), (-0.84, 0.9), 0.1, M.frame_oak)
    F.box('frame', 1.62, 0.22, 1.84, (0, 0.21, 0.04), M.frame_oak, bevel=0.014, bevel_segments=2)
    # a bead along the frame's top edge
    F.box('frame_bead', 1.632, 0.016, 1.852, (0, 0.29, 0.04), M.frame_oak, bevel=0.006)
    # headboard: oak posts and an arched, overhanging top rail round teak slats, their backs on the wall's face
    head = WALL + 0.035
    for s in (-1, 1):
        _post(f'post{s}', s * 0.775, head, 0.95, 0.07, M.frame_oak, cap='pyramid')
    _board('head_top', 1.5, 0.87, 0.95, head, 0.07, M.frame_oak, arch=0.03, bevel=0.012)
    F.box('head_low', 1.48, 0.06, 0.05, (0, 0.35, head), M.frame_oak, bevel=0.01)
    for i in range(8):
        x = -0.69 + i * 0.197
        y1 = 0.89 + 0.03 * math.cos(math.pi * x / 1.5)
        F.soft(f'slat{i}', 0.1, y1 - 0.37, 0.025, (x, (0.37 + y1) / 2, head), M.headboard_teak, round=0.01)
    _mattress('mattress', 1.56, 0.2, 1.79, top, 0.025)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.69, 0.14, 0.34, s * 0.405, top, -0.68)
    z_top, z_foot = -0.36, 0.92
    z_fold = _bedding(top, 0.78, z_top, z_foot, 0.2, M.bedding_blue)
    _finish('bedDouble', (top, 0.78, z_top, z_fold, M.bedding_blue))


# ---------------------------------------------------------------- the upholstered queen

def bedQueenUpholstered():
    top = DOUBLE_TOP
    _legs_square((-0.8, 0.8), (-0.84, 0.9), 0.08, M.legs_teak, w=0.05)
    F.soft('base', 1.72, 0.24, 1.82, (0, 0.2, 0.05), M.frame_stone, round=0.03, piping=0.007)
    # a tall headboard of five domed channels, its top scalloped round them
    xs = [-0.64 + i * 0.32 for i in range(5)]
    top_edge = []
    for x in reversed(xs):
        top_edge += [(x + 0.16 * math.cos(math.radians(a)), 1.14 + 0.16 * math.sin(math.radians(a))) for a in range(0, 181, 20)]
    outline = [(-0.86, 0.02), (0.86, 0.02), (0.86, 1.14)] + top_edge + [(-0.86, 1.14)]
    F.extrude('headboard', [(u, v) for u, v in outline], 0.04, (0, 0, WALL + 0.02), M.headboard_stone, bevel=0.012,
              bevel_segments=2)
    for i, x in enumerate(xs):
        _channel(f'channel{i}', x, 0.3, 0.42, 1.28, -0.91, 0.04, M.headboard_stone, dome=True)
    _mattress('mattress', 1.66, 0.2, 1.78, top, 0.03)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.74, 0.15, 0.34, s * 0.43, top, -0.68)
    z_top, z_foot = -0.36, 0.935
    z_fold = _bedding(top, 0.855, z_top, z_foot, 0.2, M.bedding_oat)
    _finish('bedQueenUpholstered', (top, 0.855, z_top, z_fold, M.bedding_oat))


# ---------------------------------------------------------------- the four-poster

def _folds(k, n, depth, width, folds):
    """A ring of `n` points (Blender x, y about 0) round a gathered curtain's cross-section: `width` along the bed's
    side (Blender y), `depth` across it, in `folds` soft pleats."""
    pts = []
    for i in range(n):
        a = math.tau * i / n
        r = 1 + k * math.sin(folds * a)
        pts.append((depth / 2 * math.cos(a) * r, width / 2 * math.sin(a) * r))
    return pts


def _drape(name, x, z, inward):
    """A curtain gathered at a post, hanging from the top rail down the side: pleated, wide at the rail, drawn in by
    a tie-back a metre up and spreading again to the floor. `inward`: the depth direction along the side, away from
    the post."""
    bx, by = F.at(x, 0, z)[:2]
    shift = -inward * 0.05      # Blender y: its middle a little along the side from the post
    secs = []
    for y, width, depth in ((0.12, 0.16, 0.06), (0.4, 0.15, 0.055), (0.75, 0.11, 0.05), (1.0, 0.06, 0.045),
                            (1.25, 0.1, 0.045), (1.6, 0.14, 0.04), (1.96, 0.16, 0.03)):
        secs.append([(bx + u, by + shift + v, y) for u, v in _folds(0.1, 20, depth, width, 5)])
    G.loft(f'{name}_curtain', secs, M.canopy)
    tie = [(x + 0.03 * math.cos(math.tau * i / 10), 1.0, z + inward * 0.05 + 0.04 * math.sin(math.tau * i / 10))
           for i in range(10)]
    F.sweep(f'{name}_tie', tie, 0.009, M.canopy, verts=4, closed=True)


def _turned_post(name, x, z, h, r, material, segments=10):
    """A turned post `h` tall, about `r` thick: a square-ish foot block below, then a vase, rings and a long shaft
    up to a finial."""
    prof = [(0, 0.12), (r * 1.15, 0.12), (r * 0.8, 0.16), (r * 1.15, 0.25), (r * 1.25, 0.33),
            (r * 0.85, 0.42), (r * 0.65, 0.46), (r * 1.0, 0.49), (r * 1.0, 0.52), (r * 0.62, 0.55),
            (r * 0.7, 1.05), (r * 0.55, h - 0.18), (r * 0.9, h - 0.155), (r * 0.9, h - 0.135),
            (r * 0.5, h - 0.115), (r * 0.5, h - 0.075), (r * 0.85, h - 0.045), (r * 0.6, h - 0.012), (0, h)]
    G.revolve(name, prof, at=F.at(x, 0, z), material=material, segments=segments)
    F.box(f'{name}_foot', 2 * r * 1.28, 0.12, 2 * r * 1.28, (x, 0.06, z), material, bevel=0.012, bevel_segments=1)


def _scallops(w, h, n):
    """A valance's outline `w` across and `h` deep, hanging from its top (0) in `n` scallops."""
    bottom = []
    for i in range(n * 6 + 1):
        u = w / 2 - w * i / (n * 6)
        bottom.append((u, -h + 0.04 * abs(math.sin(math.pi * i / 6))))
    return [(w / 2, 0), (-w / 2, 0)] + bottom[::-1]


def bedCanopy():
    top = DOUBLE_TOP
    # the posts' depths: the head end's feet, crest and valance stand on the wall's face
    posts = {-1: WALL + 0.045, 1: 0.92}
    for sx in (-1, 1):
        for sz in (-1, 1):
            _turned_post(f'post{sx}{sz}', sx * 0.82, posts[sz], 2.135, 0.035, M.frame_walnut)
        F.box(f'rail_top{sx}', 0.06, 0.06, 1.77, (sx * 0.82, 2.06, 0), M.frame_walnut, bevel=0.012, bevel_segments=2)
        F.box(f'rail_end{sx}', 1.57, 0.06, 0.06, (0, 2.06, posts[sx]), M.frame_walnut, bevel=0.012, bevel_segments=2)
    F.box('frame', 1.6, 0.22, 1.78, (0, 0.21, 0), M.frame_walnut, bevel=0.016, bevel_segments=2)
    F.box('frame_bead', 1.612, 0.016, 1.792, (0, 0.29, 0), M.frame_walnut, bevel=0.006)
    # a headboard of three raised panels in a frame, under an arched crest rail
    holes = [[(x + u, v) for u, v in G.rounded_rect(0.42, 0.4, 0.02, corner=2)] for x in (-0.52, 0, 0.52)]
    F.extrude('headboard', G.rounded_rect(1.57, 0.6, 0.004, corner=1), 0.04, (0, 0.68, WALL + 0.02), M.headboard_walnut,
              holes=[[(u, v + 0.02) for u, v in h] for h in holes], bevel=0.006, bevel_segments=1)
    for i, x in enumerate((-0.52, 0, 0.52)):
        F.box(f'panel{i}', 0.418, 0.398, 0.024, (x, 0.7, WALL + 0.02), M.headboard_walnut, bevel=0.009, bevel_segments=1)
    _board('crest', 1.64, 0.965, 1.1, WALL + 0.035, 0.07, M.headboard_walnut, arch=0.065, bevel=0.012, n=20)
    # the canopy: a scalloped linen valance round the open top (sleepers stay in view from above), a curtain
    # gathered at each post
    for s in (-1, 1):
        F.extrude(f'valance_side{s}', _scallops(1.77, 0.2, 5), 0.012, (s * 0.85, 2.09, 0), M.canopy, plane='side')
        F.extrude(f'valance_end{s}', _scallops(1.57, 0.2, 4), 0.012, (0, 2.09, 0.95 if s > 0 else WALL + 0.006), M.canopy)
    for sx in (-1, 1):
        for sz in (-1, 1):
            _drape(f'drape{sx}{sz}', sx * 0.83, sz * 0.8, -sz)
    _mattress('mattress', 1.56, 0.2, 1.76, top, 0)
    for s in (-1, 1):
        _pillow(f'pillow{s}', 0.69, 0.14, 0.34, s * 0.405, top, -0.68)
    # (the duvet's foot 1.5 cm past the mattress's end, not in its plane)
    z_top, z_foot = -0.36, 0.87
    z_fold = _bedding(top, 0.775, z_top, z_foot, 0.2, M.bedding_rust)
    _finish('bedCanopy', (top, 0.775, z_top, z_fold, M.bedding_rust))


# ---------------------------------------------------------------- the daybed

DAYBED_TOP = 0.51  # the old part's mattress top (characters.ts BEDS['bedDaybed'])


def _spindle(name, x, y0, y1, z, material, r=0.014):
    """A turned spindle from y0 up to y1: a bead at each end, swelling in the middle."""
    h = y1 - y0
    prof = [(0, 0), (r * 1.1, 0), (r * 1.15, 0.03), (r * 0.7, 0.06), (r * 1.0, h * 0.5), (r * 0.7, h - 0.06),
            (r * 1.15, h - 0.03), (r * 1.1, h), (0, h)]
    return G.revolve(name, prof, at=F.at(x, y0, z), material=material, segments=6)


def bedDaybed():
    top = DAYBED_TOP
    # the ends' depths: the head end's posts and arm stand on the wall's face, the base and the mattress
    # between the arms
    ends = {-1: WALL + 0.03, 1: 0.95}
    z0, z1 = ends[-1] + 0.03, 0.92
    # four posts, the back's taller, each with a ball on top; a base between them
    for sx in (-1, 1):
        h = 0.84 if sx < 0 else 0.74
        for sz in (-1, 1):
            _post(f'post{sx}{sz}', sx * 0.45, ends[sz], h, 0.06, M.frame_white)
    F.box('base', 0.9, 0.23, z1 - z0, (0, 0.215, (z0 + z1) / 2), M.frame_white, bevel=0.014, bevel_segments=2)
    F.box('base_bead', 0.912, 0.016, z1 - z0 + 0.012, (0, 0.11, (z0 + z1) / 2), M.frame_white, bevel=0.006)
    # the back along the left side: a rounded top rail over turned spindles (the headboard)
    F.sweep('back_rail', [(-0.45, 0.79, ends[-1] + 0.03), (-0.45, 0.79, 0.92)], None, M.back_white,
            profile=G.rounded_rect(0.05, 0.06, 0.018, corner=2), up=(0, 0, 1))
    F.box('back_low', 0.05, 0.03, 0.93 - ends[-1] - 0.06, (-0.45, 0.345, (ends[-1] + 0.93) / 2), M.back_white, bevel=0.008)
    for i in range(15):
        _spindle(f'back_spindle{i}', -0.45, 0.33, 0.8, -0.84 + i * 0.12, M.back_white)
    # an arm at each end, the same
    for s in (-1, 1):
        F.sweep(f'arm_rail{s}', [(-0.42, 0.665, ends[s]), (0.42, 0.665, ends[s])], None, M.frame_white,
                profile=G.rounded_rect(0.05, 0.06, 0.018, corner=2), up=(0, 0, 1))
        F.box(f'arm_low{s}', 0.86, 0.06, 0.05, (0, 0.3, ends[s]), M.frame_white, bevel=0.01)
        for i in range(5):
            _spindle(f'arm_spindle{s}{i}', -0.3 + i * 0.15, 0.32, 0.645, ends[s], M.frame_white)
    # the mattress in a fitted cover (the bedding), two back cushions clear of a sleeper, a pillow
    _mattress('mattress', 0.86, top - 0.33, z1 - z0, top, (z0 + z1) / 2, x=0.02, material=M.bedding_sage, round=0.035)
    # (thin and close to the back, leaning on it, so a sleeper's shoulder and hip stay clear of them)
    for i, z in enumerate((-0.32, 0.3)):
        c = F.soft(f'cushion{i}', 0.08, 0.34, 0.56, (-0.385, top + 0.16, z), M.cushion_rose, rot=(0, -8, 0),
                   round=0.03, puff=0.016, flat=3, piping=0.006)
        G.jitter(c, 0.003, size=0.1, seed=i, up=0.3)
    _pillow('pillow', 0.56, 0.13, 0.3, 0.06, top, -0.72)
    G.join_all('bedDaybed')
    _settle()


PAINT = ('Frame', 'Headboard', 'Bedding')


def _paint(frame, headboard, bedding):
    return list(zip((frame, headboard, bedding), PAINT))


TURN_DOWN = {'uses': {'sleep': 'turn_down', 'nap': 'turn_down'}}
# The bunk turns down the duvet of the bunk slept in: the lower one's for slot 0, the upper one's for slot 1,
# once its sleeper has climbed the ladder and crawled in (characters' `bunkUp`, 7 s: in on hands and knees by
# 4.9 s, sat up by 6), so it folds back as they turn to sit, not while they're at the ladder's foot.
BUNK_CLIMB_IN = 5.0
BUNKS = [{'clip': 'turn_down', 'slot': 0}, {'clip': 'turn_down_upper', 'slot': 1, 'delay': BUNK_CLIMB_IN}]
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
    comment='Blender-built bedroom models (tools/art/blender/families/bedroom.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. The beds turn their duvet down (`turn_down`) while a resident sleeps or '
            'naps; the futon unfolds into a bed (`unfold`) and stays open.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
