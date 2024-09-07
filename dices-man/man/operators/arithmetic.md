---
title: "Arithmetic operators"
---
# Arithmetic operators

`dices` supports the five operations `+` (addition), `-` (subtraction), `*` (multiplication), `/` (integer division), `%` (remainder). 
```dices
>>> 3 + 5
8
>>> 4 - 10
-6
>>> 23 * 2
46
>>> 12 / 3
4
>>> 11 % 5
1
```
As `dices` only support integer operations, division is approximated toward 0
```dices
>>> 10 / 3    // is 3.333
3
>>> 20 / 3    // is 6.666
6
>>> (-10) / 3 // is -3.333
-3
```

## List and maps
`dices` has, in addition to numbers, lists (`[...]`) and maps(`<|...|>`).

Sum operation act on them by summing all members. It will recurse into nested values, if present.
```dices
>>> [3, 2, 1] + 6
12
>>> <|a: 3, b: 2, c: -1|> + 4
8
>>> [3, 2, [34, 1]] + 6
46
```

The unary minus, multiplication, division and remainder instead distribute over the content
```dices
>>> - [1, 2, 3]
[-1,-2,-3]
>>> - <|a:1, b:2, c:-3|>
<|a:-1, b:-2, c:3|>
>>> 3 * [1, 2, 3]
[3,6,9]
>>> <|a:1, b:2, c:-3|> * 10
<|a:10, b:20, c:-30|>
>>> [1, 2, 3, 4, 5] / 3
[0,0,1,1,1]
```

## Strings

If used in conjunction with the arithmetic operators the string will be converted if possible to numbers
```dices
>>> 2 + "3"
5
```