mod chromosome;

use genetica::{
    crossover::dynamic_length_single_point_crossover,
    individual::Individual,
    population::{generate_population, sort_population_descending},
};
use image::{ImageBuffer, ImageReader, Rgb, RgbImage};
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Deserialize;
use std::{env, fs, process, sync::OnceLock};

use crate::chromosome::Chromosome;

#[derive(Deserialize)]
pub struct ImageDimensions {
    pub x: i32,
    pub y: i32,
}

impl ImageDimensions {
    fn total_pixels(&self) -> i32 {
        self.x * self.y
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub generations: i32,
    pub elite_count: usize,
    pub resize_dimensions: ImageDimensions,
    pub population_count: i32,
    pub crossover_probability: f32,
    pub mutation_probability: f32,
}

pub struct TargetImage {
    pub data: ImageBuffer<RGB, Vec<u8>>,
    pub length: usize,
}

pub type RGB = Rgb<u8>;
pub static TARGET_IMAGE: OnceLock<TargetImage> = OnceLock::new();

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut image_path: Option<&str> = None;
    let mut index = 0;
    let mut verbosity = 0;

    if args.is_empty() || args.len() <= 1 {
        println!("No args provided");
        process::exit(1);
    }

    while index < args.len() {
        match args[index].as_str() {
            "--image" => {
                if let Some(ip) = args.get(index + 1) {
                    println!("{ip}");
                    image_path = Some(ip);
                    index += 2;
                } else {
                    println!("No image path provided");
                    process::exit(1);
                }
            }
            "--v" => {
                verbosity = 1;
                index += 1
            }
            "--vv" => {
                verbosity = 2;
                index += 1
            }
            "--vvv" => {
                verbosity = 3;
                index += 1
            }
            _ => index += 1,
        }
    }

    let path = match image_path {
        Some(path) => path,
        None => {
            println!("No --image flag provided");
            process::exit(1);
        }
    };

    if verbosity >= 1 {
        println!("Loading image at {}", path);
    };

    let image = ImageReader::open(path)?.decode()?.resize_exact(
        CONFIG.resize_dimensions.x as u32,
        CONFIG.resize_dimensions.y as u32,
        image::imageops::FilterType::Nearest,
    );
    let data = image.into_rgb8();
    let length = data.pixels().len();
    if verbosity >= 2 {
        println!("Image pixels {length}");
    }
    TARGET_IMAGE
        .set(TargetImage { data, length })
        .map_err(|_| "Can't set image")?;

    let mut population: Vec<Chromosome> = generate_population(CONFIG.population_count);
    for num in 0..CONFIG.generations {
        if verbosity >= 2 {
            println!("Generation: {num}");
        }
        let parent1 = &population[0];
        let parent2 = &population[1];
        let (mut child1, mut child2) =
            dynamic_length_single_point_crossover(parent1, parent2, CONFIG.crossover_probability);
        child1.mutate_genes();
        child2.mutate_genes();

        let mut new_population: Vec<Chromosome> = generate_population(CONFIG.population_count - 4);

        new_population.push(child1);
        new_population.push(child2);
        new_population.push(parent1.clone());
        new_population.push(parent2.clone());
        population = new_population;
        population
            .par_iter_mut()
            .for_each(|c| c.calculate_fitness());
        sort_population_descending(&mut population);
        if verbosity >= 3 {
            let best = population.first().unwrap();
            println!("Best fitness in generation: {num}: {}", best.fitness);
        }
    }
    let best = population.first().unwrap();
    let worst = population.last().unwrap();
    let image = construct_image(best.clone());
    image.save("output/image.png")?;
    println!("Image created");
    if verbosity >= 1 {
        println!("The best chromosome's fitness is: {}", best.fitness);
        println!("The worst chromosome's fitness is: {}", worst.fitness);
    }
    Ok(())
}

fn construct_image(chromosome: Chromosome) -> RgbImage {
    let width = CONFIG.resize_dimensions.x as u32;
    let height = CONFIG.resize_dimensions.y as u32;
    let mut image = RgbImage::new(width, height);

    for (i, gene) in chromosome.genes.iter().enumerate() {
        let x = (i % width as usize) as u32;
        let y = (i / width as usize) as u32;
        if x < width && y < height {
            image.put_pixel(x, y, image::Rgb(gene.rgb.0));
        }
    }
    image
}
