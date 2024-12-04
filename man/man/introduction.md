---
title: "Introduction"
---
# Welcome to `dices`!

This is the manual for `dices`, a dice-throwing simulator, turned Turing complete programming language.
It supports standard dice notation:
```dices
>>> d6
1..=6
>>> d20
1..=20
>>> 3d10
[1..=10, 1..=10, 1..=10]
```
That can be combined with the usual arithmetic operations:
```dices
>>> 3 + 4
7
>>> 3 * 2 - 7
-1
>>> d6 + 10
11..=16
```
And operations related to dices, like filtering for highest-lowest dices:
```dices
>>> 2d20 kh 1 // throws 2 d20, then keep the highest
[1..=20]
>>> 5d10 kl 2 // throws 5 d10, then keep the 2 lower ones
[1..=10, 1..=10]
>>> 3d8 rh 1 // throws 3 d8, remove the higher one
[1..=8, 1..=8]
>>> 5d4 rl 2 // throws 5 d4, remove the two lower ones
[1..=4, 1..=4, 1..=4]
```
More programming related concept, like variables, scopes and closures, are also available:
```dices
>>> let STR_MODIFIER = 3;
>>> STR_MODIFIER
3
>>> STR_MODIFIER = 2;
>>> STR_MODIFIER
2
>>> {
...     let STR_MODIFIER = 4;
...     STR_MODIFIER
... }
4
>>> STR_MODIFIER
2
>>> let STR_THROW = || d20 + STR_MODIFIER; // this will capture the value of `STR_MODIFIER`
>>> STR_THROW()
3..=23
```

## Where to go from there
`dices` has an internal manual. Type `help("index")` in the *REPL* to see the index. The manual is not specifically targeted neither to programmers nor tabletop gamers, but to the kind of people that use a computer program to play *DnD*. I tried to make it accessible to people that only want to throw dice, but some arguments might need more programming knowledge. 

## License
This software is distributed under the **MIT** license, if you need to know. Use it at will.

## Contacts
This program was made by *zannabianca1997*, trying to resist the urge to buy a dice set.
If you found any problem with the program itself, or want to contribute, you can send a PR, or contact me at [zannabianca199712@gmail.com](mailto:zannabianca199712@gmail.com).