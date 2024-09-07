---
title: "The `join` intrisic"
---
# The `join` intrisic

`join` is a variadic version of the operator `~`. It will join all the parameters.
```dices
>>> join([1, 23], 2, [3, 4])
[1,23,2,3,4]
```
Like `~` it merges maps and concatenate string.
```dices
>>> join("Hello", " ", "World")
"Hello World"
```

If no argument is given, it returns the empty list.
```dices
>>> join()
[]
```