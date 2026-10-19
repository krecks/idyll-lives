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
# the three-drawer nightstand's top drawer slides out for an outfit too, the chest's lid lifts for a
# blanket and the hamper's while the washing is sorted; the rest are static.
#
# Run:  cargo art bedroom_storage [--preview]
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
    cream=('#EFE5D3', 'gloss'),      # a mug
    box=('#C9B79C', 'matte'),        # storage boxes on a wardrobe
    box2=('#E6DFD3', 'matte'),
    chrome=('#C9CCCF', 'chrome'),    # a hanging rail
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
DRAWER_HOLD = 2.5        # seconds it stays out once a resident starts choosing an outfit
DOOR_SECONDS = 1.2       # doors swing out 100 degrees in this long
DOOR_HOLD = 3.0          # seconds they stay open
PULL = 0.30              # how far a drawer slides out
CLOTHES = ('cloth1', 'cloth2', 'cloth3', 'cloth4')   # palette names (materials are made per model)


# --- shared pieces --------------------------------------------------------------------------------------
def _bar(name, w, h, at, material, d=0.02):
    """A bar handle (a bevelled stick) standing on the front at game point `at`."""
    F.box(name, w, h, d, at, material, bevel=min(w, h, d) * 0.4)


def _knob(name, r, at, material):
    """A round knob: a short neck and a flattened ball, its back at the game depth in `at`."""
    x, y, z = at
    F.cyl(f'{name}_neck', r * 0.45, r * 0.9, (x, y, z + r * 0.45), material, verts=10, rot=(90, 0, 0))
    F.sphere(f'{name}_ball', r, (x, y, z + r * 1.4), material, segments=12, rings=8, scale=(1, 1, 0.8))


def _folded(name, x, y, z, w, d, n, h=0.035):
    """A stack of `n` folded clothes, bottom at height `y`."""
    for i in range(n):
        F.box(f'{name}{i}', w - 0.01 * (i % 2), h, d, (x + 0.005 * (i % 2), y + h * (i + 0.5), z),
              getattr(M, CLOTHES[(i + int(x * 10)) % 4]), bevel=0.012)


def _hanging(name, x0, x1, rail_y, z, n, depth=0.42, length=0.78):
    """`n` shirts on hangers along a rail at `rail_y` from x0 to x1, hung side-on (across the depth)."""
    step = (x1 - x0) / max(1, n - 1)
    for i in range(n):
        x = x0 + i * step
        long = length - 0.12 * (i % 3 == 1)
        F.box(f'{name}{i}', 0.035, long, depth, (x, rail_y - 0.04 - long / 2, z), getattr(M, CLOTHES[i % 4]), bevel=0.015)
        F.box(f'{name}_hanger{i}', 0.012, 0.05, depth * 0.85, (x, rail_y - 0.035, z), M.chrome, bevel=0.004)


def _gap(name, w, h, x, y, front, material):
    """The dark opening a drawer leaves in the carcass when it slides out, on the carcass face behind the
    drawer front (part of the still body)."""
    F.box(f'{name}_gap', w - 0.02, h - 0.02, 0.002, (x, y, front + 0.001), material)


def _drawer(name, w, h, x, y, front, depth, material, pulls=(), contents=None):
    """A drawer front of `w` x `h` whose back is at game depth `front` (the carcass face), centred at
    (x, y), with the box behind it (inside the carcass at rest) and what's in it, as its own object with
    its origin on the carcass face: for `drawer_open`. `pulls` is [(make, args)] calls."""
    z = front + 0.01
    with G.new_parts() as parts:
        F.box(f'{name}_front', w, h, 0.02, (x, y, z), material, bevel=0.006)
        for make, args in pulls:
            make(*args)
        bh = h - 0.05                         # the box's sides, below the front's top edge
        by = y - h / 2 + 0.015 + bh / 2
        back = z - 0.01 - depth
        F.box(f'{name}_bottom', w - 0.05, 0.012, depth, (x, y - h / 2 + 0.02, z - 0.01 - depth / 2), material)
        for s in (-1, 1):
            F.box(f'{name}_side{s}', 0.014, bh, depth, (x + s * (w / 2 - 0.032), by, z - 0.01 - depth / 2),
                  material)
        F.box(f'{name}_back', w - 0.05, bh, 0.014, (x, by, back + 0.007), material)
        if contents:
            contents(x, y - h / 2 + 0.026, z - 0.01 - depth / 2)
    return G.join(parts, name, origin=F.at(x, y, front))


