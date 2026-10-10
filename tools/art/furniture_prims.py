# Primitive-part furniture that stands in for the downloaded library models the base objects and the
# personality packs used to load (Kenney Furniture Kit, Poly Haven): fridges, sinks, toilet, showers,
# sofa and armchairs, plants, CRT telly, workbench, wing and rocking chairs, tea sets, cabinets,
# stoves, washer-dryer and so on. Every function returns placeholder parts (box, cylinder, cone,
# sphere, capsule, blob, lathe, torus; see render/babylon/models.ts) placed in the box the old file
# was fitted into, so seats, tops, things standing on them and wall/back edges still line up.
#
# Conventions as furniture.py: 1 unit = 1 m, origin at the footprint centre on the floor, front +Z,
# back against the wall at -Z (backs stop at z = -0.455, inside the footprint margin). Plain finishes
# (matte, satin, gloss, metal, chrome) share one material; fabric, wood, foliage, bark and glass are
# a draw call each, so a model here uses plain finishes plus at most three of those.
#
# Used by furniture.py (base objects and style variants) and pack_<trait>.py; not run on its own.
import math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from objects_extra import box, cyl, cone, sph, rod, part   # noqa: E402

BACK = -0.455            # a floor object's back edge against the wall
CHROME, BRASS, PORCELAIN, SOIL = '#D0D3D6', '#B8925A', '#F6F4EE', '#3B2B20'
LEAVES = ['#3E6B35', '#4A7A3D', '#5B8A45', '#6F9A4E']


def lathe(size, at, color, finish, profile, **kw):
    """Surface of revolution: `profile` [radius, y] in unit space (radius 0..0.5, y -0.5..0.5)."""
    return part('lathe', size, at, color, finish, profile=[[round(r, 4), round(y, 4)] for r, y in profile], **kw)


def torus(size, at, color, finish, **kw):
    """size: [outer diameter x, tube thickness, outer diameter z]."""
    return part('torus', size, at, color, finish, **kw)


def cushion(size, at, color, finish='fabric', **kw):
    kw.setdefault('radius', round(min(size) * 0.42, 3))
    return box(size, at, color, finish, **kw)


def arc(cx, cy, cz, r, a0, a1, n, plane='xy'):
    """Points on a circle arc (degrees; 0 = +x, 90 = up in 'xy', +z in 'xz')."""
    out = []
    for i in range(n + 1):
        a = math.radians(a0 + (a1 - a0) * i / n)
        if plane == 'xy':
            out.append([cx + r * math.cos(a), cy + r * math.sin(a), cz])
        else:
            out.append([cx + r * math.cos(a), cy, cz + r * math.sin(a)])
    return out


def polyline(pts, t, color, finish, **kw):
    return [rod(a, b, t, color, finish, **kw) for a, b in zip(pts, pts[1:])]


def panels_on_arc(cx, cz, r, a0, a1, n, y0, y1, color, finish, t=0.012):
    """Thin upright panels along a horizontal arc (a curved glass screen), angles in degrees from +x."""
    out = []
    for i in range(n):
        p0 = math.radians(a0 + (a1 - a0) * i / n)
        p1 = math.radians(a0 + (a1 - a0) * (i + 1) / n)
        xa, za = cx + r * math.cos(p0), cz + r * math.sin(p0)
        xb, zb = cx + r * math.cos(p1), cz + r * math.sin(p1)
        length = math.hypot(xb - xa, zb - za) + 0.004
        yaw = -math.degrees(math.atan2(zb - za, xb - xa))
        out.append(box([length, y1 - y0, t], [(xa + xb) / 2, (y0 + y1) / 2, (za + zb) / 2], color, finish, radius=0.003, rotate=[0, yaw, 0]))
    return out


def bar_handle(x, y0, y1, z, color=CHROME, fin='chrome', t=0.022, stand=0.03):
    """A vertical bar handle on two stand-offs, its front at z + stand + t."""
    h = y1 - y0
    return [box([t, h, t], [x, (y0 + y1) / 2, z + stand + t / 2], color, fin, radius=t * 0.45),
            box([t * 0.8, t * 0.8, stand], [x, y0 + 0.03, z + stand / 2], color, fin, radius=0.004),
            box([t * 0.8, t * 0.8, stand], [x, y1 - 0.03, z + stand / 2], color, fin, radius=0.004)]


def pull(x, y, z, w, color=CHROME, fin='chrome', t=0.016):
    """A horizontal bar pull on two stand-offs, its back on z."""
    return [box([w, t, t], [x, y, z + 0.022 + t / 2], color, fin, radius=t * 0.45),
            box([t * 0.8, t * 0.8, 0.022], [x - w / 2 + 0.02, y, z + 0.011], color, fin, radius=0.003),
            box([t * 0.8, t * 0.8, 0.022], [x + w / 2 - 0.02, y, z + 0.011], color, fin, radius=0.003)]


def knob(x, y, z, d=0.035, color=BRASS, fin='metal'):
    return sph([d, d, d * 0.8], [x, y, z + d * 0.35], color, fin, segments=8)


def pot_profile(rim=0.06, foot=0.36):
    """A flower pot: tapered side, rolled rim, soil sunk below it (outer surface up, inner down)."""
    return [[0, -0.5], [foot, -0.5], [0.44, 0.5 - rim * 1.6], [0.5, 0.5 - rim * 1.2], [0.5, 0.5], [0.44, 0.5], [0.42, 0.5 - rim * 1.4], [0, 0.5 - rim * 1.4]]


def leaf(base, a, length, width, tilt, color, finish='foliage', droop=0.0, t=0.012):
    """A flat leaf (ellipsoid) whose stalk end sits at `base`, pointing out along heading `a` (deg, 0 = +z)
    and up by `tilt` degrees; `droop` bends a second half downwards."""
    ar, tr = math.radians(a), math.radians(tilt)
    d = [math.sin(ar) * math.cos(tr), math.sin(tr), math.cos(ar) * math.cos(tr)]
    if not droop:
        c = [base[i] + d[i] * length / 2 for i in range(3)]
        return [sph([width, t, length], c, color, finish, segments=8, rotate=[-tilt, a, 0])]
    half = length / 2
    c1 = [base[i] + d[i] * half / 2 for i in range(3)]
    mid = [base[i] + d[i] * half for i in range(3)]
    t2 = tilt - droop
    t2r = math.radians(t2)
    d2 = [math.sin(ar) * math.cos(t2r), math.sin(t2r), math.cos(ar) * math.cos(t2r)]
    c2 = [mid[i] + d2[i] * half / 2 for i in range(3)]
    return [sph([width * 0.8, t, half * 1.25], c1, color, finish, segments=8, rotate=[-tilt, a, 0]),
            sph([width, t, half * 1.25], c2, color, finish, segments=8, rotate=[-t2, a, 0])]


# =============================================================================================
# Kitchen appliances
# =============================================================================================
def fridge_classic(w=0.74, h=1.8, d=0.67, x=0.0, body='#F3F3F1', gap='#B9BCBF', handle=CHROME, hfin='chrome',
                   body_fin='gloss', freezer_top=True, split=0.68, handles='bar'):
    """A tall fridge-freezer, back on the wall: plinth, cabinet, two doors with a gap line, handles on
    the left (hinges right). `handles`: 'bar' (vertical chrome bars) or 'wood' (horizontal pulls, `handle`
    being the wood colour)."""
    z0 = BACK
    zf = z0 + d - 0.04                     # cabinet front; doors in front of it
    p = [box([w - 0.06, 0.06, d - 0.1], [x, 0.03, z0 + (d - 0.1) / 2], gap, 'satin', radius=0.006),     # recessed plinth
         box([w, h - 0.06, d - 0.04], [x, 0.06 + (h - 0.06) / 2, z0 + (d - 0.04) / 2], body, body_fin, radius=0.03)]
    ys = h * split
    top = (ys + 0.008, h - 0.012) if freezer_top else (0.07, ys - 0.008)
    low = (0.07, ys - 0.008) if freezer_top else (ys + 0.008, h - 0.012)
    for y0, y1 in (top, low):
        p.append(box([w - 0.012, y1 - y0, 0.04], [x, (y0 + y1) / 2, zf + 0.02], body, body_fin, radius=0.022))      # door
    p.append(box([w - 0.04, 0.014, 0.03], [x, ys, zf + 0.012], gap, 'satin', radius=0.004))                       # door gap
    hx = x - w / 2 + 0.07
    if handles == 'bar':
        a0, a1 = (ys + 0.05, ys + 0.3) if freezer_top else (ys - 0.32, ys - 0.06)
        b0, b1 = (ys - 0.42, ys - 0.06) if freezer_top else (ys + 0.06, ys + 0.5)
        p += bar_handle(hx, a0, a1, zf + 0.04, handle, hfin) + bar_handle(hx, b0, b1, zf + 0.04, handle, hfin)
    else:                                  # wooden pulls on the door edges either side of the split
        p += [box([0.32, 0.03, 0.03], [hx + 0.13, ys + 0.035, zf + 0.055], handle, 'wood', radius=0.01),
              box([0.32, 0.03, 0.03], [hx + 0.13, ys - 0.035, zf + 0.055], handle, 'wood', radius=0.01)]
    p += [box([0.1, 0.018, 0.004], [x + w / 2 - 0.12, h - 0.06 if freezer_top else ys - 0.06, zf + 0.041], gap, 'satin', radius=0.002)]  # badge
    return p


def fridge_side_by_side(w=0.88, h=1.84, d=0.69, x=0.0, steel='#C3C7CB', dark='#3A3D41'):
    """An American fridge-freezer: two tall stainless doors, long dark handles in the middle, a dispenser."""
    z0 = BACK
    zf = z0 + d - 0.045
    p = [box([w - 0.06, 0.07, d - 0.1], [x, 0.035, z0 + (d - 0.1) / 2], dark, 'satin', radius=0.006),             # kick plate
         box([w, h - 0.07, d - 0.045], [x, 0.07 + (h - 0.07) / 2, z0 + (d - 0.045) / 2], steel, 'metal', radius=0.03),
         box([w - 0.04, 0.03, 0.03], [x, 0.055, zf + 0.012], '#2B2C2E', 'satin', radius=0.006)]                    # grille
    dw = (w - 0.016) / 2
    for s in (-1, 1):
        p.append(box([dw - 0.006, h - 0.1, 0.045], [x + s * (dw / 2 + 0.004), 0.08 + (h - 0.1) / 2, zf + 0.0225], steel, 'metal', radius=0.02))
        p += bar_handle(x + s * 0.05, 0.75, 1.55, zf + 0.045, dark, 'satin', t=0.026, stand=0.035)
    dx = x - dw / 2 - 0.004
    p += [box([0.2, 0.32, 0.014], [dx, 1.18, zf + 0.05], '#2F3236', 'satin', radius=0.02),                         # dispenser
          box([0.15, 0.04, 0.004], [dx, 1.3, zf + 0.058], '#7FC8E8', 'gloss', radius=0.006),                       # display
          box([0.04, 0.08, 0.02], [dx, 1.18, zf + 0.06], '#55595F', 'satin', radius=0.008),                        # paddle
          box([0.16, 0.01, 0.05], [dx, 1.03, zf + 0.07], '#55595F', 'satin', radius=0.004)]                        # drip tray
    return p


