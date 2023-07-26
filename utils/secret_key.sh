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

# status=none isn't POSIX, probably doesn't matter but let's keep it portable
dd if=/dev/random bs=66 count=1 2>/dev/null | base64
