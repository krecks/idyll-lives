# The kit: furniture parts more than one family makes, built once here so a better one improves every model that
# uses it. Each takes a GameFrame (`frame`, game axes: x, height, depth) and the family's material, and adds its mesh.
#
#   G.kit.tube(F, 'rail', [(0, 0.2, 0), (0, 0.8, 0), (0.3, 0.9, 0)], 0.012, M.chrome)
#   G.kit.cabriole(F, 'leg_fl', 0.31, 0.3, 0.18, M.walnut, 0.02, 0.02)
#   G.kit.pull(F, 'pull', 0, 0.45, 0.02, 0.12, M.brass)          # a bar pull on a front at depth 0.02
#   G.kit.shaker(F, 'door', 0.6, 1.8, (0, 1, 0.15), 0.02, M.white)  # a frame-and-panel door
#   G.kit.songbird(F, 'robin', 0.3, 1.47, -0.1, M.brown, M.red, M.beak, M.eye, turn=-20)   # perched, feet at the point
#   G.kit.moulding(F, 'frame', [(0.25, 1.0), (0.25, 1.7), (-0.25, 1.7), (-0.25, 1.0)], [(0, 0), (0, 0.03), (0.03, 0.03), (0.03, 0)], M.gilt, at=-0.43)
#   G.kit.scroll(F, 'curl', (0, 0.66, 0.2), (0.62, 0.2), M.iron, turns=-1.1)  # a scroll in the depth-height plane
#   G.kit.bench_end(F, 'end', 0.8, 0.415, 0.19, -0.21, (0.885, -0.27), M.iron, arm=0.68)  # a cast-iron bench end
import math

from mathutils import Vector

from . import forms, shapes


def tube(frame, name, points, r, material, verts=8, round=3, scale=1.0):
    """A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of
    points; 1 keeps them sharp), its ends capped: a towel rail, a bent-tube frame, a tap's riser, a hose. Replaces
    the rods joined by spheres the families made one by one."""
    return frame.sweep(name, points, r, material, verts=verts, resolution=round, scale=scale)


def cabriole(frame, name, x, z, h, material, sx, sz, r=0.024, verts=6):
    """A cabriole leg from the frame at height `h` over (x, z) down to the floor: out by (sx, sz) at the knee,
    slim at the ankle, flaring into a pad foot that stands on the floor (its lowest point at height 0)."""
    return frame.sweep(name, [(x, h, z), (x + sx, h * 0.62, z + sz), (x + sx * 0.4, 0.06, z + sz * 0.4),
                              (x + sx * 0.55, 0.02, z + sz * 0.55), (x + sx * 0.55, 0.0, z + sz * 0.55)],
                       r, material, verts=verts, resolution=2, scale=[1.0, 1.05, 0.7, 1.1, 1.15])


def fillet(points, radius, steps=3):
    """The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
    it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
    one). For handles, bent rails and draped cloth swept with `resolution` 1."""
    pts = [Vector(p) for p in points]
    out = [pts[0]]
    for a, p, c in zip(pts, pts[1:], pts[2:]):
        d1, d2 = a - p, c - p
        if d1.length < 1e-9 or d2.length < 1e-9 or d1.angle(d2) > 3.1415:
            out.append(p)
            continue
        t = min(radius / max(1e-6, math.tan(d1.angle(d2) / 2)), d1.length * 0.5, d2.length * 0.5)
        p1, p2 = p + d1.normalized() * t, p + d2.normalized() * t
        for i in range(steps + 1):
            k = i / steps
            out.append(p1 * (1 - k) ** 2 + p * 2 * k * (1 - k) + p2 * k * k)
    out.append(pts[-1])
    return [tuple(p) for p in out]


def pull(frame, name, x, y, face, length, material, r=0.006, out=0.03, upright=False, verts=8):
    """A bar pull in one piece: out from a front at game depth `face` on two round stand-offs `length` apart
    (across it, or `upright`), its bends rounded: a drawer's or a door's handle."""
    h, b = length / 2, min(0.012, out * 0.45)
    ends = [(-h, face - 0.003), (-h, face + out), (h, face + out), (h, face - 0.003)]
    pts = [(x, y + u, z) if upright else (x + u, y, z) for u, z in ends]
    return frame.sweep(name, fillet(pts, b), r, material, verts=verts)


