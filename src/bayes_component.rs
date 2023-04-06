// bayes_component.rs
use crate::chance_component::percentize;
use crate::chance_component::Kind;
use crate::evidence_component::EvidenceCallback;
use crate::storage::export_to_markdown;
use crate::storage::parse_markdown;
use crate::storage::BayesData;
use wasm_bindgen::closure::Closure;
use web_sys::FileReader;

use crate::ChanceComponent;
use crate::EvidenceComponent;

use gloo_storage::{SessionStorage, Storage};
use wasm_bindgen::JsCast;
use yew::virtual_dom::AttrValue;

use yew::prelude::*;

pub fn likelihood(odds: Vec<f64>) -> f64 {
    odds[0] / (odds[0] + odds[1])
}

pub fn recalculate(prior: Vec<f64>, likelihoods: Vec<Vec<Vec<f64>>>) -> Vec<f64> {
    prior
        .into_iter()
        .enumerate()
        .map(|(hyp_idx, prior_val)| {
            likelihoods
                .iter()
                .fold(prior_val, |acc, ev| acc * likelihood(ev[hyp_idx].clone()))
        })
        .collect()
}

pub fn recalculate_to(prior: Vec<f64>, likelihoods: Vec<Vec<Vec<f64>>>, to: usize) -> Vec<f64> {
    prior
        .into_iter()
        .enumerate()
        .map(|(hyp_idx, prior_val)| {
            likelihoods
                .iter()
                .take(to)
                .fold(prior_val, |acc, ev| acc * likelihood(ev[hyp_idx].clone()))
        })
        .collect()
}

fn save_data(data: &BayesData) {
    let serialized = serde_json::to_string(&data).unwrap();
    SessionStorage::set("bayes_component", &serialized).unwrap();
}

#[derive(Debug)]
pub enum Msg {
    AddHypothesis,
    Prior(usize, Vec<f64>, Vec<AttrValue>),
    Evidence(usize, usize, Vec<f64>),
    EditEvidence(usize, String),
    AddEvidence,
    Posterior,
    Clear,
    Export,
    FileSelected(Option<web_sys::File>),
    FileContent(String),
}

#[derive(Properties, PartialEq, Eq)]
pub struct BayesProps {}

pub struct BayesComponent {
    pub data: BayesData,
    onload: Option<Closure<dyn FnMut(Event)>>,
    error_message: Option<String>,
}

impl Component for BayesComponent {
    type Message = Msg;
    type Properties = BayesProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        let mut data = BayesData {
            hypotheses: vec!["Hypothesis A".to_string(), "Hypothesis B".to_string()],
            prior_odds: vec![1.0, 1.0],
            posterior_odds: vec![50.0, 50.0],
            evidence: vec!["<evidence>".to_string()],
            likelihoods: vec![vec![vec![1.0, 1.0], vec![1.0, 1.0]]],
        };

        if let Ok(serialized) = SessionStorage::get::<String>("bayes_component") {
            if let Ok(loaded_data) = serde_json::from_str::<BayesData>(&serialized) {
                data = loaded_data;
            }
        }

