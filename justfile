build-release:
  # build this on ubusrv1
  # with the musl target for static linking because k3svc3.lan has an old glibc
  cargo build --release --target x86_64-unknown-linux-musl

copy-from-ubusrv1:
   scp ubusrv1.lan:/home/lex/rs/immich-refresh/target/x86_64-unknown-linux-musl/release/immich-refresh k3svc3.lan:/opt/immich-refresh
