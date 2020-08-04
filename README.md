<!-- omit in TOC -->

# NeoX NPK Tool

Tool to play with NPK files as they are used in NetEase's NeoX Engine.

It currently supports the format used in a early 2020 Version of Eve Echoes.
This includes listing of files and decompressing them with LZ4.

Support for Encrypted and ZLib files is planned and will follow soon...

For each file in the NPK the tool will try to determine the mime type and add an appropriate file extension. This is on a best guess basis and the mapping is currently somewhat limited (but handles all files in Eve Echoes mostly correct)

## Coming Soon
* RC4 Encryption Support
* Simple Encryption Support
* Zlib Support
* Re-packing of NPK
* Probably some refactoring and more 'tools' in the future

## Building

All you have to do to build it is clone it an run on of the following:

```
cargo build --release
```
or to install
```
cargo install --path .
```

## Usage

Example:

```
npktool x script.npk out
```

This will extract all the files in script.npk to the out `out/script` directory.

More info on how to use it can be found in the help section.
`npktool --help`
