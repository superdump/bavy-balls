use bevy::{
    math::{const_vec3, Quat, Vec3},
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub struct HalfCylinder {
    start: Vec3,
    end: Vec3,
    radius: f32,
    subdivisions: usize,
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

    pub fn from_scale(scale: f32) -> Self {
        let mut half_cylinder = Self::default();
        half_cylinder.start *= scale;
        half_cylinder.end *= scale;
        half_cylinder.radius *= scale;
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
