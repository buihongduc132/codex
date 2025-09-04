#\!/usr/bin/env python3
import os
import subprocess
import time
import sys

print(f"Python script PID: {os.getpid()}")

# Try the classic double-fork technique to create a daemon
try:
    # First fork
    pid1 = os.fork()
    if pid1 > 0:
        print(f"First fork created PID: {pid1}")
        # Parent exits immediately
        sys.exit(0)
    
    # Child continues - become session leader
    os.setsid()
    
    # Second fork
    pid2 = os.fork()
    if pid2 > 0:
        print(f"Second fork created PID: {pid2}")
        # First child exits
        sys.exit(0)
    
    # Grandchild - should be orphaned and adopted by init
    print(f"Daemon process PID: {os.getpid()}, PPID: {os.getppid()}")
    
    # Try to run a long-running command
    with open('/tmp/daemon_test.log', 'w') as f:
        f.write(f"Daemon started with PID {os.getpid()}\n")
        f.flush()
        
        # Sleep for a long time
        time.sleep(300)
        
        f.write("Daemon finished normally\n")

except OSError as e:
    print(f"Fork failed: {e}")
    sys.exit(1)