        Self {
            data,
            onload: None,
            error_message: None,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        log::debug!("ev: {:?}", self.data.evidence);
        let onchange_prior = ctx
            .link()
            .callback(move |(u, e, h): (usize, Vec<f64>, Vec<AttrValue>)| Msg::Prior(u, e, h));
        let onchange_posterior = ctx
            .link()
            .callback(move |(_, _, _): (usize, Vec<f64>, Vec<AttrValue>)| Msg::Posterior);

        let onchange_add_hypothesis = ctx.link().callback(|_e: bool| Msg::AddHypothesis);

        let onclick_add_evidence = ctx.link().callback(move |_e: MouseEvent| Msg::AddEvidence);
        let onclick_export = ctx.link().callback(move |_e: MouseEvent| Msg::Export);
        let onclick_clear = ctx.link().callback(|_e: MouseEvent| Msg::Clear);

        let on_file_input_change = ctx.link().callback(|e: Event| {
            Msg::FileSelected(
                e.target_dyn_into::<web_sys::HtmlInputElement>()
                    .and_then(|input| input.files().unwrap().get(0)),
            )
        });

        let onchange_evidence = |ev_idx: usize| {
            ctx.link()
                .callback(move |evidence_msg: EvidenceCallback| match evidence_msg {
                    EvidenceCallback::OddsUpdate(hyp_idx, new_odds) => {
                        Msg::Evidence(ev_idx, hyp_idx, new_odds)
                    }
                    EvidenceCallback::LabelEdit(label) => Msg::EditEvidence(ev_idx, label),
                })
        };

        let hypotheses: _ = self
            .data
            .hypotheses
            .clone()
            .into_iter()
            .map(AttrValue::from)
            .collect::<Vec<AttrValue>>();
        let hypotheses2 = hypotheses.clone();

        let display_evidence = self.data.evidence.iter().enumerate().map(move |ev|
            html!{<EvidenceComponent
                prior_odds={recalculate_to(self.data.prior_odds.clone(), self.data.likelihoods.clone() ,ev.0)}
                hypotheses={hypotheses2.clone()}
                label={ev.1.clone()}
                onchange={&onchange_evidence(ev.0)}
                likelihoods = {self.data.likelihoods[ev.0].clone()}
                />
            });

        html! {
            <div class="container">
                <div style="position: absolute;left: 30px;top: 50px;font-size: 1.3rem;width: 120px;">
                    {"Press the '%' button to convert to percentages with the same ratio"}
                    <div class="menu">
                    <button class="clear-session" onclick={onclick_clear}>{"Clear"}</button>
                    <button class="export-markdown" onclick={onclick_export}>{"Export"}</button>
                    <label class="dropzone" for="fileInput">
                    <span>{"Load"}</span>
                    <input type="file" accept=".md" id="fileInput" onchange={on_file_input_change} style="display: none;" />
                </label>

                    {if let Some(ref error_message) = self.error_message {
                        html!{ <div class="invalid">{error_message}</div> }
                    } else {
                        html!{}
                    }}
                    </div>
                </div>

                <div class="main">
                    <div class="prior">
                        <div class="left">
                            <p>{"Without seeing any evidence, how often would you expect these possibilities?"}</p>
                        </div>
                        <div class="center">
                            <ChanceComponent onchange={onchange_prior} force_chance={Some(self.data.prior_odds.clone())}
                                hypotheses={hypotheses.clone()} onadd_hypothesis={onchange_add_hypothesis} kind={Kind::Prior}/>
                        </div>
                    </div>

                    {for display_evidence}
                    <div class ="center" style={format!("width:{}px",200*hypotheses.len())}>
                    <button class="add-evidence" onclick={onclick_add_evidence}>{"Add Evidence"}</button>
                    </div>

                    <div class="posterior">
                        <div class="left">
                            <p>{"Based on the evidence given, the updated percentages are: "}</p>
                        </div>
                        <div class="center">
                            <ChanceComponent onchange={onchange_posterior} force_chance={Some(self.data.posterior_odds.clone())}
                                hypotheses={hypotheses.clone()} kind={Kind::Posterior}/>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddHypothesis => {
                if self.data.hypotheses.len() == 5 {
                    return false;
                }
                self.data.hypotheses.push("hypo".to_string());
                self.data.prior_odds.push(1.0);
                self.data.posterior_odds.push(1.0);
                for ev_idx in 0..self.data.evidence.len() {
                    self.data.likelihoods[ev_idx].push(vec![1.0, 1.0]);
                }
            }
            Msg::Prior(idx, val, hyp) => {
                self.data.hypotheses[idx] = hyp[idx].to_string();
                self.data.prior_odds[idx] = val[idx];
            }
            Msg::Posterior => {}
            Msg::AddEvidence => {
                self.data.evidence.push("<evidence>".to_string());
                self.data
                    .likelihoods
                    .push(vec![vec![1.0, 1.0]; self.data.hypotheses.len()]);
            }
            Msg::Evidence(ev_idx, hyp_idx, new_odds) => {
                self.data.likelihoods[ev_idx][hyp_idx] = new_odds;
            }
            Msg::EditEvidence(ev_idx, new_evidence) => {
                self.data.evidence[ev_idx] = new_evidence;
            }
            Msg::Clear => {
                SessionStorage::delete("bayes_component");

                self.data = BayesData {
                    hypotheses: vec!["Hypothesis A".to_string(), "Hypothesis B".to_string()],
                    prior_odds: vec![1.0, 1.0],
                    posterior_odds: vec![50.0, 50.0],
                    evidence: vec!["<evidence>".to_string()],
                    likelihoods: vec![vec![vec![1.0, 1.0], vec![1.0, 1.0]]],
                };
            }
            Msg::Export => {
                export_to_markdown(&self.data);
            }
            Msg::FileSelected(file) => {
                let link = ctx.link().clone();
                let reader = FileReader::new().unwrap();
                let onload = Closure::wrap(Box::new(move |event: Event| {
                    let file_reader: FileReader = event.target().unwrap().unchecked_into();
                    let content = file_reader.result().unwrap().as_string().unwrap();
                    link.send_message(Msg::FileContent(content));
                }) as Box<dyn FnMut(Event)>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                reader.read_as_text(&file.unwrap()).unwrap();
                self.onload = Some(onload);
            }
            Msg::FileContent(content) => match parse_markdown(&content) {
                Ok(parsed_data) => {
                    self.data = parsed_data;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Error: Invalid file format. {:?}", e));
                }
            },
        }
        self.data.posterior_odds =
            recalculate(self.data.prior_odds.clone(), self.data.likelihoods.clone());
        self.data.posterior_odds = percentize(self.data.posterior_odds.clone());

        save_data(&self.data);
        true
    }
}
