# The hobbies: music (H1: `model.piano`, `model.ukulele`, `model.guitarElectric`, `model.keyboardDigital`,
# `model.drumKit`, `model.guitarAcoustic`), games (H2: `model.chessTable`, `model.poolTable`, `model.foosball`,
# `model.dartboard`, `model.arcadeCabinet`, `model.puzzleTable`) and crafts and science (H3: `model.easel`,
# `model.workbench`, `model.telescope`, `model.sewingTable`, `model.chemistrySet`, `model.potteryWheel`,
# `model.craftTable`). Notes: hobbies.md.
#
# Each is built where the game places it, in the game's axes (`F`), and stood on the floor with
# `G.settle(centre=False)`, so a seat stays where the game seats residents. The moving parts and their clips:
#   piano          the fallboard lifts off the keys and the music desk swings out (`fallboard`)
#   drumKit        the crash and ride cymbals wobble while drummed (`cymbals`, a loop)
#   chessTable     a black pawn slides one square (`move`)
#   poolTable      the triangle lifts off the racked balls (`rack_off`)
#   foosball       the two outer rods kick back and forth (`rods`, a loop)
#   dartboard      three darts grow in the board (`darts`)
#   arcadeCabinet  the joystick waggles (`joystick`, a loop)
#   puzzleTable    a patch of finished puzzle spreads (`pieces`)
#   easel          the painting spreads across the canvas (`painting`)
#   workbench      the vice's jaw slides shut on a plank (`vice`)
#   telescope      the tube tilts up 35° (`aim`)
#   sewingTable    the presser foot lowers (`presser`); the handwheel turns (`handwheel`, a loop)
#   chemistrySet   the liquid rises in the flask (`bubble`)
#   potteryWheel   the pot rises from a lump (`clay`); the wheel head turns (`wheel`, a loop)
#   craftTable     the scrapbook's cover swings open (`book_open`)
#   ukulele, guitarElectric, guitarAcoustic
#                  the instrument is hidden while played (`off_stand`): the game draws it in the player's hands
#                  (content `held`), from this part, upright as built (its lean on the stand is the node's turn)
# The digital piano is still.
#
# Run:  cargo art hobbies [--only piano] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
import bpy  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import gamekit as G  # noqa: E402

M = G.Palette(
    # the piano and the digital piano
    piano=('#6E4630', 'gloss'),        # the lacquered case; the main part the player colours
    piano_trim=('#9C6B44', 'gloss'),   # lid, key slip, back panels, music desk
    ivory=('#F6F3EA', 'satin'),        # the white keys
    key_gap=('#B9B2A2', 'satin'),      # the lines between them
    ebony=('#141414', 'gloss'),        # the black keys
    brass=('#B8925A', 'metal'),        # pedals, candle sconces
    candle=('#F3EAD8', 'satin'),
    sheet=('#F4EFE2', 'matte'),        # sheet music, paper
    ink=('#3A3A3A', 'matte'),          # the staves
    cushion=('#7A2E2E', 'satin'),      # the piano bench's top
    kb_body=('#2A2C30', 'satin'),      # the digital piano's case
    kb_trim=('#4A4D52', 'satin'),      # its control strip and speakers
    kb_seat=('#3A3D41', 'fabric'),     # its bench
    lcd=('#7FB8A8', 'gloss'),
    # the guitars and the ukulele
    stand=('#2B2C2E', 'metal'),        # the instrument stands
    rubber=('#1A1B1D', 'satin'),       # their pads and feet
    uke=('#C08A52', 'wood'),           # the ukulele's body and neck
    uke_top=('#D9A86C', 'wood'),       # its soundboard
    fretboard=('#3B2A20', 'satin'),
    hole=('#1E1712', 'matte'),         # sound holes
    fret=('#C9CBCC', 'metal'),         # frets, tuners
    string=('#D8D8D2', 'metal'),
    bone=('#EFE8D8', 'satin'),         # nuts and saddles
    electric=('#D9483B', 'gloss'),     # the electric guitar's body
    pickguard=('#F2F2F0', 'gloss'),
    maple=('#D9B98A', 'satin'),        # its neck
    chrome=('#D0D3D6', 'chrome'),      # hardware: bridges, drum stands, rods
    black=('#2A2C30', 'gloss'),        # headstock, pickups, knobs
    amp=('#2A2C30', 'matte'),          # the amp's cabinet
    grille=('#4A4D52', 'fabric'),      # its speaker cloth
    amp_trim=('#C9A27A', 'satin'),     # its control panel and piping
    acoustic=('#C08A52', 'gloss'),     # the acoustic guitar's back and sides
    spruce=('#E2C48E', 'gloss'),       # its top
    neck=('#3B2A20', 'wood'),          # its neck and head
    tortoise=('#5A3420', 'gloss'),     # its pickguard
    # the drums
    shell=('#B03A2E', 'gloss'),        # the shells and hoops
    head=('#F2F0EA', 'satin'),         # the drum heads
    cymbal=('#C9A64A', 'metal'),
    seat=('#26272A', 'satin'),         # the throne
    # the games
    table_wood=('#7A5234', 'wood'),    # the chess and jigsaw tables
    chair_wood=('#A8754A', 'wood'),    # their chair and stool
    seat_pad=('#B5653F', 'satin'),     # the chess chair's pad
    square_light=('#E9D7B4', 'satin'),
    square_dark=('#4A2E1C', 'satin'),
    piece_light=('#F2EDE2', 'gloss'),
    piece_dark=('#1E1C1B', 'gloss'),
    pool_frame=('#6B4630', 'gloss'),   # the pool table's lacquered frame
    cloth=('#2F7A57', 'fabric'),       # its baize
    pocket=('#141414', 'matte'),
    pearl=('#EDE6D6', 'gloss'),        # the rail sights
    rack=('#A8754A', 'gloss'),         # the triangle
    cue=('#D9B98A', 'satin'),
    cue_butt=('#3B2A20', 'satin'),
    ball_yellow=('#F2C14E', 'gloss'),
    ball_blue=('#3E8EDE', 'gloss'),
    ball_red=('#D9483B', 'gloss'),
    ball_purple=('#6E3FA8', 'gloss'),
    ball_orange=('#E9853B', 'gloss'),
    ball_green=('#2F8F4E', 'gloss'),
    ball_maroon=('#8E2A2A', 'gloss'),
    ball_black=('#141414', 'gloss'),
    ball_white=('#F4F2EC', 'gloss'),
    foos=('#CDB08A', 'satin'),         # the foosball cabinet
    foos_trim=('#8E6B45', 'satin'),    # its legs and corners
    field=('#2F8F4E', 'matte'),
    line=('#F2F2F0', 'matte'),
    grip=('#2A2C30', 'satin'),         # handles, goal mouths
    cabinet=('#6E4A30', 'satin'),      # the dartboard's cabinet
    board=('#1E1F22', 'matte'),        # the board's dark segments and number ring
    board_light=('#EADBB8', 'matte'),
    ring_red=('#C23B2E', 'matte'),
    ring_green=('#2F7A4A', 'matte'),
    steel=('#C9CBCC', 'metal'),        # dart barrels
    arcade=('#1E1F22', 'satin'),       # the arcade cabinet
    art=('#2D3E8C', 'gloss'),          # its side art
    art_stripe=('#F2C14E', 'gloss'),
    bezel=('#16181B', 'gloss'),
    screen=('#3E8EDE', 'gloss', {'glow': '#7FC4FF'}),
    marquee=('#F2C14E', 'gloss', {'glow': '#FFE08A'}),
    puzzle_board=('#5E7A9A', 'matte'),
    sky=('#9CC3DB', 'matte'),          # the jigsaw's and the painting's sky
    hill=('#7FA35B', 'matte'),
    sun=('#F2C14E', 'matte'),
    blush=('#E4B8AE', 'matte'),
    card=('#C9A979', 'matte'),         # the jigsaw's box
    # crafts and science
    easel_wood=('#B98B5E', 'wood'),
    easel_dark=('#9A6E47', 'wood'),
    easel_metal=('#8E9499', 'metal'),  # its clamp and wing nuts
    canvas=('#F3EFE6', 'matte'),
    tree=('#5E8445', 'matte'),
    tree_light=('#6E9450', 'matte'),
    river=('#5E8FB0', 'matte'),
    palette=('#D2B48A', 'satin'),
    jar=('#7E9AA8', 'satin'),
    bench_top=('#C9A579', 'wood'),     # the workbench's butcher-block top
    bench_steel=('#8A9298', 'metal'),  # its tool board and the vice's pads
    bench_frame=('#A87B50', 'wood'),   # its posts, rails and shelves
    pegboard=('#C9B08A', 'matte'),     # its tool board (hardboard)
    label=('#EFEDE6', 'matte'),
    foot=('#2F3236', 'satin'),         # feet, grips, dark plastic
    toolbox=('#4E7A5A', 'satin'),
    toolbox_lid=('#5E8C68', 'satin'),
    drill=('#F2B84B', 'satin'),
    plank=('#D9C7A7', 'wood'),
    can=('#B9BDC1', 'metal'),
    vice=('#3E6FB0', 'satin'),
    tube=('#2E4E7E', 'gloss'),         # the telescope's tube
    tripod=('#F2F2F0', 'gloss'),
    optic=('#1E2A3A', 'gloss'),        # lenses
    sew_table=('#CDB08A', 'satin'),
    sew_legs=('#B08E66', 'satin'),
    machine=('#F2F1EE', 'gloss'),      # the sewing machine
    machine_trim=('#9DC6CF', 'satin'),
    fabric_pink=('#E4B8AE', 'satin'),  # the cloth being sewn
    sew_pad=('#9DAF8C', 'satin'),      # the sewing chair's pad
    thread_red=('#B5653F', 'satin'),
    thread_teal=('#4E7A7F', 'satin'),
    basket=('#C9B79C', 'matte'),
    chem_bench=('#E4E3DF', 'satin'),   # the lab bench
    chem_frame=('#2B2C2E', 'metal'),
    chem_board=('#F2F1EE', 'satin'),
    glass=('#DDEFF2', 'glass'),        # flasks and test tubes (see-through: its own draw call)
    liquid_green=('#7FD1AE', 'gloss'),
    liquid_orange=('#E9853B', 'gloss'),
    liquid_blue=('#6E8BE6', 'gloss'),
    liquid_yellow=('#F2C14E', 'gloss'),
    liquid_red=('#D9483B', 'gloss'),
    scope=('#2A2C30', 'satin'),        # the microscope
    wheel_body=('#7F8C8D', 'satin'),   # the pottery wheel
    pan=('#9AA3A5', 'satin'),          # its splash pan and wheel head
    clay=('#B5653F', 'matte'),
    clay_slip=('#9E5434', 'satin'),    # wet clay on the wheel head
    stool=('#8A5A36', 'satin'),
    craft_top=('#E3CFA8', 'wood'),     # the craft table's top
    craft_metal=('#2B2C2E', 'metal'),  # its hairpin legs
    paper=('#E7D6B8', 'matte'),
    scrapbook=('#C0473A', 'satin'),    # the scrapbook's cover
    pages=('#F4EFE2', 'matte'),
    photo=('#9CC3DB', 'gloss'),
    photo_warm=('#E9B98A', 'gloss'),
    tape_blue=('#9DC6CF', 'matte'),
    tape_gold=('#D6A64A', 'matte'),
    cup=('#4E7A7F', 'gloss'),
)


# --- helpers ----------------------------------------------------------------------------------------------
def _axis(rx, ry, rz=0.0):
    """The game direction of an old part's long axis (its local y) under its `rotate`: Euler degrees
    (yaw y, pitch x, roll z), read as the game reads them (`quatFromEuler` in render/shared/math.ts)."""
    rx, ry, rz = (math.radians(a) for a in (rx, ry, rz))
    cy, sy, cp, sp, cr, sr = (math.cos(ry / 2), math.sin(ry / 2), math.cos(rx / 2), math.sin(rx / 2),
                              math.cos(rz / 2), math.sin(rz / 2))
    x = cy * sp * cr + sy * cp * sr
    y = sy * cp * cr - cy * sp * sr
    z = cy * cp * sr - sy * sp * cr
    w = cy * cp * cr + sy * sp * sr
    return (2 * (x * y - z * w), 1 - 2 * (z * z + x * x), 2 * (y * z + x * w))


def _ends(at, length, rotate):
    """The game end points of an old part's turned cylinder (centre `at`, `length` long, `rotate`)."""
    d = _axis(*rotate)
    return (tuple(at[i] - d[i] * length / 2 for i in range(3)), tuple(at[i] + d[i] * length / 2 for i in range(3)))


def _link(name, bm, materials):
    """A mesh object from a bmesh, with `materials` in its slots (the faces' material_index)."""
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in materials:
        mesh.materials.append(m)
    return obj


def _turned(F, name, x, y, z, profile, material, segments=12, scale=(1, 1, 1)):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments, scale=scale)


def _flat_quads(F, name, quads, material):
    """Flat quads lying level, one mesh of two triangles each (the lines between keys, a board's squares):
    `quads` [(x0, x1, z0, z1, height)], in game axes, facing up."""
    bm = bmesh.new()
    for x0, x1, z0, z1, y in quads:
        bm.faces.new([bm.verts.new(F.at(x, y, z)) for x, z in ((x0, z0), (x1, z0), (x1, z1), (x0, z1))])
    for f in bm.faces:
        f.normal_update()
        if f.normal.z < 0:
            f.normal_flip()
    return _link(name, bm, [material])


def _rr(w, h, r, corner=2):
    """A rounded rectangle outline (`G.rounded_rect`), for profiles and slabs."""
    return G.rounded_rect(w, h, r, corner=corner)


def _sheet_music(F, name, w, h, at, rot=(0, 0, 0), depth=0.002):
    """A sheet of music facing the front: paper and four staves."""
    x, y, z = at
    F.box(name, w, h, depth, at, M.sheet, rot=rot)
    tilt = math.radians(rot[0])
    for k in range(4):
        v = h * (0.32 - 0.2 * k)
        F.box(f'{name}_staff{k}', w * 0.8, h * 0.06, 0.001,
              (x, y + v * math.cos(tilt), z - v * math.sin(tilt) + 0.0015 * math.cos(tilt)), M.ink, rot=rot)


def _chair(F, z, frame, pad, w=0.42, d=0.38):
    """A side chair behind a hobby table, facing the front: its seat at depth `z` with a cushion (its top at
    0.475 m, where the game seats residents), back posts rising in one piece from the floor, tapered front legs,
    stretchers, and a curved top rail and mid rail with three slats between."""
    back = z - d / 2 + 0.02
    post = _rr(0.032, 0.032, 0.008, 1)
    F.box('chair_seat', w, 0.035, d, (0, 0.4225, z), frame, bevel=0.006, bevel_segments=1)
    F.soft('chair_pad', w - 0.04, 0.03, d - 0.05, (0, 0.457, z + 0.01), pad, round=0.012, puff=0.005, flat=2,
           round_segments=1)
    for s in (-1, 1):
        x = s * (w / 2 - 0.03)
        F.sweep(f'chair_post{s}', [(x, 0.0, back + 0.012), (x, 0.42, back + 0.004), (x, 0.92, back - 0.004)], None,
                frame, profile=post, up=(1, 0, 0), scale=[0.85, 1.0, 0.9])
        F.sweep(f'chair_leg{s}', [(x, 0.405, z + d / 2 - 0.03), (x, 0.0, z + d / 2 - 0.03)], None, frame, profile=post,
                up=(1, 0, 0), scale=[1.0, 0.75])
        F.box(f'chair_stretcher{s}', 0.02, 0.02, d - 0.08, (x, 0.14, z), frame)
    F.box('chair_stretcher_front', w - 0.08, 0.02, 0.02, (0, 0.18, z + d / 2 - 0.03), frame)

    def bow(x):
        """How far back the rails curve at `x` across."""
        return back + 0.012 - 0.02 * (1 - (x / (w / 2)) ** 2)
    xs = [-(w / 2 - 0.045) + (w - 0.09) * i / 6 for i in range(7)]
    F.sweep('chair_rail_top', [(x, 0.85, bow(x)) for x in xs], None, frame, profile=_rr(0.09, 0.022, 0.01, 1))
    F.sweep('chair_rail_mid', [(x, 0.62, bow(x)) for x in xs], None, frame, profile=_rr(0.035, 0.02, 0.006, 1))
    for k in (-1, 0, 1):
        F.box(f'chair_slat{k}', 0.03, 0.2, 0.012, (k * 0.1, 0.72, bow(k * 0.1)), frame)


def _stool(F, z, top, r, legs, seat, x=0.0):
    """A round stool behind a hobby table: a seat of radius `r` with its top at `top`, its edge rounded, on four
    splayed legs joined by a foot ring."""
    _turned(F, 'stool_seat', x, top - 0.04, z, [(0, 0), (r - 0.012, 0), (r, 0.01), (r, 0.028), (r - 0.01, 0.04), (0, 0.04)],
            seat, segments=20)
    for k in range(4):
        a = 45 + 90 * k
        G.kit.tube(F, f'stool_leg{k}', [(x + 0.1 * G.cosd(a), top - 0.04, z + 0.1 * G.sind(a)),
                                        (x + 0.15 * G.cosd(a), 0.0, z + 0.15 * G.sind(a))], 0.016, legs, verts=8, round=1,
                   scale=[1.0, 0.8])
    G.torus('stool_ring', 0.131, 0.007, at=F.at(x, 0.16, z), material=legs, major_segments=20, minor_segments=4)


def _turn_round(objs, forward=0.0):
    """Turns a model built in place half a turn about the footprint's middle (each part about its own origin,
    which moves round with it), then moves it `forward` m to the front. A part's clips then open the other way."""
    for o in objs:
        G.turn(o, 180, 'Z')
        o.location = (-o.location.x, -o.location.y - forward, o.location.z)


def _upright(parts, name):
    """Joins an instrument built upright (its foot at the origin) into one part, its origin at its foot."""
    return G.join(parts, name, origin=(0, 0, 0))


def _lean(F, obj, degrees, foot):
    """Leans an upright instrument back by `degrees` and stands its foot at game `foot`. The lean is the node's
    turn, not baked into its mesh: the game takes the part upright (its foot at its origin, its face to the front)
    to draw it in a resident's hands (content `held`)."""
    obj.rotation_euler[0] = math.radians(-degrees)
    obj.location = F.at(*foot)
    return obj


def _off_stand(obj):
    """The instrument hidden while it's played (`off_stand`, quickly: the game draws it in the hands then)."""
    G.squash(obj, 'off_stand', 0.001, seconds=0.2)


def _guitar_stand(F, base_z, cradle_y, top_y, top_z, reach=0.07, back=0.2, front=0.06):
    """A black A-frame stand: a tube leaning back from a hub at depth `base_z` up to a rubber-lined yoke at
    (`top_y`, `top_z`), a height collar on it, two legs curving back and out, a front foot, and a cradle bent in
    one piece across at `cradle_y`, its arms reaching `reach` forward in rubber sleeves; the legs reach `back`
    behind the hub, the front foot `front` ahead."""
    t = (cradle_y - 0.03) / (top_y - 0.03)
    cz = base_z + t * (top_z - base_z)
    G.kit.tube(F, 'post', [(0, 0.03, base_z), (0, top_y, top_z)], 0.011, M.stand, verts=10, round=1)
    # the height collar and its knob, two thirds up
    k = 0.6
    cy, cz2 = 0.03 + k * (top_y - 0.03), base_z + k * (top_z - base_z)
    F.cyl('collar', 0.016, 0.04, (0, cy, cz2), M.rubber, verts=10, bevel=0.004, bevel_segments=1)
    G.kit.tube(F, 'collar_knob', [(0, cy, cz2), (0.035, cy, cz2)], 0.004, M.stand, verts=6, round=1)
    F.cyl('collar_knob_head', 0.011, 0.012, (0.04, cy, cz2), M.rubber, rot=(0, 90, 0), verts=10)
    F.box('hub', 0.04, 0.035, 0.05, (0, 0.03, base_z), M.stand, bevel=0.008, bevel_segments=1)
    for s in (-1, 1):
        G.kit.tube(F, f'leg{s}', G.kit.fillet([(0, 0.03, base_z), (s * 0.06, 0.024, base_z - back * 0.35),
                                                (s * 0.16, 0.012, base_z - back)], 0.05), 0.009, M.stand, verts=8, round=1)
        F.cyl(f'foot{s}', 0.013, 0.02, (s * 0.16, 0.01, base_z - back), M.rubber, verts=10, bevel=0.004, bevel_segments=1)
    G.kit.tube(F, 'foot_front', [(0, 0.022, base_z), (0, 0.012, base_z + front)], 0.009, M.stand, verts=8, round=1)
    F.cyl('foot_front_pad', 0.013, 0.02, (0, 0.01, base_z + front), M.rubber, verts=10, bevel=0.004, bevel_segments=1)
    # the cradle: bent in one piece, its arms sleeved in rubber
    arm = [(-0.1, cradle_y + 0.03, cz + reach), (-0.1, cradle_y, cz), (0.1, cradle_y, cz), (0.1, cradle_y + 0.03, cz + reach)]
    G.kit.tube(F, 'cradle', G.kit.fillet(arm, 0.025), 0.008, M.stand, verts=8, round=1)
    for s in (-1, 1):
        G.kit.tube(F, f'arm{s}', [(s * 0.1, cradle_y + 0.009, cz + 0.03), (s * 0.1, cradle_y + 0.03, cz + reach + 0.004)],
                   0.012, M.rubber, verts=8, round=1)
    # the yoke: a rubber-lined fork the neck rests in
    yoke = [(-0.03, top_y + 0.045, top_z + 0.01), (-0.03, top_y, top_z), (0.03, top_y, top_z), (0.03, top_y + 0.045, top_z + 0.01)]
    G.kit.tube(F, 'yoke', G.kit.fillet(yoke, 0.014), 0.008, M.rubber, verts=8, round=1)


def _strings(F, n, x_bridge, x_nut, y0, y1, z0, z1):
    """`n` strings from the bridge (height `y0`, depth `z0`, spread `x_bridge`) to the nut (`y1`, `z1`)."""
    for k in range(n):
        f = k - (n - 1) / 2
        G.rod(f'string{k}', F.at(f * x_bridge, y0, z0), F.at(f * x_nut, y1, z1), 0.0007, material=M.string, verts=4)


def _frets(F, n, scale, nut, w, z):
    """`n` frets down the fretboard from the nut at height `nut` (scale length `scale`), at depth `z`."""
    for k in range(1, n + 1):
        F.box(f'fret{k}', w, 0.0015, 0.002, (0, nut - scale * (1 - 2 ** (-k / 12)), z), M.fret)


# --- H1. Music ---------------------------------------------------------------------------------------------
# The piano: an upright with its back to the wall and its bench in front, the pianist facing the back
# (content `seat.facing`). Built keys to the back, then turned round (`_turn_round`) and stood on the wall's
# face. Its fallboard lies over the keys and lifts back against the case; its music desk, hinged at the top,
# swings its foot out so the sheet music leans back for reading.
PIANO_FORWARD = 0.027   # how far forward of where it was built, turned round, it stands: its back on the wall
BLACK_KEYS = [-0.577 + 0.161 * o + d for o in range(8) for d in (0, 0.023, 0.069, 0.092, 0.115)]


