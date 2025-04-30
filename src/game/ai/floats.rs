use assert_float_eq::{next_n_f64, previous_n_f64};

// Each step is derived from the previous float by incrementing the float's bits,
// as if they were an integer, by 1. For example, the next float from 1e-45 (0x00000001) would be 3e-45 (0x00000002).
const STEPS: u32 = 4;

pub fn is_near_f64(a: f64, b: f64) -> bool {
    let previous = previous_n_f64(a, STEPS);
    let next = next_n_f64(a, STEPS);
    b >= previous && b <= next
}

/// Round to significant digits.
pub fn precision_f64(x: f64, significant_digits: u32) -> f64 {
    if x == 0.0 || significant_digits == 0 {
        0.0
    } else {
        let shift = significant_digits as i32 - x.abs().log10().ceil() as i32;
        let shift_factor = 10_f64.powi(shift);

        (x * shift_factor).round() / shift_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_precision_f64() {
        const VALUE: f64 = 1.23456799;
        assert_eq!(precision_f64(VALUE, 0), 0.0);
        assert_eq!(precision_f64(VALUE, 1), 1.0);
        assert_eq!(precision_f64(VALUE, 2), 1.2);
        assert_eq!(precision_f64(VALUE, 3), 1.23);
        assert_eq!(precision_f64(VALUE, 4), 1.235);
        assert_eq!(precision_f64(VALUE, 5), 1.2346);
        assert_eq!(precision_f64(VALUE, 6), 1.23457);
        assert_eq!(precision_f64(VALUE, 7), 1.234568);
        assert_eq!(precision_f64(VALUE, 8), 1.2345680);
        assert_eq!(precision_f64(VALUE, 9), 1.23456799);
        assert_eq!(precision_f64(VALUE, 10), 1.23456799);
    }
}