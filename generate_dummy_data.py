# data/
#     abjasb3.json
#         {
#             "commit": {
#                 "sha1": "abjasb3309jag03r".
#                 "title": "Add integration test",
#                 "author": "Abc Def",
#                 "time": "2020-05-10T08:16:24.054791Z",  <- sorted by this
#                 "url": "https://..github.com..",  <- optional
#             },
#             "benches": {
#                 "topic_name": {"unit": "ms", "goal": "min", "mean": 3123.1},
#                 "read_100_files": {"unit": "ms", "goal": "min", "mean": 321, "std": 1}  <- std is optional
#             }
#         }
#     jkai0bj.json
#     92tjab9.json
#     ....

import os
import datetime
import shutil
import hashlib
import random
import json

try:
    shutil.rmtree('dummy_data')
except OSError:
    pass
os.makedirs('dummy_data', exist_ok=True)
assert os.path.exists('dummy_data')
t = datetime.datetime.utcnow()
cpplint = 0
test_scenario = 1000
for i in range(1, 101):
    sha1 = hashlib.sha1(bytes(i)).hexdigest()
    cpplint = max(cpplint + random.randint(-5, 5), 0)
    test_scenario = max(test_scenario + random.random() * 10 - 5, 0)
    t += datetime.timedelta(hours=random.randint(1, 12))

    data = {
        'commit': {
            'sha1': sha1,
            'title': 'commit %d' % i,
            'author': 'alice',
            'time': t.isoformat() + "Z",
            'url': 'https://github.com/corporate/repo/commit/%s' % sha1
        },
        'benches': {
            'cpplint warnings': {
                'unit': 'warning(s)',
                'goal': 'min',
                'mean': cpplint
            },
            'test scenario': {
                'unit': 'ms',
                'goal': 'min',
                'mean': test_scenario
            },
        }
    }

    with open('dummy_data/%s.json' % sha1, 'w') as f:
        f.write(json.dumps(data, indent=4))
