# The hobbies: music (H1: `model.piano`, `model.ukulele`, `model.guitarElectric`, `model.keyboardDigital`,
# `model.drumKit`, `model.guitarAcoustic`), games (H2: `model.chessTable`, `model.poolTable`, `model.foosball`,
# `model.dartboard`, `model.arcadeCabinet`, `model.puzzleTable`) and crafts and science (H3: `model.easel`,
# `model.workbench`, `model.telescope`, `model.sewingTable`, `model.chemistrySet`, `model.potteryWheel`,
# `model.craftTable`), over their primitive-part placeholders (`furniture.py`, `catalog.py`, `catalog2.py`),
# which stay the fallback. Notes: hobbies.md.
#
# Each is built where its placeholder stands, in the game's axes (`F`), and stood on the floor with
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
# The guitars and the ukulele are their own part, off their stand, for a clip once the music animation
# draws the instrument in the hands (08-hobbies.md); until then they stay put. The digital piano is still.
#
# Run:  python3 -B tools/art/blender/build.py hobbies [--only piano] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
import bpy  # noqa: E402

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
    bench_steel=('#8A9298', 'metal'),  # its pedestals and frame
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
    """The game direction of a placeholder part's long axis (its local y) under its `rotate`: Babylon's
    Euler degrees (yaw y, pitch x, roll z), read as check_glb.placeholder_box reads them."""
    rx, ry, rz = (math.radians(a) for a in (rx, ry, rz))
    cy, sy, cp, sp, cr, sr = (math.cos(ry / 2), math.sin(ry / 2), math.cos(rx / 2), math.sin(rx / 2),
                              math.cos(rz / 2), math.sin(rz / 2))
    x = cy * sp * cr + sy * cp * sr
    y = sy * cp * cr - cy * sp * sr
    z = cy * cp * sr - sy * sp * cr
    w = cy * cp * cr + sy * sp * sr
    return (2 * (x * y - z * w), 1 - 2 * (z * z + x * x), 2 * (y * z + x * w))


def _rod(F, name, at, length, rotate, r, material, verts=10):
    """A placeholder's turned cylinder (centre `at`, `length` long, `rotate`) as a rod between its ends."""
    d = _axis(*rotate)
    a = [at[i] - d[i] * length / 2 for i in range(3)]
    b = [at[i] + d[i] * length / 2 for i in range(3)]
    return G.rod(name, F.at(*a), F.at(*b), r, material=material, verts=verts)


def _slab(F, name, rx, ry, depth, at, material, verts=28, smooth=True, rot=(90, 0, 0)):
    """An elliptic slab facing the front (an instrument's body, a palette): `rx` across, `ry` up."""
    return F.cyl(name, rx, depth, at, material, rot=rot, scale=(1, ry / rx, 1), verts=verts, smooth=smooth)


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


def _prism(name, outline, x0, x1, material):
    """A panel cut to `outline` [(game depth, height)] running across from game x `x0` to `x1` (the arcade
    cabinet's sides and their art)."""
    bm = bmesh.new()
    a = [bm.verts.new((-x0, -z, y)) for z, y in outline]
    b = [bm.verts.new((-x1, -z, y)) for z, y in outline]
    bm.faces.new(a)
    bm.faces.new(b[::-1])
    for i in range(len(outline)):
        j = (i + 1) % len(outline)
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return _link(name, bm, [material])


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
    """A side chair behind a hobby table, facing the front: its seat at depth `z`, a pad, two back posts
    with rails and slats."""
    back = z - d / 2 + 0.02
    F.box('chair_seat', w, 0.04, d, (0, 0.42, z), frame)
    F.box('chair_pad', w - 0.04, 0.035, d - 0.05, (0, 0.4575, z + 0.01), pad, bevel=0.01, bevel_segments=1)
    for s in (-1, 1):
        F.box(f'chair_leg{s}', 0.035, 0.4, 0.035, (s * (w / 2 - 0.03), 0.2, z + d / 2 - 0.03), frame)
        F.box(f'chair_post{s}', 0.035, 0.92, 0.035, (s * (w / 2 - 0.03), 0.46, back), frame, bevel=0.006)
        F.box(f'chair_stretcher{s}', 0.02, 0.02, d - 0.08, (s * (w / 2 - 0.03), 0.14, z), frame)
    F.box('chair_rail_top', w - 0.02, 0.1, 0.03, (0, 0.85, back), frame, bevel=0.01, bevel_segments=1)
    F.box('chair_rail_mid', w - 0.08, 0.035, 0.025, (0, 0.62, back), frame)
    for k in (-1, 0, 1):
        F.box(f'chair_slat{k}', 0.03, 0.18, 0.015, (k * 0.1, 0.71, back), frame)


def _stool(F, z, top, r, legs, seat, x=0.0):
    """A round stool behind a hobby table: its seat of radius `r` with its top at `top`, four splayed legs
    and a foot ring."""
    F.cyl('stool_seat', r, 0.04, (x, top - 0.02, z), seat, verts=24, bevel=0.008)
    for k in range(4):
        a = 45 + 90 * k
        G.rod(f'stool_leg{k}', F.at(x + 0.1 * G.cosd(a), top - 0.03, z + 0.1 * G.sind(a)),
              F.at(x + 0.15 * G.cosd(a), 0.002, z + 0.15 * G.sind(a)), 0.016, material=legs, verts=8)
    G.torus('stool_ring', 0.131, 0.007, at=F.at(x, 0.16, z), material=legs, major_segments=20, minor_segments=6)


def _upright(parts, name):
    """Joins an instrument built upright (its foot at the origin) into one part, its origin at its foot."""
    return G.join(parts, name, origin=(0, 0, 0))


def _lean(F, obj, degrees, foot):
    """Leans an upright instrument back by `degrees` and stands its foot at game `foot`."""
    G.turn(obj, -degrees, 'X')
    obj.location = F.at(*foot)
    return obj


def _guitar_stand(F, base_z, cradle_y, top_y, top_z, reach=0.07, back=0.2, front=0.06):
    """A black A-frame stand: a post leaning back from its foot at depth `base_z` to a padded yoke at
    (`top_y`, `top_z`), two legs splayed back, a front foot, and a cradle across at `cradle_y` with padded
    arms reaching `reach` forward; the legs reach `back` behind its foot, the front foot `front` ahead."""
    G.rod('post', F.at(0, 0.02, base_z), F.at(0, top_y, top_z), 0.013, material=M.stand, verts=10)
    t = (cradle_y - 0.02) / (top_y - 0.02)
    cz = base_z + t * (top_z - base_z)
    G.rod('foot_front', F.at(0, 0.014, base_z - 0.01), F.at(0, 0.014, base_z + front), 0.011, material=M.stand, verts=8)
    G.rod('cradle', F.at(-0.1, cradle_y, cz), F.at(0.1, cradle_y, cz), 0.011, material=M.stand, verts=8)
    for s in (-1, 1):
        G.rod(f'leg{s}', F.at(0, 0.03, base_z), F.at(s * 0.16, 0.014, base_z - back), 0.011, material=M.stand, verts=8)
        F.sphere(f'foot{s}', 0.014, (s * 0.16, 0.014, base_z - back), M.rubber, segments=8, rings=6)
        G.rod(f'arm{s}', F.at(s * 0.1, cradle_y, cz), F.at(s * 0.1, cradle_y + 0.03, cz + reach), 0.011,
              material=M.rubber, verts=8)
        G.rod(f'yoke{s}', F.at(s * 0.03, top_y, top_z), F.at(s * 0.03, top_y + 0.045, top_z + 0.01), 0.008,
              material=M.rubber, verts=8)
    F.sphere('foot_front_pad', 0.014, (0, 0.014, base_z + front), M.rubber, segments=8, rings=6)
    G.rod('yoke', F.at(-0.03, top_y, top_z), F.at(0.03, top_y, top_z), 0.009, material=M.rubber, verts=8)


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
# The piano: an upright facing its bench at the back of the footprint, the pianist facing the front. Its
# fallboard lies over the keys and lifts back against the case; its music desk, hinged at the top, swings
# its foot out so the sheet music leans back for reading.
BLACK_KEYS = [-0.577 + 0.161 * o + d for o in range(8) for d in (0, 0.023, 0.069, 0.092, 0.115)]


def piano():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('case', 1.5, 0.53, 0.15, (0, 0.995, 0.375), M.piano, bevel=0.01)
        F.box('lid', 1.54, 0.03, 0.17, (0, 1.275, 0.37), M.piano_trim, bevel=0.008)
        F.box('lid_moulding', 1.5, 0.012, 0.012, (0, 1.254, 0.296), M.piano_trim, bevel=0.004)
        F.box('keybed', 1.46, 0.08, 0.32, (0, 0.69, 0.29), M.piano, bevel=0.01)
        F.box('key_slip', 1.36, 0.04, 0.012, (0, 0.72, 0.124), M.piano_trim, bevel=0.004)
        F.box('keys', 1.3, 0.025, 0.15, (0, 0.735, 0.205), M.ivory)
        for k in range(1, 57):
            F.box(f'gap{k}', 0.0015, 0.003, 0.148, (-0.65 + 0.023 * k, 0.747, 0.205), M.key_gap)
        for k, x in enumerate(x for x in BLACK_KEYS if x <= 0.56):
            F.box(f'black{k}', 0.012, 0.018, 0.09, (x, 0.756, 0.23), M.ebony)
        for s in (-1, 1):
            F.box(f'side{s}', 0.06, 0.78, 0.18, (s * 0.74, 0.39, 0.36), M.piano, bevel=0.01)
            F.box(f'cheek{s}', 0.06, 0.14, 0.2, (s * 0.74, 0.8, 0.22), M.piano, bevel=0.02)
            F.cone(f'leg{s}', 0.018, 0.026, 0.62, (s * 0.7, 0.31, 0.16), M.piano, verts=12)
            F.box(f'toe{s}', 0.07, 0.03, 0.12, (s * 0.7, 0.015, 0.2), M.piano, bevel=0.008)
            F.cyl(f'sconce{s}', 0.03, 0.06, (s * 0.6, 1.32, 0.38), M.brass, verts=14, bevel=0.006)
            F.cyl(f'sconce_cup{s}', 0.024, 0.012, (s * 0.6, 1.356, 0.38), M.brass, verts=14)
            F.cyl(f'candle{s}', 0.0175, 0.12, (s * 0.6, 1.42, 0.38), M.candle, verts=12)
            F.box(f'panel{s}', 0.62, 0.36, 0.006, (s * 0.36, 1.0, 0.451), M.piano_trim, bevel=0.004)
            F.box(f'panel_inset{s}', 0.54, 0.28, 0.006, (s * 0.36, 1.0, 0.454), M.piano, bevel=0.004)
        # the lower case: kneeboard, back, plinth, the pedal lyre and three pedals
        F.box('kneeboard', 1.42, 0.62, 0.025, (0, 0.34, 0.36), M.piano, bevel=0.006)
        F.box('back_lower', 1.42, 0.65, 0.02, (0, 0.325, 0.44), M.piano)
        F.box('plinth', 1.42, 0.03, 0.07, (0, 0.015, 0.39), M.piano)
        F.box('lyre', 0.22, 0.16, 0.025, (0, 0.11, 0.335), M.piano_trim, bevel=0.008)
        for x in (-0.07, 0, 0.07):
            F.box(f'pedal{x}', 0.03, 0.012, 0.09, (x, 0.04, 0.3), M.brass, bevel=0.004)
        # the bench
        F.box('bench', 0.86, 0.06, 0.34, (0, 0.42, -0.2), M.piano, bevel=0.012)
        F.box('bench_top', 0.82, 0.05, 0.3, (0, 0.465, -0.2), M.cushion, bevel=0.02)
        for sx in (-1, 1):
            for z in (-0.33, -0.07):
                F.box(f'bench_leg{sx}{z}', 0.045, 0.39, 0.045, (sx * 0.38, 0.195, z), M.piano, bevel=0.008)
            F.box(f'bench_rail{sx}', 0.02, 0.04, 0.22, (sx * 0.38, 0.12, -0.2), M.piano)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as fb:
        F.box('fallboard', 1.32, 0.02, 0.17, (0, 0.795, 0.215), M.piano, bevel=0.005)
        F.box('fallboard_lip', 1.32, 0.012, 0.012, (0, 0.787, 0.134), M.piano_trim, bevel=0.004)
    fallboard = G.hinged(G.join(fb, 'fallboard'), F.at(0, 0.805, 0.3))
    with G.new_parts() as dk:
        F.box('desk', 0.62, 0.22, 0.012, (0, 1.095, 0.292), M.piano_trim, bevel=0.004)
        F.box('desk_ledge', 0.62, 0.015, 0.025, (0, 0.992, 0.278), M.piano_trim, bevel=0.004)
        _sheet_music(F, 'music', 0.4, 0.19, (0, 1.105, 0.285))
    desk = G.hinged(G.join(dk, 'music_desk'), F.at(0, 1.205, 0.298))
    G.settle(centre=False)
    G.swing(fallboard, 'fallboard', 90, seconds=0.9, axis='X')
    G.swing(desk, 'fallboard', 18, seconds=0.9, axis='X')
    return G.meshes()


