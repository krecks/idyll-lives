# The bedroom's storage: the nightstands and small storage (BR2: the nightstand, the modern, drum and
# three-drawer nightstands, the nightstand with a lamp, the blanket chest, the laundry hamper and the
# bed-end bench) and the wardrobes and drawers (BR3: the dresser, the wardrobe, the tall chest of
# drawers, the low dresser, the single wardrobe and the armoire). Notes: bedroom_storage.md.
#
# The parts are placed in game axes (`F`, no shift), where the game places it: most stand against
# the back of their tile, their backs on the wall face (game depth -0.43), so they're built in place and
# settled with `centre=False` (`centre=` on each Model). Their tops are the content's `surface` heights.
# The BR3 pieces open for a resident choosing an outfit: a drawer slides out 30 cm (`drawer_open`), or
# the doors swing out 100 degrees (`doors_open`, `door_open`) onto a rail of clothes. Of the BR2 pieces,
# the three-drawer nightstand's top drawer slides out for a midnight snack, the chest's lid lifts for a
# blanket and the hamper's while the washing is sorted; the rest are static.
#
# Run:  cargo art bedroom_storage [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    oak=('#CDB08A', 'wood'),         # oak: the nightstand, the dresser and the low dresser
    oakleg=('#B08E66', 'wood'),      # the low dresser's legs
    pull=('#7A5638', 'wood'),        # the dresser's jewellery box
    ebony=('#2B2C2E', 'wood'),       # the dark gap behind a wood drawer
    brass=('#B8925A', 'metal'),      # brass handles, knobs and pulls, the bench's frame and the lamp's stem
    iron=('#2B2C2E', 'metal'),       # black metal: the modern nightstand's and low dresser's pulls, the chest's straps
    white=('#F1F0EC', 'satin'),      # painted white: the modern nightstand and the two wardrobes
    birch=('#CDB08A', 'satin'),      # the wardrobe's doors
    plinth=('#2F3236', 'satin'),     # plinths, and the dark gap behind a painted drawer
    clay=('#C98F6B', 'satin'),       # the drum nightstand
    clay2=('#B87D5C', 'satin'),      # its drawer band and foot
    walnut=('#8C6444', 'satin'),     # the three-drawer nightstand; the armoire's raised panels
    mahogany=('#7C563B', 'satin'),   # the armoire
    mahogany2=('#684731', 'satin'),  # its cornice, base and feet
    linen=('#E9E2D4', 'satin'),      # the tall chest of drawers
    cedar=('#A0603E', 'wood'),       # the blanket chest
    cedar2=('#8A5034', 'wood'),      # its plinth and lid edge
    cedarin=('#C98A5E', 'wood'),     # its lighter inside
    wicker=('#C9A979', 'fabric'),    # the hamper's woven basket
    wicker2=('#B8955F', 'fabric'),   # its lid, rim and bands
    olive=('#8C9468', 'fabric'),     # the bench's cushion
    olive2=('#737B54', 'fabric'),    # its buttons
    porcelain=('#E8E2D6', 'gloss'),  # the lamp's base
    bulb=('#FFF1CF', 'gloss'),       # the lamp's bulb
    shade=('#F4EBD8', 'fabric', {'glow': '#FFD9A0'}),   # the lamp's shade, glowing at night
    teal=('#4E7A7F', 'matte'),       # a book
    book2=('#B5653F', 'matte'),      # more books
    book3=('#E3D5B8', 'matte'),
    paper=('#F2EDE0', 'matte'),      # a book's pages, a box's label
    cream=('#EFE5D3', 'gloss'),      # a mug
    coffee=('#4A2E1E', 'gloss'),     # the coffee in it
    tin=('#4E7A7F', 'gloss'),        # a biscuit tin in the nightstand's drawer
    wrapper=('#C65D52', 'satin'),    # chocolate bars
    box=('#C9B79C', 'matte'),        # storage boxes on a wardrobe
    box2=('#E6DFD3', 'matte'),
    chrome=('#C9CCCF', 'chrome'),    # a hanging rail, the hangers and the wardrobes' hinges
    cloth1=('#5B7DB1', 'satin'),     # clothes: hanging and folded
    cloth2=('#C65D52', 'satin'),
    cloth3=('#E8D9B5', 'satin'),
    cloth4=('#6F8F6A', 'satin'),
    frame=('#B08E66', 'wood'),       # a picture frame on the dresser
    print1=('#4E7A7F', 'wood'),      # its picture (a stained board: wood, so it draws with the dresser)
    vase=('#8A5A38', 'wood'),        # a turned wooden vase
)

F = G.GameFrame()        # game numbers as they are: the models stand where the game places them
WALL = -0.43             # the wall's face, the back of anything against it

DRAWER_SECONDS = 0.9     # a drawer slides out 30 cm in this long
DRAWER_HOLD = 2.5        # seconds the nightstand's drawer stays out (the dressers' and wardrobes' stay open while
                         # the resident changes, the whole use)
DOOR_SECONDS = 1.2       # doors swing out 100 degrees in this long
PULL = 0.30              # how far a drawer slides out
CLOTHES = ('cloth1', 'cloth2', 'cloth3', 'cloth4')   # palette names (materials are made per model)


# --- shared pieces --------------------------------------------------------------------------------------
def _pull(name, x, y, face, length, material, r=0.006, out=0.02, upright=False):
    """A bar pull on two stand-offs, out from a front at game depth `face`."""
    return G.kit.pull(F, name, x, y, face, length, material, r=r, out=out, upright=upright)


def _knob(name, r, at, material):
    """A turned knob, its rose on the front at game depth `at[2]`."""
    return G.kit.knob(F, name, at[0], at[1], at[2], r, material)


def _top(name, w, t, d, y, z, material, x=0.0, r=0.01):
    """A top `w` x `d`, `t` thick, its surface at `y`, its edges and corners rounded."""
    return F.soft(name, w, t, d, (x, y - t / 2, z), material, round=min(r, t * 0.45))


def _front(name, w, h, x, y, z, material, style='plain', rail=None):
    """A drawer's or a door's front `w` x `h`, 0.02 thick, centred at game (x, y, z): 'plain' (its edges rounded),
    'shaker' (a frame round a recessed panel) or 'raised' (round a raised field)."""
    if style == 'plain':
        return [F.soft(name, w, h, 0.02, (x, y, z), material, round=0.006, round_segments=1)]
    return list(G.kit.shaker(F, name, w, h, (x, y, z), 0.02, material, rail=rail or min(0.06, h * 0.22),
                             raised=style == 'raised'))


def _folded(name, x, y, z, w, d, n, h=0.035):
    """A stack of `n` folded clothes, bottom at height `y`, each a soft fold, a little askew."""
    for i in range(n):
        F.soft(f'{name}{i}', w - 0.01 * (i % 2), h, d, (x + 0.005 * (i % 2), y + h * (i + 0.5), z),
               getattr(M, CLOTHES[(i + int(x * 10)) % 4]), round=0.012, round_segments=1, rot=(0, 0, 2 * (i % 3 - 1)))


