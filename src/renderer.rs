use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;
use crate::sprite::Texture;

pub struct Renderer {
    program: web_sys::WebGlProgram,
    vao: web_sys::WebGlVertexArrayObject,
    vbo: web_sys::WebGlBuffer,
    u_mvp: web_sys::WebGlUniformLocation,
    u_color: web_sys::WebGlUniformLocation,
    canvas_width: f32,
    canvas_height: f32,
}

const VERT_SRC: &str = r#"#version 300 es
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
out vec2 v_uv;
uniform mat4 u_mvp;
void main() {
    v_uv = a_uv;
    gl_Position = u_mvp * vec4(a_pos, 0.0, 1.0);
}
"#;

const FRAG_SRC: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 out_color;
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    vec4 tex = texture(u_texture, v_uv);
    if (tex.a < 0.01) discard;
    out_color = tex * u_color;
}
"#;

// Quad: 2 triangles, each vertex has (x, y, u, v)
const QUAD_VERTS: [f32; 24] = [
    // tri 1
    0.0, 0.0,  0.0, 0.0,
    1.0, 0.0,  1.0, 0.0,
    1.0, 1.0,  1.0, 1.0,
    // tri 2
    0.0, 0.0,  0.0, 0.0,
    1.0, 1.0,  1.0, 1.0,
    0.0, 1.0,  0.0, 1.0,
];

impl Renderer {
    pub fn new(gl: &GL) -> Result<Renderer, JsValue> {
        let program = compile_program(gl, VERT_SRC, FRAG_SRC)?;

        let u_mvp = gl.get_uniform_location(&program, "u_mvp")
            .ok_or("u_mvp uniform not found")?;
        let u_color = gl.get_uniform_location(&program, "u_color")
            .ok_or("u_color uniform not found")?;

        let vao = gl.create_vertex_array().ok_or("failed to create VAO")?;
        let vbo = gl.create_buffer().ok_or("failed to create VBO")?;

        gl.bind_vertex_array(Some(&vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

        // Upload quad geometry
        unsafe {
            let vert_array = js_sys::Float32Array::view(&QUAD_VERTS);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);
        }

        // pos (x,y)
        gl.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 16, 0);
        gl.enable_vertex_attrib_array(0);
        // uv (u,v)
        gl.vertex_attrib_pointer_with_i32(1, 2, GL::FLOAT, false, 16, 8);
        gl.enable_vertex_attrib_array(1);

        gl.bind_vertex_array(None);

        // Enable alpha blending
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

        Ok(Renderer {
            program,
            vao,
            vbo,
            u_mvp,
            u_color,
            canvas_width: 800.0,
            canvas_height: 600.0,
        })
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.canvas_width = w;
        self.canvas_height = h;
    }

