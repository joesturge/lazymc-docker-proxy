#!/usr/bin/env bash

start_timestamp=""

# set up start timestamp
reset_timestamp() {
	start_timestamp=$(date -u +"%Y-%m-%d %H:%M:%S")
}

# before all tests
setup() {
	reset_timestamp

	# ensure user systemd container directory exists
	mkdir --parents ~/.config/containers/systemd/

	# copy all units to systemd search path
	for unit in "${units[@]}"; do
		# copy file
		cp "$project/$unit" ~/.config/containers/systemd/"$unit"

		# substitute in lazymc-docker-proxy build location
		sed -i -e "s|\[\[LAZYMC_DOCKER_PROXY_DIR\]\]|$PWD|g" ~/.config/containers/systemd/"$unit"
	done

	# reload systemd units
	systemctl --user daemon-reload

	# start systemd services
	for service in "${services[@]}"; do
		# start service
		systemctl --user start "$service"
	done
}

# after all tests
teardown() {
	# stop systemd services
	for service in "${services[@]}"; do
		# stop service
		systemctl --user kill "$service"
	done

	# remove all units from systemd search path
	for unit in "${units[@]}"; do
		rm ~/.config/containers/systemd/"$unit"
	done

	# reload systemd units
	systemctl --user daemon-reload
}

# wait for a log line to appear in a container"s logs
wait_for_formatted_log() {
	local container=$1
	local level=$2
	local target=$3
	local logline=$4
	local timeout=${5:-60}

	local regex="$level"'\s+'"$target"'\s+\>\s+'"$logline"

	echo "Waiting: $container | $level $target > $logline" >&3

	trap 'exit 1' SIGINT SIGTERM
	until journalctl --user --since "$start_timestamp" -u "$container" 2> /dev/null | grep -qP "$regex"; do
		if [ $timeout -eq 0 ]; then
			echo "Timeout waiting for log: $logline" >&3
			exit 1
		fi
		sleep 1
		((timeout--))
	done
}

# wait for a log line to appear in a container's logs
wait_for_log() {
	local container=$1
	local logline=$2
	local timeout=${3:-60}

	echo "Waiting: $container | $logline" >&3

	trap 'exit 1' SIGINT SIGTERM
	until journalctl --user --since "$start_timestamp" -u "$container" 2> /dev/null | grep -q "$logline"; do
		if [ $timeout -eq 0 ]; then
			echo "Timeout waiting for log: $logline" >&3
			exit 1
		fi
		sleep 1
		((timeout--))
	done
}

restart_container() {
    local container=$1

    echo "Restarting container: $container" >&3
    systemctl --user restart "$container"
}