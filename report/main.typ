#set page(numbering: "1")
#set heading(numbering: "I.1.a")

#align(center, text(17pt)[ *Linewise* ])
#align(center, text(13pt)[ Using material simulation for art ])
#image("figs/example.svg")
#align(center)[Max Krummenacher]
#align(center, [#(datetime.today().year())])

#pagebreak()

= Introduction

== B-Splines

B-Splines are a type of parametric curve.
In the following sections they are defined and equations for their paths and the derivative thereof is defined.

=== Bezier Curves

A Bezier curve of order 1 is the linear interpolation of starting point $arrow(r)_0$ and an end point $arrow(r)_1$.

$ arrow(r)(s) = (1 - s) arrow(r)_0 + s arrow(r)_1 s in [0, 1] $


#figure(
  image("generated/linear.svg"),
  caption: [A Bezier curve of order 1 showing the interpolation of two points at $s=0.3$]
)<linear>

A Bezier curve of order 2 is the linear interpolation of two Bezier curves of order 1 with one ending
at the start of the other bezier spline:

$ arrow(r)(s) = (1 - s)[(1 - s) arrow(r)_0 + s arrow(r)_1] + s [(1 - s) arrow(r)_1 + s arrow(r)_2] s in [0, 1] $
$ arrow(r)(s) = (1 - s)^2 arrow(r)_0 + 2s(1 - s) arrow(r)_1 + s^2 arrow(r)_2 s in [0, 1] $

#figure(
  image("generated/quadratic.svg"),
  caption: [A Bezier curve of order 2 showing the construction of the point at $s=0.35$]
)
A Bezier curve of order 3 has four control points and is the interpolation of two order 2 Bezier curves:

$ arrow(r)(s) = (1 - s)^3 arrow(r)_0 + 3s(1 - s)^2 arrow(r)_1 + 3s^2(1 - s) arrow(r)_2 + s^3arrow(r)_3s in [0, 1] $

#figure(
  image("generated/cubic.svg"),
  caption: [A Bezier curve of order 3 showing the contruction of the point at $s=0.6$]
)

In general a Bezier curve of order $n$ is controled by $n+1$ points and the degree of the polynomial is $n$.
The path is described by:

$ arrow(r)(s) = sum_(i=0)^n vec(n, i)(1-s)^(n-i)s^(i)arrow(r)_i $


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

== Splines

B-Splines are collection of Bezier curves. The most common type of spline are splines of cubic Bezier curves.
Cubic bezier curves are useful,
because their control points can be conseptually understood as controling start and end point
and the direction at start and end.
Splines of cubic Bezier curves are commonly used in computer graphics because of their ease of use
and will be referred to as _B-splines_ from here on out.


With $arrow(r)[m]$ being the Bezier curve for the $m$th segment of a spline of $n$ segments
and using ${s} = s - m quad s in [m, m+1)$ the fractional part of $s$,
the equation for its path is:
$
arrow(r)(s) = cases(
  arrow(r)[0]({s}) quad &s in [0, 1),
  arrow(r)[1]({s}) quad &s in [1, 2),
  ...,
  arrow(r)[n-1]({s}) quad &s in [n-1, n),
)
$

Here the control points are $arrow(r)[m]_i$ where $m in {0, 1, ..., n-1}$ and $i in {0, 1, 2, 3}$.

== The Continuty of B-Splines

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

Calling the common end/start point $arrow(p)[m+1]$ and the vector $arrow(r)[m+1]_1 - arrow(p)[m+1] = arrow(v)[m+1]$
we can describe all degrees of freedom using only $arrow(p)$ and $arrow(v)$.




Thus a spline of $n$ segments can be decribed in terms points $arrow(p)_k$ and vectors $arrow(v)_k$ with $0 <= k <= n$.

In the $k$-th segment the position $arrow(r)$ and its derivative with respect to $s$ with $s in [0, 1]$ are:

$
arrow(r) &=  mat(arrow(p)_k, (arrow(p)_k + arrow(v)_k), (arrow(p)_(k+1) - arrow(v)_(k+1)), arrow(p)_(k+1))
vec((1 - s)^3, 3 (1 - s)^2 s ,3 (1 - s) s^2, s^3) \

partial / (partial s) arrow(r) &= 3 (1 - s)^2 arrow(v)_k
+ 6 (1 - s) s (arrow(p)_(k+1) - arrow(p)_k - arrow(v)_(k+1) - arrow(v)_k)
+ 3 s^2 arrow(v)_(k+1)\

