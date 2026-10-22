# The kitchen's base counters (K4): the plain counter and its corner, the counter end with open
# shelves, the outer corner counter, the drawer counter, the open-shelf counter, the butcher block and
# the marble counter. Notes: kitchen_counters.md.
#
# A base counter is a plinth, a carcass and a worktop; the front carries a drawer over two doors (the
# plain, corner and marble counters), four drawers, one door, or nothing (the open shelves). The four
# with a `sandwich` use slide their top drawer out 25 cm. The parts are placed in game axes (`F`): the
# old part's depths are shifted back by the old part's depth centre, so the model is centred on
# its tile (see GameFrame). There is one worktop height, 0.95, the content's `surface`,
# for the catalogue counters (`_unit66`) and the catalog2 ones (`_unit79`) alike. There is one depth, the
# catalogue's 0.66: the catalog2 counters keep the old 0.79-deep numbers and are drawn
# 0.66 deep through `COUNTER66` (as the old `shallow`), so the fronts of a mixed run line up. Every
# back stands on the wall face (-0.43).
#
# The worktop's nose is rolled over and its ends cut square (`_worktop`), the fronts are shaker doors and
# drawers (`_shaker`) with swept pulls (`_pull`), the drawers hold cutlery; what stands on and in them is turned
# (`_lathe`). Fronts, handles and the worktop are placed at drawn depths (`_plain`, F.depth), so they keep their
# real thickness on a counter drawn through COUNTER66, and no pull reaches past the worktop's nose.
#
# Run:  cargo art kitchen_counters [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    cab=('#EDEBE6', 'satin'),        # a cabinet carcass and its fronts; the main part the player colours
    worktop=('#E8E6E1', 'gloss'),    # a laminate worktop
    dark=('#2F3236', 'satin'),       # plinths
    chrome=('#D0D3D6', 'chrome'),    # handles and pulls
    oak=('#CDB08A', 'wood'),         # a wood end unit, open shelves and a utensil handle
    navy=('#3E5470', 'satin'),       # a painted cabinet (the marble counter)
    marble=('#F4F3F0', 'gloss'),     # a marble worktop
    vein=('#C9CBCC', 'gloss'),       # a vein through the marble
    brass=('#B8925A', 'metal'),      # brass handles
    block=('#B98A5A', 'wood'),       # a butcher-block top
    block2=('#C99A68', 'wood'),      # the top layer of the block
    leg=('#9C6B43', 'wood'),         # the block's legs and shelf
    wood2=('#8A5A38', 'satin'),      # the knife drawer's front (plain, so it draws with the body's group)
    spoon=('#CDB08A', 'satin'),      # the corner counter's wooden spoons (plain: no draw call of their own)
    basket=('#C9A979', 'satin'),     # a basket
    metal=('#2B2C2E', 'metal'),      # a dark metal handle
    porcelain=('#F5F3EE', 'gloss'),  # plates, a bowl, a jar
    cream=('#EFE5D3', 'gloss'),      # a cream jar
    rust=('#B5653F', 'gloss'),       # a terracotta pot
    teal=('#4E7A7F', 'gloss'),       # a teal pot
    sage=('#B9C9C4', 'gloss'),       # a sage pot
    yellow=('#F2C14E', 'satin'),     # a lemon
    soil=('#3B2E24', 'matte'),       # the dark back of an open shelf
    oakd=('#8A5A38', 'wood'),        # a darker wood: the brackets under a wall shelf
    steel=('#B9BDC1', 'metal'),      # a cooker hood, a bin: brushed stainless
    trim=('#3A3D41', 'satin'),       # dark trim, keys and a hood's filter
    bulb=('#FFF1CF', 'gloss'),       # a strip light or a hood lamp
    copper=('#B87333', 'metal'),     # copper pans on the pot rail
    glass=('#DCE6EA', 'glass'),      # glazed doors, jars and glasses
    back=('#C9D6DA', 'satin'),       # a cabinet's back, seen through its glass
    leaf=('#5E8F4E', 'satin'),       # a pot plant's leaves (plain: foliage is a draw call of its own)
    bin1=('#3E7CC2', 'satin'),       # recycling bins: blue, green and yellow
    bin2=('#4E9A5A', 'satin'),
    bin3=('#E3B23C', 'satin'),
    lid=('#2F3236', 'satin'),        # a bin's lid
    walnut=('#7A5234', 'satin'),     # a bar's body and slats
    island=('#9DAF8C', 'satin'),     # the kitchen island's sage stool seats
    towel=('#B4483C', 'satin'),      # a red tea towel
)

SECONDS = 1.4   # a drawer slides in this long
HOLD = 1.5      # seconds it stays open once a use starts
DOOR_SECONDS = 1.6   # a cupboard door swings in this long
DOOR_HOLD = 2.5      # seconds a pantry's doors stay open once a use starts
WALL_DOOR_SECONDS = 0.8   # a wall cabinet's door swings open in this long, while clean things are put away
# catalog2's 0.79-deep counter numbers (worktop -0.48..0.31, carcass -0.46..0.24) as drawn: 0.66 deep, the
# worktop -0.43..0.23 and the carcass -0.41..0.19 (the old `COUNTER66`; gamekit GameFrame `depths`)
COUNTER66 = ((-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23))


def _plain(F):
    """The frame `F` draws in, without its `depths`: for parts placed at drawn depths (F.depth), so a pull or a
    door keeps its real thickness on a counter drawn shallower (COUNTER66)."""
    return G.GameFrame(shift=F.shift)


def _arc(cx, cy, r, a0, a1, n):
    """Points on a circular arc round (cx, cy) from a0 to a1 degrees, n steps (n + 1 points)."""
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)), cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n)))
            for i in range(n + 1)]


def _pull(P, name, x, y, z, length, material=None, vertical=False, reach=0.019, r=0.005, base=0.0):
    """A bar pull in one piece on a front whose face is at drawn depth `z`: round legs out from the face that bend
    into the bar, its front `reach` off the face (no farther than the worktop's nose). `base`: how far behind the
    face the legs start (a pull over a recessed panel)."""
    out, h, k = reach - r, length / 2, min(0.012, reach * 0.5)

    def p(a, d):
        return (x, y + a, z + d) if vertical else (x + a, y, z + d)
    pts = [p(-h, -0.002 - base), p(-h, out - k), p(-h + k, out), p(h - k, out), p(h, out - k), p(h, -0.002 - base)]
    return P.sweep(name, pts, r, material or M.chrome, verts=8, resolution=2)


def _shaker(P, name, w, h, x, y, zb, material, rail=0.06, t=0.02, mid=None):
    """A shaker door or drawer front, its back at drawn depth `zb`: a frame `rail` wide round a flat panel set 8 mm
    in, its edges eased; `mid`, a rail across at that height too (a tall door's two panels)."""
    holes = []
    if mid is None:
        holes.append(G.rounded_rect(w - 2 * rail, h - 2 * rail, 0.002, corner=1))
    else:
        m = mid - y   # the middle rail's centre, from the door's
        for lo, hi in ((-h / 2 + rail, m - rail / 2), (m + rail / 2, h / 2 - rail)):
            holes.append([(u, v + (lo + hi) / 2) for u, v in G.rounded_rect(w - 2 * rail, hi - lo, 0.002, corner=1)])
    P.extrude(name, G.rounded_rect(w, h, 0.004, corner=1), t, (x, y, zb + t / 2), material, holes=holes, bevel=0.003,
              bevel_segments=1)
    # each panel: a margin under the frame's edge, and the field inside it standing a little proud with its edges
    # eased, its corners clear of the frame (the baked shading darkens a face by its corners)
    tp = t - 0.008
    for i, hole in enumerate(holes):
        us, vs = [u for u, _ in hole], [v for _, v in hole]
        pw, ph, pv = max(us) - min(us), max(vs) - min(vs), (max(vs) + min(vs)) / 2
        inner = G.rounded_rect(pw - 0.02, ph - 0.02, 0.002, corner=1)
        P.extrude(f'{name}_panel{i}', G.rounded_rect(pw + 0.006, ph + 0.006, 0.002, corner=1), tp,
                  (x, y + pv, zb + tp / 2), material, holes=[inner])
        P.box(f'{name}_field{i}', pw - 0.02, ph - 0.02, tp + 0.003, (x, y + pv, zb + (tp + 0.003) / 2), material,
              bevel=0.003, bevel_segments=1)


def _worktop(F, name, w, x, zb, zf, y0, y1, material, r=0.012):
    """A worktop from game depth `zb` (its back, on the wall) to `zf` (its front), drawn through F's depths, y0 to y1
    high: the front edge rolled over (postformed), its underside eased; the ends cut square, so the tops of a run
    meet without a seam."""
    zb, zf = F.depth(zb), F.depth(zf)
    prof = [(zb, y0), (zf - 0.004, y0), (zf, y0 + 0.004)] + _arc(zf - r, y1 - r, r, 0, 90, 4) + [(zb, y1)]
    return _plain(F).extrude(name, prof, w, (x, 0, 0), material, plane='side', smooth=True)


def _doors(P, w, front, y0, y1, body=None, hm=None, hy=None, x=0.0, reach=0.019):
    """Two shaker doors on the carcass front at drawn depth `front`, a 4 mm gap between them, their pulls upright on
    the meeting stiles near the top."""
    body, hm = body or M.cab, hm or M.chrome
    dw = (w - 0.004) / 2
    for s in (-1, 1):
        cx = x + s * (dw + 0.004) / 2
        _shaker(P, f'door{s}', dw, y1 - y0, cx, (y0 + y1) / 2, front, body)
        _pull(P, f'door_pull{s}', x + s * 0.034, hy if hy is not None else y1 - 0.10, front + 0.02, 0.14, hm,
              vertical=True, reach=reach)


# Cutlery as flat outlines (game x across, depth along; the handle at +depth, towards whoever opens the drawer).
def _cutlery_outline(kind, s=1.0):
    if kind == 'spoon':
        right = [(0.006, 0.085), (0.0068, 0.072), (0.0045, 0.01), (0.0028, -0.026)] + \
                [(0.017 * math.sin(math.radians(a)), -0.057 + 0.027 * math.cos(math.radians(a))) for a in (30, 60, 90, 120, 150)]
        tip = [(0.0, -0.085)]
    elif kind == 'fork':
        right = [(0.006, 0.085), (0.0068, 0.072), (0.0045, 0.01), (0.003, -0.02), (0.0105, -0.042), (0.011, -0.085)]
        tip = [(0.0075, -0.085), (0.0075, -0.058), (0.0048, -0.058), (0.0048, -0.085), (0.0014, -0.085), (0.0014, -0.058),
               (-0.0014, -0.058), (-0.0014, -0.085), (-0.0048, -0.085), (-0.0048, -0.058), (-0.0075, -0.058),
               (-0.0075, -0.085)]
    else:   # a knife: a round-ended blade, its edge on the right
        right = [(0.006, 0.085), (0.0065, 0.07), (0.006, 0.005), (0.004, -0.002), (0.009, -0.012), (0.0085, -0.06),
                 (0.005, -0.08)]
        tip = [(0.0, -0.087)]
        left = [(-0.004, -0.08), (-0.004, -0.002), (-0.006, 0.005), (-0.0065, 0.07), (-0.006, 0.085)]
        return [(u * s, v * s) for u, v in [(0.0, 0.088)] + right + tip + left]
    left = [(-u, v) for u, v in reversed(right)]
    return [(u * s, v * s) for u, v in [(0.0, 0.088)] + right + tip + left]


