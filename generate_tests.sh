#!/bin/sh

test_scenarios="stable nightly"

for file in $(find tests/templates -type f -name "*.rs"); do
    for scenario in $test_scenarios; do
        if (echo "$file" | grep -E '^tests/templates/.*-only/' >/dev/null) && ! (echo "$file" | grep -E "^tests/templates/$scenario-only/" >/dev/null); then
            continue
        fi
        target=$(echo "$file" | sed "s/^tests\/templates/tests\/$scenario/g")
        mkdir -p $(dirname "$target") && cp "$file" "$target"
    done
done
