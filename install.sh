#!/bin/bash

install_path="$HOME/.local/bin/"

echo -e "\nInstalling GITGRE\n"
echo "Default install path is '$install_path'"
echo -e "\nSet custom install path? (y/N)"

read -r response

if [[ "$response" == "y" ]]; then
    read -r -p "Enter custom install path: " custom_path
    install_path=$(eval echo "$custom_path")
    echo -e "\nCustom install path is set to '$install_path'. Is this ok? (y/n)"
    read -r response
    if [[ "$response" != "y" ]]; then
        echo "Stopping installation"
        exit 1
    fi
else
    echo "Using default install path: '$install_path'."
    mkdir -p ~/.local/bin 2>/dev/null
fi

git clone https://github.com/Vesulius/gitgre.git
cd gitgre
cargo build --release
strip ./target/release/gitgre
cp ./target/release/gitgre "$install_path/"

if [ -f "$HOME/.bashrc" ]; then
    echo "" >> ~/.bashrc
    echo "alias gg=\"$install_path/gitgre\"" >> ~/.bashrc
    source "$HOME/.bashrc"
elif [ -f "$HOME/.zshrc" ]; then
    echo "" >> ~/.zshrc
    echo "alias gg=\"$install_path/gitgre\"" >> ~/.zshrc
    source "$HOME/.zshrc"
else
    echo "Could not find .bashrc or .zshrc file in the user's home directory."
fi

cd ..
rm -fr gitgre
rm ./install.sh 2>/dev/null

echo -e "\nInstallation complete!\n"