def knob(frame, name, x, y, face, r, material, segments=12):
    """A turned knob on a front at game depth `face`: a rose against it, a waisted neck and a domed head `r` round."""
    prof = [(0, 0), (r * 0.75, 0), (r * 0.7, r * 0.15), (r * 0.36, r * 0.35), (r * 0.42, r * 0.75), (r * 0.95, r * 1.05),
            (r, r * 1.35), (r * 0.8, r * 1.7), (r * 0.4, r * 1.88), (0, r * 1.92)]
    return shapes.revolve(name, prof, at=frame.at(x, y, face), material=material, segments=segments, rot=(90, 0, 0))


def shaker(frame, name, w, h, at, t, material, rail=0.06, panel=None, recess=0.005, raised=False):
    """A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
    drawer's front): a frame `rail` wide round a panel set back `recess`, flat, or `raised` (its edges chamfered
    up to a field); `panel`: the panel's own material. Returns (frame, panel)."""
    hole = forms.rounded_rect(w - 2 * rail, h - 2 * rail, 0.003, corner=1)
    fr = frame.extrude(name, forms.rounded_rect(w, h, 0.003, corner=1), t, at, material, holes=[hole],
                       bevel=min(0.004, t * 0.2), bevel_segments=1)
    # (a hair inside the opening, not into the frame: a vertex buried in it would bake black, `shade.py`)
    pw, ph = w - 2 * rail - 0.002, h - 2 * rail - 0.002
    if not raised:
        return fr, frame.box(f'{name}_panel', pw, ph, t - 2 * recess, at, panel or material, bevel=0.002, bevel_segments=1)
    # a raised panel: a thin edge inside the frame, rising over a wide shallow chamfer to a flat field
    x, y, z = at
    b = min(rail * 0.7, pw * 0.25, ph * 0.25)

    def loop(hw, hh, d):
        return [frame.at(x + u, y + v, d) for u, v in ((hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh))]
    front = z + t / 2
    pn = forms.loft(f'{name}_panel', [loop(pw / 2, ph / 2, z - t / 2 + 0.002), loop(pw / 2, ph / 2, front - recess - 0.004),
                                      loop(pw / 2 - b, ph / 2 - b, front - recess * 0.2)],
                    material=panel or material, smooth=False)
    return fr, pn


def moulding(frame, name, outline, profile, material, at=0.0, plane='front', angle=forms.SHARP):
    """A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
    plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` ('front'), or (x, depth)
    points at height `at` ('top'); `profile` is the moulding's closed cross-section [(inset, out), ...]: `inset`
    metres in from the outline, `out` metres forward of `at` ('front') or up from it ('top'). The corners stay
    square however wide the profile (a sweep pinches them)."""
    n = len(outline)
    area = sum(outline[i][0] * outline[(i + 1) % n][1] - outline[(i + 1) % n][0] * outline[i][1] for i in range(n))
    turn = 1 if area > 0 else -1             # inward is to the left of an anticlockwise outline

    def inward(a, b):
        dx, dy = b[0] - a[0], b[1] - a[1]
        length = (dx * dx + dy * dy) ** 0.5 or 1.0
        return (-dy / length * turn, dx / length * turn)
    vs, faces = [], []
    m = len(profile)
    for i in range(n):
        p = outline[i]
        n0, n1 = inward(outline[i - 1], p), inward(p, outline[(i + 1) % n])
        k = 1 + n0[0] * n1[0] + n0[1] * n1[1]
        mx, my = (n0[0] + n1[0]) / k, (n0[1] + n1[1]) / k
        for inset, out in profile:
            u, v = p[0] + inset * mx, p[1] + inset * my
            vs.append(frame.at(u, v, at + out) if plane == 'front' else frame.at(u, at + out, v))
    for i in range(n):
        a, b = i * m, ((i + 1) % n) * m
        for j in range(m):
            j2 = (j + 1) % m
            faces.append((a + j, a + j2, b + j2, b + j))
    return forms._object(name, vs, faces, material, True, angle)


