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
    delta_(i tilde(i)) A_(j+2, k) b^3_k quad &s in [m-1, m),
    delta_(i tilde(i)) A_(j k) b^3_k quad &s in [m, m+1),
    0 &"else"
  )
$
Similarly for the derivatives with respect to $s$
$
(partial r'_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    3 delta_(i tilde(i)) B_(j+2, k) b^2_k quad &s in [m-1, m),
    3 delta_(i tilde(i)) B_(j k) b^2_k quad &s in [m, m+1),
    0 &"else"
  )
$

$
(partial r''_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    6 delta_(i tilde(i)) C_(j+2, k) b^1_k quad &s in [m-1, m),
    6 delta_(i tilde(i)) C_(j k) b^1_k quad &s in [m, m+1),
    0 &"else"
  )
$


$
partial / (partial Q[m]_(i j)) norm(arrow(r)') &= 
  cancel(2) (partial / (partial Q[m]_(i j))arrow(r)') dot arrow(r)'
  cancel(1 / 2) norm(arrow(r)')^(-1)\
&= cases(
  3 delta_(i tilde(i)) B_(j+2, k) b^2_k r'_tilde(i)norm(arrow(r)')^(-1) quad &s in [m-1, m],
  3 delta_(i tilde(i)) B_(j k) b^2_k r'_tilde(i)norm(arrow(r)')^(-1) quad &s in [m, m+1],
  0 quad &"else",
)\
&= cases(
  3 B_(j+2, k) b^2_k r'_i norm(arrow(r)')^(-1) quad &s in [m-1, m],
  3 B_(j k) b^2_k r'_i norm(arrow(r)')^(-1) quad &s in [m, m+1],
  0 quad &"else",
)\
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
  (dot.double(r)_i norm(arrow(r)')
    + dot(r)_i norm(dot(arrow(r))'))
  d s\

= rho integral_(m-1)^m
  A_(j k) b^3_k ({s}) dot.double(Q)[p,m]_(i l) A_(l n) b^3_n ({s})norm(arrow(r)')
  d s\
+ rho integral_(m-1)^m
  A_(j k) b^3_k ({s}) dot(Q)[m]_(i l) A_(l n) b^3_n ({s})norm(dot(arrow(r))')
  d s\
= dot.double(Q)[m]_(i l) rho integral_(m-1)^m
  A_(j k) b^3_k ({s})  A_(l n) b^3_n ({s})norm(arrow(r)')
  d s\
+ dot(Q)[m]_(i l) rho integral_(m-1)^m
  A_(j k) b^3_k ({s})  A_(l n) b^3_n ({s})norm(dot(arrow(r))')
  d s
$

the second integral
$
rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  (dot.double(r)_i norm(arrow(r)')
    + dot(r)_i norm(dot(arrow(r))'))
  d s\
= rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  dot.double(Q)[m+1]_(i l) A_(l n) b^3_n ({s})
  norm(arrow(r)')
d s\
+ rho integral_m^(m+1)
  A_(j-2, k) b^3_k ({s})
  dot(Q)[m+1]_(i l) A_(l n) b^3_n ({s})
  norm(dot(arrow(r))')
d s\
$
== Term 2

The second term becomes:
$
partial / (partial q) T &= partial / (partial q) 1/2 rho integral_0^n
  dot(arrow(r))^2 norm(arrow(r)') d s\
&= 1/2 rho integral_0^n
  dot(arrow(r))^2 cancel(1/2) 1/norm(arrow(r)')
  (cancel(2) arrow(r)' dot partial / (partial q) arrow(r)') d s\
&= 1/2 rho integral_0^n
  dot(arrow(r))^2
  (arrow(r)' dot partial / (partial q) arrow(r)')
  norm(arrow(r)')^(-1) d s
$

This integral again splits up into two parts

$
1/2 rho integral_(m-1)^m
  dot(arrow(r))^2
  3 delta_(i tilde(i))B_(j k) b^2_k ({s})
  3 Q[m]_(tilde(i) l) B_(l n) b^2_n ({s})
  norm(arrow(r)')^(-1) d s\
+
1/2 rho integral_m^(m+1)
  dot(arrow(r))^2
  3 delta_(i tilde(i))B_(j-2, k) b^2_k ({s})
  3 Q[m+1]_(tilde(i) l) B_(l n) b^2_n ({s})
  norm(arrow(r)')^(-1) d s\
=
9/2 rho Q[m]_(i l) integral_(m-1)^m
  dot(arrow(r))^2
  B_(j k) b^2_k ({s})
  B_(l n) b^2_n ({s})
  norm(arrow(r)')^(-1) d s\
+
9/2 rho Q[m+1]_(i l) integral_m^(m+1)
  dot(arrow(r))^2
  B_(j-2, k) b^2_k ({s})
   B_(l n) b^2_n ({s})
  norm(arrow(r)')^(-1) d s\
$

== Term 3

The third term can be split into the contributions of the different potentials in the system.

$ partial / (partial q) V $


=== Strain Energy
$
partial / (partial q) E_(s) = bold(S) partial / (partial q) ((L - L_0)^2 / (L_0^2) L)\
= bold(S) (2(L - L_0)/ L_0^2 (partial L) / (partial q) + (L - L_0)^2 / L_0^2 (partial L) / (partial q))\
= bold(S) (2 + L - L_0)(L - L_0)/ L_0^2 (partial L) / (partial q)
$
$
(partial L) / (partial Q[m]_(i j)) = partial / (partial Q[m]_(i j)) integral_0^n norm(arrow(r)')d s\
= integral_0^n
  cancel(2) (partial / (partial Q[m]_(i j))arrow(r)') dot arrow(r)'
  cancel(1 / 2) norm(arrow(r)')^(-1)
d s\
= integral_(m-1)^m
3 delta_(i tilde(i)) B_(j+2, k) b^2_k r'_tilde(i)norm(arrow(r)')^(-1)
d s\
+integral_m^(m+1)
3 delta_(i tilde(i)) B_(j k) b^2_k r'_tilde(i)norm(arrow(r)')^(-1)
d s\
= integral_(m-1)^m
3 B_(j+2, k) b^2_k r'_i norm(arrow(r)')^(-1)
d s\
+integral_m^(m+1)
3  B_(j k) b^2_k r'_i norm(arrow(r)')^(-1)
d s\
$

=== Bending Energy
$ E_b
= bold(B) integral_0^n
(arrow(r)'_0 arrow(r)''_1 - arrow(r)''_0 arrow(r)'_1)^2
/ norm(arrow(r)')^5 d s $

#pagebreak()

=== Potential Energy
$
partial / (partial q) E_Phi &= bold(Q) partial / (partial q) integral_0^n Phi(arrow(r)(s)) norm(arrow(r)') d s\

&=bold(Q)integral_0^n Phi'(arrow(r)(s)) partial / (partial q) arrow(r)norm(arrow(r)') d s
  + bold(Q)integral_0^n Phi(arrow(r)(s)) (partial / (partial q) arrow(r)') dot arrow(r)'
    norm(arrow(r)')^(-1)d s\
$
The first integral:
$
integral_0^n Phi'(arrow(r)(s)) partial / (partial q) arrow(r)norm(arrow(r)') d s
&= integral_(m-1)^m
  Phi'(arrow(r)(s))
  delta_(i tilde(i)) A_(j+2, k) b^3_k
  norm(arrow(r)')
d s\
&quad + integral_m^(m+1)
  Phi'(arrow(r)(s))
  delta_(i tilde(i)) A_(j k) b^3_k
  norm(arrow(r)')
d s
$
The second integral:

$
integral_0^n Phi'(arrow(r)(s)) partial / (partial q) arrow(r)norm(arrow(r)') d s
&= integral_(m-1)^m
Phi(arrow(r)(s))
3 B_(j+2, k) b^2_k r'_i norm(arrow(r)')^(-1)
d s\
&quad +integral_m^(m+1)
Phi(arrow(r)(s))
3  B_(j k) b^2_k r'_i norm(arrow(r)')^(-1)
d s\
$




$
partial / (partial Q[m]_(i j))E_Phi &= bold(Q)
$

$
(partial r_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    delta_(i tilde(i)) A_(j+2, k) b^3_k quad &s in [m-1, m),
    delta_(i tilde(i)) A_(j k) b^3_k quad &s in [m, m+1),
    0 &"else"
  )\
(partial r'_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    3 delta_(i tilde(i)) B_(j+2, k) b^2_k quad &s in [m-1, m),
    3 delta_(i tilde(i)) B_(j k) b^2_k quad &s in [m, m+1),
    0 &"else"
  )\
(partial r''_tilde(i)(s))/(partial Q[m]_(i j))
  = cases(
    6 delta_(i tilde(i)) C_(j+2, k) b^1_k quad &s in [m-1, m),
    6 delta_(i tilde(i)) C_(j k) b^1_k quad &s in [m, m+1),
    0 &"else"
  )\
$
=== Field Energy
$ E_arrow(v) = bold(P) integral_0^n arrow(r)' dot arrow(v)(arrow(r)) d s $

=== Pair Interaction Energy
$ E_g =
bold(C) integral_0^(n_0) integral_0^(n_1) (norm(arrow(r)'_0) norm(arrow(r)'_1)) / norm(arrow(r)_0 - arrow(r)_1) d s_1 d s_0 $

=== Boundary Energy
$ E_r = bold(R) integral_0^n f(s(arrow(r))) norm(arrow(r)') d s $
