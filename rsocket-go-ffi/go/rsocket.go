package rsocket

/*
#cgo LDFLAGS: -L../target/release -lrsocket_rust_go -lpthread -ldl -lm
#include <stdlib.h>
#include <stdint.h>

typedef struct GoRSocketClient GoRSocketClient;
typedef struct GoRSocketServer GoRSocketServer;
typedef struct GoRSocketPayload GoRSocketPayload;
typedef struct GoRSocketError GoRSocketError;
typedef struct GoRSocketPerformanceMetrics GoRSocketPerformanceMetrics;

int rsocket_go_init();
char* rsocket_go_get_version();
void rsocket_go_free_string(char* s);

GoRSocketClient* rsocket_go_client_create();
int rsocket_go_client_connect_tcp(GoRSocketClient* client, const char* addr);
int rsocket_go_client_connect_websocket(GoRSocketClient* client, const char* url);
int rsocket_go_client_is_connected(const GoRSocketClient* client);
int rsocket_go_client_request_response(GoRSocketClient* client, GoRSocketPayload* payload, void* callback, void* user_data);
int rsocket_go_client_fire_and_forget(GoRSocketClient* client, GoRSocketPayload* payload);
void rsocket_go_client_free(GoRSocketClient* client);

GoRSocketServer* rsocket_go_server_create(const char* addr);
int rsocket_go_server_start_tcp(GoRSocketServer* server, void* request_handler, void* user_data);
void rsocket_go_server_free(GoRSocketServer* server);

GoRSocketPayload* rsocket_go_payload_create_from_string(const char* data, const char* metadata);
GoRSocketPayload* rsocket_go_payload_create(const uint8_t* data, size_t data_len, const uint8_t* metadata, size_t metadata_len);
size_t rsocket_go_payload_get_data_length(const GoRSocketPayload* payload);
size_t rsocket_go_payload_get_metadata_length(const GoRSocketPayload* payload);
size_t rsocket_go_payload_copy_data(const GoRSocketPayload* payload, uint8_t* buffer, size_t buffer_len);
size_t rsocket_go_payload_copy_metadata(const GoRSocketPayload* payload, uint8_t* buffer, size_t buffer_len);
void rsocket_go_payload_free(GoRSocketPayload* payload);

char* rsocket_go_get_supported_transports();
int rsocket_go_validate_tcp_address(const char* addr);
int rsocket_go_validate_websocket_url(const char* url);

GoRSocketPerformanceMetrics* rsocket_go_performance_metrics_create();
void rsocket_go_performance_metrics_record_request(GoRSocketPerformanceMetrics* metrics, size_t bytes_sent);
void rsocket_go_performance_metrics_record_response(GoRSocketPerformanceMetrics* metrics, size_t bytes_received);
void rsocket_go_performance_metrics_record_error(GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_request_count(const GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_response_count(const GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_error_count(const GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_bytes_sent(const GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_bytes_received(const GoRSocketPerformanceMetrics* metrics);
uint64_t rsocket_go_performance_metrics_get_uptime_seconds(const GoRSocketPerformanceMetrics* metrics);
void rsocket_go_performance_metrics_free(GoRSocketPerformanceMetrics* metrics);

void rsocket_go_error_free(GoRSocketError* error);

typedef void (*GoCallback)(void* response, GoRSocketError* error, void* user_data);
*/
import "C"
import (
    "errors"
    "runtime"
    "sync"
    "unsafe"
)

func Init() error {
    result := C.rsocket_go_init()
    if result != 0 {
        return errors.New("failed to initialize RSocket")
    }
    return nil
}

func GetVersion() string {
    cVersion := C.rsocket_go_get_version()
    defer C.rsocket_go_free_string(cVersion)
    return C.GoString(cVersion)
}

func GetSupportedTransports() string {
    cTransports := C.rsocket_go_get_supported_transports()
    defer C.rsocket_go_free_string(cTransports)
    return C.GoString(cTransports)
}

type Payload struct {
    cPayload *C.GoRSocketPayload
}

