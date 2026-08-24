import urllib.request
import json
import sys

def verify_alignment():
    print("Testing Swarm-PM Mesh Integration Alignment...")
    
    # 1. Query topology to see if antigravity_pm is listed
    try:
        req = urllib.request.urlopen("http://localhost:8021/mesh/topology")
        data = json.loads(req.read().decode("utf-8"))
        nodes = [node.get("node_id") for node in data.get("nodes", [])]
        print(f"Mesh Nodes Found: {nodes}")
        
        if "antigravity_pm" not in nodes:
            print("❌ Antigravity PM node was not found in the active Swarm topology registry.")
            return False
        print("✅ Antigravity PM node verified in mesh network.")
    except Exception as e:
        print(f"❌ Failed to query topology: {e}")
        return False

    # 2. Query PM delegation endpoint
    try:
        url = "http://localhost:8021/swarm/pm/delegate"
        payload = json.dumps({
            "task": "Perform secondary validation check on the pbft consensus state ledger.",
            "sender_role": "Lead Developer"
        }).encode("utf-8")
        
        req = urllib.request.Request(
            url, 
            data=payload,
            headers={"Content-Type": "application/json"}
        )
        
        resp = urllib.request.urlopen(req)
        res_data = json.loads(resp.read().decode("utf-8"))
        print(f"Delegation Response: {res_data}")
        
        if res_data.get("status") != "received" or not res_data.get("task_logged"):
            print("❌ PM delegation endpoint returned incorrect status payload.")
            return False
            
        print("✅ PM delegation endpoint aligned successfully.")
    except Exception as e:
        print(f"❌ Failed to test delegation endpoint: {e}")
        return False
        
    print("✅ All PM integration verification tests completed successfully!")
    return True

if __name__ == "__main__":
    success = verify_alignment()
    sys.exit(0 if success else 1)
