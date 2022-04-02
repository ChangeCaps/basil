use glam::Vec3;
use rand::Rng;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{
    components::Slider,
    mesh::{Mesh, Vertex},
    println,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leaf {
    length: f32,
    width: f32,
    bend: f32,
    bend_profile: f32,
}

impl Leaf {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            length: rng.gen_range(0.1..1.0),
            width: rng.gen_range(0.1..1.0),
            bend: rng.gen_range(0.0..0.5),
            bend_profile: rng.gen_range(0.5..5.0),
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, variance: f32) {
        self.length += rng.gen_range(-0.5..0.5) * variance;
        self.width += rng.gen_range(-0.5..0.5) * variance;
        self.bend += rng.gen_range(-0.25..0.25) * variance;
        self.bend_profile += rng.gen_range(-0.25..0.25) * variance;
    }

    pub fn view(&self, callback: &Callback<LeafMessage>) -> Html {
        html! {
            <>
                <div class="property">
                    { "Length" }
                    <Slider
                        min=0.1
                        max=1.0
                        value={ self.length }
                        oninput={ callback.reform(LeafMessage::SetLength) }
                    />
                </div>
                <div class="property">
                    { "Width" }
                    <Slider
                        min=0.1
                        max=1.0
                        value={ self.width }
                        oninput={ callback.reform(LeafMessage::SetWidth) }
                    />
                </div>
                <div class="property">
                    { "Bend" }
                    <Slider
                        min=0.0
                        max=0.5
                        value={ self.bend }
                        oninput={ callback.reform(LeafMessage::SetBend) }
                    />
                </div>
                <div class="property">
                    { "Bend Factor" }
                    <Slider
                        min=0.5
                        max=5.0
                        value={ self.bend_profile }
                        oninput={ callback.reform(LeafMessage::SetBendProfile) }
                    />
                </div>
            </>
        }
    }

    pub fn generate(&self, mesh: &mut Mesh, start: Vec3, direction: Vec3, up: Vec3) {
        let steps = 5;

        println!("");
        println!("up: {}", up);

        let right = up.cross(direction).normalize();
        let up = direction.cross(right).normalize();

        println!("dir: {}", direction);
        println!("right: {}", right);
        println!("up: {}", up);

        for i in 0..=steps {
            let x = i as f32 / steps as f32;

            let bend = x.powf(self.bend_profile) * self.bend;

            let width = (x * std::f32::consts::PI).sin() * self.width / 4.0;

            let a = start + direction * x * self.length + right * width - up * bend;
            let b = start + direction * x * self.length - right * width - up * bend;

            mesh.vertices.push(Vertex {
                position: a.into(),
                normal: [0.0; 3],
                uv: [0.0; 2],
            });

            mesh.vertices.push(Vertex {
                position: b.into(),
                normal: [0.0; 3],
                uv: [0.0; 2],
            });

            let l = mesh.vertices.len() as u32;

            if i > 0 {
                mesh.indices.push(l - 0);
                mesh.indices.push(l - 1);
                mesh.indices.push(l - 2);

                mesh.indices.push(l - 1);
                mesh.indices.push(l - 3);
                mesh.indices.push(l - 2);
            }
        }
    }
}

pub enum LeafMessage {
    SetLength(f32),
    SetWidth(f32),
    SetBend(f32),
    SetBendProfile(f32),
}

impl LeafMessage {
    pub fn handle(self, leaf: &mut Leaf) {
        match self {
            Self::SetLength(x) => leaf.length = x,
            Self::SetWidth(x) => leaf.width = x,
            Self::SetBend(x) => leaf.bend = x,
            Self::SetBendProfile(x) => leaf.bend_profile = x,
        }
    }
}
