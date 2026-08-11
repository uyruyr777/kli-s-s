# The `math` Core and the `ncti` Type

Part of the interpreter's main distribution. Imported via `i:math;`
(see the [main documentation](README.md#3-imports-cores-and-plugins)).

```kli-s-s
i:math;
```

## Functions

All trigonometric/logarithmic functions and `abs`/`pow`/`sqrt` cast
their arguments to `float` and return `float`.

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `math.abs(n)` | 1 | `int` or `float` | the result type matches `n`'s type if it's `float`; otherwise the result is `int` |
| `math.pow(base, exp)` | 2 | `float` | `base ** exp` |
| `math.sqrt(n)` | 1 | `float` | |
| `math.floor(n)` | 1 | `int` | |
| `math.ceil(n)` | 1 | `int` | |
| `math.round(n)` | 1 | `int` | |
| `math.min(a, b)` | 2 | type of `a` or `b` (unchanged) | compares as `float`, but returns the original value of whichever argument is smaller |
| `math.max(a, b)` | 2 | type of `a` or `b` (unchanged) | same, but the larger one |
| `math.sin(n)` / `math.cos(n)` / `math.tan(n)` | 1 | `float` | radians |
| `math.log(n)` | 1 | `float` | natural logarithm |
| `math.log10(n)` | 1 | `float` | |
| `math.exp(n)` | 1 | `float` | |
| `math.pi()` | 0 | `float` | a **function**, not a property — call it with parentheses: `math.pi()` |
| `math.e()` | 0 | `float` | Euler's number; also a function, `math.e()` |
| `math.gcd(a, b)` | 2 | `int` | GCD, of the absolute values of `a` and `b` |
| `math.lcm(a, b)` | 2 | `int` | LCM |
| `math.factorial(n)` | 1 | `ncti` | panics if `n < 0` |
| `math.nctiPow(base, exp)` | 2 | `ncti` | `base` is cast to `ncti`; panics on a negative `exp` |

`math.min`/`math.max` don't convert the value — if you pass an `int`
and a `float`, the result is exactly the argument (with its original
type) that turned out smaller/larger when compared as `float`.

## `ncti` ("number close to infinity")

Stored as a chain of internal cells ("limbs"), each up to 10¹⁸. When a
cell "hits its limit" during addition/multiplication, the overflow
(carry) goes into the next one; if there isn't one, it's created.
That's why `ncti` grows without an upper bound, unlike `int`.

```kli-s-s
i:math;
#ncti fact (math.factorial(30));  // 30! — already more than one limb
cons.println(fact);
```

## Limitations

- Non-negative numbers only.
- Division isn't implemented (only addition, multiplication;
  `nctiPow` works via repeated multiplication).
