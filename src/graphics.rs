use std::cmp::Ordering::Equal;
#[allow(dead_code)]
use crate::{Bitmap, Pixel};

use nalgebra as na;

pub const WIDTH: i32 = 960;
pub const HEIGHT: i32 = 540;
pub const FIELD_OF_VIEW: f32 = 60.0;
pub const Z_MIN: f32 = 1.0;
pub const Z_MAX: f32 = 10.0;
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

type Rgb = (u8, u8, u8);
type P3 = na::Point3<f32>;
type V3 = na::Vector3<f32>;
type T3 = STriangle<3>;
type Triangle2D = STriangle<2>;


fn draw_low(bitmap: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, color: &Rgb) {
    let dx = x1 - x0;
    let mut dy = y1 - y0;
    let yi = if dy < 0 {
        dy = -dy;
        -1
    } else {
        1
    };

    let mut diff = 2 * dy - dx;
    let mut y = y0;
    for x in x0..x1 {
        bitmap.set_pixel(x as usize, y as usize, Pixel::from_rgb_tuple(*color));
        if diff > 0 {
            y += yi;
            diff += 2 * (dy - dx);
        } else {
            diff += 2 * dy;
        }
    }
}


fn draw_high(bitmap: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, color: &Rgb) {
    let mut dx = x1 - x0;
    let dy = y1 - y0;
    let xi = if dx < 0 {
        dx = -dx;
        -1
    } else {
        1
    };

    let mut diff = 2 * dx - dy;
    let mut x = x0;

    for y in y0..y1 {
        bitmap.set_pixel(x as usize, y as usize, Pixel::from_rgb_tuple(*color));
        if diff > 0 {
            x += xi;
            diff += 2 * (dx - dy);
        } else {
            diff += 2 * dx;
        }
    }
}


/// Draws a line between points specified.
///
/// Implementation of general bresenham algorithm using integer arithmetic.
pub fn draw_line(bitmap: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, color: &Rgb) {
    if (y1 - y0).abs() < (x1 - x0).abs() {
        if x0 > x1 {
            draw_low(bitmap, x1, y1, x0, y0, color);
        } else {
            draw_low(bitmap, x0, y0, x1, y1, color);
        }
    } else if y0 > y1 {
        draw_high(bitmap, x1, y1, x0, y0, color);
    } else {
        draw_high(bitmap, x0, y0, x1, y1, color);
    }
}


/// Line drawing algorithm optimized for drawing horizontal lines.
fn draw_horizontal_line(bitmap: &mut Bitmap, y: i32, x0: i32, x1: i32, color: &Rgb) {
    let (x_min, x_max) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
    for x in x_min..=x_max {
        bitmap.set_pixel(x as usize, y as usize, Pixel::from_rgb_tuple(*color));
    }
}


/// Line drawing algorithm optimized for drawing vertical lines.
fn draw_vertical_line(bitmap: &mut Bitmap, x: i32, y0: i32, y1: i32, color: &Rgb) {
    let (y_min, y_max) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
    for y in y_min..=y_max {
        bitmap.set_pixel(x as usize, y as usize, Pixel::from_rgb_tuple(*color));
    }
}


/// Fills the upper part of the split triangle
fn fill_top_triangle(bitmap: &mut Bitmap, v1: &na::Vector2<i32>, v2: &na::Vector2<i32>, v3: &na::Vector2<i32>, color: &Rgb) {
    let inverse_slope1 = (v2.x - v1.x) as f32 / (v2.y - v1.y) as f32;
    let inverse_slope2 = (v3.x - v1.x) as f32 / (v3.y - v1.y) as f32;

    let mut x_1 = v1.x as f32;
    let mut x_2 = v1.x as f32;

    for y in (v1.y as i32)..=(v3.y as i32) {
        draw_horizontal_line(bitmap, y, x_1 as i32, x_2 as i32, color);
        x_1 += inverse_slope1;
        x_2 += inverse_slope2;
    }
}


