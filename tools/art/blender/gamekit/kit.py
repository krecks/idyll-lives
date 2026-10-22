# The kit: furniture parts more than one family makes, built once here so a better one improves every model that
# uses it. Each takes a GameFrame (`frame`, game axes: x, height, depth) and the family's material, and adds one mesh.
#
#   G.kit.tube(F, 'rail', [(0, 0.2, 0), (0, 0.8, 0), (0.3, 0.9, 0)], 0.012, M.chrome)
#   G.kit.cabriole(F, 'leg_fl', 0.31, 0.3, 0.18, M.walnut, 0.02, 0.02)
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


__all__ = ['cabriole', 'forms', 'tube']
