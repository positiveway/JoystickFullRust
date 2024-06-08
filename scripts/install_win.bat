cd "$env:userprofile\Documents\"

git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install
.\vcpkg install libusb

cd ..
Invoke-WebRequest https://github.com/pbatard/libwdi/releases/download/v1.5.0/zadig-2.8.exe -OutFile zadig-2.8.exe
