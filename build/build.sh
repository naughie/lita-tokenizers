#!/bin/sh

set -e

__target_triple="$1"

__rand_id="$(od -vAn -N4 -tx1 </dev/urandom | tr -d ' \n')"
__image_name="lita-tokenizers-cli-${__target_triple}"
__container_name="lita-tokenizers-cli-${__target_triple}-${__rand_id}"

__dist_dir="lita-tokenizers-cli-${__target_triple}"
__asset="lita-tokenizers-cli-${__target_triple}.tar.gz"

docker build -t "${__image_name}" -f ./build/Dockerfile --build-arg TARGET_TRIPLE="${__target_triple}" .
docker create --name "${__container_name}" "${__image_name}"

mkdir -p "${__dist_dir}"
docker cp "${__container_name}:/lita-tokenizers-cli" "${__dist_dir}/lita-tokenizers-cli"
docker rm "${__container_name}"

mkdir -p "${__dist_dir}/license/kytea"
mkdir -p "${__dist_dir}/license/lita-tokenizers-cli"
mkdir -p "${__dist_dir}/license/mecab"
cp ./LICENSE-APACHE ./LICENSE-MIT "${__dist_dir}/license/lita-tokenizers-cli"
cp ./crates/kytea-sys/kytea-src/COPYING "${__dist_dir}/license/kytea"
cp ./crates/mecab-sys/mecab-src/COPYING ./crates/mecab-sys/mecab-src/BSD "${__dist_dir}/license/mecab"

tar czf "${__asset}" "${__dist_dir}"