partial^2 / (partial s^2) arrow(r) &= 6 (1 - s) (arrow(p)_(k+1) - arrow(p)_(k) - arrow(v)_(k+1) - 2 arrow(v)_(k))
- 6 s (arrow(p)_(k) - arrow(p)_(k+1) + arrow(v)_(k) + 2 arrow(v)_(k+1))
$


For efficient memory usage the splines are stored in one big `Vec<Vector>` $Q$.
For the spline $p$ with $n$ segments the data structure looks like this:
$
      &arrow(p)_0 &arrow(v)_0 &arrow(p)_1 &arrow(v)_1 &arrow(p)_2 &arrow(v)_2 ... &arrow(p)_n &arrow(v)_n\
Q = [ &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) &vec(dot, dot) ... &vec(dot, dot) &vec(dot, dot)]
$

The coordiantes controlling the shape of segment $m in {0, 1, 2, ... n-1}$ are:

$
         &arrow(p)_m    &arrow(v)_m    &arrow(p)_(m+1) &arrow(v)_(m+1)\
Q[m] = [ &vec(dot, dot) &vec(dot, dot) &vec(dot, dot)  &vec(dot, dot) ]
$

Note:
$ Q[m]_(i, j+2) = Q[m+1]_(i, j) $

Using $s - m = {s} quad s in [m, m+1)$, the fractional part of $s$, we can write the path of polymer $p$ and its derivatives in the following way:

$
r(s)_i &= &Q[m]_(i j) A_(j k) b^3_k ({s}) quad &s in [m, m+1) \

r'(s)_i &= 3 &Q[m]_(i j) B_(j k) b^2_k ({s}) quad &s in [m, m+1)\

r''(s)_i &= 6 &Q[m]_(i j) C_(j k) b^1_k ({s}) quad &s in [m, m+1)
$

with

#grid(columns: (1fr, 1fr, 1fr), align: horizon+center)[
  $ A = mat(1, 1, 0, 0; 0, 1, 0, 0; 0, 0, 1, 1; 0, 0, -1, 0) $
][
  $ B = mat(0, -1, 0; 1, -1, 0; 0, 1, 0; 0, -1, 1) $
][
  $ C = mat(-1, 1; -2, 1; 1, -1; -1, 2) $
]

and 
#grid(columns: (1fr, 1fr, 1fr), align: horizon+center)[
  $ arrow(b)^3(s) = vec((1 - s)^3, 3 (1 - s)^2 s, 3 (1 - s) s^2, s^3) $
][
  $ arrow(b)^2(s) = vec((1 - s)^2, 2 (1 - s) s, s^2) $
][
  $ arrow(b)^1(s) = vec(1 - s, s) $
]



== Energies
Each energy term exist to promote a certain goal for the fitting of the splines.
In this section all energy terms are derived and the reasoning behind them is explained.

=== Strain Energy
The strain energy promotes the splines to keep their length.
As the name suggests it is inspired by the strain energy in elastic materials.

$ E_s = bold(S) (L - L_0)^2 / L_0^2 L $

The length is definded by the path integral:

