use super::s256_field::S256Field;

use super::ec::point::PointError;

use super::ec::utils::{big_uint_to_u256, u256_parse_str, U256};
use num_bigint::BigUint;
use std::fmt;
use std::ops::{Add, Mul};

#[derive(Clone, Debug, Eq, PartialEq)]
enum PointValue {
    InfPoint,
    NormalPoint {
        /// `x` axis
        x: S256Field,
        /// `y` axis
        y: S256Field,
    },
}

impl Copy for PointValue {}

/// Elliptic curve, (y^2) % primer = (x^3 + a*x + b) % primer
#[derive(Clone, Debug, Eq, PartialEq)]
struct Secp256K1EllipticCurve {
    /// Elliptic curve `a` argument
    a: S256Field,
    /// Elliptic curve `b` argument
    b: S256Field,
}
impl Copy for Secp256K1EllipticCurve {}

impl Default for Secp256K1EllipticCurve {
    fn default() -> Self {
        Secp256K1EllipticCurve {
            a: Self::ec_a(),
            b: Self::ec_b(),
        }
    }
}

impl Secp256K1EllipticCurve {
    pub fn ec_a() -> S256Field {
        S256Field::new(U256::from(0u32))
    }

    pub fn ec_b() -> S256Field {
        S256Field::new(U256::from(7u32))
    }

    /// Secp256K1 elliptic curve group order, nG=0
    pub fn n() -> U256 {
        u256_parse_str(
            b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
            16,
        )
    }
}

/// Elliptic curve point, y^2 = x^3 + a*x + b
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct S256Point {
    point: PointValue,
    elliptic_curve: Secp256K1EllipticCurve,
}

impl fmt::Display for S256Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.point {
            PointValue::InfPoint => write!(
                f,
                "Inf_y^2 = x^3 + {}*x + {}",
                self.elliptic_curve.a, self.elliptic_curve.b
            ),
            PointValue::NormalPoint { x, y } => write!(
                f,
                "Point({}, {})_{}_{} S256Field({})",
                x.num, y.num, self.elliptic_curve.a.num, self.elliptic_curve.b.num, x.prime
            ),
        }
    }
}

impl Copy for S256Point {}

impl S256Point {
    pub fn new(x: S256Field, y: S256Field) -> Result<Self, PointError> {
        let a = Secp256K1EllipticCurve::ec_a();
        let b = Secp256K1EllipticCurve::ec_b();
        let left = y.pow(2);
        let right = x.pow(3) + a * x + b;
        if left != right {
            return Err(PointError::NotInEllipticCurves);
        }

        Ok(S256Point {
            point: PointValue::NormalPoint { x, y },
            elliptic_curve: Secp256K1EllipticCurve::default(),
        })
    }

    pub fn inf() -> Self {
        S256Point {
            point: PointValue::InfPoint,
            elliptic_curve: Secp256K1EllipticCurve::default(),
        }
    }

    pub fn is_inf(&self) -> bool {
        match self.point {
            PointValue::InfPoint => true,
            _ => false,
        }
    }

    pub fn gen_point() -> Self {
        let gx = u256_parse_str(
            b"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            16,
        );

        let gy = u256_parse_str(
            b"483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
            16,
        );
        let x = S256Field::new(gx);
        let y = S256Field::new(gy);
        S256Point::new(x, y).unwrap()
    }

    pub fn coordinate(&self) -> Option<(U256, U256)> {
        match self.point {
            PointValue::InfPoint => None,
            PointValue::NormalPoint { x, y } => Some((x.num, y.num)),
        }
    }
}

impl Add<S256Point> for S256Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.elliptic_curve != rhs.elliptic_curve {
            panic!("{}", PointError::NotInSameEllipticCurves);
        }

        let a = self.elliptic_curve.a;
        let _b = self.elliptic_curve.b;

        match (self.point, rhs.point) {
            (PointValue::NormalPoint { x, y }, PointValue::NormalPoint { x: rhs_x, y: rhs_y }) => {
                if x == rhs_x {
                    // vertical line
                    if y == rhs_y {
                        if y.num == U256::from(0) {
                            return Self::inf();
                        }
                        let s = (U256::from(3) * x.pow(2) + a) / (U256::from(2) * y);
                        let ret_x = s.pow(2) - U256::from(2) * x;
                        let ret_y = s * (x - ret_x) - y;
                        return S256Point::new(ret_x, ret_y).expect("Point add error");
                    }
                    return Self::inf();
                }

                let s = (rhs_y - y) / (rhs_x - x);
                let ret_x = s.pow(2) - x - rhs_x;
                let ret_y = s * (x - ret_x) - y;
                return S256Point::new(ret_x, ret_y).expect("Point add error");
            }
            // self or rhs is inf point
            (PointValue::InfPoint, _) => rhs,
            (_, PointValue::InfPoint) => self,
        }
    }
}

impl<T> Mul<T> for S256Point
where
    T: Into<U256>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let mut coef = rhs.into() % Secp256K1EllipticCurve::n();
        let mut current = self;

        let mut result = S256Point::inf();
        while coef > U256::from(0) {
            if coef & U256::from(1u32) == U256::from(1u32) {
                result = result + current;
            }
            current = current + current;
            coef = coef >> 1;
        }
        result
    }
}

mod test {
    use crate::secp256k1::ec::utils::{pow, u256_modmul, u256_modpow, u256_mul, u256_parse_str};
    use crate::secp256k1::s256_point::{S256Point, Secp256K1EllipticCurve};

    #[test]
    fn test_s256_point() {
        let n = Secp256K1EllipticCurve::n();
        let gen_point = S256Point::gen_point();

        assert_eq!(S256Point::inf(), gen_point * n)
    }

    #[test]
    fn test_verify_sig() {
        let z = u256_parse_str(
            b"bc62d4b80d9e36da29c16c5d4d9f11731f36052c72401a76c23c0fb5a9b74423",
            16,
        );
        let r = u256_parse_str(
            b"37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6",
            16,
        );
        let s = u256_parse_str(
            b"8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec",
            16,
        );
        let px = u256_parse_str(
            b"04519fac3d910ca7e7138f7013706f619fa8f033e6ec6e09370ea38cee6a7574",
            16,
        );
        let py = u256_parse_str(
            b"82b51eab8c27c66e26c858a079bcdf4f1ada34cec420cafc7eac1a42216fb6c4",
            16,
        );

        let n = Secp256K1EllipticCurve::n();

        let point = S256Point::new(px.into(), py.into()).unwrap();
        let s_inv = u256_modpow(s, n - u256_parse_str(b"2", 10), n);

        let u = u256_modmul(z, s_inv, n);
        let v = u256_modmul(r, s_inv, n);

        let g = S256Point::gen_point();
        let t = g * u + point * v;
        assert_eq!(r, t.coordinate().unwrap().0)
    }
}