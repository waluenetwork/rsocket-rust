package com.rsocket.rust;

public class RSocket {
    static {
        System.loadLibrary("rsocket_rust_java");
    }
    
    public static native int init();
    public static native String getVersion();
    
    public static class RSocketException extends Exception {
        public RSocketException(String message) {
            super(message);
        }
    }
}
