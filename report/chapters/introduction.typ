= Introduction

== Motivation


#pagebreak()

== B-Splines

B-Splines are a type of parametric curve.
In the following sections they are defined and equations for their paths and the derivative thereof is defined.

=== Bezier Curves

A Bezier curve of order 1 is the linear interpolation of starting point $arrow(r)_0$ and an end point $arrow(r)_1$.

$ arrow(r)(s) = (1 - s) arrow(r)_0 + s arrow(r)_1 quad s in [0, 1] $


#figure(
  image("../generated/linear.svg"),
  caption: [A Bezier curve of order 1 showing the interpolation of two points at $s=0.3$]
)<linear>

A Bezier curve of order 2 is the linear interpolation of two Bezier curves of order 1 with one ending
at the start of the other bezier spline:

$
arrow(r)(s) &= (1 - s)[(1 - s) arrow(r)_0 + s arrow(r)_1] + s [(1 - s) arrow(r)_1 + s arrow(r)_2] quad &s in [0, 1]\
arrow(r)(s) &= (1 - s)^2 arrow(r)_0 + 2s(1 - s) arrow(r)_1 + s^2 arrow(r)_2 quad &s in [0, 1]
$

#figure(
  image("../generated/quadratic.svg"),
  caption: [A Bezier curve of order 2 showing the construction of the point at $s=0.35$]
)
A Bezier curve of order 3 has four control points and is the interpolation of two order 2 Bezier curves:

$ arrow(r)(s) = (1 - s)^3 arrow(r)_0 + 3s(1 - s)^2 arrow(r)_1 + 3s^2(1 - s) arrow(r)_2 + s^3arrow(r)_3 quad s in [0, 1] $

#figure(
  image("../generated/cubic.svg"),
  caption: [A Bezier curve of order 3 showing the contruction of the point at $s=0.6$]
)

In general a Bezier curve of order $n$ is controled by $n+1$ points and the degree of the polynomial is $n$.
The path is described by:

$ arrow(r)(s) = sum_(i=0)^n vec(n, i)(1-s)^(n-i)s^(i)arrow(r)_i $<bezier_sum>

The polynomials in @bezier_sum are know as the Bernstein polynomials $b^n_i$ where $n$ is the degree of the polynomial.

$ b^n_i = vec(n, i)(1-s)^(n-i)s^(i) $

Using the Bernstein polynomial for the path a Bezier curve of degree $n$ can be written more compactly.
$ arrow(r)(s) = sum_(i=0)^n arrow(r)_i b^n_i $<general_bezier>

#figure(
  image("../generated/general.svg"),
  caption: [A Bezier curve of order 2 showing the construction of the point at $s=0.35$]
)

The derivative with respect to $s$
$
arrow(r)'(s) = sum_(i=0)^(n-1) -(n-i) vec(n, i)(1-s)^(n-i-1)s^(i)arrow(r)_i
  + sum_(i=1)^n i vec(n, i)(1-s)^(n-i)s^(i-1)arrow(r)_i 
$
Note that the first sum only goes to $n-1$ because the $n$ gives a exponent of $0$ for $(1-s)$ which makes the first term of the product rule $0$.
The second sum start from $1$ for a similar reason.

Swaping the sums and reindexing we achieve:
$
arrow(r)'(s) &= sum_(i=0)^(n-1) (i+1) vec(n, i+1)(1-s)^(n-i-1)s^(i)arrow(r)_(i + 1)
  - sum_(i=0)^(n-1) (n-i) vec(n, i)(1-s)^(n-i-1)s^(i)arrow(r)_i\
&= sum_(i=0)^(n-1) ((i+1) vec(n, i+1)arrow(r)_(i + 1) - (n-i) vec(n, i) arrow(r)_i)(1-s)^(n-i-1)s^(i)
$
Using the recurzion rules for the binomial coefficient:

$
arrow(r)'(s) &=sum_(i=0)^(n-1) (n vec(n-1, i)arrow(r)_(i + 1) - n vec(n-1, i) arrow(r)_i)(1-s)^(n-i-1)s^(i)\
  &= n sum_(i=0)^(n-1) (arrow(r)_(i + 1) - arrow(r)_i)vec(n-1, i)(1-s)^((n-1)-i)s^(i)
$
Thus the derivative of a Bezier curve of order $n$ is itself a Bezier curve of order $n-1$

Defining $Delta arrow(r)_i = arrow(r)_(i+1) - arrow(r)_i$ the forward difference operator allows us to write
the derivative as follows:
$
arrow(r)'(s) = n sum_(i=0)^(n-1) Delta arrow(r)_(i) b^(n-1)_i
$

Furthermore now the $m$th derivative is:

$
arrow(r)^((m))(s) = n!/(n-m)! sum_(i=0)^(n-m) Delta^m arrow(r)_(i) b^(n-m)_i
$

=== Splines of Bezier curves

Using @general_bezier we can define a Bezier curve with arbitrarily large degrees of freedom.
However, Bezier curves of a high degree are hard to work with.
The influence of a control becomes less local the higher the degree and
the calculation of their path becomes more computationally intesive.
This motivates the definition of splines of Bezier curves.
As their name suggest the paths splines of Bezier curves are defined in terms of segments which themselves are Bezier curves.
The most common type of splines are splines of cubic Bezier curves.
They are used in many computer graphics applications.
Cubic bezier curves are useful,
because their control points can be conseptually understood as controling start and end point
and the direction at start and end.


