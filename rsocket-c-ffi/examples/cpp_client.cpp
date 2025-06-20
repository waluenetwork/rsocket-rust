#include "rsocket_cpp.hpp"
#include <iostream>
#include <thread>
#include <chrono>

int main() {
    std::cout << "RSocket C++ FFI Client Example\n";
    std::cout << "==============================\n";
    
    try {
        rsocket_init();
        std::cout << "RSocket version: " << rsocket_get_version() << "\n";
        
        rsocket::Client client;
        
        if (!client.connectTcp("127.0.0.1:7878")) {
            std::cerr << "Failed to connect to server\n";
            return 1;
        }
        
        std::cout << "Connected to server\n";
        
        auto payload = std::make_unique<rsocket::Payload>("Hello from C++ client!");
        
        bool completed = false;
        std::string error_msg;
        
        client.requestResponse(std::move(payload), 
            [&](std::unique_ptr<rsocket::Payload> response, const std::string& error) {
                if (!error.empty()) {
                    error_msg = error;
                } else if (response) {
                    std::cout << "Received response with " << response->getDataLength() << " bytes\n";
                    std::cout << "Response data: " << response->getDataAsString() << "\n";
                }
                completed = true;
            });
        
        while (!completed) {
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }
        
        if (!error_msg.empty()) {
            std::cerr << "Error: " << error_msg << "\n";
            return 1;
        }
        
        std::cout << "Request completed successfully\n";
        
        rsocket::PerformanceMetrics metrics;
        metrics.recordRequest(21);
        metrics.recordResponse(25);
        
        std::cout << "Performance metrics:\n";
        std::cout << "  Requests: " << metrics.getRequestCount() << "\n";
        std::cout << "  Responses: " << metrics.getResponseCount() << "\n";
        std::cout << "  Bytes sent: " << metrics.getBytesSent() << "\n";
        std::cout << "  Bytes received: " << metrics.getBytesReceived() << "\n";
        
    } catch (const std::exception& e) {
        std::cerr << "Exception: " << e.what() << "\n";
        return 1;
    }
    
    return 0;
}
