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

pub struct Ball {
    pub velocity: [f32; 2]
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

struct BicubicPatch {
    controls: [[Vector3<f32>; 4]; 4]
}


fn fac(n: i32) -> i32 {
    assert!(n >= 0);

    let mut m = 1; 
    for i in 1..(n + 1) {
        m = m * i
    };
    m
}

fn binomial3(i: i32) -> f32 {
    6.0 / (fac(i) * fac(3 - i)) as f32
}

fn bernstein3(i: i32, u: f32) -> f32 {
    binomial3(i) * u.powi(i) * (1.0 - u).powi(3 - i)
}

impl BicubicPatch {
    pub fn new() -> Self {
        let v0  = Vector3::new(-0.3, -0.3, 0.0);
        let v1  = Vector3::new(0.0, 0.2, 0.1);
        let v2  = Vector3::new(0.0, 0.4, 0.1);
        let v3  = Vector3::new(0.0, 0.6, 0.0);
        let v4  = Vector3::new(0.2, 0.0, 0.0);
        let v5  = Vector3::new(0.2, 0.2, 0.0);
        let v6  = Vector3::new(0.2, 0.4, 0.0);
        let v7  = Vector3::new(0.2, 0.6, 0.0);
        let v8  = Vector3::new(0.4, 0.0, 0.0);
        let v9  = Vector3::new(0.4, -0.4, 0.0);
        let v10 = Vector3::new(0.4, 1.4, 0.9);
        let v11 = Vector3::new(0.4, 0.6, 0.0);
        let v12 = Vector3::new(0.6, 0.0, 0.0);
        let v13 = Vector3::new(0.6, 0.2, 0.0);
        let v14 = Vector3::new(0.6, 0.4, 0.0);
        let v15 = Vector3::new(0.6, 0.6, 0.0);

        BicubicPatch { controls: [[v0,  v1,  v2,  v3],
                                  [v4,  v5,  v6,  v7],
                                  [v8,  v9,  v10, v11],
                                  [v12, v13, v14, v15]] }
    }
    
    fn control(&self, i: i32, j: i32) -> Vector3<f32> {
        self.controls[i as usize][j as usize]
    }
    
    fn pos(&self, u: f32, v: f32) -> Vector3<f32> {
        // Only defined for the unit square
        assert!(0.0 <= u && u <= 1.0 && 0.0 <= v && v <= 1.0);
        
        let mut sum = Vector3::new(0.0, 0.0, 0.0);
        for i in 0..4 {
            for j in 0..4 {
                sum += (bernstein3(i, u) * bernstein3(j, v)) * self.control(i, j)
            }
        };
        sum
    }

    // Rasterize the patch into a res * res grid
    fn rasterize(&self, res: i32) -> Vec<PosNormTex> {
        let mut vec = Vec::new();
        for row in 0..res {
            for col in 0..res {
                // Generate two triangles for each square in the grid
                for (rt, ct) in [(0.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)].iter() {
                    let u = (row as f32 + rt) / res as f32;
                    let v = (col as f32 + ct) / res as f32;

                    let p = self.pos(u, v);

                    // Compute a normal using two nearby points
                    let delta_u = u - (0.5 / res as f32);
                    let delta_v = v - (0.5 / res as f32);
                    let p_u = self.pos(delta_u.abs(), v);
                    let p_v = self.pos(u, delta_v.abs());

                    // We need to flip some normals near the edge
                    let normal = if (delta_u < 0.0) && (delta_v >= 0.0) || (delta_u >= 0.0) && (delta_v < 0.0) {
                        (p - p_u).cross(p - p_v).normalize_to(-1.0)
                    } else {
                        (p - p_u).cross(p - p_v).normalize_to(1.0)
                    };

                    vec.push(PosNormTex {
                        position: p.into(),
                        normal: normal.into(),
                        tex_coord: [u, v]
                    })
                }
            }
        };
        vec
    }
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

        let patch = BicubicPatch::new();
        let mesh = create_mesh(world, patch.rasterize(32));

        let mtl = {
            let loader = world.read_resource::<Loader>();
            let mat_defaults = world.read_resource::<MaterialDefaults>();

            let albedo = loader.load_from_data([1.0, 0.0, 0.0, 0.0].into(), (), &world.read_resource());

            Material {
                albedo,
                ..mat_defaults.0.clone()
            }
        };

        let mut trans = Transform::default();
        trans.scale = Vector3::new(20.0, 20.0, 20.0);

        world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .build();

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