def mini_fridge(x=0.0, z=-0.02, w=0.56, h=0.37, d=0.46, body='#9FD3C7', trim='#7FB5A9'):
    """A low cube fridge (lid and door screen come from the pack): body, proud door, chrome handle, feet."""
    z0, z1 = z - d / 2, z + d / 2
    p = [box([w, h - 0.03, d - 0.02], [x, 0.03 + (h - 0.03) / 2, z0 + (d - 0.02) / 2], body, 'gloss', radius=0.03),
         box([w - 0.03, h - 0.08, 0.022], [x, 0.04 + (h - 0.08) / 2 + 0.01, z1 - 0.009], body, 'gloss', radius=0.02),   # door
         box([w - 0.08, 0.012, 0.02], [x, h - 0.025, z1 - 0.008], trim, 'satin', radius=0.004)]                     # top trim
    p += bar_handle(x - w / 2 + 0.06, 0.1, 0.3, z1 + 0.002, CHROME, 'chrome', t=0.018, stand=0.022)
    for fx in (x - w / 2 + 0.06, x + w / 2 - 0.06):
        for fz in (z0 + 0.06, z1 - 0.06):
            p.append(cyl([0.04, 0.03, 0.04], [fx, 0.015, fz], trim, 'satin', segments=10))
    return p


def sink_cabinet(w=0.86, h=0.92, d=0.61, x=0.0, cab='#EDEBE6', door='#E0DDD6', door_fin='satin', top='#DCDDDA',
                 basin='#C9CCD0', tap=CHROME, tap_fin='chrome', style='shaker', hole=0.48, handle=CHROME, hfin='chrome'):
    """A base cabinet with an inset basin cut into the worktop (front edge z ~0.16, top at h), two doors,
    a gooseneck tap behind the basin. style 'shaker' (framed doors) or 'slab' (flat doors, long pulls)."""
    z0 = BACK
    zt = z0 + d                               # worktop front
    zc = zt - 0.045                           # carcass front
    t = 0.04
    p = [box([w - 0.04, 0.09, d - 0.1], [x, 0.045, z0 + (d - 0.1) / 2], '#CFCBC3', 'satin', radius=0.006),          # plinth
         box([w - 0.01, h - t - 0.09, zc - z0], [x, 0.09 + (h - t - 0.09) / 2, (z0 + zc) / 2], cab, 'satin', radius=0.01)]
    # worktop as a frame round the basin opening
    hx0, hx1 = x - hole / 2, x + hole / 2
    hz0, hz1 = z0 + 0.17, zt - 0.09
    yt = h - t / 2
    p += [box([w, t, hz0 - z0], [x, yt, (z0 + hz0) / 2], top, 'gloss', radius=0.008),
          box([w, t, zt - hz1], [x, yt, (hz1 + zt) / 2], top, 'gloss', radius=0.008),
          box([hx0 - (x - w / 2), t, hz1 - hz0], [(x - w / 2 + hx0) / 2, yt, (hz0 + hz1) / 2], top, 'gloss', radius=0.008),
          box([(x + w / 2) - hx1, t, hz1 - hz0], [(hx1 + x + w / 2) / 2, yt, (hz0 + hz1) / 2], top, 'gloss', radius=0.008)]
    # the basin below the opening: floor, walls, drain
    by = h - 0.17
    bw, bd = hole - 0.004, hz1 - hz0 - 0.004
    p += [box([bw, 0.012, bd], [x, by, (hz0 + hz1) / 2], basin, 'metal', radius=0.004),
          box([bw, h - by - t / 2, 0.01], [x, (by + h - t / 2) / 2, hz0 + 0.007], basin, 'metal', radius=0.003),
          box([bw, h - by - t / 2, 0.01], [x, (by + h - t / 2) / 2, hz1 - 0.007], basin, 'metal', radius=0.003),
          box([0.01, h - by - t / 2, bd], [hx0 + 0.007, (by + h - t / 2) / 2, (hz0 + hz1) / 2], basin, 'metal', radius=0.003),
          box([0.01, h - by - t / 2, bd], [hx1 - 0.007, (by + h - t / 2) / 2, (hz0 + hz1) / 2], basin, 'metal', radius=0.003),
          cyl([0.06, 0.008, 0.06], [x, by + 0.008, (hz0 + hz1) / 2 + 0.03], '#8E9196', 'metal', segments=12)]
    # tap behind the basin
    tz = (z0 + hz0) / 2
    p += [cyl([0.055, 0.03, 0.055], [x, h + 0.015, tz], tap, tap_fin, segments=12),
          cyl([0.026, 0.26, 0.026], [x, h + 0.13, tz], tap, tap_fin, segments=10)]
    if style == 'slab':                         # square spout
        p += [box([0.026, 0.026, 0.2], [x, h + 0.26, tz + 0.087], tap, tap_fin, radius=0.006),
              box([0.07, 0.016, 0.016], [x + 0.045, h + 0.08, tz], tap, tap_fin, radius=0.005)]
    else:                                       # gooseneck: an arc over towards the basin, then down
        neck = [[x, q[1], tz + 0.07 + q[0]] for q in arc(0, h + 0.26, 0, 0.07, 180, 0, 6, 'xy')]
        p += polyline(neck, 0.024, tap, tap_fin)
        p += [cyl([0.024, 0.07, 0.024], [x, h + 0.225, tz + 0.14], tap, tap_fin, segments=10),
              box([0.07, 0.016, 0.016], [x + 0.05, h + 0.06, tz], tap, tap_fin, radius=0.005)]
    # doors
    dw = (w - 0.03) / 2
    dy0, dy1 = 0.11, h - t - 0.02
    for s in (-1, 1):
        cx = x + s * (dw / 2 + 0.005)
        p.append(box([dw, dy1 - dy0, 0.02], [cx, (dy0 + dy1) / 2, zc + 0.01], door, door_fin, radius=0.006))
        if style == 'shaker':
            p.append(box([dw - 0.1, dy1 - dy0 - 0.1, 0.012], [cx, (dy0 + dy1) / 2, zc + 0.02], cab, 'satin', radius=0.004))
            p += bar_handle(cx - s * (dw / 2 - 0.05), dy1 - 0.2, dy1 - 0.06, zc + 0.02, handle, hfin, t=0.016, stand=0.02)
        else:
            p += pull(cx, dy1 - 0.05, zc + 0.02, dw * 0.55, handle, hfin)
    return p


def stove_electric(w=0.5, h=0.86, d=0.64, x=0.0, z0=-0.45, body='#F4F3EF', hob='#2F3236'):
    """A freestanding electric cooker: ceramic hob with four plates, control strip, oven door, drawer."""
    zf = z0 + d
    p = [box([w, h - 0.03, d - 0.03], [x, (h - 0.03) / 2, z0 + (d - 0.03) / 2], body, 'gloss', radius=0.012),
         box([w + 0.004, 0.03, d - 0.02], [x, h - 0.015, z0 + (d - 0.02) / 2], hob, 'gloss', radius=0.006),
         box([w, 0.1, 0.03], [x, h + 0.02, z0 + 0.015], body, 'gloss', radius=0.01)]                                 # backguard
    for dx, dz, r in ((-0.11, -0.12, 0.17), (0.11, -0.14, 0.13), (-0.11, 0.08, 0.13), (0.11, 0.08, 0.17)):
        p += [torus([r, 0.01, r], [x + dx, h + 0.002, z0 + d / 2 + dz], '#55595F', 'satin', segments=20),
              cyl([r * 0.8, 0.004, r * 0.8], [x + dx, h + 0.001, z0 + d / 2 + dz], '#3A3D41', 'gloss', segments=20)]
    p += [box([w - 0.02, 0.07, 0.012], [x, h - 0.08, zf - 0.024], '#E3E2DE', 'satin', radius=0.006)]              # control strip
    for i in range(4):
        p.append(cyl([0.035, 0.022, 0.035], [x - 0.15 + i * 0.1, h - 0.08, zf - 0.01], '#2F3236', 'satin', rotate=[90, 0, 0], segments=12))
    p += [box([w - 0.03, 0.44, 0.02], [x, 0.43, zf - 0.02], body, 'gloss', radius=0.01),                           # oven door
          box([w - 0.14, 0.24, 0.006], [x, 0.43, zf - 0.008], '#1D2228', 'gloss', radius=0.012),
          box([w - 0.1, 0.02, 0.02], [x, 0.68, zf + 0.012], CHROME, 'chrome', radius=0.008),
          box([w - 0.03, 0.13, 0.02], [x, 0.11, zf - 0.02], body, 'gloss', radius=0.008),                          # drawer
          box([w - 0.2, 0.014, 0.012], [x, 0.15, zf - 0.006], '#C9CBCC', 'satin', radius=0.004)]
    return p


def range_cooker(x=0.6, w=0.62, h=0.92, d=0.61, body='#F1E6C8', hob='#4A4E54', glass='#4A3A30', trim='#ECE6D8'):
    """A cream retro gas range: four burners with grates, knobs, oven door with window and a chrome rail."""
    z0 = BACK
    zf = z0 + d
    p = [box([w, h - 0.04, d - 0.03], [x, (h - 0.04) / 2, z0 + (d - 0.03) / 2], body, 'gloss', radius=0.02),
         box([w - 0.06, 0.06, 0.02], [x, 0.03, zf - 0.025], hob, 'satin', radius=0.006),                             # kick
         box([w + 0.004, 0.035, d - 0.01], [x, h - 0.0175, z0 + (d - 0.01) / 2], hob, 'satin', radius=0.008),       # hob plate
         box([w, 0.16, 0.04], [x, h + 0.08, z0 + 0.02], body, 'gloss', radius=0.015)]                               # backguard
    for dx in (-0.15, 0.15):
        for dz in (-0.12, 0.12):
            cx, cz = x + dx, z0 + d / 2 + dz
            p += [cyl([0.13, 0.012, 0.13], [cx, h + 0.006, cz], '#2B2C2E', 'satin', segments=16),
                  cyl([0.07, 0.016, 0.07], [cx, h + 0.012, cz], '#8E9196', 'metal', segments=12),
                  box([0.2, 0.012, 0.016], [cx, h + 0.02, cz], '#2B2C2E', 'satin', radius=0.004),                   # grate bars
                  box([0.016, 0.012, 0.2], [cx, h + 0.02, cz], '#2B2C2E', 'satin', radius=0.004)]
    p += [box([w - 0.03, 0.1, 0.015], [x, h - 0.1, zf - 0.02], trim, 'satin', radius=0.006)]                       # knob panel
    for i in range(5):
        p.append(cyl([0.04, 0.025, 0.04], [x - 0.22 + i * 0.11, h - 0.1, zf - 0.005], hob, 'satin', rotate=[90, 0, 0], segments=12))
    p += [box([w - 0.04, 0.5, 0.02], [x, 0.38, zf - 0.02], body, 'gloss', radius=0.012),                           # oven door
          box([w - 0.2, 0.24, 0.006], [x, 0.36, zf - 0.008], glass, 'gloss', radius=0.016),
          box([w - 0.08, 0.025, 0.025], [x, 0.7, zf + 0.025], CHROME, 'chrome', radius=0.01)]
    for dx in (-(w / 2 - 0.06), w / 2 - 0.06):
        p.append(box([0.02, 0.02, 0.04], [x + dx, 0.7, zf + 0.004], CHROME, 'chrome', radius=0.005))
    return p


