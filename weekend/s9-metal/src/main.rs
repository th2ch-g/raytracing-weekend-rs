use s9_metal::{
    camera::Camera,
    color::write_color,
    hittable::{HitRecord, Hittable, Shape},
    material::{Lambertian, Metal},
    ray::Ray,
    sphere::Sphere,
    util::random_f64,
    vec3::{dot, unit_vector, Color, Point3, Vec3},
};
use std::io::{self, Write};
use std::{fs::File, rc::Rc};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: i32 = 384;
const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
const SAMPLES_PER_PIXEL: i32 = 100;
const MAX_DEPTH: i32 = 50;

const COUNT_MAX: usize = IMAGE_HEIGHT as usize * IMAGE_WIDTH as usize;

fn ray_color(r: Ray, world: &Vec<Hittable>, depth: i32) -> Color {
    let mut rec = HitRecord {
        p: Point3::new(0.0, 0.0, 0.0),
        normal: Vec3::new(0.0, 0.0, 0.0),
        material: Rc::new(Lambertian::new(Vec3::new(0.0, 0.0, 0.0))),
        t: 0.0,
        front_face: false,
    };

    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut closest_so_far = f64::INFINITY;
    let mut hit_anything = false;
    let mut temp_rec = HitRecord {
        p: Point3::new(0.0, 0.0, 0.0),
        normal: Vec3::new(0.0, 0.0, 0.0),
        material: Rc::new(Lambertian::new(Vec3::new(0.0, 0.0, 0.0))),
        t: 0.0,
        front_face: false,
    };

    for hittable in world.iter() {
        temp_rec.material = Rc::clone(&hittable.material);
        if hittable.shape.hit(r, 0.001, closest_so_far, &mut temp_rec) {
            hit_anything = true;
            closest_so_far = temp_rec.t.clone();
            rec.p = temp_rec.p.clone();
            rec.normal = temp_rec.normal.clone();
            rec.material = Rc::clone(&temp_rec.material);
            rec.t = temp_rec.t.clone();
            rec.front_face = temp_rec.front_face.clone();
        }
    }

    if hit_anything {
        let mut scattered = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        let mut attenuation = Color::new(0.0, 0.0, 0.0);

        if Rc::clone(&rec.material).scatter(r, &rec, &mut attenuation, &mut scattered) {
            return attenuation * ray_color(scattered, world, depth - 1);
        }
        return Color::new(0.0, 0.0, 0.0);
    }

    let unit_direction = unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn main() -> io::Result<()> {
    let mut out_str = format!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    let world = vec![
        Hittable::new(
            Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5),
            Lambertian::new(Vec3::new(0.8, 0.3, 0.3)),
        ),
        Hittable::new(
            Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0),
            Lambertian::new(Vec3::new(0.8, 0.8, 0.0)),
        ),
        Hittable::new(
            Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5),
            Metal::new(Vec3::new(0.8, 0.6, 0.2), 1.0),
        ),
        Hittable::new(
            Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5),
            Metal::new(Vec3::new(0.8, 0.8, 0.8), 0.2),
        ),
    ];

    let cam = Camera::new();

    let mut data_vector = vec![String::from(""); COUNT_MAX];
    let mut index = 0;

    for j in (0..IMAGE_HEIGHT).rev() {
        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + random_f64()) / (IMAGE_WIDTH - 1) as f64;
                let v = (j as f64 + random_f64()) / (IMAGE_HEIGHT - 1) as f64;
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(r, &world, MAX_DEPTH);
            }
            data_vector[index] = write_color(pixel_color, SAMPLES_PER_PIXEL);
            index += 1;
        }
    }

    out_str += &data_vector.join("\n");

    let mut file = File::create("a.ppm").unwrap();
    file.write_fmt(format_args!("{}", out_str))?;
    Ok(())
}
