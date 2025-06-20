use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub struct SIMDFrameProcessor {
    batch_size: usize,
    enable_simd: bool,
}

impl SIMDFrameProcessor {
    pub fn new(batch_size: usize) -> Self {
        let enable_simd = is_simd_supported();
        log::info!("SIMD support detected: {}", enable_simd);
        
        Self {
            batch_size,
            enable_simd,
        }
    }
    
    pub fn process_frame_batch_simd(&self, frames: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, JsValue> {
        if !self.enable_simd || frames.len() < 4 {
            return Ok(frames.to_vec());
        }
        
        let mut processed_frames = Vec::with_capacity(frames.len());
        
        for chunk in frames.chunks(4) {
            if chunk.len() == 4 {
                let simd_result = self.process_four_frames_simd(chunk)?;
                processed_frames.extend(simd_result);
            } else {
                processed_frames.extend_from_slice(chunk);
            }
        }
        
        Ok(processed_frames)
    }
    
    fn process_four_frames_simd(&self, frames: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, JsValue> {
        assert_eq!(frames.len(), 4);
        
        let mut processed = Vec::with_capacity(4);
        
        for frame in frames {
            if frame.len() >= 16 {
                let optimized_frame = self.simd_frame_transform(frame)?;
                processed.push(optimized_frame);
            } else {
                processed.push(frame.clone());
            }
        }
        
        Ok(processed)
    }
    
    fn simd_frame_transform(&self, frame: &[u8]) -> Result<Vec<u8>, JsValue> {
        let mut result = frame.to_vec();
        
        #[cfg(target_arch = "wasm32")]
        {
            if self.enable_simd {
                for chunk in result.chunks_exact_mut(16) {
                    for byte in chunk.iter_mut() {
                        *byte ^= 0x01;
                    }
                }
            }
        }
        
        Ok(result)
    }
}

pub fn is_simd_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::eval("typeof WebAssembly.SIMD !== 'undefined'")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
