use log::Level;

pub fn init_log() {
    env_logger::init();
    info!("👊 logger init successfully");
}

pub (crate) fn init_test_env_log() {
    use std::env;
    use env_logger::{Builder, Target};

    env::set_var("RUST_LOG", "trace");
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();
}