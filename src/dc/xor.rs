use std::ops::{BitXor, BitXorAssign, Add, AddAssign, Sub, SubAssign, Neg};
use std::iter::FromIterator;
use rand::{Rand, Rng};

use super::Randomize;

pub struct DcXorElem<T>(Vec<T>);

pub type DcXorMsg = DcXorElem<u8>;
pub type DcXorMsgVec = DcXorElem<DcXorMsg>;

impl<T> BitXor for DcXorElem<T>
where
    T: BitXor,
    Vec<T>: FromIterator<<T as BitXor>::Output>,
{
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        debug_assert_eq!(self.0.len(), rhs.0.len());
        DcXorElem(
            self.0
                .into_iter()
                .zip(rhs.0.into_iter())
                .map(|(a, b)| T::bitxor(a, b))
                .collect(),
        )
    }
}

impl<T> BitXorAssign for DcXorElem<T>
where
    T: BitXorAssign,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        debug_assert_eq!(self.0.len(), rhs.0.len());
        DcXorElem(
            self.0
                .iter_mut()
                .zip(rhs.0.into_iter())
                .map(|(a, b)| T::bitxor_assign(a, b))
                .collect(),
        );
    }
}

impl<T> Add for DcXorElem<T>
where
    T: BitXor,
    Vec<T>: FromIterator<<T as BitXor>::Output>,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::bitxor(self, rhs)
    }
}

impl<T> AddAssign for DcXorElem<T>
where
    T: BitXor + BitXorAssign,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        Self::bitxor_assign(self, rhs)
    }
}

impl<T> Sub for DcXorElem<T>
where
    T: BitXor,
    Vec<T>: FromIterator<<T as BitXor>::Output>,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::bitxor(self, rhs)
    }
}

impl<T> SubAssign for DcXorElem<T>
where
    T: BitXor + BitXorAssign,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        Self::bitxor_assign(self, rhs)
    }
}

impl<T> Neg for DcXorElem<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        self
    }
}

impl Randomize for u8 {
    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        *self = u8::rand(rng);
    }
}

impl<T> Randomize for Vec<T> where T: Randomize {
    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        for x in self.iter_mut() {
            x.randomize(rng)
        }
    }
}

// TODO If we had a possibility to write overlapping trait impls we could do something like:
//
// impl<T> Randomize for T where T: Rand {
//     fn randomize<R: Rng>(&mut self, rng: &mut R) {
//         *self = T::rand(rng);
//     }
// }
//
// impl<T, U> Randomize for T
// where
//     T: IntoIterator<Item = U>,
//     U: Randomize,
// {
//     fn randomize<R: Rng>(&mut self, rng: &mut R) {
//         for x in self.iter_mut() {
//             x.randomize(rng)
//         }
//     }
// }

impl<T> Randomize for DcXorElem<T> where T: Randomize {
    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        self.0.randomize(rng);
    }
}
