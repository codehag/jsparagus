import os
import errno
import select

fifo_name="/tmp/notimplemented_fifo"
notimplemented_output="/tmp/notimplemented_output"

try:
    os.mkfifo(fifo_name);
except:
    pass;

while True:
    with open(fifo_name) as fifo:
        out=open(notimplemented_output, 'a')
        select.select([fifo],[],[fifo])
        line = fifo.read()
        if (line == "q\n"):
            fifo.close()
            out.close()
            break;
        for ch in line:
            out.write(ch)

