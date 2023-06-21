// bayes_component.rs
use crate::chance_component::percentize;
use crate::chance_component::ChanceCallback;
use crate::chance_component::Kind;
use crate::evidence_component::EvidenceCallback;
use crate::storage::decode_bayes_data;
use crate::storage::encode_bayes_data;
use crate::storage::export_to_markdown;
use crate::storage::parse_markdown;
use crate::storage::BayesData;
use gloo::utils::document;
use js_sys::Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::FileReader;
use web_sys::HtmlElement;

use crate::ChanceComponent;
use crate::EvidenceComponent;
use crate::ModalComponent;

use gloo_storage::{SessionStorage, Storage};
use wasm_bindgen::JsCast;
use yew::virtual_dom::AttrValue;

use yew::prelude::*;

pub fn recalculate(prior: Vec<f64>, likelihoods: Vec<Vec<f64>>) -> Vec<f64> {
    prior
        .into_iter()
        .enumerate()
        .map(|(hyp_idx, prior_val)| {
            likelihoods
                .iter()
                .fold(prior_val, |acc, ev| acc * ev[hyp_idx])
        })
        .collect()
}

pub fn recalculate_to(prior: Vec<f64>, likelihoods: Vec<Vec<f64>>, to: usize) -> Vec<f64> {
    prior
        .into_iter()
        .enumerate()
        .map(|(hyp_idx, prior_val)| {
            likelihoods
                .iter()
                .take(to)
                .fold(prior_val, |acc, ev| acc * ev[hyp_idx])
        })
        .collect()
}

fn save_data(data: &BayesData) {
    let serialized = serde_json::to_string(&data).unwrap();
    SessionStorage::set("bayes_component", serialized).unwrap();
}

fn modify_class(elem: &HtmlElement, class_name: &str, add: bool) {
    let class_array = Array::new();
    class_array.push(&class_name.into());

    if add {
        elem.class_list()
            .add(&class_array)
            .expect("Error adding class");
    } else {
        elem.class_list()
            .remove(&class_array)
            .expect("Error removing class");
    }
}

