#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "rsocket_rust_c_ffi.h"

void response_callback(RSocketPayload* payload, RSocketError* error, void* user_data) {
    if (error != NULL) {
        printf("Error: %s\n", error->message);
        rsocket_error_free(error);
        return;
    }
    
    if (payload != NULL) {
        size_t data_len = rsocket_payload_get_data_length(payload);
        printf("Received response with %zu bytes\n", data_len);
        
        if (data_len > 0) {
            char* buffer = malloc(data_len + 1);
            if (buffer) {
                size_t copied = rsocket_payload_copy_data(payload, (uint8_t*)buffer, data_len);
                buffer[copied] = '\0';
                printf("Response data: %s\n", buffer);
                free(buffer);
            }
        }
        
        rsocket_payload_free(payload);
    }
    
    int* completed = (int*)user_data;
    *completed = 1;
}

int main() {
    printf("RSocket C FFI Simple Client Example\n");
    printf("===================================\n");
    
    if (rsocket_init() != 0) {
        printf("Failed to initialize RSocket\n");
        return 1;
    }
    
    printf("RSocket version: %s\n", rsocket_get_version());
    
    RSocketClient* client = rsocket_client_create();
    if (client == NULL) {
        printf("Failed to create client\n");
        return 1;
    }
    
    if (rsocket_client_connect_tcp(client, "127.0.0.1:7878") != 0) {
        printf("Failed to connect to server\n");
        rsocket_client_free(client);
        return 1;
    }
    
    printf("Connected to server\n");
    
    const char* message = "Hello from C client!";
    RSocketPayload* payload = rsocket_payload_create_from_string(message, NULL);
    
    int completed = 0;
    if (rsocket_client_request_response(client, payload, response_callback, &completed) != 0) {
        printf("Failed to send request\n");
        rsocket_payload_free(payload);
        rsocket_client_free(client);
        return 1;
    }
    
    while (!completed) {
        usleep(10000);
    }
    
    printf("Request completed\n");
    
    rsocket_payload_free(payload);
    rsocket_client_free(client);
    
    return 0;
}
