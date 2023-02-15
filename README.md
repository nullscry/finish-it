# Finish-it

[![Build Status](https://github.com/nullscry/finish-it/actions/workflows/ci.yml/badge.svg)](https://github.com/nullscry/finish-it/actions/workflows/ci.yml)
[![Release Status](https://github.com/nullscry/finish-it/actions/workflows/release.yml/badge.svg)](https://github.com/nullscry/finish-it/releases)
[![Crate Status](https://img.shields.io/crates/v/finish-it.svg)](https://crates.io/crates/finish-it)
[![Docs Status](https://docs.rs/finish-it/badge.svg)](https://docs.rs/crate/finish-it/)

A TUI application to track whatever is that you would actually like to finish after starting.
The project uses tui-rs and and sqlite database.

## Target Plaforms

- aarch64-linux
- x86_64-linux
- x86_64-macos
- x86_64-windows

## Installation

- Using cargo

  - ```sh
    cargo install finish-it
    ```

    will report where the binary is installed. The default location in Linux:

    ```sh
    Installing /home/<youruser>/.cargo/bin/finish-it
    ```

  - Then invoke the binary with

    ```sh
    /home/<youruser>/.cargo/bin/finish-it
    ```

    OR if `/home/<youruser>/.cargo/bin/` is in your PATH

    ```sh
    finish-it
    ```

- Using precompiled binaries

  - Download and install latest version for your architecture from [releases](https://github.com/nullscry/finish-it/releases)
  - Unzip, untar and directly use the binary like:
    ```sh
    finish-it
    ```
  - You might need to give executable permissions to the binary like:
    ```sh
    chmod +x finish-it
    ```

## Usage

Simply

```sh
finish-it
```

## Home Tab

<img src="assets/home_tab.jpg" alt="Screenshot of Home Tab">

## Topics Tab

<img src="assets/topics_tab.jpg" alt="Screenshot of Topics Tab">

## Add Tab

<img src="assets/add_tab.jpg" alt="Screenshot of Add Tab">

## Updating Progress

<img src="assets/update_tab.jpg" alt="Screenshot of Update Popup in Topics Tab">

## Deleting

### Deleting Item

<img src="assets/delete_item.jpg" alt="Screenshot of Delete Item Popup in Topics Tab">

### Deleting Topic

<img src="assets/delete_topic.jpg" alt="Screenshot of Delete Topic Popup in Topics Tab">