def songbird(frame, name, x, y, z, body, breast, beak, eye=None, turn=0.0, size=1.0, head=0.0, low=False):
    """A small songbird (a robin, a tit, a sparrow) perched with its feet at game (x, y, z), facing the front turned
    `turn` degrees towards +x: a lofted body tapering into a fanned tail, a `breast` patch, a pointed `beak`, `eye`s,
    thin legs down to the perch. `size` 1 is a robin (14 cm long); `head` tips the head down (degrees: drinking,
    pecking); `low` takes fewer sides and no eyes (a far version). About 350 triangles. Returns its pieces."""
    c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    dip = math.radians(head)

    def at(u, v, w):
        """The game point `u` across (to the bird's right), `v` forward and `w` up from its feet (at size 1)."""
        if v > 0.03:                                       # the head and beak tip down about the neck
            dv, dw = v - 0.03, w - 0.048
            v, w = 0.03 + dv * math.cos(dip) + dw * math.sin(dip), 0.048 - dv * math.sin(dip) + dw * math.cos(dip)
        u, v, w = u * size, v * size, w * size
        return frame.at(x + v * s + u * c, y + w, z + v * c - u * s)
    # (forward, half width, half height, centre height): from the tail's root to the face
    rows = [(-0.05, 0.010, 0.010, 0.040), (-0.035, 0.022, 0.024, 0.036), (-0.012, 0.029, 0.031, 0.034),
            (0.012, 0.027, 0.030, 0.038), (0.03, 0.02, 0.022, 0.048), (0.045, 0.02, 0.02, 0.058),
            (0.06, 0.016, 0.015, 0.06), (0.07, 0.007, 0.007, 0.059)]
    n = 6 if low else 10
    sections = [[at(hw * math.cos(math.tau * k / n), v, wc + hh * math.sin(math.tau * k / n)) for k in range(n)]
                for v, hw, hh, wc in rows]
    made = [forms.loft(name, sections, material=body)]
    made.append(forms.sweep(f'{name}_beak', [at(0, 0.066, 0.058), at(0, 0.088, 0.057)], 0.0055 * size, material=beak,
                            verts=5 if low else 6, scale=[1.0, 0.15]))
    breast_at = Vector(at(0, 0.01, 0.034))
    made.append(shapes.sphere(f'{name}_breast', 1.0, at=tuple(breast_at), material=breast, rot=(0, 0, -turn),
                              scale=(0.026 * size, 0.027 * size, 0.025 * size), segments=8 if low else 10,
                              rings=5 if low else 6))
    # the tail: a thin fan, cocked a little up
    made.append(forms.sweep(f'{name}_tail', [at(0, -0.04, 0.041), at(0, -0.088, 0.054)], None, material=body,
                            profile=[(-0.0018, -0.008), (0.0018, -0.008), (0.0018, 0.008), (-0.0018, 0.008)],
                            up=(0, 0, 1), scale=[0.9 * size, 1.8 * size]))
    if eye is not None and not low:
        for k in (-1, 1):
            made.append(shapes.sphere(f'{name}_eye{k}', 0.0042 * size, at=at(k * 0.0148, 0.055, 0.062), material=eye,
                                      segments=6, rings=4))
    for k in (-1, 1):
        made.append(forms.sweep(f'{name}_leg{k}', [at(k * 0.008, 0.0, 0.026), at(k * 0.009, 0.004, 0.0),
                                                   at(k * 0.009, 0.016, 0.0)], 0.0016 * size, material=beak, verts=4))
    return made


def scroll(frame, name, start, centre, material, turns=1.1, end=0.25, t=0.02, w=0.03, steps=10, profile=None):
    """A flat iron scroll in the depth-height plane at the game x of `start`: from `start` (x, height, depth) it
    curls round `centre` (height, depth) by `turns`, its radius closing to `end` of where it began, tapering as it
    goes: a bench arm's or a bracket's end."""
    x, y0, z0 = start
    cy, cz = centre
    r0 = math.hypot(y0 - cy, z0 - cz)
    a0 = math.atan2(y0 - cy, z0 - cz)
    sign = 1 if turns > 0 else -1
    pts = []
    for i in range(steps + 1):
        k = i / steps
        r = r0 * (1 - (1 - end) * k)
        a = a0 + sign * math.tau * abs(turns) * k
        pts.append((x, cy + r * math.sin(a), cz + r * math.cos(a)))
    prof = profile or forms.rounded_rect(t, w, min(t, w) * 0.3, corner=1)
    return frame.sweep(name, pts, None, material, profile=prof, up=(1, 0, 0), scale=lambda f: 1 - 0.45 * f)


