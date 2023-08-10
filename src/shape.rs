use crate::{
    material::MaterialId, model_matrices::ModelMatrixId, point::Point3, vector::Vec3,
    vertex::Vertex,
};

pub fn triangle(model_matrix_id: ModelMatrixId, material_id: MaterialId) -> Vec<Vertex> {
    vec![
        Vertex {
            position: Point3 {
                x: 0.5,
                y: -0.5,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -0.5,
                y: -0.5,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
    ]
}

pub fn square(model_matrix_id: ModelMatrixId, material_id: MaterialId, side: f32) -> Vec<Vertex> {
    let side_over_2 = side / 2.0;
    vec![
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
    ]
}

pub fn floor(model_matrix_id: ModelMatrixId, material_id: MaterialId, side: f32) -> Vec<Vertex> {
    let side_over_2 = side / 2.0;
    vec![
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: 0.0,
                z: -side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: 0.0,
                z: side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: 0.0,
                z: side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: 0.0,
                z: -side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: 0.0,
                z: -side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: 0.0,
                z: side_over_2,
            },
            model_matrix_id,
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            material_id,
        },
    ]
}

pub fn sphere(model_matrix_id: ModelMatrixId, material_id: MaterialId, radius: f32) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    // number of "longitude" lines
    let meridians = 32;

    // number of "latitude" lines
    let parallels = 32;

    let azimuth_per_meridian = std::f32::consts::TAU / meridians as f32;
    let elevation_per_parallel = std::f32::consts::PI / parallels as f32;

    // origin = (0, 0, 0)
    // "north" pole = (0, radius, 0)
    // "south" pole = (0, -radius, 0)
    for meridian in 0..meridians {
        let ring_radius = radius * elevation_per_parallel.sin();

        let azimuth = meridian as f32 * azimuth_per_meridian;
        let next_azimuth = azimuth + azimuth_per_meridian;

        let top = radius;
        let bottom = radius * f32::cos(elevation_per_parallel);

        let left_z = f32::cos(azimuth);
        let left_x = f32::sin(azimuth);

        let right_z = f32::cos(next_azimuth);
        let right_x = f32::sin(next_azimuth);

        let top = Point3 {
            x: 0.0,
            y: top,
            z: 0.0,
        };

        let bottom_left = Point3 {
            x: ring_radius * left_x,
            y: bottom,
            z: ring_radius * left_z,
        };
        let bottom_right = Point3 {
            x: ring_radius * right_x,
            y: bottom,
            z: ring_radius * right_z,
        };

        vertices.push(Vertex {
            position: top,
            model_matrix_id,
            normal: Vec3::Y,
            material_id,
        });
        vertices.push(Vertex {
            position: bottom_left,
            model_matrix_id,
            normal: Vec3::from(bottom_left).normalize(),
            material_id,
        });
        vertices.push(Vertex {
            position: bottom_right,
            model_matrix_id,
            normal: Vec3::from(bottom_right).normalize(),
            material_id,
        });
    }

    for parallel in 1..(parallels - 1) {
        let elevation = parallel as f32 * elevation_per_parallel;
        let next_elevation = elevation + elevation_per_parallel;

        let ring_radius = radius * elevation.sin().abs();
        let next_ring_radius = radius * next_elevation.sin().abs();

        for meridian in 0..meridians {
            let azimuth = meridian as f32 * azimuth_per_meridian;
            let next_azimuth = azimuth + azimuth_per_meridian;

            let top = radius * f32::cos(elevation);
            let bottom = radius * f32::cos(next_elevation);

            let left_z = f32::cos(azimuth);
            let left_x = f32::sin(azimuth);

            let right_z = f32::cos(next_azimuth);
            let right_x = f32::sin(next_azimuth);

            // top left = (azimuth, elevation)
            let top_left = Point3 {
                x: ring_radius * left_x,
                y: top,
                z: ring_radius * left_z,
            };

            // top right = (next_azimuth, elevation)
            let top_right = Point3 {
                x: ring_radius * right_x,
                y: top,
                z: ring_radius * right_z,
            };

            // bottom left = (azimuth, next_elevation)
            let bottom_left = Point3 {
                x: next_ring_radius * left_x,
                y: bottom,
                z: next_ring_radius * left_z,
            };

            // bottom right = (next_azimuth, next_elevation)
            let bottom_right = Point3 {
                x: next_ring_radius * right_x,
                y: bottom,
                z: next_ring_radius * right_z,
            };

            // TR
            vertices.push(Vertex {
                position: top_right,
                model_matrix_id,
                normal: Vec3::from(top_right).normalize(),
                material_id,
            });
            // BL
            vertices.push(Vertex {
                position: bottom_left,
                model_matrix_id,
                normal: Vec3::from(bottom_left).normalize(),
                material_id,
            });
            // BR
            vertices.push(Vertex {
                position: bottom_right,
                model_matrix_id,
                normal: Vec3::from(bottom_right).normalize(),
                material_id,
            });

            // TR
            vertices.push(Vertex {
                position: top_right,
                model_matrix_id,
                normal: Vec3::from(top_right).normalize(),
                material_id,
            });
            // TL
            vertices.push(Vertex {
                position: top_left,
                model_matrix_id,
                normal: Vec3::from(top_left).normalize(),
                material_id,
            });
            // BL
            vertices.push(Vertex {
                position: bottom_left,
                model_matrix_id,
                normal: Vec3::from(bottom_left).normalize(),
                material_id,
            });
        }
    }

    for meridian in 0..meridians {
        let ring_radius = radius * f32::sin(elevation_per_parallel);

        let azimuth = meridian as f32 * azimuth_per_meridian;
        let next_azimuth = azimuth + azimuth_per_meridian;

        let top = radius * f32::cos(std::f32::consts::PI - elevation_per_parallel);
        let bottom = -radius;

        let left_z = f32::cos(azimuth);
        let left_x = f32::sin(azimuth);

        let right_z = f32::cos(next_azimuth);
        let right_x = f32::sin(next_azimuth);

        let top_right = Point3 {
            x: ring_radius * right_x,
            y: top,
            z: ring_radius * right_z,
        };
        let top_left = Point3 {
            x: ring_radius * left_x,
            y: bottom,
            z: ring_radius * left_z,
        };
        let bottom = Point3 {
            x: 0.0,
            y: bottom,
            z: 0.0,
        };

        vertices.push(Vertex {
            position: top_right,
            model_matrix_id,
            normal: Vec3::from(top_right).normalize(),
            material_id,
        });
        vertices.push(Vertex {
            position: top_left,
            model_matrix_id,
            normal: Vec3::from(top_left).normalize(),
            material_id,
        });
        vertices.push(Vertex {
            position: bottom,
            model_matrix_id,
            normal: -Vec3::Y,
            material_id,
        });
    }

    vertices
}
