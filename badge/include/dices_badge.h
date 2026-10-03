/*
 * dices-badge: C ABI to the `dices` dice-throwing language engine.
 *
 * This header is hand-written to match `badge/src/ffi.rs`; keep the two in
 * sync. Link against the `dices_badge` static library.
 */
#ifndef DICES_BADGE_H
#define DICES_BADGE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque REPL state (engine, variables, RNG). */
typedef struct dices_repl dices_repl_t;

/** Status codes returned by dices_repl_eval(). */
enum {
    DICES_STATUS_OK = 0,
    DICES_STATUS_PARSE_ERROR = 1,
    DICES_STATUS_EVAL_ERROR = 2,
    DICES_STATUS_MANUAL_ERROR = 3,
    DICES_STATUS_INVALID_ARG = -1,
    /** OR-ed into a status when the output did not fit the buffer. */
    DICES_STATUS_TRUNCATED = 0x100,
};

/** Default wrap width used by dices_repl_new(), sized for a 320 px display. */
#define DICES_DEFAULT_WIDTH 40

/**
 * Create a new REPL with a random seed and the default wrap width.
 *
 * @return A new REPL handle, to be released with dices_repl_free().
 */
dices_repl_t *dices_repl_new(void);

/**
 * Set the wrap width (in columns) used when rendering output.
 * A non-positive width is ignored.
 */
void dices_repl_set_width(dices_repl_t *repl, int width);

/**
 * Load the prerendered manual (a blob produced by the host-side prerender
 * tool). Until this is called, the manual pages are unavailable.
 *
 * @return DICES_STATUS_OK, DICES_STATUS_MANUAL_ERROR if the blob is malformed,
 *         or DICES_STATUS_INVALID_ARG if a pointer is NULL.
 */
int dices_repl_set_manual(dices_repl_t *repl, const unsigned char *data,
                          size_t len);

/** Forget all variables and re-seed the REPL with a fresh random seed. */
void dices_repl_reset(dices_repl_t *repl);

/** Release a REPL. Passing NULL is ignored. */
void dices_repl_free(dices_repl_t *repl);

/**
 * Evaluate a `dices` program, possibly spanning multiple lines and statements.
 *
 * On any status other than DICES_STATUS_INVALID_ARG, `out` receives a
 * NUL-terminated rendering of the result or of the error (REPL-style, without
 * color). The output is truncated to `out_len - 1` bytes, in which case
 * DICES_STATUS_TRUNCATED is OR-ed into the returned status.
 *
 * @param repl    REPL handle from dices_repl_new().
 * @param input   NUL-terminated program to evaluate.
 * @param out     Buffer receiving the NUL-terminated output.
 * @param out_len Size of `out` in bytes; must be at least 1.
 * @return DICES_STATUS_OK, DICES_STATUS_PARSE_ERROR or DICES_STATUS_EVAL_ERROR,
 *         each possibly OR-ed with DICES_STATUS_TRUNCATED, or
 *         DICES_STATUS_INVALID_ARG if a pointer is NULL or out_len is 0.
 */
int dices_repl_eval(dices_repl_t *repl, const char *input, char *out,
                    size_t out_len);

#ifdef __cplusplus
}
#endif

#endif /* DICES_BADGE_H */