def piano():
    F = G.GameFrame()
    # (built keys to the back: the player's side is -z here, the wall's +z; turned round at the end)
    with G.new_parts() as still:
        F.box('case', 1.5, 0.53, 0.15, (0, 0.995, 0.375), M.piano, bevel=0.01)
        # the lid, its front edge moulded to a rounded nosing over a cove, overhanging the case
        F.extrude('lid', [(0.455, 1.26), (0.455, 1.29), (0.296, 1.29), (0.286, 1.287), (0.28, 1.279), (0.28, 1.272),
                          (0.285, 1.266), (0.294, 1.264), (0.298, 1.26)], 1.54, (0, 0, 0), M.piano_trim, plane='side',
                  bevel=0.003, bevel_segments=1)
        F.sweep('lid_cove', [(-0.75, 1.256, 0.3), (0.75, 1.256, 0.3)], None, M.piano_trim,
                profile=_rr(0.012, 0.012, 0.005, 1))
        # the upper front: a raised panel either side of the music desk, a moulded rail under them
        for s in (-1, 1):
            G.kit.shaker(F, f'upper_panel{s}', 0.36, 0.3, (s * 0.52, 1.05, 0.296), 0.012, M.piano, rail=0.045,
                         panel=M.piano_trim, raised=True)
        F.sweep('upper_rail', [(-0.74, 0.86, 0.296), (0.74, 0.86, 0.296)], None, M.piano_trim,
                profile=_rr(0.026, 0.014, 0.005, 1))
        F.box('keybed', 1.46, 0.08, 0.32, (0, 0.69, 0.29), M.piano, bevel=0.01)
        F.sweep('key_slip', [(-0.68, 0.72, 0.124), (0.68, 0.72, 0.124)], None, M.piano_trim,
                profile=[(-0.02, -0.006), (0.02, -0.006), (0.02, 0.002), (0.016, 0.006), (-0.02, 0.006)])
        F.box('keys', 1.3, 0.025, 0.15, (0, 0.735, 0.205), M.ivory)
        _flat_quads(F, 'gaps', [(-0.65 + 0.023 * k - 0.00075, -0.65 + 0.023 * k + 0.00075, 0.131, 0.279, 0.7478)
                                for k in range(1, 57)], M.key_gap)
        for k, x in enumerate(x for x in BLACK_KEYS if x <= 0.56):
            F.box(f'black{k}', 0.012, 0.018, 0.09, (x, 0.756, 0.23), M.ebony)
        F.box('key_felt', 1.3, 0.012, 0.012, (0, 0.74, 0.288), M.cushion)
        for s in (-1, 1):
            F.box(f'side{s}', 0.06, 0.78, 0.18, (s * 0.74, 0.39, 0.36), M.piano, bevel=0.01)
            # the key cheeks: rounded at the front, a scroll under
            F.extrude(f'cheek{s}', [(0.32, 0.73), (0.32, 0.87), (0.17, 0.87), (0.135, 0.86), (0.12, 0.84), (0.12, 0.79),
                                    (0.13, 0.765), (0.15, 0.75), (0.16, 0.73)], 0.06, (s * 0.74, 0, 0), M.piano,
                      plane='side', bevel=0.008, bevel_segments=2)
            # a turned console leg under each cheek, on a scrolled toe block
            _turned(F, f'leg{s}', s * 0.7, 0.03, 0.16, [(0, 0), (0.024, 0), (0.024, 0.02), (0.018, 0.04), (0.017, 0.3),
                                                        (0.024, 0.38), (0.03, 0.46), (0.03, 0.5), (0.022, 0.55),
                                                        (0.028, 0.58), (0.03, 0.62), (0, 0.62)], M.piano, segments=12)
            F.extrude(f'toe{s}', [(0.26, 0.0), (0.26, 0.035), (0.17, 0.035), (0.145, 0.03), (0.13, 0.015), (0.13, 0.0)],
                      0.07, (s * 0.7, 0, 0), M.piano, plane='side', bevel=0.005, bevel_segments=1)
            # brass candlesticks on the lid
            _turned(F, f'sconce{s}', s * 0.6, 1.29, 0.38, [(0, 0), (0.034, 0), (0.035, 0.005), (0.024, 0.01),
                                                         (0.009, 0.022), (0.007, 0.045), (0.012, 0.05), (0.026, 0.06),
                                                         (0.026, 0.064), (0.019, 0.07), (0, 0.07)], M.brass, segments=10)
            F.cyl(f'candle{s}', 0.0175, 0.12, (s * 0.6, 1.42, 0.38), M.candle, verts=12)
            F.cyl(f'wick{s}', 0.002, 0.01, (s * 0.6, 1.475, 0.38), M.ink, verts=4)
            # the lid's hinges on the back edge
            F.box(f'hinge{s}', 0.06, 0.006, 0.02, (s * 0.45, 1.293, 0.445), M.brass)
        # the lower case: a framed kneeboard in two raised panels, the back, plinth, the lyre and three pedals
        for s in (-1, 1):
            G.kit.shaker(F, f'kneeboard{s}', 0.71, 0.6, (s * 0.355, 0.35, 0.36), 0.025, M.piano, rail=0.06,
                         panel=M.piano, raised=True)
        F.box('back_lower', 1.42, 0.65, 0.02, (0, 0.325, 0.44), M.piano)
        F.sweep('plinth', [(-0.71, 0.018, 0.345), (0.71, 0.018, 0.345)], None, M.piano,
                profile=[(-0.018, -0.03), (0.018, -0.03), (0.018, 0.035), (0.012, 0.043), (0.0, 0.045),
                         (-0.018, 0.045)])
        lyre = G.smooth_outline([(0.1, 0.0), (0.1, 0.03), (0.06, 0.06), (0.05, 0.11), (0.09, 0.15), (0.11, 0.17),
                                 (-0.11, 0.17), (-0.09, 0.15), (-0.05, 0.11), (-0.06, 0.06), (-0.1, 0.03), (-0.1, 0.0)],
                                passes=1)
        F.extrude('lyre', lyre, 0.02, (0, 0.03, 0.335), M.piano_trim, bevel=0.004, bevel_segments=1)
        for x in (-0.07, 0, 0.07):
            F.sweep(f'pedal{x}', [(x, 0.05, 0.345), (x, 0.049, 0.3), (x, 0.044, 0.262), (x, 0.04, 0.252)], None,
                    M.brass, profile=_rr(0.008, 0.024, 0.003, 1), resolution=2, scale=[0.8, 0.9, 1.15, 1.1])
        # the bench: a buttoned cushion on a moulded frame, turned legs and stretchers (its top at 0.49 m)
        F.box('bench', 0.86, 0.06, 0.34, (0, 0.42, -0.2), M.piano, bevel=0.012)
        F.soft('bench_top', 0.82, 0.042, 0.3, (0, 0.461, -0.2), M.cushion, round=0.016, puff=0.008, flat=2,
               round_segments=2, piping=0.004, piping_material=M.piano_trim)
        for sx in (-1, 1):
            for z in (-0.33, -0.07):
                _turned(F, f'bench_leg{sx}{z}', sx * 0.38, 0.0, z, [(0, 0), (0.016, 0), (0.017, 0.02), (0.013, 0.05),
                                                                    (0.016, 0.2), (0.022, 0.29), (0.024, 0.33),
                                                                    (0.024, 0.39), (0, 0.39)], M.piano, segments=10)
            F.box(f'bench_rail{sx}', 0.02, 0.025, 0.22, (sx * 0.38, 0.12, -0.2), M.piano, bevel=0.005, bevel_segments=1)
        F.box('bench_stretcher', 0.74, 0.025, 0.02, (0, 0.12, -0.2), M.piano, bevel=0.005, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as fb:
        # the fallboard: a board with a rounded lip over the key slip, its maker's name in gold
        F.extrude('fallboard', [(0.3, 0.785), (0.3, 0.805), (0.142, 0.805), (0.132, 0.8), (0.128, 0.79), (0.13, 0.781),
                                (0.142, 0.785)], 1.32, (0, 0, 0), M.piano, plane='side', bevel=0.003, bevel_segments=1)
        F.box('fallboard_name', 0.12, 0.001, 0.012, (0, 0.8055, 0.2), M.brass)
        for s in (-1, 1):
            F.box(f'fallboard_hinge{s}', 0.05, 0.003, 0.012, (s * 0.5, 0.8055, 0.293), M.brass)
    fallboard = G.hinged(G.join(fb, 'fallboard'), F.at(0, 0.805, 0.3))
    with G.new_parts() as dk:
        # the music desk: a board with an arched top, pierced with fretwork, and a ledge for the music
        top = [(0.31 - 0.62 * i / 12, 1.19 + 0.02 * math.sin(math.pi * i / 12)) for i in range(13)]
        holes = [[(x + 0.026 * math.cos(math.tau * i / 10), 1.165 + 0.012 * math.sin(math.tau * i / 10)) for i in range(10)]
                 for x in (-0.265, 0.265)]
        F.extrude('desk', [(0.31, 0.985)] + top + [(-0.31, 0.985)], 0.012, (0, 0, 0.292), M.piano_trim, holes=holes,
                  bevel=0.003, bevel_segments=1)
        F.sweep('desk_ledge', [(-0.31, 0.994, 0.282), (0.31, 0.994, 0.282)], None, M.piano_trim,
                profile=[(-0.012, -0.012), (0.012, -0.012), (0.012, 0.006), (0.006, 0.012), (-0.012, 0.012)])
        _sheet_music(F, 'music', 0.4, 0.19, (0, 1.1, 0.285))
    desk = G.hinged(G.join(dk, 'music_desk'), F.at(0, 1.205, 0.298))
    _turn_round(G.meshes(), PIANO_FORWARD)
    G.settle(centre=False)
    G.swing(fallboard, 'fallboard', -90, seconds=0.9, axis='X')
    G.swing(desk, 'fallboard', -18, seconds=0.9, axis='X')
    return G.meshes()


# The ukulele and the guitars lean on a black A-frame stand. Each instrument is built upright, its foot at
# the origin and its face to the front, then leant back onto the stand: a part of its own.
def _figure8(lower, upper, n=10, passes=1):
    """An instrument's body outline (game x, height), its lower and upper bouts the ellipses `lower` and `upper`
    (middle height, half-width, half-height) meeting at a rounded waist; from the foot round."""
    (c1, a1, b1), (c2, a2, b2) = lower, upper

    def half(c, a, b, y):
        t = (y - c) / b
        return a * math.sqrt(max(0.0, 1 - t * t))
    lo, hi = c1, c2
    for _ in range(40):          # the waist: where the bouts are as wide as each other
        m = (lo + hi) / 2
        lo, hi = (m, hi) if half(c1, a1, b1, m) > half(c2, a2, b2, m) else (lo, m)
    yw = (lo + hi) / 2
    t1 = math.degrees(math.asin(max(-1.0, min(1.0, (yw - c1) / b1))))
    t2 = math.degrees(math.asin(max(-1.0, min(1.0, (yw - c2) / b2))))
    right = [(a1 * G.cosd(t), c1 + b1 * G.sind(t)) for t in (-90 + (t1 + 90) * i / n for i in range(n + 1))]
    right += [(a2 * G.cosd(t), c2 + b2 * G.sind(t)) for t in (t2 + (90 - t2) * i / n for i in range(n + 1))]
    left = [(-x, y) for x, y in reversed(right[1:-1])]
    return G.smooth_outline(right + left, passes)


def _inset(outline, d):
    """A closed outline moved `d` in all round (out, for a negative `d`)."""
    n = len(outline)
    area = sum(outline[i][0] * outline[(i + 1) % n][1] - outline[(i + 1) % n][0] * outline[i][1] for i in range(n))
    turn = 1 if area > 0 else -1
    out = []
    for i in range(n):
        (x0, y0), (x1, y1) = outline[i - 1], outline[(i + 1) % n]
        dx, dy = x1 - x0, y1 - y0
        length = math.hypot(dx, dy) or 1.0
        out.append((outline[i][0] - dy / length * d * turn, outline[i][1] + dx / length * d * turn))
    return out


def _neck(F, name, y0, y1, z, w, d, material, taper=0.85, verts=8):
    """A guitar's neck from height `y0` up to `y1`: flat under the fretboard at depth `z`, rounded behind (a D),
    `w` across and `d` deep, narrowing by `taper` to the nut."""
    prof = [(w / 2 * math.cos(math.pi * i / verts), d * math.sin(math.pi * i / verts)) for i in range(verts + 1)]
    return F.sweep(name, [(0, y0, z), (0, y1, z)], None, material, profile=prof, up=(1, 0, 0), scale=[1.0, taper],
                   angle=70)


def _dots(F, name, frets, scale, nut, z, material, r=0.003):
    """Position dots on a fretboard between the frets `frets` (its face at depth `z`)."""
    for k in frets:
        y = nut - scale * (1 - 2 ** (-(k - 0.5) / 12))
        F.cyl(f'{name}{k}', r, 0.001, (0, y, z), material, rot=(90, 0, 0), verts=6, smooth=False)


def _ukulele(F):
    with G.new_parts() as p:
        body = _figure8((0.1, 0.115, 0.1), (0.25, 0.09, 0.08), n=6)
        F.extrude('body', body, 0.065, (0, 0, 0), M.uke, bevel=0.009, bevel_segments=2)
        F.extrude('top', _inset(body, 0.006), 0.004, (0, 0, 0.033), M.uke_top)
        F.cyl('hole', 0.028, 0.004, (0, 0.205, 0.0345), M.hole, rot=(90, 0, 0), verts=18, smooth=False)
        G.torus('rosette', 0.032, 0.0025, at=F.at(0, 0.205, 0.035), material=M.fretboard, rot=(90, 0, 0),
                major_segments=24, minor_segments=4)
        F.extrude('bridge', _rr(0.07, 0.012, 0.005, 2), 0.008, (0, 0.07, 0.039), M.fretboard, bevel=0.002,
                  bevel_segments=1)
        F.box('saddle', 0.04, 0.003, 0.004, (0, 0.072, 0.044), M.bone)
        _neck(F, 'neck', 0.315, 0.565, 0.015, 0.04, 0.02, M.uke, taper=0.9)
        F.box('fretboard', 0.038, 0.23, 0.005, (0, 0.445, 0.0175), M.fretboard)
        _frets(F, 12, 0.35, 0.56, 0.038, 0.0205)
        _dots(F, 'dot', (5, 7, 10), 0.35, 0.56, 0.0205, M.bone, r=0.0025)
        F.box('nut', 0.038, 0.004, 0.004, (0, 0.561, 0.021), M.bone)
        # the head: a crown at its top
        head = [(0.019, 0.558), (0.03, 0.575), (0.031, 0.632), (0.02, 0.65), (0.0, 0.642), (-0.02, 0.65),
                (-0.031, 0.632), (-0.03, 0.575), (-0.019, 0.558)]
        F.extrude('head', G.smooth_outline(head, 1), 0.016, (0, 0, 0.002), M.fretboard, bevel=0.003, bevel_segments=1)
        for s in (-1, 1):
            for y in (0.588, 0.622):
                F.cyl(f'peg{s}{y}', 0.004, 0.022, (s * 0.036, y, 0.0), M.fret, rot=(0, 90, 0), verts=8)
                F.box(f'peg_button{s}{y}', 0.004, 0.016, 0.01, (s * 0.049, y, 0.0), M.bone)
        _strings(F, 4, 0.0075, 0.0085, 0.072, 0.561, 0.0465, 0.0235)
    return _upright(p, 'ukulele')


def ukulele():
    F = G.GameFrame()
    with G.new_parts() as still:
        _guitar_stand(F, 0.12, 0.2, 0.62, -0.005, back=0.17)
    G.join(still, 'stand', origin=(0, 0, 0))
    uke = _lean(F, _ukulele(G.GameFrame()), 15, (0, 0.215, 0.135))
    G.settle(centre=False)
    _off_stand(uke)
    return G.meshes()


# The electric guitar's solid body: a double cutaway, the treble horn short (+x), the bass horn long (game x, height
# from its foot).
ELECTRIC = [(0.0, 0.0), (0.09, 0.01), (0.155, 0.05), (0.18, 0.12), (0.17, 0.19), (0.132, 0.26), (0.13, 0.3), (0.14, 0.345),
            (0.13, 0.39), (0.11, 0.418), (0.09, 0.425), (0.075, 0.41), (0.065, 0.37), (0.05, 0.33), (0.035, 0.315),
            (0.026, 0.318), (-0.026, 0.318), (-0.036, 0.322), (-0.05, 0.35), (-0.062, 0.41), (-0.078, 0.455),
            (-0.095, 0.468), (-0.115, 0.455), (-0.128, 0.41), (-0.135, 0.34), (-0.14, 0.28), (-0.165, 0.2), (-0.183, 0.12),
            (-0.165, 0.05), (-0.09, 0.01)]
GUARD = [(0.135, 0.17), (0.12, 0.25), (0.085, 0.31), (0.045, 0.34), (-0.035, 0.342), (-0.075, 0.3), (-0.07, 0.2),
         (-0.095, 0.13), (-0.06, 0.06), (0.05, 0.05), (0.12, 0.09)]


def _electric(F):
    with G.new_parts() as p:
        F.extrude('body', G.smooth_outline(ELECTRIC, 1), 0.045, (0, 0, 0), M.electric, bevel=0.009, bevel_segments=2)
        F.extrude('guard', G.smooth_outline(GUARD, 2), 0.003, (0, 0, 0.0235), M.pickguard, bevel=0.001,
                  bevel_segments=1)
        for k, y in enumerate((0.13, 0.2, 0.27)):
            F.box(f'pickup{k}', 0.07, 0.018, 0.006, (0, y, 0.027), M.black, bevel=0.003, bevel_segments=2)
        # the bridge: a plate and six saddles, the tremolo arm
        F.box('bridge', 0.075, 0.025, 0.004, (0, 0.085, 0.0245), M.chrome, bevel=0.0015, bevel_segments=1)
        for j in range(6):
            F.box(f'saddle{j}', 0.009, 0.012, 0.006, (-0.026 + 0.0105 * j, 0.087, 0.029), M.chrome)
        G.kit.tube(F, 'trem_arm', G.kit.fillet([(0.034, 0.07, 0.027), (0.034, 0.07, 0.045), (0.06, 0.02, 0.05)], 0.01),
                   0.0018, M.chrome, verts=6, round=1)
        for k, (x, y) in enumerate(((0.09, 0.12), (0.1, 0.075), (0.085, 0.035))):
            G.kit.knob(F, f'knob{k}', x, y, 0.0235, 0.009, M.black, segments=8)
        F.box('switch_slot', 0.006, 0.03, 0.001, (0.11, 0.19, 0.0252), M.black)
        G.kit.tube(F, 'switch', [(0.11, 0.19, 0.025), (0.11, 0.19, 0.036)], 0.0018, M.chrome, verts=6, round=1)
        F.cyl('switch_tip', 0.004, 0.008, (0.11, 0.19, 0.038), M.pickguard, rot=(90, 0, 0), verts=8)
        F.cyl('jack', 0.011, 0.004, (0.12, 0.05, 0.0225), M.chrome, rot=(90, 0, 0), verts=12)
        F.cyl('strap_button', 0.005, 0.01, (0, -0.004, 0), M.chrome, verts=8)
        # a 0.648 m scale from the saddle to the nut (review M-hobbies-03: the neck was half as long again)
        nut = 0.09 + 0.648
        _neck(F, 'neck', 0.3, nut, 0.0125, 0.05, 0.022, M.maple, taper=0.86)
        F.box('neck_plate', 0.05, 0.05, 0.002, (0, 0.355, -0.0235), M.chrome, bevel=0.001, bevel_segments=1)
        F.box('fretboard', 0.048, nut - 0.355, 0.005, (0, (nut + 0.355) / 2, 0.015), M.fretboard)
        _frets(F, 15, 0.648, nut, 0.048, 0.0185)
        _dots(F, 'dot', (3, 5, 7, 9, 12), 0.648, nut, 0.0176, M.bone, r=0.0035)
        F.box('nut', 0.048, 0.004, 0.004, (0, nut + 0.001, 0.0185), M.bone)
        head = [(0.022, nut - 0.002), (0.028, nut + 0.04), (0.032, nut + 0.12), (0.046, nut + 0.148), (0.034, nut + 0.166),
                (-0.008, nut + 0.162), (-0.04, nut + 0.142), (-0.042, nut + 0.02), (-0.03, nut + 0.006), (-0.022, nut - 0.002)]
        F.extrude('head', G.smooth_outline(head, 1), 0.016, (0, 0, -0.002), M.black, bevel=0.003, bevel_segments=1)
        for k in range(6):
            y = nut + 0.023 + 0.022 * k
            F.cyl(f'tuner{k}', 0.006, 0.012, (-0.026, y, 0.011), M.chrome, verts=8, rot=(90, 0, 0))
            F.box(f'tuner_key{k}', 0.016, 0.008, 0.004, (-0.05, y, -0.006), M.chrome)
        F.box('string_tree', 0.012, 0.004, 0.004, (-0.01, nut + 0.05, 0.008), M.chrome)
        _strings(F, 6, 0.009, 0.0075, 0.087, nut + 0.001, 0.033, 0.0215)
    return _upright(p, 'guitar')


def guitarElectric():
    F = G.GameFrame()
    with G.new_parts() as still:
        _guitar_stand(F, 0.0, 0.11, 0.71, 0.005, reach=0.1, back=0.12, front=0.15)
        # the practice amp behind: a tolexed cabinet with piped edges, a cloth grille, a control panel, a strap handle
        F.box('amp', 0.46, 0.4, 0.28, (0, 0.2, -0.28), M.amp, bevel=0.02, bevel_segments=2)
        F.box('amp_baffle', 0.42, 0.29, 0.004, (0, 0.165, -0.139), M.black, bevel=0.002, bevel_segments=1)
        F.soft('amp_grille', 0.4, 0.27, 0.006, (0, 0.165, -0.137), M.grille, round=0.006, round_segments=1)
        G.kit.moulding(F, 'amp_piping', [(x, y + 0.165) for x, y in G.rounded_rect(0.4, 0.27, 0.01, 1)],
                       [(0, 0), (0, 0.004), (0.005, 0.004), (0.005, 0)],
                       M.amp_trim, at=-0.14)
        F.box('amp_panel', 0.42, 0.06, 0.004, (0, 0.355, -0.139), M.amp_trim, bevel=0.002, bevel_segments=1)
        for k in range(5):
            F.cyl(f'amp_knob{k}', 0.011, 0.014, (-0.14 + 0.06 * k, 0.355, -0.132), M.black, rot=(90, 0, 0), verts=10,
                  bevel=0.003, bevel_segments=1)
        F.cyl('amp_jack', 0.008, 0.01, (0.18, 0.355, -0.134), M.chrome, rot=(90, 0, 0), verts=8)
        F.box('amp_light', 0.01, 0.01, 0.004, (0.155, 0.355, -0.136), M.ball_red, bevel=0.002, bevel_segments=1)
        G.kit.tube(F, 'amp_handle', G.kit.fillet([(-0.08, 0.4, -0.28), (-0.075, 0.418, -0.28), (0.075, 0.418, -0.28),
                                                  (0.08, 0.4, -0.28)], 0.008), 0.008, M.black, verts=8, round=1)
        for x in (-0.085, 0.085):
            F.box(f'amp_handle_cap{x}', 0.03, 0.008, 0.03, (x, 0.402, -0.28), M.chrome, bevel=0.003, bevel_segments=1)
        F.box('amp_logo', 0.08, 0.018, 0.004, (0.12, 0.27, -0.134), M.amp_trim, bevel=0.002, bevel_segments=1)
        for sx in (-1, 1):
            for y in (0.01, 0.39):
                F.box(f'amp_corner{sx}{y}', 0.03, 0.03, 0.03, (sx * 0.215, y + 0.005, -0.145), M.chrome, bevel=0.006,
                      bevel_segments=1)
    G.join(still, 'stand', origin=(0, 0, 0))
    guitar = _lean(F, _electric(G.GameFrame()), 12, (0, 0.12, 0.106))
    G.settle(centre=False)
    _off_stand(guitar)
    return G.meshes()


def _acoustic(F):
    with G.new_parts() as p:
        # a dreadnought: a 0.51 m body, 0.40 m across the lower bout, the neck meeting it at the 14th fret, a
        # 0.645 m scale from the saddle to the nut, 1.05 m long (review M-hobbies-03: it was 1.29 m, the scale 0.97)
        saddle, scale = 0.222, 0.645
        nut = saddle + scale
        body = _figure8((0.17, 0.2, 0.17), (0.38, 0.145, 0.13), n=8)
        F.extrude('body', body, 0.1, (0, 0, 0), M.acoustic, bevel=0.01, bevel_segments=2)
        # the top inside a cream binding
        F.extrude('binding', _inset(body, 0.004), 0.003, (0, 0, 0.0495), M.bone)
        F.extrude('top', _inset(body, 0.009), 0.003, (0, 0, 0.0505), M.spruce)
        F.cyl('hole', 0.05, 0.004, (0, 0.4, 0.0515), M.hole, rot=(90, 0, 0), verts=24, smooth=False)
        G.torus('rosette', 0.057, 0.003, at=F.at(0, 0.4, 0.052), material=M.tortoise, rot=(90, 0, 0),
                major_segments=28, minor_segments=4)
        G.torus('rosette_ring', 0.064, 0.0015, at=F.at(0, 0.4, 0.052), material=M.bone, rot=(90, 0, 0),
                major_segments=28, minor_segments=3)
        guard = [(0.045, 0.36), (0.1, 0.335), (0.118, 0.29), (0.1, 0.255), (0.068, 0.268), (0.048, 0.31)]
        F.extrude('pickguard', G.smooth_outline(guard, 2), 0.002, (0, 0, 0.0525), M.tortoise)
        # the bridge: a shaped block, its saddle and six pins
        bridge = [(0.075, -0.012), (0.075, 0.006), (0.04, 0.014), (-0.04, 0.014), (-0.075, 0.006), (-0.075, -0.012),
                  (-0.04, -0.016), (0.04, -0.016)]
        F.extrude('bridge', G.smooth_outline(bridge, 1), 0.008, (0, saddle - 0.004, 0.055), M.fretboard, bevel=0.002,
                  bevel_segments=1)
        F.box('saddle', 0.075, 0.003, 0.004, (0, saddle, 0.06), M.bone)
        for k in range(6):
            F.cyl(f'pin{k}', 0.0025, 0.004, (-0.03 + 0.012 * k, saddle - 0.011, 0.06), M.bone, rot=(90, 0, 0), verts=6)
        _neck(F, 'neck', 0.48, nut, 0.0125, 0.05, 0.024, M.neck, taper=0.88)
        F.box('heel', 0.05, 0.05, 0.05, (0, 0.525, -0.01), M.neck, bevel=0.01)
        F.box('fretboard', 0.048, nut - 0.455, 0.005, (0, (nut + 0.455) / 2, 0.015), M.fretboard)
        _frets(F, 17, scale, nut, 0.048, 0.0185)
        _dots(F, 'dot', (3, 5, 7, 9, 12, 15), scale, nut, 0.0176, M.bone, r=0.0035)
        F.box('nut', 0.048, 0.004, 0.004, (0, nut + 0.001, 0.0185), M.bone)
        head = [(0.024, nut - 0.003), (0.038, nut + 0.165), (0.03, nut + 0.185), (0.0, nut + 0.18), (-0.03, nut + 0.185),
                (-0.038, nut + 0.165), (-0.024, nut - 0.003)]
        F.extrude('head', G.smooth_outline(head, 1), 0.016, (0, 0, -0.003), M.neck, bevel=0.003, bevel_segments=1)
        F.box('head_plate', 0.05, 0.03, 0.002, (0, nut + 0.15, 0.0055), M.hole, bevel=0.001, bevel_segments=1)
        for s in (-1, 1):
            for k in range(3):
                y = nut + 0.045 + 0.045 * k
                F.cyl(f'peg{s}{k}', 0.004, 0.024, (s * 0.05, y, -0.003), M.fret, rot=(0, 90, 0), verts=8)
                F.box(f'peg_button{s}{k}', 0.005, 0.018, 0.012, (s * 0.064, y, -0.003), M.fret)
                F.cyl(f'post{s}{k}', 0.004, 0.012, (s * 0.018, y, 0.008), M.fret, rot=(90, 0, 0), verts=6)
        _strings(F, 6, 0.0095, 0.0075, saddle + 0.007, nut + 0.001, 0.0625, 0.0215)
    return _upright(p, 'guitar')


def guitarAcoustic():
    F = G.GameFrame(shift=0.07)   # a little forward, where the game places it
    with G.new_parts() as still:
        _guitar_stand(F, -0.01, 0.105, 0.79, -0.057, reach=0.1, back=0.27)
    G.join(still, 'stand', origin=(0, 0, 0))
    guitar = _lean(F, _acoustic(G.GameFrame()), 8, (0, 0.12, 0.06))
    G.settle(centre=False)
    _off_stand(guitar)
    return G.meshes()


# The digital piano: a home console of 88 full-size keys (52 white at 2.35 cm) on two side panels with a
# pedal board of three pedals, a control strip and a music rest behind the keys, and a padded X-legged bench
# behind it, built facing the front and turned round to stand against the wall like the upright, the player
# facing it (content `seat.facing`). The keys' tops are at 0.745 m, as the upright's (0.72–0.76; review M-hobbies-04: they were at
# 0.845 on a three-octave keyboard on an X-stand). Still.
KB_KEY = 0.0235          # a white key's width
KB_TOP = 0.745           # the white keys' top
KB_KEYS_Z = 0.17         # the keys' middle (depth), as built (before it is turned round)
KB_FORWARD = -0.02       # turned round, how far forward it stands: its back on the wall's face
KB_W = 52 * KB_KEY       # the keyboard's width, 1.222 m


def _kb_black_keys():
    """Game x of each black key of an 88-key keyboard (A0 to C8): after A0, then C, D, F, G and A of each
    octave from C1."""
    x0 = -KB_W / 2
    edges = [1] + [2 + 7 * o + n for o in range(7) for n in (1, 2, 4, 5, 6)]
    return [x0 + KB_KEY * e for e in edges]   # A0 at the player's left: game -x as built, facing the front (+z)


def keyboardDigital():
    F = G.GameFrame()
    half = KB_W / 2 + 0.06          # the side panels' middles
    # (built facing the front, the player's side -z here; turned round at the end)
    with G.new_parts() as parts:
        # the keys, the key slip in front and the keybed under them
        F.box('keys', KB_W, 0.022, 0.15, (0, KB_TOP - 0.011, KB_KEYS_Z), M.ivory)
        _flat_quads(F, 'gaps', [(-KB_W / 2 + KB_KEY * k - 0.00075, -KB_W / 2 + KB_KEY * k + 0.00075, KB_KEYS_Z - 0.074,
                                 KB_KEYS_Z + 0.074, KB_TOP + 0.0003) for k in range(1, 52)], M.key_gap)
        for k, x in enumerate(_kb_black_keys()):
            F.box(f'black{k}', 0.012, 0.012, 0.09, (x, KB_TOP + 0.006, KB_KEYS_Z + 0.03), M.ebony)
        F.box('keybed', KB_W + 0.02, 0.05, 0.31, (0, KB_TOP - 0.047, 0.245), M.kb_body, bevel=0.006)
        F.sweep('key_slip', [(-KB_W / 2 - 0.01, KB_TOP - 0.02, KB_KEYS_Z - 0.081), (KB_W / 2 + 0.01, KB_TOP - 0.02, KB_KEYS_Z - 0.081)],
                None, M.kb_body, profile=[(-0.015, -0.006), (0.015, -0.006), (0.015, 0.002), (0.011, 0.006), (-0.015, 0.006)])
        F.box('key_felt', KB_W, 0.008, 0.01, (0, KB_TOP + 0.002, KB_KEYS_Z + 0.08), M.ball_red)
        # the case behind the keys: a rounded top with its control strip, buttons, a screen, two speaker grilles
        F.sweep('case_top', [(-KB_W / 2 - 0.01, KB_TOP + 0.03, 0.335), (KB_W / 2 + 0.01, KB_TOP + 0.03, 0.335)], None,
                M.kb_body, profile=[(-0.03, -0.065), (-0.03, 0.065), (0.02, 0.065), (0.03, 0.05), (0.03, -0.035),
                                    (0.022, -0.06), (0.008, -0.068)])
        F.box('strip', KB_W - 0.1, 0.004, 0.05, (0, KB_TOP + 0.062, 0.3), M.kb_trim, bevel=0.0015, bevel_segments=1)
        F.box('lcd', 0.12, 0.004, 0.03, (0.2, KB_TOP + 0.065, 0.3), M.lcd, bevel=0.0015, bevel_segments=1)
        for k in range(8):
            F.box(f'button{k}', 0.018, 0.006, 0.012, (-0.45 + 0.03 * k, KB_TOP + 0.066, 0.3), M.kb_body)
        F.box('led', 0.006, 0.003, 0.006, (0.29, KB_TOP + 0.065, 0.3), M.ball_red)
        for s in (-1, 1):
            F.cyl(f'knob{s}', 0.01, 0.012, (s * 0.52, KB_TOP + 0.068, 0.3), M.kb_trim, verts=12, bevel=0.003,
                  bevel_segments=1)
            F.soft(f'speaker{s}', 0.14, 0.004, 0.035, (s * 0.4, KB_TOP + 0.062, 0.345), M.kb_seat, round=0.002,
                   round_segments=1)
        # the music rest, leant back, with a ledge, and a sheet on it
        rest = [(0.35, -0.12), (0.35, 0.1), (0.33, 0.12), (-0.33, 0.12), (-0.35, 0.1), (-0.35, -0.12)]
        F.extrude('rest', rest, 0.012, (0, KB_TOP + 0.18, 0.37), M.kb_body, rot=(12, 0, 0), bevel=0.003,
                  bevel_segments=1)
        F.sweep('rest_ledge', [(-0.35, KB_TOP + 0.066, 0.355), (0.35, KB_TOP + 0.066, 0.355)], None, M.kb_body,
                profile=[(-0.006, -0.015), (0.006, -0.015), (0.008, -0.01), (0.008, 0.015), (-0.006, 0.015)])
        _sheet_music(F, 'music', 0.4, 0.2, (0, KB_TOP + 0.19, 0.362), rot=(12, 0, 0))
        # the side panels, cut back under the keys, with their key cheeks and feet; the back panel; the pedal board
        side = G.smooth_outline([(0.40, 0.0), (0.40, 0.805), (0.15, 0.805), (0.135, 0.79), (0.135, 0.7), (0.16, 0.66),
                                 (0.2, 0.55), (0.21, 0.3), (0.2, 0.1), (0.19, 0.03), (0.19, 0.0)], 1)
        cheek = [(0.37, 0.8), (0.37, 0.84), (0.13, 0.84), (0.105, 0.832), (0.095, 0.815), (0.1, 0.795), (0.12, 0.79),
                 (0.15, 0.8)]
        for s in (-1, 1):
            F.extrude(f'side{s}', side, 0.04, (s * half, 0, 0), M.kb_body, plane='side', bevel=0.006, bevel_segments=1)
            F.extrude(f'cheek{s}', G.smooth_outline(cheek, 1), 0.04, (s * half, 0, 0), M.kb_body, plane='side',
                      bevel=0.008, bevel_segments=2)
            F.extrude(f'toe{s}', [(0.41, 0.0), (0.41, 0.03), (0.06, 0.03), (0.02, 0.02), (0.01, 0.0)], 0.05,
                      (s * half, 0, 0), M.kb_trim, plane='side', bevel=0.005, bevel_segments=1)
        F.box('back', KB_W + 0.08, 0.5, 0.015, (0, 0.38, 0.39), M.kb_body)
        F.box('brace', KB_W + 0.05, 0.06, 0.02, (0, 0.6, 0.38), M.kb_trim, bevel=0.004, bevel_segments=1)
        F.box('pedal_board', 0.36, 0.09, 0.12, (0, 0.065, 0.33), M.kb_body, bevel=0.02, bevel_segments=2)
        G.kit.tube(F, 'pedal_rod', [(0, 0.11, 0.38), (0, KB_TOP - 0.07, 0.38)], 0.008, M.kb_trim, verts=8, round=1)
        for x in (-0.07, 0.0, 0.07):
            F.sweep(f'pedal{x}', [(x, 0.05, 0.3), (x, 0.049, 0.26), (x, 0.044, 0.222), (x, 0.04, 0.212)], None,
                    M.brass, profile=_rr(0.008, 0.024, 0.003, 1), resolution=2, scale=[0.8, 0.9, 1.15, 1.1])
        # headphones hung on a hook under the keybed, at the player's left
        G.kit.tube(F, 'hook', G.kit.fillet([(-0.5, KB_TOP - 0.072, 0.15), (-0.5, KB_TOP - 0.1, 0.15),
                                            (-0.5, KB_TOP - 0.1, 0.12)], 0.01), 0.003, M.kb_trim, verts=6, round=1)
        G.kit.tube(F, 'headband', [(-0.5 + 0.075 * math.cos(math.pi * i / 10), KB_TOP - 0.175 + 0.075 * math.sin(math.pi * i / 10), 0.12)
                                   for i in range(11)], 0.008, M.ebony, verts=6, round=1)
        for s in (-1, 1):
            F.cyl(f'earcup{s}', 0.035, 0.03, (-0.5 + s * 0.075, KB_TOP - 0.185, 0.12), M.ebony, rot=(0, 90, 0), verts=14,
                  bevel=0.008, bevel_segments=1)
        # the bench, its top at 0.48 m (content `seat`; a piano bench is 0.46–0.56): a padded seat on X legs
        F.soft('seat', 0.66, 0.05, 0.32, (0, 0.455, -0.2), M.kb_seat, round=0.02, puff=0.008, flat=2, round_segments=2,
               piping=0.004)
        F.box('seat_frame', 0.62, 0.02, 0.26, (0, 0.42, -0.2), M.stand, bevel=0.004, bevel_segments=1)
        for x in (-0.28, 0.28):
            for z0, z1 in ((-0.32, -0.08), (-0.08, -0.32)):
                G.kit.tube(F, f'bench_leg{x}{z0}', [(x, 0.016, z0), (x, 0.41, z1)], 0.013, M.stand, verts=8, round=1)
            F.cyl(f'bench_pivot{x}', 0.014, 0.035, (x, 0.213, -0.2), M.stand, rot=(0, 90, 0), verts=10)
            for z in (-0.32, -0.08):
                F.box(f'bench_cap{x}{z}', 0.035, 0.016, 0.05, (x, 0.008, z), M.rubber, bevel=0.005)
        G.kit.tube(F, 'bench_bar', [(-0.28, 0.213, -0.2), (0.28, 0.213, -0.2)], 0.009, M.stand, verts=8, round=1)
    G.join_all('keyboardDigital')
    _turn_round(G.meshes(), KB_FORWARD)   # built facing the front; it stands with its back on the wall's face
    G.settle(centre=False)
    return G.meshes()


# The drum kit's parts.
def _cymbal(F, name, r, at, rot, segments=24):
    """A cymbal: a domed bell rising out of a bow that thins to its edge."""
    profile = [(0, 0.021), (0.024, 0.018), (0.034, 0.009), (r * 0.6, 0.0035), (r, 0.0), (r, -0.0015), (r * 0.6, 0.002),
               (0.034, 0.006), (0.024, 0.015), (0, 0.018)]
    return G.revolve(name, profile, at=F.at(*at), material=M.cymbal, segments=segments, rot=rot)


def _drum(F, name, r, h, at, rot=(0, 0, 0), heads=(1,), lugs=6, rods=False):
    """A drum along its own axis: the shell, its heads (1 top, -1 bottom) in their hoops, and its lugs (`rods`: each
    with a tension rod into each hoop)."""
    tilt = Matrix.Rotation(math.radians(rot[0]), 4, 'X')
    centre = Vector(F.at(*at))

    def at_local(x, y, z):
        return tuple(centre + tilt @ Vector((x, y, z)))
    F.cyl(name, r, h, at, M.shell, rot=rot, verts=24)
    for s in heads:
        c = at_local(0, 0, s * (h / 2 + 0.002))
        G.cylinder(f'{name}_head{s}', r - 0.004, 0.004, at=c, material=M.head, rot=rot, verts=24, smooth=False)
        G.torus(f'{name}_hoop{s}', r + 0.002, 0.007, at=c, material=M.chrome, rot=rot, major_segments=20, minor_segments=3)
    for k in range(lugs):
        a = math.tau * (k + 0.5) / lugs
        turn = (tilt @ Matrix.Rotation(a, 4, 'Z')).to_euler('XYZ')
        rr = r + 0.006
        G.cube(f'{name}_lug{k}', (0.01, 0.014, h * 0.32), at=at_local(rr * math.cos(a), rr * math.sin(a), 0),
               material=M.chrome, rot=tuple(math.degrees(v) for v in turn))
        for s in (heads if rods else ()):
            G.cylinder(f'{name}_rod{k}{s}', 0.0025, 0.03, at=at_local((r + 0.009) * math.cos(a), (r + 0.009) * math.sin(a),
                                                                       s * (h / 2 - 0.01)),
                       material=M.chrome, rot=rot, verts=4)


def _drum_stand(F, name, x, z, top, spread=0.18):
    """A chrome stand: a tripod of legs braced to a collar, a lower tube and a thinner one sliding out of it at a
    clamp, rubber feet."""
    mid = 0.26 + (top - 0.26) * 0.55
    G.kit.tube(F, f'{name}_pole', [(x, 0.14, z), (x, mid, z)], 0.011, M.chrome, verts=8, round=1)
    G.kit.tube(F, f'{name}_upper', [(x, mid, z), (x, top, z)], 0.0075, M.chrome, verts=8, round=1)
    F.cyl(f'{name}_clamp', 0.015, 0.026, (x, mid, z), M.chrome, verts=8)
    F.cyl(f'{name}_hub', 0.017, 0.035, (x, 0.26, z), M.chrome, verts=8)
    F.cyl(f'{name}_collar', 0.015, 0.02, (x, 0.16, z), M.chrome, verts=6)
    for k in range(3):
        a = 90 + 120 * k
        c, s = G.cosd(a), G.sind(a)
        fx, fz = x + spread * c, z + spread * s
        G.kit.tube(F, f'{name}_leg{k}', [(x, 0.26, z), (fx, 0.01, fz)], 0.0065, M.chrome, verts=6, round=1)
        G.kit.tube(F, f'{name}_brace{k}', [(x, 0.16, z), (x + spread * 0.5 * c, 0.135, z + spread * 0.5 * s)], 0.004,
                   M.chrome, verts=4, round=1)
        F.cyl(f'{name}_foot{k}', 0.01, 0.014, (fx, 0.007, fz), M.seat, verts=6)


def _pedal(F, name, x, z, length=0.22):
    """A foot pedal along the depth, its heel at the back (game -z): a heel plate, a footboard tilting up to the
    front, and the frame's two posts, axle and cam over its toe."""
    z0, z1 = z - length / 2, z + length / 2
    F.box(f'{name}_heel', 0.075, 0.012, 0.06, (x, 0.006, z0 + 0.03), M.chrome, bevel=0.003, bevel_segments=1)
    F.sweep(f'{name}_board', [(x, 0.016, z0 + 0.05), (x, 0.05, z1 - 0.03)], None, M.chrome,
            profile=_rr(0.008, 0.065, 0.003, 1), scale=[0.85, 1.1])
    F.box(f'{name}_base', 0.11, 0.01, 0.06, (x, 0.005, z1), M.chrome, bevel=0.003, bevel_segments=1)
    for s in (-1, 1):
        G.kit.tube(F, f'{name}_post{s}', [(x + s * 0.05, 0.01, z1), (x + s * 0.05, 0.13, z1)], 0.006, M.chrome, verts=6,
                   round=1)
    G.kit.tube(F, f'{name}_axle', [(x - 0.05, 0.12, z1), (x + 0.05, 0.12, z1)], 0.006, M.chrome, verts=6, round=1)
    F.cyl(f'{name}_cam', 0.02, 0.02, (x, 0.12, z1), M.chrome, rot=(0, 90, 0), verts=10)
    G.kit.tube(F, f'{name}_chain', [(x, 0.05, z1 - 0.03), (x, 0.11, z1 - 0.012)], 0.004, M.seat, verts=4, round=1)


# The drum kit, spread over two tiles as a real five-piece is (review M-hobbies-05: it was packed onto one, the
# toms on the kick and the floor tom under the ride): a 22" kick to the front with two rack toms on a mount on it,
# a snare and a hi-hat to the drummer's left (game -x: they face the front, +z), a 16" floor tom to the right, the crash and ride cymbals on boom stands
# over them, and the throne behind (content `seat`: 0.52 m, `DRUM_THRONE` behind the middle). Built in place
# about the throne. The two cymbals are parts of their own, their origins on their stands' tops, and wobble
# while a resident drums (`cymbals`).
DRUM_THRONE = -0.4     # the throne's depth (content `seat.forward`)
CRASH = (-0.42, 1.18, DRUM_THRONE + 0.62)   # the cymbals' middles, on their booms
RIDE = (0.52, 1.1, DRUM_THRONE + 0.52)


def drumKit():
    F = G.GameFrame()
    T = DRUM_THRONE
    kz, ky, kr = T + 0.66, 0.29, 0.28      # the kick: its middle and radius
    with G.new_parts() as still:
        # the kick drum lies on its side, its front head to the front with a port and the maker's badge
        F.cyl('kick', kr, 0.4, (0, ky, kz), M.shell, rot=(90, 0, 0), verts=32)
        for k, z in enumerate((kz - 0.2, kz + 0.2)):
            F.cyl(f'kick_head{k}', kr - 0.01, 0.006, (0, ky, z), M.head, rot=(90, 0, 0), verts=28, smooth=False)
            G.torus(f'kick_hoop{k}', kr - 0.005, 0.013, at=F.at(0, ky, z), material=M.shell, rot=(90, 0, 0),
                    major_segments=24, minor_segments=3)
        F.cyl('kick_logo', 0.1, 0.002, (0, ky, kz + 0.2045), M.shell, rot=(90, 0, 0), verts=24, smooth=False)
        F.box('kick_badge', 0.1, 0.022, 0.002, (0, ky + 0.01, kz + 0.2058), M.head)
        F.cyl('kick_port', 0.05, 0.002, (0.13, ky - 0.1, kz + 0.2045), M.seat, rot=(90, 0, 0), verts=16, smooth=False)
        for k in range(8):
            a = 22.5 + 45 * k
            c, s = G.cosd(a), G.sind(a)
            F.box(f'kick_lug{k}', 0.016, 0.016, 0.1, ((kr + 0.003) * c, ky + (kr + 0.003) * s, kz), M.chrome,
                  rot=(0, -a, 0))
            for e in (-1, 1):
                G.kit.tube(F, f'kick_rod{k}{e}', [((kr + 0.008) * c, ky + (kr + 0.008) * s, kz + e * 0.06),
                                                  ((kr + 0.008) * c, ky + (kr + 0.008) * s, kz + e * 0.2)], 0.003,
                           M.chrome, verts=4, round=1)
        for s in (-1, 1):
            G.kit.tube(F, f'spur{s}', G.kit.fillet([(s * 0.25, 0.2, kz + 0.1), (s * 0.3, 0.12, kz + 0.13),
                                                    (s * 0.34, 0.012, kz + 0.16)], 0.04), 0.008, M.chrome, verts=6, round=1)
            F.cyl(f'spur_foot{s}', 0.011, 0.016, (s * 0.34, 0.008, kz + 0.16), M.seat, verts=8)
        # the pedal and its beater, on the batter head
        _pedal(F, 'pedal', 0, kz - 0.33)
        G.kit.tube(F, 'beater', [(0, 0.12, kz - 0.22), (0, 0.27, kz - 0.215)], 0.004, M.chrome, verts=6, round=1)
        F.cyl('beater_head', 0.024, 0.03, (0, 0.28, kz - 0.217), M.head, rot=(90, 0, 0), verts=12, bevel=0.008,
              bevel_segments=1)
        # two rack toms (10" and 12") on a mount on the kick, tilted to the drummer
        F.box('tom_clamp', 0.05, 0.03, 0.05, (0, ky + kr + 0.005, kz - 0.02), M.chrome, bevel=0.006, bevel_segments=1)
        G.kit.tube(F, 'tom_mount', [(0, ky + kr - 0.01, kz - 0.02), (0, 0.72, kz - 0.08)], 0.013, M.chrome, verts=8,
                   round=1)
        for k, (x, r, h) in enumerate(((-0.15, 0.13, 0.2), (0.15, 0.155, 0.22))):
            _drum(F, f'tom{k}', r, h, (x, 0.78, kz - 0.12), rot=(-20, 0, 0), heads=(1,))
            G.kit.tube(F, f'tom_arm{k}', G.kit.fillet([(0, 0.72, kz - 0.08), (x * 0.6, 0.72, kz - 0.12),
                                                       (x * 0.6, 0.74, kz - 0.12)], 0.02), 0.009, M.chrome, verts=6, round=1)
        # the snare on its stand, between the drummer's knees and the hi-hat, a pair of sticks on it
        _drum(F, 'snare', 0.18, 0.13, (-0.26, 0.63, T + 0.3), heads=(1, -1), lugs=8)
        F.box('snare_strainer', 0.02, 0.05, 0.025, (-0.26 + 0.19, 0.63, T + 0.3), M.chrome, bevel=0.004, bevel_segments=1)
        _drum_stand(F, 'snare_stand', -0.26, T + 0.3, 0.56, spread=0.2)
        for s in (-1, 1):
            F.box(f'snare_basket{s}', 0.2, 0.012, 0.012, (-0.26, 0.558, T + 0.3 + s * 0.06), M.chrome, bevel=0.003,
                  bevel_segments=1)
        for k, (dz, yaw) in enumerate(((-0.03, 8), (0.03, -5))):
            x0, x1 = -0.26 - 0.2, -0.26 + 0.18
            pts = [(x0, 0.7, T + 0.3 + dz - 0.02 * G.sind(yaw)), (x1, 0.7, T + 0.3 + dz + 0.02 * G.sind(yaw))]
            F.sweep(f'stick{k}', pts, 0.007, M.cue, verts=6, scale=[1.0, 0.55])
        # the floor tom on three legs, to the right
        fx, fz = 0.46, T + 0.3
        _drum(F, 'floor_tom', 0.2, 0.36, (fx, 0.42, fz), heads=(1,), lugs=8)
        for k in range(3):
            a = 30 + 120 * k
            G.kit.tube(F, f'floor_leg{k}', G.kit.fillet([(fx + 0.215 * G.cosd(a), 0.48, fz + 0.215 * G.sind(a)),
                                                         (fx + 0.245 * G.cosd(a), 0.42, fz + 0.245 * G.sind(a)),
                                                         (fx + 0.25 * G.cosd(a), 0.012, fz + 0.25 * G.sind(a))], 0.02),
                       0.007, M.chrome, verts=6, round=1)
            F.cyl(f'floor_foot{k}', 0.011, 0.016, (fx + 0.25 * G.cosd(a), 0.008, fz + 0.25 * G.sind(a)), M.seat, verts=8)
        # the hi-hat: two 14" cymbals shut on their pole, a clutch, and its pedal, by the left foot
        hx, hz = -0.52, T + 0.22
        _drum_stand(F, 'hihat', hx, hz, 0.94, spread=0.2)
        _cymbal(F, 'hihat_bottom', 0.18, (hx, 0.885, hz), (0, 0, 0), segments=18)
        _cymbal(F, 'hihat_top', 0.18, (hx, 0.906, hz), (0, 0, 0), segments=20)
        F.cyl('hihat_clutch', 0.012, 0.03, (hx, 0.94, hz), M.chrome, verts=8)
        _pedal(F, 'hihat_pedal', hx, hz - 0.12, length=0.2)
        # the crash's and the ride's boom stands
        for name, x, z, top, bx, bz in (('crash', -0.62, T + 0.74, 1.1, CRASH[0], CRASH[2]),
                                         ('ride', 0.72, T + 0.62, 1.02, RIDE[0], RIDE[2])):
            _drum_stand(F, f'{name}_stand', x, z, top, spread=0.22)
            F.cyl(f'{name}_tilter', 0.016, 0.03, (x, top, z), M.chrome, verts=10, bevel=0.003, bevel_segments=1)
            back = (x - (bx - x) * 0.25, top - 0.02, z - (bz - z) * 0.25)
            G.kit.tube(F, f'{name}_boom', [back, (bx, top + 0.08, bz)], 0.008, M.chrome, verts=6, round=1)
            G.kit.tube(F, f'{name}_weight', [back, (back[0] + (x - bx) * 0.05, back[1] - 0.004, back[2] + (z - bz) * 0.05)],
                       0.018, M.chrome, verts=10, round=1)
        # the throne: a round padded seat on a three-legged stand
        tx, tz = F.at(0, 0, T)[:2]
        rings = [(0.17, 0.452), (0.198, 0.462), (0.204, 0.48), (0.2, 0.5), (0.185, 0.513), (0.13, 0.52), (0.0, 0.521)]
        G.loft('throne', [G.ring(r, r, y, 16, at=(tx, tz)) for r, y in rings], material=M.seat)
        F.cyl('throne_base', 0.07, 0.02, (0, 0.444, T), M.chrome, verts=14)
        _drum_stand(F, 'throne_stand', 0, T, 0.44, spread=0.22)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as c:
        _cymbal(F, 'crash', 0.2, CRASH, (-14, 10, 0), segments=22)
        F.cyl('crash_felt', 0.012, 0.02, (CRASH[0], CRASH[1] + 0.02, CRASH[2]), M.seat, verts=8)
        F.cyl('crash_nut', 0.008, 0.012, (CRASH[0], CRASH[1] + 0.035, CRASH[2]), M.chrome, verts=6)
    crash = G.hinged(G.join(c, 'crash'), F.at(*CRASH))
    with G.new_parts() as r:
        _cymbal(F, 'ride', 0.25, RIDE, (-10, -8, 0), segments=32)
        F.cyl('ride_felt', 0.012, 0.02, (RIDE[0], RIDE[1] + 0.02, RIDE[2]), M.seat, verts=8)
        F.cyl('ride_nut', 0.008, 0.012, (RIDE[0], RIDE[1] + 0.035, RIDE[2]), M.chrome, verts=6)
    ride = G.hinged(G.join(r, 'ride'), F.at(*RIDE))
    G.settle(centre=False)
    G.rock(crash, 'cymbals', 7, seconds=1.0, axis='X')
    G.rock(ride, 'cymbals', 4, seconds=1.0, axis='Y')
    return G.meshes()


# --- H2. Games ---------------------------------------------------------------------------------------------
# The chess table: a pedestal table to the front with a board and a game in play, a chair behind it. A black
# pawn is its own part and slides one square while a resident plays (`move`).
SQUARE = 0.043
BOARD_Y = 0.744    # the board's top


def _square_at(col, row):
    """Game (x, depth) of a square's middle: columns across, rows from the back."""
    return (-0.1505 + SQUARE * col, 0.0995 + SQUARE * row)


def _piece(F, name, kind, col, row, material, at=None):
    """A turned chess piece on a square: pawn, rook, knight, bishop, queen or king; or standing at `at` (game
    x, height, depth), off the board. The knight's head is carved in profile, facing the other side."""
    x, z = _square_at(col, row) if at is None else (at[0], at[2])
    y = BOARD_Y if at is None else at[1]
    base = [(0, 0), (0.013, 0), (0.013, 0.004), (0.009, 0.008)]
    profiles = {
        'pawn': base + [(0.005, 0.022), (0.008, 0.025), (0.0075, 0.031), (0.005, 0.038), (0, 0.041)],
        'rook': base + [(0.008, 0.038), (0.011, 0.04), (0.011, 0.05), (0.008, 0.05), (0.008, 0.046), (0, 0.046)],
        'bishop': base + [(0.006, 0.034), (0.01, 0.036), (0.005, 0.039), (0.008, 0.046), (0.006, 0.054),
                          (0.002, 0.057), (0.003, 0.06), (0, 0.062)],
        'queen': base + [(0.006, 0.042), (0.011, 0.044), (0.006, 0.047), (0.009, 0.058), (0.011, 0.064),
                         (0.006, 0.066), (0.004, 0.07), (0, 0.071)],
        'king': base + [(0.006, 0.046), (0.011, 0.048), (0.006, 0.051), (0.009, 0.062), (0.009, 0.066),
                        (0, 0.067)],
        'knight': base + [(0.008, 0.014), (0, 0.014)],
    }
    G.revolve(name, profiles[kind], at=F.at(x, y, z), material=material, segments=6 if kind == 'pawn' else 8)
    if kind == 'king':
        F.box(f'{name}_cross', 0.005, 0.02, 0.005, (x, y + 0.075, z), material)
        F.box(f'{name}_arm', 0.014, 0.005, 0.005, (x, y + 0.077, z), material)
    if kind == 'knight':
        facing = -1 if material == M.piece_dark else 1
        head = [(-0.009, 0.012), (0.008, 0.012), (0.009, 0.026), (0.013, 0.036), (0.017, 0.044), (0.015, 0.05),
                (0.008, 0.051), (0.002, 0.056), (-0.004, 0.054), (-0.01, 0.046), (-0.012, 0.03)]
        F.extrude(f'{name}_head', [(u * facing, v) for u, v in head], 0.011, (x, y, z), material, plane='side')


def chessTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the top 0.64 × 0.48 m, room either side of the board for the taken pieces and the players' elbows (review
        # M-hobbies-02: it was 0.46 × 0.40, the board filling it); the board inlaid, framed in the table's wood
        F.extrude('top', _rr(0.64, 0.48, 0.03, 3), 0.03, (0, 0.725, 0.25), M.table_wood, plane='top', bevel=0.006,
                  bevel_segments=1)
        F.box('board', 0.37, 0.004, 0.37, (0, 0.742, 0.2505), M.table_wood, bevel=0.002, bevel_segments=1)
        F.box('board_dark', 0.344, 0.0005, 0.344, (0, 0.74425, 0.2505), M.square_dark)
        quads = []
        for col in range(8):
            for row in range(8):
                if (col + row) % 2 == 0:
                    x, z = _square_at(col, row)
                    quads.append((x - SQUARE / 2, x + SQUARE / 2, z - SQUARE / 2, z + SQUARE / 2, 0.7448))
        _flat_quads(F, 'squares', quads, M.square_light)
        F.box('apron', 0.56, 0.05, 0.4, (0, 0.685, 0.25), M.table_wood, bevel=0.004, bevel_segments=1)
        # a turned pedestal on three swept snake feet
        _turned(F, 'pedestal', 0, 0.04, 0.25, [(0, 0), (0.05, 0), (0.05, 0.03), (0.036, 0.05), (0.03, 0.1), (0.044, 0.16),
                                               (0.048, 0.2), (0.034, 0.26), (0.025, 0.42), (0.03, 0.55), (0.042, 0.6),
                                               (0.06, 0.62), (0.06, 0.64), (0, 0.64)], M.table_wood, segments=10)
        for k in range(3):
            a = 90 + 120 * k
            c, s = G.cosd(a), G.sind(a)
            pts = [(0.03 * c, 0.13, 0.25 + 0.03 * s), (0.09 * c, 0.11, 0.25 + 0.09 * s), (0.15 * c, 0.055, 0.25 + 0.15 * s),
                   (0.2 * c, 0.016, 0.25 + 0.2 * s), (0.22 * c, 0.012, 0.25 + 0.22 * s)]
            F.sweep(f'foot{k}', pts, None, M.table_wood, profile=_rr(0.05, 0.032, 0.012, 1),
                    scale=[1.0, 0.95, 0.75, 0.5, 0.48])
        _chair(F, -0.21, M.chair_wood, M.seat_pad)
        # the game: the white side to the front
        for kind, col, row in (('rook', 0, 7), ('knight', 6, 7), ('bishop', 3, 6), ('king', 4, 7)):
            _piece(F, f'w_{kind}', kind, col, row, M.piece_light)
        for k, (col, row) in enumerate(((1, 6), (2, 5), (4, 4), (6, 6))):
            _piece(F, f'w_pawn{k}', 'pawn', col, row, M.piece_light)
        for kind, col, row in (('rook', 7, 0), ('knight', 1, 0), ('bishop', 5, 3), ('king', 4, 0), ('queen', 3, 1)):
            _piece(F, f'b_{kind}', kind, col, row, M.piece_dark)
        for k, (col, row) in enumerate(((2, 1), (3, 3), (6, 2))):
            _piece(F, f'b_pawn{k}', 'pawn', col, row, M.piece_dark)
        # the taken pieces beside the board
        for k, (kind, x, z) in enumerate((('pawn', -0.25, 0.14), ('pawn', -0.27, 0.19))):
            _piece(F, f'taken_w{k}', kind, 0, 0, M.piece_light, at=(x, 0.74, z))
        for k, (kind, x, z) in enumerate((('pawn', 0.25, 0.34), ('pawn', 0.27, 0.39))):
            _piece(F, f'taken_b{k}', kind, 0, 0, M.piece_dark, at=(x, 0.74, z))
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        _piece(F, 'pawn', 'pawn', 5, 1, M.piece_dark)
    pawn = G.join(p, 'pawn')
    G.settle(centre=False)
    G.slide(pawn, 'move', (0, -SQUARE, 0), seconds=0.8)
    return G.meshes()


# The pool table: a lacquered frame on four legs, a baize bed with cushions, six pockets and sights, the
# fifteen balls racked at the foot and the cue ball at the head, and a cue lying on the cloth. The triangle
# round the balls is lifted off, carried over the foot rail and hung on the foot end for pool and trick shots
# (`rack_off`, G.path).
BALL = 0.0285
BED = 0.75         # the cloth's top: the rails' top 3.5 cm over it, at 0.785 m (a table is 0.74–0.79 to the rail; review
                   # M-hobbies-06: the cloth was at 0.785, the rails at 0.82)
RAIL_TOP = BED + 0.035
RACK_HUNG = (1.275, BED - 0.185)   # the rack's middle, hung on the foot end (game x, height): its apex at the rail's top


def poolTable():
    F = G.GameFrame()
    colours = [M.ball_yellow, M.ball_blue, M.ball_red, M.ball_purple, M.ball_black, M.ball_orange, M.ball_green,
               M.ball_maroon, M.ball_yellow, M.ball_blue, M.ball_red, M.ball_purple, M.ball_orange, M.ball_green,
               M.ball_maroon]
    apex, row = 0.5, math.sqrt(3) * BALL
    with G.new_parts() as still:
        F.box('bed', 2.3, 0.02, 1.2, (0, BED - 0.01, 0), M.cloth)
        F.box('slate', 2.3, 0.08, 1.2, (0, BED - 0.06, 0), M.pool_frame)
        F.box('apron', 2.4, 0.18, 1.3, (0, BED - 0.185, 0), M.pool_frame, bevel=0.02)
        # beads round the apron's foot and under the rails, mitred at the corners
        box = [(1.2, 0.65), (-1.2, 0.65), (-1.2, -0.65), (1.2, -0.65)]
        G.kit.moulding(F, 'apron_bead', box, [(0.0, 0.0), (-0.012, 0.004), (-0.015, 0.012), (-0.01, 0.02), (0.0, 0.022)],
                       M.pool_frame, at=BED - 0.275, plane='top')
        # a shaped skirt under the long sides' aprons, dropping to a lobe in the middle
        edge = [(-0.95 + 1.9 * i / 24, 0.0) for i in range(25)]
        edge = [(x, BED - 0.29 - 0.012 * math.cos(math.tau * x / 0.475) - 0.035 * math.exp(-(x / 0.12) ** 2)) for x, _ in edge]
        for s in (-1, 1):
            F.extrude(f'skirt{s}', [(0.95, BED - 0.2), (-0.95, BED - 0.2)] + edge, 0.02, (0, 0, s * 0.655), M.pool_frame,
                      bevel=0.004, bevel_segments=1)
        # the rails: one moulded frame round the bed, rounded over at the top outside edge
        rails = [(1.25, 0.7), (-1.25, 0.7), (-1.25, -0.7), (1.25, -0.7)]
        G.kit.moulding(F, 'rails', rails, [(0.0, 0.0), (0.0, 0.118), (0.006, 0.136), (0.02, 0.15), (0.1, 0.15),
                                           (0.1, 0.0)], M.pool_frame, at=RAIL_TOP - 0.15, plane='top')
        # the cushions: cloth noses along the rails' inside, between the pockets
        nose = [(0.0, 0.0), (0.04, 0.0), (0.038, 0.012), (0.026, 0.032), (0.012, 0.03)]
        for s in (-1, 1):
            for x0, x1 in ((-1.09, -0.07), (0.07, 1.09)):
                F.sweep(f'cushion_long{s}{x0}', [(x0, BED, s * 0.6), (x1, BED, s * 0.6)], None, M.cloth,
                        profile=[(u, v * s) for u, v in nose])
            F.sweep(f'cushion_short{s}', [(s * 1.15, BED, -0.54), (s * 1.15, BED, 0.54)], None, M.cloth,
                    profile=[(u, -v * s) for u, v in nose])
            # the pockets: leather mouths cut into the rails, dark inside
            for x in (-1.15, 0, 1.15):
                z = s * (0.6 if x else 0.625)
                _turned(F, f'pocket{s}{x}', x, RAIL_TOP - 0.002, z, [(0.0, 0.0), (0.046, 0.0), (0.058, 0.0), (0.064, 0.003),
                                                                     (0.06, 0.006), (0.048, 0.005), (0.0, 0.003)],
                        M.pocket, segments=14)
            # the sights: mother-of-pearl diamonds on the rails' tops
            for k in range(1, 4):
                for half in (-1, 1):
                    F.box(f'sight{s}{half}{k}', 0.014, 0.002, 0.014, (half * 0.2875 * k, RAIL_TOP + 0.0005, s * 0.65),
                          M.pearl, rot=(0, 0, 45))
            for z in (-0.3, 0, 0.3):
                F.box(f'sight_end{s}{z}', 0.014, 0.002, 0.014, (s * 1.2, RAIL_TOP + 0.0005, z), M.pearl, rot=(0, 0, 45))
            # turned legs, heavy, on square feet
            for sz in (-1, 1):
                _turned(F, f'leg{s}{sz}', s * 1.08, 0.02, sz * 0.5,
                        [(0, 0), (0.085, 0), (0.08, 0.02), (0.06, 0.05), (0.055, 0.1), (0.075, 0.18), (0.085, 0.23),
                         (0.075, 0.28), (0.055, 0.34), (0.06, 0.38), (0.075, 0.4), (0.075, BED - 0.265), (0, BED - 0.265)],
                        M.pool_frame, segments=10)
                F.box(f'foot{s}{sz}', 0.19, 0.02, 0.19, (s * 1.08, 0.01, sz * 0.5), M.pool_frame, bevel=0.006,
                      bevel_segments=1)
        F.box('chalk', 0.022, 0.022, 0.022, (0.9, RAIL_TOP + 0.011, 0.65), M.ball_blue, bevel=0.002, bevel_segments=1)
        # the racked balls, the apex to the head, and the cue ball
        k = 0
        for i in range(5):
            for j in range(i + 1):
                G.sphere(f'ball{k}', BALL, at=F.at(apex + i * row, BED + BALL, (j - i / 2) * 2 * BALL),
                         material=colours[k], segments=10, rings=7)
                k += 1
        G.sphere('cue_ball', BALL, at=F.at(-0.6, BED + BALL, 0), material=M.ball_white, segments=10, rings=7)
        # the cue on the cloth: a wrapped butt, a joint, a tapering shaft, a white ferrule and a chalked tip
        a, b = (-1.02, BED + 0.014, 0.42), (0.36, BED + 0.007, 0.3)

        def along(t):
            return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))
        F.sweep('cue_butt', [along(0), along(0.3)], 0.014, M.cue_butt, verts=8, scale=[1.0, 0.86])
        F.sweep('cue_wrap', [along(0.06), along(0.2)], 0.0142, M.ball_black, verts=8, scale=[1.0, 0.95])
        F.sweep('cue_joint', [along(0.3), along(0.32)], 0.0122, M.pearl, verts=8)
        F.sweep('cue_shaft', [along(0.32), along(0.99)], 0.0118, M.cue, verts=8, scale=[1.0, 0.55])
        F.sweep('cue_ferrule', [along(0.99), along(1.0)], 0.0066, M.pearl, verts=8)
        F.sphere('cue_tip', 0.0064, b, M.ball_blue, segments=8, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # the triangle: a frame with rounded corners round the rack
    centre = apex + 4 * row * 2 / 3
    corners = [(apex, 0), (apex + 4 * row, -4 * BALL), (apex + 4 * row, 4 * BALL)]
    t = 0.016
    inr = 4 * BALL / math.sqrt(3)            # the centres' triangle's inradius
    grow = (inr + BALL + t / 2) / inr
    pts = [(centre + (x - centre) * grow, z * grow) for x, z in corners]
    with G.new_parts() as rp:
        p3 = [(x, BED + 0.015, z) for x, z in pts]
        mid = tuple((p3[2][i] + p3[0][i]) / 2 for i in range(3))
        loop = G.kit.fillet([mid, p3[0], p3[1], p3[2], mid], 0.03)[:-1]
        F.sweep('rack', loop, None, M.rack, profile=_rr(0.03, t, 0.004, 1), closed=True)
    rack = G.hinged(G.join(rp, 'rack'), F.at(centre, BED + 0.015, 0))
    G.settle(centre=False)
    # Racked, the triangle is lifted off the balls, carried over the foot rail and hung up by its apex on the
    # table's foot end (its middle 2.5 cm past the rail, at the apron's height), out of the game's way.
    hung = (RACK_HUNG[0] - centre, RACK_HUNG[1] - (BED + 0.015))
    G.path(rack, 'rack_off', [(0.0, (0, 0, 0), (0, 0, 0)), (0.35, (0, 0, 0.14), (0, 0, 0)),
                              (0.9, (-hung[0], 0, 0.14), (0, 0, 0)), (1.4, (-hung[0], 0, hung[1]), (0, -90, 0))])
    return G.meshes()


# The foosball table: a birch cabinet on legs, a pitch with its lines and goals, and the eight rods of a real
# table, 15 cm apart (review M-hobbies-07: it had four, every handle on the front): from the left goal, blue's
# goalie and defence, red's attack, blue's five midfielders, red's, blue's attack, red's defence and goalie,
# each team's handles on its own side: blue's to the front, where the players stand (characters.ts `STEP_IN`
# lanes: one at the goalie and defence, `FOOS_LANES` in clips/games.ts; one at the midfield and attack), red's
# to the back, short there so the table can stand against a wall. One rod in each blue player's hands is a part
# of its own and kicks back and forth while played (`rods`, a loop); the others stay still.
ROD_Y = 0.84
FOOS_RODS = [-0.525 + 0.15 * k for k in range(8)]
# per rod: blue (front handles) or red, and its men (depth across the pitch)
FOOS_TEAMS = [(True, (0.0,)), (True, (-0.12, 0.12)), (False, (-0.18, 0.0, 0.18)),
              (True, (-0.24, -0.12, 0.0, 0.12, 0.24)), (False, (-0.24, -0.12, 0.0, 0.12, 0.24)),
              (True, (-0.18, 0.0, 0.18)), (False, (-0.12, 0.12)), (False, (0.0,))]
FOOS_KICKING = (1, 5)    # the rods that kick: blue's defence and attack


def _foos_man():
    """A foosball man's outline (game x across the pitch, height from the rod), as seen from the side: a round head,
    shoulders, the rod through the chest, legs to two flat kicking feet."""
    head = [(0.017 * math.cos(math.radians(a)), 0.054 + 0.017 * math.sin(math.radians(a))) for a in range(-55, 240, 37)]
    right = [(0.021, -0.093), (0.021, -0.085), (0.011, -0.078), (0.011, -0.02), (0.017, -0.008), (0.019, 0.028),
             (0.011, 0.038)]
    left = [(-x, y) for x, y in reversed(right)]
    return right + head + left


def _rod_men(F, name, x, front, men):
    team = M.ball_blue if front else M.ball_red
    grip = [(0, 0), (0.017, 0), (0.02, 0.012), (0.018, 0.03), (0.021, 0.05), (0.018, 0.07), (0.02, 0.085), (0.012, 0.09),
            (0, 0.09)]
    if front:   # the handle to the front, the bar's back end on the back wall's outside
        G.kit.tube(F, f'{name}_bar', [(x, ROD_Y, -0.375), (x, ROD_Y, 0.415)], 0.008, M.chrome, verts=8, round=1)
        G.revolve(f'{name}_handle', grip, at=F.at(x, ROD_Y, 0.415), material=M.grip, segments=8, rot=(90, 0, 0))
    else:       # the handle to the back, short, inside the wall's face
        G.kit.tube(F, f'{name}_bar', [(x, ROD_Y, -0.378), (x, ROD_Y, 0.375)], 0.008, M.chrome, verts=8, round=1)
        G.revolve(f'{name}_handle', grip[:3] + [(0.02, 0.04), (0.012, 0.05), (0, 0.05)], at=F.at(x, ROD_Y, -0.378),
                  material=M.grip, segments=8, rot=(-90, 0, 0))
    for z in (-0.37, 0.37):
        F.cyl(f'{name}_bumper{z}', 0.016, 0.016, (x, ROD_Y, z), M.grip, rot=(90, 0, 0), verts=6)
    for z in men:
        F.extrude(f'{name}_man{z}', [(x + u, ROD_Y + v) for u, v in _foos_man()], 0.03, (0, 0, z), team)


def foosball():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('pitch', 1.32, 0.02, 0.64, (0, 0.71, 0), M.field)
        F.box('box_floor', 1.4, 0.04, 0.72, (0, 0.68, 0), M.foos, bevel=0.01)
        for s in (-1, 1):
            F.box(f'side{s}', 1.4, 0.24, 0.04, (0, 0.8, s * 0.34), M.foos, bevel=0.01)
            F.box(f'end{s}', 0.04, 0.24, 0.64, (s * 0.68, 0.8, 0), M.foos, bevel=0.01)
            # a framed panel on each long side, bearings where the rods come through
            G.kit.shaker(F, f'side_panel{s}', 1.24, 0.19, (0, 0.795, s * 0.365), 0.012, M.foos, rail=0.035,
                         panel=M.foos_trim)
            for x in FOOS_RODS:
                F.cyl(f'bearing{s}{x}', 0.017, 0.012, (x, ROD_Y, s * 0.374), M.chrome, rot=(90, 0, 0), verts=8)
            # the goal mouths, the ball return's slot in each end
            F.box(f'goal{s}', 0.006, 0.07, 0.18, (s * 0.658, 0.755, 0), M.grip)
            F.box(f'return{s}', 0.004, 0.05, 0.09, (s * 0.7005, 0.75, 0.18), M.grip, bevel=0.0015, bevel_segments=1)
            F.box(f'box_line{s}', 0.12, 0.0015, 0.004, (s * 0.6, 0.7205, 0.14), M.line)
            F.box(f'box_line_b{s}', 0.12, 0.0015, 0.004, (s * 0.6, 0.7205, -0.14), M.line)
            F.box(f'box_front{s}', 0.004, 0.0015, 0.28, (s * 0.54, 0.7205, 0), M.line)
            for sz in (-1, 1):
                # square legs tapering to levelling feet, a cap on each corner post
                F.sweep(f'leg{s}{sz}', [(s * 0.6, 0.68, sz * 0.3), (s * 0.6, 0.03, sz * 0.3)], None, M.foos_trim,
                        profile=_rr(0.08, 0.08, 0.01, 1), up=(1, 0, 0), scale=[1.0, 0.8])
                F.cyl(f'leg_foot{s}{sz}', 0.03, 0.03, (s * 0.6, 0.015, sz * 0.3), M.grip, verts=8)
                F.box(f'corner{s}{sz}', 0.06, 0.02, 0.06, (s * 0.68, 0.93, sz * 0.34), M.foos_trim, bevel=0.006,
                      bevel_segments=1)
            F.box(f'stretcher{s}', 0.04, 0.04, 0.52, (s * 0.6, 0.12, 0), M.foos_trim)
            # a score rail of sliding beads at each end
            G.kit.tube(F, f'score{s}', [(s * 0.62, 0.935, -0.12), (s * 0.62, 0.935, 0.12)], 0.003, M.chrome, verts=6,
                       round=1)
            for k in range(5):
                F.cyl(f'bead{s}{k}', 0.011, 0.016, (s * 0.62, 0.935, -0.1 + 0.025 * k + (0.06 if k > 1 else 0)),
                      M.ball_white, rot=(90, 0, 0), verts=8)
        F.box('stretcher_long', 1.16, 0.04, 0.04, (0, 0.12, 0), M.foos_trim)
        F.box('half_line', 0.004, 0.0015, 0.62, (0, 0.7205, 0), M.line)
        G.revolve('centre_circle', [(0.077, 0.0), (0.083, 0.0), (0.083, 0.0015), (0.077, 0.0015)],
                  at=F.at(0, 0.72, 0), material=M.line, segments=24)
        F.sphere('ball', 0.018, (0.05, 0.738, 0.05), M.ball_white, segments=10, rings=6)
        for k, (x, (front, men)) in enumerate(zip(FOOS_RODS, FOOS_TEAMS)):
            if k not in FOOS_KICKING:
                _rod_men(F, f'rod{k}', x, front, men)
    G.join(still, 'body', origin=(0, 0, 0))
    rods = []
    for k in FOOS_KICKING:
        x, (front, men) = FOOS_RODS[k], FOOS_TEAMS[k]
        with G.new_parts() as p:
            _rod_men(F, f'rod{k}', x, front, men)
        rods.append(G.hinged(G.join(p, f'rod{k}'), F.at(x, ROD_Y, 0)))
    G.settle(centre=False)
    for rod in rods:
        G.rock(rod, 'rods', 55, seconds=0.8, axis='Y')
    return G.meshes()


# The dartboard: a board on the wall in a wooden cabinet, its twenty segments and rings one flat face. Three
# darts, one part, grow out of the board while a resident plays (`darts`). It hangs at its real height.
BOARD_C = 1.725     # the bull's height
FACE_Z = -0.367     # the board's face, game depth


def _dart_face(name):
    mats = [M.board, M.board_light, M.ring_red, M.ring_green]
    bands = [(0.0159, 0.099, 0), (0.099, 0.107, 2), (0.107, 0.162, 0), (0.162, 0.170, 2)]
    bm = bmesh.new()
    y = -FACE_Z

    def pt(r, a):
        return bm.verts.new((r * G.cosd(a), y, BOARD_C + r * G.sind(a)))

    for s in range(20):
        for sub in range(2):
            a0 = 81 + 18 * s + 9 * sub
            for r0, r1, k in bands:
                f = bm.faces.new([pt(r0, a0), pt(r1, a0), pt(r1, a0 + 9), pt(r0, a0 + 9)])
                f.material_index = k + s % 2
    for k in range(24):
        a0, a1 = 15 * k, 15 * k + 15
        bm.faces.new([pt(0.00635, a0), pt(0.0159, a0), pt(0.0159, a1), pt(0.00635, a1)]).material_index = 3
        bm.faces.new([pt(0, a0), pt(0.00635, a0), pt(0.00635, a1)]).material_index = 2
    return _link(name, bm, mats)


def dartboard():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('back', 0.6, 0.6, 0.02, (0, BOARD_C, -0.418), M.cabinet, bevel=0.004, bevel_segments=1)
        # the cabinet's frame: one moulding round the board, mitred, rounded over at the front
        frame = [(0.31, BOARD_C + 0.31), (-0.31, BOARD_C + 0.31), (-0.31, BOARD_C - 0.31), (0.31, BOARD_C - 0.31)]
        G.kit.moulding(F, 'frame', frame, [(0.0, 0.0), (0.0, 0.042), (0.006, 0.05), (0.022, 0.05), (0.03, 0.044),
                                           (0.034, 0.03), (0.034, 0.0)], M.cabinet, at=-0.408)
        # the board: sisal, its edge rounded, on the back, its front 1 mm behind the printed face (no z-fighting)
        G.revolve('board', [(0, 0), (0.225, 0), (0.225, 0.03), (0.222, 0.036), (0.214, 0.04), (0, 0.04)],
                  at=F.at(0, BOARD_C, -0.408), material=M.board, segments=40, rot=(90, 0, 0))
        _dart_face('face')
        for s in range(20):
            a = 90 + 18 * s
            F.box(f'number{s}', 0.012, 0.016, 0.002, (-0.197 * G.cosd(a), BOARD_C + 0.197 * G.sind(a), FACE_Z),
                  M.board_light, rot=(0, 90 - a, 0))
        # the spider's outer wire (the radial and inner wires shade the flat face's corners dark: README "Shading by
        # corners")
        G.torus('wire', 0.17, 0.0011, at=F.at(0, BOARD_C, FACE_Z + 0.0008), material=M.steel, rot=(90, 0, 0),
                major_segments=40, minor_segments=3)
        G.torus('number_ring', 0.212, 0.0016, at=F.at(0, BOARD_C, FACE_Z), material=M.steel, rot=(90, 0, 0),
                major_segments=40, minor_segments=3)
        # a chalk ledge under the board, chalk on it
        F.box('ledge', 0.5, 0.012, 0.04, (0, BOARD_C - 0.27, -0.388), M.cabinet, bevel=0.003, bevel_segments=1)
        F.box('chalk', 0.06, 0.012, 0.012, (0.15, BOARD_C - 0.258, -0.39), M.board_light, bevel=0.003, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as d:
        kite = [(0.0, 0.003), (0.018, 0.015), (0.042, 0.017), (0.042, -0.017), (0.018, -0.015), (0.0, -0.003)]
        for k, (x, by, flight) in enumerate(((0.02, BOARD_C + 0.01, M.ball_red), (-0.07, BOARD_C + 0.08, M.ball_blue),
                                             (0.09, BOARD_C - 0.06, M.ball_yellow))):
            G.kit.tube(F, f'point{k}', [(x, by, FACE_Z), (x, by, FACE_Z + 0.02)], 0.0013, M.steel, verts=6, round=1)
            G.revolve(f'barrel{k}', [(0, 0), (0.0025, 0), (0.0042, 0.006), (0.0048, 0.02), (0.0046, 0.036),
                                     (0.0032, 0.045), (0, 0.045)], at=F.at(x, by, FACE_Z + 0.02), material=M.steel,
                      segments=8, rot=(90, 0, 0))
            G.kit.tube(F, f'shaft{k}', [(x, by, FACE_Z + 0.065), (x, by, FACE_Z + 0.1)], 0.0022, flight, verts=6, round=1)
            F.extrude(f'flight{k}a', kite, 0.0015, (x, by, FACE_Z + 0.083), flight, plane='side')
            F.extrude(f'flight{k}b', [(v, u) for u, v in kite], 0.0015, (x, by, FACE_Z + 0.083), flight, plane='top')
    # Hidden at rest by flattening into the board itself (on its face, the flights would show as crosses).
    darts = G.join(d, 'darts', origin=F.at(0, BOARD_C, FACE_Z - 0.02))
    darts.scale[1] = 0.001
    G.settle(centre=False, floor=False)   # on the wall, at its real height
    G.grow(darts, 'darts', seconds=0.6, axis='Y')
    return G.meshes()


# The arcade cabinet: the classic upright, its sides cut to the profile (a control panel jutting out, the
# screen sloping back, the marquee over it), side art, a glowing screen and marquee at night. The joystick
# is its own part and waggles while a resident plays (`joystick`, a loop).
SIDE = [(-0.47, 0.0), (0.19, 0.0), (0.19, 0.84), (0.4, 0.865), (0.4, 0.955), (0.16, 1.0), (0.02, 1.5),
        (0.12, 1.53), (0.12, 1.78), (0.09, 1.82), (-0.47, 1.82)]
ART = [(-0.42, 0.08), (0.13, 0.08), (0.13, 0.8), (0.0, 1.0), (-0.1, 1.46), (0.06, 1.55), (0.06, 1.74),
       (-0.42, 1.74)]
STRIPE = [(-0.42, 0.98), (0.0, 1.0), (-0.07, 1.3), (-0.42, 1.22)]
PANEL_TILT = 10


def _panel_y(z):
    """The control panel's top at depth `z`."""
    return 0.935 - (z - 0.25) * math.tan(math.radians(PANEL_TILT))


def _on_panel(x, z, out=0.0):
    """Game point on the control panel's top at (x, depth), `out` above it."""
    t = math.radians(PANEL_TILT)
    return (x, _panel_y(z) + out * math.cos(t), z + out * math.sin(t))


def arcadeCabinet():
    F = G.GameFrame()
    with G.new_parts() as still:
        for s in (-1, 1):
            F.extrude(f'side{s}', SIDE, 0.04, (s * 0.31, 0, 0), M.arcade, plane='side', bevel=0.003, bevel_segments=1)
            F.extrude(f'art{s}', ART, 0.004, (s * 0.332, 0, 0), M.art, plane='side')
            F.extrude(f'stripe{s}', STRIPE, 0.002, (s * 0.335, 0, 0), M.art_stripe, plane='side')
            # the T-moulding along the side's edge, from the floor at the front round to the top at the back
            edge = [(s * 0.31, y, z) for z, y in [(0.19, 0.01)] + SIDE[2:-1]]
            F.sweep(f'tmould{s}', edge, None, M.art_stripe, profile=_rr(0.044, 0.006, 0.002, 1), up=(1, 0, 0))
            F.cyl(f'leveller{s}', 0.02, 0.012, (s * 0.25, 0.006, 0.12), M.bezel, verts=8)
            F.cyl(f'leveller_b{s}', 0.02, 0.012, (s * 0.25, 0.006, -0.42), M.bezel, verts=8)
        F.box('back', 0.58, 1.8, 0.3, (0, 0.9, -0.32), M.arcade)
        F.box('lower', 0.58, 0.84, 0.44, (0, 0.42, -0.03), M.arcade)
        F.box('kick_plate', 0.58, 0.08, 0.006, (0, 0.05, 0.193), M.bezel, bevel=0.002, bevel_segments=1)
        # the coin door: a framed steel door, two coin mechanisms with their slots and lit reject buttons, a lock
        F.box('coin_door', 0.3, 0.22, 0.01, (0, 0.55, 0.192), M.bezel, bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'coin_frame', [(0.155, 0.665), (-0.155, 0.665), (-0.155, 0.435), (0.155, 0.435)],
                       [(0.0, 0.0), (0.0, 0.006), (0.012, 0.006), (0.012, 0.0)], M.chrome, at=0.197)
        for x in (-0.06, 0.06):
            F.box(f'coin_plate{x}', 0.07, 0.11, 0.006, (x, 0.585, 0.199), M.chrome, bevel=0.002, bevel_segments=1)
            F.box(f'coin{x}', 0.03, 0.05, 0.006, (x, 0.6, 0.202), M.ball_red, bevel=0.002, bevel_segments=1)
            F.box(f'coin_slot{x}', 0.004, 0.026, 0.002, (x, 0.6, 0.2055), M.bezel)
            F.box(f'coin_return{x}', 0.03, 0.018, 0.004, (x, 0.545, 0.2015), M.bezel, bevel=0.0015, bevel_segments=1)
        F.cyl('coin_lock', 0.011, 0.008, (0, 0.47, 0.2), M.chrome, rot=(90, 0, 0), verts=10)
        # the control panel: a bevelled deck tilted to the player, art on it, convex buttons and the stick's washer
        F.box('panel', 0.58, 0.07, 0.3, (0, 0.9, 0.25), M.arcade, rot=(PANEL_TILT, 0, 0), bevel=0.012, bevel_segments=2)
        F.box('panel_art', 0.56, 0.004, 0.24, (0, 0.935, 0.26), M.art, rot=(PANEL_TILT, 0, 0))
        F.box('panel_edge', 0.6, 0.012, 0.012, (0, _panel_y(0.4) - 0.006, 0.4), M.art_stripe, bevel=0.004, bevel_segments=1)
        button = [(0, 0), (0.022, 0), (0.022, 0.004), (0.026, 0.005), (0.026, 0.008), (0.017, 0.01), (0.015, 0.016),
                  (0.008, 0.018), (0, 0.018)]
        for k, (x, z, colour) in enumerate(((0.05, 0.25, M.ball_red), (0.13, 0.29, M.ball_blue),
                                            (0.21, 0.25, M.ball_yellow), (0.29, 0.29, M.ball_green),
                                            (0.06, 0.33, M.ball_white), (0.14, 0.37, M.ball_white))):
            G.revolve(f'button{k}', button, at=F.at(*_on_panel(x - 0.03, z)), material=colour, segments=12,
                      rot=(PANEL_TILT, 0, 0))
        for k, x in enumerate((-0.04, 0.04)):
            G.revolve(f'start{k}', [(0, 0), (0.012, 0), (0.012, 0.008), (0.008, 0.01), (0, 0.01)],
                      at=F.at(*_on_panel(x, 0.15)), material=M.marquee, segments=10, rot=(PANEL_TILT, 0, 0))
        F.cyl('stick_washer', 0.026, 0.004, _on_panel(-0.15, 0.28, 0.002), M.bezel, verts=14, rot=(PANEL_TILT, 0, 0))
        # the screen: a curved tube behind a bezel, set back under the marquee
        F.box('screen_shelf', 0.58, 0.06, 0.24, (0, 0.97, 0.1), M.arcade)
        F.box('bezel', 0.58, 0.54, 0.03, (0, 1.25, 0.06), M.bezel, rot=(-15.6, 0, 0), bevel=0.008)
        F.soft('screen', 0.48, 0.38, 0.012, (0, 1.252, 0.077), M.screen, rot=(-15.6, 0, 0), round=0.012, puff=0.006,
               flat=3, round_segments=1)
        # the marquee: a lit box in chrome trim, its speaker grille under it
        F.box('marquee', 0.58, 0.2, 0.04, (0, 1.655, 0.1), M.marquee, bevel=0.006)
        for y in (1.555, 1.755):
            F.box(f'marquee_trim{y}', 0.58, 0.012, 0.012, (0, y, 0.122), M.chrome, bevel=0.003, bevel_segments=1)
        F.box('marquee_trim', 0.58, 0.04, 0.05, (0, 1.535, 0.095), M.bezel)
        F.box('speaker', 0.4, 0.05, 0.006, (0, 1.535, 0.121), M.arcade, bevel=0.002, bevel_segments=1)
        for k in range(7):
            F.box(f'speaker_slot{k}', 0.003, 0.034, 0.002, (-0.15 + 0.05 * k, 1.535, 0.1245), M.bezel)
        F.box('top', 0.66, 0.04, 0.6, (0, 1.8, -0.17), M.arcade, bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    base = _on_panel(-0.15, 0.28)
    with G.new_parts() as j:
        G.kit.tube(F, 'stick', [base, (base[0], base[1] + 0.075, base[2])], 0.006, M.chrome, verts=8, round=1)
        F.cyl('stick_boot', 0.012, 0.012, (base[0], base[1] + 0.008, base[2]), M.bezel, verts=10)
        F.sphere('stick_ball', 0.022, (base[0], base[1] + 0.09, base[2]), M.ball_red, segments=14, rings=9)
    stick = G.hinged(G.join(j, 'joystick'), F.at(*base))
    G.settle()  # on its Model's `centre`: its back moved onto the wall face
    G.rock(stick, 'joystick', 16, seconds=0.8, axis='X')
    return G.meshes()


# The jigsaw table: a small table to the front with a puzzle board on it, half the picture done, loose
# pieces and the box; a stool behind. A patch of finished puzzle spreads across while a resident plays
# (`pieces`), from the done half's edge.
PUZZLE_Y = 0.7585   # the board's top
PUZZLE_EDGE = 0.0   # where the done half ends, game x


def _picture(F, name, x0, x1):
    """The jigsaw's picture between game x `x0` and `x1`: sky, a hill, a band of flowers."""
    w, x = x1 - x0, (x0 + x1) / 2
    F.box(f'{name}_sky', w, 0.0015, 0.18, (x, PUZZLE_Y + 0.0008, 0.11), M.sky)
    F.box(f'{name}_hill', w, 0.0015, 0.14, (x, PUZZLE_Y + 0.0008, 0.27), M.hill)
    F.box(f'{name}_flowers', w, 0.0015, 0.04, (x, PUZZLE_Y + 0.0008, 0.36), M.blush)


def _jigsaw_piece(s=0.045):
    """A jigsaw piece's outline (centred, `s` a side): a knob out on one edge, a socket in another."""
    h = s / 2
    knob = [(h + 0.007 + 0.009 * G.cosd(a), 0.009 * G.sind(a)) for a in range(-130, 131, 52)]
    socket = [(0.009 * G.cosd(a), h - 0.007 + 0.009 * G.sind(a)) for a in range(50, -231, -56)]
    return [(-h, -h), (h, -h)] + knob + [(h, h)] + socket + [(-h, h)]


def puzzleTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.extrude('top', _rr(0.86, 0.5, 0.02, 2), 0.035, (0, 0.735, 0.2), M.table_wood, plane='top', bevel=0.006,
                  bevel_segments=1)
        for sx in (-1, 1):
            for z in (0.0, 0.4):
                _turned(F, f'leg{sx}{z}', sx * 0.38, 0.0, z, [(0, 0), (0.019, 0), (0.023, 0.02), (0.018, 0.05),
                                                             (0.021, 0.42), (0.028, 0.52), (0.024, 0.57), (0.028, 0.6),
                                                             (0.028, 0.72), (0, 0.72)], M.table_wood, segments=10)
            F.extrude(f'apron_side{sx}', [(0.37, 0.72), (0.03, 0.72), (0.03, 0.675), (0.12, 0.665), (0.2, 0.662),
                                          (0.28, 0.665), (0.37, 0.675)], 0.02, (sx * 0.38, 0, 0), M.table_wood,
                      plane='side', bevel=0.003, bevel_segments=1)
        for z in (0.0, 0.4):
            edge = [(0.36 - 0.72 * i / 12, 0.672 - 0.012 * math.sin(math.pi * i / 12)) for i in range(13)]
            F.extrude(f'apron{z}', [(0.36, 0.72), (-0.36, 0.72)] + edge[::-1], 0.02, (0, 0, z), M.table_wood,
                      bevel=0.003, bevel_segments=1)
        # the puzzle board with a raised rim, half the picture done
        F.box('board', 0.58, 0.006, 0.4, (0, 0.7555, 0.2), M.puzzle_board, bevel=0.002, bevel_segments=1)
        G.kit.moulding(F, 'board_rim', [(0.29, 0.4), (-0.29, 0.4), (-0.29, 0.0), (0.29, 0.0)],
                       [(0.0, 0.0), (0.0, 0.01), (0.004, 0.012), (0.008, 0.01), (0.008, 0.0)], M.puzzle_board,
                       at=0.7525, plane='top')
        _picture(F, 'done', -0.27, PUZZLE_EDGE)
        F.cyl('sun', 0.035, 0.0015, (-0.17, PUZZLE_Y + 0.0012, 0.08), M.sun, verts=16, smooth=False)
        for k, (x, z, yaw, colour) in enumerate(((-0.36, 0.36, 23, M.blush), (-0.21, 0.43, 42, M.sky),
                                                 (0.36, 0.05, 78, M.hill), (0.33, 0.17, 72, M.hill),
                                                 (0.2, 0.31, 31, M.sky), (0.24, 0.12, 55, M.blush))):
            y = 0.7555 if abs(x) > 0.29 else 0.7615
            F.extrude(f'loose{k}', _jigsaw_piece(), 0.004, (x, y, z), colour, plane='top', rot=(0, 0, yaw))
        # the box: its lid on, a little proud of the base, the picture on it
        F.box('box', 0.136, 0.032, 0.196, (0.365, 0.7685, 0.3), M.card, bevel=0.002, bevel_segments=1)
        F.box('lid', 0.142, 0.016, 0.202, (0.365, 0.7845, 0.3), M.card, bevel=0.002, bevel_segments=1)
        F.box('box_picture', 0.12, 0.0015, 0.09, (0.365, 0.793, 0.26), M.sky)
        F.box('box_picture_hill', 0.12, 0.0015, 0.08, (0.365, 0.793, 0.345), M.hill)
        _stool(F, -0.25, 0.48, 0.18, M.chair_wood, M.chair_wood)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        _picture(F, 'patch', PUZZLE_EDGE, 0.15)
        F.box('patch_cloud', 0.06, 0.0015, 0.03, (0.09, PUZZLE_Y + 0.0012, 0.08), M.label)
    patch = G.join(p, 'patch', origin=F.at(PUZZLE_EDGE, PUZZLE_Y, 0.2))
    patch.scale[0] = 0.001
    G.settle(centre=False)
    G.grow(patch, 'pieces', seconds=1.2, axis='X')
    return G.meshes()


# --- H3. Crafts and science --------------------------------------------------------------------------------
# The easel: three legs, a tray, a canvas leant back on it, a palette and a jar of brushes. The painting is
# one part, built on the canvas and grown across it from its left edge while a resident paints (`painting`).
EASEL_TILT = -7.29              # the canvas leans back
CANVAS = (0, 1.204, 0.041)      # its middle


def _on_canvas(u, v, out):
    """Game point `u` across, `v` up the canvas from its middle and `out` off its face."""
    t = math.radians(EASEL_TILT)
    up, normal = (math.cos(t), math.sin(t)), (-math.sin(t), math.cos(t))
    return (u, CANVAS[1] + v * up[0] + out * normal[0], CANVAS[2] + v * up[1] + out * normal[1])


def easel():
    F = G.GameFrame()
    tilt = (EASEL_TILT, 0, 0)
    leg = _rr(0.034, 0.024, 0.005, 1)
    with G.new_parts() as still:
        # three legs of square section, tapering to their feet, a brass-capped hinge where they meet
        for name, at, length, rotate in (('leg-1', (-0.175, 0.866, 0.05), 1.74, (11.03, -131.35, 0)),
                                         ('leg1', (0.175, 0.866, 0.05), 1.74, (11.03, 131.35, 0)),
                                         ('leg_back', (0, 0.826, -0.24), 1.66, (11.12, 0, 0))):
            a, b = _ends(at, length, rotate)
            foot, top = (a, b) if a[1] < b[1] else (b, a)
            F.sweep(name, [foot, top], None, M.easel_wood, profile=leg, up=(1, 0, 0),
                    scale=[0.8, 1.0])
            F.cyl(f'{name}_shoe', 0.014, 0.012, (foot[0], 0.006, foot[2]), M.foot, verts=8)
        F.box('head', 0.12, 0.08, 0.07, (0, 1.69, -0.06), M.easel_dark, bevel=0.012)
        F.cyl('hinge', 0.012, 0.13, (0, 1.69, -0.095), M.easel_metal, rot=(0, 90, 0), verts=8)
        # the mast, with the canvas clamp slid down onto the canvas and its wing nut
        F.box('mast', 0.04, 1.0, 0.025, _on_canvas(0, 0.05, -0.03), M.easel_wood, rot=tilt, bevel=0.005)
        clamp = [(0.05, -0.03), (0.05, 0.03), (-0.05, 0.03), (-0.05, -0.03), (-0.03, -0.03), (-0.03, 0.0),
                 (0.03, 0.0), (0.03, -0.03)]
        F.extrude('clamp', clamp, 0.05, (0, 1.581, -0.007), M.easel_dark, rot=tilt, bevel=0.004, bevel_segments=1)
        G.revolve('wing_nut', [(0, 0), (0.012, 0), (0.012, 0.008), (0.006, 0.012), (0.006, 0.02), (0, 0.02)],
                  at=F.at(0, 1.581, -0.032), material=M.easel_metal, segments=10, rot=(-90 + EASEL_TILT, 0, 0))
        F.box('wing', 0.05, 0.014, 0.003, (0, 1.581, -0.05), M.easel_metal, rot=tilt, bevel=0.003, bevel_segments=1)
        F.box('canvas', 0.56, 0.72, 0.025, CANVAS, M.canvas, rot=tilt, bevel=0.004)
        F.box('bar', 0.47, 0.03, 0.02, (0, 0.45, 0.102), M.easel_wood, rot=tilt, bevel=0.005)
        # the tray: a ledge with a lip and a groove for brushes, a knob that sets its height
        F.extrude('tray', [(0.06, 0.8175), (0.154, 0.8175), (0.156, 0.82), (0.156, 0.872), (0.148, 0.876),
                           (0.142, 0.872), (0.142, 0.842), (0.12, 0.842), (0.115, 0.835), (0.105, 0.835),
                           (0.1, 0.842), (0.06, 0.842)], 0.64, (0, 0, 0), M.easel_dark, plane='side', bevel=0.002,
                  bevel_segments=1)
        G.revolve('tray_knob', [(0, 0), (0.012, 0), (0.014, 0.01), (0.01, 0.02), (0, 0.022)], at=F.at(0, 0.83, 0.156),
                  material=M.easel_metal, segments=10, rot=(90, 0, 0))
        # the palette (a thumb hole through it) and its paint, two tubes of paint on the tray
        pal = G.smooth_outline([(0.09, 0.0), (0.07, 0.04), (0.0, 0.05), (-0.07, 0.04), (-0.09, 0.0), (-0.07, -0.04),
                                (-0.02, -0.045), (0.0, -0.03), (0.03, -0.045), (0.08, -0.03)], 2)
        thumb = [(0.045 + 0.009 * math.cos(math.tau * i / 10), 0.01 + 0.009 * math.sin(math.tau * i / 10)) for i in range(10)]
        F.extrude('palette', pal, 0.006, (0.12, 0.845, 0.1), M.palette, holes=[thumb], plane='top', bevel=0.0015,
                  bevel_segments=1)
        for k, (x, z, colour) in enumerate(((0.07, 0.09, M.ball_red), (0.095, 0.115, M.sun), (0.125, 0.12, M.river),
                                            (0.155, 0.11, M.label), (0.18, 0.09, M.tree))):
            F.sphere(f'dab{k}', 0.01, (x, 0.849, z), colour, scale=(1, 1, 0.45), segments=8, rings=5)
        for k, (x, colour) in enumerate(((-0.2, M.ball_blue), (-0.12, M.ball_red))):
            F.sweep(f'paint_tube{k}', [(x, 0.8525, 0.065), (x, 0.8525, 0.125)], 0.01, M.can, verts=8,
                    scale=[0.35, 1.0])
            F.cyl(f'paint_cap{k}', 0.007, 0.012, (x, 0.8525, 0.131), colour, rot=(90, 0, 0), verts=8)
            F.box(f'paint_label{k}', 0.016, 0.004, 0.03, (x, 0.862, 0.1), colour)
        # the jar of brushes on the floor: a turned jar, water in it, three brushes with ferrules and bristles
        G.revolve('jar', [(0, 0), (0.033, 0), (0.035, 0.004), (0.035, 0.064), (0.038, 0.068), (0.036, 0.072),
                          (0.03, 0.072), (0.03, 0.004), (0, 0.004)], at=F.at(0.33, 0.0, 0.3), material=M.jar, segments=14)
        F.cyl('jar_water', 0.03, 0.002, (0.33, 0.05, 0.3), M.river, verts=12, smooth=False)
        for k, (at, rotate, colour) in enumerate((((0.3, 0.165, 0.3), (7.59, -90.0, 0), M.palette),
                                                  ((0.36, 0.165, 0.325), (9.85, 50.19, 0), M.ball_red),
                                                  ((0.33, 0.165, 0.27), (7.59, 180.0, 0), M.foot))):
            a, b = _ends(at, 0.272, rotate)
            lo, hi = (a, b) if a[1] < b[1] else (b, a)

            def on(t):
                return tuple(lo[i] + (hi[i] - lo[i]) * t for i in range(3))
            F.sweep(f'brush{k}', [lo, on(0.5), on(0.82)], 0.004, colour, verts=6, scale=[0.7, 1.0, 0.85])
            F.sweep(f'ferrule{k}', [on(0.82), on(0.9)], 0.0042, M.easel_metal, verts=6, scale=[1.0, 0.9])
            F.sweep(f'bristles{k}', [on(0.9), on(0.96), on(1.0)], 0.004, M.easel_dark, verts=6, scale=[1.0, 1.1, 0.2])
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        # the painting: a sky, a green hill with a rolling skyline, the sun, two trees and a winding stream
        F.box('sky', 0.52, 0.3, 0.003, _on_canvas(0, 0.1925, 0.014), M.sky, rot=tilt)
        hill = [(0.26, -0.33), (0.26, 0.05)] + [(0.26 - 0.52 * i / 12, 0.03 + 0.03 * math.sin(math.pi * i / 6 + 0.6))
                                                for i in range(1, 12)] + [(-0.26, 0.05), (-0.26, -0.33)]
        F.extrude('grass', hill, 0.003, _on_canvas(0, 0, 0.0152), M.hill, rot=tilt)
        F.cyl('paint_sun', 0.035, 0.003, _on_canvas(0.16, 0.242, 0.016), M.sun, rot=(90 + EASEL_TILT, 0, 0),
              verts=16, smooth=False)
        F.sphere('tree', 0.17, _on_canvas(-0.11, 0.042, 0.016), M.tree, scale=(1, 0.012, 0.58),
                 rot=tilt, segments=14, rings=8)
        F.sphere('tree2', 0.12, _on_canvas(0.15, 0.032, 0.016), M.tree_light, scale=(1, 0.012, 0.5),
                 rot=tilt, segments=12, rings=8)
        F.box('trunk', 0.02, 0.07, 0.003, _on_canvas(-0.11, -0.03, 0.0162), M.fretboard, rot=tilt)
        stream = [(-0.05 + 0.3 * i / 8 + 0.03 * math.sin(i), -0.12 - 0.2 * i / 8) for i in range(9)]
        F.extrude('stream', [(x + 0.012 + 0.02 * i / 8, y) for i, (x, y) in enumerate(stream)]
                  + [(x - 0.012 - 0.02 * i / 8, y) for i, (x, y) in reversed(list(enumerate(stream)))],
                  0.003, _on_canvas(0, 0, 0.0162), M.river, rot=tilt)
    painting = G.join(p, 'painting', origin=F.at(*_on_canvas(-0.26, 0, 0.014)))
    painting.scale[0] = 0.001
    G.settle(centre=False)
    G.grow(painting, 'painting', seconds=1.2, axis='X')
    return G.meshes()


# The workbench: a thick butcher-block top on an open frame of square posts, rails and a slatted shelf below,
# the back posts carrying a steel tool board on the wall with its tools hung up; a toolbox, a drill and a tin on
# the top, and a bench vice holding a plank on edge. The top is at 0.90 m, a standing height (workbenches are
# 0.85–0.95; review M-hobbies-01: it was a steel desk at 0.78). The vice's front jaw, its screw and handle are a
# part of their own and slide shut on the plank while a resident tinkers (`vice`).
BENCH_Z = -0.0975     # the top's middle: its back on the wall face
BENCH_D = 0.665
BENCH_TOP = 0.90      # the top's surface
BENCH_UP = BENCH_TOP - 0.7795   # how far the things on the top stand over where they stood on the old desk
VICE_X = 0.1
VICE_Y = BENCH_TOP
VICE_GAP = 0.04
VICE_Z = 0.02        # the back jaw's middle
BOARD_Z = -0.4225    # the tool board's middle: its back on the wall face


def _tool_board(F):
    """The hardboard pegboard on the back posts, a shelf along its top, and the tools hung on it: a saw, a
    square, two screwdrivers, a spanner, pliers, a mallet and a tape measure."""
    z = BOARD_Z + 0.0075 + 0.001     # the board's face
    F.box('board', 1.46, 0.6, 0.015, (0, 1.27, BOARD_Z), M.pegboard, bevel=0.003, bevel_segments=1)
    # its holes: dark squares on its face, one mesh (two triangles each)
    holes = []
    for row in range(8):
        for col in range(18):
            x, y = -0.68 + 0.08 * col, 1.02 + 0.07 * row
            holes.append((x - 0.004, x + 0.004, y - 0.004, y + 0.004))
    bm = bmesh.new()
    for x0, x1, y0, y1 in holes:
        bm.faces.new([bm.verts.new(F.at(x, y, z - 0.0005)) for x, y in ((x0, y0), (x1, y0), (x1, y1), (x0, y1))])
    for f in bm.faces:
        f.normal_update()
        if f.normal.y > 0:
            f.normal_flip()
    _link('pegs', bm, [M.hole])
    # the shelf on two shaped brackets, jars of screws on it
    F.box('board_shelf', 1.46, 0.02, 0.12, (0, 1.58, BOARD_Z + 0.065), M.bench_frame, bevel=0.003, bevel_segments=1)
    for x in (-0.5, 0.5):
        F.extrude(f'shelf_bracket{x}', [(0.0, 0.0), (0.0, 0.07), (0.1, 0.07), (0.1, 0.06), (0.06, 0.045), (0.025, 0.012),
                                        (0.012, 0.0)], 0.02, (x, 1.5, BOARD_Z + 0.0075), M.bench_frame, plane='side',
                  bevel=0.002, bevel_segments=1)
    for k, (x, r, h, colour, lid) in enumerate(((-0.55, 0.035, 0.09, M.jar, M.ball_red), (-0.46, 0.03, 0.07, M.jar, M.vice),
                                                (0.5, 0.04, 0.1, M.jar, M.foot), (0.4, 0.03, 0.06, M.can, M.can))):
        G.revolve(f'shelf_jar{k}', [(0, 0), (r - 0.004, 0), (r, 0.006), (r, h - 0.012), (r * 0.8, h - 0.006), (0, h - 0.006)],
                  at=F.at(x, 1.59, BOARD_Z + 0.06), material=colour, segments=12)
        F.cyl(f'shelf_lid{k}', r * 0.85, 0.012, (x, 1.59 + h - 0.006, BOARD_Z + 0.06), lid, verts=12, bevel=0.002,
              bevel_segments=1)
    # a hand saw, its handle up: a tapering blade, the handle with its grip cut through
    F.extrude('saw_blade', [(-0.035, 1.37), (-0.055, 0.95), (-0.03, 0.95), (0.045, 1.37)], 0.0015, (-0.47, 0, z + 0.006),
              M.chrome)
    handle = G.smooth_outline([(-0.06, 1.35), (0.06, 1.35), (0.065, 1.42), (0.04, 1.47), (-0.045, 1.47), (-0.065, 1.42)], 1)
    grip = [(-0.03, 1.38), (0.03, 1.38), (0.03, 1.44), (-0.03, 1.44)]
    F.extrude('saw_handle', [(x - 0.47, y) for x, y in handle], 0.022, (0, 0, z + 0.014), M.drill,
              holes=[[(x - 0.47, y) for x, y in grip]], bevel=0.004, bevel_segments=1)
    for y in (1.36, 1.39):
        F.cyl(f'saw_screw{y}', 0.006, 0.026, (-0.49, y, z + 0.014), M.brass, rot=(90, 0, 0), verts=6)
    # a carpenter's square
    F.extrude('square', [(-0.315, 1.1), (-0.135, 1.1), (-0.135, 1.13), (-0.285, 1.13), (-0.285, 1.4), (-0.315, 1.4)],
              0.004, (0, 0, z + 0.004), M.chrome)
    # two screwdrivers: turned handles, shafts, tips
    for k, (x, colour) in enumerate(((-0.12, M.ball_red), (-0.06, M.vice))):
        G.revolve(f'driver_handle{k}', [(0, 0), (0.008, 0), (0.012, 0.012), (0.013, 0.08), (0.011, 0.095), (0.006, 0.1),
                                        (0, 0.1)], at=F.at(x, 1.42, z + 0.016), material=colour, segments=8, rot=(180, 0, 0))
        G.kit.tube(F, f'driver_shaft{k}', [(x, 1.32, z + 0.016), (x, 1.2, z + 0.016)], 0.003, M.chrome, verts=6, round=1)
        F.box(f'driver_tip{k}', 0.008, 0.012, 0.0015, (x, 1.196, z + 0.016), M.chrome)
    # a combination spanner: open jaw at the bottom, a ring at the top
    jaw = [(0.02 * math.cos(math.radians(a)), 1.13 + 0.02 * math.sin(math.radians(a))) for a in range(110, 431, 40)]
    F.extrude('spanner_jaw', [(0.007, 1.13), (0.007, 1.11)] + jaw[1:-1] + [(-0.007, 1.11), (-0.007, 1.13)], 0.005,
              (0.04, 0, z + 0.005), M.chrome)
    F.box('spanner', 0.014, 0.2, 0.004, (0.04, 1.25, z + 0.005), M.chrome)
    G.torus('spanner_ring', 0.014, 0.005, at=F.at(0.04, 1.36, z + 0.005), material=M.chrome, rot=(90, 0, 0),
            major_segments=12, minor_segments=4)
    # pliers: two crossed handles, red-sleeved, from the jaws
    for s in (-1, 1):
        G.kit.tube(F, f'plier{s}', [(0.16, 1.32, z + 0.008), (0.16 + s * 0.006, 1.27, z + 0.008), (0.16 + s * 0.028, 1.13, z + 0.008)],
                   0.005, M.chrome, verts=6, round=2)
        G.kit.tube(F, f'plier_grip{s}', [(0.16 + s * 0.01, 1.25, z + 0.008), (0.16 + s * 0.028, 1.13, z + 0.008)], 0.0075,
                   M.ball_red, verts=6, round=1)
    F.cyl('plier_pivot', 0.008, 0.012, (0.16, 1.3, z + 0.008), M.chrome, rot=(90, 0, 0), verts=8)
    # a mallet and a tape measure
    F.sweep('mallet_handle', [(0.3, 1.08, z + 0.013), (0.3, 1.33, z + 0.013)], None, M.plank,
            profile=_rr(0.024, 0.02, 0.008, 1), up=(1, 0, 0), scale=[1.0, 0.85])
    F.sweep('mallet_head', [(0.235, 1.36, z + 0.032), (0.365, 1.36, z + 0.032)], None, M.plank,
            profile=_rr(0.06, 0.06, 0.012, 2))
    G.revolve('tape', [(0, 0), (0.035, 0), (0.04, 0.005), (0.04, 0.035), (0.035, 0.04), (0, 0.04)],
              at=F.at(0.46, 1.3, z), material=M.drill, segments=14, rot=(90, 0, 0))
    F.box('tape_blade', 0.02, 0.008, 0.012, (0.47, 1.256, z + 0.02), M.chrome)
    F.cyl('tape_lock', 0.008, 0.006, (0.46, 1.3, z + 0.042), M.foot, rot=(90, 0, 0), verts=8)


def workbench():
    F = G.GameFrame()
    back, front = BENCH_Z - BENCH_D / 2, BENCH_Z + BENCH_D / 2
    with G.new_parts() as still:
        # the butcher-block top: the strips' glue lines along it
        F.box('top', 1.62, 0.05, BENCH_D, (0, BENCH_TOP - 0.025, BENCH_Z), M.bench_top, bevel=0.006)
        _flat_quads(F, 'glue_lines', [(-0.809, 0.809, back + 0.06 * k - 0.0006, back + 0.06 * k + 0.0006, BENCH_TOP + 0.0003)
                                      for k in range(1, 11)], M.bench_frame)
        # the frame: four posts (the back two up to the tool board's top), rails under the top, stretchers and
        # a slatted shelf low down; bolts where the rails meet the posts
        post, rail_y = 0.07, BENCH_TOP - 0.09
        for sx in (-1, 1):
            x = sx * 0.73
            F.box(f'post_front{sx}', post, BENCH_TOP - 0.05, post, (x, (BENCH_TOP - 0.05) / 2, front - 0.06),
                  M.bench_frame, bevel=0.004, bevel_segments=1)
            F.box(f'post_back{sx}', post, 1.6, post, (x, 0.8, back + post / 2 + 0.015), M.bench_frame, bevel=0.004,
                  bevel_segments=1)
            F.box(f'rail_side{sx}', 0.05, 0.08, BENCH_D - 0.12, (x, rail_y, BENCH_Z - 0.01), M.bench_frame)
            F.box(f'stretcher{sx}', 0.05, 0.06, BENCH_D - 0.12, (x, 0.14, BENCH_Z - 0.01), M.bench_frame)
            for y in (rail_y, 0.14):
                F.cyl(f'bolt{sx}{y}', 0.009, 0.006, (x, y, front - 0.06 + post / 2 + 0.002), M.bench_steel,
                      rot=(90, 0, 0), verts=6)
                F.cyl(f'bolt_side{sx}{y}', 0.009, 0.006, (x + sx * (post / 2 + 0.002), y, front - 0.06), M.bench_steel,
                      rot=(0, 90, 0), verts=6)
            F.box(f'post_foot{sx}f', post + 0.006, 0.012, post + 0.006, (x, 0.006, front - 0.06), M.foot)
            F.box(f'post_foot{sx}b', post + 0.006, 0.012, post + 0.006, (x, 0.006, back + post / 2 + 0.015), M.foot)
        for z, name in ((front - 0.06, 'front'), (back + 0.05, 'back')):
            F.box(f'rail_{name}', 1.39, 0.08, 0.04, (0, rail_y, z), M.bench_frame)
            F.box(f'stretcher_{name}', 1.39, 0.06, 0.04, (0, 0.14, z), M.bench_frame)
        for k in range(5):
            F.box(f'shelf_slat{k}', 1.39, 0.02, 0.09, (0, 0.18, back + 0.1 + 0.11 * k), M.bench_top, bevel=0.003,
                  bevel_segments=1)
        # on the shelf: a crate of offcuts (slatted sides) and a paint can with its handle
        cz = BENCH_Z
        F.box('crate_floor', 0.4, 0.012, 0.34, (-0.4, 0.196, cz), M.toolbox)
        for s in (-1, 1):
            for y in (0.225, 0.32):
                F.box(f'crate_slat{s}{y}', 0.4, 0.05, 0.012, (-0.4, y, cz + s * 0.164), M.toolbox, bevel=0.002,
                      bevel_segments=1)
                F.box(f'crate_end{s}{y}', 0.012, 0.05, 0.316, (-0.4 + s * 0.194, y, cz), M.toolbox, bevel=0.002,
                      bevel_segments=1)
            F.extrude(f'crate_grip{s}', _rr(0.4, 0.06, 0.004, 1), 0.012, (-0.4 + s * 0.194, 0.37, cz), M.toolbox,
                      holes=[_rr(0.1, 0.022, 0.01, 2)], plane='side', bevel=0.002, bevel_segments=1)
        for k, dx in enumerate((-0.12, 0.0, 0.1)):
            F.box(f'offcut{k}', 0.06, 0.32 - 0.05 * k, 0.02, (-0.4 + dx, 0.35 + 0.02 * k, BENCH_Z + 0.04 * k),
                  M.plank, rot=(8 - 6 * k, 0, 0), bevel=0.002, bevel_segments=1)
        G.revolve('paint_can', [(0, 0), (0.088, 0), (0.09, 0.004), (0.09, 0.172), (0.092, 0.178), (0.086, 0.18),
                                (0, 0.18)], at=F.at(0.45, 0.19, BENCH_Z - 0.05), material=M.can, segments=16)
        F.cyl('paint_label', 0.0905, 0.09, (0.45, 0.27, BENCH_Z - 0.05), M.vice, verts=16)
        G.kit.tube(F, 'paint_bail', [(0.45 + 0.09 * math.cos(math.pi * i / 8), 0.36 + 0.05 * math.sin(math.pi * i / 8),
                                      BENCH_Z - 0.05) for i in range(9)], 0.0025, M.bench_steel, verts=4, round=1)
        _tool_board(F)
        # the toolbox: a box with a lid, its edges rounded, a carrying handle on two posts and two latches
        u = BENCH_UP
        F.box('toolbox', 0.42, 0.13, 0.2, (-0.5, 0.845 + u, -0.3), M.toolbox, bevel=0.012, bevel_segments=2)
        F.soft('toolbox_lid', 0.43, 0.05, 0.21, (-0.5, 0.93 + u, -0.3), M.toolbox_lid, round=0.014, puff=0.006, flat=2,
               round_segments=2)
        for x in (-0.62, -0.38):
            F.box(f'latch{x}', 0.04, 0.035, 0.012, (x, 0.905 + u, -0.195), M.chrome, bevel=0.004, bevel_segments=1)
        G.kit.tube(F, 'toolbox_handle', G.kit.fillet([(-0.6, 0.95 + u, -0.3), (-0.6, 0.985 + u, -0.3), (-0.4, 0.985 + u, -0.3),
                                                      (-0.4, 0.95 + u, -0.3)], 0.015), 0.009, M.foot, verts=8, round=1)
        # the drill lying on its side: a body, a grip and battery, a chuck and bit
        F.sweep('drill_body', [(0.37, 0.91 + u, -0.16), (0.44, 0.912 + u, -0.16), (0.52, 0.91 + u, -0.16),
                               (0.545, 0.905 + u, -0.16)], 0.032, M.drill, verts=12, resolution=2, scale=[0.8, 1.0, 1.0, 0.6])
        F.sweep('drill_grip', [(0.495, 0.89 + u, -0.16), (0.51, 0.82 + u, -0.16)], None, M.foot,
                profile=_rr(0.045, 0.035, 0.012, 2), up=(1, 0, 0))
        F.box('drill_battery', 0.09, 0.035, 0.07, (0.51, 0.797 + u, -0.16), M.drill, bevel=0.01, bevel_segments=2)
        G.revolve('drill_chuck', [(0, 0), (0.02, 0), (0.02, 0.02), (0.014, 0.045), (0.006, 0.05), (0, 0.05)],
                  at=F.at(0.37, 0.91 + u, -0.16), material=M.foot, segments=10, rot=(0, -90, 0))
        G.kit.tube(F, 'drill_bit', [(0.32, 0.91 + u, -0.16), (0.26, 0.91 + u, -0.16)], 0.004, M.chrome, verts=6, round=1)
        G.revolve('tin', [(0, 0), (0.045, 0), (0.045, 0.054), (0.047, 0.058), (0.042, 0.06), (0, 0.06)],
                  at=F.at(0.15, 0.78 + u, -0.32), material=M.can, segments=14)
        # a claw hammer and a pencil
        F.sweep('hammer_handle', [(-0.37, 0.791 + u, -0.033), (-0.19, 0.793 + u, 0.033)], None, M.plank,
                profile=_rr(0.022, 0.03, 0.008, 1), scale=[0.9, 1.0])
        F.sweep('hammer_head', [(-0.17, 0.795 + u, -0.025), (-0.21, 0.795 + u, 0.085), (-0.215, 0.795 + u, 0.1),
                                (-0.205, 0.795 + u, 0.11)], 0.014, M.foot, verts=8, scale=[1.1, 0.9, 0.7, 0.4])
        F.sweep('pencil', [(-0.19, 0.785 + u, 0.176), (-0.05, 0.785 + u, 0.124)], 0.005, M.drill, verts=6)
        F.sweep('pencil_tip', [(-0.05, 0.785 + u, 0.124), (-0.035, 0.785 + u, 0.119)], 0.005, M.plank, verts=6,
                scale=[1.0, 0.1])
        # the vice: a base bolted to the top, its back jaw, the plank clamped on edge
        F.box('vice_base', 0.16, 0.02, 0.15, (VICE_X, VICE_Y + 0.01, VICE_Z + 0.02), M.vice, bevel=0.005)
        for dx in (-0.06, 0.06):
            F.cyl(f'vice_bolt{dx}', 0.008, 0.006, (VICE_X + dx, VICE_Y + 0.023, VICE_Z + 0.07), M.bench_steel, verts=6)
        F.sweep('vice_jaw', [(VICE_X - 0.075, VICE_Y + 0.055, VICE_Z), (VICE_X + 0.075, VICE_Y + 0.055, VICE_Z)], None,
                M.vice, profile=[(-0.035, -0.02), (0.035, -0.02), (0.035, 0.0), (0.015, 0.02), (-0.035, 0.02)])
        F.box('vice_pad', 0.13, 0.03, 0.004, (VICE_X, VICE_Y + 0.07, VICE_Z + 0.022), M.bench_steel)
        F.box('plank', 0.4, 0.08, 0.02, (VICE_X - 0.05, VICE_Y + 0.06, VICE_Z + 0.034), M.plank, bevel=0.002)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as v:
        z = VICE_Z + 0.044 + VICE_GAP
        F.sweep('vice_front', [(VICE_X - 0.075, VICE_Y + 0.055, z + 0.02), (VICE_X + 0.075, VICE_Y + 0.055, z + 0.02)], None,
                M.vice, profile=[(-0.035, -0.02), (0.015, -0.02), (0.035, 0.0), (0.035, 0.02), (-0.035, 0.02)])
        F.box('vice_front_pad', 0.13, 0.03, 0.004, (VICE_X, VICE_Y + 0.07, z + 0.002), M.bench_steel)
        for dx in (-0.05, 0.05):
            G.kit.tube(F, f'vice_guide{dx}', [(VICE_X + dx, VICE_Y + 0.035, VICE_Z - 0.03), (VICE_X + dx, VICE_Y + 0.035, z)],
                       0.006, M.chrome, verts=8, round=1)
        G.kit.tube(F, 'vice_screw', [(VICE_X, VICE_Y + 0.05, z + 0.04), (VICE_X, VICE_Y + 0.05, z + 0.09)], 0.007,
                   M.chrome, verts=8, round=1)
        F.cyl('vice_boss', 0.014, 0.02, (VICE_X, VICE_Y + 0.05, z + 0.08), M.chrome, rot=(90, 0, 0), verts=10)
        G.kit.tube(F, 'vice_handle', [(VICE_X - 0.07, VICE_Y + 0.05, z + 0.08), (VICE_X + 0.07, VICE_Y + 0.05, z + 0.08)],
                   0.005, M.chrome, verts=8, round=1)
        for dx in (-0.07, 0.07):
            F.sphere(f'vice_knob{dx}', 0.01, (VICE_X + dx, VICE_Y + 0.05, z + 0.08), M.foot, segments=8, rings=6)
    jaw = G.join(v, 'vice_jaw_front')
    G.settle(centre=False)
    G.slide(jaw, 'vice', (0, VICE_GAP, 0), seconds=0.9)
    return G.meshes()


# The telescope: a refractor on a white tripod with an eyepiece tray. The tube, with its dew shield, finder
# and eyepiece, is its own part about the mount's axle, and tilts up 35° while a resident stargazes (`aim`).
TUBE_DIR = _axis(55.0, 180.0)      # up the tube, towards the objective (back and up)
TUBE_C = (0, 1.18, -0.012)
AXLE = (0, 1.12, 0.0)


def _on_tube(t, off=0.0):
    """Game point `t` along the tube from its middle (towards the objective) and `off` above its axis."""
    normal = (0, -TUBE_DIR[2], TUBE_DIR[1])   # square to the tube, upwards (towards the front)
    return tuple(TUBE_C[i] + t * TUBE_DIR[i] + off * normal[i] * (1 if TUBE_DIR[2] < 0 else -1) for i in range(3))


def telescope():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the tripod: three legs, each an outer tube with a lock where the inner one slides out, a rubber foot
        for k, yaw in enumerate((180.0, 60.0, -60.0)):
            d = (G.sind(yaw), G.cosd(yaw))
            top, foot = (0.045 * d[0], 0.97, 0.045 * d[1]), (0.36 * d[0], 0.02, 0.36 * d[1])
            mid = tuple(top[i] + (foot[i] - top[i]) * 0.55 for i in range(3))
            F.sweep(f'leg{k}', [top, mid], None, M.tripod, profile=_rr(0.034, 0.024, 0.008, 1), up=(1, 0, 0))
            G.kit.tube(F, f'leg_inner{k}', [mid, foot], 0.01, M.tripod, verts=8, round=1)
            F.sweep(f'leg_lock{k}', [tuple(top[i] + (foot[i] - top[i]) * 0.53 for i in range(3)), mid], None, M.foot,
                    profile=_rr(0.042, 0.032, 0.01, 1), up=(1, 0, 0))
            F.sphere(f'foot{k}', 0.018, foot, M.foot, segments=8, rings=5, scale=(1, 1, 0.8))
            G.kit.tube(F, f'spreader{k}', [(0, 0.45, 0), (0.17 * d[0], 0.5, 0.17 * d[1])], 0.006, M.tripod, verts=6,
                       round=1)
        # the eyepiece tray on the spreaders: a three-lobed plate with holes, two eyepieces in it
        tray = G.smooth_outline([(0.13 * G.cosd(a), 0.13 * G.sind(a)) for a in range(90, 450, 60)], 2)
        holes = [[(0.07 * G.cosd(a) + 0.017 * math.cos(math.tau * i / 8), 0.07 * G.sind(a) + 0.017 * math.sin(math.tau * i / 8))
                  for i in range(8)] for a in (90, 210)]
        F.extrude('tray', tray, 0.012, (0, 0.456, 0), M.tripod, holes=holes, plane='top', bevel=0.003, bevel_segments=1)
        for a in (330, 210):
            G.revolve(f'eyepiece{a}', [(0, 0), (0.015, 0), (0.015, 0.03), (0.019, 0.032), (0.019, 0.05), (0.016, 0.055),
                                       (0, 0.055)], at=F.at(0.07 * G.cosd(a), 0.45, 0.07 * G.sind(a)), material=M.foot,
                      segments=10)
        # the head: a hub the legs hinge from, an alt-azimuth mount, a fork, the axle's lock knob
        F.cyl('hub', 0.065, 0.05, (0, 0.98, 0), M.tripod, verts=16, bevel=0.01, bevel_segments=1)
        for k, yaw in enumerate((180.0, 60.0, -60.0)):
            d = (G.sind(yaw), G.cosd(yaw))
            F.cyl(f'hinge{k}', 0.01, 0.05, (0.06 * d[0], 0.965, 0.06 * d[1]), M.chrome, rot=(0, 90, -yaw), verts=8)
        F.box('head', 0.1, 0.08, 0.1, (0, 1.04, 0), M.tripod, bevel=0.015)
        F.extrude('fork', [(-0.035, 0.0), (0.035, 0.0), (0.035, 0.09), (0.0, 0.12), (-0.035, 0.09)], 0.03, (0.065, 1.05, 0),
                  M.tripod, plane='side', bevel=0.006, bevel_segments=1)
        G.revolve('axle_knob', [(0, 0), (0.012, 0), (0.012, 0.01), (0.022, 0.014), (0.022, 0.026), (0.016, 0.03), (0, 0.03)],
                  at=F.at(0.08, 1.12, 0), material=M.foot, segments=12, rot=(0, -90, 0))
        # the slow-motion cable and its knob
        G.kit.tube(F, 'slow_motion', [(0.03, 1.02, 0.05), (0.05, 0.94, 0.11), (0.06, 0.85, 0.16)], 0.004, M.chrome,
                   verts=6, round=3)
        F.cyl('slow_knob', 0.012, 0.03, (0.06, 0.835, 0.168), M.foot, rot=(-30, 0, 0), verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    tube_rot = (-55.0, 0, 0)    # Blender: the tube's axis, up 35° towards the back
    with G.new_parts() as t:
        # the optical tube: a lofted body, its dew shield, the lens cell; tube rings on a dovetail
        F.cyl('tube', 0.06, 0.84, TUBE_C, M.tube, rot=tube_rot, verts=24)
        G.revolve('dew_shield', [(0.058, -0.08), (0.066, -0.08), (0.0685, -0.075), (0.0685, 0.08), (0.065, 0.08),
                                 (0.063, 0.074), (0.063, -0.074), (0.058, -0.074)],
                  at=F.at(*_on_tube(0.42)), material=M.foot, segments=24, rot=(180 + tube_rot[0], 0, 0))
        F.cyl('objective', 0.058, 0.004, _on_tube(0.44), M.optic, rot=tube_rot, verts=24, smooth=False)
        for k, at in enumerate((0.1, -0.15)):
            G.revolve(f'ring{k}', [(0.06, -0.015), (0.068, -0.015), (0.07, -0.01), (0.07, 0.01), (0.068, 0.015),
                                   (0.06, 0.015)], at=F.at(*_on_tube(at)), material=M.chrome, segments=20,
                      rot=(180 + tube_rot[0], 0, 0))
            F.box(f'ring_bolt{k}', 0.018, 0.02, 0.018, _on_tube(at, 0.075), M.chrome, rot=tube_rot, bevel=0.003,
                  bevel_segments=1)
        F.box('dovetail', 0.04, 0.32, 0.016, _on_tube(-0.025, -0.075), M.foot, rot=tube_rot, bevel=0.003, bevel_segments=1)
        F.cyl('cell', 0.05, 0.05, _on_tube(-0.445), M.foot, rot=tube_rot, verts=18)
        # the focuser: a drawtube with two knobs, the diagonal and the eyepiece
        F.cyl('focuser', 0.022, 0.07, _on_tube(-0.5), M.chrome, rot=tube_rot, verts=12)
        F.box('focus_box', 0.04, 0.04, 0.035, _on_tube(-0.47, -0.04), M.foot, rot=tube_rot, bevel=0.005, bevel_segments=1)
        for s in (-1, 1):
            F.cyl(f'focus_knob{s}', 0.014, 0.014, (s * 0.032,) + _on_tube(-0.47, -0.045)[1:], M.foot, rot=(0, 90, 0),
                  verts=10, bevel=0.003, bevel_segments=1)
        F.cyl('diagonal', 0.0175, 0.07, (0, 0.946, 0.41), M.foot, rot=(35.0, 0, 0), verts=12)
        F.cyl('eyecup', 0.02, 0.02, (0, 0.975, 0.43), M.foot, rot=(35.0, 0, 0), verts=12)
        # the finder scope on its two-ring bracket
        F.cyl('finder', 0.0175, 0.22, _on_tube(0.05, 0.11), M.tube, rot=tube_rot, verts=12)
        F.cyl('finder_lens', 0.02, 0.025, _on_tube(0.16, 0.11), M.foot, rot=tube_rot, verts=12)
        for k, at in enumerate((0.0, 0.1)):
            F.box(f'finder_foot{k}', 0.016, 0.04, 0.012, _on_tube(at, 0.08), M.chrome, rot=tube_rot, bevel=0.003,
                  bevel_segments=1)
            G.revolve(f'finder_ring{k}', [(0.0175, -0.005), (0.023, -0.005), (0.023, 0.005), (0.0175, 0.005)],
                      at=F.at(*_on_tube(at, 0.11)), material=M.chrome, segments=12, rot=(180 + tube_rot[0], 0, 0))
        F.box('saddle', 0.06, 0.05, 0.1, (0, 1.12, -0.01), M.foot, rot=tube_rot, bevel=0.006)
    tube = G.hinged(G.join(t, 'tube'), F.at(*AXLE))
    G.settle()  # on its Model's `centre`: the back leg on the wall face
    G.swing(tube, 'aim', 35, seconds=1.2, axis='X')
    return G.meshes()


# The sewing table: a table to the front with the machine, cloth under its needle, thread and a basket; a
# chair behind. The presser foot and needle lower while a resident sews or mends (`presser`), and the
# handwheel turns for as long as they do (`handwheel`, a loop).
MACHINE_Z = 0.28
# The sewer sits behind, facing the front (+z), so their left is game -x: the needle and the head are on their left
# and the pillar and the handwheel on their right, the cloth fed towards them, as on every domestic machine (review
# M-hobbies-08 read it as mirrored: it isn't, checked in play 2026-10-21).
NEEDLE_X = -0.12
WHEEL = (0.212, 0.94, MACHINE_Z)


def _knob_back(F, name, x, y, face, r, material):
    """A turned knob (`kit.knob`) on a face turned to the back (game -z), at depth `face`."""
    obj = G.kit.knob(F, name, x, y, face, r, material, segments=10)
    obj.rotation_euler[0] = math.radians(-90)
    return obj


def sewingTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the table: a top with rounded corners on tapered legs, a shaped apron and a drawer at the back (the sewer's side)
        F.extrude('top', _rr(0.9, 0.46, 0.025, 2), 0.035, (0, 0.735, 0.2), M.sew_table, plane='top', bevel=0.006,
                  bevel_segments=1)
        for sx in (-1, 1):
            for z in (0.02, 0.4):
                F.sweep(f'leg{sx}{z}', [(sx * 0.4, 0.718, z), (sx * 0.4, 0.62, z), (sx * 0.4, 0.0, z)], None, M.sew_legs,
                        profile=_rr(0.04, 0.04, 0.006, 1), up=(1, 0, 0), scale=[1.0, 1.0, 0.65])
            F.box(f'apron_side{sx}', 0.02, 0.07, 0.34, (sx * 0.4, 0.68, 0.21), M.sew_legs)
        edge = [(0.38 - 0.76 * i / 12, 0.652 - 0.012 * math.sin(math.pi * i / 12)) for i in range(13)]
        F.extrude('apron_front', [(0.38, 0.715), (-0.38, 0.715)] + edge[::-1], 0.02, (0, 0, 0.4), M.sew_legs, bevel=0.003,
                  bevel_segments=1)
        F.box('drawer', 0.4, 0.07, 0.02, (0, 0.68, 0.02), M.sew_legs, bevel=0.004, bevel_segments=1)
        F.box('drawer_bead', 0.37, 0.05, 0.004, (0, 0.68, 0.009), M.sew_legs, bevel=0.0015, bevel_segments=1)
        _knob_back(F, 'drawer_knob', 0, 0.68, 0.007, 0.009, M.brass)
        # the machine: one cast body (the pillar, the arm and the head over a gap for the needle), on a deeper bed
        body = [(-0.17, 0.79), (0.21, 0.79), (0.21, 0.82), (0.2, 0.86), (0.2, 1.0), (0.18, 1.03), (-0.14, 1.03),
                (-0.165, 1.01), (-0.165, 0.89), (-0.075, 0.885), (-0.075, 0.95), (0.11, 0.95), (0.11, 0.84),
                (0.09, 0.813), (-0.17, 0.813)]
        F.extrude('machine', body, 0.13, (0, 0, MACHINE_Z), M.machine, bevel=0.018, bevel_segments=2)
        F.extrude('bed', _rr(0.38, 0.17, 0.02, 2), 0.06, (0.02, 0.783, MACHINE_Z), M.machine, plane='top', bevel=0.01,
                  bevel_segments=2)
        # its trim: a stripe along the arm, the stitch dial and lever, a light under the head, the bobbin cover
        F.box('stripe', 0.26, 0.012, 0.002, (-0.01, 1.0, MACHINE_Z - 0.066), M.machine_trim)
        _knob_back(F, 'dial', 0.155, 0.93, MACHINE_Z - 0.063, 0.016, M.machine_trim)
        F.box('stitch_lever', 0.008, 0.04, 0.008, (0.155, 0.87, MACHINE_Z - 0.068), M.machine_trim, bevel=0.002,
              bevel_segments=1)
        F.box('bobbin_cover', 0.07, 0.002, 0.05, (NEEDLE_X + 0.05, 0.8138, MACHINE_Z + 0.02), M.chrome, bevel=0.001,
              bevel_segments=1)
        F.box('needle_plate', 0.06, 0.003, 0.05, (NEEDLE_X, 0.8145, MACHINE_Z), M.chrome)
        for k in range(3):
            F.box(f'feed_dog{k}', 0.004, 0.001, 0.02, (NEEDLE_X - 0.008 + 0.008 * k, 0.8165, MACHINE_Z + 0.004), M.foot)
        G.kit.tube(F, 'presser_bar_top', [(NEEDLE_X, 0.88, MACHINE_Z + 0.012), (NEEDLE_X, 0.86, MACHINE_Z + 0.012)],
                   0.004, M.chrome, verts=6, round=1)
        G.kit.tube(F, 'presser_lever', G.kit.fillet([(NEEDLE_X - 0.03, 0.93, MACHINE_Z + 0.05), (NEEDLE_X - 0.03, 0.93, MACHINE_Z + 0.075),
                                                     (NEEDLE_X - 0.045, 0.91, MACHINE_Z + 0.085)], 0.008), 0.003, M.chrome,
                   verts=6, round=1)
        F.box('tension_dial', 0.012, 0.03, 0.03, (NEEDLE_X + 0.05, 0.99, MACHINE_Z - 0.065), M.machine_trim, bevel=0.004,
              bevel_segments=1)
        # the spool on its pin, the thread up through the guide and down to the needle
        F.cyl('spool_pin', 0.003, 0.05, (0.05, 1.055, MACHINE_Z), M.chrome, verts=6)
        G.revolve('spool', [(0, 0), (0.017, 0), (0.017, 0.004), (0.014, 0.005), (0.014, 0.027), (0.017, 0.028),
                            (0.017, 0.032), (0, 0.032)], at=F.at(0.05, 1.03, MACHINE_Z), material=M.thread_red, segments=12)
        G.kit.tube(F, 'thread', [(0.05, 1.05, MACHINE_Z - 0.016), (-0.02, 1.035, MACHINE_Z - 0.066),
                                 (NEEDLE_X + 0.02, 0.96, MACHINE_Z - 0.068), (NEEDLE_X, 0.9, MACHINE_Z - 0.06)], 0.0008,
                   M.thread_red, verts=4, round=1)
        # the cloth fed under the needle, rumpled; a pin cushion with pins, scissors
        cloth = F.extrude('cloth', _rr(0.4, 0.3, 0.01, 1), 0.004, (-0.2, 0.7545, 0.14), M.fabric_pink, plane='top',
                          rot=(0, 0, -12))
        G.jitter(cloth, 0.003, size=0.08, seed=3.0, up=1.0)
        F.sphere('cushion', 0.03, (0.3, 0.775, 0.36), M.ball_red, scale=(1, 1, 0.75), segments=12, rings=7)
        for k, (dx, dz) in enumerate(((0.0, 0.0), (0.012, 0.01), (-0.01, 0.012), (0.008, -0.012))):
            F.sphere(f'pin{k}', 0.004, (0.3 + dx, 0.8, 0.36 + dz), [M.sun, M.thread_teal, M.label, M.vice][k], segments=6,
                     rings=4)
        for s in (-1, 1):
            F.extrude(f'blade{s}', [(0.0, -0.004), (0.11, -0.001), (0.11, 0.001), (0.0, 0.004)], 0.002,
                      (0.14, 0.7535 + 0.002 * (s + 1), 0.05), M.chrome, plane='top', rot=(0, 0, 8 * s - 30))
            G.torus(f'scissor_ring{s}', 0.014, 0.003, at=F.at(0.14 - 0.016 * G.cosd(30 - 8 * s) + s * 0.006, 0.7555,
                                                             0.05 + 0.016 * G.sind(30 - 8 * s)),
                    material=M.ball_red, major_segments=10, minor_segments=4)
        # two reels of thread, a sewing basket of yarn
        for k, (z, colour) in enumerate(((0.12, M.thread_red), (0.22, M.thread_teal))):
            G.revolve(f'reel{k}', [(0, 0), (0.028, 0), (0.028, 0.006), (0.022, 0.007), (0.022, 0.043), (0.028, 0.044),
                                   (0.028, 0.05), (0, 0.05)], at=F.at(0.36, 0.7525, z), material=M.sew_table, segments=12)
            F.cyl(f'reel_thread{k}', 0.0225, 0.036, (0.36, 0.7775, z), colour, verts=12)
        G.revolve('basket', [(0, 0), (0.094, 0), (0.106, 0.12), (0.11, 0.124), (0.106, 0.128), (0.098, 0.12),
                             (0.088, 0.012), (0, 0.012)], at=F.at(-0.32, 0.7525, 0.36), material=M.basket, segments=20)
        G.kit.tube(F, 'basket_handle', [(-0.32 + 0.105 * math.cos(math.pi * i / 8), 0.875 + 0.09 * math.sin(math.pi * i / 8), 0.36)
                                        for i in range(9)], 0.005, M.basket, verts=6, round=1)
        for k, (dx, dz, colour) in enumerate(((-0.03, 0.0, M.thread_teal), (0.035, 0.02, M.fabric_pink),
                                              (0.0, -0.04, M.machine_trim))):
            F.sphere(f'yarn{k}', 0.045, (-0.32 + dx, 0.84, 0.36 + dz), colour, segments=12, rings=8)
        G.kit.tube(F, 'knitting_needle', [(-0.36, 0.82, 0.36), (-0.24, 0.92, 0.33)], 0.0025, M.chrome, verts=4, round=1)
        _chair(F, -0.22, M.sew_table, M.sew_pad, d=0.4)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        G.kit.tube(F, 'presser_bar', [(NEEDLE_X, 0.88, MACHINE_Z + 0.012), (NEEDLE_X, 0.835, MACHINE_Z + 0.012)], 0.0035,
                   M.chrome, verts=6, round=1)
        F.box('presser_foot', 0.022, 0.006, 0.035, (NEEDLE_X, 0.833, MACHINE_Z + 0.005), M.chrome, bevel=0.002)
        G.kit.tube(F, 'needle_bar', [(NEEDLE_X, 0.88, MACHINE_Z - 0.006), (NEEDLE_X, 0.845, MACHINE_Z - 0.006)], 0.004,
                   M.chrome, verts=6, round=1)
        F.cyl('needle_clamp', 0.006, 0.008, (NEEDLE_X, 0.848, MACHINE_Z - 0.006), M.chrome, verts=6)
        G.kit.tube(F, 'needle', [(NEEDLE_X, 0.845, MACHINE_Z - 0.006), (NEEDLE_X, 0.828, MACHINE_Z - 0.006)], 0.0012,
                   M.chrome, verts=4, round=1)
    presser = G.join(p, 'presser')
    with G.new_parts() as w:
        # the handwheel: a rim, a dished face, a hub
        G.revolve('handwheel', [(0, 0), (0.03, 0), (0.038, -0.002), (0.041, 0.002), (0.041, 0.02), (0.038, 0.024),
                                (0.032, 0.022), (0.016, 0.016), (0, 0.016)], at=F.at(WHEEL[0] - 0.011, WHEEL[1], WHEEL[2]),
                  material=M.machine_trim, segments=24, rot=(0, -90, 0))
        F.cyl('handwheel_hub', 0.015, 0.03, WHEEL, M.chrome, rot=(0, 90, 0), verts=12)
        F.cyl('handwheel_knob', 0.006, 0.02, (WHEEL[0] + 0.02, WHEEL[1] + 0.026, WHEEL[2]), M.machine,
              rot=(0, 90, 0), verts=8)
    wheel = G.hinged(G.join(w, 'handwheel'), F.at(*WHEEL))
    G.settle(centre=False)
    G.slide(presser, 'presser', (0, 0, -0.016), seconds=0.4)
    G.spin(wheel, 'handwheel', turns=1, seconds=1.0, axis='X')
    return G.meshes()


# The chemistry bench: a lab bench with a back board and a shelf of test tubes, three flasks and a
# microscope. The liquid in the middle flask is its own part and rises while a resident experiments
# (`bubble`), its bubbles on top.
BENCH_TOP = 0.9
FLASK = [(0, 0), (0.054, 0), (0.06, 0.006), (0.06, 0.05), (0.024, 0.138), (0.0216, 0.15), (0.0216, 0.176), (0.025, 0.18)]
RISING = (-0.15, -0.05)


def _flask(F, name, x, z, liquid, fill):
    """An Erlenmeyer flask on the bench, glass, with `liquid` up to `fill` (m, below 0.05) inside."""
    G.revolve(name, FLASK, at=F.at(x, BENCH_TOP, z), material=M.glass, segments=16)
    if liquid:
        G.revolve(f'{name}_liquid', [(0, 0.003), (0.054, 0.003), (0.056, 0.008), (0.056, fill), (0, fill)],
                  at=F.at(x, BENCH_TOP, z), material=liquid, segments=16)


def chemistrySet():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the bench: a resin top with a raised lip, on a welded steel frame with a low shelf, levelling feet
        F.box('top', 0.92, 0.04, 0.5, (0, 0.88, -0.15), M.chem_bench, bevel=0.008)
        G.kit.moulding(F, 'lip', [(0.46, 0.1), (-0.46, 0.1), (-0.46, -0.4), (0.46, -0.4)],
                       [(0.0, 0.0), (0.0, 0.006), (0.004, 0.008), (0.008, 0.006), (0.008, 0.0)], M.chem_bench,
                       at=BENCH_TOP, plane='top')
        frame = _rr(0.04, 0.04, 0.006, 1)
        for sx in (-1, 1):
            for z in (-0.36, 0.06):
                F.sweep(f'leg{sx}{z}', [(sx * 0.42, 0.86, z), (sx * 0.42, 0.02, z)], None, M.chem_frame, profile=frame,
                        up=(1, 0, 0))
                F.cyl(f'foot{sx}{z}', 0.018, 0.02, (sx * 0.42, 0.01, z), M.scope, verts=8)
            F.sweep(f'rail_side{sx}', [(sx * 0.42, 0.835, -0.34), (sx * 0.42, 0.835, 0.04)], None, M.chem_frame,
                    profile=_rr(0.04, 0.03, 0.005, 1))
        for z in (-0.36, 0.06):
            F.sweep(f'rail{z}', [(-0.4, 0.835, z), (0.4, 0.835, z)], None, M.chem_frame, profile=_rr(0.04, 0.03, 0.005, 1))
        F.box('shelf_low', 0.86, 0.02, 0.44, (0, 0.3, -0.15), M.chem_bench, bevel=0.004, bevel_segments=1)
        for k, (x, w, h) in enumerate(((-0.25, 0.22, 0.16), (0.02, 0.18, 0.12))):
            F.box(f'storage_box{k}', w, h, 0.3, (x, 0.31 + h / 2, -0.17), M.jar, bevel=0.008, bevel_segments=1)
            F.box(f'storage_label{k}', 0.08, 0.03, 0.002, (x, 0.31 + h * 0.6, -0.019), M.label)
        # the back board with a shelf of test tubes, and a periodic table on it
        F.box('board', 0.8, 0.5, 0.04, (0, 1.15, -0.38), M.chem_board, bevel=0.008)
        for row in range(4):
            for col in range(9):
                if row == 0 and 0 < col < 8:
                    continue
                F.box(f'element{row}_{col}', 0.026, 0.026, 0.002, (0.08 + 0.03 * col, 1.06 + 0.03 * (3 - row), -0.359),
                      [M.liquid_green, M.liquid_yellow, M.liquid_blue, M.liquid_orange][(col + row) % 4])
        F.box('shelf', 0.8, 0.02, 0.16, (0, 1.25, -0.3), M.chem_bench, bevel=0.004)
        # the rack of test tubes on the shelf: a top plate with holes, a base and ends
        holes = [[(-0.24 + 0.12 * k + 0.017 * math.cos(math.tau * i / 8), 0.017 * math.sin(math.tau * i / 8))
                  for i in range(8)] for k in range(5)]
        F.extrude('rack', _rr(0.6, 0.05, 0.004, 1), 0.012, (-0.06, 1.3, -0.29), M.chem_frame, holes=holes, plane='top')
        F.box('rack_base', 0.6, 0.008, 0.05, (-0.06, 1.264, -0.29), M.chem_frame)
        for x in (-0.36, 0.24):
            F.box(f'rack_end{x}', 0.012, 0.05, 0.05, (x, 1.285, -0.29), M.chem_frame)
        for k, colour in enumerate((M.liquid_green, M.liquid_yellow, M.liquid_orange, M.liquid_blue, M.liquid_red)):
            x = -0.3 + 0.12 * k
            G.revolve(f'tube{k}', [(0, 0.0), (0.012, 0.003), (0.015, 0.012), (0.015, 0.14), (0.017, 0.142)],
                      at=F.at(x, 1.26, -0.29), material=M.glass, segments=10)
            G.revolve(f'tube_liquid{k}', [(0, 0.002), (0.012, 0.007), (0.012, 0.05 + 0.012 * (k % 3)),
                                          (0, 0.05 + 0.012 * (k % 3))], at=F.at(x, 1.26, -0.29), material=colour,
                      segments=10)
        # the flasks, the rising one empty at rest
        _flask(F, 'flask_green', -0.3, -0.1, M.liquid_green, 0.045)
        _flask(F, 'flask_rising', RISING[0], RISING[1], None, 0)
        _flask(F, 'flask_blue', 0.25, -0.08, M.liquid_blue, 0.03)
        # a beaker with a spout and graduations
        G.revolve('beaker', [(0, 0), (0.038, 0), (0.04, 0.003), (0.04, 0.09), (0.043, 0.095)], at=F.at(0.36, BENCH_TOP, -0.25),
                  material=M.glass, segments=14)
        G.revolve('beaker_liquid', [(0, 0.003), (0.037, 0.003), (0.037, 0.04), (0, 0.04)],
                  at=F.at(0.36, BENCH_TOP, -0.25), material=M.liquid_yellow, segments=14)
        for k in range(3):
            F.box(f'beaker_mark{k}', 0.012, 0.0015, 0.001, (0.36, BENCH_TOP + 0.03 + 0.02 * k, -0.2095), M.label)
        # a retort stand: a base plate, a rod, a clamp holding a test tube over the burner
        F.box('stand_base', 0.12, 0.012, 0.18, (-0.38, BENCH_TOP + 0.006, -0.25), M.chem_frame, bevel=0.003,
              bevel_segments=1)
        G.kit.tube(F, 'stand_rod', [(-0.4, BENCH_TOP, -0.31), (-0.4, BENCH_TOP + 0.32, -0.31)], 0.005, M.chrome, verts=6,
                   round=1)
        F.box('stand_boss', 0.024, 0.024, 0.024, (-0.4, BENCH_TOP + 0.24, -0.31), M.chem_frame, bevel=0.003,
              bevel_segments=1)
        G.kit.tube(F, 'stand_arm', [(-0.4, BENCH_TOP + 0.24, -0.31), (-0.4, BENCH_TOP + 0.24, -0.22)], 0.004, M.chrome,
                   verts=6, round=1)
        G.torus('stand_clamp', 0.018, 0.004, at=F.at(-0.4, BENCH_TOP + 0.24, -0.2), material=M.chem_frame,
                major_segments=10, minor_segments=4)
        G.revolve('stand_tube', [(0, 0.0), (0.012, 0.003), (0.015, 0.012), (0.015, 0.12), (0.017, 0.122)],
                  at=F.at(-0.4, BENCH_TOP + 0.18, -0.2), material=M.glass, segments=10)
        # the microscope: a cast base and arm, the stage, a turret of objectives, the tube and eyepiece, focus knobs
        sx, sz = 0.05, -0.22
        F.extrude('scope_base', G.smooth_outline([(0.06, -0.075), (0.06, 0.04), (0.03, 0.075), (-0.03, 0.075),
                                                  (-0.06, 0.04), (-0.06, -0.075)], 1), 0.02,
                  (sx, BENCH_TOP + 0.01, sz), M.scope, plane='top', bevel=0.005, bevel_segments=1)
        F.sweep('scope_arm', [(sx, BENCH_TOP + 0.02, sz - 0.06), (sx, BENCH_TOP + 0.1, sz - 0.065),
                              (sx, BENCH_TOP + 0.19, sz - 0.04), (sx, BENCH_TOP + 0.23, sz + 0.0)], None, M.scope,
                profile=_rr(0.036, 0.04, 0.012, 1), up=(1, 0, 0), resolution=2, scale=[1.1, 1.0, 0.9, 0.8])
        F.box('scope_stage', 0.09, 0.008, 0.08, (sx, BENCH_TOP + 0.09, sz + 0.02), M.scope, bevel=0.003)
        F.box('slide', 0.06, 0.002, 0.022, (sx, BENCH_TOP + 0.095, sz + 0.02), M.glass)
        for s in (-1, 1):
            F.box(f'stage_clip{s}', 0.006, 0.002, 0.03, (sx + s * 0.025, BENCH_TOP + 0.096, sz + 0.02), M.chrome)
        G.revolve('scope_light', [(0, 0), (0.016, 0), (0.018, 0.006), (0.012, 0.012), (0, 0.012)],
                  at=F.at(sx, BENCH_TOP + 0.02, sz + 0.02), material=M.chrome, segments=10)
        F.cyl('scope_turret', 0.022, 0.02, (sx, BENCH_TOP + 0.15, sz + 0.02), M.chrome, verts=12)
        for k in range(3):
            a = 90 + 120 * k
            F.cone(f'scope_objective{k}', 0.008, 0.005, 0.035, (sx + 0.012 * G.cosd(a), BENCH_TOP + 0.125,
                                                                 sz + 0.02 + 0.012 * G.sind(a)), M.chrome, verts=8)
        F.cyl('scope_tube', 0.016, 0.12, (sx, BENCH_TOP + 0.21, sz + 0.005), M.scope, rot=(-20, 0, 0), verts=12)
        F.cyl('scope_eyepiece', 0.013, 0.03, (sx, BENCH_TOP + 0.275, sz - 0.018), M.chrome, rot=(-20, 0, 0), verts=10)
        for s in (-1, 1):
            G.revolve(f'scope_knob{s}', [(0, 0), (0.016, 0), (0.017, 0.004), (0.017, 0.01), (0.012, 0.014), (0, 0.014)],
                      at=F.at(sx + s * 0.02, BENCH_TOP + 0.08, sz - 0.05), material=M.chrome, segments=12,
                      rot=(0, -s * 90, 0))
        F.box('notes', 0.14, 0.002, 0.18, (-0.02, BENCH_TOP + 0.001, 0.0), M.sheet, rot=(0, 0, -8))
        for k in range(5):
            F.box(f'notes_line{k}', 0.1, 0.0005, 0.003, (-0.02, BENCH_TOP + 0.0022, -0.05 + 0.025 * k), M.ink, rot=(0, 0, -8))
        F.sweep('pencil', [(0.07, BENCH_TOP + 0.005, -0.02), (0.08, BENCH_TOP + 0.005, 0.1)], 0.004, M.liquid_yellow, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        G.revolve('liquid', [(0, 0.003), (0.054, 0.003), (0.056, 0.008), (0.056, 0.05), (0.04, 0.09), (0, 0.09)],
                  at=F.at(RISING[0], BENCH_TOP, RISING[1]), material=M.liquid_orange, segments=16)
        for k, (dx, dy, r) in enumerate(((0.0, 0.093, 0.009), (0.015, 0.088, 0.007), (-0.012, 0.089, 0.006))):
            F.sphere(f'bubble{k}', r, (RISING[0] + dx, BENCH_TOP + dy, RISING[1] + 0.005 * k), M.liquid_orange,
                     segments=8, rings=6)
    # Hidden at rest by flattening into the bench top (on the flask's floor, it would show through the glass).
    liquid = G.join(p, 'liquid', origin=F.at(RISING[0], BENCH_TOP - 0.01, RISING[1]))
    liquid.scale[2] = 0.001
    G.settle()  # on its Model's `centre`: its back moved onto the wall face
    G.grow(liquid, 'bubble', seconds=1.5, axis='Z')
    return G.meshes()


# The pottery wheel: a low, narrow motor body on four feet under a 0.48 m splash pan, the wheel head and a pot
# on it, a foot pedal by the right foot (game +x: the potter faces the front, +z) and a stool behind. The potter sits with the knees either side of the pan
# (characters.ts `activityPose`, `tinker`), the shins either side of the body, as at a real wheel (review
# I-hobbies-10: the knees went into a 0.66 m drum). The pot is its own part and rises from a squat lump while a
# resident throws (`clay`); the wheel head turns for as long as they do (`wheel`, a loop).
WHEEL_Z = 0.12
HEAD_TOP = 0.475
PAN_R = 0.24        # the splash pan's rim
POT = [(0, 0), (0.064, 0), (0.092, 0.04), (0.1, 0.09), (0.08, 0.15), (0.048, 0.184), (0.052, 0.2), (0.04, 0.2),
       (0.036, 0.18), (0, 0.176)]
LUMP = 0.3          # the pot's height at rest, as a share of its height thrown


def potteryWheel():
    F = G.GameFrame(shift=0.02)        # 2 cm forward: the stool's back on the wall face
    with G.new_parts() as still:
        # the body: a narrow rounded housing under the pan, set forward, on four rubber feet, louvres in its front
        F.soft('base', 0.26, 0.3, 0.42, (0, 0.19, WHEEL_Z + 0.08), M.wheel_body, round=0.03, round_segments=2)
        F.box('base_plinth', 0.3, 0.025, 0.46, (0, 0.04, WHEEL_Z + 0.08), M.wheel_body, bevel=0.006)
        for sx in (-1, 1):
            for z in (-0.1, 0.26):
                F.cyl(f'foot{sx}{z}', 0.022, 0.028, (sx * 0.12, 0.014, WHEEL_Z + z), M.foot, verts=10)
        for k in range(4):
            F.box(f'vent{k}', 0.16, 0.008, 0.004, (0, 0.17 + 0.022 * k, WHEEL_Z + 0.291), M.foot)
        F.box('badge', 0.08, 0.02, 0.003, (0, 0.28, WHEEL_Z + 0.2915), M.chrome, bevel=0.0015, bevel_segments=1)
        F.cyl('column', 0.06, 0.06, (0, 0.37, WHEEL_Z), M.wheel_body, verts=16)
        # the splash pan: a basin with a rolled rim, in two halves (the seam across it), a drain plug
        pan = [(0, -0.5), (0.3, -0.5), (0.42, -0.2), (0.48, 0.42), (0.5, 0.5), (0.48, 0.56), (0.455, 0.5), (0.44, 0.45),
               (0.36, -0.15), (0.25, -0.35), (0, -0.35)]
        G.revolve('splash_pan', [(r * PAN_R * 2, z * 0.12) for r, z in pan], at=F.at(0, 0.42, WHEEL_Z), material=M.pan,
                  segments=32)
        F.box('pan_seam', 0.004, 0.004, 0.46, (0, 0.462, WHEEL_Z), M.wheel_body)
        F.cyl('drain', 0.012, 0.012, (0.12, 0.4, WHEEL_Z - 0.14), M.foot, verts=8)
        F.cyl('spindle', 0.04, 0.03, (0, 0.43, WHEEL_Z), M.chrome, verts=12)
        # the pedal on its cable, a speed switch, tools: a sponge, a wire, a rib
        F.extrude('pedal', [(0.1, 0.0), (-0.1, 0.0), (-0.1, 0.02), (-0.06, 0.05), (0.1, 0.05)], 0.12, (0.3, 0, 0.0),
                  M.foot, plane='side', bevel=0.006, bevel_segments=1)
        F.box('pedal_tread', 0.1, 0.004, 0.14, (0.3, 0.052, 0.02), M.wheel_body, rot=(-6, 0, 0))
        G.kit.tube(F, 'pedal_cable', [(0.3, 0.01, 0.1), (0.25, 0.012, 0.16), (0.15, 0.03, WHEEL_Z + 0.08),
                                      (0.13, 0.05, WHEEL_Z + 0.08)], 0.005, M.foot, verts=6, round=3)
        F.box('switch', 0.004, 0.04, 0.06, (0.132, 0.25, WHEEL_Z + 0.2), M.foot, bevel=0.002)
        F.cyl('switch_knob', 0.01, 0.012, (0.138, 0.25, WHEEL_Z + 0.2), M.ball_red, rot=(0, 90, 0), verts=8)
        F.box('sponge', 0.06, 0.025, 0.04, (-0.17, 0.395, WHEEL_Z + 0.06), M.drill, bevel=0.008, bevel_segments=1)
        G.kit.tube(F, 'wire_tool', [(0.12, 0.39, WHEEL_Z + 0.16), (0.18, 0.39, WHEEL_Z + 0.1)], 0.0015, M.chrome,
                   verts=4, round=1)
        for k, (x, z) in enumerate(((0.12, WHEEL_Z + 0.16), (0.18, WHEEL_Z + 0.1))):
            F.cyl(f'wire_toggle{k}', 0.005, 0.03, (x, 0.392, z), M.stool, rot=(0, 90, 45), verts=6)
        F.extrude('rib', G.smooth_outline([(0.0, -0.03), (0.05, -0.02), (0.06, 0.0), (0.05, 0.02), (0.0, 0.03),
                                           (-0.02, 0.0)], 1), 0.004, (-0.15, 0.395, WHEEL_Z - 0.12), M.stool, plane='top')
        _stool(F, -0.3, 0.44, 0.15, M.stool, M.stool)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as w:
        G.revolve('wheel_head', [(0, 0), (0.15, 0), (0.168, 0.006), (0.17, 0.016), (0.17, 0.026), (0.165, 0.03), (0, 0.03)],
                  at=F.at(0, 0.445, WHEEL_Z), material=M.pan, segments=32)
        F.cyl('head_ring', 0.135, 0.001, (0, HEAD_TOP + 0.0005, WHEEL_Z), M.wheel_body, verts=24, smooth=False)
        F.cyl('slip', 0.11, 0.002, (0, HEAD_TOP + 0.001, WHEEL_Z), M.clay_slip, verts=24, smooth=False)
        for s in (-1, 1):
            F.cyl(f'bat_pin{s}', 0.008, 0.012, (s * 0.135, HEAD_TOP + 0.006, WHEEL_Z), M.chrome, verts=8)
        F.box('clay_smear', 0.05, 0.002, 0.012, (0.0, HEAD_TOP + 0.001, WHEEL_Z + 0.135), M.clay_slip)
    wheel = G.hinged(G.join(w, 'wheel_head'), F.at(0, HEAD_TOP, WHEEL_Z))
    with G.new_parts() as p:
        # the pot: a thrown body with throwing rings, its wall's thickness showing at the rim
        rings = [(0, 0), (0.064, 0), (0.084, 0.02), (0.096, 0.05), (0.1, 0.09), (0.094, 0.12), (0.08, 0.15), (0.058, 0.176),
                 (0.048, 0.186), (0.052, 0.2), (0.046, 0.2), (0.04, 0.188), (0.036, 0.18), (0, 0.176)]
        G.jitter(G.revolve('pot', rings, at=F.at(0, HEAD_TOP, WHEEL_Z), material=M.clay, segments=24), 0.0012, size=0.03,
                 seed=1.7, up=0.0)
    pot = G.join(p, 'pot', origin=F.at(0, HEAD_TOP, WHEEL_Z))
    pot.scale[2] = LUMP
    G.settle(centre=False)
    G.grow(pot, 'clay', seconds=1.5, axis='Z')
    G.spin(wheel, 'wheel', turns=1, seconds=1.0, axis='Z')
    return G.meshes()


# The craft table: a birch top on black hairpin legs, the scrapbook, a stack of paper, a pencil pot, washi
# tape and a box; a stool behind. The scrapbook's cover swings open over its spine for scrapbooking
# (`book_open`), showing the photos on its first page.
TABLE_TOP = 0.7525
BOOK = (-0.1, 0.2)          # the scrapbook's middle (game x, depth)
BOOK_W, BOOK_D = 0.3, 0.22
PAGES_TOP = TABLE_TOP + 0.004 + 0.015


def craftTable():
    F = G.GameFrame()
    bx, bz = BOOK
    with G.new_parts() as still:
        # a birch top, its plywood edge showing, on four hairpin legs (each one rod bent in a U), a cutting mat on it
        F.extrude('top', _rr(1.7, 0.5, 0.015, 2), 0.035, (0, 0.735, 0.2), M.craft_top, plane='top', bevel=0.004,
                  bevel_segments=1)
        for sx in (-1, 1):
            for z in (0.0, 0.4):
                x = sx * 0.78
                F.box(f'plate{sx}{z}', 0.08, 0.006, 0.08, (x, 0.7145, z), M.craft_metal, bevel=0.002, bevel_segments=1)
                pin = [(x, 0.712, z - 0.03), (x, 0.006, z - 0.004), (x, 0.006, z + 0.004), (x, 0.712, z + 0.03)]
                G.kit.tube(F, f'pin{sx}{z}', G.kit.fillet(pin, 0.006), 0.0055, M.craft_metal, verts=6, round=1)
        F.box('mat', 0.6, 0.002, 0.4, (-0.05, TABLE_TOP + 0.001, 0.19), M.tree, bevel=0.001, bevel_segments=1)
        _flat_quads(F, 'mat_grid', [(-0.34 + 0.05 * k, -0.338 + 0.05 * k, 0.0, 0.38, TABLE_TOP + 0.0025) for k in range(12)]
                    + [(-0.34, 0.24, 0.02 + 0.05 * k, 0.022 + 0.05 * k, TABLE_TOP + 0.0025) for k in range(8)], M.label)
        # the scrapbook: back cover, pages (their edges in leaves), spine with rings, the first page's photos
        F.box('back_cover', BOOK_W, 0.004, BOOK_D, (bx, TABLE_TOP + 0.002, bz), M.scrapbook, bevel=0.0015)
        F.box('pages', BOOK_W - 0.01, 0.015, BOOK_D - 0.01, (bx + 0.003, TABLE_TOP + 0.0115, bz), M.pages)
        for k in range(3):
            F.box(f'page_edge{k}', 0.0005, 0.0006, BOOK_D - 0.012, (bx + BOOK_W / 2 - 0.0018, TABLE_TOP + 0.007 + 0.004 * k,
                                                                    bz), M.paper)
        F.box('spine', 0.008, 0.023, BOOK_D, (bx - BOOK_W / 2, TABLE_TOP + 0.0115, bz), M.scrapbook, bevel=0.002)
        for k in range(4):
            G.torus(f'ring{k}', 0.01, 0.0015, at=F.at(bx - BOOK_W / 2 + 0.004, TABLE_TOP + 0.0125, bz - 0.08 + 0.053 * k),
                    material=M.craft_metal, rot=(90, 0, 0), major_segments=8, minor_segments=3)
        for k, (dx, dz, w, d, colour) in enumerate(((-0.06, -0.04, 0.1, 0.07, M.photo), (0.07, 0.03, 0.08, 0.1, M.photo_warm),
                                                    (-0.05, 0.06, 0.07, 0.05, M.tape_gold))):
            F.box(f'page_photo{k}', w, 0.001, d, (bx + dx, PAGES_TOP + 0.0004, bz + dz), colour, rot=(0, 0, 5 - 4 * k))
        # paper, a pencil pot of pencils, a craft knife and a ruler on the mat, washi tape, glue, a box of buttons
        F.box('paper', 0.4, 0.008, 0.3, (0.25, TABLE_TOP + 0.004, 0.15), M.paper, rot=(0, 0, 8), bevel=0.002)
        F.box('paper2', 0.3, 0.004, 0.22, (0.27, TABLE_TOP + 0.01, 0.16), M.blush, rot=(0, 0, -6), bevel=0.0015)
        G.revolve('pencil_pot', [(0, 0), (0.04, 0), (0.042, 0.1), (0.037, 0.1), (0.035, 0.006), (0, 0.006)],
                  at=F.at(0.5, TABLE_TOP, 0.33), material=M.cup, segments=14)
        for k, (at, rotate, colour) in enumerate((((0.485, 0.83, 0.32), (12.6, -153.43, 0), M.ball_red),
                                                  ((0.515, 0.825, 0.34), (13.95, 26.57, 0), M.sun),
                                                  ((0.5, 0.828, 0.31), (10.0, 90.0, 0), M.tape_blue))):
            a, b = _ends(at, 0.15, rotate)
            lo, hi = (a, b) if a[1] < b[1] else (b, a)
            F.sweep(f'pencil{k}', [lo, hi], None, colour, profile=G.circle(0.004, 6))
            F.sweep(f'pencil_tip{k}', [hi, tuple(hi[i] + (hi[i] - lo[i]) * 0.06 for i in range(3))], 0.004, M.paper,
                    verts=6, scale=[1.0, 0.15])
        F.extrude('ruler', _rr(0.3, 0.03, 0.002, 1), 0.002, (-0.05, TABLE_TOP + 0.003, 0.36), M.chrome, plane='top',
                  rot=(0, 0, 3))
        F.sweep('knife', [(0.14, TABLE_TOP + 0.007, 0.35), (0.25, TABLE_TOP + 0.007, 0.375)], 0.006, M.ball_yellow, verts=6)
        F.sweep('knife_blade', [(0.25, TABLE_TOP + 0.007, 0.375), (0.27, TABLE_TOP + 0.007, 0.38)], 0.004, M.chrome,
                verts=4, scale=[1.0, 0.1])
        for k, (x, z, colour) in enumerate(((0.65, 0.22, M.tape_blue), (0.74, 0.14, M.tape_gold), (0.66, 0.08, M.blush))):
            G.revolve(f'tape{k}', [(0.026, 0), (0.052, 0), (0.052, 0.015), (0.026, 0.015), (0.026, 0)],
                      at=F.at(x, TABLE_TOP, z), material=colour, segments=16)
        G.revolve('glue', [(0, 0), (0.02, 0), (0.02, 0.07), (0.012, 0.08), (0.006, 0.09), (0.0, 0.092)],
                  at=F.at(0.6, TABLE_TOP, 0.36), material=M.label, segments=10)
        F.cyl('glue_cap', 0.008, 0.015, (0.6, TABLE_TOP + 0.09, 0.36), M.ball_red, verts=8)
        F.box('box', 0.16, 0.05, 0.1, (0.72, TABLE_TOP + 0.025, 0.37), M.scrapbook, bevel=0.01)
        F.box('box_lid', 0.165, 0.012, 0.105, (0.72, TABLE_TOP + 0.054, 0.37), M.tape_gold, bevel=0.004)
        _stool(F, -0.25, 0.48, 0.19, M.craft_metal, M.craft_top)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as c:
        F.box('cover', BOOK_W, 0.004, BOOK_D, (bx, PAGES_TOP + 0.002, bz), M.scrapbook, bevel=0.0015)
        F.box('cover_label', 0.12, 0.002, 0.07, (bx + 0.02, PAGES_TOP + 0.005, bz - 0.02), M.pages, bevel=0.0008,
              bevel_segments=1)
        F.box('cover_photo', 0.08, 0.002, 0.06, (bx - 0.05, PAGES_TOP + 0.005, bz + 0.05), M.photo, rot=(0, 0, -6))
        for k, colour in enumerate((M.tape_blue, M.tape_gold)):
            F.box(f'cover_tape{k}', 0.03, 0.0025, 0.01, (bx - 0.05 + 0.03 * (2 * k - 1), PAGES_TOP + 0.0055,
                                                          bz + 0.05 + 0.03 * (2 * k - 1)), colour, rot=(0, 0, 40))
        F.box('inside_photo', 0.1, 0.001, 0.08, (bx + 0.03, PAGES_TOP - 0.0003, bz), M.photo_warm)
    cover = G.hinged(G.join(c, 'cover'), F.at(bx - BOOK_W / 2, PAGES_TOP + 0.004, bz))
    G.settle(centre=False)
    G.swing(cover, 'book_open', 176, seconds=1.0, axis='Y')
    return G.meshes()


# The seated hobbies' parts wait while the resident sits down (characters.ts: the `sitDown` clip, about 1 s), so
# the board, the keys, the presser and the wheel move once the hands are there, as the weight bench's bar does
# (fitness.py `BAR_DELAY`). A part that goes round while used does so from then too: `every` a hair under its
# round (its clip, 1.0 s), so the rounds run on without a pause.
SIT = 0.9


def _after_sitting(clip, every=None):
    return {'clip': clip, 'delay': SIT, **({'every': every} if every else {})}


FAMILY = G.Family(
    'hobbies', M,
    [G.Model('piano', piano, size=(1.54, 1.48, 0.827), key='model.piano', centre=(0, -0.0435 + PIANO_FORWARD),
             motions={'uses': {'play': _after_sitting('fallboard')}}, paint=[('piano', 'Body'), ('piano_trim', 'Trim')]),
     G.Model('ukulele', ukulele, size=(0.348, 0.841, 0.258), key='model.ukulele', centre=(0, 0.065),
             motions={'uses': {'music': 'off_stand'}}, paint=[('uke', 'Body & neck'), ('uke_top', 'Top'), ('stand', 'Stand')]),
     G.Model('guitarElectric', guitarElectric, size=(0.46, 1.001, 0.584), key='model.guitarElectric',
             centre=(0, -0.128), motions={'uses': {'music': 'off_stand'}}, paint=[('electric', 'Guitar'), ('amp', 'Amp')]),
     G.Model('keyboardDigital', keyboardDigital, size=(1.392, 1.047, 0.77), key='model.keyboardDigital',
             centre=(0, -0.025 + KB_FORWARD), paint=[('kb_body', 'Body'), ('kb_trim', 'Trim')]),
     G.Model('drumKit', drumKit, size=(1.732, 1.238, 1.165), key='model.drumKit', centre=(0.05, -0.0175),
             motions={'uses': {'drum': _after_sitting('cymbals', 0.9), 'practise': _after_sitting('cymbals', 0.9)}},
             paint=[('shell', 'Shells'), ('chrome', 'Hardware')]),
     G.Model('guitarAcoustic', guitarAcoustic, size=(0.4, 1.161, 0.405), key='model.guitarAcoustic',
             centre=(0, -0.0217), motions={'uses': {'music': 'off_stand'}}, paint=[('acoustic', 'Body'), ('spruce', 'Top'), ('stand', 'Stand')]),
     G.Model('chessTable', chessTable, size=(0.64, 0.92, 0.89), key='model.chessTable', centre=(0, 0.045),
             motions={'uses': {'chess': _after_sitting('move')}}, paint=[('table_wood', 'Table'), ('chair_wood', 'Chair')]),
     G.Model('poolTable', poolTable, size=(2.5, 0.807, 1.4), key='model.poolTable',
             motions={'uses': {'pool': 'rack_off', 'trickShots': 'rack_off'}},
             paint=[('pool_frame', 'Frame'), ('cloth', 'Cloth')]),
     G.Model('foosball', foosball, size=(1.42, 0.947, 0.935), key='model.foosball', centre=(0, 0.0375),
             motions={'loops': {'foosball': 'rods', 'practise': 'rods'}}, paint=[('foos', 'Table'), ('foos_trim', 'Trim')]),
     G.Model('dartboard', dartboard, size=(0.62, 0.62, 0.07), key='model.dartboard', centre=(0, -0.393), floor=False,
             motions={'uses': {'darts': 'darts'}}, paint=[('board', 'Board'), ('cabinet', 'Cabinet')]),
     G.Model('arcadeCabinet', arcadeCabinet, size=(0.672, 1.82, 0.872), key='model.arcadeCabinet',
             centre=(0, 0.006), motions={'loops': {'arcade': 'joystick'}},
             paint=[('arcade', 'Cabinet'), ('art', 'Art')]),
     G.Model('puzzleTable', puzzleTable, size=(0.865, 0.794, 0.891), key='model.puzzleTable', centre=(0.0025, 0.0155),
             motions={'uses': {'jigsaw': _after_sitting('pieces')}}, paint=[('table_wood', 'Table'), ('chair_wood', 'Stool')]),
     G.Model('easel', easel, size=(0.704, 1.747, 0.762), key='model.easel', centre=(0.0318, -0.0347),
             motions={'uses': {'paint': 'painting'}}, paint=[('easel_wood', 'Wood'), ('easel_metal', 'Metal')]),
     G.Model('workbench', workbench, size=(1.62, 1.69, 0.665), key='model.workbench', centre=(0, -0.0975),
             motions={'uses': {'tinker': 'vice'}},
             paint=[('bench_top', 'Top'), ('bench_frame', 'Frame'), ('pegboard', 'Tool board')]),
     G.Model('telescope', telescope, size=(0.664, 1.522, 0.912), key='model.telescope', centre=(0, 0.026),
             motions={'uses': {'stargaze': 'aim'}}, paint=[('tube', 'Tube'), ('tripod', 'Tripod')]),
     G.Model('sewingTable', sewingTable, size=(0.9, 1.08, 0.89), key='model.sewingTable', centre=(0, 0.025),
             motions={'uses': {use: [_after_sitting('presser'), _after_sitting('handwheel', 0.9)] for use in ('sew', 'mend')}},
             paint=[('sew_table', 'Table'), ('sew_legs', 'Legs'), ('machine', 'Machine')]),
     G.Model('chemistrySet', chemistrySet, size=(0.92, 1.4, 0.5), key='model.chemistrySet', centre=(0, -0.18),
             motions={'uses': {'experiment': 'bubble'}}, paint=[('chem_bench', 'Bench'), ('glass', 'Glassware')]),
     G.Model('potteryWheel', potteryWheel, size=(0.6, 0.544, 0.88), key='model.potteryWheel', centre=(0.06, 0.01),
             motions={'uses': {'throw': [_after_sitting('clay'), _after_sitting('wheel', 0.9)]}},
             paint=[('wheel_body', 'Wheel'), ('stool', 'Stool')]),
     G.Model('craftTable', craftTable, size=(1.7, 0.904, 0.89), key='model.craftTable', centre=(0, 0.005),
             motions={'uses': {'scrapbook': _after_sitting('book_open')}}, paint=[('craft_top', 'Wood'), ('craft_metal', 'Metal')])],
    comment='Blender-built hobby models (tools/art/blender/families/hobbies.py): music, games, crafts and '
            'science. Listed in the base manifest\'s packs. The materials map onto the game\'s finishes so '
            'a model costs one draw call per finish group. The piano lifts its fallboard, the drums\' '
            'cymbals and the foosball rods move while played, the darts, the painting, the jigsaw patch, '
            'the pot and the liquid grow, and the other hobbies move a part while a resident uses them.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
