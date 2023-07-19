FROM docker.io/library/rust:bullseye as builder

RUN apt-get update && \
    apt-get install --yes \
      cmake \
      make \
      libzstd-dev \
      wget

RUN git clone https://github.com/google/flatbuffers.git && \
    cd flatbuffers && \
    git checkout v22.9.29 && \
    cmake . && \
    make -j flatc && \
    mv flatc /usr/bin

ENV TDENGINE_VERSION=3.0.4.2
RUN wget -c https://www.taosdata.com/assets-download/3.0/TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz \
   && tar xvf TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz \
   && cd TDengine-client-${TDENGINE_VERSION} \
   && ./install_client.sh \
   && cd ../ \
   && rm -rf TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz TDengine-client-${TDENGINE_VERSION}

COPY . .
RUN cargo install \
   --path . \
   --root /usr/local

FROM docker.io/library/debian:bullseye-slim
RUN apt-get update && \
    apt-get install --yes \
      tini \
      wget

ENV TDENGINE_VERSION=3.0.4.2
RUN wget -c https://www.taosdata.com/assets-download/3.0/TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz \
   && tar xvf TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz \
   && cd TDengine-client-${TDENGINE_VERSION} \
   && ./install_client.sh \
   && cd ../ \
   && rm -rf TDengine-client-${TDENGINE_VERSION}-Linux-x64.tar.gz TDengine-client-${TDENGINE_VERSION}

COPY --from=builder /usr/local/bin/trace-archiver /usr/local/bin/trace-archiver
WORKDIR /root
RUN mkdir data
COPY .env .env

ENTRYPOINT ["/usr/bin/tini", "--"]
