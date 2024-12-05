# Language

## Intrisics

- `serialize`: serialization of safe `Value`s (subset of `print`)
- `repr`: current implementation of `print`, make print print strings verbatim

- `exec`: Parse and run a `dices` expression
    - has an optional parameter that enable injecting variables
    - return both the last value and the global variables
- `load`: like exec, but return either the globals OR a variable called `__EXPORT__` if it exists

## REPL

- `import`: combination of `load` and `read_file`. Read a file, execute it, return either its global scope or the declared export

## Lua integration

- Write Lua bindings
- Add the `LuaFunction` datatype

## Docs

## Tests

- Extend test coverage

# Server

## API

- Add the session querying
- Add the game API

# Client

## JS

- Setup the JS project
- Plan the UI

## Binding

- Write the WASM bindings
