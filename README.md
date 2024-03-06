# GITGRE

Simple tui to switch branches with fuzzyfind.

![gitgre_demo](https://github.com/Vesulius/gitgre/assets/67437973/67ed9c3b-22a0-45c1-84a8-381206297c08)

## Install

This builds the project locally - **requires rust compiler**

```
curl -O https://raw.githubusercontent.com/Vesulius/gitgre/master/install.sh && bash ./install.sh
```

You can now switch branches with **gg** command

## Remove



Run
```
rm ~/.local/bin/gitgre
```

## Why the name

For long time I had simple bash command that used git + grep to switch branches. It's quite useful:

```
gitgre() {git checkout $(git branch | grep -m 1 --ignore-case $1)}
```
