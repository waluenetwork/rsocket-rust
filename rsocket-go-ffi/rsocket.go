package main

/*
#cgo LDFLAGS: -L./target/release -lrsocket_go_ffi
#include <stdlib.h>

typedef struct CRSocketClient CRSocketClient;

typedef struct {
    char* data;
    size_t data_len;
    char* metadata;
    size_t metadata_len;
} CRSocketPayload;

CRSocketClient* rsocket_go_create_tcp_client(const char* address);
CRSocketClient* rsocket_go_create_websocket_client(const char* address);
CRSocketClient* rsocket_go_create_quic_client(const char* address);
CRSocketClient* rsocket_go_create_iroh_client(const char* address);

int rsocket_go_request_response_sync(
    CRSocketClient* client,
    const char* data,
    size_t data_len,
    char** response_data,
    size_t* response_len
);

int rsocket_go_fire_and_forget(
    CRSocketClient* client,
    const char* data,
    size_t data_len
);

void rsocket_c_free_client(CRSocketClient* client);
void rsocket_go_free_string(char* s);
*/
import "C"
import (
	"errors"
	"unsafe"
)

type RSocketClient struct {
	client *C.CRSocketClient
}

type TransportType int

const (
	TCP TransportType = iota
	WebSocket
	QUIC
	Iroh
)

func NewRSocketClient(transportType TransportType, address string) (*RSocketClient, error) {
	cAddress := C.CString(address)
	defer C.free(unsafe.Pointer(cAddress))
	
	var client *C.CRSocketClient
	
	switch transportType {
	case TCP:
		client = C.rsocket_go_create_tcp_client(cAddress)
	case WebSocket:
		client = C.rsocket_go_create_websocket_client(cAddress)
	case QUIC:
		client = C.rsocket_go_create_quic_client(cAddress)
	case Iroh:
		client = C.rsocket_go_create_iroh_client(cAddress)
	default:
		return nil, errors.New("unsupported transport type")
	}
	
	if client == nil {
		return nil, errors.New("failed to create RSocket client")
	}
	
	return &RSocketClient{client: client}, nil
}

func (c *RSocketClient) RequestResponse(data string) (string, error) {
	cData := C.CString(data)
	defer C.free(unsafe.Pointer(cData))
	
	var responseData *C.char
	var responseLen C.size_t
	
	result := C.rsocket_go_request_response_sync(
		c.client,
		cData,
		C.size_t(len(data)),
		&responseData,
		&responseLen,
	)
	
	if result != 0 {
		return "", errors.New("request-response failed")
	}
	
	defer C.rsocket_go_free_string(responseData)
	
	response := C.GoStringN(responseData, C.int(responseLen))
	return response, nil
}

func (c *RSocketClient) FireAndForget(data string) error {
	cData := C.CString(data)
	defer C.free(unsafe.Pointer(cData))
	
	result := C.rsocket_go_fire_and_forget(
		c.client,
		cData,
		C.size_t(len(data)),
	)
	
	if result != 0 {
		return errors.New("fire-and-forget failed")
	}
	
	return nil
}

func (c *RSocketClient) Close() {
	if c.client != nil {
		C.rsocket_c_free_client(c.client)
		c.client = nil
	}
}

func main() {
}
