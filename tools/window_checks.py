"""Platform-independent checks shared by native window interaction runners."""
import math


def check_texture_resize(previous_metrics, previous_status, metrics, status):
    """Reject a resized OS window whose GPU feedback/graph targets stayed stale.

    The fixture adds no images, buffers or overlay. A change in pixel area must
    therefore change the owned texture allocation in the same direction.
    """
    for sizes in (previous_metrics, metrics):
        if (len(sizes) != 12 or min(sizes[:4]) <= 0
                or not math.isfinite(sizes[4]) or sizes[4] <= 0):
            raise RuntimeError("invalid window dimensions or scale")
    for state in (previous_status, status):
        if len(state) < 18 or state[5:8] != [1, 0, 0] or state[14] != 0:
            raise RuntimeError("unexpected resources/overlay during resize check")
        if not math.isfinite(state[13]) or state[13] <= 0:
            raise RuntimeError("invalid owned texture allocation")
        if any(state[index] != 0 for index in (4, 9, 10, 11, 12, 15, 17)) or state[16] != 1:
            raise RuntimeError("unexpected scheduling, recovery, compilation or diagnostic state")
    area_change = metrics[2] * metrics[3] - previous_metrics[2] * previous_metrics[3]
    byte_change = status[13] - previous_status[13]
    if ((area_change != 0 and area_change * byte_change <= 0)
            or (area_change == 0 and byte_change != 0)):
        raise RuntimeError("GPU texture allocation did not follow the window's pixel area")
