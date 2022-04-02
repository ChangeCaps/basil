mod components;
mod generate;
mod mesh;
mod texture;

use generate::{PlantDna, PlantMessage};
use rand::{prelude::StdRng, SeedableRng};
use yew::prelude::*;

use crate::components::{DnaOptions, PlantViewer};

#[macro_export]
macro_rules! println {
    ($($tt:tt)*) => {
        {
            ::web_sys::console::log_1(&format!($($tt)*).into());
        }
    };
}

pub enum Message {
    ChangeCurrentDna(PlantMessage),
}

pub struct App {
    pub rng: StdRng,
    pub current_dna: PlantDna,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut rng = StdRng::seed_from_u64(42069);

        Self {
            current_dna: PlantDna::new(&mut rng),
            rng,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::ChangeCurrentDna(msg) => {
                msg.handle(&mut self.rng, &mut self.current_dna);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="top-bar">
                    <div class="basil-logo">
                        <a href="/index.html">{ "Basil" }</a>
                    </div>
                </div>

                <div class="main-view">
                    <div class="dna-options">
                        <DnaOptions
                            dna={ self.current_dna.clone() }
                            change_dna={ ctx.link().callback(Message::ChangeCurrentDna) }
                        />
                    </div>

                    <PlantViewer dna={ self.current_dna.clone() } />
                </div>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