func NewPayload(data string, metadata string) *Payload {
    cData := C.CString(data)
    defer C.free(unsafe.Pointer(cData))
    
    var cMetadata *C.char
    if metadata != "" {
        cMetadata = C.CString(metadata)
        defer C.free(unsafe.Pointer(cMetadata))
    }
    
    cPayload := C.rsocket_go_payload_create_from_string(cData, cMetadata)
    if cPayload == nil {
        return nil
    }
    
    payload := &Payload{cPayload: cPayload}
    runtime.SetFinalizer(payload, (*Payload).free)
    return payload
}

func NewPayloadFromBytes(data []byte, metadata []byte) *Payload {
    var dataPtr *C.uint8_t
    var metadataPtr *C.uint8_t
    
    if len(data) > 0 {
        dataPtr = (*C.uint8_t)(unsafe.Pointer(&data[0]))
    }
    if len(metadata) > 0 {
        metadataPtr = (*C.uint8_t)(unsafe.Pointer(&metadata[0]))
    }
    
    cPayload := C.rsocket_go_payload_create(
        dataPtr, C.size_t(len(data)),
        metadataPtr, C.size_t(len(metadata)),
    )
    if cPayload == nil {
        return nil
    }
    
    payload := &Payload{cPayload: cPayload}
    runtime.SetFinalizer(payload, (*Payload).free)
    return payload
}

func (p *Payload) GetData() []byte {
    if p.cPayload == nil {
        return nil
    }
    
    length := C.rsocket_go_payload_get_data_length(p.cPayload)
    if length == 0 {
        return nil
    }
    
    buffer := make([]byte, length)
    copied := C.rsocket_go_payload_copy_data(
        p.cPayload,
        (*C.uint8_t)(unsafe.Pointer(&buffer[0])),
        C.size_t(length),
    )
    
    return buffer[:copied]
}

func (p *Payload) GetMetadata() []byte {
    if p.cPayload == nil {
        return nil
    }
    
    length := C.rsocket_go_payload_get_metadata_length(p.cPayload)
    if length == 0 {
        return nil
    }
    
    buffer := make([]byte, length)
    copied := C.rsocket_go_payload_copy_metadata(
        p.cPayload,
        (*C.uint8_t)(unsafe.Pointer(&buffer[0])),
        C.size_t(length),
    )
    
    return buffer[:copied]
}

func (p *Payload) GetDataAsString() string {
    data := p.GetData()
    if data == nil {
        return ""
    }
    return string(data)
}

func (p *Payload) GetMetadataAsString() string {
    metadata := p.GetMetadata()
    if metadata == nil {
        return ""
    }
    return string(metadata)
}

func (p *Payload) free() {
    if p.cPayload != nil {
        C.rsocket_go_payload_free(p.cPayload)
        p.cPayload = nil
    }
}

type Client struct {
    cClient *C.GoRSocketClient
    mu      sync.Mutex
}

func NewClient() *Client {
    cClient := C.rsocket_go_client_create()
    if cClient == nil {
        return nil
    }
    
    client := &Client{cClient: cClient}
    runtime.SetFinalizer(client, (*Client).free)
    return client
}

func (c *Client) ConnectTCP(address string) error {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient == nil {
        return errors.New("client is closed")
    }
    
    cAddr := C.CString(address)
    defer C.free(unsafe.Pointer(cAddr))
    
    result := C.rsocket_go_client_connect_tcp(c.cClient, cAddr)
    if result != 0 {
        return errors.New("failed to connect via TCP")
    }
    
    return nil
}

func (c *Client) ConnectWebSocket(url string) error {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient == nil {
        return errors.New("client is closed")
    }
    
    cURL := C.CString(url)
    defer C.free(unsafe.Pointer(cURL))
    
    result := C.rsocket_go_client_connect_websocket(c.cClient, cURL)
    if result != 0 {
        return errors.New("failed to connect via WebSocket")
    }
    
    return nil
}

func (c *Client) IsConnected() bool {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient == nil {
        return false
    }
    
    return C.rsocket_go_client_is_connected(c.cClient) == 1
}

type ResponseCallback func(*Payload, error)

func (c *Client) RequestResponse(payload *Payload, callback ResponseCallback) error {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient == nil {
        return errors.New("client is closed")
    }
    
    if payload == nil || payload.cPayload == nil {
        return errors.New("payload is nil")
    }
    
    callbackID := storeCallback(callback)
    
    result := C.rsocket_go_client_request_response(
        c.cClient,
        payload.cPayload,
        C.GoCallback(C.go_response_callback),
        unsafe.Pointer(uintptr(callbackID)),
    )
    
    if result != 0 {
        removeCallback(callbackID)
        return errors.New("failed to send request")
    }
    
    return nil
}

