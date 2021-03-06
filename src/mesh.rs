use std::{ops::Deref, sync::Arc};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn get_position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }

    pub fn get_normal(&self) -> [f32; 3] {
        self.normal
    }

    pub fn set_normal(&mut self, normal: [f32; 3]) {
        self.normal = normal;
    }

    pub fn get_uv(&self) -> [f32; 2] {
        self.uv
    }

    pub fn set_uv(&mut self, uv: [f32; 2]) {
        self.uv = uv;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn vertex_data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn index_data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }

    pub fn width(&self) -> f32 {
        self.vertices
            .iter()
            .map(|vert| f32::sqrt(vert.position[0].powi(2) + vert.position[2].powi(2)))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.5)
    }

    pub fn height(&self) -> f32 {
        self.vertices
            .iter()
            .map(|vert| vert.position[1])
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.5)
    }

    pub fn buffers(&self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("basil-vertex-buffer"),
            contents: self.vertex_data(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("basil-index-buffer"),
            contents: self.index_data(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn calculate_normals(&mut self) {
        for vertex in self.vertices.iter_mut() {
            vertex.normal = [0.0; 3];
        }

        for i in 0..self.indices.len() / 3 {
            let i0 = self.indices[i * 3 + 0] as usize;
            let i1 = self.indices[i * 3 + 1] as usize;
            let i2 = self.indices[i * 3 + 2] as usize;

            if i0 >= self.vertices.len() || i1 >= self.vertices.len() || i2 >= self.vertices.len() {
                continue;
            }

            let v0 = self.vertices[i0];
            let v1 = self.vertices[i1];
            let v2 = self.vertices[i2];

            let p0: Vec3 = v0.position.into();
            let p1: Vec3 = v1.position.into();
            let p2: Vec3 = v2.position.into();

            let n0: Vec3 = v0.normal.into();
            let n1: Vec3 = v1.normal.into();
            let n2: Vec3 = v2.normal.into();

            let normal = Vec3::cross(p1 - p0, p2 - p1);

            self.vertices[i0].normal = From::from(n0 + normal);
            self.vertices[i1].normal = From::from(n1 + normal);
            self.vertices[i2].normal = From::from(n2 + normal);
        }

        for vertex in self.vertices.iter_mut() {
            let normalized = Vec3::from(vertex.normal).normalize_or_zero();
            vertex.normal = normalized.into();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SharedMesh {
    pub mesh: Arc<Mesh>,
}

impl SharedMesh {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh: Arc::new(mesh),
        }
    }
}

impl Deref for SharedMesh {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        self.mesh.as_ref()
    }
}
