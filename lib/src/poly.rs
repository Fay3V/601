use std::{
    cmp,
    fmt::Display,
    ops::{Add, Mul, Sub},
};

#[cfg(feature = "poles")]
use faer::{Mat, complex::Complex};

#[derive(Clone)]
pub struct Poly {
    coeffs: Vec<f64>,
}

pub struct DispPoly<'a, const V: char>(pub &'a Poly);

impl<'a, const V: char> Display for DispPoly<'a, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (deg, coef) in self
            .0
            .coeffs
            .iter()
            .enumerate()
            .filter(|(_, coef)| **coef != 0.0)
        {
            let n = self.0.coeffs.len();
            let coef = if deg != 0 {
                if coef.is_sign_positive() {
                    write!(f, " + ")?;
                    coef.clone()
                } else {
                    write!(f, " - ")?;
                    coef.abs()
                }
            } else {
                if coef.is_sign_negative() {
                    write!(f, "-")?;
                }
                coef.abs()
            };

            match n - deg - 1 {
                0 => write!(f, "{coef}")?,
                1 => {
                    if coef != 1.0 {
                        write!(f, "{coef}")?
                    }
                    write!(f, "{V}")?
                }
                deg => {
                    if coef != 1.0 {
                        write!(f, "{coef}")?
                    }
                    write!(f, "{V}^{deg}")?
                }
            }
        }
        Ok(())
    }
}

impl Poly {
    pub fn new<const N: usize>(coeffs: [f64; N]) -> Self {
        assert!(coeffs[0] != 0.0, "zero poly is not supported");
        assert!(N != 0);
        Self {
            coeffs: coeffs.into(),
        }
    }

    pub fn from_vec(coeffs: Vec<f64>) -> Self {
        assert!(coeffs[0] != 0.0, "zero poly is not supported");
        assert!(coeffs.len() != 0);
        Self { coeffs }
    }

    pub fn coeffs(self) -> Vec<f64> {
        self.coeffs
    }

    pub fn reciprocal(&self) -> Poly {
        let mut coeffs = self.coeffs.clone();
        coeffs.reverse();
        // unwrap because at least one coeff is not 0.0
        let first_non_zero = coeffs.iter().position(|v| *v != 0.0).unwrap();
        coeffs.drain(0..first_non_zero);
        Poly { coeffs }
    }

    #[cfg(feature = "poles")]
    pub fn roots(&self) -> Vec<Complex<f64>> {
        // Build the companion matrix (n-1 x n-1)
        let n = self.coeffs.len();
        let mut c = Mat::<f64>::zeros(n - 1, n - 1);

        // Fill subdiagonal with ones
        for i in 1..(n - 1) {
            c[(i, i - 1)] = 1.0;
        }

        // Fill first row with -a_i / a_n
        let a_n = self.coeffs[0];
        for j in 0..(n - 1) {
            c[(0, j)] = -self.coeffs[j + 1] / a_n;
        }

        // Compute eigenvalues — these are the polynomial roots
        c.eigenvalues().unwrap()
    }
}

impl Mul<Poly> for Poly {
    type Output = Poly;

    fn mul(self, rhs: Poly) -> Self::Output {
        let slen = self.coeffs.len();
        let olen = rhs.coeffs.len();
        let prod = (0..slen + olen - 1)
            .map(|i| {
                let mut p = 0.0;
                let kstart = cmp::max(olen, i + 1) - olen;
                let kend = cmp::min(slen, i + 1);
                for k in kstart..kend {
                    p = p + self.coeffs[k] * rhs.coeffs[i - k];
                }
                p
            })
            .collect();
        Poly::from_vec(prod)
    }
}

impl Add<Poly> for Poly {
    type Output = Poly;

    fn add(self, rhs: Poly) -> Self::Output {
        let self_len = self.coeffs.len();
        let rhs_len = rhs.coeffs.len();
        let max_len = cmp::max(self_len, rhs_len);
        let min_len = cmp::min(self_len, rhs_len);
        let mut sum = Vec::with_capacity(max_len);

        if min_len != max_len {
            sum.resize(max_len - min_len, 0.0);
        }

        let other = if self_len < rhs_len {
            sum.extend_from_slice(&self.coeffs);
            &rhs.coeffs
        } else {
            sum.extend_from_slice(&rhs.coeffs);
            &self.coeffs
        };

        for (s, o) in sum.iter_mut().zip(other.iter()) {
            *s += o;
        }

        Poly::from_vec(sum)
    }
}

impl Sub<Poly> for Poly {
    type Output = Poly;

    fn sub(self, rhs: Poly) -> Self::Output {
        let self_len = self.coeffs.len();
        let rhs_len = rhs.coeffs.len();
        let max_len = cmp::max(self_len, rhs_len);
        let min_len = cmp::min(self_len, rhs_len);
        let mut sum = Vec::with_capacity(max_len);

        if min_len != max_len {
            sum.resize(max_len - min_len, 0.0);
        }

        let other = if self_len < rhs_len {
            sum.extend_from_slice(&self.coeffs);
            &rhs.coeffs
        } else {
            sum.extend_from_slice(&rhs.coeffs);
            &self.coeffs
        };

        for (s, o) in sum.iter_mut().zip(other.iter()) {
            *s -= o;
        }

        Poly::from_vec(sum)
    }
}

#[cfg(test)]
mod tests {
    use crate::poly::{DispPoly, Poly};
    use faer::complex::Complex;
    use std::cmp::Ordering;

