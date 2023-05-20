use yew::prelude::*;
use comrak::{markdown_to_html, ComrakOptions};

#[derive(Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub content: String,
    pub is_open: bool,
    pub on_close: Callback<()>,
}

pub enum Msg {
    ToggleModal,
}

pub struct ModalComponent {
    is_open: bool,
}

impl Component for ModalComponent {
    type Message = Msg;
    type Properties = ModalProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        ModalComponent {
            is_open: ctx.props().is_open,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.is_open = !self.is_open;
                ctx.props().on_close.emit(());
                true
            }
        }
    }


    fn view(&self, ctx: &yew::Context<Self>) -> Html {

        let document = web_sys::window().unwrap().document().unwrap();
let container = document.create_element("div").unwrap();
container.set_inner_html(&markdown_to_html(&ctx.props().content, &ComrakOptions::default()));
let node = yew::virtual_dom::VNode::VRef(container.into());

        if self.is_open {
            html! {
                <div class="modal">
                <div class="modal-content">
                    <span class="close-button" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>{ "✕" }</span>
                    {node}
                </div>
            </div>
            }
        } else {
            html! {}
        }
    }
}
