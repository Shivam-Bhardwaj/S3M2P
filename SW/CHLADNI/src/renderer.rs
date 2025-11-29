// WebGL2 renderer for Chladni visualization
// Renders wave field and particles

use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader};

use crate::{ChladniSimulation, Particle};

pub struct WaveRenderer {
    gl: GL,
    particle_program: Option<WebGlProgram>,
    wave_program: Option<WebGlProgram>,
    particle_buffer: Option<WebGlBuffer>,
}

impl WaveRenderer {
    pub fn new(gl: GL) -> Self {
        Self {
            gl,
            particle_program: None,
            wave_program: None,
            particle_buffer: None,
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        // Particle shader
        let particle_vert = r#"#version 300 es
            in vec2 a_position;
            uniform vec2 u_resolution;

            void main() {
                vec2 pos = (a_position / u_resolution) * 2.0 - 1.0;
                pos.y = -pos.y;
                gl_Position = vec4(pos, 0.0, 1.0);
                gl_PointSize = 2.0;
            }
        "#;

        let particle_frag = r#"#version 300 es
            precision highp float;
            out vec4 fragColor;

            void main() {
                float dist = length(gl_PointCoord - 0.5);
                if (dist > 0.5) discard;
                fragColor = vec4(0.9, 0.85, 0.7, 1.0); // Sand color
            }
        "#;

        self.particle_program = Some(self.create_program(particle_vert, particle_frag)?);

        // Wave field shader (for background)
        let wave_vert = r#"#version 300 es
            in vec2 a_position;
            out vec2 v_texCoord;

            void main() {
                gl_Position = vec4(a_position, 0.0, 1.0);
                v_texCoord = a_position * 0.5 + 0.5;
            }
        "#;

        let wave_frag = r#"#version 300 es
            precision highp float;
            in vec2 v_texCoord;
            out vec4 fragColor;
            uniform sampler2D u_waveField;

            void main() {
                float amplitude = texture(u_waveField, v_texCoord).r;

                // Color based on amplitude
                vec3 lowColor = vec3(0.05, 0.05, 0.1);   // Dark blue
                vec3 highColor = vec3(0.2, 0.4, 0.8);    // Bright blue

                float t = amplitude * 0.5 + 0.5;
                vec3 color = mix(lowColor, highColor, t * t);

                fragColor = vec4(color, 1.0);
            }
        "#;

        self.wave_program = Some(self.create_program(wave_vert, wave_frag)?);

        // Create particle buffer
        self.particle_buffer = Some(self.gl.create_buffer().ok_or("Failed to create buffer")?);

        Ok(())
    }

    fn create_program(&self, vert_src: &str, frag_src: &str) -> Result<WebGlProgram, JsValue> {
        let vert_shader = self.compile_shader(GL::VERTEX_SHADER, vert_src)?;
        let frag_shader = self.compile_shader(GL::FRAGMENT_SHADER, frag_src)?;

        let program = self.gl.create_program().ok_or("Failed to create program")?;
        self.gl.attach_shader(&program, &vert_shader);
        self.gl.attach_shader(&program, &frag_shader);
        self.gl.link_program(&program);

        if !self
            .gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let info = self.gl.get_program_info_log(&program).unwrap_or_default();
            return Err(JsValue::from_str(&format!("Link error: {}", info)));
        }

        Ok(program)
    }

    fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
        let shader = self
            .gl
            .create_shader(shader_type)
            .ok_or("Failed to create shader")?;
        self.gl.shader_source(&shader, source);
        self.gl.compile_shader(&shader);

        if !self
            .gl
            .get_shader_parameter(&shader, GL::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let info = self.gl.get_shader_info_log(&shader).unwrap_or_default();
            return Err(JsValue::from_str(&format!("Compile error: {}", info)));
        }

        Ok(shader)
    }

    pub fn render(&self, sim: &ChladniSimulation, width: f32, height: f32) {
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.gl.clear_color(0.02, 0.02, 0.05, 1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT);

        // Render particles
        if let (Some(program), Some(buffer)) = (&self.particle_program, &self.particle_buffer) {
            self.gl.use_program(Some(program));

            // Upload particle positions
            let positions: Vec<f32> = sim
                .particles
                .iter()
                .filter(|p| p.active)
                .flat_map(|p| [p.pos.x, p.pos.y])
                .collect();

            self.gl.bind_buffer(GL::ARRAY_BUFFER, Some(buffer));
            unsafe {
                let array = js_sys::Float32Array::view(&positions);
                self.gl.buffer_data_with_array_buffer_view(
                    GL::ARRAY_BUFFER,
                    &array,
                    GL::DYNAMIC_DRAW,
                );
            }

            // Set uniforms
            let res_loc = self.gl.get_uniform_location(program, "u_resolution");
            self.gl.uniform2f(
                res_loc.as_ref(),
                sim.config.grid_size as f32,
                sim.config.grid_size as f32,
            );

            // Set attributes
            let pos_loc = self.gl.get_attrib_location(program, "a_position") as u32;
            self.gl.enable_vertex_attrib_array(pos_loc);
            self.gl
                .vertex_attrib_pointer_with_i32(pos_loc, 2, GL::FLOAT, false, 0, 0);

            // Draw particles
            let particle_count = positions.len() / 2;
            self.gl.draw_arrays(GL::POINTS, 0, particle_count as i32);
        }
    }
}