$ L = integral_gamma d l = integral_0^n norm(arrow(r)') d s $

=== Bending Energy
The bending energy promotes splines which are straight.

$ E_b &= bold(B) integral_gamma k^2 d l \
&= bold(B) integral_0^n (arrow(r)'_0 arrow(r)''_1 - arrow(r)''_0 arrow(r)'_1)^2
/ norm(arrow(r)')^5 d s $

=== Potential Energy
The potential energy promotes the correct density of the splines.
$ E_Phi &= bold(Q) integral_gamma Phi(arrow(r))d l \
&= bold(Q) integral_0^n Phi(arrow(r)(t)) norm(arrow(r)') d s $

=== Field Energy
The field energy promotes the alignement of the splines to a vector field.
$ E_arrow(v) &= bold(P) integral_gamma (arrow(r)' dot arrow(v)(arrow(r)))/norm(arrow(r)') d l\
&= bold(P) integral_0^n arrow(r)' dot arrow(v)(arrow(r))d s $

=== Pair Interaction Energy
The pair interaction energy creates a repulsive force between the splines.
In contrast to all other energies this energy depends on two splines.

$ E_g &= bold(C) integral_(gamma_0) integral_(gamma_1) 1 / norm(arrow(r)_0 - arrow(r)_1) d l_1 d l_0 \
&= bold(C) integral_0^(n_0) integral_0^(n_1) (norm(arrow(r)'_0) norm(arrow(r)'_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $

=== Boundary Energy
The boundary energy exist so the splines stay within the boundaries during the simulation.
Let $d(arrow(r))$ be the signed distance function to the boundary, defined such that $d(arrow(r)) < 0$ if $arrow(r)$ is with in the simulation boundaries.

Let $f(x) = cases(oo "if" x > 0, 1/x^2 "else") $
$ E_r &= bold(R) integral_gamma f(d(arrow(r))) d l \ &= bold(R) integral_0^n f(d(arrow(r))) norm(arrow(r)') d s $

=== Total Energy
In @tab.energy the formulas for the energies are summerized some constants are renamed and the parameters are given.

#figure(
  table(
    columns: 2,
    inset: 10pt,
    align: horizon,
    table.header(
      [*Energy*],
      [*Formula*],
    ),
    [strain energy],
    $ E_s = bold(S) (L - L_0)^2 / L_0^2 L $,
    [bending energy],
    $ E_b
    = bold(B) integral_0^n
    (arrow(r)'_0 arrow(r)''_1 - arrow(r)''_0 arrow(r)'_1)^2
    / norm(arrow(r)')^5 d s $,
    [potential energy],
    $ E_Phi = bold(Q) integral_0^n Phi(arrow(r)(s)) norm(arrow(r)') d s $,
    [field energy],
    $ E_arrow(v) = bold(P) integral_0^n arrow(r)' dot arrow(v)(arrow(r)) d s $,
    [pair interaction energy],
    $ E_g =
    bold(C) integral_0^(n_0) integral_0^(n_1) (norm(arrow(r)'_0) norm(arrow(r)'_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $,
    [boundary energy],
    $ E_r = bold(R) integral_0^n f(s(arrow(r))) norm(arrow(r)') d s $,
  ),
  caption: [All energy terms],
)<tab.energy>

The total energy $E_"tot"$ is simply the sum of all energies.
$ E_"tot" &= sum_i (E_(s, i) + E_(b, i) + E_(Phi, i) + E_(arrow(v), i) + E_(r, i)) + sum_i sum_(j, j > i)E_(g, i j) $

= Monte Carlo Model


= Lagrangian Model

For the Lagranian view two new matrices $dot(Q)$ and $dot.double(Q)$ are defined, refering to the first and second derivative
of the coordinates with respect to time.

The generalized cooridnates are $arrow(p)_(i j)$ and $arrow(v)_(i j)$ with $1 <= i <= N$ and $0 <= j <= n_i$
where $N$ is the total number of particles and $n_i$ is the number of segments of particle $i$.

$ E_"kin" = integral_0^L 1/ 2 arrow(v)^2 rho d l
= 1/2 rho integral_0^n dot(arrow(r))^2 norm(arrow(r)') d s $

Setting $V$ to $E_"tot"$ and $T$ to the sum of kinetic energies $sum_i E_("kin", i)$

Pluging $L = T + V$ into Lagrange's equation and assuming no external forces we get:

For any $q$ in our coordinates:

$
d / (d t) (partial / (partial dot(q)) T) - partial / (partial q) T + partial / (partial q) V = 0
$

Consider each term seprately.

The Lagranian equation results in a system of equations, one linear equation for every coordinate.
Observe that the equation that most coordiantes can be referred to in two ways:

$ Q[m]_(i, j+2) = Q[m+1]_(i, j) $

To refer to each coordinate once $m$ has to be in ${0, 1, 2, ... n}$ and $j$ from 1 to 2 for a polymer with $n$ segments.

$
(partial r_tilde(i)(s))/(partial Q[m]_(i j))
  = (partial Q[tilde(m)]_(tilde(i) tilde(j)))/(partial Q[m]_(i j)) A_(tilde(j) k) b^3_k ({s}) quad s in [tilde(m), tilde(m)+1) \
$
$
(partial Q[tilde(m)]_(tilde(i) tilde(j)))/(partial Q[m]_(i j))
  = delta_(i tilde(i))(delta_(j tilde(j))delta_(m tilde(m)) + delta_(m, tilde(m)-1) delta_(j, tilde(j)+2))
$
$
(partial r_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    delta_(i tilde(i)) A_(j+2 k) b^3_k quad &s in [m-1, m),
    delta_(i tilde(i)) A_(j k) b^3_k quad &s in [m, m+1),
    0 &"else"
  )
$
Similarly for the derivatives with respect to $s$
$
(partial r'_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    3 delta_(i tilde(i)) B_(j+2 k) b^2_k quad &s in [m-1, m),
    3 delta_(i tilde(i)) B_(j k) b^2_k quad &s in [m, m+1),
    0 &"else"
  )
$

$
(partial r''_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    6 delta_(i tilde(i)) C_(j+2 k) b^1_k quad &s in [m-1, m),
    6 delta_(i tilde(i)) C_(j k) b^1_k quad &s in [m, m+1),
    0 &"else"
  )
$


== Term 1

For any coordinate $q$ used to describe polymer $p$ the derivate is zero for all terms in $T$ except for the contribution of
the polymer $p$ itself.

The first term becomes:
$
d / (d t) (partial / (partial dot(q)) T) &=
  d / (d t) (partial / (partial dot(q)) 1/2 rho integral_0^n dot(arrow(r))^2
    norm(arrow(r)' ) d s)\
&= rho d / (d t) integral_0^n
  (dot(arrow(r)) dot (partial dot(arrow(r))) / (partial dot(q))) 
  norm(arrow(r)') d s\
&= rho integral_0^n
  (dot.double(arrow(r)) dot (partial dot(arrow(r))) / (partial dot(q)))
    norm(arrow(r)')
  + (dot(arrow(r)) dot (partial dot(arrow(r))) / (partial dot(q))) 
    norm(dot(arrow(r))')d s\
$
Note that $(partial dot(arrow(r))) / (partial dot(q))$ is independent of $t$ since $dot(arrow(r))$ depends linearly on $dot(q)$.
Thus, its derivative with respect to $t$ is 0.


$
d / (d t) ((partial T) / ( partial dot(Q)[m]_(i j)) ) &= rho integral_0^n
  (partial dot(r)_tilde(i)) / (partial dot(Q)[m]_(i j))
  (dot.double(r)_tilde(i) norm(arrow(r)')
    + dot(r)_tilde(i) norm(dot(arrow(r))'))
  d s\
&= rho integral_(m-1)^m
  delta_(i tilde(i))A_(j+2, k) b^3_k ({s})
  (dot.double(r)_tilde(i) norm(arrow(r)')
    + dot(r)_tilde(i) norm(dot(arrow(r))'))
  d s\
&quad + rho integral_m^(m+1)
  delta_(i tilde(i))A_(j k) b^3_k ({s})
  (dot.double(r)_tilde(i) norm(arrow(r)')
    + dot(r)_tilde(i) norm(dot(arrow(r))'))
  d s\
&= rho integral_(m-1)^m
  A_(j+2, k) b^3_k ({s})
  (dot.double(r)_i norm(arrow(r)')
    + dot(r)_i norm(dot(arrow(r))'))
  d s\
&quad + rho integral_m^(m+1)
  A_(j k) b^3_k ({s})
  (dot.double(r)_i norm(arrow(r)')
    + dot(r)_i norm(dot(arrow(r))'))
  d s\
$

the first integral

$
rho integral_(m-1)^m
  A_(j k) b^3_k ({s})
  (dot.double(r)_i norm((partial arrow(r)) / (partial s))
    + dot(r)_i norm((partial dot(arrow(r))) / (partial s)))
  d s\

= rho integral_(m-1)^m
  A_(j k) b^3_k ({s}) dot.double(Q)[p,m]_(i l) A_(l n) b^3_n ({s})norm((partial arrow(r)) / (partial s))
  d s\
+ rho integral_(m-1)^m
  A_(j k) b^3_k ({s}) dot(Q)[m]_(i l) A_(l n) b^3_n ({s})norm((partial dot(arrow(r))) / (partial s))
  d s\
= dot.double(Q)[m]_(i l) rho integral_(m-1)^m
  A_(j k) b^3_k ({s})  A_(l n) b^3_n ({s})norm((partial arrow(r)) / (partial s))
  d s\
+ dot(Q)[p,m]_(i l) rho integral_(m-1)^m
  A_(j k) b^3_k ({s})  A_(l n) b^3_n ({s})norm((partial dot(arrow(r))) / (partial s))
  d s
$

the second integral
$
rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  (dot.double(r)_i norm((partial arrow(r)) / (partial s))
    + dot(r)_i norm((partial dot(arrow(r))) / (partial s)))
  d s\
= rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  dot.double(Q)[m+1]_(i l) A_(l n) b^3_n ({s})
  norm((partial arrow(r)) / (partial s))
d s\
+ rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  dot(Q)[m+1]_(i l) A_(l n) b^3_n ({s})
  norm((partial dot(arrow(r))) / (partial s))
d s\
$
== Term 2

The second term becomes:
$
partial / (partial q) T &= partial / (partial q) 1/2 rho integral_0^n
  dot(arrow(r))^2 norm((partial arrow(r)) / (partial s)) d s\
&= 1/2 rho integral_0^n
  dot(arrow(r))^2 cancel(1/2) 1/norm((partial arrow(r)) / (partial s))
  (cancel(2) (partial arrow(r)) / (partial s) dot partial / (partial q) (partial arrow(r)) / (partial s)) d s\
&= 1/2 rho integral_0^n
  dot(arrow(r))^2
  ((partial arrow(r)) / (partial s) dot partial / (partial q) (partial arrow(r)) / (partial s))
  norm((partial arrow(r)) / (partial s))^(-1) d s
$

This integral again splits up into two parts

$
1/2 rho integral_(m-1)^m
  dot(arrow(r))^2
  3 delta_(i tilde(i))B_(j k) b^2_k ({s})
  3 Q[m]_(tilde(i) l) B_(l n) b^2_n ({s})
  norm((partial arrow(r)) / (partial s))^(-1) d s\
+
1/2 rho integral_m^(m+1)
  dot(arrow(r))^2
  3 delta_(i tilde(i))B_(j-2, k) b^2_k ({s})
  3 Q[m+1]_(tilde(i) l) B_(l n) b^2_n ({s})
  norm((partial arrow(r)) / (partial s))^(-1) d s\
=
9/2 rho Q[m]_(i l) integral_(m-1)^m
  dot(arrow(r))^2
  B_(j k) b^2_k ({s})
  B_(l n) b^2_n ({s})
  norm((partial arrow(r)) / (partial s))^(-1) d s\
+
9/2 rho Q[m+1]_(i l) integral_m^(m+1)
  dot(arrow(r))^2
  B_(j-2, k) b^2_k ({s})
   B_(l n) b^2_n ({s})
  norm((partial arrow(r)) / (partial s))^(-1) d s\
$

== Term 3

The third term can be split into the contributions of the different potentials in the system.

$ partial / (partial q) V $


= Strain Energy
$
partial / (partial q) E_(s) = bold(S) partial / (partial q) ((L - L_0)^2 / (L_0^2) L)\
= bold(S) (2(L - L_0)/ L_0^2 (partial L) / (partial q) + (L - L_0)^2 / L_0^2 (partial L) / (partial q))\
= bold(S) (2 + L - L_0)(L - L_0)/ L_0^2 (partial L) / (partial q)
$
$
(partial L) / (partial Q[m]_(i j)) = partial / (partial Q[m]_(i j)) integral_0^n norm((partial arrow(r))/ (partial s))d s\
= integral_0^n
  cancel(2) (partial / (partial Q[m]_(i j))(partial arrow(r))/(partial s)) dot (partial arrow(r))/ (partial s)
  cancel(1 / 2) norm((partial arrow(r))/ (partial s))^(-1)
d s\
$


    = bending energy
    $ E_(b, i)
    =  bold(B) integral_0^(n_
    (dot(arrow(r))_(i, x) dot.double(arrow(r))_(i, y) - dot.double(arrow(r))_(i, x) dot(arrow(r))_(i, y))^2
    / norm(dot(arrow(r))_i)^5 d t $,
    = potential energy
    $ E_(Phi, i) = bold(Q) integral_0^(n_i) Phi(arrow(r)_i (t)) norm(dot(arrow(r))_i) d t $,
    [field energy],
    $ E_(arrow(v), i) = bold(P) integral_0^(n_i) dot(arrow(r)_i) dot arrow(v)(arrow(r)_i) d t $,
    [pair interaction energy],
    $ E_(g, i j) =
    bold(C) integral_0^(n_i) integral_0^(n_j) (norm(dot(arrow(r))_i) norm(dot(arrow(r))_j)) / norm(arrow(r)_i - arrow(r)_j) d t_j d t_i $,
    [boundary energy],
    $ E_(r, i) = bold(R) integral_0^(n_i) f(s(arrow(r)_i)) norm(dot(arrow(r)_i)) d t $,



