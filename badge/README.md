# `dices-badge`

Core of `dices` packaged as a C-callable static library, so it can run on the
*Romhack Camp 2026 badge* (ESP32-S3) as a [Tactility](https://github.com/TactilityProject/Tactility)
app.

This crate deliberately excludes everything that needs a desktop environment
(`reedline`, `crossterm`, `clap`, config discovery, `man-server`, ...): it
exposes a minimal REPL engine that renders its output as plain text, exactly
like the default manual printer of the CLI does.

## Staged plan

1. **This crate**: C ABI (`include/dices_badge.h`) on top of `parser`, `engine`,
   `std`, `print`, tested on the host.
2. Tactility app shell (C++/LVGL): textarea input + output pane, linking the
   staticlib.
3. Cross-compilation for `xtensa-esp32s3-espidf` (espup toolchain) and on-badge
   integration via TactilityTool.

## API

See `include/dices_badge.h`. In short:

```c
dices_repl_t *repl = dices_repl_new();
char buf[256];
dices_repl_eval(repl, "3d6 kh 1", buf, sizeof buf); // -> e.g. "[1..=6]"
dices_repl_free(repl);
```

The standard library runs in **sandboxed** mode (no filesystem access), which
is what a badge wants.
