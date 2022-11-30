#!/bin/bash

path="$(pwd)/eeing"
sed "s|insert_path|$path|" eeing.json > ~/.mozilla/native-messaging-hosts/eeing.json
