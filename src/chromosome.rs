use genetica::individual::{DynamicLengthIndividual, Generate, Individual, Mutate};
use image::Rgb;
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};

use crate::{CONFIG, RGB, TARGET_IMAGE};
use std::simd::{
    f32x4,
    num::{SimdFloat, SimdUint},
    u8x4,
};

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
        let width = CONFIG.resize_dimensions.x;

        let error_count: f32 = (0..CONFIG.resize_dimensions.total_pixels() as usize)
            .into_par_iter()
            .map(|i| {
                if i < self.genes.len() {
                    let x = (i % width as usize) as u32;
                    let y = (i / width as usize) as u32;
                    let rgb = &self.genes[i].rgb.0;
                    let pixel = image.data.get_pixel(x, y);
                    // 0 is just padding
                    let pixel_vec =
                        f32x4::from_array([pixel[0] as f32, pixel[1] as f32, pixel[2] as f32, 0.0]);
                    let rgb_vec =
                        f32x4::from_array([rgb[0] as f32, rgb[1] as f32, rgb[2] as f32, 0.0]);

                    (pixel_vec - rgb_vec).abs().reduce_sum()
                } else {
                    0.0
                }
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
