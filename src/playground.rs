#![allow(dead_code)]

use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;
use nalgebra::Matrix3;

use crate::util::{Vec3, AsRgb, Color, Ray, Point3};

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

pub fn test_image() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let c = Color::new(
                x as f64 / (IMAGE_WIDTH - 1) as f64,
                y as f64 / (IMAGE_HEIGHT - 1) as f64,
                0.25,
            );

            img.put_pixel(x, IMAGE_HEIGHT - 1 - y, c.as_rgb());
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("test_output.png").unwrap();
}

pub fn test_vectormath() {
    let mut v = Vec3::new(2., 3., 4.);
    let v2 = Vec3::new(1., 2., 3.);
    v[0] += 1.;
    v /= 2.;

    println!("{}, {}, {}", v[0], v[1], v[2]);
    println!("{}, {}, {}", v.x, v.y, v.z);
    println!("{}", v - v2);

    println!("{}", v.component_mul(&v2)); // v * v
    println!("{}", v.norm()); // length
    println!("{}", v.norm_squared()); // length_squared
    println!("{}", v.dot(&v2));
    println!("{}", v.normalize()); // unit_vector
}

pub fn test_ray_cylinder_math() {
    // Imaginge two lines C and P
    // C(t1) = K + t1 * l
    let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 0.0)*5.);
    // P(t2) = A + t2 * b
    let ray2 = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -2.0, 1.0));

    // Let F be the point on line C closed to line P and
    // let G be the point on line P closed to line C.

    // Plane through ray and the line G to F.
    // The line GtoF is the shortest connection from P to C and also always perpendicular to both lines.
    // ( If it would not be perpendicular, we could find a closer connection).
    // The cross product is by definition perpendicular to both operand vectors as well, so use that to span the plane.
    //  E(t3,t4) = K + t3 * l + t4 * n
    let n = ray.direction().cross(&ray2.direction());

    // Find point that is on E and P:
    // E(t3, t4) == P(t2)
    // t3 * l + t4 * n - t2 * b = A - K
    // M * t = A - K with M = columns(l;n;b) and t=(t3, t4, -t2)

    let m = Matrix3::from_columns(&[ray.direction(), n, ray2.direction()]);
    // t = M^-1 * (A - K)
    let t = m.try_inverse().unwrap() * (ray2.origin() - ray.origin());

    // Calculate G=P(t2)
    let t2 = -t.z;
    let p_ray2 = ray2.at(t2);
    // Should be the same as E(t3, t4)
    let p_ray2_check = ray.origin() + t.x * ray.direction() + t.y * n;

    // Calculate F=C(t1) by projecting the line KtoG onto C(_)
    let k_to_g = p_ray2 - ray.origin();
    let p_ray = ray.origin() + k_to_g.dot(&ray.direction()) / ray.direction().magnitude_squared() * ray.direction();

    // n and k are perpendicular, so E(t3, 0) should be F
    let p_ray_check = ray.origin() + t.x * ray.direction();

    println!("t={:?}", t);
    println!("p_ray2={:?}", p_ray2);
    println!("p_ray2={:?} check", p_ray2_check);
    println!("p_ray={:?}", p_ray);
    println!("p_ray={:?}", p_ray_check);
    
}