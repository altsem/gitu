#!/bin/sh

podman build -f rec.Dockerfile -t rec .
podman run -v "$PWD":/vhs/ --rm rec -o rec.gif rec.tape
