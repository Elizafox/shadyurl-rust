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

// Useful mathematical functions

use num::Float;

pub trait FloatMathUtil: Float {
    // This algorithm is taken from Python's isclose() algorithm in C
    fn is_close(self, n: Self) -> bool {
        let abs_tol = Self::from(0.0).unwrap();
        let rel_tol = Self::from(1e-05).unwrap();

        if self == n {
            return true;
        }

        if self.is_infinite() || n.is_infinite() {
            return false;
        }

        let diff = (n - self).abs();
        ((diff <= (rel_tol * n).abs()) || (diff <= (rel_tol * self).abs())) || (diff <= abs_tol)
    }
}

impl FloatMathUtil for f32 {}
impl FloatMathUtil for f64 {}
