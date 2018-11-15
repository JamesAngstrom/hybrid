use amethyst::{
    ecs::prelude::*,
    core::timing::{Time},
    core::nalgebra::{
        base::{Unit},
        Point, Point3, Isometry3, Vector3
    },
    core::Transform,
    renderer::{DebugLinesComponent, Rgba}
};
use gilrs::{Event, Button::*, Axis::*};
use gilrs::ev::EventType::*;
use glm;
use ncollide3d::query::{Ray, RayCast, PointQuery};

use std::f32::consts::*;
use std::time::Instant;

use hybrid::Ball;
use hybrid::Chunk;

pub struct BallSystem {
    pub velocity: glm::Vec3,
    pub rotation: f32,
    pub left_stick: glm::Vec2,
    pub right_stick: glm::Vec2
}

impl BallSystem {
    pub fn new() -> Self {
        BallSystem {
            velocity: glm::vec3(0.0, 0.0, 0.0),
            rotation: 0.0,
            left_stick: glm::vec2(0.0, 0.0),
            right_stick: glm::vec2(0.0, 0.0)
        }
    }
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
        WriteStorage<'s, DebugLinesComponent>,
        Write<'s, Vec<Event>>,
        Read<'s, Time>
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (balls, chunks, mut transforms, mut debuglines, mut events, time): Self::SystemData) {
        let start = Instant::now();

        for (_ball, mut transform, mut debugline) in (&balls, &mut transforms, &mut debuglines).join() {
            let start = Instant::now();

            for event in events.drain(..) {
                match event {
                    Event { id: _, event: ButtonPressed(South, _), time: _ } => {
                        transform.translate_x(-1.0);
                    },
                    Event { id: _, event: AxisChanged(LeftStickX, x, _), time: _ } =>
                        self.left_stick.x = x,
                    Event { id: _, event: AxisChanged(LeftStickY, y, _), time: _ } =>
                        self.left_stick.y = y,
                    Event { id: _, event: AxisChanged(RightStickX, x, _), time: _ } =>
                        self.right_stick.x = x,
                    Event { id: _, event: AxisChanged(RightStickY, y, _), time: _ } =>
                        self.right_stick.y = y,

                    _ => ()
                }
            };

            let elapsed = start.elapsed();
            println!("* Input: {:?}", elapsed);

            let start = Instant::now();

            let mut intersection_point = None;

            for chunk in (&chunks).join() {
                let point = Point::from(*transform.translation());

                if chunk.bounding_box.contains_point(&Isometry3::identity(), &point) {
                    // We find our intersection point with the bezier surface by first raycasting down, and if that fails raycast up.
                    for direction in [-1.0, 1.0].iter() {
                        let ray = Ray {
                            origin: point,
                            dir: glm::vec3(0.0, *direction, 0.0)
                        };

                        match chunk.collision_mesh.toi_and_normal_and_uv_with_ray(&Isometry3::identity(), &ray, false) {
                            Some(hit) => {
                                let uv = hit.uvs.unwrap();
                                const BEZIER_SMOOTHNESS: i32 = 256; // Higher = smoother
                                let normal = chunk.patch.normal(BEZIER_SMOOTHNESS, clamp(uv.x), clamp(uv.y));

                                intersection_point = Some((ray.origin + ray.dir * hit.toi, normal, direction));
                                break
                            }
                            None => ()
                        }
                    }
                };
                if intersection_point.is_some() { break };
            };

            let elapsed = start.elapsed();
            // debug format:
            println!("* Collision: {:?}", elapsed);

            let start = Instant::now();

            const SPEED: f32 = 20.0;
            const MASS: f32 = 80.0;
            const DRAG_COEFFICIENT: f32 = 1.0;

            let gravity = glm::vec3(0.0, -0.0098, 0.0);
            let up = glm::vec3(0.0, 1.0, 0.0);
            let speed = self.velocity.magnitude();
            let drag_scalar = DRAG_COEFFICIENT * (f32::powi(speed, 2) / 2.0);
            let drag = if speed >= 0.001 { self.velocity.normalize() * -drag_scalar } else { glm::vec3(0.0, 0.0, 0.0) };

            self.rotation += self.left_stick.x / 30.0;
            self.rotation = if self.rotation >= 2.0 * PI { self.rotation - 2.0 * PI } else { self.rotation };
            self.rotation = if self.rotation <  0.0 * PI { self.rotation + 2.0 * PI } else { self.rotation };

            match intersection_point {
                None => {
                    // Player is not within any surface collision box, and free-falling
                    let accel = (MASS * gravity + drag) / MASS;
                    self.velocity += accel * time.delta_seconds();
                    transform.translate(self.velocity);
                },
                Some((p, normal, _)) => {
                    // How soft the surface is
                    const SQUISHYNESS: f32 = 1.0;
                    let height = transform.translation().y - p.y;
                    if height >= 0.0 {
                        let squish = if height <= SQUISHYNESS { f32::sin(height * PI / (SQUISHYNESS * 2.0)) } else { 1.0 };
                        let accel = (MASS * gravity + drag) / MASS;
                        self.velocity += accel * (squish * 0.01) * time.delta_seconds();
                        transform.translate(self.velocity);
                    };
                    if transform.translation().y <= p.y {
                        transform.set_y(p.y);
                    }

                    //transform.translation.x = p.x;
                    //transform.translation.y = p.y;

                    //debugline.clear();

                    let angle = glm::rotate_vec3(&(up.cross(&normal)), -(0.5 * PI), &normal);
                    let rotation = glm::quat_angle_axis(self.rotation, &normal);
                    let dir = glm::quat_cross_vec(&rotation, &angle);
                    //let dir2 = glm::rotate_vec3(&(dir.cross(&normal)), -(0.5 * PI), &normal);
                    //let dir2 = if dir2.z >= 0.0 { dir2 * -1.0 } else { dir2 };

                    transform.translate(dir * 0.2 * -dir.y);

                    //debugline.add_direction(Point3::new(p.x, p.y + 2.0, p.z), angle * 2.0, Rgba::green());
                    //debugline.add_direction(Point3::new(p.x, p.y + 2.0, p.z), angle * -2.0, Rgba::green());
                    //debugline.add_direction(Point3::new(p.x, p.y + 2.0, p.z), dir * 2.0, Rgba::red());
                    //debugline.add_direction(Point3::new(p.x, p.y + 2.0, p.z), dir * -2.0, Rgba::white());
                    //debugline.add_direction(Point3::new(p.x, p.y, p.z), normal * 5.0, Rgba::blue());

                    transform.look_at(Vector3::new(p.x, p.y, p.z) + dir, up);
                }
            }

            transform.translate_x(self.right_stick.x * SPEED * time.delta_seconds());
            transform.translate_z(-self.right_stick.y * SPEED * time.delta_seconds());

            let elapsed = start.elapsed();
            println!("* Physics: {:?}", elapsed);
        }

        let elapsed = start.elapsed();
        println!("Player movement system: {:?}", elapsed);

    }
}