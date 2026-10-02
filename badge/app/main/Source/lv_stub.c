/*
 * Stub LVGL custom-malloc hooks for the throwaway dices.elf link.
 *
 * LVGL is compiled with LV_STDLIB_CUSTOM: lv_mem.c references
 * lv_{malloc,realloc,free}_core, which the Tactility OS implements in its own
 * lvgl-module (the loader resolves them at app load time). The normal IDF
 * link (dices.elf) is an EXECUTABLE and therefore demands every symbol be
 * defined, but only dices.app.elf is packaged and installed - these stubs
 * just let the throwaway link complete. They are never executed: the packaged
 * app uses the OS implementation.
 */
#include <stddef.h>

void *lv_malloc_core(size_t size) {
    (void)size;
    return NULL;
}

void *lv_realloc_core(void *p, size_t new_size) {
    (void)p;
    (void)new_size;
    return NULL;
}

void lv_free_core(void *p) {
    (void)p;
}
