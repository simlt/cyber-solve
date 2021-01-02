use lazy_static::lazy_static;
use config::{Config, File};
use std::sync::RwLock;

lazy_static! {
	static ref SETTINGS: RwLock<Config> = RwLock::new(load());
}
fn load() -> Config {
    let mut settings: Config = Config::default();
    // Add in `./settings.json`
    settings.merge(File::with_name("config/settings.json")).unwrap();

    settings
}

// pub fn config() -> Config {
//     let settings = SETTINGS.read().unwrap();
//     *settings
// }
pub fn cfg_i32(key: &str) -> i32 {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<i32>(key).unwrap()
}
pub fn cfg_str(key: &str) -> &str {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<&str>(key).unwrap()
}