def _hanger(name, x, rail_y, z, half, length, material):
    """A garment on a hanger, hung side-on from a rail at (rail_y, z): the wire hook over the rail and the hanger's
    shoulders `half` either way, and the garment below them, `length` long, its shoulders sloping."""
    top = rail_y - 0.045
    hook = [(x, rail_y - 0.004, z + 0.016), (x, rail_y + 0.02, z + 0.006), (x, rail_y + 0.02, z - 0.008),
            (x, rail_y + 0.004, z - 0.018), (x, rail_y - 0.03, z), (x, top - 0.035, z - half), (x, top - 0.035, z + half),
            (x, rail_y - 0.03, z)]
    F.sweep(f'{name}_hanger', G.kit.fillet(hook, 0.006, 2), 0.0035, M.chrome, verts=4)
    hd = half + 0.015
    outline = [(-hd, top - 0.06), (-0.04, top), (0.04, top), (hd, top - 0.06), (hd * 0.94, top - length),
               (-hd * 0.94, top - length)]
    F.extrude(name, G.smooth_outline(outline, passes=1), 0.035, (x, 0, z), material, plane='side', bevel=0.008,
              bevel_segments=1)


def _hanging(name, x0, x1, rail_y, z, n, depth=0.42, length=0.78):
    """`n` garments on hangers along a rail at `rail_y` from x0 to x1, hung side-on (across the depth)."""
    step = (x1 - x0) / max(1, n - 1)
    for i in range(n):
        long = length - 0.12 * (i % 3 == 1) - 0.25 * (i % 4 == 3)
        _hanger(f'{name}{i}', x0 + i * step, rail_y, z, depth / 2 - 0.02, long, getattr(M, CLOTHES[i % 4]))


def _gap(name, w, h, x, y, front, material):
    """The dark opening a drawer leaves in the carcass when it slides out, on the carcass face behind the
    drawer front (part of the still body)."""
    F.box(f'{name}_gap', w - 0.02, h - 0.02, 0.002, (x, y, front + 0.001), material)


def _drawer(name, w, h, x, y, front, depth, material, pulls=(), contents=None, style='plain'):
    """A drawer front of `w` x `h` whose back is at game depth `front` (the carcass face), centred at
    (x, y), with the box behind it (inside the carcass at rest) and what's in it, as its own object with
    its origin on the carcass face: for `drawer_open`. `pulls` is [(make, args)] calls."""
    z = front + 0.01
    with G.new_parts() as parts:
        _front(f'{name}_front', w, h, x, y, z, material, style)
        for make, args in pulls:
            make(*args)
        bh = h - 0.05                         # the box's sides, below the front's top edge
        by = y - h / 2 + 0.015 + bh / 2
        back = z - 0.01 - depth
        F.box(f'{name}_bottom', w - 0.05, 0.012, depth, (x, y - h / 2 + 0.02, z - 0.01 - depth / 2), material)
        for s in (-1, 1):
            F.box(f'{name}_side{s}', 0.014, bh, depth, (x + s * (w / 2 - 0.032), by, z - 0.01 - depth / 2),
                  material, bevel=0.003, bevel_segments=1)
        F.box(f'{name}_back', w - 0.05, bh, 0.014, (x, by, back + 0.007), material)
        if contents:
            contents(x, y - h / 2 + 0.026, z - 0.01 - depth / 2)
    return G.join(parts, name, origin=F.at(x, y, front))


def _shell(w, y0, y1, back, front, material, t=0.02, name='shell'):
    """An open-fronted carcass: two sides, a top, a bottom and a back, `t` thick."""
    d, zc, h, yc = front - back, (back + front) / 2, y1 - y0, (y0 + y1) / 2
    for s in (-1, 1):
        F.box(f'{name}_side{s}', t, h, d, (s * (w - t) / 2, yc, zc), material, bevel=0.004, bevel_segments=1)
    F.box(f'{name}_top', w - 2 * t, t, d, (0, y1 - t / 2, zc), material, bevel=0.003, bevel_segments=1)
    F.box(f'{name}_bottom', w - 2 * t, t, d, (0, y0 + t / 2, zc), material, bevel=0.003, bevel_segments=1)
    F.box(f'{name}_back', w - 2 * t, h - 2 * t, t, (0, yc, back + t / 2), material)


def _books(name, x0, y, z, n, h=0.2, d=0.15):
    """A row of `n` books standing on a shelf at height `y`, from x0 rightwards, a band on each spine."""
    x = x0
    mats = (M.teal, M.book2, M.book3)
    for i in range(n):
        t = 0.025 + 0.008 * (i % 3)
        hh = h - 0.02 * (i % 2)
        F.box(f'{name}{i}', t, hh, d, (x + t / 2, y + hh / 2, z), mats[i % 3], bevel=0.003, bevel_segments=1)
        F.box(f'{name}_band{i}', t + 0.002, 0.012, 0.004, (x + t / 2, y + hh * 0.78, z + d / 2), mats[(i + 1) % 3])
        x += t + 0.003


def _book(name, x, y, z, w, t, d, cover):
    """A closed book lying flat at game (x, y: its bottom, z): two boards, a spine on its left, the pages between."""
    for k, yy in enumerate((y + 0.002, y + t - 0.002)):
        F.box(f'{name}_board{k}', w, 0.004, d, (x, yy, z), cover, bevel=0.0015, bevel_segments=1)
    F.cyl(f'{name}_spine', t / 2, d, (x - w / 2 + 0.004, y + t / 2, z), cover, rot=(90, 0, 0), verts=8)
    F.box(f'{name}_pages', w - 0.01, t - 0.006, d - 0.008, (x + 0.002, y + t / 2, z), M.paper)


def _mug(name, x, y, z):
    """A mug of coffee standing at game (x, y, z), its handle to the left."""
    G.revolve(name, [(0, 0), (0.037, 0), (0.04, 0.005), (0.041, 0.1), (0.036, 0.1), (0.035, 0.012), (0, 0.012)],
              at=F.at(x, y, z), material=M.cream, segments=16)
    F.cyl(f'{name}_coffee', 0.0355, 0.004, (x, y + 0.083, z), M.coffee, verts=16)
    F.sweep(f'{name}_handle', [(x - 0.038, y + 0.078, z), (x - 0.062, y + 0.072, z), (x - 0.066, y + 0.05, z),
                               (x - 0.06, y + 0.028, z), (x - 0.038, y + 0.024, z)], 0.006, M.cream, verts=6, resolution=2)


# --- BR2: the nightstand: oak, a drawer over an open niche, on four short tapered legs -----------------
def _classic_nightstand(decor=True):
    front = -0.02
    d, zc = front - WALL, (WALL + front) / 2
    for sx in (-1, 1):
        for z in (WALL + 0.035, front - 0.035):
            F.cone(f'leg{sx}{z}', 0.022, 0.015, 0.145, (sx * 0.22, 0.0725, z), M.oak, verts=12)
    _shell(0.52, 0.14, 0.535, WALL, front, M.oak, name='case')
    # a shaped apron under the niche, between the front legs
    edge = [(0.24 - 0.48 * i / 12, 0.1 + 0.025 * math.cos(math.pi * (0.24 - 0.48 * i / 12) / 0.48)) for i in range(13)]
    F.extrude('apron', [(0.24, 0.142), (-0.24, 0.142)] + edge[::-1], 0.02, (0, 0, front - 0.012), M.oak, bevel=0.003,
              bevel_segments=1)
    F.box('shelf', 0.48, 0.02, d - 0.02, (0, 0.37, zc + 0.01), M.oak, bevel=0.003, bevel_segments=1)
    F.box('drawer_case', 0.48, 0.135, d - 0.02, (0, 0.4575, zc - 0.01), M.oak)
    _front('drawer', 0.48, 0.14, 0, 0.45, front + 0.01, M.oak)
    _knob('knob', 0.018, (0, 0.45, front + 0.02), M.brass)
    _top('top', 0.54, 0.025, 0.44, 0.56, WALL + 0.22, M.oak)
    _books('niche_book', -0.2, 0.16, -0.22, 4, h=0.17, d=0.15)
    if decor:
        _book('book', 0.1, 0.56, -0.16, 0.16, 0.03, 0.22, M.teal)
        _mug('mug', -0.12, 0.56, -0.12)


