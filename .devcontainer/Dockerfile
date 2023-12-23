FROM cffnpwr/git as git


FROM rust:1.73

ARG UID=1000
ARG GID=1000
ARG USERNAME=developer

WORKDIR /workspace

COPY --from=git /usr/bin/git /usr/bin

RUN apt-get update && \
  apt-get install -y --no-install-recommends sudo && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/* && \
  groupadd --gid $GID $USERNAME && \
  useradd --uid $UID --gid $GID -m -s /bin/bash $USERNAME && \
  echo "${USERNAME}   ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers && \
  su ${USERNAME} -c "rustup default nightly" && \
  su ${USERNAME} -c "rustup component add rustfmt" && \
  su ${USERNAME} -c "cargo install cargo-watch"

USER ${USERNAME}
