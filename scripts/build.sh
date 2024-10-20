build_clean=false
update_rust=false
use_nightly=false
use_polonius=false
new_trait_solver=false
use_another_linker=false

# exit when any command fails
set -e

# to request sudo in the beginning
sudo uname -r

rust_version="stable"
if $use_nightly
then
  rust_version="nightly"
fi

cd ..

if $build_clean
then
rm -f ./Cargo.lock
rm -rf ./target
cargo clean
fi

build_flags="-C target-cpu=native"

if $use_polonius
then
  rust_version="nightly"
  build_flags="-Z polonius $build_flags"
fi

if $new_trait_solver
then
  rust_version="nightly"
  build_flags="-Z next-solver=coherence $build_flags"
fi

if $use_another_linker
then
  build_flags="-C link-arg=-fuse-ld=lld $build_flags"
fi

if $update_rust
then
rustup install $rust_version
rustup update
fi

echo $build_flags

# release build
cargo +$rust_version rustc --release -- $build_flags

# polonius debug build
#cargo +nightly rustc -- -Z polonius

# polonius release build
#cargo +nightly rustc --release -- -Z polonius

#These apply it for all dependencies
#export RUSTFLAGS="$build_flags"
#CARGO_BUILD_RUSTFLAGS="-Z polonius" cargo build --release

cd ./scripts
chmod +x ./run.sh
./run.sh