# The ukulele and the guitars lean on a black A-frame stand. Each instrument is built upright, its foot at
# the origin and its face to the front, then leant back onto the stand: a part of its own.
def _ukulele(F):
    with G.new_parts() as p:
        _slab(F, 'lower_bout', 0.115, 0.1, 0.065, (0, 0.1, 0), M.uke)
        _slab(F, 'upper_bout', 0.09, 0.08, 0.065, (0, 0.25, 0), M.uke)
        _slab(F, 'top_lower', 0.109, 0.094, 0.004, (0, 0.1, 0.033), M.uke_top, smooth=False)
        _slab(F, 'top_upper', 0.084, 0.074, 0.004, (0, 0.25, 0.033), M.uke_top, smooth=False)
        F.cyl('hole', 0.028, 0.004, (0, 0.205, 0.0345), M.hole, rot=(90, 0, 0), verts=18, smooth=False)
        G.torus('rosette', 0.032, 0.0025, at=F.at(0, 0.205, 0.035), material=M.fretboard, rot=(90, 0, 0),
                major_segments=24, minor_segments=4)
        F.box('bridge', 0.07, 0.012, 0.008, (0, 0.07, 0.039), M.fretboard)
        F.box('saddle', 0.04, 0.003, 0.004, (0, 0.072, 0.044), M.bone)
        F.box('neck', 0.04, 0.25, 0.022, (0, 0.445, 0.004), M.uke)
        F.box('fretboard', 0.038, 0.23, 0.005, (0, 0.445, 0.0175), M.fretboard)
        _frets(F, 12, 0.35, 0.56, 0.038, 0.0205)
        F.box('nut', 0.038, 0.004, 0.004, (0, 0.561, 0.021), M.bone)
        F.box('head', 0.055, 0.085, 0.016, (0, 0.605, 0.002), M.fretboard, bevel=0.006)
        for s in (-1, 1):
            for y in (0.588, 0.622):
                F.cyl(f'peg{s}{y}', 0.004, 0.022, (s * 0.036, y, 0.0), M.fret, rot=(0, 90, 0), verts=8)
                F.box(f'peg_button{s}{y}', 0.004, 0.016, 0.01, (s * 0.049, y, 0.0), M.bone, bevel=0.002)
        _strings(F, 4, 0.0075, 0.0085, 0.072, 0.561, 0.0465, 0.0235)
    return _upright(p, 'ukulele')


def ukulele():
    F = G.GameFrame()
    with G.new_parts() as still:
        _guitar_stand(F, 0.12, 0.2, 0.62, -0.005, back=0.17)
    G.join(still, 'stand', origin=(0, 0, 0))
    _lean(F, _ukulele(G.GameFrame()), 15, (0, 0.215, 0.135))
    G.settle(centre=False)
    return G.meshes()


def _electric(F):
    with G.new_parts() as p:
        _slab(F, 'lower', 0.18, 0.14, 0.045, (0, 0.14, 0), M.electric, verts=32)
        _slab(F, 'waist', 0.15, 0.12, 0.045, (0.01, 0.29, 0), M.electric, verts=28)
        _slab(F, 'horn_bass', 0.045, 0.075, 0.045, (-0.09, 0.39, 0), M.electric, verts=16)
        _slab(F, 'horn_treble', 0.04, 0.055, 0.045, (0.095, 0.37, 0), M.electric, verts=16)
        _slab(F, 'guard', 0.1, 0.13, 0.003, (0.025, 0.2, 0.0235), M.pickguard, verts=24, smooth=False)
        F.box('guard_neck', 0.07, 0.12, 0.003, (-0.005, 0.33, 0.0235), M.pickguard)
        for k, y in enumerate((0.13, 0.2, 0.27)):
            F.box(f'pickup{k}', 0.07, 0.018, 0.006, (0, y, 0.027), M.black, bevel=0.003)
        F.box('bridge', 0.075, 0.025, 0.008, (0, 0.085, 0.027), M.chrome, bevel=0.002)
        for k, (x, y) in enumerate(((0.09, 0.12), (0.1, 0.075), (0.085, 0.035))):
            F.cyl(f'knob{k}', 0.011, 0.014, (x, y, 0.028), M.black, rot=(90, 0, 0), verts=10)
        F.box('switch', 0.006, 0.03, 0.006, (0.11, 0.19, 0.027), M.pickguard)
        F.box('neck', 0.05, 0.62, 0.025, (0, 0.73, 0.0), M.maple)
        F.box('fretboard', 0.048, 0.6, 0.005, (0, 0.73, 0.015), M.fretboard)
        _frets(F, 21, 0.65, 1.035, 0.048, 0.0185)
        F.box('nut', 0.048, 0.004, 0.004, (0, 1.036, 0.0185), M.bone)
        F.box('head', 0.075, 0.16, 0.016, (0.008, 1.11, -0.002), M.black, bevel=0.012)
        for k in range(6):
            F.cyl(f'tuner{k}', 0.006, 0.018, (-0.026, 1.05 + 0.022 * k, 0.012), M.chrome, verts=8, rot=(90, 0, 0))
        _strings(F, 6, 0.009, 0.0075, 0.087, 1.036, 0.033, 0.0215)
    return _upright(p, 'guitar')


def guitarElectric():
    F = G.GameFrame()
    with G.new_parts() as still:
        _guitar_stand(F, 0.12, 0.2, 0.82, -0.05, front=0.03)
        # the practice amp behind
        F.box('amp', 0.46, 0.4, 0.28, (0, 0.2, -0.28), M.amp, bevel=0.02)
        F.box('amp_grille', 0.4, 0.26, 0.004, (0, 0.17, -0.139), M.grille, bevel=0.006)
        F.box('amp_panel', 0.42, 0.06, 0.004, (0, 0.355, -0.139), M.amp_trim, bevel=0.004)
        for k in range(5):
            F.cyl(f'amp_knob{k}', 0.011, 0.012, (-0.14 + 0.06 * k, 0.355, -0.133), M.black, rot=(90, 0, 0), verts=10)
        F.cyl('amp_jack', 0.008, 0.01, (0.18, 0.355, -0.134), M.chrome, rot=(90, 0, 0), verts=8)
        F.box('amp_handle', 0.16, 0.02, 0.03, (0, 0.41, -0.28), M.black, bevel=0.008)
        F.box('amp_logo', 0.08, 0.018, 0.004, (0.12, 0.27, -0.136), M.amp_trim)
        for sx in (-1, 1):
            for y in (0.01, 0.39):
                F.box(f'amp_corner{sx}{y}', 0.03, 0.03, 0.03, (sx * 0.215, y + 0.005, -0.145), M.chrome, bevel=0.006)
    G.join(still, 'stand', origin=(0, 0, 0))
    _lean(F, _electric(G.GameFrame()), 12, (0, 0.21, 0.106))
    G.settle(centre=False)
    return G.meshes()


def _acoustic(F):
    with G.new_parts() as p:
        _slab(F, 'lower_bout', 0.18, 0.21, 0.1, (0, 0.21, 0), M.acoustic, verts=32)
        _slab(F, 'upper_bout', 0.14, 0.16, 0.1, (0, 0.51, 0), M.acoustic, verts=28)
        _slab(F, 'top_lower', 0.173, 0.203, 0.004, (0, 0.21, 0.05), M.spruce, verts=32, smooth=False)
        _slab(F, 'top_upper', 0.133, 0.153, 0.004, (0, 0.51, 0.05), M.spruce, verts=28, smooth=False)
        F.cyl('hole', 0.045, 0.004, (0, 0.41, 0.0515), M.hole, rot=(90, 0, 0), verts=24, smooth=False)
        G.torus('rosette', 0.052, 0.003, at=F.at(0, 0.41, 0.052), material=M.tortoise, rot=(90, 0, 0),
                major_segments=28, minor_segments=4)
        _slab(F, 'pickguard', 0.045, 0.06, 0.002, (0.06, 0.34, 0.0525), M.tortoise, verts=16, smooth=False)
        F.box('bridge', 0.11, 0.025, 0.008, (0, 0.13, 0.055), M.fretboard, bevel=0.003)
        F.box('saddle', 0.07, 0.003, 0.004, (0, 0.135, 0.06), M.bone)
        F.box('neck', 0.05, 0.5, 0.025, (0, 0.87, 0.0), M.neck)
        F.box('heel', 0.05, 0.05, 0.05, (0, 0.645, -0.01), M.neck, bevel=0.01)
        F.box('fretboard', 0.048, 0.48, 0.005, (0, 0.865, 0.015), M.fretboard)
        _frets(F, 19, 0.64, 1.105, 0.048, 0.0185)
        F.box('nut', 0.048, 0.004, 0.004, (0, 1.106, 0.0185), M.bone)
        F.box('head', 0.08, 0.18, 0.016, (0, 1.2, -0.003), M.neck, bevel=0.008)
        for s in (-1, 1):
            for k in range(3):
                y = 1.15 + 0.045 * k
                F.cyl(f'peg{s}{k}', 0.004, 0.024, (s * 0.05, y, -0.003), M.fret, rot=(0, 90, 0), verts=8)
                F.box(f'peg_button{s}{k}', 0.005, 0.018, 0.012, (s * 0.064, y, -0.003), M.fret, bevel=0.002)
        _strings(F, 6, 0.0095, 0.0075, 0.137, 1.106, 0.0625, 0.0215)
    return _upright(p, 'guitar')


def guitarAcoustic():
    F = G.GameFrame(shift=0.07)   # a little forward, where its placeholder stands
    with G.new_parts() as still:
        _guitar_stand(F, -0.01, 0.19, 1.0, -0.075, reach=0.1, back=0.27)
    G.join(still, 'stand', origin=(0, 0, 0))
    _lean(F, _acoustic(G.GameFrame()), 8, (0, 0.205, 0.06))
    G.settle(centre=False)
    return G.meshes()


# The digital piano: a slim keyboard on a black X-stand with a padded X-legged bench behind it. Still.
KB_BLACK = [-0.38 + 0.273 * o + d for o in range(4) for d in (0, 0.039, 0.117, 0.156, 0.195)]


