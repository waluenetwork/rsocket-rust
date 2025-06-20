package main

import (
    "fmt"
    "log"
    "sync"
    "time"
    
    rsocket "../go"
)

func main() {
    fmt.Println("RSocket Go FFI Performance Benchmark")
    fmt.Println("====================================")
    
    if err := rsocket.Init(); err != nil {
        log.Fatalf("Failed to initialize RSocket: %v", err)
    }
    
    client := rsocket.NewClient()
    if client == nil {
        log.Fatal("Failed to create client")
    }
    
    if err := client.ConnectTCP("127.0.0.1:7878"); err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    
    const numRequests = 1000
    const messageSize = 1024
    
    message := make([]byte, messageSize)
    for i := range message {
        message[i] = byte(i % 256)
    }
    
    fmt.Printf("Sending %d requests with %d byte payloads...\n", numRequests, messageSize)
    
    start := time.Now()
    var wg sync.WaitGroup
    var successCount int64
    var errorCount int64
    var mu sync.Mutex
    
    for i := 0; i < numRequests; i++ {
        wg.Add(1)
        
        payload := rsocket.NewPayloadFromBytes(message, nil)
        if payload == nil {
            log.Fatal("Failed to create payload")
        }
        
        err := client.RequestResponse(payload, func(response *rsocket.Payload, err error) {
            defer wg.Done()
            
            mu.Lock()
            if err != nil {
                errorCount++
            } else {
                successCount++
            }
            mu.Unlock()
        })
        
        if err != nil {
            wg.Done()
            mu.Lock()
            errorCount++
            mu.Unlock()
        }
    }
    
    wg.Wait()
    duration := time.Since(start)
    
    fmt.Printf("\nBenchmark Results:\n")
    fmt.Printf("Duration: %v\n", duration)
    fmt.Printf("Successful requests: %d\n", successCount)
    fmt.Printf("Failed requests: %d\n", errorCount)
    fmt.Printf("Requests per second: %.2f\n", float64(successCount)/duration.Seconds())
    fmt.Printf("Average latency: %v\n", duration/time.Duration(successCount))
}
