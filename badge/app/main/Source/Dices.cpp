#include "Dices.h"

#include <dices_badge.h>

#include <lvgl/widgets/toolbar.h>

#include <cstring>

constexpr auto *TAG = "Dices";

namespace {

constexpr size_t EVAL_BUFFER_SIZE = 2048;
constexpr size_t OUTPUT_MAX_LENGTH = 8192;

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
}

void onMatrixPressed(lv_event_t *e) {
  if (lv_event_get_code(e) != LV_EVENT_VALUE_CHANGED)
    return;

  auto *ctx = static_cast<Context *>(lv_event_get_user_data(e));
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

  // History of inputs and results (not clickable: never joins the key group)
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

  // Dice token keyboard
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
                                    LV_SYMBOL_OK, ""};

  lv_obj_t *matrix = lv_buttonmatrix_create(parent);
  lv_buttonmatrix_set_map(matrix, buttonMap);
  lv_obj_set_size(matrix, lv_pct(100), lv_pct(40));
  // Claim the arrow keys: with these flags the badge's keypad driver sends
  // LV_KEY_UP/DOWN/LEFT/RIGHT to the matrix instead of walking the focus ring
  lv_obj_add_flag(matrix, LV_OBJ_FLAG_USER_1);
  lv_obj_add_flag(matrix, LV_OBJ_FLAG_USER_2);
  lv_obj_add_event_cb(matrix, onMatrixPressed, LV_EVENT_VALUE_CHANGED, ctx);
}
