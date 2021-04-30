#![feature(drain_filter)]
#![feature(destructuring_assignment)]

use ggez::{
    conf::Conf, conf::NumSamples, conf::WindowMode, conf::WindowSetup, event, nalgebra::Point2,
    timer, Context, ContextBuilder, GameResult,
};
use rand::prelude::*;
use rand_distr::Normal;

const ANT_SPEED: f64 = 10.0;
const SCENT_LIFE: u32 = 1000;
const ANT_DETECTION_RANGE: f64 = 15.0;
const HOME_SIZE: f64 = 50.0;

const X_SIZE: u32 = 1000;
const Y_SIZE: u32 = 1000;

struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Coordinate {
        Coordinate {
            x: match x {
                x if x > X_SIZE as f64 => X_SIZE as f64,
                _ if x < 0.0 => 0.0,
                _ => x,
            },
            y: match y {
                y if y > Y_SIZE as f64 => Y_SIZE as f64,
                _ if y < 0.0 => 0.0,
                _ => y,
            },
        }
    }

    pub fn dist(&self, coor: &Coordinate) -> f64 {
        let dist_x = (self.x - coor.x) * (self.x - coor.x);
        let dist_y = (self.y - coor.y) * (self.y - coor.y);
        return (dist_x + dist_y).sqrt();
    }

    pub fn dir(&self, coor: &Coordinate) -> f64 {
        let mut angle = ((self.y - coor.y) / (self.x - coor.x)).atan().to_degrees();
        if angle > 359.9 {
            angle = 0.0;
        }

        return angle;
    }

    pub fn traverse_vec(&self, dir: f64, dist: f64) -> Coordinate {
        let y = dist * dir.sin();
        let x = dist * dir.cos();
        Coordinate::new(self.x + x, self.y + y)
    }
}

