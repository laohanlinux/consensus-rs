FROM zhrong/bft

MAINTAINER 0x80

COPY examples/c1.toml /data/
COPY examples/c2.toml /data/
COPY examples/c3.toml /data/
COPY examples/c4.toml /data/
COPY examples/c5.toml /data/
COPY examples/docker_build.sh /data/
COPY examples/docker_start.sh /data/

WORKDIR /root

RUN  /data/docker_start.sh && tail -f /tmp/c1.log
