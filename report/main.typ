#align(
  center,
  text(
    17pt,
  )[
      *A Monte Carlo Method for Image Decomposition into Collections of B-Splines*
    ],
)
#align(center)[Max Krummenacher]


= B-Splines

== Cubic Bezier Curves

#image("figs/fig1.svg")

A

== $C_2$ Continuous B-Splines of Cubic Bezier Curves


Thus a spline of $n$ segments can be decribed in terms points $arrow(p)_k$ and vectors $arrow(v)_k$ with $0 <= k <= n$.

In the $k$-th segment the position $arrow(r)$ and its derivative with respect to $s$ with $s in [0, 1]$ are:

$
arrow(r) &= (1 - s)^3 arrow(p)_k
      + 3 (1 - s)^2 s (arrow(p)_k + arrow(v)_k)
      + 3 (1 - s) s^2(arrow(p)_(k+1) - arrow(v)_(k+1))
      + s^3 arrow(p)_(k+1)\

partial / (partial s) arrow(r) &= 3 (1 - s)^2 arrow(v)_k
            + 6 (1 - s) s (arrow(p)_(k+1) - arrow(p)_k - arrow(v)_(k+1) - arrow(v)_k)
            + 3 s^2 arrow(v)_(k+1)\

partial^2 / (partial s^2) arrow(r) &= 6 (1 - s) (arrow(p)_(k+1) - arrow(p)_(k) - arrow(v)_(k+1) - 2 arrow(v)_(k))
            - 6 s (arrow(p)_(k) - arrow(p)_(k+1) + arrow(v)_(k) + 2 arrow(v)_(k+1))
$
By setting $k = floor(s)$ and $hat(s) = {s}$ the fractional part of $s$ this definition can be extended to whole range of $s$ $[0, n]$

$ k = (dot(arrow(r))_x dot.double(arrow(r))_y - dot.double(arrow(r))_x dot(arrow(r))_y) / norm(dot(arrow(r)))^3 $

$ d l = norm(dot(arrow(r))) d s $

= Energies
Each energy term exist to promote a certain goal for the fitting of the splines.
In this section all energy terms are derived and the reasoning behind them is explained.

== Strain Energy
The strain energy promotes the splines to keep their length.
As the name suggests it is inspired by the strain energy in elastic materials.

$ E_s = bold(S) (L - L_0)^2 / L_0^2 L $

== Bending Energy
The bending energy promotes splines which are straight.

$ E_b &= bold(B) integral_0^L k^2 d l \
  &= bold(B) integral_0^n (dot(arrow(r))_x dot.double(arrow(r))_y - dot.double(arrow(r))_x dot(arrow(r))_y)^2
  / norm(dot(arrow(r)))^5 d s $

== Potential Energy
The potential energy promotes the correct density of the splines.
$ E_Phi &= bold(Q) integral_0^L Phi(arrow(r))d l \
  &= bold(Q) integral_0^n Phi(arrow(r)(t)) norm(dot(arrow(r))) d s $

== Field Energy
// TODO: absolute value impl
The field energy promotes the alignement of the splines to a vector field.
$ E_arrow(v) &= bold(P) integral_0^L - abs(dot(arrow(r)) dot arrow(v)(arrow(r)))/norm(dot(arrow(r))) d l\
  &= - bold(P) integral_0^n abs(dot(arrow(r)) dot arrow(v)(arrow(r)))d s $


== Pair Interaction Energy
The pair interaction energy creates a repulsive force between the splines.
In contrast to all other energies this energy depends on two splines.

