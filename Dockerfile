FROM fedora:33
WORKDIR /usr/local/bin
COPY ./target/release/latex_microservice /usr/local/bin/latex_microservice
RUN dnf install texlive-full -y
RUN dnf install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["latex_microservice"]
