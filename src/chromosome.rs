use genetica::individual::{Generate, Individual, Mutate};
use image::Rgba;

use crate::CONFIG;

#[macro_export]
macro_rules! generate_value {
    ($type:ty, $lower:expr, $upper:expr) => {{
        let value: $type = rand::random_range($lower..=$upper);
        value
    }};
}

#[derive(Debug, Clone)]
pub struct GeneType {
    pub rgba: Rgba<u8>,
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
