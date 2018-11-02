use amethyst::{
    ecs::prelude::*,
    core::Transform,
};
use gilrs::{Event, Button::*};
use gilrs::ev::EventType::*;

use hybrid::Ball;

pub struct BallSystem {
    pub reader: Option<ReaderId<Event>>,
}

impl<'s> System<'s> for BallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Write<'s, Vec<Event>>
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = None;
    }

    fn run(&mut self, (balls, mut transforms, mut events): Self::SystemData) {
        for (_ball, mut transform) in (&balls, &mut transforms).join() {

            for event in events.drain(..) {
                println!("Recieved {:?}", event);
                match event {
                    Event { id: _, event: ButtonPressed(South, _), time: _ } =>
                        transform.translation.x -= 1.0,
                    _ => ()
                }
            };

            transform.translation.x += 0.01
        }
    }
}