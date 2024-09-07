---
title: "The `parse` intrisic"
---
# The `parse` intrisic

`parse` converts strings into values. It does support nulls, integers, booleans, strings and maps and lists of supported values. It does not support comments and anything containing intrisics and closures.
```dices
>>> parse(" 42 ")
42
>>> parse("true")
true
>>> parse("\"Hello\"")
"Hello"
>>> parse("<|c: true, answer: 42|>")
<|c: true, answer: 42|>
```
Limited to the supported values `parse` is guarantee to roundtrip with the [`to_string` intrisic](man:std/conversions/to_string).
```dices
>>> parse(to_string(<|c: [2, 3, 4], answer: 42|>))
<|c: [2,3,4], answer: 42|>
```