def keyboardDigital():
    F = G.GameFrame()
    with G.new_parts() as parts:
        F.box('case', 0.9, 0.1, 0.3, (0, 0.78, 0.22), M.kb_body, bevel=0.015)
        F.box('keys', 0.8, 0.02, 0.15, (0, 0.835, 0.17), M.ivory)
        for k in range(1, 21):
            F.box(f'gap{k}', 0.0015, 0.003, 0.148, (-0.4 + 0.039 * k, 0.8455, 0.17), M.key_gap)
        for k, x in enumerate(x for x in KB_BLACK if x <= 0.37):
            F.box(f'black{k}', 0.02, 0.016, 0.09, (x + 0.0195, 0.852, 0.2), M.ebony)
        F.box('strip', 0.9, 0.03, 0.08, (0, 0.845, 0.32), M.kb_trim, bevel=0.008)
        F.box('lcd', 0.12, 0.004, 0.035, (0, 0.861, 0.31), M.lcd)
        for k in range(6):
            F.box(f'button{k}', 0.022, 0.006, 0.014, (0.1 + 0.03 * k, 0.861, 0.31), M.kb_body)
        for s in (-1, 1):
            F.cyl(f'speaker{s}', 0.035, 0.004, (s * 0.33, 0.861, 0.31), M.kb_body, verts=16, smooth=False)
            F.cyl(f'knob{s}', 0.012, 0.012, (s * 0.2, 0.864, 0.31), M.kb_body, verts=12)
        # the music rest, leant back, and a sheet on it
        F.box('rest', 0.42, 0.22, 0.012, (0, 0.98, 0.335), M.kb_body, rot=(12, 0, 0), bevel=0.004)
        F.box('rest_ledge', 0.42, 0.012, 0.03, (0, 0.872, 0.318), M.kb_body)
        _sheet_music(F, 'music', 0.32, 0.18, (0, 0.99, 0.327), rot=(12, 0, 0))
        # the X-stand: a pair of crossed tubes at each end, a bar across
        for x in (-0.34, 0.34):
            for k, yaw in enumerate((0.0, 180.0)):
                _rod(F, f'x{x}{k}', (x, 0.37, 0.22), 0.754, (21.8, yaw, 0), 0.015, M.stand)
            for z in (0.08, 0.36):
                F.box(f'cap{x}{z}', 0.04, 0.02, 0.06, (x, 0.01, z), M.rubber, bevel=0.006)
        G.rod('x_bar', F.at(-0.34, 0.37, 0.22), F.at(0.34, 0.37, 0.22), 0.012, material=M.stand, verts=8)
        # the bench
        F.box('seat', 0.62, 0.06, 0.32, (0, 0.42, -0.2), M.kb_seat, bevel=0.03)
        F.box('seat_frame', 0.58, 0.02, 0.26, (0, 0.385, -0.2), M.stand)
        for x in (-0.26, 0.26):
            for z, yaw in ((-0.26, 0.0), (-0.14, 180.0)):
                _rod(F, f'bench_leg{x}{z}', (x, 0.195, z), 0.408, (17.1, yaw, 0), 0.014, M.stand)
            for z in (-0.32, -0.08):
                F.box(f'bench_cap{x}{z}', 0.035, 0.016, 0.05, (x, 0.008, z), M.rubber, bevel=0.005)
    G.join_all('keyboardDigital')
    G.settle(centre=False)
    return G.meshes()


# The drum kit: a kick drum to the front with two rack toms on it, a snare, a floor tom and a hi-hat, the
# crash and ride cymbals on stands and a throne behind. The two cymbals are parts of their own, their
# origins on their stands' tops, and wobble while a resident drums (`cymbals`, a loop).
def _cymbal(F, name, r, at, rot):
    profile = [(0, 0.016), (0.02, 0.015), (0.028, 0.008), (r, 0.0), (r, -0.002), (0.028, 0.005), (0.02, 0.012),
               (0, 0.013)]
    return G.revolve(name, profile, at=F.at(*at), material=M.cymbal, segments=24, rot=rot)


def _drum(F, name, r, h, at, rot=(0, 0, 0), heads=(1,), lugs=6):
    """A drum along its own axis: the shell, its heads (1 top, -1 bottom) and hoops, and its lugs."""
    rx = math.radians(rot[0])
    up = (0, math.cos(rx), -math.sin(rx))  # game direction of the drum's axis (rot about Blender X)
    F.cyl(name, r, h, at, M.shell, rot=rot, verts=28)
    for s in heads:
        c = tuple(at[i] + s * up[i] * (h / 2 + 0.002) for i in range(3))
        F.cyl(f'{name}_head{s}', r - 0.006, 0.004, c, M.head, rot=rot, verts=28, smooth=False)
        G.torus(f'{name}_hoop{s}', r, 0.008, at=F.at(*c), material=M.chrome, rot=rot, major_segments=24, minor_segments=4)
    for k in range(lugs):
        a = 360 * k / lugs
        x, z = at[0] + r * G.cosd(a), at[2] + r * G.sind(a)
        F.box(f'{name}_lug{k}', 0.018, h * 0.4, 0.018, (x, at[1], z), M.chrome, rot=rot)


def _drum_stand(F, name, x, z, top, spread=0.18):
    """A chrome stand: a pole up to `top` on three splayed legs."""
    G.rod(f'{name}_pole', F.at(x, 0.2, z), F.at(x, top, z), 0.01, material=M.chrome, verts=8)
    for k in range(3):
        a = 90 + 120 * k
        G.rod(f'{name}_leg{k}', F.at(x, 0.24, z), F.at(x + spread * G.cosd(a), 0.006, z + spread * G.sind(a)), 0.007,
              material=M.chrome, verts=6)


def drumKit():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the kick drum lies on its side, its front head to the front
        F.cyl('kick', 0.25, 0.34, (0, 0.27, 0.2), M.shell, rot=(90, 0, 0), verts=32)
        for k, z in enumerate((0.028, 0.372)):
            F.cyl(f'kick_head{k}', 0.24, 0.006, (0, 0.27, z), M.head, rot=(90, 0, 0), verts=32, smooth=False)
            G.torus(f'kick_hoop{k}', 0.245, 0.013, at=F.at(0, 0.27, z), material=M.shell, rot=(90, 0, 0),
                    major_segments=28, minor_segments=4)
        F.cyl('kick_logo', 0.09, 0.002, (0, 0.27, 0.3765), M.shell, rot=(90, 0, 0), verts=24, smooth=False)
        for k in range(8):
            a = 22.5 + 45 * k
            F.box(f'kick_lug{k}', 0.018, 0.018, 0.08, (0.252 * G.cosd(a), 0.27 + 0.252 * G.sind(a), 0.2), M.chrome)
        for s in (-1, 1):
            G.rod(f'spur{s}', F.at(s * 0.2, 0.16, 0.3), F.at(s * 0.3, 0.006, 0.34), 0.008, material=M.chrome, verts=6)
        # the pedal and its beater
        F.box('pedal', 0.08, 0.015, 0.2, (0, 0.03, -0.09), M.chrome, rot=(-6, 0, 0), bevel=0.004)
        G.rod('beater', F.at(0, 0.06, -0.02), F.at(0, 0.24, 0.005), 0.005, material=M.chrome, verts=6)
        F.sphere('beater_head', 0.022, (0, 0.24, 0.01), M.head, segments=10, rings=6)
        # two rack toms on a mount, tilted to the drummer
        G.rod('tom_mount', F.at(0, 0.5, 0.2), F.at(0, 0.6, 0.17), 0.012, material=M.chrome, verts=8)
        for k, (x, r, h) in enumerate(((-0.14, 0.13, 0.2), (0.16, 0.12, 0.18))):
            _drum(F, f'tom{k}', r, h, (x, 0.64, 0.15), rot=(-20, 0, 0), heads=(1,))
            G.rod(f'tom_arm{k}', F.at(0, 0.6, 0.17), F.at(x * 0.6, 0.6, 0.15), 0.009, material=M.chrome, verts=6)
        # the snare on its stand
        _drum(F, 'snare', 0.17, 0.12, (-0.28, 0.6, -0.08), heads=(1, -1), lugs=8)
        _drum_stand(F, 'snare_stand', -0.28, -0.08, 0.53)
        # the floor tom on three legs
        _drum(F, 'floor_tom', 0.18, 0.26, (0.3, 0.32, -0.05), heads=(1,), lugs=6)
        for k in range(3):
            a = 30 + 120 * k
            G.rod(f'floor_leg{k}', F.at(0.3 + 0.19 * G.cosd(a), 0.38, -0.05 + 0.19 * G.sind(a)),
                  F.at(0.3 + 0.22 * G.cosd(a), 0.006, -0.05 + 0.22 * G.sind(a)), 0.007, material=M.chrome, verts=6)
        # the hi-hat: two cymbals shut on their pole, and its pedal
        _drum_stand(F, 'hihat', -0.32, -0.16, 0.9)
        _cymbal(F, 'hihat_bottom', 0.14, (-0.32, 0.85, -0.16), (0, 0, 0))
        _cymbal(F, 'hihat_top', 0.14, (-0.32, 0.868, -0.16), (0, 0, 0))
        F.box('hihat_pedal', 0.07, 0.012, 0.18, (-0.32, 0.03, -0.24), M.chrome, rot=(-6, 0, 0), bevel=0.004)
        # the crash's and the ride's stands
        _drum_stand(F, 'crash_stand', -0.3, 0.18, 1.0)
        _drum_stand(F, 'ride_stand', 0.3, 0.12, 1.05)
        # the throne
        F.cyl('throne', 0.2, 0.07, (0, 0.495, -0.3), M.seat, verts=20, bevel=0.02)
        F.cyl('throne_base', 0.05, 0.03, (0, 0.445, -0.3), M.chrome, verts=12)
        _drum_stand(F, 'throne_stand', 0, -0.3, 0.45, spread=0.2)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as c:
        _cymbal(F, 'crash', 0.17, (-0.3, 1.0, 0.18), (-10, 8, 0))
        F.cyl('crash_felt', 0.012, 0.02, (-0.3, 1.02, 0.18), M.seat, verts=8)
    crash = G.hinged(G.join(c, 'crash'), F.at(-0.3, 1.0, 0.18))
    with G.new_parts() as r:
        _cymbal(F, 'ride', 0.18, (0.3, 1.05, 0.12), (-10, -8, 0))
        F.cyl('ride_felt', 0.012, 0.02, (0.3, 1.07, 0.12), M.seat, verts=8)
    ride = G.hinged(G.join(r, 'ride'), F.at(0.3, 1.05, 0.12))
    G.settle()  # on its Model's `centre`: its back moved onto the wall face
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


def _piece(F, name, kind, col, row, material):
    """A turned chess piece on a square: pawn, rook, knight, bishop, queen or king."""
    x, z = _square_at(col, row)
    base = [(0, 0), (0.013, 0), (0.013, 0.004), (0.009, 0.008)]
    profiles = {
        'pawn': base + [(0.005, 0.022), (0.008, 0.024), (0.004, 0.026), (0.007, 0.03), (0.008, 0.034),
                        (0.006, 0.039), (0, 0.041)],
        'rook': base + [(0.008, 0.038), (0.011, 0.04), (0.011, 0.05), (0.008, 0.05), (0.008, 0.046), (0, 0.046)],
        'bishop': base + [(0.006, 0.034), (0.01, 0.036), (0.005, 0.039), (0.008, 0.046), (0.006, 0.054),
                          (0.002, 0.057), (0.003, 0.06), (0, 0.062)],
        'queen': base + [(0.006, 0.042), (0.011, 0.044), (0.006, 0.047), (0.009, 0.058), (0.011, 0.064),
                         (0.006, 0.066), (0.004, 0.07), (0, 0.071)],
        'king': base + [(0.006, 0.046), (0.011, 0.048), (0.006, 0.051), (0.009, 0.062), (0.009, 0.066),
                        (0, 0.067)],
        'knight': base + [(0.008, 0.014), (0, 0.014)],
    }
    G.revolve(name, profiles[kind], at=F.at(x, BOARD_Y, z), material=material, segments=8)
    if kind == 'king':
        F.box(f'{name}_cross', 0.005, 0.02, 0.005, (x, BOARD_Y + 0.075, z), material)
        F.box(f'{name}_arm', 0.014, 0.005, 0.005, (x, BOARD_Y + 0.077, z), material)
    if kind == 'knight':
        facing = -1 if material == M.piece_dark else 1
        F.box(f'{name}_neck', 0.014, 0.032, 0.016, (x, BOARD_Y + 0.03, z), material, rot=(facing * 12, 0, 0))
        F.box(f'{name}_head', 0.012, 0.012, 0.024, (x, BOARD_Y + 0.045, z + facing * 0.008), material)


def chessTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.46, 0.03, 0.4, (0, 0.725, 0.25), M.table_wood, bevel=0.006, bevel_segments=1)
        F.box('board', 0.37, 0.004, 0.37, (0, 0.742, 0.2505), M.chair_wood, bevel=0.002)
        F.box('board_dark', 0.344, 0.0005, 0.344, (0, 0.74425, 0.2505), M.square_dark)
        for col in range(8):
            for row in range(8):
                if (col + row) % 2 == 0:
                    x, z = _square_at(col, row)
                    F.box(f'square{col}{row}', SQUARE, 0.0006, SQUARE, (x, 0.7444, z), M.square_light)
        F.cone('pedestal', 0.035, 0.026, 0.62, (0, 0.4, 0.25), M.table_wood, verts=12)
        F.cone('foot', 0.16, 0.1, 0.05, (0, 0.025, 0.25), M.table_wood, verts=16)
        F.cone('collar', 0.05, 0.07, 0.06, (0, 0.68, 0.25), M.table_wood, verts=12)
        F.box('apron', 0.4, 0.04, 0.34, (0, 0.69, 0.25), M.table_wood)
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
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        _piece(F, 'pawn', 'pawn', 5, 1, M.piece_dark)
    pawn = G.join(p, 'pawn')
    G.settle(centre=False)
    G.slide(pawn, 'move', (0, -SQUARE, 0), seconds=0.8)
    return G.meshes()


# The pool table: a lacquered frame on four legs, a baize bed with cushions, six pockets and sights, the
# fifteen balls racked at the foot and the cue ball at the head, and a cue lying on the cloth. The triangle
# round the balls lifts off for pool and trick shots (`rack_off`).
BALL = 0.0285
BED = 0.785        # the cloth's top


