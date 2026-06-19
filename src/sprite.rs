use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlImageElement, WebGl2RenderingContext as GL, WebGlTexture};
use std::collections::HashMap;

/// A single loaded texture with dimensions
pub struct Texture {
    pub gl_texture: WebGlTexture,
    pub width: u32,
    pub height: u32,
}

/// One frame of animation
pub struct Frame {
    pub texture_key: String,
    pub duration_ms: f64,
}

/// A named animation sequence
pub struct Animation {
    pub name: String,
    pub frames: Vec<Frame>,
    pub looping: bool,
}

/// Tracks which animation is playing and the current frame
pub struct AnimationState {
    pub current_animation: String,
    pub frame_index: usize,
    pub elapsed_ms: f64,
}

impl AnimationState {
    pub fn new(animation: &str) -> Self {
        AnimationState {
            current_animation: animation.to_string(),
            frame_index: 0,
            elapsed_ms: 0.0,
        }
    }

    pub fn set_animation(&mut self, name: &str) {
        if self.current_animation != name {
            self.current_animation = name.to_string();
            self.frame_index = 0;
            self.elapsed_ms = 0.0;
        }
    }

    pub fn update(&mut self, delta_ms: f64, animations: &HashMap<String, Animation>) {
        let Some(anim) = animations.get(&self.current_animation) else {
            return;
        };
        if anim.frames.is_empty() {
            return;
        }

        self.elapsed_ms += delta_ms;
        let frame = &anim.frames[self.frame_index];

        if self.elapsed_ms >= frame.duration_ms {
            self.elapsed_ms -= frame.duration_ms;
            self.frame_index += 1;
            if self.frame_index >= anim.frames.len() {
                if anim.looping {
                    self.frame_index = 0;
                } else {
                    self.frame_index = anim.frames.len() - 1;
                }
            }
        }
    }

    pub fn current_texture_key<'a>(&self, animations: &'a HashMap<String, Animation>) -> Option<&'a str> {
        let anim = animations.get(&self.current_animation)?;
        let frame = anim.frames.get(self.frame_index)?;
        Some(&frame.texture_key)
    }
}

/// Load a texture from an already-loaded HtmlImageElement
pub fn upload_image_to_texture(gl: &GL, img: &HtmlImageElement) -> Result<Texture, JsValue> {
    let texture = gl.create_texture().ok_or("failed to create texture")?;
    gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

    gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);

    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

    gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
        GL::TEXTURE_2D,
        0,
        GL::RGBA as i32,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        img,
    )?;

    Ok(Texture {
        gl_texture: texture,
        width: img.natural_width(),
        height: img.natural_height(),
    })
}

