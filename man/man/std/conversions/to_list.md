---
title: "The `to_list` intrisic"
---
# The `to_list` intrisic

The `to_list` intrisic convert a value into a list. It is used internally by the `~` operator in case it does not recognize one of the operands.

If the value is a list, it remains unchanged.
```dices
>>> to_list([1,2,3])
[1,2,3]
```

If the value is a map, it becomes a list of the list values, in key order.
```dices
>>> to_list(<|c: false, a: null, b: true|>)
[null, true, false]
```

In all the other cases a list is made with the element as a single value.
```dices
>>> to_list(true)
[true]
>>> d20 ~ d4 // this is enabled by that behaviour
[1..=20, 1..=4]
```