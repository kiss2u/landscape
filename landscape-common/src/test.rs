#[macro_export]
macro_rules! init_tracing {
    () => {{
        static TRACING_INIT: std::sync::Once = std::sync::Once::new();
        static TRACING_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> =
            std::sync::OnceLock::new();

        TRACING_INIT.call_once(|| {
            let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
            let _ = TRACING_GUARD.set(guard);
            let _ = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_writer(non_blocking)
                .try_init();
        });
    }};
}
