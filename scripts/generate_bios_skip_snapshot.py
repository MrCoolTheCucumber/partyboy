import os
import subprocess
import sys
from pathlib import Path


current_working_directory = os.getcwd()
path = Path(current_working_directory)

while path.name != "partyboy":
    if len(path.parts) == 1:
        print("Unable to find partyboy root directory.")
        sys.exit();
    path = path.parent

path = path.joinpath("util/gen_bios_skip_snapshot")
os.chdir(path)

subprocess.call([
    'cargo',
    'r',
    '--',
    '-b',
    '..\\..\\bin\\_cgb_boot.bin',
    '-r',
    '..\\..\\test_roms\\smbd.gbc',
    '-o',
    '..\\..\\bin\\bios_skip_snapshot.bin'
])
