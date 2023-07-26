#!/bin/sh
# SPDX-License-Identifier: CC0-1.0
#
# utils/password_hash.sh
#
# This file is a component of ShadyURL by Elizabeth Myers.
#
# To the extent possible under law, the person who associated CC0 with
# ShadyURL has waived all copyright and related or neighboring rights
# to ShadyURL.
#
# You should have received a copy of the CC0 legalcode along with this
# work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

usage()
{
	echo "Usage: $0 [-t N] [-m N] [-p N] [-h]" >&2
}

cleanup()
{
	stty "$1"
}

bail()
{
	echo "$1" >&2
	exit 1
}

gen_randint()
{
	# Needed on macOS
	export LC_ALL="C"

	range0="$1"
	range1="$2"
	i=""
	until [ -n "$i" ] && [ "$i" -gt "$range0" -a "$i" -lt "$range1" ]
	do
		# status=none isn't portable, so do this
		i="$(dd if=/dev/random bs=1 count=1 2>/dev/null | od -tu1 -An | xargs)"
	done
	echo "$i"
}

gen_salt()
{
	# Needed on macOS
	export LC_ALL="C"

	saltlen="$(gen_randint "12" "24")" || bail "Could not get random int"
	(tr -dc '[:print:]' </dev/random | head -c "$saltlen") || bail "Could not generate salt"
}

gen_hash()
{
	# Needed on macOS
	export LC_ALL="C"

	ITERATIONS="$1"
	MEMORY="$2"
	PARALLELISM="$3"

	which -s argon2 || bail "argon2 is not installed; install it via your OS's libargon2 or argon2 package"

	if [ -t 0 ]; then
		tty="$(tty)" || bail "Could not get tty"
		orig_stty="$(stty -g)" || bail "Could not get tty parameters"

		trap exit INT HUP QUIT TERM;
		trap 'cleanup "$orig_stty"' EXIT;

		printf "Password: " >$tty
		stty -echo
		read pw1 || bail "Could not read password"
		stty "$orig_stty"

		printf "\nRetype Password: " >$tty
		stty -echo
		read pw2 || bail "Could not read password"
		stty "$orig_stty"
		echo >$tty

		[ "$pw1" == "$pw2" ] || bail "Passwords didn't match"
		passwd="$pw1"

		salt="$(gen_salt)" || bail "Could not generate salt"
	else
		passwd="$1"
	fi

	(printf "$passwd" | argon2 "$salt" -id -e -m "$MEMORY" -t "$ITERATIONS" -p "$PARALLELISM" -v 13) || bail "Could not generate argon2 hash"
}

# Defaults
ITERATIONS=12
MEMORY=16
PARALLELISM=4

args=`getopt t:m:p:h $*`
if [ $? -ne 0 ]; then
	usage "$0"
	exit 2
fi
set -- $args
while :; do
	case "$1" in
	-t)
		printf "%d" "$2" &>/dev/null || bail "-t must be an integer"
		ITERATIONS="$2"
		shift; shift
		;;
	-m)
		printf "%d" "$2" &>/dev/null || bail "-m must be an integer"
		MEMORY="$2"
		shift; shift
		;;
	-p)
		printf "%d" "$2" &>/dev/null || bail "-p must be an integer"
		PARALLELISM="$2"
		shift; shift
		;;
	-h)
		usage "$0"
		exit
		;;
	--)
		shift; break
		;;
	esac
done

passwd="$(gen_hash "$ITERATIONS" "$MEMORY" "$PARALLELISM")" || bail "Could not generate password"
printf "\nPASSWORD_HASH='%s'\n" "${passwd}"
