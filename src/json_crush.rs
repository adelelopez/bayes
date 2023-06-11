use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = JSONCrush)]
    pub fn crush(input: &str) -> String;

    #[wasm_bindgen(js_namespace = JSONCrush)]
    pub fn uncrush(input: &str) -> String;
}
