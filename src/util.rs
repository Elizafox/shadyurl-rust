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

pub(crate) mod bits {
    use num::PrimInt;

    pub(crate) fn count_bits<T: PrimInt>(mut n: T) -> u32 {
        let mut count = 0;
        while n != T::zero() {
            count += 1;
            n = n >> 1;
        }
        count
    }
}

pub(crate) mod format {
    use time::{
        convert::{Day, Hour, Microsecond, Millisecond, Minute, Nanosecond, Second, Week},
        Duration,
    };

    use super::math::is_close;

    // This implementation is heavily modified from the time crate.
    pub(crate) fn humanize_duration(duration: Duration) -> String {
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
                if is_close(value.round(), 1.0) {
                    return format!("{} {suffix}", $singular);
                } else if value > 1.0 {
                    return format!("{value:.0} {} {suffix}", $plural);
                }
            };
        }

        const AVERAGE_YEAR: f64 = 365.2425;
        const AVERAGE_MONTH: f64 = AVERAGE_YEAR / 12.0;

        // Even if this produces a de-normal float, because we're rounding we don't really care.
        let seconds = duration.unsigned_abs().as_secs_f64();

        item!(
            "a year",
            "years",
            seconds / (Second::per(Day) as f64 * AVERAGE_YEAR)
        );
        item!(
            "a month",
            "months",
            seconds / (Second::per(Day) as f64 * AVERAGE_MONTH)
        );
        item!("a week", "weeks", seconds / Second::per(Week) as f64);
        item!("a day", "days", seconds / Second::per(Day) as f64);
        item!("an hour", "hours", seconds / Second::per(Hour) as f64);
        item!("a minute", "minutes", seconds / Second::per(Minute) as f64);
        item!("a second", "seconds", seconds);
        item!(
            "a millisecond",
            "milliseconds",
            seconds * Millisecond::per(Second) as f64
        );
        item!(
            "a microsecond",
            "microseconds",
            seconds * Microsecond::per(Second) as f64
        );
        item!(
            "a nanosecond",
            "nanoseconds",
            seconds * Nanosecond::per(Second) as f64
        );
        format!("an instant {suffix}")
    }
}

pub(crate) mod macros {
    #[macro_export]
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

pub(crate) mod math {
    use num::Float;

    pub(crate) fn is_close<T: Float>(a: T, b: T) -> bool {
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
}

pub(crate) mod net {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use ipnetwork::{IpNetwork, IpNetworkError};

    use super::bits::count_bits;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum NetworkPrefixError {
        #[error(transparent)]
        IpNetwork(#[from] IpNetworkError),

        #[error("IP address types are mismatched")]
        IpTypeMismatch,
    }

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum AddressError {
        #[error("Incorrect size: {}", .0)]
        IncorrectSize(usize),
    }

    pub(crate) fn vec_to_ipaddr(addr: Vec<u8>) -> Result<IpAddr, AddressError> {
        let addr = match addr.len() {
            4 => IpAddr::from(TryInto::<[u8; 4]>::try_into(addr).unwrap()),
            16 => IpAddr::from(TryInto::<[u8; 16]>::try_into(addr).unwrap()),
            _ => return Err(AddressError::IncorrectSize(addr.len())),
        };

        Ok(addr.to_canonical())
    }

    pub(crate) fn find_networks(
        start: IpAddr,
        end: IpAddr,
    ) -> Result<Vec<IpNetwork>, NetworkPrefixError> {
        let res = match (start, end) {
            (IpAddr::V4(start_ip), IpAddr::V4(end_ip)) => {
                let mut start_int: u32 = start_ip.into();
                let end_int: u32 = end_ip.into();

                let mut res = Vec::new();

                while start_int <= end_int {
                    // SAFETY: safe cast, we can never have > 255
                    let nbits = start_int
                        .trailing_zeros()
                        .min(count_bits(end_int - start_int + 1) - 1)
                        as u8;
                    res.push(IpNetwork::new(
                        IpAddr::V4(Ipv4Addr::from(start_int)),
                        32 - nbits,
                    )?);
                    start_int += 1 << nbits;
                    if start_int - 1 == u32::MAX {
                        break;
                    }
                }

                res
            }
            (IpAddr::V6(start_ip), IpAddr::V6(end_ip)) => {
                let mut start_int: u128 = start_ip.into();
                let end_int: u128 = end_ip.into();

                let mut res = Vec::new();

                while start_int <= end_int {
                    // SAFETY: safe cast, we can never have > 255
                    let nbits = start_int
                        .trailing_zeros()
                        .min(count_bits(end_int - start_int + 1) - 1)
                        as u8;
                    res.push(IpNetwork::new(
                        IpAddr::V6(Ipv6Addr::from(start_int)),
                        128 - nbits,
                    )?);
                    start_int += 1 << nbits;
                    if start_int - 1 == u128::MAX {
                        break;
                    }
                }

                res
            }
            _ => return Err(NetworkPrefixError::IpTypeMismatch),
        };

        Ok(res)
    }
}
