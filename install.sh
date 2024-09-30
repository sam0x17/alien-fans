#!/bin/sh
set -e
cargo build --release
sudo cp target/release/alien-fans /usr/bin/
sudo cp alien-fans.service /etc/systemd/system/
sudo systemctl enable --now alien-fans
sudo systemctl status alien-fans
