package com.rsocket.rust;

public class RSocketServer implements AutoCloseable {
    private long nativePtr;
    
    public RSocketServer(String address) {
        this.nativePtr = create(address);
        if (this.nativePtr == 0) {
            throw new RuntimeException("Failed to create RSocket server");
        }
    }
    
    private static native long create(String address);
    private static native int startTcp(long serverPtr, RequestHandler handler);
    private static native void free(long serverPtr);
    
    public void startTcp(RequestHandler handler) throws RSocket.RSocketException {
        int result = startTcp(nativePtr, handler);
        if (result != 0) {
            throw new RSocket.RSocketException("Failed to start TCP server");
        }
    }
    
    @Override
    public void close() {
        if (nativePtr != 0) {
            free(nativePtr);
            nativePtr = 0;
        }
    }
    
    @Override
    protected void finalize() throws Throwable {
        close();
        super.finalize();
    }
    
    public interface RequestHandler {
        void onRequest(Payload request);
    }
}
