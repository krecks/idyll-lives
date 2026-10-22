# The bookworm pack (P1): the window-seat reading nook, the rare-books library wall, the writer's bureau, the
# book-club tea trolley, the fireside reading lounge and the bedside book stack.
# Notes: pack_bookworm.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the nook's seat cushion sinks (`sink_in`), the library ladder rolls along
# its rail (`ladder`), the bureau's writing flap drops (`fall_front`), the teapot pours (`pour`), the fire
# grows in the grate (`flames`) and the top book of the bedside stack opens (`book_up`).
#
# Built with the forms and the kit (README "The toolkit"): books whose page edges show cream on the same box,
# frame-and-panel fronts (`kit.shaker`), turned knobs, mouldings run round the cornices, stuffed cushions
# (`F.soft`), bent brass and iron in one sweep, lofted china.
#
# Run:  cargo art pack_bookworm [--only readingNook] [--preview]
import math
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
    # added with the detail pass
    land=('#C9B47A', 'satin'),       # the globe's continents
    log_end=('#B89270', 'satin'),    # the logs' cut ends
    tea=('#7A4A26', 'gloss'),        # tea in the cups
    ink=('#1E2A3A', 'gloss'),
)

BOOKS = ('book_a', 'book_b', 'book_c', 'book_d', 'book_e', 'book_f', 'book_g')   # by name: made anew after each reset
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


# --- books, china and lamps, shared by the six -------------------------------------------------------------------
def _pages(obj, lying=False):
    """Gives a book's box its page edges in cream on the same mesh (no more triangles): the top of a standing book,
    the three sides but the spine (its front) of one lying down."""
    obj.data.materials.append(M.page)
    for p in obj.data.polygons:
        n = p.normal
        if (abs(n.z) < 0.5 and n.y > -0.5) if lying else n.z > 0.5:
            p.material_index = 1
    return obj


def _book(F, name, w, h, d, at, cover, rot=(0, 0, 0), lying=False):
    return _pages(F.box(name, w, h, d, at, cover, rot=rot), lying)


def _books(F, rnd, x0, x1, y, z_back, depth, h_max, fill=0.92, stack=False, lean=False):
    """A row of books standing on `y` from `x0` towards `x1` (game), spines to the front, backs on `z_back`;
    the gap left at the end takes a short pile lying flat (`stack`) or a book leaning on the row (`lean`)."""
    x, end = x0 + 0.004, x0 + (x1 - x0) * fill
    while x < end:
        t = rnd.uniform(0.022, 0.05)
        h = h_max * rnd.uniform(0.66, 1.0)
        d = depth * rnd.uniform(0.78, 1.0)
        _book(F, _name('book'), t, h, d, (x + t / 2, y + h / 2, z_back + d / 2), getattr(M, rnd.choice(BOOKS)))
        x += t + 0.002
    if stack and x1 - x > 0.16:
        w = min(0.2, x1 - x - 0.02)
        _pile(F, rnd, x + 0.01 + w / 2, z_back + depth * 0.45, y, 3, w, depth * 0.85)
    elif lean and x1 - x > 0.08:
        # a book leaning on the row, on its bottom edge nearest it, its top against the last one
        h, t = h_max * 0.9, 0.03
        a = math.asin(min(0.45, (x1 - x - t - 0.008) / h))
        px = x + h * math.sin(a)
        c = (px + t / 2 * math.cos(a) - h / 2 * math.sin(a), y + t / 2 * math.sin(a) + h / 2 * math.cos(a))
        _book(F, _name('book'), t, h, depth * 0.9, (c[0], c[1], z_back + depth * 0.45), getattr(M, rnd.choice(BOOKS)),
              rot=(0, math.degrees(a), 0))


def _pile(F, rnd, x, z, y, n, w=0.2, d=0.15):
    """`n` books lying in a pile on `y`, centred on (x, z); returns the height of its top."""
    for _ in range(n):
        h = rnd.uniform(0.025, 0.045)
        _book(F, _name('pile'), w * rnd.uniform(0.85, 1.05), h, d * rnd.uniform(0.9, 1.05),
              (x + rnd.uniform(-0.01, 0.01), y + h / 2, z), getattr(M, rnd.choice(BOOKS)), rot=(0, 0, rnd.uniform(-10, 10)),
              lying=True)
        y += h
    return y


def _mug(F, x, y, z, material, fill=None):
    """A mug standing on `y`: a turned body with its lip and a hollow, tea in it, a handle in one bent piece."""
    G.revolve(_name('mug'), [(0, 0), (0.036, 0), (0.04, 0.006), (0.042, 0.088), (0.039, 0.092), (0.035, 0.088),
                             (0.033, 0.07), (0, 0.07)], at=F.at(x, y, z), material=material, segments=14)
    F.cyl(_name('mug_tea'), 0.033, 0.002, (x, y + 0.072, z), fill or M.tea, verts=12)
    F.sweep(_name('mug_handle'), G.kit.fillet([(x + 0.036, y + 0.074, z), (x + 0.066, y + 0.07, z), (x + 0.064, y + 0.028, z),
                                               (x + 0.038, y + 0.022, z)], 0.012), 0.0065, material, verts=6)


def _lamp(F, x, y, z, h=0.42, shade=None):
    """A brass table lamp standing on `y`: a stepped round foot, a stem with a collar, a socket, a drum shade open
    top and bottom with rolled brass rims, a finial over it."""
    shade = shade or M.shade
    G.revolve(_name('lamp_foot'), [(0, 0), (0.07, 0), (0.07, 0.008), (0.062, 0.014), (0.05, 0.02), (0.03, 0.03),
                                   (0.02, 0.04), (0.012, 0.045), (0, 0.045)], at=F.at(x, y, z), material=M.brass,
              segments=14)
    G.kit.tube(F, _name('lamp_stem'), [(x, y + 0.04, z), (x, y + h * 0.62, z)], 0.008, M.brass, verts=8, round=1)
    G.revolve(_name('lamp_collar'), [(0, 0), (0.014, 0), (0.017, 0.008), (0.014, 0.016), (0, 0.016)],
              at=F.at(x, y + h * 0.3, z), material=M.brass, segments=10)
    G.revolve(_name('lamp_socket'), [(0, 0), (0.018, 0), (0.018, 0.05), (0.012, 0.06), (0, 0.06)],
              at=F.at(x, y + h * 0.62, z), material=M.brass, segments=10)
    rb, rt, sy, sh = 0.3 * h, 0.2 * h, y + h * 0.56, h * 0.42
    G.revolve(_name('lamp_shade'), [(rb, 0), (rt, sh), (rt - 0.004, sh), (rb - 0.004, 0), (rb, 0)],
              at=F.at(x, sy, z), material=shade, segments=24)
    for r, yy in ((rb - 0.002, sy + 0.003), (rt - 0.002, sy + sh - 0.003)):
        G.torus(_name('lamp_rim'), r, 0.0035, at=F.at(x, yy, z), material=M.brass, major_segments=24, minor_segments=3)
    G.revolve(_name('lamp_finial'), [(0, 0), (0.006, 0), (0.011, 0.01), (0.006, 0.02), (0, 0.022)],
              at=F.at(x, sy + sh, z), material=M.brass, segments=8)


def _knob(F, name, x, y, face, r, material=None, segments=8):
    """A small turned knob on a front at game depth `face` (kit.knob's shape in fewer rings: these are many)."""
    prof = [(0, 0), (r * 0.7, 0), (r * 0.38, r * 0.45), (r * 0.9, r * 0.95), (r, r * 1.3), (r * 0.6, r * 1.75), (0, r * 1.85)]
    return G.revolve(name, prof, at=F.at(x, y, face), material=material or M.brass, segments=segments, rot=(90, 0, 0))


