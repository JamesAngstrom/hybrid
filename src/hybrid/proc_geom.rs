
use amethyst::{
    renderer::{PosNormTex},
    core::nalgebra::{Vector2, Vector3}
};

use rand::{thread_rng, Rng};

use glm;
use nalgebra::geometry::{Point2, Point3};
use ncollide3d::shape::{TriMesh};

#[derive(Clone, Copy)]
pub enum Dir8 {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}
use self::Dir8::*;

#[derive(Clone, Copy)]
pub struct ControlPlane {
    pos: glm::Vec3,
    rotation: glm::Quat,
    north: f32,
    east: f32,
    south: f32,
    west: f32
}

impl ControlPlane {
    pub fn new() -> Self {
        ControlPlane {
            pos: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            // All 0.3 gives sensible results
            north: 1.0,
            east: 0.4,
            south: -0.3,
            west: -0.4,
        }
    }

    pub fn point(&self, dir : Dir8) -> glm::Vec3 {
        let (x, y) = match dir {
            North => (0.0, self.north),
            NorthEast => (self.east, self.north),
            East => (self.east, 0.0),
            SouthEast => (self.east, self.south),
            South => (0.0, self.south),
            SouthWest => (self.west, self.south),
            West => (self.west, 0.0),
            NorthWest => (self.west, self.north)
        };

        glm::quat_cross_vec(&self.rotation, &glm::vec3(x, 0.0, y)) + self.pos
    }

    pub fn center(&self) -> glm::Vec3 {
        glm::quat_cross_vec(&self.rotation, &glm::vec3(0.0, 0.0, 0.0)) + self.pos
    }

    pub fn rasterize(&self) -> Vec<PosNormTex> {
        let mut vec = Vec::new();

        let a = self.point(NorthEast);
        let b = self.point(SouthEast);
        let c = self.point(SouthWest);
        let d = self.point(NorthWest);

        for p in [c, b, a, a, d, c].iter() {
            vec.push(PosNormTex {
                position: *p,
                normal: Vector3::new(0.0, 1.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            })
        };
        vec
    }
}

pub struct BicubicPatch {
    controls: [[glm::Vec3; 4]; 4]
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
    /// Create a bicubic patch from four control planes, one for each corner.
    pub fn new(sw: &ControlPlane, nw: &ControlPlane, ne: &ControlPlane, se: &ControlPlane) -> Self {
        let v0  = sw.center();
        let v1  = sw.point(North);
        let v2  = nw.point(South);
        let v3  = nw.center();
        let v4  = sw.point(East);
        let v5  = sw.point(NorthEast);
        let v6  = nw.point(SouthEast);
        let v7  = nw.point(East);
        let v8  = se.point(West);
        let v9  = se.point(NorthWest);
        let v10 = ne.point(SouthWest);
        let v11 = ne.point(West);
        let v12 = se.center();
        let v13 = se.point(North);
        let v14 = ne.point(South);
        let v15 = ne.center();

        // To clarify the directions of the points in this matrix:
        //            South <---+---> North
        //                      |
        //                      V East
        BicubicPatch { controls: [[v0,  v1,  v2,  v3],
                                  [v4,  v5,  v6,  v7],
                                  [v8,  v9,  v10, v11],
                                  [v12, v13, v14, v15]] }

    }

    fn control(&self, i: i32, j: i32) -> glm::Vec3 {
        self.controls[i as usize][j as usize]
    }
    
    fn pos(&self, u: f32, v: f32) -> glm::Vec3 {
        // Only defined for the unit square
        assert!(0.0 <= u && u <= 1.0 && 0.0 <= v && v <= 1.0);
        
        let mut sum = glm::vec3(0.0, 0.0, 0.0);
        for i in 0..4 {
            for j in 0..4 {
                sum += (bernstein3(i, u) * bernstein3(j, v)) * self.control(i, j)
            }
        };
        sum
    }

