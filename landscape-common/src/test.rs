#[macro_export]
macro_rules! init_tracing {
    () => {
        let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(non_blocking)
            .init();
    };
}
