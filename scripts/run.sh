project_name="JoystickFullRust"

# exit when any command fails
set -e

# Show input devices info
#cat /proc/bus/input/devices

cd ../target/release/
sudo nice -n -20 ./$project_name