fn update_bar_widths(
    elem: &HtmlElement,
    collapsed: bool,
    prior_odds: Vec<f64>,
    likelihoods: Vec<Vec<f64>>,
    ev: usize,
) {
    let p = percentize(recalculate_to(prior_odds.clone(), likelihoods.clone(), ev));
    let q = percentize(recalculate_to(prior_odds, likelihoods.clone(), ev + 1));
    for idx in 0..(elem.children().length() as usize) {
        let el = elem
            .children()
            .get_with_index(idx as u32)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let i = idx / 2;
        if !collapsed {
            if idx % 2 == 0 {
                modify_class(&el, "normalized", false);
                el.style()
                    .set_property("width", &format!("{}%", p[i] * likelihoods[ev][i]))
                    .expect("Error setting width");
            } else {
                modify_class(&el, "collapsed", false);
                el.style()
                    .set_property("width", &format!("{}%", p[i] * (1.0 - likelihoods[ev][i])))
                    .expect("Error setting width");
            }
        } else if idx % 2 == 0 {
            modify_class(&el, "normalized", true);
            el.style()
                .set_property("width", &format!("{}%", q[i]))
                .expect("Error setting width");
        } else {
            modify_class(&el, "collapsed", true);
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    AddHypothesis,
    Prior(usize, Vec<f64>, Vec<AttrValue>),
    Evidence(usize, usize, f64),
    EditEvidence(usize, String),
    AddEvidence,
    Posterior,
    Clear,
    Export,
    FileSelected(Option<web_sys::File>),
    FileContent(String),
    ToggleModal,
    GenerateLink,
    UpdateData(BayesData),
    ClearUrl,
    DeleteHypothesis(usize),
    DeleteEvidence(usize),
}

#[derive(Properties, PartialEq, Eq)]
pub struct BayesProps {}

pub struct BayesComponent {
    pub data: BayesData,
    onload: Option<Closure<dyn FnMut(Event)>>,
    error_message: Option<String>,
    is_modal_open: bool,
    _hashchange_listener: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

impl Component for BayesComponent {
    type Message = Msg;
    type Properties = BayesProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let mut data = BayesData {
            hypotheses: vec!["Hypothesis A".to_string(), "Hypothesis B".to_string()],
            prior_odds: vec![1.0, 1.0],
            posterior_odds: vec![50.0, 50.0],
            evidence: vec!["Evidence 1".to_string()],
            likelihoods: vec![vec![0.5, 0.5]],
        };

        if let Ok(serialized) = SessionStorage::get::<String>("bayes_component") {
            if let Ok(loaded_data) = serde_json::from_str::<BayesData>(&serialized) {
                data = loaded_data;
            }
        }

        let location = web_sys::window().unwrap().location();
        if let Ok(hash) = location.hash() {
            if !hash.is_empty() {
                let encoded_data = &hash[1..];
                if let Ok(decoded_data) = decode_bayes_data(encoded_data) {
                    data = decoded_data;
                }
            }
        }

        let link = ctx.link().clone();
        let hashchange_listener = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let url = web_sys::window().unwrap().location().href().unwrap();
            let encoded_data = url.split('#').last().unwrap_or("");
            let decoded_data = decode_bayes_data(encoded_data);

            if let Ok(loaded_data) = decoded_data {
                link.send_message(Msg::UpdateData(loaded_data));
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(
                "hashchange",
                hashchange_listener.as_ref().unchecked_ref(),
            )
            .unwrap();

        Self {
            data,
            onload: None,
            error_message: None,
            is_modal_open: true,
            _hashchange_listener: Some(hashchange_listener),
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onchange_prior =
            ctx.link()
                .callback(move |chance_msg: ChanceCallback| match chance_msg {
                    ChanceCallback::EditHypothesis(u, e, h) => Msg::Prior(u, e, h),
                    ChanceCallback::Delete(u) => Msg::DeleteHypothesis(u),
                });
        let onchange_posterior = ctx.link().callback(move |_: ChanceCallback| Msg::Posterior);

        let onchange_add_hypothesis = ctx.link().callback(|_e: bool| Msg::AddHypothesis);

        let onclick_add_evidence = ctx.link().callback(move |_e: MouseEvent| Msg::AddEvidence);
        let onclick_export = ctx.link().callback(move |_e: MouseEvent| Msg::Export);
        let onclick_clear = ctx.link().callback(|_e: MouseEvent| Msg::Clear);
        let onclick_help = ctx.link().callback(|_e: MouseEvent| Msg::ToggleModal);
        let onclick_generate_link = ctx.link().callback(|_e: MouseEvent| Msg::GenerateLink);

        let toggle_modal = ctx.link().callback(|_| Msg::ToggleModal);

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
                    EvidenceCallback::Delete => Msg::DeleteEvidence(ev_idx),
                })
        };

        let prior_odds = self.data.prior_odds.clone();
        let likelihoods = self.data.likelihoods.clone();

        let onmousemove = move |e: MouseEvent| {
            let elements = document().get_elements_by_class_name("bart");

            for i in 0..elements.length() {
                if let Some(elem) = elements.item(i) {
                    let elem = elem.dyn_into::<HtmlElement>().unwrap();
                    let elem_rect = elem.get_bounding_client_rect();

                    let above_cursor = (e.client_y() as f64) < elem_rect.top();
                    if above_cursor {
                        update_bar_widths(
                            &elem,
                            false,
                            prior_odds.clone(),
                            likelihoods.clone(),
                            i as usize,
                        );
                    } else {
                        update_bar_widths(
                            &elem,
                            true,
                            prior_odds.clone(),
                            likelihoods.clone(),
                            i as usize,
                        );
                    }
                }
            }
        };

        let hypotheses = self
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
                last = {ev.0 == self.data.evidence.len() -1 }
                />
            });

        html! {
            <div class="container" onmousemove={onmousemove}>
                    <div class="menu">
                    <b><a href="">{"bayescalc.io"}</a></b>
                    <button class="clear-session" onclick={onclick_help}>{"Help"}</button>
                    <button class="clear-session" onclick={onclick_clear}>{"Clear"}</button>
                    <button class="clear-session" onclick={onclick_generate_link}>{"Link"}</button>
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

                <ModalComponent
                is_open={self.is_modal_open}
                on_close={toggle_modal}
                />

                <div class="main">
                    <div class="prior">
                        <div class="left">
                        <p> {"Prior"}</p>
                        </div>
                        <div class="center">
                            <ChanceComponent onchange={onchange_prior} force_chance={Some(self.data.prior_odds.clone())}
                                hypotheses={hypotheses.clone()} onadd_hypothesis={onchange_add_hypothesis} kind={Kind::Prior}/>
                        </div>
                    </div>

                    {for display_evidence}
                    {if hypotheses.is_empty() {
                        html!(  <div style={format!("width:{}px",400)}>
                        <button class="add-evidence" onclick={onclick_add_evidence}>{"Add Evidence"}</button>
                        </div>)
                    } else {
                        html!(
                    <div class ="center" style={format!("width:{}px",200*hypotheses.len())}>
                    <button class="add-evidence" onclick={onclick_add_evidence}>{"Add Evidence"}</button>
                    </div>)
                    }}

                    <div class="posterior">
                        <div class="left">
                        <p> {"Posterior"}</p>
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
                self.data.hypotheses.push(format!(
                    "Hypothesis {}",
                    (b'A' + self.data.hypotheses.len() as u8) as char
                ));
                self.data.prior_odds.push(1.0);
                self.data.posterior_odds.push(1.0);
                for ev_idx in 0..self.data.evidence.len() {
                    self.data.likelihoods[ev_idx].push(0.5);
                }
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::DeleteHypothesis(hyp_idx) => {
                self.data.hypotheses.remove(hyp_idx);
                self.data.prior_odds.remove(hyp_idx);
                self.data.posterior_odds.remove(hyp_idx);
                for ev_idx in 0..self.data.evidence.len() {
                    self.data.likelihoods[ev_idx].remove(hyp_idx);
                }
            }
            Msg::Prior(idx, val, hyp) => {
                self.data.hypotheses[idx] = hyp[idx].to_string();
                self.data.prior_odds[idx] = val[idx];
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::Posterior => {}
            Msg::AddEvidence => {
                self.data
                    .evidence
                    .push(format!("Evidence {}", self.data.evidence.len() + 1));
                self.data
                    .likelihoods
                    .push(vec![0.5; self.data.hypotheses.len()]);
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::DeleteEvidence(ev_idx) => {
                self.data.evidence.remove(ev_idx);
                self.data.likelihoods.remove(ev_idx);
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::Evidence(ev_idx, hyp_idx, new_odds) => {
                self.data.likelihoods[ev_idx][hyp_idx] = new_odds;
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::EditEvidence(ev_idx, new_evidence) => {
                self.data.evidence[ev_idx] = new_evidence;
                ctx.link().send_message(Msg::ClearUrl);
            }
            Msg::Clear => {
                SessionStorage::delete("bayes_component");
                ctx.link().send_message(Msg::ClearUrl);

                self.data = BayesData {
                    hypotheses: vec!["Hypothesis A".to_string(), "Hypothesis B".to_string()],
                    prior_odds: vec![1.0, 1.0],
                    posterior_odds: vec![50.0, 50.0],
                    evidence: vec!["Evidence 1".to_string()],
                    likelihoods: vec![vec![0.5, 0.5]],
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
                    ctx.link().send_message(Msg::ClearUrl);
                }
                Err(e) => {
                    self.error_message = Some(format!("Error: Invalid file format. {:?}", e));
                }
            },
            Msg::ToggleModal => {
                self.is_modal_open = !self.is_modal_open;
            }
            Msg::GenerateLink => {
                let encoded = encode_bayes_data(&self.data).unwrap();
                let url = format!(
                    "{}#{}",
                    document().location().unwrap().href().unwrap(),
                    encoded
                );

                let window = web_sys::window().unwrap();
                let history = window.history().unwrap();
                history
                    .push_state_with_url(&JsValue::NULL, "", Some(&url))
                    .unwrap();
            }
            Msg::UpdateData(new_data) => {
                self.data = new_data;
            }
            Msg::ClearUrl => {
                let url = web_sys::window().unwrap().location().href().unwrap();
                if url.contains('#') && url.split('#').last().is_some() {
                    let window = web_sys::window().unwrap();
                    let history = window.history().unwrap();
                    history
                        .replace_state_with_url(&JsValue::NULL, "", Some("/"))
                        .unwrap_or_else(|err| {
                            log::error!("Failed to clear URL: {:?}", err);
                        });
                }
            }
        }
        self.data.posterior_odds =
            recalculate(self.data.prior_odds.clone(), self.data.likelihoods.clone());
        self.data.posterior_odds = percentize(self.data.posterior_odds.clone());

        save_data(&self.data);
        true
    }
}
