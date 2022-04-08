use glam::Vec3;
use yew::prelude::*;

use super::MeshViewer;
use crate::{
    generate::PlantDna,
    mesh::{Mesh, SharedMesh},
    texture::{SharedTexture, Texture},
};

#[derive(PartialEq, Properties)]
pub struct Properties {
    pub dna: PlantDna,
    pub rotation: Option<f32>,
    #[prop_or_default]
    pub on_rotate: Callback<f32>,
}

pub struct PlantViewer {
    pub dna: PlantDna,
    pub mesh: SharedMesh,
    pub texture: SharedTexture,
}

impl Component for PlantViewer {
    type Message = ();
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let mut mesh = Mesh::default();
        ctx.props().dna.generate_mesh(
            &mut mesh,
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, -0.01).normalize(),
            Vec3::Y,
        );
        mesh.calculate_normals();

        let texture = Texture::white();

        Self {
            dna: ctx.props().dna.clone(),
            mesh: SharedMesh::new(mesh),
            texture: SharedTexture::new(texture),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.dna != ctx.props().dna {
            *self = Self::create(ctx);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <MeshViewer
                mesh={ self.mesh.clone() }
                texture={ self.texture.clone() }
                rotation={ ctx.props().rotation }
                on_rotate={ ctx.props().on_rotate.clone() }
            />
        }
    }
}
