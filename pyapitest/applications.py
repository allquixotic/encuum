from typing import Mapping
from common import comm

def main():
    #comm.save_auth_req("Applications.getList", {"type": "cancelled"})
    #comm.save_auth_req("Applications.getApplication", {"application_id": os.environ('application_id')})
    types: Mapping[str, str] = comm.auth_req("Applications.getTypes", {})
    
    for t in types.keys():
        comm.save_auth_req("Applications.getList", {"type": t}, f"_{t}")

if __name__ == "__main__":
    main()