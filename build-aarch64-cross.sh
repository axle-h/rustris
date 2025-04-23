#!/bin/bash
set -e

DEST=target/aarch64-unknown-linux-gnu/release

rm -rf $DEST
#mkdir -p $DEST/lib
mkdir -p $DEST
docker build . -t rustris-aarch64 -f Dockerfile.aarch64
docker create --name rustris_aarch64 rustris-aarch64

docker cp rustris_aarch64:/app/target/aarch64-unknown-linux-gnu/release/rustris $DEST
#docker cp rustris_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_gfx-1.0.so.0.0.2 $DEST/lib/libSDL2_gfx.so
#docker cp rustris_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_image-2.0.so.0.2.2 $DEST/lib/libSDL2_image.so
#docker cp rustris_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_mixer-2.0.so.0.2.2 $DEST/lib/libSDL2_mixer.so
#docker cp rustris_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_ttf-2.0.so.0.14.1 $DEST/lib/libSDL2_ttf.so

docker rm -f rustris_aarch64

scp $DEST/rustris  root@10.0.0.117:/roms/ports/rustris