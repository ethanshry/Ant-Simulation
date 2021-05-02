#![feature(drain_filter)]
#![feature(destructuring_assignment)]

// TODO refactor so we don't need these in all files
const X_SIZE: f32 = 500.0;
const Y_SIZE: f32 = 500.0;

pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    pub fn new(x: f32, y: f32) -> Coordinate {
        Coordinate { x, y }
    }

    pub fn enforce_bounds(&mut self, x_low: f32, x_high: f32, y_low: f32, y_high: f32) {
        self.x = match self.x {
            _ if self.x > x_high => X_SIZE,
            _ if self.x < x_low => x_low,
            _ => self.x,
        };
        self.y = match self.y {
            _ if self.y > y_high => y_high,
            _ if self.y < y_low => y_low,
            _ => self.y,
        };
    }

    pub fn dist(&self, coor: &Coordinate) -> f32 {
        let dist_x = (self.x - coor.x) * (self.x - coor.x);
        println!("varx {}", self.x);
        println!("varx {}", coor.x);
        println!("varx {}", self.x - coor.x);
        println!("x {}", dist_x);
        let dist_y = (self.y - coor.y) * (self.y - coor.y);
        println!("y {}", dist_y);
        return (dist_x + dist_y).sqrt();
    }

    pub fn direction(&self, coor: &Coordinate) -> f32 {
        let x_diff = coor.x - self.x;
        let y_diff = coor.y - self.y;
        let mut angle = (y_diff / x_diff).atan().to_degrees();
        if x_diff < 0.0 && y_diff > 0.0 {
            angle += 180.0;
        } else if x_diff < 0.0 && y_diff < 0.0 {
            angle -= 180.0;
        }

        // get the angle into positive terms
        while angle > 360.0 {
            angle -= 360.0
        }
        while angle < 0.0 {
            angle += 360.0
        }

        return angle;
    }

    pub fn traverse_direction(&self, dir: f32, dist: f32) -> Coordinate {
        let y = dist * dir.to_radians().sin();
        let x = dist * dir.to_radians().cos();
        Coordinate::new(self.x + x, self.y + y)
    }
}

impl Into<ggez::mint::Point2<f32>> for Coordinate {
    fn into(self) -> ggez::mint::Point2<f32> {
        ggez::mint::Point2 {
            x: self.x,
            y: self.y,
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

#[test]
fn coordinate_distance() {
    let a = Coordinate::new(5.0, 5.0);
    let b = Coordinate::new(0.0, 0.0);

    assert_eq!(a.dist(&b), 7.071068);
    assert_eq!(a.dist(&b), b.dist(&a));

    let a = Coordinate::new(-4.0, 0.0);
    let b = Coordinate::new(0.0, 0.0);

    assert_eq!(a.dist(&b), 4.0);
    assert_eq!(a.dist(&b), b.dist(&a));

    let a = Coordinate::new(0.0, 4.0);
    let b = Coordinate::new(0.0, 0.0);

    assert_eq!(a.dist(&b), 4.0);
    assert_eq!(a.dist(&b), b.dist(&a));
}

#[test]
fn coordinate_direction() {
    let a = Coordinate::new(5.0, 5.0);
    let b = Coordinate::new(0.0, 0.0);

    assert_eq!(a.direction(&b), 225.0);
    assert_eq!(b.direction(&a), 45.0);
}

#[test]
fn coordinate_traverse() {
    let a = Coordinate::new(0.0, 0.0);
    let new_coor = a.traverse_direction(0.0, 5.0);
    assert_eq!(new_coor.y, a.y);
    assert_eq!(new_coor.x, a.x + 5.0);

    let new_coor = a.traverse_direction(135.0, 7.071068);
    assert_eq!(new_coor.y, a.y + 5.0);
    assert_eq!(new_coor.x, a.x - 5.0);

    // TODO probably should properly check the signs for these results
    let new_coor = a.traverse_direction(225.0, 7.071068);
    assert!(f32::abs(new_coor.y - (a.y - 5.0)) < 0.1);
    assert!(f32::abs(new_coor.x - (a.x - 5.0)) < 0.1);

    let new_coor = a.traverse_direction(128.0, 13.13);
    assert!(f32::abs(a.dist(&new_coor) - 13.13) < 0.1);
    assert!(f32::abs(a.direction(&new_coor) - 128.0) < 0.1);
}