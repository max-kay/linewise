import math

import drawsvg as draw

FIG_DIR = "./report/figs"

FULL_WIDTH = 2480 # A4

WIDTH = FULL_WIDTH*0.8

TALL_ASPECT = 3/4
WIDE_ASPECT = 9/16

MAIN_STROKE_WIDTH = 5
SECONDARY_STROKE_WIDTH = 3
ARROW_HEAD_FACTOR = 5
POINT_RAD = 15
NORMAL_LINE = {"fill": "none", "stroke": "black", "stroke_width": MAIN_STROKE_WIDTH}
SECONDARY_LINE = {"fill": "none", "stroke": "black", "stroke_width": SECONDARY_STROKE_WIDTH, "stroke_dasharray": "20 10"}

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

def point(x, y, *, fill=True, color="black"):
    if fill:
        return draw.Ellipse(x, y, POINT_RAD, POINT_RAD, fill=color, stroke=color, stroke_width = SECONDARY_STROKE_WIDTH)
    else:
        return draw.Ellipse(x, y, POINT_RAD, POINT_RAD, fill="none", stroke=color, stroke_width = SECONDARY_STROKE_WIDTH)

def fig1():
    img = draw.Drawing(WIDTH, HEIGHT)
    points = [(WIDTH*0.2, HEIGHT*0.8), (WIDTH*0.35, HEIGHT* 0.2), (WIDTH * 0.7, HEIGHT*0.25), (WIDTH* 0.8, HEIGHT*0.8)]
    path = draw.Path(**NORMAL_LINE)
    path.M(*points[0])
    path.C(*points[1], *points[2], *points[3])
    img.append(path)
    img.append(draw.Line(*points[0], *points[1], **SECONDARY_LINE))
    img.append(draw.Line(*points[2], *points[3], **SECONDARY_LINE))
    img.append(point(*points[0]))
    img.append(point(*points[1], fill=False))
    img.append(point(*points[2], fill=False))
    img.append(point(*points[3]))

    img.save_svg(f"{FIG_DIR}/fig1.svg")

def fig2():

    pass

def main():
    fig1()
    fig2()

if __name__ == "__main__":
    main()