/// Fills the bottom part of the split triangle
fn fill_bottom_triangle(bitmap: &mut Bitmap, v1: &na::Vector2<i32>, v2: &na::Vector2<i32>, v3: &na::Vector2<i32>, color: &Rgb) {
    let inverse_slope1 = (v3.x - v1.x) as f32 / (v3.y - v1.y) as f32;
    let inverse_slope2 = (v3.x - v2.x) as f32 / (v3.y - v2.y) as f32;

    let mut x_1 = v3.x as f32;
    let mut x_2 = v3.x as f32;

    for y in ((v1.y as i32)..=(v3.y as i32)).rev() {
        draw_horizontal_line(bitmap, y, x_1 as i32, x_2 as i32, color);
        x_1 -= inverse_slope1;
        x_2 -= inverse_slope2;
    }
}



pub struct STriangle<const S: usize> {
    vertices: [na::SVector<i32, S>; 3],
}


impl<const S: usize> From<[na::SVector<i32, S>; 3]> for STriangle<S> {
    fn from(vertices: [na::SVector<i32, S>; 3]) -> Self {
        Self { vertices }
    }
}


impl<const S: usize> STriangle<S> {
    pub fn new(v0: na::SVector<i32, S>, v1: na::SVector<i32, S>, v2: na::SVector<i32, S>) -> Self {
        Self { vertices: [v0, v1, v2] }
    }
}


/// Triangle's vertices go counter clockwise.
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    v0: P3,
    v1: P3,
    v2: P3,
    normal: V3,
}


impl From<[P3; 3]> for Triangle {
    fn from(vertices: [P3; 3]) -> Self {
        let [v0, v1, v2] = vertices;
        Self::new(v0, v1, v2)
    }
}


impl Triangle {
    pub fn new(v0: P3, v1: P3, v2: P3) -> Self {
        let vec_01 = V3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let vec_02 = V3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
        
        Self { v0, v1, v2, normal: (vec_01.cross(&vec_02)).normalize() }
    }

    pub fn normal_vector(v0: &P3, v1: &P3, v2: &P3) -> V3 {
        let vec_01 = V3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let vec_02 = V3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
        (vec_01.cross(&vec_02)).normalize()
    }

    pub fn vertices(&self) -> impl Iterator<Item=P3> {
        [self.v0, self.v1, self.v2].into_iter()
    }

    pub fn apply_rotation(&mut self, transform: &na::Rotation3<f32>) {
        self.v0 = transform.transform_point(&self.v0);
        self.v1 = transform.transform_point(&self.v1);
        self.v2 = transform.transform_point(&self.v2);
    }

    pub fn apply_isometry(&mut self, transform: &na::Isometry3<f32>) {
        self.v0 = transform.transform_point(&self.v0);
        self.v1 = transform.transform_point(&self.v1);
        self.v2 = transform.transform_point(&self.v2);
    }

    pub fn apply_translation(&mut self, transform: &na::Translation3<f32>) {
        self.v0 = transform.transform_point(&self.v0);
        self.v1 = transform.transform_point(&self.v1);
        self.v2 = transform.transform_point(&self.v2);
    }

    pub fn apply_scaling(&mut self, transform: &na::Scale3<f32>) {
        self.v0 = transform.transform_point(&self.v0);
        self.v1 = transform.transform_point(&self.v1);
        self.v2 = transform.transform_point(&self.v2);
    }

    pub fn apply_projection(&mut self, transform: &na::Perspective3<f32>) {
        self.v0 = transform.project_point(&self.v0);
        self.v1 = transform.project_point(&self.v1);
        self.v2 = transform.project_point(&self.v2);
    }
}


pub fn fill_triangle(bitmap: &mut Bitmap, triangle: &mut Triangle2D, color: &Rgb) {
    triangle.vertices.sort_unstable_by(|point1, point2| { point1.y.partial_cmp(&point2.y).unwrap() });

    let [v1, v2, v3] = &triangle.vertices;
    if v2.y == v3.y {
        fill_top_triangle(bitmap, v1, v2, v3, color);
    } else if v1.y == v2.y {
        fill_bottom_triangle(bitmap, v1, v2, v3, color);
    } else {
        let v4 = na::Vector2::<i32>::new(
            v1.x + ((v2.y - v1.y) as f32 / (v3.y - v1.y) as f32 * (v3.x - v1.x) as f32) as i32,
            v2.y
        );
        fill_top_triangle(bitmap, v1, v2, &v4, color);
        fill_bottom_triangle(bitmap, v2, &v4, v3, color);
    }
}