def _shell(w, y0, y1, back, front, material, t=0.02, name='shell'):
    """An open-fronted carcass: two sides, a top, a bottom and a back, `t` thick."""
    d, zc, h, yc = front - back, (back + front) / 2, y1 - y0, (y0 + y1) / 2
    for s in (-1, 1):
        F.box(f'{name}_side{s}', t, h, d, (s * (w - t) / 2, yc, zc), material, bevel=0.004)
    F.box(f'{name}_top', w - 2 * t, t, d, (0, y1 - t / 2, zc), material, bevel=0.003)
    F.box(f'{name}_bottom', w - 2 * t, t, d, (0, y0 + t / 2, zc), material, bevel=0.003)
    F.box(f'{name}_back', w - 2 * t, h - 2 * t, t, (0, yc, back + t / 2), material)


def _books(name, x0, y, z, n, h=0.2, d=0.15):
    """A row of `n` books standing on a shelf at height `y`, from x0 rightwards."""
    x = x0
    mats = (M.teal, M.book2, M.book3)
    for i in range(n):
        t = 0.025 + 0.008 * (i % 3)
        hh = h - 0.02 * (i % 2)
        F.box(f'{name}{i}', t, hh, d, (x + t / 2, y + hh / 2, z), mats[i % 3], bevel=0.003)
        x += t + 0.003