    pub fn begin_frame(&self, gl: &GL) {
        gl.clear_color(0.47, 0.82, 0.95, 1.0); // lighter sky blue
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));
    }

    pub fn end_frame(&self, gl: &GL) {
        gl.bind_vertex_array(None);
    }

    /// Draw a sprite at pixel position (x, y) with given pixel size
    pub fn draw_sprite(
        &self,
        gl: &GL,
        texture: &Texture,
        x: f32,
        y: f32,
        scale: f32,
        flip_x: bool,
    ) {
        let w = texture.width as f32 * scale;
        let h = texture.height as f32 * scale;

        let sx = if flip_x { -2.0 * w / self.canvas_width } else { 2.0 * w / self.canvas_width };
        let sy = 2.0 * h / self.canvas_height;
        let tx = if flip_x {
            2.0 * (x - w * 0.5 + w) / self.canvas_width - 1.0
        } else {
            2.0 * (x - w * 0.5) / self.canvas_width - 1.0
        };
        // Align to the bottom: y represents the feet anchor position
        let ty = 1.0 - 2.0 * y / self.canvas_height;

        // Column-major 4x4 matrix
        #[rustfmt::skip]
        let mvp: [f32; 16] = [
            sx,  0.0, 0.0, 0.0,
            0.0, sy,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_mvp), false, &mvp);
        gl.uniform4f(Some(&self.u_color), 1.0, 1.0, 1.0, 1.0);

        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture.gl_texture));

        gl.draw_arrays(GL::TRIANGLES, 0, 6);
    }

    /// Draw a sprite stretched to cover a specific rectangle (e.g. for backgrounds or UI)
    pub fn draw_sprite_stretched(
        &self,
        gl: &GL,
        texture: &Texture,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let sx = 2.0 * w / self.canvas_width;
        let sy = 2.0 * h / self.canvas_height;
        let tx = 2.0 * x / self.canvas_width - 1.0;
        let ty = 1.0 - 2.0 * (y + h) / self.canvas_height;

        #[rustfmt::skip]
        let mvp: [f32; 16] = [
            sx,  0.0, 0.0, 0.0,
            0.0, sy,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_mvp), false, &mvp);
        gl.uniform4f(Some(&self.u_color), 1.0, 1.0, 1.0, 1.0);

        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture.gl_texture));

        gl.draw_arrays(GL::TRIANGLES, 0, 6);
    }

    /// Draw a sprite repeating (tiled) to fill a specific rectangle (e.g. for ground patterns)
    pub fn draw_sprite_tiled(
        &self,
        gl: &GL,
        texture: &Texture,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tile_scale_x: f32,
        tile_scale_y: f32,
    ) {
        let sx = 2.0 * w / self.canvas_width;
        let sy = 2.0 * h / self.canvas_height;
        let tx = 2.0 * x / self.canvas_width - 1.0;
        let ty = 1.0 - 2.0 * (y + h) / self.canvas_height;

        #[rustfmt::skip]
        let mvp: [f32; 16] = [
            sx,  0.0, 0.0, 0.0,
            0.0, sy,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_mvp), false, &mvp);
        gl.uniform4f(Some(&self.u_color), 1.0, 1.0, 1.0, 1.0);

        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture.gl_texture));

        // Temporarily set texture wrapping to repeat
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32);

        // Define tiled UV quad vertices on-the-fly and bind to draw
        let u_max = w / tile_scale_x;
        let v_max = h / tile_scale_y;
        let quad_verts: [f32; 24] = [
            0.0, 0.0,  0.0, 0.0,
            1.0, 0.0,  u_max, 0.0,
            1.0, 1.0,  u_max, v_max,
            
            0.0, 0.0,  0.0, 0.0,
            1.0, 1.0,  u_max, v_max,
            0.0, 1.0,  0.0, v_max,
        ];

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        unsafe {
            let vert_array = js_sys::Float32Array::view(&quad_verts);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STREAM_DRAW);
        }

        gl.draw_arrays(GL::TRIANGLES, 0, 6);

        // Restore default quad buffer
        const STATIC_QUAD: [f32; 24] = [
            0.0, 0.0,  0.0, 0.0,
            1.0, 0.0,  1.0, 0.0,
            1.0, 1.0,  1.0, 1.0,
            0.0, 0.0,  0.0, 0.0,
            1.0, 1.0,  1.0, 1.0,
            0.0, 1.0,  0.0, 1.0,
        ];
        unsafe {
            let vert_array = js_sys::Float32Array::view(&STATIC_QUAD);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);
        }

        // Restore clamp to edge wraps
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    }

    /// Draw a filled rect (no texture, solid color)
    pub fn draw_rect(&self, gl: &GL, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        let sx = 2.0 * w / self.canvas_width;
        let sy = 2.0 * h / self.canvas_height;
        let tx = 2.0 * x / self.canvas_width - 1.0;
        let ty = 1.0 - 2.0 * (y + h) / self.canvas_height;

        #[rustfmt::skip]
        let mvp: [f32; 16] = [
            sx,  0.0, 0.0, 0.0,
            0.0, sy,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_mvp), false, &mvp);
        gl.uniform4f(Some(&self.u_color), r, g, b, a);
        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.draw_arrays(GL::TRIANGLES, 0, 6);
    }
}

fn compile_shader(gl: &GL, shader_type: u32, src: &str) -> Result<web_sys::WebGlShader, JsValue> {
    let shader = gl.create_shader(shader_type).ok_or("failed to create shader")?;
    gl.shader_source(&shader, src);
    gl.compile_shader(&shader);
    if !gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        return Err(JsValue::from_str(&format!("shader compile error: {log}")));
    }
    Ok(shader)
}

fn compile_program(gl: &GL, vert: &str, frag: &str) -> Result<web_sys::WebGlProgram, JsValue> {
    let vs = compile_shader(gl, GL::VERTEX_SHADER, vert)?;
    let fs = compile_shader(gl, GL::FRAGMENT_SHADER, frag)?;
    let program = gl.create_program().ok_or("failed to create program")?;
    gl.attach_shader(&program, &vs);
    gl.attach_shader(&program, &fs);
    gl.link_program(&program);
    if !gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        return Err(JsValue::from_str(&format!("program link error: {log}")));
    }
    Ok(program)
}
