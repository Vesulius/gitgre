#!/bin/bash

git clone https://github.com/Vesulius/gitgre.git
cd gitgre
cargo build --release
mkdir ~/.local/bin 2>/dev/null
strip ./target/release/gitgre
cp ./target/release/gitgre ~/.local/bin/

if [ -f "$HOME/.bashrc" ]; then
    echo "" >> ~/.bashrc
    echo "alias gg=\"~/.local/bin/gitgre\"" >> ~/.bashrc
    source ~/.bashrc
elif [ -f "$HOME/.zshrc" ]; then
    echo "" >> ~/.zshrc
    echo "alias gg=\"~/.local/bin/gitgre\"" >> ~/.zshrc
    source ~/.zshrc
else
    echo "Could not find .bashrc or .zshrc file in the user's home directory."
fi

cd ..
rm -fr gitgre
rm ./install.sh
