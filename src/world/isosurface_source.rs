// Copyright 2021 Tristam MacDonald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Isosurface definitions for use in multiple examples
use isosurface::{
    distance::Signed,
    math::Vec3,
    source::{HermiteSource, ScalarSource},
};

pub const CHUNK_LENGTH: f32 = 32.;

pub struct IsosurfaceSource {
    pub chunkx: i32,
    pub chunkz: i32,
    pub epsilon: f32,
}

impl IsosurfaceSource {
    pub fn new(chunkx: i32, chunkz: i32) -> Self {
        Self {
            chunkx, chunkz,
            epsilon: 0.000001,
        }
    }

        
    pub fn heightfn(&self, x: f32, z: f32) -> f32 {
        let x = (self.chunkx as f32) + x;
        let z = (self.chunkz as f32) + z;
        0.5 + 0.1 * (0.1 + x * 1.11).sin() + 
            0.5 * ((x * 0.0911).sin() + (z * 0.0811).sin()) * 
                (0.1 * (x * 10.011).sin() * (x * 1.0311).sin() + 
                0.1 * (x * 9.3011).sin() * (x * 2.4311).sin() + 
                0.1 * (z * 17.3011).sin() * (z * 1.9311).sin()) + 
            0.1 * (0.1 + z * 1.31).sin() + 
            0.1 * (z * 1.0311).sin()
    }
}


impl ScalarSource for IsosurfaceSource {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        // Must return the signed distance (i.e. negative for coordinates inside
        // the surface), as our Marching Cubes implementation will evaluate the
        // surface at the zero-crossing.
        // 
        // self.source.sample_scalar(q)
        Signed(self.heightfn(p.x, p.z) - p.y)//TODO
    }
}

// impl<S: VectorSource + ScalarSource> VectorSource for IsosurfaceSource<S> {
//     fn sample_vector(&self, p: Vec3) -> Directed {
//         self.source.sample_vector(p)
//     }
// }

impl HermiteSource for IsosurfaceSource {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        let dx = Vec3::new(self.epsilon, 0.0, 0.0);
        let vx = self.sample_scalar(p + dx).0 - self.sample_scalar(p - dx).0;

        let dy = Vec3::new(0.0, self.epsilon, 0.0);
        let vy = self.sample_scalar(p + dy).0 - self.sample_scalar(p - dy).0;

        let dz = Vec3::new(0.0, 0.0, self.epsilon);
        let vz = self.sample_scalar(p + dz).0 - self.sample_scalar(p - dz).0;

        Vec3::new(vx, vy, vz) / (2.0 * self.epsilon)
    }
}