func (c *Client) FireAndForget(payload *Payload) error {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient == nil {
        return errors.New("client is closed")
    }
    
    if payload == nil || payload.cPayload == nil {
        return errors.New("payload is nil")
    }
    
    result := C.rsocket_go_client_fire_and_forget(c.cClient, payload.cPayload)
    if result != 0 {
        return errors.New("failed to send fire-and-forget")
    }
    
    return nil
}

func (c *Client) free() {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if c.cClient != nil {
        C.rsocket_go_client_free(c.cClient)
        c.cClient = nil
    }
}

type PerformanceMetrics struct {
    cMetrics *C.GoRSocketPerformanceMetrics
}

func NewPerformanceMetrics() *PerformanceMetrics {
    cMetrics := C.rsocket_go_performance_metrics_create()
    if cMetrics == nil {
        return nil
    }
    
    metrics := &PerformanceMetrics{cMetrics: cMetrics}
    runtime.SetFinalizer(metrics, (*PerformanceMetrics).free)
    return metrics
}

func (m *PerformanceMetrics) RecordRequest(bytesSent int) {
    if m.cMetrics != nil {
        C.rsocket_go_performance_metrics_record_request(m.cMetrics, C.size_t(bytesSent))
    }
}

func (m *PerformanceMetrics) RecordResponse(bytesReceived int) {
    if m.cMetrics != nil {
        C.rsocket_go_performance_metrics_record_response(m.cMetrics, C.size_t(bytesReceived))
    }
}

func (m *PerformanceMetrics) RecordError() {
    if m.cMetrics != nil {
        C.rsocket_go_performance_metrics_record_error(m.cMetrics)
    }
}

func (m *PerformanceMetrics) GetRequestCount() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_request_count(m.cMetrics))
}

func (m *PerformanceMetrics) GetResponseCount() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_response_count(m.cMetrics))
}

func (m *PerformanceMetrics) GetErrorCount() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_error_count(m.cMetrics))
}

func (m *PerformanceMetrics) GetBytesSent() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_bytes_sent(m.cMetrics))
}

func (m *PerformanceMetrics) GetBytesReceived() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_bytes_received(m.cMetrics))
}

func (m *PerformanceMetrics) GetUptimeSeconds() uint64 {
    if m.cMetrics == nil {
        return 0
    }
    return uint64(C.rsocket_go_performance_metrics_get_uptime_seconds(m.cMetrics))
}

func (m *PerformanceMetrics) free() {
    if m.cMetrics != nil {
        C.rsocket_go_performance_metrics_free(m.cMetrics)
        m.cMetrics = nil
    }
}

var (
    callbackMu    sync.Mutex
    callbackMap   = make(map[uintptr]ResponseCallback)
    callbackIDGen uintptr = 1
)

func storeCallback(callback ResponseCallback) uintptr {
    callbackMu.Lock()
    defer callbackMu.Unlock()
    
    id := callbackIDGen
    callbackIDGen++
    callbackMap[id] = callback
    return id
}

func removeCallback(id uintptr) {
    callbackMu.Lock()
    defer callbackMu.Unlock()
    
    delete(callbackMap, id)
}

func getCallback(id uintptr) (ResponseCallback, bool) {
    callbackMu.Lock()
    defer callbackMu.Unlock()
    
    callback, exists := callbackMap[id]
    return callback, exists
}

func go_response_callback(response unsafe.Pointer, error *C.GoRSocketError, userData unsafe.Pointer) {
    callbackID := uintptr(userData)
    callback, exists := getCallback(callbackID)
    if !exists {
        return
    }
    
    defer removeCallback(callbackID)
    
    var payload *Payload
    var err error
    
    if error != nil {
        err = errors.New("request failed")
        C.rsocket_go_error_free(error)
    } else if response != nil {
        payload = &Payload{cPayload: (*C.GoRSocketPayload)(response)}
        runtime.SetFinalizer(payload, (*Payload).free)
    }
    
    callback(payload, err)
}
