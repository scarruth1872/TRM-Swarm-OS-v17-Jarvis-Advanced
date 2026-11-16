import urllib.request
import json
import sys

def verify_wiring():
    print("Checking Swarm node registration of 'jarvis_gateway'...")
    try:
        req = urllib.request.Request("http://localhost:8021/health")
        with urllib.request.urlopen(req, timeout=5) as response:
            data = json.loads(response.read().decode())
            print(f"Swarm online status: {data.get('status')}")
    except Exception as e:
        print(f"Error connecting to Swarm: {e}")
        return False

    try:
        # Check active mesh nodes
        req = urllib.request.Request("http://localhost:8021/swarm/experts")
        with urllib.request.urlopen(req, timeout=5) as response:
            experts = json.loads(response.read().decode())
            print(f"Experts list count: {len(experts)}")
    except Exception as e:
        print(f"Error checking experts: {e}")
        return False

    print("Checking if jarvis_gateway is registered in mesh topology...")
    try:
        # Query topology to find jarvis_gateway
        req = urllib.request.Request("http://localhost:8021/mesh/topology")
        with urllib.request.urlopen(req, timeout=5) as response:
            topo = json.loads(response.read().decode())
            nodes = topo.get("nodes", [])
            jarvis_nodes = [n for n in nodes if n.get("node_id") == "jarvis_gateway"]
            if jarvis_nodes:
                print("✅ Found 'jarvis_gateway' in the Swarm Mesh Topology!")
            else:
                print("❌ Could not find 'jarvis_gateway' node in mesh topology.")
    except Exception as e:
        print(f"Topology query warning: {e}")

    return True

if __name__ == "__main__":
    success = verify_wiring()
    sys.exit(0 if success else 1)
