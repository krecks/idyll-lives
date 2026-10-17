# Far versions built by hand (`Model(far=G.far_of(build))`): the model's own build run again at low detail,
# so the far version has the same shape, colours and size as the model, standing where it stands. The build
# asks `G.low(full, reduced)` for each detail (segments, rings, counts, bevels) and `G.is_low()` for what
# to leave out (a flower's centre, the moving parts kept apart: a far version is all one still part).
#
# Better than `far=0.25` (decimating to a quarter) for small, separate parts (flowers, leaves, lumps, a bale's
# twine): decimating shrinks thin parts into spikes and pulls the lowest vertices off the floor. Draw every
# random number the full build draws, in the same order, so the far version's plants stand where the
# model's do (`low` chooses only how finely each is built).

# Whether a far version is being built: a list, so a family that imports it shares the one flag.
LOW = [False]


def low(full, reduced):
    """`full` detail, or `reduced` while a far version is being built."""
    return reduced if LOW[0] else full


def is_low():
    """Whether a far version is being built (`far_of`)."""
    return LOW[0]


def far_of(build):
    """A model's far version (`Model(far=G.far_of(build))`): the same build at low detail (`low`)."""
    def far():
        LOW[0] = True
        try:
            return build()
        finally:
            LOW[0] = False
    far.__name__ = f'{getattr(build, "__name__", "build")}_far'
    return far
