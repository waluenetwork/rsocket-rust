package com.rsocket.rust;

import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;

public class RSocketClient implements AutoCloseable {
    private long nativePtr;
    
    public RSocketClient() {
        this.nativePtr = create();
        if (this.nativePtr == 0) {
            throw new RuntimeException("Failed to create RSocket client");
        }
    }
    
    private static native long create();
    private static native int connectTcp(long clientPtr, String address);
    private static native int connectWebSocket(long clientPtr, String url);
    private static native int requestResponse(long clientPtr, long payloadPtr, ResponseCallback callback);
    private static native int fireAndForget(long clientPtr, long payloadPtr);
    private static native void free(long clientPtr);
    
    public void connectTcp(String address) throws RSocket.RSocketException {
        int result = connectTcp(nativePtr, address);
        if (result != 0) {
            throw new RSocket.RSocketException("Failed to connect via TCP");
        }
    }
    
    public void connectWebSocket(String url) throws RSocket.RSocketException {
        int result = connectWebSocket(nativePtr, url);
        if (result != 0) {
            throw new RSocket.RSocketException("Failed to connect via WebSocket");
        }
    }
    
    public CompletableFuture<Payload> requestResponse(Payload payload) {
        CompletableFuture<Payload> future = new CompletableFuture<>();
        
        ResponseCallback callback = new ResponseCallback() {
            @Override
            public void onResponse(Payload response) {
                future.complete(response);
            }
            
            @Override
            public void onError(String error) {
                future.completeExceptionally(new RSocket.RSocketException(error));
            }
        };
        
        int result = requestResponse(nativePtr, payload.nativePtr, callback);
        if (result != 0) {
            future.completeExceptionally(new RSocket.RSocketException("Failed to send request"));
        }
        
        return future;
    }
    
    public void fireAndForget(Payload payload) throws RSocket.RSocketException {
        int result = fireAndForget(nativePtr, payload.nativePtr);
        if (result != 0) {
            throw new RSocket.RSocketException("Failed to send fire-and-forget");
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
    
    public interface ResponseCallback {
        void onResponse(Payload response);
        void onError(String error);
    }
}
