#!/bin/sh

used_vars=$(grep -Eho -- '--[a-z0-9-]+' "$@" | sort -u)

echo ':root {'

for var in $used_vars; do
    grep -- "$var:" crates/frontend/assets/css/vars.css
done

echo '}'
