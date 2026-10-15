# The bookworm pack (P1): the window-seat reading nook, the rare-books library wall, the writer's bureau, the
# book-club tea trolley, the fireside reading lounge and the bedside book stack, over their primitive-part
# placeholders (`tools/art/pack_bookworm.py`, `packs/bookworm/manifest.json`), which stay the fallback.
# Notes: pack_bookworm.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the nook's seat cushion sinks (`sink_in`), the library ladder rolls along
# its rail (`ladder`), the bureau's writing flap drops (`fall_front`), the teapot pours (`pour`), the fire
# grows in the grate (`flames`) and the top book of the bedside stack opens (`book_up`).
#
# Run:  python3 -B tools/art/blender/build.py pack_bookworm [--only readingNook] [--preview]
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#9A6A45', 'wood'),        # the library walnut: cases, shelves, the bureau; the main part
    panel=('#B88A60', 'wood'),       # drawer fronts, inset panels, the bureau's flap
    honey=('#C79A62', 'wood'),       # the ladder, the bureau chair, the trolley's shelves
    walnut=('#8E6243', 'satin'),     # the lounge's wood, plain (the lounge's fabric takes its textured draw)
    back=('#E8DCC6', 'matte'),       # the cases' backs
    brass=('#C9A54A', 'metal'),
    iron=('#3F4247', 'metal'),       # the fire grate, the hood and flue
    dark=('#2F3236', 'satin'),       # castors, spectacles, the typewriter platen
    stone=('#B9B0A2', 'satin'),      # the hearth stone
    fabric=('#C9B48A', 'fabric'),    # the nook's seat cushion and throw
    throw=('#7A2E2E', 'fabric'),
    cushion=('#E6DCC6', 'satin'),    # the nook's back cushions
    pillow_a=('#2F5D62', 'satin'),
    pillow_b=('#B5653F', 'satin'),
    velvet=('#4F6B55', 'fabric'),    # the wing chairs
    rug=('#9C4A3A', 'fabric'),
    rug_inner=('#C9A46E', 'fabric'),
    seatpad=('#7A2E2E', 'satin'),    # the bureau chair's pad
    linen=('#EDE3CF', 'satin'),      # the trolley's cloth
    shade=('#E9D5A8', 'satin'),      # lamp shades
    lamp_amber=('#D6A64A', 'satin'),
    porcelain=('#F6F3EA', 'gloss'),
    glaze=('#2F5D62', 'gloss'),      # the tea set's band, the biscuit tin
    cake=('#C8964F', 'matte'),
    icing=('#F4E9D8', 'satin'),
    page=('#F4EEDF', 'matte'),
    globe=('#6E9FB4', 'gloss'),
    bust=('#E9E4DA', 'satin'),
    log=('#7A5A40', 'satin'),
    wicker=('#C9A46E', 'satin'),
    flame=('#F28C28', 'gloss'),
    flame_core=('#FFC24A', 'gloss'),
    ember=('#3A2420', 'satin'),
    mug_a=('#2F5D62', 'gloss'),
    mug_b=('#D6A64A', 'gloss'),
    # book spines: plain satin, so a wall of books costs no draw call of its own
    book_a=('#7A2E2E', 'satin'),
    book_b=('#2F5D62', 'satin'),
    book_c=('#B5653F', 'satin'),
    book_d=('#D6A64A', 'satin'),
    book_e=('#3E5A3A', 'satin'),
    book_f=('#E6DCC6', 'satin'),
    book_g=('#5A4A7A', 'satin'),
)

BOOKS = ('book_a', 'book_b', 'book_c', 'book_d', 'book_e', 'book_f', 'book_g')   # by name: made anew after each reset
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _books(F, rnd, x0, x1, y, z_back, depth, h_max, fill=0.92, stack=False):
    """A row of books standing on `y` from `x0` towards `x1` (game), spines to the front, backs on `z_back`;
    the gap left at the end takes a short pile lying flat when `stack`."""
    x, end = x0 + 0.004, x0 + (x1 - x0) * fill
    while x < end:
        t = rnd.uniform(0.022, 0.05)
        h = h_max * rnd.uniform(0.66, 1.0)
        d = depth * rnd.uniform(0.78, 1.0)
        F.box(_name('book'), t, h, d, (x + t / 2, y + h / 2, z_back + d / 2), getattr(M, rnd.choice(BOOKS)))
        x += t + 0.002
    if stack and x1 - x > 0.16:
        w = min(0.2, x1 - x - 0.02)
        _pile(F, rnd, x + 0.01 + w / 2, z_back + depth * 0.45, y, 3, w, depth * 0.85)


def _pile(F, rnd, x, z, y, n, w=0.2, d=0.15):
    """`n` books lying in a pile on `y`, centred on (x, z); returns the height of its top."""
    for _ in range(n):
        h = rnd.uniform(0.025, 0.045)
        F.box(_name('pile'), w * rnd.uniform(0.85, 1.05), h, d * rnd.uniform(0.9, 1.05),
              (x + rnd.uniform(-0.01, 0.01), y + h / 2, z), getattr(M, rnd.choice(BOOKS)), rot=(0, 0, rnd.uniform(-10, 10)))
        y += h
    return y


