---
title: The random number generator
---
# The random number generator

The `rng` module contains intrisics useful to control the random number generator.

## Seeding the RNG

Using the `seed` intrisic one can seed the random number generator with any combination of `dices` value. After a call to `seed` with at least one parameter, the RNG state depends only from the parameters provided, and is fully repeatable.

```dices
>>> seed([1,2,true], null);
>>> let a = 10d10
[_,_,_,_,_,_,_,_,_,_]
>>> seed([1,2,true], null); // seed the generator with the same values
>>> let b = 10d10  // Return the same results
[_,_,_,_,_,_,_,_,_,_]
```

If one instead want to return to the usual behavior, `seed` with no parameters seed the RNG from the system entropy.
```dices
>>> seed()
>>> 10d10  // Get random numbers
[_,_,_,_,_,_,_,_,_,_]
```

## Saving and restoring the RNG

A snapshot of the RNG state can be obtained using the `save_rng` intrisic, and restored with the `restore_rng` intrisic.

```dices
>>> let state = std.rng.save() // save the state of the RNG
# ...
>>> let a = 10d10
[_,_,_,_,_,_,_,_,_,_]
>>> std.rng.restore(state)     // restore the RNG at the same state
>>> let b = 10d10  // Return the same results
[_,_,_,_,_,_,_,_,_,_]
```