# --- the window-seat reading nook (2x1): a cushioned seat between two little bookcases ---------------------
def _case(F, rnd, x, top, shelves, fills, stacks, lean=()):
    """A little open bookcase centred on `x`, its back on the wall: sides with a rounded front edge, a back,
    shelves with a lipped front, books on each."""
    for dx in (-0.195, 0.195):
        F.box(_name('side'), 0.03, top, 0.4, (x + dx, top / 2, WALL + 0.2), M.body, bevel=0.006, bevel_segments=1)
    F.box(_name('back'), 0.36, top - 0.06, 0.012, (x, top / 2, WALL + 0.006), M.back)
    for k, y in enumerate(shelves):
        F.box(_name('shelf'), 0.36, 0.025, 0.38, (x, y, WALL + 0.2), M.body, bevel=0.004, bevel_segments=1)
        F.box(_name('shelf_lip'), 0.36, 0.035, 0.012, (x, y + 0.005, WALL + 0.392), M.panel)
        if k < len(fills):
            _books(F, rnd, x - 0.175, x + 0.175, y + 0.0125, WALL + 0.02, 0.24, 0.3 if k < 3 else 0.22,
                   fill=fills[k], stack=k in stacks, lean=k in lean)


def readingNook():
    F = G.GameFrame()
    rnd = random.Random(3)
    with G.new_parts() as still:
        for s in (-1, 1):
            _case(F, rnd, s * 0.73, 1.53, (0.04, 0.42, 0.8, 1.18), (0.9, 0.62, 0.95, 0.5), (1, 3), lean=(2,) if s > 0 else ())
            F.box(_name('case_kick'), 0.36, 0.035, 0.012, (s * 0.73, 0.0175, WALL + 0.39), M.panel)
        _mug(F, 0.64, 0.4325, -0.15, M.mug_a)
        # the header over the seat and cases: a frieze, and a cornice moulding run round its front and ends
        F.box('header', 1.82, 0.12, 0.38, (0, 1.59, WALL + 0.19), M.body, bevel=0.006, bevel_segments=1)
        G.kit.moulding(F, 'cornice', [(-0.91, WALL + 0.03), (0.91, WALL + 0.03), (0.91, WALL + 0.39), (-0.91, WALL + 0.39)],
                       [(0.02, 0.0), (-0.004, 0.0), (-0.004, 0.018), (-0.016, 0.03), (-0.024, 0.05), (-0.03, 0.066),
                        (-0.03, 0.09), (0.02, 0.09)], M.body, at=1.62, plane='top')
        F.box('header_bead', 1.82, 0.016, 0.01, (0, 1.545, WALL + 0.385), M.panel, bevel=0.004, bevel_segments=1)
        # the alcove's ceiling and its valance: a board with an arched, stepped lower edge
        F.box('alcove_top', 1.06, 0.02, 0.4, (0, 1.52, WALL + 0.2), M.body)
        edge = [(-0.53 + 1.06 * i / 16, 0.0) for i in range(17)]
        edge = [(x, 1.4 + 0.07 * (1 - (x / 0.53) ** 2) ** 0.6) for x, _ in edge]
        F.extrude('valance', [(0.53, 1.53), (-0.53, 1.53)] + edge, 0.025, (0, 0, -0.045), M.body, bevel=0.004,
                  bevel_segments=1)
        F.sweep('valance_bead', [(x, y - 0.006, -0.03) for x, y in edge[1:-1]], 0.006, M.panel, verts=4)
        # a pendant reading light: a cord from a rose to a domed brass shade with a rolled rim and its bulb
        F.cyl('pendant_rose', 0.035, 0.012, (0, 1.504, -0.12), M.brass, verts=12, bevel=0.003, bevel_segments=1)
        G.kit.tube(F, 'pendant_cord', [(0, 1.5, -0.12), (0, 1.27, -0.12)], 0.003, M.dark, verts=5, round=1)
        G.revolve('pendant', [(0.11, 0), (0.105, 0.012), (0.08, 0.05), (0.045, 0.085), (0.02, 0.105), (0.012, 0.13),
                              (0, 0.13), (0, 0.12), (0.016, 0.1), (0.04, 0.08), (0.075, 0.045), (0.1, 0.008)],
                  at=F.at(0, 1.15, -0.12), material=M.lamp_amber, segments=16)
        G.torus('pendant_rim', 0.108, 0.005, at=F.at(0, 1.152, -0.12), material=M.brass, major_segments=20, minor_segments=4)
        F.sphere('pendant_bulb', 0.03, (0, 1.165, -0.12), M.page, segments=10, rings=6)
        # the back of the seat: two panels (mostly behind the cushions)
        F.box('seat_back', 1.04, 1.02, 0.02, (0, 0.96, WALL + 0.01), M.back)
        for x in (-0.26, 0.26):
            F.box(_name('seat_back_panel'), 0.44, 0.5, 0.012, (x, 1.12, WALL + 0.026), M.panel, bevel=0.004,
                  bevel_segments=1)
        # the seat box with two frame-and-panel drawers and turned knobs, on a recessed kick
        F.box('seat_box', 1.04, 0.33, 0.6, (0, 0.195, WALL + 0.3), M.body, bevel=0.008, bevel_segments=1)
        F.box('seat_lip', 1.06, 0.03, 0.62, (0, 0.345, WALL + 0.31), M.body, bevel=0.008, bevel_segments=1)
        for x in (-0.25, 0.25):
            G.kit.shaker(F, _name('drawer'), 0.46, 0.22, (x, 0.2, 0.18), 0.02, M.panel, rail=0.045)
            _knob(F, _name('knob'), x, 0.2, 0.19, 0.013)
        F.box('plinth', 1.0, 0.03, 0.02, (0, 0.015, 0.155), M.panel, bevel=0.004, bevel_segments=1)
        # back cushions, stuffed and piped, and two throw pillows
        for x in (-0.25, 0.25):
            F.soft(_name('back_cushion'), 0.48, 0.42, 0.13, (x, 0.68, -0.33), M.cushion, rot=(-8, 0, 0), round=0.045,
                   puff=0.03, piping=0.005, flat=3)
        F.soft('pillow_l', 0.3, 0.28, 0.1, (-0.38, 0.6, -0.22), M.pillow_a, rot=(-12, -6, 30), round=0.04, puff=0.035,
               flat=2)
        F.soft('pillow_r', 0.28, 0.26, 0.1, (0.38, 0.6, -0.23), M.pillow_b, rot=(-12, 6, -28), round=0.04, puff=0.035,
               flat=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushion and the throw folded over it, joined with their origin on the underside: `sink_in`
    # presses them 2 cm while a resident is curled up on it.
    with G.new_parts() as seat:
        F.soft('seat_cushion', 1.02, 0.078, 0.6, (0, 0.399, -0.125), M.fabric, round=0.03, puff=0.012, piping=0.005,
               flat=3)
        t, z0, z1, x = 0.012, -0.4, 0.172, 0.28
        y = 0.45 + t / 2
        pts = [(x, y, z0), (x, y + 0.004, (z0 + z1) / 2), (x, y + 0.002, z1 - 0.03), (x, y - 0.012, z1 + 0.002),
               (x, y - 0.04, z1 + t / 2 + 0.006), (x, y - 0.15, z1 + t / 2 + 0.008)]
        throw = F.sweep('throw', pts, None, M.throw, profile=G.rounded_rect(0.42, t, t / 2, corner=1), up=(1, 0, 0),
                        resolution=3)
        G.jitter(throw, 0.004, size=0.12, seed=2.0, up=0.6)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 0.36, -0.125))
    objs = G.settle(centre=False)
    G.squash(seat, 'sink_in', (1, 1, 0.78), seconds=0.6)
    return objs


# --- the rare-books library wall (2x1): floor-to-ceiling shelves, a brass rail and a rolling ladder ---------
LADDER_TOP, LADDER_FOOT = (1.98, -0.02), (0.02, 0.36)   # (height, depth) of the rails' top and foot


