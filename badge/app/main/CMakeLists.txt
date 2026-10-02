include("$ENV{TACTILITY_SDK_PATH}/TactilitySDK.cmake")

file(GLOB_RECURSE SOURCE_FILES Source/*.c*)

# The Rust staticlib and its std libraries are linked via ELF_LIBS, set in the
# root CMakeLists.txt (project_elf() assembles the final link from there).

tactility_component_register(
    SRCS ${SOURCE_FILES}
)

# C ABI header of the Rust core (badge/include/dices_badge.h)
target_include_directories(${COMPONENT_LIB} PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/../../include"
)

# The normal IDF link (dices.elf) resolves the dices_* symbols through the
# component's own link libraries; -z notext lets Xtensa ld place the address
# constants the optimized Rust code emits in read-only sections. The loader
# link (dices.app.elf) gets the same set via ELF_LIBS in the root CMakeLists.
#
# dices.elf is only a build artifact — the packaged app is dices.app.elf, and
# OS-provided symbols (e.g. lv_malloc_core) resolve at load time there. The
# EXEC link cannot know about them, so tolerate unresolved symbols.
target_link_libraries(${COMPONENT_LIB} PRIVATE
    "-Wl,--unresolved-symbols=ignore-all"
    "-Wl,--orphan-handling=warn"
    "-Wl,--defsym=lv_malloc_core=0x40374000"
    "-Wl,--defsym=lv_free_core=0x40374000"
    "-Wl,--defsym=lv_realloc_core=0x40374000"
    "-Wl,-z,notext"
    "-Wl,--whole-archive"
    "${DICES_STATICLIB}"
    "-Wl,--no-whole-archive"
    ${DICES_LINK_LIBS}
)
