---
title: "The `mult` intrisic"
---
# The `mult` intrisic

`mult` is a variadic version of the operator `*`. It will multiply all the parameters.
```dices
>>> mult(1,2,3)
6
```
Like `*` it distribute over lists and maps.

If no argument is given, it returns 1.
```dices
>>> mult()
1
```