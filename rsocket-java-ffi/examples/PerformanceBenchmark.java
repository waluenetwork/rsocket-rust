import com.rsocket.rust.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicLong;

public class PerformanceBenchmark {
    public static void main(String[] args) {
        System.out.println("RSocket Java FFI Performance Benchmark");
        System.out.println("======================================");
        
        RSocket.init();
        
        final int numRequests = 1000;
        final int messageSize = 1024;
        
        byte[] message = new byte[messageSize];
        for (int i = 0; i < messageSize; i++) {
            message[i] = (byte) (i % 256);
        }
        
        System.out.printf("Sending %d requests with %d byte payloads...%n", numRequests, messageSize);
        
        try (RSocketClient client = new RSocketClient()) {
            client.connectTcp("127.0.0.1:7878");
            
            long startTime = System.currentTimeMillis();
            CountDownLatch latch = new CountDownLatch(numRequests);
            AtomicLong successCount = new AtomicLong(0);
            AtomicLong errorCount = new AtomicLong(0);
            
            for (int i = 0; i < numRequests; i++) {
                try (Payload payload = Payload.fromBytes(message, null)) {
                    CompletableFuture<Payload> future = client.requestResponse(payload);
                    
                    future.whenComplete((response, throwable) -> {
                        if (throwable != null) {
                            errorCount.incrementAndGet();
                        } else {
                            successCount.incrementAndGet();
                            if (response != null) {
                                response.close();
                            }
                        }
                        latch.countDown();
                    });
                }
            }
            
            latch.await();
            long duration = System.currentTimeMillis() - startTime;
            
            System.out.println("\nBenchmark Results:");
            System.out.println("Duration: " + duration + "ms");
            System.out.println("Successful requests: " + successCount.get());
            System.out.println("Failed requests: " + errorCount.get());
            System.out.printf("Requests per second: %.2f%n", (double) successCount.get() / (duration / 1000.0));
            System.out.printf("Average latency: %.2fms%n", (double) duration / successCount.get());
            
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
