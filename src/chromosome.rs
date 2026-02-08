use genetica::individual::{DynamicLengthIndividual, Generate, Individual, Mutate};
use image::Rgba;

use crate::{CONFIG, RGBA, TARGET_IMAGE};

#[macro_export]
macro_rules! generate_value {
    ($type:ty, $lower:expr, $upper:expr) => {{
        let value: $type = rand::random_range($lower..=$upper);
        value
    }};
}

#[derive(Debug, Clone)]
pub struct GeneType {
    pub rgba: RGBA,
}

impl Generate for GeneType {
    fn generate() -> Self {
        GeneType {
            rgba: Rgba([
                generate_value!(u8, 0, 255),
                generate_value!(u8, 0, 255),
                generate_value!(u8, 0, 255),
                generate_value!(u8, 0, 255),
            ]),
        }
    }
}

impl Mutate for GeneType {
    fn mutate(&mut self) {
        if generate_value!(f32, 0.00, 1.00) <= CONFIG.mutation_probability {
            // How many individual values to mutate
            let mutate_amount = generate_value!(u8, 0, 4);
            let mut mutated_value_positions: Vec<usize> = vec![];
            for _ in 0..mutate_amount {
                let position = loop {
                    let position = generate_value!(usize, 0, 3);
                    if !mutated_value_positions.contains(&position) {
                        break position;
                    };
                };
                self.rgba[position] = generate_value!(u8, 0, 255);
                mutated_value_positions.push(position);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chromosome {
    pub genes: Vec<GeneType>,
    pub fitness: Option<f32>,
}

impl Individual for Chromosome {
    type GeneType = GeneType;
    fn new() -> Self {
        let genes: Vec<GeneType> = (0..=10000).map(|_| GeneType::generate()).collect();
        Chromosome {
            genes,
            fitness: None,
        }
    }
    fn mutate_genes(&mut self) {
        self.genes_mut().iter_mut().for_each(|g| g.mutate());
    }
    fn fitness(&self) -> Option<f32> {
        self.fitness
    }
    fn fitness_mut(&mut self) -> &mut Option<f32> {
        &mut self.fitness
    }

    fn calculate_fitness(&mut self) {
        let mut fitness: f32 = 0.0;
        let image = TARGET_IMAGE.get().unwrap();
        for (x, y, pixel) in image.data.enumerate_pixels() {
            let i = (y * 100 + x) as usize;
            if i < self.genes.len() {
                let rgba = &self.genes[i].rgba.0;
                for i in 0..4 {
                    fitness -= (pixel.0[i] as f32 - rgba[i] as f32).abs();
                }
            }
        }
        self.fitness = Some(fitness)
    }
}

impl DynamicLengthIndividual for Chromosome {
    fn genes(&self) -> &Vec<Self::GeneType> {
        &self.genes
    }
    fn genes_mut(&mut self) -> &mut Vec<Self::GeneType> {
        &mut self.genes
    }
}
