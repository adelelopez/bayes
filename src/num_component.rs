use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

pub enum Msg {
    SetInput(f64),
    DoNothing,
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
                Ok(_)
                    if val_str.ends_with('.')
                        || (val_str.ends_with('0') && val_str.matches('.').count() == 1) =>
                {
                    Msg::DoNothing
                }
                Ok(val) => Msg::SetInput(val),
                Err(_) => match val_str.as_str() {
                    "" => Msg::Blank,
                    _ if val_str.ends_with('.') && val_str.matches('.').count() == 1 => {
                        Msg::DoNothing
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

        if ctx.props().display_only {
            let val = ctx.props().force_value.unwrap_or(self.value) as f32;
            html! {
                <div class={class_str}>
                {val}
                </div>
            }
        } else {
            let value = if self.editing {
                self.get_input_element()
                    .map(|input_element| input_element.value())
                    .unwrap_or_default()
            } else if self.is_blank {
                "".to_string()
            } else {
                ctx.props().force_value.unwrap_or(self.value).to_string()
            };

            html! {
                <input ref={input_ref} class={class_str}
                      value={value} onblur={onblur}
                    type="text" {oninput} placeholder={placeholder} />
            }
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        if ctx.props().force_value != None {
            self.value = ctx.props().force_value.unwrap();
        }
        true
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        if ctx.props().force_value != None {
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
            Msg::DoNothing => {
                self.editing = true;
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
