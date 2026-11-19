#!/usr/bin/env python3
"""Brief fixed 720p scene sample using existing status metrics; no GPU-time claim."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
import struct
import time
from osc_client import ManagedRenderer

BASE = "/scshader/v1/hydra/"


def scene():
    def node(i, name, kind, receiver, values):
        arguments = [b"\2" + struct.pack(">I", v[0]) if isinstance(v, tuple)
                     else b"\0" + struct.pack(">f", v) for v in values]
        return struct.pack(">IBBH", i, kind, len(arguments), len(name)) + name.encode() + struct.pack(">I", receiver) + b"".join(arguments)
    nodes = [node(1,"osc",0,0,[10,.1,.8]), node(2,"kaleid",1,1,[5]),
             node(3,"rotate",1,2,[.2,.1]), node(4,"color",2,3,[.8,.5,1,1]),
             node(5,"noise",0,0,[3,.2]), node(6,"scale",1,5,[1.3,1,1,.5,.5]),
             node(7,"blend",3,4,[(6,),.3]), node(8,"shape",0,0,[5,.4,.08]),
             node(9,"modulate",4,7,[(8,),.04])]
    return b"HYIR\0\1\0\0" + struct.pack(">III",len(nodes),0,len(nodes)) + b"".join(nodes)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer",required=True,type=Path)
    parser.add_argument("--output-dir",required=True,type=Path)
    args=parser.parse_args()
    if args.output_dir.exists():parser.error("output directory must be fresh")
    output=args.output_dir.resolve();output.mkdir(parents=True)
    record=dict(ok=False,renderer_sha256=hashlib.sha256(args.renderer.read_bytes()).hexdigest(),
                render_pixels=[1280,720],warmup_seconds=1,sample_seconds=3,gpu_time="not instrumented; gpuFrameMs=-1 means no timestamp measurement")
    try:
        with ManagedRenderer(args.renderer,57223,output/"renderer.log",extra_args=["--backend","metal"]) as owner:
            client=owner.client;record["ready"]=owner.ready_reply
            port=client.socket.getsockname()[1]
            client.send(BASE+"capabilities","iii",(1,1,port));epoch=client.expect(BASE+"capabilities.reply")[1]
            client.send(BASE+"open","issii",(2,epoch,"scene-profile",1,port));session=client.expect(BASE+"opened")[2]
            client.send(BASE+"resolution","siiii",(epoch,session,3,1280,720));client.expect(BASE+"ack")
            client.send("/scshader/v1/window/metrics");metrics=client.expect("/scshader/v1/window/metrics.reply")
            ratio=metrics[4]
            client.send("/scshader/v1/window/resize","ii",(round(1280/ratio),round(720/ratio)))
            data=scene();prefix=(epoch,session,4)
            client.send(BASE+"begin","siiiii",prefix+(0,0,len(data)));client.expect(BASE+"ack")
            client.send(BASE+"chunk","siiib",prefix+(0,data));client.expect(BASE+"ack")
            client.send(BASE+"commit","sii",prefix);client.expect(BASE+"committed")
            client.send(BASE+"present","siii",(epoch,session,5,0));client.expect(BASE+"presented")
            time.sleep(1)
            samples=[];deadline=time.monotonic()+3
            while time.monotonic()<deadline:
                client.send("/scshader/v1/status");state=client.expect("/scshader/v1/status.reply")
                assert len(state)==18 and all(math.isfinite(state[i]) for i in [0,2,3,13]),state
                assert state[10:13]==[0,0,0] and state[15:18]==[0,1,0],state
                samples.append(state);time.sleep(.1)
            assert samples[-1][1]>samples[0][1]+30,"scene frames did not advance"
            allocations=sorted({s[13] for s in samples})
            assert len(allocations)==1,"owned texture allocation changed during the fixed scene"
            client.send("/scshader/v1/window/metrics");record["window_metrics"]=client.expect("/scshader/v1/window/metrics.reply")
            assert record["window_metrics"][2:4]==[1280,720],record["window_metrics"]
            record.update(ok=True,samples=len(samples),frames_advanced=samples[-1][1]-samples[0][1],
                          fps_median=statistics.median(s[0] for s in samples),
                          cpu_frame_ms_median=statistics.median(s[3] for s in samples),
                          cpu_frame_ms_max=max(s[3] for s in samples),owned_texture_bytes_estimate=allocations[0],raw_samples=samples)
            print(json.dumps({k:v for k,v in record.items() if k!="raw_samples"},indent=2))
    finally:(output/"result.json").write_text(json.dumps(record,indent=2)+"\n")


if __name__=="__main__":main()
