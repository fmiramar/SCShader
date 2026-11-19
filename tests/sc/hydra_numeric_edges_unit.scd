(
var h=SCHydra.new,sequence,check={|ok,message|if(ok.not){Error(message).throw}};
try {
 sequence=h.seq([0,1]).offset(-0.25);
 ("NEGATIVE OFFSET PAYLOAD: "++sequence.values).postln;
 check.value(sequence.values[1]==(-0.25),"negative offset must preserve pinned JavaScript signed remainder");
 sequence.offset(-1.25);check.value(sequence.values[1]==(-0.25),"negative offset truncates whole cycles");
 sequence.offset(1.25);check.value(sequence.values[1]==0.25,"positive offset remainder");
 check.value(try{sequence.offset(inf);false}{true},"nonfinite offset must reject");
 check.value(sequence.values[1]==0.25,"invalid offset changed the last valid phase");
 check.value(h.seq([1,3]).offset(0.5).fit(2,6).values.copyRange(0,3)==[1,0,0,0],"fit metadata/drop offset");
 check.value(h.seq([0,1]).ease(\easeInOutQuad).values.copyRange(0,3)==[1,0,1,1],"quadratic easing payload");
 check.value(try{h.seq([0,1]).ease(\easeInCubic);false}{true},"unsupported easing must reject explicitly");
 check.value(try{h.shape(h.param(\sides,3)).snapshot;true}{false},"typed front-end should serialize numeric request for compiler domain checks");
 h.free;"SCSHADER_HYDRA_NUMERIC_EDGES_OK".postln;0.exit;
}{|error|error.reportError;h.free;1.exit};
)