With $arrow(r)[m]$ being the Bezier curve for the $m$th segment of a spline of $n$ segments
and using ${s} = s - m quad s in [m, m+1)$ the fractional part of $s$,
the equation for the path of such a spline is:
$
arrow(r)(s) = cases(
  arrow(r)[0]({s}) quad &s in [0, 1),
  arrow(r)[1]({s}) quad &s in [1, 2),
  ...,
  arrow(r)[n-1]({s}) quad &s in [n-1, n),
)
$

Here the control points are $arrow(r)[m]_i$ where $m in {0, 1, ..., n-1}$ and $i in {0, 1, 2, 3}$.

=== Continuty of Splines

In almost every application continuous B-splines are used.
Since each segment on a spline is in itself $C_oo$-continuous only the points at $ s in {1, 2, ..., n-1} $

For $s -> 1$ a Bezier curve approaches its last control point and $s -> 0$ a Bezier curve approaches its start point.
Thus the condition for a continuous B-spline is:

$
arrow(r)[m]_3 = arrow(r)[m+1]_0
$

This trivialy means the end point of segment $m$ must be the start point of segment $m+1$.

Simillarly for the first derivative:

$
Delta arrow(r)[m]_2 &= Delta arrow(r)[m+1]_0\
arrow(r)[m]_3 - arrow(r)[m]_2 &= arrow(r)[m+1]_1 - arrow(r)[m+1]_0
$

Renaming the common end/start point $arrow(r)[m]_3 = arrow(r)[m+1]_0 = arrow(p)[m+1]$
and the vector $arrow(r)[m+1]_1 - arrow(p)[m+1] = arrow(r)[m]_3 - arrow(r)[m]_2 = arrow(v)[m+1]$
we can describe all degrees of freedom of a $C^2$-continuous spline of cubic Besier curves
using $arrow(p)[m]$ and $arrow(v)[m]$ with $m in {0, 1, ..., n}$.
#figure(image("../generated/continous_bezier.svg"))
To calculate the controlpoints the following equations can be used.
$
arrow(r)[m]_0 &= arrow(p)[m]\
arrow(r)[m]_1 &= arrow(p)[m] + arrow(v)[m]\
arrow(r)[m]_2 &= arrow(p)[m+1] - arrow(v)[m+1]\
arrow(r)[m]_3 &= arrow(p)[m+1]\
$

Curves of this type are used for the implementation of this simulation
and will from here on out be referred to simply as _splines_.

This choice was made for two reasons.
Making sure the result of the simulation is visually apealing requires that the splines don't have any kinks.
I.e. the splines should not change their direction suddenly.
This does not require $C^1$-continuity as between two segments in a spline the derivative of the path would still be allowed
to change in magnitude without changing its direction.

This however makes the splines harder to represent efficiently.
For this reason the choice was made to enforce $C^1$-continuity.

=== Representiation of the Splines

Sorting all points $arrow(p)[m]$ and vectors $arrow(v)[m]$ in to a list $Q$:
$
      &arrow(p)[0] &arrow(v)[0] &arrow(p)[1] &arrow(v)[1] &arrow(p)[2] &arrow(v)[2] ... &arrow(p)[n] &arrow(v)[n]\
Q = [ &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) ... &vec(dot, dot) &vec(dot, dot)]
$

We can define a special indexing operation to get the
coordiantes controlling the shape of segment $m in {0, 1, 2, ...,  n-1}$ as a $2 times 4$ matrix:

$
         &arrow(p)[m] quad &arrow(v)[m] quad &arrow(p)[m+1] quad &arrow(v)[m+1]\
Q[m] = [ &vec(dot, dot) quad &vec(dot, dot) quad &vec(dot, dot) quad &vec(dot, dot) ]
$

Note:
$ Q[m]_(i, j+2) = Q[m+1]_(i, j) $

Using $s - m = {s} quad s in [m, m+1)$ – the fractional part of $s$ – and Einstein sum notation,
we can write the path of the spline and its derivatives in the following way:

$
r(s)_i &= &Q[m]_(i j) A_(j k) b^3_k ({s}) quad &s in [m, m+1) \

r'(s)_i &= 3 &Q[m]_(i j) B_(j k) b^2_k ({s}) quad &s in [m, m+1)\

r''(s)_i &= 6 &Q[m]_(i j) C_(j k) b^1_k ({s}) quad &s in [m, m+1)
$

with

#grid(columns: (1fr, 1fr, 1fr), align: horizon+center)[
  $ A = mat(
    1, 1, 0, 0;
    0, 1, 0, 0;
    0, 0, 1, 1;
    0, 0, -1, 0
  ) $
][
  $ B = mat(
    0, -1, 0;
    1, -1, 0;
    0, 1, 0;
    0, -1, 1
  ) $
][
  $ C = mat(
    -1, 1;
    -2, 1;
    1, -1;
    -1, 2
  ) $
]

and 
#grid(columns: (1fr, 1fr, 1fr), align: horizon+center)[
  $ arrow(b)^3(s) = vec((1 - s)^3, 3 (1 - s)^2 s, 3 (1 - s) s^2, s^3) $
][
  $ arrow(b)^2(s) = vec((1 - s)^2, 2 (1 - s) s, s^2) $
][
  $ arrow(b)^1(s) = vec(1 - s, s) $
]
