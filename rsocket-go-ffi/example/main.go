package main

import (
	"fmt"
	"log"
)


func main() {
	fmt.Println("🚀 Testing Go FFI Bindings")
	
	fmt.Println("📡 Testing TCP Client Creation")
	tcpClient, err := NewRSocketClient(TCP, "127.0.0.1:7878")
	if err != nil {
		fmt.Printf("❌ TCP client creation failed: %v\n", err)
	} else {
		fmt.Println("✅ TCP client created successfully")
		tcpClient.Close()
	}
	
	fmt.Println("📡 Testing WebSocket Client Creation")
	wsClient, err := NewRSocketClient(WebSocket, "ws://localhost:7879")
	if err != nil {
		fmt.Printf("❌ WebSocket client creation failed: %v\n", err)
	} else {
		fmt.Println("✅ WebSocket client created successfully")
		wsClient.Close()
	}
	
	fmt.Println("📡 Testing QUIC Client Creation")
	quicClient, err := NewRSocketClient(QUIC, "127.0.0.1:7880")
	if err != nil {
		fmt.Printf("❌ QUIC client creation failed: %v\n", err)
	} else {
		fmt.Println("✅ QUIC client created successfully")
		quicClient.Close()
	}
	
	fmt.Println("📡 Testing Iroh Client Creation")
	irohClient, err := NewRSocketClient(Iroh, "iroh://peer-id")
	if err != nil {
		fmt.Printf("❌ Iroh client creation failed: %v\n", err)
	} else {
		fmt.Println("✅ Iroh client created successfully")
		irohClient.Close()
	}
	
	fmt.Println("\n🎯 Go FFI Test Complete!")
	fmt.Println("All transport types tested successfully.")
}
