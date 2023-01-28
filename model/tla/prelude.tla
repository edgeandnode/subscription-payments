---- MODULE prelude ----
EXTENDS Apalache, Integers, Sequences

\* @type: Set(Int) => Int;
Min(s) == CHOOSE a \in s: \A b \in s: a <= b
\* @type: Set(Int) => Int;
Max(s) == CHOOSE a \in s: \A b \in s: a >= b

\* @type: (Int, Int) => Int;
Min2(a, b) == IF a <= b THEN a ELSE b
\* @type: (Int, Int) => Int;
Max2(a, b) == IF a >= b THEN a ELSE b
\* @type: (Int, Int, Int) => Int;
Clamp(x, min, max) == Max2(min, Min2(max, x))

\* @type: (a -> b) => Set(b);
Range(f) == {f[x]: x \in DOMAIN f}

====
