// chance_component.rs
use crate::label_component::LabelCallback;
use crate::LabelComponent;
use crate::NumComponent;
use is_close::all_close;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

pub fn percentize(odds: Vec<f64>) -> Vec<f64> {
    let total = odds.iter().sum::<f64>();
    odds.iter().map(|x| 100.0 * x / total).collect()
}

fn pair_sum(idx: usize, odds: &[f64]) -> f64 {
    odds[idx] + odds[idx + 1]
}

pub enum Msg {
    Odds(usize, f64),
    PriorSlide(usize, f64, f64),
    Percentize,
    AddHypothesis,
    EditHypothesis(usize, String),
    Delete(usize),
    DoNothing,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Prior,
    Posterior,
}

#[derive(Properties, PartialEq)]
pub struct ChanceProps {
    pub onchange: Callback<ChanceCallback>,
    #[prop_or(None)]
    pub onadd_hypothesis: Option<Callback<bool>>,
    #[prop_or(None)]
    pub force_chance: Option<Vec<f64>>,
    pub hypotheses: Vec<AttrValue>,
    #[prop_or(Kind::Prior)]
    pub kind: Kind,
    pub color: Vec<usize>,
}

pub enum ChanceCallback {
    Delete(usize),
    EditHypothesis(usize, Vec<f64>, Vec<AttrValue>),
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
        let odds = self.odds.clone();

        let is_percent = if all_close!(self.odds.clone(), percentize(self.odds.clone())) {
            "no_button"
        } else {
            ""
        };

        let onclick_add_hypothesis = {
            ctx.link()
                .callback(move |_e: MouseEvent| Msg::AddHypothesis)
        };

        let display_hypotheses = ctx.props().hypotheses.iter().enumerate().map(|(idx, hyp)| {
            html! {
                <LabelComponent
                class={AttrValue::from(format!("b{} c{} hyp", ctx.props().color[idx],idx))}
                placeholder={AttrValue::from(hyp.clone())}
                onchange={ctx.link().callback(move |label_change: LabelCallback| match label_change {
                    LabelCallback::Delete => Msg::Delete(idx),
                    LabelCallback::LabelEdit(label) => Msg::EditHypothesis(idx, label),
                })}
                display_only={ctx.props().kind != Kind::Prior}
                deleteable={true}
                />
            }
        });

        let display_odds = ctx.props().hypotheses.iter().enumerate().map(|(idx, _)| {
            html! {
                <div class={format!("b{} c{}", ctx.props().color[idx], idx)}>
                    <NumComponent min_value={0.0} max_value={None}
                    force_value={self.force_odds[idx]} class={AttrValue::from("odds")}
                    placeholder={AttrValue::from("1")} onchange={ctx.link().callback(move |odds: f64| Msg::Odds(idx, odds))}
                    display_only={ctx.props().kind == Kind::Posterior}/>
                    <div class="percent">
                        <button class={is_percent} onclick={ctx.link().callback(move |_e: MouseEvent| Msg::Percentize)}>{"%"}</button>
                    </div>
                </div>
            }
        });

        let display_bar = ctx.props().hypotheses.iter().enumerate().map(|(idx, _)| {
            let onslide = {
                let percents = percents.clone();
                let odds = odds.clone();
                move |hyp_idx: usize| {
                    ctx.link().callback(move |e: InputEvent| {
                        e.stop_propagation();
                        let input_el: HtmlInputElement = e.target_unchecked_into();
                        let val_str = input_el.value();
                        let total_odds: f64 = odds.iter().sum();

                        let subtotal = pair_sum(hyp_idx, &percents.clone());
                        match val_str.parse::<f64>() {
                            Ok(val) => Msg::PriorSlide(hyp_idx, total_odds*subtotal*val*0.0001,total_odds*subtotal*(100.0-val)*0.0001),
                            Err(_) => Msg::DoNothing,
                        }
                    })
                }
            };

            html!{
                <>
                if ctx.props().kind == Kind::Prior {
                    <div class={format!("b{} c{}", ctx.props().color[idx], idx)} style={format!("width:{}%", percents[idx])}>
                        if idx < ctx.props().hypotheses.len() - 1 {
                            <input type="range" min=0.0
                            max={100.0}
                            step={0.1}
                            value={AttrValue::from((percents[idx]/pair_sum(idx, &percents)*100.0).to_string())}
                            class="prior-slider" oninput={onslide(idx)}
                            style={format!("width: {}px", pair_sum(idx, &percents)*2.0 *(ctx.props().hypotheses.len() as f64))}
                            />
                        }
                    </div>
                    if idx < ctx.props().hypotheses.len() - 1 {
                        <div class="triangle-bot"></div>
                    }
                } else {
                    <div class={format!("b{} c{}", ctx.props().color[idx], idx)} style={format!("width:{}%", percents[idx])}></div>
                }
                </>
            }
        });

        let cols = ctx.props().hypotheses.len();
        let col_str = format!("{}px", cols * 100);

        let style = format!("display: grid; width: {};", col_str);

        html! {
            <div style="display:flex">
            <div>
            <div style={style}>
            {for display_hypotheses}
            {for display_odds}
            </div>

            <div class="prior-bar" style={format!("width:{}px",200*cols)}>

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

                onchange.emit(ChanceCallback::EditHypothesis(
                    idx,
                    self.odds.clone(),
                    self.hypotheses.clone(),
                ));

                true
            }
            Msg::PriorSlide(idx, val1, val2) => {
                log::debug!("i {} {} {}", idx, val1, val2);
                if val1 < 0.0 {
                    return true;
                }
                self.odds[idx] = val1;
                self.odds[idx + 1] = val2;
                self.force_odds[idx] = None;
                self.force_odds[idx + 1] = None;

                onchange.emit(ChanceCallback::EditHypothesis(
                    idx,
                    self.odds.clone(),
                    self.hypotheses.clone(),
                ));

                onchange.emit(ChanceCallback::EditHypothesis(
                    idx + 1,
                    self.odds.clone(),
                    self.hypotheses.clone(),
                ));

                true
            }
            Msg::Percentize => {
                self.odds = percentize(self.odds.clone());
                self.force_odds = self.odds.clone().into_iter().map(Some).collect();
                for idx in 0..self.odds.len() {
                    onchange.emit(ChanceCallback::EditHypothesis(
                        idx,
                        self.odds.clone(),
                        self.hypotheses.clone(),
                    ));
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
                onchange.emit(ChanceCallback::EditHypothesis(
                    idx,
                    self.odds.clone(),
                    self.hypotheses.clone(),
                ));
                true
            }
            Msg::Delete(idx) => {
                onchange.emit(ChanceCallback::Delete(idx));
                true
            }
            Msg::DoNothing => false,
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
