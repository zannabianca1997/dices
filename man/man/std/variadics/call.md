---
title: "The `call` intrisic"
---
# The `call` intrisic

`call` is an intrisic that makes one able to call values with the arguments provided by a list. It accepts two arguments: the first is the value to call, the second a list with the parameters.
```dices
#>>> let call = std.variadics.call;
>>> call(sum, [3,2,1])
6
```