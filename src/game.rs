use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use std::collections::HashMap;
use crate::renderer::Renderer;
use crate::input::InputState;
use crate::sprite::{Texture, Animation, AnimationState};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
}

#[wasm_bindgen]
pub struct Game {
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    renderer: Renderer,
    input: InputState,
    last_time: f64,
    textures: HashMap<String, Texture>,
    animations: HashMap<String, Animation>,
    homer_state: AnimationState,
    homer_x: f32,
    homer_y: f32,
    homer_target_x: f32,
    homer_target_y: f32,
    homer_scale: f32,
    homer_facing_left: bool,
    state: GameState,
    
    // Auto-wandering & Chores state
    homer_current_job: Option<String>,
    homer_job_timer_ms: f64,
    homer_job_duration_ms: f64,
    wander_timer_ms: f64,
    cash: i32,
}

#[wasm_bindgen]
impl Game {
    pub fn new(canvas_id: &str) -> Result<Game, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        let gl = canvas
            .get_context("webgl2")?
            .ok_or("WebGL2 not supported")?
            .dyn_into::<WebGl2RenderingContext>()?;

        let renderer = Renderer::new(&gl)?;
        let input = InputState::new();
        let animations = crate::sprite::homer_animations();
        let homer_state = AnimationState::new("idle");

        let cx = canvas.width() as f32 / 2.0 - 50.0;
        let cy = canvas.height() as f32 / 2.0 - 80.0;

