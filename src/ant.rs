#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use crate::navigable::Navigable;
use crate::scent::Scent;
use crate::{coordinate::Coordinate, Bounded};
use ggez::graphics::{GlBackendSpec, ImageGeneric, MeshBuilder};
use rand::prelude::*;

const ANT_SPEED: f32 = 1.0;
const SCENT_LIFE: u32 = 300;
const ANT_DETECTION_RANGE: f32 = 15.0;
const HOME_SIZE: f32 = 25.0;

const X_SIZE: f32 = 500.0;
const Y_SIZE: f32 = 500.0;

pub struct Ant<'a> {
    pub position: Coordinate,
    pub direction: f32, // angle 0 -> 359
    pub has_food: bool,
    pub speed: f32,
    pub life: u32,
    pub keyframe: usize,
    frames: &'a [ImageGeneric<GlBackendSpec>],
}

impl<'a> Ant<'a> {
    pub fn new(x: f32, y: f32, frames: &'a [ImageGeneric<GlBackendSpec>]) -> Ant {
        let mut r = rand::thread_rng();
        let dir: f32 = r.gen::<f32>();
        Ant {
            position: Coordinate::new(x, y),
            direction: dir * 359.9,
            has_food: false,
            speed: ANT_SPEED,
            life: 4000,
            keyframe: 0,
            frames: frames,
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
        if let Some(targets) = targets {
            if let Some(p) = targets.get_nearest(
                &self.position,
                ANT_DETECTION_RANGE,
                self.speed,
                self.direction,
            ) {
                self.direction = self.position.direction(&p);
                self.position = self.position.traverse_direction(self.direction, self.speed);
                return;
            }
        }

        let direction = waypoints.get_avg_direction(
            &self.position,
            ANT_DETECTION_RANGE,
            self.speed,
            self.direction,
        );
        self.direction = direction;
        let pos = self.position.clone();
        self.position = self.position.traverse_direction(self.direction, self.speed);
        self.direction = pos.direction(&self.position);
    }

    /*pub fn draw<'b>(&mut self, mesh: &'b mut MeshBuilder) -> &'b mut MeshBuilder {
        let output = mesh.circle(
            ggez::graphics::DrawMode::Stroke(ggez::graphics::StrokeOptions::DEFAULT),
            self.position.clone(),
            ANT_DETECTION_RANGE as f32,
            0.1,
            ggez::graphics::Color::from_rgb(255, 0, 0),
        );
        self.keyframe += 1;
        output.texture(
            self.frames
                .get(self.keyframe % self.frames.len())
                .unwrap()
                .clone(),
        )
        /*output.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.position.clone(),
            5 as f32,
            0.1,
            ggez::graphics::Color::from_rgb(220, 15, 0),
        )*/
    }*/
    pub fn draw_debug<'b>(&mut self, mesh: &'b mut MeshBuilder) -> &'b mut MeshBuilder {
        mesh.circle(
            ggez::graphics::DrawMode::Stroke(ggez::graphics::StrokeOptions::DEFAULT),
            self.position.clone(),
            ANT_DETECTION_RANGE as f32,
            0.1,
            ggez::graphics::Color::from_rgb(255, 0, 0),
        )
        /*self.keyframe += 1;
        output.texture(
            self.frames
                .get(self.keyframe % self.frames.len())
                .unwrap()
                .clone(),
        )*/
        /*output.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.position.clone(),
            5 as f32,
            0.1,
            ggez::graphics::Color::from_rgb(220, 15, 0),
        )*/
    }
    pub fn draw<'b>(&mut self, ctx: &mut ggez::Context) {
        let params = ggez::graphics::DrawParam::default()
            .offset(ggez::mint::Vector2 { x: 0.5, y: 0.5 })
            .rotation(self.direction)
            .scale(ggez::mint::Vector2 { y: 1.0, x: 1.0 })
            .dest(ggez::mint::Vector2 {
                y: self.position.y * 2.0,
                x: self.position.x * 2.0,
            })
            .color(ggez::graphics::Color::from_rgb(255, 0, 255));
        ggez::graphics::draw(
            ctx,
            &self
                .frames
                .get(self.keyframe % self.frames.len())
                .unwrap()
                .clone(),
            params,
        )
        .unwrap();
        self.keyframe += 1;
        /*
        let output = mesh.circle(
            ggez::graphics::DrawMode::Stroke(ggez::graphics::StrokeOptions::DEFAULT),
            self.position.clone(),
            ANT_DETECTION_RANGE as f32,
            0.1,
            ggez::graphics::Color::from_rgb(255, 0, 0),
        );
        self.keyframe += 1;
        output.texture(
            self.frames
                .get(self.keyframe % self.frames.len())
                .unwrap()
                .clone(),
        )*/
        /*output.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.position.clone(),
            5 as f32,
            0.1,
            ggez::graphics::Color::from_rgb(220, 15, 0),
        )*/
    }
}
