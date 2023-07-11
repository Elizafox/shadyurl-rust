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

echo "$(LC_ALL=C tr -dc '[:alnum:]+_<>,.-!@%^&*()[]{}\|:;?/`' </dev/urandom | head -c 64)"