# --- BR2: the nightstand: oak, a drawer over an open niche, on four short tapered legs -----------------
def _classic_nightstand(decor=True):
    front = -0.02
    d, zc = front - WALL, (WALL + front) / 2
    for sx in (-1, 1):
        for z in (WALL + 0.035, front - 0.035):
            F.cone(f'leg{sx}{z}', 0.022, 0.015, 0.145, (sx * 0.22, 0.0725, z), M.oak, verts=12)
    _shell(0.52, 0.14, 0.535, WALL, front, M.oak, name='case')
    F.box('shelf', 0.48, 0.02, d - 0.02, (0, 0.37, zc + 0.01), M.oak)
    F.box('drawer_case', 0.48, 0.135, d - 0.02, (0, 0.4575, zc - 0.01), M.oak)
    F.box('drawer', 0.48, 0.14, 0.02, (0, 0.45, front + 0.01), M.oak, bevel=0.006)
    _knob('knob', 0.016, (0, 0.45, front + 0.02), M.brass)
    F.box('top', 0.54, 0.025, 0.44, (0, 0.5475, WALL + 0.22), M.oak, bevel=0.006)
    _books('niche_book', -0.2, 0.16, -0.22, 4, h=0.17, d=0.15)
    if decor:
        F.box('book', 0.16, 0.03, 0.22, (0.1, 0.575, -0.16), M.teal, bevel=0.004)
        F.cyl('mug', 0.04, 0.1, (-0.12, 0.61, -0.12), M.cream, verts=16, bevel=0.004)
        G.torus('mug_handle', 0.025, 0.007, at=F.at(-0.165, 0.615, -0.12), material=M.cream, rot=(90, 0, 0),
                major_segments=12, minor_segments=6)


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
        G.revolve('lamp_base', [(0, 0), (0.045, 0), (0.068, 0.035), (0.08, 0.085), (0.066, 0.135),
                                (0.03, 0.168), (0.022, 0.18), (0, 0.18)],
                  at=F.at(0, 0.56, z), material=M.porcelain, segments=20)
        F.cyl('lamp_stem', 0.008, 0.13, (0, 0.80, z), M.brass, verts=10)
        F.cyl('lamp_socket', 0.016, 0.04, (0, 0.88, z), M.brass, verts=12)
        F.sphere('lamp_bulb', 0.035, (0, 0.935, z), M.bulb, segments=14, rings=8, scale=(1, 1, 1.15))
        # the drum shade, open at both ends (a ring of thickness), and the spider holding it
        G.revolve('lamp_shade', [(0.15, 0), (0.132, 0.22), (0.126, 0.22), (0.144, 0.0), (0.15, 0)],
                  at=F.at(0, 0.86, z), material=M.shade, segments=28, smooth=False)
        G.torus('lamp_ring', 0.128, 0.003, at=F.at(0, 1.075, z), material=M.brass, major_segments=20,
                minor_segments=4)
        for a in (0, 120, 240):
            G.rod(f'lamp_spoke{a}', F.at(0, 1.075, z), F.at(0.128 * G.cosd(a), 1.075, z + 0.128 * G.sind(a)),
                  0.0025, material=M.brass, verts=4)
    G.join(still, 'nightstandLamp', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the modern nightstand: white, an oak top and legs, a drawer over an open niche of books -----------
def nightstandModern():
    with G.new_parts() as still:
        front, back = 0.16, -0.42
        for sx in (-1, 1):
            for z in (-0.37, 0.11):
                F.cone(f'leg{sx}{z}', 0.018, 0.013, 0.125, (sx * 0.23, 0.0625, z), M.oak, verts=12)
        _shell(0.56, 0.12, 0.53, back, front, M.white, name='case')
        F.box('shelf', 0.52, 0.02, 0.56, (0, 0.37, (back + front) / 2 + 0.01), M.white)
        F.box('drawer_case', 0.52, 0.14, 0.54, (0, 0.45, (back + front) / 2 - 0.01), M.white)
        F.box('drawer', 0.52, 0.135, 0.02, (0, 0.445, front + 0.01), M.white, bevel=0.005)
        _bar('pull', 0.12, 0.014, (0, 0.49, front + 0.03), M.iron, d=0.016)
        F.box('top', 0.58, 0.03, 0.6, (0, 0.545, -0.13), M.oak, bevel=0.008)
        _books('niche_book', -0.22, 0.14, -0.1, 5, h=0.19, d=0.16)
        F.box('niche_box', 0.14, 0.09, 0.16, (0.15, 0.185, -0.08), M.box, bevel=0.01)
    G.join(still, 'nightstandModern', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the drum nightstand: a terracotta drum with a drawer band, an oak top and a round foot ------------
def nightstandRound():
    z = -0.06
    with G.new_parts() as still:
        F.cyl('foot', 0.2, 0.04, (0, 0.02, z), M.clay2, verts=28, bevel=0.006)
        F.cyl('drum', 0.25, 0.5, (0, 0.29, z), M.clay, verts=36, bevel=0.01)
        F.cyl('band', 0.254, 0.12, (0, 0.42, z), M.clay2, verts=36, bevel=0.004)
        for y in (0.358, 0.482):
            G.torus(f'groove{y}', 0.254, 0.004, at=F.at(0, y, z), material=M.plinth, major_segments=36,
                    minor_segments=4)
        _knob('knob', 0.016, (0, 0.42, z + 0.252), M.brass)
        F.cyl('top', 0.27, 0.03, (0, 0.555, z), M.oak, verts=36, bevel=0.008)
    G.join(still, 'nightstandRound', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the three-drawer nightstand: walnut, three drawers with brass bars; the top one a part of its own ---
def nightstandDrawers():
    front, back = 0.16, -0.42
    with G.new_parts() as still:
        F.box('plinth', 0.52, 0.05, 0.54, (0, 0.025, (back + front) / 2 - 0.01), M.walnut, bevel=0.004)
        F.box('case', 0.56, 0.52, 0.58, (0, 0.31, (back + front) / 2), M.walnut, bevel=0.008)
        F.box('top', 0.58, 0.03, 0.6, (0, 0.585, -0.13), M.walnut, bevel=0.008)
        for i, y in enumerate((0.335, 0.185)):
            F.box(f'drawer{i}', 0.52, 0.135, 0.02, (0, y, front + 0.01), M.walnut, bevel=0.005)
            _bar(f'pull{i}', 0.12, 0.014, (0, y + 0.02, front + 0.03), M.brass, d=0.016)
        _gap('drawer', 0.52, 0.135, 0, 0.485, front, M.plinth)
    G.join(still, 'nightstandDrawers', origin=(0, 0, 0))
    # the top drawer with folded clothes in it, sliding out while a resident picks an outfit
    drawer = _drawer('drawer', 0.52, 0.135, 0, 0.485, front, 0.44, M.walnut,
                     pulls=[(_bar, ('pull_top', 0.12, 0.014, (0, 0.505, front + 0.03), M.brass, 0.016))],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 0.44, 0.34))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the blanket chest: a cedar box on a plinth with iron straps and side handles; the lid a part --------
def cedarChest():
    w, d, y0, y1 = 0.9, 0.46, 0.06, 0.455
    t = 0.025
    with G.new_parts() as still:
        F.box('plinth', 0.94, 0.06, 0.5, (0, 0.03, 0), M.cedar2, bevel=0.008)
        for s in (-1, 1):
            F.box(f'wall_z{s}', w, y1 - y0, t, (0, (y0 + y1) / 2, s * (d - t) / 2), M.cedar, bevel=0.004)
            F.box(f'wall_x{s}', t, y1 - y0, d - 2 * t, (s * (w - t) / 2, (y0 + y1) / 2, 0), M.cedar, bevel=0.004)
        F.box('floor', w - 2 * t, 0.02, d - 2 * t, (0, y0 + 0.01, 0), M.cedarin)
        for s in (-1, 1):
            F.box(f'strap{s}', 0.04, 0.38, 0.008, (s * 0.3, 0.255, d / 2 + 0.004), M.iron, bevel=0.002)
            # a side handle: a bar on two posts
            F.box(f'handle{s}', 0.014, 0.022, 0.16, (s * (w / 2 + 0.03), 0.33, 0), M.iron, bevel=0.005)
            for p in (-1, 1):
                F.box(f'handle_post{s}{p}', 0.03, 0.022, 0.014, (s * (w / 2 + 0.015), 0.33, p * 0.073), M.iron)
        F.box('latch', 0.08, 0.06, 0.012, (0, 0.415, d / 2 + 0.006), M.brass, bevel=0.003)
        F.box('blanket', w - 2 * t - 0.02, 0.12, d - 2 * t - 0.02, (0, y0 + 0.08, 0), M.cloth3, bevel=0.03)
    G.join(still, 'cedarChest', origin=(0, 0, 0))
    # the lid, hinged along its back edge: it lifts while a resident takes out a blanket (`lid_open`)
    with G.new_parts() as parts:
        # (its underside 2 mm below the edge band's, not in its plane: seen when the lid is up)
        F.box('lid', 0.94, 0.047, 0.5, (0, 0.4765, 0), M.cedar, bevel=0.008)
        F.box('lid_edge', 0.96, 0.012, 0.52, (0, 0.461, 0), M.cedar2, bevel=0.004)
        F.box('lid_panel', 0.8, 0.006, 0.36, (0, 0.502, 0), M.cedar2, bevel=0.002)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.455, -0.25))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -95, seconds=0.9, axis='X')


