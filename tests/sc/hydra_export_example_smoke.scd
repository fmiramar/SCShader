// Opt-in: execute actual screenshot/record/resolution blocks back-to-back.
(
var root=ShaderDef.filenameSymbol.asString.dirname.dirname,text=File.readAllString(root +/+ "examples/hydra/11_export.scd"),extract,done=false,finish;
extract={|a,b|text.copyRange(text.find(a)+a.size,text.find(b)-1)};
finish={|ok,msg|var server=~hydraExportServer,exit={if(ok){"SCSHADER_HYDRA_EXPORT_EXAMPLE_OK".postln;0.exit}{1.exit}};
 if(done.not){done=true;msg.postln;~hydraExport.tryPerform(\free);if(server.notNil){server.quit(exit)}{exit.value};SystemClock.sched(2,{exit.value;nil})};
};
~hydraExportFolder="SCSHADER_EXPORT_TEST_DIR".getenv +/+ "example";
extract.value("// BEGIN STARTUP","// END STARTUP").replace("57218","57222")
 .replace("~hydraExportServer.waitForBoot","~hydraExportServer.rendererPath_(\"SCSHADER_RENDERER\".getenv);~hydraExportServer.waitForBoot").interpret;
Routine({while{~hydraExportReady!=true and:{done.not}}{0.05.wait};if(done.not){try{
 // Simulate running the next IDE blocks immediately, without waiting between them.
 extract.value("// BEGIN SCREENSHOT","// END SCREENSHOT").interpret;
 extract.value("// BEGIN RECORD","// END RECORD").replace("5.wait","1.2.wait").interpret;
 extract.value("// BEGIN RESOLUTION","// END RESOLUTION").interpret;
 extract.value("// BEGIN WINDOW","// END WINDOW").interpret;
 extract.value("// BEGIN FOLLOW","// END FOLLOW").interpret;
 extract.value("// BEGIN CLEANUP","// END CLEANUP").interpret;
 while{~hydraExportRunner.notNil}{0.05.wait};
 if(~hydraExportCompleted!=6 or:{~hydraExportTaskError.notNil}){Error("Queued example failed: "++~hydraExportTaskError).throw};
 if(~hydraExportSerial!=3){Error("Queued file numbering did not preserve all three exports").throw};
 while{~hydraExportClosing==true}{0.05.wait};~hydraExportStop.value;
 finish.value(true,"Actual example steps 2..7 queued back-to-back: PNG/MP4/resolution/window/follow and cleanup completed in order");
}{|error|error.reportError;finish.value(false,error.errorString)}}}).play(SystemClock);
SystemClock.sched(25,{finish.value(false,"Queued export example timed out");nil});
)