pub struct Mesh {
    triangles: Vec<Triangle>,
    model_processing_buffer: Vec<Triangle>,
    color: Rgb,
    angle_acc: f32,
    timer: std::time::Instant,
    translation: na::Isometry3<f32>,
    projection: na::Perspective3<f32>,
    screen_space_translation: na::Translation3<f32>,
    screen_space_scaling: na::Scale3<f32>,
    eye: P3,
    target: P3,
}


impl Mesh {
    pub fn new(triangles: Vec<Triangle>, color: Rgb) -> Self {
        let eye = P3::new(-0.5, -0.5, 10.0);
        let target = P3::new(-0.5, -0.5, 9.0);
        let up = V3::y();

        let translation = na::Isometry3::face_towards(&eye, &target, &up);

        let fov = std::f32::consts::FRAC_PI_6;
        let aspect_ratio = ASPECT_RATIO;
        let znear = 0.1;
        let zfar = 100.0;

        let projection = na::Perspective3::new(aspect_ratio, fov, znear, zfar);

        let screen_space_translation = na::Translation3::new(1.0, 1.0, 0.0);
        let screen_space_scaling = na::Scale3::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, 1.0);

        let model_processing_buffer = Vec::with_capacity(36);

        Self {
            triangles,
            model_processing_buffer,
            color,
            angle_acc: 0.0,
            timer: std::time::Instant::now(),
            projection,
            translation,
            screen_space_translation,
            screen_space_scaling,
            eye,
            target
        }
    }

    pub fn update(&mut self, bitmap: &mut Bitmap) {
        let z_rot = na::Rotation3::new(V3::z() * std::f32::consts::FRAC_PI_6 * self.angle_acc);
        let x_rot = na::Rotation3::new(V3::x() * std::f32::consts::FRAC_PI_2 * self.angle_acc);
        let y_rot = na::Rotation3::new(V3::y() * std::f32::consts::FRAC_PI_2 * self.angle_acc * 0.5);
        self.angle_acc += self.timer.elapsed().as_secs_f32();
        self.timer = std::time::Instant::now();

        self.model_processing_buffer.clear();

        let camera_direction = V3::new(
            self.eye.x - self.target.x,
            self.eye.y - self.target.y,
            self.eye.z - self.target.z
        );
        let normalized_camera_direction = camera_direction.normalize();

        for triangle in &mut self.triangles {
            // triangle.apply_rotation(&y_rot);
            // triangle.apply_isometry(&self.translation);

            // let normal = triangle.normal_vector();
            // println!("{}", normal.dot(&normalized_camera_direction));
            // if normal.dot(&normalized_camera_direction) > 0.0 {
            //     triangle.apply_projection(&self.projection);
            //     triangle.apply_translation(&self.screen_space_translation);
            //     triangle.apply_scaling(&self.screen_space_scaling);
            //     for vertex in triangle.vertices() {
            //         println!("{}", &vertex);
            //         self.model_processing_buffer.push((vertex.x as i32, vertex.y as i32));
            //     }
            // }
            let mut normals = Vec::<P3>::with_capacity(3);
            let mut points = Vec::<P3>::with_capacity(3);
            for vertex in triangle.vertices() {
                let z_rotated = z_rot.transform_point(&vertex);
                let xz_rotated = x_rot.transform_point(&z_rotated);
                //let y_rotated = y_rot.transform_point(&vertex);
                let moved = self.translation.transform_point(&xz_rotated);
                normals.push(moved.clone());

                let projected = self.projection.project_point(&moved);
                let screen_trans = self.screen_space_translation.transform_point(&projected);
                let screen_scaled = self.screen_space_scaling.transform_point(&screen_trans);
                points.push(screen_scaled);
            }
            let normal = Triangle::normal_vector(&normals[0], &normals[1], &normals[2]);
            self.model_processing_buffer.push(Triangle { v0: points[0], v1: points[1], v2: points[2], normal } );
        }
        self.model_processing_buffer.sort_unstable_by(|triangle1, triangle2| {
            ((triangle1.v0.z + triangle1.v1.z + triangle1.v2.z) / 3.0).partial_cmp(
                &((triangle2.v0.z + triangle2.v1.z + triangle2.v2.z) / 3.0)).unwrap_or(Equal)
            });
        for triangle in &self.model_processing_buffer {
            let normal = triangle.normal;
            let color_coef = normal.dot(&normalized_camera_direction);
            if color_coef < 0.0 {
                let color = (
                    (-color_coef * 255.0) as u8,
                    (-color_coef * 255.0) as u8,
                    0,
                );
                draw_triangle_outline(bitmap,
                                     triangle.v0.x as i32,
                                     triangle.v0.y as i32,
                                     triangle.v1.x as i32,
                                     triangle.v1.y as i32,
                                     triangle.v2.x as i32,
                                     triangle.v2.y as i32,
                                     &(0,0,0));
                draw_filled_triangle(bitmap,
                                     triangle.v0.x as i32,
                                     triangle.v0.y as i32,
                                     triangle.v1.x as i32,
                                     triangle.v1.y as i32,
                                     triangle.v2.x as i32,
                                     triangle.v2.y as i32,
                                     &color);
            }
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        let unit_cube_model: Vec<Triangle> = vec![
            // SOUTH
            Triangle::new(P3::new( 0.0, 0.0, 0.0), P3::new(0.0, 1.0, 0.0), P3::new(1.0, 1.0, 0.0 )),
            Triangle::new(P3::new( 0.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(1.0, 0.0, 0.0 )) ,

            // EAST
            Triangle::new(P3::new( 1.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(1.0, 1.0, 1.0 )),
            Triangle::new(P3::new( 1.0, 0.0, 0.0), P3::new(1.0, 1.0, 1.0), P3::new(1.0, 0.0, 1.0 )),

            // NORTH
            Triangle::new(P3::new( 1.0, 0.0, 1.0), P3::new(1.0, 1.0, 1.0), P3::new(0.0, 1.0, 1.0 )),
            Triangle::new(P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 1.0, 1.0), P3::new(0.0, 0.0, 1.0 )),

            // WEST
            Triangle::new(P3::new( 0.0, 0.0, 1.0), P3::new(0.0, 1.0, 1.0), P3::new(0.0, 1.0, 0.0 )),
            Triangle::new(P3::new( 0.0, 0.0, 1.0), P3::new(0.0, 1.0, 0.0), P3::new(0.0, 0.0, 0.0 )),

            // TOP
            Triangle::new(P3::new( 0.0, 1.0, 0.0), P3::new(0.0, 1.0, 1.0), P3::new(1.0, 1.0, 1.0 )),
            Triangle::new(P3::new( 0.0, 1.0, 0.0), P3::new(1.0, 1.0, 1.0), P3::new(1.0, 1.0, 0.0 )),

            // BOTTOM
            Triangle::new(P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 0.0, 1.0), P3::new(0.0, 0.0, 0.0 )),
            Triangle::new(P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 0.0, 0.0), P3::new(1.0, 0.0, 0.0 )),
        ];

        let color = (100, 100, 100);

        Self::new(
            unit_cube_model,
            color
        )
    }
}


