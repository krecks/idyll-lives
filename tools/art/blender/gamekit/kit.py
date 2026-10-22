# The kit: furniture parts more than one family makes, built once here so a better one improves every model that
# uses it. Each takes a GameFrame (`frame`, game axes: x, height, depth) and the family's material, and adds one mesh.
#
#   G.kit.tube(F, 'rail', [(0, 0.2, 0), (0, 0.8, 0), (0.3, 0.9, 0)], 0.012, M.chrome)
#   G.kit.cabriole(F, 'leg_fl', 0.31, 0.3, 0.18, M.walnut, 0.02, 0.02)
#   G.kit.moulding(F, 'frame', [(0.25, 1.0), (0.25, 1.7), (-0.25, 1.7), (-0.25, 1.0)], [(0, 0), (0, 0.03), (0.03, 0.03), (0.03, 0)], M.gilt, at=-0.43)
from . import forms


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


__all__ = ['cabriole', 'forms', 'moulding', 'tube']
