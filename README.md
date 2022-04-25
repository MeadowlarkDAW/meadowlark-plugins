# rusty-daw-plugins
A (WIP) suite of high quality audio plugins for use with the RustyDAW project

Please take a look at the [`DSP Design Document`] for more details if you are interested in contributing!

<hr/>

[`DSP Design Document`]: https://github.com/MeadowlarkDAW/Meadowlark/blob/main/DSP_DESIGN_DOC.md

## How To Build

Make sure you have the `nightly` version of rust installed.

Note, if you are developing plugins in this repo, you can set this directory to use the nightly compiler by default by running this command in the root directory of this repo:
```
rustup override add nightly
```

To build the debug/testing version of a plugin, run this command (replacing `example-gain-baseplug-nogui` with whatever plugin you wish to build):
```
cargo +nightly build --package example-gain-baseplug-nogui
```

You can build an optimized release version of the plugin using:
```
cargo +nightly build --package example-gain-baseplug-nogui
```

Some plugins have a feature flag you can use to enable SIMD optimizations (recommended if you are not developing or testing the plugin).
```
cargo +nightly build --package example-gain-baseplug-nogui --release --features example-gain-baseplug-nogui/optimized_simd
```

### Installing

- Linux
    - Copy the resulting `.so` file to the `~./vst` directory (replace `release` with `debug` if you built the plugin in debug mode):
    ```
    cp ./target/release/libexample_gain_baseplug_nogui.so ~/.vst/
    ```
- Windows
    - Copy the resulting `.dll` file in this directory to the VST directory of your system:
    ```
    ./target/release/libexample_gain_baseplug_nogui.dll
    ```
- MacOS
    - First make sure the provided script has the permissions to run (you only need to do this once):
    ```
    sudo chmod +x osx_vst_bundler.sh
    ```
    - Next, run the provided script (replace `release` with `debug` if you built the plugin in debug mode):
    ```
    sudo sh osx_vst_bundler.sh ExampleGain ./target/release/libexample_gain_baseplug_nogui.dylib
    ```