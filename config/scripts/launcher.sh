#!/usr/bin/env bash
pkill -x fuzzel && exit 0
exec fuzzel --placeholder "buscar apps"
