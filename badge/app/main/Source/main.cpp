#include "Dices.h"

#include <app/event.h>
#include <app/paths.h>
#include <app/manager.h>
#include <app/scheduler.h>

#include <dices_badge.h>

#include <lvgl_window_manager/window_manager.h>

#include <tactility/check.h>

#include <cstdio>
#include <cstdlib>

namespace {

/** Must match app.id in manifest.properties. */
constexpr const char* APP_ID = "site.zannabianca1997.dices";

/**
 * The manual is prerendered at build time and shipped in the app's assets: the
 * app does not link a markdown renderer to be able to show it.
 */
void loadManual(dices_repl_t* repl) {
    char path[192];
    if (app_paths_get_assets_path(APP_ID, "manual.bin", path, sizeof(path)) != ERROR_NONE) {
        return;
    }
    FILE* file = fopen(path, "rb");
    if (file == nullptr) {
        return;
    }
    fseek(file, 0, SEEK_END);
    const long size = ftell(file);
    fseek(file, 0, SEEK_SET);
    if (size > 0) {
        auto* data = static_cast<unsigned char*>(malloc(static_cast<size_t>(size)));
        if (data != nullptr && fread(data, 1, static_cast<size_t>(size), file) == static_cast<size_t>(size)) {
            dices_repl_set_manual(repl, data, static_cast<size_t>(size));
        }
        free(data);
    }
    fclose(file);
}

} // namespace

extern "C" {


int main(int argc, char *argv[]) {
  AppInstanceId app_instance_id = app_scheduler_current_app_id();
  Context ctx{};
  ctx.appInstanceId = app_instance_id;
  ctx.repl = dices_repl_new();
  loadManual(ctx.repl);
  struct TaskEventGroup event_group{};
  task_event_group_construct(&event_group);

  struct AppEventSubscription sub{};
  check(app_event_subscribe(&sub, &event_group) == ERROR_NONE);

  WindowId window =
      window_manager_create(app_instance_id, dicesCreateWidgets, &ctx);
  bool should_close = false;
  while (!should_close) {
    task_event_group_wait_any(&event_group, nullptr, portMAX_DELAY);

    struct AppEvent event;
    while (app_event_poll(&sub, &event) == ERROR_NONE) {
      if (event.type == APP_EVENT_CLOSE) {
        should_close = true;
      }
      if (should_close)
        break;
    }
  }

  window_manager_remove(window);
  check(app_event_unsubscribe(&sub) == ERROR_NONE);
  task_event_group_destruct(&event_group);

  dices_repl_free(ctx.repl);

  return 0;
}
}
