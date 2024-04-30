FROM rust:1.77.2-bookworm

WORKDIR /vcpkg
RUN git clone --depth 1 --branch 2024.03.25 https://github.com/microsoft/vcpkg.git .
RUN ./bootstrap-vcpkg.sh
RUN ./vcpkg install --vcpkg-root=/vcpkg --triplet=arm64-linux-release sdl2 sdl2-image sdl2-gfx sdl2-mixer sdl2-ttf

ENV VCPKG_ROOT=/vcpkg \
    VCPKGRS_TRIPLET=arm64-linux-release

WORKDIR /app

ADD . .
RUN cargo build --release

CMD tail -f /dev/null