import sys
import json

def validate_spec(spec):
    # 1. Verify OAuth 2.1 & PKCE
    connections = spec.get("connections", [])
    for conn in connections:
        name = conn.get("name", "unknown")
        
        # Verify PKCE
        if not conn.get("pkce", False):
            return f"Security Violation: Connection '{name}' must have PKCE enabled."
            
        # Verify OAuth 2.1
        auth_flow = conn.get("auth_flow")
        if auth_flow != "oauth-2.1":
            return f"Security Violation: Connection '{name}' must use oauth-2.1 (got '{auth_flow}')."
            
        # Verify Audience is not wildcard
        audience = conn.get("audience")
        if audience == "*":
            return f"Security Violation: Connection '{name}' cannot use wildcard '*' audience."

    # 2. Metadata sanitization checks (recursive)
    def check_meta(obj, path):
        if isinstance(obj, dict):
            for k, v in obj.items():
                if k.lower() == "tenant" and str(v).lower() == "admin":
                    return f"Security Violation: Privilege escalation block detected in {path}"
                res = check_meta(v, f"{path}.{k}")
                if res:
                    return res
        elif isinstance(obj, list):
            for i, item in enumerate(obj):
                res = check_meta(item, f"{path}[{i}]")
                if res:
                    return res
        return None

    # Check project meta
    project_meta = spec.get("_meta")
    if project_meta:
        res = check_meta(project_meta, "project._meta")
        if res:
            return res

    # Check module meta
    modules = spec.get("modules", [])
    for m in modules:
        m_name = m.get("name", "unknown")
        meta = m.get("_meta")
        if meta:
            res = check_meta(meta, f"module('{m_name}')._meta")
            if res:
                return res

    return None

def main():
    # Read the request from stdin
    req_json = sys.stdin.read()
    if not req_json:
        sys.exit(1)
        
    request = json.loads(req_json)
    hook = request.get("hook")
    payload = request.get("payload", {})
    spec = payload.get("spec", {})
    
    error_msg = None
    success = True
    
    if hook == "pre_validate":
        error_msg = validate_spec(spec)
        if error_msg:
            success = False
            
    # Output the response
    response = {
        "success": success,
        "error": error_msg,
        "data": {
            "message": f"Linter successfully processed hook: {hook}" if success else f"Linter failed: {error_msg}"
        }
    }
    
    print(json.dumps(response))

if __name__ == "__main__":
    main()
