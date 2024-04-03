/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util/string.rs
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

// Useful utilities for working with/creating strings

use once_cell::sync::Lazy;
use rand::{
    distributions::{DistString, Uniform},
    prelude::*,
};
use time::{
    convert::{Day, Hour, Microsecond, Millisecond, Minute, Nanosecond, Second, Week},
    Duration,
};

use super::math::FloatMathUtil;

// Convert a Duration into something for humans.
// This implementation is heavily modified from the time crate.
pub fn humanize_duration(duration: Duration) -> String {
    const AVERAGE_YEAR: f64 = 365.2425;
    const AVERAGE_MONTH: f64 = AVERAGE_YEAR / 12.0;

    let suffix = if duration.is_positive() {
        "ago"
    } else {
        "from now"
    };

    // Concise, rounded representation.

    if duration.is_zero() {
        return "now".to_string();
    }

    /// Format the first item that produces a value greater than 1 and then break.
    macro_rules! item {
        ($singular:literal, $plural:literal, $value:expr) => {
            let value = $value;
            if value.round().is_close(1.0) {
                return format!("{} {suffix}", $singular);
            } else if value > 1.0 {
                return format!("{value:.0} {} {suffix}", $plural);
            }
        };
    }

    // Even if this produces a de-normal float, because we're rounding we don't really care.
    let seconds = duration.unsigned_abs().as_secs_f64();

    item!(
        "a year",
        "years",
        seconds / (f64::from(Second::per(Day)) * AVERAGE_YEAR)
    );
    item!(
        "a month",
        "months",
        seconds / (f64::from(Second::per(Day)) * AVERAGE_MONTH)
    );
    item!("a week", "weeks", seconds / f64::from(Second::per(Week)));
    item!("a day", "days", seconds / f64::from(Second::per(Day)));
    item!("an hour", "hours", seconds / f64::from(Second::per(Hour)));
    item!(
        "a minute",
        "minutes",
        seconds / f64::from(Second::per(Minute))
    );
    item!("a second", "seconds", seconds);
    item!(
        "a millisecond",
        "milliseconds",
        seconds * f64::from(Millisecond::per(Second))
    );
    item!(
        "a microsecond",
        "microseconds",
        seconds * f64::from(Microsecond::per(Second))
    );
    item!(
        "a nanosecond",
        "nanoseconds",
        seconds * f64::from(Nanosecond::per(Second))
    );
    format!("an instant {suffix}")
}

// Generate a random string using web-safe characters
// (This means safe for URL's *and* HTML, without escaping)
pub struct WebsafeAlphabet;

impl Distribution<u8> for WebsafeAlphabet {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        const GEN_ASCII_STR_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                abcdefghijklmnopqrstuvwxyz\
                0123456789\
                $-_+!*,";
        const MAX: usize = 26 + 26 + 10 + 7;
        let range = Lazy::new(|| Uniform::new(0, MAX));

        // SAFETY: guaranteed to be within bounds
        unsafe { *GEN_ASCII_STR_CHARSET.get_unchecked((*range).sample(rng)) }
    }
}

impl DistString for WebsafeAlphabet {
    fn append_string<R: Rng + ?Sized>(&self, rng: &mut R, string: &mut String, len: usize) {
        unsafe {
            let v = string.as_mut_vec();
            v.extend(self.sample_iter(rng).take(len));
        }
    }
}
