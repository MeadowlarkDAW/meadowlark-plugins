This is a simple gain plugin to demonstrate how to develop and test RustyDAW DSP as a VST plugin.

## Build & Install Instructions

You will need nighly rust:
```
rustup toolchain install nightly
```

Next, cd into the `rusty-daw-plugins` folder and run this command to set nightly as the default toolchain for this workspace:
```
rustup override add nightly
```

Now you can build the plugin with:
```
cargo build --package example-gain-plugin
```

Finally, either set your plugin host to scan the `target` directory, or copy the resulting `libexample_gain_plugin.so` (Linux) or `libexample_gain_plugin.dll` (Windows) file into you VST plugin folder. Mac users will need to follow the steps described at the bottom of the [`vst-rs readme`].

[`vst-rs readme`]: https://github.com/RustAudio/vst-rs
