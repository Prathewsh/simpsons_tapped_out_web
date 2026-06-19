use wasm_bindgen::prelude::*;

mod game;
mod renderer;
mod input;
mod assets;
mod sprite;
mod utils;

pub use game::Game;

#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn create_game(canvas_id: &str) -> Result<Game, JsValue> {
    Game::new(canvas_id)
}

/// Get the list of texture paths Homer needs, as JSON
#[wasm_bindgen]
pub fn homer_texture_manifest() -> String {
    let paths = sprite::homer_texture_paths();
    let entries: Vec<String> = paths
        .iter()
        .map(|(key, path)| format!(r#"{{"key":"{}","path":"{}"}}"#, key, path))
        .collect();
    format!("[{}]", entries.join(","))
}

/// Get the list of texture paths Bart needs, as JSON
#[wasm_bindgen]
pub fn bart_texture_manifest() -> String {
    let paths = sprite::bart_texture_paths();
    let entries: Vec<String> = paths
        .iter()
        .map(|(key, path)| format!(r#"{{"key":"{}","path":"{}"}}"#, key, path))
        .collect();
    format!("[{}]", entries.join(","))
}

