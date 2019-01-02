#!/bin/bash

dir=`pwd` && docker run -ti --rm -v $dir/examples/:/data/ -v $dir/docker_start.sh:/data/docker_start.sh zhrong/bft /data/docker_start.sh && tail -f /tmp/c2.log
