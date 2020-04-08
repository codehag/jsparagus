import os
import errno

fifo_name="/tmp/notimplemented_fifo"
notimplemented_output="/tmp/notimplemented_output"

try:
    os.mkfifo(fifo_name);
except:
    pass;


while True:
    with open(fifo_name) as fifo:
        for line in fifo:
            for ch in line:
                if (ch == "q"):
                    fifo.close();
                    break;
                with open(notimplemented_output, 'a') as f:
                    f.write(ch);
                break;
            break;
