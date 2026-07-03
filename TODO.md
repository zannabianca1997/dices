## Language

- [ ] Member access
  - [x] Expression
  - [ ] Assign (lhs)
- [x] `std` keyword

## Standard library

- [x] Rng
  - [x] `seed`
  - [x] `save` and `restore`
- [x] Conversions
  - [x] `to_json`, `from_json` (as `convert.json.serialize`/`deserialize`)
  - [x] `parse` (as `convert.dices.serialize`)
  - [x] `to_string` (as `convert.string`)
  - [x] `to_list` (as `convert.list`)
  - [x] `to_number` (as `convert.number`)
  - [x] `to_bool` (as `convert.bool`)
- [x] Variadics
  - [x] `join`
  - [x] `call`
  - [x] `sum`
  - [ ] `mult` (deferred, need to think of the behaviour)
- [x] Sys
  - [x] `now`
  - [x] `read` and `write` (if supported)
- [ ] Repl
  - [ ] `help`
  - [x] `quit`
  - [x] `print`
- [ ] Prelude

## Tui

- [x] Rendering the manual
  - [x] Rendering the examples
  - [ ] `less`-like display
  - [ ] links

## Documentation

- [x] Manual
  - [x] Embedding pages
  - [ ] `covers` annotation
  - [x] Parsing the examples
  - [ ] Porting of the old manual
  - [ ] Cover the standard library
  - [ ] Generating pages for injected values

## Testing

- [ ] Mantests
  - [ ] ValueMatcher
  - [ ] Parse all tests from the manual
  - [ ] Generate tests
- [ ] Manual coverage
- [ ] Manual links integrity
