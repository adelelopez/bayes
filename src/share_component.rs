use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

#[wasm_bindgen(inline_js = "
    export function writeToClipboard(text) {
        return navigator.clipboard.writeText(text);
    }
")]
extern "C" {
    fn writeToClipboard(text: &str) -> js_sys::Promise;
}

#[derive(Properties, PartialEq, Clone)]
pub struct ShareProps {
    pub link: AttrValue,
    pub show: bool,
    pub on_close: Callback<()>,
}

pub enum Msg {
    Hide,
    Copy,
    LinkCopied(Result<(), JsValue>),
}

pub struct ShareComponent {
    link: String,
    show: bool,
    copy_success: bool,
}

impl Component for ShareComponent {
    type Message = Msg;
    type Properties = ShareProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            link: ctx.props().link.to_string(),
            show: ctx.props().show,
            copy_success: false,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Hide => {
                self.show = false;
                ctx.props().on_close.emit(());
                self.copy_success = false;
            }

            Msg::Copy => {
                let link = self.link.clone();
                let callback = ctx.link().callback(Msg::LinkCopied);
                let future = JsFuture::from(writeToClipboard(&link));
                wasm_bindgen_futures::spawn_local(async move {
                    let result = future.await.map(|_| ());
                    callback.emit(result);
                });
                self.copy_success = false;
            }

            Msg::LinkCopied(result) => match result {
                Ok(_) => {
                    self.copy_success = true;
                    log::info!("Copied to clipboard successfully");
                }
                Err(err) => {
                    log::error!("Failed to copy to clipboard: {:?}", err);
                }
            },
        }
        true
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.show = ctx.props().show;
        self.link = ctx.props().link.to_string();
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let link_text = self.link.clone();
        let onblur = ctx.link().callback(|_: FocusEvent| Msg::Hide);
        if self.show {
            html! {
                <div class="share-link" onblur={onblur} >
                <span class="close-button" onclick={ctx.link().callback(|_| Msg::Hide)}>{ "✕" }</span>
                <div class="link-container">
                    <input type="text" class="link-text" value={ link_text } readonly=true/>
                    if self.copy_success {

                        <button class="copy-button" onclick={ctx.link().callback(|_| Msg::Copy)}>{ "Copied ✅" }</button>

                    } else {

                    <button class="copy-button" onclick={ctx.link().callback(|_| Msg::Copy)}>{ "Copy" }</button>

                    }


                </div>
            </div>

            }
        } else {
            html! {}
        }
    }
}
