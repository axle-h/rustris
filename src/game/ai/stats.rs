pub fn std_dev(values: &[f32]) -> f32 {
    if values.is_empty() {
        // HACK
        return 0.0;
    }

    let mean = values.iter().copied().sum::<f32>() / values.len() as f32;
    let variance = values.iter()
        .map(|&value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f32>() / values.len() as f32;
    variance.sqrt()
}

pub trait StdDev {
    fn std_dev(&self) -> f32;
}

impl StdDev for &[f32] {
    fn std_dev(&self) -> f32 {
        std_dev(self)
    }
}

impl StdDev for Vec<f32> {
    fn std_dev(&self) -> f32 {
        std_dev(self)
    }
}