def _mug(F, x, y, z, material):
    G.revolve(_name('mug'), [(0, 0), (0.038, 0), (0.042, 0.09), (0.036, 0.09), (0.032, 0.012), (0, 0.012)],
              at=F.at(x, y, z), material=material, segments=16)
    G.torus(_name('mug_handle'), 0.024, 0.007, at=F.at(x + 0.045, y + 0.045, z), material=material, rot=(90, 0, 0),
            major_segments=12, minor_segments=6)


def _lamp(F, x, y, z, h=0.42, shade=None):
    """A brass table lamp standing on `y`: a domed foot, a stem and a drum shade."""
    shade = shade or M.shade
    G.revolve(_name('lamp_foot'), [(0, 0), (0.065, 0), (0.06, 0.015), (0.025, 0.035), (0, 0.04)], at=F.at(x, y, z),
              material=M.brass, segments=18)
    F.cyl(_name('lamp_stem'), 0.011, h * 0.62, (x, y + 0.03 + h * 0.31, z), M.brass, verts=10)
    G.revolve(_name('lamp_shade'), [(0.3 * h, 0), (0.21 * h, h * 0.42), (0.2 * h, h * 0.42), (0.29 * h, 0.004)],
              at=F.at(x, y + h * 0.56, z), material=shade, segments=24)


# --- the window-seat reading nook (2x1): a cushioned seat between two little bookcases ---------------------
def readingNook():
    F = G.GameFrame()
    rnd = random.Random(3)
    with G.new_parts() as still:
        for s in (-1, 1):
            x = s * 0.73
            for dx in (-0.195, 0.195):                                            # the case's sides
                F.box(_name('side'), 0.03, 1.5, 0.4, (x + dx, 0.75, WALL + 0.2), M.body, bevel=0.006)
            F.box(_name('back'), 0.36, 1.44, 0.012, (x, 0.75, WALL + 0.006), M.back)
            for y in (0.04, 0.42, 0.8, 1.18):
                F.box(_name('shelf'), 0.36, 0.025, 0.38, (x, y, WALL + 0.2), M.body, bevel=0.004)
            for k, y in enumerate((0.0525, 0.4325, 0.8125, 1.1925)):
                _books(F, rnd, x - 0.175, x + 0.175, y, WALL + 0.02, 0.24, 0.3 if k < 3 else 0.22,
                       fill=(0.9, 0.62, 0.95, 0.5)[k], stack=k in (1, 3))
        _mug(F, 0.64, 0.4325, -0.15, M.mug_a)
        # the header over the seat, its valance and a pendant reading light
        F.box('header', 1.88, 0.18, 0.42, (0, 1.62, WALL + 0.21), M.body, bevel=0.012)
        F.box('valance', 1.06, 0.06, 0.4, (0, 1.5, WALL + 0.2), M.body, bevel=0.008)
        F.box('valance_bead', 1.06, 0.02, 0.02, (0, 1.47, -0.04), M.panel, bevel=0.006)
        F.cyl('pendant_rod', 0.008, 0.22, (0, 1.36, -0.12), M.brass, verts=8)
        G.revolve('pendant', [(0.11, 0), (0.035, 0.1), (0.012, 0.11), (0, 0.11)], at=F.at(0, 1.15, -0.12),
                  material=M.lamp_amber, segments=20)
        # the panelled back of the seat
        F.box('seat_back', 1.04, 1.02, 0.02, (0, 0.96, WALL + 0.01), M.back)
        for x in (-0.26, 0.26):
            F.box(_name('seat_back_panel'), 0.44, 0.5, 0.012, (x, 1.12, WALL + 0.026), M.panel, bevel=0.004)
        # the seat box with its two drawers
        F.box('seat_box', 1.04, 0.36, 0.61, (0, 0.18, WALL + 0.305), M.body, bevel=0.01)
        for x in (-0.25, 0.25):
            F.box(_name('drawer'), 0.46, 0.22, 0.012, (x, 0.18, 0.186), M.panel, bevel=0.005)
            F.sphere(_name('knob'), 0.016, (x, 0.18, 0.2), M.brass, segments=10, rings=6)
        F.box('plinth', 1.0, 0.03, 0.02, (0, 0.015, 0.17), M.panel, bevel=0.004)
        # back cushions and two throw pillows
        for x in (-0.25, 0.25):
            F.box(_name('back_cushion'), 0.48, 0.42, 0.13, (x, 0.68, -0.33), M.cushion, rot=(-8, 0, 0),
                  bevel=0.05, bevel_segments=3)
        F.box('pillow_l', 0.3, 0.28, 0.1, (-0.38, 0.6, -0.22), M.pillow_a, rot=(-12, -6, 30), bevel=0.045,
              bevel_segments=3)
        F.box('pillow_r', 0.28, 0.26, 0.1, (0.38, 0.6, -0.23), M.pillow_b, rot=(-12, 6, -28), bevel=0.045,
              bevel_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushion and the throw over it, joined with their origin on the underside: `sink_in` presses
    # them 2 cm while a resident is curled up on it.
    with G.new_parts() as seat:
        F.box('seat_cushion', 1.02, 0.09, 0.6, (0, 0.405, -0.125), M.fabric, bevel=0.03, bevel_segments=3)
        F.box('throw_top', 0.42, 0.02, 0.6, (0.28, 0.457, -0.12), M.throw, bevel=0.008)
        F.box('throw_drop', 0.42, 0.16, 0.02, (0.28, 0.38, 0.186), M.throw, bevel=0.008)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 0.36, -0.125))
    objs = G.settle(centre=False)
    G.squash(seat, 'sink_in', (1, 1, 0.78), seconds=0.6)
    return objs


