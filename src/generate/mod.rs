mod branch;
mod distribution;
mod leaf;

use glam::Vec3;
use rand::Rng;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::mesh::Mesh;

use self::{
    branch::{Branch, BranchMessage},
    distribution::{Distribution, DistributionMessage},
    leaf::{Leaf, LeafMessage},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlantDna {
    Leaf(Leaf),
    Branch(Branch),
    Distribution(Distribution),
}

impl PlantDna {
    pub fn new(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u32..3) {
            0 => Self::Leaf(Leaf::new(rng)),
            1 => Self::Branch(Branch::new(rng)),
            2 => Self::Distribution(Distribution::new(rng)),
            _ => unreachable!(),
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, variant: f32) {
        match self {
            Self::Leaf(leaf) => leaf.mutate(rng, variant),
            Self::Branch(branch) => branch.mutate(rng, variant),
            Self::Distribution(distribution) => distribution.mutate(rng, variant),
        }
    }

    pub fn view(&self, callback: &Callback<PlantMessage>) -> Html {
        let variant_html = match self {
            Self::Leaf(leaf) => leaf.view(&callback.reform(PlantMessage::Leaf)),
            Self::Branch(branch) => branch.view(&callback.reform(PlantMessage::Branch)),
            Self::Distribution(distribution) => {
                distribution.view(&callback.reform(PlantMessage::Distribution))
            }
        };

        let variant_name = match self {
            Self::Leaf(_) => "Leaf",
            Self::Branch(_) => "Branch",
            Self::Distribution(_) => "Distribution",
        };

        let opts = ["Leaf", "Branch", "Distribution"];

        let opts = opts.into_iter().map(|opt| {
            let onclick = callback.reform(move |_| PlantMessage::Base(String::from(opt)));

            html! {
                <option { onclick } selected={ opt == variant_name }>{ opt }</option>
            }
        });

        html! {
            <>
                <select>{ for opts }</select>
                <div class="container">
                    { variant_html }
                </div>
            </>
        }
    }

    pub fn generate_mesh(&self, mesh: &mut Mesh, start: Vec3, direction: Vec3, up: Vec3) {
        match self {
            Self::Leaf(leaf) => leaf.generate(mesh, start, direction, up),
            Self::Branch(branch) => branch.generate(mesh, start, direction, up),
            Self::Distribution(distribution) => distribution.generate(mesh, start, direction, up),
        }
    }
}

pub enum PlantMessage {
    Base(String),
    Leaf(LeafMessage),
    Branch(BranchMessage),
    Distribution(DistributionMessage),
}

impl PlantMessage {
    pub fn handle(self, rng: &mut impl Rng, plant: &mut PlantDna) {
        match (self, plant) {
            (Self::Base(base), plant) => match base.as_str() {
                "Leaf" => *plant = PlantDna::Leaf(Leaf::new(rng)),
                "Branch" => *plant = PlantDna::Branch(Branch::new(rng)),
                "Distribution" => *plant = PlantDna::Distribution(Distribution::new(rng)),
                _ => panic!("invalid base type"),
            },
            (Self::Leaf(msg), PlantDna::Leaf(leaf)) => msg.handle(leaf),
            (Self::Branch(msg), PlantDna::Branch(branch)) => msg.handle(branch),
            (Self::Distribution(msg), PlantDna::Distribution(distribution)) => {
                msg.handle(rng, distribution)
            }
            _ => panic!("invalid message"),
        }
    }
}