def poolTable():
    F = G.GameFrame()
    colours = [M.ball_yellow, M.ball_blue, M.ball_red, M.ball_purple, M.ball_black, M.ball_orange, M.ball_green,
               M.ball_maroon, M.ball_yellow, M.ball_blue, M.ball_red, M.ball_purple, M.ball_orange, M.ball_green,
               M.ball_maroon]
    apex, row = 0.5, math.sqrt(3) * BALL
    with G.new_parts() as still:
        F.box('bed', 2.3, 0.02, 1.2, (0, BED - 0.01, 0), M.cloth)
        F.box('slate', 2.3, 0.08, 1.2, (0, 0.725, 0), M.pool_frame)
        F.box('apron', 2.4, 0.18, 1.3, (0, 0.6, 0), M.pool_frame, bevel=0.02)
        for s in (-1, 1):
            F.box(f'rail_long{s}', 2.5, 0.15, 0.1, (0, 0.745, s * 0.65), M.pool_frame, bevel=0.012)
            F.box(f'rail_short{s}', 0.1, 0.15, 1.2, (s * 1.2, 0.745, 0), M.pool_frame, bevel=0.012)
            F.box(f'cushion_short{s}', 0.03, 0.03, 1.06, (s * 1.135, BED + 0.015, 0), M.cloth)
            for half in (-1, 1):
                F.box(f'cushion_long{s}{half}', 1.02, 0.03, 0.03, (half * 0.575, BED + 0.015, s * 0.585), M.cloth)
            for x in (-1.15, 0, 1.15):
                F.cyl(f'pocket{s}{x}', 0.058, 0.004, (x, 0.822, s * (0.6 if x else 0.625)), M.pocket, verts=16)
            for k in range(1, 4):
                for half in (-1, 1):
                    F.box(f'sight{s}{half}{k}', 0.012, 0.002, 0.02, (half * 0.2875 * k, 0.821, s * 0.65), M.pearl)
            for z in (-0.3, 0, 0.3):
                F.box(f'sight_end{s}{z}', 0.02, 0.002, 0.012, (s * 1.2, 0.821, z), M.pearl)
            for sz in (-1, 1):
                F.box(f'leg{s}{sz}', 0.15, 0.52, 0.15, (s * 1.08, 0.27, sz * 0.5), M.pool_frame, bevel=0.02)
                F.box(f'foot{s}{sz}', 0.19, 0.02, 0.19, (s * 1.08, 0.01, sz * 0.5), M.pool_frame, bevel=0.006)
        F.box('chalk', 0.022, 0.022, 0.022, (0.9, 0.831, 0.65), M.ball_blue)
        # the racked balls, the apex to the head, and the cue ball
        k = 0
        for i in range(5):
            for j in range(i + 1):
                G.sphere(f'ball{k}', BALL, at=F.at(apex + i * row, BED + BALL, (j - i / 2) * 2 * BALL),
                         material=colours[k], segments=12, rings=8)
                k += 1
        G.sphere('cue_ball', BALL, at=F.at(-0.6, BED + BALL, 0), material=M.ball_white, segments=12, rings=8)
        # the cue on the cloth
        G.rod('cue_butt', F.at(-1.02, BED + 0.014, 0.42), F.at(-0.5, BED + 0.012, 0.38), 0.014, material=M.cue_butt)
        G.rod('cue_shaft', F.at(-0.5, BED + 0.012, 0.38), F.at(0.36, BED + 0.008, 0.3), 0.009, material=M.cue)
        F.sphere('cue_tip', 0.008, (0.36, BED + 0.008, 0.3), M.ball_blue, segments=8, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the triangle: three bars round the rack
    centre = apex + 4 * row * 2 / 3
    corners = [(apex, 0), (apex + 4 * row, -4 * BALL), (apex + 4 * row, 4 * BALL)]
    t = 0.016
    inr = 4 * BALL / math.sqrt(3)            # the centres' triangle's inradius
    grow = (inr + BALL + t / 2) / inr
    pts = [(centre + (x - centre) * grow, z * grow) for x, z in corners]
    with G.new_parts() as rp:
        for i in range(3):
            (x0, z0), (x1, z1) = pts[i], pts[(i + 1) % 3]
            length = math.hypot(x1 - x0, z1 - z0) + t
            yaw = math.degrees(math.atan2(-(z1 - z0), -(x1 - x0)))
            F.box(f'rack{i}', length, 0.03, t, ((x0 + x1) / 2, BED + 0.015, (z0 + z1) / 2), M.rack, rot=(0, 0, yaw),
                  bevel=0.004)
    rack = G.join(rp, 'rack')
    G.settle(centre=False)
    G.slide(rack, 'rack_off', (0, 0, 0.25), seconds=0.9)
    return G.meshes()


# The foosball table: a birch cabinet on legs, a pitch with its lines and goals, and four rods of players,
# red and blue, their handles to the front where the players stand. The two outer rods, one each, are parts
# of their own and kick back and forth while played (`rods`, a loop); the inner two stay still.
ROD_Y = 0.84


def _rod_men(F, name, x, team):
    G.rod(f'{name}_bar', F.at(x, ROD_Y, -0.48), F.at(x, ROD_Y, 0.44), 0.009, material=M.chrome, verts=10)
    F.cyl(f'{name}_handle', 0.02, 0.09, (x, ROD_Y, 0.46), M.grip, rot=(90, 0, 0), verts=12, bevel=0.006)
    for z in (-0.37, 0.37):
        F.cyl(f'{name}_bumper{z}', 0.016, 0.016, (x, ROD_Y, z), M.grip, rot=(90, 0, 0), verts=10)
    for z in (-0.18, 0.0, 0.18):
        F.sphere(f'{name}_head{z}', 0.018, (x, 0.888, z), team, segments=10, rings=6)
        F.box(f'{name}_body{z}', 0.034, 0.07, 0.046, (x, 0.835, z), team, bevel=0.008)
        F.box(f'{name}_legs{z}', 0.026, 0.05, 0.03, (x, 0.775, z), team, bevel=0.005)


def foosball():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('pitch', 1.32, 0.02, 0.64, (0, 0.71, 0), M.field)
        F.box('box_floor', 1.4, 0.04, 0.72, (0, 0.68, 0), M.foos, bevel=0.01)
        for s in (-1, 1):
            F.box(f'side{s}', 1.4, 0.24, 0.04, (0, 0.8, s * 0.34), M.foos, bevel=0.01)
            F.box(f'end{s}', 0.04, 0.24, 0.64, (s * 0.68, 0.8, 0), M.foos, bevel=0.01)
            F.box(f'goal{s}', 0.006, 0.07, 0.18, (s * 0.658, 0.755, 0), M.grip)
            F.box(f'box_line{s}', 0.12, 0.0015, 0.004, (s * 0.6, 0.7205, 0.14), M.line)
            F.box(f'box_line_b{s}', 0.12, 0.0015, 0.004, (s * 0.6, 0.7205, -0.14), M.line)
            F.box(f'box_front{s}', 0.004, 0.0015, 0.28, (s * 0.54, 0.7205, 0), M.line)
            for sz in (-1, 1):
                F.box(f'leg{s}{sz}', 0.08, 0.68, 0.08, (s * 0.6, 0.34, sz * 0.3), M.foos_trim, bevel=0.01)
                F.box(f'corner{s}{sz}', 0.06, 0.02, 0.06, (s * 0.68, 0.93, sz * 0.34), M.foos_trim, bevel=0.006)
                F.box(f'leg_foot{s}{sz}', 0.1, 0.02, 0.1, (s * 0.6, 0.01, sz * 0.3), M.foos_trim, bevel=0.005)
            F.box(f'stretcher{s}', 0.04, 0.04, 0.52, (s * 0.6, 0.12, 0), M.foos_trim)
            # a score rail of beads at each end
            G.rod(f'score{s}', F.at(s * 0.62, 0.935, -0.12), F.at(s * 0.62, 0.935, 0.12), 0.004, material=M.chrome, verts=6)
            for k in range(5):
                F.sphere(f'bead{s}{k}', 0.012, (s * 0.62, 0.935, -0.1 + 0.025 * k + (0.06 if k > 1 else 0)),
                         M.ball_white, segments=8, rings=6)
        F.box('half_line', 0.004, 0.0015, 0.62, (0, 0.7205, 0), M.line)
        G.torus('centre_circle', 0.08, 0.003, at=F.at(0, 0.7205, 0), material=M.line, major_segments=28, minor_segments=4)
        F.sphere('ball', 0.018, (0.05, 0.738, 0.05), M.ball_white, segments=10, rings=6)
        _rod_men(F, 'rod_b', -0.15, M.ball_blue)
        _rod_men(F, 'rod_c', 0.15, M.ball_red)
    G.join(still, 'body', origin=(0, 0, 0))
    rods = []
    for name, x, team in (('rod_red', -0.45, M.ball_red), ('rod_blue', 0.45, M.ball_blue)):
        with G.new_parts() as p:
            _rod_men(F, name, x, team)
        rods.append(G.hinged(G.join(p, name), F.at(x, ROD_Y, 0)))
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
        F.box('back', 0.62, 0.62, 0.02, (0, BOARD_C, -0.418), M.cabinet, bevel=0.006)
        for s in (-1, 1):
            F.box(f'frame_side{s}', 0.03, 0.62, 0.05, (s * 0.295, BOARD_C, -0.383), M.cabinet, bevel=0.006)
            F.box(f'frame_edge{s}', 0.56, 0.03, 0.05, (0, BOARD_C + s * 0.295, -0.383), M.cabinet, bevel=0.006)
        F.cyl('board', 0.225, 0.04, (0, BOARD_C, -0.388), M.board, rot=(90, 0, 0), verts=40)
        _dart_face('face')
        for s in range(20):
            a = 90 + 18 * s
            F.box(f'number{s}', 0.012, 0.016, 0.002, (-0.197 * G.cosd(a), BOARD_C + 0.197 * G.sind(a), FACE_Z),
                  M.board_light, rot=(0, 90 - a, 0))
        G.torus('wire', 0.17, 0.0012, at=F.at(0, BOARD_C, FACE_Z), material=M.steel, rot=(90, 0, 0),
                major_segments=40, minor_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as d:
        for k, (x, y, flight) in enumerate(((0.02, BOARD_C + 0.01, M.ball_red), (-0.07, BOARD_C + 0.08, M.ball_blue),
                                            (0.09, BOARD_C - 0.06, M.ball_yellow))):
            G.rod(f'point{k}', F.at(x, y, FACE_Z), F.at(x, y, FACE_Z + 0.02), 0.0015, material=M.steel, verts=6)
            G.rod(f'barrel{k}', F.at(x, y, FACE_Z + 0.02), F.at(x, y, FACE_Z + 0.065), 0.0045, material=M.steel, verts=8)
            G.rod(f'shaft{k}', F.at(x, y, FACE_Z + 0.065), F.at(x, y, FACE_Z + 0.1), 0.0025, material=flight, verts=6)
            F.box(f'flight{k}a', 0.03, 0.002, 0.04, (x, y, FACE_Z + 0.105), flight)
            F.box(f'flight{k}b', 0.002, 0.03, 0.04, (x, y, FACE_Z + 0.105), flight)
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


def arcadeCabinet():
    F = G.GameFrame()
    with G.new_parts() as still:
        for s in (-1, 1):
            _prism(f'side{s}', SIDE, s * 0.29, s * 0.33, M.arcade)
            _prism(f'art{s}', ART, s * 0.33, s * 0.334, M.art)
            _prism(f'stripe{s}', STRIPE, s * 0.334, s * 0.336, M.art_stripe)
        F.box('back', 0.58, 1.8, 0.3, (0, 0.9, -0.32), M.arcade)
        F.box('lower', 0.58, 0.84, 0.44, (0, 0.42, -0.03), M.arcade)
        F.box('kick_plate', 0.58, 0.08, 0.006, (0, 0.04, 0.193), M.bezel)
        F.box('coin_door', 0.3, 0.2, 0.01, (0, 0.55, 0.192), M.bezel, bevel=0.004)
        for x in (-0.06, 0.06):
            F.box(f'coin{x}', 0.03, 0.05, 0.006, (x, 0.58, 0.199), M.ball_red, bevel=0.002)
        F.box('panel', 0.58, 0.07, 0.3, (0, 0.9, 0.25), M.arcade, rot=(PANEL_TILT, 0, 0), bevel=0.01)
        F.box('panel_art', 0.56, 0.004, 0.24, (0, 0.935, 0.26), M.art, rot=(PANEL_TILT, 0, 0))
        for k, (x, z, colour) in enumerate(((0.05, 0.25, M.ball_red), (0.13, 0.29, M.ball_blue),
                                            (0.21, 0.25, M.ball_yellow), (0.29, 0.29, M.ball_green))):
            F.cyl(f'button{k}', 0.024, 0.016, (x - 0.03, _panel_y(z) + 0.006, z), colour, verts=12,
                  rot=(PANEL_TILT, 0, 0))
        F.cyl('stick_washer', 0.025, 0.004, (-0.15, _panel_y(0.28) + 0.002, 0.28), M.bezel, verts=14,
              rot=(PANEL_TILT, 0, 0))
        F.box('screen_shelf', 0.58, 0.06, 0.24, (0, 0.97, 0.1), M.arcade)
        F.box('bezel', 0.58, 0.54, 0.03, (0, 1.25, 0.06), M.bezel, rot=(-15.6, 0, 0), bevel=0.008)
        F.box('screen', 0.48, 0.38, 0.006, (0, 1.252, 0.079), M.screen, rot=(-15.6, 0, 0), bevel=0.004)
        F.box('marquee', 0.58, 0.2, 0.04, (0, 1.655, 0.1), M.marquee, bevel=0.006)
        F.box('marquee_trim', 0.58, 0.04, 0.05, (0, 1.535, 0.095), M.bezel)
        F.box('speaker', 0.4, 0.05, 0.006, (0, 1.535, 0.121), M.arcade)
        F.box('top', 0.66, 0.04, 0.6, (0, 1.8, -0.17), M.arcade, bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    base = (-0.15, _panel_y(0.28), 0.28)
    with G.new_parts() as j:
        G.rod('stick', F.at(*base), F.at(base[0], base[1] + 0.075, base[2]), 0.006, material=M.chrome, verts=8)
        F.sphere('stick_ball', 0.022, (base[0], base[1] + 0.09, base[2]), M.ball_red, segments=12, rings=8)
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


def puzzleTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.86, 0.035, 0.5, (0, 0.735, 0.2), M.table_wood, bevel=0.008)
        for sx in (-1, 1):
            for z in (0.0, 0.4):
                F.box(f'leg{sx}{z}', 0.045, 0.72, 0.045, (sx * 0.38, 0.36, z), M.table_wood, bevel=0.006)
            F.box(f'apron_side{sx}', 0.02, 0.06, 0.36, (sx * 0.38, 0.69, 0.2), M.table_wood)
        for z in (0.0, 0.4):
            F.box(f'apron{z}', 0.72, 0.06, 0.02, (0, 0.69, z), M.table_wood)
        F.box('board', 0.58, 0.006, 0.4, (0, 0.7555, 0.2), M.puzzle_board, bevel=0.002)
        _picture(F, 'done', -0.27, PUZZLE_EDGE)
        F.cyl('sun', 0.035, 0.0015, (-0.17, PUZZLE_Y + 0.0012, 0.08), M.sun, verts=16, smooth=False)
        for k, (x, z, yaw, colour) in enumerate(((-0.36, 0.36, 23, M.blush), (-0.21, 0.43, 42, M.sky),
                                                 (0.36, 0.05, 78, M.hill), (0.33, 0.17, 72, M.hill),
                                                 (0.2, 0.31, 31, M.sky), (0.24, 0.12, 55, M.blush))):
            F.box(f'loose{k}', 0.045, 0.006, 0.045, (x, 0.7555 if abs(x) > 0.29 else 0.7615, z), colour,
                  rot=(0, 0, yaw), bevel=0.002)
        F.box('box', 0.14, 0.04, 0.2, (0.365, 0.7725, 0.3), M.card, bevel=0.004)
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
    with G.new_parts() as still:
        for s in (-1, 1):
            _rod(F, f'leg{s}', (s * 0.175, 0.866, 0.05), 1.74, (11.03, s * 131.35, 0), 0.016, M.easel_wood, verts=8)
        _rod(F, 'leg_back', (0, 0.826, -0.24), 1.66, (11.12, 0, 0), 0.016, M.easel_wood, verts=8)
        F.box('head', 0.12, 0.08, 0.07, (0, 1.69, -0.06), M.easel_dark, bevel=0.012)
        F.box('mast', 0.04, 1.0, 0.025, _on_canvas(0, 0.05, -0.03), M.easel_wood, rot=tilt, bevel=0.005)
        F.box('clamp', 0.1, 0.06, 0.05, (0, 1.581, -0.007), M.easel_metal, rot=tilt, bevel=0.008)
        F.cyl('wing_nut', 0.012, 0.02, (0, 1.581, -0.04), M.easel_metal, rot=(90 + EASEL_TILT, 0, 0), verts=10)
        F.box('canvas', 0.56, 0.72, 0.025, CANVAS, M.canvas, rot=tilt, bevel=0.004)
        F.box('bar', 0.47, 0.03, 0.02, (0, 0.45, 0.102), M.easel_wood, rot=tilt, bevel=0.005)
        F.box('tray', 0.64, 0.025, 0.09, (0, 0.83, 0.105), M.easel_dark, bevel=0.006)
        F.box('tray_lip', 0.64, 0.04, 0.012, (0, 0.85, 0.148), M.easel_dark, bevel=0.004)
        F.cyl('tray_knob', 0.012, 0.02, (0, 0.83, 0.16), M.easel_metal, rot=(90, 0, 0), verts=10)
        # the palette and its paint, on the tray
        _slab(F, 'palette', 0.08, 0.045, 0.008, (0.14, 0.847, 0.1), M.palette, rot=(0, 0, 0), smooth=False)
        F.cyl('thumb_hole', 0.009, 0.002, (0.18, 0.8515, 0.1), M.easel_dark, verts=10, smooth=False)
        for k, (x, z, colour) in enumerate(((0.09, 0.09, M.ball_red), (0.115, 0.115, M.sun), (0.145, 0.085, M.river),
                                            (0.17, 0.115, M.label))):
            F.sphere(f'dab{k}', 0.011, (x, 0.853, z), colour, scale=(1, 1, 0.45), segments=8, rings=5)
        # the jar of brushes on the floor
        F.cone('jar', 0.035, 0.032, 0.07, (0.33, 0.035, 0.3), M.jar, verts=14)
        for k, (at, rotate, colour) in enumerate((((0.3, 0.165, 0.3), (7.59, -90.0, 0), M.palette),
                                                  ((0.36, 0.165, 0.325), (9.85, 50.19, 0), M.ball_red),
                                                  ((0.33, 0.165, 0.27), (7.59, 180.0, 0), M.foot))):
            _rod(F, f'brush{k}', at, 0.272, rotate, 0.004, colour, verts=6)
            d = _axis(*rotate)
            F.sphere(f'bristles{k}', 0.006, tuple(at[i] + d[i] * 0.14 for i in range(3)), M.easel_dark,
                     scale=(1, 1, 1.8), segments=6, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        F.box('sky', 0.52, 0.3, 0.003, _on_canvas(0, 0.1925, 0.014), M.sky, rot=tilt)
        F.box('grass', 0.52, 0.38, 0.003, _on_canvas(0, -0.148, 0.014), M.hill, rot=tilt)
        F.cyl('paint_sun', 0.035, 0.003, _on_canvas(0.16, 0.242, 0.016), M.sun, rot=(90 + EASEL_TILT, 0, 0),
              verts=16, smooth=False)
        F.sphere('tree', 0.17, _on_canvas(-0.11, 0.042, 0.016), M.tree, scale=(1, 0.012, 0.58),
                 rot=tilt, segments=14, rings=8)
        F.sphere('tree2', 0.12, _on_canvas(0.15, 0.032, 0.016), M.tree_light, scale=(1, 0.012, 0.5),
                 rot=tilt, segments=12, rings=8)
        F.box('trunk', 0.02, 0.07, 0.003, _on_canvas(-0.11, -0.03, 0.0155), M.fretboard, rot=tilt)
        F.box('stream', 0.2, 0.05, 0.003, _on_canvas(0.05, -0.198, 0.0155), M.river, rot=tilt)
    painting = G.join(p, 'painting', origin=F.at(*_on_canvas(-0.26, 0, 0.014)))
    painting.scale[0] = 0.001
    G.settle(centre=False)
    G.grow(painting, 'painting', seconds=1.2, axis='X')
    return G.meshes()


# The workbench: a butcher-block top on two steel drawer pedestals, a toolbox, a drill and a tin, and a
# bench vice holding a plank on edge. The vice's front jaw, its screw and handle are a part of their own and
# slide shut on the plank while a resident tinkers (`vice`).
BENCH_Z = -0.0975     # the top's middle: its back on the wall face
VICE_X = 0.1
VICE_Y = 0.7795       # the top's surface
VICE_GAP = 0.04
VICE_Z = 0.02        # the back jaw's middle


def workbench():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 1.62, 0.035, 0.665, (0, 0.762, BENCH_Z), M.bench_top, bevel=0.006)
        F.box('frame', 1.6, 0.025, 0.645, (0, 0.732, BENCH_Z), M.bench_steel, bevel=0.004)
        F.box('modesty', 0.8, 0.4, 0.02, (0, 0.52, -0.41), M.bench_steel, bevel=0.004)
        F.box('drawer', 0.78, 0.09, 0.02, (0, 0.67, 0.21), M.bench_steel, bevel=0.006)
        F.box('drawer_pull', 0.16, 0.014, 0.014, (0, 0.67, 0.227), M.chrome, bevel=0.005)
        for sx in (-1, 1):
            x = sx * 0.6
            F.box(f'pedestal{sx}', 0.4, 0.67, 0.63, (x, 0.39, -0.115), M.bench_steel, bevel=0.01)
            for k, (y, h) in enumerate(((0.185, 0.218), (0.405, 0.178), (0.613, 0.193))):
                F.box(f'front{sx}{k}', 0.37, h, 0.018, (x, y, 0.209), M.bench_steel, bevel=0.006)
                F.box(f'pull{sx}{k}', 0.12, 0.016, 0.016, (x, y + h / 2 - 0.035, 0.226), M.chrome, bevel=0.006)
                F.box(f'label{sx}{k}', 0.05, 0.025, 0.003, (x, y + h / 2 - 0.07, 0.219), M.label, bevel=0.002)
            for dx in (-0.15, 0.15):
                for z in (-0.39, 0.16):
                    F.cyl(f'foot{sx}{dx}{z}', 0.0175, 0.055, (x + dx, 0.0275, z), M.foot, verts=8)
        # the toolbox
        F.box('toolbox', 0.42, 0.16, 0.2, (-0.5, 0.86, -0.3), M.toolbox, bevel=0.02)
        F.box('toolbox_lid', 0.43, 0.035, 0.21, (-0.5, 0.93, -0.3), M.toolbox_lid, bevel=0.012)
        for x in (-0.62, -0.38):
            F.box(f'latch{x}', 0.06, 0.03, 0.03, (x, 0.88, -0.195), M.chrome, bevel=0.006)
        G.rod('toolbox_handle', F.at(-0.6, 0.985, -0.3), F.at(-0.4, 0.985, -0.3), 0.01, material=M.foot, verts=8)
        for x in (-0.6, -0.4):
            G.rod(f'toolbox_post{x}', F.at(x, 0.948, -0.3), F.at(x, 0.985, -0.3), 0.01, material=M.foot, verts=8)
        # the drill lying on its side
        F.cyl('drill_body', 0.03, 0.16, (0.45, 0.91, -0.16), M.drill, rot=(0, 90, 0), verts=14)
        F.sphere('drill_back', 0.03, (0.53, 0.91, -0.16), M.drill, segments=14, rings=8)
        F.box('drill_grip', 0.05, 0.13, 0.05, (0.5, 0.85, -0.16), M.foot, rot=(0, -12, 0), bevel=0.015)
        F.box('drill_battery', 0.09, 0.03, 0.07, (0.51, 0.795, -0.16), M.drill, bevel=0.012)
        F.cyl('drill_chuck', 0.02, 0.05, (0.345, 0.91, -0.16), M.foot, rot=(0, 90, 0), verts=10)
        F.cyl('drill_bit', 0.004, 0.06, (0.29, 0.91, -0.16), M.chrome, rot=(0, 90, 0), verts=6)
        F.cyl('tin', 0.045, 0.06, (0.15, 0.81, -0.32), M.can, verts=14)
        # a hammer and a pencil
        F.box('hammer_handle', 0.2, 0.022, 0.03, (-0.28, 0.791, 0.0), M.plank, rot=(0, 0, 20), bevel=0.008)
        F.box('hammer_head', 0.03, 0.03, 0.11, (-0.19, 0.795, 0.032), M.foot, rot=(0, 0, 20), bevel=0.006)
        F.box('pencil', 0.15, 0.01, 0.01, (-0.12, 0.785, 0.15), M.drill, rot=(0, 0, -20), bevel=0.003)
        # the vice: a base, its back jaw, the plank clamped on edge
        F.box('vice_base', 0.16, 0.02, 0.15, (VICE_X, VICE_Y + 0.01, VICE_Z + 0.02), M.vice, bevel=0.005)
        F.box('vice_jaw', 0.15, 0.07, 0.04, (VICE_X, VICE_Y + 0.055, VICE_Z), M.vice, bevel=0.006)
        F.box('vice_pad', 0.13, 0.03, 0.004, (VICE_X, VICE_Y + 0.07, VICE_Z + 0.022), M.bench_steel)
        F.box('plank', 0.4, 0.08, 0.02, (VICE_X - 0.05, VICE_Y + 0.06, VICE_Z + 0.034), M.plank, bevel=0.002)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as v:
        z = VICE_Z + 0.044 + VICE_GAP
        F.box('vice_front', 0.15, 0.07, 0.04, (VICE_X, VICE_Y + 0.055, z + 0.02), M.vice, bevel=0.006)
        F.box('vice_front_pad', 0.13, 0.03, 0.004, (VICE_X, VICE_Y + 0.07, z + 0.002), M.bench_steel)
        for dx in (-0.05, 0.05):
            G.rod(f'vice_guide{dx}', F.at(VICE_X + dx, VICE_Y + 0.035, VICE_Z - 0.03), F.at(VICE_X + dx, VICE_Y + 0.035, z),
                  0.006, material=M.chrome, verts=8)
        G.rod('vice_screw', F.at(VICE_X, VICE_Y + 0.05, z + 0.04), F.at(VICE_X, VICE_Y + 0.05, z + 0.09), 0.007,
              material=M.chrome, verts=8)
        G.rod('vice_handle', F.at(VICE_X - 0.07, VICE_Y + 0.05, z + 0.08), F.at(VICE_X + 0.07, VICE_Y + 0.05, z + 0.08),
              0.005, material=M.chrome, verts=8)
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
        for k, yaw in enumerate((180.0, 60.0, -60.0)):
            d = (G.sind(yaw), G.cosd(yaw))
            top, foot = (0.045 * d[0], 0.97, 0.045 * d[1]), (0.36 * d[0], 0.02, 0.36 * d[1])
            G.rod(f'leg{k}', F.at(*top), F.at(*foot), 0.015, material=M.tripod, verts=8)
            F.sphere(f'foot{k}', 0.02, foot, M.foot, segments=8, rings=6)
            G.rod(f'spreader{k}', F.at(0, 0.45, 0), F.at(0.17 * d[0], 0.5, 0.17 * d[1]), 0.006, material=M.tripod, verts=6)
        F.cyl('tray', 0.13, 0.015, (0, 0.45, 0), M.tripod, verts=3)
        for x, z in ((0.04, 0.02), (-0.03, -0.03)):
            F.cyl(f'eyepiece{x}', 0.015, 0.05, (x, 0.48, z), M.chrome, verts=10)
        F.cyl('hub', 0.06, 0.05, (0, 0.98, 0), M.tripod, verts=16)
        F.box('head', 0.1, 0.08, 0.1, (0, 1.04, 0), M.tripod, bevel=0.015)
        F.box('fork', 0.03, 0.12, 0.07, (0.065, 1.1, 0), M.tripod, bevel=0.008)
        F.cyl('axle_knob', 0.02, 0.025, (0.09, 1.12, 0), M.foot, rot=(0, 90, 0), verts=12)
        G.rod('slow_motion', F.at(0.03, 1.02, 0.05), F.at(0.06, 0.85, 0.16), 0.004, material=M.chrome, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    tube_rot = (-55.0, 0, 0)    # Blender: the tube's axis, up 35° towards the back
    with G.new_parts() as t:
        F.cyl('tube', 0.06, 0.84, TUBE_C, M.tube, rot=tube_rot, verts=24)
        F.cyl('dew_shield', 0.0675, 0.16, _on_tube(0.42), M.foot, rot=tube_rot, verts=24)
        F.cyl('objective', 0.058, 0.004, _on_tube(0.499), M.optic, rot=tube_rot, verts=24, smooth=False)
        for k, at in enumerate((0.1, -0.15)):
            F.cyl(f'ring{k}', 0.068, 0.03, _on_tube(at), M.chrome, rot=tube_rot, verts=24)
        F.cyl('cell', 0.05, 0.05, _on_tube(-0.445), M.foot, rot=tube_rot, verts=18)
        F.cyl('focuser', 0.02, 0.07, _on_tube(-0.5), M.chrome, rot=tube_rot, verts=12)
        F.cyl('diagonal', 0.0175, 0.07, (0, 0.946, 0.41), M.foot, rot=(35.0, 0, 0), verts=12)
        F.cyl('eyecup', 0.02, 0.02, (0, 0.975, 0.43), M.foot, rot=(35.0, 0, 0), verts=12)
        F.cyl('finder', 0.0175, 0.22, _on_tube(0.05, 0.11), M.tube, rot=tube_rot, verts=12)
        F.box('finder_foot', 0.02, 0.04, 0.02, _on_tube(0.05, 0.08), M.chrome, rot=tube_rot, bevel=0.005)
        F.box('saddle', 0.06, 0.05, 0.1, (0, 1.12, -0.01), M.foot, rot=tube_rot, bevel=0.006)
    tube = G.hinged(G.join(t, 'tube'), F.at(*AXLE))
    G.settle()  # on its Model's `centre`: the back leg on the wall face
    G.swing(tube, 'aim', 35, seconds=1.2, axis='X')
    return G.meshes()


# The sewing table: a table to the front with the machine, cloth under its needle, thread and a basket; a
# chair behind. The presser foot and needle lower while a resident sews or mends (`presser`), and the
# handwheel turns for as long as they do (`handwheel`, a loop).
MACHINE_Z = 0.28
NEEDLE_X = -0.12
WHEEL = (0.212, 0.94, MACHINE_Z)


def sewingTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.9, 0.035, 0.46, (0, 0.735, 0.2), M.sew_table, bevel=0.008)
        for sx in (-1, 1):
            for z in (0.02, 0.4):
                F.box(f'leg{sx}{z}', 0.04, 0.72, 0.04, (sx * 0.4, 0.36, z), M.sew_legs, bevel=0.006)
            F.box(f'apron_side{sx}', 0.02, 0.07, 0.34, (sx * 0.4, 0.68, 0.21), M.sew_legs)
        F.box('apron_front', 0.76, 0.07, 0.02, (0, 0.68, 0.4), M.sew_legs)
        F.box('drawer', 0.4, 0.07, 0.02, (0, 0.68, 0.02), M.sew_legs, bevel=0.004)
        F.sphere('drawer_knob', 0.012, (0, 0.68, 0.005), M.brass, segments=8, rings=6)
        # the machine: bed, pillar, arm, head
        F.box('bed', 0.38, 0.06, 0.17, (0.02, 0.783, MACHINE_Z), M.machine, bevel=0.015)
        F.box('pillar', 0.09, 0.2, 0.14, (0.155, 0.91, MACHINE_Z), M.machine, bevel=0.025)
        F.box('arm', 0.36, 0.08, 0.13, (0.02, 0.99, MACHINE_Z), M.machine, bevel=0.03)
        F.box('head', 0.08, 0.13, 0.12, (NEEDLE_X, 0.945, MACHINE_Z), M.machine, bevel=0.02)
        F.box('stripe', 0.3, 0.012, 0.002, (0.0, 1.0, MACHINE_Z - 0.066), M.machine_trim)
        F.cyl('dial', 0.022, 0.012, (0.155, 0.93, MACHINE_Z - 0.073), M.machine_trim, rot=(90, 0, 0), verts=14)
        F.box('needle_plate', 0.06, 0.003, 0.05, (NEEDLE_X, 0.8145, MACHINE_Z), M.chrome)
        G.rod('presser_bar_top', F.at(NEEDLE_X, 0.88, MACHINE_Z + 0.012), F.at(NEEDLE_X, 0.86, MACHINE_Z + 0.012),
              0.004, material=M.chrome, verts=6)
        F.cyl('spool_pin', 0.003, 0.05, (0.05, 1.055, MACHINE_Z), M.chrome, verts=6)
        F.cyl('spool', 0.016, 0.03, (0.05, 1.045, MACHINE_Z), M.thread_red, verts=12)
        G.rod('thread', F.at(0.05, 1.05, MACHINE_Z - 0.016), F.at(NEEDLE_X, 0.9, MACHINE_Z - 0.06), 0.0008,
              material=M.thread_red, verts=4)
        # the cloth, the thread and the basket
        F.box('cloth', 0.4, 0.004, 0.3, (-0.2, 0.7545, 0.14), M.fabric_pink, rot=(0, 0, 12), bevel=0.0015)
        for k, (z, colour) in enumerate(((0.12, M.thread_red), (0.22, M.thread_teal))):
            F.cyl(f'reel{k}', 0.022, 0.05, (0.36, 0.78, z), colour, verts=14)
            for y in (0.755, 0.805):
                F.cyl(f'reel_flange{k}{y}', 0.028, 0.006, (0.36, y, z), M.sew_table, verts=14)
        G.revolve('basket', [(0, 0), (0.097, 0), (0.11, 0.12), (0.101, 0.12), (0.088, 0.012), (0, 0.012)],
                  at=F.at(-0.32, 0.7525, 0.36), material=M.basket, segments=20)
        for k, (dx, dz, colour) in enumerate(((-0.03, 0.0, M.thread_teal), (0.035, 0.02, M.fabric_pink),
                                              (0.0, -0.04, M.machine_trim))):
            F.sphere(f'yarn{k}', 0.045, (-0.32 + dx, 0.84, 0.36 + dz), colour, segments=12, rings=8)
        _chair(F, -0.22, M.sew_table, M.sew_pad, d=0.4)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        G.rod('presser_bar', F.at(NEEDLE_X, 0.88, MACHINE_Z + 0.012), F.at(NEEDLE_X, 0.835, MACHINE_Z + 0.012), 0.0035,
              material=M.chrome, verts=6)
        F.box('presser_foot', 0.022, 0.006, 0.035, (NEEDLE_X, 0.833, MACHINE_Z + 0.005), M.chrome, bevel=0.002)
        G.rod('needle_bar', F.at(NEEDLE_X, 0.88, MACHINE_Z - 0.006), F.at(NEEDLE_X, 0.845, MACHINE_Z - 0.006), 0.004,
              material=M.chrome, verts=6)
        G.rod('needle', F.at(NEEDLE_X, 0.845, MACHINE_Z - 0.006), F.at(NEEDLE_X, 0.828, MACHINE_Z - 0.006), 0.0012,
              material=M.chrome, verts=4)
    presser = G.join(p, 'presser')
    with G.new_parts() as w:
        F.cyl('handwheel', 0.04, 0.022, WHEEL, M.machine_trim, rot=(0, 90, 0), verts=24, bevel=0.004)
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
FLASK = [(0, 0), (0.06, 0), (0.06, 0.054), (0.0216, 0.144), (0.0216, 0.18), (0.024, 0.18)]
RISING = (-0.15, -0.05)


def _flask(F, name, x, z, liquid, fill):
    """An Erlenmeyer flask on the bench, glass, with `liquid` up to `fill` (m, below 0.054) inside."""
    G.revolve(name, FLASK, at=F.at(x, BENCH_TOP, z), material=M.glass, segments=18)
    if liquid:
        G.revolve(f'{name}_liquid', [(0, 0.003), (0.056, 0.003), (0.056, fill), (0, fill)],
                  at=F.at(x, BENCH_TOP, z), material=liquid, segments=18)


def chemistrySet():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.92, 0.04, 0.5, (0, 0.88, -0.15), M.chem_bench, bevel=0.008)
        for sx in (-1, 1):
            for z in (-0.36, 0.06):
                F.box(f'leg{sx}{z}', 0.04, 0.86, 0.04, (sx * 0.42, 0.43, z), M.chem_frame, bevel=0.006)
        F.box('shelf_low', 0.9, 0.03, 0.44, (0, 0.3, -0.15), M.chem_frame, bevel=0.006)
        F.box('board', 0.8, 0.5, 0.04, (0, 1.15, -0.38), M.chem_board, bevel=0.008)
        F.box('shelf', 0.8, 0.02, 0.16, (0, 1.25, -0.3), M.chem_bench, bevel=0.004)
        # the rack of test tubes on the shelf
        F.box('rack', 0.6, 0.012, 0.05, (-0.06, 1.3, -0.29), M.chem_frame)
        for x in (-0.36, 0.24):
            F.box(f'rack_end{x}', 0.012, 0.05, 0.05, (x, 1.285, -0.29), M.chem_frame)
        for k, colour in enumerate((M.liquid_green, M.liquid_yellow, M.liquid_orange, M.liquid_blue, M.liquid_red)):
            x = -0.3 + 0.12 * k
            G.revolve(f'tube{k}', [(0, 0.0), (0.015, 0.006), (0.015, 0.14), (0.017, 0.14)],
                      at=F.at(x, 1.26, -0.29), material=M.glass, segments=12)
            G.revolve(f'tube_liquid{k}', [(0, 0.002), (0.012, 0.007), (0.012, 0.05 + 0.012 * (k % 3)),
                                          (0, 0.05 + 0.012 * (k % 3))], at=F.at(x, 1.26, -0.29), material=colour,
                      segments=12)
        # the flasks, the rising one empty at rest
        _flask(F, 'flask_green', -0.3, -0.1, M.liquid_green, 0.045)
        _flask(F, 'flask_rising', RISING[0], RISING[1], None, 0)
        _flask(F, 'flask_blue', 0.25, -0.08, M.liquid_blue, 0.03)
        G.revolve('beaker', [(0, 0), (0.04, 0), (0.04, 0.09), (0.043, 0.095)], at=F.at(0.36, BENCH_TOP, -0.25),
                  material=M.glass, segments=16)
        G.revolve('beaker_liquid', [(0, 0.003), (0.037, 0.003), (0.037, 0.04), (0, 0.04)],
                  at=F.at(0.36, BENCH_TOP, -0.25), material=M.liquid_yellow, segments=16)
        # the microscope
        F.box('scope_base', 0.12, 0.02, 0.15, (0.05, BENCH_TOP + 0.01, -0.22), M.scope, bevel=0.008)
        F.box('scope_arm', 0.04, 0.2, 0.05, (0.05, BENCH_TOP + 0.11, -0.27), M.scope, rot=(-10, 0, 0), bevel=0.012)
        F.box('scope_stage', 0.09, 0.008, 0.08, (0.05, BENCH_TOP + 0.09, -0.2), M.scope, bevel=0.003)
        F.box('slide', 0.06, 0.002, 0.022, (0.05, BENCH_TOP + 0.095, -0.2), M.glass)
        F.cyl('scope_turret', 0.022, 0.02, (0.05, BENCH_TOP + 0.15, -0.2), M.chrome, verts=12)
        F.cyl('scope_objective', 0.008, 0.035, (0.05, BENCH_TOP + 0.125, -0.2), M.chrome, verts=8)
        F.cyl('scope_tube', 0.016, 0.12, (0.05, BENCH_TOP + 0.21, -0.215), M.scope, rot=(-20, 0, 0), verts=12)
        F.cyl('scope_eyepiece', 0.013, 0.03, (0.05, BENCH_TOP + 0.275, -0.238), M.chrome, rot=(-20, 0, 0), verts=10)
        for s in (-1, 1):
            F.cyl(f'scope_knob{s}', 0.016, 0.012, (0.05 + s * 0.028, BENCH_TOP + 0.08, -0.27), M.chrome,
                  rot=(0, 90, 0), verts=12)
        F.box('notes', 0.14, 0.002, 0.18, (-0.02, BENCH_TOP + 0.001, 0.0), M.sheet, rot=(0, 0, -8))
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as p:
        G.revolve('liquid', [(0, 0.003), (0.056, 0.003), (0.056, 0.054), (0.04, 0.095), (0, 0.095)],
                  at=F.at(RISING[0], BENCH_TOP, RISING[1]), material=M.liquid_orange, segments=18)
        for k, (dx, dy, r) in enumerate(((0.0, 0.098, 0.009), (0.015, 0.093, 0.007), (-0.012, 0.094, 0.006))):
            F.sphere(f'bubble{k}', r, (RISING[0] + dx, BENCH_TOP + dy, RISING[1] + 0.005 * k), M.liquid_orange,
                     segments=8, rings=6)
    # Hidden at rest by flattening into the bench top (on the flask's floor, it would show through the glass).
    liquid = G.join(p, 'liquid', origin=F.at(RISING[0], BENCH_TOP - 0.01, RISING[1]))
    liquid.scale[2] = 0.001
    G.settle()  # on its Model's `centre`: its back moved onto the wall face
    G.grow(liquid, 'bubble', seconds=1.5, axis='Z')
    return G.meshes()


# The pottery wheel: a round body with a splash pan, the wheel head and a pot on it, a pedal and a stool
# behind. The pot is its own part and rises from a squat lump while a resident throws (`clay`); the wheel
# head turns for as long as they do (`wheel`, a loop).
WHEEL_Z = 0.12
HEAD_TOP = 0.475
POT = [(0, 0), (0.064, 0), (0.092, 0.04), (0.1, 0.09), (0.08, 0.15), (0.048, 0.184), (0.052, 0.2), (0.04, 0.2),
       (0.036, 0.18), (0, 0.176)]
LUMP = 0.3          # the pot's height at rest, as a share of its height thrown


def potteryWheel():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('base', 0.33, 0.36, (0, 0.18, WHEEL_Z), M.wheel_body, verts=32, bevel=0.02)
        pan = [(0, -0.5), (0.3, -0.5), (0.42, -0.2), (0.5, 0.5), (0.45, 0.5), (0.36, -0.15), (0.25, -0.35), (0, -0.35)]
        G.revolve('splash_pan', [(r * 0.7, z * 0.12) for r, z in pan], at=F.at(0, 0.42, WHEEL_Z), material=M.pan,
                  segments=32)
        F.cyl('spindle', 0.04, 0.03, (0, 0.43, WHEEL_Z), M.chrome, verts=12)
        F.box('pedal', 0.12, 0.04, 0.2, (0.22, 0.02, -0.12), M.foot, rot=(-6, 0, 0), bevel=0.01)
        G.rod('pedal_cable', F.at(0.22, 0.01, 0.0), F.at(0.18, 0.05, WHEEL_Z - 0.3), 0.005, material=M.foot, verts=6)
        F.box('switch', 0.06, 0.04, 0.02, (0.2, 0.3, WHEEL_Z - 0.32), M.foot, bevel=0.005)
        F.box('sponge', 0.06, 0.025, 0.04, (-0.24, 0.395, WHEEL_Z + 0.14), M.drill, bevel=0.008)
        G.rod('wire_tool', F.at(0.2, 0.39, WHEEL_Z + 0.18), F.at(0.27, 0.39, WHEEL_Z + 0.12), 0.003, material=M.chrome,
              verts=4)
        _stool(F, -0.3, 0.44, 0.18, M.stool, M.stool)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as w:
        F.cyl('wheel_head', 0.17, 0.03, (0, 0.46, WHEEL_Z), M.pan, verts=32, bevel=0.004)
        F.cyl('slip', 0.11, 0.002, (0, HEAD_TOP + 0.001, WHEEL_Z), M.clay_slip, verts=24, smooth=False)
        for s in (-1, 1):
            F.cyl(f'bat_pin{s}', 0.008, 0.012, (s * 0.135, HEAD_TOP + 0.006, WHEEL_Z), M.chrome, verts=8)
        F.box('clay_smear', 0.05, 0.002, 0.012, (0.0, HEAD_TOP + 0.001, WHEEL_Z + 0.135), M.clay_slip)
    wheel = G.hinged(G.join(w, 'wheel_head'), F.at(0, HEAD_TOP, WHEEL_Z))
    with G.new_parts() as p:
        G.revolve('pot', POT, at=F.at(0, HEAD_TOP, WHEEL_Z), material=M.clay, segments=24)
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
        F.box('top', 1.7, 0.035, 0.5, (0, 0.735, 0.2), M.craft_top, bevel=0.008)
        for sx in (-1, 1):
            for z in (0.0, 0.4):
                x = sx * 0.78
                F.box(f'plate{sx}{z}', 0.08, 0.006, 0.08, (x, 0.7145, z), M.craft_metal)
                for dz in (-0.03, 0.03):
                    G.rod(f'pin{sx}{z}{dz}', F.at(x, 0.712, z + dz), F.at(x, 0.004, z), 0.0055, material=M.craft_metal,
                          verts=8)
        # the scrapbook: back cover, pages, spine, and the first page's photos (under the cover at rest)
        F.box('back_cover', BOOK_W, 0.004, BOOK_D, (bx, TABLE_TOP + 0.002, bz), M.scrapbook, bevel=0.0015)
        F.box('pages', BOOK_W - 0.01, 0.015, BOOK_D - 0.01, (bx + 0.003, TABLE_TOP + 0.0115, bz), M.pages)
        F.box('spine', 0.008, 0.023, BOOK_D, (bx - BOOK_W / 2, TABLE_TOP + 0.0115, bz), M.scrapbook, bevel=0.002)
        for k, (dx, dz, w, d, colour) in enumerate(((-0.06, -0.04, 0.1, 0.07, M.photo), (0.07, 0.03, 0.08, 0.1, M.photo_warm),
                                                    (-0.05, 0.06, 0.07, 0.05, M.tape_gold))):
            F.box(f'page_photo{k}', w, 0.001, d, (bx + dx, PAGES_TOP + 0.0004, bz + dz), colour, rot=(0, 0, 5 - 4 * k))
        # paper, a pencil pot, washi tape and a box
        F.box('paper', 0.4, 0.008, 0.3, (0.25, TABLE_TOP + 0.004, 0.15), M.paper, rot=(0, 0, 8), bevel=0.002)
        F.box('paper2', 0.3, 0.004, 0.22, (0.27, TABLE_TOP + 0.01, 0.16), M.blush, rot=(0, 0, -6), bevel=0.0015)
        G.revolve('pencil_pot', [(0, 0), (0.04, 0), (0.042, 0.1), (0.037, 0.1), (0.035, 0.006), (0, 0.006)],
                  at=F.at(0.5, TABLE_TOP, 0.33), material=M.cup, segments=16)
        for k, (at, rotate, colour) in enumerate((((0.485, 0.83, 0.32), (12.6, -153.43, 0), M.ball_red),
                                                  ((0.515, 0.825, 0.34), (13.95, 26.57, 0), M.sun),
                                                  ((0.5, 0.828, 0.31), (10.0, 90.0, 0), M.tape_blue))):
            _rod(F, f'pencil{k}', at, 0.15, rotate, 0.004, colour, verts=6)
        for k, (x, z, colour) in enumerate(((0.65, 0.22, M.tape_blue), (0.74, 0.14, M.tape_gold))):
            G.torus(f'tape{k}', 0.04, 0.013, at=F.at(x, TABLE_TOP + 0.013, z), material=colour, major_segments=16,
                    minor_segments=6)
        F.box('box', 0.16, 0.05, 0.1, (0.72, TABLE_TOP + 0.025, 0.37), M.scrapbook, bevel=0.01)
        F.box('box_lid', 0.165, 0.012, 0.105, (0.72, TABLE_TOP + 0.054, 0.37), M.tape_gold, bevel=0.004)
        _stool(F, -0.25, 0.48, 0.19, M.craft_metal, M.craft_top)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as c:
        F.box('cover', BOOK_W, 0.004, BOOK_D, (bx, PAGES_TOP + 0.002, bz), M.scrapbook, bevel=0.0015)
        F.box('cover_label', 0.12, 0.002, 0.07, (bx + 0.02, PAGES_TOP + 0.005, bz - 0.02), M.pages)
        F.box('cover_photo', 0.08, 0.002, 0.06, (bx - 0.05, PAGES_TOP + 0.005, bz + 0.05), M.photo, rot=(0, 0, -6))
        F.box('inside_photo', 0.1, 0.001, 0.08, (bx + 0.03, PAGES_TOP - 0.0003, bz), M.photo_warm)
    cover = G.hinged(G.join(c, 'cover'), F.at(bx - BOOK_W / 2, PAGES_TOP + 0.004, bz))
    G.settle(centre=False)
    G.swing(cover, 'book_open', 176, seconds=1.0, axis='Y')
    return G.meshes()


FAMILY = G.Family(
    'hobbies', M,
    [G.Model('piano', piano, size=(1.54, 1.48, 0.827), key='model.piano', centre=(0, 0.0435),
             motions={'uses': {'play': 'fallboard'}}, paint=[('piano', 'Body'), ('piano_trim', 'Trim')]),
     G.Model('ukulele', ukulele, size=(0.348, 0.841, 0.258), key='model.ukulele', centre=(0, 0.065),
             paint=[('uke', 'Instrument'), ('stand', 'Stand')]),
     G.Model('guitarElectric', guitarElectric, size=(0.46, 1.374, 0.584), key='model.guitarElectric',
             centre=(0, -0.128), paint=[('electric', 'Guitar'), ('amp', 'Amp')]),
     G.Model('keyboardDigital', keyboardDigital, size=(0.9, 1.092, 0.75), key='model.keyboardDigital',
             centre=(0, 0.015), paint=[('kb_body', 'Body'), ('kb_trim', 'Trim')]),
     G.Model('drumKit', drumKit, size=(0.978, 1.088, 0.885), key='model.drumKit', centre=(0.0076, 0.0125),
             motions={'loops': {'drum': 'cymbals', 'practise': 'cymbals'}},
             paint=[('shell', 'Shells'), ('chrome', 'Hardware')]),
     G.Model('guitarAcoustic', guitarAcoustic, size=(0.36, 1.482, 0.405), key='model.guitarAcoustic',
             centre=(0, -0.0217), paint=[('acoustic', 'Guitar'), ('stand', 'Stand')]),
     G.Model('chessTable', chessTable, size=(0.46, 0.92, 0.85), key='model.chessTable', centre=(0, 0.025),
             motions={'uses': {'chess': 'move'}}, paint=[('table_wood', 'Table'), ('chair_wood', 'Trim')]),
     G.Model('poolTable', poolTable, size=(2.5, 0.842, 1.4), key='model.poolTable',
             motions={'uses': {'pool': 'rack_off', 'trickShots': 'rack_off'}},
             paint=[('pool_frame', 'Frame'), ('cloth', 'Cloth')]),
     G.Model('foosball', foosball, size=(1.42, 0.947, 0.985), key='model.foosball', centre=(0, 0.0125),
             motions={'loops': {'foosball': 'rods'}}, paint=[('foos', 'Table'), ('foos_trim', 'Trim')]),
     G.Model('dartboard', dartboard, size=(0.62, 0.62, 0.07), key='model.dartboard', centre=(0, -0.393), floor=False,
             motions={'uses': {'darts': 'darts'}}, paint=[('board', 'Board'), ('cabinet', 'Cabinet')]),
     G.Model('arcadeCabinet', arcadeCabinet, size=(0.672, 1.82, 0.872), key='model.arcadeCabinet',
             centre=(0, 0.006), motions={'loops': {'arcade': 'joystick'}},
             paint=[('arcade', 'Cabinet'), ('art', 'Art')]),
     G.Model('puzzleTable', puzzleTable, size=(0.865, 0.794, 0.891), key='model.puzzleTable', centre=(0.0025, 0.0155),
             motions={'uses': {'jigsaw': 'pieces'}}, paint=[('table_wood', 'Table'), ('chair_wood', 'Trim')]),
     G.Model('easel', easel, size=(0.704, 1.747, 0.762), key='model.easel', centre=(0.0318, -0.0347),
             motions={'uses': {'paint': 'painting'}}, paint=[('easel_wood', 'Wood'), ('easel_metal', 'Metal')]),
     G.Model('workbench', workbench, size=(1.62, 0.995, 0.665), key='model.workbench', centre=(0, -0.0975),
             motions={'uses': {'tinker': 'vice'}}, paint=[('bench_top', 'Wood'), ('bench_steel', 'Metal')]),
     G.Model('telescope', telescope, size=(0.664, 1.522, 0.912), key='model.telescope', centre=(0, 0.026),
             motions={'uses': {'stargaze': 'aim'}}, paint=[('tube', 'Tube'), ('tripod', 'Tripod')]),
     G.Model('sewingTable', sewingTable, size=(0.9, 1.08, 0.89), key='model.sewingTable', centre=(0, 0.025),
             motions={'uses': {'sew': 'presser', 'mend': 'presser'}, 'loops': {'sew': 'handwheel', 'mend': 'handwheel'}},
             paint=[('sew_table', 'Table'), ('machine', 'Machine')]),
     G.Model('chemistrySet', chemistrySet, size=(0.92, 1.4, 0.5), key='model.chemistrySet', centre=(0, -0.18),
             motions={'uses': {'experiment': 'bubble'}}, paint=[('chem_bench', 'Bench'), ('glass', 'Glassware')]),
     G.Model('potteryWheel', potteryWheel, size=(0.7, 0.544, 0.95), key='model.potteryWheel', centre=(0, -0.005),
             motions={'uses': {'throw': 'clay'}, 'loops': {'throw': 'wheel'}},
             paint=[('wheel_body', 'Wheel'), ('stool', 'Stool')]),
     G.Model('craftTable', craftTable, size=(1.7, 0.904, 0.89), key='model.craftTable', centre=(0, 0.005),
             motions={'uses': {'scrapbook': 'book_open'}}, paint=[('craft_top', 'Wood'), ('craft_metal', 'Metal')])],
    comment='Blender-built hobby models (tools/art/blender/families/hobbies.py): music, games, crafts and science. '
            'Replaces the primitive-part placeholders by key; listed in the base manifest\'s packs. The placeholder '
            'is kept as a fallback if a file fails to load, and the materials map onto the game\'s finishes so a '
            'model costs one draw call per finish group. The piano lifts its fallboard, the drums\' cymbals and the '
            'foosball rods move while played, the darts, the painting, the jigsaw patch, the pot and the liquid '
            'grow, and the other hobbies move a part while a resident uses them.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
