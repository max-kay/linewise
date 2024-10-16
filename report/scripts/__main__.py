import math

import drawsvg as draw

WIDTH = 400
HEIGHT = 300
MAIN_STROKE_WIDTH = 2
ARROW_HEAD_FACTOR = 5

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
    dashes="none",
    width: float = MAIN_STROKE_WIDTH / 2,
) -> draw.Group:
    angle = math.atan2(
        y_end - y_start,
        x_end - x_start,
    )
    offset = to_cartesian(width * ARROW_HEAD_FACTOR, angle)

    group = draw.Group()

    path = draw.Path(
        fill="none", stroke=color, stroke_width=width, stroke_dasharray=dashes
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

def fig1():
    img = draw.Drawing(WIDTH, HEIGHT)
    points = [(WIDTH*0.2, HEIGHT*0.8), (WIDTH*0.35, HEIGHT* 0.2), (WIDTH * 0.7, HEIGHT*0.25), (WIDTH* 0.8, HEIGHT*0.8)]
    path = draw.Path()
    path.M(*points[0])
    path.C(*points[1], *points[2], *points[2])
    img.append(path)
    print()
    img.save_svg("figs/fig1.svg")


def main():
    fig1()

if __name__ == "__main__":
    main()
