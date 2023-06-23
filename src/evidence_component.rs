// evidence_component.rs
use crate::chance_component::percentize;
use crate::label_component::LabelCallback;
use crate::LabelComponent;
// use crate::SliderComponent;
use crate::NumComponent;
use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew::virtual_dom::AttrValue;

#[derive(Properties, PartialEq)]
pub struct EvidenceProps {
    pub hypotheses: Vec<AttrValue>,
    pub label: String,
    pub onchange: Callback<EvidenceCallback>,
    pub prior_odds: Vec<f64>,
    pub likelihoods: Vec<f64>,
    #[prop_or(false)]
    pub last: bool,
}

pub struct EvidenceComponent {
    pub evidence: AttrValue,
    pub likelihoods: Vec<f64>,
    pub bayes_factors: Vec<f64>,
}

pub enum Msg {
    EditLabel(AttrValue),
    Likelihood(usize, f64),
    Delete,
    DoNothing,
}

pub enum EvidenceCallback {
    OddsUpdate(usize, f64),
    LabelEdit(String),
    Delete,
}

pub fn normalize(odds: Vec<f64>) -> Vec<f64> {
    let total = odds.iter().sum::<f64>();
    odds.iter().map(|x| x / total).collect()
}

pub fn log_odds_in_db(odds: Vec<f64>) -> Vec<f64> {
    normalize(odds)
        .iter()
        .map(|x| 10.0 * (x / (1.0 - x)).log(10.0))
        .collect()
}

impl Component for EvidenceComponent {
    type Message = Msg;
    type Properties = EvidenceProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            evidence: AttrValue::from(ctx.props().label.clone()),
            bayes_factors: normalize(ctx.props().likelihoods.clone())
                .iter()
                .map(|x| (x / (1.0 - x)).log(10.0))
                .collect(),
            likelihoods: ctx.props().likelihoods.clone(),
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onchange_label =
            ctx.link()
                .callback(move |label_change: LabelCallback| match label_change {
                    LabelCallback::Delete => Msg::Delete,
                    LabelCallback::LabelEdit(label) => Msg::EditLabel(AttrValue::from(label)),
                });

        let onchange_odds = move |hyp_idx: usize| {
            ctx.link()
                .callback(move |new_odds: f64| Msg::Likelihood(hyp_idx, new_odds * 0.01))
        };

        let ontouchmove = move |hyp_idx: usize| {
            ctx.link().callback(move |e: TouchEvent| {
                e.prevent_default();
                let input_el: HtmlInputElement = e.target_unchecked_into();
                let val_str = input_el.value();

                match val_str.parse::<f64>() {
                    Ok(val) => Msg::Likelihood(hyp_idx, val),
                    Err(_) => Msg::DoNothing,
                }
            })
        };

        let onslide = move |hyp_idx: usize| {
            ctx.link().callback(move |e: InputEvent| {
                let input_el: HtmlInputElement = e.target_unchecked_into();
                let val_str = input_el.value();

                match val_str.parse::<f64>() {
                    Ok(val) => Msg::Likelihood(hyp_idx, val),
                    Err(_) => Msg::DoNothing,
                }
            })
        };

        let prior_odds_percent = percentize(ctx.props().prior_odds.clone());

        let display_after_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
            html!{
                <>
                    <div class={format!("d{idx} c0", idx=odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0]*self.likelihoods[odds.0])}>

                    </div>
                    <div class={format!("d{idx} c1", idx=odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0]*(1.0-self.likelihoods[odds.0]))}>

                    </div>
                </>
                });

        let display_hypothesis_evidence = ctx.props().hypotheses.iter().enumerate().map(move |hypotheses|
            html!{
            <div class={format!("d{idx}", idx=hypotheses.0)}>
                <NumComponent min_value={0.0} max_value={100.0}
                force_value={Some(self.likelihoods[hypotheses.0]*100.0)} class={AttrValue::from("like")}
                placeholder={AttrValue::from("50")} onchange={&onchange_odds(hypotheses.0)}
                />
                <input type="range" min=0.0 max=1.0 step={0.001} value={AttrValue::from((self.likelihoods[hypotheses.0]).to_string())} class="slider" ontouchmove={ontouchmove(hypotheses.0)} oninput={onslide(hypotheses.0)} />
                <div class="before-bar">
                    <div class={"c0"} style={format!("width:{}%", 100.0*self.likelihoods[hypotheses.0])}>
                    </div>
                    <div class="triangle-top"></div>
                    <div class="triangle-bot"></div>
                    <div class={"c1"} style={format!("width:{}%", 100.0*(1.0-self.likelihoods[hypotheses.0]))}>
                    </div>
                </div>
                <div class="percent-symbol">
                <button class="no_button" >{"%"}</button>
                </div>
            </div>
         });

        // let display_log_odds = ctx.props().hypotheses.iter().enumerate().map(move |hypothesis|
        //     html!{
        //         <div class= {format!("e{idx}", idx=hypothesis.0)}>
        //         <b>
        //         { format_num::format_num!("+.1f", self.bayes_factors.clone()[hypothesis.0])}{" db"}
        //         </b>
        //         </div>
        //     });

        let col_str = "200px ".repeat(10);
        html! {
            <div class="evidence-item">
            <div class = "left">
                <div class = "ev">
                <LabelComponent
                    class={AttrValue::from("evidence")}
                placeholder={AttrValue::from(self.evidence.clone())}
                onchange={&onchange_label}
                deleteable={ctx.props().last}
                />
            //    <div class="log-odds">
            //    {for display_log_odds}
            //    </div>
                </div>
            </div>

            <div class="all-sliders">
            <div style={format!(r#"display: grid; grid-template-columns: {}; align-items: center;"#, col_str)}>
            // <SliderComponent likelihood={self.likelihoods[0]} />

            {for display_hypothesis_evidence}
            </div>
            </div>


            <div class="after-bar" style={format!("width:{}px",200*ctx.props().hypotheses.len())}>
            <div class="bart">
            {for display_after_bar}
            </div>
            </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());
        self.bayes_factors = log_odds_in_db(self.likelihoods.clone());
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());
        self.bayes_factors = log_odds_in_db(self.likelihoods.clone());
        true
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());

        while self.likelihoods.len() < ctx.props().hypotheses.len() {
            self.likelihoods.push(0.5);
        }
        self.bayes_factors = log_odds_in_db(self.likelihoods.clone());

        match msg {
            Msg::EditLabel(val) => {
                self.evidence = val.clone();
                ctx.props()
                    .onchange
                    .emit(EvidenceCallback::LabelEdit(val.to_string()));
                true
            }

            Msg::Likelihood(hyp_idx, new_odds) => {
                self.likelihoods[hyp_idx] = new_odds;
                self.bayes_factors = log_odds_in_db(self.likelihoods.clone());
                ctx.props()
                    .onchange
                    .emit(EvidenceCallback::OddsUpdate(hyp_idx, new_odds));
                true
            }
            Msg::Delete => {
                ctx.props().onchange.emit(EvidenceCallback::Delete);
                true
            }

            Msg::DoNothing => false,
        }
    }
}
