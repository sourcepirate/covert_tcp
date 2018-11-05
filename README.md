## Covert TCP

A experiment to send data to other host by hiding the data inside of ip identification header.

An Implementation in Rust.

## Demo Sending

```
# Send the data from source to destination 1 packet at a time.
RUST_LOG=info ./covert_tcp send 127.0.0.1:8000 127.0.0.1:8001 hello.txt

# For listening to tcp connections on IPV4 transport.
RUST_LOG=info ./covert_tcp recv lo0

```

## License
MIT