#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include "rsocket_rust_c_ffi.h"

void benchmark_payload_creation() {
    printf("Benchmarking payload creation...\n");
    
    const int iterations = 10000;
    const char* test_data = "This is test data for benchmarking payload creation performance";
    const char* test_metadata = "metadata";
    
    clock_t start = clock();
    
    for (int i = 0; i < iterations; i++) {
        RSocketPayload* payload = rsocket_payload_create_from_string(test_data, test_metadata);
        rsocket_payload_free(payload);
    }
    
    clock_t end = clock();
    double duration = ((double)(end - start)) / CLOCKS_PER_SEC;
    
    printf("Created and freed %d payloads in %.3f seconds\n", iterations, duration);
    printf("Average: %.6f seconds per payload\n", duration / iterations);
    printf("Throughput: %.0f payloads/second\n", iterations / duration);
}

void benchmark_performance_metrics() {
    printf("\nBenchmarking performance metrics...\n");
    
    RSocketPerformanceMetrics* metrics = rsocket_performance_metrics_create();
    if (!metrics) {
        printf("Failed to create performance metrics\n");
        return;
    }
    
    const int iterations = 100000;
    clock_t start = clock();
    
    for (int i = 0; i < iterations; i++) {
        rsocket_performance_metrics_record_request(metrics, 100);
        rsocket_performance_metrics_record_response(metrics, 150);
        if (i % 100 == 0) {
            rsocket_performance_metrics_record_error(metrics);
        }
    }
    
    clock_t end = clock();
    double duration = ((double)(end - start)) / CLOCKS_PER_SEC;
    
    printf("Recorded %d metrics operations in %.3f seconds\n", iterations * 2, duration);
    printf("Average: %.6f seconds per operation\n", duration / (iterations * 2));
    printf("Throughput: %.0f operations/second\n", (iterations * 2) / duration);
    
    printf("\nFinal metrics:\n");
    printf("  Requests: %lu\n", rsocket_performance_metrics_get_request_count(metrics));
    printf("  Responses: %lu\n", rsocket_performance_metrics_get_response_count(metrics));
    printf("  Errors: %lu\n", rsocket_performance_metrics_get_error_count(metrics));
    printf("  Bytes sent: %lu\n", rsocket_performance_metrics_get_bytes_sent(metrics));
    printf("  Bytes received: %lu\n", rsocket_performance_metrics_get_bytes_received(metrics));
    printf("  Uptime: %lu seconds\n", rsocket_performance_metrics_get_uptime_seconds(metrics));
    
    rsocket_performance_metrics_free(metrics);
}

void benchmark_client_creation() {
    printf("\nBenchmarking client creation...\n");
    
    const int iterations = 1000;
    clock_t start = clock();
    
    for (int i = 0; i < iterations; i++) {
        RSocketClient* client = rsocket_client_create();
        if (client) {
            rsocket_client_free(client);
        }
    }
    
    clock_t end = clock();
    double duration = ((double)(end - start)) / CLOCKS_PER_SEC;
    
    printf("Created and freed %d clients in %.3f seconds\n", iterations, duration);
    printf("Average: %.6f seconds per client\n", duration / iterations);
    printf("Throughput: %.0f clients/second\n", iterations / duration);
}

void benchmark_data_copying() {
    printf("\nBenchmarking data copying...\n");
    
    const int iterations = 50000;
    const char* large_data = "This is a larger piece of test data that will be used to benchmark the data copying performance of the RSocket C FFI payload system. It contains multiple sentences to make it more realistic.";
    const size_t data_len = strlen(large_data);
    
    RSocketPayload* payload = rsocket_payload_create_from_string(large_data, NULL);
    if (!payload) {
        printf("Failed to create payload for benchmarking\n");
        return;
    }
    
    char* buffer = malloc(data_len + 1);
    if (!buffer) {
        printf("Failed to allocate buffer\n");
        rsocket_payload_free(payload);
        return;
    }
    
    clock_t start = clock();
    
    for (int i = 0; i < iterations; i++) {
        size_t copied = rsocket_payload_copy_data(payload, (uint8_t*)buffer, data_len);
        if (copied != data_len) {
            printf("Unexpected copy length: %zu vs %zu\n", copied, data_len);
            break;
        }
    }
    
    clock_t end = clock();
    double duration = ((double)(end - start)) / CLOCKS_PER_SEC;
    
    printf("Copied %zu bytes %d times in %.3f seconds\n", data_len, iterations, duration);
    printf("Total data copied: %.2f MB\n", (data_len * iterations) / (1024.0 * 1024.0));
    printf("Copy throughput: %.2f MB/second\n", (data_len * iterations) / (1024.0 * 1024.0) / duration);
    
    free(buffer);
    rsocket_payload_free(payload);
}

int main() {
    printf("RSocket C FFI Performance Benchmark\n");
    printf("===================================\n");
    
    if (rsocket_init() != 0) {
        printf("Failed to initialize RSocket\n");
        return 1;
    }
    
    printf("RSocket version: %s\n\n", rsocket_get_version());
    
    benchmark_payload_creation();
    benchmark_performance_metrics();
    benchmark_client_creation();
    benchmark_data_copying();
    
    printf("\nBenchmark completed successfully!\n");
    return 0;
}
