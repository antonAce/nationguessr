cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/nationguessr target/bootstrap
zip -j target/lambda.zip target/bootstrap
