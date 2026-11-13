(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/04_audio_reactive.scd",text=File.readAllString(file),start,end,done=false,finish,ampMax=0,ampMin=1,onsetMax=0,fft,wave,fftMax=0,waveMax=0;
finish={|ok,message|if(done.not){done=true;message.postln;fft.tryPerform(\free);wave.tryPerform(\free);~hydraAudio.tryPerform(\free);if(~hydraAudioAnalysis.notNil){ok=ok and:{~hydraAudioAnalysis.buses.values.every {|b|b.hydraMappingCount(~hydraAudio)==0}}};~hydraAudioAnalysis.tryPerform(\free);~hydraAudioAnalysis.tryPerform(\buses).tryPerform(\values).do(_.free);~hydraAudioSource.tryPerform(\free);~hydraAudioBuffer.tryPerform(\free);~hydraAudioBus.tryPerform(\free);~hydraAudioServer.tryPerform(\quit);~hydraAudioVisualServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_AUDIO_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
start=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;end=text.find("// END STARTUP");
text.copyRange(start,end-1).replace("~hydraAudioServer=Server.default;","~hydraAudioServer=Server(\\hydraAudioOwned,NetAddr(\"127.0.0.1\",57205),ServerOptions.new.numInputBusChannels_(0));").replace("57204","57206").replace("~hydraAudioMonitor=0.2","~hydraAudioMonitor=0").interpret;
Routine({while{~hydraAudioReady!=true and:{done.not}}{0.1.wait};if(done.not){
fft=ShaderFFTTexture(~hydraAudioServer,~hydraAudioBus.index,256,15,~hydraAudioVisualServer);
wave=ShaderWaveformTexture(~hydraAudioServer,~hydraAudioBus.index,64,0.05,15,~hydraAudioVisualServer);
fft.binBus(2,0.2);wave.pointBus(16,0.2);fft.start;wave.start;
400.do{var a=~hydraAudioAnalysis[\amplitude].value,o=~hydraAudioAnalysis[\onset].value;ampMax=ampMax.max(a);ampMin=ampMin.min(a);onsetMax=onsetMax.max(o);fftMax=fftMax.max(fft.binBus(2).value);waveMax=waveMax.max(wave.pointBus(16).value.abs);0.01.wait};[ampMin,ampMax,onsetMax,fftMax,waveMax].postln;finish.value(ampMax>0.005 and:{ampMin<0.001 and:{onsetMax>0.1 and:{fftMax>0.00001 and:{waveMax>0.00001 and:{~hydraAudioVisualServer.lastError.isNil}}}}},"Looping PlayBuf amplitude/onset and two Hydra bus mappings")}}).play(SystemClock);
SystemClock.sched(25,{finish.value(false,"Audio bridge timeout");nil});
)
