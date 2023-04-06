// evidence_component.rs
use crate::chance_component::{percentize, Kind};
use crate::ChanceComponent;
use crate::LabelComponent;
use stylist::css;

use yew::prelude::*;
use yew::virtual_dom::AttrValue;


#[derive(Properties, PartialEq)]
pub struct EvidenceProps {
    pub hypotheses: Vec<AttrValue>,
    pub label: String,
    pub onchange: Callback<EvidenceCallback>,
    pub prior_odds: Vec<f64>,
    pub likelihoods: Vec<Vec<f64>>,
}

pub struct EvidenceComponent {
    pub evidence: AttrValue,
    pub likelihoods: Vec<Vec<f64>>,
    pub bayes_factor: f64,
    pub label_ref: NodeRef,
}

pub enum Msg {
    EditLabel(AttrValue),
    Likelihood(usize, usize, Vec<f64>),
}

pub enum EvidenceCallback {
    OddsUpdate(usize, Vec<f64>),
    LabelEdit(String)
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

        let onchange_label =  ctx.link().callback(move |label: String| Msg::EditLabel(AttrValue::from(label)));
        let onchange = ctx.props().onchange.clone();
        
        let onchange_odds = move |hyp_idx: usize| {
            ctx.link()
                .callback(move |(idx, new_odds, _): (usize, Vec<f64>, Vec<AttrValue>)| {
                    onchange.emit(EvidenceCallback::OddsUpdate(hyp_idx, new_odds.clone()));
                    Msg::Likelihood(hyp_idx, idx, new_odds)
                })
        };

        let prior_odds = ctx.props().prior_odds.clone();
        let prior_odds_percent = percentize(prior_odds.clone());
        let display_before_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
            html!{
                <div class={format!("c{idx}", idx=odds.0)} style={format!("width:{}%", prior_odds_percent[odds.0])}>
                {" "}
            </div>
            });

        let prior_odds_percent2 = percentize(prior_odds.clone());

        let display_after_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
            html!{
                <>
                    <div class={format!("d{idx} c0", idx=odds.0)} style={format!("width:{}%", prior_odds_percent2[odds.0]*percentize(self.likelihoods[odds.0].clone())[0])}>
                    {" "}
                    </div>
                    <div class={format!("d{idx} c1", idx=odds.0)} style={format!("width:{}%", prior_odds_percent2[odds.0]*percentize(self.likelihoods[odds.0].clone())[1])}>
                    {" "}
                    </div>
    
                </>
                });

        let display_hypothesis_evidence = ctx.props().hypotheses.iter().enumerate().map(move |hypotheses|
            html!{
            <>
            <div class={format!("d{idx}", idx=hypotheses.0)}>
            <ChanceComponent 
                onchange={&onchange_odds.clone()(hypotheses.0)}
                hypotheses={vec!(AttrValue::from("if"), AttrValue::from("if not"))} kind={Kind::Evidence}
                force_chance={Some(ctx.props().likelihoods[hypotheses.0].clone())}
            />
            </div>
            </>
         });

        let col_str = "100px ".repeat(10);
        html! {
            <div class="evidence">

            // <div class="before-bar" style={format!("width:{}px",200*ctx.props().hypotheses.len())}>
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
                {"For each hypothesis, how often would you expect this evidence given all of the above evidence?"}
            </div>

            <div class={css!(r#"display: flex; grid-template-columns: 100px ${cols}; align-items: center;"#, cols=col_str)}>

            {for display_hypothesis_evidence}
            </div>

            // <div class="before-bar" style={format!("width:{}px",200*ctx.props().hypotheses.len())}>
            // {for display_after_bar}
            // </div>

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
            self.likelihoods.push(vec![1.0, 1.0]);
        }

        match msg {
            Msg::EditLabel(val) => {          
                self.evidence = val.clone();
                ctx.props().onchange.emit(EvidenceCallback::LabelEdit(val.to_string()));
                true
            }
            Msg::Likelihood(hyp_idx, idx, new_odds) => {
                self.likelihoods[hyp_idx][idx] = new_odds[idx];           
                true
            }
        }
    }
}
