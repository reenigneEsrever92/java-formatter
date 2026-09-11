#!/usr/bin/env sh
# Fetch a real, large Java project to use as the corpus for the CLI's
# directory-mode benchmark (crates/cli/benches/dir_mode.rs).
#
# The corpus is downloaded into target/bench-corpus/ (inside the gitignored
# target dir, so it never pollutes the tree) and is used read-only by the
# benchmark, which works on a private copy.
#
# Idempotent: when the project is already present the download is skipped.
# Override the project / ref / directory name with environment variables:
#
#   BENCH_REPO=apache/hadoop BENCH_REF=rel/release-3.4.0 \
#     BENCH_DIR_NAME=hadoop scripts/fetch-bench-corpus.sh
#
# Default: spring-framework v6.1.6, a ~5.7k-file Java codebase.

set -eu

REPO=${BENCH_REPO:-spring-projects/spring-framework}
REF=${BENCH_REF:-v6.1.6}
DIR_NAME=${BENCH_DIR_NAME:-spring-framework}

DEST=target/bench-corpus
URL="https://codeload.github.com/${REPO}/tar.gz/refs/tags/${REF}"
TARBALL="${DEST}/${DIR_NAME}.tar.gz"

mkdir -p "$DEST"

if [ -d "$DEST/$DIR_NAME" ]; then
    echo "corpus already present: $DEST/$DIR_NAME"
    echo "$(find "$DEST/$DIR_NAME" -name '*.java' | wc -l | tr -d ' ') java files, \
$(du -sh "$DEST/$DIR_NAME" | cut -f1)"
    exit 0
fi

echo "downloading $URL"
if ! curl -L --fail --silent --show-error "$URL" -o "$TARBALL"; then
    echo "error: download failed (network is blocked unless the sandbox grants it)" >&2
    exit 1
fi

echo "extracting…"
# The tarball's root directory is `<repo>-<ref>`, whose exact shape varies
# (e.g. with/without a "v" prefix); unpack and rename to a stable name.
first=$(tar -tzf "$TARBALL" | sed -n '1p' | cut -d/ -f1)
tar -xzf "$TARBALL" -C "$DEST"
rm -f "$TARBALL"
mv "$DEST/$first" "$DEST/$DIR_NAME"

echo "corpus ready: $DEST/$DIR_NAME"
echo "$(find "$DEST/$DIR_NAME" -name '*.java' | wc -l | tr -d ' ') java files, \
$(du -sh "$DEST/$DIR_NAME" | cut -f1)"
