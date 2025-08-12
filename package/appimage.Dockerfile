
# Fedora-based Dockerfile for building SamRewritten AppImage
# ----------------------------------------------------------
# Heavily inspired by:
# https://github.com/13hannes11/gtk4-rs-docker/blob/main/appimage/Dockerfile

FROM fedora:36

# Build arguments and environment
ARG RUST_VERSION=stable
ENV RUST_VERSION=${RUST_VERSION}
ENV APPIMAGE_VERSION=continuous
ENV APPIMAGE_EXTRACT_AND_RUN=1
ENV PATH=/root/.cargo/bin:$PATH

# Install build dependencies and clean up
RUN dnf install -y \
		gtk4-devel \
		gcc \
		libadwaita-devel \
		openssl-devel \
		wget \
		file \
		desktop-file-utils \
		appstream \
		squashfs-tools \
	&& dnf clean all

# Install Rust toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& . /root/.cargo/env \
	&& rustup install ${RUST_VERSION}

# Install cargo-appimage (custom fork)
RUN . /root/.cargo/env && cargo install --git https://github.com/PaulCombal/cargo-appimage.git

# Download and extract AppImage tool
RUN wget https://github.com/AppImage/appimagetool/releases/download/${APPIMAGE_VERSION}/appimagetool-x86_64.AppImage \
	&& chmod +x appimagetool-x86_64.AppImage \
	&& ./appimagetool-x86_64.AppImage --appimage-extract \
	&& ln -nfs /squashfs-root/usr/bin/appimagetool /usr/bin/appimagetool

# Set working directory
WORKDIR /mnt

CMD ["/bin/bash"]