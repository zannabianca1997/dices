---
title: "Closures"
---
# Closures

Closures are expressions that can be stored for later execution. They can be used for example to store throws for later.
```dices
>>> let hit_with_mace = || d20 + 3
# <closure without parameters>
>>> hit_with_mace()
4..=23
>>> hit_with_mace()
4..=23
```

The closure can have any number of parameters.
```dices
>>> let add_multiply = |a,b,c| a * b + c
# <closure with 3 parameters>
>>> add_multiply(3, 2, -3)
3
```

Finally, closures can capture values from the environment. The values are constant, no reference is kept to the original variable.
```dices
>>> let STR = 3;
>>> let hit_with_mace = || d20 + STR
# <closure without parameters (captured 1 values)>
>>> hit_with_mace()
4..=23
>>> STR = -50;
>>> hit_with_mace()
4..=23
```