def kitchen_cabinet(w=0.82, h=0.9, d=0.59, x=0.0, cab='#DDEFE6', door='#CDE6DA', top='#F4F4F2', handle=CHROME):
    """A base unit: drawer over two doors, plinth, worktop (top at h, front z ~0.135)."""
    z0 = BACK
    zc = z0 + d - 0.035
    t = 0.035
    p = [box([w - 0.04, 0.08, d - 0.1], [x, 0.04, z0 + (d - 0.1) / 2], '#B9C7C0', 'satin', radius=0.006),
         box([w - 0.01, h - t - 0.08, zc - z0], [x, 0.08 + (h - t - 0.08) / 2, (z0 + zc) / 2], cab, 'satin', radius=0.01),
         box([w, t, d], [x, h - t / 2, z0 + d / 2], top, 'gloss', radius=0.008),
         box([w - 0.03, 0.15, 0.02], [x, h - t - 0.095, zc + 0.01], door, 'satin', radius=0.008)]
    p += pull(x, h - t - 0.095, zc + 0.02, 0.22, handle, 'chrome')
    dw = (w - 0.03) / 2
    for s in (-1, 1):
        cx = x + s * (dw / 2 + 0.005)
        p.append(box([dw, h - t - 0.29, 0.02], [cx, 0.1 + (h - t - 0.29) / 2, zc + 0.01], door, 'satin', radius=0.008))
        p += bar_handle(cx - s * (dw / 2 - 0.045), h - t - 0.36, h - t - 0.22, zc + 0.02, handle, 'chrome', t=0.016, stand=0.02)
    return p


def microwave(x=-0.02, y=0.815, z0=-0.42, w=0.44, h=0.29, d=0.36, body='#F2EDE2', panel='#C9CBCC', display='#2E3A46', glass='#3A4652'):
    """A countertop microwave standing on y: door window on the left, keypad panel on the right."""
    zf = z0 + d
    pw = 0.12
    p = [box([w, h - 0.012, d - 0.01], [x, y + 0.006 + (h - 0.012) / 2, z0 + (d - 0.01) / 2], body, 'gloss', radius=0.02),
         box([w - pw - 0.02, h - 0.05, 0.016], [x - pw / 2 - 0.005, y + h / 2, zf - 0.004], body, 'gloss', radius=0.01),     # door
         box([w - pw - 0.1, h - 0.11, 0.004], [x - pw / 2 - 0.02, y + h / 2, zf + 0.005], glass, 'gloss', radius=0.012),     # window
         box([0.02, h - 0.1, 0.02], [x + w / 2 - pw - 0.03, y + h / 2, zf + 0.014], panel, 'satin', radius=0.008),          # handle
         box([pw, h - 0.04, 0.01], [x + w / 2 - pw / 2 - 0.01, y + h / 2, zf - 0.006], panel, 'satin', radius=0.006),     # keypad
         box([pw - 0.03, 0.04, 0.004], [x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.001], display, 'gloss', radius=0.004),
         box([pw - 0.07, 0.016, 0.003], [x + w / 2 - pw / 2 - 0.01, y + h - 0.06, zf + 0.003], '#7FE0A0', 'gloss', radius=0.002)]
    for i in range(3):
        for j in range(3):
            p.append(box([0.022, 0.016, 0.006], [x + w / 2 - pw / 2 - 0.04 + i * 0.03, y + h - 0.12 - j * 0.03, zf + 0.001], '#E9E7E2', 'satin', radius=0.004))
    p.append(cyl([0.045, 0.02, 0.045], [x + w / 2 - pw / 2 - 0.01, y + 0.06, zf + 0.004], '#E9E7E2', 'satin', rotate=[90, 0, 0], segments=12))  # dial
    for fx in (x - w / 2 + 0.04, x + w / 2 - 0.04):
        p.append(cyl([0.03, 0.008, 0.03], [fx, y + 0.004, zf - 0.05], '#3A3D41', 'satin', segments=8))
    return p


# =============================================================================================
# Laundry
# =============================================================================================
def washer_dryer(x=-0.56, w=0.68, h=1.72, d=0.62, body='#F7F7F5', trim='#D5D7D8', display='#2E3A46', glass='#C9E0E8'):
    """A washer with a dryer stacked on top: two cabinets, control strips, porthole doors."""
    z0 = BACK
    zf = z0 + d
    hh = h / 2
    p = []
    for k in range(2):
        y0 = k * hh
        p += [box([w, hh - 0.006, d - 0.02], [x, y0 + (hh - 0.006) / 2 + 0.003, z0 + (d - 0.02) / 2], body, 'gloss', radius=0.02),
              box([w - 0.03, 0.1, 0.012], [x, y0 + hh - 0.075, zf - 0.016], trim, 'satin', radius=0.006),                # control strip
              box([0.16, 0.05, 0.004], [x + 0.12, y0 + hh - 0.075, zf - 0.009], display, 'gloss', radius=0.006),
              box([0.07, 0.02, 0.002], [x + 0.1, y0 + hh - 0.075, zf - 0.006], '#7FC8E8', 'gloss', radius=0.003),
              cyl([0.07, 0.025, 0.07], [x - 0.2, y0 + hh - 0.075, zf - 0.004], CHROME, 'chrome', rotate=[90, 0, 0], segments=16),   # dial
              box([0.04, 0.02, 0.006], [x - 0.06, y0 + hh - 0.075, zf - 0.008], trim, 'satin', radius=0.004)]
        cy = y0 + (hh - 0.13) / 2 + 0.03
        p += [torus([0.46, 0.05, 0.46], [x, cy, zf - 0.005], CHROME, 'chrome', rotate=[90, 0, 0], segments=28),           # porthole ring
              cyl([0.4, 0.012, 0.4], [x, cy, zf - 0.008], glass, 'glass', rotate=[90, 0, 0], segments=28),
              cyl([0.36, 0.01, 0.36], [x, cy, zf - 0.03], '#8C949B', 'satin', rotate=[90, 0, 0], segments=24),          # drum seen through
              box([0.04, 0.1, 0.03], [x + 0.25, cy, zf + 0.006], trim, 'satin', radius=0.012)]                          # door handle
        if k == 0:
            p.append(box([w - 0.03, 0.05, 0.012], [x, 0.03, zf - 0.016], trim, 'satin', radius=0.006))                # washer kick
    return p


# =============================================================================================
# Bathroom
# =============================================================================================
def toilet(x=0.0, z0=-0.44, body=PORCELAIN, fin='gloss', seat=None, seat_fin='gloss', lever=CHROME, lever_fin='chrome'):
    """A close-coupled toilet: pedestal bowl (seat top 0.43), cistern on the wall, lid up against it."""
    seat = seat or body
    bz = z0 + 0.43                           # bowl centre
    p = [lathe([0.3, 0.2, 0.36], [x, 0.1, bz - 0.04], body, fin, [[0, -0.5], [0.5, -0.5], [0.42, 0.0], [0.38, 0.5], [0, 0.5]], segments=24),   # foot
         lathe([0.4, 0.2, 0.5], [x, 0.3, bz], body, fin, [[0, -0.5], [0.3, -0.5], [0.36, -0.1], [0.48, 0.38], [0.5, 0.5], [0, 0.5]], segments=24),  # bowl
         cyl([0.42, 0.025, 0.52], [x, 0.4125, bz], seat, seat_fin, segments=28),                                          # seat
         cyl([0.3, 0.004, 0.38], [x, 0.426, bz + 0.01], '#DDE3E4', 'gloss', segments=24),                                  # water seen in the bowl
         box([0.38, 0.16, 0.24], [x, 0.36, z0 + 0.12], body, fin, radius=0.04),                                             # bowl back
         box([0.42, 0.36, 0.18], [x, 0.62, z0 + 0.09], body, fin, radius=0.035),                                            # cistern
         box([0.44, 0.035, 0.2], [x, 0.818, z0 + 0.1], body, fin, radius=0.015),                                            # cistern lid
         cyl([0.4, 0.022, 0.44], [x, 0.63, z0 + 0.2], seat, seat_fin, rotate=[-97, 0, 0], segments=28),                 # lid, up
         box([0.06, 0.015, 0.03], [x - 0.15, 0.74, z0 + 0.195], lever, lever_fin, radius=0.006)]                          # lever
    return p


