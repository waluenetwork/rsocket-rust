#[macro_use]
extern crate log;

use rsocket_rust::{prelude::*, Client};
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use rsocket_rust_transport_quinn::QuinnClientTransport;
use core::num;
use std::env;
use std::time::Instant;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Semaphore;
use futures::future::join_all;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let args: Vec<String> = env::args().collect();
    
    println!("{:?}", args);

    let rsocket_method = args[1].clone(); // serial_loop, concurrent_unbounded concurrent_controlled
    let performance_test = args[2].clone();
    let url = args[3].clone(); // 127.0.0.1:7878
    let transport_name = args[4].clone(); // tcp or websocket    
    let num_requests = args[5].parse::<i32>().unwrap(); // request count

    let client; 

    if transport_name == "tcp" {
        client = RSocketFactory::connect()
            .transport(TcpClientTransport::from(url))
            .acceptor(Box::new(|| {
                // Return a responder.
                Box::new(EchoRSocket)
            }))
            .start()
            .await
            .expect("Connect failed!");
    } else if transport_name == "websocket" {
        // let addr1 = format!("ws://{}", url);
        // let addr: &str = addr1.as_str();
        let addr: &str = &*format!("ws://{}", url);
        client = RSocketFactory::connect()
            .transport(WebsocketClientTransport::from(addr))
            .acceptor(Box::new(|| {
                // Return a responder.
                Box::new(EchoRSocket)
            }))
            .start()
            .await
            .expect("Connect failed!");
    } else if transport_name == "quic" {
        client = RSocketFactory::connect()
            .transport(QuinnClientTransport::from(url))
            .acceptor(Box::new(|| Box::new(EchoRSocket)))
            .start()
            .await
            .expect("Failed to connect to QUIC server!");

    } else {
        return Err(anyhow::anyhow!("Unsupported transport: {}", transport_name));
    }


    if performance_test == "serial_loop"{
        if rsocket_method == "request_response" {
            serial_loop_rr(client, transport_name, num_requests).await
        } else if rsocket_method == "fire_and_forget" {
            serial_loop_fnf(client, transport_name, num_requests).await
        } else {
            return Err(anyhow::anyhow!("Unsupported rsocket method: {}", rsocket_method));
        }
    } else if performance_test == "concurrent_unbounded"{
        if rsocket_method == "request_response" {
            concurrent_unbounded_rr(client, transport_name, num_requests).await
        } else if rsocket_method == "fire_and_forget" {
            concurrent_unbounded_fnf(client, transport_name, num_requests).await
        } else {
            return Err(anyhow::anyhow!("Unsupported rsocket method: {}", rsocket_method));
        }
    } else if performance_test == "concurrent_controlled"{
        let num_of_parallelism = args.get(5)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100); // default is 1

        if rsocket_method == "request_response" {
            concurrent_controlled_rr(client, transport_name, num_requests, num_of_parallelism).await
        } else if rsocket_method == "fire_and_forget" {
            concurrent_controlled_fnf(client, transport_name, num_requests, num_of_parallelism).await
        } else {
            return Err(anyhow::anyhow!("Unsupported rsocket method: {}", rsocket_method));
        }
    }
    else {
        return Err(anyhow::anyhow!("Unsupported performance test: {}", performance_test));
    }
}



