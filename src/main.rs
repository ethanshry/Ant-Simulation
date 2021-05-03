#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use std::vec;

use ggez::{
    conf::Conf,
    conf::NumSamples,
    conf::WindowMode,
    conf::WindowSetup,
    event,
    graphics::{Mesh, MeshBuilder},
    timer, Context, ContextBuilder, GameResult,
};
use rand::prelude::*;
use rand_distr::Normal;

mod ant;
mod coordinate;
mod navigable;
mod scent;

use ant::Ant;
use coordinate::Coordinate;
use navigable::Navigable;
use scent::{Scent, SCENT_LIFE};

const ANT_SPEED: f32 = 3.0;
const ANT_DETECTION_RANGE: f32 = 50.0;
const HOME_SIZE: f32 = 25.0;

const X_SIZE: f32 = 500.0;
const Y_SIZE: f32 = 500.0;

struct State {
    dt: std::time::Duration,
    home_position: Coordinate,
    food_positions: Vec<Coordinate>,
    ants: Vec<Ant>,
    home_scents: Vec<Scent>,
    food_scents: Vec<Scent>,
    home_food: u32,
}

impl State {
    pub fn new() -> State {
        let food_positions = vec![];

        State {
            dt: std::time::Duration::new(0, 0),
            home_position: Coordinate::new(X_SIZE / 2.0, Y_SIZE / 2.0),
            food_positions,
            ants: vec![],
            home_scents: vec![],
            food_scents: vec![],
            home_food: 6,
        }
    }
}

trait Bounded {
    fn enforce_x_bounds(&self) -> f32;
    fn enforce_y_bounds(&self) -> f32;
}

impl Bounded for f32 {
    fn enforce_x_bounds(&self) -> f32 {
        if *self > X_SIZE {
            return X_SIZE;
        } else if *self < 0.0 {
            return 0.0;
        } else {
            return *self;
        }
    }

    fn enforce_y_bounds(&self) -> f32 {
        if *self > Y_SIZE {
            return Y_SIZE;
        } else if *self < 0.0 {
            return 0.0;
        } else {
            return *self;
        }
    }
}

