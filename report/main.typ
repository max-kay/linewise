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

A

== C_2 Continuous B-Splines of Cubic Bezier Curves

$ hat(t) = t - floor(t) $
$ arrow(r)(t) = (1 - hat(t))^3 arrow(r)_floor(t) (1- hat(t)) ... $
$ dot(arrow(r))(t) = $
$ dot.double(arrow(r))(t) = $
$ k = (dot(arrow(r))_x dot.double(arrow(r))_y - dot.double(arrow(r))_x dot(arrow(r))_y) / norm(dot(arrow(r)))^3 $

$ d l = norm(dot(arrow(r))) d t $

= Energies
Each energy term exist to promote a certain goal for the fitting of the splines.
In this section all energy terms are derived and the reasoning behind them is explained.

== Strain Energy
The strain energy promotes the splines to keep their length.

$ E_s = integral_0^L sigma^2/(2 Y w) d l $
$ Y = sigma / epsilon => sigma = Y epsilon $
$ E_s &= integral_0^L w epsilon^2 Y / 2 d l \
  &= 1/2 w Y epsilon^2 integral_0^n norm(dot(arrow(r))) d t \
  &= 1/2 w Y epsilon^2 L \
  &= 1/2 w Y (L - L_0)^2 / L_0^2 L $

== Bending Energy
The bending energy promotes splines which are straight.

$ E_b = integral_0^L M^2 / (2 Y I) d l $
$ Y I = M/k => M = Y I k $
$ I = w^3 / 12 $
$ E_b &= integral_0^n (Y I)/2 k(t)^2 norm(dot(arrow(r))) d t \
  &= 1/24 Y w^3 integral_0^n (dot(arrow(r))_x dot.double(arrow(r))_y - dot.double(arrow(r))_x dot(arrow(r))_y)^2
  / norm(dot(arrow(r)))^5 d t $

== Potential Energy
The potential energy promotes the correct density of the splines.
$ E_Phi &= integral_0^L q Phi(arrow(r))d l \
  &= q integral_0^n Phi(arrow(r)(t)) norm(dot(arrow(r))) d t $

== Field Energy
The field energy promotes the alignement of the splines to a vector field.
$ E_arrow(v) &= integral_0^L p (dot(arrow(r)) dot arrow(v)(arrow(r)))/norm(dot(arrow(r))) d l\
  &= p integral_0^n dot(arrow(r)) dot arrow(v)(arrow(r)) d t $


== Pair Interaction Energy
The pair interaction energy creates a repulsive force between the splines.
In contrast to all other energies this energy depends on two splines $gamma_0$ and $gamma_1$.

$ E_g (gamma_0, gamma_1) &= integral_0^(L_0) integral_0^(L_1) rho / norm(arrow(r)_0 - arrow(r)_1) d l_1 d l_0 \
  &= rho integral_0^(n_0) integral_0^(n_1) (norm(dot(arrow(r))_0) norm(dot(arrow(r))_1)) / norm(arrow(r)_0 - arrow(r)_1) d t_1 d t_0 $

== Border Energy
The border energy exist so the splines stay within the boundaries during the simulation.
Let $f(arrow(r))$ be the shortest distance to the border.
$ E_r &= integral_0^L a f(arrow(r)) d l \ &= a integral_0^n f(arrow(r)) norm(dot(arrow(r))) d t $


== Total Energy
In @tab.energy the formulas for the energies are summerized some constants are renamed and the parameters are given.

#figure(
  table(
    columns: (auto, auto, auto),
    inset: 10pt,
    align: horizon,
    table.header(
      [*Energy*], [*Formula*], [*Parameter*],
    ),
    [strain energy], $ E_(s, i)  = S (L_i - L_(i, 0))^2 / L_(i, 0)^2 L_i$, [S],
    [bending energy], $ E_(b, i)
      = B integral_0^(n_i) 
      (dot(arrow(r))_(i, x) dot.double(arrow(r))_(i, y) - dot.double(arrow(r))_(i, x) dot(arrow(r))_(i, y))^2
      / norm(dot(arrow(r))_i)^5 d t $, [B],
    [potential energy], $ E_(Phi, i) = Q integral_0^(n_i) Phi(arrow(r)_i (t)) norm(dot(arrow(r))_i) d t $, [Q],
    [field energy], $ E_(arrow(v), i) = P integral_0^(n_i) dot(arrow(r)_i) dot arrow(v)(arrow(r)_i) d t $, [P],
    [pair interaction energy], $ E_(g, i j) =
      C integral_0^(n_i) integral_0^(n_j) (norm(dot(arrow(r))_i) norm(dot(arrow(r))_j)) / norm(arrow(r)_i - arrow(r)_j) d t_j d t_i $, [C],
    [border energy], $ E_(r, i) = A integral_0^(n_i) f(arrow(r)_i) norm(dot(arrow(r)_i)) d t $ , [A],
  ),
  caption: [All energy terms]
)<tab.energy>

The total energy $E_"tot"$ is simply the sum of all energies.

$ E_"tot" = sum_i (E_(s, i) + E_(b, i) + E_(Phi, i) + E_(arrow(v), i) + E_(r, i)) + sum_i sum_(j, j > i)E_(g, i j) $

