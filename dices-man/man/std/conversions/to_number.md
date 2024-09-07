---
title: "The `to_number` intrisic"
---
# The `to_number` intrisic

The `to_number` intrisic try to convert a value into a number. It is used internally by the arithmetic operators in case they don't recognize one of the operands.

If the value is a number, it remains unchanged.
```dices
>>> to_number([1])
1
```

If the value is a one element list or map, it is applied to that element
```dices
>>> to_number([1])
1
>>> to_number(<|a: 1|>)
1
>>> to_number(<|a: [<|b: 1|>]|>)
1
```

If the value is a string, [`parse`](man:std/conversions/parse) is invoked, and the result is examined
```dices
>>> to_number("42")
42
>>> to_number("[42]")
42
```

If the value is a boolean, it is converted into either 0 or 1
```dices
>>> to_number(true)
1
>>> to_number(false)
0
```

Otherwise, an error is thrown.