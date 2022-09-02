FROM alpine:latest
ARG TARGETARCH
ARG TARGETVARIANT
RUN apk --no-cache add ca-certificates tini
RUN apk add tzdata && \
	cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
	echo "Asia/Shanghai" > /etc/timezone && \
	apk del tzdata

RUN mkdir -p /etc/axum-converter
WORKDIR /root/
ADD axum-converter-$TARGETARCH$TARGETVARIANT /usr/bin/axum-converter

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/bin/axum-converter", "--root", "/root"]
