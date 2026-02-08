mod chromosome;

use genetica::{
    crossover::dynamic_length_single_point_crossover,
    individual::Individual,
    population::{generate_population, sort_population_ascending, sort_population_descending},
};
use image::{ImageBuffer, ImageReader, Rgba, RgbaImage};
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Deserialize;
use std::{env, fs, process, sync::OnceLock};

use crate::chromosome::Chromosome;

#[derive(Deserialize)]
pub struct Config {
    pub generations: i32,
    pub population_count: i32,
    pub crossover_probability: f32,
    pub mutation_probability: f32,
}

pub struct TargetImage {
    pub data: ImageBuffer<RGBA, Vec<u8>>,
    pub length: usize,
}

pub type RGBA = Rgba<u8>;
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
    if args.is_empty() {
        println!("No image provided");
        process::exit(1);
    }

    let image_path = &args[0].trim();
    println!("Loading image at {}", image_path);
    let image = ImageReader::open(image_path)?.decode()?.resize_exact(
        100,
        100,
        image::imageops::FilterType::Nearest,
    );
    let data = image.into_rgba8();
    let length = data.pixels().len();
    println!("Image length: {length}");
    TARGET_IMAGE
        .set(TargetImage { data, length })
        .map_err(|_| "Can't set image")?;

    let mut population: Vec<Chromosome> = generate_population(CONFIG.population_count);
    for a in 0..CONFIG.generations {
        println!("Generation 1: {a}");
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
    }
    let best = population.first().unwrap();
    let worst = population.last().unwrap();
    let image = construct_image(best.clone());
    image.save("output/image.png")?;
    println!("Image created");
    println!(
        "The best chromosome's fitness is: {}",
        best.fitness.unwrap()
    );
    println!(
        "The worst chromosome's fitness is: {}",
        worst.fitness.unwrap()
    );
    Ok(())
}

fn construct_image(chromosome: Chromosome) -> RgbaImage {
    let width = 100;
    let height = 100;
    let mut image = RgbaImage::new(width, height);

    for (i, gene) in chromosome.genes.iter().enumerate() {
        let x = (i % width as usize) as u32;
        let y = (i / width as usize) as u32;
        if x < width && y < height {
            image.put_pixel(x, y, image::Rgba(gene.rgba.0));
        }
    }
    image
}
