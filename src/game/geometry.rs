use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.cmp(&other.x) {
            Ordering::Equal => self.y.cmp(&other.y),
            other => other,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub const fn from_u32(x: u32, y: u32) -> Point {
        Point { x: x as i32, y: y as i32 }
    }

    pub fn translate(&self, x: i32, y: i32) -> Point {
        Point {
            x: self.x + x,
            y: self.y + y,
        }
    }

    pub fn rotate(&self, clockwise: bool) -> Point {
        if clockwise {
            Point {
                x: self.y,
                y: -self.x,
            }
        } else {
            Point {
                x: -self.y,
                y: self.x,
            }
        }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Point::new(x, y)
    }
}

impl From<(u32, u32)> for Point {
    fn from((x, y): (u32, u32)) -> Self {
        Point::from_u32(x, y)
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y)
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point::new(rhs.x + self.x, rhs.y + self.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::North
    }
}

impl Rotation {
    pub fn rotate(&self, clockwise: bool) -> Rotation {
        match self {
            Rotation::North => {
                if clockwise {
                    Rotation::East
                } else {
                    Rotation::West
                }
            }
            Rotation::East => {
                if clockwise {
                    Rotation::South
                } else {
                    Rotation::North
                }
            }
            Rotation::South => {
                if clockwise {
                    Rotation::West
                } else {
                    Rotation::East
                }
            }
            Rotation::West => {
                if clockwise {
                    Rotation::North
                } else {
                    Rotation::South
                }
            }
        }
    }
    pub fn angle(&self) -> f64 {
        // match self {
        //     Rotation::North => 0.0,
        //     Rotation::East => PI / 2.0,
        //     Rotation::South => PI,
        //     Rotation::West => 3.0 * PI / 2.0
        // }
        match self {
            Rotation::North => 0.0,
            Rotation::East => 90.0,
            Rotation::South => 180.0,
            Rotation::West => 270.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clockwise_rotation() {
        assert_eq!(Rotation::North.rotate(true), Rotation::East);
        assert_eq!(Rotation::West.rotate(true), Rotation::North);
    }

    #[test]
    fn anticlockwise_rotation() {
        assert_eq!(Rotation::North.rotate(false), Rotation::West);
        assert_eq!(Rotation::East.rotate(false), Rotation::North);
    }
}
