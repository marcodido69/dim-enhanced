FROM node:22-bookworm AS web
WORKDIR /ui
COPY ui/package*.json ./
# Husky no debe intentar instalar hooks dentro de la imagen (no hay .git en /ui).
ENV HUSKY=0
RUN yarn install --network-timeout 600000
COPY ui ./
ENV NODE_OPTIONS=--openssl-legacy-provider
RUN yarn run build

FROM debian:bookworm AS ffmpeg
ARG DEBIAN_FRONTEND=noninteractive
WORKDIR /static
ARG TARGETPLATFORM
RUN echo ${TARGETPLATFORM}
RUN apt-get update && \
    apt-get install -y --no-install-recommends wget unzip tar ca-certificates xz-utils && \
    rm -rf /var/lib/apt/lists/*

RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
    wget https://github.com/Dusk-Labs/ffmpeg-static/releases/download/ffmpeg-all-0.0.1/ffmpeg && \
    wget https://github.com/Dusk-Labs/ffmpeg-static/releases/download/ffmpeg-all-0.0.1/ffprobe && \
    ls -la . && \
    pwd \
    ; fi

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
    wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz && \
    tar --strip-components 1 -xf ffmpeg-release-arm64-static.tar.xz \
    ; fi

RUN if [ "${TARGETPLATFORM}" = "linux/arm/v7" ]; then \
    wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-armhf-static.tar.xz && \
    tar --strip-components 1 -xf ffmpeg-release-armhf-static.tar.xz \
    ; fi

RUN chmod +x /static/ffmpeg && chmod +x /static/ffprobe


FROM rust:bookworm AS dim
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    libva-dev \
    libva-drm2 \
    libva2 \
    libsqlite3-dev \
    sqlite3 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /dim
COPY . ./
# Usamos el toolchain stable que trae la imagen; el archivo del repo haría que
# rustup intente descargar otro toolchain dentro del build.
RUN rm -f rust-toolchain.toml
COPY --from=web /ui/build ui/build
ARG DATABASE_URL="sqlite://dim_dev.db"

# Los build scripts de dim-core/dim-database/dim-web leen CARGO_TARGET_DIR, que
# Cargo no define por sí solo: sin esto el build falla con un panic en build.rs.
ENV CARGO_TARGET_DIR=/dim/target

# Sometimes we may need to quickly build a test image
ARG RUST_BUILD=release
RUN if [ "$RUST_BUILD" = "debug" ]; then \
        cargo build --features vaapi && \
        mv ./target/debug/dim ./target/dim \
    ; fi

RUN if [ "$RUST_BUILD" = "release" ]; then \
        cargo build --features vaapi --release && \
        mv ./target/release/dim ./target/dim \
    ; fi

FROM debian:bookworm
ENV RUST_BACKTRACE=full
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libfontconfig1 \
    libfribidi0 \
    libharfbuzz0b \
    libtheora0 \
    libva-drm2 \
    libva2 \
    libvorbis0a \
    libvorbisenc2 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=ffmpeg /static/ffmpeg /opt/dim/utils/ffmpeg
COPY --from=ffmpeg /static/ffprobe /opt/dim/utils/ffprobe
COPY --from=dim /dim/target/dim /opt/dim/dim

EXPOSE 8000
VOLUME ["/opt/dim/config"]

ENV RUST_LOG=info
WORKDIR /opt/dim
CMD ["./dim"]
