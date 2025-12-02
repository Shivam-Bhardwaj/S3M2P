//! WebGL2 3D renderer with orbit controls

use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::PI;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, HtmlCanvasElement};

/// Orbit camera for 3D view
#[derive(Clone, Debug)]
pub struct Camera {
    /// Target point (what we're looking at)
    pub target: Vec3,
    /// Distance from target
    pub distance: f32,
    /// Rotation around Y axis (azimuth, in radians)
    pub azimuth: f32,
    /// Rotation around X axis (elevation, in radians)
    pub elevation: f32,
    /// Field of view (degrees)
    pub fov: f32,
    /// Aspect ratio (width / height)
    pub aspect: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 150.0,
            azimuth: PI / 4.0,      // 45° from +X axis
            elevation: PI / 6.0,    // 30° above XY plane
            fov: 45.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Camera {
    /// Compute camera position from spherical coordinates
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.elevation.cos() * self.azimuth.cos();
        let y = self.distance * self.elevation.cos() * self.azimuth.sin();
        let z = self.distance * self.elevation.sin();
        self.target + Vec3::new(x, y, z)
    }

    /// Compute view matrix (look-at)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Z)
    }

    /// Compute projection matrix (perspective)
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    /// Compute combined view-projection matrix
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Orbit camera (delta in radians)
    pub fn orbit(&mut self, delta_azimuth: f32, delta_elevation: f32) {
        self.azimuth += delta_azimuth;
        self.elevation = (self.elevation + delta_elevation).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
    }

    /// Zoom camera (negative = zoom in, positive = zoom out)
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).max(10.0).min(500.0);
    }

    /// Pan camera (move target in screen space)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        // Convert screen-space movement to world-space
        let right = Vec3::new(-self.azimuth.sin(), self.azimuth.cos(), 0.0);
        let up = Vec3::Z;

        let scale = self.distance * 0.001; // Scale based on distance
        self.target += right * delta_x * scale + up * delta_y * scale;
    }
}

/// Rendering mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Solid shaded with lighting
    Shaded,
    /// Wireframe only
    Wireframe,
    /// Solid with wireframe overlay
    ShadedWireframe,
}

/// WebGL2 renderer
pub struct WebGLRenderer {
    pub gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    camera: Camera,
    mode: RenderMode,

    // Shader programs
    main_program: Option<WebGlProgram>,

    // Drag state for orbit controls
    pub is_dragging: bool,
    pub last_mouse_x: f32,
    pub last_mouse_y: f32,
}

impl WebGLRenderer {
    /// Create a new WebGL2 renderer
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let gl = canvas
            .get_context("webgl2")
            .map_err(|e| format!("Failed to get WebGL2 context: {:?}", e))?
            .ok_or("WebGL2 context is None")?
            .dyn_into::<WebGl2RenderingContext>()
            .map_err(|e| format!("Failed to cast to WebGL2: {:?}", e))?;

        // Enable depth testing
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_func(WebGl2RenderingContext::LEQUAL);

        // Enable backface culling
        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        let width = canvas.width() as f32;
        let height = canvas.height() as f32;
        let mut camera = Camera::default();
        camera.aspect = width / height;

