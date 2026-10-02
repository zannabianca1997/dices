#pragma once

#include <lvgl.h>

// Opaque handle from badge/include/dices_badge.h (the Rust core)
typedef struct dices_repl dices_repl_t;

struct Context {
    uint32_t appInstanceId;

    /** REPL instance, created and freed by main() */
    dices_repl_t* repl = nullptr;

    lv_obj_t* outputArea = nullptr;
    lv_obj_t* inputArea = nullptr;
};

/** window_manager_create()'s WindowCreateWidgetsFn - @a userData is the Context* for this instance. */
void dicesCreateWidgets(lv_obj_t* parent, void* userData);
