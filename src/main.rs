extern crate amethyst;
extern crate gilrs;
extern crate rand;
extern crate nalgebra;
extern crate nalgebra_glm as glm;
extern crate ncollide3d;

mod hybrid;
mod systems;

use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    ecs::prelude::*,
    ecs::shred::ResourceId,
    core::EventReader,
    assets::PrefabLoaderSystem,
    renderer::{DisplayConfig, DrawShaded, DrawSkybox, DrawTriplanar, DrawDebugLines, PosColorNorm, PosNormTex, Pipeline, RenderBundle, Stage},
    utils::{application_root_dir, scene::BasicScenePrefab},
};

use std::ops::Deref;
use std::sync::{Arc, Mutex};

// Controller setup.
// I'm not really sure how you're supposed to do Controller input in amethyst, so this is probably a massive hack.
use gilrs::{Gilrs, Event};

#[derive(Default)]
struct PadEventReader;

struct Pad(Arc<Mutex<Gilrs>>);

impl<'a> SystemData<'a> for Pad {
    fn setup(res: &mut Resources) {
        let gilrs = Gilrs::new().unwrap();
        res.insert(Pad(Arc::new(Mutex::new(gilrs))));
    }

    fn fetch(res: &'a Resources) -> Self {
        let r = res.fetch::<Pad>();
        let Pad(gilrs) = r.deref();
        Pad(gilrs.clone())
    }

    fn reads() -> Vec<ResourceId> {
        Vec::new()
    }

    fn writes() -> Vec<ResourceId> {
        Vec::new()
    }
}

impl<'a> EventReader<'a> for PadEventReader {
    type SystemData = Pad;
    type Event = Event;

    fn read(&mut self, Pad(gilrs_mutex): Pad, vec: &mut Vec<Event>) {
        if let Ok(ref mut gilrs) = gilrs_mutex.try_lock() {
            while let Some(event) = gilrs.next_event() {
                vec.push(event)
            }
        } else {
            panic!("Failed to aquire controller lock")
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use hybrid::Hybrid;

    let app_root = application_root_dir();

    let config = DisplayConfig::load(format!("{}/resources/display_config.ron", app_root));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawTriplanar::<PosNormTex>::new())
            .with_pass(DrawSkybox::new())
            .with_pass(DrawDebugLines::<PosColorNorm>::new())
    );

    let assets_dir = format!("{}/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<BasicScenePrefab<Vec<PosNormTex>>>::default(), "", &[])
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_bundle(TransformBundle::new())?
        .with(systems::BallSystem::new(), "ball_system", &[])
        .with(systems::FollowSystem::new(), "follow_system", &[]);
    let mut game = CoreApplication::<_, gilrs::Event, PadEventReader>::new(assets_dir, Hybrid, game_data)?;
    game.run();

    Ok(())
}
