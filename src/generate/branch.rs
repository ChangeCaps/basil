use std::f32::consts::{PI, TAU};

use glam::{Quat, Vec3};
use rand::Rng;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{
    components::Slider,
    mesh::{Mesh, Vertex},
};

use super::{PlantDna, PlantMessage};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Branch {
    pub length: f32,
    pub radius: f32,
    pub bend: f32,
    pub taper: f32,
    pub end: Box<PlantDna>,
}

impl Branch {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            length: rng.gen_range(0.1..2.0),
            radius: rng.gen_range(0.05..0.5),
            bend: rng.gen_range(0.0..0.75),
            taper: rng.gen_range(0.0..1.0),
            end: Box::new(PlantDna::new(rng)),
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, variance: f32) {
        self.length += rng.gen_range(-0.5..0.5) * variance;
        self.radius += rng.gen_range(-0.25..0.25) * variance;
        self.bend += rng.gen_range(-0.25..0.25) * variance;
        self.taper += rng.gen_range(-0.5..0.5) * variance;
    }

    pub fn view(&self, callback: &Callback<BranchMessage>) -> Html {
        html! {
            <>
                <div class="property">
                    { "Length" }
                    <Slider
                        min=0.1
                        max=2.0
                        value={ self.length }
                        oninput={ callback.reform(BranchMessage::SetLength) }
                    />
                </div>
                <div class="property">
                    { "Radius" }
                    <Slider
                        min=0.05
                        max=0.5
                        value={ self.radius }
                        oninput={ callback.reform(BranchMessage::SetRadius) }
                    />
                </div>
                <div class="property">
                    { "Bend" }
                    <Slider
                        min=0.0
                        max=0.75
                        value={ self.bend }
                        oninput={ callback.reform(BranchMessage::SetBend) }
                    />
                </div>
                <div class="property">
                    { "Taper" }
                    <Slider
                        min=0.0
                        max=1.0
                        value={ self.taper }
                        oninput={ callback.reform(BranchMessage::SetTaper) }
                    />
                </div>
                { self.end.view(&callback.reform(|msg| BranchMessage::ChangeEnd(Box::new(msg)))) }
            </>
        }
    }

    pub fn generate(&self, mesh: &mut Mesh, mut start: Vec3, mut direction: Vec3, real_up: Vec3) {
        let steps = 5;
        let radial = 5;

        let right = direction.cross(real_up).normalize();
        let mut up = right.cross(direction).normalize();

        let end_radius = self.radius * self.taper;

        let bend = Quat::from_axis_angle(right, -self.bend / steps as f32 * PI);

        for i in 0..=steps {
            let x = i as f32 / steps as f32;
            let radius = self.radius * (1.0 - x) + end_radius * x;

            for j in 0..radial {
                let r = j as f32 / radial as f32 * TAU;
                let (r_sin, r_cos) = r.sin_cos();

                let p = start + right * r_cos * radius + up * r_sin * radius;

                mesh.vertices.push(Vertex {
                    position: p.into(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                });

                if i > 0 {
                    let i0 = mesh.vertices.len() as u32 - 1;
                    let i1 = if j < radial - 1 {
                        i0 + 1
                    } else {
                        i0 - radial + 1
                    };

                    let i2 = i0 - radial;
                    let i3 = i1 - radial;

                    mesh.indices.push(i0);
                    mesh.indices.push(i2);
                    mesh.indices.push(i1);

                    mesh.indices.push(i1);
                    mesh.indices.push(i2);
                    mesh.indices.push(i3);
                }
            }

            if i < steps {
                start += direction * x * self.length;
                direction = bend * direction;
                up = bend * up;
            }
        }

        self.end.generate_mesh(mesh, start, direction, real_up);
    }
}

pub enum BranchMessage {
    SetLength(f32),
    SetRadius(f32),
    SetBend(f32),
    SetTaper(f32),
    ChangeEnd(Box<PlantMessage>),
}

impl BranchMessage {
    pub fn handle(self, rng: &mut impl Rng, branch: &mut Branch) {
        match self {
            Self::SetLength(x) => branch.length = x,
            Self::SetRadius(x) => branch.radius = x,
            Self::SetBend(x) => branch.bend = x,
            Self::SetTaper(x) => branch.taper = x,
            Self::ChangeEnd(msg) => msg.handle(rng, &mut branch.end),
        }
    }
}
