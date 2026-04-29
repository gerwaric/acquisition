# Building Acquisition

Acquisition can be build on Windows, macOS, and Linux using CMake and Qt 6.

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt Network Authorization

## Windows

Windows releases are currently built with:
- Windows 11
- Qt Creator (Community) with Qt 6.10.x using MSVC 2022 64-bit
- Inno Setup 6.5.4 for installer creation

You can also build Acquisition with Visual Studio 2022 and the Qt Visual Studio Tools extension.

## Linux

These instructions are based on using Ubuntu 22.04 LTS. Not every step is explicit--e.g. you will have to figure out how to use git and/or install packages on your distribution of choice.

You might need to modify some of the deploy scripts and/or CMakeLists.txt depending on which specific version of Qt you have installed and where you've installed it.

1. Start from a working Ubuntu installation.

2. Download and run the Qt Online Installer for open-source use:

https://www.qt.io/download-open-source

Notes:
- You may need to sign up for a Qt account.
- Make sure the following packages are installed:
    - libxcb-cursor0
    - libxcb-cursor-dev
	- libgl-dev
    - libssl-dev
    - libvulkan-dev
    - gcc-13 and g++-13 (for std::expected, which is used by the glaze json library)
    - libcurl4-openssl-dev
    - zlib1g-dev
- Other packages may be required on other distributions.

When runing the installer, choose "Custom Installation". Then, in addition to the default selection, add the following:
- Select "Desktop" under Qt -> Qt 6.10.x
- Select "Qt Network Authorization" under Qt -> Qt 6.10.x -> Additional Libraries

Ubuntu 22.04 LTS comes with OpenSSL 3.x, buf if you are using a distribution that does not, you may need to build and install it yourself:
- Select "OpenSSL 3.x Toolkit" under Qt -> Build Tools
- Build OpenSSL 3.x from the toolkit that Qt installed.
- Make sure OPENSSL_ROOT_DIR is set properly.

3. Open the acquisition project in Qt Creator. If something wasn't configured properly, you'll get an error in this step, which is often difficult to debug. Once you've got the project opened, you should be able to build it.

4. Create the AppImage for deployment.

You will need to make sure you have linuxdeploy:
- `wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage`
- `wget https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage`

**NOTE**: older version of linuxdeploy prior to August of 2025 will not work because were not able to handle changes in Qt 6.6 that broke library packaging for SQL drivers in some circumstances.

Make sure they moved into ~/bin because this location is hard-coded in the deploy script.
Make sure they are executable, e.g. via chmod u+x *.AppImage

Run deploy-linux.sh from within the acquisition project folder.

5. Run Acquisition.

Look for an AppImage file in the deploy/ directory.

## macOS

macOS releases are currently built with:
- macOS Sequoia 15.x on Apple silicon
- Qt Creator with Qt 6.10.x for macOS
- XCode 26.x