pub fn draw_triangle_outline(bitmap: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: &Rgb) {
    draw_line(bitmap, x0, y0, x1, y1, &color);
    draw_line(bitmap, x0, y0, x2, y2, &color);
    draw_line(bitmap, x1, y1, x2, y2, &color);
}


pub fn draw_lines(bitmap: &mut Bitmap) {
    let triangle = Triangle2D::from([
        na::Vector2::new(WIDTH / 2, HEIGHT / 4),
        na::Vector2::new(WIDTH / 4, 3 * HEIGHT / 4),
        na::Vector2::new(3 * WIDTH / 4, HEIGHT / 2)
    ]);

    let color = (255, 255, 0);

    draw_line(bitmap, triangle.vertices[0].x, triangle.vertices[0].y, triangle.vertices[1].x, triangle.vertices[1].y, &color); // top left
    draw_line(bitmap, triangle.vertices[0].x, triangle.vertices[0].y, triangle.vertices[2].x, triangle.vertices[2].y, &color); // top right
    draw_line(bitmap, triangle.vertices[1].x, triangle.vertices[1].y, triangle.vertices[2].x, triangle.vertices[2].y, &color); // bottom line
}


pub fn draw_filled_triangle(bitmap: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: &Rgb) {
    let mut triangle = Triangle2D::from([
        na::Vector2::new(x0, y0),
        na::Vector2::new(x1, y1),
        na::Vector2::new(x2, y2)
    ]);

    fill_triangle(bitmap, &mut triangle, &color);
}


