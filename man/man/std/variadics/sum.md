---
title: "The `sum` intrisic"
---
# The `sum` intrisic

`sum` is a variadic version of the operator `+`. It will sum all the parameters.
```dices
>>> sum(1,2,3)
6
```
Like `+` it flattens lists and maps.

If no argument is given, it returns 0.
```dices
>>> sum()
0
```