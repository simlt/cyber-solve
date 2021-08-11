# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.183.0/containers/rust/.devcontainer/base.Dockerfile

# FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1
FROM rust:1-bullseye

# Setup oh-my-zsh, default powerline10k theme, no plugins installed
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v1.1.1/zsh-in-docker.sh)" && \
    chsh -s $(which zsh) 

# Install custom packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    less

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # OpenCV
    libopencv-dev \
    # Leptonica
    libleptonica-dev libtesseract-dev clang tesseract-ocr libclang-dev
