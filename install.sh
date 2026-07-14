#!/bin/bash
set -e

echo -e "\e[36m🐾 Installing Tomodachi...\e[0m"

if ! command -v cargo &> /dev/null; then
    echo -e "\e[31mCargo not found! Please install Rust (rustup.rs) first.\e[0m"
    exit 1
fi

echo -e "\e[33mBuilding Tomodachi (release mode)...\e[0m"
cargo build --release

echo -e "\e[33mInstalling shell hooks and startup scripts...\e[0m"
./target/release/tomodachi-client.exe install

echo -e "\e[33mStarting Tomodachi daemon...\e[0m"
./target/release/tomodachi-daemon.exe & disown

echo -e "\e[32mDone! Tomodachi is now running in your system tray.\e[0m"
echo -e "\e[36mPlease restart your terminal to activate the shell hooks.\e[0m"
