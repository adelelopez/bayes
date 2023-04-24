// chance_component.rs
use crate::LabelComponent;
use crate::NumComponent;
use is_close::all_close;
use stylist::css;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

// pub fn normalize(odds: Vec<f64>) -> Vec<f64> {
//     let total = odds.iter().sum::<f64>();
//     odds.iter().map(|x| x / total).collect()
// }

pub fn percentize(odds: Vec<f64>) -> Vec<f64> {
    let total = odds.iter().sum::<f64>();
    odds.iter().map(|x| 100.0 * x / total).collect()
}

pub enum Msg {
    Odds(usize, f64),
    Percentize,
    AddHypothesis,
    EditHypothesis(usize, String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Prior,
    Evidence,
    Posterior,
}

#[derive(Properties, PartialEq)]
pub struct ChanceProps {
    pub onchange: Callback<(usize, Vec<f64>, Vec<AttrValue>)>,
    #[prop_or(None)]
    pub onadd_hypothesis: Option<Callback<bool>>,
    #[prop_or(None)]
    pub force_chance: Option<Vec<f64>>,
    pub hypotheses: Vec<AttrValue>,
    #[prop_or(Kind::Prior)]
    pub kind: Kind,
}

pub struct ChanceComponent {
    prev_num_hypotheses: usize,
    hypotheses: Vec<AttrValue>,
    odds: Vec<f64>,
    force_odds: Vec<Option<f64>>,
}

impl Component for ChanceComponent {
    type Message = Msg;
    type Properties = ChanceProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let length = ctx.props().hypotheses.len();
        let init_val = if ctx.props().kind == Kind::Posterior {
            100.0 / length as f64
        } else {
            1.0
        };
        let init_force = if ctx.props().kind == Kind::Posterior {
            if ctx.props().force_chance.is_some() {
                ctx.props()
                    .force_chance
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(Some)
                    .collect()
            } else {
                vec![Some(100.0 / length as f64); length]
            }
        } else {
            vec![None; length]
        };
        let init_odds = if ctx.props().force_chance.is_some() {
            ctx.props().force_chance.clone().unwrap()
        } else {
            vec![init_val; length]
        };
        let mut instance = Self {
            prev_num_hypotheses: length,
            hypotheses: ctx.props().hypotheses.clone(),
            odds: init_odds,
            force_odds: init_force,
        };

        instance.update_force_chance(&ctx.props().force_chance);

        instance
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let percents = percentize(self.odds.clone());
        let is_percent = if all_close!(self.odds.clone(), percentize(self.odds.clone())) {
            "no_button"
        } else {
            ""
        };

        let onchange_odds = |idx: usize| ctx.link().callback(move |odds: f64| Msg::Odds(idx, odds));
        let onchange_hypothesis = |idx: usize| {
            ctx.link()
                .callback(move |label: String| Msg::EditHypothesis(idx, label))
        };

        let onclick_percentize =
            |_: usize| ctx.link().callback(move |_e: MouseEvent| Msg::Percentize);
        let onclick_add_hypothesis = {
            ctx.link()
                .callback(move |_e: MouseEvent| Msg::AddHypothesis)
        };

        let display_hypotheses = ctx.props().hypotheses.iter().enumerate().map(move |hyp| {
            html! {
                <LabelComponent
                class={AttrValue::from(format!("c{idx} hyp", idx=hyp.0))}
                placeholder={AttrValue::from(hyp.1.clone())}
                onchange={&onchange_hypothesis(hyp.0)}
                display_only={ctx.props().kind != Kind::Prior}
                />
            }
        });

        let display_odds = ctx.props().hypotheses.iter().enumerate().map(move |odds| {
            html! {<div class={format!("c{idx}", idx=odds.0)}>
            <NumComponent min_value={0.0} max_value={None}
            force_value={self.force_odds[odds.0]} class={AttrValue::from("odds")}
            placeholder={AttrValue::from("1")} onchange={&onchange_odds(odds.0)}
            display_only={ctx.props().kind == Kind::Posterior}/>
            <div class="percent">
            <button class={is_percent} onclick={onclick_percentize(odds.0)}>{"%"}</button>
            </div>  </div>}
        });

        let display_bar = ctx.props().hypotheses.iter().enumerate().map(move |odds|
            html!{
                <div class={format!("c{idx}", idx=odds.0)} style={format!("width:{}%", percents[odds.0])}>
                </div>
        });

        let cols = if ctx.props().kind == Kind::Evidence {
            2
        } else {
            2 * (ctx.props().hypotheses.len())
        };
        let col_str = format!("{}px", cols * 100);

        html! {
            <div style="display:flex">
            <div>
            <div class={css!(r#"display: grid; width: ${cols};"#, cols=col_str)}>
            {for display_hypotheses}
            {for display_odds}
            </div>

            <div class={css!(r#"display: flex;"#)} style={format!("height: 20px; width:{}px",100*cols)}>
            {for display_bar}
            </div>
            </div>
            if ctx.props().kind == Kind::Prior && ctx.props().hypotheses.len() < 5 {
                <button class="add-hypothesis" onclick={onclick_add_hypothesis} >{"+"}</button>
            }

            </div>
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        if ctx.props().force_chance
            != Some(
                self.force_odds
                    .iter()
                    .filter_map(|x| *x)
                    .collect::<Vec<f64>>(),
            )
        {
            self.update_force_chance(&ctx.props().force_chance);
        }
        if self.odds.len() < ctx.props().hypotheses.len() {
            self.force_odds = self.odds.clone().into_iter().map(Some).collect();
            self.odds.push(1.0);
            self.force_odds.push(None);
        }
        self.prev_num_hypotheses = ctx.props().hypotheses.len();
        self.hypotheses = ctx.props().hypotheses.clone();

        true
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        if self.odds.len() != ctx.props().hypotheses.len() {
            self.force_odds = self.odds.clone().into_iter().map(Some).collect();
            self.odds.push(1.0);
            self.force_odds.push(Some(1.0));
        }
        for idx in 0..self.force_odds.len() {
            if self.force_odds[idx].is_some() {
                self.odds[idx] = self.force_odds[idx].unwrap()
            }
        }

        self.hypotheses = ctx.props().hypotheses.clone();
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let onchange = ctx.props().onchange.clone();
        self.hypotheses = ctx.props().hypotheses.clone();

        match msg {
            Msg::Odds(idx, val) => {
                if val < 0.0 {
                    return true;
                }
                self.odds[idx] = val;
                self.force_odds[idx] = None;

                onchange.emit((idx, self.odds.clone(), self.hypotheses.clone()));

                true
            }
            Msg::Percentize => {
                self.odds = percentize(self.odds.clone());
                self.force_odds = self.odds.clone().into_iter().map(Some).collect();
                for idx in 0..self.odds.len() {
                    onchange.emit((idx, self.odds.clone(), self.hypotheses.clone()));
                }
                true
            }
            Msg::AddHypothesis => {
                if ctx.props().onadd_hypothesis.is_some() {
                    ctx.props().onadd_hypothesis.clone().unwrap().emit(true);
                }
                true
            }
            Msg::EditHypothesis(idx, label) => {
                self.hypotheses[idx] = AttrValue::from(label);
                onchange.emit((idx, self.odds.clone(), self.hypotheses.clone()));
                true
            }
        }
    }
}

impl ChanceComponent {
    fn update_force_chance(&mut self, force_chance: &Option<Vec<f64>>) {
        if let Some(force_chance) = force_chance {
            self.odds = force_chance.clone();
            self.force_odds = self.odds.clone().into_iter().map(Some).collect();
        }
    }
}