    pub fn normal(&self, res: i32, u: f32, v: f32) -> glm::Vec3 {
        let p = self.pos(u, v);

        // Compute a normal using two nearby points
        let delta_u = u - (0.5 / res as f32);
        let delta_v = v - (0.5 / res as f32);
        let p_u = self.pos(if delta_u < 0.0 { u + (0.5 / res as f32) } else { delta_u }, v);
        let p_v = self.pos(u, if delta_v < 0.0 { v + (0.5 / res as f32) } else { delta_v });

        // We need to flip some normals near the edge
        if (delta_u < 0.0) && (delta_v >= 0.0) || (delta_u >= 0.0) && (delta_v < 0.0) {
            (p - p_u).cross(&(p - p_v)).normalize()
        } else {
            (p - p_u).cross(&(p - p_v)).normalize() * -1.0
        }
    }

    // Rasterize the patch into a res * res grid
    pub fn rasterize_with<F, A>(&self, res: i32, f : F) -> Vec<A>
    where F: Fn(glm::Vec3, glm::Vec3, f32, f32) -> A {
        let mut vec = Vec::new();
        for row in 0..res {
            for col in 0..res {
                // Generate two triangles for each square in the grid
                for (rt, ct) in [(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (0.0, 0.0), (1.0, 1.0), (1.0, 0.0)].iter() {
                    let u = (row as f32 + rt) / res as f32;
                    let v = (col as f32 + ct) / res as f32;

                    let p = self.pos(u, v);
                    let normal = self.normal(32, u, v);

                    vec.push(f(p, normal, u, v))
                }
            }
        };
        vec
    }

    pub fn rasterize(&self, res: i32) -> Vec<PosNormTex> {
        self.rasterize_with(res, |p, n, u, v| PosNormTex { position: p, normal: n, tex_coord: Vector2::new(u, v) })
    }

    // TODO: pass a more general translation + scale here
    pub fn collision_mesh(&self, res: i32, scale: f32) -> TriMesh<f32> {
        let mut points = Vec::new();
        let mut i: usize = 0;
        let mut indices = Vec::new();
        let mut uvs = Vec::new();

        for (p, u, v) in self.rasterize_with(res, |p, _, u, v| (p, u, v)) {
            points.push(Point3::new(scale * p.x, scale * p.y, scale * p.z));
            if i % 3 == 0 {
                indices.push(Point3::new(i, i + 1, i + 2));
            };
            i = i + 1;
            uvs.push(Point2::new(u, v))
        };

        TriMesh::new(points, indices, Some(uvs))
    }
}

const SIZE: usize = 64;

pub struct ControlSurface {
    pub controls: [[ControlPlane; SIZE]; SIZE]
}

impl ControlSurface {
    pub fn new() -> Self {
        let mut surface = [[ControlPlane::new(); SIZE]; SIZE];
        for i in 0..SIZE {
            for j in 0..SIZE {
                let mut rng = thread_rng();
                let q = surface[i][j].rotation;
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-0.7, 0.7), &glm::vec3(0.0, 0.0, 1.0));
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-1.2, 1.2), &glm::vec3(0.0, 1.0, 0.0));
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-0.7, 0.7), &glm::vec3(1.0, 0.0, 0.0));

                surface[i][j].rotation = q;
                surface[i][j].pos = glm::vec3(i as f32, rng.gen_range(-0.8, 0.8), j as f32);

                //if i == 3 && j == 3 {
                //    surface[i][j].pos.y = -0.7;
                //    surface[i][j].south = 0.8;
                //    surface[i][j].west = -0.9;
                //};
            }
        };
        ControlSurface { controls: surface }
    }

    pub fn rasterize(&self) -> Vec<PosNormTex> {
        let mut vec = Vec::new();

        for i in 0..SIZE {
            for j in 0..SIZE {
                let mut control = self.controls[i][j].rasterize();
                vec.append(&mut control);
            }
        };
        vec
    }
}