def nightstand():
    with G.new_parts() as still:
        _classic_nightstand()
    G.join(still, 'nightstand', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the nightstand with a lamp: the same nightstand, a gourd lamp with a drum shade on it --------------
def nightstandLamp():
    with G.new_parts() as still:
        _classic_nightstand(decor=False)
        z = -0.26
        G.revolve('lamp_foot', [(0, 0), (0.05, 0), (0.052, 0.006), (0.044, 0.012), (0, 0.012)], at=F.at(0, 0.56, z),
                  material=M.brass, segments=20)
        G.revolve('lamp_base', [(0, 0.01), (0.045, 0.01), (0.068, 0.04), (0.08, 0.088), (0.066, 0.135),
                                (0.03, 0.166), (0.022, 0.18), (0, 0.18)],
                  at=F.at(0, 0.56, z), material=M.porcelain, segments=20)
        G.revolve('lamp_collar', [(0, 0), (0.026, 0), (0.026, 0.012), (0.012, 0.022), (0, 0.022)],
                  at=F.at(0, 0.735, z), material=M.brass, segments=12)
        F.cyl('lamp_stem', 0.008, 0.13, (0, 0.80, z), M.brass, verts=10)
        F.cyl('lamp_socket', 0.016, 0.04, (0, 0.88, z), M.brass, verts=12)
        F.sphere('lamp_bulb', 0.035, (0, 0.935, z), M.bulb, segments=14, rings=8, scale=(1, 1, 1.15))
        # the drum shade, open at both ends (a ring of thickness), trimmed at both edges, on a harp and spider
        G.revolve('lamp_shade', [(0.15, 0), (0.132, 0.22), (0.126, 0.22), (0.144, 0.0), (0.15, 0)],
                  at=F.at(0, 0.86, z), material=M.shade, segments=28, smooth=False)
        for k, (r, y) in enumerate(((0.149, 0.864), (0.132, 1.076))):
            F.sweep(f'lamp_trim{k}', [(r * math.cos(math.tau * i / 28), y, z + r * math.sin(math.tau * i / 28))
                                      for i in range(28)], 0.004, M.shade, verts=4, closed=True)
        G.torus('lamp_ring', 0.128, 0.003, at=F.at(0, 1.075, z), material=M.brass, major_segments=20,
                minor_segments=4)
        for a in (0, 120, 240):
            G.rod(f'lamp_spoke{a}', F.at(0, 1.075, z), F.at(0.128 * G.cosd(a), 1.075, z + 0.128 * G.sind(a)),
                  0.0025, material=M.brass, verts=4)
        for s in (-1, 1):
            F.sweep(f'lamp_harp{s}', [(0, 0.895, z), (s * 0.03, 0.93, z), (s * 0.035, 1.0, z), (0, 1.075, z)], 0.0025,
                    M.brass, verts=4, resolution=2)
        F.sphere('lamp_finial', 0.012, (0, 1.08, z), M.brass, segments=10, rings=6, scale=(1, 1, 0.8))
    G.join(still, 'nightstandLamp', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the modern nightstand: white, an oak top and legs, a drawer over an open niche of books -----------
# 0.47 m deep, as the classic one (it was 0.63, as deep as its tile and past the pillows: M-bedroom-02).
NS_FRONT, NS_TOP = 0.0, (0.46, -0.20)    # the case's front; the top's depth and middle


def _splayed(name, x, z, h, r, material, out=0.016):
    """A round leg tapering from the case at height `h` over (x, z) to a foot splayed out by `out`."""
    sx, sz = math.copysign(out, x), math.copysign(out, z + 0.21)
    return G.kit.tube(F, name, [(x, h, z), (x + sx, 0.002, z + sz)], r, material, verts=10, round=1, scale=[1, 0.65])


def nightstandModern():
    with G.new_parts() as still:
        front, back = NS_FRONT, -0.42
        for sx in (-1, 1):
            for z in (-0.37, front - 0.05):
                _splayed(f'leg{sx}{z}', sx * 0.23, z, 0.13, 0.018, M.oak)
        _shell(0.56, 0.12, 0.53, back, front, M.white, name='case')
        F.box('shelf', 0.52, 0.02, front - back - 0.02, (0, 0.37, (back + front) / 2 + 0.01), M.white)
        F.box('drawer_case', 0.52, 0.14, front - back - 0.04, (0, 0.45, (back + front) / 2 - 0.01), M.white)
        _front('drawer', 0.52, 0.135, 0, 0.445, front + 0.01, M.white)
        _pull('pull', 0, 0.485, front + 0.02, 0.12, M.iron, r=0.005, out=0.016)
        _top('top', 0.58, 0.03, NS_TOP[0], 0.56, NS_TOP[1], M.oak, r=0.012)
        _books('niche_book', -0.22, 0.14, -0.2, 5, h=0.19, d=0.16)
        F.soft('niche_box', 0.14, 0.08, 0.16, (0.15, 0.18, -0.18), M.box, round=0.01, round_segments=1)
        F.soft('niche_box_lid', 0.146, 0.02, 0.166, (0.15, 0.225, -0.18), M.box, round=0.008, round_segments=1)
    G.join(still, 'nightstandModern', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the drum nightstand: a terracotta drum with a drawer band, an oak top and a round foot ------------
def nightstandRound():
    z = -0.06
    with G.new_parts() as still:
        G.revolve('foot', [(0, 0), (0.19, 0), (0.2, 0.006), (0.2, 0.034), (0.19, 0.04), (0, 0.04)],
                  at=F.at(0, 0, z), material=M.clay2, segments=36)
        # a drum swelling a little to its middle, its lower edge rounded under
        G.revolve('drum', [(0, 0.04), (0.228, 0.04), (0.244, 0.05), (0.25, 0.075), (0.253, 0.2), (0.253, 0.34),
                           (0.25, 0.52), (0.242, 0.54), (0, 0.54)], at=F.at(0, 0, z), material=M.clay, segments=36)
        F.cyl('band', 0.256, 0.12, (0, 0.42, z), M.clay2, verts=36, bevel=0.003, bevel_segments=1)
        for y in (0.358, 0.482):
            G.torus(f'groove{y}', 0.256, 0.004, at=F.at(0, y, z), material=M.plinth, major_segments=36,
                    minor_segments=4)
        # the drawer's front: the band's front third, between two seams
        for a in (-35, 35):
            F.box(f'seam{a}', 0.004, 0.118, 0.006, (0.257 * G.sind(a), 0.42, z + 0.257 * G.cosd(a)), M.plinth,
                  rot=(0, 0, -a))
        _knob('knob', 0.0185, (0, 0.42, z + 0.255), M.brass)
        G.revolve('top', [(0, 0.54), (0.26, 0.54), (0.268, 0.547), (0.27, 0.556), (0.268, 0.565), (0.26, 0.57),
                          (0, 0.57)], at=F.at(0, 0, z), material=M.oak, segments=36)
    G.join(still, 'nightstandRound', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the three-drawer nightstand: walnut, three drawers with brass bars; the top one a part of its own ---
# 0.47 m deep, as the modern one; a midnight snack in the top drawer (the use `snack`).
def nightstandDrawers():
    front, back = NS_FRONT, -0.42
    with G.new_parts() as still:
        F.box('plinth', 0.52, 0.05, front - back - 0.04, (0, 0.025, (back + front) / 2 - 0.01), M.walnut, bevel=0.004,
              bevel_segments=1)
        F.soft('case', 0.56, 0.52, front - back, (0, 0.31, (back + front) / 2), M.walnut, round=0.01, round_segments=2)
        _top('top', 0.58, 0.03, NS_TOP[0], 0.6, NS_TOP[1], M.walnut, r=0.012)
        for i, y in enumerate((0.335, 0.185)):
            _front(f'drawer{i}', 0.52, 0.135, 0, y, front + 0.01, M.walnut)
            _pull(f'pull{i}', 0, y + 0.02, front + 0.02, 0.11, M.brass, r=0.005, out=0.016)
        _gap('drawer', 0.52, 0.135, 0, 0.485, front, M.plinth)
    G.join(still, 'nightstandDrawers', origin=(0, 0, 0))
    # the top drawer with snacks and a paperback in it, sliding out while a resident takes a snack
    drawer = _drawer('drawer', 0.52, 0.135, 0, 0.485, front, 0.36, M.walnut,
                     pulls=[(_pull, ('pull_top', 0, 0.505, front + 0.02, 0.11, M.brass, 0.005, 0.016))],
                     contents=_drawer_snacks)
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the blanket chest: a cedar box on a plinth with iron straps and side handles; the lid a part --------
def cedarChest():
    w, d, y0, y1 = 0.9, 0.46, 0.06, 0.455
    t = 0.025
    with G.new_parts() as still:
        # the plinth: a moulded band on bracket feet, cut away in an arch between them
        F.soft('plinth_band', 0.94, 0.025, 0.5, (0, 0.0475, 0), M.cedar2, round=0.008, round_segments=1)
        for s in (-1, 1):
            arch = [(0.47 - 0.94 * i / 16, 0.012 + 0.02 * math.sin(math.pi * i / 16)) for i in range(17)]
            arch = [(0.47, 0.0), (0.47, 0.036), (-0.47, 0.036), (-0.47, 0.0), (-0.39, 0.0)] + \
                [(u, v) for u, v in arch[::-1] if abs(u) < 0.39] + [(0.39, 0.0)]
            F.extrude(f'plinth_x{s}', arch[::-1], 0.02, (0, 0, s * 0.24), M.cedar2, bevel=0.003, bevel_segments=1)
            side = [(0.25, 0.0), (0.25, 0.036), (-0.25, 0.036), (-0.25, 0.0), (-0.17, 0.0), (-0.12, 0.022),
                    (0.12, 0.022), (0.17, 0.0)]
            F.extrude(f'plinth_z{s}', side[::-1], 0.02, (s * 0.46, 0, 0), M.cedar2, plane='side', bevel=0.003,
                      bevel_segments=1)
        for s in (-1, 1):
            F.box(f'wall_z{s}', w, y1 - y0, t, (0, (y0 + y1) / 2, s * (d - t) / 2), M.cedar, bevel=0.004,
                  bevel_segments=1)
            F.box(f'wall_x{s}', t, y1 - y0, d - 2 * t, (s * (w - t) / 2, (y0 + y1) / 2, 0), M.cedar, bevel=0.004,
                  bevel_segments=1)
        F.box('floor', w - 2 * t, 0.02, d - 2 * t, (0, y0 + 0.01, 0), M.cedarin)
        # two raised panels in frames on the front
        for s in (-1, 1):
            G.kit.shaker(F, f'front_panel{s}', 0.34, 0.27, (s * 0.2, 0.255, d / 2 + 0.006), 0.012, M.cedar2, rail=0.035,
                         panel=M.cedar, raised=True, recess=0.003)
        # iron corner brackets with rivets, and a drop handle each side on a plate
        for s in (-1, 1):
            for y in (0.09, 0.425):
                F.box(f'corner_front{s}{y}', 0.05, 0.05, 0.004, (s * (w / 2 - 0.025), y, d / 2 + 0.002), M.iron,
                      bevel=0.0015, bevel_segments=1)
                F.box(f'corner_side{s}{y}', 0.004, 0.05, 0.05, (s * (w / 2 + 0.002), y, d / 2 - 0.025), M.iron,
                      bevel=0.0015, bevel_segments=1)
                F.sphere(f'rivet{s}{y}', 0.005, (s * (w / 2 - 0.02), y, d / 2 + 0.004), M.iron, segments=6, rings=3,
                         scale=(1, 1, 0.5))
            hx = s * (w / 2 + 0.006)
            F.box(f'handle_plate{s}', 0.008, 0.05, 0.13, (hx, 0.36, 0), M.iron, bevel=0.002, bevel_segments=1)
            F.sweep(f'handle{s}', G.kit.fillet([(s * (w / 2 + 0.014), 0.365, -0.055), (s * (w / 2 + 0.024), 0.3, -0.055),
                                                (s * (w / 2 + 0.024), 0.3, 0.055), (s * (w / 2 + 0.014), 0.365, 0.055)],
                                               0.014), 0.006, M.iron, verts=6)
        # the lock: a brass escutcheon with a black keyhole, under the lid's front edge
        F.soft('latch', 0.07, 0.06, 0.006, (0, 0.415, d / 2 + 0.003), M.brass, round=0.002, round_segments=1)
        F.cyl('keyhole', 0.006, 0.004, (0, 0.42, d / 2 + 0.006), M.iron, rot=(90, 0, 0), verts=8)
        F.box('keyhole_slot', 0.004, 0.012, 0.004, (0, 0.411, d / 2 + 0.006), M.iron)
        b = F.soft('blanket', w - 2 * t - 0.02, 0.1, d - 2 * t - 0.02, (0, y0 + 0.08, 0), M.cloth3, round=0.03,
                   puff=0.012, flat=3)
        G.jitter(b, 0.006, size=0.1, seed=4, up=0.5)
    G.join(still, 'cedarChest', origin=(0, 0, 0))
    # the lid, hinged along its back edge: it lifts while a resident takes out a blanket (`lid_open`)
    with G.new_parts() as parts:
        # (its underside 2 mm below the edge band's, not in its plane: seen when the lid is up)
        F.soft('lid', 0.94, 0.047, 0.5, (0, 0.4765, 0), M.cedar, round=0.012, round_segments=2)
        F.soft('lid_edge', 0.96, 0.012, 0.52, (0, 0.461, 0), M.cedar2, round=0.005, round_segments=1)
        b = 0.04
        F.extrude('lid_frame', G.rounded_rect(0.84, 0.4, 0.01, corner=1), 0.006, (0, 0.5, 0), M.cedar2, plane='top',
                  holes=[G.rounded_rect(0.74, 0.3, 0.01, corner=1)], bevel=0.002, bevel_segments=1)
        # the raised field inside it, chamfered up to its flat top (the chest's top, 0.505)
        G.loft('lid_panel', [[F.at(u * k, y, v * k2) for u, v in ((1, 1), (-1, 1), (-1, -1), (1, -1))]
                             for y, k, k2 in ((0.499, 0.369, 0.149), (0.5005, 0.369, 0.149),
                                              (0.505, 0.369 - b, 0.149 - b))],
               material=M.cedar, smooth=False)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.455, -0.25))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -95, seconds=0.9, axis='X')


# --- the laundry hamper: a woven basket, wider at the top, with bands; the lid a part ----------------------
def _braid(name, r, y, z, material, n=48, cord=0.009):
    """A braided band round the hamper at height `y`, `r` from its middle: a cord swelling and narrowing in turn."""
    pts = [(r * math.cos(math.tau * i / n), y, z + r * math.sin(math.tau * i / n)) for i in range(n)]
    F.sweep(name, pts, cord, material, verts=4, closed=True, scale=[1 + 0.3 * (i % 2) for i in range(n)])


def laundryHamper():
    z = -0.2
    bx, by = F.at(0, 0, z)[:2]

    def rib(y, r, k):
        return [(bx + r * (1 + k * math.cos(24 * a)) * math.cos(a), by + r * (1 + k * math.cos(24 * a)) * math.sin(a), y)
                for a in (math.tau * i / 48 for i in range(48))]
    with G.new_parts() as still:
        # the basket: woven staves wider at the top, its wall 14 mm thick, open, and a woven floor inside
        outer = [(0.0, 0.196, 0.0), (0.02, 0.2, 0.012)] + [(y, 0.2 + 0.031 * y / 0.56, 0.014) for y in (0.15, 0.3, 0.45)] + \
            [(0.56, 0.231, 0.012)]
        secs = [rib(y, r, k) for y, r, k in outer] + [rib(0.56, 0.217, 0.0), rib(0.025, 0.188, 0.0)]
        G.loft('basket', secs, M.wicker)
        for y in (0.04, 0.2, 0.4):
            _braid(f'band{y}', 0.2 + 0.031 * y / 0.56 + 0.004, y, z, M.wicker2)
        _braid('rim', 0.222, 0.552, z, M.wicker2, cord=0.01)
        # the washing in it, near the top, and a sleeve hanging out under the lid
        for i, (x, zz, y, m) in enumerate(((-0.05, -0.04, 0.44, M.cloth1), (0.07, 0.05, 0.46, M.cloth2),
                                           (0.0, 0.07, 0.42, M.cloth3))):
            c = F.sphere(f'washing{i}', 0.13, (x, y, z + zz), m, segments=12, rings=6, scale=(1, 0.9, 0.4))
            G.jitter(c, 0.012, size=0.06, seed=i, up=1.0)
        F.sweep('sleeve', G.kit.fillet([(0.03, 0.5, z + 0.17), (0.03, 0.556, z + 0.222), (0.03, 0.548, z + 0.234),
                                        (0.035, 0.4, z + 0.231)], 0.012), None, M.cloth1,
                profile=G.rounded_rect(0.07, 0.01, 0.004, corner=1), up=(1, 0, 0))
    G.join(still, 'laundryHamper', origin=(0, 0, 0))
    # the lid, hinged at its back: it lifts while a resident sorts the washing; a domed woven top, a loop to lift it
    with G.new_parts() as parts:
        G.loft('lid', [G.ring(r, r, y, 32, at=(bx, by)) for r, y in ((0.226, 0.56), (0.236, 0.565), (0.236, 0.585),
                                                                      (0.226, 0.598), (0.15, 0.606), (0.02, 0.608))],
               M.wicker2)
        _braid('lid_rim', 0.229, 0.578, z, M.wicker2, n=40, cord=0.0065)
        F.sweep('lid_loop', [(-0.03, 0.604, z), (-0.022, 0.624, z), (0, 0.629, z), (0.022, 0.624, z), (0.03, 0.604, z)],
                0.006, M.wicker2, verts=6, resolution=2)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.56, z - 0.236))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -70, seconds=0.7, axis='X')   # not past upright: it stands at the wall


