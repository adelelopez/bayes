// evidence_component.rs
use crate::chance_component::percentize;
use crate::LabelComponent;
use crate::NumComponent;
use web_sys::HtmlInputElement;

use stylist::css;

use yew::prelude::*;
use yew::virtual_dom::AttrValue;

#[derive(Properties, PartialEq)]
pub struct EvidenceProps {
    pub hypotheses: Vec<AttrValue>,
    pub label: String,
    pub onchange: Callback<EvidenceCallback>,
    pub prior_odds: Vec<f64>,
    pub likelihoods: Vec<f64>,
}

pub struct EvidenceComponent {
    pub evidence: AttrValue,
    pub likelihoods: Vec<f64>,
    pub bayes_factor: f64,
    pub label_ref: NodeRef,
}

pub enum Msg {
    EditLabel(AttrValue),
    Likelihood(usize, f64),

    DoNothing,
}

pub enum EvidenceCallback {
    OddsUpdate(usize, f64),
    LabelEdit(String),
}

impl Component for EvidenceComponent {
    type Message = Msg;
    type Properties = EvidenceProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            evidence: AttrValue::from(ctx.props().label.clone()),
            bayes_factor: 1.0,
            likelihoods: ctx.props().likelihoods.clone(),
            label_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onchange_label = ctx
            .link()
            .callback(move |label: String| Msg::EditLabel(AttrValue::from(label)));

        let onchange_odds = move |hyp_idx: usize| {
            ctx.link()
                .callback(move |new_odds: f64| Msg::Likelihood(hyp_idx, new_odds * 0.01))
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

        let prior_odds = ctx.props().prior_odds.clone();
        // let prior_odds_percent = percentize(prior_odds.clone());
        // let display_before_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
        //     html!{
        //         <div class={format!("c{idx}", idx=odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0])}>
        //         {" "}
        //     </div>
        //     });

        let prior_odds_percent2 = percentize(prior_odds);

        let display_after_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
            html!{
                <>
                    <div class={format!("d{idx} c0", idx=odds.0)} style={format!("width:{}%", prior_odds_percent2[odds.0]*self.likelihoods[odds.0])}>
                    {" "}
                    </div>
                    <div class={format!("d{idx} c1", idx=odds.0)} style={format!("width:{}%", prior_odds_percent2[odds.0]*(1.0-self.likelihoods[odds.0]))}>
                    {" "}
                    </div>
                </>
                });

        let display_hypothesis_evidence = ctx.props().hypotheses.iter().enumerate().map(move |hypotheses|
            html!{
            <>
            <div class={format!("d{idx}", idx=hypotheses.0)}>
                <NumComponent min_value={0.0} max_value={100.0}
                force_value={Some(self.likelihoods[hypotheses.0]*100.0)} class={AttrValue::from("like")}
                placeholder={AttrValue::from("50")} onchange={&onchange_odds(hypotheses.0)}
                />
                <input type="range" min=0.0 max=1.0 step={0.001} value={AttrValue::from((self.likelihoods[hypotheses.0]).to_string())} class="slider" oninput={onslide(hypotheses.0)} />
                <div class="before-bar">
                    <div class={"c0"} style={format!("width:{}%", 100.0*self.likelihoods[hypotheses.0])}>
                    {" "}
                    </div>
                    <div class={"c1"} style={format!("width:{}%", 100.0*(1.0-self.likelihoods[hypotheses.0]))}>
                    {" "}
                    </div>
                </div>
                <div class="percent-symbol">
                <button class="no_button" >{"%"}</button>
                </div>
            </div>
            </>
         });

        let col_str = "200px ".repeat(10);
        html! {
            <div class="evidence">

            // <div class="after-bar" style={format!("width:{}px",200*ctx.props().hypotheses.len())}>
            // {for display_before_bar}
            // </div>

            <div class = "left">
                <div class = "ev">
                <LabelComponent
                    class={AttrValue::from("evidence")}
                placeholder={AttrValue::from(self.evidence.clone())}
                onchange={&onchange_label}
                node_ref={Some(self.label_ref.clone())}
                focus_on_mount=true

                />
                </div>
            </div>

            <div class={css!(r#"display: grid; grid-template-columns: ${cols}; align-items: center;"#, cols=col_str)}>

            {for display_hypothesis_evidence}
            </div>

            <div class="after-bar" style={format!("width:{}px",200*ctx.props().hypotheses.len())}>
            {for display_after_bar}
            </div>

            </div>
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());
        true
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.likelihoods = ctx.props().likelihoods.clone();
        self.evidence = AttrValue::from(ctx.props().label.clone());

        while self.likelihoods.len() < ctx.props().hypotheses.len() {
            self.likelihoods.push(0.5);
        }

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
                ctx.props()
                    .onchange
                    .emit(EvidenceCallback::OddsUpdate(hyp_idx, new_odds));
                true
            }

            Msg::DoNothing => false,
        }
    }
}
