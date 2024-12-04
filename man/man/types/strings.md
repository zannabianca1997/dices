---
title: "Strings"
---
# Strings
Strings are piece of text. They can be represented with the quoted notation
```dices
>>> "Hello world!"
"Hello world!"
```
Strings are unicode, and one can use escape codes to insert quotes, newline and other chars
```dices
>>> "Those are\n two lines"
"Those are\n two lines"
>>> "Hello \"world\""
"Hello \"world\""
```
The accepted escape codes are:

|  Code | Character       |   Escape   |                                                               |
|:-----:|-----------------|:----------:|---------------------------------------------------------------|
|   92  | Reverse slash   |    `\\`    |                                                               |
|   10  | Newline         |    `\n`    |                                                               |
|   13  | Return carriage |    `\r`    |                                                               |
|   9   | Tabulation      |    `\t`    |                                                               |
|   0   | Null            |    `\0`    |                                                               |
|   39  | Single quote    |    `\'`    | Can also be written literally                                 |
|   34  | Double quote    |    `\"`    |                                                               |
| 0-127 | Ascii           |   `\xHH`   | Two hexadecimal digit, from 00 to 7F                          |
| Other | Unicode         | `\u{HH..}` | Up to 6 hexadecimal digits, must be a valid unicode codepoint |

Conversion to and from other values can be made with the intrisics [`parse`](man:std/conversions/parse) and [`to_string`](man:std/conversions/to_string).
Parse can parse all values except closures and intrisics. Parse **cannot** parse arbitrary expressions or comments.
```dices
>>> parse("3")
3
>>> parse("<|a: 4, \"the answer\": 42|>")
<|a: 4, "the answer":42|>
>>> to_string([1,2,3])
# [1,2,3]
```

Single characters of the string can be accessed by indexing it with square brackets. The index is 0-based, meaning that the characters `x[0]` is the first one.
```dices
>>> let x = "Hello";
>>> x[0]
"H"
>>> x[1 + 1]
"l"
```
If the index is negative, the string is instead indexed from the end, with `x[-1]` being the last characters.
```dices
>>> let x = "Hello";
>>> x[-1]
"o"
>>> x[-2]
"l"
```
Finally, if the index is known and positive, one can index the string with the `.` notation.
```dices
>>> let x = "Hello";
>>> x.0
"H"
>>> x.2
"l"
```
