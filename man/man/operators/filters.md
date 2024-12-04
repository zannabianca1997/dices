---
title: "Filter operators"
---
# Filter operators

`dices` provide four filter operators. They take a list on their left side, and remove or keep the number of elements requested by their right side.
The operators are:
- `kh`: keep the n highest values
- `kl`: keep the n lowest values
- `rh`: remove the n highest values
- `rl`: remove the n lowest values
The order of the final list is unspecified.
```dices
>>> [1, 2, 30, 4, 5, 60, 7] rh 2
# [1,2,4,5,7]
>>> [1, 2, 30, 4, 5, 60, -7] rl 2
# [2,30,4,5,60]
```

They can be used in conjuntion with `d` to express what in tabletop gaming is called *throwing with (dis)advantage*.
```dices
>>> 2d20 kh 1 // throws 2 d20, keep the highest
[1..=20]
>>> 2d20 kl 1 // throws 2 d20, keep the lowest
[1..=20]
```