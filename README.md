# lazymc-docker-proxy

By using this image, you can combine the straightforward management of Minecraft servers using [itzg/minecraft-server](https://github.com/itzg/docker-minecraft-server) with the efficiency of [lazymc](https://github.com/timvisee/lazymc).

If you have multiple Docker Minecraft servers and you are low on resources or want to save compute power, this project is for you. With `lazymc-docker-proxy`, you can dynamically start a Docker Minecraft server when a player joins and shut it down when the server is idle.

This project is also somewhat inspired by [lazytainer](https://github.com/vmorganp/Lazytainer).

# Usage

Here is a minimal Docker Compose example using [itzg/minecraft-server](https://github.com/itzg/docker-minecraft-server) as the server:

```yaml
services:
    lazymc:
        container_name: lazymc
        environment:
            # Point to the service name of the Minecraft server
            - SERVER_ADDRESS=mc:25565
            # Required to find the container to manage it
            - LAZYMC_GROUP=mc
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
        build: .

    # Standard Docker Minecraft server, also works with other server types
    mc:
        image: itzg/minecraft-server:java21
        container_name: minecraft-server
        # We need to add a label here so that lazymc-docker-proxy knows which
        # container to manage
        labels:
            - lazymc.group=mc
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

## Environment Variables

Here is a full list of the environment variables supported by this image (* is required):

- ***SERVER_ADDRESS** - The address of the Docker Minecraft server to manage, should use the Docker network address, such as `mc:25565`.
- ***LAZYMC_GROUP** - The value of the `lazymc.group` label assigned to the Docker Minecraft server. This is used by the image to start or stop the server when lazymc triggers it.
- **SERVER_WAKE_WHITELIST** - To wake the server, the user must be in the server whitelist if enabled on the server.
- **SERVER_BLOCK_BANNED_IPS** - Block banned IPs as listed in banned-ips.json in the server directory.
- **SERVER_DROP_BANNED_IPS** - Drop connections from banned IPs.
- **SERVER_PROBE_ON_START** - Probe required server details when starting lazymc, wakes server on start.
- **SERVER_FORGE** - Set to true if this server runs Forge.
- **SERVER_SEND_PROXY_V2** - Add HAProxy v2 header to proxied connections.
- **TIME_SLEEP_AFTER** - Sleep after a number of seconds.
- **TIME_MINIMUM_ONLINE_TIME** - Minimum time in seconds to stay online when the server is started.
- **MOTD_SLEEPING** - MOTD, shown in the server browser when sleeping.
- **MOTD_STARTING** - MOTD, shown in the server browser when starting.
- **MOTD_STOPPING** - MOTD, shown in the server browser when stopping.
- **RCON_ENABLED** - Enable sleeping server through RCON.
- **RCON_PORT** - Server RCON port. Must differ from public and server port.
- **RCON_PASSWORD** - Server RCON password.

> Note: `wake_on_crash` and `wake_on_start` are not configurable due to how lazymc starts the server. When running in Docker Compose, all containers are started by default, so `wake_on_start` must also be true when using this image. `wake_on_crash` is also true as it is recommended to launch the Minecraft server with `restart: no`.

If you want more details or have issues, you can also refer to the lazymc [documentation](https://github.com/timvisee/lazymc/tree/master).

Also, refer to the lazymc [config example](https://github.com/timvisee/lazymc/blob/master/res/lazymc.toml). You may notice that the environment variables are named in the same way. This is intentional...
