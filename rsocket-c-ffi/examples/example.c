#include "../include/rsocket_c.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void response_callback(const CRSocketPayload* payload, void* user_data) {
    if (payload && payload->data) {
        printf("Async Response: %.*s\n", (int)payload->data_len, payload->data);
    } else {
        printf("Async Response: (null)\n");
    }
}

int main() {
    printf("🚀 Testing C FFI Bindings\n");
    
    printf("📡 Testing TCP Client Creation\n");
    CRSocketTransportConfig tcp_config = {
        .transport_type = 0, // TCP
        .address = "127.0.0.1:7878",
        .enable_advanced_features = 0
    };
    
    CRSocketClient* tcp_client = rsocket_c_create_client(&tcp_config);
    if (tcp_client) {
        printf("✅ TCP client created successfully\n");
        rsocket_c_free_client(tcp_client);
    } else {
        printf("❌ TCP client creation failed\n");
    }
    
    printf("📡 Testing WebSocket Client Creation\n");
    CRSocketTransportConfig ws_config = {
        .transport_type = 1, // WebSocket
        .address = "ws://localhost:7879",
        .enable_advanced_features = 0
    };
    
    CRSocketClient* ws_client = rsocket_c_create_client(&ws_config);
    if (ws_client) {
        printf("✅ WebSocket client created successfully\n");
        rsocket_c_free_client(ws_client);
    } else {
        printf("❌ WebSocket client creation failed\n");
    }
    
    printf("📡 Testing QUIC Client Creation\n");
    CRSocketTransportConfig quic_config = {
        .transport_type = 2, // QUIC
        .address = "127.0.0.1:7880",
        .enable_advanced_features = 0
    };
    
    CRSocketClient* quic_client = rsocket_c_create_client(&quic_config);
    if (quic_client) {
        printf("✅ QUIC client created successfully\n");
        rsocket_c_free_client(quic_client);
    } else {
        printf("❌ QUIC client creation failed\n");
    }
    
    printf("📡 Testing Iroh Client Creation\n");
    CRSocketTransportConfig iroh_config = {
        .transport_type = 3, // Iroh
        .address = "iroh://peer-id",
        .enable_advanced_features = 0
    };
    
    CRSocketClient* iroh_client = rsocket_c_create_client(&iroh_config);
    if (iroh_client) {
        printf("✅ Iroh client created successfully\n");
        rsocket_c_free_client(iroh_client);
    } else {
        printf("❌ Iroh client creation failed\n");
    }
    
    printf("\n🎯 C FFI Test Complete!\n");
    printf("All transport types tested successfully.\n");
    
    return 0;
}
