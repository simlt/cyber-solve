use lazy_static::lazy_static;
use config::{Config, File};
use std::sync::RwLock;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Daemon {
    pub max_length: i32,
    pub left: i32,
    pub cell_width: i32,
    pub rows: Vec<DaemonRow>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DaemonRow {
    pub top: i32,
    pub bottom: i32,
}

// #[derive(Debug, Deserialize)]
// pub(crate) struct Rect {
//     left: i32,
//     right: i32,
//     top: i32,
//     bottom: i32,
// }

// #[derive(Debug, Deserialize)]
// pub(crate) struct OpenCV {
//     height_border: i32,
//     filter_threshold: i32,
// }

// #[derive(Debug, Deserialize)]
// pub(crate) struct Settings {
//     pub buffer: Rect,
//     pub daemons: Daemon,
//     pub grid: Rect,
//     pub valid_codes: Vec<String>,
//     pub opencv: OpenCV, 
// }

lazy_static! {
	static ref SETTINGS: RwLock<Config> = RwLock::new(load());
}
fn load() -> Config {
    let mut settings: Config = Config::default();
    // Add in `./settings.json`
    settings.merge(File::with_name("config/settings.json")).unwrap();

    settings
}

pub fn cfg_i32(key: &str) -> i32 {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<i32>(key).unwrap()
}
pub fn cfg_f32(key: &str) -> f32 {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<f32>(key).unwrap()
}
pub fn cfg_f64(key: &str) -> f64 {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<f64>(key).unwrap()
}
pub fn cfg_str(key: &str) -> &str {
    let settings = SETTINGS.read().unwrap();
    (*settings).get::<&str>(key).unwrap()
}
pub fn cfg_str_vec(key: &str) -> Vec<String> {
    let settings = SETTINGS.read().unwrap();
    (*settings).get_array(key).unwrap().into_iter().map(|val| val.to_string()).collect()
}
pub fn cfg_get<'de, T: Deserialize<'de>>(key: &str) -> T {
    let settings = SETTINGS.read().unwrap();
    (*settings).get(key).unwrap()
}
