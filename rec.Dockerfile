FROM ghcr.io/charmbracelet/vhs

RUN apt-get install -y git

COPY target/debug/gitu /bin/gitu