# --- the bed-end bench: a buttoned olive cushion on a slim brass frame ------------------------------------
def benchBedEnd():
    with G.new_parts() as still:
        # each end one bent tube, down, across and down; rails along it, a stretcher low between the ends
        for sx in (-1, 1):
            x = sx * 0.74
            F.sweep(f'end{sx}', G.kit.fillet([(x, 0.004, -0.16), (x, 0.335, -0.16), (x, 0.335, 0.16), (x, 0.004, 0.16)],
                                             0.03, 3), 0.012, M.brass, verts=10)
            F.sweep(f'end_low{sx}', [(x, 0.09, -0.16), (x, 0.09, 0.16)], 0.007, M.brass, verts=8)
            for sz in (-1, 1):
                F.cyl(f'glide{sx}{sz}', 0.014, 0.006, (x, 0.003, sz * 0.16), M.brass, verts=10)
        for sz in (-1, 1):
            F.sweep(f'rail{sz}', [(-0.74, 0.335, sz * 0.16), (0.74, 0.335, sz * 0.16)], 0.01, M.brass, verts=10)
        F.sweep('stretcher', [(-0.74, 0.09, 0), (0.74, 0.09, 0)], 0.007, M.brass, verts=8)
        # the cushion: piped, buttoned in two staggered rows, puffed between (its top, the seat, at 0.47)
        F.soft('cushion', 1.6, 0.105, 0.42, (0, 0.4075, 0), M.olive, round=0.035, puff=0.01, flat=5, piping=0.007)
        for row, z in enumerate((-0.09, 0.09)):
            for i in range(6 - row):
                x = -0.625 + 0.25 * i + 0.125 * row
                F.sphere(f'button{row}{i}', 0.014, (x, 0.462, z), M.olive2, segments=8, rings=5, scale=(1, 1, 0.5))
    G.join(still, 'benchBedEnd', origin=(0, 0, 0))
    G.settle(centre=False)


