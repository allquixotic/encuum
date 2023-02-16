"""traceit.py

Traces the call stack.

Usage:

import sys
import traceit

sys.setprofile(traceit.traceit)
"""

import sys


WHITE_LIST = {'pyapitest'}      # Look for these words in the file path.
EXCLUSIONS = {'<'}          # Ignore <listcomp>, etc. in the function name.


def tracefunc(frame, event, arg):

    if event == "call":
        tracefunc.stack_level += 1

        unique_id = frame.f_code.co_filename+str(frame.f_lineno)
        if unique_id in tracefunc.memorized:
            return

        # Part of filename MUST be in white list.
        if any(x in frame.f_code.co_filename for x in WHITE_LIST) \
            and \
          not any(x in frame.f_code.co_name for x in EXCLUSIONS):

            if 'self' in frame.f_locals:
                class_name = frame.f_locals['self'].__class__.__name__
                func_name = class_name + '.' + frame.f_code.co_name
            else:
                func_name = frame.f_code.co_name

            func_name = f"{func_name}:->{tracefunc.stack_level*2}s()"
            txt = '{: <40} # {}, {}'.format(
                    func_name, frame.f_code.co_filename, frame.f_lineno)
            print(txt)

            tracefunc.memorized.add(unique_id)

    elif event == "return":
        tracefunc.stack_level -= 1


tracefunc.memorized = set()
tracefunc.stack_level = 0
