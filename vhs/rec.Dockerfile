FROM ghcr.io/charmbracelet/vhs

RUN apt-get install -y git
COPY target/debug/gitu /bin/gitu

RUN git clone https://github.com/altsem/gitu.git /gitu
WORKDIR /gitu

ENTRYPOINT vhs -o /vhs/rec.gif /vhs/rec.tape
