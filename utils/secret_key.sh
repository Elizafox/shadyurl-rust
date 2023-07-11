#!/bin/sh

echo "$(LC_ALL=C tr -dc '[:alnum:]+_<>,.-!@%^&*()[]{}\|:;?/`' </dev/urandom | head -c 64)"
