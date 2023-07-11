#!/bin/sh

cleanup()
{
	stty "$1"
}

bail()
{
	echo "$1"
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
	saltlen="$(gen_randint "8" "16" || bail "Could not get random int")"
	(tr -dc '[:alnum:]' </dev/random | head -c "$saltlen") || bail "Could not generate salt"
}

gen_passwd()
{
	# Needed on macOS
	export LC_ALL="C"

	which -s argon2 || bail "argon2 is not installed; install it via your OS's libargon2 or argon2 package"

	orig_stty="$(stty -g)"

	trap exit INT HUP QUIT TERM
	trap 'cleanup "$orig_stty"' EXIT

	stty -echo

	printf "Password: "
	read pw1 || bail "Could not read password"

	printf "\nRetype Password: "
	read pw2 || bail "Could not read password"

	echo

	[ "$pw1" == "$pw2" ] || bail "Passwords didn't match"

	salt="$(gen_salt || bail "Could not generate salt")"

	(printf "$pw1" | argon2 "$salt" -id -e -t 12 -m 16 -v 13) || bail "Could not generate argon2 hash"
}
