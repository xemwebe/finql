//! Useful helper functions that do not belong to any other module

/// Returns true if some optional String argument is not None and  the value equals a given str reference
pub fn some_equal(opt: &Option<String>, s: &str) -> bool {
    match opt {
        None => false,
        Some(opt_s) => opt_s == s,
    }
}
