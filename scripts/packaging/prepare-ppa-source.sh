#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: prepare-ppa-source.sh <version> <ubuntu-series> [ppa-revision]

Creates a vendored Debian source package under target/ppa-source and prints
the generated *_source.changes path.
USAGE
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage
  exit 2
fi

version="$1"
series="$2"
ppa_revision="${3:-1}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_root="$root/target/ppa-source"
source_dir="$build_root/skillmux-$version"
debian_version="${version}-1~ppa${ppa_revision}~${series}1"

rm -rf "$build_root"
mkdir -p "$source_dir"

git -C "$root" archive --format=tar HEAD | tar -xf - -C "$source_dir"

rm -rf "$source_dir/debian"
cp -R "$root/packaging/debian" "$source_dir/debian"
chmod +x "$source_dir/debian/rules"

cat > "$source_dir/debian/changelog" <<EOF
skillmux (${debian_version}) ${series}; urgency=medium

  * Release ${version} to Launchpad PPA.

 -- ${DEBFULLNAME:-Kingdee AI Team} <${DEBEMAIL:-noreply@example.com}>  $(date -R)
EOF

(
  cd "$source_dir"
  mkdir -p .cargo
  cargo vendor vendor > .cargo/config.toml
)

tar --sort=name --mtime="@0" --owner=0 --group=0 --numeric-owner \
  --exclude='./debian' \
  -czf "$build_root/skillmux_${version}.orig.tar.gz" \
  -C "$build_root" "skillmux-$version"

(
  cd "$source_dir"
  debuild_args=(-S -sa)
  if [[ -n "${DEBSIGN_KEYID:-}" ]]; then
    debuild_args+=("-k${DEBSIGN_KEYID}")
  else
    debuild_args+=(-us -uc)
  fi
  debuild "${debuild_args[@]}"
)

changes_file="$(find "$build_root" -maxdepth 1 -name "skillmux_${debian_version}_source.changes" -print -quit)"
if [[ -z "$changes_file" ]]; then
  echo "Failed to find generated source changes file in $build_root" >&2
  exit 1
fi

echo "$changes_file"
