# lazymc-docker-proxy

By using this image, you can combine the straightforward management of Minecraft servers using [itzg/minecraft-server](https://github.com/itzg/docker-minecraft-server) with the efficiency of [lazymc](https://github.com/timvisee/lazymc).

If you have multiple Docker Minecraft servers and you are low on resources or want to save compute power, this project is for you. With `lazymc-docker-proxy`, you can dynamically start a Docker Minecraft server when a player joins and shut it down when the server is idle.

This project is also somewhat inspired by [lazytainer](https://github.com/vmorganp/Lazytainer).

![example](images/example.png)

# Usage

## Vanilla minimal Example

Here is a minimal Docker Compose example using [itzg/minecraft-server](https://github.com/itzg/docker-minecraft-server) as the server:

```yaml
services:
  lazymc:
    container_name: lazymc
    image: ghcr.io/joesturge/lazymc-docker-proxy:latest
    restart: unless-stopped
    volumes:
      # you should mount the minecraft server dir under /server, using read only.
      - data:/server:ro
      # you need to supply the docker socket, so that the container can run docker command
      - /var/run/docker.sock:/var/run/docker.sock:ro
    ports:
      # lazymc-docker-proxy acts as a proxy, so there is
      # no need to expose the server port on the Minecraft container
      - "25565:25565"

  # Standard Docker Minecraft server, also works with other server types
  mc:
    image: itzg/minecraft-server:java21
    container_name: minecraft-server
    # We need to add a label here so that lazymc-docker-proxy knows which
    # container to manage
    labels:
      # Set lazymc.enabled to true to enable lazymc on this container
      - lazymc.enabled=true
      # Required to find the container to manage it
      - lazymc.group=mc
      # Point to the service name of the Minecraft server
      - lazymc.server.address=mc:25565
    tty: true
    stdin_open: true
    # This container should be managed solely by the lazymc container
    # so set restart to no, or else the container will start again...
    restart: no
    environment:
      EULA: "TRUE"
    volumes:
      - data:/data

volumes:
  data:
```

## Forge 1.19.2

```yaml
services:
  lazymc:
    container_name: lazymc
    image: ghcr.io/joesturge/lazymc-docker-proxy:latest
    restart: unless-stopped
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - data:/server:ro
    ports:
      - "25565:25565"

  mc:
    image: itzg/minecraft-server:java21
    container_name: mc
    labels:
      - lazymc.enabled=true
      - lazymc.group=mc
      - lazymc.server.address=mc:25565
      # The minecraft client version of the forge server
      - lazymc.public.version=1.19.2
      # The minecraft protocol version of the forge server
      - lazymc.public.protocol=760
      # Set to true on a forge server
      - lazymc.server.forge=true
    tty: true
    stdin_open: true
    restart: no
    environment:
      EULA: "TRUE"
      TYPE: FORGE
      # The minecraft client version
      VERSION: "1.19.2"
    volumes:
      - data:/data

volumes:
  data:
```

## Configuration using labels

The suggested way to manage the lazymc settings on your minecraft containers is to use container labels.
This was done so that in the future, it would be possible to manage and proxy to multiple servers.

Here is the list of the supported container labels which can be added to the minecraft server container
which will be picked up by `lazymc-docker-proxy` (\* is required):

- **\*lazymc.enabled=true** - Enable this to inform `lazymc-docker-proxy` that this container should be managed.
- **\*lazymc.server.address** - The address of the Docker Minecraft server to manage, should use the Docker network address, such as `mc:25565`.
- **\*lazymc.group** - This is used by `lazymc-docker-proxy` to locate the container to start and stop
- **lazymc.port** - The internal port which is opened by lazymc to proxy to the server. Defaults to `25565`.
- **lazymc.motd.sleeping** - MOTD, shown in the server browser when sleeping.
- **lazymc.motd.starting** - MOTD, shown in the server browser when starting.
- **lazymc.motd.stopping** - MOTD, shown in the server browser when stopping.
- **lazymc.public.protocol** - The minecraft client version to use. See this page for information: https://minecraft.fandom.com/wiki/Protocol_version
- **lazymc.public.version** - The minecraft protocol version to use. See this page for information: https://minecraft.fandom.com/wiki/Protocol_version
- **lazymc.server.block_banned_ips** - To wake the server, the user must be in the server whitelist if enabled on the server. Defaults to `/server`.
- **lazymc.server.directory** - The location of the volume mount within `lazymc-docker-proxy` which contains data for this minecraft server
- **lazymc.server.drop_banned_ips** - Block banned IPs as listed in banned-ips.json in the server directory.
- **lazymc.server.forge** - Drop connections from banned IPs.
- **lazymc.server.probe_on_start** - Probe required server details when starting lazymc, wakes server on start.
- **lazymc.server.send_proxy_v2** - Set to true if this server runs Forge.
- **lazymc.server.wake_whitelist** - Add HAProxy v2 header to proxied connections.
- **lazymc.time.minimum_online_time** - Sleep after a number of seconds.
- **lazymc.time.sleep_after** - Minimum time in seconds to stay online when the server is started.

> Note: `wake_on_crash` and `wake_on_start` are not configurable due to how lazymc starts the server. When running in Docker Compose, all containers are started by default, so `wake_on_start` must also be true when using this image. `wake_on_crash` is also true as it is recommended to launch the Minecraft server with `restart: no`.

> Note: `rcon` configurations are not supported as this app relies on the SIGTERM signal to stop the server.

If you want more details or have issues, you can also refer to the lazymc [documentation](https://github.com/timvisee/lazymc/tree/master).

Also, refer to the lazymc [config example](https://github.com/timvisee/lazymc/blob/master/res/lazymc.toml). You may notice that the environment variables are named in the same way. This is intentional...

## Environment Variables

You can enable debug logging using the `RUST_LOG` env var.

- **RUST_LOG** - Set this to `trace` or `debug` to troubleshoot issues.

### Deprecated

> ⚠️ Using environment variables to configure the server is deprecated and will be removed in the next release, please use container labels on the minecraft server instead. These are still available for the time being.

Here is a full list of the environment variables supported by this image (\* is required):

- **\*SERVER_ADDRESS** - The address of the Docker Minecraft server to manage, should use the Docker network address, such as `mc:25565`.
- **\*LAZYMC_GROUP** - The value of the `lazymc.group` label assigned to the Docker Minecraft server. This is used by the image to start or stop the server when lazymc triggers it.
- **LAZYMC_PORT** - The internal port which is opened by lazymc to proxy to the server. Defaults to `25565`.
- **MOTD_SLEEPING** - MOTD, shown in the server browser when sleeping.
- **MOTD_STARTING** - MOTD, shown in the server browser when starting.
- **MOTD_STOPPING** - MOTD, shown in the server browser when stopping.
- **PUBLIC_VERSION** - The minecraft client version to use. See this page for information: https://minecraft.fandom.com/wiki/Protocol_version
- **PUBLIC_PROTOCOL** - The minecraft protocol version to use. See this page for information: https://minecraft.fandom.com/wiki/Protocol_version
- **SERVER_WAKE_WHITELIST** - To wake the server, the user must be in the server whitelist if enabled on the server.
- **SERVER_BLOCK_BANNED_IPS** - Block banned IPs as listed in banned-ips.json in the server directory.
- **SERVER_DIRECTORY** - The location of the volume mount within `lazymc-docker-proxy` which contains data for this minecraft server. Defaults to `/server`.
- **SERVER_DROP_BANNED_IPS** - Drop connections from banned IPs.
- **SERVER_PROBE_ON_START** - Probe required server details when starting lazymc, wakes server on start.
- **SERVER_FORGE** - Set to true if this server runs Forge.
- **SERVER_SEND_PROXY_V2** - Add HAProxy v2 header to proxied connections.
- **TIME_SLEEP_AFTER** - Sleep after a number of seconds.
- **TIME_MINIMUM_ONLINE_TIME** - Minimum time in seconds to stay online when the server is started.
- **RUST_LOG** - Set this to `trace` or `debug` to troubleshoot issues.

# Development

Thanks for wanting to contribute! Please read the [contributing guidelines](CONTRIBUTING.md) first off.

## Compile locally

To develop changes fork this repo and ensure you have the following installed:

- [rust](https://www.rust-lang.org/tools/install)
- [docker + docker compose](https://docs.docker.com/get-docker/)
- [bats testing](https://bats-core.readthedocs.io/en/stable/installation.html)

Then run the following

```bash
cargo build --release
```

to build the executable under `target/release` directory.

## Docker

To build the dockerfile locally you can run (from the project root)

```bash
docker build .
```

It is good to also check this before committing to make sure you did not break the build

## Testing

There is a bats integration test for this application. Which performs the following:

1. Builds the docker compose file located at `test/bats/docker-compose.yaml`
2. Starts the docker compose environment for this app and a minecraft container
3. Waits for the minecraft server to start
4. Waits for lazymc to stop the minecraft container

This check is performed in using github actions on every commit

## Changelog

This project uses [keepachangelog](https://keepachangelog.com/en/1.1.0/) to maintain it's changelog.
If you are proposing a change please update the [changelog](CHANGELOG.md) by adding your changes under the `[Unreleased]` header

# Thanks

Thanks for taking the time to check this project out, I hope it helps you manage your minecraft servers for years to come!
