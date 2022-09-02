FROM tindy2013/subconverter:latest
ARG TARGETARCH
ARG TARGETVARIANT

RUN mkdir -p /etc/axum-subconverter
WORKDIR /root/
ADD axum-subconverter-$TARGETARCH$TARGETVARIANT /usr/bin/axum-subconverter
COPY generate.ini /root/
RUN mkdir /root/nodes
EXPOSE 3000
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/bin/axum-subconverter", "--root", "/root"]
