mod chromosome;

use lazy_static::lazy_static;
use serde::Deserialize;
use std::{fs, process};

#[derive(Deserialize)]
pub struct Config {
    pub generations: i32,
    pub population_count: i32,
    pub crossover_probability: f32,
    pub mutation_probability: f32,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let config_string = match fs::read_to_string("config.toml") {
            Ok(cs) => cs,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };
        match toml::from_str(&config_string) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    };
}

fn main() {
    println!("Hello, world!");
}
