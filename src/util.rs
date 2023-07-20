/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util.rs
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

pub(crate) mod macros {
    macro_rules! arr {
        (
            $( #[$attr:meta] )*
            $v:vis $id:ident $name:ident: [$ty:ty; _] = $value:expr
        ) => {
            $( #[$attr] )*
            $v $id $name: [$ty; $value.len()] = $value;
        }
    }

    pub(crate) use arr;
}

pub(crate) use arr;
