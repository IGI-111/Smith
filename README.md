<h1 align="center">
  <a href="https://github.com/IGI-111/Smith">
  <img src="img/smith.png" alt="Smith" width="378" height="175"/>
  </a>
</h1>

<a href="https://crates.io/crates/smith"><img src="https://img.shields.io/crates/v/smith.svg" alt="Crate status"/></a>
<a href="https://travis-ci.org/IGI-111/Smith"><img src="https://travis-ci.org/IGI-111/Smith.svg?branch=master" alt="Build status"/></a>

Smith is a simple terminal-based text editor written in Rust.

## Install

Using Cargo:
```
cargo install smith
```

To compile Smith with clipboard support on Ubuntu, you may need to install some libraries:
```
sudo apt-get install -qq xorg-dev libxcb-render-util0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```


## Features

* line numbers
* syntax highlighting
* undo/redo
* standard keybindings (Ctrl-S, Ctrl-Z, Ctrl-C, Esc...)
* mouse support
* clipboard support

With more planned such as user configurations, search & replace, persistent undo, etc.

Here's what it looks like editing its own source code:

<h2 align="center">
  <img  src="img/screenshot.png" alt="Smith in action"/>
</h2>
