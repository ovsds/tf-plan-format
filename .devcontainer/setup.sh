#!/bin/bash

export DEBIAN_FRONTEND=noninteractive

echo "Installing common tools..."
apt-get update
apt-get install -y \
    curl \
    git


echo "Installing nvm..."
curl https://raw.githubusercontent.com/creationix/nvm/master/install.sh | bash
mkdir -p ~/.config/husky
{
    echo ""
    echo 'export NVM_DIR="$HOME/.nvm"'
    echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm'
    echo '[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion'
} > ~/.config/husky/init.sh


echo "Installing Rust..."
apt-get install -y \
    build-essential

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

/root/.cargo/bin/rustup install nightly
/root/.cargo/bin/rustup component add rustfmt
/root/.cargo/bin/rustup component add rustfmt --toolchain nightly
/root/.cargo/bin/rustup component add clippy
/root/.cargo/bin/rustup component add clippy --toolchain nightly

/root/.cargo/bin/cargo install cargo-llvm-cov
/root/.cargo/bin/rustup component add llvm-tools-preview --toolchain stable-x86_64-unknown-linux-gnu

echo "Installing Taskfile..."
sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b /usr/local/bin
