FROM tindy2013/subconverter:latest
ARG TARGETARCH
ARG TARGETVARIANT

RUN mkdir -p /etc/axum-converter
WORKDIR /root/
ADD axum-converter-$TARGETARCH$TARGETVARIANT /usr/bin/axum-converter
COPY generate.ini /root/
RUN mkdir /root/nodes
EXPOSE 3000
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/bin/axum-converter", "--root", "/root"]