    fn real_roots(p: Poly) -> Vec<f64> {
        let roots = p.roots();
        let mut roots = roots
            .into_iter()
            .map(|c| {
                assert_eq!(c.im, 0.0);
                c.re
            })
            .collect::<Vec<_>>();
        roots.sort_by(f64::total_cmp);
        roots
    }

    fn complex_roots(p: Poly) -> Vec<Complex<f64>> {
        let mut roots = p.roots();
        roots.sort_by(|c1, c2| match c1.re.total_cmp(&c2.re) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => c1.im.total_cmp(&c2.im),
        });
        roots
    }

    fn float_assert_eq(left_val: f64, right_val: f64) {
        if !((left_val - right_val).abs() <= 10e-9) {
            panic!(
                r#"assertion `left == right` failed
left: {left_val:?}
right: {right_val:?}"#
            );
        }
    }

    fn complex_float_assert(left_val: Complex<f64>, right_val: Complex<f64>) {
        float_assert_eq(left_val.re, right_val.re);
        float_assert_eq(left_val.im, right_val.im);
    }

    fn vec_float_assert_eq(left_val: &[f64], right_val: &[f64]) {
        assert_eq!(left_val.len(), right_val.len());
        for (left_val, right_val) in left_val.iter().zip(right_val) {
            float_assert_eq(*left_val, *right_val);
        }
    }

    #[test]
    fn first_deg() {
        let p = Poly::new([2.0, -4.0]);
        let roots = real_roots(p);
        assert_eq!(roots.len(), 1);
        float_assert_eq(2.0, roots[0]);
    }

    #[test]
    fn second_deg() {
        let p = Poly::new([1.0, -5.0, 6.0]);
        let roots = real_roots(p);
        assert_eq!(roots.len(), 2);
        float_assert_eq(2.0, roots[0]);
        float_assert_eq(3.0, roots[1]);
    }

    #[test]
    fn second_deg_complex() {
        let p = Poly::new([1.0, 4.0, 5.0]);
        let roots = complex_roots(p);
        assert_eq!(roots.len(), 2);
        complex_float_assert(Complex { re: -2.0, im: -1.0 }, roots[0]);
        complex_float_assert(Complex { re: -2.0, im: 1.0 }, roots[1]);
        // float_assert!(3.0, roots[1]);
        // let p = Poly::new([1.0, -6.0, 11.0, -6.0]);
        // let p = Poly::new([1.0, -1.0, 1.0, -1.0]);
        // let p = Poly::new([1.0, -10.0, 35.0, -50.0, 24.0]);
        // let p = Poly::new([1.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn multiple_roots() {
        let p = Poly::new([1.0, 2.0, 1.0]);
        let roots = real_roots(p);
        assert_eq!(roots.len(), 2);
        float_assert_eq(-1.0, roots[0]);
        float_assert_eq(-1.0, roots[1]);
    }

    #[test]
    fn test_mul() {
        macro_rules! test_mul {
            ($p1:expr, $p2:expr, $expected:expr) => {{
                let p1 = Poly::new($p1);
                // println!("p1 = {}", DispPoly::<'x'>(&p1));
                let p2 = Poly::new($p2);
                // println!("p2 = {}", DispPoly::<'x'>(&p2));
                let p3 = p1 * p2;
                // println!("p3 = {}", DispPoly::<'x'>(&p3));
                vec_float_assert_eq(&p3.coeffs, &$expected);
            }};
        }
        test_mul!([1.0], [1.0], [1.0]);
        test_mul!([1.0], [2.0], [2.0]);
        test_mul!([1.0, 0.0], [2.0], [2.0, 0.0]);
        test_mul!([1.0, 0.0], [1.0, 0.0], [1.0, 0.0, 0.0]);
        test_mul!([1.0, -1.0], [1.0, -1.0], [1.0, -2.0, 1.0]);
        test_mul!([1.0, -3.0], [2.0, 1.0, 2.0], [2.0, -5.0, -1.0, -6.0]);
        test_mul!(
            [1.0, -2.0, 3.0, -4.0],
            [2.0, 1.0, -5.0, 6.0],
            [2.0, -3.0, -1.0, 11.0, -31.0, 38.0, -24.0]
        );
    }

    #[test]
    fn test_add() {
        macro_rules! test_add {
            ($p1:expr, $p2:expr, $expected:expr) => {{
                let p1 = Poly::new($p1);
                // println!("p1 = {}", DispPoly::<'x'>(&p1));
                let p2 = Poly::new($p2);
                // println!("p2 = {}", DispPoly::<'x'>(&p2));
                let p3 = p1 + p2;
                // println!("p3 = {}", DispPoly::<'x'>(&p3));
                vec_float_assert_eq(&p3.coeffs, &$expected);
            }};
        }
        test_add!([1.0], [1.0], [2.0]);
        test_add!([1.0], [2.0], [3.0]);
        // test_add!([1.0], [-1.0], [0.0]); zero poly is not supported
        test_add!([1.0, 0.0], [2.0], [1.0, 2.0]);
        test_add!([1.0, 0.0], [1.0, 0.0], [2.0, 0.0]);
        test_add!([1.0, -1.0], [1.0, -1.0], [2.0, -2.0]);
        test_add!([1.0, -3.0], [2.0, 1.0, 2.0], [2.0, 2.0, -1.0]);
        test_add!(
            [1.0, -2.0, 3.0, -4.0],
            [2.0, 1.0, -5.0, 6.0],
            [3.0, -1.0, -2.0, 2.0]
        );
        test_add!(
            [1.0, 1.0, 1.0, 1.0, 1.0, -3.0],
            [-1.0, -1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0, 0.0, 0.0, -3.0]
        );
    }
}
