/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util/bits.rs
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

use std::mem::size_of;

use num::PrimInt;

// NOTE: do not implement this trait for (type width in bits) > u32::MAX
pub trait IntBitUtil: PrimInt {
    // Count the minimum number of bits required to represent this number
    #[allow(clippy::cast_possible_truncation)]
    fn bit_length(&self) -> u32 {
        assert!((size_of::<Self>() << 3) <= u32::MAX as usize);
        (size_of::<Self>() << 3) as u32 - self.leading_zeros()
    }
}

impl IntBitUtil for u8 {}
impl IntBitUtil for u16 {}
impl IntBitUtil for u32 {}
impl IntBitUtil for u64 {}
impl IntBitUtil for u128 {}
