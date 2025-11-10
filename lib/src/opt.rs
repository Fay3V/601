pub fn range(x_min: f64, x_max: f64, steps: u32) -> impl Iterator<Item = f64> {
    let incr = (x_max - x_min) / steps as f64;
    dbg!((x_min, x_max, incr));
    (0..=steps).scan(x_min, move |current, _| {
        let out = Some(*current);
        *current += incr;
        out
    })
}

pub fn opt_over_line<X, Y, F, C>(
    objective: F,
    mut xs: impl Iterator<Item = X>,
    is_best: C,
) -> (Y, X)
where
    F: Fn(X) -> Y,
    C: Fn(Y, Y) -> bool,
    X: Copy + std::fmt::Debug,
    Y: Copy + std::fmt::Debug,
{
    let mut best_x = xs.next().expect("at least one element");
    let mut best_objective = objective(best_x);
    // dbg!((best_x, best_objective));
    for x in xs {
        let obj = objective(x);
        // dbg!((x, obj));
        if is_best(obj, best_objective) {
            best_objective = obj;
            best_x = x;
        }
    }
    (best_objective, best_x)
}

#[cfg(test)]
mod tests {
    use crate::opt::{opt_over_line, range};
    use std::cmp::Ordering;
    fn float_assert_eq(left_val: f64, right_val: f64) {
        if !((left_val - right_val).abs() <= 10e-3) {
            panic!(
                r#"assertion `left == right` failed
left: {left_val:?}
right: {right_val:?}"#
            );
        }
    }

    #[test]
    fn it_works() {
        let (best_obj_value, best_x) = opt_over_line(
            |x: f64| x.powi(2) - x,
            range(0.0, 1.0, 10),
            |y1, y2| matches!(y1.total_cmp(&y2), Ordering::Less),
        );
        float_assert_eq(best_obj_value, -0.25);
        float_assert_eq(best_x, 0.5);

        let (best_obj_value, best_x) = opt_over_line(
            |x: f64| x.powi(5) - 7.0 * x.powi(3) + 6.0 * x.powi(2) + 2.0,
            range(0.0, 10.0, 1000),
            |y1, y2| matches!(y1.total_cmp(&y2), Ordering::Less),
        );
        dbg!((best_obj_value, best_x));
        float_assert_eq(best_x, 1.66);
        float_assert_eq(best_obj_value, -0.88);
    }
}
