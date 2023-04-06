use bayes_component::BayesComponent;
use chance_component::ChanceComponent;
use evidence_component::EvidenceComponent;
use label_component::LabelComponent;
use num_component::NumComponent;
use yew::prelude::*;

mod bayes_component;
mod chance_component;
mod evidence_component;
mod label_component;
mod num_component;
mod storage;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <BayesComponent/>
        </>
    }
}
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("App is starting");
    yew::start_app::<App>();
}
