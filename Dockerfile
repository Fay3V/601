# Dockerfile to build Python 2.6.6 with working _ssl
FROM ubuntu:14.04

# Prevent prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive


COPY 601.sources.list /etc/apt/sources.list.d

# RUN sed -i -e "s/archive.ubuntu.com/old-releases.ubuntu.com/g" /etc/apt/sources.list

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    unzip \
    wget \
    libssl-dev \
    libbz2-dev \
    libreadline-dev \
    libsqlite3-dev \
    libncurses5-dev \
    zlib1g-dev \
    libffi-dev \
    python-setuptools \
    python-dev \
    python-pip \
    tar \ 
    libblas-dev \
    liblapack-dev \
    gfortran \
    libfreetype6-dev \
    libpng-dev \
    tk-dev \
    tcl-dev \
    # python2.7 python*2.7-tk python2.7-numpy \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp

COPY patches /tmp/patches

RUN wget https://www.python.org/ftp/python/2.6.6/Python-2.6.6.tgz && tar -xzf Python-2.6.6.tgz

WORKDIR /tmp/Python-2.6.6

RUN cp /tmp/patches/010_ssl_no_ssl2_no_ssl3.patch . && \
    cp /tmp/patches/002_readline63.patch . && \
    cp /tmp/patches/003_tk86.patch . && \
    patch -p1 < 010_ssl_no_ssl2_no_ssl3.patch && \
    patch -p1 < 002_readline63.patch && \
    patch -p1 < 003_tk86.patch && \
    export CFLAGS="-I/usr/include -I/usr/include/openssl -I/usr/include/x86_64-linux-gnu" && \
    export LDFLAGS="-L/usr/lib/x86_64-linux-gnu -L/usr/lib" && \
    ./configure --prefix=/usr/local/python2.6 && \
    make -j$(nproc) && make install

ENV PATH="/usr/local/python2.6/bin:$PATH"

RUN wget https://files.pythonhosted.org/packages/source/s/setuptools/setuptools-36.8.0.zip && \
    unzip setuptools-36.8.0.zip && \
    cd setuptools-36.8.0 && \
    python2.6 setup.py install

RUN cd /tmp && \
    wget https://github.com/numpy/numpy/archive/refs/tags/v1.8.2.tar.gz && \
    tar -xzf v1.8.2.tar.gz && \
    cd numpy-1.8.2 && \
    python2.6 setup.py build && \
    python2.6 setup.py install

RUN wget https://files.pythonhosted.org/packages/source/p/pycparser/pycparser-2.14.tar.gz && \
    tar -xzf pycparser-2.14.tar.gz && \
    cd pycparser-2.14 && \
    python2.6 setup.py install

RUN wget https://files.pythonhosted.org/packages/source/c/cffi/cffi-1.11.5.tar.gz && \
    tar -xzf cffi-1.11.5.tar.gz && \
    cd cffi-1.11.5 && \
    python2.6 setup.py build && \
    python2.6 setup.py install 
 
# Copy your tar.gz into the container
COPY lab/afbbebccae39bfa42f9d071e9ed10453_lib601-3-500.tar.gz /tmp/lib601-3-500.tar.gz

WORKDIR /tmp

# Extract and install
RUN tar -xzf  lib601-3-500.tar.gz && \
    cd lib601-3-500 && \
    python setup.py install

WORKDIR /course

# Set default command
CMD [ "bash" ]

