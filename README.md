# Rust Vulkan Samples
A collection of cross-platform Vulkan samples written using the Rust programming language.

Currently supports Windows/OSX/Linux and Android.

# Setup

## __Rust Setup__
[Install Rust](https://www.rust-lang.org/tools/install)
<!-- -->
    Chocolatey : choco install rustup --pre

## __Additional requirements for building dependencies__
* Python
* CMake
* Ninja
* Git
<!-- -->
    Chocolatey : choco install python --pre
                 choco install cmake
                 choco install ninja
                 choco install git

<!-- -->
## _Required environment variables_
* CMAKE (set to path of cmake executable)

## __Android Setup__
The following steps are required to build for android
### Prerequisites
Android NDK/SDK:

  * [Standalone NDK](https://developer.android.com/ndk/downloads)

  * [SDK Command line tools or SDK/NDoco install android-sdk Packaged With Android Studio](https://developer.android.com/studio#downloads)

  * After installing SDK run 'sdk manager' from the bin folder and install latest build tools

<!-- -->
    Chocolatey: choco install android-ndk android-sdk
<!-- -->
## _Required environment variables_
  * ANDROID_NDK_ROOT
  * ANDROID_SDK_ROOT
  * ANDROID_HOME (same as ANDROID_SDK_ROOT)

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

     Chocolatey: choco install openjdk
## __Clippy (optional)__
Clippy is a helpful rust linter that can be installed as follows:
<!-- -->
    'rustup component add clippy-preview'
    (use rustup component list to see up to date name)

# __Build Steps__
## Windows/OSX/Linux

    cargo build --example [example_name]
<!-- -->
    cargo run --example [example_name]

## Android
    cargo apk build --example [example_name]
<!-- -->
    cargo apk run --example [example_name]
# Notes
* It is not necesarry to do build and run steps seperately
* 'example_name' can be any one of the root file names inside of root example folder


