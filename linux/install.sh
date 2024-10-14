#!/bin/bash

mkdir ~/.mozilla/native-messaging-hosts

path="$(pwd)"

if [[ $1 == "-r" ]]; then
	path="$path/../target/release"
fi

sed "s|insert_path|$path/eeing|" eeing.json > ~/.mozilla/native-messaging-hosts/eeing.json
