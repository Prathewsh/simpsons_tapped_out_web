/// Placeholder asset registry — will map string keys to loaded GPU textures
pub struct AssetRegistry {
    // textures: HashMap<String, WebGlTexture>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        AssetRegistry {}
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