def _ladder_at(t):
    """(height, depth) a fraction `t` down the ladder from its top."""
    return (LADDER_TOP[0] + (LADDER_FOOT[0] - LADDER_TOP[0]) * t, LADDER_TOP[1] + (LADDER_FOOT[1] - LADDER_TOP[1]) * t)


def _bust(F, x, y, z, s=1.0):
    """A little marble bust on `y`: a turned socle, sloping shoulders, a neck and a head with hair."""
    bx, by, bz = F.at(x, y, z)
    G.revolve(_name('bust_socle'), [(0, 0), (0.035 * s, 0), (0.035 * s, 0.008 * s), (0.022 * s, 0.02 * s),
                                    (0.026 * s, 0.03 * s), (0, 0.03 * s)], at=(bx, by, bz), material=M.bust, segments=12)
    G.loft(_name('bust_chest'), [G.ring(rx * s, ry * s, bz + h * s, 14, square=sq, at=(bx, by)) for h, rx, ry, sq in (
        (0.028, 0.03, 0.024, 2), (0.04, 0.06, 0.03, 3), (0.08, 0.065, 0.034, 3.4), (0.1, 0.056, 0.03, 2.8),
        (0.11, 0.03, 0.022, 2), (0.112, 0.02, 0.018, 2))], material=M.bust)
    G.loft(_name('bust_head'), [G.ring(rx * s, ry * s, bz + h * s, 12, square=2.2, at=(bx, by + cy * s)) for h, rx, ry, cy in (
        (0.11, 0.016, 0.017, 0.0), (0.13, 0.017, 0.018, -0.002), (0.14, 0.026, 0.03, -0.006), (0.16, 0.03, 0.036, -0.002),
        (0.18, 0.03, 0.035, 0.0), (0.195, 0.022, 0.026, 0.002), (0.202, 0.004, 0.004, 0.002))], material=M.bust)
    G.jitter(G.boulder(_name('bust_hair'), 0.034 * s, at=(bx, by + 0.006 * s, bz + 0.172 * s), material=M.bust,
                       scale=(0.95, 1.05, 0.85), seed=2.0, noise=0.15, subdivisions=2), 0.002 * s, size=0.01, seed=1.0)


def _globe(F, x, y, z):
    """A desk globe on `y`: a turned foot and stem, the ball tilted on its axis, land on it, a brass half meridian."""
    G.revolve('globe_foot', [(0, 0), (0.06, 0), (0.06, 0.008), (0.045, 0.016), (0.014, 0.03), (0.01, 0.06), (0, 0.06)],
              at=F.at(x, y, z), material=M.brass, segments=16)
    c = (x, y + 0.17, z)
    F.sphere('globe', 0.1, c, M.globe, segments=20, rings=12)
    for k, (a, b, r) in enumerate(((30, 20, 0.04), (160, -15, 0.035), (250, 35, 0.03))):
        F.sphere(_name('land'), r, (c[0] + 0.088 * G.cosd(a) * G.cosd(b), c[1] + 0.088 * G.sind(b),
                                    c[2] + 0.088 * G.sind(a) * G.cosd(b)), M.land, scale=(1, 0.45, 1.3),
                 rot=(b, 0, -a), segments=8, rings=5)
    tilt = 23.5
    pts = [(c[0] + 0.115 * math.sin(math.radians(a + tilt)), c[1] + 0.115 * math.cos(math.radians(a + tilt)), c[2])
           for a in range(-180, 1, 15)]
    F.sweep('globe_meridian', pts, None, M.brass, profile=G.rounded_rect(0.012, 0.004, 0.0015, corner=1), up=(0, 1, 0))


