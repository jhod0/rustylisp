#!/usr/bin/env bash
find . -name '*.rs' | grep -v '.*tests\.rs' | xargs nl | tail -n 1 | awk ' { print $1 } '
