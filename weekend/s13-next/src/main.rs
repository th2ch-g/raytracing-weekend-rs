use s13_next::{
    camera::Camera,
    color::write_color,
    hittable::{HitRecord, Hittable},
    material::{Dielectric, Lambertian, Metal},
    ray::Ray,
    sphere::Sphere,
    util::{random_f64, random_f64_range},
    vec3::{unit_vector, Color, Point3, Vec3},
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
            closest_so_far = temp_rec.t;
            rec.p = temp_rec.p;
            rec.normal = temp_rec.normal;
            rec.material = Rc::clone(&temp_rec.material);
            rec.t = temp_rec.t;
            rec.front_face = temp_rec.front_face;
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

fn random_scene() -> Vec<Hittable> {
    let mut world = Vec::new();

    let groud_material = Lambertian::new(Vec3::new(0.5, 0.5, 0.5));
    world.push(Hittable::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0), groud_material));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_f64();
            let center = Point3::new(a as f64 + 0.9 * random_f64(), 0.2, b as f64 + 0.9 * random_f64());

            if (center - Point3::new(4.0, 0.2, 0.0)).len() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Vec3::random() * Vec3::random();
                    let sphere_material = Lambertian::new(albedo);
                    world.push(Hittable::new(Sphere::new(center, 0.2), sphere_material));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = random_f64_range(0.0, 0.5);
                    let sphere_material = Metal::new(albedo, fuzz);
                    world.push(Hittable::new(Sphere::new(center, 0.2), sphere_material));
                } else {
                    let sphere_material = Dielectric::new(1.5);
                    world.push(Hittable::new(Sphere::new(center, 0.2), sphere_material));
                }
            }
        }
    }

    world.push(Hittable::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0), Dielectric::new(1.5)));
    world.push(Hittable::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0), Lambertian::new(Vec3::new(0.4, 0.2, 0.1))));
    world.push(Hittable::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0), Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0)));
    world
}

fn main() -> io::Result<()> {
    let mut out_str = format!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    let world = random_scene();

    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

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
