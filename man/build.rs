use std::env;

fn main() {
    cargo_build::rerun_if_env_changed!("CARGO_CFG_DEBUG_ASSERTIONS");
    if env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        // No need to watch the directory if we are dynamically loading
        return;
    }
    // Rerun if any of the embedded pages changed
    cargo_build::rerun_if_changed!("pages")
}