def _flat(P, name, outline, x, y, z, material, t=0.0025):
    """A flat piece (cutlery, a spatula) lying at drawn (x, y, z), `t` thick, its outline in (x, depth)."""
    return P.extrude(name, outline, t, (x, y + t / 2, z), material, plane='top')


def _cutlery(P, name, x, y, z, width=0.4, depth=0.34):
    """A cutlery tray standing at height `y` with its front at drawn depth `z`: a pale wooden tray of four
    compartments across, spoons, forks and knives in three, a ladle-length serving spoon in the fourth."""
    zc = z - depth / 2
    P.box(f'{name}_floor', width, 0.006, depth, (x, y + 0.003, zc), M.spoon)
    for i in range(5):
        P.box(f'{name}_wall{i}', 0.006, 0.045, depth, (x - width / 2 + 0.003 + (width - 0.006) * i / 4, y + 0.0225, zc), M.spoon)
    for s in (-1, 1):
        P.box(f'{name}_end{s}', width, 0.045, 0.006, (x, y + 0.0225, zc + s * (depth / 2 - 0.003)), M.spoon)
    cw = (width - 0.006) / 4
    for i, kind in enumerate(('spoon', 'fork', 'knife')):
        cx = x - width / 2 + 0.003 + cw * (i + 0.5)
        for j in range(2):
            _flat(P, f'{name}_{kind}{j}', _cutlery_outline(kind), cx + (j - 0.5) * 0.012, y + 0.006 + j * 0.0028,
                  zc + 0.01 - 0.02 * j, M.steel)
    _flat(P, f'{name}_server', _cutlery_outline('spoon', 1.55), x + width / 2 - cw / 2, y + 0.006, zc, M.steel)


def _drawer(P, name, w, y, zf, material, hm=None, handle_w=0.18, contents=True, reach=0.019, rail=0.04, h=0.17):
    """A shaker drawer front, its face at drawn depth `zf`, with a bar pull and the box behind it (hidden inside the
    carcass at rest, a cutlery tray in it), as its own object, for `drawer_open`."""
    zb = zf - 0.02
    with G.new_parts() as parts:
        _shaker(P, f'{name}_front', w, h, 0, y, zb, material, rail=rail)
        _pull(P, f'{name}_pull', 0, y, zf, handle_w, hm, reach=reach, base=0.008)
        F = P
        F.box(f'{name}_bottom', w - 0.06, 0.012, 0.5, (0, y - 0.074, zb - 0.25), material)
        for s in (-1, 1):
            F.box(f'{name}_side{s}', 0.014, 0.14, 0.5, (s * (w / 2 - 0.037), y - 0.01, zb - 0.25), material, bevel=0.003,
                  bevel_segments=1)
        F.box(f'{name}_back', w - 0.06, 0.14, 0.014, (0, y - 0.01, zb - 0.493), material, bevel=0.003, bevel_segments=1)
        if contents:
            _cutlery(P, f'{name}_tray', -w / 2 + 0.26, y - 0.068, zb - 0.02)
    return G.join(parts, name, origin=P.at(0, y, zf - 0.01))


def _unit66(F, w=1.0, body=None, top=None):
    """A base unit as the catalogue's `k_base`: worktop to 0.95, front face at depth 0.14."""
    body, top = body or M.cab, top or M.worktop
    F.box('carcass', w - 0.004, 0.82, 0.6, (0, 0.51, -0.16), body, bevel=0.01)
    F.box('plinth', w - 0.04, 0.10, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.004)
    _worktop(F, 'worktop', w, 0, -0.48, 0.18, 0.91, 0.95, top)


def _unit79(F, w=1.0, x=0.0, body=None, top=None, worktop=True):
    """A base unit as catalog2's `counter_body`: worktop to 0.95, front face at depth 0.24 (`worktop` False: none,
    for a unit under a worktop laid over more than it)."""
    body, top = body or M.cab, top or M.worktop
    F.box('carcass', w - 0.004, 0.8, 0.7, (x, 0.5, -0.11), body, bevel=0.01)
    F.box('plinth', w - 0.004, 0.10, 0.64, (x, 0.05, -0.14), M.dark, bevel=0.004)
    if worktop:
        _worktop(F, 'worktop', w, x, -0.48, 0.31, 0.90, 0.95, top)


def _lathe(P, name, profile, x, y, z, material, verts=12):
    """A round thing turned about the upright through drawn (x, y, z), its foot there: `profile` [(radius, height),
    ...] from the bottom up, inside and out (a radius of 0 closes it)."""
    bx, by, bz = P.at(x, y, z)
    return G.loft(name, [G.ring(max(r, 1e-5), max(r, 1e-5), bz + h, verts, at=(bx, by)) for r, h in profile],
                  material=material)


def _plates(P, name, x, y, z, r, n, material, h=0.013, verts=14):
    """A stack of `n` plates standing on `y`: each a foot, a rim and a lip, the top one's well."""
    prof = [(0.0, 0.0), (0.55 * r, 0.0)]
    for i in range(n):
        b = i * h
        prof += [(0.62 * r, b + 0.003), (r, b + 0.75 * h), (0.97 * r, b + h)]
    prof += [(0.62 * r, n * h - 0.004), (0.0, n * h - 0.004)]
    return _lathe(P, name, prof, x, y, z, material, verts)


def _bowl(P, name, x, y, z, r, h, material, verts=12):
    """A bowl standing on `y`, hollow: its foot, a curved side, a rolled lip."""
    prof = [(0.0, 0.0), (0.42 * r, 0.0), (0.48 * r, 0.08 * h), (0.8 * r, 0.35 * h), (0.97 * r, 0.75 * h), (r, h),
            (0.92 * r, h), (0.86 * r, 0.72 * h), (0.6 * r, 0.3 * h), (0.0, 0.2 * h)]
    return _lathe(P, name, prof, x, y, z, material, verts)


def _mug(P, name, x, y, z, r, h, material, turn=0.0, verts=12):
    """A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the front)."""
    _lathe(P, name, [(0.0, 0.0), (0.88 * r, 0.0), (r, 0.06 * h), (r, h), (0.86 * r, h), (0.86 * r, 0.15 * h), (0.0, 0.15 * h)],
           x, y, z, material, verts)
    c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    pts = [(x + c * d, y + t * h, z + s * d) for d, t in ((r - 0.003, 0.8), (r + 0.016, 0.82), (r + 0.024, 0.55),
                                                          (r + 0.015, 0.27), (r - 0.003, 0.25))]
    P.sweep(f'{name}_handle', pts, 0.0045, material, verts=6, resolution=2)


def _jug(P, name, x, y, z, r, h, material, turn=0.0, verts=14):
    """A pitcher standing on `y`: a full belly narrowing to a flared neck, hollow at the top, a handle at `turn`."""
    _lathe(P, name, [(0.0, 0.0), (0.8 * r, 0.0), (0.95 * r, 0.1 * h), (r, 0.38 * h), (0.78 * r, 0.75 * h), (0.74 * r, 0.88 * h),
                     (0.84 * r, h), (0.76 * r, h), (0.66 * r, 0.86 * h), (0.0, 0.84 * h)], x, y, z, material, verts)
    c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    pts = [(x + c * d, y + t * h, z + s * d) for d, t in ((0.72 * r, 0.9), (0.9 * r + 0.03, 0.88), (r + 0.03, 0.6),
                                                          (r + 0.01, 0.32), (0.92 * r, 0.25))]
    P.sweep(f'{name}_handle', pts, 0.006, material, verts=6, resolution=2)


def _canister(P, name, x, y, z, r, h, material, lid, verts=14):
    """A lidded canister standing on `y`: a straight body, a lid with a knob."""
    _lathe(P, name, [(0.0, 0.0), (0.92 * r, 0.0), (r, 0.04 * h), (r, 0.86 * h), (0.0, 0.86 * h)], x, y, z, material, verts)
    _lathe(P, f'{name}_lid', [(0.0, 0.84 * h), (1.04 * r, 0.84 * h), (1.04 * r, 0.9 * h), (0.3 * r, 0.95 * h), (0.3 * r, h),
                              (0.0, h)], x, y, z, lid, verts)


def _can(P, name, x, y, z, r, h, material, verts=12):
    """A tin can standing on `y`: a body with a ridge round its middle and a rolled rim round its lid."""
    _lathe(P, name, [(0.0, 0.0), (0.96 * r, 0.0), (r, 0.03 * h), (r, 0.48 * h), (0.97 * r, 0.5 * h), (r, 0.52 * h),
                     (r, 0.95 * h), (1.03 * r, 0.97 * h), (1.03 * r, h), (0.0, h)], x, y, z, material, verts)


# --- the plain counter: 1x1, a drawer over two doors, the drawer sliding out for a sandwich -----------
def counter():
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _unit66(F, w=1.0)
        _doors(F, w=0.974, front=0.14, y0=0.12, y1=0.72, hy=0.62)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.974, y=0.815, zf=0.16, material=M.cab)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


def _utensil_crock(F, cx, y, cz):
    """A stoneware crock on the worktop at (cx, y, cz), a wooden spoon, a slotted spatula and a whisk standing in it
    (their tops at 1.26)."""
    _lathe(F, 'crock', [(0.0, 0.0), (0.06, 0.0), (0.066, 0.008), (0.07, 0.03), (0.064, 0.15), (0.066, 0.172),
                        (0.07, 0.18), (0.062, 0.18), (0.058, 0.16), (0.056, 0.03), (0.0, 0.03)], cx, y, cz, M.porcelain,
           verts=16)
    F.sweep('spoon_handle', [(cx - 0.01, y + 0.04, cz), (cx - 0.024, y + 0.16, cz + 0.006), (cx - 0.034, y + 0.248, cz + 0.01)],
            0.0065, M.spoon, verts=6, scale=[1.0, 0.9, 0.75])
    F.sphere('spoon_bowl', 0.028, (cx - 0.037, y + 0.276, cz + 0.011), M.spoon, scale=(0.8, 0.4, 1.25), segments=10, rings=6)
    F.sweep('spatula_handle', [(cx + 0.012, y + 0.04, cz - 0.004), (cx + 0.026, y + 0.16, cz - 0.01),
                               (cx + 0.034, y + 0.215, cz - 0.012)], 0.0065, M.spoon, verts=6, scale=[1.0, 0.9, 0.8])
    slots = [[(u + du, v) for u, v in G.rounded_rect(0.007, 0.04, 0.0035, corner=2)] for du in (-0.013, 0.0, 0.013)]
    F.extrude('spatula', G.rounded_rect(0.052, 0.07, 0.014, corner=2), 0.004, (cx + 0.036, y + 0.248, cz - 0.012), M.spoon,
              holes=slots, rot=(0, 0, 8))
    F.cyl('whisk_handle', 0.008, 0.12, (cx + 0.004, y + 0.11, cz + 0.022), M.steel, verts=8)
    for i in range(3):
        a = math.radians(60 * i)
        c, s = math.cos(a), math.sin(a)
        loop = [(d, t) for d, t in ((0.003, 0.17), (0.018, 0.215), (0.024, 0.255), (0.016, 0.292), (0.0, 0.308),
                                    (-0.016, 0.292), (-0.024, 0.255), (-0.018, 0.215), (-0.003, 0.17))]
        F.sweep(f'whisk_wire{i}', [(cx + 0.004 + c * d, y + t, cz + 0.022 + s * d) for d, t in loop], 0.0015, M.steel, verts=4,
                resolution=2)


