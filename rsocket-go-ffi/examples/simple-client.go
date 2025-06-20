package main

import (
    "fmt"
    "log"
    "sync"
    "time"
    
    rsocket "../go"
)

func main() {
    fmt.Println("RSocket Go FFI Simple Client Example")
    fmt.Println("====================================")
    
    if err := rsocket.Init(); err != nil {
        log.Fatalf("Failed to initialize RSocket: %v", err)
    }
    
    fmt.Printf("RSocket version: %s\n", rsocket.GetVersion())
    fmt.Printf("Supported transports: %s\n", rsocket.GetSupportedTransports())
    
    client := rsocket.NewClient()
    if client == nil {
        log.Fatal("Failed to create client")
    }
    
    if err := client.ConnectTCP("127.0.0.1:7878"); err != nil {
        log.Fatalf("Failed to connect to server: %v", err)
    }
    
    fmt.Println("Connected to server")
    
    payload := rsocket.NewPayload("Hello from Go client!", "")
    if payload == nil {
        log.Fatal("Failed to create payload")
    }
    
    var wg sync.WaitGroup
    wg.Add(1)
    
    err := client.RequestResponse(payload, func(response *rsocket.Payload, err error) {
        defer wg.Done()
        
        if err != nil {
            fmt.Printf("Error: %v\n", err)
            return
        }
        
        if response != nil {
            fmt.Printf("Received response: %s\n", response.GetDataAsString())
        }
    })
    
    if err != nil {
        log.Fatalf("Failed to send request: %v", err)
    }
    
    wg.Wait()
    fmt.Println("Request completed")
}
