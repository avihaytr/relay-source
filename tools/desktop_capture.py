# SPDX-License-Identifier: MPL-2.0
"""A thin GStreamer bridge; Rust owns portal consent, TLS and session lifetime.

Only the PipeWire FD granted by the user's screen picker is consumed. stdout is
length-prefixed H.264 access units, never diagnostic text or raw screen pixels.
"""
import signal
import struct
import sys
import gi

gi.require_version("Gst", "1.0")
from gi.repository import Gst

Gst.init(None)
fd, node = int(sys.argv[1]), int(sys.argv[2])
target = ""
if len(sys.argv) > 3 and sys.argv[3] != "0":
    target = ' target-object="%s"' % int(sys.argv[3])
width,height,fps,bitrate=(map(int,sys.argv[4:8]) if len(sys.argv)>=8 else (1280,720,30,4000))
if (width,height,fps,bitrate) not in [(1280,720,30,4000),(1280,720,60,8000),(1920,1080,60,16000)]:
    raise ValueError("Unsupported quality profile")
threads=4 if width>=1920 else 2
pipeline = Gst.parse_launch(
    'pipewiresrc fd=%d path="%d"%s do-timestamp=true keepalive-time=100 '
    '! queue max-size-buffers=2 max-size-bytes=0 max-size-time=0 leaky=downstream '
    '! videoconvert ! videoscale add-borders=true '
    f'! video/x-raw,format=I420,width={width},height={height} '
    f'! videorate ! video/x-raw,framerate={fps}/1 '
    f'! x264enc tune=zerolatency speed-preset=ultrafast bitrate={bitrate} '
    f'key-int-max={fps} bframes=0 byte-stream=true threads={threads} '
    '! h264parse config-interval=-1 '
    '! video/x-h264,stream-format=byte-stream,alignment=au,profile=constrained-baseline '
    '! appsink name=frames sync=false max-buffers=1 drop=false' % (fd, node, target)
)
sink = pipeline.get_by_name("frames")
bus = pipeline.get_bus()
running = True


def stop(*_):
    global running
    running = False


signal.signal(signal.SIGTERM, stop)
signal.signal(signal.SIGINT, stop)
try:
    if pipeline.set_state(Gst.State.PLAYING) == Gst.StateChangeReturn.FAILURE:
        raise RuntimeError("Could not start screen capture")
    while running:
        sample = sink.emit("try-pull-sample", Gst.SECOND // 4)
        if sample is not None:
            buffer = sample.get_buffer()
            data = buffer.extract_dup(0, buffer.get_size())
            if not data or len(data) > 4 * 1024 * 1024:
                raise RuntimeError("Invalid encoded screen frame")
            sys.stdout.buffer.write(struct.pack(">I", len(data)))
            sys.stdout.buffer.write(data)
            sys.stdout.buffer.flush()
        message = bus.pop_filtered(Gst.MessageType.ERROR | Gst.MessageType.EOS)
        if message:
            if message.type == Gst.MessageType.ERROR:
                error, debug = message.parse_error()
                raise RuntimeError(str(error) + " " + str(debug))
            break
except BrokenPipeError:
    pass
finally:
    pipeline.set_state(Gst.State.NULL)
