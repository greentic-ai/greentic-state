#!/usr/bin/env bash
set -euo pipefail

list_crates() {
  cargo metadata --format-version 1 --no-deps \
  | jq -r '.packages[] | "\(.name) \(.version) \(.manifest_path)"'
}

crate_dir_from_manifest() {
  local manifest="$1"
  dirname "$manifest"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  command="${1:-}"
  case "${command}" in
    list_crates)
      list_crates
      ;;
    crate_dir_from_manifest)
      crate_dir_from_manifest "${2:?manifest path required}"
      ;;
    *)
      echo "usage: $0 <list_crates|crate_dir_from_manifest> [args...]" >&2
      exit 1
      ;;
  esac
fi
