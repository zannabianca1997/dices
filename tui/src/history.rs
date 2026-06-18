use reedline::{FileBackedHistory, History};

use crate::config::history::HistoryConfig;

pub fn history(config: HistoryConfig) -> reedline::Result<impl History> {
    if let Some(file) = config.file() {
        FileBackedHistory::with_file(config.capacity, file)
    } else {
        FileBackedHistory::new(config.capacity)
    }
}
