// Renderer-side timed-event benchmark. This measures queued event application,
// not GPU presentation or display scan-out. Run it once on a 60 Hz display and,
// where available, once on a 120 Hz display; it intentionally makes no claim
// that either result is sample-accurate.
(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var server = ShaderServer.new(\scshaderTimingBenchmark, "127.0.0.1", 57145);
var events = 32, interval = 1 / 60, results = List.new;
var complete = false, finish, report;

report = {
    var sorted = results.copy.sort;
    var percentile = { |fraction| sorted[((sorted.size - 1) * fraction).round.asInteger] };
    (
        events: sorted.size,
        meanMs: (sorted.sum / sorted.size) * 1000,
        medianMs: percentile.value(0.50) * 1000,
        p95Ms: percentile.value(0.95) * 1000,
        p99Ms: percentile.value(0.99) * 1000,
        maxMs: sorted.last * 1000
    )
};

finish = { |passed, message|
    if(complete.not) {
        complete = true;
        message.postln;
        server.quit({ passed.if({ 0.exit }, { 1.exit }) });
        SystemClock.sched(2.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

if(PathName(rendererPath).isFile.not) {
    finish.value(false, "SCSHADER_TIMING_BENCHMARK_MISSING_RENDERER");
} {
    server.rendererPath_(rendererPath);
    server.waitForBoot({ |shaderServer|
        shaderServer.clockSync(8, 0.02, 1.0, { |receiver, estimate|
            events.do { |index|
                shaderServer.scheduleMarker(0.12 + (index * interval), { |timingServer, timing|
                    results.add(timing[\appliedRendererTime] - timing[\scheduledRendererTime]);
                    if(results.size == events) {
                        [\SCShaderTimingBenchmark, report.value].postln;
                        finish.value(true, "SCSHADER_TIMING_BENCHMARK_OK");
                    };
                }, [[
                    '/scshader/v1/uniform/f', 1, "amount", ((index % 2) == 0).if(0.15, 0.9)
                ]]);
            };
        });
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_TIMING_BENCHMARK_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(12.0, {
        finish.value(false, "SCSHADER_TIMING_BENCHMARK_TIMEOUT");
        nil
    });
};
)
