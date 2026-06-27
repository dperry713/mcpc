import sys
import json

def main():
    # Read the request from stdin
    req_json = sys.stdin.read()
    if not req_json:
        sys.exit(1)
        
    request = json.loads(req_json)
    
    # Process the hook
    hook = request.get("hook")
    
    # Output the response
    response = {
        "success": True,
        "error": None,
        "data": {
            "message": f"Linter successfully processed hook: {hook}"
        }
    }
    
    print(json.dumps(response))

if __name__ == "__main__":
    main()
