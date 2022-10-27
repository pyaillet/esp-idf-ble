ARG VARIANT=bullseye
FROM debian:${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Arguments
ARG CONTAINER_USER=esp
ARG CONTAINER_GROUP=esp
ARG TOOLCHAIN_VERSION=1.64.0.0
ARG ESP_IDF_VERSION=release/v4.4
ARG ESP_BOARD=esp32,esp32s2,esp32s3
ARG INSTALL_RUST_TOOLCHAIN=install-rust-toolchain.sh

# Install dependencies
RUN apt-get update \
    && apt-get install -y git curl gcc clang ninja-build libudev-dev unzip xz-utils tmux \
    python3 python3-pip python3-venv libusb-1.0-0 libssl-dev pkg-config libtinfo5 libpython2.7 \
    ninja-build gettext libtool libtool-bin autoconf automake cmake g++ doxygen ripgrep fzf socat \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts

RUN git clone https://github.com/neovim/neovim.git \
    && cd neovim && make CMAKE_BUILD_TYPE=RelWithDebInfo \
    && make install && pip3 install pynvim \
    && cd .. && rm -Rf neovim \
    && curl -fsSL https://deb.nodesource.com/setup_current.x | bash - \
    && apt-get update && apt-get install -y nodejs \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts \
    && curl -fsSL -o /tmp/go1.17.linux-amd64.tar.gz https://golang.org/dl/go1.17.linux-amd64.tar.gz \
    && tar -zxvf /tmp/go1.17.linux-amd64.tar.gz -C /usr/local/ \
    && echo "export PATH=/usr/local/go/bin:${PATH}" | tee /etc/profile.d/go.sh


# Set users
RUN adduser --disabled-password --gecos "" ${CONTAINER_USER}
USER ${CONTAINER_USER}
WORKDIR /home/${CONTAINER_USER}

# Install Rust toolchain, extra crates and esp-idf
ENV PATH=${PATH}:/home/${CONTAINER_USER}/.cargo/bin:/home/${CONTAINER_USER}/opt/bin

RUN cargo install espup --git https://github.com/esp-rs/espup \
    && espup install --espidf-version branch:${ESP_IDF_VERSION} --target ${ESP_BOARD} --toolchain-version ${TOOLCHAIN_VERSION}

# Install web-flash and wokwi-server
RUN curl -L https://github.com/bjoernQ/esp-web-flash-server/releases/latest/download/web-flash-x86_64-unknown-linux-gnu.zip \
    -o /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip \
    && unzip /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip \
    -d /home/${CONTAINER_USER}/.cargo/bin/ \
    && rm /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip
RUN chmod u+x /home/${CONTAINER_USER}/.cargo/bin/web-flash
RUN curl -L https://github.com/MabezDev/wokwi-server/releases/latest/download/wokwi-server-x86_64-unknown-linux-gnu.zip \
    -o /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip \
    && unzip /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip \
    -d /home/${CONTAINER_USER}/.cargo/bin/ \
    && rm /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip
RUN chmod u+x /home/${CONTAINER_USER}/.cargo/bin/wokwi-server
RUN curl -fLo ~/.config/nvim/init.vim --create-dirs https://raw.githubusercontent.com/charlyx/vim-config/master/init.vim \
    && curl -fLo ~/.config/nvim/autoload/plug.vim --create-dirs https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim \
    && mkdir -p ~/.cache/nvim

# Activate ESP environment
RUN echo "source /home/${CONTAINER_USER}/export-esp.sh" >> ~/.bashrc \
    && echo "alias vi=/usr/local/bin/nvim" >> ~/.bashrc \
    && echo "alias vim=/usr/local/bin/nvim" >> ~/.bashrc

CMD [ "/bin/bash" ]
