pub mod ec;
mod s256_field;
pub mod s256_point;

mod test {
    use super::ec::field_element::FieldElement;
    use super::ec::point::Point;
    use super::ec::utils::{big_uint_to_u256, u512_to_big_uint, u512_to_u256, U256, U512};
    use crate::secp256k1::ec::utils::u256_parse_str;
    use num_bigint::BigUint;

    #[test]
    fn test_big_num() {
        let gx = BigUint::parse_bytes(
            b"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            16,
        )
        .unwrap();

        let gy = BigUint::parse_bytes(
            b"483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
            16,
        )
        .unwrap();

        let p = U512::from(2u32).pow(U512::from(256u32))
            - U512::from(2u32).pow(U512::from(32u32))
            - U512::from(977u32);
        let p = u512_to_big_uint(p);

        let a = gy.modpow(&BigUint::from(2u32), &p);
        let b = (gx.modpow(&BigUint::from(3u32), &p) + BigUint::from(7u32)) % p;

        let a = big_uint_to_u256(&a);
        let b = big_uint_to_u256(&b);
        assert_eq!(a, b)
    }

    #[test]
    fn test_u256_scala_mul() {
        let gx = u256_parse_str(
            b"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            16,
        );

        let gy = u256_parse_str(
            b"483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
            16,
        );

        let p = U512::from(2u32).pow(U512::from(256u32))
            - U512::from(2u32).pow(U512::from(32u32))
            - U512::from(977u32);
        let p = u512_to_u256(p);

        let n = u256_parse_str(
            b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
            16,
        );

        let x = FieldElement::new(gx, p);
        let y = FieldElement::new(gy, p);

        let seven = FieldElement::new(U256::from(7), p);
        let zero = FieldElement::new(U256::from(0), p);

        let gen_point = Point::new(x, y, zero, seven).unwrap();

        assert_eq!(gen_point * n, Point::inf(zero, seven));
    }
}