fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    match impellers_codegen::generate_bindigns() {
        Ok(_) => {
            tracing::info!("successfully completed generating bindings");
        }
        Err(e) => tracing::error!("failed to generate bindings because {e:?}"),
    }
}
