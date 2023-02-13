from time import sleep
from typing import Mapping
from common import comm
import os

def main():
    for subforum_id in os.getenv("subforum_ids").split(","):
        comm.save_auth_req("Forum.getForum", {"forum_id": subforum_id}, f"_{subforum_id}")
    exit(0)
    forum_ids = os.getenv("forum_ids").split(",")
    for forum_id in forum_ids:
        rslt = comm.save_auth_req("Forum.getCategoriesAndForums", {"preset_id": forum_id}, f"_{forum_id}")
        for subforum_id in rslt['subforums'].keys():
            comm.save_auth_req("Forum.getForum", {"forum_id": subforum_id}, f"_{subforum_id}")
            sleep(0.5)
        for subforum_id in [x.keys() for x in rslt['categories'].values()]:
            comm.save_auth_req("Forum.getForum", {"forum_id": subforum_id}, f"_{subforum_id}")


if __name__ == "__main__":
    main()