def bench_end(frame, name, x, seat, front, back, top, material, arm=None, t=0.026, w=0.045, foot=0.06, steps=3,
              corner=1, scrolls=True):
    """A cast-iron bench end in one plane at game x, each member a flat bar `t` thick and `w` deep: the back leg
    rising in one curve into the upright, leaning back to `top` (height, depth); the front leg, straight up and, with
    an `arm` height, bent back into the armrest that meets the upright, its front curling down in a scroll; the seat
    rail between them at `seat` (its top, under the slats); pad feet `foot` long at `front` and `back` (their depths).
    `steps` rounds the curves (1 for a far version), `corner` the bars' edges (0: square), `scrolls` the curls."""
    prof = forms.rounded_rect(t, w, min(t, w) * 0.3 if corner else 0, corner=max(1, corner))
    ty, tz = top
    rail = seat - w / 2
    bz = back + 0.045                      # where the upright passes the seat rail
    made = []
    # the back leg into the upright: evenly spaced points, curved through (no corner to overshoot)
    pts = [(x, 0.0, back), (x, seat * 0.5, back + 0.03), (x, rail, bz),
           (x, rail + (ty - rail) * 0.5, (bz + tz) / 2 + 0.012), (x, ty, tz)]
    made.append(frame.sweep(f'{name}_back', pts, None, material, profile=prof, up=(1, 0, 0), resolution=steps))

    def upright_z(y):                      # the upright's depth at height y (above the seat), for the arm to meet
        f = (y - rail) / (ty - rail)
        return bz + (tz - bz) * f + 0.012 * math.sin(math.pi * f)
    fz = front - 0.015
    if arm is None:
        pts = [(x, 0.0, front), (x, rail, fz)]
        made.append(frame.sweep(f'{name}_front', pts, None, material, profile=prof, up=(1, 0, 0)))
    else:
        pts = fillet([(x, 0.0, front), (x, rail, fz), (x, arm - w / 2, fz - 0.005), (x, arm - w / 2, upright_z(arm) + w / 2)],
                     0.05, steps=max(1, steps))
        made.append(frame.sweep(f'{name}_front', pts, None, material, profile=prof, up=(1, 0, 0)))
        if scrolls:
            r = 0.038
            made.append(scroll(frame, f'{name}_scroll', (x, arm - w / 2, fz - 0.012), (arm - w / 2 - r, fz - 0.012),
                               material, turns=-1.2, t=t, w=w * 0.75, steps=4 * max(1, steps)))
    if scrolls:
        # a stretcher sagging between the legs, and a curl in the angle of the front leg and the seat rail
        made.append(frame.sweep(f'{name}_stretcher', [(x, 0.13, front - 0.01), (x, 0.1, (front + back) / 2),
                                                       (x, 0.13, back + 0.02)], None, material, profile=prof, up=(1, 0, 0),
                                resolution=steps, scale=0.75))
        made.append(scroll(frame, f'{name}_bracket', (x, rail - w * 0.6, fz - w * 0.4), (rail - w * 0.6 - 0.035, fz - 0.06),
                           material, turns=0.85, t=t, w=w * 0.6, steps=4 * max(1, steps)))
    made.append(frame.sweep(f'{name}_rail', [(x, rail, fz + 0.01), (x, rail, bz - 0.01)], None, material, profile=prof,
                            up=(1, 0, 0)))
    for z in (front, back):
        made.append(frame.box(f'{name}_foot{z}', t + 0.02, 0.02, foot, (x, 0.01, z), material,
                              bevel=0.006 if corner else 0.0, bevel_segments=1))
    return made


__all__ = ['bench_end', 'cabriole', 'fillet', 'forms', 'knob', 'moulding', 'pull', 'scroll', 'shaker', 'songbird', 'tube']
