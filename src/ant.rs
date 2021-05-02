#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use crate::coordinate::Coordinate;
use crate::scent::{Navigable, Scent};
use ggez::graphics::MeshBuilder;
use rand::prelude::*;

const ANT_SPEED: f32 = 15.0;
const SCENT_LIFE: u32 = 300;
const ANT_DETECTION_RANGE: f32 = 50.0;
const HOME_SIZE: f32 = 25.0;

const X_SIZE: f32 = 500.0;
const Y_SIZE: f32 = 500.0;

pub struct Ant {
    pub position: Coordinate,
    pub direction: f32, // angle 0 -> 359
    pub has_food: bool,
    pub speed: f32,
    pub life: u32,
}

impl Ant {
    pub fn new(x: f32, y: f32) -> Ant {
        let mut r = rand::thread_rng();
        let dir: f32 = r.gen::<f32>();
        Ant {
            position: Coordinate::new(x, y),
            direction: dir * 359.9,
            has_food: false,
            speed: ANT_SPEED,
            life: 4000,
        }
    }

    pub fn traverse<T>(&mut self, waypoints: &T) -> ()
    where
        T: Navigable,
    {
        let (position, direction) = waypoints.get_next_point(
            &self.position,
            ANT_DETECTION_RANGE,
            self.speed,
            self.direction,
        );
        self.position = position.to_owned();
        self.direction = direction;
    }

    pub fn draw<'a>(&mut self, mesh: &'a mut MeshBuilder) -> &'a mut MeshBuilder {
        mesh.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.position.clone(),
            5 as f32,
            1.0,
            ggez::graphics::Color::from_rgb(220, 15, 0),
        )
    }
}
