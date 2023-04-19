use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

pub enum Msg {
    SetInput(AttrValue),
    Blur,
    EditMode,
    // Delete,
}

#[derive(Properties, PartialEq)]
pub struct LabelProps {
    pub placeholder: AttrValue,
    pub class: AttrValue,
    pub onchange: Callback<String>,
    #[prop_or(false)]
    pub display_only: bool,
    #[prop_or(false)]
    pub focus_on_mount: bool,
    pub node_ref: Option<NodeRef>,
}

pub struct LabelComponent {
    input_ref: NodeRef,
    value: AttrValue,
    edit_mode: bool,
    should_focus: bool,
}

impl Component for LabelComponent {
    type Message = Msg;
    type Properties = LabelProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        LabelComponent {
            input_ref: NodeRef::default(),
            value: ctx.props().placeholder.clone(),
            edit_mode: false,
            should_focus: ctx.props().focus_on_mount,
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.value = ctx.props().placeholder.clone();
        true
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, _first_render: bool) {
        self.value = ctx.props().placeholder.clone();

        if self.should_focus {
            if let Some(input_el) = self.input_ref.cast::<HtmlInputElement>() {
                let _ = input_el.focus();
            }
            self.should_focus = false;
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetInput(value) => {
                self.value = value.clone();
                ctx.props().onchange.emit(value.to_string());
                true
            }
            Msg::Blur => {
                self.edit_mode = false;
                true
            }
            Msg::EditMode => {
                self.edit_mode = true;
                self.should_focus = true;
                true
            } // Msg::Delete => {
              //     self.value = "".into();
              //     true
              // }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input_el: HtmlInputElement = e.target_unchecked_into();
            let val_str = input_el.value();
            Msg::SetInput(AttrValue::from(val_str))
        });

        let onblur = ctx.link().callback(|_: FocusEvent| Msg::Blur);
        let onclick_edit = ctx.link().callback(|_: MouseEvent| Msg::EditMode);
        // let onclick_delete = ctx.link().callback(|_: MouseEvent| Msg::Delete);

        let value = self.value.clone();

        let input_ref = match &ctx.props().node_ref {
            Some(node_ref) => node_ref.clone(),
            None => self.input_ref.clone(),
        };

        html! {
            <div class={ctx.props().class.to_string()}>
                {if self.edit_mode {
                    html! {
                        <>
                            <input
                                class="label-input"
                                type="text"
                                value={value.clone()}
                                oninput={oninput}
                                onblur={onblur}
                                ref={input_ref}
                            />
                        </>
                    }
                } else {
                    html! {
                        <>
                            <div class="label">{value.clone()}</div>
                            if !ctx.props().display_only {
                            <button class="label-button-edit" onclick={onclick_edit}>{"✎"}</button>
                            // <button class="label-button-x" onclick={onclick_delete}>{"✕"}</button>
                            }
                        </>
                    }
                }}
            </div>
        }
    }
}