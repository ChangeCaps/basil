use std::f32::consts::{FRAC_PI_2, TAU};

use glam::Vec3;
use rand::{prelude::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{components::Slider, mesh::Mesh};

use super::{PlantDna, PlantMessage};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Distribution {
    pub seed: u64,
    pub amount: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub value: Box<PlantDna>,
}

impl Distribution {
    pub fn new(rng: &mut impl Rng) -> Self {
        let min_angle = rng.gen_range(-FRAC_PI_2..FRAC_PI_2);
        let max_angle = rng.gen_range(-FRAC_PI_2..FRAC_PI_2);

        Self {
            seed: rng.gen(),
            amount: rng.gen_range(0.5..5.0),
            min_angle: min_angle.min(max_angle),
            max_angle: min_angle.max(max_angle),
            value: Box::new(PlantDna::new(rng)),
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, variance: f32) {
        self.amount += rng.gen_range(-0.5..0.5) * variance;
        self.min_angle += rng.gen_range(-0.5..0.5) * variance;
        self.max_angle += rng.gen_range(-0.5..0.5) * variance;
        self.value.mutate(rng, variance);

        if self.min_angle > self.max_angle {
            std::mem::swap(&mut self.min_angle, &mut self.max_angle);
        }
    }

    pub fn view(&self, callback: &Callback<DistributionMessage>) -> Html {
        html! {
            <>
                <div class="property">
                    { "Amount" }
                    <Slider
                        min=0.5
                        max=5.0
                        value={ self.amount }
                        oninput={ callback.reform(DistributionMessage::SetAmount) }
                    />
                </div>
                <div class="property">
                    { "Min Angle" }
                    <Slider
                        min={ -FRAC_PI_2 }
                        max={ FRAC_PI_2 }
                        value={ self.min_angle }
                        oninput={ callback.reform(DistributionMessage::SetMinAngle) }
                    />
                </div>
                <div class="property">
                    { "Max Angle" }
                    <Slider
                        min={ -FRAC_PI_2 }
                        max={ FRAC_PI_2 }
                        value={ self.max_angle }
                        oninput={ callback.reform(DistributionMessage::SetMaxAngle) }
                    />
                </div>
                { self.value.view(&callback.reform(|msg| DistributionMessage::ChangeValue(Box::new(msg)))) }
            </>
        }
    }

    pub fn generate(&self, mesh: &mut Mesh, start: Vec3, direction: Vec3, real_up: Vec3) {
        if self.amount == 0.0 {
            return;
        }

        let right = real_up.cross(direction).normalize();
        let up = direction.cross(right).normalize();

        let mut rng = StdRng::seed_from_u64(self.seed);

        let amount = self.amount.powi(2).round() as usize;

        for _ in 0..amount {
            let angle = rng.gen_range(0.0..TAU);

            let h = if self.min_angle == self.max_angle {
                self.min_angle
            } else {
                rng.gen_range(self.min_angle..self.max_angle)
            };

            let (sinh, cosh) = h.sin_cos();

            let (sin, cos) = angle.sin_cos();

            let d = direction * sinh + up * cos * cosh + right * sin * cosh;

            self.value
                .generate_mesh(mesh, start, d.normalize(), real_up);
        }
    }
}

pub enum DistributionMessage {
    ChangeValue(Box<PlantMessage>),
    SetAmount(f32),
    SetMinAngle(f32),
    SetMaxAngle(f32),
}

impl DistributionMessage {
    pub fn handle(self, rng: &mut impl Rng, distribution: &mut Distribution) {
        match self {
            Self::ChangeValue(msg) => msg.handle(rng, &mut distribution.value),
            Self::SetAmount(x) => distribution.amount = x,
            Self::SetMinAngle(x) => {
                distribution.min_angle = x;
                distribution.max_angle = distribution.max_angle.max(x);
            }
            Self::SetMaxAngle(x) => {
                distribution.max_angle = x;
                distribution.min_angle = distribution.min_angle.min(x);
            }
        }
    }
}
