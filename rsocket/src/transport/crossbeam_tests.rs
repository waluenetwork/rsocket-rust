#[cfg(test)]
mod tests {
    use crate::transport::{CrossbeamOptimizedSocket, SimdFrameProcessor};
    use crate::frame::Frame;
    use crate::payload::Payload;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_crossbeam_socket_basic_operations() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let socket = CrossbeamOptimizedSocket::new(1, tx, None);
        
        let frames: Vec<Frame> = (0..10)
            .map(|i| {
                use crate::frame;
                frame::RequestResponse::builder(i, 0)
                    .build()
            })
            .collect();
        
        let results = socket.process_frames_optimized(&frames);
        assert_eq!(results.len(), frames.len());
    }
    
    #[tokio::test]
    async fn test_simd_frame_processing() {
        let processor = SimdFrameProcessor::new();
        
        let frames: Vec<Frame> = (0..100)
            .map(|i| {
                use crate::frame;
                frame::RequestResponse::builder(i, 0)
                    .build()
            })
            .collect();
        
        let results = processor.batch_process_frames(&frames);
        
        assert_eq!(results.len(), frames.len());
        for result in results.iter() {
            assert!(!result.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_crossbeam_channel_performance() {
        let (tx, rx) = crossbeam_channel::unbounded();
        
        let sender_handle = tokio::spawn(async move {
            for i in 0..10000 {
                tx.send(i).unwrap();
            }
        });
        
        let receiver_handle = tokio::spawn(async move {
            let mut count = 0;
            while let Ok(_) = rx.recv() {
                count += 1;
                if count >= 10000 {
                    break;
                }
            }
            count
        });
        
        let sender_result = timeout(Duration::from_secs(5), sender_handle).await;
        let receiver_result = timeout(Duration::from_secs(5), receiver_handle).await;
        
        assert!(sender_result.is_ok());
        assert!(receiver_result.is_ok());
        assert_eq!(receiver_result.unwrap().unwrap(), 10000);
    }
    
    #[tokio::test]
    async fn test_concurrent_frame_processing() {
        use std::sync::Arc;
        use tokio::task::JoinSet;
        
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let socket = Arc::new(CrossbeamOptimizedSocket::new(1, tx, None));
        
        let mut join_set = JoinSet::new();
        
        for i in 0..100 {
            let socket_clone = socket.clone();
            join_set.spawn(async move {
                let frame = {
                    use crate::frame;
                    frame::RequestResponse::builder(i, 0).build()
                };
                
                socket_clone.add_frame_to_batch(frame);
                tokio::time::sleep(Duration::from_millis(1)).await;
                i
            });
        }
        
        let mut completed = 0;
        while let Some(result) = join_set.join_next().await {
            assert!(result.is_ok());
            completed += 1;
        }
        
        assert_eq!(completed, 100);
    }
}
