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

use num::PrimInt;

pub trait HasILog2Checked: PrimInt {
    fn checked_ilog2(self) -> Option<u32>;
}

impl HasILog2Checked for u8 {
    fn checked_ilog2(self) -> Option<u32> {
        Self::checked_ilog2(self)
    }
}

impl HasILog2Checked for u16 {
    fn checked_ilog2(self) -> Option<u32> {
        Self::checked_ilog2(self)
    }
}

impl HasILog2Checked for u32 {
    fn checked_ilog2(self) -> Option<u32> {
        Self::checked_ilog2(self)
    }
}

impl HasILog2Checked for u64 {
    fn checked_ilog2(self) -> Option<u32> {
        Self::checked_ilog2(self)
    }
}

impl HasILog2Checked for u128 {
    fn checked_ilog2(self) -> Option<u32> {
        Self::checked_ilog2(self)
    }
}

pub trait IntBitUtil: HasILog2Checked {
    fn bit_length(&self) -> u32 {
        // This will generate reasonable code, believe it or not.
        self.checked_ilog2().map_or(0, |v| v + 1)
    }
}

impl IntBitUtil for u8 {}
impl IntBitUtil for u16 {}
impl IntBitUtil for u32 {}
impl IntBitUtil for u64 {}
impl IntBitUtil for u128 {}
