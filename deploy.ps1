cd .\deploy
Remove-Item -Path ".\Acquisition_Install.exe" -Force -ErrorAction SilentlyContinue

cd .\packages\acquisition
Remove-Item -Path .\data -Recurse -Force -ErrorAction SilentlyContinue
New-Item -Path . -Name data -ItemType "directory"
cd .\data

#Get copy acquisition.exe and generate its dependencies
Copy-Item "..\..\..\..\..\build-acquisition-Desktop_Qt_5_15_2_MSVC2019_64bit-Release\release\acquisition.exe" -Destination ".\acquisition.exe"
Copy-Item "C:\Qt\Tools\OpenSSL\Win_x64\bin\libcrypto-1_1-x64.dll" -Destination ".\libcrypto-1_1-x64.dll"
Copy-Item "C:\Qt\Tools\OpenSSL\Win_x64\bin\libssl-1_1-x64.dll" -Destination ".\libssl-1_1-x64.dll"
& 'C:\Qt\5.15.2\msvc2019_64\bin\windeployqt.exe' ".\acquisition.exe" --release --no-compiler-runtime --dir .

cd ..\..\..

& 'C:\Qt\QtIFW-4.2.0\bin\binarycreator.exe' -c .\config\config.xml -p .\packages Acquisition_Install.exe