impl Into<ggez::mint::Point2<f32>> for Coordinate {
    fn into(self) -> ggez::mint::Point2<f32> {
        ggez::mint::Point2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Clone for Coordinate {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

struct Ant {
    position: Coordinate,
    direction: f64, // angle 0 -> 359
    has_food: bool,
    speed: f64,
    life: u32,
}

impl Ant {
    pub fn new(x: f64, y: f64) -> Ant {
        let mut r = rand::thread_rng();
        let dir: f64 = r.gen::<f64>();
        Ant {
            position: Coordinate::new(x, y),
            direction: dir * 359.9,
            has_food: false,
            speed: ANT_SPEED,
            life: 1000,
        }
    }
}

struct HomeScent {
    position: Coordinate,
    direction: f64, // angle 0 -> 359
    life: u32,
}

impl HomeScent {
    pub fn new(x: f64, y: f64, direction: f64) -> HomeScent {
        HomeScent {
            position: Coordinate::new(x, y),
            direction,
            life: SCENT_LIFE,
        }
    }
}

struct FoodScent {
    position: Coordinate,
    direction: f64, // angle 0 -> 359
    life: u32,
}

impl FoodScent {
    pub fn new(x: f64, y: f64, direction: f64) -> FoodScent {
        FoodScent {
            position: Coordinate::new(x, y),
            direction,
            life: SCENT_LIFE,
        }
    }
}

struct State {
    dt: std::time::Duration,
    home_position: Coordinate,
    food_positions: Vec<Coordinate>,
    ants: Vec<Ant>,
    home_scents: Vec<HomeScent>,
    food_scents: Vec<FoodScent>,
    home_food: u32,
}

trait Navigable {
    fn get_next_point(
        &self,
        pos: &Coordinate,
        range: f64,
        dist: f64,
        dir: f64,
    ) -> (Coordinate, f64);
}

impl Navigable for Vec<FoodScent> {
    fn get_next_point(
        &self,
        pos: &Coordinate,
        range: f64,
        dist: f64,
        dir: f64,
    ) -> (Coordinate, f64) {
        let mut final_pos = None;
        let mut final_dir = None;
        for coor in self {
            let dist = pos.dist(&coor.position);
            if dist < range {
                (final_pos, final_dir) = match final_pos {
                    Some(p) => match pos.dist(p) > dist {
                        true => (Some(&coor.position), Some(&coor.direction)),
                        false => (final_pos, final_dir),
                    },
                    None => (Some(&coor.position), Some(&coor.direction)),
                }
            }
        }
        let direction: f64 = match final_pos {
            Some(p) => {
                // get direction from the coordinate we are given
                pos.dir(p)
            }
            None => {
                // we were unable to find a position, so we need to make one up
                let distribution = Normal::new(dir, 20.0).unwrap();
                let mut direction = distribution.sample(&mut rand::thread_rng());
                // match to a valid direction
                while direction > 359.9 {
                    direction -= 359.9;
                }
                while direction < 0.0 {
                    direction += 359.9
                }
                direction
            }
        };
        if let None = final_dir {
            final_dir = Some(&direction);
        }
        // now go from a direction and a coordinate to a new coordinate
        (
            pos.traverse_vec(direction, dist),
            final_dir.unwrap().clone(),
        )
    }
}
// TODO there had to be a beter way to do this than to impl the same exact code twice
impl Navigable for Vec<HomeScent> {
    fn get_next_point(
        &self,
        pos: &Coordinate,
        range: f64,
        dist: f64,
        dir: f64,
    ) -> (Coordinate, f64) {
        let mut final_pos = None;
        let mut final_dir = None;
        for coor in self {
            let dist = pos.dist(&coor.position);
            if dist < range {
                (final_pos, final_dir) = match final_pos {
                    Some(p) => match pos.dist(p) > dist {
                        true => (Some(&coor.position), Some(&coor.direction)),
                        false => (final_pos, final_dir),
                    },
                    None => (Some(&coor.position), Some(&coor.direction)),
                }
            }
        }
        let direction: f64 = match final_pos {
            Some(p) => {
                // get direction from the coordinate we are given
                pos.dir(p)
            }
            None => {
                // we were unable to find a position, so we need to make one up
                let distribution = Normal::new(dir, 20.0).unwrap();
                let mut direction = distribution.sample(&mut rand::thread_rng());
                // match to a valid direction
                while direction > 359.9 {
                    direction -= 359.9;
                }
                while direction < 0.0 {
                    direction += 359.9
                }
                direction
            }
        };
        if let None = final_dir {
            final_dir = Some(&direction);
        }
        // now go from a direction and a coordinate to a new coordinate
        (
            pos.traverse_vec(direction, dist),
            final_dir.unwrap().clone(),
        )
    }
}

impl State {
    pub fn new() -> State {
        let food_positions = vec![];

        State {
            dt: std::time::Duration::new(0, 0),
            home_position: Coordinate::new(500.0, 500.0),
            food_positions,
            ants: vec![],
            home_scents: vec![],
            food_scents: vec![],
            home_food: 100,
        }
    }
}

fn enforce_x_bounds(x: f64) -> f64 {
    if x > X_SIZE as f64 {
        return X_SIZE as f64;
    } else if x < 0.0 {
        return 0.0;
    } else {
        return x;
    }
}

fn enforce_y_bounds(y: f64) -> f64 {
    if y > Y_SIZE as f64 {
        return Y_SIZE as f64;
    } else if y < 0.0 {
        return 0.0;
    } else {
        return y;
    }
}

fn gen_food_cluster(size: u32, x: f64, y: f64) -> Vec<Coordinate> {
    let mut coords = vec![];

    let x = enforce_x_bounds(x);
    let y = enforce_y_bounds(y);

    let mut r = rand::thread_rng();

    for _ in 0..size {
        // get baseline variance
        // TODO make this circular and not squareish or smtn
        let x_var: f64 = r.gen::<f64>() * (size as f64) / 4.0;
        let y_var: f64 = r.gen::<f64>() * (size as f64) / 4.0;

        // differ by some amount
        let x_pos = x_var - ((size as f64) / 2.0) + x;
        let y_pos = y_var - ((size as f64) / 2.0) + y;
        coords.push(Coordinate::new(
            enforce_x_bounds(x_pos),
            enforce_y_bounds(y_pos),
        ));
    }

    return coords;
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 10) {
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
                        .push(FoodScent::new(a.position.x, a.position.y, new_dir));

                    // walk
                    let next_pos = self.home_scents.get_next_point(
                        &a.position,
                        ANT_DETECTION_RANGE,
                        ANT_SPEED,
                        a.direction,
                    );

                    a.position = next_pos.0;
                    a.direction = next_pos.1;

                    // see if we have reached home
                    if a.position.dist(&self.home_position) < 50.0 {
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
                        .push(HomeScent::new(a.position.x, a.position.y, new_dir));

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
                        self.food_positions.remove(f);
                        a.has_food = true;
                    }

                    // walk
                    let next_pos = self.food_scents.get_next_point(
                        &a.position,
                        ANT_DETECTION_RANGE,
                        ANT_SPEED,
                        a.direction,
                    );

                    a.position = next_pos.0;
                    a.direction = next_pos.1;
                }
            }
            self.ants = Vec::drain_filter(&mut self.ants, |a| a.life > 0).collect();
        }
        self.dt = timer::delta(ctx);
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        println!("frame_time: {}", self.dt.as_millis());
        println!("ants: {}", self.ants.len());
        println!("home scents: {}", self.home_scents.len());
        println!("food scents: {}", self.food_scents.len());
        println!("home food: {}", self.home_food);
        let mut scene = &mut ggez::graphics::MeshBuilder::new();
        scene.circle(
            ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
            self.home_position.clone(),
            HOME_SIZE as f32,
            1.0,
            ggez::graphics::Color::from_rgb(200, 15, 0),
        );

