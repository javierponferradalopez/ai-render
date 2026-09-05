#!/bin/bash
# Generates the marketplace.json from the release tag, on stdout. Never edited
# by hand.
#
#   catalog.sh <tag> <zip> [repo]
#
# It is a correctness requirement, not a convenience: measured, `/plugin update`
# downloads the whole zip *before* comparing identities, so a fix published
# without bumping the version gets downloaded, thrown away and never mentioned
# —4.1 s to say «already at the latest version»—. Taking `version`, `url` and
# `sha256` from the same place, and from the same zip, is what makes forgetting
# the bump impossible.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
readonly ROOT

# shellcheck source=json.sh
. "$ROOT/publishing/json.sh"

die() {
  printf 'catalog: %s\n' "$1" >&2
  exit 1
}

[ $# -ge 2 ] || die 'usage: catalog.sh <tag> <zip> [repo]'
readonly TAG=$1
readonly ZIP=$2
readonly REPO=${3:-${GITHUB_REPOSITORY:-javierponferradalopez/flipchart}}
readonly VERSION=${TAG#v}

[ -f "$ZIP" ] || die "there is no zip at $ZIP"

# The manifest is read from inside the zip that is about to be published, not
# from the working tree: that way the catalog cannot describe anything other
# than the box it serves.
readonly MANIFEST=.claude-plugin/plugin.json
manifest=$(unzip -p "$ZIP" "$MANIFEST") || die "the zip does not carry $MANIFEST"
name=$(field name "$manifest") || die "$MANIFEST declares no name"
description=$(field description "$manifest") || die "$MANIFEST declares no description"
declared=$(field version "$manifest") || die "$MANIFEST declares no version"

# The one that rules is the one in the manifest inside the zip, and the tag's is
# the one the user is going to read. A catalog that mixes them publishes a lie.
[ "$declared" = "$VERSION" ] \
  || die "the zip declares $declared and the tag says $VERSION"

# The JSON is composed with `printf`, so a `"` or a `\` in a value would break
# it silently. The premise is checked instead of trusted.
for value in "$name" "$description"; do
  case $value in
    *'"'* | *'\'*) die "a manifest value carries quotes or backslashes: $value" ;;
  esac
done

# Measured: `sha256` is optional in the host's schema, and an entry without it
# installs just the same and checks nothing, without warning. Publishing it
# empty would silently disarm the vehicle's only integrity defence.
digest=$(shasum -a 256 "$ZIP" | cut -d' ' -f1)
[ ${#digest} -eq 64 ] || die "the sha256 of the zip did not come out: '$digest'"

# Immutable on purpose: a pinned digest points at an exact byte, so if the URL
# could change its content the pin would be worth nothing.
readonly URL="https://github.com/$REPO/releases/download/$TAG/$(basename "$ZIP")"

cat <<JSON
{
  "name": "$name",
  "description": "$description",
  "owner": { "name": "${REPO%%/*}" },
  "plugins": [
    {
      "name": "$name",
      "description": "$description",
      "version": "$VERSION",
      "source": {
        "source": "archive",
        "url": "$URL",
        "sha256": "$digest"
      }
    }
  ]
}
JSON
