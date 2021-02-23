FROM debian:buster-slim
WORKDIR /usr/local/bin
COPY ./target/release/latex_microservice /usr/local/bin/latex_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install texlive-full -y
RUN apt-get install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["latex_microservice"]