def _drawer_snacks(x, y, z):
    """A nightstand drawer's midnight snacks, bottom at `y` round (x, z): a biscuit tin, two chocolate bars
    and a paperback."""
    G.revolve('tin', [(0, 0), (0.068, 0), (0.07, 0.005), (0.07, 0.06), (0.072, 0.062), (0.072, 0.07), (0, 0.07)],
              at=F.at(x - 0.12, y, z), material=M.tin, segments=16)
    for i, dx in enumerate((0.03, 0.07)):
        F.box(f'bar{i}', 0.05, 0.015, 0.16, (x + dx, y + 0.0075 + 0.015 * i, z + 0.02 * i), M.wrapper, bevel=0.004,
              bevel_segments=1)
    _book('paperback', x + 0.18, y, z - 0.02, 0.12, 0.025, 0.18, M.book2)


# --- BR3: the dresser: oak, two small drawers over two wide ones; the middle (wide) one slides out -------
def _drawer_clothes(x, y, z, w=1.2, d=0.36):
    """Folded clothes in a drawer: stacks across its width, bottom at `y`."""
    n = max(2, int(w / 0.3))
    step = (w - 0.1) / n
    for i in range(n):
        _folded(f'folded{i}_', x - (w - 0.1) / 2 + step * (i + 0.5), y, z, step - 0.03, d, 2 + (i % 2))


def _bracket(name, x, front, back, material, h=0.09, w=0.14):
    """A bracket foot at the front corner on the side of `x` (its outer edge at x): an ogee-cut plate on the front
    and one down the side, mitred round the corner."""
    s = 1 if x > 0 else -1
    curve = [(w - 0.012, h - 0.004), (w - 0.035, h - 0.022), (w - 0.06, h - 0.04), (w - 0.085, h - 0.05),
             (w - 0.1, h - 0.062), (w - 0.11, 0.0)]
    out = [(0.0, 0.0), (0.0, h)] + curve
    F.extrude(f'{name}_front', [(x - s * u, v) for u, v in out][::s], 0.02, (0, 0, front), material, bevel=0.003,
              bevel_segments=1)
    F.extrude(f'{name}_side', [(front - 0.01 - u, v) for u, v in out][::-1], 0.02, (x - s * 0.01, 0, 0), material,
              plane='side', bevel=0.003, bevel_segments=1)


