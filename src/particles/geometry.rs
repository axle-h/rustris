use std::ops::{Add, AddAssign, Mul};

#[derive(Clone, Copy, Debug)]
pub struct PointF {
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

impl PointF {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const ZERO: PointF = PointF::new(0.0, 0.0);

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn is_zero(&self) -> bool {
        self == &PointF::ZERO
    }

    pub fn normalize(&self) -> Self {
        if self.is_zero() {
            return self.clone();
        }

        let n = self.x * self.x + self.y * self.y;
        if approx_eq!(n, 1.0) {
            // already normalized.
            return self.clone();
        }
        let n = n.sqrt();
        if approx_0!(n) {
            // risking overflow, back to origin
            return PointF::ZERO;
        }
        let n = 1.0 / n;
        Self::new(self.x * n, self.y * n)
    }
}

impl PartialEq for PointF {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(self.x, other.x) && approx_eq!(self.y, other.y)
    }

    fn ne(&self, other: &Self) -> bool {
        approx_ne!(self.x, other.x) || approx_ne!(self.y, other.y)
    }
}

impl Add for PointF {
    type Output = PointF;

    fn add(self, rhs: Self) -> Self::Output {
        PointF::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for PointF {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f64> for PointF {
    type Output = PointF;

    fn mul(self, rhs: f64) -> Self::Output {
        PointF::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<PointF> for PointF {
    type Output = PointF;

    fn mul(self, rhs: PointF) -> Self::Output {
        PointF::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl From<(f64, f64)> for PointF {
    fn from((x, y): (f64, f64)) -> PointF {
        PointF::new(x, y)
    }
}

impl Into<(f64, f64)> for PointF {
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

    pub fn contains_point<P : Into<PointF>>(&self, point: P) -> bool {
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
    fn origin_normalizes_to_origin() {
        assert_eq!(PointF::ZERO.normalize(), PointF::ZERO);
    }

    #[test]
    fn normalized_point_normalizes_to_same_point() {
        let point = PointF::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        assert_eq!(point.normalize(), point);
    }

    #[test]
    fn normalize_point() {
        let normal = PointF::new(123.456, 789.0).normalize();
        assert_eq!(normal, PointF::new(0.15459048205347672, 0.9879786348188272));
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