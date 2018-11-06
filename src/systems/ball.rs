use amethyst::{
    ecs::prelude::*,
    core::cgmath::{Rotation3, InnerSpace},
    core::Transform,
};
use amethyst::core::cgmath as cgmath;
use gilrs::{Event, Button::*};
use gilrs::ev::EventType::*;
use glm;
use nalgebra::geometry::{Point3, Isometry3};
use ncollide3d::query::{Ray, RayCast, PointQuery};

use std::f32::consts::*;

use hybrid::Ball;
use hybrid::Chunk;

pub struct BallSystem {
    pub reader: Option<ReaderId<Event>>,
}

fn clamp(n: f32) -> f32 {
    if n <= 0.0 {
        0.0
    } else if n >= 1.0 {
        1.0
    } else {
        n
    }
}

impl<'s> System<'s> for BallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        ReadStorage<'s, Chunk>,
        WriteStorage<'s, Transform>,
        Write<'s, Vec<Event>>
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = None;
    }

    fn run(&mut self, (balls, chunks, mut transforms, mut events): Self::SystemData) {
        for (_ball, mut transform) in (&balls, &mut transforms).join() {

            for event in events.drain(..) {
                println!("Recieved {:?}", event);
                match event {
                    Event { id: _, event: ButtonPressed(South, _), time: _ } =>
                        transform.translation.x -= 1.0,
                    _ => ()
                }
            };

            let mut intersection_point = None;
            for chunk in (&chunks).join() {
                let point = Point3::new(transform.translation.x, transform.translation.y, transform.translation.z);

                if chunk.bounding_box.contains_point(&Isometry3::identity(), &point) {
                    // We find our intersection point with the bezier surface by first raycasting down, and if that fails raycast up.
                    for direction in [-1.0, 1.0].iter() {
                        let ray = Ray {
                            origin: point,
                            dir: glm::vec3(0.0, 0.0, *direction)
                        };

                        match chunk.collision_mesh.toi_and_normal_and_uv_with_ray(&Isometry3::identity(), &ray, false) {
                            Some(hit) => {
                                let uv = hit.uvs.unwrap();
                                const BEZIER_SMOOTHNESS: i32 = 256; // Higher = smoother
                                let normal = chunk.patch.normal(BEZIER_SMOOTHNESS, clamp(uv.x), clamp(uv.y));

                                println!("{:?}", uv);
                                intersection_point = Some((ray.origin + ray.dir * hit.toi, normal, direction));
                                break
                            }
                            None => ()
                        }
                    }
                };
                if intersection_point.is_some() { break };
            };

            match intersection_point {
                None => (),
                Some((p, normal, _)) => {
                    transform.translation.x = p.x;
                    transform.translation.y = p.y;
                    transform.translation.z = p.z + 0.5;
                    println!("{:?}", normal);

                    transform.rotation =
                        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::new(0.0, 1.0, 0.0), cgmath::Rad(0.5 * PI)) *
                        cgmath::Quaternion::from_sv(1.0, cgmath::Vector3::new(normal.x, normal.y, normal.z)).normalize();
                }
            }

            transform.translation.x += 0.02
        }
    }
}