# --- the laundry hamper: a woven basket, wider at the top, with bands; the lid a part ----------------------
def laundryHamper():
    z = -0.2
    with G.new_parts() as still:
        G.revolve('basket', [(0, 0), (0.2, 0), (0.231, 0.56), (0.217, 0.56), (0.188, 0.025), (0, 0.025)],
                  at=F.at(0, 0, z), material=M.wicker, segments=28)
        for y in (0.04, 0.2, 0.4):
            G.torus(f'band{y}', 0.2 + 0.031 * y / 0.56 + 0.003, 0.009, at=F.at(0, y, z), material=M.wicker2,
                    major_segments=28, minor_segments=6)
        G.torus('rim', 0.226, 0.012, at=F.at(0, 0.552, z), material=M.wicker2, major_segments=28,
                minor_segments=6)
        # the washing in it, near the top, and a sleeve hanging out under the lid
        for i, (x, zz, y, m) in enumerate(((-0.05, -0.04, 0.44, M.cloth1), (0.07, 0.05, 0.46, M.cloth2),
                                           (0.0, 0.07, 0.42, M.cloth3))):
            F.sphere(f'washing{i}', 0.13, (x, y, z + zz), m, segments=12, rings=6, scale=(1, 0.9, 0.4))
        F.box('sleeve', 0.06, 0.16, 0.01, (0.03, 0.49, z + 0.231), M.cloth1, bevel=0.004, rot=(0, 8, 0))
    G.join(still, 'laundryHamper', origin=(0, 0, 0))
    # the lid, hinged at its back: it lifts while a resident sorts the washing
    with G.new_parts() as parts:
        F.cyl('lid', 0.236, 0.04, (0, 0.58, z), M.wicker2, verts=28, bevel=0.012)
        F.sphere('lid_knob', 0.03, (0, 0.61, z), M.wicker2, segments=12, rings=6, scale=(1, 1, 0.7))
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.56, z - 0.236))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -70, seconds=0.7, axis='X')   # not past upright: it stands at the wall


