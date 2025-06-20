import com.rsocket.rust.*;
import java.util.concurrent.CompletableFuture;

public class SimpleClient {
    public static void main(String[] args) {
        System.out.println("RSocket Java FFI Simple Client Example");
        System.out.println("====================================");
        
        int result = RSocket.init();
        if (result != 0) {
            System.err.println("Failed to initialize RSocket");
            return;
        }
        
        System.out.println("RSocket version: " + RSocket.getVersion());
        
        try (RSocketClient client = new RSocketClient()) {
            client.connectTcp("127.0.0.1:7878");
            System.out.println("Connected to server");
            
            try (Payload payload = Payload.fromString("Hello from Java client!", "")) {
                CompletableFuture<Payload> future = client.requestResponse(payload);
                
                try (Payload response = future.get()) {
                    System.out.println("Received response: " + response.getDataAsString());
                }
            }
            
            System.out.println("Request completed");
            
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
