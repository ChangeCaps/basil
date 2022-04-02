use yew::prelude::*;

use crate::generate::{PlantDna, PlantMessage};

#[derive(PartialEq, Properties)]
pub struct Properties {
    pub dna: PlantDna,
    pub change_dna: Callback<PlantMessage>,
}

pub struct DnaOptions {}

impl Component for DnaOptions {
    type Message = ();
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        ctx.props().dna.view(&ctx.props().change_dna)
    }
}
