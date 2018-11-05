use amethyst::{
    prelude::*,
    ecs::prelude::*,
    core::Transform,
    core::cgmath::prelude::InnerSpace,
    core::cgmath::{Vector3, Deg},
    assets::{Loader},
    renderer::{MeshHandle, Rgba, Projection, PosNormTex, Camera, Material, MaterialDefaults, ObjFormat, Light, PointLight},
};

use gilrs::Event;

use rand::{thread_rng, Rng};

mod proc_geom;

pub struct Ball {
    pub velocity: [f32; 2]
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

fn create_mesh(world: &World, vertices: Vec<PosNormTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), (), &world.read_resource())
}

pub struct Hybrid;

impl<'a, 'b> State<GameData<'a, 'b>, Event> for Hybrid {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        initialize_camera(world);
        initialize_lights(world);

        let (mesh, mtl) = {
            let meshes = &world.read_resource();
            let textures = &world.read_resource();

            let loader = world.read_resource::<Loader>();
            let mat_defaults = world.read_resource::<MaterialDefaults>();

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
            Vec::<Event>::new(),
        );

        world
            .create_entity()
            .with(mesh)
            .with(mtl.clone())
            .with(trans)
            .with(Ball {
                velocity: [0.0, 0.0]
            })
            .build();

        // Control Surface
        let cs = proc_geom::ControlSurface::new();

        // let mesh = create_mesh(world, cs.rasterize());

        // let mtl = {
        //     let loader = world.read_resource::<Loader>();
        //     let mat_defaults = world.read_resource::<MaterialDefaults>();

        //     let albedo = loader.load_from_data([1.0, 1.0, 0.0, 0.0].into(), (), &world.read_resource());

        //     Material {
        //         albedo,
        //         ..mat_defaults.0.clone()
        //     }
        // };

        // let mut trans = Transform::default();
        // trans.scale = Vector3::new(8.0, 8.0, 8.0);
        // trans.translation.x = -10.0;

        // world
        //     .create_entity()
        //     .with(mesh)
        //     .with(mtl)
        //     .with(trans)
        //     .build();

        // Create grid of bicubic patches
        for i in 0..3 {
            for j in 0..3 {
                let patch = proc_geom::BicubicPatch::new(
                    &cs.controls[i][j],
                    &cs.controls[i][j + 1],
                    &cs.controls[i + 1][j + 1],
                    &cs.controls[i + 1][j]
                );
                let mesh = create_mesh(world, patch.rasterize(32));

                let mtl = {
                    let loader = world.read_resource::<Loader>();
                    let mat_defaults = world.read_resource::<MaterialDefaults>();

                    let mut rng = thread_rng();
                    let albedo = loader.load_from_data([rng.gen_range(0.7, 1.0), rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), 0.0].into(), (), &world.read_resource());

                    Material {
                        albedo,
                        ..mat_defaults.0.clone()
                    }
                };

                let mut trans = Transform::default();
                trans.scale = Vector3::new(8.0, 8.0, 8.0);
                trans.translation.x = -10.0;

                world
                    .create_entity()
                    .with(mesh)
                    .with(mtl)
                    .with(trans)
                    .build();
            }
        }
    }

    fn handle_event(
        &mut self,
        data: StateData<GameData>,
        event: Event
    ) -> Trans<GameData<'a, 'b>, Event> {
        let mut events = data.world.write_resource::<Vec<Event>>();
        events.push(event);

        println!("{:?}", event);
        Trans::None
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>, Event> {
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
