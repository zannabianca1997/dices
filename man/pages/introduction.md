---
title: Introduction
---
# Dices

Welcome to `dices`, an overblown dice simulator, turned turing complete language.
You can use this program to throw dice, and much more.

## TL;DR Examples
They say an image is worth a thousand words, but this is a text interface, so let's get you started with some examples.
As the most basic usage, `dices` can, you guess it, throw dices:
```dices
>> d20
15
>> d6
3
```
If you need multiple dice, you can request them:
```dices
>> 5d12       // homogeneus throws
[12, 3, 4, 12, 9]
>> d12 ~ d6   // different throws
[7, 6]
>> 4d12 ~ 2d6 // multiple sets of different dices
[4, 3, 5, 7, 3, 4]
```
There are some basic filters, like keep high values, keep low, etc. (see [filters](man://operators/filters))
```dices
>> 2d20kh1 + 3  // throw 2 d20, keep the highest result, add 3
23
```

Integer math is implemented:
```dices
>> 3+2
5
>> d20 - 10
-5
```
and when used within a sum multiple throws are summed together:
```dices
>> 2d6+d4
8
>> +5d20
47
```
See [math](man://operators/math) for more details.

You can store values for later:
```dices
>> let str = 5 // creating variable str
5
>> 4d6 + str   // using it later
22
```
And even store throws:
```dices
>> let axe_damage = || 4d6 + 5
|| ((d(6)) ^ (4)) + (5)
>> axe_damage()
24
>> axe_damage()
19
```
Those parentheses remind you of functions? They should, because they are full-blown *closures*, that you can leverage for parametrized throws (see [closures](man://closures)).
For example:
```dices
>> let throw_with_advantage = |modifier| 2d20kh1 + modifier
|modifier| (((d(20)) ^ (2)) kh (1)) + (modifier)
>> throw_with_advantage(0)
3
>> throw_with_advantage(-3)
8
```

## Where to go from here
You can see the whole manual using [index](man://index). The manual is not specifically targeted neither to programmers nor tabletop gamers, but to the kind of people that use a computer program to play *DnD*. I tried to make it accessible to people that only want to throw dice, but some arguments need more programming knowledge. 

## License
This software is distributed under the **MIT** license, if you need to know (see [LICENSE](man://LICENSE)). Use it at will.

## Contacts
This program was made by *zannabianca1997*, trying to resist the urge to buy a dice set.
If you found any problem with the program itself, or want to contribute, you can send a PR, or contact me at `zannabianca199712@gmail.com`.