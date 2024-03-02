FROM ghcr.io/charmbracelet/vhs

RUN apt-get update && apt-get install -y git neovim
ENV EDITOR=nvim
RUN git config --global user.email "you@example.com"
RUN git config --global user.name "Your Name"

COPY target/debug/gitu /bin/gitu
RUN git clone https://github.com/altsem/gitu.git /gitu
WORKDIR /gitu

ENTRYPOINT vhs -o /vhs/rec.gif /vhs/rec.tape
