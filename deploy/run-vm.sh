#!/usr/bin/env bash

set -euxo pipefail

parent_path=$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd -P)
cd "$parent_path"

IMAGE_URL="https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img"
# https://www.baeldung.com/linux/shell-extract-url-base-filename
IMAGE_NAME="${IMAGE_URL##*/}"
ARTIFACTS_DIR="artifacts"
SEED_IMG="seed.img"
CONFIG_DIR="cloud-init"
OVERLAY_DISK="overlay.qcow2"

download_img() {
    curl --skip-existing -L "${IMAGE_URL}" --output-dir "${ARTIFACTS_DIR}" -o "${IMAGE_NAME}"
}

create_seed_img() {
    # https://docs.cloud-init.io/en/24.3/howto/run_cloud_init_locally.html#create-an-iso-disk
    genisoimage \
        -output "${ARTIFACTS_DIR}/${SEED_IMG}" \
        -volid cidata -rational-rock -joliet \
        "${CONFIG_DIR}"/user-data \
        "${CONFIG_DIR}"/meta-data \
        "${CONFIG_DIR}"/network-config
}

create_disk() {
    if [ ! -f "${ARTIFACTS_DIR}/${OVERLAY_DISK}" ]; then
        qemu-img create \
            -f qcow2 \
            -b "${IMAGE_NAME}" \
            -B qcow2 \
            "${ARTIFACTS_DIR}/${OVERLAY_DISK}" \
            15G
    fi
}

run_vm() {
    qemu-system-x86_64 \
        -m 1024 \
        -net nic -net user \
        -drive file="${ARTIFACTS_DIR}/${OVERLAY_DISK}",index=0,format=qcow2,media=disk \
        -drive file="${ARTIFACTS_DIR}/${SEED_IMG}",index=1,media=cdrom \
        -machine accel=kvm:tcg
    # TODO: what does machine accel=kvm:tcg mean
}

clean_overlay() {
    rm "${ARTIFACTS_DIR}/${OVERLAY_DISK}"
}

clean_download() {
    rm "${ARTIFACTS_DIR}/${IMAGE_NAME}"
}

clean() {
    rm -r "${ARTIFACTS_DIR}"
}

copy_files_to_img() {
    # TODO: better way to sync project
    guestfish --rw -a "${ARTIFACTS_DIR}/${OVERLAY_DISK}" -m /dev/sda1 << _EOF_
    mkdir-p /home/ubuntu/keyprod
    copy-in ../src/ /home/ubuntu/keyprod/
    copy-in ../build.rs /home/ubuntu/keyprod/
    copy-in ../Cargo.lock /home/ubuntu/keyprod/
    copy-in ../Cargo.toml /home/ubuntu/keyprod/
    copy-in ../Makefile /home/ubuntu/keyprod/
    chmod 0777 /home/ubuntu/keyprod/
_EOF_
}

main() {
    mkdir -p "${ARTIFACTS_DIR}"

    download_img
    create_disk
    create_seed_img
    copy_files_to_img
    run_vm
}

main
