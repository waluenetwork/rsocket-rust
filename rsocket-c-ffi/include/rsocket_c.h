#ifndef RSOCKET_C_H
#define RSOCKET_C_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

typedef struct CRSocketClient CRSocketClient;

typedef struct {
    char* data;
    size_t data_len;
    char* metadata;
    size_t metadata_len;
} CRSocketPayload;

typedef struct {
    int transport_type;
    const char* address;
    int enable_advanced_features;
} CRSocketTransportConfig;

typedef void (*CRSocketCallback)(const CRSocketPayload* payload, void* user_data);

CRSocketClient* rsocket_c_create_client(const CRSocketTransportConfig* config);

int rsocket_c_request_response_async(
    CRSocketClient* client,
    const CRSocketPayload* payload,
    CRSocketCallback callback,
    void* user_data
);

int rsocket_c_request_response_sync(
    CRSocketClient* client,
    const char* data,
    size_t data_len,
    char** response_data,
    size_t* response_len
);

void rsocket_c_free_client(CRSocketClient* client);
void rsocket_c_free_payload(CRSocketPayload* payload);

#ifdef __cplusplus
}
#endif

#endif
