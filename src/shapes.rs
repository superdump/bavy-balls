use std::ops::Range;

use bevy::{
    math::{const_vec3, Quat, Vec3},
    prelude::Mesh,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_rapier3d::{na::Point3, prelude::ColliderShape};
use rand::{prelude::SmallRng, SeedableRng};

use crate::paths::WormPathIterator;

pub struct HalfCylinder {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
    pub subdivisions: usize,
}

const START: Vec3 = const_vec3!([0.0, 0.0, -0.5]);
const END: Vec3 = const_vec3!([0.0, 0.0, 0.5]);

impl HalfCylinder {
    pub const fn new() -> Self {
        Self {
            start: START,
            end: END,
            radius: 0.5,
            subdivisions: 10,
        }
    }

    pub fn from_radius_and_length(radius: f32, length: f32) -> Self {
        let mut half_cylinder = Self::default();
        half_cylinder.start *= length;
        half_cylinder.end *= length;
        half_cylinder.radius = radius;
        half_cylinder
    }
}

impl Default for HalfCylinder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HalfCylinder> for Mesh {
    fn from(shape: HalfCylinder) -> Self {
        let HalfCylinder {
            start,
            end,
            radius,
            subdivisions,
        } = shape;
        let vertex_count = (subdivisions + 1) * 2;

        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);

        let up = Vec3::Y;
        let forward = (end - start).normalize_or_zero();
        let right = up.cross(-forward).normalize_or_zero() * radius;
        for i in 0..=subdivisions {
            // start point
            let offset = Quat::from_axis_angle(
                forward,
                std::f32::consts::PI * i as f32 / subdivisions as f32,
            ) * right;
            let normal = (-offset.normalize_or_zero()).to_array();
            positions.push((start + offset).to_array());
            normals.push(normal);
            uvs.push([0.0, 0.0]);
            // end point
            positions.push((end + offset).to_array());
            normals.push(normal);
            uvs.push([0.0, 0.0]);
        }

        let mut indices = Vec::with_capacity(subdivisions * 2);
        for i in 0..subdivisions as u32 {
            let offset = i as u32 * 2;
            indices.extend_from_slice(&[
                offset + 2,
                offset,
                offset + 1,
                offset + 1,
                offset + 3,
                offset + 2,
            ]);
        }
        let indices = Indices::U32(indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(indices));
        mesh
    }
}

pub struct HalfCylinderPath {
    pub start: Vec3,
    pub forward: Vec3,
    pub radius: f32,
    pub segment_length: f32,
    pub n_segments: usize,
    pub subdivisions: usize,
    pub seed: u64,
    pub yaw_range: Range<f32>,
    pub pitch_range: Range<f32>,
}

const NEGATIVE_Z: Vec3 = const_vec3!([0.0, 0.0, -1.0]);

const YAW_RANGE: Range<f32> =
    (-0.9 * std::f32::consts::FRAC_PI_2)..(0.9 * std::f32::consts::FRAC_PI_2);
const PITCH_RANGE: Range<f32> =
    (-0.9 * std::f32::consts::FRAC_PI_2)..(-0.1 * std::f32::consts::FRAC_PI_2);

impl HalfCylinderPath {
    pub const fn new() -> Self {
        Self {
            start: Vec3::ZERO,
            forward: NEGATIVE_Z,
            radius: 0.5,
            segment_length: 1.0,
            n_segments: 100,
            subdivisions: 10,
            seed: 1234,
            yaw_range: YAW_RANGE,
            pitch_range: PITCH_RANGE,
        }
    }
}

impl Default for HalfCylinderPath {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HalfCylinderPath> for Mesh {
    fn from(shape: HalfCylinderPath) -> Self {
        let HalfCylinderPath {
            start,
            forward,
            radius,
            segment_length,
            n_segments,
            subdivisions,
            seed,
            yaw_range,
            pitch_range,
        } = shape;
        let vertex_count = (subdivisions + 1) * (n_segments + 1);

        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);

        let up = Vec3::Y;
        let mut position = start;
        let worm_path_iter = WormPathIterator {
            rng: SmallRng::seed_from_u64(seed),
            yaw_range,
            pitch_range,
        };
        for rotation in worm_path_iter.take(n_segments + 1) {
            let forward = rotation * forward;
            let right = up.cross(-forward).normalize_or_zero() * radius;
            for i in 0..=subdivisions {
                let offset = Quat::from_axis_angle(
                    forward,
                    std::f32::consts::PI * i as f32 / subdivisions as f32,
                ) * right;
                let normal = (-offset.normalize_or_zero()).to_array();
                positions.push((position + offset).to_array());
                normals.push(normal);
                uvs.push([0.0, 0.0]);
            }
            position += forward * segment_length;
        }

        let mut indices = Vec::with_capacity(n_segments * subdivisions * 6);
        let segment_vertex_count = subdivisions as u32 + 1;
        for i in 0..n_segments as u32 {
            let segment_offset = segment_vertex_count * i;
            for j in 0..subdivisions as u32 {
                let offset = segment_offset + j;
                indices.extend_from_slice(&[
                    offset + 1,
                    offset,
                    offset + segment_vertex_count,
                    offset + segment_vertex_count,
                    offset + segment_vertex_count + 1,
                    offset + 1,
                ]);
            }
        }
        let indices = Indices::U32(indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(indices));
        mesh
    }
}

pub fn mesh_to_collider_shape(mesh: &Mesh) -> Option<ColliderShape> {
    let vertices = if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        positions
            .iter()
            .map(|p| Point3::from_slice(p))
            .collect::<Vec<_>>()
    } else {
        return None;
    };
    let indices = if let Some(Indices::U32(indices)) = mesh.indices() {
        indices
            .chunks_exact(3)
            .map(|tri| [tri[0], tri[1], tri[2]])
            .collect::<Vec<_>>()
    } else {
        return None;
    };
    Some(ColliderShape::trimesh(vertices, indices))
}
