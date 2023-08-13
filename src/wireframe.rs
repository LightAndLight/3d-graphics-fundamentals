use crate::{
    gpu_buffer::GpuBuffer,
    matrix::Matrix4,
    model_matrices::{ModelMatrices, ModelMatrixId},
    point::Point3,
    render_wireframe,
};

pub struct Wireframe {
    pub model_matrix_id: ModelMatrixId,
    pub vertex_buffer_offset: u32,
}

pub fn add<T: IntoIterator<Item = (Point3, Point3)>>(
    queue: &wgpu::Queue,
    model_matrices: &mut ModelMatrices,
    vertex_buffer: &mut GpuBuffer<render_wireframe::VertexInput>,
    model_matrix: Matrix4,
    lines: T,
) -> Wireframe {
    let model_matrix_id = model_matrices.insert(queue, model_matrix);

    let vertex_buffer_offset = vertex_buffer.len();

    for (from, to) in lines {
        vertex_buffer.insert(
            queue,
            render_wireframe::VertexInput {
                position: from,
                model_matrix_id,
            },
        );
        vertex_buffer.insert(
            queue,
            render_wireframe::VertexInput {
                position: to,
                model_matrix_id,
            },
        );
    }

    Wireframe {
        model_matrix_id,
        vertex_buffer_offset,
    }
}
