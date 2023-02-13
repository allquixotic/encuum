from functools import cache, cached_property
from time import sleep
from types import CodeType
from typing import Any, Mapping, MutableMapping
from jsonrpcclient import request
import requests
import os, sys, trace, json
from dotenv import load_dotenv
import traceit

class Common:
    def __init__(self):
        load_dotenv()

    @cached_property
    def website(self):
        return os.getenv("website")
    
    @cached_property
    def email(self):
        return os.getenv("email")
    
    @cached_property
    def password(self):
        return os.getenv("password")

    @cached_property
    def session_id(self):
        retval = os.getenv("session_id") or self.req("User.login", {"email": self.email, "password": self.password})['session_id']
        #Exit the program with an error if the value is None or empty.
        if not retval:
            print("Error: session_id is empty.")
            exit(1)
        else:
            print(f"session_id: {retval}")
        return retval

    def req(self, method: str, params: Mapping[str, Any]):
        try:
            retval = json.loads(requests.post(f"https://{self.website}/api/v1/api.php", json=request(method, params=params)).text)['result']
        except Exception as e:
            print(f"Error: {e}")
            return None

        sleep(0.4)
        return retval
    
    def auth_req(self, method: str, params: MutableMapping[str, Any]):
        params['session_id'] = self.session_id
        return self.req(method, params)
    
    #Save the result of an auth_req call to a file of the name of the method called.
    def save_auth_req(self, method: str, params: MutableMapping[str, Any], fna: str = ""):
        r = self.auth_req(method, params)
        if r is not None:
            with open(f"{method}{fna}.json.log", "w") as f:
                f.write(json.dumps(r, indent=4))
        return r

#sys.setprofile(traceit.tracefunc)
comm = Common()