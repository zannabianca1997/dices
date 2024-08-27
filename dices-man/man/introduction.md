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
and with more dice-related ones, like filtering for highest-lowest dices:
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
And more programming related concept, like variables, scopes and closures:
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

## Running
To run the REPL, simply clone the repository and run from the command line inside the directory
```sh
$ cargo run
```
You will need to have [cargo](https://doc.rust-lang.org/cargo/) installed.
