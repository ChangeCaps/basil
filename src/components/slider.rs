use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Properties {
    #[prop_or_else(|| 0.0)]
    pub min: f32,
    #[prop_or_else(|| 1.0)]
    pub max: f32,
    #[prop_or_else(|| 0.5)]
    pub value: f32,
    #[prop_or_else(|| 100000)]
    pub steps: i32,
    #[prop_or_default]
    pub oninput: Callback<f32>,
}

impl Properties {
    fn range(&self) -> f32 {
        self.max - self.min
    }

    fn value_i32(&self) -> i32 {
        let x = (self.value - self.min) / self.range();
        (x * self.steps as f32) as i32
    }
}

pub struct Slider {
    node_ref: NodeRef,
}

impl Component for Slider {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let steps = ctx.props().steps as f32;
        let min = ctx.props().min;
        let range = ctx.props().range();
        let node_ref = self.node_ref.clone();
        let changed = ctx.props().oninput.reform(move |_| {
            let element = node_ref.cast::<HtmlInputElement>().unwrap();

            element.value_as_number() as f32 / steps * range + min
        });

        html! {
            <input
                class="slider"
                type="range"
                ref={ &self.node_ref }
                min=0
                max={ ctx.props().steps.to_string() }
                value={ ctx.props().value_i32().to_string() }
                oninput={ &changed }
            />
        }
    }
}
