#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "setup.sh: $*" >&2
    exit 1
}

download() {
    curl --fail --silent --show-error --location --retry 3 --output "$2" "$1" ||
        fail "could not download $1, check that the release exists"
}

# Everything runs from main so that a truncated `curl | bash` download executes nothing.
main() {
    if [[ $# -lt 1 || -z "$1" ]]; then
        echo "usage: setup.sh <version> [<install dir>]" >&2
        exit 2
    fi
    local version="$1"
    local install_dir="${2:-${HOME}/.local/bin}"

    local kernel machine os arch
    kernel="$(uname -s)"
    machine="$(uname -m)"
    case "${kernel}" in
        Linux) os=linux ;;
        Darwin) os=darwin ;;
        *) fail "unsupported OS: ${kernel}" ;;
    esac
    case "${machine}" in
        x86_64 | amd64) arch=amd64 ;;
        aarch64 | arm64) arch=arm64 ;;
        *) fail "unsupported architecture: ${machine}" ;;
    esac

    local asset="tf_plan_format-${version}-${os}-${arch}.tar.gz"
    local base_url="https://github.com/ovsds/tf-plan-format/releases/download/${version}"
    # Global, not local: the EXIT trap runs after main has returned.
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "${tmp_dir}"' EXIT

    download "${base_url}/${asset}" "${tmp_dir}/${asset}"
    download "${base_url}/checksums.txt" "${tmp_dir}/checksums.txt"

    local expected actual
    expected="$(awk -v name="${asset}" '$2 == name { print $1 }' "${tmp_dir}/checksums.txt")"
    [[ -n "${expected}" ]] || fail "no checksum for ${asset} in checksums.txt"
    if command -v sha256sum >/dev/null; then
        actual="$(sha256sum "${tmp_dir}/${asset}" | awk '{ print $1 }')"
    else
        actual="$(shasum -a 256 "${tmp_dir}/${asset}" | awk '{ print $1 }')"
    fi
    [[ "${actual}" == "${expected}" ]] || fail "checksum mismatch for ${asset}: expected ${expected}, got ${actual}"

    tar -xzf "${tmp_dir}/${asset}" -C "${tmp_dir}" tf_plan_format
    mkdir -p "${install_dir}"
    # An exported CDPATH would make cd print or pick another directory
    install_dir="$(CDPATH='' cd -- "${install_dir}" && pwd)"
    install -m 0755 "${tmp_dir}/tf_plan_format" "${install_dir}/tf_plan_format"
    # Linux builds need glibc: fail here rather than at first use on musl systems like Alpine
    "${install_dir}/tf_plan_format" --version >/dev/null ||
        fail "installed ${install_dir}/tf_plan_format but it does not run on this system"
    echo "Installed ${install_dir}/tf_plan_format"
}

main "$@"
