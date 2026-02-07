FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl jq
RUN curl -sSL https://raw.githubusercontent.com/takumi3488/gqlforge/master/install.sh | bash -s
ENV PATH="${PATH}:~/.gqlforge/bin"
