import sys
sys.path.append('./target/debug')
import rustyusn

TEST_FILE = u'testdata/record.usn'

t1 = rustyusn.open_file(TEST_FILE)
print t1
