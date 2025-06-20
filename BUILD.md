# Building Acquisition

Acquisition can be build on Windows, macOS, and Linux using CMake and Qt 6.9.

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt HTTP Server
- Qt Network Authorization
- Qt WebSockets

## Windows

On Windows you can also build Acquisition with Visual Studio 2022 and the Qt Visual Studio Tools extension.

Windows releases are currently built with:
- Windows 11
- Qt Creator (Community) with Qt 6.9.x using MSVC 2022 64-bit
- Visual Studio 2022 with Qt Visual Studio Tools 3.x for editing, debugging, and testing
- Inno Setup 6.3.3 for installer creation

## Linux

These instructions are based on using Ubuntu 24 LTS. Not every step is explicit--e.g. you will have to figure out how to use git and/or install packages on your distribution of choice.

You might need to modify some of the deploy scripts and/or CMakeLists.txt depending on which specific version of Qt you have installed and where you've installed it.

1. Start from a working Ubuntu installation.

2. Download and run the Qt Online Installer for open-source use:

https://www.qt.io/download-open-source

Notes:
- You may need to sign up for a Qt account.
- Make sure the following packages are installed:
    - xcb-cursor0
    - libxcb-cursor0
	- libgl-dev
    - libssl-dev
    - libvulkan-dev
- Other packages may be required on other distributions.

When runing the installer, choose "Custom Installation". Then, in addition to the default selection, add the following:
- Select "Desktop" under Qt -> Qt 6.9.x
- Select "Qt Http Server" under Qt -> Qt 6.9.x -> Additional Libraries
- Select "Qt Network Authorization" under Qt -> Qt 6.9.x -> Additional Libraries
- Select "Qt WebSockets" under Qt -> Qt 6.9.x -> Additional Libraries

If you are using a distribution that does not come with OpenSSL 3.x, you'll need to build and install it yourself:
- Select "OpenSSL 3.0.x Toolkit" under Qt -> Build Tools
- Build OpenSSL 3.x from the toolkit that Qt installed.
- Make sure OPENSSL_ROOT_DIR is set properly.

3. Open the acquisition project in Qt Creator. If something wasn't configured properly, you'll get an error in this step, which is often difficult to debug. Once you've got the project opened, you should be able to build it.

4. Create the AppImage for deployment.

You will need to download special forks of linuxdeploy and the qt plugin:
- https://github.com/dantti/linuxdeploy
- https://github.com/dantti/linuxdeploy-plugin-qt
(This is because of an SQL-related change to Qt 6.6 that means certain librarries
have to be excluded from packaging, and the above fork adds that feature).

Download the x86_65 AppImages for linuxdeploy and linuxdeploy-plugin-qt.
Make sure they moved into ~/bin because this location is hard-coded in the deploy script.
Make sure they are executable, e.g. via chmod u+x

Run deploy-linux.sh from within the acquisition project folder.

5. Run Acquisition.

Look for an AppImage file in the deploy/ directory.

## macOS

macOS releases are currently built with:
- macOS Sequoia 15.x on Apple silicon
- Qt Creator with Qt 6.9 for macOS
- XCode 16.x