def shower_cabin(w=0.9, h=2.15, tray='#F4F4F2', tray_fin='gloss', frame=CHROME, frame_fin='chrome', glass='#DCE6E8',
                 wall='#E4ECEC', joint='#C8D3D4', head=None, fittings=True):
    """A square 1x1 cabin: low tray, tiled back and left walls, glass right side and sliding front doors,
    frame posts and rails; with `fittings`, a riser rail with a hand shower, a mixer and an overhead head."""
    head = head or frame
    s = w / 2
    T = 0.07                                  # tray top
    p = [box([w, T - 0.012, w], [0, (T - 0.012) / 2, 0], tray, tray_fin, radius=0.02),
         box([w - 0.06, 0.012, w - 0.06], [0, T - 0.018, 0], tray, tray_fin, radius=0.01),
         cyl([0.08, 0.006, 0.08], [0.1, T - 0.012, 0.1], frame, frame_fin, segments=12)]                             # drain
    for zz in (-s + 0.006, s - 0.006):       # tray rim (front/back) and sides: the floor sits a little lower
        p.append(box([w, 0.012, 0.03], [0, T - 0.006, zz + (0.009 if zz < 0 else -0.009)], tray, tray_fin, radius=0.004))
    p += [box([w, h - T, 0.03], [0, T + (h - T) / 2, -s + 0.015], wall, 'satin', radius=0.006),                       # back wall panel
          box([0.03, h - T, w - 0.03], [-s + 0.015, T + (h - T) / 2, 0.015], wall, 'satin', radius=0.006)]            # left wall panel
    for k in range(1, 7):                                                                                                 # tile joints
        y = T + k * (h - T) / 7
        p += [box([w - 0.04, 0.008, 0.004], [0, y, -s + 0.031], joint, 'satin', radius=0),
              box([0.004, 0.008, w - 0.07], [-s + 0.031, y, 0.015], joint, 'satin', radius=0)]
    for k in range(1, 3):
        p += [box([0.008, h - T, 0.004], [-s + k * w / 3, T + (h - T) / 2, -s + 0.031], joint, 'satin', radius=0),
              box([0.004, h - T, 0.008], [-s + 0.031, T + (h - T) / 2, -s + k * w / 3 + 0.03], joint, 'satin', radius=0)]
    p += [box([0.012, h - T - 0.06, w - 0.06], [s - 0.012, T + (h - T - 0.06) / 2, 0.0], glass, 'glass', radius=0.003),  # right side
          box([w * 0.55, h - T - 0.08, 0.012], [-s + w * 0.3, T + (h - T - 0.08) / 2 + 0.01, s - 0.012], glass, 'glass', radius=0.003),   # fixed front pane
          box([w * 0.5, h - T - 0.1, 0.012], [s - w * 0.27, T + (h - T - 0.1) / 2 + 0.02, s - 0.03], glass, 'glass', radius=0.003)]       # sliding door
    for px, pz in ((s - 0.015, s - 0.015), (s - 0.015, -s + 0.035)):
        p.append(box([0.03, h - T, 0.03], [px, T + (h - T) / 2, pz], frame, frame_fin, radius=0.006))                  # posts
    p += [box([w - 0.03, 0.03, 0.03], [0.015, h - 0.015, s - 0.015], frame, frame_fin, radius=0.006),                  # top rails
          box([0.03, 0.03, w - 0.03], [s - 0.015, h - 0.015, 0.0], frame, frame_fin, radius=0.006),
          box([w - 0.03, 0.02, 0.03], [0.015, T + 0.01, s - 0.015], frame, frame_fin, radius=0.006),
          box([0.022, 0.4, 0.022], [s - w * 0.5 + 0.03, 1.05, s - 0.012], frame, frame_fin, radius=0.009)]              # door handle
    if not fittings:
        return p
    # riser rail with a hand shower, mixer, overhead head on an arm
    p += [box([0.025, 0.8, 0.025], [-0.1, 1.3, -s + 0.06], head, frame_fin, radius=0.01),
          box([0.03, 0.03, 0.04], [-0.1, 0.9, -s + 0.05], head, frame_fin, radius=0.008),
          box([0.03, 0.03, 0.04], [-0.1, 1.7, -s + 0.05], head, frame_fin, radius=0.008),
          part('capsule', [0.05, 0.2, 0.05], [-0.1, 1.48, -s + 0.1], head, frame_fin, rotate=[-20, 0, 0]),
          cyl([0.09, 0.02, 0.09], [-0.1, 1.58, -s + 0.13], head, frame_fin, rotate=[-20, 0, 0], segments=14),
          box([0.16, 0.08, 0.05], [0.12, 1.1, -s + 0.055], head, frame_fin, radius=0.02),
          cyl([0.05, 0.04, 0.05], [0.07, 1.1, -s + 0.09], head, frame_fin, rotate=[90, 0, 0], segments=12),
          cyl([0.05, 0.04, 0.05], [0.17, 1.1, -s + 0.09], head, frame_fin, rotate=[90, 0, 0], segments=12),
          rod([0, 2.0, -s + 0.03], [0, 2.02, -s + 0.3], 0.022, head, frame_fin),
          cyl([0.24, 0.025, 0.24], [0, 2.0, -s + 0.3], head, frame_fin, segments=24)]
    return p


def shower_round(w=0.9, h=2.1, tray='#F4F3EF', tray_fin='satin', frame=CHROME, frame_fin='chrome', glass='#E2EAEA', wall='#F4F3EF'):
    """A D-shaped 1x1 cabin: square back, round front (curved glass), white back panel, rain head."""
    s = w / 2
    T = 0.08
    r = s
    p = [box([w, T, s], [0, T / 2, -s / 2], tray, tray_fin, radius=0.02),
         cyl([w, T, w], [0, T / 2, 0], tray, tray_fin, segments=36),
         cyl([w - 0.06, 0.006, w - 0.06], [0, T + 0.001, 0], '#E9E7E2', 'satin', segments=36),
         cyl([0.08, 0.006, 0.08], [0, T + 0.004, 0.05], frame, frame_fin, segments=12),
         box([w, h - T, 0.03], [0, T + (h - T) / 2, -s + 0.015], wall, tray_fin, radius=0.01),                             # back panel
         box([0.025, h - T, s - 0.02], [-s + 0.0125, T + (h - T) / 2, -s / 2 - 0.01], glass, 'glass', radius=0.003),        # straight sides
         box([0.025, h - T, s - 0.02], [s - 0.0125, T + (h - T) / 2, -s / 2 - 0.01], glass, 'glass', radius=0.003)]
    p += panels_on_arc(0, 0, r - 0.015, 0, 180, 12, T, h - 0.03, glass, 'glass')                                          # curved front
    p += polyline([[q[0], h - 0.015, q[2]] for q in arc(0, 0, 0, r - 0.015, 0, 180, 12, 'xz')], 0.025, frame, frame_fin)  # top ring
    p += polyline([[q[0], T + 0.01, q[2]] for q in arc(0, 0, 0, r - 0.015, 0, 180, 12, 'xz')], 0.02, frame, frame_fin)
    for x in (-s + 0.0125, s - 0.0125):
        p.append(box([0.03, h - T, 0.03], [x, T + (h - T) / 2, 0.0], frame, frame_fin, radius=0.008))                 # door jambs
    p += [box([0.02, 0.5, 0.02], [0.12, 1.1, r - 0.012], frame, frame_fin, radius=0.008),                                # door knob bar
          rod([0, 1.98, -s + 0.03], [0, 2.0, -0.12], 0.02, frame, frame_fin),
          cyl([0.26, 0.02, 0.26], [0, 1.99, -0.1], frame, frame_fin, segments=24),
          cyl([0.06, 0.06, 0.06], [0.0, 1.1, -s + 0.05], frame, frame_fin, rotate=[90, 0, 0], segments=12),
          box([0.1, 0.02, 0.02], [0.04, 1.1, -s + 0.085], frame, frame_fin, radius=0.006)]
    return p


# =============================================================================================
# Seating
# =============================================================================================
def cabriole_leg(x, z, h, color, sx=0.0, sz=0.0, t=0.045):
    """A curved (cabriole) leg from the frame at height h down to a small foot, splayed by (sx, sz)."""
    knee = [x + sx * 0.6, h * 0.62, z + sz * 0.6]
    foot = [x + sx, 0.03, z + sz]
    return [rod([x, h, z], knee, t, color, 'wood', segments=10),
            rod(knee, [foot[0] - sx * 0.3, 0.06, foot[2] - sz * 0.3], t * 0.8, color, 'wood', segments=10),
            sph([t * 1.1, 0.05, t * 1.1], foot, color, 'wood', segments=8)]


def back_slices(x0, x1, n, ytop, y0, z, d, color, finish='fabric', tilt=-8, radius=0.05):
    """An upholstered back drawn as n upright channels whose tops follow ytop(u), u in 0..1 across."""
    p = []
    sw = (x1 - x0) / n
    for i in range(n):
        u = (i + 0.5) / n
        top = ytop(u)
        p.append(box([sw + 0.01, top - y0, d], [x0 + sw * (i + 0.5), (top + y0) / 2, z], color, finish, radius=radius, rotate=[tilt, 0, 0]))
    return p


def sofa_classic(fabric='#DCD5CA', wood='#6E4E36', cushion_col=None, pillows=('#B5653F', '#9DAF8C')):
    """2x1 camel-back sofa (two seats at x = +-0.5, seat top 0.44) with show-wood rail, scroll arms and
    cabriole legs."""
    cushion_col = cushion_col or fabric
    zb, zf = BACK, 0.4
    w = 1.86
    p = [box([w - 0.16, 0.18, zf - zb - 0.08], [0, 0.27, (zb + zf) / 2 - 0.02], fabric, 'fabric', radius=0.05),          # seat deck
         box([w - 0.14, 0.05, 0.035], [0, 0.17, zf - 0.07], wood, 'wood', radius=0.012)]                                   # apron rail
    p.append(sph([0.08, 0.06, 0.03], [0, 0.17, zf - 0.05], wood, 'wood', segments=10))                                     # carved shell
    # camel back: channels rising to a hump in the middle, wood rail on top
    def top(u):
        return 0.74 + 0.16 * math.exp(-((u - 0.5) / 0.22) ** 2)
    p += [box([1.7, 0.42, 0.1], [0, 0.53, zb + 0.07], fabric, 'fabric', radius=0.04)]                                 # back board
    p += back_slices(-0.84, 0.84, 7, top, 0.32, zb + 0.13, 0.17, fabric, tilt=-8, radius=0.06)
    rail = [[-0.84 + 1.68 * i / 14, top(i / 14) + 0.015, zb + 0.08] for i in range(15)]
    p += polyline(rail, 0.04, wood, 'wood')
    # scroll arms
    for s in (-1, 1):
        ax = s * (w / 2 - 0.1)
        p += [box([0.16, 0.36, zf - zb - 0.12], [ax, 0.42, (zb + zf) / 2 - 0.02], fabric, 'fabric', radius=0.06),
              cyl([0.2, zf - zb - 0.16, 0.2], [ax + s * 0.01, 0.6, (zb + zf) / 2 + 0.0], fabric, 'fabric', rotate=[90, 0, 0], segments=18),
              cyl([0.16, 0.03, 0.16], [ax + s * 0.01, 0.6, zf - 0.065], wood, 'wood', rotate=[90, 0, 0], segments=18),     # scroll face
              box([0.05, 0.28, 0.035], [ax + s * 0.01, 0.38, zf - 0.065], wood, 'wood', radius=0.015)]                     # arm post
    # seat cushions (top 0.44), a bolster of pillows
    for s in (-1, 1):
        p.append(cushion([0.78, 0.1, zf - zb - 0.26], [s * 0.4, 0.39, (zb + zf) / 2 + 0.06], cushion_col, radius=0.04))
    if pillows:
        p += [cushion([0.36, 0.34, 0.11], [-0.6, 0.6, zb + 0.24], pillows[0], rotate=[-14, 18, 0]),
              cushion([0.34, 0.32, 0.11], [0.6, 0.6, zb + 0.24], pillows[1], rotate=[-14, -18, 0])]
    for x in (-(w / 2 - 0.12), w / 2 - 0.12):
        sx = 0.025 if x > 0 else -0.025
        p += cabriole_leg(x, zf - 0.08, 0.18, wood, sx, 0.025) + cabriole_leg(x, zb + 0.08, 0.18, wood, sx, -0.02)
    return p


