= Model
Each energy term exist to promote a certain goal for the fitting of the splines.
In this section all energy terms are derived and the reasoning behind them is explained.

== Intrinsic Terms

The intrinsic terms are what control the local shape of the spline.

=== Strain Energy
The strain energy promotes the splines to keep their length.
As the name suggests it is inspired by the strain energy in elastic materials.

$ E_s = bold(S) (L - L_0)^2 / L_0^2 L $

The length is definded by the path integral:

$ L = integral_gamma d l = integral_0^n norm(arrow(r)') d s $

For a consistant look the strain energy is calculated per segment.
The control points stay equally spaced through the spline.

$ E_s = bold(S) sum_m (L[m] - L[m]_0)^2 / (L[m]_0^2) L[m] $
with:
$ L[m] = integral_m^(m+1)norm(arrow(r)') d s $

=== Bending Energy
The bending energy promotes splines which are straight.

$ E_b &= bold(B) integral_gamma k^2 d l \
&= bold(B) integral_0^n (arrow(r)'_0 arrow(r)''_1 - arrow(r)''_0 arrow(r)'_1)^2
/ norm(arrow(r)')^5 d s $

== Environment Terms

The environment terms are what are used to control the resulting image macroscopically.
The dertermination of the potential $Phi$ and vector field $arrow(v)$ are treated seperatly in @image_processing

=== Potential Energy
The potential energy promotes the correct density of the splines.
$ E_Phi &= bold(Q) integral_gamma Phi(arrow(r))d l \
&= bold(Q) integral_0^n Phi(arrow(r)(t)) norm(arrow(r)') d s $

=== Field Energy
The field energy promotes the alignement of the splines to a vector field.
$ E_arrow(v) &= bold(P) integral_gamma (arrow(r)' dot arrow(v)(arrow(r)))/norm(arrow(r)') d l\
&= bold(P) integral_0^n arrow(r)' dot arrow(v)(arrow(r))d s $

=== Boundary Energy
The boundary energy exist so the splines stay within the boundaries during the simulation.
Let $d(arrow(r))$ be the signed distance function to the boundary, defined such that $d(arrow(r)) < 0$ if $arrow(r)$ is with in the simulation boundaries.

Let $f(x) = cases(oo "if" x > 0, 1/x^2 "else") $
$ E_r &= bold(R) integral_gamma f(d(arrow(r))) d l \ &= bold(R) integral_0^n f(d(arrow(r))) norm(arrow(r)') d s $

== Pair Interaction
The pair interaction energy creates a repulsive force between the splines.
In contrast to all other energies this energy depends on two splines.

$ E_g &= bold(C) integral_(gamma_0) integral_(gamma_1) 1 / norm(arrow(r)_0 - arrow(r)_1) d l_1 d l_0 \
&= bold(C) integral_0^(n_0) integral_0^(n_1) (norm(arrow(r)'_0) norm(arrow(r)'_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $

== Kinetic Energy
The kinetic energy is only used in the Lagrangian model.

$
E_"kin" = integral_gamma rho dot(arrow(r))^2 d l
$


$
E_"kin" &= sum_(m=0)^(n-1)integral_m^(m+1)rho dot(arrow(r))^2 norm(arrow(r)') d s\
  &= bold(M) sum_(m=0)^(n-1) 1/ L[m]integral_m^(m+1)dot(arrow(r))^2 norm(arrow(r)') d s
$



== Summary
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
    [boundary energy],
    $ E_r = bold(R) integral_0^n f(s(arrow(r))) norm(arrow(r)') d s $,
    [pair interaction energy],
    $ E_g =
    bold(C) integral_0^(n_0) integral_0^(n_1) (norm(arrow(r)'_0) norm(arrow(r)'_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $,
  ),
  caption: [All energy terms],
)<tab.energy>

