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

pub(crate) fn count_trailing_zeroes<T: PrimInt>(mut n: T) -> u32 {
    let mut count = 0;
    while n != T::zero() {
        count += 1;
        n = n >> 1;
    }
    count
}