def armchair_bergere(fabric='#DCD5CA', wood='#6E4E36', pillow='#9DAF8C'):
    """1x1 bergere: round-topped back in a wooden frame, padded arms on open wood supports, seat top 0.42."""
    zb, zf = -0.445, 0.37
    w = 0.78
    p = [box([w - 0.12, 0.16, zf - zb - 0.1], [0, 0.26, (zb + zf) / 2 - 0.02], fabric, 'fabric', radius=0.05),
         box([w - 0.1, 0.05, 0.035], [0, 0.17, zf - 0.07], wood, 'wood', radius=0.012),
         cushion([w - 0.2, 0.1, zf - zb - 0.24], [0, 0.37, (zb + zf) / 2 + 0.05], fabric, radius=0.04)]       # seat top 0.42
    # rounded back: channels under an arched top, wooden frame round it
    def top(u):
        return 0.68 + 0.26 * math.sqrt(max(0.0, 1 - ((u - 0.5) / 0.56) ** 2))
    p += back_slices(-0.3, 0.3, 6, top, 0.3, zb + 0.115, 0.12, fabric, tilt=-10, radius=0.04)
    frame = [[-0.34 + 0.68 * i / 12, top(i / 12) + 0.025, zb + 0.06] for i in range(13)]
    frame = [[-0.34, 0.42, zb + 0.05]] + frame + [[0.34, 0.42, zb + 0.05]]
    p += polyline(frame, 0.035, wood, 'wood')
    az0, az1 = zb + 0.1, zf - 0.03
    for s in (-1, 1):
        ax = s * (w / 2 - 0.07)
        p += [cushion([0.1, 0.06, az1 - az0 - 0.08], [ax, 0.625, (az0 + az1) / 2 - 0.02], fabric, radius=0.028),         # arm pad
              box([0.06, 0.03, az1 - az0], [ax, 0.585, (az0 + az1) / 2], wood, 'wood', radius=0.012),                     # arm rail
              sph([0.07, 0.05, 0.06], [ax, 0.59, az1 - 0.01], wood, 'wood', segments=10),                                 # scroll end
              rod([ax, 0.3, az1 - 0.04], [ax, 0.58, az1 - 0.02], 0.045, wood, 'wood', segments=10),                       # front post
              box([0.05, 0.25, az1 - az0 - 0.14], [ax, 0.45, (az0 + az1) / 2 - 0.05], fabric, 'fabric', radius=0.02)]     # side panel
    p.append(cushion([0.34, 0.3, 0.1], [0, 0.6, zb + 0.2], pillow, rotate=[-14, 0, 0]))
    for x in (-(w / 2 - 0.09), w / 2 - 0.09):
        sx = 0.02 if x > 0 else -0.02
        p += cabriole_leg(x, zf - 0.08, 0.18, wood, sx, 0.02) + cabriole_leg(x, zb + 0.07, 0.18, wood, sx, -0.02)
    return p


def wing_chair(x, z=-0.02, fabric='#4F6B55', wood='#6E4E36', w=0.66, d=0.64):
    """A wing chair centred on (x, z), facing +Z: seat top 0.45, tall back with wings, rolled arms."""
    z0, z1 = z - d / 2, z + d / 2
    p = [box([w - 0.06, 0.2, d - 0.04], [x, 0.24, z], fabric, 'fabric', radius=0.05),
         cushion([w - 0.24, 0.11, d - 0.22], [x, 0.395, z + 0.07], fabric, radius=0.045)]                           # seat (top 0.45)
    def top(u):
        return 0.96 + 0.08 * math.sqrt(max(0.0, 1 - ((u - 0.5) / 0.6) ** 2))
    p += back_slices(x - w / 2 + 0.06, x + w / 2 - 0.06, 4, top, 0.3, z0 + 0.09, 0.15, fabric, tilt=-7, radius=0.06)
    for s in (-1, 1):
        ax = x + s * (w / 2 - 0.07)
        p += [box([0.12, 0.3, d - 0.12], [ax, 0.47, z + 0.02], fabric, 'fabric', radius=0.05),                       # arm
              cyl([0.15, d - 0.14, 0.15], [ax + s * 0.015, 0.62, z + 0.03], fabric, 'fabric', rotate=[90, 0, 0], segments=16),
              box([0.1, 0.36, 0.3], [x + s * (w / 2 - 0.06), 0.84, z0 + 0.17], fabric, 'fabric', radius=0.05, rotate=[0, -s * 12, 0])]  # wing
    for lx in (x - w / 2 + 0.07, x + w / 2 - 0.07):
        for lz in (z0 + 0.06, z1 - 0.06):
            p.append(cyl([0.045, 0.14, 0.045], [lx, 0.07, lz], wood, 'wood', taper=0.7, segments=10))
    return p


def rocking_chair(z=-0.02, wood='#8A5A36', w=0.6):
    """A spindle-back rocker centred on z (seat board top 0.44), curved rockers, turned arms."""
    p = [box([w - 0.1, 0.035, 0.44], [0, 0.42, z], wood, 'wood', radius=0.012)]
    R = 1.1
    for s in (-1, 1):
        x = s * (w / 2 - 0.07)
        for i in range(7):                                                                    # rockers
            a0, a1 = math.radians(-21 + i * 42 / 7), math.radians(-21 + (i + 1) * 42 / 7)
            p.append(rod([x, R + 0.025 - R * math.cos(a0), z + R * math.sin(a0)], [x, R + 0.025 - R * math.cos(a1), z + R * math.sin(a1)],
                         0.05, wood, 'wood', shape='box', t2=0.03, radius=0.008))
        p += [rod([x, 0.04, z + 0.2], [x, 0.42, z + 0.18], 0.035, wood, 'wood', segments=8),                        # front legs
              rod([x, 0.035, z - 0.2], [x, 0.42, z - 0.19], 0.035, wood, 'wood', segments=8),
              rod([x, 0.42, z - 0.19], [x * 1.02, 1.0, z - 0.32], 0.04, wood, 'wood', segments=8),                  # back stiles
              sph([0.05, 0.05, 0.05], [x * 1.02, 1.02, z - 0.325], wood, 'wood', segments=8),
              box([0.055, 0.03, 0.44], [x * 1.06, 0.66, z - 0.02], wood, 'wood', radius=0.012),                       # arms
              rod([x * 1.06, 0.42, z + 0.17], [x * 1.06, 0.66, z + 0.17], 0.03, wood, 'wood', segments=8),
              rod([x, 0.15, z - 0.19], [x, 0.15, z + 0.19], 0.022, wood, 'wood', segments=8)]                         # stretchers
    for i in range(6):
        xx = -0.15 + i * 0.06
        p.append(rod([xx, 0.44, z - 0.18], [xx, 0.94, z - 0.3], 0.02, wood, 'wood', segments=8))
    p += [box([w - 0.08, 0.08, 0.04], [0, 0.97, z - 0.31], wood, 'wood', radius=0.015, rotate=[-12, 0, 0]),       # crest rail
          rod([-w / 2 + 0.07, 0.15, z + 0.0], [w / 2 - 0.07, 0.15, z + 0.0], 0.022, wood, 'wood', segments=8)]
    return p


def painted_chair(paint='#F2F0EA', fin='satin', z=0.0):
    """A painted kitchen chair: seat top 0.46, slatted seat, splat back."""
    p = [box([0.42, 0.03, 0.42], [0, 0.445, z + 0.01], paint, fin, radius=0.01)]
    for i in range(4):
        p.append(box([0.09, 0.012, 0.4], [-0.15 + i * 0.1, 0.466, z + 0.01], paint, fin, radius=0.004))
    for x in (-0.18, 0.18):
        p += [box([0.035, 0.43, 0.035], [x, 0.215, z + 0.19], paint, fin, radius=0.008),
              box([0.035, 0.96, 0.035], [x, 0.48, z - 0.2], paint, fin, radius=0.008, rotate=[-4, 0, 0]),
              box([0.025, 0.025, 0.36], [x, 0.16, z], paint, fin, radius=0.006)]
    p += [box([0.36, 0.08, 0.03], [0, 0.9, z - 0.218], paint, fin, radius=0.012, rotate=[-4, 0, 0]),
          box([0.36, 0.03, 0.025], [0, 0.6, z - 0.205], paint, fin, radius=0.008, rotate=[-4, 0, 0]),
          box([0.11, 0.28, 0.02], [0, 0.75, z - 0.21], paint, fin, radius=0.02, rotate=[-4, 0, 0])]
    return p


# =============================================================================================
# Tables, cabinets, shelves
# =============================================================================================
def side_table(w=0.55, h=0.55, d=0.45, x=0.0, z=0.0, wood='#B07A4E', shelf=0.14, top_col=None):
    """A square wooden side table: top at h, apron, lower shelf."""
    top_col = top_col or wood
    p = [box([w, 0.03, d], [x, h - 0.015, z], top_col, 'wood', radius=0.008),
         box([w - 0.06, 0.06, d - 0.06], [x, h - 0.06, z], wood, 'wood', radius=0.006),
         box([w - 0.05, 0.02, d - 0.05], [x, shelf, z], top_col, 'wood', radius=0.005)]
    for lx in (x - w / 2 + 0.03, x + w / 2 - 0.03):
        for lz in (z - d / 2 + 0.03, z + d / 2 - 0.03):
            p.append(box([0.04, h - 0.03, 0.04], [lx, (h - 0.03) / 2, lz], wood, 'wood', radius=0.008))
    return p


def drawer_chest(w=0.8, h=0.5, d=0.42, x=0.0, wood='#9A6A45', front='#B88A60', handle=BRASS, rows=2, cols=3, legs=0.06):
    """A low chest of small drawers (vintage apothecary style): top at h, back on the wall."""
    z0 = BACK
    zf = z0 + d
    p = [box([w, 0.03, d], [x, h - 0.015, z0 + d / 2], wood, 'wood', radius=0.008),
         box([w - 0.02, h - legs - 0.03, d - 0.02], [x, legs + (h - legs - 0.03) / 2, z0 + (d - 0.02) / 2], wood, 'wood', radius=0.006),
         box([w - 0.01, 0.03, 0.02], [x, legs + 0.015, zf - 0.01], wood, 'wood', radius=0.006)]
    for lx in (x - w / 2 + 0.04, x + w / 2 - 0.04):
        for lz in (z0 + 0.04, zf - 0.04):
            p.append(lathe([0.05, legs, 0.05], [lx, legs / 2, lz], wood, 'wood', [[0, -0.5], [0.3, -0.5], [0.5, 0.1], [0.45, 0.5], [0, 0.5]], segments=10))
    fw = (w - 0.06) / cols
    fh = (h - legs - 0.08) / rows
    for i in range(cols):
        for j in range(rows):
            cx = x - w / 2 + 0.03 + fw * (i + 0.5)
            cy = legs + 0.035 + fh * (j + 0.5)
            p += [box([fw - 0.012, fh - 0.012, 0.016], [cx, cy, zf - 0.002], front, 'wood', radius=0.006),
                  box([0.07, 0.022, 0.02], [cx, cy + 0.01, zf + 0.012], handle, 'metal', radius=0.008),                  # cup pull
                  box([0.05, 0.03, 0.003], [cx, cy + 0.045, zf + 0.007], '#EFE6D2', 'matte', radius=0.002)]           # label card
    return p


