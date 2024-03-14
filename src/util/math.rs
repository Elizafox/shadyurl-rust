/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util/math.rs
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

use num::Float;

// This algorithm is taken from Python's isclose() algorithm in C
pub fn is_close<T: Float>(a: T, b: T) -> bool {
    let abs_tol = T::from(0.0).unwrap();
    let rel_tol = T::from(1e-05).unwrap();

    if a == b {
        return true;
    }

    if a.is_infinite() || b.is_infinite() {
        return false;
    }

    let diff = (b - a).abs();

    ((diff <= (rel_tol * b).abs()) || (diff <= (rel_tol * a).abs())) || (diff <= abs_tol)
}
