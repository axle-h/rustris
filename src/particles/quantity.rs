use crate::particles::geometry::PointF;

#[derive(Clone, Debug, PartialEq)]
pub struct VariableQuantity<T : Clone> {
    quantity: T,
    variance: T
}

impl<T : Clone> VariableQuantity<T> {
    pub fn new(quantity: T, variance: T) -> Self {
        Self { quantity, variance }
    }
}

impl<T : Clone> From<(T, T)> for VariableQuantity<T> {
    fn from((quantity, variance): (T, T)) -> Self {
        VariableQuantity::new(quantity, variance)
    }
}

fn rand_signed_f64() -> f64 {
    2.0 * rand::random::<f64>() - 1.0
}

impl From<f64> for VariableQuantity<f64> {
    fn from(quantity: f64) -> Self {
        VariableQuantity::new(quantity, 0.0)
    }
}

impl From<PointF> for VariableQuantity<PointF> {
    fn from(quantity: PointF) -> Self {
        VariableQuantity::new(quantity, PointF::ZERO)
    }
}

impl VariableQuantity<f64> {
    pub fn next(&self) -> f64 {
        self.quantity + self.variance * rand_signed_f64()
    }
}

impl VariableQuantity<PointF> {
    pub fn next(&self) -> PointF {
        self.quantity + self.variance * PointF::new(rand_signed_f64(), rand_signed_f64())
    }
}