def slatted_console(w=0.9, h=0.62, d=0.415, x=0.0, wood='#7A5234', slat='#8E6443', legs='#3A3550', leg_h=0.16):
    """A mid-century record console: walnut case on splayed legs, slatted sliding doors, top at h."""
    z0 = BACK
    zf = z0 + d
    p = [box([w, h - leg_h, d], [x, leg_h + (h - leg_h) / 2, z0 + d / 2], wood, 'wood', radius=0.012),
         box([w - 0.06, h - leg_h - 0.08, 0.012], [x, leg_h + (h - leg_h) / 2, zf + 0.002], '#4A3426', 'wood', radius=0.004)]
    n = 22
    sw = (w - 0.08) / n
    for i in range(n):
        p.append(box([sw * 0.62, h - leg_h - 0.1, 0.016], [x - w / 2 + 0.04 + sw * (i + 0.5), leg_h + (h - leg_h) / 2, zf + 0.008], slat, 'wood', radius=0.003))
    p += [box([0.012, h - leg_h - 0.06, 0.03], [x, leg_h + (h - leg_h) / 2, zf + 0.004], wood, 'wood', radius=0.004)]
    for lx in (x - w / 2 + 0.07, x + w / 2 - 0.07):
        sx = -0.03 if lx < x else 0.03
        for lz, sz in ((z0 + 0.07, -0.025), (zf - 0.07, 0.025)):
            p.append(rod([lx + sx, 0.006, lz + sz], [lx, leg_h + 0.01, lz], 0.03, legs, 'satin', segments=8))
            p.append(cyl([0.032, 0.012, 0.032], [lx + sx, 0.006, lz + sz], BRASS, 'metal', segments=8))
    return p


def cube_shelf(w=0.9, h=1.3, d=0.31, cols=3, rows=4, wood='#E3CFA8', bins='#C9CBCC'):
    """An open cube shelf (back on the wall), fabric bins in the bottom row, a few things in the cubes."""
    z0 = BACK
    t = 0.022
    zc = z0 + d / 2
    p = [box([w, t, d], [0, t / 2, zc], wood, 'wood', radius=0.004), box([w, t, d], [0, h - t / 2, zc], wood, 'wood', radius=0.004),
         box([w - 0.02, h - 0.02, 0.008], [0, h / 2, z0 + 0.004], '#EDE6DA', 'matte', radius=0.002)]
    for i in range(cols + 1):
        p.append(box([t, h, d], [-w / 2 + t / 2 + (w - t) * i / cols, h / 2, zc], wood, 'wood', radius=0.004))
    cw, ch = (w - t) / cols, (h - t) / rows
    for j in range(1, rows):
        p.append(box([w - 0.01, t, d], [0, t / 2 + ch * j, zc], wood, 'wood', radius=0.004))
    for i in range(cols):                                                                           # bins, bottom row
        cx = -w / 2 + t / 2 + cw * (i + 0.5)
        p += [box([cw - t - 0.012, ch - t - 0.02, d - 0.03], [cx, t + (ch - t) / 2 - 0.005, zc + 0.01], bins, 'fabric', radius=0.02),
              box([0.08, 0.02, 0.006], [cx, t + ch - 0.07, z0 + d - 0.003], '#9EA2A6', 'satin', radius=0.008)]
    cx = lambda i: -w / 2 + t / 2 + cw * (i + 0.5)
    cy = lambda j: t + ch * j
    # books upright (row 2, left), a lying stack (row 3, right), vase and bowl, a little plant
    x = cx(0) - cw / 2 + 0.03
    for k, (bw, bh, c) in enumerate([(0.03, 0.24, '#5E6F8C'), (0.025, 0.22, '#E9E4DA'), (0.035, 0.25, '#B5653F'), (0.028, 0.2, '#9DAF8C'), (0.03, 0.23, '#E9E4DA')]):
        p.append(box([bw, bh, 0.2], [x + bw / 2, cy(2) + bh / 2, zc], c, 'satin', radius=0.004))
        x += bw + 0.003
    p.append(box([0.03, 0.22, 0.2], [x + 0.06, cy(2) + 0.1, zc], '#D6A64A', 'satin', radius=0.004, rotate=[0, 0, -20]))
    for k, c in enumerate(('#E9E4DA', '#5E6F8C', '#9DAF8C')):
        p.append(box([0.2, 0.03, 0.16], [cx(2), cy(3) + 0.015 + k * 0.031, zc], c, 'satin', radius=0.004, rotate=[0, (k - 1) * 7, 0]))
    p += [lathe([0.12, 0.22, 0.12], [cx(1), cy(2) + 0.11, zc], '#F2F0EA', 'gloss',
                [[0, -0.5], [0.35, -0.5], [0.5, -0.1], [0.3, 0.35], [0.22, 0.5], [0.18, 0.5], [0, 0.4]], segments=16),
          lathe([0.18, 0.06, 0.18], [cx(1), cy(1) + 0.03, zc], '#9EA2A6', 'gloss', [[0, -0.5], [0.3, -0.5], [0.5, 0.5], [0.45, 0.5], [0, -0.3]], segments=16),
          lathe([0.12, 0.11, 0.12], [cx(0), cy(3) + 0.055, zc], '#F2F0EA', 'satin', pot_profile(), segments=14)]
    p += [part('blob', [0.16, 0.12, 0.16], [cx(0), cy(3) + 0.15, zc], LEAVES[2], 'foliage', noise=0.3),
          box([0.16, 0.2, 0.02], [cx(2), cy(1) + 0.1, z0 + 0.05], '#C9B79C', 'matte', radius=0.006, rotate=[-8, 0, 0]),   # framed print
          box([0.12, 0.15, 0.022], [cx(2), cy(1) + 0.1, z0 + 0.052], '#9DAF8C', 'matte', radius=0.004, rotate=[-8, 0, 0])]
    return p


def metal_desk(w=1.62, h=0.78, d=0.69, x=0.0, body='#8A9298', top='#E3E4E2', handle=CHROME):
    """A steel office desk used as a workbench: two drawer pedestals, a modesty panel, a light top."""
    z0 = BACK
    zf = z0 + d
    pw = 0.4
    p = [box([w, 0.035, d], [x, h - 0.0175, z0 + d / 2], top, 'satin', radius=0.008),
         box([w - 0.02, 0.025, d - 0.02], [x, h - 0.045, z0 + d / 2], body, 'metal', radius=0.004),
         box([w - 2 * pw - 0.02, 0.4, 0.02], [x, h - 0.26, z0 + 0.12], body, 'metal', radius=0.004),                # modesty panel
         box([w - 2 * pw - 0.04, 0.09, 0.02], [x, h - 0.11, zf - 0.03], body, 'metal', radius=0.006),               # centre drawer
         box([0.16, 0.014, 0.014], [x, h - 0.11, zf - 0.012], handle, 'chrome', radius=0.005)]
    for s in (-1, 1):
        cx = x + s * (w / 2 - pw / 2 - 0.01)
        p.append(box([pw, h - 0.11, d - 0.04], [cx, 0.05 + (h - 0.11) / 2 + 0.005, z0 + (d - 0.04) / 2], body, 'metal', radius=0.01))
        for j, (y0, y1) in enumerate(((0.07, 0.3), (0.31, 0.5), (0.51, h - 0.065))):
            p += [box([pw - 0.03, y1 - y0 - 0.012, 0.018], [cx, (y0 + y1) / 2, zf - 0.04 + 0.009], body, 'metal', radius=0.006),
                  box([0.12, 0.016, 0.016], [cx, y1 - 0.04, zf - 0.022 + 0.008], handle, 'chrome', radius=0.006),
                  box([0.05, 0.025, 0.003], [cx, y1 - 0.075, zf - 0.03], '#EFEDE6', 'matte', radius=0.002)]
        for lz in (z0 + 0.04, zf - 0.08):
            p.append(cyl([0.035, 0.05, 0.035], [cx - s * (pw / 2 - 0.05), 0.025, lz], '#2F3236', 'satin', segments=8))
            p.append(cyl([0.035, 0.05, 0.035], [cx + s * (pw / 2 - 0.05), 0.025, lz], '#2F3236', 'satin', segments=8))
    return p


def wooden_crate(x=-0.08, z0=-0.45, w=0.66, h=0.6, d=0.565, wood='#A87547', dark='#7C5432', straw='#D2B872'):
    """An open-topped slatted crate: corner posts, three slats a side, a floor bedded with straw and shards."""
    z1 = z0 + d
    zc = (z0 + z1) / 2
    p = [box([w - 0.04, 0.03, d - 0.04], [x, 0.04, zc], dark, 'wood', radius=0.004),
         part('blob', [w - 0.08, 0.2, d - 0.08], [x, 0.14, zc], straw, 'matte', noise=0.12)]                      # straw bedding
    for i, (dx, dz, r) in enumerate(((-0.12, -0.05, 20), (0.1, 0.06, -35), (0.02, -0.12, 70))):                 # broken plates in it
        p.append(box([0.1, 0.012, 0.06], [x + dx, 0.235, zc + dz], PORCELAIN, 'gloss', radius=0.003, rotate=[8 * (i - 1), r, 6]))
    for px in (x - w / 2 + 0.03, x + w / 2 - 0.03):
        for pz in (z0 + 0.03, z1 - 0.03):
            p.append(box([0.06, h, 0.06], [px, h / 2, pz], dark, 'wood', radius=0.008))
    sh = (h - 0.02) / 3
    for k in range(3):
        y = 0.01 + sh * (k + 0.5)
        p += [box([w - 0.02, sh - 0.025, 0.022], [x, y, z1 - 0.011], wood, 'wood', radius=0.005),
              box([w - 0.02, sh - 0.025, 0.022], [x, y, z0 + 0.011], wood, 'wood', radius=0.005),
              box([0.022, sh - 0.025, d - 0.02], [x - w / 2 + 0.011, y, zc], wood, 'wood', radius=0.005),
              box([0.022, sh - 0.025, d - 0.02], [x + w / 2 - 0.011, y, zc], wood, 'wood', radius=0.005)]
    p += [box([0.16, 0.035, 0.03], [x, h - 0.09, z1 + 0.004], dark, 'wood', radius=0.012)]                      # hand hole lip
    return p


