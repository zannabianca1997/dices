---
title: "Joining"
---
# Joining

The joining operator `~` works on lists, maps and strings, joining them together.

For lists, it concatenates them:
```dices
>>> [1, 2, 3] ~ [4, 5, 6]
[1,2,3,4,5,6]
>>> [-1, -2] ~ 3d6
[-1, -2, 1..=6, 1..=6, 1..=6]
```

Maps are joined together, with the values of the second taking precedence
```dices
>>> <|a: 1, x: 2|> ~ <|b: 4, x: 5|>
<| a: 1, b: 4, x:5 |>
```

Strings are directly concatenated
```dices
>>> "Hello " ~ "world"
"Hello world"
```

For all other combinations, the two operands are converted to lists (with the same logic of the [`to_list` intrisic](man:std/conversions/to_list)) and joined.
This usually means making a list with a single element, resulting in the addition of the element to the list
```dices
>>> [1, 2, 3] ~ "a"
[1,2,3,"a"]
>>> null ~ [1, true, 2]
[null, 1, true, 2]
>>> 3 ~ 2
[3,2]
>>> 3d4 ~ d6 // this is the most common usage, 
...          // adding a single dice without summing
[1..=4, 1..=4, 1..=4, 1..=6]
```

The exception are maps, that are flattened into their values, sorted by the keys alphabetical order
```dices
>>> [] ~ <|c: 32, a: 21, "d": 14, b:82|>
[21, 82, 32, 14]
```