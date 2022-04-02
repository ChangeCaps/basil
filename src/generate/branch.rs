use glam::Vec3;
use rand::Rng;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::mesh::Mesh;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Branch {}

impl Branch {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {}
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, variant: f32) {}

    pub fn view(&self, callback: &Callback<BranchMessage>) -> Html {
        html! {}
    }

    pub fn generate(&self, mesh: &mut Mesh, start: Vec3, direction: Vec3, up: Vec3) {
        let steps = 5;

        let right = direction.cross(up).normalize();
        let up = right.cross(direction).normalize();
    }
}

pub enum BranchMessage {}

impl BranchMessage {
    pub fn handle(self, branch: &mut Branch) {
        match self {}
    }
}
