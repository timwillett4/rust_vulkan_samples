# Rust Vulkan Samples
A collection of cross-platform Vulkan samples written using the Rust programming language

# Setup

## __Rust Setup__
[Install Rust](https://www.rust-lang.org/tools/install)
<!-- -->
    Chocolatey : choco install rustup --pre

## __Android Setup__
The following steps are required to build for android
### Prerequisites
* Android NDK:

  * [Standalone NDK](https://developer.android.com/ndk/downloads)

  * [Packaged With Android Studio](https://developer.android.com/studio#downloads)

    chocolatey: choco install android-ndk

### Add Android Targets

    'rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android'
 <!-- -->
 ### Update Cargo Config
 __NOTE:__ As far as I can tell environment variables are not supported so you will need to hardcode your NDK path.  If you find otherwise please let me know so I can update accordingly.
<!-- -->
Add the following to ~/.cargo/config (add file if it doesn't exist):
<!-- -->
    [target.aarch64-linux-android]
    ar = "[ANDROID_NDK_HOME]/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android-ar"
    linker = "[ANDROID_NDK_HOME]/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android30-clang.cmd"

    [target.armv7-linux-androideabi]
    ar = "[ANDROID_NDK_HOME]/toolchains/llvm/prebuilt/windows-x86_64/bin/arm-linux-androideabi-ar"
    linker = "[ANDROID_NDK_HOME]/ndk-bundle/toolchains/llvm/prebuilt/windows-x86_64/bin/armv7a-linux-androideabi30-clang.cmd"

    [target.i686-linux-android]
    ar = "[ANDROID_NDK_HOME]/toolchains/llvm/prebuilt/windows-x86_64/bin/i686-linux-android-ar"
    linker = "[ANDROID_NDK_HOME]/toolchains/llvm/prebuilt/windows-x86_64/bin/i686-linux-android30-clang.cmd"

### Cargo APK Builder
Cargo APK builder must be installed to build android apks:

    'cargo install cargo-apk'

### JDK
If you get errors when signing package you likely need to update jdk to latest version:

[JDK Installation](https://www.oracle.com/sa/java/technologies/javase-downloads.html)

     chocolatey: choco install openjdk
## __Clippy (optional)__
Clippy is a helpful rust liner that can be installed as follows:
<!-- -->
    'rustup add component clippy-preview'
    (use rustup component list to see up to date name)

# __Build Steps__
#

    cargo build --example [example_name]
<!-- -->
    cargo run --example [example_name]
#
## Android
    cargo apk build --example [example_name]
<!-- -->
    cargo apk run --example [example_name]
# Notes
* It is not necesarry to do build and run steps seperately
* 'example_name' can be any one of the sub_directories inside of root example folder


