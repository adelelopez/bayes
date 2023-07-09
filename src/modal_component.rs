use comrak::{markdown_to_html, ComrakOptions};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
}

pub enum Msg {
    ToggleModal,
    // NextFile,
    // PrevFile,
    ReceiveMarkdown(String),
}

pub struct ModalComponent {
    is_open: bool,
    markdown_content: Option<String>,
    // file_index: i32,
}

impl ModalComponent {
    async fn fetch_markdown(_file_name: &str) -> Result<String, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let request = Request::new_with_str_and_init("./tutorial/file1.md", &opts)?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into().unwrap();
        let text = JsFuture::from(resp.text()?).await?;
        let markdown = text.as_string().unwrap();

        Ok(markdown)
    }
}

impl Component for ModalComponent {
    type Message = Msg;
    type Properties = ModalProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let component = ModalComponent {
            is_open: ctx.props().is_open,
            markdown_content: None,
            // file_index: 1,
        };
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match ModalComponent::fetch_markdown("file1").await {
                Ok(markdown) => link.send_message(Msg::ReceiveMarkdown(markdown)),
                Err(err) => log::debug!("{:?}", err),
            }
        });

        component
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.is_open = ctx.props().is_open;
        true
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.is_open = !self.is_open;
                ctx.props().on_close.emit(());
                true
            }
            // Msg::NextFile => {
            //     // self.update_markdown(ctx, 1);
            //     true
            // }
            // Msg::PrevFile => {
            //     if self.file_index > 1 {
            //         // self.update_markdown(ctx, -1);
            //     }
            //     true
            // }
            Msg::ReceiveMarkdown(content) => {
                self.markdown_content = Some(content);
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let binding = String::new();
        let markdown_content = self.markdown_content.as_ref().unwrap_or(&binding);

        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_inner_html(&markdown_to_html(
            markdown_content,
            &ComrakOptions::default(),
        ));
        let node = yew::virtual_dom::VNode::VRef(container.into());

        if self.is_open {
            html! {
                <div class="modal">
                <div class="modal-content">
                    <span class="close-button" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>{ "✕" }</span>
                    <article>
                        {node}
                    </article>
                    // <button onclick={ctx.link().callback(|_| Msg::PrevFile)}>{ "Prev" }</button>
                    // <button onclick={ctx.link().callback(|_| Msg::NextFile)}>{ "Next" }</button>
                </div>
            </div>
            }
        } else {
            html! {}
        }
    }
}
