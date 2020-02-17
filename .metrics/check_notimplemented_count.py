#!/usr/bin/python
import os.path
import json

filename = 'count/not-implemented.json'
badge_filename = 'badges/not-implemented.json'

if os.path.isfile(badge_filename):
    with open(filename, 'r') as f:
        data = json.load(f)
        if len(data) != 0 and data[-1]["total_count"] != data[-2]["total_count"]:
            print("true")
        else :
            print("false")
else:
    print("true")
