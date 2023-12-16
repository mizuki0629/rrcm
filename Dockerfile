ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

RUN echo "dash dash/sh boolean false" | debconf-set-selections
RUN DEBIAN_FRONTEND=noninteractive dpkg-reconfigure dash
