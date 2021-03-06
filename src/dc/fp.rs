use std::ops::{Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign};
use std::cmp::Ordering;
use rand::Rng;
use rand::distributions::{Standard, Distribution};
use serde::{Serialize, Deserialize};

use super::Randomize;

// The field size.
const P: u128 = (1 << 127) - 1;

// A finite field element.
//
// It is consistent iff 0 <= self.0 <= P.
// Note that this implies that the zero element has two internal representations.
#[derive(Clone, Copy, Default, Debug)]
pub struct Fp(u128);

impl Serialize for Fp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ::serde::Serializer
    {
        let u = u128::from(*self);
        serializer.serialize_u128(u)
    }
}

impl<'de> Deserialize<'de> for Fp {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Fp, D::Error>
        where D: ::serde::Deserializer<'de>
    {
        use serde::de;

        struct Visitor;
        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = Fp;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a u128 x such that 0 <= x < p where p = 2**127 - 1")
            }

            fn visit_u128<E>(self, x: u128) -> Result<Fp, E>
                where E: de::Error
            {
                if x >= P {
                    let unexp_str = format!("the u128 with value {:x} >= 2**127 - 1", x);
                    let unexp = de::Unexpected::Other(&unexp_str);
                    return Err(de::Error::invalid_value(unexp, &self))
                }
                Ok(Fp::from_u127(x))
            }
        }

        deserializer.deserialize_u128(Visitor { })
    }
}

#[inline]
fn as_limbs(x: u128) -> (u64, u64) {
    ((x >> 64) as u64, x as u64)
}

trait Reduce: Sized {
    fn reduce_once(self) -> u128;

    #[inline]
    fn reduce_once_assert(self) -> u128 {
        let red: u128 = self.reduce_once();
        debug_assert!(red <= P);
        red
    }
}

impl Reduce for u128 {
    #[inline]
    fn reduce_once(self) -> u128 {
        (self & P) + (self >> 127)
    }
}

impl Reduce for (u128, u128) {
    #[inline]
    fn reduce_once(self) -> u128 {
        let (h, l) = self;
        // shift = (h, l) >> 127
        let shift = (h << 1) | (l >> 127);
        (l & P) + shift
    }
}

impl Fp {
    #[inline]
    pub fn from_u127(x: u128) -> Self {
        // x == P is explicitly allowed.
        // This introduces a negligible bias towards the zero element
        // if x is uniformly random from {0,1}^127.
        debug_assert!(x <= P);
        Fp(x)
    }

    #[inline]
    pub fn from_u128_discard_msb(x: u128) -> Self {
        Self::from_u127(x & P)
    }

    #[inline]
    pub fn prime() -> u128 {
        P
    }
}

impl From<Fp> for u128 {
    #[inline]
    fn from(x: Fp) -> u128 {
        if x.0 == P { 0 } else { x.0 }
    }
}

impl Distribution<Fp> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Fp {
        Fp::from_u128_discard_msb(rng.gen::<u128>())
    }

}

impl Randomize for Fp {
    #[inline]
    fn randomize<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        *self = rng.gen::<Fp>();
    }

}

impl Neg for Fp {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Fp(P - self.0)
    }
}

impl Add for Fp {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Fp((self.0 + other.0).reduce_once_assert())
    }
}

impl AddAssign for Fp {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl Sub for Fp {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl SubAssign for Fp {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl Mul for Fp {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let (sh, sl) = as_limbs(self.0);
        let (oh, ol) = as_limbs(other.0);

        // (64 bits * 63 bits) + (64 bits * 63 bits) = 128 bits
        let m: u128 = (sh as u128 * ol as u128) + (oh as u128 * sl as u128);
        let (mh, ml) = as_limbs(m);

        // (64 bits * 64 bits) + 128 bits = 129 bits
        let (rl, carry) = (sl as u128 * ol as u128).overflowing_add((ml as u128) << 64);

        // (63 bits * 63 bits) + 64 bits + 1 bit = 127 bits
        let rh: u128 = (sh as u128 * oh as u128) + (mh as u128) + (carry as u128);

        Fp((rh, rl).reduce_once().reduce_once_assert())
    }
}

impl MulAssign for Fp {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl PartialEq for Fp {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        u128::from(*self) == u128::from(*other)
    }
}

impl Eq for Fp {}

impl PartialOrd for Fp {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        u128::from(*self).partial_cmp(&u128::from(*other))
    }
}

impl Ord for Fp {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neg() {
        assert_eq!(-Fp(0), Fp(0));
        assert_eq!(-Fp(5), Fp(P - 5));
    }

    #[test]
    fn add() {
        assert_eq!(Fp(7) + Fp(5), Fp(12));
        assert_eq!(Fp(P - 2) + Fp(5), Fp(3));
        assert_eq!(
            Fp(75661398932549814984099328258351945610) + Fp(154440289138086217180118920884960981429),
            Fp(59960504610166800432530945427428821312)
        );
    }

    #[test]
    fn sub() {
        assert_eq!(Fp(7) - Fp(5), Fp(2));
        assert_eq!(Fp(4) - Fp(8), Fp(P - 4));
    }

    #[test]
    fn mul() {
        assert_eq!(Fp(4) * Fp(3), Fp(12));
        assert_eq!(Fp(P) * Fp(291298091), Fp(0));
        assert_eq!(
            Fp(14766549069271113692204649107775507741) * Fp(153613967287097206589234951623852979690),
            Fp(113548737858505840193892055835373785352)
        );
        // Two reductions are necessary for the following example.
        assert_eq!(
            Fp(75661398932549814984099328258351945610) * Fp(154440289138086217180118920884960981429),
            Fp(109146875586984049909139102289297416971)
        );
    }

    #[test]
    fn eq() {
        assert_eq!(Fp(0), Fp(P));
        assert!(Fp(17) != Fp(4));
        assert!(Fp(0) != Fp(4));
        assert!(Fp(P) != Fp(17));
    }

    #[test]
    fn ord() {
        assert!(Fp(0) < Fp(1));
        assert!(Fp(17) > Fp(0));
        assert!(Fp(P) < Fp(1));
        assert!(Fp(23) > Fp(P));
    }

    #[test]
    fn assign() {
        let mut a = Fp(17);

        a += Fp(3);
        assert_eq!(a, Fp(20));

        a -= Fp(5);
        assert_eq!(a, Fp(15));

        a *= Fp(2);
        assert_eq!(a, Fp(30));
    }
}
