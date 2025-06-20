package com.rsocket.rust;

public class Payload implements AutoCloseable {
    private long nativePtr;
    
    private Payload(long nativePtr) {
        this.nativePtr = nativePtr;
    }
    
    public static native long create(byte[] data, byte[] metadata);
    public static native long createFromString(String data, String metadata);
    public static native int getDataLength(long payloadPtr);
    public static native byte[] getData(long payloadPtr);
    public static native void free(long payloadPtr);
    
    public static Payload fromBytes(byte[] data, byte[] metadata) {
        long ptr = create(data, metadata);
        if (ptr == 0) {
            throw new RuntimeException("Failed to create payload");
        }
        return new Payload(ptr);
    }
    
    public static Payload fromString(String data, String metadata) {
        long ptr = createFromString(data, metadata);
        if (ptr == 0) {
            throw new RuntimeException("Failed to create payload");
        }
        return new Payload(ptr);
    }
    
    public byte[] getData() {
        return getData(nativePtr);
    }
    
    public String getDataAsString() {
        byte[] data = getData();
        return data != null ? new String(data) : null;
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
}
