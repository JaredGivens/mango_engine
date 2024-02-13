set -e  # Exit immediately if a command exits with a non-zero status

# Script dependencies

# Check if dasel is installed, and prompt user to install if not
if ! command -v dasel &> /dev/null; then
    read -p "dasel is required for this script. Do you want to install it? (y/n) " install_dasel
    if [[ $install_dasel == "y" || $install_dasel == "Y" ]]; then
        brew install dasel
    else
        exit 1
    fi
fi

# Check if Xcode Command Line Tools are set up, and prompt user to set up if not
if ! xcode-select -p &> /dev/null; then
    read -p "Xcode Command Line Tools are required for this script. Do you want to set them up? (y/n) " setup_xcode
    if [[ $setup_xcode == "y" || $setup_xcode == "Y" ]]; then
        xcode-select --install
        sudo xcode-select -s /Applications/Xcode.app
        echo "Please restart your terminal after installing Xcode Command Line Tools."
        exit 0
    else
        exit 1
    fi
fi

# Check for cargo bundle
if ! cargo install --list | grep "cargo-bundle" &> /dev/null; then
    echo "This project requires the rust package cargo-bundle"
    read -p "Do you wish to install? (y/n) " install_bundle
    if [[ $install_bundle == "y" || $install_bundle == "Y" ]]; then
        cargo install cargo-bundle
    fi
fi

# Check if iOS targets are installed using rustup
targets=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios")
missing_targets=()
for target in "${targets[@]}"; do
    if ! rustup target list | grep "$target (installed)" &> /dev/null; then
        missing_targets+=("$target")
    fi
done

if [ ${#missing_targets[@]} -ne 0 ]; then
    echo "The following Rust targets are not installed: ${missing_targets[*]}"
    read -p "Do you want to install them? (y/n) " install_targets
    if [[ $install_targets == "y" || $install_targets == "Y" ]]; then
        for target in "${missing_targets[@]}"; do
            rustup target add "$target"
        done
    else
        exit 1
    fi
fi

# End of dependencies checks

# to list available devices
# xcrun simctl list devices
SIM_TARGET="iPhone 15 Pro"

# Cargo.toml parsing
APP_NAME=$(cat Cargo.toml | dasel -r toml '.package.name' | tr -d "'")
BUNDLE_ID=$(cat Cargo.toml | dasel -r toml '.package.metadata.bundle.identifier' | tr -d "'")

# for production
# rustup target add aarch64-apple-ios

# for development
# rustup target add aarch64-apple-ios-sim

# Bundle the app with target for release build
cargo bundle --target aarch64-apple-ios-sim

# Boot the iOS simulator
if ! xcrun simctl list devices | grep "$SIM_TARGET" | grep -q "Booted"; then
    echo "Booting $SIM_TARGET..."
    xcrun simctl boot "$SIM_TARGET"
    sleep 5
fi

# Set iOS sim as target window
open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app

# Install and launch the app on the booted simulator
APP_PATH="target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app"
xcrun simctl install booted "$APP_PATH"
xcrun simctl launch --console booted "$BUNDLE_ID"