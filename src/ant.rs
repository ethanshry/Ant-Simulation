#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use crate::navigable::Navigable;
use crate::scent::Scent;
use crate::{coordinate::Coordinate, Bounded};
use ggez::graphics::MeshBuilder;
use rand::prelude::*;

const ANT_SPEED: f32 = 3.0;
const SCENT_LIFE: u32 = 300;
const ANT_DETECTION_RANGE: f32 = 25.0;
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

    /// Try to get closer to the next target
    ///
    /// # Arguments
    /// - `targets` things which we want to go to, should be prioritied over waypoints
    /// - `waypoints` things which direct us to targets
    pub fn traverse<T, U>(&mut self, targets: Option<&T>, waypoints: &U) -> ()
    where
        T: Navigable,
        U: Navigable,
    {
        match targets {
            Some(targets) => {
                let position = targets.get_nearest(
                    &self.position,
                    ANT_DETECTION_RANGE,
                    self.speed,
                    self.direction,
                );
                self.direction = self.position.direction(&position);
                self.position = self.position.traverse_direction(self.direction, self.speed);
            }
            None => {
                let direction = waypoints.get_avg_direction(
                    &self.position,
                    ANT_DETECTION_RANGE,
                    self.speed,
                    self.direction,
                );
                self.direction = direction;
                self.position = self.position.traverse_direction(self.direction, self.speed);
            }
        }
    }

    pub fn draw<'a>(&mut self, mesh: &'a mut MeshBuilder) -> &'a mut MeshBuilder {
        let output = mesh.circle(
            ggez::graphics::DrawMode::Stroke(ggez::graphics::StrokeOptions::DEFAULT),
            self.position.clone(),
            ANT_DETECTION_RANGE as f32,
            0.1,
            ggez::graphics::Color::from_rgb(255, 0, 0),
        );
        output.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.position.clone(),
            5 as f32,
            0.1,
            ggez::graphics::Color::from_rgb(220, 15, 0),
        )
    }
}
