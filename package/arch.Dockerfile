
# ArchLinux Dockerfile for building SamRewritten packages
# ------------------------------------------------------
# This container provides a safe, reproducible environment for building Arch packages.

FROM archlinux:latest

# Install build dependencies and sudo
RUN pacman -Syu --noconfirm --needed \
    base-devel \
    git \
    rust \
    gtk4 \
    libadwaita \
    sudo

# Create a non-root user for building packages (recommended for security)
RUN useradd -m -g users -G wheel builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

USER builder
WORKDIR /mnt
CMD ["bash"]
