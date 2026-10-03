#pragma once

#include <lvgl.h>

// Opaque handle from badge/include/dices_badge.h (the Rust core)
typedef struct dices_repl dices_repl_t;

/** What shares the screen with the output history. */
enum class InputMode {
  /** Dice token pad (the default). */
  Dice,
  /** Full character keyboard, for `.`, `(`, `{` and letters. */
  Full,
  /** Both collapsed: the history gets the whole screen and the arrows scroll it. */
  Results,
};

struct Context {
  uint32_t appInstanceId;

  /** REPL instance, created and freed by main() */
  dices_repl_t *repl = nullptr;

  lv_obj_t *outputArea = nullptr;
  lv_obj_t *inputArea = nullptr;

  /** Dice token pad and the full character keyboard; at most one is visible. */
  lv_obj_t *tokenPad = nullptr;
  lv_obj_t *fullKeyboard = nullptr;

  InputMode inputMode = InputMode::Dice;
};

/** window_manager_create()'s WindowCreateWidgetsFn - @a userData is the
 * Context* for this instance. */
void dicesCreateWidgets(lv_obj_t *parent, void *userData);