# --- the bed-end bench: a buttoned olive cushion on a slim brass frame ------------------------------------
def benchBedEnd():
    with G.new_parts() as still:
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.cone(f'leg{sx}{sz}', 0.015, 0.012, 0.335, (sx * 0.74, 0.1675, sz * 0.16), M.brass, verts=10)
            # the end rails and a low rail between the legs
            F.box(f'end{sx}', 0.02, 0.02, 0.34, (sx * 0.74, 0.335, 0), M.brass, bevel=0.004)
            F.box(f'end_low{sx}', 0.012, 0.012, 0.32, (sx * 0.74, 0.09, 0), M.brass)
        for sz in (-1, 1):
            F.box(f'rail{sz}', 1.5, 0.02, 0.02, (0, 0.335, sz * 0.16), M.brass, bevel=0.004)
        F.box('stretcher', 1.46, 0.012, 0.012, (0, 0.09, 0), M.brass)
        F.box('cushion', 1.6, 0.125, 0.42, (0, 0.4075, 0), M.olive, bevel=0.035, bevel_segments=3)
        for row, z in enumerate((-0.09, 0.09)):
            for i in range(6 - row):
                x = -0.625 + 0.25 * i + 0.125 * row
                F.sphere(f'button{row}{i}', 0.016, (x, 0.462, z), M.olive2, segments=10, rings=5,
                         scale=(1, 1, 0.55))
    G.join(still, 'benchBedEnd', origin=(0, 0, 0))
    G.settle(centre=False)


# --- BR3: the dresser: oak, two small drawers over two wide ones; the middle (wide) one slides out -------
def _drawer_clothes(x, y, z, w=1.2, d=0.36):
    """Folded clothes in a drawer: stacks across its width, bottom at `y`."""
    n = max(2, int(w / 0.3))
    step = (w - 0.1) / n
    for i in range(n):
        _folded(f'folded{i}_', x - (w - 0.1) / 2 + step * (i + 0.5), y, z, step - 0.03, d, 2 + (i % 2))


def dresser():
    front = 0.07
    with G.new_parts() as still:
        for sx in (-1, 1):
            for z in (WALL + 0.03, front - 0.03):
                F.box(f'foot{sx}{z}', 0.05, 0.06, 0.05, (sx * 0.665, 0.03, z), M.oak, bevel=0.008)
        F.box('plinth', 1.4, 0.03, 0.48, (0, 0.075, (WALL + front) / 2), M.oak, bevel=0.006)
        F.box('case', 1.4, 0.77, 0.5, (0, 0.475, (WALL + front) / 2), M.oak, bevel=0.008)
        F.box('top', 1.44, 0.03, 0.52, (0, 0.875, -0.17), M.oak, bevel=0.008)
        for s in (-1, 1):
            F.box(f'drawer_top{s}', 0.66, 0.2, 0.02, (s * 0.35, 0.72, front + 0.01), M.oak, bevel=0.006)
            _bar(f'pull_top{s}', 0.12, 0.018, (s * 0.35, 0.75, front + 0.03), M.brass)
            _bar(f'pull_low{s}', 0.12, 0.018, (s * 0.35, 0.27, front + 0.03), M.brass)
        F.box('drawer_low', 1.36, 0.2, 0.02, (0, 0.24, front + 0.01), M.oak, bevel=0.006)
        # on top: a framed picture, a turned vase and a box
        F.box('picture_frame', 0.36, 0.46, 0.03, (0.32, 1.12, -0.3), M.frame, bevel=0.008, rot=(-6, 0, 0))
        F.box('picture', 0.29, 0.38, 0.006, (0.32, 1.12, -0.283), M.print1, rot=(-6, 0, 0))
        G.revolve('vase', [(0, 0), (0.045, 0), (0.06, 0.06), (0.05, 0.15), (0.028, 0.2), (0.034, 0.22),
                           (0, 0.22)], at=F.at(-0.45, 0.89, -0.2), material=M.vase, segments=18)
        F.box('jewellery_box', 0.22, 0.07, 0.16, (-0.12, 0.925, -0.16), M.pull, bevel=0.01)
        _gap('drawer', 1.36, 0.2, 0, 0.48, front, M.ebony)
    G.join(still, 'dresser', origin=(0, 0, 0))
    drawer = _drawer('drawer', 1.36, 0.2, 0, 0.48, front, 0.44, M.oak,
                     pulls=[(_bar, (f'pull_mid{s}', 0.12, 0.018, (s * 0.35, 0.51, front + 0.03), M.brass))
                            for s in (-1, 1)],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 1.2, 0.36))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the wardrobe: a white carcass, two birch doors with brass bars, a rail of shirts and shelves --------
