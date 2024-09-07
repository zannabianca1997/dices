---
title: "Variables"
---
# Variables

`dices` support variables to store data and functions for later use. Use the `let` statement to create a new variable.
```dices
>>> let x = 3;
>>> x
3
```
Once created, a variable can be modified with the `=` operator.
```dices
#>>> let x = 3;
>>> x
3
>>> x = true;
>>> x
true
```
Notice that both `let` and `=` are expression, returning the setted value
```dices
>>> let x = 2
2
>>> (x = 3) + x
6
```
Variables can be `let`ted multiple times.
```dices
>>> let x = 3
3
>>> let x = true
true
```

## Scoping

With the brackets `{..}` one can create a scope. It can contains multiple expressions, separated by `;`. 
The scope will return the value of the last expression:
```dices
>>> { 3; 4 + 5; 7 * 2}
14
```

Variables created in the scope do not escape it, and shadows the one at the outside with the same name:
```dices
>>> let x = 2 // out of the scope x is 2
2
>>> {
...   let x = 3; // this x shadows the external one
...   x          // return the value of x inside the scope
... }
3   
>>> x // x outside is unchanged
2
```

If instead the variable is not defined, but written to, it will modify the value outside:
```dices
>>> let x = 2 // out of the scope x is 2
2
>>> {
...   x = 3; // this sets the x variable
...   x      // this x is the one outside!
... }
3   
>>> x // x outside is changed
3
```