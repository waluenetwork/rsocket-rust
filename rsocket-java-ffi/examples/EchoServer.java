import com.rsocket.rust.*;

public class EchoServer {
    public static void main(String[] args) {
        System.out.println("RSocket Java FFI Echo Server Example");
        System.out.println("===================================");
        
        int result = RSocket.init();
        if (result != 0) {
            System.err.println("Failed to initialize RSocket");
            return;
        }
        
        System.out.println("RSocket version: " + RSocket.getVersion());
        
        try (RSocketServer server = new RSocketServer("127.0.0.1:7878")) {
            server.startTcp(new RSocketServer.RequestHandler() {
                @Override
                public void onRequest(Payload request) {
                    System.out.println("Received request: " + request.getDataAsString());
                }
            });
            
            System.out.println("Server started on 127.0.0.1:7878");
            System.out.println("Press Enter to stop...");
            System.in.read();
            
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