//region
// pub fn projection(bitmap: &mut Bitmap) {
//     let unit_cube_model: Vec<P3> = vec![
//         // SOUTH
//         P3::new(0.0, 0.0, 0.0), P3::new(0.0, 1.0, 0.0), P3::new(1.0, 1.0, 0.0),
//         P3::new( 0.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(1.0, 0.0, 0.0) ,
//
//         // EAST
//         P3::new( 1.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(1.0, 1.0, 1.0 ),
//         P3::new( 1.0, 0.0, 0.0), P3::new(1.0, 1.0, 1.0), P3::new(1.0, 0.0, 1.0 ),
//
//         // NORTH
//         P3::new( 1.0, 0.0, 1.0), P3::new(1.0, 1.0, 1.0), P3::new(0.0, 1.0, 1.0 ),
//         P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 1.0, 1.0), P3::new(0.0, 0.0, 1.0 ),
//
//         // WEST
//         P3::new( 0.0, 0.0, 1.0), P3::new(0.0, 1.0, 1.0), P3::new(0.0, 1.0, 0.0 ),
//         P3::new( 0.0, 0.0, 1.0), P3::new(0.0, 1.0, 0.0), P3::new(0.0, 0.0, 0.0 ),
//
//         // TOP
//         P3::new( 0.0, 1.0, 0.0), P3::new(0.0, 1.0, 1.0), P3::new(1.0, 1.0, 1.0 ),
//         P3::new( 0.0, 1.0, 0.0), P3::new(1.0, 1.0, 1.0), P3::new(1.0, 1.0, 0.0 ),
//
//         // BOTTOM
//         P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 0.0, 1.0), P3::new(0.0, 0.0, 0.0 ),
//         P3::new( 1.0, 0.0, 1.0), P3::new(0.0, 0.0, 0.0), P3::new(1.0, 0.0, 0.0)
//     ];
//
//     let eye = P3::new(0.0, 0.0, - 5.0);
//     let target = P3::new(0.0, 0.0, 4.0);
//     let up = V3::y();
//
//     let translation = na::Isometry3::face_towards( & eye, & target, & up);
//
//     let fov = std::f32::consts::FRAC_PI_6;
//     let aspect_ratio = ASPECT_RATIO;
//     let znear = 0.1;
//     let zfar = 100.0;
//
//     let projection = na::Perspective3::new(aspect_ratio, fov, znear, zfar);
//
//     let screen_translation = na::Translation3::new(1.0, 1.0, 0.0);
//     let screen_scaling = na::Scale3::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, 1.0);
//     let z_rot = na::Rotation3::new(V3::z() * std::f32::consts::FRAC_PI_6);
//     let x_rot = na::Rotation3::new(V3::z() * std::f32::consts::FRAC_PI_3);
//
//     let color= (100, 100, 100);
//     let mut processed_model: Vec<(i32, i32) > = Vec::with_capacity(36);
//     for point in unit_cube_model {
//         let z_rotated = z_rot.transform_point( & point);
//         let xz_rotated = x_rot.transform_point( & z_rotated);
//         let moved = translation.transform_point( &xz_rotated);
//         let projected = projection.project_point(& moved);
//         let screen_trans = screen_translation.transform_point( & projected);
//         let screen_scaled = screen_scaling.transform_point( & screen_trans);
//         processed_model.push((screen_scaled.x as i32, screen_scaled.y as i32));
//     }
//     for chunk in processed_model.chunks(3) {
//         if let & [(x0, y0), (x1, y1), (x2, y2)] = chunk {
//             draw_triangle_outline(bitmap, x0, y0, x1, y1, x2, y2, & color);
//         }
//     }
// }
//endregion
