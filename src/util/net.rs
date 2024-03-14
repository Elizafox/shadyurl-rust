/* SPDX-License-Identifier: CC0-1.0
 *
 * src/util/net.rs
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

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::{IpNetwork, IpNetworkError};

use super::bits::count_trailing_zeroes;

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

pub(crate) fn find_networks(start: IpAddr, end: IpAddr) -> Result<Vec<IpNetwork>, NetworkPrefixError> {
    let res = match (start, end) {
        (IpAddr::V4(start_ip), IpAddr::V4(end_ip)) => {
            let mut start_int: u32 = start_ip.into();
            let end_int: u32 = end_ip.into();

            let mut res = Vec::new();

            while start_int <= end_int {
                // SAFETY: safe cast, we can never have > 255
                #[allow(clippy::cast_possible_truncation)]
                let nbits = start_int
                    .trailing_zeros()
                    .min(count_trailing_zeroes(end_int - start_int + 1) - 1)
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
                #[allow(clippy::cast_possible_truncation)]
                let nbits = start_int
                    .trailing_zeros()
                    .min(count_trailing_zeroes(end_int - start_int + 1) - 1)
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
