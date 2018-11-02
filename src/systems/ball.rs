use amethyst::{
    ecs::prelude::*,
    core::Transform,
    shrev::EventChannel,
};
use gilrs::Event;

use hybrid::Ball;

pub struct BallSystem;

impl<'s> System<'s> for BallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, EventChannel<Event>>
    );

    fn run(&mut self, (balls, mut transforms, input): Self::SystemData) {
        for (_ball, mut transform) in (&balls, &mut transforms).join() {
            transform.translation.x += 0.01
        }
    }
}