# --- the corner counter: 1x1, a deep L unit with a drawer, two doors and a utensil pot ----------------
def counterCorner():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        F.box('carcass', 1.0, 0.82, 0.89, (0, 0.51, 0.015), M.cab, bevel=0.012)       # its back on the wall face
        F.box('plinth', 0.96, 0.10, 0.85, (0, 0.05, -0.005), M.dark, bevel=0.006)
        _worktop(F, 'worktop', 1.0, 0, -0.43, 0.50, 0.91, 0.95, M.worktop)
        _doors(F, w=0.974, front=0.46, y0=0.12, y1=0.72, hy=0.62)
        # a crock of wooden spoons, a spatula and a whisk on the worktop
        _utensil_crock(F, -0.3, 0.95, -0.3)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.974, y=0.815, zf=0.48, material=M.cab)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


def _shelf_unit(F, x):
    """The counter end's open unit of oak at x (0.16 wide): a back, two sides, a bottom and two shelves, each
    shelf with a lip along its front."""
    F.box('end_back', 0.16, 0.8, 0.03, (x, 0.5, -0.43), M.oak, bevel=0.004)
    for s in (-1, 1):
        F.box(f'end_side{s}', 0.012, 0.8, 0.7, (x + s * 0.074, 0.5, -0.11), M.oak, bevel=0.004)
    F.box('end_bottom', 0.16, 0.05, 0.7, (x, 0.125, -0.11), M.oak, bevel=0.004)
    for i, y in enumerate((0.38, 0.66)):
        F.box(f'end_shelf{i}', 0.136, 0.025, 0.66, (x, y, -0.11), M.oak, bevel=0.004)
        F.box(f'end_lip{i}', 0.136, 0.018, 0.012, (x, y + 0.018, 0.21), M.oak, bevel=0.003, bevel_segments=1)


# --- the counter end: 1x1, a unit with one door and an open end unit of two shelves -------------------
def counterEnd():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    P, front = _plain(F), F.depth(0.24)
    with G.new_parts() as still:
        _unit79(F, w=0.84, x=-0.08, worktop=False)
        _shaker(P, 'door', 0.808, 0.74, -0.08, 0.49, front, M.cab)
        _pull(P, 'door_pull', -0.452, 0.74, front + 0.02, 0.14, vertical=True)
        # one worktop over the unit and the open end, as a run's
        _worktop(F, 'worktop', 1.0, 0, -0.48, 0.31, 0.90, 0.95, M.worktop)
        # the open end unit, a jug, a canister and two tins on its shelves
        _shelf_unit(F, 0.42)
        _jug(F, 'jug', 0.42, 0.3925, 0.0, 0.05, 0.12, M.porcelain, turn=-90)
        _canister(F, 'jar', 0.42, 0.6725, -0.15, 0.05, 0.12, M.cream, M.spoon)
        _can(F, 'tin', 0.42, 0.15, -0.05, 0.05, 0.09, M.rust)
        _can(F, 'tin2', 0.42, 0.15, 0.08, 0.045, 0.11, M.teal)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the outer corner counter: 1x1, a short unit with a side panel, a plate and a lemon ---------------
