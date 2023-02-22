from typing import Mapping
from common import comm
import os

def main():
    wikis = os.environ('wiki_ids').split(",")
    for wiki_id in wikis:
        a = comm.save_auth_req("Wiki.getPageList", {'preset_id': wikis}, f"_{wiki_id}")

if __name__ == "__main__":
    main()