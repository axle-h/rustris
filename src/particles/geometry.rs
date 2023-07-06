use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Vec2D {
    x: f64,
    y: f64
}

const CMP_EPSILON: f64 = 1e-6;

macro_rules! approx_eq {
    ($a:expr, $b:expr) => {{
        ($a - $b).abs() < CMP_EPSILON
    }};
}

macro_rules! approx_ne {
    ($a:expr, $b:expr) => {{
        ($a - $b).abs() >= CMP_EPSILON
    }};
}

macro_rules! approx_0 {
    ($a:expr) => {{
        $a.abs() < CMP_EPSILON
    }};
}

impl Vec2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const ZERO: Vec2D = Vec2D::new(0.0, 0.0);

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn is_zero(&self) -> bool {
        self == &Vec2D::ZERO
    }

    pub fn unit_vector(&self) -> Self {
        if self.is_zero() {
            return self.clone();
        }

        let n = self.x.powi(2) + self.y.powi(2);
        if approx_eq!(n, 1.0) {
            // already normalized.
            return self.clone();
        }
        let n = n.sqrt();
        Self::new(self.x / n, self.y / n)
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.x().powi(2) + self.y().powi(2)
    }
}

impl PartialEq for Vec2D {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(self.x, other.x) && approx_eq!(self.y, other.y)
    }

    fn ne(&self, other: &Self) -> bool {
        approx_ne!(self.x, other.x) || approx_ne!(self.y, other.y)
    }
}

impl Add for Vec2D {
    type Output = Vec2D;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f64> for Vec2D {
    type Output = Vec2D;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2D::new(self.x * rhs, self.y * rhs)
    }
}

impl Sub for Vec2D {
    type Output = Vec2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl From<(f64, f64)> for Vec2D {
    fn from((x, y): (f64, f64)) -> Vec2D {
        Vec2D::new(x, y)
    }
}

impl Into<(f64, f64)> for Vec2D {
    fn into(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectF {
    x: f64,
    y: f64,
    width: f64,
    height: f64
}

impl RectF {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        assert!(width > 0.0);
        assert!(height > 0.0);
        Self { x, y, width, height }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    /// Returns the x-position of the left side of this rectangle.
    pub fn left(&self) -> f64 {
        self.x
    }

    /// Returns the x-position of the right side of this rectangle.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Returns the y-position of the top side of this rectangle.
    pub fn top(&self) -> f64 {
        self.y
    }

    /// Returns the y-position of the bottom side of this rectangle.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn contains_point<P : Into<Vec2D>>(&self, point: P) -> bool {
        let point = point.into();
        let (x, y) = point.into();
        let inside_x = x >= self.left() && x < self.right();
        inside_x && (y >= self.top() && y < self.bottom())
    }
}

impl From<(f64, f64, f64, f64)> for RectF {
    fn from((x, y, width, height): (f64, f64, f64, f64)) -> RectF {
        RectF::new(x, y, width, height)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_1_SQRT_2;
    use super::*;

    #[test]
    fn unit_vector_of_zero_is_zero() {
        assert_eq!(Vec2D::ZERO.unit_vector(), Vec2D::ZERO);
    }

    #[test]
    fn unit_vector_of_unit_vector_is_equal() {
        let point = Vec2D::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        assert_eq!(point.unit_vector(), point);
    }

    #[test]
    fn unit_vector() {
        let normal = Vec2D::new(123.456, 789.0).unit_vector();
        assert_eq!(normal, Vec2D::new(0.15459048205347672, 0.9879786348188272));
    }

    #[test]
    fn rect_contains_point() {
        let rect = RectF::new(1.0, 2.0, 3.0, 4.0);
        assert!(rect.contains_point((1.0, 2.0)));
        assert!(!rect.contains_point((0.0, 1.0)));
        assert!(rect.contains_point((3.0, 5.0)));
        assert!(!rect.contains_point((4.0, 6.0)));
    }
}