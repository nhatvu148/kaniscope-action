# Releasing

`action.yml` pins the container to **this release's own image tag**:

```yaml
image: 'docker://ghcr.io/nhatvu148/kaniscope-action:v0.1.29'
```

A consumer on any git ref runs the image that ref's `action.yml` names. So the
git tag decides the image: `@v0.1.29` always runs the v0.1.29 image, and moving
the floating `v1` git tag moves `@v1` to the new image in the same step.

It used to name the floating `:v1` image instead, which meant every git tag ran
whatever image was last pushed under `:v1`, not its own code. Between 2026-08-18
and 2026-09-04 that left `@v1` eight `pr-review-core` minor versions behind
`main`, and nothing failed to say so.

## The order, and why

```sh
# 1. The release commit (a PR, merged to main) bumps the engine if there is one
#    and points action.yml at the tag it is about to get:
#      image: 'docker://ghcr.io/nhatvu148/kaniscope-action:v0.1.29'
#    The PR check and the tag-push jobs fail if this line does not match.

# 2. Annotated tag on that commit. Its body becomes the release notes verbatim;
#    release.yml reads the annotation.
git tag -a v0.1.29 -m "v0.1.29

Whatever the release notes should say, in markdown."

# 3. Push it. This triggers publish.yml (checks the pin, then builds and pushes
#    the :v0.1.29 image) and release.yml (checks the pin, then creates the
#    release).
git push origin v0.1.29

# 4. WAIT for "Publish image" to go green. Until it does, the tagged commit
#    names an image that does not exist yet.

# 5. Only now move the floating git tag.
git tag -f v1 v0.1.29^{}
git push --force origin v1
```

Step 1 is what is new. Forgetting it is caught rather than shipped:
`scripts/check-image-pin.sh v0.1.29` fails both tag-push workflows before
anything is built or listed, and the pin check on PRs rejects a floating tag.

## Checks worth doing

`consumer check (published @v1)` is the one that matters. It reads `action.yml`
at the `v1` tag, pulls the image it names anonymously, and runs the action
through `@v1` the way a real user does. Green there means a consumer gets what
you think they get.

The review log names the engine that ran (`with pr-review-core X.Y.Z`), so a
consumer's run says which engine reviewed it.

To confirm the image exists under the new tag:

```sh
gh api /users/nhatvu148/packages/container/kaniscope-action/versions \
  --jq '[.[] | select(.metadata.container.tags[]? == "v0.1.29")] | .[0]
        | "\(.updated_at)  \(.metadata.container.tags)"'
```

## The `:v1` image tag

publish.yml still pushes the floating `:v1` image on every release. Nothing
current reads it. It is kept for git tags cut before images were pinned
(v0.1.28 and older), whose `action.yml` names `:v1`. Stopping it would freeze
those tags on whatever `:v1` held last.

## Don't

- **Don't `workflow_dispatch` publish.yml from an untagged branch.** The
  `enable` guard keeps it from moving the `:v1` image tag onto an untagged
  build, and an untagged build has no `:vX.Y.Z` tag for any `action.yml` to pin.
- **Don't move `v1` to a commit whose image has not been published.** `@v1`
  would then fail to pull for every consumer.
- **Don't re-push an existing version tag's image.** Users pinned to
  `@vX.Y.Z` expect that image never to change.
