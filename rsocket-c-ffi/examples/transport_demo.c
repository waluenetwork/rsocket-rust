#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "rsocket_rust_c_ffi.h"

void demo_transport_utilities() {
    printf("RSocket C FFI Transport Demo\n");
    printf("============================\n");
    
    const char* supported = rsocket_get_supported_transports();
    printf("Supported transports: %s\n\n", supported);
    
    const char* transport_names[] = {"tcp", "websocket", "ws", "quic", "iroh-p2p", "unknown"};
    int num_transports = sizeof(transport_names) / sizeof(transport_names[0]);
    
    printf("Transport Support Matrix:\n");
    printf("%-12s %-15s %-10s\n", "Name", "Type", "Supported");
    printf("%-12s %-15s %-10s\n", "----", "----", "---------");
    
    for (int i = 0; i < num_transports; i++) {
        RSocketTransportType type = rsocket_parse_transport_type(transport_names[i]);
        int is_supported = rsocket_is_transport_supported(type);
        
        const char* type_name = rsocket_transport_type_to_string(type);
        
        printf("%-12s %-15s %-10s\n", 
               transport_names[i], 
               type_name,
               is_supported ? "Yes" : "No");
        
        rsocket_free_string((char*)type_name);
    }
    
    printf("\nTransport Type Enum Values:\n");
    printf("TCP: %d\n", (int)TCP);
    printf("WebSocket: %d\n", (int)WEBSOCKET);
    printf("QUIC: %d\n", (int)QUIC);
    printf("Iroh P2P: %d\n", (int)IrohP2P);
    
    rsocket_free_string((char*)supported);
}

void demo_client_connection_attempts() {
    printf("\nClient Connection Demo\n");
    printf("======================\n");
    
    RSocketClient* client = rsocket_client_create();
    if (!client) {
        printf("Failed to create client\n");
        return;
    }
    
    printf("Client created successfully\n");
    printf("Initial connection status: %s\n", 
           rsocket_client_is_connected(client) ? "Connected" : "Not connected");
    
    printf("\nAttempting TCP connection to 127.0.0.1:7878...\n");
    int tcp_result = rsocket_client_connect_tcp(client, "127.0.0.1:7878");
    printf("TCP connection result: %s\n", tcp_result == 0 ? "Success" : "Failed");
    printf("Connection status after TCP attempt: %s\n", 
           rsocket_client_is_connected(client) ? "Connected" : "Not connected");
    
    printf("\nAttempting WebSocket connection to ws://127.0.0.1:8080...\n");
    int ws_result = rsocket_client_connect_websocket(client, "ws://127.0.0.1:8080");
    printf("WebSocket connection result: %s\n", ws_result == 0 ? "Success" : "Failed");
    printf("Connection status after WebSocket attempt: %s\n", 
           rsocket_client_is_connected(client) ? "Connected" : "Not connected");
    
    printf("\nNote: Connection attempts failed because no servers are running.\n");
    printf("This is expected behavior for the demo.\n");
    
    rsocket_client_free(client);
    printf("Client freed successfully\n");
}

void demo_payload_operations() {
    printf("\nPayload Operations Demo\n");
    printf("=======================\n");
    
    const char* data = "Hello, RSocket C FFI!";
    const char* metadata = "demo-metadata";
    
    RSocketPayload* payload1 = rsocket_payload_create_from_string(data, metadata);
    if (!payload1) {
        printf("Failed to create payload from string\n");
        return;
    }
    
    printf("Created payload from string:\n");
    printf("  Data length: %zu\n", rsocket_payload_get_data_length(payload1));
    printf("  Metadata length: %zu\n", rsocket_payload_get_metadata_length(payload1));
    
    size_t data_len = rsocket_payload_get_data_length(payload1);
    char* buffer = malloc(data_len + 1);
    if (buffer) {
        size_t copied = rsocket_payload_copy_data(payload1, (uint8_t*)buffer, data_len);
        buffer[copied] = '\0';
        printf("  Copied data: \"%s\"\n", buffer);
        free(buffer);
    }
    
    rsocket_payload_free(payload1);
    
    uint8_t raw_data[] = {0x48, 0x65, 0x6C, 0x6C, 0x6F}; // "Hello"
    uint8_t raw_metadata[] = {0x01, 0x02, 0x03, 0x04};
    
    RSocketPayload* payload2 = rsocket_payload_create(
        raw_data, sizeof(raw_data),
        raw_metadata, sizeof(raw_metadata)
    );
    
    if (payload2) {
        printf("\nCreated payload from raw bytes:\n");
        printf("  Data length: %zu\n", rsocket_payload_get_data_length(payload2));
        printf("  Metadata length: %zu\n", rsocket_payload_get_metadata_length(payload2));
        
        rsocket_payload_free(payload2);
    }
    
    printf("Payload operations completed successfully\n");
}

int main() {
    if (rsocket_init() != 0) {
        printf("Failed to initialize RSocket\n");
        return 1;
    }
    
    printf("RSocket version: %s\n\n", rsocket_get_version());
    
    demo_transport_utilities();
    demo_client_connection_attempts();
    demo_payload_operations();
    
    printf("\nTransport demo completed successfully!\n");
    return 0;
}
