import math

import drawsvg as draw

FULL_WIDTH = 2480  # A4

WIDTH = FULL_WIDTH * 0.8

TALL_ASPECT = 3 / 4
WIDE_ASPECT = 9 / 16

MAIN_STROKE_WIDTH = 5
SECONDARY_STROKE_WIDTH = 3
ARROW_HEAD_FACTOR = 5
POINT_RAD = 15
NORMAL_LINE = {
    "fill": "none",
    "stroke": "black",
    "stroke_width": MAIN_STROKE_WIDTH,
}
LINES = [
    NORMAL_LINE,
    {
        "fill": "none",
        "stroke": "black",
        "stroke_width": SECONDARY_STROKE_WIDTH,
        "stroke_dasharray": "20 10",
    },
    {
        "fill": "none",
        "stroke": "black",
        "stroke_width": SECONDARY_STROKE_WIDTH / 3,
        "stroke_dasharray": "none",
    },
]


def add(lhs: tuple[float, float], rhs: tuple[float, float]) -> tuple[float, float]:
    return (lhs[0] + rhs[0], lhs[1] + rhs[1])


def sub(lhs: tuple[float, float], rhs: tuple[float, float]) -> tuple[float, float]:
    return (lhs[0] - rhs[0], lhs[1] - rhs[1])


def to_cartesian(r: float, theta: float) -> tuple[float, float]:
    return (r * math.cos(theta), r * math.sin(theta))


def make_regular_polygon(
    x: float, y: float, n: int, radius: float, theta_0: float = 0, fill="black"
) -> draw.Path:
    path = draw.Path(fill=fill)
    start = to_cartesian(radius, theta_0)
    path.M(x + start[0], y + start[1])
    for i in range(1, n):
        p = to_cartesian(radius, theta_0 + 2 * math.pi * (i / n))
        path.L(x + p[0], y + p[1])
    return path


def arrow(
    x_start: float,
    y_start: float,
    x_end: float,
    y_end: float,
    color="black",
    stroke_dasharray="none",
    width: float = MAIN_STROKE_WIDTH / 2,
) -> draw.Group:
    angle = math.atan2(
        y_end - y_start,
        x_end - x_start,
    )
    offset = to_cartesian(width * ARROW_HEAD_FACTOR, angle)

    group = draw.Group()

    path = draw.Path(
        fill="none", stroke=color, stroke_width=width, stroke_dasharray=stroke_dasharray
    )
    path.M(x_start, y_start)
    path.L(
        x_end - offset[0] * 3 / 2,
        y_end - offset[1] * 3 / 2,
    )
    group.append(path)

    triangle = make_regular_polygon(
        x_end - offset[0],
        y_end - offset[1],
        3,
        width * ARROW_HEAD_FACTOR,
        angle,
        fill=color,
    )
    group.append(triangle)
    return group


def point(x, y, *, fill=True):
    if fill:
        return draw.Ellipse(
            x,
            y,
            POINT_RAD,
            POINT_RAD,
            fill="black",
            stroke="black",
            stroke_width=SECONDARY_STROKE_WIDTH,
        )
    else:
        return draw.Ellipse(
            x,
            y,
            POINT_RAD,
            POINT_RAD,
            fill="white",
            stroke="black",
            stroke_width=SECONDARY_STROKE_WIDTH,
        )


def lerp(
    p_1: tuple[float, float], p_2: tuple[float, float], s: float
) -> tuple[float, float]:
    return (p_1[0] * (1 - s) + p_2[0] * s, p_1[1] * (1 - s) + p_2[1] * s)
