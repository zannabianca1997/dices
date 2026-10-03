#include "Dices.h"

#include <dices_badge.h>

#include <lvgl/widgets/toolbar.h>

#include <cstring>

constexpr auto *TAG = "Dices";

namespace {

constexpr size_t EVAL_BUFFER_SIZE = 2048;
constexpr size_t OUTPUT_MAX_LENGTH = 8192;

/** How much of the screen the pads take while typing. */
constexpr int PAD_HEIGHT_PCT = 45;

static void setInputMode(Context *ctx, InputMode mode);

void appendOutput(Context *ctx, const char *text) {
  // Keep the history bounded: reset it when it grows too long
  if (strlen(lv_textarea_get_text(ctx->outputArea)) + strlen(text) >=
      OUTPUT_MAX_LENGTH) {
    lv_textarea_set_text(ctx->outputArea, "... history cleared ...\n");
  }
  lv_textarea_add_text(ctx->outputArea, text);
  // Move the cursor to the end: the textarea scrolls along with it
  lv_textarea_set_cursor_pos(ctx->outputArea, LV_TEXTAREA_CURSOR_LAST);
}

void evaluate(Context *ctx) {
  const char *input = lv_textarea_get_text(ctx->inputArea);
  if (strlen(input) == 0)
    return;

  appendOutput(ctx, ">>> ");
  appendOutput(ctx, input);
  appendOutput(ctx, "\n");

  static char result[EVAL_BUFFER_SIZE];
  int status = dices_repl_eval(ctx->repl, input, result, sizeof(result));
  if (status & DICES_STATUS_TRUNCATED) {
    appendOutput(ctx, "(...output truncated)\n");
  }
  if (strlen(result) > 0) {
    appendOutput(ctx, result);
    appendOutput(ctx, "\n");
  } else if (status == DICES_STATUS_OK) {
    appendOutput(ctx, "null\n");
  }

  lv_textarea_set_text(ctx->inputArea, "");

  // The answer is what matters now: give the history the whole screen, where
  // the arrow keys scroll it (see onHistoryKey).
  setInputMode(ctx, InputMode::Results);
}

/** Text of the button that swaps the dice pad for the full keyboard. */
static constexpr const char *TOGGLE_KEYBOARD_LABEL = "abc " LV_SYMBOL_KEYBOARD;

/**
 * Show the dice pad, the full keyboard or neither.
 *
 * Focus is deliberately left alone: Tactility's keypad sends its arrows as
 * LV_KEY_PREV/NEXT (focus navigation) and only turns them into LV_KEY_UP/DOWN
 * for the widget that has focus and claims them (LV_OBJ_FLAG_USER_1/2). The pads
 * claim them; the history is scrolled by us instead (see onPadKey), so nothing
 * here has to steal focus from wherever the keypad already put it.
 */
static void setInputMode(Context *ctx, InputMode mode) {
  ctx->inputMode = mode;

  if (mode == InputMode::Dice) {
    lv_obj_remove_flag(ctx->tokenPad, LV_OBJ_FLAG_HIDDEN);
  } else {
    lv_obj_add_flag(ctx->tokenPad, LV_OBJ_FLAG_HIDDEN);
  }

  if (mode == InputMode::Full) {
    lv_obj_remove_flag(ctx->fullKeyboard, LV_OBJ_FLAG_HIDDEN);
  } else {
    lv_obj_add_flag(ctx->fullKeyboard, LV_OBJ_FLAG_HIDDEN);
  }
}

/** Scroll the history by one screenful-ish step, clamped to its extent. */
void scrollHistory(Context *ctx, bool up) {
  constexpr int32_t STEP = 40;

  int32_t y = lv_obj_get_scroll_y(ctx->outputArea) + (up ? -STEP : STEP);
  if (y < 0)
    y = 0;

  lv_obj_scroll_to_y(ctx->outputArea, y, LV_ANIM_OFF);
}

/**
 * With the pads collapsed the history has the screen: the arrows scroll it and
 * any other key brings the dice pad back. The pads keep the keypad's focus while
 * hidden, which is what delivers these keys here.
 */
void onPadKey(lv_event_t *e) {
  if (lv_event_get_code(e) != LV_EVENT_KEY)
    return;

  auto *ctx = static_cast<Context *>(lv_event_get_user_data(e));
  if (ctx->inputMode != InputMode::Results)
    return; // a visible pad handles its own navigation

  uint32_t key = lv_event_get_key(e);
  if (key == LV_KEY_UP || key == LV_KEY_DOWN) {
    scrollHistory(ctx, key == LV_KEY_UP);
    return;
  }

  setInputMode(ctx, InputMode::Dice);
}

/**
 * The full keyboard's OK evaluates the line, and its mode button (the keyboard
 * symbol, which LVGL reports as LV_EVENT_CANCEL) goes back to the dice pad, so
 * both give a way home.
 */
void onKeyboardEvent(lv_event_t *e) {
  auto *ctx = static_cast<Context *>(lv_event_get_user_data(e));

  switch (lv_event_get_code(e)) {
  case LV_EVENT_READY:
    // evaluate() collapses the keyboards so the result can be read
    evaluate(ctx);
    break;
  case LV_EVENT_CANCEL:
    setInputMode(ctx, InputMode::Dice);
    break;
  default:
    break;
  }
}

void onMatrixPressed(lv_event_t *e) {
  if (lv_event_get_code(e) != LV_EVENT_VALUE_CHANGED)
    return;

  auto *ctx = static_cast<Context *>(lv_event_get_user_data(e));

  // Collapsed: the first pad press just brings the pad back (and is swallowed,
  // so it cannot append a token to the next expression).
  if (ctx->inputMode == InputMode::Results) {
    setInputMode(ctx, InputMode::Dice);
    return;
  }

  lv_obj_t *matrix = lv_event_get_current_target_obj(e);
  uint32_t buttonId = lv_buttonmatrix_get_selected_button(matrix);
  if (buttonId == LV_BUTTONMATRIX_BUTTON_NONE)
    return;
  const char *text = lv_buttonmatrix_get_button_text(matrix, buttonId);
  if (text == nullptr)
    return;

  if (strcmp(text, LV_SYMBOL_OK) == 0) {
    evaluate(ctx);
  } else if (strcmp(text, LV_SYMBOL_BACKSPACE) == 0) {
    lv_textarea_delete_char(ctx->inputArea);
  } else if (strcmp(text, TOGGLE_KEYBOARD_LABEL) == 0) {
    setInputMode(ctx, InputMode::Full);
  } else {
    lv_textarea_add_text(ctx->inputArea, text);
  }
}

} // namespace

