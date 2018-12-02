use log::Level;

pub fn init_log() {
    env_logger::init();
    info!("👊 logger init successfully");
}