def dresser():
    front = 0.07
    with G.new_parts() as still:
        # bracket feet at the front corners, block feet at the back, a shaped apron between the front ones
        for sx in (-1, 1):
            _bracket(f'foot{sx}', sx * 0.7, front, WALL, M.oak)
            F.box(f'foot_back{sx}', 0.05, 0.06, 0.05, (sx * 0.665, 0.03, WALL + 0.03), M.oak, bevel=0.006,
                  bevel_segments=1)
        edge = [(0.56 - 1.12 * i / 24, 0.055 - 0.02 * math.sin(math.pi * i / 24) ** 4) for i in range(25)]
        F.extrude('apron', [(0.56, 0.09), (-0.56, 0.09)] + edge[::-1], 0.02, (0, 0, front - 0.01), M.oak, bevel=0.003,
                  bevel_segments=1)
        F.box('plinth', 1.4, 0.03, 0.48, (0, 0.075, (WALL + front) / 2), M.oak, bevel=0.006, bevel_segments=1)
        F.box('case', 1.4, 0.77, 0.5, (0, 0.475, (WALL + front) / 2), M.oak, bevel=0.008, bevel_segments=1)
        _top('top', 1.44, 0.03, 0.52, 0.89, -0.17, M.oak, r=0.012)
        for s in (-1, 1):
            _front(f'drawer_top{s}', 0.66, 0.2, s * 0.35, 0.72, front + 0.01, M.oak)
            _pull(f'pull_top{s}', s * 0.35, 0.75, front + 0.02, 0.1, M.brass, out=0.014)
            _pull(f'pull_low{s}', s * 0.35, 0.27, front + 0.02, 0.1, M.brass, out=0.014)
        _front('drawer_low', 1.36, 0.2, 0, 0.24, front + 0.01, M.oak)
        # on top: a framed picture on its easel, a turned vase and a box with a lid
        F.extrude('picture_frame', G.rounded_rect(0.36, 0.46, 0.004, corner=1), 0.03, (0.32, 1.12, -0.3), M.frame,
                  holes=[G.rounded_rect(0.27, 0.37, 0.004, corner=1)], bevel=0.006, bevel_segments=2, rot=(-6, 0, 0))
        F.box('picture', 0.268, 0.368, 0.006, (0.32, 1.12, -0.29), M.print1, rot=(-6, 0, 0))
        F.box('picture_easel', 0.04, 0.3, 0.01, (0.32, 1.0, -0.36), M.frame, rot=(18, 0, 0))
        G.revolve('vase', [(0, 0), (0.042, 0), (0.046, 0.006), (0.06, 0.06), (0.05, 0.15), (0.027, 0.195),
                           (0.026, 0.21), (0.034, 0.22), (0.022, 0.222), (0.02, 0.17), (0, 0.17)],
                  at=F.at(-0.45, 0.89, -0.2), material=M.vase, segments=18)
        F.soft('jewellery_box', 0.22, 0.055, 0.16, (-0.12, 0.9175, -0.16), M.pull, round=0.008, round_segments=1)
        F.soft('jewellery_lid', 0.226, 0.018, 0.166, (-0.12, 0.951, -0.16), M.pull, round=0.007, round_segments=1)
        F.box('jewellery_clasp', 0.024, 0.02, 0.004, (-0.12, 0.94, -0.078), M.brass, bevel=0.0015, bevel_segments=1)
        _gap('drawer', 1.36, 0.2, 0, 0.48, front, M.ebony)
    G.join(still, 'dresser', origin=(0, 0, 0))
    drawer = _drawer('drawer', 1.36, 0.2, 0, 0.48, front, 0.44, M.oak,
                     pulls=[(_pull, (f'pull_mid{s}', s * 0.35, 0.51, front + 0.02, 0.1, M.brass, 0.006, 0.014))
                            for s in (-1, 1)],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 1.2, 0.36))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the wardrobe: a white carcass, two birch doors with brass bars, a rail of shirts and shelves --------
def _wardrobe_inside(w, y0, y1, back, front, rail_y, split=True):
    """A wardrobe's inside: a rail of clothes on hangers (the left half when `split`, with shelves of folded
    clothes on the right) under a shelf."""
    d, zc = front - back, (back + front) / 2
    hang_x1 = -0.02 if split else w / 2 - 0.08
    F.box('hat_shelf', (w / 2 if split else w) - 0.04, 0.02, d - 0.04,
          (-w / 4 if split else 0, rail_y + 0.12, zc - 0.01), M.white)
    F.cyl('rail', 0.012, (w / 2 if split else w) - 0.06, (-w / 4 if split else 0, rail_y, zc), M.chrome,
          verts=10, rot=(0, 90, 0))
    n = 5 if split else max(3, int(w / 0.16))
    _hanging('shirt', -w / 2 + 0.1, hang_x1 - 0.08, rail_y, zc, n, depth=min(0.42, d - 0.1))
    # a box on the hat shelf, under the carcass's top and inside its side (it came through both)
    bw = min(0.3, w / 2 - 0.06)
    _storage_box('folded_box', bw, 0.13, 0.3, (-w / 4, rail_y + 0.13, zc), M.box)
    if split:
        F.box('divider', 0.02, y1 - y0 - 0.04, d - 0.02, (0, (y0 + y1) / 2, zc - 0.01), M.white)
        for i, y in enumerate((y0 + 0.45, y0 + 0.9, y0 + 1.35)):
            F.box(f'shelf{i}', w / 2 - 0.04, 0.02, d - 0.04, (w / 4, y, zc - 0.01), M.white)
            _folded(f'stack{i}_', w / 4 - 0.18, y + 0.01, zc, 0.3, 0.32, 3)
            _folded(f'stackb{i}_', w / 4 + 0.18, y + 0.01, zc, 0.3, 0.32, 2 + i % 2)


def _storage_box(name, w, h, d, at, material):
    """A lidded storage box standing at game `at` (its bottom), a label on its front."""
    x, y, z = at
    F.soft(name, w, h - 0.02, d, (x, y + (h - 0.02) / 2, z), material, round=0.008, round_segments=1)
    F.soft(f'{name}_lid', w + 0.008, 0.028, d + 0.008, (x, y + h - 0.014, z), material, round=0.008, round_segments=1)
    F.box(f'{name}_label', w * 0.35, 0.04, 0.002, (x, y + h * 0.45, z + d / 2 + 0.001), M.paper)


def _hinges(name, x, ys, z):
    """The knuckles of a door's hinges on its hinge line (x, z), on the still carcass."""
    for y in ys:
        F.cyl(f'{name}{y}', 0.006, 0.06, (x, y, z), M.chrome, verts=8)


