#!/bin/sh

set -e

__rand_id="$(od -vAn -N4 -tx1 </dev/urandom | tr -d ' \n')"
__image_name="lita-tokenizers-cli"
__container_name="lita-tokenizers-cli-bincopy-${__rand_id}"

docker build -t "${__image_name}" -f ./build/Dockerfile .
docker create --name "${__container_name}" "${__image_name}"
docker cp "${__container_name}:/lita-tokenizers-cli" ./lita-tokenizers-cli
docker rm "${__container_name}"