void dicesCreateWidgets(lv_obj_t *parent, void *userData) {
  auto *ctx = static_cast<Context *>(userData);

  lv_obj_remove_flag(parent, LV_OBJ_FLAG_SCROLLABLE);
  lv_obj_set_flex_flow(parent, LV_FLEX_FLOW_COLUMN);
  lv_obj_set_style_pad_row(parent, 2, LV_STATE_DEFAULT);

  lv_obj_t *toolbar = lvgl_toolbar_create(parent, "Dices");
  lv_obj_align(toolbar, LV_ALIGN_TOP_MID, 0, 0);

  // History of inputs and results. It is clickable so that it can take focus
  // when the pads are collapsed: the arrow keys then scroll it (onHistoryKey
  // scrolls explicitly, since a textarea cursor alone would jump word by word).
  ctx->outputArea = lv_textarea_create(parent);
  lv_textarea_set_one_line(ctx->outputArea, false);
  lv_obj_set_width(ctx->outputArea, LV_PCT(100));
  lv_obj_set_flex_grow(ctx->outputArea, 1);
  lv_obj_remove_flag(ctx->outputArea, LV_OBJ_FLAG_CLICKABLE);
  lv_textarea_set_text(ctx->outputArea, "dices!\n");

  // One-line input
  ctx->inputArea = lv_textarea_create(parent);
  lv_textarea_set_one_line(ctx->inputArea, true);
  lv_textarea_set_placeholder_text(ctx->inputArea, "d20");
  lv_obj_set_width(ctx->inputArea, LV_PCT(100));

  // Dice token pad. The last row is a single full-width button that swaps this
  // pad for LVGL's full keyboard: dice tokens are the common case, but the
  // standard library needs `.`, `(` and `{`.
  static const char *buttonMap[] = {"1",          "2",
                                    "3",          "d",
                                    "10",         "\n",
                                    "4",          "5",
                                    "6",          "20",
                                    "100",        "\n",
                                    "7",          "8",
                                    "9",          "kh",
                                    "kl",         "\n",
                                    "0",          "+",
                                    "^",          LV_SYMBOL_BACKSPACE,
                                    LV_SYMBOL_OK, "\n",
                                    TOGGLE_KEYBOARD_LABEL,
                                    "",           "",
                                    "",           "",
                                    ""};

  ctx->tokenPad = lv_buttonmatrix_create(parent);
  lv_buttonmatrix_set_map(ctx->tokenPad, buttonMap);
  lv_obj_set_size(ctx->tokenPad, lv_pct(100), lv_pct(PAD_HEIGHT_PCT));

  // The toggle spans the row; its four placeholders are hidden, so the pad has
  // no dead stops when navigating with the badge's arrow keys.
  lv_buttonmatrix_set_button_ctrl(ctx->tokenPad, 20, LV_BUTTONMATRIX_CTRL_WIDTH_5);
  for (uint32_t i = 21; i <= 24; i++) {
    lv_buttonmatrix_set_button_ctrl(ctx->tokenPad, i,
                                    LV_BUTTONMATRIX_CTRL_HIDDEN);
  }

  // Claim the arrow keys: with these flags the badge's keypad driver sends
  // LV_KEY_UP/DOWN/LEFT/RIGHT to the matrix instead of walking the focus ring.
  // That is also what lets a collapsed pad scroll the history (onPadKey).
  lv_obj_add_flag(ctx->tokenPad, LV_OBJ_FLAG_USER_1);
  lv_obj_add_flag(ctx->tokenPad, LV_OBJ_FLAG_USER_2);
  lv_obj_add_event_cb(ctx->tokenPad, onMatrixPressed, LV_EVENT_VALUE_CHANGED,
                      ctx);
  lv_obj_add_event_cb(ctx->tokenPad, onPadKey, LV_EVENT_KEY, ctx);

  // Full keyboard, hidden until asked for. `1#` reaches the symbol map (with
  // `(`, `)`, `{`, `}`, `;`, quotes and brackets), `abc`/`ABC` the letters.
  ctx->fullKeyboard = lv_keyboard_create(parent);
  lv_keyboard_set_textarea(ctx->fullKeyboard, ctx->inputArea);
  lv_obj_set_size(ctx->fullKeyboard, lv_pct(100), lv_pct(PAD_HEIGHT_PCT));
  lv_obj_add_flag(ctx->fullKeyboard, LV_OBJ_FLAG_USER_1);
  lv_obj_add_flag(ctx->fullKeyboard, LV_OBJ_FLAG_USER_2);
  lv_obj_add_event_cb(ctx->fullKeyboard, onKeyboardEvent, LV_EVENT_READY, ctx);
  lv_obj_add_event_cb(ctx->fullKeyboard, onKeyboardEvent, LV_EVENT_CANCEL, ctx);
  lv_obj_add_event_cb(ctx->fullKeyboard, onPadKey, LV_EVENT_KEY, ctx);

  setInputMode(ctx, InputMode::Dice);
}
