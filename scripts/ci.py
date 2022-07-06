# Nothing to see here

import os
import ssl
import sys
import urllib.request
import shutil
from pathlib import Path

ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE

current_working_directory = os.getcwd()
path = Path(current_working_directory)

while path.name != "partyboy":
    if len(path.parts) == 1:
        print("Unable to find partyboy root directory.")
        sys.exit();
    path = path.parent

path = path.joinpath("bin")
os.chdir(path)

file_name = "_cgb_boot.bin"
url = "https://gbdev.gg8.se/files/roms/bootroms/cgb_boot.bin"

if os.path.exists(file_name):
    print("Skipping {}, already found.".format(file_name))
else:
    print("Fetching {}.".format(file_name))
    with urllib.request.urlopen(url, context=ctx) as response, open(file_name, 'wb') as out_file:
        shutil.copyfileobj(response, out_file)
    print("Fetched.")
    