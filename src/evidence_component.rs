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
    pub color: Vec<usize>,
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
            bayes_factors: log_odds_in_db(normalize(ctx.props().likelihoods.clone())),
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
                    <div class={format!("b{} d{} c0", ctx.props().color[odds.0], odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0]*self.likelihoods[odds.0])}>

                    </div>
                    <div class={format!("a{} d{} c1", ctx.props().color[odds.0], odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0]*(1.0-self.likelihoods[odds.0]))}>

                    </div>
                </>
                });

        let display_hypothesis_evidence = ctx.props().hypotheses.iter().enumerate().map(move |hypotheses|
            html!{
            <div class={format!("d{}", hypotheses.0)}>
                <div class="evidence-center">
                    <NumComponent min_value={0.0} max_value={100.0}
                    force_value={Some(self.likelihoods[hypotheses.0]*100.0)} class={AttrValue::from("like")}
                    placeholder={AttrValue::from("50")} onchange={&onchange_odds(hypotheses.0)}
                    />
                    <div class="percent-symbol">
                       <button class="no_button" >{"%"}</button>
                    </div>
                </div>
                <input type="range" min=0.0 max=1.0 step={0.001} value={AttrValue::from((self.likelihoods[hypotheses.0]).to_string())} class="slider" ontouchmove={ontouchmove(hypotheses.0)} oninput={onslide(hypotheses.0)} />
                <div class="before-bar">
                    <div class={format!("b{} c0",ctx.props().color[hypotheses.0])} style={format!("width:{}%", 100.0*self.likelihoods[hypotheses.0])}>
                    </div>
                    <div class="triangle-top"></div>
                    <div class="triangle-bot"></div>
                    <div class={format!("a{} c1",ctx.props().color[hypotheses.0])} style={format!("width:{}%", 100.0*(1.0-self.likelihoods[hypotheses.0]))}>
                    </div>
                </div>
            </div>
         });

        let (max_index, _) = self.bayes_factors.iter().enumerate().fold(
            (Some(0), Some(f64::NEG_INFINITY)),
            |(max_idx, max_val), (idx, &val)| match val.partial_cmp(&max_val.unwrap()) {
                Some(std::cmp::Ordering::Greater) => (Some(idx), Some(val)),
                Some(std::cmp::Ordering::Equal) => (None, Some(val)),
                _ => (max_idx, max_val),
            },
        );

        let display_log_odds = if let Some(max_idx) = max_index {
            html! {
                <div class={format!("e{idx}", idx=ctx.props().color[max_idx])}>
                <b>
                {format_num::format_num!("+.1f", self.bayes_factors[max_idx])}{" db"}
                </b>
                </div>
            }
        } else {
            html! {
                <div class="eblank">
                <b>
                {"0 db"}
                </b>
                </div>
            }
        };
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
                <div class="log-odds mobile">
                {display_log_odds.clone()}
                </div>

                </div>
            </div>

            <div class="all-sliders">
            <div style={format!(r#"display: grid; grid-template-columns: {}; align-items: center;"#, col_str)}>
            // <SliderComponent likelihood={self.likelihoods[0]} />

            {for display_hypothesis_evidence}
            </div>
            </div>

            <div class="log-odds">
            {display_log_odds}
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
