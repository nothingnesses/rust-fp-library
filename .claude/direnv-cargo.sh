#!/usr/bin/env bash
direnv allow && eval "$(direnv export bash)" && cargo "$@"
