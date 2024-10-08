use crate::hittable::HitRecord;
use crate::pdf::PDF;
use crate::ray::Ray;
use crate::texture::Texture;
use nalgebra::Vector3;
use rand::Rng;
use std::f32;

fn random_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::thread_rng();
    let unit = Vector3::new(1.0, 1.0, 1.0);
    loop {
        let p = 2.0 * Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()) - unit;
        if p.magnitude_squared() < 1.0 {
            return p;
        }
    }
}

fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(n) * n
}

fn refract(v: &Vector3<f32>, n: &Vector3<f32>, ni_over_nt: f32) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(n);
    let discriminant = 1.0 - ni_over_nt.powi(2) * (1.0 - dt.powi(2));
    if discriminant > 0.0 {
        let refracted = ni_over_nt * (uv - n * dt) - n * discriminant.sqrt();
        Some(refracted)
    } else {
        None
    }
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

pub enum ScatterRecord<'a> {
    Specular {
        specular_ray: Ray,
        attenuation: Vector3<f32>,
    },
    Scatter {
        pdf: PDF<'a>,
        attenuation: Vector3<f32>,
    },
}

pub trait Material: Sync {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn scattering_pdf(&self, _ray: &Ray, _hit: &HitRecord, _scattered: &Ray) -> f32 {
        1.0
    }

    fn emitted(&self, _ray: &Ray, _hit: &HitRecord) -> Vector3<f32> {
        Vector3::zeros()
    }
}

#[derive(Clone)]
pub struct Lambertian<T: Texture> {
    albedo: T,
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Self {
        Lambertian { albedo }
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord::Scatter {
            pdf: PDF::cosine(hit.normal),
            attenuation: self.albedo.value(hit.u, hit.v, &hit.p),
        })
    }

    fn scattering_pdf(&self, _ray: &Ray, hit: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit.normal.dot(&scattered.direction().normalize()).max(0.0);
        cosine / f32::consts::PI
    }
}

#[derive(Clone)]
pub struct Metal {
    albedo: Vector3<f32>,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vector3<f32>, fuzz: f32) -> Self {
        Metal {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
        let mut reflected = reflect(&ray.direction().normalize(), &hit.normal);
        if self.fuzz > 0.0 {
            reflected += self.fuzz * random_in_unit_sphere()
        };
        if reflected.dot(&hit.normal) > 0.0 {
            Some(ScatterRecord::Specular {
                specular_ray: Ray::new(hit.p, reflected, ray.time()),
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Dielectric {
    ref_idx: f32,
}

impl Dielectric {
    pub fn new(ref_idx: f32) -> Self {
        Dielectric { ref_idx }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Vector3::new(1.0, 1.0, 1.0);
        let (outward_normal, ni_over_nt, cosine) = if ray.direction().dot(&hit.normal) > 0.0 {
            let cosine =
                self.ref_idx * ray.direction().dot(&hit.normal) / ray.direction().magnitude();
            (-hit.normal, self.ref_idx, cosine)
        } else {
            let cosine = -ray.direction().dot(&hit.normal) / ray.direction().magnitude();
            (hit.normal, 1.0 / self.ref_idx, cosine)
        };
        if let Some(refracted) = refract(&ray.direction(), &outward_normal, ni_over_nt) {
            let reflect_prob = schlick(cosine, self.ref_idx);
            if rand::thread_rng().gen::<f32>() >= reflect_prob {
                return Some(ScatterRecord::Specular {
                    specular_ray: Ray::new(hit.p, refracted, ray.time()),
                    attenuation,
                });
            }
        }
        let reflected = reflect(&ray.direction(), &hit.normal);
        Some(ScatterRecord::Specular {
            specular_ray: Ray::new(hit.p, reflected, ray.time()),
            attenuation,
        })
    }
}

#[derive(Clone)]
pub struct DiffuseLight<T: Texture> {
    emit: T,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(emit: T) -> Self {
        DiffuseLight { emit }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vector3<f32> {
        if hit.normal.dot(&ray.direction()) < 0.0 {
            self.emit.value(hit.u, hit.v, &hit.p)
        } else {
            Vector3::zeros()
        }
    }
}
