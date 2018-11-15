use amethyst::{
    ecs::prelude::*,
    core::Transform,
    core::nalgebra::{
        Vector3, Quaternion, UnitQuaternion, Unit
    },
    core::timing::{Time},
    renderer::{Camera}
};
use glm;

use std::time::Instant;

use hybrid::Follow;

pub struct FollowSystem {
    target: Option<Entity>
}

impl FollowSystem {
    pub fn new() -> Self {
        FollowSystem { target: None }
    }
}

impl<'s> System<'s> for FollowSystem {
    type SystemData = (
        ReadStorage<'s, Follow>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (followers, cameras, mut transforms, time): Self::SystemData) {
        let start = Instant::now();

        let point = match self.target {
            None => Vector3::new(0.0, 0.0, 0.0),
            Some(target) => {
                *transforms.get(target).unwrap().translation()
            }
        };

        for (follow, _camera, mut transform) in (&followers, &cameras, &mut transforms).join() {
            self.target = Some(follow.entity); // Won't take effect until next frame

            const SPEED: f32 = 10.0;
            let dir = point - transform.translation();
            if dir.magnitude() > 35.0 {
                transform.translate(dir.normalize() * SPEED * time.delta_seconds());
            }
            transform.set_y(point.y + 15.0);

            let dir = {
                let eye = transform.translation();
                (point - eye).normalize()
            };

            transform.look_at(point, Vector3::new(0.0, 1.0, 0.0));
        }
        let elapsed = start.elapsed();
        println!("Camera movement system: {:?}", elapsed);
    }
}