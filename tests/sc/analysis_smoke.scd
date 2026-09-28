(
var server = Server(\scshaderAnalysisSmoke, NetAddr("127.0.0.1", 57310));
var sourceBus, sourceSynth, analysis, completed = false, finish;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        analysis.tryPerform(\free);
        sourceSynth.tryPerform(\free);
        sourceBus.tryPerform(\free);
        server.quit;
        SystemClock.sched(2.0, {
            passed.if({ "SCSHADER_ANALYSIS_OK".postln; 0.exit }, {
                "SCSHADER_ANALYSIS_FAILED".postln; 1.exit
            });
            nil
        });
    };
};

server.options.numOutputBusChannels = 2;
server.waitForBoot({
    Routine({
        sourceBus = Bus.audio(server, 1);
        SynthDef(\scshaderAnalysisSmokeSource, { |out = 0|
            Out.ar(out, SinOsc.ar(440, 0, 0.1));
        }).add;
        server.sync;
        sourceSynth = Synth(\scshaderAnalysisSmokeSource, [\out, sourceBus], server);
        analysis = ShaderAnalysis(server, sourceBus.index,
            [\amplitude, \pitch, \confidence, \centroid, \flatness, \onset, \zeroCrossing], 30);
        analysis.start;
        1.5.wait;
        if(
            (analysis[\amplitude].value > 0.02)
            and: { analysis[\pitch].value > 300 }
            and: { analysis[\pitch].value < 600 }
            and: { analysis[\confidence].value > 0 }
            and: { analysis[\centroid].value > 100 }
            and: { analysis[\flatness].value >= 0 }
            and: { analysis[\zeroCrossing].value > 0 }
        ) {
            finish.value(true, "SCSHADER_ANALYSIS_ROUNDTRIP");
        } {
            finish.value(false, "SCSHADER_ANALYSIS_BAD_VALUES: %".format([
                analysis[\amplitude].value,
                analysis[\pitch].value,
                analysis[\confidence].value,
                analysis[\centroid].value,
                analysis[\flatness].value,
                analysis[\onset].value,
                analysis[\zeroCrossing].value
            ]));
        };
    }).play(SystemClock);
}, { |failedServer|
    finish.value(false, "SCSHADER_ANALYSIS_BOOT_FAILED: %".format(failedServer));
});
server.boot;
SystemClock.sched(12.0, { finish.value(false, "SCSHADER_ANALYSIS_TIMEOUT"); nil });
)
