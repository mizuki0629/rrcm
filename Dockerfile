FROM rust:slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    wget \
    git \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz && \
    tar -zxvf cargo-binstall-x86_64-unknown-linux-musl.tgz && \
    mv cargo-binstall $CARGO_HOME/bin/cargo-binstall && \
    rm cargo-binstall-x86_64-unknown-linux-musl.tgz

ARG USER=rustuser
ARG UID=1000
ARG GID=1000
RUN groupadd -g ${GID} ${USER} \
    && useradd -u ${UID} -g ${USER} -m ${USER}

USER ${USER}
RUN cargo binstall -y \
    cargo-nextest \
    cargo-make \
    cargo-edit \
    cargo-outdated \
    cargo-readme

ENTRYPOINT ["cargo", "make"]
CMD ["--help"]
