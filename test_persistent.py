#\!/usr/bin/env python3
import subprocess
import time
import os
import signal

print(f"Python script PID: {os.getpid()}")

# Try to spawn a persistent process
proc = subprocess.Popen([
    'bash', '-c', 
    'echo "Child process PID: $$"; sleep 300'
], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

print(f"Spawned child PID: {proc.pid}")

# Try to detach from the child
try:
    # This won't help - the child is still in our process group
    time.sleep(5)
    print("Python script exiting...")
except KeyboardInterrupt:
    print("Interrupted")
    