# --- the rare-books library wall (2x1): floor-to-ceiling shelves, a brass rail and a rolling ladder ---------
LADDER_TOP, LADDER_FOOT = (1.98, -0.02), (0.02, 0.36)   # (height, depth) of the rails' top and foot


def _ladder_at(t):
    """(height, depth) a fraction `t` down the ladder from its top."""
    return (LADDER_TOP[0] + (LADDER_FOOT[0] - LADDER_TOP[0]) * t, LADDER_TOP[1] + (LADDER_FOOT[1] - LADDER_TOP[1]) * t)


def libraryWall():
    F = G.GameFrame()
    rnd = random.Random(11)
    with G.new_parts() as still:
        F.box('cornice', 1.88, 0.06, 0.42, (0, 2.31, WALL + 0.21), M.body, bevel=0.01)
        F.box('frieze', 1.84, 0.04, 0.4, (0, 2.26, WALL + 0.2), M.panel, bevel=0.006)
        F.box('plinth', 1.84, 0.1, 0.38, (0, 0.05, WALL + 0.19), M.body, bevel=0.006)
        F.box('back', 1.8, 2.16, 0.012, (0, 1.18, WALL + 0.006), M.back)
        for x in (-0.91, -0.3, 0.3, 0.91):
            F.box(_name('upright'), 0.04, 2.14, 0.38, (x, 1.17, WALL + 0.19), M.body, bevel=0.006)
        ys = (0.1, 0.52, 0.92, 1.32, 1.72, 2.12)
        for y in ys:
            F.box(_name('shelf'), 1.8, 0.03, 0.37, (0, y, WALL + 0.19), M.body, bevel=0.004)
        bays = ((-0.89, -0.32), (-0.28, 0.28), (0.32, 0.89))
        for i, y in enumerate(ys[:-1]):
            for j, (a, b) in enumerate(bays):
                if (i, j) == (3, 2):                 # the globe's bay
                    _books(F, rnd, a, b, y + 0.015, WALL + 0.02, 0.28, 0.32, fill=0.4)
                    continue
                if (i, j) == (2, 0):                 # the bust's bay
                    _books(F, rnd, a + 0.16, b, y + 0.015, WALL + 0.02, 0.28, 0.32, fill=0.9)
                    continue
                k = (i * 3 + j) % 4
                _books(F, rnd, a, b, y + 0.015, WALL + 0.02, 0.28, 0.34, fill=(0.95, 0.7, 0.85, 0.55)[k], stack=k == 3)
        # a globe on its stand and a marble bust
        F.cyl('globe_foot', 0.06, 0.03, (0.72, 1.35, -0.25), M.brass, verts=16)
        F.cyl('globe_stem', 0.01, 0.04, (0.72, 1.38, -0.25), M.brass, verts=8)
        F.sphere('globe', 0.1, (0.72, 1.5, -0.25), M.globe, segments=20, rings=12)
        G.torus('globe_ring', 0.115, 0.006, at=F.at(0.72, 1.5, -0.25), material=M.brass, rot=(90, 20, 0),
                major_segments=24, minor_segments=6)
        F.box('bust_plinth', 0.12, 0.06, 0.12, (-0.8, 0.98, -0.28), M.bust, bevel=0.008)
        F.sphere('bust_shoulders', 0.07, (-0.8, 1.05, -0.28), M.bust, scale=(1.2, 0.7, 0.75), segments=16, rings=10)
        F.sphere('bust_head', 0.05, (-0.8, 1.13, -0.28), M.bust, scale=(0.85, 0.95, 1.1), segments=16, rings=10)
        # the brass rail the ladder hangs on, on brackets
        G.rod('rail', F.at(-0.92, LADDER_TOP[0], LADDER_TOP[1]), F.at(0.92, LADDER_TOP[0], LADDER_TOP[1]), 0.0125,
              material=M.brass, verts=12)
        for x in (-0.6, 0.0, 0.6):
            G.rod(_name('bracket'), F.at(x, LADDER_TOP[0], WALL + 0.38), F.at(x, LADDER_TOP[0], LADDER_TOP[1]), 0.01,
                  material=M.brass, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rolling ladder: two rails, seven treads, hooks over the brass rail and wheels at its feet.
    with G.new_parts() as ladder:
        for x in (0.15, 0.55):
            top, foot = F.at(x, *LADDER_TOP), F.at(x, LADDER_FOOT[0] + 0.03, LADDER_FOOT[1])
            G.rod(_name('stile'), top, foot, 0.022, material=M.honey, verts=8)
            G.torus(_name('hook'), 0.03, 0.008, at=F.at(x, LADDER_TOP[0] + 0.01, LADDER_TOP[1]), material=M.body,
                    rot=(90, 0, 90), major_segments=12, minor_segments=6)
            F.cyl(_name('wheel'), 0.03, 0.02, (x, 0.03, LADDER_FOOT[1]), M.body, rot=(0, 90, 0), verts=14)
        for k in range(1, 8):
            y, z = _ladder_at(k / 8)
            F.box(_name('tread'), 0.4, 0.025, 0.07, (0.35, y, z), M.honey, bevel=0.006)
    ladder = G.join(ladder, 'ladder')
    objs = G.settle(centre=False)
    G.slide(ladder, 'ladder', (0.4, 0, 0), seconds=1.2)    # 40 cm along the rail, towards the middle
    return objs


def _turn_round(objs):
    """Turns a model built in place half a turn about the footprint's middle (each part about its own origin,
    which moves round with it), as hobbies.py turns the piano. A part's clips then open the other way."""
    for o in objs:
        G.turn(o, 180, 'Z')
        o.location = (-o.location.x, -o.location.y, o.location.z)


# --- the writer's bureau (1x1): its back on the wall, the chair in front; the writer faces the bureau ------
# (content `seat.facing: "back"`). Built as the placeholder first stood, the chair at the back, then turned round.
def writingBureau():
    F = G.GameFrame()
    rnd = random.Random(5)
    with G.new_parts() as still:
        # two pedestals of drawers facing the writer, a kneehole between them
        for s in (-1, 1):
            x = s * 0.31
            F.box(_name('pedestal'), 0.2, 0.73, 0.28, (x, 0.365, 0.3), M.body, bevel=0.008)
            for y in (0.17, 0.4, 0.6):
                F.box(_name('drawer'), 0.17, 0.17 if y < 0.3 else 0.15, 0.012, (x, y, 0.155), M.panel, bevel=0.005)
                F.sphere(_name('knob'), 0.013, (x, y, 0.145), M.brass, segments=10, rings=6)
        F.box('modesty', 0.42, 0.6, 0.015, (0, 0.42, 0.43), M.body, bevel=0.004)
        F.box('modesty_panel', 0.34, 0.46, 0.008, (0, 0.42, 0.44), M.panel, bevel=0.004)
        F.box('top', 0.84, 0.04, 0.32, (0, 0.75, 0.29), M.body, bevel=0.01)
        # the hutch on the desk: pigeonholes behind the flap, an open shelf above, a curved crest
        F.box('hutch_back', 0.8, 0.52, 0.02, (0, 1.03, 0.43), M.body)
        for x in (-0.19, 0.19):                            # panels on its back (on the wall; out in a room, it shows)
            F.box(_name('hutch_panel'), 0.32, 0.4, 0.008, (x, 1.03, 0.442), M.panel, bevel=0.003)
        for x in (-0.4, 0.4):
            F.box(_name('hutch_side'), 0.022, 0.52, 0.17, (x, 1.03, 0.355), M.body, bevel=0.004)
        F.box('hutch_shelf', 0.78, 0.02, 0.16, (0, 0.99, 0.355), M.body)
        F.box('hutch_top', 0.84, 0.03, 0.2, (0, 1.305, 0.35), M.body, bevel=0.008)
        F.cyl('crest', 0.06, 0.8, (0, 1.29, 0.4), M.panel, rot=(0, 90, 0), verts=20, scale=(1, 0.7, 1))
        for x in (-0.125, 0.0, 0.125):
            F.box(_name('divider'), 0.012, 0.2, 0.15, (x, 0.875, 0.35), M.body)
        for x, m in ((-0.19, M.page), (-0.06, M.cushion), (0.19, M.shade)):
            F.box(_name('letters'), 0.1, 0.09, 0.08, (x, 0.82, 0.35), m, rot=(0, -6, 0))
        for x in (-0.32, 0.32):                            # small drawers either side of the flap
            F.box(_name('little_drawer'), 0.13, 0.08, 0.012, (x, 0.83, 0.27), M.panel, bevel=0.004)
            F.box(_name('little_drawer'), 0.13, 0.08, 0.012, (x, 0.93, 0.27), M.panel, bevel=0.004)
            for y in (0.83, 0.93):
                F.sphere(_name('knob'), 0.009, (x, y, 0.26), M.brass, segments=8, rings=6)
            F.box(_name('little_case'), 0.13, 0.2, 0.15, (x, 0.88, 0.355), M.body)
        _books(F, rnd, -0.38, 0.2, 1.0, 0.3, 0.12, 0.2, fill=0.95)
        F.cyl('inkwell', 0.03, 0.05, (0.3, 1.025, 0.34), M.dark, verts=12)
        F.cyl('ink_cap', 0.016, 0.015, (0.3, 1.055, 0.34), M.brass, verts=10)
        _lamp(F, -0.33, 0.77, 0.21, h=0.34, shade=M.lamp_amber)
        _mug(F, 0.33, 0.77, 0.2, M.mug_b)
        # the chair, the writer's seat at 0.45
        F.box('chair_seat', 0.42, 0.04, 0.4, (0, 0.41, -0.1), M.honey, bevel=0.012)
        F.box('chair_pad', 0.38, 0.03, 0.36, (0, 0.445, -0.09), M.seatpad, bevel=0.012, bevel_segments=3)
        for dx in (-0.18, 0.18):
            F.cone(_name('front_leg'), 0.018, 0.014, 0.39, (dx, 0.195, 0.06), M.honey, verts=10)
            G.rod(_name('back_leg'), F.at(dx, 0.0, -0.27), F.at(dx, 0.93, -0.29), 0.018, material=M.honey, verts=10)
        F.box('chair_crest', 0.4, 0.13, 0.035, (0, 0.85, -0.29), M.honey, bevel=0.015)
        F.box('chair_rail', 0.4, 0.04, 0.03, (0, 0.65, -0.285), M.honey, bevel=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # The writing flap, closed up over the pigeonholes, hinged on its foot: `fall_front` drops it onto the desk.
    with G.new_parts() as flap:
        F.box('flap', 0.5, 0.2, 0.02, (0, 0.875, 0.265), M.panel, bevel=0.005)
        F.box('flap_inset', 0.42, 0.14, 0.006, (0, 0.875, 0.253), M.body, bevel=0.003)
        F.box('flap_pull', 0.06, 0.015, 0.015, (0, 0.95, 0.248), M.honey, bevel=0.005)
    flap = G.hinged(G.join(flap, 'flap'), F.at(0, 0.775, 0.265))
    _turn_round(G.meshes())
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(flap, 'fall_front', 90, seconds=0.8, axis='X')
    return objs


# --- the book-club tea trolley (1x1): oak shelves on a brass frame, a tea set on a cloth ---------------------
def _teapot(F, x, y, z):
    """The teapot, spout to the right (+x game): body, lid, knob, spout and handle."""
    G.revolve(_name('pot'), [(0, 0), (0.05, 0), (0.075, 0.03), (0.08, 0.06), (0.068, 0.1), (0.04, 0.115), (0, 0.115)],
              at=F.at(x, y, z), material=M.porcelain, segments=22)
    G.torus(_name('pot_band'), 0.079, 0.006, at=F.at(x, y + 0.055, z), material=M.glaze, major_segments=22, minor_segments=6)
    F.sphere(_name('pot_knob'), 0.014, (x, y + 0.125, z), M.glaze, segments=10, rings=6)
    G.rod(_name('spout'), F.at(x + 0.06, y + 0.04, z), F.at(x + 0.13, y + 0.11, z), 0.014, material=M.porcelain, verts=10)
    G.torus(_name('pot_handle'), 0.035, 0.008, at=F.at(x - 0.085, y + 0.065, z), material=M.porcelain, rot=(90, 0, 0),
            major_segments=14, minor_segments=6)


def _cup(F, x, y, z):
    F.cyl(_name('saucer'), 0.055, 0.008, (x, y + 0.004, z), M.porcelain, verts=18)
    G.revolve(_name('cup'), [(0, 0), (0.025, 0), (0.042, 0.05), (0.038, 0.05), (0.022, 0.006), (0, 0.006)],
              at=F.at(x, y + 0.008, z), material=M.porcelain, segments=16)
    G.torus(_name('cup_band'), 0.04, 0.004, at=F.at(x, y + 0.05, z), material=M.glaze, major_segments=16, minor_segments=4)


def teaTrolley():
    F = G.GameFrame()
    rnd = random.Random(7)
    with G.new_parts() as still:
        for y in (0.32, 0.78):
            F.box(_name('shelf'), 0.72, 0.03, 0.42, (0, y, -0.2), M.honey, bevel=0.008)
            for z in (-0.41, 0.01):                       # brass gallery rails along the long sides
                G.rod(_name('gallery'), F.at(-0.36, y + 0.05, z), F.at(0.36, y + 0.05, z), 0.006, material=M.brass, verts=8)
        for x in (-0.36, 0.36):
            for z in (-0.41, 0.01):
                F.cyl(_name('post'), 0.013, 0.82 if x < 0 else 0.86, (x, 0.47 if x < 0 else 0.49, z), M.brass, verts=10)
                G.torus(_name('castor'), 0.024, 0.012, at=F.at(x, 0.036, z), material=M.dark, rot=(90, 0, 90),
                        major_segments=14, minor_segments=6)
                F.cyl(_name('fork'), 0.008, 0.03, (x, 0.07, z), M.brass, verts=8)
        G.rod('handle', F.at(0.36, 0.92, -0.41), F.at(0.36, 0.92, 0.01), 0.014, material=M.brass, verts=10)
        F.box('cloth', 0.6, 0.006, 0.36, (0, 0.798, -0.2), M.linen, bevel=0.002)
        F.box('cloth_drop', 0.6, 0.08, 0.006, (0, 0.76, -0.01), M.linen, bevel=0.002)
        _cup(F, 0.02, 0.801, -0.12)
        _cup(F, 0.2, 0.801, -0.27)
        G.revolve('milk_jug', [(0, 0), (0.028, 0), (0.034, 0.05), (0.026, 0.075), (0.03, 0.08), (0, 0.072)],
                  at=F.at(0.05, 0.801, -0.29), material=M.porcelain, segments=16)
        G.revolve('sugar_bowl', [(0, 0), (0.03, 0), (0.04, 0.035), (0.034, 0.05), (0, 0.05)],
                  at=F.at(0.22, 0.801, -0.1), material=M.porcelain, segments=16)
        F.sphere('sugar_knob', 0.01, (0.22, 0.857, -0.1), M.glaze, segments=8, rings=6)
        # below: the biscuit tin, a cake on its stand and two books
        F.cyl('tin', 0.1, 0.12, (-0.2, 0.395, -0.22), M.glaze, verts=18)
        F.cyl('tin_lid', 0.105, 0.02, (-0.2, 0.465, -0.22), M.brass, verts=18)
        G.revolve('cake_stand', [(0, 0), (0.05, 0), (0.02, 0.02), (0.02, 0.07), (0.12, 0.075), (0.12, 0.085), (0, 0.085)],
                  at=F.at(0.14, 0.335, -0.2), material=M.porcelain, segments=16)
        F.cyl('cake', 0.09, 0.06, (0.14, 0.45, -0.2), M.cake, verts=16)
        F.cyl('icing', 0.092, 0.012, (0.14, 0.486, -0.2), M.icing, verts=16)
        _pile(F, rnd, 0.0, -0.36, 0.335, 2, 0.18, 0.12)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as pot:
        _teapot(F, -0.17, 0.801, -0.2)
    # `pour` tips it towards its spout about the foot's edge under the spout.
    pot = G.hinged(G.join(pot, 'teapot'), F.at(-0.12, 0.801, -0.2))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(pot, 'pour', -30, seconds=0.8, axis='Y')
    return objs


# --- the fireside reading lounge (2x2): two wing chairs before an open fire grate under a hood ----------------
STOVE = (0.0, 0.62)   # the grate's middle (x, depth)


def _wing_chair(F, x, z=0.0):
    """A wing chair centred on (x, z), facing the front: seat top 0.45, a tall back with wings, rolled arms."""
    w, d = 0.66, 0.64
    F.box(_name('chair_base'), w - 0.06, 0.22, d - 0.04, (x, 0.25, z), M.velvet, bevel=0.04, bevel_segments=3)
    F.box(_name('chair_seat'), w - 0.24, 0.1, d - 0.2, (x, 0.4, z + 0.07), M.velvet, bevel=0.035, bevel_segments=3)
    F.box(_name('chair_back'), w - 0.1, 0.7, 0.15, (x, 0.7, z - d / 2 + 0.1), M.velvet, rot=(7, 0, 0), bevel=0.06,
          bevel_segments=3)
    for s in (-1, 1):
        ax = x + s * (w / 2 - 0.07)
        F.box(_name('chair_arm'), 0.12, 0.26, d - 0.14, (ax, 0.47, z + 0.03), M.velvet, bevel=0.04, bevel_segments=2)
        F.cyl(_name('chair_roll'), 0.075, d - 0.14, (ax + s * 0.012, 0.6, z + 0.03), M.velvet, rot=(90, 0, 0), verts=16)
        F.box(_name('chair_wing'), 0.09, 0.4, 0.28, (x + s * (w / 2 - 0.05), 0.86, z - d / 2 + 0.2), M.velvet,
              rot=(0, 0, s * 12), bevel=0.04, bevel_segments=2)
        for lz in (z - d / 2 + 0.07, z + d / 2 - 0.07):
            F.cone(_name('chair_leg'), 0.025, 0.017, 0.14, (ax, 0.07, lz), M.walnut, verts=10)


def firesideLounge():
    F = G.GameFrame()
    rnd = random.Random(5)
    sx, sz = STOVE
    with G.new_parts() as still:
        F.cyl('rug', 0.85, 0.012, (0, 0.006, 0.2), M.rug, verts=40, scale=(1, 0.76, 1))
        F.cyl('rug_field', 0.75, 0.014, (0, 0.007, 0.2), M.rug_inner, verts=40, scale=(1, 0.75, 1))
        # the hearth stone, the iron grate on its legs with logs, the hood on its flue
        F.cyl('hearth', 0.3, 0.05, (sx, 0.025, sz), M.stone, verts=28, bevel=0.01)
        G.revolve('grate', [(0.0, 0.0), (0.16, 0.0), (0.2, 0.14), (0.185, 0.14), (0.15, 0.015), (0.0, 0.015)],
                  at=F.at(sx, 0.12, sz), material=M.iron, segments=20)
        for k in range(3):
            a = 120 * k + 30
            F.cyl(_name('grate_leg'), 0.012, 0.08, (sx + 0.15 * G.cosd(a), 0.09, sz + 0.15 * G.sind(a)), M.iron, verts=8)
        for k, (a, dx) in enumerate(((20, -0.04), (-25, 0.05), (90, 0.0))):
            F.cyl(_name('log'), 0.035, 0.28, (sx + dx, 0.17 + 0.03 * (k == 2), sz), M.log, rot=(90, 0, a), verts=10)
        F.cyl('embers', 0.13, 0.02, (sx, 0.16, sz), M.ember, verts=18)
        G.revolve('hood', [(0.25, 0.0), (0.235, 0.02), (0.07, 0.26), (0.06, 0.28), (0.06, 0.29), (0.24, 0.03)],
                  at=F.at(sx, 1.0, sz), material=M.iron, segments=28)
        F.cyl('flue', 0.06, 0.68, (sx, 1.62, sz), M.iron, verts=20)
        # the chairs, a side table with books and tea between them, a low bookcase behind
        _wing_chair(F, -0.5)
        _wing_chair(F, 0.5)
        F.cyl('table_top', 0.15, 0.03, (0, 0.56, -0.16), M.walnut, verts=22, bevel=0.006)
        G.revolve('table_stem', [(0.12, 0), (0.12, 0.02), (0.03, 0.06), (0.04, 0.3), (0.025, 0.54), (0, 0.545)],
                  at=F.at(0, 0, -0.16), material=M.walnut, segments=16)
        top = _pile(F, rnd, -0.02, -0.17, 0.575, 3, 0.15, 0.11)
        _mug(F, 0.07, 0.575, -0.09, M.porcelain)
        F.box('case', 1.7, 0.6, 0.3, (0, 0.3, -0.78), M.walnut, bevel=0.01)
        F.box('case_mouth', 1.6, 0.48, 0.02, (0, 0.32, -0.626), M.back)
        _books(F, rnd, -0.8, 0.1, 0.6, -0.9, 0.2, 0.3, fill=0.75, stack=True)
        _lamp(F, 0.6, 0.6, -0.78, h=0.5)
        # a wicker log basket by the fire
        G.revolve('basket', [(0, 0), (0.15, 0), (0.17, 0.24), (0.16, 0.24), (0.14, 0.02), (0, 0.02)],
                  at=F.at(0.68, 0, 0.62), material=M.wicker, segments=20, scale=(1, 0.76, 1))
        for k in range(3):
            F.cyl(_name('spare_log'), 0.035, 0.32, (0.62 + 0.05 * k, 0.26, 0.62), M.log, rot=(90, 30 * k - 30, 0), verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The fire: tongues of flame on the logs, joined with their origin on the grate and built flat, so
    # `flames` grows them up while residents read by it.
    with G.new_parts() as fire:
        for k, (dx, dz, r, h, m) in enumerate(((0.0, 0.0, 0.07, 0.3, M.flame), (-0.07, 0.03, 0.05, 0.2, M.flame),
                                                (0.07, -0.03, 0.05, 0.22, M.flame), (0.02, 0.05, 0.04, 0.16, M.flame_core),
                                                (-0.02, -0.04, 0.035, 0.18, M.flame_core))):
            F.cone(_name('flame'), r, 0.0, h, (sx + dx, 0.17 + h / 2, sz + dz), m, verts=10)
    fire = G.join(fire, 'flames', origin=F.at(sx, 0.17, sz))
    fire.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(fire, 'flames', seconds=0.8)
    return objs


# --- the bedside book stack (1x1): a little chest of drawers piled with books, a lamp and tea ----------------
def bedsideStack():
    F = G.GameFrame()
    rnd = random.Random(9)
    h, d = 0.5, 0.43
    zf = WALL + d
    with G.new_parts() as still:
        F.box('chest_top', 0.8, 0.03, d, (0, h - 0.015, WALL + d / 2), M.body, bevel=0.008)
        F.box('chest', 0.78, h - 0.09, d - 0.02, (0, 0.06 + (h - 0.09) / 2, WALL + (d - 0.02) / 2), M.body, bevel=0.006)
        for lx in (-0.36, 0.36):
            for lz in (WALL + 0.04, zf - 0.04):
                G.revolve(_name('leg'), [(0, 0), (0.015, 0), (0.025, 0.035), (0.022, 0.06), (0, 0.06)], at=F.at(lx, 0, lz),
                          material=M.body, segments=12)
        fw, fh = 0.74 / 3, (h - 0.14) / 2
        for i in range(3):
            for j in range(2):
                cx, cy = -0.37 + fw * (i + 0.5), 0.095 + fh * (j + 0.5)
                F.box(_name('drawer'), fw - 0.014, fh - 0.014, 0.016, (cx, cy, zf - 0.002), M.panel, bevel=0.005)
                F.box(_name('pull'), 0.07, 0.02, 0.018, (cx, cy - 0.01, zf + 0.012), M.brass, bevel=0.006)
                F.box(_name('label'), 0.05, 0.028, 0.004, (cx, cy + 0.04, zf + 0.007), M.page, bevel=0.001)
        _pile(F, rnd, -0.16, -0.25, h, 5, 0.22, 0.16)
        top = _pile(F, rnd, 0.1, -0.15, h, 2, 0.2, 0.15)
        F.box('book_block', 0.19, 0.03, 0.145, (0.1, top + 0.015, -0.15), M.page)
        F.box('book_back', 0.2, 0.006, 0.15, (0.1, top + 0.003, -0.15), M.book_a)
        F.box('book_spine', 0.006, 0.042, 0.15, (0.2, top + 0.021, -0.15), M.book_a)
        _lamp(F, 0.28, h, -0.33, h=0.44)
        _mug(F, -0.3, h, -0.06, M.mug_a)
        for s in (-1, 1):                               # reading glasses folded on the top
            G.torus(_name('lens'), 0.02, 0.003, at=F.at(-0.07 + s * 0.024, h + 0.004, -0.05), material=M.dark,
                    major_segments=14, minor_segments=4)
        F.box('glasses_arm', 0.09, 0.004, 0.004, (-0.07, h + 0.004, -0.07), M.dark)
        _pile(F, rnd, 0.32, 0.18, 0.0, 6, 0.24, 0.18)    # a pile on the floor beside the chest
    G.join(still, 'body', origin=(0, 0, 0))
    # The top book's cover, hinged on the spine at its right: `book_up` opens it over to the right.
    with G.new_parts() as cover:
        F.box('cover', 0.2, 0.006, 0.15, (0.1, top + 0.033, -0.15), M.book_a)
        F.box('cover_title', 0.08, 0.002, 0.03, (0.1, top + 0.037, -0.15), M.book_d)
    cover = G.hinged(G.join(cover, 'cover'), F.at(0.2, top + 0.033, -0.15))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(cover, 'book_up', 170, seconds=0.8, axis='Y')
    return objs


FAMILY = G.Family(
    'pack_bookworm', M,
    [G.Model('readingNook', readingNook, size=(1.88, 1.71, 0.646), key='model.bookworm.readingNook',
             centre=(0, -0.107), motions={'uses': {'curlUp': 'sink_in'}},
             paint=[('body', 'Wood'), ('fabric', 'Fabric'), ('cushion', 'Cushions')]),
     G.Model('libraryWall', libraryWall, size=(1.88, 2.34, 0.82), key='model.bookworm.libraryWall',
             centre=(0, -0.02), motions={'uses': {'research': 'ladder', 'browse': 'ladder'}},
             paint=[('body', 'Wood'), ('honey', 'Ladder')]),
     G.Model('writingBureau', writingBureau, size=(0.852, 1.35, 0.758), key='model.bookworm.writingBureau',
             centre=(0.006, -0.051), motions={'uses': {'draft': 'fall_front', 'letter': 'fall_front'}},
             paint=[('body', 'Wood'), ('seatpad', 'Fabric'), ('honey', 'Chair')]),
     G.Model('teaTrolley', teaTrolley, size=(0.746, 0.939, 0.492), key='model.bookworm.teaTrolley',
             centre=(0, -0.184), motions={'uses': {'brew': 'pour'}, 'hold': 1.5},
             paint=[('honey', 'Wood'), ('linen', 'Fabric'), ('brass', 'Frame')]),
     G.Model('firesideLounge', firesideLounge, size=(1.7, 1.96, 1.85), key='model.bookworm.firesideLounge',
             centre=(0, -0.005), motions={'uses': {'readByFire': 'flames', 'readAloud': 'flames'}},
             paint=[('walnut', 'Wood'), ('velvet', 'Fabric'), ('rug', 'Rug')]),
     G.Model('bedsideStack', bedsideStack, size=(0.846, 0.931, 0.74), key='model.bookworm.bedsideStack',
             centre=(0.023, -0.06), motions={'uses': {'chapter': 'book_up'}},
             paint=[('body', 'Wood'), ('shade', 'Fabric'), ('panel', 'Drawers')])],
    comment='Blender-built bookworm pack models (tools/art/blender/families/pack_bookworm.py). Replaces the '
            'primitive-part placeholders of packs/bookworm by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. Each has one moving part its use plays: the nook\'s cushion sinks, the ladder rolls, '
            'the bureau\'s flap drops, the teapot pours, the fire grows and the top book opens.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
