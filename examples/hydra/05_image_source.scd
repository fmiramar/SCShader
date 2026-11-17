// Step 19. Recompile classes and reopen this file. Run startup once.
// Wait for IMAGE READY; then run one block at a time and wait for its result.
// BEGIN STARTUP
(
if(~hydraImageServer.notNil){Error("Run cleanup before restarting image sources").throw};
~hydraImageReady=false;
~hydraImageServer=ShaderServer.new(\hydraImages,"127.0.0.1",57210);
~hydraImageAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders";
~hydraImageServer.rendererArgs_(["--shader",~hydraImageAssets +/+ "hydra_black.wgsl"]);
~hydraImageServer.waitForBoot({|server|
 ~hydraImage=SCHydra.new(server);~hydraImage.waitForReady({|h|
  h.image(\s0,~hydraImageAssets +/+ "hydra_image_test.ppm",{|loaded|
   if(loaded[\ok]){h.src(\s0).out(\o0,{|r|if(r[\ok]){h.render(\o0,{|shown|~hydraImageReady=shown[\ok];"IMAGE READY: red/green above blue/yellow; stretched to the window".postln})}{r.postln}})}{loaded.postln};
  });
 },{|h,error|error.postln});
});
)
// END STARTUP

// After IMAGE READY, replace the image without recompiling the expression.
(
~hydraImage.image(\s0,~hydraImageAssets +/+ "hydra_image_second.ppm",{|r|r.postln});
)

// Process the loaded source. Scale/crop changes UV; aspect is stretched by default.
(
~hydraImage.src(\s0).scale(1.2).invert(1).out(\o0,{|r|r.postln});
)

// Clear test: first restore plain src, then clear the image. Expect black.
// This avoids invert(1) turning a cleared black source into white.
(
~hydraImage.src(\s0).out(\o0,{|ready|
    if(ready[\ok]) {~hydraImage.clearSource(\s0,{|r|r.postln})} {ready.postln};
});
)

// Cleanup releases source images, histories and this visual server.
(
~hydraImage.free;~hydraImageServer.quit({~hydraImageServer=nil;"IMAGES STOPPED".postln});
)
