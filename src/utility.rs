///! Some helper functions used in various places

/// Fuzzy compare of two floats with absolute tolerance
pub fn fuzzy_eq_absolute(x: f64, y: f64, tol: f64 ) -> bool {
    (x-y).abs() <= tol
}
