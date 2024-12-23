use tracing_config::config::{
    model::{Filter, FmtLayer, FmtLayerFormatter, Layer, Level, SpanEvents, TracingConfig, Writer},
    ArcMutexGuard,
};

pub(crate) fn default_tracing_config() -> TracingConfig {
    TracingConfig {
        title: String::from("Default tracing configs"),
        filters: FromIterator::from_iter([(
            "root".to_owned(),
            Filter {
                level: if cfg!(debug_assertions) {
                    Level::Debug
                } else {
                    Level::Info
                },
                directives: None,
            },
        )]),
        layers: FromIterator::from_iter([(
            "console".to_owned(),
            Layer::Fmt(FmtLayer {
                filter: None,
                writer: "stdout".to_owned(),
                formatter: FmtLayerFormatter::Full,
                span_events: SpanEvents::None,
                ansi: atty::is(atty::Stream::Stdout),
                time: None,
                level: None,
                target: None,
                file: None,
                line_number: None,
                thread_ids: None,
                thread_names: None,
                span_list: None,
                current_span: None,
                flatten_event: None,
            }),
        )]),
        writers: FromIterator::from_iter([("stdout".to_owned(), Writer::StandardOutput)]),
    }
}

pub(crate) fn init(
    logging_config: &TracingConfig,
) -> Result<ArcMutexGuard, tracing_config::TracingConfigError> {
    tracing_config::config::init_config(false, logging_config)
}
