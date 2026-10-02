//! Validates that the static library is consumable from C: it compiles the
//! smoke program in `tests/c_smoke/main.c` with the system C compiler, links it
//! against `libdices_badge.a` using `rustc` as the linker driver (which is what
//! brings in Rust std and its native dependencies), and runs it.
//!
//! This is the same link shape the Tactility app will perform, so it de-risks
//! the on-badge integration.

use std::path::PathBuf;
use std::process::Command;

fn target_dir() -> PathBuf {
    std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target"))
}

fn c_compiler() -> &'static str {
    ["gcc", "clang", "cc"]
        .into_iter()
        .find(|compiler| {
            Command::new(compiler)
                .arg("--version")
                .output()
                .is_ok_and(|out| out.status.success())
        })
        .expect("no C compiler found (looked for gcc, clang, cc)")
}

/// Run `rustc` with the given arguments, inheriting this build's rustc when
/// cargo exposes it.
fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into())
}

fn build_staticlib() -> PathBuf {
    let staticlib = target_dir().join("debug").join("libdices_badge.a");
    assert!(
        staticlib.exists(),
        "static library not found at {} (cargo builds it before running tests)",
        staticlib.display()
    );
    staticlib
}

#[test]
fn staticlib_links_and_runs_from_c() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let staticlib = build_staticlib();

    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(&tmp).unwrap();

    // Compile the C smoke program
    let object = tmp.join("dices_c_smoke.o");
    let compile = Command::new(c_compiler())
        .arg("-c")
        .arg("-std=c11")
        .arg("-I")
        .arg(manifest.join("include"))
        .arg("-o")
        .arg(&object)
        .arg(manifest.join("tests/c_smoke/main.c"))
        .status()
        .expect("failed to spawn the C compiler");
    assert!(compile.success(), "failed to compile the C smoke program");

    // Link it with rustc as driver: it knows how to place Rust std and the
    // native libraries it needs, replicating the link shape of a consumer.
    let wrapper = tmp.join("dices_c_smoke_wrapper.rs");
    std::fs::write(
        &wrapper,
        r#"
        use std::os::raw::c_int;

        extern "C" {
            fn run_smoke() -> c_int;
        }

        fn main() {
            std::process::exit(unsafe { run_smoke() });
        }
        "#,
    )
    .unwrap();

    let binary = tmp.join("dices_c_smoke");
    let link = Command::new(rustc())
        .arg("--edition")
        .arg("2021")
        .arg("-o")
        .arg(&binary)
        .arg(&wrapper)
        .arg("-C")
        .arg(format!("link-arg={}", object.display()))
        .arg("-C")
        .arg(format!("link-arg={}", staticlib.display()))
        .status()
        .expect("failed to spawn rustc");
    assert!(link.success(), "failed to link the C smoke program");

    let run = Command::new(&binary)
        .output()
        .expect("failed to run the C smoke program");
    assert!(
        run.status.success(),
        "C smoke program failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        stdout.contains("c smoke ok"),
        "unexpected smoke output: {stdout}"
    );
}
