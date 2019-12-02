#!/usr/bin/env python

# Generate a fallback directory whitelist/blacklist line for every fingerprint
# passed as an argument.
#
# Usage:
# generateFallbackDirLine.py fingerprint ...

import sys

import stem.descriptor.remote as remote
import stem.util.tor_tools as tor_tools

if len(sys.argv) <= 1:
  print('Usage: %s fingerprint ...' % sys.argv[0])
  sys.exit(1)

input_list = sys.argv[1:]

for fingerprint in input_list:
  if not tor_tools.is_valid_fingerprint(fingerprint):
    print("'%s' isn't a valid relay fingerprint" % fingerprint)
    sys.exit(1)

found_list = []
desc_query = remote.get_server_descriptors(input_list,
                                           retries=3,
                                           timeout=30)
for desc in desc_query.run():
  assert desc.fingerprint in input_list
  # Skip duplicates on retries
  if desc.fingerprint in found_list:
    continue
  found_list.append(desc.fingerprint)

  if not desc.dir_port:
    print("# %s needs a DirPort" % desc.fingerprint)
  else:
    ipv6_addresses = [(address, port)
                      for address, port, is_ipv6 in desc.or_addresses
                      if is_ipv6]
    ipv6_field = (' ipv6=[%s]:%s' % ipv6_addresses[0]
                  if ipv6_addresses
                  else '')
    print('%s:%s orport=%s id=%s%s # %s' % (desc.address,
                                            desc.dir_port,
                                            desc.or_port,
                                            desc.fingerprint,
                                            ipv6_field,
                                            desc.nickname))

for fingerprint in input_list:
  if fingerprint not in found_list:
    print("# {} not found in current descriptors".format(fingerprint))