        Ok(Game {
            canvas,
            gl,
            renderer,
            input,
            last_time: 0.0,
            textures: HashMap::new(),
            animations,
            homer_state,
            homer_x: cx,
            homer_y: cy,
            homer_target_x: cx,
            homer_target_y: cy,
            homer_scale: 0.5,
            homer_facing_left: false,
            state: GameState::Menu,
            homer_current_job: None,
            homer_job_timer_ms: 0.0,
            homer_job_duration_ms: 0.0,
            wander_timer_ms: 0.0,
            cash: 0,
        })
    }

    /// Register a texture that was loaded from JS
    pub fn register_texture(&mut self, key: &str, img: &web_sys::HtmlImageElement) -> Result<(), JsValue> {
        let tex = crate::sprite::upload_image_to_texture(&self.gl, img)?;
        self.textures.insert(key.to_string(), tex);
        Ok(())
    }

    pub fn update(&mut self, timestamp: f64) -> Result<(), JsValue> {
        let delta_ms = if self.last_time == 0.0 {
            16.0
        } else {
            timestamp - self.last_time
        };
        self.last_time = timestamp;

        let w = self.canvas.width() as f32;
        let h = self.canvas.height() as f32;

        // Click handler depends on state
        if let Some((cx, cy)) = self.input.consume_click() {
            let _px = cx as f32;
            let _py = cy as f32;

            match self.state {
                GameState::Menu => {
                    self.state = GameState::Playing;
                }
                GameState::Playing => {
                    // Let JS handle selecting Homer to show tasks,
                    // clicking on empty ground is ignored or handled on JS overlay layer.
                }
            }
        }

        self.renderer.set_viewport(w, h);
        self.renderer.begin_frame(&self.gl);

        match self.state {
            GameState::Menu => {
                // Draw background stretched to fit
                if let Some(tex) = self.textures.get("menu_bg") {
                    self.renderer.draw_sprite_stretched(&self.gl, tex, 0.0, 0.0, w, h);
                }

                // Draw centered logo (foreground)
                if let Some(tex) = self.textures.get("menu_fg") {
                    // Target width 50% of screen width, keep aspect ratio
                    let fg_w = w * 0.45;
                    let aspect = tex.height as f32 / tex.width as f32;
                    let fg_h = fg_w * aspect;
                    let fg_x = (w - fg_w) / 2.0;
                    let fg_y = (h - fg_h) / 3.0; // Place slightly above absolute center
                    self.renderer.draw_sprite_stretched(&self.gl, tex, fg_x, fg_y, fg_w, fg_h);
                }
            }
            GameState::Playing => {
                // Handle active job timers
                let mut completed_job = None;
                if let Some(ref job) = self.homer_current_job {
                    self.homer_job_timer_ms += delta_ms;
                    self.homer_state.set_animation(job);

                    if self.homer_job_timer_ms >= self.homer_job_duration_ms {
                        completed_job = Some(job.clone());
                    }
                }

                if let Some(job) = completed_job {
                    // Complete Job!
                    self.homer_current_job = None;
                    self.homer_state.set_animation("idle");
                    
                    // Give reward!
                    if job == "clean" {
                        self.cash += 50;
                    } else if job == "play" {
                        self.cash += 100;
                    } else if job == "drink" {
                        self.cash += 150;
                    }
                    
                    // Notify JS of completion
                    trigger_reward_toast(&job, self.cash);
                } else if self.homer_current_job.is_none() {
                    // Move Homer toward target or wander
                    let dx = self.homer_target_x - self.homer_x;
                    let dy = self.homer_target_y - self.homer_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > 5.0 {
                        let speed = 120.0; // pixel/sec walk speed
                        let step = speed * (delta_ms as f32 / 1000.0);
                        let ratio = step.min(dist) / dist;
                        self.homer_x += dx * ratio;
                        self.homer_y += dy * ratio;

                        if dx.abs() > 2.0 {
                            self.homer_facing_left = dx < 0.0;
                        }

                        if dy.abs() > dx.abs() {
                            if dy > 0.0 {
                                self.homer_state.set_animation("walk_front");
                            } else {
                                self.homer_state.set_animation("walk_back");
                            }
                        } else {
                            self.homer_state.set_animation("walk_front");
                        }
                    } else {
                        self.homer_state.set_animation("idle");

                        // Auto-wander routine when idle
                        self.wander_timer_ms += delta_ms;
                        if self.wander_timer_ms >= 4000.0 {
                            self.wander_timer_ms = 0.0;
                            // Random point anywhere on screen (with safety padding)
                            let rx = 50.0 + (js_sys::Math::random() as f32) * (w - 100.0);
                            let ry = 100.0 + (js_sys::Math::random() as f32) * (h - 150.0);
                            self.homer_target_x = rx;
                            self.homer_target_y = ry;
                        }
                    }
                }

                // Advance animation
                self.homer_state.update(delta_ms, &self.animations);

                // Draw ground (tiled grass texture covering full screen)
                if let Some(tex) = self.textures.get("grass_tile") {
                    // Repeat every 64 pixels
                    self.renderer.draw_sprite_tiled(&self.gl, tex, 0.0, 0.0, w, h, 64.0, 64.0);
                } else {
                    self.renderer.draw_rect(&self.gl, 0.0, 0.0, w, h, 0.34, 0.70, 0.24, 1.0);
                }

                // Draw Homer
                if let Some(tex_key) = self.homer_state.current_texture_key(&self.animations) {
                    if let Some(tex) = self.textures.get(tex_key) {
                        self.renderer.draw_sprite(&self.gl, tex, self.homer_x, self.homer_y, self.homer_scale, self.homer_facing_left);
                    }
                }
            }
        }

        self.renderer.end_frame(&self.gl);
        Ok(())
    }

    /// Check if a click point is within Homer's bounding box
    pub fn is_homer_clicked(&self, click_x: f32, click_y: f32) -> bool {
        if self.state != GameState::Playing || self.homer_current_job.is_some() {
            return false;
        }

        // Bounding box size (centered on x, bottom at y)
        let homer_w = 120.0 * self.homer_scale;
        let homer_h = 240.0 * self.homer_scale;

        click_x >= (self.homer_x - homer_w * 0.5) && click_x <= (self.homer_x + homer_w * 0.5) &&
        click_y >= (self.homer_y - homer_h) && click_y <= self.homer_y
    }

    /// Start a task chore for Homer
    pub fn assign_homer_job(&mut self, job_name: &str, duration_seconds: f64) {
        self.homer_current_job = Some(job_name.to_string());
        self.homer_job_timer_ms = 0.0;
        self.homer_job_duration_ms = duration_seconds * 1000.0;
    }

    /// Return screen coordinates of Homer so menu popup can align to him
    pub fn get_homer_screen_pos(&self) -> Vec<f32> {
        let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
        let homer_h = 240.0 * self.homer_scale;
        vec![
            self.homer_x / dpr,
            (self.homer_y - homer_h) / dpr
        ]
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.input.mouse_x = x;
        self.input.mouse_y = y;
    }

    pub fn on_click(&mut self, x: f64, y: f64) {
        self.input.last_click = Some((x, y));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.gl.viewport(0, 0, width as i32, height as i32);
    }
}

/// JS helper bind to trigger UI elements
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = triggerRewardToast)]
    fn trigger_reward_toast(job: &str, total_cash: i32);
}
