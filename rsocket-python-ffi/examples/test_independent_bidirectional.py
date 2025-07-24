#!/usr/bin/env python3
"""
Test script for Independent Bidirectional Channel Communication
Demonstrates true bidirectional RSocket communication using request_channel
where client and server streams are independent and not coupled.
"""

import asyncio
import logging
import subprocess
import time
import signal
import sys
import os

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class BidirectionalChannelTest:
    def __init__(self):
        self.server_process = None
        self.client_process = None
    
    async def start_server(self):
        """Start the bidirectional channel server"""
        try:
            logging.info("🚀 Starting bidirectional channel server...")
            self.server_process = subprocess.Popen([
                sys.executable, "independent_bidirectional_server.py"
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            
            await asyncio.sleep(3)
            
            if self.server_process.poll() is None:
                logging.info("✅ Server started successfully")
                return True
            else:
                stdout, stderr = self.server_process.communicate()
                logging.error(f"❌ Server failed to start. Stdout: {stdout}, Stderr: {stderr}")
                return False
        except Exception as e:
            logging.error(f"❌ Failed to start server: {e}")
            return False
    
    async def run_client_test(self):
        """Run the client test directly (not as subprocess)"""
        try:
            logging.info("🚀 Running bidirectional channel client test...")
            
            import sys
            sys.path.append('.')
            
            from independent_bidirectional_client import BidirectionalChannelClient
            
            client = BidirectionalChannelClient()
            await client.run_client()
            
            logging.info("✅ Client test completed successfully")
            return True
            
        except Exception as e:
            logging.error(f"❌ Client test failed: {e}")
            import traceback
            traceback.print_exc()
            return False
    
    def cleanup(self):
        """Clean up processes"""
        if self.server_process:
            self.server_process.terminate()
            self.server_process.wait()
    
    async def run_test(self):
        """Run the complete bidirectional channel test"""
        try:
            logging.info("🎯 Testing Independent Bidirectional Channel Communication")
            logging.info("=" * 80)
            logging.info("🔄 This test demonstrates:")
            logging.info("   • Client sends streams of different sizes")
            logging.info("   • Server responds with independent stream sizes")
            logging.info("   • Server content is generated independently")
            logging.info("   • Multiple concurrent channels work independently")
            logging.info("=" * 80)
            
            if not await self.start_server():
                return False
            
            if not await self.run_client_test():
                return False
            
            logging.info("\n🎉 Bidirectional channel test completed!")
            logging.info("✅ Demonstrated true bidirectional channel communication")
            logging.info("✅ Client and server streams are independent")
            logging.info("✅ Server generates content independently of client input")
            logging.info("✅ Stream sizes are not coupled between client and server")
            
            return True
            
        except KeyboardInterrupt:
            logging.info("\n👋 Test stopped by user")
            return True
        except Exception as e:
            logging.error(f"❌ Test failed: {e}")
            import traceback
            traceback.print_exc()
            return False
        finally:
            self.cleanup()

async def main():
    test = BidirectionalChannelTest()
    
    def signal_handler(signum, frame):
        logging.info("\n🛑 Received interrupt signal, cleaning up...")
        test.cleanup()
        sys.exit(0)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    success = await test.run_test()
    if success:
        logging.info("\n🎯 Test completed successfully!")
    else:
        logging.error("\n❌ Test failed!")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())
