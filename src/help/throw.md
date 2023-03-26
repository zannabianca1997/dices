# Throw

**Usage:** `throw <THROW>` or `t <THROW>` or `<THROW>`

Throw a dice, a set of dices, or more complex throws and display the result.
Throws are expressed in a mathematical notation. Repetition, summation, multiplication and filtering are available to express more complex throws.
Being the principal command used the name can be omitted.

## Dice
A single dice is expressed with the usual notation `dX` where `X` is the number of faces. Its result will run between 1 and `X`. 
Pay attention that `N` need not to be one of the usual dice sizes: a 15 sided dice can be thrown with `d15`. 
Moreover `X` can be even non-constant: `d(d6 + 1)` is a valid expression, throwing a d6, adding 1, and throwing a dice with that number of faces.

## Constants
A plain integer is a throw that always return the same value. `4` will always return 4.

## Concatenation
Throws can be concatenated with the `+` operator.
**Attention:** while often used as such, concatenation is not summation. `d6 + d4` means _'throw a d6, then a d4'_, and will return two numbers.

## Operator precedence
Square braket are used to modify operator precedence.
All binary operators associate to the left. Precedence is given to unary `-` and operator `d`, in both his unary and binary (repetition) form, then `^`, `*`, and the filter operators with the same precedence, then `+` and `-` at last.

## Summation
Concatenated throws can be summed up by enclosing them in round braket. For example, `(d4 + d6)` means _'throw a d6 and a d4, then sum the result'_, or `(3d20)` means _'sum the results of three d20 dices'_.
Binary `-` will negate his right operand, as multiplicating by -1.

**Attention:** Note the difference between `(4 + 5)` will return 9, while `[4 + 5]` returns 4 **and** 5.

## Repetition
Repetition of throws is made with the `^` operator: `(d6 + d5) ^ 4` means _'throw a d6 and d5, sum them, repeat 4 times'_. Every throw is indipendent from the others.
Again, repetition does not imply summation: the previous example will return 4 values. The second value can be not constant, but must be a scalar: `d20 ^ d6` will throw as many d20 as the d6 specify, while `d20 ^ [d5 + d5]` will throw an error.
Repetition of a single dice throw is possible via the special notation `YdX`, meaning repeat `Y` times the throwing of a `dX`. `4d20` is equivalent to `d20 ^ 4`.

## Multiplication
Multiplication is made with the operator `*`. One of the two operands must be a scalar, and it will return _the values returned by the other, multiplied by the value returned by that one_. For example, `4 * d5` will throw a d5, then multiply the value returned by 4. Multiplication by -1 can be shortened using the unary `-` operator.

**Attention:** Note the difference: `d6 * 4` will return one value, `d6 ^ 4` will return four values.

## Filtering low/high
There are 4 filter operations: `kh`, `kl`, `rh`, and `rl`. They all take a scalar as a second operand, and filter the values given by the first.
- `kh N`: return the `N` highest values
- `kl N`: return the `N` lowest values
- `rh N`: remove the `N` highest values
- `rl N`: remove the `N` lowest values