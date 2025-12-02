//! Mesh generation for box geometry

use glam::{Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

/// Simple mesh for rendering
pub struct Mesh {
    pub vertices: Vec<f32>,   // positions + normals (6 floats per vertex)
    pub indices: Vec<u16>,
}

impl Mesh {
    /// Create a box mesh from min/max corners
    pub fn create_box(min: Vec3, max: Vec3) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 8 corners of the box
        let corners = [
            Vec3::new(min.x, min.y, min.z), // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        // 6 faces with normals
        let faces = [
            // Bottom face (-Z)
            ([0, 1, 2, 3], Vec3::new(0.0, 0.0, -1.0)),
            // Top face (+Z)
            ([4, 5, 6, 7], Vec3::new(0.0, 0.0, 1.0)),
            // Front face (-Y)
            ([0, 1, 5, 4], Vec3::new(0.0, -1.0, 0.0)),
            // Back face (+Y)
            ([2, 3, 7, 6], Vec3::new(0.0, 1.0, 0.0)),
            // Left face (-X)
            ([0, 3, 7, 4], Vec3::new(-1.0, 0.0, 0.0)),
            // Right face (+X)
            ([1, 2, 6, 5], Vec3::new(1.0, 0.0, 0.0)),
        ];

        for (corner_indices, normal) in faces.iter() {
            let base_idx = (vertices.len() / 6) as u16;

            // Add 4 vertices for this face
            for &corner_idx in corner_indices.iter() {
                let pos = corners[corner_idx];
                vertices.push(pos.x);
                vertices.push(pos.y);
                vertices.push(pos.z);
                vertices.push(normal.x);
                vertices.push(normal.y);
                vertices.push(normal.z);
            }

            // Add 2 triangles (6 indices) for this quad
            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);

            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }

        Self { vertices, indices }
    }
}

/// GPU buffer for a mesh
pub struct MeshBuffer {
    pub vertex_buffer: WebGlBuffer,
    pub index_buffer: WebGlBuffer,
    pub index_count: i32,
}

impl MeshBuffer {
    /// Upload mesh to GPU
    pub fn from_mesh(gl: &WebGl2RenderingContext, mesh: &Mesh) -> Result<Self, String> {
        use wasm_bindgen::JsCast;
        use js_sys::{Float32Array, Uint16Array};

        // Create vertex buffer
        let vertex_buffer = gl
            .create_buffer()
            .ok_or("Failed to create vertex buffer")?;

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        let vertex_array = unsafe {
            Float32Array::view(&mesh.vertices)
        };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertex_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Create index buffer
        let index_buffer = gl
            .create_buffer()
            .ok_or("Failed to create index buffer")?;

        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        let index_array = unsafe {
            Uint16Array::view(&mesh.indices)
        };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as i32,
        })
    }
}
