use amethyst::{
    ecs::prelude::*,
    core::Transform,
    core::timing::{Time},
    core::cgmath::{Quaternion, Rotation, Vector3, InnerSpace},
    renderer::{Camera}
};
use glm;

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
        let point = match self.target {
            None => Vector3::new(0.0, 0.0, 0.0),
            Some(target) => {
                transforms.get(target).unwrap().translation
            }
        };

        for (follow, _camera, mut transform) in (&followers, &cameras, &mut transforms).join() {
            self.target = Some(follow.entity); // Won't take effect until next frame

            const SPEED: f32 = 10.0;
            let dir = point - transform.translation;
            if dir.magnitude() > 20.0 {
                transform.translation += dir.normalize() * SPEED * time.delta_seconds()
            }
            transform.translation.z = point.z + 5.0;

            let eye = transform.translation;
            let dir = (point - eye).normalize();
            let look = Quaternion::look_at(dir, Vector3::new(0.0, 0.0, 1.0));
            transform.rotation = Quaternion::new(look.v.y, look.v.z, look.s, -look.v.x);
            // let correct = Quaternion::new(look.v.y, look.v.z, look.s, -look.v.x);

            // let glm_lh = glm::quat_look_at_lh(&(glm::vec3(dir.x, dir.y, dir.z) * -1.0), &glm::vec3(0.0, 0.0, 1.0));
            // let lh = Quaternion::new(-glm_lh.coords.y, -glm_lh.coords.z, -glm_lh.coords.w, glm_lh.coords.x);

            // let glm_rh = glm::quat_look_at_rh(&glm::vec3(dir.x, dir.y, dir.z), &glm::vec3(0.0, 0.0, 1.0));
            // let rh = Quaternion::new(glm_rh.coords.w, -glm_rh.coords.x, -glm_rh.coords.y, -glm_rh.coords.z);

            //println!("{:?}, {:?}, {:?}", correct, rh, lh); // All the same
            // transform.rotation = correct //Quaternion::new(look.v.y, look.v.z, look.s, -look.v.x);
        }
    }
}