def wardrobe():
    back, front = WALL, 0.15
    with G.new_parts() as still:
        F.box('plinth', 1.80, 0.06, 0.54, (0, 0.03, (back + front) / 2 - 0.01), M.plinth, bevel=0.004, bevel_segments=1)
        _shell(1.84, 0.06, 2.11, back, front, M.white, t=0.025, name='case')
        _wardrobe_inside(1.84, 0.06, 2.11, back, front, 1.8)
        _storage_box('box_big', 0.4, 0.3, 0.32, (-0.5, 2.11, -0.18), M.box)
        _storage_box('box_small', 0.32, 0.22, 0.3, (-0.08, 2.11, -0.18), M.box2)
        for s in (-1, 1):
            _hinges(f'hinge{s}_', s * 0.913, (0.3, 1.085, 1.86), front + 0.01)
    G.join(still, 'wardrobe', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            _front(f'door{s}', 0.905, 2.04, s * 0.4575, 1.085, front + 0.01, M.birch, 'shaker', rail=0.09)
            _pull(f'handle{s}', s * 0.06, 1.1, front + 0.02, 0.36, M.brass, r=0.008, out=0.018, upright=True)
        doors.append(G.hinged(G.join(parts, f'door{"_left" if s < 0 else "_right"}'),
                              F.at(s * 0.91, 1.085, front + 0.01)))
    G.settle(centre=False)
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


# --- the tall chest of drawers: linen-white, five drawers with brass bars, on a dark plinth --------------
# 0.51 m deep (it was 0.66, as deep as its tile: M-bedroom-02).
CHEST_FRONT = 0.04


def chestOfDrawers():
    front = CHEST_FRONT
    with G.new_parts() as still:
        F.box('plinth', 0.82, 0.06, front - WALL - 0.04, (0, 0.03, (WALL + front) / 2 - 0.01), M.plinth, bevel=0.004,
              bevel_segments=1)
        F.box('case', 0.86, 1.07, front - WALL, (0, 0.595, (WALL + front) / 2), M.linen, bevel=0.008, bevel_segments=1)
        _top('top', 0.88, 0.03, front + 0.015 - WALL, 1.16, (WALL + front + 0.015) / 2, M.linen, r=0.012)
        for i, y in enumerate((0.179, 0.387, 0.803, 1.011)):
            _front(f'drawer{i}', 0.815, 0.194, 0, y, front + 0.01, M.linen, 'shaker', rail=0.04)
            _pull(f'pull{i}', 0, y + 0.031, front + 0.02, 0.14, M.brass, r=0.005, out=0.016)
        _gap('drawer', 0.815, 0.194, 0, 0.595, front, M.plinth)
    G.join(still, 'chestOfDrawers', origin=(0, 0, 0))
    drawer = _drawer('drawer', 0.815, 0.194, 0, 0.595, front, 0.42, M.linen,
                     pulls=[(_pull, ('pull_mid', 0, 0.626, front + 0.02, 0.14, M.brass, 0.005, 0.016))],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 0.72, 0.34), style='shaker')
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the low dresser: oak, nine drawers in three columns on tapered legs; the middle one slides out ------
def dresserLow():
    front = 0.165
    ys, xs = (0.2217, 0.435, 0.6483), (-0.59, 0.0, 0.59)
    with G.new_parts() as still:
        for sx in (-1, 1):
            for z in (-0.39, 0.12):
                G.kit.tube(F, f'leg{sx}{z}', [(sx * 0.84, 0.1, z), (sx * 0.852, 0.002, z + math.copysign(0.012, z + 0.1))],
                           0.022, M.oakleg, verts=10, round=1, scale=[1, 0.7])
        F.soft('case', 1.8, 0.67, front - WALL, (0, 0.435, (WALL + front) / 2), M.oak, round=0.014, round_segments=2)
        _top('top', 1.82, 0.03, 0.61, 0.8, (WALL + 0.18) / 2, M.oak, r=0.012)
        F.box('reveal', 1.78, 0.008, 0.58, (0, 0.768, (WALL + 0.18) / 2), M.ebony)
        for x in xs:
            for y in ys:
                if (x, y) == (0.0, 0.435):
                    continue
                _front(f'drawer{x}{y}', 0.575, 0.1993, x, y, front + 0.01, M.oak)
                _pull(f'pull{x}{y}', x, y + 0.032, front + 0.02, 0.14, M.iron, r=0.005, out=0.016)
        _gap('drawer', 0.575, 0.1993, 0, 0.435, front, M.ebony)
    G.join(still, 'dresserLow', origin=(0, 0, 0))
    drawer = _drawer('drawer', 0.575, 0.1993, 0, 0.435, front, 0.5, M.oak,
                     pulls=[(_pull, ('pull_mid', 0, 0.467, front + 0.02, 0.14, M.iron, 0.005, 0.016))],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 0.5, 0.4))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the single wardrobe: white, one door hinged on the left with a brass bar, a rail of shirts ----------
# 0.60 m wide, its door 0.58 (it was 0.90, with a door as wide as a room's: M-bedroom-03).
WS_W = 0.6


def wardrobeSingle():
    back, front = WALL, 0.14
    w = WS_W
    with G.new_parts() as still:
        F.box('plinth', w - 0.04, 0.05, 0.53, (0, 0.025, (back + front) / 2 - 0.01), M.plinth, bevel=0.004,
              bevel_segments=1)
        _shell(w, 0.05, 2.04, back, front, M.white, t=0.022, name='case')
        _wardrobe_inside(w, 0.05, 2.04, back, front, 1.74, split=False)
        _storage_box('box_top', 0.3, 0.24, 0.3, (-0.1, 2.04, -0.2), M.box)
        _hinges('hinge', -(w - 0.02) / 2, (0.3, 1.045, 1.79), front + 0.01)
    G.join(still, 'wardrobeSingle', origin=(0, 0, 0))
    with G.new_parts() as parts:
        _front('door', w - 0.02, 1.97, 0, 1.045, front + 0.01, M.white, 'shaker', rail=0.08)
        _pull('handle', w / 2 - 0.06, 1.1, front + 0.02, 0.44, M.brass, r=0.008, out=0.018, upright=True)
    door = G.hinged(G.join(parts, 'door'), F.at(-(w - 0.02) / 2, 1.045, front + 0.01))
    G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=DOOR_SECONDS)


# --- the armoire: mahogany, a cornice, raised-panel doors with brass knobs, on bun feet -------------------
def _crown(name, x, front, back, y, material, h=0.069, out=0.05):
    """A crown moulding round the front and sides at height `y` (its bottom), its path `x` either side and at
    game depth `front`: a cove rising and swelling out `out` to a flat top."""
    prof = [(0.0, 0.0), (0.0, -0.006), (0.012, -0.012), (0.024, -0.02), (0.036, -0.032), (0.046, -0.044),
            (0.054, -out), (h, -out), (h, 0.03), (0.0, 0.03)]
    path = G.kit.fillet([(-x, y, back), (-x, y, front), (x, y, front), (x, y, back)], 0.012, 2)
    F.sweep(name, path, None, material, profile=prof, up=(0, 0, 1), angle=30)


