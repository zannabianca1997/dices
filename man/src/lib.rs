//! Collection of manual pages

static INDEX: phf::Map<&'static str, Page> = include!(concat!(env!("OUT_DIR"), "/index.rs"));

pub struct Page {
    pub title: &'static str,
    pub content: &'static str,
}

pub fn man(page: &str) -> Option<&'static Page> {
    INDEX.get(page)
}
