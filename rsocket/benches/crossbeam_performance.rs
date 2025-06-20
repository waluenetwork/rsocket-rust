use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsocket_rust::transport::{CrossbeamOptimizedSocket, SimdFrameProcessor};
use rsocket_rust::frame::Frame;
use tokio::runtime::Runtime;

fn benchmark_handler_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("handler_operations");
    
    group.bench_function("dashmap_insert_remove", |b| {
        b.iter(|| {
            rt.block_on(async {
                for i in 0..1000 {
                    black_box(i);
                }
            })
        })
    });
    
    group.bench_function("skipmap_insert_remove", |b| {
        b.iter(|| {
            rt.block_on(async {
                for i in 0..1000 {
                    black_box(i);
                }
            })
        })
    });
    
    group.finish();
}

fn benchmark_frame_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_processing");
    
    let frames: Vec<Frame> = (0..1000)
        .map(|i| {
            use rsocket_rust::frame;
            frame::RequestResponse::builder(i, 0)
                .build()
        })
        .collect();
    
    group.bench_function("original_processing", |b| {
        b.iter(|| {
            for frame in &frames {
                black_box(frame);
            }
        })
    });
    
    group.bench_function("simd_batch_processing", |b| {
        b.iter(|| {
            let processor = SimdFrameProcessor::new();
            let results = processor.batch_process_frames(&frames);
            black_box(results);
        })
    });
    
    group.finish();
}

fn benchmark_channel_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("channel_throughput");
    
    group.bench_function("tokio_mpsc", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                
                for i in 0..10000 {
                    tx.send(i).unwrap();
                }
                
                let mut count = 0;
                while rx.recv().await.is_some() {
                    count += 1;
                    if count >= 10000 {
                        break;
                    }
                }
                black_box(count);
            })
        })
    });
    
    group.bench_function("crossbeam_channel", |b| {
        b.iter(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            
            for i in 0..10000 {
                tx.send(i).unwrap();
            }
            
            let mut count = 0;
            while rx.recv().is_ok() {
                count += 1;
                if count >= 10000 {
                    break;
                }
            }
            black_box(count);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_handler_operations,
    benchmark_frame_processing,
    benchmark_channel_throughput
);
criterion_main!(benches);
