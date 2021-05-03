use crate::coordinate::Coordinate;
/// Used to define things which should be able to be navigated
pub trait Navigable {
    /// Returns the nearest (Coordinate, Angle) in the Navigable item
    fn get_nearest(&self, pos: &Coordinate, range: f32, dist: f32, dir: f32) -> Coordinate;

    /// Returns the nearest (Coordinate, Angle) in the Naviable item
    fn get_avg_direction(&self, pos: &Coordinate, range: f32, dist: f32, dir: f32) -> f32;
}
