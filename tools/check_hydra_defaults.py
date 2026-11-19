#!/usr/bin/env python3
"""Compare pinned API input metadata with native registry; executes metadata in JSC only."""
import argparse
import json
from pathlib import Path
import re
import struct
import subprocess
from check_hydra_compatibility import REFERENCE, PROJECT


def f32(value):
    return struct.pack('>f',float(value))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference',type=Path,required=True)
    args=parser.parse_args()
    source=subprocess.check_output(['git','--no-replace-objects','-C',str(args.reference),'show',f'{REFERENCE}:src/glsl/glsl-functions.js'],text=True,encoding='utf-8')
    script=source.replace('export default','var definitions =',1)+'''\nfunction run(){return JSON.stringify(definitions().map(d=>({name:d.name,type:d.type,inputs:d.inputs})));}\n'''
    result=subprocess.run(['/usr/bin/osascript','-l','JavaScript','-'],input=script,text=True,capture_output=True,check=True,timeout=15)
    definitions=json.loads(result.stdout)
    registry=(PROJECT/'renderer/src/hydra/registry.rs').read_text(encoding='utf-8')
    blocks=re.findall(r'Spec\s*\{\s*name:\s*"(\w+)",(.*?)\n\s*\}',registry,re.S)
    native={}
    for name,block in blocks:
        fields={key:re.search(rf'{key}:\s*&\[(.*?)\]',block,re.S).group(1) for key in ['args','_names','_defaults']}
        native[name]=dict(types=re.findall(r'ValueType::(\w+)',fields['args']),names=re.findall(r'"(\w+)"',fields['_names']),defaults=[float(v.strip()) for v in fields['_defaults'].split(',') if v.strip()])
    assert len(native)==len(definitions)==52
    mapped={'float':'Float','vec4':'Vector','sampler2D':'Texture'}
    for definition in definitions:
        n=native[definition['name']]
        types=[mapped[v['type']] for v in definition['inputs']]
        names=[v['name'] for v in definition['inputs']]
        if definition['type'] in ['combine','combineCoord']:
            types=['Expression']+types;names=['texture']+names
        values=[]
        for v in definition['inputs']:
            if v['type']=='sampler2D':
                assert v['default'] is None  # Upstream NaN marks the required texture input.
                continue
            values.extend(v['default'] if isinstance(v['default'],list) else [v['default']])
        assert n['types']==types,(definition['name'],'types',n,types)
        assert n['names']==names,(definition['name'],'names',n,names)
        assert [f32(v) for v in n['defaults']]==[f32(v) for v in values],(definition['name'],'defaults',n['defaults'],values)
    print(json.dumps(dict(ok=True,definitions=52,typed_input_names_and_defaults='matched at f32 precision',required_texture_default='upstream NaN; native explicit slot API/default o0',reference=REFERENCE,browser_gpu=False),indent=2))


if __name__=='__main__':main()