/// Build Homer's animation definitions
pub fn homer_animations() -> HashMap<String, Animation> {
    let mut anims = HashMap::new();

    anims.insert("idle".to_string(), Animation {
        name: "idle".to_string(),
        frames: vec![
            Frame { texture_key: "homer_idle_1".to_string(), duration_ms: 2000.0 },
            Frame { texture_key: "homer_idle_blink_1".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "homer_idle_blink_2".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "homer_idle_1".to_string(), duration_ms: 3000.0 },
            Frame { texture_key: "homer_idle_blink_1".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "homer_idle_blink_2".to_string(), duration_ms: 100.0 },
        ],
        looping: true,
    });

    anims.insert("walk_front".to_string(), Animation {
        name: "walk_front".to_string(),
        frames: (1..=4).map(|i| Frame {
            texture_key: format!("homer_front_walk_{i}"),
            duration_ms: 150.0,
        }).collect(),
        looping: true,
    });

    anims.insert("walk_back".to_string(), Animation {
        name: "walk_back".to_string(),
        frames: (1..=4).map(|i| Frame {
            texture_key: format!("homer_back_walk_{i}"),
            duration_ms: 150.0,
        }).collect(),
        looping: true,
    });

    anims.insert("clean".to_string(), Animation {
        name: "clean".to_string(),
        frames: vec![
            Frame { texture_key: "homer_clean_0".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_clean_1".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_clean_2".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_clean_3".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_clean_4".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_clean_5".to_string(), duration_ms: 200.0 },
        ],
        looping: true,
    });

    anims.insert("play".to_string(), Animation {
        name: "play".to_string(),
        frames: vec![
            Frame { texture_key: "homer_play_1".to_string(), duration_ms: 250.0 },
            Frame { texture_key: "homer_play_2".to_string(), duration_ms: 250.0 },
            Frame { texture_key: "homer_play_3".to_string(), duration_ms: 250.0 },
            Frame { texture_key: "homer_play_7".to_string(), duration_ms: 250.0 },
        ],
        looping: true,
    });

    anims.insert("drink".to_string(), Animation {
        name: "drink".to_string(),
        frames: vec![
            Frame { texture_key: "homer_drink_0".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_drink_1".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_drink_2".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_drink_3".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_drink_4".to_string(), duration_ms: 200.0 },
            Frame { texture_key: "homer_drink_5".to_string(), duration_ms: 200.0 },
        ],
        looping: true,
    });

    anims
}

/// Map of texture key -> asset file path (relative to web root)
pub fn homer_texture_paths() -> Vec<(String, String)> {
    let mut paths = vec![];

    // Idle
    paths.push(("homer_idle_1".into(), "assets/Characters/Homer/Idle/homer_idle_image_1.png".into()));
    paths.push(("homer_idle_blink_1".into(), "assets/Characters/Homer/Idle/homer_idle_blink_image_1.png".into()));
    paths.push(("homer_idle_blink_2".into(), "assets/Characters/Homer/Idle/homer_idle_blink_image_2.png".into()));

    // Walk front
    for i in 1..=4 {
        paths.push((
            format!("homer_front_walk_{i}"),
            format!("assets/Characters/Homer/Walk/homer_front_walk_image_{i}.png"),
        ));
    }

    // Walk back
    for i in 1..=4 {
        paths.push((
            format!("homer_back_walk_{i}"),
            format!("assets/Characters/Homer/Walk/homer_back_walk_image_{i}.png"),
        ));
    }

    // Menu Assets
    paths.push(("menu_bg".into(), "assets/Menu/main_menu_background.webp".into()));


    // Job Assets - Clean Up
    for i in 1..=6 {
        paths.push((
            format!("homer_clean_{}", i - 1),
            format!("assets/Characters/Homer/Clean Up Springfield/homer_pick_trash_active_image_{i}.png"),
        ));
    }

    // Job Assets - Play with myPad
    for i in [1, 2, 3, 7] {
        paths.push((
            format!("homer_play_{i}"),
            format!("assets/Characters/Homer/Play with his myPad/homer_play_happy_little_elves_active_image_{i}.png"),
        ));
    }

    // Job Assets - Drink Beer
    for i in 0..=5 {
        paths.push((
            format!("homer_drink_{i}"),
            format!("assets/Characters/Homer/Drink Beer/homer_drink_beer_image_{i}.png"),
        ));
    }

    // Tiles
    paths.push(("grass_tile".into(), "assets/tiles/grass.png".into()));

    paths
}

/// Build Bart's animation definitions
pub fn bart_animations() -> HashMap<String, Animation> {
    let mut anims = HashMap::new();

    anims.insert("idle".to_string(), Animation {
        name: "idle".to_string(),
        frames: vec![
            Frame { texture_key: "bart_idle_1".to_string(), duration_ms: 2000.0 },
            Frame { texture_key: "bart_idle_blink_1".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "bart_idle_blink_2".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "bart_idle_1".to_string(), duration_ms: 3000.0 },
            Frame { texture_key: "bart_idle_blink_1".to_string(), duration_ms: 100.0 },
            Frame { texture_key: "bart_idle_blink_2".to_string(), duration_ms: 100.0 },
        ],
        looping: true,
    });

    anims.insert("walk_front".to_string(), Animation {
        name: "walk_front".to_string(),
        frames: (1..=4).map(|i| Frame {
            texture_key: format!("bart_front_walk_{i}"),
            duration_ms: 150.0,
        }).collect(),
        looping: true,
    });

    anims.insert("walk_back".to_string(), Animation {
        name: "walk_back".to_string(),
        frames: (1..=4).map(|i| Frame {
            texture_key: format!("bart_back_walk_{i}"),
            duration_ms: 150.0,
        }).collect(),
        looping: true,
    });

    anims.insert("skateboard".to_string(), Animation {
        name: "skateboard".to_string(),
        frames: (1..=6).map(|i| Frame {
            texture_key: format!("bart_skateboard_{i}"),
            duration_ms: 120.0,
        }).collect(),
        looping: true,
    });

    anims.insert("slingshot".to_string(), Animation {
        name: "slingshot".to_string(),
        frames: (1..=15).map(|i| Frame {
            texture_key: format!("bart_slingshot_{i}"),
            duration_ms: 100.0,
        }).collect(),
        looping: true,
    });

    anims.insert("play_simulator".to_string(), Animation {
        name: "play_simulator".to_string(),
        frames: (1..=18).map(|i| Frame {
            texture_key: format!("bart_play_simulator_{i}"),
            duration_ms: 120.0,
        }).collect(),
        looping: true,
    });

    anims
}

/// Map of texture key -> asset file path (relative to web root) for Bart
pub fn bart_texture_paths() -> Vec<(String, String)> {
    let mut paths = vec![];

    // Idle
    paths.push(("bart_idle_1".into(), "assets/Characters/Bart/Idle/bart_idle_image_1.png".into()));
    paths.push(("bart_idle_blink_1".into(), "assets/Characters/Bart/Idle/bart_idle_blink_image_1.png".into()));
    paths.push(("bart_idle_blink_2".into(), "assets/Characters/Bart/Idle/bart_idle_blink_image_2.png".into()));

    // Walk front
    for i in 1..=4 {
        paths.push((
            format!("bart_front_walk_{i}"),
            format!("assets/Characters/Bart/Walk/bart_front_walk_image_{i}.png"),
        ));
    }

    // Walk back
    for i in 1..=4 {
        paths.push((
            format!("bart_back_walk_{i}"),
            format!("assets/Characters/Bart/Walk/bart_back_walk_image_{i}.png"),
        ));
    }

    // Job Assets - Skateboard
    for i in 1..=6 {
        paths.push((
            format!("bart_skateboard_{i}"),
            format!("assets/Characters/Bart/Skateboard/bart_skateboarding_front_image_{i}.png"),
        ));
    }

    // Job Assets - Slingshot
    for i in 1..=15 {
        paths.push((
            format!("bart_slingshot_{i}"),
            format!("assets/Characters/Bart/Slingshot/bart_walk_slingshot_shoot_slingshot_image_{i}.png"),
        ));
    }

    // Job Assets - Play Yard Work Simulator
    for i in 1..=18 {
        paths.push((
            format!("bart_play_simulator_{i}"),
            format!("assets/Characters/Bart/Play Yard Work Simulator/bart_do_virtual_job_active_2_image_{i}.png"),
        ));
    }

    paths
}

