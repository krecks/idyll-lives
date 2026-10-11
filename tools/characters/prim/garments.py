"""
Garment vocabulary shared by the body's fields and the clothes (clothes.py): the shell categories
and flags written into a shell's fields, the sleeve parameter's scale, and the landmarks (heights
on the base body) the cuts are made from.

Shell fields: (hem distance (m), flag, category, 0); categories 0 top, 1 bottom, 2 shoes; flags
1 sole (drawn darker), 2 waistband, 3 skirt.
"""

import numpy as np

# Categories (fields.z of garment vertices; the runtime colours them with the top / bottom / shoe colour).
TOP, BOTTOM, SHOES = 0, 1, 2
# Part flags (fields.y): 1 = sole / welt, 2 = waistband, 3 = skirt.
FLAG_SOLE, FLAG_BAND, FLAG_SKIRT = 1, 2, 3
# The sleeve parameter is scaled to metres in the predicates (about an arm's length).
ARM_M = 0.55


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


class Landmarks:
    """Heights (m) on a body's rest pose: pelvis, hips, knees, ankles, the hem and the waistband."""

    def __init__(self, P, gender):
        self.gender = gender
        self.pelvis = P['pelvis'][1]
        self.neck = P['neck_01']
        self.hip, self.knee, self.ankle = P['thigh_l'][1], P['calf_l'][1], P['foot_l'][1]
        self.hem = self.pelvis + (0.02 if gender == 'male' else 0.045)
        self.waist = self.pelvis + (0.075 if gender == 'male' else 0.09)
