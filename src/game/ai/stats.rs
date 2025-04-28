pub fn std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        // HACK
        return 0.0;
    }

    let mean = values.iter().copied().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|&value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

pub trait StdDev {
    fn std_dev(&self) -> f64;
}

impl StdDev for &[f64] {
    fn std_dev(&self) -> f64 {
        std_dev(self)
    }
}

impl StdDev for Vec<f64> {
    fn std_dev(&self) -> f64 {
        std_dev(self)
    }
}

