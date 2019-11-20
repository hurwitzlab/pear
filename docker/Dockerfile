FROM ubuntu:latest
RUN apt-get -y update
#RUN apt-get install -y parallel wget git

WORKDIR /app
COPY pear-0.9.11-linux-x86_64 /app/pear/

ENV PATH=/app/pear/bin:$PATH

CMD ["pear"]
