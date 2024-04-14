#! /bin/sh
sudo apt update && sudo apt upgrade
sudo apt install build-essential git libfprint-2-2 libfprint-2-dev libfprint-2-doc biometric-driver-community-multidevice
curl https://sh.rustup.rs -sSf | sh
sudo apt install librust-gobject-sys-dev librust-gio-sys-dev librust-glib-sys-dev pkg-config cmake libssh-dev librust-clang-sys-dev
#for tauri
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
cargo install create-tauri-app --locked
