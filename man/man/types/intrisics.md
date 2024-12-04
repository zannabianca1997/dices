---
title: "Intrisics"
---
# Intrisics

Intrisics are callable that are provided to give access to additional capability. Some intrisics are provided by the language, while others change with the kind instance (for example the `read_file` intrisic is provided only when there is filesystem access). The intrisic can be accessed through the std library, accessed from `std`. A full list of intrisics is available at `std.intrisics`. Additionally, some intrisics are injected by default into the environment. The list is available at `std.prelude`.
```dices
>>> std
# <|prelude: <|..|>, intrisics:<|..|>, ..|>
>>> std.intrisics
# <|..|>
>>> std.prelude
# <|..|>
```
The documentation of `std` is available in the manual at the topic [`"std"`](man:std).