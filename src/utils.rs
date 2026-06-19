use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        $crate::utils::log(&format!($($t)*))
    };
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        $crate::utils::error(&format!($($t)*))
    };
}
