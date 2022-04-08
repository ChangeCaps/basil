mod components;
mod generate;
mod mesh;
mod texture;

use generate::{PlantDna, PlantMessage};
use rand::{prelude::StdRng, Rng, SeedableRng};
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
    KeepDna,
    SelectDna(usize),
}

pub struct App {
    pub rng: StdRng,
    pub current_dna: PlantDna,
    pub dna_options: [PlantDna; 7],
}

impl App {
    pub fn plant_mutations<const T: usize>(dna: &PlantDna, rng: &mut impl Rng) -> [PlantDna; T] {
        let mutations = (0..T).into_iter().map(|_| {
            let mut dna = dna.clone();
            dna.mutate(rng, 0.2);
            dna
        });

        TryFrom::try_from(mutations.collect::<Vec<_>>()).unwrap()
    }
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut rng = StdRng::seed_from_u64(42069);

        let current_dna = PlantDna::new(&mut rng);

        let dna_options = Self::plant_mutations(&current_dna, &mut rng);

        Self {
            current_dna,
            rng,
            dna_options: TryFrom::try_from(dna_options).unwrap(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::ChangeCurrentDna(msg) => {
                msg.handle(&mut self.rng, &mut self.current_dna);
                let dna_options = Self::plant_mutations(&self.current_dna, &mut self.rng);
                self.dna_options = dna_options;
            }
            Message::KeepDna => {
                let dna_options = Self::plant_mutations(&self.current_dna, &mut self.rng);
                self.dna_options = dna_options;
            }
            Message::SelectDna(idx) => {
                let dna_options = Self::plant_mutations(&self.dna_options[idx], &mut self.rng);
                self.current_dna = self.dna_options[idx].clone();
                self.dna_options = dna_options;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let plant_options = self.dna_options.iter().enumerate().map(|(i, dna)| {
            html! {
                <div
                    class="plant-option"
                    onclick={ ctx.link().callback(move |_| Message::SelectDna(i)) }
                >
                    <PlantViewer
                        rotation=0.0
                        dna={ dna.clone() }
                    />
                </div>
            }
        });

        html! {
            <>
                <div class="top-bar">
                    <div class="basil-logo">
                        <a href="/index.html">{ "Basil" }</a>
                    </div>
                </div>

                <div class="dna-options">
                    <DnaOptions
                        dna={ self.current_dna.clone() }
                        change_dna={ ctx.link().callback(Message::ChangeCurrentDna) }
                    />
                </div>

                <div class="main-view">
                    <div class="plant-options">
                        <div
                            class="plant-option"
                            onclick={ ctx.link().callback(|_| Message::KeepDna) }
                        >
                            <PlantViewer
                                rotation=0.0
                                dna={ self.current_dna.clone() }
                            />
                        </div>
                        { for plant_options }
                    </div>
                </div>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