def _wardrobe_inside(w, y0, y1, back, front, rail_y, split=True):
    """A wardrobe's inside: a rail of shirts (the left half when `split`, with shelves of folded clothes
    on the right) under a shelf."""
    d, zc = front - back, (back + front) / 2
    hang_x1 = -0.02 if split else w / 2 - 0.08
    F.box('hat_shelf', (w / 2 if split else w) - 0.04, 0.02, d - 0.04,
          (-w / 4 if split else 0, rail_y + 0.12, zc - 0.01), M.white)
    F.cyl('rail', 0.012, (w / 2 if split else w) - 0.06, (-w / 4 if split else 0, rail_y, zc), M.chrome,
          verts=10, rot=(0, 90, 0))
    n = 5 if split else max(3, int(w / 0.16))
    _hanging('shirt', -w / 2 + 0.1, hang_x1 - 0.08, rail_y, zc, n, depth=min(0.42, d - 0.1))
    F.box('folded_box', 0.3, 0.2, 0.3, (-w / 4 if split else -w / 4, rail_y + 0.23, zc), M.box, bevel=0.01)
    if split:
        F.box('divider', 0.02, y1 - y0 - 0.04, d - 0.02, (0, (y0 + y1) / 2, zc - 0.01), M.white)
        for i, y in enumerate((y0 + 0.45, y0 + 0.9, y0 + 1.35)):
            F.box(f'shelf{i}', w / 2 - 0.04, 0.02, d - 0.04, (w / 4, y, zc - 0.01), M.white)
            _folded(f'stack{i}_', w / 4 - 0.18, y + 0.01, zc, 0.3, 0.32, 3)
            _folded(f'stackb{i}_', w / 4 + 0.18, y + 0.01, zc, 0.3, 0.32, 2 + i % 2)


def wardrobe():
    back, front = WALL, 0.15
    with G.new_parts() as still:
        F.box('plinth', 1.80, 0.06, 0.54, (0, 0.03, (back + front) / 2 - 0.01), M.plinth, bevel=0.004)
        _shell(1.84, 0.06, 2.11, back, front, M.white, t=0.025, name='case')
        _wardrobe_inside(1.84, 0.06, 2.11, back, front, 1.8)
        F.box('box_big', 0.4, 0.3, 0.32, (-0.5, 2.26, -0.18), M.box, bevel=0.012)
        F.box('box_small', 0.32, 0.22, 0.3, (-0.08, 2.22, -0.18), M.box2, bevel=0.012)
    G.join(still, 'wardrobe', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            F.box(f'door{s}', 0.905, 2.04, 0.02, (s * 0.4575, 1.085, front + 0.01), M.birch, bevel=0.006)
            F.box(f'door_panel{s}', 0.7, 1.76, 0.006, (s * 0.4575, 1.085, front + 0.023), M.birch, bevel=0.003)
            _bar(f'handle{s}', 0.02, 0.4, (s * 0.06, 1.1, front + 0.035), M.brass, d=0.022)
        doors.append(G.hinged(G.join(parts, f'door{"_left" if s < 0 else "_right"}'),
                              F.at(s * 0.91, 1.085, front + 0.01)))
    G.settle(centre=False)
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


# --- the tall chest of drawers: linen-white, five drawers with brass bars, on a dark plinth --------------
def chestOfDrawers():
    front = 0.185
    with G.new_parts() as still:
        F.box('plinth', 0.82, 0.06, 0.58, (0, 0.03, (WALL + front) / 2 - 0.01), M.plinth, bevel=0.004)
        F.box('case', 0.86, 1.07, front - WALL, (0, 0.595, (WALL + front) / 2), M.linen, bevel=0.008)
        F.box('top', 0.88, 0.03, 0.63, (0, 1.145, (WALL + 0.2) / 2), M.linen, bevel=0.008)
        for i, y in enumerate((0.179, 0.387, 0.803, 1.011)):
            F.box(f'drawer{i}', 0.815, 0.194, 0.02, (0, y, front + 0.01), M.linen, bevel=0.006)
            _bar(f'pull{i}', 0.16, 0.014, (0, y + 0.031, front + 0.03), M.brass)
        _gap('drawer', 0.815, 0.194, 0, 0.595, front, M.plinth)
    G.join(still, 'chestOfDrawers', origin=(0, 0, 0))
    drawer = _drawer('drawer', 0.815, 0.194, 0, 0.595, front, 0.5, M.linen,
                     pulls=[(_bar, ('pull_mid', 0.16, 0.014, (0, 0.626, front + 0.03), M.brass))],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 0.72, 0.4))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the low dresser: oak, nine drawers in three columns on tapered legs; the middle one slides out ------
