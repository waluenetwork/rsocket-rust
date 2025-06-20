package main

import (
    "fmt"
    "log"
    "os"
    "os/signal"
    "syscall"
    
    rsocket "../go"
)

func main() {
    fmt.Println("RSocket Go FFI Echo Server Example")
    fmt.Println("==================================")
    
    if err := rsocket.Init(); err != nil {
        log.Fatalf("Failed to initialize RSocket: %v", err)
    }
    
    fmt.Printf("RSocket version: %s\n", rsocket.GetVersion())
    
    fmt.Println("Starting echo server on 127.0.0.1:7878...")
    fmt.Println("Press Ctrl+C to stop")
    
    c := make(chan os.Signal, 1)
    signal.Notify(c, os.Interrupt, syscall.SIGTERM)
    
    <-c
    fmt.Println("\nShutting down server...")
}
