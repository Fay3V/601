use core::f64::consts::PI;

/// A is an angle in radians;  return an equivalent angle between plus
/// and minus pi
pub fn fix_angle_plus_minus_pi(a: f64) -> f64 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

///  @param a1: number representing angle; no restriction on range
///  @param a2: number representing angle; no restriction on range
///  @param eps: positive number
///  @returns: C{True} if C{a1} is within C{eps} of C{a2}.  Don't use
///  within for this, because angles wrap around!
pub(crate) fn near_angle(a1: f64, a2: f64, eps: f64) -> bool {
    fix_angle_plus_minus_pi(a1 - a2).abs() < eps
}