def counterCornerOuter():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    P, front = _plain(F), F.depth(0.24)
    with G.new_parts() as still:
        F.box('carcass', 0.74, 0.8, 0.7, (-0.13, 0.5, -0.11), M.cab, bevel=0.01)
        F.box('plinth', 0.70, 0.10, 0.64, (-0.15, 0.05, -0.14), M.dark, bevel=0.004)
        _worktop(F, 'worktop', 0.81, -0.095, -0.48, 0.31, 0.90, 0.95, M.worktop)
        _shaker(P, 'door', 0.708, 0.74, -0.13, 0.49, front, M.cab)
        _pull(P, 'door_pull', -0.452, 0.74, front + 0.02, 0.14, vertical=True)
        # the end panel that closes the run, a towel rail on its outer face with a tea towel over it
        F.box('panel', 0.02, 0.74, 0.66, (0.25, 0.49, -0.11), M.cab, bevel=0.006)
        zr = F.depth(0.12)
        P.sweep('towel_rail', [(0.258, 0.74, zr - 0.13), (0.274, 0.74, zr - 0.122), (0.274, 0.74, zr + 0.03),
                               (0.258, 0.74, zr + 0.04)], 0.005, M.chrome, verts=8, resolution=2)
        zt = zr - 0.045
        P.sweep('towel', [(0.266, 0.56, zt), (0.266, 0.735, zt), (0.2675, 0.7475, zt), (0.274, 0.7505, zt),
                          (0.2805, 0.7475, zt), (0.282, 0.735, zt), (0.282, 0.6, zt)], None, M.towel,
                profile=G.rounded_rect(0.15, 0.003, 0.0014, corner=1), up=(0, 1, 0), resolution=2)
        # a plate with two lemons and a leaf on the worktop
        _plates(F, 'plate', 0.12, 0.95, 0.08, 0.12, 1, M.porcelain, h=0.016, verts=20)
        for i, (x, z, t) in enumerate(((0.10, 0.07, 10), (0.16, 0.12, 70))):
            G.revolve(f'lemon{i}', [(0.0, -0.05), (0.008, -0.046), (0.022, -0.035), (0.033, -0.015), (0.035, 0.0),
                                    (0.033, 0.015), (0.022, 0.035), (0.008, 0.046), (0.0, 0.05)],
                      at=F.at(x, 1.0, z), material=M.yellow, segments=10, rot=(0, 90, t))
        F.sphere('leaf', 0.03, (0.06, 0.968, 0.11), M.leaf, scale=(1.4, 0.12, 0.6), rot=(0, 0, 35), segments=8, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the drawer counter: 1x1, four deep drawers under the worktop (static) ----------------------------
def counterDrawers():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    P, front = _plain(F), F.depth(0.24)
    with G.new_parts() as still:
        _unit79(F, w=1.0)
        for i, y in enumerate((0.2125, 0.3975, 0.5825)):
            _shaker(P, f'drawer{i}', 0.97, 0.173, 0, y, front, M.cab, rail=0.04)
            _pull(P, f'drawer_pull{i}', 0, y, front + 0.02, 0.30, base=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # the top drawer with its box and a cutlery tray behind, sliding out while a resident puts clean dishes away
    drawer = _drawer(P, 'drawer', 0.97, 0.7675, front + 0.02, M.cab, handle_w=0.30, h=0.173)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the open-shelf counter: 1x1, an open carcass with two shelves of plates and pots (static) --------
def counterOpenShelf():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        F.box('plinth', 0.996, 0.10, 0.64, (0, 0.05, -0.14), M.dark, bevel=0.004)
        _worktop(F, 'worktop', 1.0, 0, -0.48, 0.31, 0.90, 0.95, M.worktop)
        # an open carcass: a back, two sides, a top and a bottom, with two shelves between
        # (dark inside, the cabinet's colour outside, for one standing free)
        F.box('back', 0.996, 0.8, 0.02, (0, 0.5, -0.42), M.soil, bevel=0.004)
        F.box('back_skin', 0.996, 0.8, 0.02, (0, 0.5, -0.44), M.cab, bevel=0.004)
        for s in (-1, 1):
            F.box(f'side{s}', 0.03, 0.8, 0.66, (s * 0.483, 0.5, -0.11), M.cab, bevel=0.004)
        F.box('top', 0.996, 0.03, 0.7, (0, 0.885, -0.11), M.cab, bevel=0.004)
        F.box('bottom', 0.996, 0.05, 0.7, (0, 0.125, -0.11), M.cab, bevel=0.004)
        for i, y in enumerate((0.16, 0.5)):
            F.box(f'shelf{i}', 0.93, 0.025, 0.66, (0, y, -0.11), M.oak, bevel=0.004)
            F.box(f'shelf_lip{i}', 0.93, 0.04, 0.02, (0, y + 0.0075, 0.21), M.oak, bevel=0.004, bevel_segments=1)
        # plates, nested bowls and a serving dish below; a jug, a lidded casserole and a dish above
        lo, hi = 0.1725, 0.5125
        _plates(F, 'plates0', -0.32, lo, -0.1, 0.1, 5, M.porcelain, verts=12)
        for i in range(3):
            _bowl(F, f'bowl{i}', -0.08, lo + 0.022 * i, -0.1, 0.075 - 0.008 * i, 0.07 - 0.006 * i, M.cream, verts=10)
        _plates(F, 'plates1', 0.14, lo, -0.1, 0.085, 4, M.porcelain, verts=12)
        _bowl(F, 'bowl3', 0.35, lo, -0.12, 0.065, 0.075, M.cream, verts=10)
        _jug(F, 'jug', -0.3, hi, -0.1, 0.07, 0.16, M.sage, turn=200, verts=12)
        _lathe(F, 'pot', [(0.0, 0.0), (0.07, 0.0), (0.075, 0.008), (0.075, 0.1), (0.0, 0.1)], -0.08, hi, -0.1, M.teal,
               verts=12)
        _lathe(F, 'pot_lid', [(0.0, 0.098), (0.078, 0.098), (0.074, 0.11), (0.04, 0.125), (0.012, 0.127), (0.016, 0.142),
                              (0.0, 0.145)], -0.08, hi, -0.1, M.teal, verts=12)
        for s in (-1, 1):
            F.box(f'pot_handle{s}', 0.024, 0.014, 0.04, (-0.08 + s * 0.086, hi + 0.085, -0.1), M.teal, bevel=0.006,
                  bevel_segments=1)
        _bowl(F, 'dish0', 0.25, hi, -0.1, 0.11, 0.05, M.porcelain, verts=12)
        _bowl(F, 'dish1', 0.25, hi + 0.04, -0.1, 0.1, 0.045, M.porcelain, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


def _knives(P, name, x, y, z, width=0.56, depth=0.3):
    """A knife insert standing at height `y`, its front at drawn depth `z`: a wooden tray with a chef's knife, a
    bread knife, a carving knife and a paring knife lying in it, their handles black."""
    zc = z - depth / 2
    P.box(f'{name}_floor', width, 0.006, depth, (x, y + 0.003, zc), M.spoon)
    for i in range(6):
        P.box(f'{name}_rib{i}', 0.008, 0.02, depth - 0.02, (x - width / 2 + 0.04 + i * (width - 0.08) / 5, y + 0.01, zc),
              M.spoon)
    for i, (k, dx) in enumerate(((1.45, 0.048), (1.6, 0.144), (1.35, 0.24), (0.95, 0.336))):
        cx = x - width / 2 + dx
        _flat(P, f'{name}_blade{i}', _cutlery_outline('knife', k), cx, y + 0.006, zc - 0.005, M.steel)
        P.box(f'{name}_grip{i}', 0.0145 * k, 0.009, 0.08 * k, (cx, y + 0.0105, zc - 0.005 + 0.046 * k), M.metal,
              bevel=0.003, bevel_segments=1)


# --- the butcher block: 1x1, a block on legs with a shelf, a basket and a knife drawer ----------------
def butcherBlock():
    F = G.GameFrame(shift=-0.0055)
    with G.new_parts() as still:
        # the block, its sides showing the staves it is glued up from, and the top layer of staves
        F.box('top', 0.9, 0.10, 0.8, (0, 0.88, 0), M.block, bevel=0.012)
        for i in range(12):
            F.box(f'top_layer{i}', 0.07, 0.035, 0.74, (-0.385 + 0.07 * i, 0.9325, 0), M.block2, bevel=0.003,
                  bevel_segments=1)
        # rounded legs, aprons round the drawer, stretchers carrying the slatted shelf
        leg = G.rounded_rect(0.07, 0.07, 0.012, corner=2)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.extrude(f'leg{sx}{sz}', leg, 0.83, (sx * 0.41, 0.415, sz * 0.36), M.leg, plane='top', bevel=0.004,
                          bevel_segments=1)
            F.box(f'apron_side{sx}', 0.022, 0.18, 0.65, (sx * 0.395, 0.74, 0), M.leg, bevel=0.003, bevel_segments=1)
            F.box(f'stretcher{sx}', 0.03, 0.04, 0.65, (sx * 0.41, 0.17, 0), M.leg, bevel=0.004, bevel_segments=1)
            F.box(f'apron_front{sx}', 0.03, 0.18, 0.022, (sx * 0.362, 0.74, 0.374), M.leg, bevel=0.003, bevel_segments=1)
        F.box('apron_back', 0.75, 0.18, 0.022, (0, 0.74, -0.374), M.leg, bevel=0.003, bevel_segments=1)
        for i, z in enumerate((-0.24, -0.08, 0.08, 0.24)):
            F.box(f'slat{i}', 0.8, 0.025, 0.12, (0, 0.2, z), M.leg, bevel=0.004)
        # a woven basket and a stoneware crock on the shelf
        _basket(F, 'basket', 0.4, 0.2, 0.3, -0.16, 0.2125, 0)
        _canister(F, 'box', 0.22, 0.2125, 0.06, 0.085, 0.17, M.porcelain, M.spoon, verts=16)
    G.join(still, 'body', origin=(0, 0, 0))
    # the knife drawer under the block, sliding out for a sandwich: its front in the apron, a long dark pull
    zf = 0.40
    with G.new_parts() as parts:
        F.box('drawer_front', 0.7, 0.17, 0.02, (0, 0.72, zf - 0.01), M.wood2, bevel=0.006, bevel_segments=2)
        _pull(F, 'drawer_pull', 0, 0.72, zf, 0.56, M.metal)
        F.box('drawer_bottom', 0.64, 0.012, 0.5, (0, 0.646, zf - 0.27), M.wood2)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.014, 0.12, 0.5, (s * 0.313, 0.7, zf - 0.27), M.wood2, bevel=0.003, bevel_segments=1)
        F.box('drawer_back', 0.64, 0.12, 0.014, (0, 0.7, zf - 0.513), M.wood2, bevel=0.003, bevel_segments=1)
        _knives(F, 'drawer_knives', 0, 0.652, zf - 0.04)
    drawer = G.join(parts, 'drawer', origin=F.at(0, 0.72, zf - 0.01))
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the marble counter: 1x1, a navy unit with a marble top and brass handles, a drawer for sandwich --
def counterMarble():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    P, front = _plain(F), F.depth(0.24)
    with G.new_parts() as still:
        _unit79(F, w=1.0, body=M.navy, top=M.marble)
        _doors(P, w=0.97, front=front, y0=0.12, y1=0.72, body=M.navy, hm=M.brass, hy=0.62)
        # veins wandering through the marble, thin and fading at their ends
        vein = G.rounded_rect(0.0016, 0.012, 0.0007, corner=1)
        for i, pts in enumerate((((-0.42, -0.36), (-0.25, -0.2), (-0.12, -0.12), (0.05, 0.02), (0.2, 0.16), (0.3, 0.27)),
                                 ((0.05, -0.4), (0.12, -0.22), (0.2, -0.12), (0.34, 0.0), (0.46, 0.1)),
                                 ((-0.46, 0.05), (-0.33, 0.12), (-0.22, 0.2), (-0.12, 0.26)))):
            F.sweep(f'vein{i}', [(x, 0.9505, z) for x, z in pts], None, M.vein, profile=vein, resolution=3, smooth=False,
                    scale=[0.3] + [1.0 - 0.25 * (k % 2) for k in range(len(pts) - 2)] + [0.3])
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(P, 'drawer', w=0.97, y=0.815, zf=front + 0.02, material=M.navy, hm=M.brass)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# === K5: tall and wall storage =========================================================================
# Tall larders (`pantry`, `pantryTall`), wall cabinets (`wallCabinet`, `wallCabinetGlass`,
# `wallCabinetCorner`), a cooker hood (`rangeHood`), a spice rack, a pot rail, a wall shelf
# (`kitchenShelfWall`), a pedal bin (`trashCan`) and recycling bins. The two larders' doors swing open
# (`doors_open`) for a snack, the wall cabinets' while clean things are put away in them, the pedal bin's lid
# while a plate is scraped and the recycling bins' lids while the recycling is sorted; the hood, rack, rail
# and shelf are still. A wall thing sits on the wall face (z = WALL)
# at its real height: `floor=False`, centred in x and depth.

WALL = -0.43   # the back face of a wall thing, in game depth (docs/design/objects.md)


def _case(F, w, h, d, x, y, zback, material, back=None, t=0.02, p='case'):
    """An open case: a back at depth `zback`, two sides, a top and a bottom, open at the front."""
    F.box(f'{p}_back', w, h, t, (x, y, zback + t / 2), back or material, bevel=0.004)
    for s in (-1, 1):
        F.box(f'{p}_side{s}', t, h, d, (x + s * (w / 2 - t / 2), y, zback + d / 2), material, bevel=0.004)
    F.box(f'{p}_top', w - 2 * t, t, d, (x, y + h / 2 - t / 2, zback + d / 2), material, bevel=0.004)
    F.box(f'{p}_bottom', w - 2 * t, t, d, (x, y - h / 2 + t / 2, zback + d / 2), material, bevel=0.004)


def _shelf(F, name, w, y, zback, d, material=None, lip=None):
    """A shelf across a case, from the back face `zback`; `lip`, a strip standing up along its front in that
    material (a larder's, to keep the tins on)."""
    F.box(name, w, 0.02, d, (0, y, zback + d / 2), material or M.worktop, bevel=0.003)
    if lip:
        F.box(f'{name}_lip', w, 0.035, 0.012, (0, y + 0.0075, zback + d - 0.006), lip, bevel=0.003, bevel_segments=1)


def _lathe_at(name, profile, at, material, verts=12, rot=(0, 0, 0)):
    """A round thing turned about its own axis (Blender z before `rot`), its foot on `at` (Blender): for one hung
    or lying on its side (a pan facing the front)."""
    return G.loft(name, [G.ring(max(r, 1e-5), max(r, 1e-5), h, verts) for r, h in profile], material=material, at=at,
                  rot=rot)


def _jar(F, name, x, y, z, colour, h=0.13, r=0.045, lid=True, lidmat=None, verts=10):
    """A glass jar standing on `y` at depth `z`: its body rounding into a shoulder and neck, what is in it, and a
    screw lid."""
    _lathe(F, f'{name}_glass', [(0.0, 0.0), (r, 0.0), (r, 0.78 * h), (0.8 * r, 0.9 * h), (0.0, 0.9 * h)], x, y, z, M.glass,
           verts)
    F.cyl(f'{name}_in', r * 0.86, h * 0.6, (x, y + h * 0.32, z), colour, verts=verts)
    if lid:
        _lathe(F, f'{name}_lid', [(0.0, 0.9 * h), (0.86 * r, 0.9 * h), (0.86 * r, h + 0.012), (0.0, h + 0.012)], x, y, z,
               lidmat or M.chrome, verts)


def _basket(F, name, w, h, d, x, y, z, material=None, verts=16):
    """A woven basket sitting on `y`: a rounded body narrowing to its foot, ribbed with the weave, hollow, a rolled
    rim round its top."""
    material = material or M.basket
    bx, by, bz = F.at(x, y, z)
    secs = []
    for i in range(7):
        t = i / 6
        k = 0.88 + 0.12 * t
        bump = 0.005 if i % 2 else 0.0
        secs.append(G.ring(w / 2 * k + bump, d / 2 * k + bump, bz + h * t, verts, square=5, at=(bx, by)))
    secs.append(G.ring(w / 2 - 0.012, d / 2 - 0.012, bz + h, verts, square=5, at=(bx, by)))
    secs.append(G.ring(w / 2 - 0.016, d / 2 - 0.016, bz + 0.03, verts, square=5, at=(bx, by)))
    G.loft(name, secs, material=material)
    G.sweep(f'{name}_rim', [(px, py, bz + h) for px, py, _ in G.ring(w / 2 + 0.002, d / 2 + 0.002, 0, verts, square=5,
                                                                       at=(bx, by))],
            0.008, material=material, verts=6, closed=True)


def _cornice(F, w, d, y, z, material, t=0.04):
    """A cornice capping a tall unit, `t` thick from `y` up: a fillet under a moulded cap with a rounded nosing."""
    F.box('cornice_fillet', w - 0.016, t * 0.35, d - 0.008, (0, y + t * 0.175, z - 0.004), material, bevel=0.003,
          bevel_segments=1)
    F.box('cornice', w, t * 0.65, d, (0, y + t * 0.675, z), material, bevel=t * 0.25, bevel_segments=3)


def _box_food(F, name, w, h, d, x, y, z, material, label=None):
    """A packet of food standing on `y`: a box with a label band round it."""
    F.box(name, w, h, d, (x, y + h / 2, z), material, bevel=0.004, bevel_segments=1)
    if label:
        F.box(f'{name}_label', w + 0.002, h * 0.3, d + 0.002, (x, y + h * 0.55, z), label)


def _larder_stock(F, tops, z):
    """Food on a larder's shelves (their tops `tops`, bottom up), at depth `z`: tins, canisters, packets, bowls."""
    a, b, c, d = tops
    for i, (x, m) in enumerate(((-0.31, M.rust), (-0.21, M.cream), (-0.11, M.teal))):
        _can(F, f'can{i}', x, a, z, 0.042, 0.11, m, verts=8)
    _can(F, 'can3', -0.21, a + 0.11, z, 0.042, 0.11, M.rust, verts=8)
    _box_food(F, 'cereal0', 0.17, 0.27, 0.07, 0.1, a, z - 0.04, M.yellow, M.porcelain)
    _box_food(F, 'cereal1', 0.15, 0.24, 0.065, 0.28, a, z - 0.03, M.sage, M.cream)
    for i, (x, r, h) in enumerate(((-0.29, 0.06, 0.2), (-0.15, 0.055, 0.16), (-0.03, 0.05, 0.12))):
        _canister(F, f'canister{i}', x, b, z, r, h, M.porcelain, M.spoon, verts=12)
    _box_food(F, 'packet0', 0.12, 0.18, 0.08, 0.15, b, z, M.rust, M.porcelain)
    _box_food(F, 'packet1', 0.12, 0.15, 0.08, 0.29, b, z + 0.02, M.teal, M.porcelain)
    for i in range(3):
        _bowl(F, f'bowl{i}', -0.24, c + 0.02 * i, z, 0.09 - 0.008 * i, 0.075 - 0.006 * i, M.cream, verts=10)
    _lathe(F, 'casserole', [(0.0, 0.0), (0.09, 0.0), (0.095, 0.01), (0.095, 0.09), (0.0, 0.09)], 0.05, c, z, M.rust)
    _lathe(F, 'casserole_lid', [(0.0, 0.088), (0.098, 0.088), (0.09, 0.1), (0.04, 0.115), (0.012, 0.117), (0.016, 0.13),
                                (0.0, 0.132)], 0.05, c, z, M.rust)
    _can(F, 'can4', 0.27, c, z, 0.045, 0.12, M.yellow, verts=8)
    _box_food(F, 'packet2', 0.2, 0.18, 0.07, -0.2, d, z - 0.03, M.cream, M.rust)
    _box_food(F, 'packet3', 0.16, 0.16, 0.07, 0.02, d, z - 0.02, M.porcelain, M.teal)
    _plates(F, 'platter', 0.24, d, z, 0.11, 3, M.porcelain, verts=12)


def _tall_doors(F, w, h, y, front, mid, pull_y, pull_len, hm):
    """The two shaker doors of a larder, `w` wide each, on the case front at depth `front`, a middle rail at `mid`, a
    long pull on each meeting stile; each its own part hinged on its outer edge."""
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            _shaker(F, f'door{s}_front', w - 0.002, h, s * (w / 2 + 0.001), y, front - 0.01, M.cab, mid=mid)
            _pull(F, f'door{s}_pull', s * 0.034, pull_y, front + 0.01, pull_len, hm, vertical=True)
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * w, y, front)))
    return doors