        Ok(Self {
            gl,
            canvas,
            camera,
            mode: RenderMode::Shaded,
            main_program: None,
            is_dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        })
    }

    /// Initialize shaders
    pub fn init_shaders(&mut self) -> Result<(), String> {
        let program = create_shader_program(&self.gl, VERTEX_SHADER, FRAGMENT_SHADER)?;
        self.main_program = Some(program);
        Ok(())
    }

    /// Get camera reference
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get mutable camera reference
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Update camera aspect ratio (call on resize)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.camera.aspect = width as f32 / height as f32;
    }

    /// Clear the canvas
    pub fn clear(&self) {
        // Lighter dark background for better visibility
        self.gl.clear_color(0.12, 0.12, 0.15, 1.0); // Lighter than too.foo for better contrast
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
    }

    /// Begin frame
    pub fn begin_frame(&self) {
        self.clear();
    }

    /// End frame
    pub fn end_frame(&self) {
        self.gl.flush();
    }

    /// Draw a mesh with given color
    pub fn draw_mesh(&self, mesh_buffer: &super::mesh::MeshBuffer, color: Vec3) {
        if let Some(ref program) = self.main_program {
            self.gl.use_program(Some(program));

            // Bind buffers
            self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&mesh_buffer.vertex_buffer));
            self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&mesh_buffer.index_buffer));

            // Set up vertex attributes
            let position_loc = self.gl.get_attrib_location(program, "a_position") as u32;
            let normal_loc = self.gl.get_attrib_location(program, "a_normal") as u32;

            self.gl.enable_vertex_attrib_array(position_loc);
            self.gl.enable_vertex_attrib_array(normal_loc);

            let stride = 6 * 4; // 6 floats * 4 bytes
            self.gl.vertex_attrib_pointer_with_i32(position_loc, 3, WebGl2RenderingContext::FLOAT, false, stride, 0);
            self.gl.vertex_attrib_pointer_with_i32(normal_loc, 3, WebGl2RenderingContext::FLOAT, false, stride, 3 * 4);

            // Set uniforms
            let model_loc = self.gl.get_uniform_location(program, "u_model");
            let vp_loc = self.gl.get_uniform_location(program, "u_view_projection");
            let normal_matrix_loc = self.gl.get_uniform_location(program, "u_normal_matrix");
            let light_dir_loc = self.gl.get_uniform_location(program, "u_light_dir");
            let light_color_loc = self.gl.get_uniform_location(program, "u_light_color");
            let ambient_loc = self.gl.get_uniform_location(program, "u_ambient_color");
            let diffuse_loc = self.gl.get_uniform_location(program, "u_diffuse_color");
            let specular_loc = self.gl.get_uniform_location(program, "u_specular_color");
            let shininess_loc = self.gl.get_uniform_location(program, "u_shininess");
            let camera_pos_loc = self.gl.get_uniform_location(program, "u_camera_pos");

            // Model matrix (identity for now)
            let model = Mat4::IDENTITY;
            self.gl.uniform_matrix4fv_with_f32_array(model_loc.as_ref(), false, &model.to_cols_array());

            // View-projection matrix
            let vp = self.camera.view_projection();
            self.gl.uniform_matrix4fv_with_f32_array(vp_loc.as_ref(), false, &vp.to_cols_array());

            // Normal matrix (upper-left 3x3 of inverse transpose of model)
            let normal_matrix = Mat4::IDENTITY.inverse().transpose();
            let normal_mat3 = [
                normal_matrix.x_axis.x, normal_matrix.x_axis.y, normal_matrix.x_axis.z,
                normal_matrix.y_axis.x, normal_matrix.y_axis.y, normal_matrix.y_axis.z,
                normal_matrix.z_axis.x, normal_matrix.z_axis.y, normal_matrix.z_axis.z,
            ];
            self.gl.uniform_matrix3fv_with_f32_array(normal_matrix_loc.as_ref(), false, &normal_mat3);

            // Lighting (brighter for better visibility)
            let light_dir = Vec3::new(0.5, 0.5, 0.7).normalize();
            self.gl.uniform3f(light_dir_loc.as_ref(), light_dir.x, light_dir.y, light_dir.z);
            self.gl.uniform3f(light_color_loc.as_ref(), 1.0, 0.98, 0.95);
            self.gl.uniform3f(ambient_loc.as_ref(), 0.5, 0.5, 0.55); // Much brighter ambient

            // Material
            self.gl.uniform3f(diffuse_loc.as_ref(), color.x, color.y, color.z);
            self.gl.uniform3f(specular_loc.as_ref(), 0.3, 0.3, 0.3);
            self.gl.uniform1f(shininess_loc.as_ref(), 32.0);

            // Camera position
            let cam_pos = self.camera.position();
            self.gl.uniform3f(camera_pos_loc.as_ref(), cam_pos.x, cam_pos.y, cam_pos.z);

            // Draw
            self.gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                mesh_buffer.index_count,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );
        }
    }
}

// ============================================================================
// Shader Source Code
// ============================================================================

const VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 a_position;
in vec3 a_normal;
in vec2 a_texcoord;

uniform mat4 u_model;
uniform mat4 u_view_projection;
uniform mat3 u_normal_matrix;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_texcoord;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_position = world_pos.xyz;
    v_normal = u_normal_matrix * a_normal;
    v_texcoord = a_texcoord;
    gl_Position = u_view_projection * world_pos;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 v_position;
in vec3 v_normal;
in vec2 v_texcoord;

uniform vec3 u_light_dir;
uniform vec3 u_light_color;
uniform vec3 u_ambient_color;
uniform vec3 u_diffuse_color;
uniform vec3 u_specular_color;
uniform float u_shininess;
uniform vec3 u_camera_pos;

out vec4 out_color;

void main() {
    // Normalize interpolated normal
    vec3 N = normalize(v_normal);
    vec3 L = normalize(u_light_dir);
    vec3 V = normalize(u_camera_pos - v_position);
    vec3 H = normalize(L + V);

    // Phong lighting
    vec3 ambient = u_ambient_color * u_diffuse_color;

    float diff = max(dot(N, L), 0.0);
    vec3 diffuse = u_light_color * u_diffuse_color * diff;

    float spec = pow(max(dot(N, H), 0.0), u_shininess);
    vec3 specular = u_light_color * u_specular_color * spec;

    vec3 color = ambient + diffuse + specular;
    out_color = vec4(color, 1.0);
}
"#;

// ============================================================================
// Shader Compilation Utilities
// ============================================================================

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| "Unable to create shader object".to_string())?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error compiling shader".to_string()))
    }
}

fn create_shader_program(
    gl: &WebGl2RenderingContext,
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, String> {
    let vert_shader = compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vert_source)?;
    let frag_shader = compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_source)?;

    let program = gl
        .create_program()
        .ok_or_else(|| "Unable to create shader program".to_string())?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    if gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error linking program".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_position() {
        let camera = Camera::default();
        let pos = camera.position();

        // Should be at positive distance from target
        assert!(pos.length() > 0.0);
    }

    #[test]
    fn test_camera_orbit() {
        let mut camera = Camera::default();
        let initial_azimuth = camera.azimuth;

        camera.orbit(0.1, 0.0);
        assert!((camera.azimuth - initial_azimuth - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::default();
        let initial_distance = camera.distance;

        camera.zoom(-10.0);
        assert!(camera.distance < initial_distance);

        camera.zoom(1000.0); // Try to zoom way out
        assert!(camera.distance <= 500.0); // Should be clamped
    }
}
