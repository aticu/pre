#!/bin/sh

test_scenarios="stable nightly"

for file in $(find tests/templates -type f -name "*.rs"); do
    for scenario in $test_scenarios; do
        target=$(echo "$file" | sed "s/^tests\/templates/tests\/$scenario/g")
        mkdir -p $(dirname "$target") && cp "$file" "$target"
    done
done
