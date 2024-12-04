---
title: "Introduction"
---
# The `dices` datatypes

`dices` support eight type of values: [nulls](man:types/nulls), [bools](man:types/bools), [ints](man:types/nulls), [strings](man:types/strings), [lists](man:types/lists), [maps](man:types/maps), [closures](man:types/closures) and [intrisics](man:types/intrisics).

As a dinamically-typed language, all variables in `dices` can be of any type. Conversion is liberally applied when the wrong type is encountered.
```dices
>>> 5 - "2"
3
>>> 23 + true
24
>>> "Hello" ~ [3]
["Hello", 3]
```
This is mainly done in the spirit of `dices` being a language targeted to *table top gaming*, and so to avoid bogging down the language with programming concepts. If you need to do more complex stuff, look out for the upcoming **LUA integration**.
