use crate::{point::Point3, vector::Vec3};

/// `plane.normal.dot(Vec3{x, y, z} - plane.point) == 0`
pub struct Plane {
    normal: Vec3,
    point: Point3,
}

impl Plane {
    pub fn new(normal: Vec3, point: Point3) -> Self {
        Plane {
            normal: normal.normalize(),
            point,
        }
    }

    fn distance_to_point(&self, point: Point3) -> f32 {
        /*
        Projection of `a` in the direction of `b` is `|a| cos theta = |a| * dot(a, b) / |a||b| = dot(a, b) / |b|`.

        The distance from a plane to point `P` is the projection of any vector running from a point `p` on the plane to `P` onto the
        plane's normal vector. Since the plane's normal has length 1, `dot(P - p, n) / |n| = dot(P - p, n)`.
         */
        let v = point - self.point;
        v.dot(self.normal)
    }

    fn intersect_segment(&self, from: Point3, to: Point3) -> Option<Point3> {
        let dir = to - from;

        let denom = dir.dot(self.normal);
        if denom == 0.0 {
            return None;
        }

        let t = (self.point - from).dot(self.normal) / denom;
        if (0.0..=1.0).contains(&t) {
            Some(from + t * dir)
        } else {
            None
        }
    }
}

#[test]
fn test_distance_to_point_1() {
    // Plane of `y = 1`
    let plane = Plane::new(
        Vec3::Y,
        Point3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    );

    assert_eq!(plane.distance_to_point(Point3::ZERO), -1.0);
    assert_eq!(
        plane.distance_to_point(Point3 {
            x: 1.0,
            y: 1.0,
            z: 1.0
        }),
        0.0
    );
    assert_eq!(
        plane.distance_to_point(Point3 {
            x: 33.0,
            y: 2.0,
            z: 21.0
        }),
        1.0
    );
}

pub struct Triangle(pub Point3, pub Point3, pub Point3);

impl IntoIterator for Triangle {
    type Item = Point3;

    type IntoIter = <[Point3; 3] as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        [self.0, self.1, self.2].into_iter()
    }
}

/// The result of clipping a triangle against a plane.
pub enum ClippedTriangle {
    /// The triangle is entirely "above" the plane. The plane's normal points "up".
    Accept,

    /// The triangle is entirely "below" the plane.
    Reject,

    /**
    The plane intersects the triangle. One vertex is accepted and two are rejected.
    The accepted vertex and the two plane-triangle intersection points form a new, fully
    accepted triangle.
    */
    Split1(Triangle),

    /**
    The plane intersects the triangle. Two vertices are accepted and one is rejected.
    The two accepted vertices and two plane-triangle intersection points form a fully accepted
    quadrilateral, which is split into two fully accepted triangles.
    */
    Split2(Triangle, Triangle),
}

pub fn clip_triangle(plane: &Plane, triangle: &Triangle) -> ClippedTriangle {
    let d0 = plane.distance_to_point(triangle.0);
    let d1 = plane.distance_to_point(triangle.1);
    let d2 = plane.distance_to_point(triangle.2);

    let accepted_count: usize = [d0, d1, d2]
        .into_iter()
        .map(|x| if x >= 0.0 { 1 } else { 0 })
        .sum();

    match accepted_count {
        0 => ClippedTriangle::Reject,
        1 => {
            let (accepted0, rejected) = if d0 >= 0.0 {
                (triangle.0, [triangle.1, triangle.2])
            } else if d1 >= 0.0 {
                (triangle.1, [triangle.0, triangle.2])
            } else {
                (triangle.2, [triangle.0, triangle.1])
            };

            let accepted1 = plane.intersect_segment(accepted0, rejected[0]).unwrap();
            let accepted2 = plane.intersect_segment(accepted0, rejected[1]).unwrap();

            ClippedTriangle::Split1(Triangle(accepted0, accepted1, accepted2))
        }
        2 => {
            let ([accepted0, accepted1], rejected) = if d0 < 0.0 {
                ([triangle.1, triangle.2], triangle.0)
            } else if d1 < 0.0 {
                ([triangle.0, triangle.2], triangle.1)
            } else {
                ([triangle.0, triangle.1], triangle.2)
            };

            let accepted2 = plane.intersect_segment(accepted0, rejected).unwrap();
            let accepted3 = plane.intersect_segment(accepted1, rejected).unwrap();

            ClippedTriangle::Split2(
                Triangle(accepted0, accepted1, accepted2),
                Triangle(accepted1, accepted3, accepted3),
            )
        }
        3 => ClippedTriangle::Accept,
        _ => unreachable!(),
    }
}
