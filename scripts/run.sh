#!/usr/bin/env bash

# exit when any command fails
set -e

project_name="JoystickFullRust"


THIS_SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Show input devices info
#cat /proc/bus/input/devices

cd "$THIS_SCRIPT_DIR"/../target/release/
sudo nice -n -20 ./$project_name
