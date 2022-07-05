import os
import sys
import subprocess

ignored_tests = [
    "intr_2_mode0_timing_sprites",
    "intr_2_oam_ok_timing",
    "stat_lyc_onoff",
    #"mbc1::multicart_rom_8Mb"
]

def transform(test_name):
    return "test({t})".format(t=test_name)

ignored_tests = list(map(transform, ignored_tests))

filter_exp = "not ({tests})".format(tests=" | ".join(ignored_tests))

os.environ["NEXTEST_EXPERIMENTAL_FILTER_EXPR"] = "1"

ret_code = subprocess.call([
    'cargo',
    'nextest',
    'run',
    '--profile',
    'ci',
    '-E',
    filter_exp
])

sys.exit(ret_code)
