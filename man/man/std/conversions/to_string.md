---
title: "The `to_string` intrisic"
---
# The `to_string` intrisic

The `to_string` intrisic convert a value into a string.
```dices
>>> to_string(true)
"true"
>>> to_string(34)
"34"
>>> to_string([1,2,3])
"[1, 2, 3]"
>>> to_string(|x| x+1)
# "<closure with 1 parameters>"
```
If the value is supported by [`parse`](man:std/conversions/parse) the value can be parsed back from the string.
