use crate::NumComponent;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::MouseEvent;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Properties, PartialEq)]
pub struct SliderProps {
    pub likelihood: f64,
}

pub struct SliderComponent {
    is_down: bool,
    slider_width: f64,
    likelihood: f64,
    raw_likelihood: f64,
    mouse_start_position: f64,
    slider_start_position: f64,
}

pub enum Msg {
    MouseDown(MouseEvent),
    MouseLeave,
    MouseUp,
    MouseMove(MouseEvent),
    Likelihood(f64),
}

impl Component for SliderComponent {
    type Message = Msg;
    type Properties = SliderProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            is_down: false,
            slider_width: 200.0,
            likelihood: ctx.props().likelihood,
            raw_likelihood: 0.5,
            mouse_start_position: 0.0,
            slider_start_position: 0.5,
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MouseDown(e) => {
                self.is_down = true;
                let target = e.target().unwrap().unchecked_into::<HtmlElement>();
                self.slider_width = target.offset_width() as f64;
                self.likelihood = sigmoid((self.raw_likelihood * 2.0) - 1.0);
                self.mouse_start_position = e.client_x() as f64;
                self.slider_start_position = self.raw_likelihood;
            }
            Msg::MouseLeave => {
                // self.is_down = false;
            }
            Msg::MouseUp => {
                self.is_down = false;
            }
            Msg::MouseMove(e) => {
                if !self.is_down {
                    return false;
                }
                e.prevent_default();
                let move_pos = e.client_x() as f64;
                let delta = move_pos - self.mouse_start_position;
                self.raw_likelihood =
                    self.slider_start_position + (0.3 * delta / self.slider_width);
                self.raw_likelihood = self.raw_likelihood.max(0.0).min(1.0);
                self.likelihood = sigmoid((self.raw_likelihood * 2.0) - 1.0);
            }
            Msg::Likelihood(_) => {}
        }
        log::debug!("{}", self.likelihood);
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onchange_odds = ctx
            .link()
            .callback(move |new_odds: f64| Msg::Likelihood(new_odds * 0.01));

        html! {
            <div class={"d0"}
            onmousedown={ctx.link().callback(Msg::MouseDown)}
            onmouseleave={ctx.link().callback(|_| Msg::MouseLeave)}
            onmouseup={ctx.link().callback(|_| Msg::MouseUp)}
            onmousemove={ctx.link().callback(Msg::MouseMove)}
            >
            <div class="before-bar">
                    <div class={"c0"} style={format!("width:{}%", 100.0*self.likelihood)}>
                    </div>
                    <div class="triangle-top"></div>
                    <div class="triangle-bot"></div>
                    <div class={"c1"} style={format!("width:{}%", 100.0*(1.0-self.likelihood))}>
                    </div>
                </div>
                <NumComponent min_value={0.0} max_value={100.0}
                force_value={Some(self.likelihood*100.0)} class={AttrValue::from("like")}
                placeholder={AttrValue::from("50")} onchange={&onchange_odds}
                />
                <div class="percent-symbol">
                <button class="no_button" >{"%"}</button>
                </div>
            </div>
        }
    }
}
