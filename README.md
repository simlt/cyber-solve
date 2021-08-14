# Install requirements

## Windows

``` sh
# Install llvm through chocolatey
choco install llvm

# Then use vcpkg for opencv and its deps
# tessless deps: install vcpkg, then from the vcpkg folder run
cd C:\src\win32\vcpkg


# [optional] To support freetype opencv module https://docs.opencv.org/4.5.3/d4/dfc/group__freetype.html install the following before installing opencv4
.\vcpkg install --triplet=x64-windows-static-md freetype harfbuzz
# then open the file .\vcpkg\ports\opencv4\portfile.cmake, and add a line with the option
# -DWITH_FREETYPE=ON
# inside of vcpkg_cmake_configure OPTION section

# Now, install opencv4
.\vcpkg install --triplet=x64-windows-static-md tesseract opencv4[contrib,nonfree] # freetype 
```

## Debian/Ubuntu

```sh
# OpenCV deps
apt-get install libopencv-dev

# Install tessless deps
apt-get install libleptonica-dev libtesseract-dev clang tesseract-ocr libclang-dev
```

## Mac

```sh
# OpenCV deps
brew install opencv libclang

# Install tessless deps
brew install tesseract leptonica
```

## Extra files

Manually download the following files:

`deps\opencv_world412.dll` (only for dynamic library builds)
`assets\tesseract\eng.traineddata`

NOTE: to run the executable when using dynamic library builds, it is necessary to place a copy of the dll in the same folder of the executable to be run.