async fn serial_loop_rr(client: Client, transport_name: String, num_requests: i32) -> Result<()> {
    
    // let num_requests = 10000;
    // let transport_name = "tcp";
    
    let req = Payload::builder().set_data_utf8(&*format!("Benchmark payload from {}", transport_name)).build();

    let start = Instant::now();

    for i in 0..num_requests {
        match client.request_response(req.clone()).await {
            Ok(response) => {
                // You can log every N responses to avoid flooding
                if i % 1000 == 0 {
                    info!("✅ Request {} succeeded: {:?}", i + 1, response);
                }
            }
            Err(e) => {
                error!("❌ Request {} failed: {}", i + 1, e);
                break;
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();
    let rps = num_requests as f64 / duration;

    println!("✅ {}: {:.3}s total, {:.1} req/s", transport_name, duration, rps);

    Ok(())

}


async fn serial_loop_fnf(client: Client, transport_name: String, num_requests: i32) -> Result<()> {
    
    // let num_requests = 10000;
    // let transport_name = "tcp";
    
    let req = Payload::builder().set_data_utf8(&*format!("Benchmark payload from {}", transport_name)).build();

    let start = Instant::now();

    for i in 0..num_requests {
        match client.fire_and_forget(req.clone()).await {
            Ok(()) => {
                if i % 1000 == 0 {
                    info!("✅ Sent fire-and-forget {}", i + 1);
                }
            }
            Err(e) => {
                error!("❌ Failed to send fire-and-forget {}: {}", i + 1, e);
                break;
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();
    let rps = num_requests as f64 / duration;

    println!("✅ {}: {:.3}s total, {:.1} req/s", transport_name, duration, rps);

    Ok(())

}



async fn concurrent_unbounded_rr(client: Client, transport_name: String, num_requests: i32) -> Result<()> {
    let start = Instant::now();
    let mut tasks = FuturesUnordered::new();

    for i in 0..num_requests {
        let req = Payload::builder().set_data_utf8(&format!("Benchmark payload {} from {}", i, transport_name)).build();

        let client = client.clone(); // make sure client is cloneable or use Arc
        let payload = req.clone();
        tasks.push(tokio::spawn(async move {
            client.request_response(payload).await
        }));
    }

    let mut success = 0;

    while let Some(result) = tasks.next().await {
        match result {
            Ok(Ok(_)) => success += 1,
            Ok(Err(e)) => eprintln!("❌ error: {}", e),
            Err(join_err) => eprintln!("❌ join error: {}", join_err),
        }
    }

    let duration = start.elapsed().as_secs_f64();
    let rps = success as f64 / duration;
    println!("✅ Concurrent TCP: {:.3}s total, {:.1} req/s", duration, rps);


    Ok(())
}

async fn concurrent_unbounded_fnf(client: Client, transport_name: String, num_requests: i32) -> Result<()> {
    let start = Instant::now();
    let mut tasks = FuturesUnordered::new();

    for i in 0..num_requests {
        let req = Payload::builder().set_data_utf8(&format!("Benchmark payload {} from {}", i, transport_name)).build();

        let client = client.clone(); // make sure client is cloneable or use Arc
        let payload = req.clone();
        tasks.push(tokio::spawn(async move {
            client.fire_and_forget(payload).await
        }));
    }

    let mut success = 0;

    while let Some(result) = tasks.next().await {
        match result {
            Ok(Ok(_)) => success += 1,
            Ok(Err(e)) => eprintln!("❌ error: {}", e),
            Err(join_err) => eprintln!("❌ join error: {}", join_err),
        }
    }
    
    let failed = num_requests - success;
    println!("✅ Sent: {}, ❌ Failed: {}", success, failed);
    
    let duration = start.elapsed().as_secs_f64();
    let rps = success as f64 / duration;
    println!("✅ Concurrent TCP: {:.3}s total, {:.1} req/s", duration, rps);

    Ok(())
}



async fn concurrent_controlled_rr(client: Client, transport_name: String, num_requests: i32, num_of_parallelism: usize) -> Result<()> {

    let req = Payload::builder().set_data_utf8(&*format!("Benchmark payload from {}", transport_name)).build();


    let semaphore = Arc::new(Semaphore::new(num_of_parallelism)); // max 100 concurrent requests
    let start = Instant::now();
    let mut handles = Vec::with_capacity(num_requests.try_into().unwrap());

    for _ in 0..num_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let payload = req.clone();
        let handle = tokio::spawn(async move {
            let _permit = permit;
            client.request_response(payload).await
        });
        handles.push(handle);
    }

    let results = join_all(handles).await;
    let ok_count = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

    let duration = start.elapsed().as_secs_f64();
    let rps = ok_count as f64 / duration;

    println!("✅ Concurrent TCP: {:.3}s total, {:.1} req/s", duration, rps);

    Ok(())
}

async fn concurrent_controlled_fnf(client: Client, transport_name: String, num_requests: i32, num_of_parallelism: usize) -> Result<()> {

    let req: Payload = Payload::builder().set_data_utf8(&*format!("Benchmark payload from {}", transport_name)).build();


    let semaphore = Arc::new(Semaphore::new(num_of_parallelism)); // max 100 concurrent requests
    let start = Instant::now();
    let mut handles = Vec::with_capacity(num_requests.try_into().unwrap());

    for _ in 0..num_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let payload = req.clone();
        let handle = tokio::spawn(async move {
            let _permit = permit;
            client.request_response(payload).await
        });
        handles.push(handle);
    }

    let results = join_all(handles).await;
    let ok_count = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

    let duration = start.elapsed().as_secs_f64();
    let rps = ok_count as f64 / duration;

    println!("✅ Concurrent TCP: {:.3}s total, {:.1} req/s", duration, rps);

    Ok(())
}
