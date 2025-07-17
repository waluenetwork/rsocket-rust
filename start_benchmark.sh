


cargo run --example cli-performance-test request_response serial_loop 127.0.0.1:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test request_response serial_loop 127.0.0.1:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test request_response serial_loop 127.0.0.1:7880 quic 10000 2> /dev/null

# cargo run --example cli-performance-test request_response serial_loop 3.72.75.86:7878 tcp 10000 2> /dev/null
# cargo run --example cli-performance-test request_response serial_loop 3.72.75.86:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test request_response serial_loop 3.72.75.86:7880 quic 10000 2> /dev/null


cargo run --example cli-performance-test request_response concurrent_unbounded 127.0.0.1:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test request_response concurrent_unbounded 127.0.0.1:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test request_response concurrent_unbounded 127.0.0.1:7880 quic 10000 2> /dev/null

cargo run --example cli-performance-test request_response concurrent_unbounded 3.72.75.86:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test request_response concurrent_unbounded 3.72.75.86:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test request_response concurrent_unbounded 3.72.75.86:7880 quic 10000 2> /dev/null



cargo run --example cli-performance-test request_response concurrent_controlled 127.0.0.1:7878 tcp 10000 100 2> /dev/null
cargo run --example cli-performance-test request_response concurrent_controlled 127.0.0.1:7879 websocket 10000 100 2> /dev/null
# cargo run --example cli-performance-test request_response concurrent_controlled 127.0.0.1:7880 quic 10000 100 2> /dev/null

cargo run --example cli-performance-test request_response concurrent_controlled 3.72.75.86:7878 tcp 10000 100 2> /dev/null
cargo run --example cli-performance-test request_response concurrent_controlled 3.72.75.86:7879 websocket 10000 100 2> /dev/null
# cargo run --example cli-performance-test request_response concurrent_controlled 3.72.75.86:7880 quic 10000 100 2> /dev/null











cargo run --example cli-performance-test fire_and_forget serial_loop 127.0.0.1:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test fire_and_forget serial_loop 127.0.0.1:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget serial_loop 127.0.0.1:7880 quic 10000 2> /dev/null

cargo run --example cli-performance-test fire_and_forget serial_loop 3.72.75.86:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test fire_and_forget serial_loop 3.72.75.86:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget serial_loop 3.72.75.86:7880 quic 10000 2> /dev/null


cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 127.0.0.1:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 127.0.0.1:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 127.0.0.1:7880 quic 10000 2> /dev/null

cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 3.72.75.86:7878 tcp 10000 2> /dev/null
cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 3.72.75.86:7879 websocket 10000 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget concurrent_unbounded 3.72.75.86:7880 quic 10000 2> /dev/null


cargo run --example cli-performance-test fire_and_forget concurrent_controlled 127.0.0.1:7878 tcp 10000 100 2> /dev/null
cargo run --example cli-performance-test fire_and_forget concurrent_controlled 127.0.0.1:7879 websocket 10000 100 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget concurrent_controlled 127.0.0.1:7880 quic 10000 100 2> /dev/null

cargo run --example cli-performance-test fire_and_forget concurrent_controlled 3.72.75.86:7878 tcp 10000 100 2> /dev/null
cargo run --example cli-performance-test fire_and_forget concurrent_controlled 3.72.75.86:7879 websocket 10000 100 2> /dev/null
# cargo run --example cli-performance-test fire_and_forget concurrent_controlled 3.72.75.86:7880 quic 10000 100 2> /dev/null





