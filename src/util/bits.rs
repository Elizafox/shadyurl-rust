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

// NOTE: do not implement this type for size_of::<Size> > u32::MAX
// (size_of::<Self>() regrettably is not a compile-time expression, so we can't do a static assert)
pub trait IntBitUtil: Sized + PrimInt {
    // Count the minimum number of bits required to represent this number
    #[allow(clippy::cast_possible_truncation)]
    fn bit_length(&self) -> u32 {
        (size_of::<Self>() - (self.leading_zeros() as usize)) as u32
    }
}

impl IntBitUtil for u8 {}
impl IntBitUtil for u16 {}
impl IntBitUtil for u32 {}
impl IntBitUtil for u64 {}
impl IntBitUtil for u128 {}
