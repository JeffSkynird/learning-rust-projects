/// Add all numbers.
pub fn add(nums: &[f64]) -> f64 {
    nums.iter().copied().sum()
}

/// Subtraction is left-associative.
/// E.g.: [10, 3, 2] => (10 - 3 - 2) = 5
pub fn sub(nums: &[f64]) -> f64 {
    let (first, rest) = nums.split_first().expect("at least 2 numbers");
    rest.iter().fold(*first, |acc, &x| acc - x)
}

/// Multiply all numbers.
pub fn mul(nums: &[f64]) -> f64 {
    nums.iter().product()
}

/// Divide is left-associative. Error if any divisor is zero.
/// E.g.: [20, 2, 5] => (20 / 2 / 5) = 2
pub fn div(nums: &[f64]) -> Result<f64, &'static str> {
    let (first, rest) = nums.split_first().expect("at least 2 numbers");
    let mut acc = *first;
    for &x in rest {
        if x == 0.0 {
            return Err("Error: division by zero");
        }
        acc /= x;
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn test_add_basic() {
        let nums = [1.0, 2.5, 3.5];
        let res = add(&nums);
        assert!(approx_eq(res, 7.0));
    }

    #[test]
    fn test_add_with_negatives() {
        let nums = [10.0, -3.0, -2.0, 5.0];
        let res = add(&nums);
        assert!(approx_eq(res, 10.0));
    }

    #[test]
    fn test_sub_left_associative() {
        let nums = [10.0, 3.0, 2.0];
        let res = sub(&nums);
        assert!(approx_eq(res, 5.0));
    }

    #[test]
    fn test_mul_basic() {
        let nums = [2.0, 3.0, 4.0];
        let res = mul(&nums);
        assert!(approx_eq(res, 24.0));
    }

    #[test]
    fn test_mul_with_zero() {
        let nums = [2.0, 0.0, 100.0];
        let res = mul(&nums);
        assert!(approx_eq(res, 0.0));
    }

    #[test]
    fn test_div_left_associative_ok() {
        let nums = [20.0, 2.0, 5.0];
        let res = div(&nums).unwrap();
        assert!(approx_eq(res, 2.0));
    }

    #[test]
    fn test_div_by_zero_in_rest_errors() {
        let nums = [10.0, 0.0];
        let err = div(&nums).unwrap_err();
        assert!(err.contains("division by zero"));
    }

    #[test]
    fn test_div_with_zero_first_is_ok() {
        let nums = [0.0, 2.0, 5.0];
        let res = div(&nums).unwrap();
        assert!(approx_eq(res, 0.0));
    }
}
