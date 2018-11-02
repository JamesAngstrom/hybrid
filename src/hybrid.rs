use amethyst::{
    prelude::*,
    ecs::prelude::*,
    core::Transform,
    core::cgmath::{Vector3, Deg},
    shrev::EventChannel,
    assets::{Loader},
    renderer::{Rgba, Projection, Camera, PosNormTex, Material, MaterialDefaults, ObjFormat, Light, PointLight},
};

use gilrs::Event;

pub struct Ball {
    pub velocity: [f32; 2]
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct Score {
    pub score: i32,
}

pub struct Hybrid;

impl<'a, 'b> State<GameData<'a, 'b>, Event> for Hybrid {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        initialize_camera(world);
        initialize_lights(world);

        let (mesh, mtl) = {
            let mat_defaults = world.read_resource::<MaterialDefaults>();
            let loader = world.read_resource::<Loader>();

            let meshes = &world.read_resource();
            let textures = &world.read_resource();

            let mesh = loader.load("mesh/teapot.obj", ObjFormat, (), (), meshes);
            let albedo = loader.load_from_data([0.0, 0.0, 1.0, 0.0].into(), (), textures);

            let mat = Material {
                albedo,
                ..mat_defaults.0.clone()
            };

            (mesh, mat)
        };

        let mut trans = Transform::default();
        trans.translation = Vector3::new(-5.0, 0.0, 0.0);

        world.add_resource(
            Score { score: 0 },
        );

        world.add_resource(
            Vec::<Event>::new(),
        );

        world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Ball {
                velocity: [0.0, 0.0]
            })
            .build();

    }

    fn handle_event(
        &mut self,
        data: StateData<GameData>,
        event: Event
    ) -> Trans<GameData<'a, 'b>, Event> {
        let mut score = data.world.write_resource::<Score>();
        let mut events = data.world.write_resource::<Vec<Event>>();
        events.push(event);

        println!("{:?}", event);
        Trans::None
    }

    fn update(&mut self, mut data: StateData<GameData>) -> Trans<GameData<'a, 'b>, Event> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_position(Vector3::new(0.0, -20.0, 10.0));
    transform.set_rotation(Deg(75.96), Deg(0.0), Deg(0.0));

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.0, Deg(60.0))))
        .with(transform)
        .build();
}


fn initialize_lights(world: &mut World) {
    let light: Light = PointLight {
        intensity: 100.0,
        radius: 1.0,
        color: Rgba::white(),
        ..Default::default()
    }.into();

    let mut transform = Transform::default();
    transform.set_position(Vector3::new(5.0, -20.0, 15.0));

    // Add point light.
    world.create_entity().with(light).with(transform).build();
}
