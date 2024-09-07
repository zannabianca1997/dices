---
title: "Repeat"
---
# Repeat

The repeat operator `^` evaluate is left side as many times as the right side asks. The results are used to create a list.
```dices
>>> 3 ^ 4
[3,3,3,3]
>>> d6 ^ 10 // notice how the d6 is thrown multiple times
...         // and gives different results
[1..=6, 1..=6, 1..=6, 1..=6, 1..=6, 1..=6, 1..=6, 1..=6, 1..=6, 1..=6]
```
This is how multiple dice throwing is implemented. In fact `XdY == dY ^ X` for every `X` and `Y`.

With the use of [variables](man:variables) `^` can be used as a "poor man for"
```dices
>>> let x = 1;
>>> { x = x+1 } ^ 5 // evaluate the scope 5 time
[2,3,4,5,6]
```