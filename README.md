# Install requirements

## Windows

``` sh
# Opencv deps through chocolatey
choco install llvm # opencv 

# OR just use vcpkg for opencv
# tessless deps: install vcpkg, then from the vcpkg folder run
cd C:\src\win32\vcpkg
# lz4 is a dep of opencv4[contrib]
.\vcpkg install --triplet=x64-windows-static-md lz4 tesseract opencv4 # opencv4[contrib,nonfree] freetype 
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

`deps\opencv_world412.dll`
`assets\tesseract\eng.traineddata`

NOTE: To run the executable, it is necessary to place a copy of the dll in the same folder of the executable to be run.
