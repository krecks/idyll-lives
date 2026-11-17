# The named places' fittings (content/civic.json): the reception, hotel and duty desks, the bank's teller counter,
# the teacher's desk, the church's lectern, pews and altar, the pub's bar, the pupil's desk, the waiting room's
# bench, the consulting room's couch, the shops' clothes rail, plant table and mannequin, the cash machine, the
# sports hall's wall bars, the changing room's lockers, the hotel's luggage trolley and the bank's vault door.
# Notes: civic.md.
#
# The parts are placed in game axes (`F`), where the game places them. Desks and counters with `staff` face the
# customer at the front (+z); whoever serves stands behind them (content `staff.at`, 0.85 m back), so their
# monitors, drawers and tills face the back. Seats keep the content's heights: the pew's and the waiting bench's
# 0.45 m, slots spread across their three tiles; the pupil's desk and the couch the default seat's (0.45 m, at
# the footprint's middle). Things against a wall (the lockers, the cash machine, the wall bars, the vault door)
# have their backs on the wall face (-0.43). Nothing moves: none of their uses opens anything.
#
# Run:  cargo art civic [--only pew] [--preview]
import json
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from mathutils import Vector  # noqa: E402

M = G.Palette(
    # the reception desk
    desk=('#ECEAE5', 'satin'),        # its white carcass and panels; the main part the player colours
    oak=('#C9A77C', 'wood'),          # its oak ledge and band; the teacher's desk; the wall bars
    plinth=('#3A3D41', 'satin'),      # dark kick plinths and recesses
    monitor=('#2B2C2E', 'satin'),     # monitors, keyboards' cases, phones
    screen=('#1E2A36', 'gloss'),
    keys=('#D9DADB', 'satin'),        # keyboards and mice
    acrylic=('#DCE6EA', 'gloss'),     # the leaflet stand (opaque gloss: glass would cost a draw call)
    leaflet1=('#3E8EDE', 'gloss'), leaflet2=('#F2C14E', 'gloss'), leaflet3=('#5EC28B', 'gloss'),
    leaflet4=('#D9483B', 'gloss'),
    bell=('#C8CCD0', 'chrome'),       # service bells, the clothes rail, the mannequin's stand
    pot=('#F5F3EE', 'gloss'),         # plant pots
    soil=('#4A3A2C', 'matte'),
    leaf=('#5E8F4E', 'satin'),        # a desk plant's leaves
    # the hotel's desk
    walnut=('#5E3B26', 'wood'),       # its walnut carcass and panels; the main part the player colours
    marble=('#EDEAE4', 'gloss', {'roughness': 0.28}),   # its ledge
    brass=('#C9A54A', 'metal'),       # its kick strip, lamp and hooks; the bar's rail; the trolley; the altar's
    book=('#7A1F24', 'satin'),        # the guest book's and the register's covers, hymn books
    page=('#F2EEE3', 'matte'),        # pages
    shade=('#2F6B4A', 'gloss', {'glow': '#FFD9A0'}),    # the banker's lamp's green shade
    fob=('#B8322B', 'satin'),         # the room keys' fobs
    # the duty desk
    counter=('#53657A', 'satin'),     # its blue-grey carcass; the main part the player colours
    laminate=('#D9D6CE', 'satin'),    # its ledge and worktop
    glass=('#DDEBF0', 'glass'),       # the security screens
    frame=('#B9BDC1', 'metal'),       # aluminium: screen frames, the teller's grille, the locker frames
    police=('#24407A', 'gloss'),      # the duty desk's chequered band
    radio=('#1E1F22', 'satin'),
    # the teller counter
    teller=('#B88A5A', 'wood'),       # its oak front and screen rails; the main part the player colours
    note=('#8FB98A', 'matte'),        # banknotes
    coin=('#C9A54A', 'metal'),
    # the bar
    mahogany=('#5A3220', 'wood'),     # its carcass, panels and top; the main part the player colours
    handle=('#F2EEE3', 'gloss'),      # the pumps' ceramic handles
    clip1=('#2E5470', 'gloss'), clip2=('#B8322B', 'gloss'), clip3=('#D6A64A', 'gloss'),   # pump clips
    beer=('#B8741E', 'gloss'),        # a pint (opaque: glass would cost a draw call)
    froth=('#F4EBD6', 'matte'),
    towel=('#3E5470', 'matte'),       # the bar runner, beer mats
    # the pupil's desk
    pine=('#C9A26B', 'wood'),         # its desk and seat; the main part the player colours
    iron=('#2B302E', 'metal'),        # its cast-iron frame; the vault door's wheel
    ink=('#1E2A4A', 'gloss'),
    # the pew and the altar
    pew_oak=('#7A5234', 'wood'),      # the pew; the lectern
    stone=('#D8D2C4', 'satin'),       # the altar's stone
    linen=('#F7F3EA', 'fabric'),      # its fair linen
    frontal=('#5B2A6E', 'fabric'),    # its frontal
    candle=('#F7F3EA', 'satin'),
    flame=('#FFC85A', 'gloss', {'glow': '#FFC070'}),
    # the waiting bench
    shell=('#3F8C8C', 'satin'),       # its moulded seats; the main part the player colours
    steel=('#9EA3A8', 'metal'),       # its beam and legs
    mag1=('#3E5470', 'gloss'), mag2=('#D9483B', 'gloss'), mag3=('#D6A64A', 'gloss'), mag4=('#9DAF8C', 'gloss'),
    # the couch
    vinyl=('#7FA7B8', 'satin'),       # its wipe-clean pad; the main part the player colours
    white=('#EEF0F0', 'satin'),       # its frame
    paper=('#FFFFFF', 'matte'),       # its paper sheet
    # the clothes rail and the mannequin
    cloth1=('#C0392B', 'fabric'), cloth2=('#2B3A55', 'fabric'), cloth3=('#E9C46A', 'fabric'),
    cloth4=('#F2EEE3', 'fabric'), cloth5=('#5E8F6E', 'fabric'), cloth6=('#C98F6B', 'fabric'),
    dummy=('#F2F0EA', 'gloss'),       # the mannequin's body
    # the plant table
    slat=('#B08850', 'wood'),         # its slats; the main part the player colours
    galv=('#A9AEB1', 'metal'),        # its galvanised frame
    terracotta=('#C0663E', 'satin'),
    foliage=('#4F7F3F', 'foliage'),
    bloom1=('#E9A6B4', 'gloss'), bloom2=('#F2C14E', 'gloss'), bloom3=('#8E6CC9', 'gloss'),
    label=('#F7F5EE', 'matte'),       # price tags, name cards
    tray=('#2B2C2E', 'satin'),        # seed trays
    # the cash machine
    atm=('#4A4F55', 'satin'),         # its cabinet; the main part the player colours
    fascia=('#C9CCCF', 'satin'),
    atm_screen=('#2A4A6A', 'gloss', {'glow': '#9FD3FF'}),
    sign=('#2E7D4F', 'gloss', {'glow': '#BFF5D0'}),     # its lit band
    slot=('#1E1F22', 'matte'),        # its slots, the lockers' vents
    # the lockers
    locker=('#5C7A99', 'satin'),      # their doors; the main part the player colours
    # the luggage trolley
    carpet=('#8C1F28', 'fabric'),     # its deck; with the brass, its colours
    case1=('#3E5470', 'satin'), case2=('#C98F6B', 'satin'),
    rubber=('#1E1F22', 'satin'),
    # the vault door
    vault=('#8E959B', 'metal'),       # its door and frame; the main part the player colours
    vault_dark=('#5C6168', 'metal'),
)

F = G.GameFrame()
WALL = -0.43          # the wall's face, game depth (one tile deep)

# Heights the content names (civic.json): kept here and checked against it.
SEAT = 0.45           # pew and waitingBench seat.height (forward 0)


def _content_facts():
    path = os.path.join(G.ASSETS, '..', 'content', 'civic.json')
    with open(path) as f:
        objs = {o['id']: o for o in json.load(f)['objects']}
    for i in ('pew', 'waitingBench'):
        assert objs[i]['seat'] == {'height': SEAT, 'forward': 0}, f'{i}: seat moved'
        assert objs[i]['slots'] == 3, f'{i}: slots moved'