fn gen_food_cluster(size: u32, x: f32, y: f32) -> Vec<Coordinate> {
    let mut coords = vec![];

    let x = x.enforce_x_bounds();
    let y = y.enforce_y_bounds();

    let mut r = rand::thread_rng();

    for _ in 0..size {
        // get baseline variance
        // TODO make this circular and not squareish or smtn
        let x_var: f32 = r.gen::<f32>() * (size as f32) / 4.0;
        let y_var: f32 = r.gen::<f32>() * (size as f32) / 4.0;

        // differ by some amount
        let x_pos = x_var - ((size as f32) / 2.0) + x;
        let y_pos = y_var - ((size as f32) / 2.0) + y;
        coords.push(Coordinate::new(
            x_pos.enforce_x_bounds(),
            y_pos.enforce_y_bounds(),
        ));
    }

    return coords;
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 30) {
            if self.home_food > 5 {
                self.ants
                    .push(Ant::new(self.home_position.x, self.home_position.y));
                self.home_food -= 5;
            }
            self.home_scents = Vec::drain_filter(&mut self.home_scents, |s| {
                s.life -= 1;
                s.life > 0
            })
            .collect();
            self.food_scents = Vec::drain_filter(&mut self.food_scents, |s| {
                s.life -= 1;
                s.life > 0
            })
            .collect();
            for mut a in self.ants.iter_mut() {
                a.life -= 1;
                // if the ant is dead, turn its body into some food
                if a.life == 0 {
                    // TODO handle the fact that ant death should leave food scents
                    let mut new_food = gen_food_cluster(3, a.position.x, a.position.y);
                    self.food_positions.append(&mut new_food);
                }
                // else behave based on food
                if a.has_food {
                    let new_dir = match a.direction {
                        d if d > 180.0 => d - 180.0,
                        d => d + 180.0,
                    };

                    self.food_scents
                        .push(Scent::new(a.position.x, a.position.y, new_dir));

                    // walk
                    a.traverse(Some(&self.home_position), &self.home_scents);

                    // see if we have reached home
                    if a.position.dist(&self.home_position) < HOME_SIZE {
                        // we have
                        a.has_food = false;
                        self.home_food += 1;
                    }
                } else {
                    let new_dir = match a.direction {
                        d if d > 180.0 => d - 180.0,
                        d => d + 180.0,
                    };

                    self.home_scents
                        .push(Scent::new(a.position.x, a.position.y, new_dir));

                    // see if we should eat a food
                    let mut food_to_eat: Option<usize> = None;
                    for (i, f) in self.food_positions.iter().enumerate() {
                        if a.position.dist(f) < ANT_SPEED {
                            // ant can reach food in next "hop"
                            food_to_eat = match food_to_eat {
                                None => Some(i),
                                Some(prev_food) => {
                                    // SAFE: the match guarantees f is a valid coordinate
                                    if a.position.dist(self.food_positions.get(prev_food).unwrap())
                                        > a.position.dist(f)
                                    {
                                        Some(i)
                                    } else {
                                        Some(prev_food)
                                    }
                                }
                            };
                        }
                    }

                    if let Some(f) = food_to_eat {
                        a.position = self.food_positions.get(f).unwrap().to_owned();
                        a.direction = match a.direction {
                            d if d > 180.0 => d - 180.0,
                            d => d + 180.0,
                        };
                        self.food_positions.remove(f);
                        a.has_food = true;
                        continue;
                    }

                    // walk
                    a.traverse(Some(&self.food_positions), &self.food_scents);
                }
            }
            self.ants = Vec::drain_filter(&mut self.ants, |a| {
                a.life > 0 || a.position != a.position.check_bounds(0.0, X_SIZE, 0.0, Y_SIZE)
            })
            .collect();
        }
        self.dt = timer::delta(ctx);
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // println!("frame_time: {}", self.dt.as_millis());
        // println!("ants: {}", self.ants.len());
        // println!("home scents: {}", self.home_scents.len());
        // println!("food scents: {}", self.food_scents.len());
        // println!("home food: {}", self.home_food);
        let mut scene = &mut ggez::graphics::MeshBuilder::new();
        scene.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.home_position.clone(),
            HOME_SIZE as f32,
            1.0,
            ggez::graphics::Color::from_rgb(46, 19, 0),
        );

        for hs in self.home_scents.iter_mut() {
            let life = match hs.life {
                l if l > SCENT_LIFE => SCENT_LIFE,
                l => l,
            };
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                hs.position.clone(),
                5.0 * (life as f32 / SCENT_LIFE as f32),
                1.0,
                ggez::graphics::Color::from_rgb(0, 44, 190),
            );
        }

        for fs in self.food_scents.iter_mut() {
            let life = match fs.life {
                l if l > SCENT_LIFE => SCENT_LIFE,
                l => l,
            };
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                fs.position.clone(),
                5.0 * (life as f32 / SCENT_LIFE as f32),
                1.0,
                ggez::graphics::Color::from_rgb(190, 190, 0),
            );
        }

        for f in self.food_positions.iter_mut() {
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                f.clone(),
                5 as f32,
                1.0,
                ggez::graphics::Color::from_rgb(15, 200, 15),
            );
        }

        for a in self.ants.iter_mut() {
            scene = a.draw(scene);
            println!("direction: {}", a.direction);
        }
        let scene = scene.build(ctx).unwrap();
        ggez::graphics::clear(ctx, ggez::graphics::Color::from_rgb(0, 0, 0));
        let mut params = ggez::graphics::DrawParam::default();
        params.scale = ggez::mint::Vector2 {
            x: 2.0 as f32,
            y: 2.0 as f32,
        };
        ggez::graphics::draw(ctx, &scene, params).unwrap();
        ggez::graphics::present(ctx).unwrap();
        Ok(())
    }
}

pub fn main() {
    let mut state = State::new();

    state.home_scents.push(Scent {
        position: Coordinate {
            x: X_SIZE / 2.0,
            y: Y_SIZE / 2.0,
        },
        direction: 0.0,
        life: u32::MAX,
    });

    // gen food clusters
    for _ in 0..15 {
        let mut r = rand::thread_rng();

        // get baseline variance
        let x: f32 = r.gen::<f32>() * (X_SIZE as f32);
        let y: f32 = r.gen::<f32>() * (Y_SIZE as f32);

        let mut cluster = gen_food_cluster(150, x.enforce_x_bounds(), y.enforce_y_bounds());
        state.food_positions.append(&mut cluster);
    }

    let mut c = Conf::new();
    c.window_setup = WindowSetup {
        title: "Ant Simulator".to_owned(),
        samples: NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: true,
    };
    c.window_mode = WindowMode {
        width: X_SIZE * 2.0,
        height: Y_SIZE * 2.0,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: X_SIZE * 2.0,
        min_height: Y_SIZE * 2.0,
        max_width: X_SIZE * 2.0,
        max_height: Y_SIZE * 2.0,
        resizable: false,
    };
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("hello_ggez", "awesome_person")
        .conf(c)
        .build()
        .unwrap();
    event::run(ctx, event_loop, &mut state).unwrap();
}
