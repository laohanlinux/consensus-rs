use toml::value::Value as Toml;

use std::fs::{self, File};
use std::env;

use crate::config::Config;

pub(crate) mod utils;


pub(crate) fn t_config() -> Config {
    let s = env::current_dir().unwrap().to_string_lossy().to_string() + &"/src/mocks/mock_config.toml".to_owned();
    println!("--> {:?}", s);
    let config: Config = toml::from_str(&fs::read_to_string(s).unwrap()).unwrap();
    config
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn t_() {
       let config =  t_config();
       println!("{:?}", config);
    }
}