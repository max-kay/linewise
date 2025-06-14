import drawsvg as draw

from lib import (
    LINES,
    NORMAL_LINE,
    TALL_ASPECT,
    WIDE_ASPECT,
    WIDTH,
    arrow,
    add,
    lerp,
    point,
    sub,
)


def linear() -> draw.Drawing:
    param = 0.3
    height = WIDTH * 0.25
    img = draw.Drawing(WIDTH, height)
    points = [(WIDTH * 0.15, height * 0.92), (WIDTH * 0.85, height * 0.08)]
    point_2 = lerp(points[0], points[1], param)
    img.append(draw.Line(*points[0], *points[1], **LINES[1]))
    img.append(draw.Line(*points[0], *point_2, **NORMAL_LINE))
    img.append(point(*points[0], fill=False))
    img.append(point(*points[1], fill=False))
    img.append(point(*point_2))
    return img


def quadratic() -> draw.Drawing:
    param = 0.35
    height = WIDTH * TALL_ASPECT
    img = draw.Drawing(WIDTH, height)
    points = [
        (WIDTH * 0.2, height * 0.8),
        (WIDTH * 0.45, height * 0.2),
        (WIDTH * 0.8, height * 0.8),
    ]

    path = draw.Path(**LINES[1])
    path.M(*points[0])
    path.Q(*points[1], *points[2])
    img.append(path)
    img.append(draw.Line(*points[0], *points[1], **LINES[2]))
    img.append(draw.Line(*points[1], *points[2], **LINES[2]))

    p1 = lerp(points[0], points[1], param)
    p2 = lerp(points[1], points[2], param)
    p3 = lerp(p1, p2, param)
    new_points = [points[0], p1, p3]

    img.append(draw.Line(*p1, *p2, **LINES[2]))

    path = draw.Path(**NORMAL_LINE)
    path.M(*new_points[0])
    path.Q(*new_points[1], *new_points[2])
    img.append(path)

    for p in points:
        img.append(point(*p, fill=False))
    img.append(point(*p1, fill=False))
    img.append(point(*p2, fill=False))
    img.append(point(*p3))
    return img


def cubic() -> draw.Drawing:
    param = 0.6
    height = WIDTH * TALL_ASPECT
    img = draw.Drawing(WIDTH, height)
    points = [
        (WIDTH * 0.2, height * 0.8),
        (WIDTH * 0.35, height * 0.2),
        (WIDTH * 0.7, height * 0.25),
        (WIDTH * 0.8, height * 0.8),
    ]
    dots = []
    path = draw.Path(**LINES[1])
    path.M(*points[0])
    path.C(*points[1], *points[2], *points[3])
    img.append(path)
    img.append(draw.Line(*points[0], *points[1], **LINES[2]))
    img.append(draw.Line(*points[1], *points[2], **LINES[2]))
    img.append(draw.Line(*points[2], *points[3], **LINES[2]))

    dots.append(point(*points[0], fill=False))
    dots.append(point(*points[1], fill=False))
    dots.append(point(*points[2], fill=False))
    dots.append(point(*points[3], fill=False))

    n_points = [
        lerp(points[0], points[1], param),
        lerp(points[1], points[2], param),
        lerp(points[2], points[3], param),
    ]

    for p in n_points:
        dots.append(point(*p, fill=False))
    for i in range(len(n_points) - 1):
        img.append(draw.Line(*n_points[i], *n_points[i + 1], **LINES[2]))

    nn_points = [
        lerp(n_points[0], n_points[1], param),
        lerp(n_points[1], n_points[2], param),
    ]

    for p in nn_points:
        dots.append(point(*p, fill=False))
    for i in range(len(nn_points) - 1):
        img.append(draw.Line(*nn_points[i], *nn_points[i + 1], **LINES[2]))

    nnn = lerp(nn_points[0], nn_points[1], param)

    dots.append(point(*nnn))

    for p in dots:
        img.append(p)
    path = draw.Path(**NORMAL_LINE)
    path.M(*points[0])
    path.C(*n_points[0], *nn_points[0], *nnn)
    img.append(path)
    return img


def lerp_reduce(
    points: list[tuple[float, float]], param: float
) -> list[tuple[float, float]]:
    return [lerp(points[i], points[i + 1], param) for i in range(len(points) - 1)]


def full_lerp_reduce(
    points: list[tuple[float, float]], param: float
) -> tuple[float, float]:
    new_points = points
    while len(new_points) > 1:
        new_points = lerp_reduce(new_points, param)
    return new_points[0]


def general() -> draw.Drawing:
    param = 0.3
    height = WIDTH * WIDE_ASPECT
    img = draw.Drawing(WIDTH, height)
    f_points = [
        (0.07, 0.85),
        (0.17, 0.29),
        (0.4, 0.38),
        (0.65, 0.79),
        (0.83, 0.57),
        (0.93, 0.14),
    ]
    c_points = [(p[0] * WIDTH, p[1] * height) for p in f_points]

    new_points = c_points
    lines = draw.Group()
    points = draw.Group()
    while len(new_points) > 1:
        for i in range(len(new_points) - 1):
            lines.append(draw.Line(*new_points[i], *new_points[i + 1], **LINES[2]))
        for p in new_points:
            points.append(point(*p, fill=False))
        new_points = lerp_reduce(new_points, param)
    points.append(point(*new_points[0]))

    param_steps = 3000
    full_path = draw.Path(**LINES[1])
    full_path.M(*c_points[0])
    until_path = draw.Path(**LINES[0])
    until_path.M(*c_points[0])
    for i in range(1, param_steps):
        s = i / (param_steps - 1)
        pos = full_lerp_reduce(c_points, s)
        full_path.L(*pos)
        if s <= param:
            until_path.L(*pos)

    img.append(full_path)
    img.append(until_path)
    img.append(lines)
    img.append(points)
    return img


def continous_bezier() -> draw.Drawing:
    height = WIDTH * WIDE_ASPECT
    img = draw.Drawing(WIDTH, height)
    pts = [
        (0.1, 0.9),
        (0.3, 0.32),
        (0.6, 0.45),
        (0.9, 0.9),
    ]
    pts = [(p[0] * WIDTH, p[1] * height) for p in pts]
    vecs = [
        (0.2, -0.2),
        (0.1, -0.08),
        (0.07, 0.1),
        (0.15, 0.1),
    ]
    vecs = [(p[0] * WIDTH, p[1] * height) for p in vecs]

    path = draw.Path(**NORMAL_LINE)
    points = draw.Group()
    points.append(point(*pts[0]))
    path.M(*pts[0])
    for i in range(len(pts) - 1):
        path.C(*add(pts[i], vecs[i]), *sub(pts[i + 1], vecs[i + 1]), *pts[i + 1])
        points.append(
            arrow(
                *pts[i],
                *add(pts[i], vecs[i]),
            )
        )
        points.append(
            arrow(
                *pts[i + 1],
                *sub(pts[i + 1], vecs[i + 1]),
                stroke_dasharray="20 10",
            )
        )
        points.append(point(*pts[i + 1]))
    img.append(path)
    img.append(points)
    return img
