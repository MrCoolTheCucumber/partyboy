import os
import ssl
import sys
import urllib.request
import shutil
import zipfile
from pathlib import Path

ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE


def download_zip_file(url, file_name):
    if os.path.exists(file_name):
        print("Skipping {}, already found.".format(file_name))
        return

    print("Fetching {}.".format(file_name))
    with urllib.request.urlopen(url, context=ctx) as response, open(file_name + ".zip", 'wb') as out_file:
        shutil.copyfileobj(response, out_file)

    zip_file_path = None
    with zipfile.ZipFile(file_name + ".zip", 'r') as zip_ref:
        zip_file_path = zip_ref.filelist[0].filename
        zip_ref.extractall()

    os.rename(zip_file_path, file_name + "/")
    os.remove(file_name + ".zip")
    return


def download_gb_file(url, file_name):
    if os.path.exists(file_name + ".gb"):
        print("Skipping {}, already found.".format(file_name))
        return

    print("Fetching {}.".format(file_name))
    with urllib.request.urlopen(url) as response, open(file_name + ".gb", 'wb') as out_file:
        shutil.copyfileobj(response, out_file)
    return


current_working_directory = os.getcwd()
path = Path(current_working_directory)

while path.name != "partyboy":
    if len(path.parts) == 1:
        print("Unable to find partyboy root directory.")
        sys.exit();
    path = path.parent

path = path.joinpath("test/test_roms")
path.mkdir(exist_ok=True)
os.chdir(path)

blargg_url = "https://github.com/retrio/gb-test-roms/archive/refs/heads/master.zip"
download_zip_file(blargg_url, "blargg")

mooneye_url = "https://gekkio.fi/files/mooneye-test-suite/mts-20211031-2031-86d1acf/mts-20211031-2031-86d1acf.zip"
download_zip_file(mooneye_url, "mooneye")

dmg_acid_2_url = "https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb"
download_gb_file(dmg_acid_2_url, "dmg-acid2")

cgb_acid_2_url = "https://github.com/mattcurrie/cgb-acid2/releases/download/v1.1/cgb-acid2.gbc"
download_gb_file(cgb_acid_2_url, "cgb-acid2")
