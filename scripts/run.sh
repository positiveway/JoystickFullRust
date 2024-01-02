project_name="JoystickFullRust"

# exit when any command fails
set -e

cd ../target/release/
sudo nice -n -20 ./$project_name
