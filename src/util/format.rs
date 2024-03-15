/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util/format.rs
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

use time::{
    convert::{Day, Hour, Microsecond, Millisecond, Minute, Nanosecond, Second, Week},
    Duration,
};

use super::math::FloatMathUtil;

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