        for f in self.food_positions.iter_mut() {
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                f.clone(),
                5 as f32,
                1.0,
                ggez::graphics::Color::from_rgb(15, 200, 15),
            );
        }

        for hs in self.home_scents.iter_mut() {
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                hs.position.clone(),
                5.0 * (hs.life / SCENT_LIFE) as f32,
                1.0,
                ggez::graphics::Color::from_rgb(0, 44, 190),
            );
        }

        for fs in self.food_scents.iter_mut() {
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                fs.position.clone(),
                5.0 * (fs.life / SCENT_LIFE) as f32,
                1.0,
                ggez::graphics::Color::from_rgb(190, 190, 0),
            );
        }

        for a in self.ants.iter_mut() {
            scene = scene.circle(
                ggez::graphics::DrawMode::Fill(ggez::graphics::FillOptions::DEFAULT),
                a.position.clone(),
                5 as f32,
                1.0,
                ggez::graphics::Color::from_rgb(220, 15, 0),
            );
        }
        let scene = scene.build(ctx).unwrap();
        ggez::graphics::clear(ctx, ggez::graphics::Color::from_rgb(0, 0, 0));
        ggez::graphics::draw(ctx, &scene, ggez::graphics::DrawParam::default()).unwrap();
        ggez::graphics::present(ctx).unwrap();
        Ok(())
    }
}

pub fn main() {
    let mut state = State::new();

    // gen food clusters
    for _ in 0..6 {
        let mut r = rand::thread_rng();

        // get baseline variance
        let x: f64 = r.gen::<f64>() * (X_SIZE as f64);
        let y: f64 = r.gen::<f64>() * (Y_SIZE as f64);

        let mut cluster = gen_food_cluster(150, enforce_x_bounds(x), enforce_y_bounds(y));
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
        width: 1000.0,
        height: 1000.0,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 1000.0,
        min_height: 1000.0,
        max_width: 1000.0,
        max_height: 1000.0,
        resizable: false,
    };
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("hello_ggez", "awesome_person")
        .conf(c)
        .build()
        .unwrap();
    event::run(ctx, event_loop, &mut state).unwrap();
}