def dresserLow():
    front = 0.165
    ys, xs = (0.2217, 0.435, 0.6483), (-0.59, 0.0, 0.59)
    with G.new_parts() as still:
        for sx in (-1, 1):
            for z in (-0.39, 0.12):
                F.cone(f'leg{sx}{z}', 0.022, 0.016, 0.1, (sx * 0.84, 0.05, z), M.oakleg, verts=12)
        F.box('case', 1.8, 0.67, front - WALL, (0, 0.435, (WALL + front) / 2), M.oak, bevel=0.008)
        F.box('top', 1.82, 0.03, 0.61, (0, 0.785, (WALL + 0.18) / 2), M.oak, bevel=0.008)
        for x in xs:
            for y in ys:
                if (x, y) == (0.0, 0.435):
                    continue
                F.box(f'drawer{x}{y}', 0.575, 0.1993, 0.02, (x, y, front + 0.01), M.oak, bevel=0.006)
                _bar(f'pull{x}{y}', 0.16, 0.014, (x, y + 0.032, front + 0.03), M.iron)
        _gap('drawer', 0.575, 0.1993, 0, 0.435, front, M.ebony)
    G.join(still, 'dresserLow', origin=(0, 0, 0))
    drawer = _drawer('drawer', 0.575, 0.1993, 0, 0.435, front, 0.5, M.oak,
                     pulls=[(_bar, ('pull_mid', 0.16, 0.014, (0, 0.467, front + 0.03), M.iron))],
                     contents=lambda x, y, z: _drawer_clothes(x, y, z, 0.5, 0.4))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -PULL, 0), seconds=DRAWER_SECONDS)


# --- the single wardrobe: white, one door hinged on the left with a brass bar, a rail of shirts ----------
def wardrobeSingle():
    back, front = WALL, 0.14
    with G.new_parts() as still:
        F.box('plinth', 0.86, 0.05, 0.53, (0, 0.025, (back + front) / 2 - 0.01), M.plinth, bevel=0.004)
        _shell(0.9, 0.05, 2.04, back, front, M.white, t=0.022, name='case')
        _wardrobe_inside(0.9, 0.05, 2.04, back, front, 1.74, split=False)
        F.box('box_top', 0.3, 0.24, 0.3, (-0.2, 2.16, -0.2), M.box, bevel=0.012)
    G.join(still, 'wardrobeSingle', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.box('door', 0.88, 1.97, 0.02, (0, 1.045, front + 0.01), M.white, bevel=0.006)
        F.box('door_panel', 0.7, 1.72, 0.006, (0, 1.045, front + 0.023), M.white, bevel=0.003)
        _bar('handle', 0.02, 0.5, (0.36, 1.1, front + 0.035), M.brass, d=0.024)
    door = G.hinged(G.join(parts, 'door'), F.at(-0.44, 1.045, front + 0.01))
    G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=DOOR_SECONDS)


# --- the armoire: mahogany, a cornice, raised-panel doors with brass knobs, on bun feet -------------------
def armoire():
    back, front = WALL, 0.115
    with G.new_parts() as still:
        for sx in (-1, 1):
            for z in (-0.38, 0.08):
                F.sphere(f'foot{sx}{z}', 0.045, (sx * 0.8, 0.06, z), M.mahogany2, segments=12, rings=8,
                         scale=(1, 1, 1.3))
        F.box('base', 1.74, 0.1, 0.59, (0, 0.17, (back + 0.16) / 2), M.mahogany2, bevel=0.012)
        _shell(1.7, 0.22, 1.97, back, front, M.mahogany, t=0.03, name='case')
        F.box('cornice', 1.82, 0.1, 0.6, (0, 2.02, (back + 0.17) / 2), M.mahogany, bevel=0.015)
        F.box('crown', 1.86, 0.04, 0.62, (0, 2.09, (back + 0.19) / 2), M.mahogany2, bevel=0.01)
        # the face frame's rails above and below the doors
        F.box('rail_top', 1.7, 0.07, 0.02, (0, 1.935, front - 0.01), M.mahogany, bevel=0.004)
        F.box('rail_low', 1.7, 0.04, 0.02, (0, 0.23, front - 0.01), M.mahogany, bevel=0.004)
        # inside: a shelf, a rail of shirts and folded linen below
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
            F.box(f'door{s}', 0.815, 1.66, 0.025, (s * 0.4125, 1.08, front + 0.0125), M.mahogany, bevel=0.008)
            for y in (1.42, 0.72):
                F.box(f'panel{s}{y}', 0.62, 0.6, 0.012, (s * 0.4125, y, front + 0.031), M.walnut, bevel=0.012)
            _knob(f'knob{s}', 0.024, (s * 0.09, 1.08, front + 0.025), M.brass)
        doors.append(G.hinged(G.join(parts, f'door{"_left" if s < 0 else "_right"}'),
                              F.at(s * 0.82, 1.08, front + 0.0125)))
    G.settle(centre=False)
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


