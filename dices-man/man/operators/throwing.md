---
title: "Throwing"
---
# Throwing

The most basic usage of `dices` is the usual dice notation `dX`, were `X` is the number of faces:
```dices
>>> d20
1..=20
>>> d6
1..=6
```
`d` generate a number between 1 and its parameter. The parameter can be more complex that a fixed number:
```dices
>>> d(3 + 4)
1..=7
>>> d(d6)
1..=6
```

## Throwing multiple dices
To throw more dices, you can use the notation `XdY`, meaning throwing `X` dices with `Y` faces.
```dices
>>> 3d6
[1..=6,1..=6,1..=6]
>>> (3+2)d6  // throws 5 d6
[1..=6,1..=6,1..=6,1..=6,1..=6]
>>> (d3)d4   /* throws a d3 then throw 
...             as many d4 as the dice say */
[1..=4] || [1..=4,1..=4] || [1..=4,1..=4,1..=4]
```
Usually one need the total sum of the thrown dices. The easiest way to get it is to put the `+` operator before the expression
```dices
>>> +3d6
3..=18
```