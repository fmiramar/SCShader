(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var examplePath = "SCSHADER_EXAMPLE".getenv ? (project +/+ "examples/04_typed_controls.scd");
var finish, completed = false;
finish = { |passed|
    if(completed.not) {
        completed = true;
        v.tryPerform(\quit);
        SystemClock.sched(0.5, {
            passed.if({ "SCSHADER_TYPED_EXAMPLE_OK".postln; 0.exit }, {
                "SCSHADER_TYPED_EXAMPLE_FAILED".postln; 1.exit
            }); nil
        });
    };
};
thisProcess.interpreter.executeFile(examplePath);
// The two-second glide begins after asynchronous GPU/shader initialization.
// Observe its exact endpoint within the existing overall timeout rather than
// assuming startup took less than 1.5 seconds on every driver.
SystemClock.sched(0.1, {
    if(completed) { nil } {
        if(v.isRunning and: { x.notNil and: { x.isRunning } }) {
            if(x.lastError.notNil) { finish.value(false) } {
                x.get(\color, { |value|
                    if(value.notNil and: { (value - [0.9, 0.2, 0.5]).abs.maxItem < 0.00001 }) {
                        finish.value(true);
                    };
                });
            };
        };
        0.1
    }
});
SystemClock.sched(8, { finish.value(false); nil });
)
