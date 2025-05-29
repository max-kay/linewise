import drawsvg as draw

from lib import NORMAL_LINE, SECONDARY_LINE, TALL_ASPECT, WIDTH, lerp, point


def linear() -> draw.Drawing:
    param = 0.3
    height = WIDTH * 0.25
    img = draw.Drawing(WIDTH, height)
    points = [(WIDTH * 0.15, height * 0.92), (WIDTH * 0.85, height * 0.08)]
    point_2 = lerp(points[0], points[1], param)
    img.append(draw.Line(*points[0], *points[1], **SECONDARY_LINE))
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

    path = draw.Path(**SECONDARY_LINE)
    path.M(*points[0])
    path.Q(*points[1], *points[2])
    img.append(path)
    img.append(draw.Line(*points[0], *points[1], **SECONDARY_LINE))
    img.append(draw.Line(*points[1], *points[2], **SECONDARY_LINE))

    p1 = lerp(points[0], points[1], param)
    p2 = lerp(points[1], points[2], param)
    p3 = lerp(p1, p2, param)
    new_points = [points[0], p1, p3]

    img.append(draw.Line(*p1, *p2, **SECONDARY_LINE))

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
    path = draw.Path(**SECONDARY_LINE)
    path.M(*points[0])
    path.C(*points[1], *points[2], *points[3])
    img.append(path)
    img.append(draw.Line(*points[0], *points[1], **SECONDARY_LINE))
    img.append(draw.Line(*points[1], *points[2], **SECONDARY_LINE))
    img.append(draw.Line(*points[2], *points[3], **SECONDARY_LINE))

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
        img.append(draw.Line(*n_points[i], *n_points[i + 1], **SECONDARY_LINE))

    nn_points = [
        lerp(n_points[0], n_points[1], param),
        lerp(n_points[1], n_points[2], param),
    ]

    for p in nn_points:
        dots.append(point(*p, fill=False))
    for i in range(len(nn_points) - 1):
        img.append(draw.Line(*nn_points[i], *nn_points[i + 1], **SECONDARY_LINE))

    nnn = lerp(nn_points[0], nn_points[1], param)

    dots.append(point(*nnn))

    for p in dots:
        img.append(p)
    path = draw.Path(**NORMAL_LINE)
    path.M(*points[0])
    path.C(*n_points[0], *nn_points[0], *nnn)
    img.append(path)
    return img
