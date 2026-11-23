#!/usr/bin/env bash
# Matuwall's hooks find octopus-theme on PATH.
export PATH="$HOME/.local/bin:$PATH"
pkill -x matuwall && exit 0
exec matuwall
