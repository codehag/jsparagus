import os
import errno
import select

fifo_name="/tmp/notimplemented_fifo"
notimplemented_output="/tmp/notimplemented_output"

try:
    os.mkfifo(fifo_name);
except:
    pass;

print "opening";
with open(fifo_name) as fifo:
    with open(notimplemented_output, 'a') as f:
        while True:
            select.select([fifo],[],[fifo])
            line = fifo.read()
            print repr(line)
            if (line == "q\n"):
                fifo.close();
                break;
            for ch in line:
                f.write(ch);

