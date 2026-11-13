#!/usr/bin/env python3
"""Short pinned array semantics trace in JavaScriptCore; reads external Git objects."""
import json
from pathlib import Path
import subprocess
from check_hydra_temporal_reference import PIN

def source(path):
    return subprocess.check_output(['git','--no-replace-objects','-C','build/hydra-reference','show',f'{PIN}:{path}'],text=True)
easing=source('src/lib/easing-functions.js').replace('export default','var easing =')
arrays=source('src/lib/array-utils.js').replace("import easing from './easing-functions.js'",'').replace('export default','var arrayUtils =')
script=easing+'\n'+arrays+'''
function run() {
 arrayUtils.init();
 const a=[0,1].smooth();
 const trace=[0,1,2,3,4,-1].map(t=>arrayUtils.getValue(a)({time:t,bpm:30}));
 if(JSON.stringify(trace)!==JSON.stringify([0.5,0,0.5,1,0.5,1])) throw Error("centered interpolation");
 if(arrayUtils.getValue([0,1].fast(0))({time:2,bpm:30})!==1) throw Error("speed zero fallback");
 if(arrayUtils.getValue([0,1].ease('easeInOutQuad'))({time:1.5,bpm:30})!==0.125) throw Error("quadratic ease");
 const fit=[1,3].offset(0.5).fit(2,6);
 if(fit._offset!==undefined || fit[0]!==2 || fit[1]!==6) throw Error("fit metadata");
 return JSON.stringify({smoothed_trace:trace,speed_zero_fallback:true,fit_drops_offset:true});
}
'''
result=subprocess.run(['/usr/bin/osascript','-l','JavaScript','-'],input=script,text=True,capture_output=True,check=True,timeout=20)
print(json.dumps(json.loads(result.stdout),indent=2))
print('SCSHADER_HYDRA_PARAMETER_REFERENCE_OK')
