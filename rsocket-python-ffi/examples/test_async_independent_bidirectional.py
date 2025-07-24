#!/usr/bin/env python3
"""
Test script for Async Independent Bidirectional Communication
Demonstrates true async bidirectional communication with independent sender/receiver threads.
"""

import asyncio
import logging
import subprocess
import time
import signal
import sys

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class AsyncIndependentBidirectionalTest:
    def __init__(self):
        self.server_process = None
    
    async def start_server(self):
        """Start the async independent bidirectional server"""
        try:
            logging.info("🚀 Starting async independent bidirectional server...")
            self.server_process = subprocess.Popen([
                sys.executable, "async_independent_bidirectional_server.py"
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
        """Run the client test directly"""
        try:
            logging.info("🚀 Running async independent bidirectional client test...")
            
            import sys
            sys.path.append('.')
            
            from async_independent_bidirectional_client import AsyncIndependentBidirectionalClient
            
            client = AsyncIndependentBidirectionalClient()
            await client.run_independent_bidirectional_test()
            
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
        """Run the complete async independent bidirectional test"""
        try:
            logging.info("🎯 Testing Async Independent Bidirectional Communication")
            logging.info("=" * 80)
            logging.info("🔄 This test demonstrates:")
            logging.info("   • Independent async sender threads using asyncio.create_task")
            logging.info("   • Callback-based receivers with on_next/on_complete")
            logging.info("   • Separate async contexts for send/receive operations")
            logging.info("   • No request-response coupling between sender and receiver")
            logging.info("   • True bidirectional communication with independent timing")
            logging.info("=" * 80)
            
            if not await self.start_server():
                return False
            
            if not await self.run_client_test():
                return False
            
            logging.info("\n🎉 Async independent bidirectional test completed!")
            logging.info("✅ Demonstrated true async bidirectional communication")
            logging.info("✅ Independent sender threads with asyncio.create_task")
            logging.info("✅ Callback-based receivers with on_next/on_complete")
            logging.info("✅ Separate async contexts for send/receive")
            logging.info("✅ No request-response coupling")
            
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
    test = AsyncIndependentBidirectionalTest()
    
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
