---
title: "Introduction"
---
# The `dices` datatypes

`dices` support eight type of values: [nulls](man:values/nulls), [bools](man:values/bools), [ints](man:values/nulls), [strings](man:values/strings), [lists](man:values/lists), [maps](man:values/maps), [closures](man:values/closures) and [intrisics](man:values/intrisics).

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
