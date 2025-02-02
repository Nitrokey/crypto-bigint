//! [`UInt`] bitwise left shift operations.

use crate::{limb::HI_BIT, Limb, UInt, Word};
use core::ops::{Shl, ShlAssign};

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `self << 1` in constant-time, returning the overflowing bit as a `Word` that is either 0...0 or 1...1.
    pub(crate) const fn shl_1(&self) -> (Self, Word) {
        let mut shifted_bits = [0; LIMBS];
        let mut i = 0;
        while i < LIMBS {
            shifted_bits[i] = self.limbs[i].0 << 1;
            i += 1;
        }

        let mut carry_bits = [0; LIMBS];
        let mut i = 0;
        while i < LIMBS {
            carry_bits[i] = self.limbs[i].0 >> HI_BIT;
            i += 1;
        }

        let mut limbs = [Limb(0); LIMBS];

        limbs[0] = Limb(shifted_bits[0]);
        let mut i = 1;
        while i < LIMBS {
            limbs[i] = Limb(shifted_bits[i] | carry_bits[i - 1]);
            i += 1;
        }

        (
            UInt::new(limbs),
            carry_bits[LIMBS - 1].wrapping_mul(Word::MAX),
        )
    }

    /// Computes `self << shift`.
    ///
    /// NOTE: this operation is variable time with respect to `n` *ONLY*.
    ///
    /// When used with a fixed `n`, this function is constant-time with respect
    /// to `self`.
    #[inline(always)]
    pub const fn shl_vartime(&self, n: usize) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];

        if n >= Limb::BIT_SIZE * LIMBS {
            return Self { limbs };
        }

        let shift_num = n / Limb::BIT_SIZE;
        let rem = n % Limb::BIT_SIZE;
        let nz = Limb(rem as Word).is_nonzero();
        let lshift_rem = rem as Word;
        let rshift_rem = Limb::ct_select(Limb::ZERO, Limb((Limb::BIT_SIZE - rem) as Word), nz).0;

        let mut i = LIMBS - 1;
        while i > shift_num {
            let mut limb = self.limbs[i - shift_num].0 << lshift_rem;
            let hi = self.limbs[i - shift_num - 1].0 >> rshift_rem;
            limb |= hi & nz;
            limbs[i] = Limb(limb);
            i -= 1
        }
        limbs[shift_num] = Limb(self.limbs[0].0 << lshift_rem);

        Self { limbs }
    }

    /// Computes a left shift on a wide input as `(lo, hi)`.
    ///
    /// NOTE: this operation is variable time with respect to `n` *ONLY*.
    ///
    /// When used with a fixed `n`, this function is constant-time with respect
    /// to `self`.
    #[inline(always)]
    pub const fn shl_vartime_wide(lower_upper: (Self, Self), n: usize) -> (Self, Self) {
        let (lower, mut upper) = lower_upper;
        let new_lower = lower.shl_vartime(n);
        upper = upper.shl_vartime(n);
        if n >= LIMBS * Limb::BIT_SIZE {
            upper = upper.bitor(&lower.shl_vartime(n - LIMBS * Limb::BIT_SIZE));
        } else {
            upper = upper.bitor(&lower.shr_vartime(LIMBS * Limb::BIT_SIZE - n));
        }

        (new_lower, upper)
    }
}

impl<const LIMBS: usize> Shl<usize> for UInt<LIMBS> {
    type Output = UInt<LIMBS>;

    /// NOTE: this operation is variable time with respect to `rhs` *ONLY*.
    ///
    /// When used with a fixed `rhs`, this function is constant-time with respect
    /// to `self`.
    fn shl(self, rhs: usize) -> UInt<LIMBS> {
        self.shl_vartime(rhs)
    }
}

impl<const LIMBS: usize> Shl<usize> for &UInt<LIMBS> {
    type Output = UInt<LIMBS>;

    /// NOTE: this operation is variable time with respect to `rhs` *ONLY*.
    ///
    /// When used with a fixed `rhs`, this function is constant-time with respect
    /// to `self`.
    fn shl(self, rhs: usize) -> UInt<LIMBS> {
        self.shl_vartime(rhs)
    }
}

impl<const LIMBS: usize> ShlAssign<usize> for UInt<LIMBS> {
    /// NOTE: this operation is variable time with respect to `rhs` *ONLY*.
    ///
    /// When used with a fixed `rhs`, this function is constant-time with respect
    /// to `self`.
    fn shl_assign(&mut self, rhs: usize) {
        *self = self.shl_vartime(rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, UInt, U128, U256};

    const N: U256 =
        U256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141");

    const TWO_N: U256 =
        U256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD755DB9CD5E9140777FA4BD19A06C8282");

    const FOUR_N: U256 =
        U256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFAEABB739ABD2280EEFF497A3340D90504");

    const SIXTY_FIVE: U256 =
        U256::from_be_hex("FFFFFFFFFFFFFFFD755DB9CD5E9140777FA4BD19A06C82820000000000000000");

    const EIGHTY_EIGHT: U256 =
        U256::from_be_hex("FFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD03641410000000000000000000000");

    const SIXTY_FOUR: U256 =
        U256::from_be_hex("FFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD03641410000000000000000");

    #[test]
    fn shl_simple() {
        let mut t = U256::from(1u8);
        assert_eq!(t << 1, U256::from(2u8));
        t = U256::from(3u8);
        assert_eq!(t << 8, U256::from(0x300u16));
    }

    #[test]
    fn shl1() {
        assert_eq!(N << 1, TWO_N);
    }

    #[test]
    fn shl2() {
        assert_eq!(N << 2, FOUR_N);
    }

    #[test]
    fn shl65() {
        assert_eq!(N << 65, SIXTY_FIVE);
    }

    #[test]
    fn shl88() {
        assert_eq!(N << 88, EIGHTY_EIGHT);
    }

    #[test]
    fn shl256() {
        assert_eq!(N << 256, U256::default());
    }

    #[test]
    fn shl64() {
        assert_eq!(N << 64, SIXTY_FOUR);
    }

    #[test]
    fn shl_wide_1_1_128() {
        assert_eq!(
            UInt::shl_vartime_wide((U128::ONE, U128::ONE), 128),
            (U128::ZERO, U128::ONE)
        );
    }

    #[test]
    fn shl_wide_max_0_1() {
        assert_eq!(
            UInt::shl_vartime_wide((U128::MAX, U128::ZERO), 1),
            (U128::MAX.sbb(&U128::ONE, Limb::ZERO).0, U128::ONE)
        );
    }

    #[test]
    fn shl_wide_max_max_256() {
        assert_eq!(
            UInt::shl_vartime_wide((U128::MAX, U128::MAX), 256),
            (U128::ZERO, U128::ZERO)
        );
    }
}
