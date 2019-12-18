//! Custom macro definitions

#[macro_export]
/// Assert macro to check whether to floats are equal within a given tolerance
macro_rules! assert_fuzzy_eq {
    ( $ left : expr , $ right : expr, $ tol : expr ) => {{
        match (&($left), &($right), &($tol)) {
            (left_val, right_val, tol) => {
                if !((*left_val - *right_val).abs() < *tol) {
                    panic!(
                        "assertion failed: left differs from right by more than `{:?}` \
                                   (left: `{:?}`, right: `{:?}`)",
                        *tol, *left_val, *right_val
                    )
                }
            }
        }
    }};
}