def planter_trough(y0=0.615, w=0.84, h=0.39, d=0.38, x=0.0, wood='#C49A6C', dark='#A07A52'):
    """A slatted wooden trough (open top, soil and plants come from the pack) standing on y0."""
    z0 = BACK + 0.005
    z1 = z0 + d
    zc = (z0 + z1) / 2
    p = [box([w - 0.04, 0.025, d - 0.04], [x, y0 + 0.0125, zc], dark, 'wood', radius=0.004),
         box([w - 0.05, h - 0.1, d - 0.05], [x, y0 + 0.03 + (h - 0.1) / 2, zc], dark, 'wood', radius=0.004)]       # inside, below the soil
    for px in (x - w / 2 + 0.025, x + w / 2 - 0.025):
        for pz in (z0 + 0.025, z1 - 0.025):
            p.append(box([0.05, h, 0.05], [px, y0 + h / 2, pz], dark, 'wood', radius=0.008))
    sh = h / 3
    for k in range(3):
        y = y0 + sh * (k + 0.5)
        p += [box([w - 0.02, sh - 0.012, 0.02], [x, y, z1 - 0.01], wood, 'wood', radius=0.004),
              box([w - 0.02, sh - 0.012, 0.02], [x, y, z0 + 0.01], wood, 'wood', radius=0.004),
              box([0.02, sh - 0.012, d - 0.02], [x - w / 2 + 0.01, y, zc], wood, 'wood', radius=0.004),
              box([0.02, sh - 0.012, d - 0.02], [x + w / 2 - 0.01, y, zc], wood, 'wood', radius=0.004)]
    p += [box([w + 0.02, 0.02, 0.05], [x, y0 + h + 0.01, z1 - 0.02], dark, 'wood', radius=0.005),               # rim
          box([w + 0.02, 0.02, 0.05], [x, y0 + h + 0.01, z0 + 0.02], dark, 'wood', radius=0.005)]
    return p


# =============================================================================================
# Electronics
# =============================================================================================
def crt_tv_on_table(wood='#B07A4E', casing='#6B4A33', bezel='#3A3633', screen='#22313A', top_y=0.55):
    """A wood-veneer CRT on a small two-shelf wooden table (back on the wall, front z ~0.06)."""
    z0, z1 = BACK + 0.015, 0.06
    zc = (z0 + z1) / 2
    w, d = 0.62, z1 - z0
    p = [box([w, 0.03, d], [0, top_y - 0.015, zc], wood, 'wood', radius=0.008),
         box([w - 0.05, 0.02, d - 0.05], [0, 0.12, zc], wood, 'wood', radius=0.005),
         box([w - 0.06, 0.05, 0.02], [0, top_y - 0.05, z1 - 0.03], wood, 'wood', radius=0.006)]
    for lx in (-w / 2 + 0.03, w / 2 - 0.03):
        for lz in (z0 + 0.03, z1 - 0.03):
            p.append(box([0.04, top_y - 0.03, 0.04], [lx, (top_y - 0.03) / 2, lz], wood, 'wood', radius=0.008))
    # things on the lower shelf: a VCR and tapes
    p += [box([0.4, 0.08, 0.3], [0, 0.17, zc + 0.02], '#4A4744', 'satin', radius=0.01),
          box([0.16, 0.012, 0.004], [-0.08, 0.18, zc + 0.172], '#1E1F22', 'gloss', radius=0.002),
          box([0.05, 0.016, 0.004], [0.12, 0.18, zc + 0.172], '#7FE0A0', 'gloss', radius=0.002),
          box([0.19, 0.11, 0.025], [-0.12, 0.27, zc + 0.04], '#2F3236', 'satin', radius=0.004, rotate=[0, 0, -8]),
          box([0.19, 0.025, 0.11], [0.14, 0.225, zc + 0.04], '#B5653F', 'satin', radius=0.004)]
    # the telly: wooden box, dark bezel, bulging screen, control column, rabbit ears
    ty, th = top_y + 0.19, 0.36
    tz = zc - 0.01
    p += [box([0.58, th, 0.4], [0, ty, tz], casing, 'wood', radius=0.03),
          box([0.4, 0.3, 0.02], [-0.07, ty, tz + 0.2], bezel, 'satin', radius=0.03),
          sph([0.36, 0.26, 0.05], [-0.07, ty, tz + 0.21], screen, 'gloss', segments=16),
          box([0.1, 0.3, 0.012], [0.21, ty, tz + 0.202], '#B9B4AA', 'satin', radius=0.01)]
    for k in range(3):
        p.append(cyl([0.045, 0.025, 0.045], [0.21, ty + 0.09 - k * 0.075, tz + 0.215], '#2F3236', 'satin', rotate=[90, 0, 0], segments=12))
    p += [box([0.06, 0.03, 0.004], [0.21, ty - 0.13, tz + 0.21], '#3A3D41', 'satin', radius=0.004),
          cyl([0.06, 0.02, 0.06], [0.0, ty + th / 2 + 0.01, tz - 0.08], '#2F3236', 'satin', segments=12),
          rod([0.0, ty + th / 2 + 0.02, tz - 0.08], [-0.16, ty + th / 2 + 0.36, tz - 0.12], 0.008, '#B9BDC1', 'metal'),
          rod([0.0, ty + th / 2 + 0.02, tz - 0.08], [0.18, ty + th / 2 + 0.33, tz - 0.12], 0.008, '#B9BDC1', 'metal'),
          sph([0.016, 0.016, 0.016], [-0.16, ty + th / 2 + 0.36, tz - 0.12], '#B9BDC1', 'metal', segments=6),
          sph([0.016, 0.016, 0.016], [0.18, ty + th / 2 + 0.33, tz - 0.12], '#B9BDC1', 'metal', segments=6)]
    return p


def flat_tv(y0=0.58, z0=-0.43, w=0.8, h=0.53, frame='#D5D6D4', screen='#4FC3A1'):
    """A slim TV on a centre foot standing on y0, its back at z0; the screen shows a game in progress."""
    ph = h - 0.07
    py = y0 + 0.07 + ph / 2
    zc = z0 + 0.08
    p = [box([0.3, 0.014, 0.14], [0, y0 + 0.007, zc], frame, 'satin', radius=0.006),
         box([0.06, 0.07, 0.03], [0, y0 + 0.045, zc - 0.03], frame, 'satin', radius=0.01),
         box([w, ph, 0.04], [0, py, zc - 0.01], frame, 'satin', radius=0.012),
         box([w - 0.03, ph - 0.03, 0.004], [0, py + 0.003, zc + 0.011], screen, 'gloss', radius=0.006),
         box([w - 0.03, 0.1, 0.003], [0, py - ph / 2 + 0.07, zc + 0.0135], '#3FA88A', 'gloss', radius=0.004),      # floor band
         sph([0.07, 0.07, 0.006], [-0.12, py + 0.06, zc + 0.014], '#F4F1EA', 'gloss', segments=10),                 # coach (head)
         box([0.06, 0.13, 0.006], [-0.12, py - 0.04, zc + 0.014], '#F2A541', 'gloss', radius=0.02),
         box([0.2, 0.04, 0.003], [0.2, py + 0.14, zc + 0.014], '#F4F1EA', 'gloss', radius=0.01),                    # score bars
         box([0.14, 0.03, 0.003], [0.17, py + 0.09, zc + 0.014], '#E9C46A', 'gloss', radius=0.01)]
    return p


def speaker_cabinet(w=0.5, h=0.86, d=0.415, x=0.0, body='#8A6BC0', trim='#54456E', grille='#3B2F4E'):
    """A party speaker: tall cabinet, big woofer, tweeter, port, trim band, handles, on small feet."""
    z0 = BACK
    zf = z0 + d
    p = [box([w, h - 0.03, d], [x, 0.03 + (h - 0.03) / 2, z0 + d / 2], body, 'gloss', radius=0.04),
         box([w + 0.004, 0.04, d], [x, h - 0.12, z0 + d / 2 + 0.002], trim, 'satin', radius=0.012)]
    for fx in (x - w / 2 + 0.06, x + w / 2 - 0.06):
        for fz in (z0 + 0.06, zf - 0.06):
            p.append(cyl([0.05, 0.03, 0.05], [fx, 0.015, fz], trim, 'satin', segments=10))
    wy = 0.36
    p += [torus([0.38, 0.03, 0.38], [x, wy, zf + 0.005], trim, 'satin', rotate=[90, 0, 0], segments=28),
          cone([0.34, 0.06, 0.34], [x, wy, zf - 0.02], grille, 'satin', taper=0.3, rotate=[90, 0, 0], segments=28),
          sph([0.1, 0.1, 0.05], [x, wy, zf + 0.0], '#C9CCD0', 'metal', segments=12),
          torus([0.16, 0.02, 0.16], [x, 0.66, zf + 0.004], trim, 'satin', rotate=[90, 0, 0], segments=20),
          sph([0.11, 0.11, 0.05], [x, 0.66, zf - 0.004], '#C9CCD0', 'metal', segments=12),
          box([0.22, 0.04, 0.012], [x, 0.11, zf + 0.002], '#1E1A26', 'satin', radius=0.02),                            # bass port
          box([0.3, 0.012, 0.004], [x, 0.15, zf + 0.004], '#F2C14E', 'gloss', radius=0.004)]                           # light strip
    for s in (-1, 1):
        p.append(box([0.012, 0.05, 0.14], [x + s * (w / 2 + 0.004), h - 0.25, z0 + d / 2], trim, 'satin', radius=0.006))
    return p


# =============================================================================================
# Plants
# =============================================================================================
def potted_arrowhead(pot='#C46A43', pot_rim='#CF7A50', y_pot=0.34, d_pot=0.4):
    """A terracotta pot with a lush arrowhead plant: three rings of leaves on stalks, the low ones drooping
    over the rim, the high ones held up."""
    green = ['#4F8A3E', '#5E9A48', '#6FAA52', '#7DB85C', '#8CC266']
    p = [lathe([d_pot, y_pot, d_pot], [0, y_pot / 2, 0], pot, 'satin', pot_profile(rim=0.08, foot=0.34), segments=24),
         torus([d_pot * 1.02, 0.03, d_pot * 1.02], [0, y_pot - 0.03, 0], pot_rim, 'satin', segments=24),
         cyl([d_pot * 0.82, 0.01, d_pot * 0.82], [0, y_pot - 0.045, 0], SOIL, 'matte', segments=20)]
    ys = y_pot - 0.04
    rings = [  # count, heading offset, stalk length, stalk tilt, leaf length, leaf width, leaf tilt, droop
        (8, 0, 0.15, 35, 0.21, 0.16, 5, 35), (8, 22, 0.27, 60, 0.23, 0.17, 30, 20), (6, 10, 0.36, 76, 0.22, 0.16, 50, 0)]
    k = 0
    for n, off, sl, st, ll, lw, lt, dr in rings:
        for i in range(n):
            a = off + i * 360 / n + (k * 37) % 17
            ar, tr = math.radians(a), math.radians(st + (k % 3) * 4)
            base = [math.sin(ar) * 0.03, ys, math.cos(ar) * 0.03]
            tip = [base[0] + math.sin(ar) * math.cos(tr) * sl, ys + math.sin(tr) * sl, base[2] + math.cos(ar) * math.cos(tr) * sl]
            p.append(rod(base, tip, 0.012, '#7FA35A', 'foliage', segments=6))
            p += leaf(tip, a, ll, lw, lt, green[k % 5], droop=dr, t=0.016)
            k += 1
    return p