def _rod(name, a, b, r, m, verts=8):
    """A round bar between two game points."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=m, verts=verts)


def _tube(name, pts, r, m, corner=0.03, verts=8):
    """A bent tube through game points, its corners rounded without overshooting."""
    return G.kit.tube(F, name, G.kit.fillet(pts, corner), r, m, verts=verts, round=1)


def _bell(name, x, y, z):
    """A chrome service bell standing at game (x, y, z)."""
    G.revolve(name, [(0, 0), (0.045, 0), (0.045, 0.008), (0.04, 0.012), (0.035, 0.03), (0.022, 0.045),
                     (0.008, 0.05), (0.006, 0.062), (0.01, 0.066), (0, 0.068)], at=F.at(x, y, z), material=M.bell,
              segments=14)


def _monitor(name, x, y, z, w=0.5, h=0.3, face=-1):
    """A flat monitor standing on a top at height `y`, its screen to the back (face -1, the staff's side)."""
    F.box(f'{name}_foot', 0.2, 0.012, 0.15, (x, y + 0.006, z), M.monitor, bevel=0.004)
    F.box(f'{name}_neck', 0.04, 0.17, 0.025, (x, y + 0.09, z - face * 0.035), M.monitor, bevel=0.006)
    F.box(name, w, h, 0.025, (x, y + 0.13 + h / 2, z - face * 0.01), M.monitor, bevel=0.006)
    F.box(f'{name}_screen', w - 0.025, h - 0.025, 0.004, (x, y + 0.13 + h / 2, z - face * 0.01 + face * 0.0125), M.screen)


def _keyboard(name, x, y, z, w=0.42):
    F.box(name, w, 0.016, 0.14, (x, y + 0.008, z), M.monitor, bevel=0.004)
    F.box(f'{name}_keys', w - 0.03, 0.006, 0.11, (x, y + 0.018, z), M.keys)
    F.box(f'{name}_mouse', 0.06, 0.022, 0.1, (x + w / 2 + 0.1, y + 0.011, z), M.keys, bevel=0.01)


def _phone(name, x, y, z, turn=180):
    """A desk phone, its keys to the one at it (turn 180: the back)."""
    with G.new_parts() as parts:
        F.box(f'{name}_base', 0.2, 0.05, 0.18, (0, 0.025, 0), M.monitor, bevel=0.012)
        F.box(f'{name}_handset', 0.06, 0.035, 0.2, (-0.065, 0.065, 0), M.monitor, bevel=0.014)
        for r in range(4):
            for c in range(3):
                F.box(f'{name}_key{r}{c}', 0.022, 0.006, 0.018, (0.01 + c * 0.03, 0.052, 0.05 - r * 0.028), M.keys)
    ph = G.join(parts, name)
    ph.rotation_euler[2] = math.radians(turn)
    ph.location = F.at(x, y, z)


def _book(name, w, t, d, at, cover, rot=(0, 0, 0)):
    """A closed hardback `w` x `t` x `d` lying at game `at` (its middle): its boards and the pages between them."""
    x, y, z = at
    with G.new_parts() as parts:
        F.box(f'{name}_boards', w, t, d, (0, 0, 0), cover, bevel=0.003, bevel_segments=1)
        F.box(f'{name}_pages', w - 0.012, t - 0.006, d - 0.006, (0.007, 0, 0), M.page)
    b = G.join(parts, name)
    b.rotation_euler = tuple(math.radians(a) for a in rot)
    b.location = F.at(x, y, z)
    return b


def _open_book(name, w, d, at, cover, tilt=0.0, t=0.02):
    """An open book lying at game `at` (the middle of its spine, on what it lies on), `w` across both pages, its
    page blocks rounding up from the spine; `tilt` degrees, its far (+z) edge up."""
    x, y, z = at
    with G.new_parts() as parts:
        F.box(f'{name}_cover', w + 0.02, 0.004, d + 0.02, (0, 0.002, 0), cover)
        for s in (-1, 1):
            G.loft(f'{name}_pages{s}', [
                [F.at(s * u, y0, v) for u, v in ((0.004, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (0.004, d / 2))]
                for y0 in (0.004, 0.004 + t)], material=M.page)
            # the top page curving up from the spine
            F.sweep(f'{name}_curl{s}', [(s * 0.004, 0.004 + t * 0.4, 0), (s * w * 0.12, 0.004 + t * 1.25, 0),
                                       (s * w * 0.3, 0.004 + t * 1.05, 0), (s * w * 0.49, 0.004 + t, 0)], None, M.page,
                    profile=[(0.0012, -d / 2), (0.0012, d / 2), (-0.0012, d / 2), (-0.0012, -d / 2)], up=(0, 0, 1),
                    resolution=2)
    b = G.join(parts, name)
    G.set_origin(b, F.at(0, 0, 0))
    b.rotation_euler[0] = math.radians(-tilt)
    b.location = F.at(x, y, z)
    return b


def _plant(name, x, y, z, r=0.07):
    """A small potted plant standing at game (x, y, z)."""
    G.revolve(f'{name}_pot', [(0, 0), (r * 0.7, 0), (r * 0.8, r * 0.1), (r, r * 1.3), (r * 0.9, r * 1.3), (0, r * 1.2)],
              at=F.at(x, y, z), material=M.pot, segments=16)
    for k, (dx, dy, dz, s) in enumerate(((0, 1.9, 0, 0.9), (-0.45, 1.6, 0.2, 0.7), (0.45, 1.65, -0.2, 0.75))):
        G.jitter(F.sphere(f'{name}_leaf{k}', r * s, (x + dx * r, y + dy * r, z + dz * r), M.leaf, scale=(1, 1, 0.8),
                          segments=10, rings=6), r * 0.15, size=r * 0.6, seed=k * 1.3 + x)


# --- the front desks: 2x1, a high ledge to the front over a low worktop at the back ------------------------------
LEDGE = 1.08          # the customer's ledge
WORK = 0.75           # the worktop behind it


def _desk_carcass(body, ledge, band=None):
    """The carcass the reception, hotel and duty desks share: ends stepping from the ledge down to the worktop, a
    front panel, a dark kick, an inner panel facing the staff, the ledge and the worktop, a drawer pedestal."""
    for s in (-1, 1):
        F.extrude(f'end{s}', [(-0.4, 0.0), (0.4, 0.0), (0.4, LEDGE - 0.02), (0.06, LEDGE - 0.02), (0.06, WORK - 0.02),
                              (-0.4, WORK - 0.02)], 0.04, (s * 0.91, 0, 0), body, plane='side', bevel=0.004)
    F.box('front', 1.78, LEDGE - 0.1, 0.03, (0, 0.06 + (LEDGE - 0.1) / 2 + 0.0, 0.375), body, bevel=0.004)
    F.box('kick', 1.78, 0.06, 0.03, (0, 0.03, 0.355), M.plinth)
    F.box('inner', 1.78, LEDGE - WORK - 0.01, 0.02, (0, (LEDGE + WORK) / 2 - 0.015, 0.07), body)
    F.box('worktop', 1.78, 0.03, 0.47, (0, WORK - 0.015, -0.165), body if band is None else ledge, bevel=0.004)
    F.box('pedestal', 0.44, WORK - 0.05, 0.46, (0.66, (WORK - 0.03) / 2 + 0.01, -0.16), body, bevel=0.004)
    for i in range(3):
        y = 0.13 + i * 0.22
        F.box(f'drawer{i}', 0.41, 0.2, 0.012, (0.66, y, -0.396), body, bevel=0.003)
        F.box(f'drawer_pull{i}', 0.14, 0.012, 0.014, (0.66, y + 0.06, -0.408), M.frame, bevel=0.004)


def receptionDesk():
    with G.new_parts() as parts:
        _desk_carcass(M.desk, M.oak)
        F.box('ledge', 1.92, 0.035, 0.42, (0, LEDGE + 0.0175, 0.225), M.oak, bevel=0.008)
        F.box('band', 1.8, 0.16, 0.015, (0, 0.8, 0.398), M.oak, bevel=0.003)
        for i in range(13):                              # oak fins under the band
            F.box(f'fin{i}', 0.03, 0.56, 0.02, (-0.84 + i * 0.14, 0.4, 0.4), M.oak, bevel=0.004)
        top = WORK
        _monitor('monitor', -0.15, top, -0.08)
        _keyboard('keyboard', -0.2, top, -0.27)
        _phone('phone', 0.5, top, -0.18)
        G.revolve('pens', [(0, 0), (0.035, 0), (0.035, 0.1), (0.03, 0.1), (0.03, 0.006), (0, 0.006)],
                  at=F.at(-0.7, top, -0.15), material=M.monitor, segments=12)
        for k in range(3):
            _rod(f'pen{k}', (-0.7 + 0.012 * (k - 1), top + 0.03, -0.15 + 0.01 * k),
                 (-0.7 + 0.03 * (k - 1), top + 0.16, -0.15 + 0.02 * (k - 1)), 0.005, [M.leaflet1, M.leaflet4, M.monitor][k], 6)
        # on the ledge: a bell, a stand of leaflets, a plant
        y = LEDGE + 0.035
        _bell('bell', 0.15, y, 0.3)
        LX, LZ = -0.5, 0.22
        F.box('stand_base', 0.36, 0.012, 0.2, (LX, y + 0.006, LZ), M.acrylic)
        for row in range(3):
            z, h = LZ + 0.06 - row * 0.06, 0.07 + row * 0.06
            F.box(f'stand_pocket{row}', 0.36, h, 0.006, (LX, y + h / 2, z + 0.02), M.acrylic, rot=(-8, 0, 0))
            for c in range(3):
                lh = h + 0.06
                F.box(f'leaflet{row}{c}', 0.1, lh, 0.006, (LX - 0.12 + c * 0.12, y + lh / 2 + 0.01, z + 0.005),
                      [M.leaflet1, M.leaflet2, M.leaflet3, M.leaflet4][(row * 3 + c) % 4], rot=(-8, 0, 0))
        _plant('plant', 0.68, y, 0.24)
    G.join(parts, 'desk', origin=(0, 0, 0))
    G.settle()


def hotelReception():
    with G.new_parts() as parts:
        _desk_carcass(M.walnut, M.marble)
        # its front: three raised walnut panels over a brass kick strip
        for i, x in enumerate((-0.595, 0.0, 0.595)):
            G.kit.shaker(F, f'panel{i}', 0.57, 0.84, (x, 0.56, 0.398), 0.022, M.walnut, rail=0.07, raised=True)
        F.box('kick_strip', 1.8, 0.05, 0.006, (0, 0.06, 0.393), M.brass)
        # the marble ledge, its nose rounded, on a walnut moulding
        F.extrude('ledge', [(0.03, 0.0), (0.42, 0.0), (0.436, 0.006), (0.443, 0.018), (0.436, 0.03), (0.42, 0.036),
                            (0.03, 0.036)], 1.94, (0, LEDGE, 0), M.marble, plane='side')
        F.box('ledge_bead', 1.86, 0.03, 0.03, (0, LEDGE - 0.015, 0.398), M.walnut, bevel=0.008)
        y = LEDGE + 0.036
        _open_book('guest_book', 0.42, 0.3, (-0.2, y, 0.25), M.book)
        _rod('pen', (0.06, y + 0.006, 0.18), (0.2, y + 0.006, 0.24), 0.005, M.brass, 6)
        _bell('bell', 0.3, y, 0.3)
        # a banker's lamp: a brass base and stem, the green glass shade
        G.revolve('lamp_base', [(0, 0), (0.08, 0), (0.08, 0.012), (0.06, 0.03), (0, 0.032)], at=F.at(0.68, y, 0.2),
                  material=M.brass, segments=20)
        F.cyl('lamp_stem', 0.01, 0.26, (0.68, y + 0.16, 0.15), M.brass, verts=8)
        F.box('lamp_arm', 0.02, 0.02, 0.1, (0.68, y + 0.29, 0.2), M.brass)
        bx, by, bz = F.at(0.68, y + 0.3, 0.24)
        G.loft('lamp_shade', [[(bx + px, by + py * 0.55, bz + pz) for px, py, pz in G.ring(0.2, 0.09, h, 16, square=2.6)]
                              for h in (0.0, 0.03)] + [[(bx + px, by + py * 0.25, bz + pz + 0.0) for px, py, pz in
                                                        G.ring(0.18, 0.04, 0.065, 16, square=2.6)]], material=M.shade)
        # the room keys on hooks, a board standing at the back of the worktop for the guests to see
        KX = -0.6
        F.box('key_board', 0.56, 0.66, 0.03, (KX, WORK + 0.33, -0.385), M.walnut, bevel=0.006)
        F.box('key_cap', 0.6, 0.03, 0.06, (KX, WORK + 0.675, -0.375), M.walnut, bevel=0.008)
        for r in range(3):
            for c in range(4):
                hx, hy = KX - 0.195 + c * 0.13, WORK + 0.56 - r * 0.19
                F.cyl(f'hook{r}{c}', 0.006, 0.04, (hx, hy, -0.35), M.brass, rot=(90, 0, 0), verts=6)
                if (r * 4 + c) % 5 != 3:                 # a few rooms out
                    F.box(f'key{r}{c}', 0.012, 0.06, 0.004, (hx, hy - 0.035, -0.345), M.brass)
                    G.revolve(f'fob{r}{c}', [(0, 0), (0.016, 0), (0.02, 0.03), (0.016, 0.07), (0, 0.075)],
                              at=F.at(hx, hy - 0.14, -0.345), material=M.fob, segments=8)
        _phone('phone', 0.2, WORK, -0.2)
        _book('ledger', 0.3, 0.035, 0.22, (-0.15, WORK + 0.0175, -0.2), M.book, rot=(0, 0, 8))
    G.join(parts, 'desk', origin=(0, 0, 0))
    G.settle()


def dutyDesk():
    with G.new_parts() as parts:
        _desk_carcass(M.counter, M.laminate, band=True)
        F.box('ledge', 1.92, 0.03, 0.42, (0, LEDGE + 0.015, 0.225), M.laminate, bevel=0.008)
        # a chequered band along the front
        F.box('band', 1.78, 0.1, 0.008, (0, 0.82, 0.394), M.label)
        G.kit.chequer(F, 'chequer', -0.88, 0.88, 0.87, 0.398, 0.05, 1, M.police, rows=2)
        # the security screen on the ledge: glass in an aluminium frame, the hatch under its middle pane
        y = LEDGE + 0.03
        H = 0.8
        ZS = 0.1
        F.box('screen_rail', 1.92, 0.04, 0.05, (0, y + H, ZS), M.frame, bevel=0.006)
        for i, x in enumerate((-0.94, -0.3, 0.3, 0.94)):
            F.box(f'mullion{i}', 0.04, H, 0.05, (x, y + H / 2, ZS), M.frame, bevel=0.005)
        for s in (-1, 1):
            F.box(f'pane{s}', 0.6, H - 0.02, 0.008, (s * 0.62, y + H / 2, ZS), M.glass)
        F.box('pane_middle', 0.56, H - 0.2, 0.008, (0, y + 0.19 + (H - 0.2) / 2, ZS), M.glass)
        F.box('hatch_rail', 0.56, 0.03, 0.05, (0, y + 0.175, ZS), M.frame, bevel=0.005)
        G.revolve('grille', [(0, 0), (0.06, 0), (0.06, 0.008), (0, 0.008)], at=F.at(0, y + 0.42, ZS + 0.004),
                  material=M.frame, segments=16, rot=(90, 0, 0))
        _open_book('log_book', 0.36, 0.26, (0.0, y, 0.29), M.counter)
        _rod('pen', (0.24, y + 0.006, 0.25), (0.36, y + 0.006, 0.3), 0.005, M.monitor, 6)
        _tube('pen_chain', [(0.36, y + 0.006, 0.3), (0.42, y + 0.003, 0.33), (0.45, y + 0.003, 0.18)], 0.0025, M.frame,
              corner=0.02, verts=4)
        _monitor('monitor', -0.2, WORK, -0.1)
        _keyboard('keyboard', -0.2, WORK, -0.28)
        # a radio on its charger
        F.box('charger', 0.1, 0.04, 0.12, (0.25, WORK + 0.02, -0.2), M.radio, bevel=0.006)
        F.box('radio', 0.06, 0.13, 0.035, (0.25, WORK + 0.1, -0.2), M.radio, bevel=0.01)
        F.cyl('aerial', 0.006, 0.08, (0.265, WORK + 0.2, -0.2), M.radio, verts=6)
        F.box('radio_screen', 0.04, 0.03, 0.004, (0.25, WORK + 0.13, -0.218), M.screen)
    G.join(parts, 'desk', origin=(0, 0, 0))
    G.settle()


# --- the teller counter: 2x1, oak under glass, the teller behind --------------------------------------------------
def tellerCounter():
    with G.new_parts() as parts:
        _desk_carcass(M.teller, M.laminate, band=True)
        for i, x in enumerate((-0.595, 0.0, 0.595)):
            G.kit.shaker(F, f'panel{i}', 0.57, 0.84, (x, 0.56, 0.398), 0.022, M.teller, rail=0.07, raised=True)
        F.box('ledge', 1.92, 0.035, 0.42, (0, LEDGE + 0.0175, 0.225), M.teller, bevel=0.008)
        y = LEDGE + 0.035
        H = 0.85
        ZS = 0.08
        # the glass from the ledge nearly to the ceiling's height, oak rails top and bottom, a grille to speak through,
        # a deal tray under it for the notes
        F.box('screen_sill', 1.92, 0.05, 0.08, (0, y + 0.025, ZS), M.teller, bevel=0.008)
        F.box('screen_rail', 1.92, 0.06, 0.08, (0, y + H, ZS), M.teller, bevel=0.01)
        for i, x in enumerate((-0.94, 0.0, 0.94)):
            F.box(f'post{i}', 0.04, H - 0.05, 0.06, (x, y + H / 2 + 0.02, ZS), M.teller, bevel=0.006)
        for s in (-1, 1):
            F.box(f'pane{s}', 0.9, H - 0.08, 0.008, (s * 0.47, y + 0.05 + (H - 0.08) / 2, ZS), M.glass)
            G.revolve(f'grille{s}', [(0, 0), (0.065, 0), (0.065, 0.006), (0.055, 0.01), (0, 0.01)],
                      at=F.at(s * 0.47, y + 0.42, ZS + 0.004), material=M.frame, segments=18, rot=(90, 0, 0))
            for k in range(4):
                F.box(f'grille{s}_slot{k}', 0.07, 0.006, 0.002, (s * 0.47, y + 0.39 + k * 0.02, ZS + 0.015), M.slot)
            # the deal tray, sunk through the ledge under the glass
            F.box(f'tray{s}', 0.32, 0.012, 0.26, (s * 0.47, y + 0.004, ZS), M.frame, bevel=0.004)
            F.box(f'tray{s}_dish', 0.28, 0.004, 0.22, (s * 0.47, y + 0.011, ZS), M.slot)
            # on the teller's side: notes and coins in a tray, a stamp
            F.box(f'cash{s}', 0.36, 0.05, 0.28, (s * 0.5, WORK + 0.025, -0.22), M.monitor, bevel=0.004)
            for k in range(4):
                F.box(f'notes{s}{k}', 0.06, 0.012, 0.13, (s * 0.5 - 0.12 + k * 0.08, WORK + 0.054, -0.27), M.note)
                F.cyl(f'coins{s}{k}', 0.014, 0.02, (s * 0.5 - 0.12 + k * 0.08, WORK + 0.06, -0.14), M.coin, verts=8)
        _monitor('monitor', 0.5, WORK, -0.05, w=0.4, h=0.26)
        _keyboard('keyboard', -0.35, WORK, -0.05, w=0.36)
        # a pen on a chain for the customers
        _rod('pen', (0.0, y + 0.006, 0.3), (0.14, y + 0.006, 0.36), 0.005, M.monitor, 6)
        _tube('pen_chain', [(0.0, y + 0.006, 0.3), (-0.05, y + 0.003, 0.25), (-0.02, y + 0.003, 0.14)], 0.0025, M.frame,
              corner=0.02, verts=4)
    G.join(parts, 'counter', origin=(0, 0, 0))
    G.settle()


# --- the teacher's desk: 2x1, oak, a pedestal of drawers to the teacher's side ------------------------------------
def teacherDesk():
    T = 0.76
    with G.new_parts() as parts:
        F.box('top', 1.5, 0.03, 0.7, (0, T - 0.015, 0), M.oak, bevel=0.006)
        # the pedestal on the right of the class's view, its drawers to the teacher at the back
        F.box('pedestal', 0.42, T - 0.04, 0.64, (0.5, (T - 0.04) / 2 + 0.01, 0), M.oak, bevel=0.004)
        F.box('pedestal_plinth', 0.4, 0.04, 0.62, (0.5, 0.02, 0.0), M.plinth)
        for i in range(3):
            y = 0.17 + i * 0.2
            G.kit.shaker(F, f'drawer{i}', 0.38, 0.18, (0.5, y, -0.326), 0.014, M.oak, rail=0.04)
            G.kit.knob(F, f'knob{i}', 0.5, y, -0.333, 0.014, M.brass).rotation_euler[0] = math.radians(-90)
        for s in (-1, 1):
            F.box(f'leg{s}', 0.05, T - 0.03, 0.05, (-0.7, (T - 0.03) / 2, s * 0.3), M.oak, bevel=0.004)
        F.box('apron_side', 0.03, 0.1, 0.56, (-0.7, T - 0.08, 0.0), M.oak)
        F.box('apron_back', 0.98, 0.08, 0.03, (-0.22, T - 0.07, -0.31), M.oak)
        # the modesty panel to the class
        G.kit.shaker(F, 'modesty', 0.98, 0.5, (-0.215, T - 0.28, 0.31), 0.02, M.oak, rail=0.06, raised=True)
        # the register, exercise books, an apple, a pot of pencils and a globe
        _book('register', 0.24, 0.025, 0.32, (0.05, T + 0.0125, -0.08), M.leaflet3, rot=(0, 0, -6))
        for k in range(5):
            _book(f'exercise{k}', 0.21, 0.008, 0.29, (0.45 + 0.005 * (k % 2), T + 0.004 + k * 0.0085, 0.1),
                  [M.leaflet1, M.leaflet4, M.leaflet2, M.leaflet1, M.leaflet3][k], rot=(0, 0, k * 4 - 6))
        F.sphere('apple', 0.04, (0.25, T + 0.037, 0.16), M.leaflet4, scale=(1, 1, 0.92), segments=14, rings=10)
        F.cyl('apple_stalk', 0.003, 0.025, (0.25, T + 0.083, 0.16), M.soil, verts=5)
        G.revolve('pencil_pot', [(0, 0), (0.035, 0), (0.035, 0.1), (0.031, 0.1), (0.031, 0.006), (0, 0.006)],
                  at=F.at(0.62, T, -0.2), material=M.pot, segments=14)
        for k in range(4):
            a = k * 1.6
            _rod(f'pencil{k}', (0.62 + 0.01 * math.cos(a), T + 0.02, -0.2 + 0.01 * math.sin(a)),
                 (0.62 + 0.035 * math.cos(a), T + 0.17, -0.2 + 0.035 * math.sin(a)), 0.004,
                 [M.leaflet2, M.leaflet4, M.leaflet1, M.leaflet3][k], 6)
        GX, GZ = -0.5, -0.12
        G.revolve('globe_base', [(0, 0), (0.08, 0), (0.08, 0.015), (0.05, 0.03), (0.02, 0.04), (0.015, 0.09), (0, 0.09)],
                  at=F.at(GX, T, GZ), material=M.walnut, segments=16)
        F.sweep('globe_arc', [(GX + 0.13 * math.sin(a), T + 0.23 + 0.13 * math.cos(a), GZ)
                              for a in [math.radians(d) for d in range(150, 371, 20)]], None, M.brass,
                profile=G.rounded_rect(0.006, 0.012, 0.0), up=(0, 1, 0))
        F.sphere('globe', 0.12, (GX, T + 0.23, GZ), M.leaflet1, segments=20, rings=14)
        # its lands: flattened patches lying on the sphere
        centre = Vector(F.at(GX, T + 0.23, GZ))
        for k, (dx, dy, dz, r) in enumerate(((0.5, 0.5, 0.7, 0.06), (-0.7, -0.1, 0.7, 0.05), (0.2, -0.6, -0.75, 0.07),
                                             (-0.6, 0.6, -0.5, 0.045))):
            n = Vector(F.at(dx, dy, dz)).normalized()
            land = G.sphere(f'land{k}', r, at=tuple(centre + n * 0.116), material=M.leaflet3, scale=(1, 0.8, 0.1),
                            segments=10, rings=6)
            land.rotation_mode = 'QUATERNION'
            land.rotation_quaternion = n.to_track_quat('Z', 'Y')
    G.join(parts, 'desk', origin=(0, 0, 0))
    G.settle()


# --- the lectern: 1x1, a carved oak stand, the book sloping to the reader at the back -----------------------------
def lectern():
    with G.new_parts() as parts:
        F.box('base', 0.56, 0.06, 0.56, (0, 0.03, 0), M.pew_oak, bevel=0.012)
        F.box('base_step', 0.44, 0.05, 0.44, (0, 0.085, 0), M.pew_oak, bevel=0.01)
        G.revolve('column', [(0, 0.11), (0.09, 0.11), (0.09, 0.14), (0.06, 0.16), (0.05, 0.2), (0.045, 0.45),
                             (0.07, 0.48), (0.07, 0.52), (0.045, 0.55), (0.04, 0.85), (0.06, 0.88), (0.075, 0.92),
                             (0, 0.92)], material=M.pew_oak, segments=16)
        # the desk: a sloping board on a box, its front to the church carved with a cross
        F.extrude('desk_side_l', [(-0.24, 0.98), (0.24, 1.2), (0.24, 0.92), (-0.24, 0.92)], 0.03, (0.3, 0, 0),
                  M.pew_oak, plane='side', bevel=0.004)
        F.extrude('desk_side_r', [(-0.24, 0.98), (0.24, 1.2), (0.24, 0.92), (-0.24, 0.92)], 0.03, (-0.3, 0, 0),
                  M.pew_oak, plane='side', bevel=0.004)
        F.box('desk_front', 0.57, 0.28, 0.03, (0, 1.06, 0.225), M.pew_oak, bevel=0.004)
        F.box('desk_back', 0.57, 0.06, 0.03, (0, 0.95, -0.225), M.pew_oak, bevel=0.004)
        F.box('desk_floor', 0.57, 0.02, 0.45, (0, 0.93, 0), M.pew_oak)
        slope = math.degrees(math.atan2(0.22, 0.48))
        F.box('desk_top', 0.66, 0.025, 0.54, (0, 1.1, 0.0), M.pew_oak, bevel=0.006, rot=(-slope, 0, 0))
        F.box('desk_lip', 0.62, 0.05, 0.025, (0, 1.0, -0.255), M.pew_oak, bevel=0.006)
        G.kit.moulding(F, 'front_frame', [(-0.25, 0.94), (0.25, 0.94), (0.25, 1.17), (-0.25, 1.17)],
                       [(0, 0), (0, 0.012), (0.006, 0.014), (0.02, 0.014), (0.02, 0)], M.pew_oak, at=0.24)
        F.box('cross_v', 0.035, 0.18, 0.012, (0, 1.055, 0.246), M.brass, bevel=0.003)
        F.box('cross_h', 0.12, 0.035, 0.012, (0, 1.09, 0.246), M.brass, bevel=0.003)
        # the big book, open on the slope, its ribbon over the lower edge
        b = _open_book('bible', 0.46, 0.34, (0, 0, 0), M.book, t=0.035)
        b.rotation_euler[0] = math.radians(-slope)
        b.location = F.at(0, 1.1 + 0.014 / math.cos(math.radians(slope)), 0.0)
        F.box('ribbon', 0.016, 0.16, 0.003, (0.02, 1.0, -0.27), M.fob)
    G.join(parts, 'lectern', origin=(0, 0, 0))
    G.settle()


# --- the bar: 2x1, panelled mahogany, three pumps, a brass foot rail ------------------------------------------------
def barCounter():
    TOP = 1.05
    with G.new_parts() as parts:
        F.box('carcass', 1.9, TOP - 0.06, 0.33, (0, (TOP - 0.06) / 2 + 0.01, 0.065), M.mahogany, bevel=0.006)
        F.box('plinth', 1.88, 0.08, 0.58, (0, 0.04, -0.07), M.plinth)
        # the barman's side: shelves under the top, open to the back, glasses and bottles on them
        for x in (-0.935, 0.0, 0.935):
            F.box(f'back_end{x}', 0.03, TOP - 0.06, 0.3, (x, (TOP - 0.06) / 2 + 0.01, -0.245), M.mahogany, bevel=0.004)
        for y in (0.095, 0.5):
            F.box(f'back_shelf{y}', 1.84, 0.03, 0.29, (0, y, -0.245), M.mahogany, bevel=0.003)
        for k in range(10):
            x = -0.83 + k * 0.18 + (0.04 if k >= 5 else 0)
            G.revolve(f'glass{k}', [(0, 0), (0.03, 0), (0.033, 0.01), (0.038, 0.13), (0.035, 0.13), (0.03, 0.012), (0, 0.012)],
                      at=F.at(x, 0.515, -0.25), material=M.acrylic, segments=12)
        for k in range(6):
            x = -0.78 + k * 0.3
            G.revolve(f'bottle{k}', [(0, 0), (0.035, 0), (0.035, 0.18), (0.014, 0.23), (0.012, 0.28), (0, 0.28)],
                      at=F.at(x, 0.11, -0.25), material=[M.leaf, M.beer, M.mag1][k % 3], segments=12)
        for i, x in enumerate((-0.62, 0.0, 0.62)):
            G.kit.shaker(F, f'panel{i}', 0.6, 0.78, (x, 0.53, 0.241), 0.022, M.mahogany, rail=0.07, raised=True)
        # the top: a thick slab with a rounded nose to the drinkers, a bead under it
        F.extrude('top', [(-0.4, 0.0), (0.33, 0.0), (0.355, 0.006), (0.37, 0.022), (0.37, 0.034), (0.355, 0.05),
                          (0.33, 0.056), (-0.4, 0.056)], 1.96, (0, TOP - 0.056, 0), M.mahogany, plane='side')
        F.sweep('cove', [(-0.95, TOP - 0.068, 0.236), (0.95, TOP - 0.068, 0.236)], None, M.mahogany,
                profile=[(0.012, -0.01), (0.012, 0.012), (-0.012, 0.012), (-0.012, 0.004), (0.0, 0.0), (0.006, -0.01)])
        # the brass foot rail on three brackets
        _rod('rail', (-0.92, 0.2, 0.4), (0.92, 0.2, 0.4), 0.024, M.brass, 14)
        for x in (-0.92, 0.92):
            F.sphere(f'rail_end{x}', 0.03, (x, 0.2, 0.4), M.brass, segments=12, rings=8)
        for x in (-0.7, 0.0, 0.7):
            _tube(f'bracket{x}', [(x, 0.32, 0.25), (x, 0.32, 0.36), (x, 0.2, 0.4)], 0.012, M.brass, corner=0.03)
        # the pumps over a drip tray at the back of the top
        F.box('drip_tray', 0.7, 0.02, 0.12, (0, TOP + 0.01, -0.1), M.bell, bevel=0.004)
        for i, (x, clip) in enumerate(((-0.25, M.clip1), (0.0, M.clip2), (0.25, M.clip3))):
            G.revolve(f'pump{i}', [(0, 0), (0.045, 0), (0.045, 0.01), (0.035, 0.02), (0.03, 0.1), (0.04, 0.12),
                                   (0.02, 0.13), (0, 0.13)], at=F.at(x, TOP, -0.2), material=M.bell, segments=16)
            F.sweep(f'spout{i}', [(x, TOP + 0.09, -0.17), (x, TOP + 0.12, -0.12), (x, TOP + 0.08, -0.09)], 0.008,
                    M.bell, resolution=3, verts=8)
            G.revolve(f'handle{i}', [(0, 0), (0.016, 0), (0.02, 0.05), (0.024, 0.2), (0.02, 0.26), (0.012, 0.28), (0, 0.28)],
                      at=F.at(x, TOP + 0.13, -0.2), material=M.handle, segments=12)
            F.box(f'clip{i}', 0.065, 0.085, 0.01, (x, TOP + 0.31, -0.18), clip, bevel=0.004)
        # a runner along the front with two pints on it, beer mats
        F.box('runner', 1.2, 0.006, 0.12, (0, TOP + 0.003, 0.2), M.towel)
        for k, x in enumerate((-0.45, 0.5)):
            G.revolve(f'pint{k}', [(0, 0), (0.03, 0), (0.033, 0.01), (0.038, 0.11), (0.041, 0.13), (0, 0.13)],
                      at=F.at(x, TOP + 0.006, 0.2), material=M.beer, segments=16)
            G.revolve(f'head{k}', [(0, 0), (0.041, 0), (0.041, 0.018), (0.03, 0.022), (0, 0.022)],
                      at=F.at(x, TOP + 0.136, 0.2), material=M.froth, segments=16)
        for k, x in enumerate((-0.8, 0.82)):
            F.box(f'mat{k}', 0.095, 0.004, 0.095, (x, TOP + 0.002, 0.18), M.clip2 if k else M.clip1)
    G.join(parts, 'bar', origin=(0, 0, 0))
    G.settle()


# --- the pupil's desk: 1x1, a lidded desk with an inkwell joined to its seat on a cast-iron frame ---------------
def schoolDesk():
    with G.new_parts() as parts:
        # the cast-iron sides: a runner on the floor, the desk's leg, the seat's leg rising into the back post
        for s in (-1, 1):
            x = s * 0.27
            _tube(f'runner{s}', [(x, 0.016, -0.25), (x, 0.016, 0.46)], 0.016, M.iron)
            _tube(f'desk_leg{s}', [(x, 0.016, 0.42), (x, 0.56, 0.42)], 0.016, M.iron)
            _tube(f'seat_leg{s}', [(x, 0.016, -0.2), (x, 0.42, -0.2), (x, 0.82, -0.245)], 0.016, M.iron, corner=0.05)
            _tube(f'seat_arm{s}', [(x, 0.42, -0.2), (x, 0.42, 0.12)], 0.013, M.iron)
            _tube(f'brace{s}', [(x, 0.016, 0.05), (x, 0.3, 0.2), (x, 0.56, 0.3)], 0.012, M.iron, corner=0.08)
            F.sphere(f'foot{s}a', 0.022, (x, 0.016, -0.25), M.iron, segments=8, rings=6)
            F.sphere(f'foot{s}b', 0.022, (x, 0.016, 0.46), M.iron, segments=8, rings=6)
        _rod('stretcher', (-0.27, 0.2, 0.2), (0.27, 0.2, 0.2), 0.012, M.iron)
        # the seat and the back's two slats
        F.box('seat', 0.58, 0.025, 0.32, (0, 0.4325 + 0.0, -0.04), M.pine, bevel=0.008)
        for k, y in enumerate((0.6, 0.75)):
            F.box(f'slat{k}', 0.58, 0.08, 0.022, (0, y, -0.225 - (y - 0.42) * 0.11 - 0.012), M.pine, bevel=0.006,
                  rot=(-6, 0, 0))
        # the desk: a box with a sloping lid towards the pupil, a flat strip at the far edge with the inkwell and a
        # groove for the pen
        side = [(0.18, 0.56), (0.48, 0.56), (0.48, 0.7), (0.4, 0.7), (0.18, 0.665)]
        for s in (-1, 1):
            F.extrude(f'box_side{s}', side, 0.02, (s * 0.29, 0, 0), M.pine, plane='side', bevel=0.003)
        F.box('box_front', 0.6, 0.14, 0.02, (0, 0.63, 0.47), M.pine, bevel=0.003)
        F.box('box_back', 0.56, 0.1, 0.02, (0, 0.61, 0.19), M.pine, bevel=0.003)
        F.box('box_floor', 0.56, 0.015, 0.29, (0, 0.567, 0.33), M.pine)
        F.box('strip', 0.62, 0.022, 0.09, (0, 0.711, 0.445), M.pine, bevel=0.005)
        slope = math.degrees(math.atan2(0.035, 0.22))
        F.box('lid', 0.62, 0.02, 0.235, (0, 0.692, 0.287), M.pine, bevel=0.005, rot=(-slope, 0, 0))
        F.box('groove', 0.3, 0.004, 0.018, (-0.08, 0.7225, 0.43), M.soil)
        G.revolve('inkwell', [(0, 0), (0.022, 0), (0.024, 0.006), (0.024, 0.02), (0.016, 0.026), (0, 0.026)],
                  at=F.at(0.2, 0.712, 0.445), material=M.pot, segments=12)
        F.cyl('ink', 0.009, 0.004, (0.2, 0.7385, 0.445), M.ink, verts=10)
        _rod('pen', (-0.2, 0.7275, 0.43), (0.02, 0.7275, 0.432), 0.004, M.ink, 6)
        _book('exercise', 0.17, 0.006, 0.23, (-0.05, 0.705, 0.28), M.leaflet1, rot=(-slope, 0, 8))
    G.join(parts, 'desk', origin=(0, 0, 0))
    G.settle()


# --- the pew: 3x1, an oak bench with a back, shaped ends and a book ledge behind ---------------------------------
def pew():
    with G.new_parts() as parts:
        F.box('seat', 2.84, 0.04, 0.42, (0, SEAT - 0.02, -0.01), M.pew_oak, bevel=0.012)
        F.box('seat_rail', 2.84, 0.08, 0.03, (0, SEAT - 0.08, 0.18), M.pew_oak, bevel=0.006)
        F.box('back', 2.84, 0.5, 0.03, (0, SEAT + 0.27, -0.255), M.pew_oak, bevel=0.006, rot=(-8, 0, 0))
        F.box('back_rail', 2.84, 0.04, 0.07, (0, SEAT + 0.53, -0.29), M.pew_oak, bevel=0.015)
        F.box('stretcher', 2.84, 0.1, 0.03, (0, 0.16, -0.1), M.pew_oak, bevel=0.006)
        # the ledge for the books behind, for the row at the back
        F.box('ledge', 2.84, 0.02, 0.12, (0, 0.8, -0.36), M.pew_oak, bevel=0.004)
        F.box('ledge_lip', 2.84, 0.04, 0.015, (0, 0.82, -0.415), M.pew_oak, bevel=0.004)
        for k in range(7):
            x = -1.25 + k * 0.4 + 0.03 * (k % 3)
            _book(f'hymnal{k}', 0.13, 0.03, 0.1, (x, 0.825, -0.36), M.book if k % 3 else M.mag1, rot=(0, 0, 90 - 6 * (k % 2)))
        # the ends: shaped boards rising to a scrolled top at the back, a raised field in each
        end = G.smooth_outline([(-0.38, 0.0), (0.24, 0.0), (0.24, 0.6), (0.2, 0.66), (0.08, 0.68), (-0.04, 0.74),
                                (-0.14, 0.86), (-0.2, 0.98), (-0.26, 1.04), (-0.34, 1.04), (-0.38, 0.98)], passes=2)
        field = G.smooth_outline([(-0.3, 0.12), (0.16, 0.12), (0.16, 0.56), (0.06, 0.6), (-0.06, 0.66), (-0.16, 0.78),
                                  (-0.24, 0.9), (-0.3, 0.9)], passes=2)
        for s in (-1, 1):
            F.extrude(f'end{s}', end, 0.06, (s * 1.45, 0, 0), M.pew_oak, plane='side', bevel=0.008)
            F.extrude(f'field{s}', field, 0.076, (s * 1.45, 0, 0), M.pew_oak, plane='side', bevel=0.006)
    G.join(parts, 'pew', origin=(0, 0, 0))
    G.settle()


# --- the waiting bench: 3x1, three moulded seats on a steel beam, tables between with magazines ------------------
def waitingBench():
    shell_side = G.smooth_outline([(0.23, SEAT - 0.035), (0.22, SEAT - 0.005), (0.05, SEAT), (-0.12, SEAT - 0.01),
                                   (-0.19, SEAT + 0.03), (-0.22, SEAT + 0.15), (-0.25, SEAT + 0.32), (-0.27, SEAT + 0.4),
                                   (-0.29, SEAT + 0.39), (-0.27, SEAT + 0.15), (-0.24, SEAT + 0.02), (-0.15, SEAT - 0.03),
                                   (0.05, SEAT - 0.02), (0.2, SEAT - 0.025)], passes=1)
    with G.new_parts() as parts:
        F.box('beam', 2.7, 0.08, 0.06, (0, 0.3, -0.06), M.steel, bevel=0.006)
        for x in (-1.25, 1.25):
            F.box(f'post{x}', 0.05, 0.28, 0.05, (x, 0.16, -0.06), M.steel, bevel=0.005)
            F.box(f'foot{x}', 0.06, 0.03, 0.52, (x, 0.015, -0.04), M.steel, bevel=0.006)
            for z in (-0.28, 0.2):
                F.cyl(f'glide{x}{z}', 0.022, 0.012, (x, 0.006, z), M.radio, verts=10)
        for i, x in enumerate((-1.0, 0.0, 1.0)):
            F.extrude(f'shell{i}', shell_side, 0.5, (x, 0, 0), M.shell, plane='side', bevel=0.012)
            F.box(f'bracket{i}', 0.3, 0.1, 0.24, (x, 0.37, -0.04), M.steel, bevel=0.006)
        for k, x in enumerate((-0.5, 0.5)):
            F.box(f'table{k}', 0.4, 0.025, 0.44, (x, SEAT - 0.01, -0.02), M.laminate, bevel=0.006)
            F.box(f'table_arm{k}', 0.06, 0.12, 0.2, (x, 0.38, -0.06), M.steel, bevel=0.006)
            for j in range(3 + k):
                F.box(f'mag{k}{j}', 0.21, 0.006, 0.28, (x + 0.02 * ((j * 7) % 3 - 1), SEAT + 0.006 + j * 0.0065, -0.02),
                      [M.mag1, M.mag2, M.mag3, M.mag4][(j + 2 * k) % 4], rot=(0, 0, (j * 37 % 30) - 15))
    G.join(parts, 'bench', origin=(0, 0, 0))
    G.settle()


# --- the couch: 1x2, a padded couch on a white frame, its head raised at the back, a paper sheet over it ---------
def examCouch():
    PAD = 0.45        # the pad's top: the default seat's height
    HINGE = -0.3      # where the head rest rises
    with G.new_parts() as parts:
        for x in (-0.25, 0.25):
            for z in (-0.82, 0.82):
                _rod(f'leg{x}{z}', (x, 0.03, z), (x, PAD - 0.09, z), 0.02, M.white, 10)
                F.cyl(f'foot{x}{z}', 0.026, 0.03, (x, 0.015, z), M.radio, verts=10)
            _rod(f'rail{x}', (x, PAD - 0.09, -0.84), (x, PAD - 0.09, 0.84), 0.016, M.white, 10)
            _rod(f'low_rail{x}', (x, 0.14, -0.82), (x, 0.14, 0.82), 0.012, M.white, 8)
        for z in (-0.82, 0.82):
            _rod(f'cross{z}', (-0.25, PAD - 0.09, z), (0.25, PAD - 0.09, z), 0.016, M.white, 10)
        F.box('shelf', 0.5, 0.012, 1.5, (0, 0.15, 0.0), M.white)
        F.box('board', 0.6, 0.02, 1.88 - 0.62, (0, PAD - 0.07, 0.31), M.white, bevel=0.004)
        F.soft('pad', 0.62, 0.07, 1.22, (0, PAD - 0.035, 0.32), M.vinyl, round=0.025, puff=0.004, resting=True)
        # the head rest, raised 30 degrees from its hinge
        a = math.radians(30)
        L = 0.64
        cz, cy = HINGE - L / 2 * math.cos(a), PAD - 0.035 + L / 2 * math.sin(a)
        F.soft('head', 0.62, 0.07, L, (0, cy, cz), M.vinyl, round=0.025, puff=0.004, resting=True, rot=(30, 0, 0))
        _rod('strut', (0, PAD - 0.09, HINGE - 0.05), (0, cy - 0.06, cz - 0.05), 0.014, M.white, 8)
        # the paper: a roll on brackets at the head, a sheet down the head rest and along the pad
        ez, ey = HINGE - L * math.cos(a), PAD - 0.035 + L * math.sin(a)
        for x in (-0.27, 0.27):
            F.box(f'roll_bracket{x}', 0.012, 0.08, 0.03, (x, ey + 0.0, ez - 0.03), M.white)
        F.cyl('roll', 0.045, 0.5, (0, ey + 0.03, ez - 0.03), M.paper, rot=(0, 90, 0), verts=14)
        F.box('sheet_head', 0.5, 0.003, L - 0.02, (0, cy + 0.041 / math.cos(a), cz + 0.041 * math.sin(a)), M.paper,
              rot=(30, 0, 0))
        F.box('sheet', 0.5, 0.003, 1.2, (0, PAD + 0.0055, 0.32), M.paper)
    G.join(parts, 'couch', origin=(0, 0, 0))
    G.settle()


# --- the clothes rail: 2x1, a chrome rail on castored feet, garments on hangers ----------------------------------
def _garment(name, x, kind, cloth):
    """A garment on its hanger, seen edge on: a shirt, a dress, a coat or trousers over the bar."""
    top = 1.4
    shapes = {
        'shirt': [(-0.2, top - 0.06), (-0.04, top), (0.04, top), (0.2, top - 0.06), (0.22, top - 0.5), (0.2, top - 0.6),
                  (-0.2, top - 0.6), (-0.22, top - 0.5)],
        'dress': [(-0.18, top - 0.06), (-0.04, top), (0.04, top), (0.18, top - 0.06), (0.15, top - 0.35), (0.26, top - 0.85),
                  (-0.26, top - 0.85), (-0.15, top - 0.35)],
        'coat': [(-0.23, top - 0.07), (-0.05, top), (0.05, top), (0.23, top - 0.07), (0.25, top - 0.4), (0.25, top - 0.8),
                 (-0.25, top - 0.8), (-0.25, top - 0.4)],
        'trousers': [(-0.16, top - 0.03), (0.16, top - 0.03), (0.17, top - 0.5), (-0.17, top - 0.5)],
    }
    t = {'coat': 0.06, 'trousers': 0.03}.get(kind, 0.04)
    F.extrude(name, G.smooth_outline(shapes[kind], passes=1), t, (x, 0, 0), cloth, plane='side', bevel=0.008)
    _tube(f'{name}_hook', [(x, top - 0.005, 0.0), (x, top + 0.07, 0.0), (x, top + 0.115, -0.02), (x, top + 0.1, -0.045)],
          0.003, M.bell, corner=0.02, verts=6)


def clothesRack():
    RAIL = 1.5
    with G.new_parts() as parts:
        for s in (-1, 1):
            x = s * 0.86
            F.box(f'foot{s}', 0.04, 0.03, 0.52, (x, 0.075, 0), M.bell, bevel=0.006)
            for z in (-0.23, 0.23):
                F.cyl(f'castor{s}{z}', 0.03, 0.025, (x, 0.032, z), M.radio, rot=(0, 90, 0), verts=12)
                F.box(f'fork{s}{z}', 0.035, 0.03, 0.03, (x, 0.07, z), M.bell)
        _tube('frame', [(-0.86, 0.09, 0), (-0.86, RAIL, 0), (0.86, RAIL, 0), (0.86, 0.09, 0)], 0.016, M.bell,
              corner=0.06, verts=10)
        kinds = ['coat', 'shirt', 'shirt', 'dress', 'shirt', 'trousers', 'dress', 'shirt', 'coat', 'shirt', 'dress',
                 'trousers', 'shirt', 'shirt', 'dress']
        cloths = [M.cloth2, M.cloth1, M.cloth4, M.cloth3, M.cloth5, M.cloth2, M.cloth1, M.cloth6, M.cloth6, M.cloth4,
                  M.cloth5, M.cloth6, M.cloth3, M.cloth1, M.cloth2]
        for k, (kind, cloth) in enumerate(zip(kinds, cloths)):
            _garment(f'garment{k}', -0.72 + k * 0.103, kind, cloth)
    G.join(parts, 'rail', origin=(0, 0, 0))
    G.settle()


# --- the plant table: 2x1, slatted staging on a galvanised frame, potted plants and seed trays ------------------
def _pot_plant(name, x, y, z, k):
    r = 0.08
    G.revolve(f'{name}_pot', [(0, 0), (r * 0.72, 0), (r * 0.95, r * 1.6), (r * 1.02, r * 1.62), (r * 1.02, r * 1.8),
                              (r * 0.95, r * 1.8), (0, r * 1.7)], at=F.at(x, y, z), material=M.terracotta, segments=12)
    top = y + r * 1.75
    if k % 3 == 2:   # an upright plant, leaves up a stem
        F.cyl(f'{name}_stem', 0.008, 0.3, (x, top + 0.15, z), M.foliage, verts=6)
        for j in range(5):
            a = j * 2.4
            G.jitter(F.sphere(f'{name}_leaf{j}', 0.05, (x + 0.05 * math.cos(a), top + 0.08 + j * 0.05, z + 0.05 * math.sin(a)),
                              M.foliage, scale=(1.3, 0.6, 0.6), segments=8, rings=5), 0.01, size=0.04, seed=k + j)
        return
    for j, (dx, dy, dz, s) in enumerate(((0, 0.06, 0, 1.0), (-0.05, 0.04, 0.03, 0.7), (0.05, 0.045, -0.03, 0.75),
                                         (0.02, 0.04, 0.05, 0.6))):
        G.jitter(F.sphere(f'{name}_mound{j}', 0.08 * s, (x + dx, top + dy, z + dz), M.foliage, scale=(1, 1, 0.75),
                          segments=9, rings=5), 0.015, size=0.05, seed=k * 2.1 + j)
    if k % 3 == 1:   # in flower
        for j in range(6):
            a = j * 1.05 + k
            F.sphere(f'{name}_bloom{j}', 0.018, (x + 0.06 * math.cos(a), top + 0.1 + 0.02 * (j % 2), z + 0.06 * math.sin(a)),
                     [M.bloom1, M.bloom2, M.bloom3][(k // 3) % 3], segments=6, rings=4)
    F.box(f'{name}_tag', 0.03, 0.05, 0.003, (x + 0.05, top + 0.02, z + 0.07), M.label, rot=(-15, 0, 0))


def plantTable():
    TOP = 0.75
    with G.new_parts() as parts:
        for x in (-0.88, 0.88):
            for z in (-0.36, 0.36):
                F.box(f'leg{x}{z}', 0.04, TOP - 0.025, 0.04, (x, (TOP - 0.025) / 2, z), M.galv, bevel=0.004)
            F.box(f'end_rail{x}', 0.03, 0.04, 0.72, (x, TOP - 0.045, 0), M.galv)
            F.box(f'low_end{x}', 0.03, 0.03, 0.72, (x, 0.22, 0), M.galv)
        for z in (-0.36, 0.36):
            F.box(f'rail{z}', 1.76, 0.04, 0.03, (0, TOP - 0.045, z), M.galv)
            F.box(f'low_rail{z}', 1.76, 0.03, 0.03, (0, 0.22, z), M.galv)
        for i in range(8):
            F.box(f'slat{i}', 1.88, 0.022, 0.08, (0, TOP - 0.011, -0.36 + i * 0.103), M.slat, bevel=0.004)
        for i in range(5):
            F.box(f'low_slat{i}', 1.76, 0.02, 0.08, (0, 0.245, -0.32 + i * 0.16), M.slat, bevel=0.004)
        k = 0
        for z in (-0.19, 0.19):
            for i in range(5):
                _pot_plant(f'plant{k}', -0.72 + i * 0.36, TOP, z + 0.02 * ((i % 2) * 2 - 1), k)
                k += 1
        # seed trays on the shelf below
        for t, x in enumerate((-0.45, 0.4)):
            F.box(f'tray{t}', 0.56, 0.05, 0.36, (x, 0.28, 0.0), M.tray, bevel=0.004)
            F.box(f'tray{t}_soil', 0.54, 0.004, 0.34, (x, 0.305, 0.0), M.soil)
            for r in range(3):
                for c in range(5):
                    F.sphere(f'seedling{t}{r}{c}', 0.03, (x - 0.2 + c * 0.1, 0.32, -0.11 + r * 0.11), M.foliage,
                             scale=(1, 1, 0.6), segments=5, rings=3)
    G.join(parts, 'table', origin=(0, 0, 0))
    G.settle()


# --- the cash machine: 1x1 against a wall, a lit band over the screen and keypad ----------------------------------
def atm():
    B = WALL
    with G.new_parts() as parts:
        F.box('plinth', 0.62, 0.06, 0.46, (0, 0.03, B + 0.23), M.plinth)
        F.box('lower', 0.66, 0.95, 0.5, (0, 0.06 + 0.475, B + 0.25), M.atm, bevel=0.008)
        F.box('upper', 0.66, 0.62, 0.36, (0, 1.01 + 0.31, B + 0.18), M.atm, bevel=0.008)
        # the fascia: the screen in a sloping recess, the keypad on a shelf, the slots
        F.box('fascia', 0.56, 0.42, 0.02, (0, 1.3, B + 0.36), M.fascia, bevel=0.004, rot=(10, 0, 0))
        F.box('screen', 0.36, 0.26, 0.006, (0, 1.32, B + 0.373), M.atm_screen, rot=(10, 0, 0))
        F.box('shelf', 0.6, 0.04, 0.16, (0, 1.0, B + 0.56), M.fascia, bevel=0.008, rot=(-12, 0, 0))
        for r in range(4):
            for c in range(3):
                F.box(f'key{r}{c}', 0.04, 0.012, 0.028, (-0.12 + c * 0.05, 1.026 - r * 0.0075, B + 0.52 + r * 0.035 - 0.0),
                      M.keys if c < 3 else M.leaflet4, bevel=0.003, rot=(-12, 0, 0))
        for k, m in enumerate((M.leaflet4, M.leaflet2, M.leaflet3)):
            F.box(f'act{k}', 0.06, 0.012, 0.028, (0.07, 1.026 - k * 0.0075, B + 0.52 + k * 0.035), m, bevel=0.003,
                  rot=(-12, 0, 0))
        F.box('card_slot', 0.1, 0.04, 0.03, (0.2, 1.08, B + 0.515), M.fascia, bevel=0.004)
        F.box('card_mouth', 0.07, 0.006, 0.004, (0.2, 1.08, B + 0.531), M.slot)
        F.box('card_light', 0.07, 0.01, 0.004, (0.2, 1.096, B + 0.531), M.sign)
        F.box('cash_slot', 0.3, 0.05, 0.02, (0, 0.88, B + 0.508), M.fascia, bevel=0.004)
        F.box('cash_mouth', 0.26, 0.012, 0.004, (0, 0.88, B + 0.519), M.slot)
        F.box('receipt_slot', 0.08, 0.008, 0.004, (-0.2, 1.08, B + 0.502), M.slot)
        # privacy wings beside the keypad, and the hood with its lit band
        for s in (-1, 1):
            F.extrude(f'wing{s}', [(B + 0.36, 0.95), (B + 0.62, 0.95), (B + 0.62, 1.05), (B + 0.4, 1.5), (B + 0.36, 1.5)],
                      0.015, (s * 0.325, 0, 0), M.atm, plane='side', bevel=0.004)
        F.box('hood', 0.7, 0.05, 0.52, (0, 1.655, B + 0.26), M.atm, bevel=0.01)
        F.box('band', 0.66, 0.1, 0.012, (0, 1.6, B + 0.512), M.sign)
        F.box('band_rim', 0.7, 0.012, 0.03, (0, 1.545, B + 0.505), M.fascia)
    G.join(parts, 'atm', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the wall bars: 1x1 on a wall, beech rungs between two uprights, the top ones standing out -------------------
def _bars_z(y):
    """The rungs' depth at height `y`: against the wall low down, standing out over the top."""
    if y <= 2.0:
        return WALL + 0.09
    return WALL + 0.09 + (y - 2.0) / 0.5 * 0.12


def wallBars():
    with G.new_parts() as parts:
        up = G.smooth_outline([(WALL + 0.02, 0.0), (WALL + 0.16, 0.0), (WALL + 0.16, 2.0), (WALL + 0.28, 2.5),
                               (WALL + 0.14, 2.5), (WALL + 0.02, 2.0)], passes=1)
        for s in (-1, 1):
            F.extrude(f'upright{s}', up, 0.07, (s * 0.43, 0, 0), M.oak, plane='side', bevel=0.008)
            for y in (0.12, 1.2, 2.3):
                F.box(f'bracket{s}{y}', 0.09, 0.08, 0.02, (s * 0.43, y, WALL + 0.01), M.steel, bevel=0.003)
        for i in range(16):
            y = 0.15 + i * 0.152
            F.cyl(f'rung{i}', 0.018, 0.82, (0, y, _bars_z(y)), M.oak, rot=(0, 90, 0), verts=12)
    G.join(parts, 'bars', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the altar: 2x1, stone under a white linen and a frontal, two candlesticks and a cross ------------------------
def _candlestick(name, x, y, z):
    G.revolve(name, [(0, 0), (0.08, 0), (0.08, 0.015), (0.06, 0.03), (0.03, 0.05), (0.02, 0.1), (0.03, 0.12),
                     (0.016, 0.14), (0.014, 0.3), (0.024, 0.32), (0.05, 0.34), (0.05, 0.35), (0, 0.35)],
              at=F.at(x, y, z), material=M.brass, segments=16)
    F.cyl(f'{name}_candle', 0.022, 0.24, (x, y + 0.35 + 0.12, z), M.candle, verts=12)
    F.sphere(f'{name}_flame', 0.012, (x, y + 0.6 + 0.018, z), M.flame, scale=(1, 1, 2.2), segments=8, rings=6)


def altar():
    with G.new_parts() as parts:
        F.box('step', 1.92, 0.06, 0.84, (0, 0.03, 0), M.stone, bevel=0.008)
        F.box('body', 1.7, 0.92, 0.7, (0, 0.06 + 0.46, -0.02), M.stone, bevel=0.006)
        F.box('mensa', 1.82, 0.06, 0.8, (0, 1.01, -0.02), M.stone, bevel=0.012)
        # the fair linen over the top, hanging down the ends
        F.box('linen', 1.86, 0.008, 0.82, (0, 1.044, -0.02), M.linen)
        for s in (-1, 1):
            F.box(f'linen_drop{s}', 0.008, 0.4, 0.82, (s * 0.934, 0.85, -0.02), M.linen)
        # the frontal down the front, a gold cross and fringe on it
        F.box('frontal', 1.3, 0.8, 0.01, (0, 0.64, 0.344), M.frontal, bevel=0.002)
        F.box('frontal_band', 1.3, 0.06, 0.012, (0, 1.01, 0.346), M.linen)
        F.box('fringe', 1.3, 0.04, 0.012, (0, 0.26, 0.346), M.brass)
        F.box('frontal_cross_v', 0.06, 0.4, 0.004, (0, 0.65, 0.351), M.brass)
        F.box('frontal_cross_h', 0.26, 0.06, 0.004, (0, 0.72, 0.351), M.brass)
        T = 1.048
        for s in (-1, 1):
            _candlestick(f'candlestick{s}', s * 0.62, T, -0.12)
        G.revolve('cross_base', [(0, 0), (0.09, 0), (0.09, 0.02), (0.06, 0.04), (0.04, 0.08), (0, 0.08)],
                  at=F.at(0, T, -0.2), material=M.brass, segments=16)
        F.box('cross_v', 0.035, 0.5, 0.025, (0, T + 0.33, -0.2), M.brass, bevel=0.005)
        F.box('cross_h', 0.26, 0.035, 0.025, (0, T + 0.45, -0.2), M.brass, bevel=0.005)
        _open_book('missal', 0.3, 0.22, (0.3, T, 0.08), M.book, tilt=8)
    G.join(parts, 'altar', origin=(0, 0, 0))
    G.settle()


# --- the lockers: 2x1 against a wall, five columns of two, vents, locks and name cards -----------------------------
def lockerRow():
    D = 0.45
    FRONT = WALL + D
    with G.new_parts() as parts:
        F.box('plinth', 1.88, 0.1, D - 0.04, (0, 0.05, WALL + (D - 0.04) / 2), M.plinth)
        F.box('carcass', 1.9, 1.72, D - 0.02, (0, 0.1 + 0.86, WALL + (D - 0.02) / 2), M.frame, bevel=0.004)
        F.box('top', 1.92, 0.025, D, (0, 1.8325, WALL + D / 2), M.frame, bevel=0.005)
        for c in range(5):
            x = -0.76 + c * 0.38
            for t in range(2):
                y0 = 0.12 + t * 0.85
                y = y0 + 0.415
                F.box(f'door{c}{t}', 0.36, 0.83, 0.016, (x, y, FRONT - 0.012), M.locker, bevel=0.004)
                for v in range(4):
                    for yy in (y0 + 0.73, y0 + 0.1):
                        F.box(f'vent{c}{t}{v}{yy}', 0.18, 0.008, 0.004, (x, yy + v * 0.022 - 0.033, FRONT - 0.003), M.slot)
                F.cyl(f'lock{c}{t}', 0.014, 0.012, (x + 0.13, y, FRONT - 0.002), M.bell, rot=(90, 0, 0), verts=10)
                F.box(f'tag_frame{c}{t}', 0.11, 0.05, 0.004, (x, y0 + 0.62, FRONT - 0.002), M.frame)
                F.box(f'tag{c}{t}', 0.095, 0.036, 0.004, (x, y0 + 0.62, FRONT - 0.0005), M.label)
                F.box(f'number{c}{t}', 0.04, 0.03, 0.003, (x, y0 + 0.77, FRONT - 0.001), M.label)
    G.join(parts, 'lockers', origin=(0, 0, 0))
    G.settle(centre=False)


# --- the mannequin: 1x1, a shop dummy in a jacket and trousers on a chrome stand ---------------------------------
def mannequin():
    with G.new_parts() as parts:
        G.revolve('base', [(0, 0), (0.2, 0), (0.2, 0.008), (0.19, 0.014), (0, 0.016)], material=M.bell, segments=32)
        F.cyl('rod', 0.012, 0.6, (0, 0.31, -0.12), M.bell, verts=10)
        # the legs, in trousers, and shoes
        for s in (-1, 1):
            F.sweep(f'leg{s}', [(s * 0.085, 0.95, -0.01), (s * 0.09, 0.55, 0.0), (s * 0.1, 0.1, -0.02)], 0.07, M.cloth2,
                    verts=12, resolution=2, scale=[1.0, 0.72, 0.62])
            F.soft(f'shoe{s}', 0.085, 0.07, 0.24, (s * 0.1, 0.051, 0.03), M.radio, round=0.03, puff=0.01)
        # the body: hips to shoulders, in a jacket
        bx, by, _ = F.at(0, 0, -0.01)
        G.loft('torso', [G.ring(rx, rz, y, 20, square=2.2, at=(bx, by)) for y, rx, rz in
                         ((0.88, 0.17, 0.11), (0.98, 0.17, 0.11), (1.1, 0.14, 0.095), (1.28, 0.165, 0.11), (1.4, 0.19, 0.1),
                          (1.44, 0.17, 0.09), (1.47, 0.07, 0.06))], material=M.cloth1)
        # the shirt in the jacket's open neck, its buttons
        F.extrude('shirt', [(-0.06, 1.455), (0.06, 1.455), (0.0, 1.27)], 0.02, (0, 0, 0.088), M.cloth4, plane='front')
        for k in range(3):
            F.sphere(f'button{k}', 0.009, (0, 1.2 - k * 0.08, 0.097 - k * 0.002), M.radio, segments=8, rings=6)
        for s in (-1, 1):
            F.sweep(f'arm{s}', [(s * 0.19, 1.4, -0.01), (s * 0.23, 1.15, 0.0), (s * 0.24, 0.9, 0.04)], 0.045, M.cloth1,
                    verts=10, resolution=2, scale=[1.0, 0.85, 0.75])
            F.sphere(f'hand{s}', 0.04, (s * 0.245, 0.84, 0.05), M.dummy, scale=(0.6, 0.8, 1.3), segments=10, rings=8)
        F.cyl('neck', 0.045, 0.12, (0, 1.5, -0.01), M.dummy, verts=12)
        F.sphere('head', 0.1, (0, 1.64, 0.0), M.dummy, scale=(0.82, 0.95, 1.15), segments=18, rings=12)
    G.join(parts, 'mannequin', origin=(0, 0, 0))
    G.settle()


# --- the luggage trolley: 1x1, a carpeted deck on castors under a brass arch, two cases on it ---------------------
def luggageTrolley():
    DECK = 0.17
    with G.new_parts() as parts:
        F.box('deck', 0.88, 0.05, 0.52, (0, DECK - 0.025, 0), M.carpet, bevel=0.006)
        F.box('deck_rim', 0.9, 0.03, 0.54, (0, DECK - 0.06, 0), M.brass, bevel=0.006)
        for x in (-0.38, 0.38):
            for z in (-0.2, 0.2):
                F.box(f'fork{x}{z}', 0.03, 0.06, 0.05, (x, DECK - 0.09, z), M.brass)
                F.cyl(f'wheel{x}{z}', 0.045, 0.03, (x, 0.045, z), M.rubber, rot=(0, 90, 0), verts=14)
        # the uprights and the arch, a hanging rail across
        for s in (-1, 1):
            F.cyl(f'upright{s}', 0.016, 1.45, (s * 0.42, DECK + 0.725, 0), M.brass, verts=10)
            F.sphere(f'knob{s}', 0.03, (s * 0.42, DECK + 1.46, 0), M.brass, segments=12, rings=8)
            _rod(f'bumper{s}', (-0.42, DECK + 0.08, s * 0.25), (0.42, DECK + 0.08, s * 0.25), 0.012, M.brass, 8)
        F.sweep('arch', [(0.42 * math.cos(a), DECK + 1.45 + 0.2 * math.sin(a), 0)
                         for a in [math.radians(d) for d in range(0, 181, 15)]], 0.016, M.brass, verts=10)
        _rod('hanging_rail', (-0.42, DECK + 1.3, 0), (0.42, DECK + 1.3, 0), 0.013, M.brass, 8)
        F.sphere('crown', 0.035, (0, DECK + 1.67, 0), M.brass, segments=12, rings=8)
        # a case standing, another lying on the deck, a garment bag on the rail
        F.soft('case_big', 0.5, 0.62, 0.24, (-0.12, DECK + 0.31, 0.06), M.case1, round=0.03, puff=0.01)
        _tube('case_big_handle', [(-0.2, DECK + 0.62, 0.06), (-0.2, DECK + 0.66, 0.06), (-0.04, DECK + 0.66, 0.06),
                                  (-0.04, DECK + 0.62, 0.06)], 0.01, M.rubber, corner=0.015)
        F.soft('case_small', 0.36, 0.18, 0.42, (0.22, DECK + 0.09, -0.02), M.case2, round=0.03, puff=0.01)
        _tube('case_small_handle', [(0.4, DECK + 0.07, -0.08), (0.43, DECK + 0.07, -0.08), (0.43, DECK + 0.07, 0.04),
                                    (0.4, DECK + 0.07, 0.04)], 0.009, M.rubber, corner=0.012)
        F.soft('garment_bag', 0.08, 0.7, 0.42, (0.28, DECK + 1.3 - 0.36, -0.03), M.radio, round=0.03, puff=0.012)
        _tube('bag_hook', [(0.28, DECK + 1.28, -0.03), (0.28, DECK + 1.34, -0.03), (0.28, DECK + 1.33, -0.0)], 0.004,
              M.bell, corner=0.015, verts=6)
    G.join(parts, 'trolley', origin=(0, 0, 0))
    G.settle()


# --- the vault door: 2x1 on a wall, a round steel door a foot thick in its frame, its wheel and dial -------------
def vaultDoor():
    C = 1.0           # the door's middle, its height
    R = 0.74
    with G.new_parts() as parts:
        F.box('frame', 1.92, 1.92, 0.05, (0, 0.96, WALL + 0.025), M.vault_dark, bevel=0.01)
        bx, by, _ = F.at(0, 0, 0)
        G.revolve('door', [(0, 0), (R, 0), (R, 0.13), (R - 0.03, 0.16), (R - 0.12, 0.16), (R - 0.14, 0.18),
                           (0, 0.18)], at=F.at(0, C, WALL + 0.05), material=M.vault, segments=48, rot=(90, 0, 0))
        G.revolve('door_rim', [(R + 0.06, 0), (R + 0.06, 0.03), (R + 0.03, 0.06), (R + 0.005, 0.06), (R + 0.005, 0)],
                  at=F.at(0, C, WALL + 0.05), material=M.vault_dark, segments=48, rot=(90, 0, 0))
        for k in range(16):
            a = math.tau * k / 16
            F.cyl(f'bolt{k}', 0.022, 0.02, (0.66 * math.cos(a), C + 0.66 * math.sin(a), WALL + 0.21 - 0.03 + 0.01),
                  M.bell, rot=(90, 0, 0), verts=10)
        # the wheel: a hub, six spokes ending in knobs, a ring
        WZ = WALL + 0.29
        F.cyl('hub', 0.08, 0.08, (0, C, WALL + 0.25), M.bell, rot=(90, 0, 0), verts=20)
        for k in range(6):
            a = math.tau * k / 6 + 0.26
            _rod(f'spoke{k}', (0.06 * math.cos(a), C + 0.06 * math.sin(a), WZ), (0.38 * math.cos(a), C + 0.38 * math.sin(a), WZ),
                 0.014, M.bell, 10)
            F.sphere(f'spoke_knob{k}', 0.026, (0.4 * math.cos(a), C + 0.4 * math.sin(a), WZ), M.bell, segments=12, rings=8)
        G.torus('wheel', 0.3, 0.018, at=F.at(0, C, WZ), material=M.bell, rot=(90, 0, 0), major_segments=40,
                minor_segments=8)
        # the dial over it
        F.cyl('dial', 0.075, 0.03, (0, C + 0.5, WALL + 0.245), M.iron, rot=(90, 0, 0), verts=24)
        F.cyl('dial_face', 0.06, 0.006, (0, C + 0.5, WALL + 0.262), M.bell, rot=(90, 0, 0), verts=24)
        F.box('dial_mark', 0.008, 0.03, 0.004, (0, C + 0.59, WALL + 0.232), M.leaflet4)
        # the hinge on the right
        F.cyl('hinge', 0.06, 0.9, (-0.86, C, WALL + 0.13), M.vault_dark, verts=16)
        for y in (C - 0.3, C + 0.3):
            F.box(f'hinge_arm{y}', 0.2, 0.12, 0.08, (-0.74, y, WALL + 0.15), M.vault_dark, bevel=0.01)
    G.join(parts, 'vault', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


FAMILY = G.Family(
    'civic', M,
    [G.Model('receptionDesk', receptionDesk, size=(1.92, 1.374, 0.85), key='model.receptionDesk',
             paint=[('desk', 'Desk'), ('oak', 'Ledge')]),
     G.Model('hotelReception', hotelReception, size=(1.94, 1.481, 0.858), key='model.hotelReception',
             paint=[('walnut', 'Desk'), ('marble', 'Top')]),
     G.Model('dutyDesk', dutyDesk, size=(1.92, 1.93, 0.85), key='model.dutyDesk',
             paint=[('counter', 'Counter'), ('laminate', 'Top')]),
     G.Model('tellerCounter', tellerCounter, size=(1.92, 1.995, 0.85), key='model.tellerCounter',
             paint=[('teller', 'Counter'), ('laminate', 'Worktop')]),
     G.Model('teacherDesk', teacherDesk, size=(1.5, 1.124, 0.71), key='model.teacherDesk',
             paint=[('oak', 'Desk')]),
     G.Model('lectern', lectern, size=(0.66, 1.25, 0.56), key='model.lectern', paint=[('pew_oak', 'Wood')]),
     G.Model('barCounter', barCounter, size=(1.96, 1.47, 0.82), key='model.barCounter',
             paint=[('mahogany', 'Bar'), ('brass', 'Rail')]),
     G.Model('schoolDesk', schoolDesk, size=(0.62, 0.828, 0.778), key='model.schoolDesk',
             paint=[('pine', 'Wood'), ('iron', 'Frame')]),
     G.Model('pew', pew, size=(2.976, 1.04, 0.67), key='model.pew', paint=[('pew_oak', 'Wood')]),
     G.Model('waitingBench', waitingBench, size=(2.7, 0.85, 0.53), key='model.waitingBench',
             paint=[('shell', 'Seats'), ('steel', 'Frame')]),
     G.Model('examCouch', examCouch, size=(0.62, 0.809, 1.869), key='model.examCouch',
             paint=[('vinyl', 'Pad'), ('white', 'Frame')]),
     G.Model('clothesRack', clothesRack, size=(1.76, 1.514, 0.52), key='model.clothesRack',
             paint=[('bell', 'Rail')]),
     G.Model('plantTable', plantTable, size=(1.88, 1.206, 0.801), key='model.plantTable', far=0.25,
             paint=[('slat', 'Slats'), ('terracotta', 'Pots')]),
     G.Model('atm', atm, size=(0.7, 1.68, 0.641), key='model.atm', centre=(0, -0.11), paint=[('atm', 'Cabinet'), ('fascia', 'Fascia')]),
     G.Model('wallBars', wallBars, size=(0.95, 2.5, 0.25), key='model.wallBars', centre=(0, -0.305), floor=False, paint=[('oak', 'Wood')]),
     G.Model('altar', altar, size=(1.92, 1.692, 0.85), key='model.altar',
             paint=[('stone', 'Stone'), ('frontal', 'Frontal')]),
     G.Model('lockerRow', lockerRow, size=(1.92, 1.845, 0.454), key='model.lockerRow', centre=(0, -0.203),
             paint=[('locker', 'Doors'), ('frame', 'Frame')]),
     G.Model('mannequin', mannequin, size=(0.547, 1.755, 0.4), key='model.mannequin',
             paint=[('cloth1', 'Jacket'), ('cloth2', 'Trousers'), ('dummy', 'Dummy')]),
     G.Model('luggageTrolley', luggageTrolley, size=(0.9, 1.874, 0.54), key='model.luggageTrolley',
             paint=[('brass', 'Frame'), ('carpet', 'Carpet')]),
     G.Model('vaultDoor', vaultDoor, size=(1.92, 1.92, 0.316), key='model.vaultDoor', centre=(0, -0.272), floor=False,
             paint=[('vault', 'Door'), ('vault_dark', 'Frame')])],
    comment='Blender-built fittings for the town\'s named places (tools/art/blender/families/civic.py; '
            'content/civic.json): front desks and counters, the church\'s pews, lectern and altar, the pub\'s bar, '
            'school desks, the waiting room\'s bench, the couch, shop rails and the rest. Listed in the base '
            'manifest\'s packs. Nothing moves.',
)

if __name__ == '__main__':
    _content_facts()
    FAMILY.run(sys.argv)
