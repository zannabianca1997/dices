---
title: "Maps"
---
# Maps
Maps connect unique strings to values. They can be specified with the `<|..|>` notation. If the key is a valid identifier it can be specified without the quotes.
```dices
>>> <|a: d6, b: true|>
<|a: 1..=6, b: true|>
>>> <|answer: 42, "complex key": null|>
<|answer: 42, "complex key": null|>
```

They can be merged with the join `~` operator, with the second map values taking precedence over the first one.
```dices
>>> <|a: 1, b: 2|> ~ <|c: 3, "d": 4|>
<|a: 1, b: 2, c: 3, "d": 4|>
>>> <|a: 1, x: 2|> ~ <|b: 4, x: 5|>
<| a: 1, b: 4, x:5 |>
```
If instead the map is joined with a value of another type, it is transformed into the list of values, sorted by the keys.
```dices
>>> ["hey"] ~ <|c: 32, a: 21, "d": 14, b:82|>
["hey", 21, 82, 32, 14]
```

Single elements of the map can be accessed by indexing it with square brackets.
```dices
>>> let x = <|answer: 42, "complex key": true|>;
>>> x["answer"]
42
```
If the index is known, one can index the map with the `.` notation.
```dices
>>> let x = <|answer: 42, "complex key": true|>;
>>> x.answer
42
>>> x."complex key"
true
```
