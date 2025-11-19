# Dispatcher

> Intelligent delivery order distribution engine.

A highly abstract and extensible delivery order distribution engine Rust library, specially designed for delivery scenarios such as takeout, express delivery, and taxi hailing, and supports a variety of intelligent order dispatch strategies.



## Build & Run

```bash
# compile
cargo build # or cargo build --release

# run main program
cargo run

# run examples
cargo run --example basic
cargo run --example food_delivery
cargo run --example ride_hailing

# run test
cargo test

# generate API docs
cargo doc --open --no-deps
```



## API Test

```bash
# run backend server
cargo run --bin server # for dev mode
cargo run --bin server --release # for release mode

# health check
curl http://127.0.0.1:8080/health

# fetch drivers
curl http://127.0.0.1:8080/api/drivers

# test dispatch
curl -X POST http://127.0.0.1:8080/api/dispatch \
  -H "Content-Type: application/json" \
  -d '{"order_type":"外卖订单","pickup_lat":39.9050,"pickup_lng":116.4080,"delivery_lat":39.9150,"delivery_lng":116.4180}'
  
# change strategy
curl -X POST http://127.0.0.1:8080/api/strategy \
  -H "Content-Type: application/json" \
  -d '{"strategy": "load_balanced"}'
```



## Web Demo

```bash
# Method 1: Use Python 3
cd web
python3 -m http.server 8000
# browser access http://localhost:8000

# Method 2: Just open index.html
```

![](https://github.com/i0Ek3/dispatcher/blob/main/assets/images/dispatcher.jpg)