$ E_g  &= bold(C) integral_0^(L_0) integral_0^(L_1) 1 / norm(arrow(r)_0 - arrow(r)_1) d l_1 d l_0 \
  &= bold(C) integral_0^(n_0) integral_0^(n_1) (norm(dot(arrow(r))_0) norm(dot(arrow(r))_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $

== Boundary Energy
The boundary energy exist so the splines stay within the boundaries during the simulation.
Let $d(arrow(r))$ be the signed distance function to the boundary, defined such that $d(arrow(r)) < 0$ if $arrow(r)$ is with in the simulation boundaries.

Let $f(x) = cases(oo "if" x > 0, 1/x^2 "else") $
$ E_r &= bold(R) integral_0^L f(d(arrow(r))) d l \ &= bold(R) integral_0^n f(d(arrow(r))) norm(dot(arrow(r))) d s $


== Total Energy
In @tab.energy the formulas for the energies are summerized some constants are renamed and the parameters are given.

#figure(
  table(
    columns: 2,
    inset: 10pt,
    align: horizon,
    table.header(
      [*Energy*], [*Formula*]
    ),
    [strain energy],
    $ E_(s, i)  = bold(S) (L_i - L_(i, 0))^2 / L_(i, 0)^2 L_i$,

    [bending energy],
    $ E_(b, i)
      = bold(B) integral_0^(n_i) 
      (dot(arrow(r))_(i, x) dot.double(arrow(r))_(i, y) - dot.double(arrow(r))_(i, x) dot(arrow(r))_(i, y))^2
      / norm(dot(arrow(r))_i)^5 d t $,

    [potential energy],
    $ E_(Phi, i) = bold(Q) integral_0^(n_i) Phi(arrow(r)_i (t)) norm(dot(arrow(r))_i) d t $,

    [field energy],
    $ E_(arrow(v), i) = bold(P) integral_0^(n_i) dot(arrow(r)_i) dot arrow(v)(arrow(r)_i) d t $,

    [pair interaction energy],
    $ E_(g, i j) =
      bold(C) integral_0^(n_i) integral_0^(n_j) (norm(dot(arrow(r))_i) norm(dot(arrow(r))_j)) / norm(arrow(r)_i - arrow(r)_j) d t_j d t_i $,

    [boundary energy],
    $ E_(r, i) = bold(R) integral_0^(n_i) f(s(arrow(r)_i)) norm(dot(arrow(r)_i)) d t $,
  ),
  caption: [All energy terms]
)<tab.energy>

The total energy $E_"tot"$ is simply the sum of all energies.
$ E_"tot" &= sum_i (E_(s, i) + E_(b, i) + E_(Phi, i) + E_(arrow(v), i) + E_(r, i)) + sum_i sum_(j, j > i)E_(g, i j) $
or

$ E_"tot" &= sum_i (E_(s, i) + E_(b, i) + E_(Phi, i) + E_(arrow(v), i) + E_(r, i)) + 1/2 sum_i sum_(j, j != i)E_(g, i j) $


= Lagrangian View

The generalized cooridnates are $arrow(p)_(i j)$ and $arrow(v)_(i j)$ with $1 <= i <= N$ and $0 <= j <= n_i$
where $N$ is the total number of particles and $n_i$ is the number of segments of particle $i$.



$ E_"kin" = integral_0^L 1/ 2 arrow(v)^2 rho d l
  = 1/2 rho integral_0^n (dot(arrow(r)))^2 norm((partial arrow(r)) / (partial s)) d s $

Setting $V$ to $E_"tot"$ and $T$ to the sum of kinetic energies $sum_i E_("kin", i)$

Pluging $L = T + V$ into Lagrange's equation and assuming no external forces we get:


For any $q$ in our coordinates:

$
d / (d t) (partial / (partial dot(q)) T) - partial / (partial q) T + partial / (partial q) V = 0
$

With fixed $i$ and $q in {arrow(p)_(i j)} union {arrow(v)_(i j)}$ $T$ can be rewritten as $E_("kin",i)$ since this is the only term in $T$
which depends on any cooridate with index $i$.

The first term becomes:
$
d / (d t) (partial / (partial dot(q)) T) &=
    d / (d t) (partial / (partial dot(q)) 1/2 rho integral_0^n_i (dot(arrow(r_i)))^2 norm((partial arrow(r_i)) / (partial s)) d s)\
  &= rho d / (d t) integral_0^n_i (dot(arrow(r_i)) dot (partial dot(arrow(r_i))) / (partial dot(q))) 
    norm((partial arrow(r_i)) / (partial s)) d s\
  &=  rho integral_0^n_i (
        (dot.double(arrow(r_i)) dot (partial dot(arrow(r_i))) / (partial dot(q))) 
        + (dot(arrow(r_i)) dot d / (d t) (partial dot(arrow(r_i))) / (partial dot(q)))
      )
    norm((partial arrow(r_i)) / (partial s)) d s\
$
Note that in out case $(partial dot(arrow(r_i))) / (partial dot(q))$ is independent of $t$ since $dot(arrow(r))$ depends linearly on $q$.
Thus, its derivative with respect to $t$ is 0.
$
d / (d t) (partial / (partial dot(q)) T) = rho integral_0^n_i 
    dot.double(arrow(r_i)) dot (partial dot(arrow(r_i))) / (partial dot(q))
    norm((partial arrow(r_i)) / (partial s)) d s\

$
Furthermore note that $arrow(r)_i$ only depends on the coordinates with subscript $j$ for $s in (j-1, j)$ and $s in (j, j+1)$.
For $j = 0$ and $j = n$ the first or the second interval respectively is not included.
For $arrow(p)_(i j)$ we get
$
 rho integral_(j-1)^(j)
    dot.double(arrow(r_i)) (partial dot(arrow(r_i))) / (partial dot(arrow(p))_(i j))
    norm((partial arrow(r_i)) / (partial s)) d s
  =rho integral_(0)^(1)
    dot.double(arrow(r))_(i j)(3s^2(1-s) + s^3)
    norm((partial arrow(r)_(i j)) / (partial s)) d s
$
$
 rho integral_(j)^(j+1)
    dot.double(arrow(r_i)) (partial dot(arrow(r_i))) / (partial dot(arrow(p))_(i j))
    norm((partial arrow(r_i)) / (partial s)) d s
  =rho integral_(0)^(1)
    dot.double(arrow(r))_(i j+1)((1 - s)^3 + 3s(1 - s)^2)
    norm((partial arrow(r)_(i j+1)) / (partial s)) d s
$

Similarly for $arrow(v)_(i j)$

$
 rho integral_(j-1)^(j)
    dot.double(arrow(r_i)) (partial dot(arrow(r_i))) / (partial dot(arrow(v))_(i j))
    norm((partial arrow(r_i)) / (partial s)) d s
  =rho integral_(0)^(1)
    dot.double(arrow(r))_(i j)(-3s^2(1-s))
    norm((partial arrow(r)_(i j)) / (partial s)) d s
$
$
 rho integral_(j)^(j+1)
    dot.double(arrow(r_i)) (partial dot(arrow(r_i))) / (partial dot(arrow(v))_(i j))
    norm((partial arrow(r_i)) / (partial s)) d s
  =rho integral_(0)^(1)
    dot.double(arrow(r))_(i j+1)(3s(1 - s)^2)
    norm((partial arrow(r)_(i j+1)) / (partial s)) d s
$

The second term becomes:
$
partial / (partial q) T &= partial / (partial q) 1/2 rho integral_0^n_i (dot(arrow(r_i)))^2 norm((partial arrow(r_i)) / (partial s)) d s\
  &= 1/2 rho integral_0^n_i (dot(arrow(r_i)))^2 cancel(1/2) 1/norm((partial arrow(r_i)) / (partial s)) ( cancel(2) (partial arrow(r_i)) / (partial s) dot partial / (partial q)  (partial arrow(r_i)) / (partial s)) d s\
  &= 1/2 rho integral_0^n_i (dot(arrow(r_i)))^2  ((partial arrow(r_i)) / (partial s) dot partial / (partial q)  (partial arrow(r_i)) / (partial s))/norm((partial arrow(r_i)) / (partial s))  d s
$



