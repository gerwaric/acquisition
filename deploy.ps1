cd .\deploy
Remove-Item -Path ".\Acquisition_Install.exe" -Force -ErrorAction SilentlyContinue

cd .\packages\acquisition
Remove-Item -Path .\data -Recurse -Force -ErrorAction SilentlyContinue
New-Item -Path . -Name data -ItemType "directory"
cd .\data

#Get copy acquisition.exe and generate its dependencies
Copy-Item "..\..\..\..\..\build-acquisition-Desktop_x86_windows_msvc2019_pe_64bit-Release\release\acquisition.exe" -Destination ".\acquisition.exe"
& 'C:\Qt\5.15.0\msvc2019_64\bin\windeployqt.exe' ".\acquisition.exe" --release --no-compiler-runtime --dir .

cd ..\..\..

& 'C:\Qt\QtIFW-3.2.2\bin\binarycreator.exe' -c .\config\config.xml -p .\packages Acquisition_Install.exe