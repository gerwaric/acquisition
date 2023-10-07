set WINDEPLOY="C:\Qt\5.15.2\msvc2019_64\bin\windeployqt.exe"
set INNO="C:\Program Files (x86)\Inno Setup 6\Compil32.exe"

rmdir /S /Q .\deploy

mkdir .\deploy\packages\acquisition\data

cd .\deploy\packages\acquisition\data

copy ..\..\..\..\..\build-acquisition-Desktop_Qt_5_15_2_MSVC2019_64bit-Release\release\acquisition.exe .
copy "C:\Program Files\OpenSSL-Win64\\bin\libcrypto-1_1-x64.dll" .
copy "C:\Program Files\OpenSSL-Win64\bin\libssl-1_1-x64.dll" .

%WINDEPLOY% .\acquisition.exe --release --no-compiler-runtime --dir .

cd ..\..\..\..

%INNO% /cc installer.iss