use crate::{
    aabb::Aabb,
    material::MaterialId,
    matrix::Matrix4,
    objects::{ObjectData, ObjectId, Objects},
    point::Point3,
    vector::Vec3,
    vertex::Vertex,
    vertex_buffer::VertexBuffer,
};

pub fn load_model(
    queue: &wgpu::Queue,
    objects: &mut Objects,
    vertex_buffer: &mut VertexBuffer,
    file_name: &str,
    transform: Matrix4,
    material_id: MaterialId,
) -> Aabb {
    let (models, _materials) = tobj::load_obj(
        file_name,
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap();

    let model = &models[0];

    let object_id = objects.insert(queue, ObjectData { transform });
    let vertices = {
        let mut vertices = Vec::with_capacity(model.mesh.indices.len());

        if model.mesh.face_arities.is_empty() {
            // all faces are triangles
            if model.mesh.normals.is_empty() {
                extract_and_normalise(object_id, material_id, model, &mut vertices);
            } else {
                /*
                assert!(
                    !model.mesh.normals.is_empty(),
                    "model {} is missing vertex normals",
                    file_name
                );
                */
                for index in model.mesh.indices.iter() {
                    let index = *index as usize;
                    vertices.push(Vertex {
                        position: Point3 {
                            x: model.mesh.positions[3 * index],
                            y: model.mesh.positions[3 * index + 1],
                            z: model.mesh.positions[3 * index + 2],
                        },
                        object_id,
                        normal: Vec3 {
                            x: model.mesh.normals[3 * index],
                            y: model.mesh.normals[3 * index + 1],
                            z: model.mesh.normals[3 * index + 2],
                        },
                        material_id,
                    });
                }
            }
        } else {
            panic!("mesh is not triangulated");
        }

        vertices
    };
    vertex_buffer.insert_many(queue, &vertices);

    let model_aabb = vertices.into_iter().fold(Aabb::EMPTY, |aabb, vertex| {
        aabb.union(Aabb::point(vertex.position))
    });

    transform * model_aabb
}

enum NormalStyle {
    /// For flat shading.
    Face,
    /// For smooth shading.
    Vertex,
}

const NORMAL_STYLE: NormalStyle = NormalStyle::Vertex;

fn extract_and_normalise(
    object_id: ObjectId,
    material_id: MaterialId,
    model: &tobj::Model,
    vertices: &mut Vec<Vertex>,
) {
    let mut normals: Vec<Vec3> = match NORMAL_STYLE {
        NormalStyle::Vertex => std::iter::repeat(Vec3::ZERO)
            .take(model.mesh.positions.len())
            .collect(),
        NormalStyle::Face => Vec::new(),
    };

    for triangle in model.mesh.indices.chunks(3) {
        let [index_a, index_b, index_c] = triangle
                        else { unreachable!() };

        let index_a = *index_a as usize;
        let index_b = *index_b as usize;
        let index_c = *index_c as usize;

        let a = Point3 {
            x: model.mesh.positions[3 * index_a],
            y: model.mesh.positions[3 * index_a + 1],
            z: model.mesh.positions[3 * index_a + 2],
        };

        let b = Point3 {
            x: model.mesh.positions[3 * index_b],
            y: model.mesh.positions[3 * index_b + 1],
            z: model.mesh.positions[3 * index_b + 2],
        };

        let c = Point3 {
            x: model.mesh.positions[3 * index_c],
            y: model.mesh.positions[3 * index_c + 1],
            z: model.mesh.positions[3 * index_c + 2],
        };

        // Assumes CCW vertex order.
        let face_normal = (b - a).cross(c - a).normalize();

        let normal = match NORMAL_STYLE {
            NormalStyle::Face => face_normal,
            NormalStyle::Vertex => {
                normals[index_a] += face_normal;
                normals[index_b] += face_normal;
                normals[index_c] += face_normal;
                Vec3::ZERO
            }
        };

        vertices.push(Vertex {
            position: a,
            object_id,
            normal,
            material_id,
        });

        vertices.push(Vertex {
            position: b,
            object_id,
            normal,
            material_id,
        });

        vertices.push(Vertex {
            position: c,
            object_id,
            normal,
            material_id,
        });
    }

    if let NormalStyle::Vertex = NORMAL_STYLE {
        for triangle in model.mesh.indices.chunks(3).enumerate() {
            let (face, [index_a, index_b, index_c]) = triangle
                        else { unreachable!() };

            let index_a = *index_a as usize;
            let index_b = *index_b as usize;
            let index_c = *index_c as usize;

            vertices[3 * face].normal = normals[index_a].normalize();
            vertices[3 * face + 1].normal = normals[index_b].normalize();
            vertices[3 * face + 2].normal = normals[index_c].normalize();
        }
    }
}
