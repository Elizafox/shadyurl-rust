#!/bin/sh
# SPDX-License-Identifier: CC0-1.0
#
# utils/secret_key.sh
#
# This file is a component of ShadyURL by Elizabeth Myers.
#
# To the extent possible under law, the person who associated CC0 with
# ShadyURL has waived all copyright and related or neighboring rights
# to ShadyURL.
#
# You should have received a copy of the CC0 legalcode along with this
# work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

# Attempt to find a base64 utility
for u in "base64" "b64encode" "uuencode" "openssl"; do
	if [ -n "$(which "$u")" ]; then
		UTIL="$u"
		break
	fi
done

case "$UTIL" in
base64)
	UTIL_FLAGS=""
	;;
b64encode)
	UTIL_FLAGS="-r -"
	;;
uuencode)
	UTIL_FLAGS="-m -r -"
	;;
openssl)
	UTIL_FLAGS="base64"
	;;
*)
	echo "No base64 utility found. Please install base64, b64encode, uuencode, or openssl." >&2
	exit 1
	;;
esac

# status=none isn't POSIX, probably doesn't matter but let's keep it portable
echo "$(dd if=/dev/random bs=66 count=1 2>/dev/null | $UTIL $UTIL_FLAGS | tr -d '\n ')"
