use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/banners"]
struct Banners;

fn interpolate_version(content: &str) -> String {
    let re = lazy_regex::regex!(r"\{\{\s*version\s*\}\}");
    re.replace_all(content, env!("CARGO_PKG_VERSION"))
        .into_owned()
}

fn load_banner(name: &str) -> String {
    if let Some(path) = crate::config::banners_dir() {
        if let Ok(custom) = std::fs::read_to_string(path.join(name).with_extension("md")) {
            return interpolate_version(&custom);
        }
    }

    let embedded = Banners::get(&format!("{name}.md")).expect("embedded banner must exist");
    let content = String::from_utf8_lossy(&embedded.data);
    interpolate_version(&content)
}

pub fn opening() -> String {
    load_banner("opening")
}

pub fn closing() -> String {
    load_banner("closing")
}
