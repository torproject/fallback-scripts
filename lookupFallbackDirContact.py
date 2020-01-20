#!/usr/bin/env python

# Lookup fallback directory contact lines for every fingerprint passed as an
# argument.
#
# Usage:
# lookupFallbackDirContact.py fingerprint ...

# Future imports for Python 2.7, mandatory in 3.0
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

import sys

import stem.descriptor.remote as remote
import stem.util.tor_tools as tor_tools

if len(sys.argv) <= 1:
  print("Usage: {} fingerprint ...".format(sys.argv[0]))
  sys.exit(1)

input_list = sys.argv[1:]

for fingerprint in input_list:
  if not tor_tools.is_valid_fingerprint(fingerprint):
    print("'%s' isn't a valid relay fingerprint" % fingerprint)
    sys.exit(2)

found_list = []
# we need descriptors, because the consensus does not have contact infos
desc_query = remote.get_server_descriptors(input_list,
                                           retries=3,
                                           timeout=30)
for desc in desc_query.run():
  assert desc.fingerprint in input_list
  # Skip duplicates on retries
  if desc.fingerprint in found_list:
    continue
  found_list.append(desc.fingerprint)

  if desc.contact:
    # Most ContactInfos should be UTF-8
    contact = desc.contact.decode(encoding="utf-8", errors="replace")
  else:
    contact = "(no contact)"
  print("{} {}".format(desc.fingerprint, contact))

for fingerprint in input_list:
  if fingerprint not in found_list:
    print("{} (descriptor not found)".format(fingerprint))