def libraryWall():
    F = G.GameFrame()
    rnd = random.Random(11)
    with G.new_parts() as still:
        # the cornice: a frieze under a crown moulding run round its front and ends; a moulded plinth
        F.box('frieze', 1.82, 0.1, 0.38, (0, 2.22, WALL + 0.19), M.panel, bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'cornice', [(-0.91, WALL + 0.03), (0.91, WALL + 0.03), (0.91, WALL + 0.39), (-0.91, WALL + 0.39)],
                       [(0.03, 0.0), (-0.004, 0.0), (-0.004, 0.012), (-0.012, 0.022), (-0.022, 0.04), (-0.03, 0.058),
                        (-0.03, 0.07), (-0.022, 0.074), (-0.022, 0.08), (0.03, 0.08)], M.body, at=2.26, plane='top')
        F.box('plinth', 1.84, 0.1, 0.37, (0, 0.05, WALL + 0.185), M.body)
        G.kit.moulding(F, 'plinth_mould', [(-0.91, WALL + 0.012), (0.91, WALL + 0.012), (0.91, WALL + 0.38),
                                           (-0.91, WALL + 0.38)],
                       [(0.01, 0.0), (-0.012, 0.0), (-0.012, 0.09), (-0.006, 0.1), (0.0, 0.11), (0.01, 0.11)], M.body,
                       at=0.0, plane='top')
        F.box('back', 1.8, 2.16, 0.012, (0, 1.18, WALL + 0.006), M.back)
        for x in (-0.91, -0.3, 0.3, 0.91):
            F.box(_name('upright'), 0.04, 2.14, 0.38, (x, 1.17, WALL + 0.19), M.body, bevel=0.006, bevel_segments=1)
            F.box(_name('upright_bead'), 0.024, 2.1, 0.008, (x, 1.17, WALL + 0.383), M.panel, bevel=0.003,
                  bevel_segments=1)
        ys = (0.1, 0.52, 0.92, 1.32, 1.72, 2.12)
        for y in ys:
            F.box(_name('shelf'), 1.8, 0.03, 0.37, (0, y, WALL + 0.19), M.body, bevel=0.004, bevel_segments=1)
            F.box(_name('shelf_edge'), 1.8, 0.04, 0.012, (0, y - 0.004, WALL + 0.374), M.panel, bevel=0.003,
                  bevel_segments=1)
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
                _books(F, rnd, a, b, y + 0.015, WALL + 0.02, 0.28, 0.34, fill=(0.95, 0.7, 0.85, 0.55)[k], stack=k == 3,
                       lean=k == 1)
        # a globe on its stand and a marble bust
        _globe(F, 0.72, 1.335, -0.25)
        F.box('bust_plinth', 0.12, 0.06, 0.12, (-0.8, 0.98, -0.28), M.bust, bevel=0.008, bevel_segments=1)
        _bust(F, -0.8, 1.01, -0.28, 1.1)
        # the brass rail the ladder hangs on, turned finials at its ends, on brackets from round wall plates
        G.kit.tube(F, 'rail', [(-0.9, LADDER_TOP[0], LADDER_TOP[1]), (0.9, LADDER_TOP[0], LADDER_TOP[1])], 0.0125, M.brass,
                   verts=10, round=1)
        for s in (-1, 1):
            G.revolve(_name('rail_finial'), [(0, 0), (0.016, 0), (0.02, 0.008), (0.012, 0.018), (0.008, 0.024), (0, 0.026)],
                      at=F.at(s * 0.9, LADDER_TOP[0], LADDER_TOP[1]), material=M.brass, segments=10,
                      rot=(0, s * 90, 0))
        for x in (-0.6, 0.0, 0.6):
            G.kit.tube(F, _name('bracket'), [(x, LADDER_TOP[0], WALL + 0.38), (x, LADDER_TOP[0], LADDER_TOP[1])], 0.008,
                       M.brass, verts=6, round=1)
            F.cyl(_name('bracket_rose'), 0.022, 0.01, (x, LADDER_TOP[0], WALL + 0.385), M.brass, rot=(90, 0, 0), verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rolling ladder: two stiles, seven treads, hooks over the rail and wheels in forks at its feet, all wood (a
    # metal on it would cost the ladder a second draw).
    with G.new_parts() as ladder:
        for x in (0.15, 0.55):
            top, foot = _ladder_at(0.015), (LADDER_FOOT[0] + 0.05, LADDER_FOOT[1] - 0.008)
            F.sweep(_name('stile'), [(x, foot[0], foot[1]), (x, top[0], top[1])], None, M.honey,
                    profile=G.rounded_rect(0.05, 0.032, 0.008, corner=1), up=(1, 0, 0))
            hook = G.kit.fillet([(x, 1.93, 0.0), (x, 2.0, -0.002), (x, 2.004, -0.032), (x, 1.975, -0.036)], 0.012)
            G.kit.tube(F, _name('hook'), hook, 0.007, M.body, verts=6, round=1)
            F.box(_name('fork'), 0.04, 0.05, 0.03, (x, 0.055, LADDER_FOOT[1]), M.body, bevel=0.006, bevel_segments=1)
            F.cyl(_name('wheel'), 0.03, 0.02, (x, 0.03, LADDER_FOOT[1]), M.body, rot=(0, 90, 0), verts=14, bevel=0.004,
                  bevel_segments=1)
        for k in range(1, 8):
            y, z = _ladder_at(k / 8)
            F.box(_name('tread'), 0.38, 0.025, 0.08, (0.35, y, z), M.honey, bevel=0.006, bevel_segments=1)
            if k in (2, 6):
                G.kit.tube(F, _name('tie'), [(0.15, y - 0.03, z + 0.005), (0.55, y - 0.03, z + 0.005)], 0.005, M.body,
                           verts=5, round=1)
    ladder = G.join(ladder, 'ladder')
    objs = G.settle(centre=False)
    G.slide(ladder, 'ladder', (0.4, 0, 0), seconds=1.2)    # 40 cm along the rail, towards the middle
    return objs


# --- the writer's bureau (1x1): its back on the wall, the chair in front; the writer faces the bureau ------
# (content `seat.facing: "back"`). Built facing the front as it stands (2026-10-22: it was built the other way
# round and turned, `_turn_round`; the flap's clip is the same).
def _bureau_chair(F, z):
    """The writer's chair, its seat (pad top 0.46) centred at depth `z`, its back towards the front: raked back legs
    in one curve each, turned front legs, a bowed crest rail over a vase-shaped splat, a stuffed pad."""
    zb = z + 0.17                                    # the back legs' foot
    for s in (-1, 1):
        x = s * 0.18
        F.sweep(_name('back_leg'), [(x, 0.0, zb), (x, 0.42, zb - 0.01), (x, 0.7, zb), (x, 0.93, zb + 0.025)],
                None, M.honey, profile=G.rounded_rect(0.034, 0.03, 0.008, corner=1), up=(1, 0, 0), resolution=2)
        G.revolve(_name('front_leg'), [(0, 0), (0.013, 0), (0.016, 0.02), (0.014, 0.05), (0.019, 0.2), (0.016, 0.33),
                                       (0.02, 0.345), (0.02, 0.39), (0, 0.39)], at=F.at(x, 0, z - 0.16),
                  material=M.honey, segments=10)
        G.kit.tube(F, _name('side_stretcher'), [(x, 0.14, z - 0.16), (x, 0.14, zb)], 0.009, M.honey, verts=6, round=1)
    G.kit.tube(F, 'front_stretcher', [(-0.18, 0.18, z - 0.16), (0.18, 0.18, z - 0.16)], 0.009, M.honey, verts=6, round=1)
    # the seat: a board with a rounded front, the pad on it
    seat = [(0.21, 0.19), (-0.21, 0.19)] + [(-0.21 + 0.42 * i / 8, -0.19 - 0.02 * math.sin(math.pi * i / 8)) for i in range(9)]
    F.extrude('chair_seat', G.smooth_outline(seat, 1), 0.035, (0, 0.41, z), M.honey, plane='top', bevel=0.008,
              bevel_segments=1)
    F.soft('chair_pad', 0.38, 0.032, 0.36, (0, 0.436, z - 0.01), M.seatpad, round=0.018, puff=0.008, piping=0.004, flat=2,
           round_segments=1)
    # the crest rail, bowed back between the legs' tops, and the splat under it
    F.sweep('chair_crest', [(-0.2, 0.86, zb + 0.02), (-0.1, 0.875, zb + 0.028), (0, 0.88, zb + 0.032), (0.1, 0.875, zb + 0.028),
                            (0.2, 0.86, zb + 0.02)], None, M.honey, profile=G.rounded_rect(0.085, 0.024, 0.01, corner=1),
            up=(0, 0, 1), resolution=2)
    F.box('chair_rail', 0.36, 0.035, 0.025, (0, 0.5, zb - 0.003), M.honey, bevel=0.008, bevel_segments=1)
    splat = [(0.04, 0.0), (0.06, 0.06), (0.075, 0.14), (0.05, 0.22), (0.035, 0.27), (0.07, 0.31), (0.08, 0.34)]
    outline = splat + [(-u, v) for u, v in reversed(splat)]
    F.extrude('chair_splat', G.smooth_outline(outline, 1), 0.016, (0, 0.5, zb + 0.012), M.honey, rot=(-4, 0, 0),
              bevel=0.003, bevel_segments=1)


def writingBureau():
    F = G.GameFrame()
    rnd = random.Random(5)
    with G.new_parts() as still:
        # two pedestals of drawers facing the writer, a kneehole between them, on a plinth each
        for s in (-1, 1):
            x = s * 0.31
            F.box(_name('pedestal'), 0.2, 0.69, 0.28, (x, 0.385, -0.3), M.body, bevel=0.006, bevel_segments=1)
            F.box(_name('pedestal_plinth'), 0.21, 0.04, 0.29, (x, 0.02, -0.3), M.panel, bevel=0.006, bevel_segments=1)
            for y, hh in ((0.17, 0.2), (0.4, 0.2), (0.6, 0.16)):
                G.kit.shaker(F, _name('drawer'), 0.172, hh, (x, y, -0.152), 0.016, M.panel, rail=0.032)
                _knob(F, _name('knob'), x, y, -0.144, 0.011)
        F.box('modesty', 0.42, 0.6, 0.015, (0, 0.42, -0.43), M.body, bevel=0.004, bevel_segments=1)
        F.box('modesty_panel', 0.34, 0.46, 0.008, (0, 0.42, -0.418), M.panel, bevel=0.003, bevel_segments=1)
        # the desk top, its edge moulded
        F.extrude('top', G.rounded_rect(0.84, 0.32, 0.02, corner=2), 0.04, (0, 0.75, -0.29), M.body, plane='top',
                  bevel=0.01, bevel_segments=2)
        # the hutch: a back, sides, a shelf of books over the pigeonholes, a moulded top and an arched crest
        F.box('hutch_back', 0.8, 0.52, 0.02, (0, 1.03, -0.43), M.body)
        for x in (-0.19, 0.19):                            # panels on its back (on the wall; out in a room, it shows)
            F.box(_name('hutch_panel'), 0.32, 0.4, 0.008, (x, 1.03, -0.442), M.panel, bevel=0.003, bevel_segments=1)
        for x in (-0.4, 0.4):
            F.box(_name('hutch_side'), 0.022, 0.52, 0.17, (x, 1.03, -0.355), M.body, bevel=0.004, bevel_segments=1)
        F.box('hutch_shelf', 0.78, 0.02, 0.16, (0, 0.99, -0.355), M.body)
        F.extrude('hutch_top', G.rounded_rect(0.84, 0.2, 0.012, corner=2), 0.03, (0, 1.305, -0.35), M.body, plane='top',
                  bevel=0.008, bevel_segments=2)
        crest = [(-0.38 + 0.76 * (1 - i / 20), 1.318 + 0.032 * math.sin(math.pi * (1 - i / 20)) ** 0.7) for i in range(21)]
        F.extrude('crest', crest, 0.025, (0, 0, -0.42), M.panel, bevel=0.004, bevel_segments=1)
        for x in (-0.125, 0.0, 0.125):
            F.box(_name('divider'), 0.012, 0.2, 0.15, (x, 0.875, -0.35), M.body)
        for x, m, r in ((0.19, M.page, 6), (0.06, M.cushion, -4), (-0.19, M.shade, 3)):
            F.box(_name('letters'), 0.1, 0.09, 0.08, (x, 0.82, -0.35), m, rot=(0, r, 0))
            F.box(_name('envelope'), 0.1, 0.004, 0.06, (x, 0.869, -0.33), M.page, rot=(-60, r, 0))
        for x in (-0.32, 0.32):                            # small drawers either side of the flap
            F.box(_name('little_case'), 0.13, 0.2, 0.15, (x, 0.88, -0.355), M.body)
            for y in (0.83, 0.93):
                F.box(_name('little_drawer'), 0.124, 0.08, 0.012, (x, y, -0.27), M.panel, bevel=0.003, bevel_segments=1)
                _knob(F, _name('knob'), x, y, -0.264, 0.007, segments=6)
        _books(F, rnd, -0.2, 0.38, 1.0, -0.42, 0.12, 0.2, fill=0.95)
        # an inkwell with its quill, a lamp and a mug of tea
        G.revolve('inkwell', [(0, 0), (0.032, 0), (0.034, 0.03), (0.022, 0.045), (0.014, 0.05), (0, 0.05)],
                  at=F.at(-0.3, 0.77, -0.22), material=M.ink, segments=12)
        F.cyl('ink_cap', 0.016, 0.012, (-0.3, 0.826, -0.22), M.brass, verts=10)
        F.sweep('quill', [(-0.29, 0.81, -0.22), (-0.27, 0.92, -0.23), (-0.24, 1.0, -0.245)], None, M.page,
                profile=G.rounded_rect(0.004, 0.03, 0.0015, corner=1), up=(0, 1, 0), resolution=2, scale=[0.15, 1.0, 0.3])
        _lamp(F, 0.33, 0.77, -0.21, h=0.34, shade=M.lamp_amber)
        _mug(F, -0.18, 0.77, -0.17, M.mug_b)
        _bureau_chair(F, 0.092)
    G.join(still, 'body', origin=(0, 0, 0))
    # The writing flap, closed up over the pigeonholes, hinged on its foot: `fall_front` drops it onto the desk.
    with G.new_parts() as flap:
        G.kit.shaker(F, 'flap', 0.5, 0.2, (0, 0.875, -0.265), 0.02, M.panel, rail=0.035, raised=True)
        F.box('flap_lining', 0.44, 0.15, 0.004, (0, 0.875, -0.277), M.seatpad)
        G.kit.pull(F, 'flap_pull', 0, 0.94, -0.255, 0.06, M.brass, r=0.004, out=0.016)
        F.cyl('flap_key', 0.008, 0.004, (0, 0.9, -0.254), M.brass, rot=(90, 0, 0), verts=8)
    flap = G.hinged(G.join(flap, 'flap'), F.at(0, 0.775, -0.265))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(flap, 'fall_front', 90, seconds=0.8, axis='X')
    return objs


# --- the book-club tea trolley (1x1): oak shelves on a brass frame, a tea set on a cloth ---------------------
def _band(obj, material, lo, hi):
    """Paints the faces of a turned part whose middles lie between heights `lo` and `hi` (its own, from its foot) in
    `material`: a glazed band on china, with no more triangles."""
    obj.data.materials.append(material)
    for p in obj.data.polygons:
        if lo <= p.center.z <= hi:
            p.material_index = len(obj.data.materials) - 1
    return obj


def _teapot(F, x, y, z):
    """The teapot, spout to the right (+x game): a lofted round body with a glazed band, a lid and knob, a spout
    rising in one tapering curve, a looped handle."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.05), (0.006, 0.058), (0.03, 0.074), (0.058, 0.08), (0.085, 0.072), (0.102, 0.056), (0.11, 0.042)]
    pot = G.loft(_name('pot'), [G.ring(r, r, bz + h, 18, at=(bx, by)) for h, r in rows], material=M.porcelain)
    _band(pot, M.glaze, 0.045, 0.072)
    G.revolve(_name('pot_lid'), [(0, 0), (0.045, 0), (0.045, 0.006), (0.032, 0.018), (0.012, 0.024), (0.008, 0.03),
                                 (0.012, 0.036), (0, 0.04)], at=(bx, by, bz + 0.104), material=M.porcelain, segments=14)
    F.sweep(_name('spout'), [(x + 0.07, y + 0.035, z), (x + 0.1, y + 0.06, z), (x + 0.118, y + 0.1, z),
                             (x + 0.135, y + 0.12, z)], 0.016, M.porcelain, verts=8, resolution=2, scale=[1.0, 0.75, 0.5, 0.45])
    F.sweep(_name('pot_handle'), [(x - 0.072, y + 0.09, z), (x - 0.11, y + 0.088, z), (x - 0.118, y + 0.055, z),
                                  (x - 0.096, y + 0.028, z), (x - 0.074, y + 0.026, z)], None, M.porcelain,
            profile=G.rounded_rect(0.016, 0.008, 0.003, corner=1), up=(0, 1, 0), resolution=2)


def _cup(F, x, y, z):
    """A teacup on its saucer, tea in it, a glazed band round its rim and an ear handle."""
    G.revolve(_name('saucer'), [(0, 0), (0.032, 0), (0.055, 0.008), (0.052, 0.011), (0.03, 0.006), (0, 0.006)],
              at=F.at(x, y, z), material=M.porcelain, segments=12)
    cup = G.revolve(_name('cup'), [(0, 0), (0.022, 0), (0.03, 0.012), (0.04, 0.04), (0.043, 0.05), (0.04, 0.05),
                                   (0.036, 0.04), (0.026, 0.014), (0, 0.014)], at=F.at(x, y + 0.006, z),
                    material=M.porcelain, segments=12)
    _band(cup, M.glaze, 0.036, 0.051)
    F.cyl(_name('cup_tea'), 0.036, 0.002, (x, y + 0.046, z), M.tea, verts=10)
    F.sweep(_name('cup_handle'), G.kit.fillet([(x + 0.04, y + 0.044, z), (x + 0.058, y + 0.042, z), (x + 0.056, y + 0.022, z),
                                               (x + 0.034, y + 0.02, z)], 0.008, steps=2), 0.004, M.porcelain, verts=5)


def teaTrolley():
    F = G.GameFrame()
    rnd = random.Random(7)
    zb, zf = -0.41, 0.01
    with G.new_parts() as still:
        for y in (0.32, 0.78):
            F.extrude(_name('shelf'), G.rounded_rect(0.72, 0.42, 0.025, corner=2), 0.03, (0, y, -0.2), M.honey, plane='top',
                      bevel=0.006, bevel_segments=1)
            for z in (zb, zf):                       # brass gallery rails along the long sides, on little spindles
                G.kit.tube(F, _name('gallery'), [(-0.36, y + 0.05, z), (0.36, y + 0.05, z)], 0.0055, M.brass, verts=6,
                           round=1)
                for x in (-0.18, 0.0, 0.18):
                    G.kit.tube(F, _name('spindle'), [(x, y + 0.015, z), (x, y + 0.05, z)], 0.003, M.brass, verts=4, round=1)
        # the frame: each end one bent tube, up from the castors and over (the right end's top the push handle)
        for x in (-0.36, 0.36):
            top = 0.86 if x < 0 else 0.92
            path = [(x, 0.06, zb), (x, top, zb), (x, top, zf), (x, 0.06, zf)]
            if x > 0:
                path = [(x, 0.06, zb), (x, 0.86, zb), (x + 0.0, top, zb + 0.04), (x, top, zf - 0.04), (x, 0.86, zf),
                        (x, 0.06, zf)]
            G.kit.tube(F, _name('frame'), G.kit.fillet(path, 0.04), 0.012, M.brass, verts=8, round=1)
            for z in (zb, zf):
                F.box(_name('fork'), 0.02, 0.03, 0.03, (x, 0.055, z), M.brass)
                F.cyl(_name('castor'), 0.034, 0.018, (x, 0.034, z), M.dark, rot=(0, 90, 0), verts=12)
        G.kit.tube(F, 'grip', [(0.36, 0.92, zb + 0.08), (0.36, 0.92, zf - 0.08)], 0.016, M.dark, verts=8, round=1)
        # the cloth on the top shelf, hanging over its front edge
        F.box('cloth', 0.6, 0.004, 0.42, (0, 0.797, -0.2), M.linen)
        F.sweep('cloth_drop', [(-0.3, 0.795, 0.012), (0.3, 0.795, 0.012)], None, M.linen,
                profile=[(0.004, 0.0), (0.004, -0.004), (-0.03, -0.008), (-0.07, -0.007), (-0.07, -0.003),
                         (-0.03, -0.004), (0.0, 0.0)], up=(0, 0, 1))
        _cup(F, 0.02, 0.799, -0.12)
        _cup(F, 0.2, 0.799, -0.27)
        jug = G.revolve('milk_jug', [(0, 0), (0.026, 0), (0.032, 0.03), (0.03, 0.055), (0.026, 0.07), (0.03, 0.08),
                                     (0.027, 0.08), (0.022, 0.07), (0, 0.07)], at=F.at(0.05, 0.799, -0.29),
                        material=M.porcelain, segments=12)
        _band(jug, M.glaze, 0.06, 0.081)
        F.sweep('jug_handle', G.kit.fillet([(0.08, 0.86, -0.29), (0.1, 0.855, -0.29), (0.096, 0.82, -0.29),
                                            (0.078, 0.815, -0.29)], 0.01), 0.004, M.porcelain, verts=5)
        bowl = G.revolve('sugar_bowl', [(0, 0), (0.026, 0), (0.04, 0.02), (0.04, 0.035), (0.035, 0.045), (0, 0.045)],
                         at=F.at(0.22, 0.799, -0.1), material=M.porcelain, segments=12)
        _band(bowl, M.glaze, 0.018, 0.036)
        G.revolve('sugar_knob', [(0, 0), (0.008, 0), (0.01, 0.008), (0, 0.016)], at=F.at(0.22, 0.844, -0.1),
                  material=M.glaze, segments=8)
        # below: the biscuit tin, a cake on its stand and two books
        F.cyl('tin', 0.1, 0.12, (-0.2, 0.395, -0.22), M.glaze, verts=18)
        F.cyl('tin_lid', 0.105, 0.02, (-0.2, 0.465, -0.22), M.brass, verts=18)
        G.revolve('cake_stand', [(0, 0), (0.05, 0), (0.05, 0.008), (0.018, 0.02), (0.016, 0.07), (0.12, 0.074),
                                 (0.122, 0.082), (0, 0.082)], at=F.at(0.14, 0.335, -0.2), material=M.porcelain, segments=16)
        # the cake, a slice cut from it, its icing running over the edge
        cake = [(0.0, 0.0)] + [(0.09 * G.cosd(a), 0.09 * G.sind(a)) for a in range(40, 361, 20)]
        F.extrude('cake', cake, 0.06, (0.14, 0.447, -0.2), M.cake, plane='top')
        F.extrude('icing', [(0.0, 0.0)] + [(0.092 * G.cosd(a), 0.092 * G.sind(a)) for a in range(40, 361, 20)], 0.012,
                  (0.14, 0.483, -0.2), M.icing, plane='top', bevel=0.004, bevel_segments=1)
        _pile(F, rnd, 0.0, -0.36, 0.335, 2, 0.18, 0.12)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as pot:
        _teapot(F, -0.17, 0.799, -0.2)
    # `pour` tips it towards its spout about the foot's edge under the spout.
    pot = G.hinged(G.join(pot, 'teapot'), F.at(-0.12, 0.799, -0.2))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(pot, 'pour', -30, seconds=0.8, axis='Y')
    return objs


# --- the fireside reading lounge (2x2): two wing chairs before an open fire grate under a hood ----------------
STOVE = (0.0, 0.62)   # the grate's middle (x, depth)
HOOD_Y = 0.6          # the hood's rim, right over the flames (they reach 0.47); 2026-10-21: 1.0, over the air (M-packs-10)


def _wing_chair(F, x, z=0.0):
    """A wing chair centred on (x, z), facing the front: a stuffed seat cushion (top 0.45) on a sprung base, a
    tall buttoned back, wings curving forward from it, arms rolled outward, short cabriole legs."""
    w, d = 0.66, 0.64
    z0 = z - d / 2
    F.soft(_name('chair_base'), w - 0.08, 0.2, d - 0.06, (x, 0.26, z + 0.01), M.velvet, round=0.04, flat=1,
           round_segments=1)
    F.soft(_name('chair_seat'), w - 0.24, 0.088, d - 0.2, (x, 0.394, z + 0.07), M.velvet, round=0.03, puff=0.012,
           piping=0.005, flat=2)
    F.soft(_name('chair_back'), w - 0.1, 0.72, 0.15, (x, 0.71, z0 + 0.1), M.velvet, rot=(7, 0, 0), round=0.06, puff=0.03,
           flat=2)
    for s in (-1, 1):
        ax = x + s * (w / 2 - 0.065)
        # the arm: a panel rising into a roll that leans out, swept front to back
        roll = [(0.56 + 0.06 * math.sin(math.radians(a)), s * 0.065 * math.cos(math.radians(a)) + s * 0.01)
                for a in range(-40, 230, 30)]
        section = [(0.3, s * -0.05), (0.3, s * 0.05)] + roll + [(0.5, s * -0.05)]
        F.sweep(_name('chair_arm'), [(ax, 0, z0 + 0.1), (ax, 0, z + d / 2 - 0.035)], None, M.velvet, profile=section,
                angle=50)
        F.cyl(_name('chair_scroll'), 0.06, 0.02, (ax + s * 0.01, 0.565, z + d / 2 - 0.03), M.velvet, rot=(90, 0, 0),
              verts=14)
        # the wing: a shaped panel from the back's top curving down to the arm
        wing = G.smooth_outline([(-0.3, 0.55), (-0.3, 1.03), (-0.2, 1.07), (-0.08, 1.02), (0.0, 0.9), (0.02, 0.74),
                                 (0.06, 0.6), (0.0, 0.56)], 1)
        F.extrude(_name('chair_wing'), [(z + u, v) for u, v in wing], 0.075, (x + s * (w / 2 - 0.065), 0, 0), M.velvet,
                  plane='side', rot=(0, 0, s * 6), bevel=0.025, bevel_segments=1)
        for lz, dz in ((z0 + 0.07, -1), (z + d / 2 - 0.07, 1)):
            G.kit.cabriole(F, _name('chair_leg'), ax, lz, 0.16, M.walnut, s * 0.012, dz * 0.012, r=0.02, verts=6)


def _log(F, name, a, b, r, seed, material=None):
    """A split log from game point `a` to `b`: a barked, slightly crooked round, its cut ends pale."""
    mid = tuple((a[i] + b[i]) / 2 + (0.012 if i == 1 else 0.0) for i in range(3))
    log = F.sweep(name, [a, mid, b], r, material or M.log, verts=8, resolution=2, scale=[1.0, 1.06, 0.94])
    G.jitter(log, r * 0.12, size=0.03, seed=seed)
    for k, p in enumerate((a, b)):
        q = tuple(p[i] + (p[i] - mid[i]) * 0.001 / max(1e-6, math.dist(p, mid)) for i in range(3))
        F.sweep(f'{name}_end{k}', [p, q], r * 0.88 * (0.94 if k else 1.0), M.log_end, verts=8)


def _flame(F, name, x, y, z, r, h, material, turn=0.0):
    """A tongue of flame standing on `y`: a body lofted from a round foot, swelling, then drawn up into a flickering
    tip that leans and twists."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.7), (0.15, 1.0), (0.4, 0.85), (0.65, 0.5), (0.85, 0.22), (1.0, 0.02)]
    secs = []
    for k, (t, s) in enumerate(rows):
        lean = 0.25 * r * t * t
        secs.append(G.ring(r * s, r * s * 0.7, bz + h * t, 8, at=(bx - lean * G.cosd(turn), by - lean * G.sind(turn))))
    return G.loft(name, secs, material=material)


def firesideLounge():
    F = G.GameFrame()
    rnd = random.Random(5)
    sx, sz = STOVE
    with G.new_parts() as still:
        # an oval rug: its border, a field and a medallion in the middle
        F.cyl('rug', 0.85, 0.012, (0, 0.006, 0.2), M.rug, verts=40, scale=(1, 0.76, 1), bevel=0.004, bevel_segments=1)
        F.cyl('rug_field', 0.75, 0.014, (0, 0.007, 0.2), M.rug_inner, verts=40, scale=(1, 0.75, 1))
        F.cyl('rug_medallion', 0.3, 0.016, (0, 0.008, 0.2), M.rug, verts=24, scale=(1, 0.7, 1))
        # the hearth stone, the iron fire basket on its legs with logs, the hood on its flue right over them
        F.cyl('hearth', 0.3, 0.05, (sx, 0.025, sz), M.stone, verts=24, bevel=0.01, bevel_segments=1)
        G.revolve('grate', [(0.0, 0.0), (0.15, 0.0), (0.16, 0.015), (0.145, 0.015), (0.0, 0.015)],
                  at=F.at(sx, 0.12, sz), material=M.iron, segments=16)
        for k in range(12):                          # the basket's bars, bowed out
            a = 30 * k + 15
            c, s_ = G.cosd(a), G.sind(a)
            G.kit.tube(F, _name('grate_bar'), [(sx + 0.15 * c, 0.13, sz + 0.15 * s_), (sx + 0.2 * c, 0.2, sz + 0.2 * s_),
                                               (sx + 0.205 * c, 0.27, sz + 0.205 * s_)], 0.006, M.iron, verts=4, round=2)
        G.torus('grate_ring', 0.205, 0.008, at=F.at(sx, 0.27, sz), material=M.iron, major_segments=20, minor_segments=4)
        for k in range(3):
            a = 120 * k + 30
            c, s_ = G.cosd(a), G.sind(a)
            G.kit.tube(F, _name('grate_leg'), [(sx + 0.12 * c, 0.13, sz + 0.12 * s_), (sx + 0.16 * c, 0.07, sz + 0.16 * s_),
                                               (sx + 0.17 * c, 0.05, sz + 0.17 * s_)], 0.012, M.iron, verts=6, round=2)
        for k, (a, dx, dy) in enumerate(((20, -0.04, 0.0), (-25, 0.05, 0.0), (90, 0.0, 0.035))):
            c, s_ = G.cosd(a) * 0.14, G.sind(a) * 0.14
            _log(F, _name('log'), (sx + dx - c, 0.17 + dy, sz - s_), (sx + dx + c, 0.17 + dy, sz + s_), 0.035, k * 1.7)
        F.cyl('embers', 0.13, 0.02, (sx, 0.15, sz), M.ember, verts=14)
        G.loft('hood', [G.ring(r, r, b + HOOD_Y, 24, at=F.at(sx, 0, sz)[:2]) for r, b in (
            (0.25, 0.0), (0.25, 0.02), (0.2, 0.08), (0.12, 0.18), (0.075, 0.25), (0.062, 0.28))], material=M.iron)
        G.torus('hood_rim', 0.25, 0.01, at=F.at(sx, HOOD_Y + 0.005, sz), material=M.iron, major_segments=24,
                minor_segments=4)
        # the flue up to the ceiling (2.8 m), its joints banded, and its collar there: the hood hangs from it
        F.cyl('flue', 0.06, 2.8 - HOOD_Y - 0.28, (sx, (2.8 + HOOD_Y + 0.28) / 2, sz), M.iron, verts=16)
        for y in (1.3, 2.05):
            F.cyl(_name('flue_band'), 0.066, 0.025, (sx, y, sz), M.iron, verts=16)
        F.cyl('flue_collar', 0.09, 0.03, (sx, 2.785, sz), M.iron, verts=16, bevel=0.005, bevel_segments=1)
        # the chairs, a tripod table with books and tea between them, a low bookcase behind
        _wing_chair(F, -0.5)
        _wing_chair(F, 0.5)
        F.extrude('table_top', G.rounded_rect(0.3, 0.3, 0.15, corner=6), 0.03, (0, 0.56, -0.16), M.walnut, plane='top',
                  bevel=0.008, bevel_segments=2)
        G.revolve('table_stem', [(0.03, 0.12), (0.042, 0.16), (0.03, 0.22), (0.026, 0.4), (0.034, 0.48), (0.03, 0.54),
                                 (0.0, 0.545), (0.0, 0.12)], at=F.at(0, 0, -0.16), material=M.walnut, segments=12)
        for k in range(3):
            a = 120 * k + 90
            c, s_ = G.cosd(a), G.sind(a)
            F.sweep(_name('table_leg'), [(0.03 * c, 0.2, -0.16 + 0.03 * s_), (0.1 * c, 0.1, -0.16 + 0.1 * s_),
                                         (0.17 * c, 0.025, -0.16 + 0.17 * s_), (0.19 * c, 0.013, -0.16 + 0.19 * s_)],
                    0.016, M.walnut, verts=6, resolution=2, scale=[1.2, 1.0, 0.7, 0.8])
        _pile(F, rnd, -0.02, -0.17, 0.575, 3, 0.15, 0.11)
        _mug(F, 0.07, 0.575, -0.09, M.porcelain)
        # the low bookcase: a moulded top, a plinth, two bays of books and a lamp on it
        F.box('case', 1.7, 0.56, 0.3, (0, 0.3, -0.78), M.walnut, bevel=0.008, bevel_segments=1)
        F.extrude('case_top', G.rounded_rect(1.7, 0.3, 0.01, corner=1), 0.03, (0, 0.585, -0.78), M.walnut, plane='top',
                  bevel=0.008, bevel_segments=2)
        F.box('case_plinth', 1.7, 0.05, 0.3, (0, 0.025, -0.78), M.walnut, bevel=0.006, bevel_segments=1)
        F.box('case_mouth', 1.6, 0.46, 0.02, (0, 0.32, -0.626), M.back)
        F.box('case_shelf', 1.6, 0.02, 0.26, (0, 0.32, -0.76), M.walnut)
        F.box('case_divider', 0.03, 0.46, 0.27, (0, 0.32, -0.76), M.walnut)
        _books(F, rnd, -0.8, 0.0, 0.33, -0.9, 0.2, 0.24, fill=0.9, stack=True)
        _books(F, rnd, 0.02, 0.8, 0.33, -0.9, 0.2, 0.24, fill=0.7, lean=True)
        _books(F, rnd, -0.8, 0.0, 0.1, -0.9, 0.2, 0.2, fill=0.85, lean=True)
        _books(F, rnd, 0.02, 0.8, 0.1, -0.9, 0.2, 0.2, fill=0.95)
        _lamp(F, 0.6, 0.6, -0.78, h=0.5)
        # a wicker log basket by the fire: woven, a rim, two logs in it
        G.revolve('basket', [(0, 0), (0.15, 0), (0.17, 0.24), (0.16, 0.24), (0.14, 0.02), (0, 0.02)],
                  at=F.at(0.68, 0, 0.62), material=M.wicker, segments=18, scale=(1, 0.76, 1))
        for k, y in enumerate((0.08, 0.16)):
            r = 0.15 + 0.02 * y / 0.24
            G.torus(_name('weave'), 1.0, 0.006, at=F.at(0.68, y, 0.62), material=M.wicker, major_segments=18,
                    minor_segments=3).scale = (r + 0.003, (r + 0.003) * 0.76, 1.0)
        G.torus('basket_rim', 1.0, 0.012, at=F.at(0.68, 0.24, 0.62), material=M.wicker, major_segments=18,
                minor_segments=4).scale = (0.165, 0.165 * 0.76, 1.0)
        for k in range(3):
            a = 30 * k - 30
            c, s_ = G.cosd(a) * 0.16, G.sind(a) * 0.16
            _log(F, _name('spare_log'), (0.62 + 0.05 * k - s_ * 0.3, 0.2 + 0.03 * k, 0.62 - c),
                 (0.62 + 0.05 * k + s_ * 0.3, 0.27 + 0.03 * k, 0.62 + c), 0.034, 5 + k)
    G.join(still, 'body', origin=(0, 0, 0))
    # The fire: tongues of flame on the logs, joined with their origin on the grate and built flat, so
    # `flames` grows them up while residents read by it.
    with G.new_parts() as fire:
        for k, (dx, dz, r, h, m, t) in enumerate(((0.0, 0.0, 0.07, 0.3, M.flame, 0), (-0.07, 0.03, 0.05, 0.2, M.flame, 120),
                                                  (0.07, -0.03, 0.05, 0.22, M.flame, 250),
                                                  (0.02, 0.05, 0.04, 0.16, M.flame_core, 60),
                                                  (-0.02, -0.04, 0.035, 0.18, M.flame_core, 200))):
            _flame(F, _name('flame'), sx + dx, 0.17, sz + dz, r, h, m, turn=t)
    fire = G.join(fire, 'flames', origin=F.at(sx, 0.17, sz))
    fire.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(fire, 'flames', seconds=0.8)
    return objs


# --- the bedside book stack (1x1): a nightstand-sized chest of drawers (0.6 wide) piled with books, a lamp and tea --
def bedsideStack():
    F = G.GameFrame()
    rnd = random.Random(9)
    h, d = 0.5, 0.43
    zf = WALL + d
    with G.new_parts() as still:
        F.extrude('chest_top', G.rounded_rect(0.6, d, 0.012, corner=2), 0.03, (0, h - 0.015, WALL + d / 2), M.body,
                  plane='top', bevel=0.008, bevel_segments=2)
        F.box('chest', 0.58, h - 0.09, d - 0.02, (0, 0.06 + (h - 0.09) / 2, WALL + (d - 0.02) / 2), M.body, bevel=0.006,
              bevel_segments=1)
        F.box('chest_plinth', 0.59, 0.025, d - 0.01, (0, 0.0725, WALL + (d - 0.01) / 2), M.body, bevel=0.005,
              bevel_segments=1)
        for lx in (-0.26, 0.26):
            for lz in (WALL + 0.04, zf - 0.04):
                G.revolve(_name('leg'), [(0, 0), (0.014, 0), (0.018, 0.012), (0.026, 0.035), (0.024, 0.05), (0.02, 0.06),
                                         (0, 0.06)], at=F.at(lx, 0, lz), material=M.body, segments=8)
        fw, fh = 0.54 / 2, (h - 0.14) / 2
        for i in range(2):
            for j in range(2):
                cx, cy = -0.27 + fw * (i + 0.5), 0.095 + fh * (j + 0.5)
                G.kit.shaker(F, _name('drawer'), fw - 0.014, fh - 0.014, (cx, cy, zf - 0.002), 0.016, M.panel, rail=0.03)
                # a brass cup pull and a card frame with its label
                F.sweep(_name('pull'), [(cx - 0.035, cy - 0.006, zf + 0.006), (cx - 0.03, cy - 0.022, zf + 0.012),
                                        (cx, cy - 0.03, zf + 0.016), (cx + 0.03, cy - 0.022, zf + 0.012),
                                        (cx + 0.035, cy - 0.006, zf + 0.006)], None, M.brass,
                        profile=G.rounded_rect(0.012, 0.004, 0.0015, corner=1), up=(0, 0, 1))
                F.box(_name('pull_plate'), 0.08, 0.012, 0.004, (cx, cy - 0.004, zf + 0.008), M.brass)
                F.box(_name('label_frame'), 0.054, 0.032, 0.004, (cx, cy + 0.035, zf + 0.007), M.brass)
                F.box(_name('label'), 0.046, 0.024, 0.002, (cx, cy + 0.035, zf + 0.009), M.page)
        _pile(F, rnd, -0.12, -0.25, h, 5, 0.22, 0.16)
        top = _pile(F, rnd, 0.1, -0.15, h, 2, 0.2, 0.15)
        # the open book's pages and back board under its cover, the spine rounded
        F.box('book_block', 0.19, 0.03, 0.145, (0.1, top + 0.015, -0.15), M.page)
        F.box('book_back', 0.2, 0.006, 0.15, (0.1, top + 0.003, -0.15), M.book_a)
        F.cyl('book_spine', 0.021, 0.15, (0.198, top + 0.0195, -0.15), M.book_a, rot=(90, 0, 0), verts=10,
              scale=(0.4, 1, 1))
        F.box('book_ribbon', 0.008, 0.002, 0.08, (0.06, top + 0.031, -0.075), M.book_d)
        _lamp(F, 0.17, h, -0.3, h=0.42)
        _mug(F, -0.22, h, -0.06, M.mug_a)
        # reading glasses folded on the top: two rims, a bridge, the arms folded behind
        gx, gy, gz = -0.07, h + 0.004, -0.05
        for s in (-1, 1):
            G.torus(_name('lens'), 0.02, 0.0028, at=F.at(gx + s * 0.024, gy, gz), material=M.dark, major_segments=10,
                    minor_segments=4)
        F.sweep('glasses_bridge', [(gx - 0.006, gy, gz + 0.006), (gx, gy + 0.004, gz + 0.008), (gx + 0.006, gy, gz + 0.006)],
                0.0022, M.dark, verts=4, resolution=2)
        F.sweep('glasses_arms', G.kit.fillet([(gx - 0.044, gy, gz + 0.002), (gx - 0.048, gy + 0.004, gz - 0.016),
                                              (gx + 0.04, gy + 0.004, gz - 0.02), (gx + 0.046, gy + 0.008, gz - 0.006)],
                                             0.006), 0.0022, M.dark, verts=4)
        _pile(F, rnd, 0.2, 0.2, 0.0, 6, 0.24, 0.18)      # a pile on the floor in front of the chest
    G.join(still, 'body', origin=(0, 0, 0))
    # The top book's cover, hinged on the spine at its right: `book_up` opens it over to the right.
    with G.new_parts() as cover:
        F.box('cover', 0.2, 0.006, 0.15, (0.1, top + 0.033, -0.15), M.book_a, bevel=0.0015, bevel_segments=1)
        F.box('cover_title', 0.08, 0.002, 0.03, (0.1, top + 0.037, -0.15), M.book_d)
        F.box('cover_rule', 0.16, 0.0015, 0.004, (0.1, top + 0.0365, -0.2), M.book_d)
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
     G.Model('firesideLounge', firesideLounge, size=(1.7, 2.8, 1.85), key='model.bookworm.firesideLounge',
             centre=(0, -0.005), motions={'uses': {'readByFire': 'flames', 'readAloud': 'flames'}},
             paint=[('walnut', 'Wood'), ('velvet', 'Fabric'), ('rug', 'Rug')]),
     G.Model('bedsideStack', bedsideStack, size=(0.626, 0.931, 0.73), key='model.bookworm.bedsideStack',
             centre=(0.023, -0.06), motions={'uses': {'chapter': 'book_up'}},
             paint=[('body', 'Wood'), ('shade', 'Fabric'), ('panel', 'Drawers')])],
    comment='Blender-built bookworm pack models (tools/art/blender/families/pack_bookworm.py). Listed in '
            'the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. Each has one moving part its use plays: the nook\'s cushion '
            'sinks, the ladder rolls, the bureau\'s flap drops, the teapot pours, the fire grows and the '
            'top book opens.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