# --- the larder: 1x1, a tall cupboard with two doors that swing open for a snack -----------------------
def pantry():
    F = G.GameFrame(shift=0.141)
    with G.new_parts() as still:
        _case(F, 0.92, 1.92, 0.6, 0, 1.06, -0.46, M.cab)
        F.box('plinth', 0.88, 0.1, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.004)
        _cornice(F, 0.94, 0.62, 2.02, -0.15, M.cab)
        for i, y in enumerate((0.5, 0.88, 1.26, 1.64)):
            _shelf(F, f'shelf{i}', 0.86, y, -0.44, 0.56, lip=M.cab)
        _larder_stock(F, (0.51, 0.89, 1.27, 1.65), -0.17)
        _basket(F, 'floor_basket', 0.34, 0.16, 0.3, -0.18, 0.12, -0.18)
        _basket(F, 'basket', 0.36, 0.2, 0.3, -0.2, 2.06, -0.2)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = _tall_doors(F, 0.456, 1.88, 1.06, 0.14, 1.0, 1.05, 0.3, M.chrome)
    G.settle()
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


# --- the tall larder: 1x1, two doors over two drawers, a basket on top ---------------------------------
def pantryTall():
    F = G.GameFrame(shift=0.104)
    with G.new_parts() as still:
        _case(F, 0.92, 2.06, 0.66, 0, 1.08, -0.45, M.cab)
        F.box('plinth', 0.9, 0.1, 0.6, (0, 0.05, -0.14), M.dark, bevel=0.004)
        _cornice(F, 0.96, 0.7, 2.11, -0.11, M.cab, t=0.05)
        for i, y in enumerate((0.24, 0.48)):
            _shaker(F, f'drawer{i}', 0.9, 0.22, 0, y, 0.2, M.cab, rail=0.045)
            _pull(F, f'drawer{i}_pull', 0, y, 0.22, 0.3, M.brass, base=0.008)
        for i, y in enumerate((0.74, 1.12, 1.5, 1.88)):
            _shelf(F, f'shelf{i}', 0.86, y, -0.43, 0.62, lip=M.cab)
        _larder_stock(F, (0.75, 1.13, 1.51, 1.89), -0.14)
        _basket(F, 'basket', 0.3, 0.22, 0.26, -0.22, 2.16, -0.16)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = _tall_doors(F, 0.453, 1.44, 1.34, 0.21, 1.36, 0.8, 0.3, M.brass)
    G.settle()
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


def _strip_light(F, w, y, z):
    """A strip light under a wall cabinet, its top at `y`: a slim housing and its diffuser."""
    F.box('light_housing', w, 0.008, 0.036, (0, y - 0.004, z), M.cab, bevel=0.002, bevel_segments=1)
    F.box('light', w - 0.02, 0.006, 0.024, (0, y - 0.009, z), M.bulb, bevel=0.002, bevel_segments=1)


# --- the wall cabinet: 1x1, two doors with pulls under it and a strip light; plates, bowls and mugs
# inside, the doors swinging open while a resident puts the clean things away (`doors_open`)
def wallCabinet():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.98, 0.72, 0.34, 0, 1.81, WALL, M.cab)
        _shelf(F, 'shelf', 0.94, 1.81, WALL + 0.02, 0.3, M.cab)
        _plates(F, 'plates', -0.24, 1.47, WALL + 0.18, 0.1, 6, M.porcelain, h=0.015, verts=12)
        for i in range(3):
            _bowl(F, f'bowl{i}', 0.2, 1.47 + 0.02 * i, WALL + 0.18, 0.075 - 0.007 * i, 0.06 - 0.005 * i, M.porcelain, verts=10)
        for i, x in enumerate((-0.3, -0.16, 0.12, 0.26)):
            _mug(F, f'mug{i}', x, 1.82, WALL + 0.17, 0.038, 0.09, M.porcelain, turn=-60 + 40 * i, verts=10)
        _strip_light(F, 0.9, 1.45, WALL + 0.07)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            _shaker(F, f'door{s}', 0.482, 0.7, s * 0.244, 1.81, WALL + 0.34, M.cab)
            _pull(F, f'door{s}_pull', s * 0.034, 1.56, WALL + 0.36, 0.14, vertical=True)   # on the doors, near their bottom
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.487, 1.81, WALL + 0.34)))
    G.settle(centre=False, floor=False)
    G.swing(doors[0], 'doors_open', 100, seconds=WALL_DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=WALL_DOOR_SECONDS)


def _tumbler(F, name, x, y, z, r, h, verts=10):
    """A drinking glass standing on `y`: a thick base, thin walls flaring a little to the rim, hollow."""
    _lathe(F, name, [(0.0, 0.0), (0.88 * r, 0.0), (0.9 * r, 0.12 * h), (r, h), (0.92 * r, h), (0.8 * r, 0.14 * h),
                     (0.0, 0.14 * h)], x, y, z, M.glass, verts)


def _wine_glass(F, name, x, y, z, r, h, verts=10):
    """A wine glass standing on `y`: a foot, a stem and a bowl."""
    _lathe(F, name, [(0.0, 0.0), (0.75 * r, 0.0), (0.7 * r, 0.03 * h), (0.1 * r, 0.06 * h), (0.09 * r, 0.42 * h),
                     (0.6 * r, 0.52 * h), (r, 0.72 * h), (0.86 * r, h), (0.8 * r, h), (0.9 * r, 0.72 * h), (0.0, 0.5 * h)],
           x, y, z, M.glass, verts)


# --- the glazed wall cabinet: 1x1, glass doors showing glasses and plates; the right one opens while
# clean things are put away (`door_open`) ---------------------------------------------------------------
def wallCabinetGlass():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.96, 0.72, 0.36, 0, 1.81, WALL, M.cab, back=M.back)
        for i, y in enumerate((1.47, 1.84)):
            _shelf(F, f'shelf{i}', 0.9, y, WALL, 0.32)
        for i, x in enumerate((-0.37, -0.28)):
            _tumbler(F, f'glass{i}', x, 1.48, WALL + 0.12 + 0.06 * i, 0.033, 0.11)
        for i, x in enumerate((-0.17, -0.06)):
            _wine_glass(F, f'wine{i}', x, 1.48, WALL + 0.15, 0.037, 0.17)
        _jug(F, 'carafe', 0.12, 1.48, WALL + 0.16, 0.05, 0.2, M.glass, turn=-90, verts=12)
        _plates(F, 'plates', 0.24, 1.85, WALL + 0.17, 0.1, 5, M.porcelain, h=0.016)
        for i, x in enumerate((-0.3, -0.12)):
            _bowl(F, f'bowl{i}', x, 1.85, WALL + 0.17, 0.07, 0.06, M.porcelain)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        cx = s * 0.24
        with G.new_parts() as parts:
            # a frame round the glass, its inner edge eased, a glazing bar across the middle
            F.extrude(f'door{s}_frame', G.rounded_rect(0.46, 0.7, 0.004, corner=1), 0.02, (cx, 1.81, WALL + 0.37), M.cab,
                      holes=[G.rounded_rect(0.37, 0.61, 0.003, corner=1)], bevel=0.003, bevel_segments=1)
            F.box(f'door{s}_bar', 0.37, 0.022, 0.014, (cx, 1.81, WALL + 0.373), M.cab, bevel=0.003, bevel_segments=1)
            F.box(f'door{s}_glass', 0.38, 0.62, 0.006, (cx, 1.81, WALL + 0.365), M.glass)
            _pull(F, f'door{s}_pull', s * 0.04, 1.6, WALL + 0.38, 0.14, vertical=True, reach=0.026)
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.48, 1.81, WALL + 0.36)))
    G.settle(centre=False, floor=False)
    # only the right door, in front of the plates, swings open: each glazed door is two draws (frame, glass)
    G.swing(doors[1], 'door_open', -100, seconds=WALL_DOOR_SECONDS)


# --- the corner wall cabinet: 1x1, a door on one side and open shelves on the other -------------------
def wallCabinetCorner():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.6, 0.72, 0.36, -0.18, 1.81, WALL, M.cab)
        F.box('shelf', 0.56, 0.02, 0.32, (-0.18, 1.81, WALL + 0.18), M.cab, bevel=0.003)
        _plates(F, 'plates', -0.3, 1.47, WALL + 0.19, 0.1, 5, M.porcelain, h=0.015)
        for i, x in enumerate((-0.36, -0.22, -0.08)):
            _mug(F, f'mug{i}', x, 1.82, WALL + 0.18, 0.038, 0.09, M.porcelain, turn=-40 + 50 * i)
        # the open end: three shelves rounded at their outer front corner, between the case and a side panel
        shelf = [(0.12, -0.17), (0.48, -0.17), (0.48, 0.0), (0.29, 0.17), (0.12, 0.17)]
        shelf = shelf[:2] + G.smooth_outline([(0.48, -0.03), (0.48, 0.1), (0.36, 0.17), (0.22, 0.17)], passes=2,
                                             closed=False)[1:-1] + shelf[3:]
        for i, y in enumerate((1.46, 1.81, 2.155)):
            F.extrude(f'end_shelf{i}', shelf, 0.025, (0, y, WALL + 0.17), M.oak, plane='top',
                      bevel=0.004, bevel_segments=1)
        F.box('end_panel', 0.03, 0.72, 0.34, (0.46, 1.81, WALL + 0.17), M.oak, bevel=0.006)
        _lathe(F, 'pot', [(0.0, 0.0), (0.042, 0.0), (0.05, 0.11), (0.054, 0.12), (0.054, 0.13), (0.046, 0.13),
                          (0.043, 0.115), (0.0, 0.115)], 0.26, 1.4725, WALL + 0.16, M.sage)
        for i, (dx, dz, a) in enumerate(((0.0, 0.0, 0), (0.02, 0.03, 70), (-0.025, 0.02, 150), (0.01, -0.03, 230),
                                         (-0.015, -0.015, 300))):
            F.sphere(f'leaf{i}', 0.04, (0.26 + dx, 1.62 + 0.012 * (i % 2), WALL + 0.16 + dz), M.leaf,
                     scale=(1.0, 0.45, 0.35), rot=(0, 25, a), segments=8, rings=4)
        _bowl(F, 'bowl', 0.3, 1.8225, WALL + 0.14, 0.07, 0.05, M.porcelain)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as parts:
        _shaker(F, 'door', 0.58, 0.7, -0.18, 1.81, WALL + 0.36, M.cab)
        _pull(F, 'door_pull', 0.06, 1.6, WALL + 0.38, 0.14, vertical=True, reach=0.026)
    door = G.join(parts, 'door', origin=F.at(-0.47, 1.81, WALL + 0.36))
    G.settle(centre=False, floor=False)
    G.swing(door, 'door_open', 100, seconds=WALL_DOOR_SECONDS)


def _rect_ring(F, x0, x1, z0, z1, y):
    """A rectangle in plan at height `y`, game corners (x0, z0) to (x1, z1), as Blender points (for a loft)."""
    return [F.at(x, y, z) for x, z in ((x0, z0), (x1, z0), (x1, z1), (x0, z1))]


