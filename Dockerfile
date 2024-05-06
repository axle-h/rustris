FROM rust:1.77.2-bullseye

RUN apt-get update && apt-get install -y libsdl2-dev libsdl2-image-dev libsdl2-gfx-dev libsdl2-mixer-dev libsdl2-ttf-dev libmp3lame-dev libdrm-dev libgbm-dev

WORKDIR /app

ADD . .
RUN cargo build --release

CMD tail -f /dev/null