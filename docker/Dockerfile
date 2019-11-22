FROM rust:1.39.0-stretch as rust
RUN git clone https://github.com/hurwitzlab/pear.git
RUN cd pear/run_pear && cargo build --release && cp target/release/run_pear /usr/local/bin

FROM ubuntu:latest
RUN apt-get update -y
RUN apt-get install -y parallel libgomp1
RUN mkdir .parallel && touch .parallel/will-cite
COPY --from=rust /usr/local/bin /usr/local/bin
COPY --from=rust /usr/local/lib /usr/local/lib
ENV LD_LIBRARY_PATH /usr/local/lib

WORKDIR /app
COPY pear-0.9.11-linux-x86_64 /app/pear/

ENV PATH=/app/pear/bin:$PATH

CMD ["run_pear"]
