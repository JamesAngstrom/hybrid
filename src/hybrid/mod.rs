use amethyst::{
    prelude::*,
    ecs::prelude::*,
    core::Transform,
    core::cgmath::{Vector3, Deg, InnerSpace},
    assets::{Loader, AssetStorage},
    renderer::{Shape, MeshHandle, DebugLinesComponent, JpgFormat, Texture, TextureHandle, TriplanarMaterial, Rgba, Projection, SkyboxColor,
               PosNormTex, PosNormTangTex, Camera, AmbientColor, Material, MaterialDefaults, TextureMetadata, ObjFormat, Light, DirectionalLight, PointLight},
};
use gilrs::Event;
use nalgebra::geometry::{Isometry3};
use ncollide3d::{
    shape::TriMesh,
    bounding_volume::{AABB, HasBoundingVolume, BoundingVolume}
};

use rand::{thread_rng, Rng};

mod proc_geom;

pub struct Follow {
    pub entity: Entity
}

impl Component for Follow {
    type Storage = DenseVecStorage<Self>;
}

pub struct Ball {
    pub velocity: [f32; 2]
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

// The bezier patches that make up the terrain are marked with the Chunk component
pub struct Chunk {
    pub collision_mesh: TriMesh<f32>,
    pub patch: proc_geom::BicubicPatch,
    pub bounding_box: AABB<f32>
}

impl Component for Chunk {
    type Storage = VecStorage<Self>;
}

fn create_mesh(world: &World, vertices: Vec<PosNormTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), (), &world.read_resource())
}

pub fn load_texture<N>(name: N, world: &World) -> TextureHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        JpgFormat,
        TextureMetadata::srgb(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

pub struct Hybrid;

impl<'a, 'b> State<GameData<'a, 'b>, Event> for Hybrid {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        world.register::<Chunk>();
        world.register::<Follow>();

        initialize_lights(world);

        let (mesh, mtl) = {
            let meshes = &world.read_resource();
            let textures = &world.read_resource();

            let loader = world.read_resource::<Loader>();
            let mat_defaults = world.read_resource::<MaterialDefaults>();
            
            let mesh = loader.load("mesh/teapot.obj", ObjFormat, (), (), meshes);
            let albedo = loader.load_from_data([1.0, 0.0, 1.0, 0.0].into(), (), textures);

            let mat = Material {
                albedo,
                ..mat_defaults.0.clone()
            };

            (mesh, mat)
        };

        let mut trans = Transform::default();
        trans.scale = Vector3::new(0.3, 0.3, 0.3);
        trans.translation = Vector3::new(5.0, 30.0, 5.0);

        world.add_resource(
            Vec::<Event>::new(),
        );

        let player = world
            .create_entity()
            .with(mesh)
            .with(mtl.clone())
            .with(trans)
            .with(DebugLinesComponent::new())
            .with(Ball {
                velocity: [0.0, 0.0]
            })
            .build();

        initialize_camera(world, player);

        // Control Surface
        let cs = proc_geom::ControlSurface::new();

        // let mesh = create_mesh(world, cs.rasterize());

        // let mtl = {
        //     let loader = world.read_resource::<Loader>();
        //     let mat_defaults = world.read_resource::<MaterialDefaults>();

        //     let albedo = loader.load_from_data([1.0, 1.0, 0.0, 0.5].into(), (), &world.read_resource());

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
        for i in 0..5 {
            for j in 0..5 {
                println!("i: {} j: {}", i, j);

                let patch = proc_geom::BicubicPatch::new(
                    &cs.controls[i][j],
                    &cs.controls[i][j + 1],
                    &cs.controls[i + 1][j + 1],
                    &cs.controls[i + 1][j]
                );
                let mesh = create_mesh(world, patch.rasterize(2));
                let mut collision_mesh = patch.collision_mesh(16, 8.0);

                let mtl_xy = {
                    let loader = world.read_resource::<Loader>();
                    let mat_defaults = world.read_resource::<MaterialDefaults>();

                    let mut rng = thread_rng();
                    let albedo = load_texture("texture/Rock08_col.jpg", world);

                    Material {
                        albedo,
                        ..mat_defaults.0.clone()
                    }
                };
                let mtl_yz = {
                    let loader = world.read_resource::<Loader>();
                    let mat_defaults = world.read_resource::<MaterialDefaults>();

                    let mut rng = thread_rng();
                    let albedo = load_texture("texture/Rock07_col.jpg", world);
                    let emission = load_texture("texture/noise.jpg", world);

                    Material {
                        albedo,
                        emission,
                        ..mat_defaults.0.clone()
                    }
                };
                let mtl_xz = {
                    let loader = world.read_resource::<Loader>();
                    let mat_defaults = world.read_resource::<MaterialDefaults>();

                    let mut rng = thread_rng();
                    let albedo = load_texture("texture/Ice04_col.jpg", world);
                    let emission = load_texture("texture/Snow01_col.jpg", world);

                    //let albedo = loader.load_from_data([rng.gen_range(0.4, 0.6), rng.gen_range(0.0, 0.4), rng.gen_range(0.0, 0.4), 0.0].into(), (), &world.read_resource());

                    Material {
                        albedo,
                        emission,
                        ..mat_defaults.0.clone()
                    }
                };

                let mut trans = Transform::default();
                trans.scale = Vector3::new(8.0, 8.0, 8.0);
                trans.translation.x = 0.0;
                let mut bounding_box: AABB<f32> = collision_mesh.clone().bounding_volume(&Isometry3::identity());
                bounding_box.loosen(3.0);

                world
                    .create_entity()
                    .with(mesh)
                    .with(TriplanarMaterial {
                        mtl_xy: mtl_xy,
                        mtl_yz: mtl_yz,
                        mtl_xz: mtl_xz
                    })
                    .with(trans)
                    .with(Chunk {
                        collision_mesh: collision_mesh,
                        patch: patch,
                        bounding_box: bounding_box
                    })
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

fn initialize_camera(world: &mut World, target: Entity) {
    let mut transform = Transform::default();
    transform.set_position(Vector3::new(0.0, 10.0, 100.0));
    //transform.set_rotation(Deg(90.0), Deg(0.0), Deg(0.0));

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.0, Deg(60.0))))
        .with(transform)
        .with(Follow { entity: target })
        .build();
}


fn initialize_lights(world: &mut World) {
    world.add_resource(AmbientColor(Rgba(0.15, 0.18, 0.35, 1.0)));
    {
        let mut skybox = world.write_resource::<SkyboxColor>();
        skybox.zenith = Rgba(0.04, 0.05, 0.37, 1.0);
        skybox.nadir = Rgba::black(); 
    }

    let dir = Vector3::new(0.7, -1.0, 0.8).normalize();

    let light: Light = DirectionalLight {
        color: Rgba(0.4, 0.4, 0.5, 1.0),
        direction: [dir.x, dir.y, dir.z]
    }.into();

    let mut transform = Transform::default();
    transform.set_position(Vector3::new(5.0, 20.0, 15.0));

    world.create_entity().with(light).with(transform).build();

}
