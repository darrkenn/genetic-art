use genetica::individual::{DynamicLengthIndividual, Generate, Individual, Mutate};
use image::Rgb;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};

use crate::{CONFIG, RGB, TARGET_IMAGE};
use std::simd::{num::SimdUint, u8x4};

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
            .into_par_iter()
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
        let image = TARGET_IMAGE.get().unwrap();

        let error_count: f32 = image
            .data
            .enumerate_pixels()
            .collect::<Vec<_>>()
            .par_chunks(CONFIG.resize_dimensions.total_pixels() as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|(x, y, pixel)| {
                        let i = (y * CONFIG.resize_dimensions.x as u32 + x) as usize;
                        if i < self.genes.len() {
                            let rgb = &self.genes[i].rgb.0;

                            // 0 is just padding
                            let pixel_vec =
                                u8x4::from_array([pixel.0[0], pixel.0[1], pixel.0[2], 0]);
                            let rgb_vec = u8x4::from_array([rgb[0], rgb[1], rgb[2], 0]);

                            let sum = (pixel_vec - rgb_vec).reduce_sum();
                            sum as f32
                        } else {
                            0.0
                        }
                    })
                    .sum::<f32>()
            })
            .sum();

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
