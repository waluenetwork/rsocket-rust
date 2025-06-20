#pragma once

#include "rsocket_rust_c_ffi.h"
#include <string>
#include <memory>
#include <functional>
#include <vector>
#include <stdexcept>

namespace rsocket {

class Payload {
public:
    Payload(const std::string& data, const std::string& metadata = "")
        : payload_(rsocket_payload_create_from_string(data.c_str(), 
                   metadata.empty() ? nullptr : metadata.c_str())) {}
    
    Payload(const std::vector<uint8_t>& data, const std::vector<uint8_t>& metadata = {})
        : payload_(rsocket_payload_create(
            data.data(), data.size(),
            metadata.empty() ? nullptr : metadata.data(), metadata.size())) {}
    
    ~Payload() {
        if (payload_) {
            rsocket_payload_free(payload_);
        }
    }
    
    Payload(Payload&& other) noexcept : payload_(other.payload_) {
        other.payload_ = nullptr;
    }
    
    Payload& operator=(Payload&& other) noexcept {
        if (this != &other) {
            if (payload_) {
                rsocket_payload_free(payload_);
            }
            payload_ = other.payload_;
            other.payload_ = nullptr;
        }
        return *this;
    }
    
    Payload(const Payload&) = delete;
    Payload& operator=(const Payload&) = delete;
    
    size_t getDataLength() const {
        return payload_ ? rsocket_payload_get_data_length(payload_) : 0;
    }
    
    size_t getMetadataLength() const {
        return payload_ ? rsocket_payload_get_metadata_length(payload_) : 0;
    }
    
    std::string getDataAsString() const {
        if (!payload_) return "";
        
        size_t len = getDataLength();
        if (len == 0) return "";
        
        std::vector<uint8_t> buffer(len);
        size_t copied = rsocket_payload_copy_data(payload_, buffer.data(), len);
        return std::string(buffer.begin(), buffer.begin() + copied);
    }
    
    RSocketPayload* release() {
        RSocketPayload* p = payload_;
        payload_ = nullptr;
        return p;
    }
    
private:
    RSocketPayload* payload_;
};

class Client {
public:
    Client() : client_(rsocket_client_create()) {
        if (!client_) {
            throw std::runtime_error("Failed to create RSocket client");
        }
    }
    
    ~Client() {
        if (client_) {
            rsocket_client_free(client_);
        }
    }
    
    Client(const Client&) = delete;
    Client& operator=(const Client&) = delete;
    
    bool connectTcp(const std::string& address) {
        return rsocket_client_connect_tcp(client_, address.c_str()) == 0;
    }
    
    bool connectWebSocket(const std::string& url) {
        return rsocket_client_connect_websocket(client_, url.c_str()) == 0;
    }
    
    bool isConnected() const {
        return rsocket_client_is_connected(client_) == 1;
    }
    
    using ResponseCallback = std::function<void(std::unique_ptr<Payload>, const std::string&)>;
    
    bool requestResponse(std::unique_ptr<Payload> payload, ResponseCallback callback) {
        auto* cb_data = new ResponseCallback(std::move(callback));
        
        return rsocket_client_request_response(
            client_, 
            payload->release(),
            [](RSocketPayload* response, RSocketError* error, void* user_data) {
                auto* callback = static_cast<ResponseCallback*>(user_data);
                
                if (error) {
                    (*callback)(nullptr, std::string(error->message));
                    rsocket_error_free(error);
                } else if (response) {
                    auto cpp_payload = std::make_unique<Payload>("", "");
                    (*callback)(std::move(cpp_payload), "");
                    rsocket_payload_free(response);
                } else {
                    (*callback)(nullptr, "Unknown error");
                }
                
                delete callback;
            },
            cb_data
        ) == 0;
    }
    
    bool fireAndForget(std::unique_ptr<Payload> payload) {
        return rsocket_client_fire_and_forget(client_, payload->release()) == 0;
    }
    
private:
    RSocketClient* client_;
};

class PerformanceMetrics {
public:
    PerformanceMetrics() : metrics_(rsocket_performance_metrics_create()) {
        if (!metrics_) {
            throw std::runtime_error("Failed to create performance metrics");
        }
    }
    
    ~PerformanceMetrics() {
        if (metrics_) {
            rsocket_performance_metrics_free(metrics_);
        }
    }
    
    PerformanceMetrics(const PerformanceMetrics&) = delete;
    PerformanceMetrics& operator=(const PerformanceMetrics&) = delete;
    
    void recordRequest(size_t bytes_sent) {
        rsocket_performance_metrics_record_request(metrics_, bytes_sent);
    }
    
    void recordResponse(size_t bytes_received) {
        rsocket_performance_metrics_record_response(metrics_, bytes_received);
    }
    
    void recordError() {
        rsocket_performance_metrics_record_error(metrics_);
    }
    
    uint64_t getRequestCount() const {
        return rsocket_performance_metrics_get_request_count(metrics_);
    }
    
    uint64_t getResponseCount() const {
        return rsocket_performance_metrics_get_response_count(metrics_);
    }
    
    uint64_t getErrorCount() const {
        return rsocket_performance_metrics_get_error_count(metrics_);
    }
    
    uint64_t getBytesSent() const {
        return rsocket_performance_metrics_get_bytes_sent(metrics_);
    }
    
    uint64_t getBytesReceived() const {
        return rsocket_performance_metrics_get_bytes_received(metrics_);
    }
    
    uint64_t getUptimeSeconds() const {
        return rsocket_performance_metrics_get_uptime_seconds(metrics_);
    }
    
private:
    RSocketPerformanceMetrics* metrics_;
};

} // namespace rsocket
