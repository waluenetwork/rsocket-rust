use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🚀 Multi-Language FFI Demo");
    println!("Testing all language bindings for RSocket Rust");
    
    println!("\n📦 Building all FFI packages...");
    
    let ffi_packages = [
        "rsocket_ffi_core",
        "rsocket_python_ffi", 
        "rsocket_js_ffi",
        "rsocket_c_ffi",
        "rsocket_go_ffi",
    ];
    
    for package in &ffi_packages {
        println!("Building {}...", package);
        let output = Command::new("cargo")
            .args(&["build", "-p", package])
            .output()
            .expect("Failed to execute cargo build");
        
        if output.status.success() {
            println!("✅ {} built successfully", package);
        } else {
            println!("❌ {} build failed", package);
            println!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    println!("\n🎯 Multi-Language FFI Demo Complete!");
    println!("All FFI packages have been tested for compilation.");
    println!("\nTo test individual language bindings:");
    println!("- Python: cd rsocket-python-ffi && make test");
    println!("- JavaScript: cd rsocket-js-ffi && make test");
    println!("- C/C++: cd rsocket-c-ffi && make test");
    println!("- Go: cd rsocket-go-ffi && make test");
}