def _key(id):
    return f'model.{id}'


DRAWER_USE = {'uses': {'outfit': 'drawer_open'}, 'hold': DRAWER_HOLD}

FAMILY = G.Family(
    'bedroom_storage', M,
    # BR2: the three-drawer nightstand's top drawer slides out for an outfit, the chest's lid lifts for a
    # blanket and the hamper's while the washing is sorted
    [G.Model('nightstand', nightstand, size=(0.54, 0.66, 0.468), key=_key('nightstand'), centre=(0, -0.1958),
             paint=[('oak', 'Body'), ('brass', 'Handles')]),
     G.Model('nightstandModern', nightstandModern, size=(0.58, 0.56, 0.628), key=_key('nightstandModern'),
             centre=(0, -0.116), paint=[('white', 'Body'), ('iron', 'Handles')]),
     G.Model('nightstandRound', nightstandRound, size=(0.54, 0.57, 0.56), key=_key('nightstandRound'),
             centre=(0, -0.0498), paint=[('clay', 'Body'), ('brass', 'Handles')]),
     G.Model('nightstandDrawers', nightstandDrawers, size=(0.58, 0.6, 0.628), key=_key('nightstandDrawers'),
             centre=(0, -0.116), motions=DRAWER_USE, paint=[('walnut', 'Body'), ('brass', 'Handles')]),
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
             motions=DRAWER_USE, paint=[('oak', 'Body'), ('brass', 'Handles')]),
     G.Model('wardrobe', wardrobe, size=(1.84, 2.41, 0.626), key=_key('wardrobe'), centre=(0, -0.117),
             motions={'uses': {'outfit': 'doors_open'}, 'hold': DOOR_HOLD},
             # (the white carcass is its body; the birch doors are their own part)
             paint=[('white', 'Body'), ('birch', 'Doors'), ('brass', 'Handles')]),
     G.Model('chestOfDrawers', chestOfDrawers, size=(0.88, 1.16, 0.655), key=_key('chestOfDrawers'),
             centre=(0, -0.1025), motions=DRAWER_USE, paint=[('linen', 'Body'), ('brass', 'Handles')]),
     G.Model('dresserLow', dresserLow, size=(1.82, 0.8, 0.635), key=_key('dresserLow'), centre=(0, -0.1125),
             motions=DRAWER_USE, paint=[('oak', 'Body'), ('iron', 'Handles')]),
     G.Model('wardrobeSingle', wardrobeSingle, size=(0.9, 2.28, 0.617), key=_key('wardrobeSingle'),
             centre=(0, -0.1215), motions={'uses': {'outfit': 'door_open'}, 'hold': DOOR_HOLD},
             paint=[('white', 'Body'), ('brass', 'Handles')]),
     G.Model('armoire', armoire, size=(1.86, 2.109, 0.628), key=_key('armoire'), centre=(0, -0.1162),
             motions={'uses': {'outfit': 'doors_open'}, 'hold': DOOR_HOLD},
             paint=[('mahogany', 'Body'), ('brass', 'Handles')])],
    comment='Blender-built bedroom storage (tools/art/blender/families/bedroom_storage.py): the '
            'nightstands, the blanket chest, the laundry hamper and the bed-end bench (BR2), and the '
            'dressers, chests and wardrobes (BR3). Listed in the base manifest\'s packs. A dresser or '
            'chest slides its middle drawer out 30 cm (`drawer_open`), a wardrobe swings its doors open '
            '(`doors_open`, `door_open`) while a resident picks an outfit.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
