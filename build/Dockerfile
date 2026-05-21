FROM python:3.14-trixie AS app_builder

ARG TARGET_TRIPLE=x86_64-unknown-linux-musl

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --target ${TARGET_TRIPLE}

RUN pip install cargo-zigbuild

WORKDIR /app/lita-tokenizers

COPY . .

RUN . /opt/cargo/env \
    && cargo-zigbuild zigbuild --target ${TARGET_TRIPLE} -p lita-tokenizers-cli --profile release-minified \
    && cp ./target/${TARGET_TRIPLE}/release-minified/lita-tokenizers-cli /lita-tokenizers-cli
