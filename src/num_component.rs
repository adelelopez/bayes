use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

pub enum Msg {
    SetInput(f64),
    SoftInput(f64),
    Blank,
    Blur,
}

#[derive(Properties, PartialEq, Debug)]
pub struct NumProps {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub force_value: Option<f64>,
    pub placeholder: AttrValue,
    pub class: AttrValue,
    pub onchange: Callback<f64>,
    #[prop_or(false)]
    pub display_only: bool,
}

#[derive(Debug)]
pub struct NumComponent {
    input_ref: NodeRef,
    value: f64,
    is_blank: bool,
    editing: bool,
}

fn is_int_value(num: f32) -> bool {
    let rounded = num.round();
    (num - rounded).abs() <= std::f32::EPSILON
}

fn is_slider_value(num: f32) -> bool {
    let rounded = (num * 10.0).round() / 10.0;
    (num - rounded).abs() <= std::f32::EPSILON
}

impl Component for NumComponent {
    type Message = Msg;
    type Properties = NumProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            input_ref: NodeRef::default(),
            value: ctx.props().force_value.unwrap_or(1.0),
            is_blank: false,
            editing: false,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let cur_val = self.value;
        let class = &ctx.props().class;
        let onblur = ctx.link().callback(|_: FocusEvent| Msg::Blur);

        let oninput = ctx.link().callback(move |e: InputEvent| {
            let input_el: HtmlInputElement = e.target_unchecked_into();
            let cursor_position = input_el.selection_start().unwrap_or(Some(0));
            let val_str = input_el.value();

            let msg = match val_str.parse() {
                Ok(val) => Msg::SoftInput(val),
                Err(_) => match val_str.as_str() {
                    "" => Msg::Blank,
                    _ if val_str.matches('.').count() == 1
                        && val_str.chars().all(|c| c.is_numeric() || c == '.') =>
                    {
                        Msg::SoftInput(cur_val)
                    }
                    _ => Msg::SetInput(cur_val),
                },
            };

            if let Msg::SetInput(_) = msg {
                if let Some(pos) = cursor_position {
                    gloo_timers::callback::Timeout::new(0, move || {
                        input_el.set_selection_range(pos, pos).ok();
                    })
                    .forget();
                }
            }
            msg
        });

        let input_ref = self.input_ref.clone();
        let placeholder = ctx.props().placeholder.clone();

        let invalid = ctx.props().min_value.map_or(false, |min| min > cur_val)
            || ctx.props().max_value.map_or(false, |max| max < cur_val);
        let class_str = class.to_string() + if invalid { " invalid" } else { "" };

        let val = ctx.props().force_value.unwrap_or(self.value) as f32;

        if ctx.props().display_only {
            html! {
                <div class={class_str}>
                {format!("{:.*}", 5, val)}
                </div>
            }
        } else {
            let value = if self.editing {
                self.get_input_element()
                    .map(|input_element| input_element.value())
                    .unwrap_or_default()
            } else if self.is_blank {
                "".to_string()
            } else if is_int_value(val) {
                val.round().to_string()
            } else if !is_slider_value(val) {
                val.to_string()
            } else {
                format_num::format_num!("#.1f", val)
            };
            html! {
                <input ref={input_ref} class={class_str}
                      value={value} onblur={onblur}
                    type="text" {oninput} placeholder={placeholder} />
            }
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        if ctx.props().force_value.is_some() {
            self.value = ctx.props().force_value.unwrap();
        }
        true
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        if ctx.props().force_value.is_some() {
            self.value = ctx.props().force_value.unwrap();
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.is_blank = false;
        match msg {
            Msg::SetInput(val) => {
                self.value = val;
                self.editing = false;
                self.is_blank = false;
                ctx.props().onchange.emit(val);
                true
            }
            Msg::SoftInput(val) => {
                self.value = val;
                self.editing = true;
                self.is_blank = true;
                ctx.props().onchange.emit(val);
                true
            }
            Msg::Blur => {
                self.editing = false;
                true
            }
            Msg::Blank => {
                self.value = 1.0;
                self.is_blank = true;
                self.editing = true;
                ctx.props().onchange.emit(1.0);
                true
            }
        }
    }
}

impl NumComponent {
    fn get_input_element(&self) -> Option<HtmlInputElement> {
        self.input_ref.cast::<HtmlInputElement>()
    }
}
