#!/usr/bin/env nix-shell
#! nix-shell -i bash -p bash

# Enable bash strict mode
set -euo pipefail
IFS=$'\n\t'

sudo mount -t tmpfs -o size=4G /dev/null /dev/shm
sleep 2
sudo nohup dockerd --bip 172.18.0.1/16 </dev/null >/dev/null 2>&1 &