def armoire():
    back, front = WALL, 0.115
    with G.new_parts() as still:
        for sx in (-1, 1):
            for z in (-0.38, 0.08):
                G.revolve(f'foot{sx}{z}', [(0, 0), (0.028, 0), (0.044, 0.018), (0.048, 0.045), (0.04, 0.07),
                                           (0.026, 0.085), (0.026, 0.12), (0, 0.12)],
                          at=F.at(sx * 0.8, 0, z), material=M.mahogany2, segments=14)
        F.soft('base', 1.74, 0.1, 0.59, (0, 0.17, (back + 0.16) / 2), M.mahogany2, round=0.012, round_segments=1)
        _crown('base_moulding', 0.85, front + 0.005, back, 0.22, M.mahogany2, h=0.03, out=0.018)
        _shell(1.7, 0.22, 1.97, back, front, M.mahogany, t=0.03, name='case')
        F.box('cornice', 1.76, 0.07, 0.57, (0, 2.005, (back + 0.14) / 2), M.mahogany, bevel=0.006, bevel_segments=1)
        _crown('crown', 0.88, 0.148, back, 2.04, M.mahogany2)
        # the face frame round the doors: rails above and below, stiles with a bead
        F.box('rail_top', 1.7, 0.07, 0.02, (0, 1.935, front - 0.01), M.mahogany, bevel=0.004, bevel_segments=1)
        F.box('rail_low', 1.7, 0.04, 0.02, (0, 0.23, front - 0.01), M.mahogany, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            F.box(f'stile{s}', 0.04, 1.69, 0.02, (s * 0.83, 1.09, front - 0.006), M.mahogany, bevel=0.004,
                  bevel_segments=1)
            F.cyl(f'stile_bead{s}', 0.006, 1.6, (s * 0.836, 1.09, front + 0.004), M.mahogany, verts=6)
        # inside: a shelf, a rail of clothes on hangers and folded linen above
        zc = (back + front) / 2
        F.box('shelf', 1.64, 0.02, 0.5, (0, 1.7, zc - 0.01), M.mahogany)
        F.cyl('rail', 0.012, 1.6, (0, 1.6, zc), M.chrome, verts=10, rot=(0, 90, 0))
        _hanging('shirt', -0.72, 0.72, 1.6, zc, 9, depth=0.4, length=0.8)
        for i, x in enumerate((-0.5, 0.0, 0.5)):
            _folded(f'linen{i}_', x, 1.71, zc, 0.36, 0.34, 3)
    G.join(still, 'armoire', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            x = s * 0.4125
            holes = [[(u, v + dy) for u, v in G.rounded_rect(0.58, 0.56, 0.004, corner=1)] for dy in (0.34, -0.36)]
            F.extrude(f'door{s}', G.rounded_rect(0.815, 1.66, 0.004, corner=1), 0.025, (x, 1.08, front + 0.0125),
                      M.mahogany, holes=holes, bevel=0.004, bevel_segments=1)
            for dy in (0.34, -0.36):
                b = 0.07
                G.loft(f'panel{s}{dy}', [[F.at(x + u * hw, 1.08 + dy + v * hh, d) for u, v in ((1, 1), (-1, 1), (-1, -1), (1, -1))]
                                         for hw, hh, d in ((0.289, 0.279, front + 0.004), (0.289, 0.279, front + 0.017),
                                                           (0.289 - b, 0.279 - b, front + 0.03))], material=M.walnut,
                       smooth=False)
            _knob(f'knob{s}', 0.024, (s * 0.09, 1.08, front + 0.025), M.brass)
            if s > 0:
                F.soft('escutcheon', 0.026, 0.05, 0.004, (0.045, 1.0, front + 0.027), M.brass, round=0.0015,
                       round_segments=1)
                F.box('keyhole', 0.005, 0.016, 0.003, (0.045, 1.0, front + 0.029), M.mahogany2)
        doors.append(G.hinged(G.join(parts, f'door{"_left" if s < 0 else "_right"}'),
                              F.at(s * 0.82, 1.08, front + 0.0125)))
    G.settle(centre=False)
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


def _key(id):
    return f'model.{id}'


DRAWER_USE = {'uses': {'outfit': 'drawer_open'}, 'hold': DRAWER_HOLD}
# Open while the resident changes into another outfit (content `changesOutfit`), shut as they leave.
CHANGE_DRAWER = {'uses': {'outfit': 'drawer_open'}}
CHANGE_DOORS = {'uses': {'outfit': 'doors_open'}}
SNACK_DRAWER = {'uses': {'snack': 'drawer_open'}, 'hold': DRAWER_HOLD}   # the three-drawer nightstand's snack

FAMILY = G.Family(
    'bedroom_storage', M,
    # BR2: the three-drawer nightstand's top drawer slides out for a snack, the chest's lid lifts for a
    # blanket and the hamper's while the washing is sorted
    [G.Model('nightstand', nightstand, size=(0.54, 0.66, 0.468), key=_key('nightstand'), centre=(0, -0.1958),
             paint=[('oak', 'Body'), ('brass', 'Handles')]),
     G.Model('nightstandModern', nightstandModern, size=(0.58, 0.56, 0.468), key=_key('nightstandModern'),
             centre=(0, -0.196), paint=[('white', 'Body'), ('iron', 'Handles')]),
     G.Model('nightstandRound', nightstandRound, size=(0.54, 0.57, 0.56), key=_key('nightstandRound'),
             centre=(0, -0.0498), paint=[('clay', 'Body'), ('brass', 'Handles')]),
     G.Model('nightstandDrawers', nightstandDrawers, size=(0.58, 0.6, 0.468), key=_key('nightstandDrawers'),
             centre=(0, -0.196), motions=SNACK_DRAWER, paint=[('walnut', 'Body'), ('brass', 'Handles')]),
     G.Model('nightstandLamp', nightstandLamp, size=(0.54, 1.08, 0.468), key=_key('nightstandLamp'),
             centre=(0, -0.1958), paint=[('oak', 'Body'), ('shade', 'Shade')]),
     G.Model('cedarChest', cedarChest, size=(0.974, 0.505, 0.52), key=_key('cedarChest'),
             motions={'uses': {'takeBlanket': 'lid_open'}, 'hold': 2.0}, paint=[('cedar', 'Body'), ('iron', 'Handles')]),
     G.Model('laundryHamper', laundryHamper, size=(0.476, 0.631, 0.476), key=_key('laundryHamper'),
             centre=(0, -0.2), motions={'uses': {'sortWashing': 'lid_open'}}, paint=[('wicker', 'Hamper')]),
     G.Model('benchBedEnd', benchBedEnd, size=(1.6, 0.471, 0.42), key=_key('benchBedEnd'),
             paint=[('brass', 'Frame'), ('olive', 'Cushion')]),
     # BR3: each opens while a resident picks an outfit
     G.Model('dresser', dresser, size=(1.44, 1.349, 0.54), key=_key('dresser'), centre=(0, -0.16),
             motions=CHANGE_DRAWER, paint=[('oak', 'Body'), ('brass', 'Handles')]),
     G.Model('wardrobe', wardrobe, size=(1.84, 2.41, 0.626), key=_key('wardrobe'), centre=(0, -0.117),
             motions=CHANGE_DOORS,
             # (the white carcass is its body; the birch doors are their own part)
             paint=[('white', 'Body'), ('birch', 'Doors'), ('brass', 'Handles')]),
     G.Model('chestOfDrawers', chestOfDrawers, size=(0.88, 1.16, 0.51), key=_key('chestOfDrawers'),
             centre=(0, -0.175), motions=CHANGE_DRAWER, paint=[('linen', 'Body'), ('brass', 'Handles')]),
     G.Model('dresserLow', dresserLow, size=(1.82, 0.8, 0.635), key=_key('dresserLow'), centre=(0, -0.1125),
             motions=CHANGE_DRAWER, paint=[('oak', 'Body'), ('iron', 'Handles')]),
     G.Model('wardrobeSingle', wardrobeSingle, size=(0.6, 2.28, 0.617), key=_key('wardrobeSingle'),
             centre=(0, -0.1215), motions={'uses': {'outfit': 'door_open'}},
             paint=[('white', 'Body'), ('brass', 'Handles')]),
     G.Model('armoire', armoire, size=(1.86, 2.109, 0.628), key=_key('armoire'), centre=(0, -0.1162),
             motions=CHANGE_DOORS,
             paint=[('mahogany', 'Body'), ('brass', 'Handles')])],
    comment='Blender-built bedroom storage (tools/art/blender/families/bedroom_storage.py): the '
            'nightstands, the blanket chest, the laundry hamper and the bed-end bench (BR2), and the '
            'dressers, chests and wardrobes (BR3). Listed in the base manifest\'s packs. A dresser or '
            'chest slides its middle drawer out 30 cm (`drawer_open`), a wardrobe swings its doors open '
            '(`doors_open`, `door_open`) while a resident picks an outfit.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
