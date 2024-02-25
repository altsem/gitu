#!/bin/sh

podman build -f vhs/rec.Dockerfile -t rec .
podman run -v "$PWD"/vhs:/vhs/ --rm rec