# --- the cooker hood: 1x1, a canopy and a chimney with a lamp and a filter ------------------------------
def rangeHood():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        # a two-piece chimney, the upper one a little narrower, a seam between
        F.box('flue', 0.3, 0.29, 0.3, (0, 1.815, -0.28), M.steel, bevel=0.006)
        F.box('flue_upper', 0.29, 0.29, 0.29, (0, 2.105, -0.285), M.steel, bevel=0.006)
        # the canopy: an upright band, then sloping up to the chimney, its back on the wall
        F.box('canopy', 0.9, 0.05, 0.5, (0, 1.575, -0.18), M.steel, bevel=0.006, bevel_segments=2)
        G.loft('canopy_slope', [_rect_ring(F, -0.45, 0.45, -0.43, 0.07, 1.599), _rect_ring(F, -0.45, 0.45, -0.43, 0.07, 1.605),
                                _rect_ring(F, -0.155, 0.155, -0.43, -0.125, 1.675)], material=M.steel, angle=20)
        # the trim along its front, its buttons and a little display
        F.box('trim', 0.9, 0.025, 0.02, (0, 1.58, 0.075), M.trim, bevel=0.004)
        for i in range(4):
            F.cyl(f'key{i}', 0.0065, 0.004, (0.2 + 0.04 * i, 1.58, 0.083), M.steel, rot=(90, 0, 0), verts=10)
        F.box('display', 0.05, 0.012, 0.004, (0.13, 1.58, 0.083), M.bulb)
        # under it: the grease filter with its slots, two round lamps
        slots = [[(u + (i - 4.5) * 0.08, v) for u, v in G.rounded_rect(0.05, 0.36, 0.008, corner=1)] for i in range(10)]
        F.extrude('filter', G.rounded_rect(0.84, 0.44, 0.01, corner=1), 0.01, (0, 1.551, -0.18), M.dark, holes=slots,
                  plane='top')
        F.box('filter_back', 0.82, 0.004, 0.42, (0, 1.556, -0.18), M.trim)
        for s in (-1, 1):
            F.cyl(f'lamp{s}', 0.028, 0.008, (s * 0.3, 1.542, -0.0), M.bulb, verts=12)
            F.cyl(f'lamp_ring{s}', 0.034, 0.004, (s * 0.3, 1.548, -0.0), M.steel, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the spice rack: 1x1 wall, three rails of little jars ---------------------------------------------
def spiceRack():
    F = G.GameFrame(shift=0.0)
    colours = (M.rust, M.teal, M.sage, M.cream, M.yellow, M.porcelain)
    with G.new_parts() as still:
        # the board: an arched crest with a hole to hang it by
        crest = [(0.26, -0.22), (0.26, 0.17)] + [(0.26 * math.cos(math.radians(a)), 0.17 + 0.05 * math.sin(math.radians(a)))
                                                 for a in range(15, 180, 15)] + [(-0.26, 0.17), (-0.26, -0.22)]
        hole = [(0.011 * math.cos(math.tau * i / 10), 0.19 + 0.011 * math.sin(math.tau * i / 10)) for i in range(10)]
        F.extrude('board', crest, 0.02, (0, 1.52, WALL + 0.01), M.oak, holes=[hole], bevel=0.003, bevel_segments=1)
        for i, y in enumerate((1.33, 1.47, 1.61)):
            F.box(f'shelf{i}', 0.5, 0.015, 0.09, (0, y, WALL + 0.055), M.oak, bevel=0.003)
            # a brass rail on two posts along the shelf's front, in one piece
            G.kit.tube(F, f'rail{i}', [(-0.244, y, WALL + 0.1), (-0.244, y + 0.05, WALL + 0.1), (-0.236, y + 0.058, WALL + 0.1),
                                       (0.236, y + 0.058, WALL + 0.1), (0.244, y + 0.05, WALL + 0.1), (0.244, y, WALL + 0.1)],
                       0.005, M.brass, verts=6, round=1)
            for j in range(6):
                _jar(F, f'jar{i}{j}', -0.2 + j * 0.08, y + 0.0075, WALL + 0.055,
                     colours[(i + j) % 6], h=0.075, r=0.024, lidmat=M.dark, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


def _s_hook(F, name, x, y, z):
    """An S-hook over a rail whose middle is at (x, y, z): over the rail, down its front, curling up at the bottom."""
    pts = [(-0.019, -0.01), (-0.021, 0.012), (0.0, 0.024), (0.021, 0.012), (0.018, -0.02), (0.006, -0.045),
           (-0.006, -0.062), (0.002, -0.076), (0.016, -0.071)]
    F.sweep(name, [(x, y + dy, z + dz) for dz, dy in pts], 0.0035, M.metal, verts=6, resolution=2)


# --- the pot rail: 1x1 wall, a rail of hooks with pans, a pot and ladles -------------------------------
def potRail():
    F = G.GameFrame(shift=0.0)
    z, y = WALL + 0.08, 1.6
    with G.new_parts() as still:
        # the rail with ball ends, on two arms from round wall plates
        G.kit.tube(F, 'rail', [(-0.435, y, z), (0.435, y, z)], 0.014, M.metal, verts=12, round=1)
        for s in (-1, 1):
            F.sphere(f'finial{s}', 0.02, (s * 0.435, y, z), M.metal, segments=10, rings=6)
            F.cyl(f'plate{s}', 0.024, 0.01, (s * 0.38, y, WALL + 0.005), M.metal, rot=(90, 0, 0), verts=12)
            F.sweep(f'arm{s}', [(s * 0.38, y, WALL + 0.005), (s * 0.38, y - 0.004, z - 0.03), (s * 0.38, y, z)], 0.008,
                    M.metal, verts=8, resolution=2)
        for x, kind in ((-0.32, 'pan'), (-0.12, 'ladle'), (0.06, 'pot'), (0.28, 'pan'), (0.4, 'ladle')):
            _s_hook(F, f'hook{x}', x, y, z)
            low = y - 0.075   # where the hook's curl holds what hangs from it
            if kind == 'pan':
                # a frying pan facing the front, its handle up to the hook
                _lathe_at(f'pan{x}', [(0.0, 0.0), (0.1, 0.0), (0.118, 0.008), (0.13, 0.04), (0.124, 0.04), (0.11, 0.012),
                                      (0.0, 0.006)], F.at(x, 1.21, WALL + 0.04), M.copper, verts=18, rot=(90, 0, 0))
                F.sweep(f'handle{x}', [(x, 1.33, WALL + 0.06), (x, 1.42, WALL + 0.07), (x, low - 0.01, z)], None, M.metal,
                        profile=G.rounded_rect(0.022, 0.008, 0.003, corner=1), up=(1, 0, 0), resolution=2)
                G.torus(f'eye{x}', 0.009, 0.0035, at=F.at(x, low - 0.002, z), material=M.metal, rot=(0, 90, 0),
                        major_segments=8, minor_segments=4)
            elif kind == 'pot':
                # a saucepan hung by its bail handle, its lid on
                _lathe(F, f'pot{x}', [(0.0, 0.0), (0.086, 0.0), (0.096, 0.012), (0.096, 0.14), (0.1, 0.146), (0.0, 0.146)],
                       x, 1.29, WALL + 0.12, M.copper, verts=16)
                _lathe(F, f'pot_lid{x}', [(0.0, 0.144), (0.097, 0.144), (0.06, 0.158), (0.01, 0.16), (0.014, 0.17),
                                          (0.0, 0.172)], x, 1.29, WALL + 0.12, M.copper, verts=16)
                F.sweep(f'bail{x}', [(x - 0.1, 1.4, WALL + 0.12), (x - 0.092, 1.47, WALL + 0.11), (x - 0.03, low - 0.01, z),
                                     (x + 0.03, low - 0.01, z), (x + 0.092, 1.47, WALL + 0.11), (x + 0.1, 1.4, WALL + 0.12)],
                        0.004, M.metal, verts=6, resolution=2)
            else:
                # a ladle: its bowl, its handle up to a curl round the hook
                _lathe(F, f'bowl{x}', [(0.0, 0.0), (0.03, 0.004), (0.046, 0.025), (0.05, 0.05), (0.045, 0.05), (0.03, 0.012),
                                       (0.0, 0.009)], x, 1.236, WALL + 0.09, M.steel, verts=12)
                F.sweep(f'handle{x}', [(x, 1.286, WALL + 0.045), (x, 1.33, WALL + 0.06), (x, 1.45, WALL + 0.075),
                                       (x, low - 0.012, z - 0.004), (x, low + 0.004, z + 0.008)], None, M.steel,
                        profile=G.rounded_rect(0.014, 0.005, 0.002, corner=1), up=(1, 0, 0), resolution=2)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


def _corbel(F, name, x, y, z, h, d, material):
    """A shelf bracket in profile: its back on the wall at `z`, its top under the shelf at `y`, `d` deep and `h` tall,
    the brace curving from the wall out under the shelf."""
    k = math.copysign(0.025, d)   # a negative `d` reaches back (under an island's overhang)
    outline = [(z, y), (z + d, y)] + \
              [(z + d - (d - k) * math.sin(math.radians(a)), y - 0.025 - (h - 0.045) * (1 - math.cos(math.radians(a))))
               for a in range(0, 91, 15)] + [(z + k, y - h + 0.01), (z, y - h)]
    F.extrude(name, outline, 0.03, (x, 0, 0), material, plane='side', bevel=0.003, bevel_segments=1)


# --- the wall shelf: 1x1 wall, two shelves of jars, plates, pots and a plant ---------------------------
def kitchenShelfWall():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        for i, y in enumerate((1.4, 1.75)):
            F.box(f'shelf{i}', 0.9, 0.03, 0.24, (0, y, WALL + 0.12), M.oak, bevel=0.006, bevel_segments=2)
        for s in (-1, 1):
            _corbel(F, f'bracket{s}', s * 0.36, 1.385, WALL, 0.095, 0.2, M.oakd)
            _corbel(F, f'bracket_top{s}', s * 0.36, 1.735, WALL, 0.08, 0.16, M.oakd)
        _jar(F, 'jar0', -0.32, 1.415, WALL + 0.12, M.yellow, h=0.13, r=0.045)
        _jar(F, 'jar1', -0.2, 1.415, WALL + 0.12, M.porcelain, h=0.16, r=0.045)
        _jar(F, 'jar2', -0.08, 1.415, WALL + 0.12, M.rust, h=0.13, r=0.045)
        _plates(F, 'plates', 0.22, 1.415, WALL + 0.12, 0.11, 4, M.porcelain, h=0.014)
        _canister(F, 'pot0', -0.25, 1.765, WALL + 0.12, 0.05, 0.11, M.teal, M.spoon)
        _mug(F, 'pot1', -0.12, 1.765, WALL + 0.12, 0.04, 0.09, M.cream, turn=-30)
        # a trailing plant in a pot: a crown of leaves, two vines trailing over the shelf's front
        _lathe(F, 'plant_pot', [(0.0, 0.0), (0.045, 0.0), (0.058, 0.085), (0.062, 0.088), (0.062, 0.1), (0.054, 0.1),
                                (0.052, 0.09), (0.0, 0.09)], 0.25, 1.765, WALL + 0.12, M.porcelain, verts=14)
        F.cyl('soil', 0.053, 0.006, (0.25, 1.858, WALL + 0.12), M.soil, verts=12)
        for i, (dx, dy, dz, a, t) in enumerate(((0.0, 0.19, 0.0, 0, 10), (0.05, 0.13, 0.03, 60, 35), (-0.05, 0.14, 0.01, 130, 35),
                                                (0.02, 0.12, -0.05, 200, 40), (-0.03, 0.1, 0.05, 280, 45),
                                                (0.07, 0.07, -0.02, 20, 60), (-0.07, 0.075, -0.03, 160, 55))):
            F.sphere(f'leaf{i}', 0.045, (0.25 + dx, 1.85 + dy, WALL + 0.12 + dz), M.leaf, scale=(1.0, 0.5, 0.18),
                     rot=(t, 0, a), segments=8, rings=4)
            # its stem, up from the soil
            F.sweep(f'stem{i}', [(0.25 + dx * 0.15, 1.858, WALL + 0.12 + dz * 0.15), (0.25 + dx * 0.6, 1.85 + dy * 0.6, WALL + 0.12 + dz * 0.6),
                                 (0.25 + dx * 0.9, 1.85 + dy - 0.012, WALL + 0.12 + dz * 0.9)], 0.0025, M.leaf, verts=4,
                    resolution=2)
        for k, (x1, z1) in enumerate(((0.33, 0.212), (0.18, 0.215))):
            vine = [(0.25 + (x1 - 0.25) * 0.4, 1.86, WALL + 0.15), (x1, 1.79, WALL + z1 - 0.01), (x1 + 0.01 * (1 - 2 * k), 1.69, WALL + z1 + 0.005),
                    (x1 + 0.02 * (1 - 2 * k), 1.6, WALL + z1)]
            F.sweep(f'vine{k}', vine, 0.0025, M.leaf, verts=4, resolution=2)
            for j, (px, py, pz) in enumerate(vine[1:]):
                F.sphere(f'vine_leaf{k}{j}', 0.028, (px + 0.012 * (1 - 2 * (j % 2)), py - 0.015, pz), M.leaf,
                         scale=(1.0, 0.6, 0.18), rot=(70, 0, 30 * (1 - 2 * (j % 2))), segments=6, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the pedal bin: 1x1, a stainless drum with a domed lid and a pedal (static) -----------------------
def trashCan():
    F = G.GameFrame(shift=0.172)
    with G.new_parts() as still:
        # the drum, a little wider at the top, a rolled bead round its rim; a dark plastic foot ring
        _lathe(F, 'drum', [(0.0, 0.02), (0.144, 0.02), (0.148, 0.59), (0.152, 0.6), (0.152, 0.612), (0.146, 0.618),
                           (0.14, 0.618), (0.14, 0.45), (0.0, 0.45)], 0, 0, -0.2, M.steel, verts=24)
        _lathe(F, 'base', [(0.0, 0.0), (0.138, 0.0), (0.15, 0.006), (0.152, 0.03), (0.145, 0.034), (0.0, 0.034)], 0, 0, -0.2,
               M.dark, verts=24)
        # the inner bucket's rim, seen when the lid lifts
        _lathe(F, 'bucket', [(0.0, 0.46), (0.128, 0.46), (0.132, 0.6), (0.142, 0.614), (0.142, 0.62), (0.134, 0.62),
                             (0.124, 0.47), (0.0, 0.47)], 0, 0, -0.2, M.lid, verts=24)
        # the pedal, rounded, on its hinge block; the lid's hinge at the back
        pedal = G.smooth_outline([(-0.05, -0.07), (0.05, -0.07), (0.045, 0.01), (-0.045, 0.01)], passes=2)
        F.extrude('pedal', pedal, 0.016, (0, 0.035, 0), M.dark, plane='top', bevel=0.004,
                  bevel_segments=1)
        F.box('pedal_hinge', 0.08, 0.03, 0.03, (0, 0.03, -0.07), M.dark, bevel=0.006, bevel_segments=1)
        F.box('hinge', 0.08, 0.03, 0.04, (0, 0.625, -0.335), M.dark, bevel=0.008, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # the lid, hinged at its back edge, lifting while a resident scrapes a plate into the bin
    with G.new_parts() as parts:
        _lathe(F, 'lid', [(0.0, 0.617), (0.155, 0.617), (0.155, 0.632), (0.15, 0.638), (0.14, 0.652), (0.11, 0.669),
                          (0.06, 0.681), (0.0, 0.685)], 0, 0, -0.2, M.lid, verts=24)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.62, -0.355))
    G.settle()
    G.swing(lid, 'lid_open', -75, seconds=0.5, axis='X')


# --- the recycling bins: 1x1, three coloured bins with lids, each lid hinged at its back; they lift one
# after another while a resident sorts the recycling (`lid0_open` .. `lid2_open`)
BINS = ((-0.31, 'bin1'), (0.0, 'bin2'), (0.31, 'bin3'))


def recyclingBins():
    F = G.GameFrame(shift=0.212)
    with G.new_parts() as still:
        for i, (x, c) in enumerate(BINS):
            bx, by, _ = F.at(x, 0, -0.22)
            m = getattr(M, c)
            # the bin: a rounded tub narrowing to its foot, a rolled rim, hollow inside
            secs = [G.ring(0.132, 0.198, 0.0, 16, square=6, at=(bx, by)), G.ring(0.136, 0.202, 0.015, 16, square=6, at=(bx, by)),
                    G.ring(0.143, 0.208, 0.58, 16, square=6, at=(bx, by)), G.ring(0.145, 0.21, 0.6, 16, square=6, at=(bx, by)),
                    G.ring(0.145, 0.21, 0.62, 16, square=6, at=(bx, by)), G.ring(0.132, 0.197, 0.62, 16, square=6, at=(bx, by)),
                    G.ring(0.128, 0.193, 0.08, 16, square=6, at=(bx, by))]
            G.loft(f'body{i}', secs, material=m)
            # a dark foot band, and the recycling arrows on its front
            G.loft(f'foot{i}', [G.ring(0.134, 0.2, 0.0, 16, square=6, at=(bx, by)), G.ring(0.136, 0.202, 0.03, 16, square=6, at=(bx, by)),
                                G.ring(0.0, 0.0, 0.03, 16, at=(bx, by))], material=M.lid)
            tri = [(0.05 * math.cos(math.radians(90 + 120 * k)), 0.05 * math.sin(math.radians(90 + 120 * k))) for k in range(3)]
            inner = [(u * 0.55, v * 0.55) for u, v in tri]
            F.extrude(f'emblem{i}', G.smooth_outline(tri, passes=1), 0.004, (x, 0.34, -0.012), M.porcelain,
                      holes=[G.smooth_outline(inner, passes=1)])
            # the hand grip under the lid's front
            G.kit.tube(F, f'handle{i}', [(x - 0.06, 0.6, -0.012), (x - 0.06, 0.6, -0.002), (x - 0.052, 0.6, 0.006),
                                         (x + 0.052, 0.6, 0.006), (x + 0.06, 0.6, -0.002), (x + 0.06, 0.6, -0.012)], 0.009,
                       M.lid, verts=8, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    lids = []
    for i, (x, c) in enumerate(BINS):
        with G.new_parts() as parts:
            m = getattr(M, c)
            F.soft(f'lid{i}', 0.3, 0.036, 0.42, (x, 0.638, -0.23), m, round=0.012, puff=(0, 0.004, 0), flat=2)
            F.box(f'lid{i}_lip', 0.16, 0.02, 0.025, (x, 0.64, -0.0125), m, bevel=0.008, bevel_segments=2)
            for s in (-1, 1):
                F.cyl(f'lid{i}_hinge{s}', 0.01, 0.06, (x + s * 0.09, 0.635, -0.43), m, rot=(0, 90, 0), verts=8)
        lids.append(G.hinged(G.join(parts, f'lid{i}'), F.at(x, 0.62, -0.44)))
    G.settle()
    for i, lid in enumerate(lids):
        G.swing(lid, f'lid{i}_open', -80, seconds=0.5, axis='X')


# --- K6: islands and bars ---------------------------------------------------------------------------
# A freestanding island: cupboards at the front (+Z), where residents stand to use it, as the kitchen island;
# the worktop overhangs the back (-Z) 30 cm for stools, its edge on the wall face.
def _island_top(F, w, d, z, material):
    """An island's worktop, `w` by `d` centred at depth `z`, 0.90 to 0.95: its corners rounded in plan, its edges
    eased all round (it stands free)."""
    F.extrude('worktop', G.rounded_rect(w, d, 0.025, corner=3), 0.05, (0, 0.925, z), material, plane='top', bevel=0.008,
              bevel_segments=2, smooth=True, angle=50)


def _island_back(F, w, z, y0, y1, n, material, corbels):
    """The back of an island's carcass (the stools' side, its face at depth `z`): `n` applied shaker panels between
    y0 and y1, and corbels at `corbels` (x) holding the overhang up."""
    pw = (w - 0.04) / n
    for i in range(n):
        x = -(w - 0.04) / 2 + pw * (i + 0.5)
        F.extrude(f'back_panel{i}', G.rounded_rect(pw - 0.03, y1 - y0, 0.004, corner=1), 0.01, (x, (y0 + y1) / 2, z - 0.005),
                  material, holes=[G.rounded_rect(pw - 0.13, y1 - y0 - 0.1, 0.003, corner=1)], bevel=0.003,
                  bevel_segments=1)
        # the field inside, its corners clear of the frame (the baked shading darkens a face by its corners)
        F.box(f'back_field{i}', pw - 0.14, y1 - y0 - 0.11, 0.006, (x, (y0 + y1) / 2, z - 0.003), material, bevel=0.002,
              bevel_segments=1)
    for i, x in enumerate(corbels):
        _corbel(F, f'corbel{i}', x, 0.9, z, 0.3, -0.24, material)


def _island(F, w, body=None, top=None, handles=None):
    """A freestanding island unit as catalog2's `island_unit`: worktop to 0.95, shaker cupboards at the front, applied
    panels on its ends and its back, corbels under the overhang."""
    body, top, handles = body or M.cab, top or M.worktop, handles or M.chrome
    F.box('carcass', w - 0.16, 0.82, 0.58, (0, 0.51, 0.16), body, bevel=0.012)
    F.box('plinth', w - 0.2, 0.10, 0.52, (0, 0.05, 0.16), M.dark, bevel=0.004)
    _island_top(F, w - 0.04, 0.90, 0.02, top)
    n = max(1, round((w - 0.16) / 0.46))
    dw = (w - 0.18) / n
    for i in range(n):
        x = -(w - 0.18) / 2 + dw * (i + 0.5)
        _shaker(F, f'door{i}', dw - 0.006, 0.72, x, 0.5, 0.445, body)
        _pull(F, f'door{i}_pull', x, 0.83, 0.465, 0.14, handles, reach=0.016)
    ex = (w - 0.16) / 2
    for s in (-1, 1):
        F.box(f'end{s}', 0.04, 0.92, 0.6, (s * ex, 0.46, 0.15), body, bevel=0.008)
        # an applied frame on the end's outer face
        F.extrude(f'end_frame{s}', G.rounded_rect(0.52, 0.74, 0.004, corner=1), 0.01, (s * (ex + 0.025), 0.5, 0.15), body,
                  holes=[G.rounded_rect(0.42, 0.64, 0.003, corner=1)], plane='side', bevel=0.003, bevel_segments=1)
        F.box(f'end_field{s}', 0.006, 0.63, 0.41, (s * (ex + 0.023), 0.5, 0.15), body, bevel=0.002, bevel_segments=1)
    corbels = [s * (ex - 0.06) for s in (-1, 1)] + ([0.0] if w > 2 else [])
    _island_back(F, w - 0.16, -0.13, 0.14, 0.86, max(1, round((w - 0.16) / 0.7)), body, corbels)


# --- the kitchen island: 2x1, two stools at the front, a drawer for a sandwich -----------------------
def kitchenIsland():
    F = G.GameFrame(shift=-0.0495)
    with G.new_parts() as still:
        # the cupboards face the front (+Z), where residents stand to use the island; the worktop
        # overhangs the back (-Z), where the stools are set (housegen sets them at a used table's back)
        F.box('carcass', 1.84, 0.86, 0.56, (0, 0.47, 0.16), M.cab, bevel=0.014)
        F.box('plinth', 1.82, 0.08, 0.54, (0, 0.04, 0.16), M.dark, bevel=0.004)
        _island_top(F, 1.94, 0.82, 0.04, M.worktop)
        for s in (-1, 1):                                       # two doors under the drawers
            _shaker(F, f'door{s}', 0.856, 0.6, s * 0.47, 0.35, 0.43, M.cab)
            _pull(F, f'door{s}_pull', s * 0.47, 0.615, 0.45, 0.14)
            # a fixed drawer front either side of the one that opens, to match it
            _shaker(F, f'drawer{s}', 0.443, 0.17, s * 0.6775, 0.77, 0.43, M.cab, rail=0.04)
            _pull(F, f'drawer{s}_pull', s * 0.6775, 0.77, 0.45, 0.14, base=0.008)
        _island_back(F, 1.84, -0.12, 0.1, 0.84, 3, M.cab, (-0.86, 0.0, 0.86))
        # no stools of its own, and a bare top: stools and the things on its top (a fruit bowl, mugs)
        # are things of their own (housegen sets them), so nothing is doubled or stands in a stool
    G.join(still, 'body', origin=(0, 0, 0))
    # the drawer over the doors, at the front where residents stand
    drawer = _drawer(F, 'drawer', 0.9, 0.77, 0.45, M.cab, handle_w=0.3)
    G.settle()
    # sandwich slides the drawer out to the front
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)

# --- the small island: 1x1, cupboards at the front, a bare top (static) -----------------------------
def islandSmall():
    F = G.GameFrame()
    with G.new_parts() as still:
        _island(F, 1.0, handles=M.brass)       # a bare top: housegen sets things on it
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

# --- the long island: 3x1, six cupboards at the front, a bare top (static) ---------------------------
def islandLong():
    F = G.GameFrame()
    with G.new_parts() as still:
        _island(F, 3.0, handles=M.metal)       # a bare top: housegen sets things on it
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

# --- a bar: a slatted body, a top to 1.06 overhanging the front (+Z) and a foot rail (static) ---------
def _bar(F, bw, bx, end=False, body=None, top=None, rail=None):
    """A bar as catalog2's `bar`: a body faced with rounded staves, a top at 1.06 overhanging the front (+Z), its
    nose rolled and its ends square (so a run's tops meet), a brass foot rail on curved brackets."""
    body, top, rail = body or M.walnut, top or M.oak, rail or M.brass
    F.box('body', bw, 0.98, 0.5, (bx, 0.49, -0.12), body, bevel=0.01)
    prof = [(-0.48, 1.0), (0.38, 1.0)] + _arc(0.38, 1.02, 0.02, -90, 0, 2)[1:] + _arc(0.375, 1.035, 0.025, 0, 90, 3) + \
           [(-0.468, 1.06)] + _arc(-0.468, 1.048, 0.012, 90, 180, 2)[1:]
    F.extrude('top', prof, 1.0, (0, 0, 0), top, plane='side', smooth=True)
    F.box('ledge', bw, 0.04, 0.26, (bx, 0.92, -0.36), top, bevel=0.006)
    n = int(bw / 0.1)
    for i in range(n):
        # swept, so each stave has rings up its length for the baked shading to darken at its ends only
        x = -0.5 + bw / n * (i + 0.5)
        F.sweep(f'slat{i}', [(x, 0.01 + 0.92 * k / 4, 0.135) for k in range(5)], None, body,
                profile=G.rounded_rect(bw / n - 0.012, 0.022, 0.0095, corner=3), up=(1, 0, 0), angle=50)
    # a rail across the top of the staves and a kick board along their foot
    F.box('apron', bw, 0.06, 0.03, (bx, 0.9, 0.15), body, bevel=0.006, bevel_segments=2)
    F.box('kick', bw, 0.07, 0.026, (bx, 0.035, 0.148), body, bevel=0.005, bevel_segments=1)
    x1 = -0.5 + bw + (0.02 if end else 0.0)
    G.kit.tube(F, 'rail', [(-0.5, 0.2, 0.26), (x1, 0.2, 0.26)], 0.02, rail, verts=12, round=1)
    for x in (-0.38, -0.5 + bw - 0.12):
        F.sweep(f'bracket{x}', [(x, 0.12, 0.146), (x, 0.13, 0.2), (x, 0.165, 0.245), (x, 0.18, 0.26)], 0.009, rail, verts=8,
                resolution=2)
        F.cyl(f'bracket_rose{x}', 0.022, 0.008, (x, 0.12, 0.148), rail, rot=(90, 0, 0), verts=12)
    if end:
        F.extrude('waterfall', [(-0.48, 0.0), (0.4, 0.0), (0.4, 0.985)] + _arc(0.385, 0.985, 0.015, 0, 90, 2)[1:] +
                  [(-0.48, 1.0)], 0.06, (0.47, 0, 0), top, plane='side', smooth=True)

def barSegment():
    F = G.GameFrame(shift=0.40)
    with G.new_parts() as still:
        _bar(F, 1.0, 0.0)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

def barEnd():
    F = G.GameFrame(shift=0.40)
    with G.new_parts() as still:
        _bar(F, 0.92, -0.04, end=True)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
# its top under its content `surface`.
FAMILY = G.Family(
    'kitchen_counters', M,
    [G.Model('counter', counter, size=(1.0, 0.95, 0.66), key='model.counter', centre=(0, -0.10),
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCorner', counterCorner, size=(1.0, 1.26, 0.93), key='model.counterCorner', centre=(0, 0.035),
             # sandwich, as the counter
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterEnd', counterEnd, size=(1.0, 0.95, 0.66), key='model.counterEnd', centre=(0, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCornerOuter', counterCornerOuter, size=(0.81, 1.035, 0.66), key='model.counterCornerOuter', centre=(-0.095, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterDrawers', counterDrawers, size=(1.0, 0.95, 0.66), key='model.counterDrawers', centre=(0, -0.10),
             # putting clean dishes away here (cooking putAwayAt prep, anim fetch) slides the top drawer out
             motions={'uses': {'fetch': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterOpenShelf', counterOpenShelf, size=(1.0, 0.95, 0.66), key='model.counterOpenShelf', centre=(0, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('butcherBlock', butcherBlock, size=(0.9, 0.95, 0.813), key='model.butcherBlock',
             # sandwich slides the knife drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('leg', 'Base'), ('block', 'Block'), ('block2', 'Top')]),
     G.Model('counterMarble', counterMarble, size=(1.0, 0.953, 0.66), key='model.counterMarble', centre=(0, -0.10),
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('navy', 'Cabinet'), ('marble', 'Marble top'), ('brass', 'Handles')]),
     # K5: tall and wall storage
     G.Model('pantry', pantry, size=(0.94, 2.27, 0.63), key='model.pantry', centre=(0, -0.115),
             # snack fetches from the larder: both doors swing open
             motions={'uses': {'snack': 'doors_open'}, 'hold': DOOR_HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('pantryTall', pantryTall, size=(0.96, 2.38, 0.71), key='model.pantryTall', centre=(0, -0.08),
             # snack, as the larder
             motions={'uses': {'snack': 'doors_open'}, 'hold': DOOR_HOLD},
             paint=[('cab', 'Cabinet'), ('brass', 'Handles')]),
     # putting clean dishes away (a `cupboard` station, cooking putAwayAt, anim fetch) opens the doors
     G.Model('wallCabinet', wallCabinet, size=(0.98, 0.732, 0.379), key='model.wallCabinet',
             centre=(0, -0.2405), floor=False, motions={'uses': {'fetch': 'doors_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('wallCabinetGlass', wallCabinetGlass, size=(0.96, 0.72, 0.406), key='model.wallCabinetGlass',
             centre=(0, -0.227), floor=False, motions={'uses': {'fetch': 'door_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('wallCabinetCorner', wallCabinetCorner, size=(0.96, 0.722, 0.406), key='model.wallCabinetCorner',
             centre=(0, -0.227), floor=False, motions={'uses': {'fetch': 'door_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('rangeHood', rangeHood, size=(0.9, 0.712, 0.515), key='model.rangeHood',
             centre=(0, -0.1725), floor=False,
             paint=[('steel', 'Hood'), ('trim', 'Trim')]),
     G.Model('spiceRack', spiceRack, size=(0.52, 0.44, 0.106), key='model.spiceRack',
             centre=(0, -0.377), floor=False,
             paint=[('oak', 'Rack')]),
     G.Model('potRail', potRail, size=(0.91, 0.545, 0.22), key='model.potRail',
             centre=(0, -0.32), floor=False,
             paint=[('metal', 'Rail'), ('copper', 'Pots')]),
     G.Model('kitchenShelfWall', kitchenShelfWall, size=(0.9, 0.76, 0.24), key='model.kitchenShelfWall',
             centre=(0, -0.31), floor=False,
             paint=[('oak', 'Shelves'), ('oakd', 'Brackets')]),
     G.Model('trashCan', trashCan, size=(0.31, 0.685, 0.36), key='model.trashCan', centre=(0, -0.247),
             # scraping a plate into the bin (the cooking `bin` station, anim clean) lifts the lid
             motions={'uses': {'clean': 'lid_open'}, 'hold': 1.0},
             paint=[('steel', 'Bin'), ('lid', 'Lid')]),
     G.Model('recyclingBins', recyclingBins, size=(0.92, 0.66, 0.455), key='model.recyclingBins', centre=(0, -0.212),
             # sorting the recycling lifts the three lids one after another
             motions={'uses': {'sortRecycling': [{'clip': f'lid{i}_open', 'delay': 0.6 * i} for i in range(3)]},
                      'hold': 0.9},
             paint=[('bin3', 'Left bin'), ('bin2', 'Middle bin'), ('bin1', 'Right bin'), ('lid', 'Handles')]),   # as seen from the front
     # K6: islands and bars
     G.Model('kitchenIsland', kitchenIsland, size=(1.94, 0.95, 0.839), key='model.kitchenIsland', centre=(0, 0.0495),
             # sandwich slides the drawer out (breakfast is eaten standing, or on a stool at the island)
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('islandSmall', islandSmall, size=(0.96, 0.95, 0.911), key='model.islandSmall', centre=(0, 0.0255),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('islandLong', islandLong, size=(2.96, 0.95, 0.911), key='model.islandLong', centre=(0, 0.0255),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('barSegment', barSegment, size=(1.0, 1.06, 0.89), key='model.barSegment', centre=(0, 0.015),
             paint=[('walnut', 'Cabinet'), ('oak', 'Worktop')]),
     G.Model('barEnd', barEnd, size=(1.0, 1.06, 0.89), key='model.barEnd', centre=(0, 0.015),
             paint=[('walnut', 'Cabinet'), ('oak', 'Worktop')])],
    comment='Blender-built kitchen base counters (tools/art/blender/families/kitchen_counters.py). Listed '
            'in the base manifest\'s packs. Each counter is a plinth, a carcass and a worktop (to 0.95): '
            'the plain, corner and marble counters carry a drawer over two doors and slide it out 25 cm '
            '(`drawer_open`) while a resident makes a sandwich, as does the butcher block\'s knife drawer. '
            'The tall and wall storage (K5) adds a larder whose doors swing open for a snack; the islands '
            'and bars (K6) add freestanding islands with their cupboards at the front and a worktop '
            'overhanging the back for stools, and slatted bars with a top at 1.06, the kitchen island '
            'sliding a drawer out between its stools for a sandwich.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
