#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use crate::coordinate::Coordinate;
use crate::navigable::Navigable;
use rand::prelude::*;
use rand_distr::Normal;

pub const SCENT_LIFE: u32 = 300;

pub struct Scent {
    pub position: Coordinate,
    pub direction: f32,
    pub life: u32,
}

impl Scent {
    pub fn new(x: f32, y: f32, direction: f32) -> Scent {
        Scent {
            position: Coordinate::new(x, y),
            direction,
            life: SCENT_LIFE / 2,
        }
    }
}

impl Navigable for Vec<Scent> {
    fn get_nearest(&self, pos: &Coordinate, range: f32, dist: f32, dir: f32) -> Coordinate {
        let mut final_pos = None;
        let mut final_dir = None;
        for s in self {
            let dist = pos.dist(&s.position);
            if dist < range {
                match final_pos {
                    Some(p) => {
                        if s.position.dist(pos) < s.position.dist(p) {
                            final_pos = Some(&s.position);
                            final_dir = Some(s.direction);
                        }
                    }
                    None => {
                        final_pos = Some(&s.position);
                        final_dir = Some(s.direction);
                    }
                }
            }
        }
        if let None = final_pos {
            // we were unable to find a position, so we need to make one up
            let distribution = Normal::new(dir, 1.0).unwrap();
            let mut direction = distribution.sample(&mut rand::thread_rng());
            // match to a valid direction
            while direction > 359.9 {
                direction -= 359.9;
            }
            while direction < 0.0 {
                direction += 359.9
            }
            final_dir = Some(direction);
        }
        // now go from a direction and a coordinate to a new coordinate
        pos.traverse_direction(final_dir.unwrap(), dist)
    }

    fn get_avg_direction(&self, pos: &Coordinate, range: f32, dist: f32, dir: f32) -> f32 {
        //let mut final_pos = None;
        let mut final_dir = None;
        //let mut last_strength = None;
        let mut scent_count = 0;
        for s in self {
            let dist = pos.dist(&s.position);
            if dist < range {
                match final_dir {
                    Some(d) => {
                        final_dir = Some(d + s.direction);
                    }
                    None => {
                        final_dir = Some(s.direction);
                    }
                }
                scent_count = scent_count + 1;
            }
        }
        if let None = final_dir {
            // we were unable to find a position, so we need to make one up
            let distribution = Normal::new(dir, 1.0).unwrap();
            let mut direction = distribution.sample(&mut rand::thread_rng());
            // match to a valid direction
            while direction > 359.9 {
                direction -= 359.9;
            }
            while direction < 0.0 {
                direction += 359.9
            }
            final_dir = Some(direction);
            scent_count = 1;
        }
        let direction = final_dir.unwrap() / (scent_count as f32);
        // now go from a direction and a coordinate to a new coordinate

        final_dir.unwrap().clone()
    }
}
