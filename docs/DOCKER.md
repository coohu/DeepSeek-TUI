# Docker

CodeWhale publishes a multi-arch Linux image to GitHub Container Registry
for each release.

```bash
docker pull ghcr.io/hmbown/deepseek:latest
```

## Quick start

Run the published image with a Docker-managed data volume:

```bash
docker volume create deepseek-home

docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -w /workspace \
  ghcr.io/hmbown/deepseek:latest
```

Use a pinned release tag for reproducible installs:

```bash
docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -w /workspace \
  ghcr.io/hmbown/deepseek:vX.Y.Z
```

Replace `vX.Y.Z` with a tag from
[GitHub Releases](https://github.com/Hmbown/CodeWhale/releases).

## Default image contract

`ghcr.io/hmbown/deepseek:latest` and the semver tags are conservative runtime
images:

- the container runs as the non-root `deepseek` user with UID/GID `1000:1000`
- the image does not grant passwordless `sudo`
- the image is meant to run CodeWhale against mounted workspaces, not to mutate
  the base operating system at runtime
- user state belongs in a volume mounted at `/home/deepseek/.deepseek`

That default is intentional. Keep using it for the smallest trust boundary. If a
project needs `apt-get`, compiler toolchains, Node/Python package managers,
custom CA certificates, or other host-like setup inside Docker, build an
explicit toolbox image instead of changing the default image contract.

## Opt-in toolbox/custom image

The repository includes an example
[`docs/examples/Dockerfile.toolbox`](examples/Dockerfile.toolbox) that extends
the official image with passwordless `sudo` and common development packages.
Build it with a pinned CodeWhale tag when you want repeatable project
environments:

```bash
docker build -f docs/examples/Dockerfile.toolbox \
  --build-arg CODEWHALE_IMAGE=ghcr.io/hmbown/deepseek:vX.Y.Z \
  --build-arg TOOLBOX_PACKAGES="git openssh-client curl build-essential pkg-config python3 python3-pip nodejs npm" \
  -t deepseek-toolbox:my-project .
```

Use `latest` only for throwaway testing. For shared projects, keep the
`CODEWHALE_IMAGE` value pinned and review package additions like any other
development-environment change.

Run the toolbox image with the same workspace and state mounts:

```bash
docker volume create deepseek-my-project-home

docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-my-project-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -w /workspace \
  deepseek-toolbox:my-project
```

Inside this opt-in image, CodeWhale can use commands such as
`sudo apt-get update` and `sudo apt-get install -y <package>`. For repeatable
containers, prefer baking those packages into the toolbox Dockerfile instead of
letting a long-lived container drift.

Do not bake API keys, SSH private keys, or other secrets into custom images.
Pass API keys at runtime and mount any SSH material deliberately, preferably
read-only and only for projects that need it.

## Multiple independent projects

Use one named state volume per project so sessions, config, skills, memory, and
the offline queue do not bleed across workspaces:

```bash
project="$(basename "$PWD")"
image="deepseek-toolbox:${project}"
docker volume create "deepseek-${project}-home"

docker run --rm -it \
  --name "deepseek-${project}" \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v "deepseek-${project}-home:/home/deepseek/.deepseek" \
  -v "$PWD:/workspace" \
  -w /workspace \
  "$image"
```

For projects with different toolchains, build different toolbox tags, for
example `deepseek-toolbox:frontend` and `deepseek-toolbox:backend`. The
separate launcher idea discussed in issue #2217 can build on this contract, but
it is intentionally outside the core Docker image.

## Project bootstrap scripts

CodeWhale does not automatically execute `.deepseek/setup.sh` or
`.deepseek/setup.sh`. If you keep one of those files as a local project
recipe, run it explicitly. For shared team setup, prefer a committed project
script or the toolbox Dockerfile so the environment can be reviewed and
rebuilt.

For example, to run a committed bootstrap script before starting CodeWhale:

```bash
docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-my-project-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -w /workspace \
  --entrypoint bash \
  deepseek-toolbox:my-project \
  -lc './scripts/bootstrap-dev.sh && exec deepseek'
```

Use the toolbox image for bootstrap scripts that need `sudo`. The default image
will not elevate privileges.

## Custom CA certificates and proxies

For corporate proxies, dev-sidecar, or self-signed internal services, prefer
baking trusted CA certificates into a custom toolbox image:

```dockerfile
USER root
COPY docker/certs/*.crt /usr/local/share/ca-certificates/
RUN update-ca-certificates
USER deepseek
```

All files copied into `/usr/local/share/ca-certificates/` must use the `.crt`
extension. Keep private CA material out of public images.

For a local-only run, mount certificates read-only and update the trust store at
container start:

```bash
docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-my-project-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -v "$PWD/docker/certs:/usr/local/share/ca-certificates/local:ro" \
  -w /workspace \
  --entrypoint bash \
  deepseek-toolbox:my-project \
  -lc 'sudo update-ca-certificates && exec deepseek'
```

This CA workflow requires the opt-in toolbox image because the default image
does not include passwordless `sudo`.

## Local build

Build the image locally from a checkout:

```bash
docker build -t deepseek .
```

Then run it with the same Docker-managed data volume:

```bash
docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v deepseek-home:/home/deepseek/.deepseek \
  -v "$PWD:/workspace" \
  -w /workspace \
  deepseek
```

Docker Hub publishing is not configured; GHCR is the supported prebuilt image
registry.

## Environment variables

| Variable              | Required | Description                                      |
|-----------------------|----------|--------------------------------------------------|
| `DEEPSEEK_API_KEY`    | yes      | DeepSeek API key                                 |
| `DEEPSEEK_BASE_URL`   | no       | Custom API base URL (e.g. `https://api.deepseek.com`) |
| `DEEPSEEK_NO_COLOR`   | no       | Set to `1` to disable terminal colour output     |

## Volumes

Mount `/home/deepseek/.deepseek` to persist sessions, config, skills, memory,
and the offline queue across container restarts. A Docker-managed named volume
is the safest default because Docker creates it with ownership the container can
write:

```bash
-v deepseek-home:/home/deepseek/.deepseek
```

Without this mount the container starts fresh each time.

If you bind-mount an existing host directory instead, the image runs as the
non-root `deepseek` user with UID/GID `1000:1000`. The mounted directory must be
writable by that user, or startup can fail while creating runtime directories
under `.deepseek/tasks`. On Linux hosts, either use the named volume above or
prepare the bind mount explicitly:

```bash
mkdir -p ~/.deepseek
sudo chown -R 1000:1000 ~/.deepseek

docker run --rm -it \
  -e DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" \
  -v ~/.deepseek:/home/deepseek/.deepseek \
  ghcr.io/hmbown/deepseek:latest
```

That `chown` changes ownership of the host `~/.deepseek` directory. Skip it if
you do not want the container UID to own your local config, and use a named
volume instead.

## Non-interactive / pipeline usage

When stdin is not a TTY, `deepseek` drops to the dispatcher's one-shot mode
(`deepseek -c "…"`). Pipe a prompt on stdin:

```bash
echo "Explain the Cargo.toml in structured English." | \
  docker run --rm -i -e DEEPSEEK_API_KEY ghcr.io/hmbown/deepseek:latest
```

## Building locally

```bash
# Single platform (your host architecture)
docker build -t deepseek .

# Multi-platform (requires a builder with emulation)
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t deepseek .
```

## Devcontainer

The repository includes a [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json)
configuration for VS Code / GitHub Codespaces. It pre-installs the Rust toolchain,
rust-analyzer, and the `deepseek` binary. Open the repo in a devcontainer to get a
ready-to-use development environment.

## Release status

Docker image publishing is part of the release gate. The image is published to
GHCR for `linux/amd64` and `linux/arm64` with semver tags plus `latest`.
