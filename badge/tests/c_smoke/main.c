/*
 * Smoke program linked against libdices_badge.a from C, run by tests/c_link.rs.
 * It validates that the staticlib is consumable from C with a plain C ABI,
 * which is exactly what the Tactility app will do.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "dices_badge.h"

static int failures = 0;

#define CHECK(cond, message)                                                   \
    do {                                                                       \
        if (!(cond)) {                                                         \
            fprintf(stderr, "FAIL: %s\n", message);                            \
            failures++;                                                        \
        }                                                                      \
    } while (0)

int run_smoke(void) {
    dices_repl_t *repl = dices_repl_new();
    CHECK(repl != NULL, "dices_repl_new returned NULL");

    char buf[512];

    int status = dices_repl_eval(repl, "1 + 2", buf, sizeof buf);
    CHECK(status == DICES_STATUS_OK, "eval of `1 + 2` did not succeed");
    CHECK(strcmp(buf, "3") == 0, "eval of `1 + 2` did not render `3`");

    status = dices_repl_eval(repl, "d6", buf, sizeof buf);
    CHECK(status == DICES_STATUS_OK, "eval of `d6` did not succeed");
    long roll = strtol(buf, NULL, 10);
    CHECK(roll >= 1 && roll <= 6, "eval of `d6` did not render a number in 1..=6");

    dices_repl_eval(repl, "let x = 40;", buf, sizeof buf);
    CHECK(buf[0] == '\0', "eval of `let x = 40;` should render nothing");
    dices_repl_eval(repl, "x + 2", buf, sizeof buf);
    CHECK(strcmp(buf, "42") == 0, "variables do not persist across calls");

    status = dices_repl_eval(repl, "let = ;", buf, sizeof buf);
    CHECK(status == DICES_STATUS_PARSE_ERROR, "parse error not reported");
    CHECK(strlen(buf) > 0, "parse error not rendered");

    status = dices_repl_eval(repl, "no_such_variable", buf, sizeof buf);
    CHECK(status == DICES_STATUS_EVAL_ERROR, "eval error not reported");

    status = dices_repl_eval(repl, "[1,2,3,4,5] ~ [6,7,8,9,10]", buf, 8);
    CHECK(status == (DICES_STATUS_OK | DICES_STATUS_TRUNCATED),
          "truncation not reported");
    CHECK(strlen(buf) < 8, "truncated output overflows the buffer");

    dices_repl_set_width(repl, 10);
    dices_repl_reset(repl);
    status = dices_repl_eval(repl, "x", buf, sizeof buf);
    CHECK(status == DICES_STATUS_EVAL_ERROR, "reset does not forget variables");

    dices_repl_free(repl);

    if (failures == 0) {
        printf("c smoke ok\n");
        return 0;
    }
    fprintf(stderr, "c smoke failed with %d failure(s)\n", failures);
    return 1;
}
