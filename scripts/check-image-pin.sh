#!/bin/sh
# Check the image action.yml runs is pinned to an exact release.
#
#   scripts/check-image-pin.sh           the pin is an exact vX.Y.Z image tag
#   scripts/check-image-pin.sh v0.1.29   ...and it is this release's own tag
#
# Why: a git ref of this action runs whatever image its action.yml names. When
# that was the floating :v1 image, every git tag ran the last image pushed
# under :v1, not its own code, and @v1 sat eight engine releases behind with
# nothing failing. Pinning each release commit to its own immutable image tag
# makes the git ref decide the image, so moving the v1 git tag moves both.
set -eu

repo_image="ghcr.io/nhatvu148/kaniscope-action"
action="${ACTION_YML:-action.yml}"

# The one `image:` line under `runs:`. Quotes are optional in YAML.
image=$(sed -n "s/^[[:space:]]*image:[[:space:]]*['\"]\{0,1\}\([^'\"[:space:]]*\)['\"]\{0,1\}[[:space:]]*\$/\1/p" "$action")
count=$(printf '%s\n' "$image" | grep -c . || true)
if [ "$count" -ne 1 ]; then
  echo "::error file=$action::expected exactly one image: line, found $count"
  exit 1
fi

tag=${image##*:}
case "$image" in
  "docker://$repo_image:"*) ;;
  *)
    echo "::error file=$action::image is $image, not docker://$repo_image:<tag>"
    exit 1
    ;;
esac
if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "::error file=$action::image tag :$tag floats; pin an exact release (:vX.Y.Z) so a git ref cannot run a different image than its own"
  exit 1
fi

if [ $# -ge 1 ] && [ "$1" != "$tag" ]; then
  echo "::error file=$action::releasing $1 but action.yml runs :$tag. Bump the image line to :$1 in the release commit, then tag that commit"
  exit 1
fi
echo "action.yml runs $image"
