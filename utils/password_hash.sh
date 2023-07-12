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
	range0="$1"
	range1="$2"
	i=""
	until [ -n "$i" ] && [ "$i" -gt "$range0" -a "$i" -lt "$range1" ]
	do
		i="$(dd if=/dev/random bs=1 count=1 status=none | od -tu1 -An | xargs)"
	done
	echo "$i"
}

gen_salt()
{
	saltlen="$(gen_randint "12" "24" || bail "Could not get random int")"
	(tr -dc '[:print:]' </dev/random | head -c "$saltlen") || bail "Could not generate salt"
}

gen_passwd()
{
	# Needed on macOS
	export LC_ALL="C"

	which -s argon2 || bail "argon2 is not installed; install it via your OS's libargon2 or argon2 package"

	if [ -t 0 ]; then
		tty="$(tty)"
		orig_stty="$(stty -g)"

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

		salt="$(gen_salt || bail "Could not generate salt")"
	else
		passwd="$1"
	fi

	(printf "$passwd" | argon2 "$salt" -id -e -t 12 -m 16 -v 13) || bail "Could not generate argon2 hash"
}

passwd="$(gen_passwd || bail "Could not generate password")"
printf "\nPASSWORD_HASH='%s'\n" "${passwd}"
