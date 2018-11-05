extern crate nalgebra_glm as glm;

use amethyst::{
    renderer::{PosNormTex}
};

use rand::{thread_rng, Rng};

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
    rotation: glm::Quat
}

impl ControlPlane {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        ControlPlane {
            pos: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity()
        }
    }

    pub fn point(&self, dir : Dir8) -> glm::Vec3 {
        let (x, y) = match dir {
            North => (0.0, 0.3),
            NorthEast => (0.3, 0.3),
            East => (0.3, 0.0),
            SouthEast => (0.3, -0.3),
            South => (0.0, -0.3),
            SouthWest => (-0.3, -0.3),
            West => (-0.3, 0.0),
            NorthWest => (-0.3, 0.3)
        };

        glm::quat_cross_vec(&self.rotation, &glm::vec3(x, y, 0.0)) + self.pos
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
                position: [p.x, p.y, p.z],
                normal: [0.0, 0.0, 1.0],
                tex_coord: [0.0, 0.0]
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

    // Rasterize the patch into a res * res grid
    pub fn rasterize(&self, res: i32) -> Vec<PosNormTex> {
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
                        (p - p_u).cross(&(p - p_v)).normalize() * -1.0
                    } else {
                        (p - p_u).cross(&(p - p_v)).normalize()
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

const SIZE: usize = 4;

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
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-0.6, 0.6), &glm::vec3(0.0, 0.0, 1.0));
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-0.6, 0.6), &glm::vec3(0.0, 1.0, 0.0));
                let q = glm::quat_rotate_normalized_axis(&q, rng.gen_range(-0.6, 0.6), &glm::vec3(1.0, 0.0, 0.0));

                surface[i][j].rotation = q;
                surface[i][j].pos = glm::vec3(i as f32, j as f32, rng.gen_range(-0.5, 0.5))
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