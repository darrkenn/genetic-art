use genetica::individual::{DynamicLengthIndividual, Generate, Individual, Mutate};
use image::Rgb;

use crate::{CONFIG, RGB, TARGET_IMAGE};

#[macro_export]
macro_rules! generate_value {
    ($type:ty, $lower:expr, $upper:expr) => {{
        let value: $type = rand::random_range($lower..=$upper);
        value
    }};
}

#[derive(Debug, Clone)]
pub struct GeneType {
    pub rgb: RGB,
}

impl Generate for GeneType {
    fn generate() -> Self {
        GeneType {
            rgb: Rgb([
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
            let rgb_channel = generate_value!(usize, 0, 2);
            let delta = generate_value!(i32, -20, 20);
            let changed_value = (self.rgb.0[rgb_channel] as i32 + delta).clamp(0, 255);
            self.rgb.0[rgb_channel] = changed_value as u8;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chromosome {
    pub genes: Vec<GeneType>,
    pub fitness: f32,
}

impl Individual for Chromosome {
    type GeneType = GeneType;
    fn new() -> Self {
        let genes: Vec<GeneType> = (0..=CONFIG.resize_dimensions.total_pixels())
            .map(|_| GeneType::generate())
            .collect();
        Chromosome {
            genes,
            fitness: 0.00,
        }
    }
    fn mutate_genes(&mut self) {
        self.genes_mut().iter_mut().for_each(|g| g.mutate());
    }
    fn fitness(&self) -> f32 {
        self.fitness
    }
    fn fitness_mut(&mut self) -> &mut f32 {
        &mut self.fitness
    }

    fn calculate_fitness(&mut self) {
        //let mut fitness: f32 = 0.0;
        let mut error_count: f32 = 0.0;
        let image = TARGET_IMAGE.get().unwrap();
        for (x, y, pixel) in image.data.enumerate_pixels() {
            let i = (y * CONFIG.resize_dimensions.x as u32 + x) as usize;
            if i < self.genes.len() {
                let rgb = &self.genes[i].rgb.0;
                for p in 0..3 {
                    error_count += (pixel.0[p] as f32 - rgb[p] as f32).abs();
                    //fitness -= (pixel.0[i] as f32 - rgb[i] as f32).abs();
                }
            }
        }
        self.fitness = -error_count;
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
