# `dices` 0.3.2
This is a program able to simulate dice-throwing. It supports traditional dice notation, but also mathematical operations, variables, and closures. 

## Running
> [!NOTE]
> If you know your way around git and the Rust build system, just clone the repository and run `cargo run +nightly --release`.

To compile and run this program, you must first ensure that both [`git`](https://git-scm.com/) and [`cargo`](https://doc.rust-lang.org/cargo/) are present on your system. Then from a console clone the repository and run the project:
```sh
$ git clone https://github.com/zannabianca1997/dices.git
$ cd dices
$ cargo run +nightly --release
```
You should see a lot of text, and then a prompt similar to `>>>`. Type `quit()` or press `Ctrl + D` to exit.

## Where to go from there
`dices` has an internal manual. Type `help()` in the *REPL* to get started. The manual is not specifically targeted neither to programmers nor tabletop gamers, but to the kind of people that use a computer program to play *DnD*. I tried to make it accessible to people that only want to throw dice, but some arguments might need more programming knowledge. 

## License
This software is distributed under the **MIT** license, if you need to know. Use it at will.

## Contacts
This program was made by *zannabianca1997*, trying to resist the urge to buy a dice set.
If you found any problem with the program itself, or want to contribute, you can send a PR, or contact me at [zannabianca199712@gmail.com](mailto:zannabianca199712@gmail.com).
