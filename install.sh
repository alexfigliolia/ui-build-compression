set -e 

REPO_ROOT=$(git rev-parse --show-toplevel)

cd $REPO_ROOT

command_exists() {
    command -v "$1"
}

if command_exists rustc && command_exists cargo; then
    echo "Rust is installed."
else
    echo "Installing rust"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi

echo "Installing UI Build Compression"

cargo install ui-build-compression