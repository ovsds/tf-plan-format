FROM ubuntu:22.04

COPY setup.sh .
RUN bash ./setup.sh
RUN rm setup.sh

ENV PATH="/root/.local/bin:/root/.local/share/mise/shims:$PATH"
