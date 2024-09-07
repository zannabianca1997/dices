---
title: "Lists"
---
# Lists
Lists can contain any number of values, of any type. They are inserted with the `[..]` notation.
They are also the product of the repeat `^` operator, and of throwing multiple dices
```dices
>>> [1, true, "hello"]
[1, true, "hello"]
>>> 3 ^ 5
[3,3,3,3,3]
>>> 3d6
[1..=6, 1..=6, 1..=6]
```

They can be concatenated with the join `~` operator, that will also add single values or flatten maps.
```dices
>>> [1, 2, 3] ~ [4, 5, 6]
[1,2,3,4,5,6]
>>> [true, false] ~ null
[true, false, null]
>>> ["Hello"] ~ <|b: "beatiful", w: "world"|>
["Hello", "beatiful", "world"]
```

Single elements of the list can be accessed by indexing it with square brackets. The index is 0-based, meaning that the element `x[0]` is the first one.
```dices
>>> let x = [3, 2, 1];
>>> x[0]
3
>>> x[1 + 1]
1
```
If the index is negative, the list is instead indexed from the end, with `x[-1]` being the last element.
```dices
>>> let x = [1, 2, 3];
>>> x[-1]
3
>>> x[-2]
2
```
Finally, if the index is known and positive, one can index the list with the `.` notation.
```dices
>>> let x = [3, 2, 1];
>>> x.0
3
>>> x.2
1
```