def potted_ivy(pot='#B9673F', band='#9C5434', y_pot=0.32, d_pot=0.44):
    """A glazed terracotta pot with a bushy golden pothos: a mound of leaves, vines trailing over the rim."""
    green = ['#4F8A3E', '#6FAA52', '#8CBB5A', '#A7C76A', '#5E9A48']
    y0 = 0.06
    p = [lathe([d_pot, y_pot, d_pot], [0, y0 + y_pot / 2, 0], pot, 'gloss',
               [[0, -0.5], [0.38, -0.5], [0.46, -0.1], [0.5, 0.35], [0.47, 0.5], [0.43, 0.5], [0.4, 0.38], [0, 0.38]], segments=24),
         torus([d_pot * 0.97, 0.025, d_pot * 0.97], [0, y0 + y_pot * 0.38, 0], band, 'gloss', segments=24),
         cyl([d_pot * 0.82, 0.01, d_pot * 0.82], [0, y0 + y_pot - 0.02, 0], SOIL, 'matte', segments=20)]
    top = y0 + y_pot
    p += [part('blob', [0.4, 0.24, 0.4], [0, top + 0.1, 0], '#4A7A3D', 'foliage', noise=0.25)]                # dense heart
    for k in range(38):                                                                        # leaves scattered on a low dome
        u = (k + 0.5) / 38
        el = math.degrees(math.asin(u)) * 0.95                                                 # elevation of the leaf spot
        a = k * 137.5
        ar, er = math.radians(a), math.radians(el)
        c = [math.sin(ar) * math.cos(er) * 0.27, top + 0.04 + math.sin(er) * 0.3, math.cos(ar) * math.cos(er) * 0.27]
        tilt = el * 0.8 + 10 * ((k * 7) % 3 - 1)
        p.append(sph([0.13, 0.018, 0.15], c, green[k % 5], 'foliage', segments=8, rotate=[-tilt, a + 15 * ((k * 5) % 3 - 1), 0]))
    for k in range(6):                                                                         # trailing vines with leaves
        a = math.radians(15 + k * 60)
        for j in range(5):
            r, y = d_pot / 2 + 0.015 + 0.022 * j, top - 0.01 - 0.062 * j
            aa = a + 0.14 * j * (1 if k % 2 else -1)
            p.append(sph([0.09, 0.016, 0.075], [r * math.sin(aa), y, r * math.cos(aa)], green[(k + j) % 5], 'foliage',
                         segments=8, rotate=[-65, math.degrees(aa), 0]))
    return p


def potted_aloe(pot='#F2F0EA', h=0.27):
    """A small white pot with an aloe: pointed leaves fanning up from the middle."""
    p = [lathe([0.13, 0.11, 0.13], [0, 0.055, 0], pot, 'satin', pot_profile(rim=0.05, foot=0.4), segments=16),
         cyl([0.11, 0.008, 0.11], [0, 0.1, 0], SOIL, 'matte', segments=12)]
    for k in range(9):
        a = k * 40
        tilt = 70 - (k % 3) * 14
        ln = h - 0.1 - (k % 3) * 0.02
        ar, tr = math.radians(a), math.radians(tilt)
        base = [math.sin(ar) * 0.012, 0.1, math.cos(ar) * 0.012]
        tip = [base[0] + math.sin(ar) * math.cos(tr) * ln, 0.1 + math.sin(tr) * ln, base[2] + math.cos(ar) * math.cos(tr) * ln]
        p.append(rod(base, tip, 0.03, ['#6F9A6A', '#7FA877', '#5E8A5C'][k % 3], 'foliage', shape='cone', t2=0.014, segments=6))
    return p


def fern(x, y, z, pot='#C7744F', d=0.16, h=0.14, spread=0.26, green=('#4F8A45', '#5E9A4E', '#6FA85A')):
    """A potted fern standing on y: arching fronds (two segments each) round a terracotta pot."""
    p = [lathe([d, h, d], [x, y + h / 2, z], pot, 'satin', pot_profile(rim=0.07), segments=16),
         cyl([d * 0.84, 0.008, d * 0.84], [x, y + h - 0.012, z], SOIL, 'matte', segments=12)]
    for k in range(9):
        a = k * 40 + (k % 2) * 12
        tilt = 55 if k % 3 else 70
        p += leaf([x, y + h - 0.01, z], a, spread * (0.9 + 0.2 * (k % 2)), 0.075, tilt, green[k % 3], droop=75, t=0.014)
    return p


def pendant_globe(drop=0.95, d=0.43, globe='#F2F1EC', metal=CHROME):
    """A globe pendant: ceiling rose and cord at the top (y = drop), opal glass globe below."""
    gy = d / 2
    p = [cyl([0.12, 0.03, 0.12], [0, drop - 0.015, 0], metal, 'chrome', segments=16),
         cyl([0.008, drop - d - 0.06, 0.008], [0, (drop - 0.03 + d + 0.03) / 2, 0], '#3A3D41', 'satin', segments=6),
         cyl([0.07, 0.05, 0.07], [0, d + 0.01, 0], metal, 'chrome', segments=14),
         sph([d, d, d], [0, gy, 0], globe, 'gloss', segments=20),
         sph([0.1, 0.1, 0.1], [0, gy, 0], '#FFF1CF', 'gloss', segments=8)]
    return p


# =============================================================================================
# Tableware and bottles
# =============================================================================================
TEAPOT = [[0, -0.5], [0.36, -0.5], [0.48, -0.25], [0.5, 0.0], [0.44, 0.25], [0.3, 0.42], [0.18, 0.5], [0, 0.5]]
CUP = [[0, -0.5], [0.32, -0.5], [0.5, 0.45], [0.47, 0.5], [0.4, 0.4], [0, 0.0]]


def teapot(x, y, z, d=0.16, h=0.13, body=PORCELAIN, accent='#4E6FA8', heading=90):
    """A round teapot standing on y: body, lid and knob, spout and handle along the heading (deg, 90 = +x)."""
    a = math.radians(heading)
    ux, uz = math.sin(a), math.cos(a)
    cy = y + h / 2
    return [lathe([d, h, d], [x, cy, z], body, 'gloss', TEAPOT, segments=18),
            torus([d * 0.98, 0.012, d * 0.98], [x, y + h * 0.42, z], accent, 'gloss', segments=18),
            cyl([d * 0.4, 0.015, d * 0.4], [x, y + h + 0.005, z], body, 'gloss', segments=14),
            sph([0.03, 0.025, 0.03], [x, y + h + 0.022, z], accent, 'gloss', segments=8),
            rod([x + ux * d * 0.4, y + h * 0.35, z + uz * d * 0.4], [x + ux * d * 0.78, y + h * 0.82, z + uz * d * 0.78], 0.025, body, 'gloss',
                shape='cone', t2=0.015, segments=8),
            torus([h * 0.6, 0.018, h * 0.6], [x - ux * d * 0.5, cy + 0.005, z - uz * d * 0.5], body, 'gloss', rotate=[90, heading + 90, 0], segments=12)]


def teacup(x, y, z, d=0.07, body=PORCELAIN, accent='#4E6FA8', tea='#8A4E28', heading=90):
    a = math.radians(heading)
    return [cyl([d * 1.9, 0.01, d * 1.9], [x, y + 0.005, z], body, 'gloss', segments=16),                              # saucer
            torus([d * 1.85, 0.008, d * 1.85], [x, y + 0.01, z], accent, 'gloss', segments=16),
            lathe([d, d * 0.75, d], [x, y + 0.01 + d * 0.375, z], body, 'gloss', CUP, segments=14),
            cyl([d * 0.84, 0.004, d * 0.84], [x, y + 0.01 + d * 0.62, z], tea, 'gloss', segments=12),
            torus([0.035, 0.008, 0.035], [x + math.sin(a) * d * 0.55, y + 0.01 + d * 0.4, z + math.cos(a) * d * 0.55], body, 'gloss',
                  rotate=[90, heading + 90, 0], segments=10)]


def tea_set(x, y, z, w=0.62, d=0.33, body=PORCELAIN, accent='#4E6FA8', tray=None):
    """Teapot, cups on saucers, milk jug and sugar bowl laid out in a w x d area centred on (x, z), on y."""
    p = []
    if tray:
        p += [box([w, 0.012, d], [x, y + 0.006, z], tray, 'wood', radius=0.006),
              box([w, 0.02, 0.012], [x, y + 0.016, z - d / 2 + 0.006], tray, 'wood', radius=0.004),
              box([w, 0.02, 0.012], [x, y + 0.016, z + d / 2 - 0.006], tray, 'wood', radius=0.004)]
        y += 0.012
    sx = w / 0.62
    p += teapot(x - 0.17 * sx, y, z - 0.02, body=body, accent=accent, heading=80)
    p += teacup(x + 0.02 * sx, y, z + 0.07, body=body, accent=accent, heading=120)
    p += teacup(x + 0.2 * sx, y, z - 0.06, body=body, accent=accent, heading=60)
    if w > 0.5:
        p += [lathe([0.07, 0.08, 0.07], [x + 0.03 * sx, y + 0.04, z - 0.09], body, 'gloss', [[0, -0.5], [0.4, -0.5], [0.5, 0.2], [0.4, 0.5], [0, 0.5]], segments=12),
              lathe([0.08, 0.06, 0.08], [x + 0.21 * sx, y + 0.03, z + 0.09], body, 'gloss', [[0, -0.5], [0.35, -0.5], [0.5, 0.1], [0.42, 0.5], [0, 0.5]], segments=12),
              sph([0.025, 0.02, 0.025], [x + 0.21 * sx, y + 0.065, z + 0.09], accent, 'gloss', segments=6)]
    return p


def wine_bottles(x0, y, z, colors=('#2E4A30', '#5A1F2A', '#3E5A2E', '#6B3A1E', '#2E3A4A'), h=0.3, gap=0.1):
    """A row of wine bottles standing on y, from x0 along +x."""
    p = []
    for i, c in enumerate(colors):
        x = x0 + i * gap
        hh = h * (1 - 0.06 * (i % 2))
        p += [lathe([0.075, hh, 0.075], [x, y + hh / 2, z], c, 'gloss',
                    [[0, -0.5], [0.5, -0.5], [0.5, 0.12], [0.36, 0.24], [0.15, 0.32], [0.14, 0.5], [0, 0.5]], segments=12),
              cyl([0.078, hh * 0.24, 0.078], [x, y + hh * 0.3, z], ['#F2EBD9', '#E9D9B8', '#F4F1EA'][i % 3], 'satin', segments=12),
              cyl([0.026, hh * 0.12, 0.026], [x, y + hh * 0.94, z], ['#7A2E2E', '#C9A24A', '#2F3236'][i % 3], 'satin', segments=8)]
    return p
