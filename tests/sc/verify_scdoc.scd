(
var pages = ["Classes/ShaderServer", "Classes/ShaderWindow", "Classes/ShaderTexture", "Classes/ShaderBuffer", "Classes/ShaderFFTTexture", "Classes/ShaderWaveformTexture", "Classes/ShaderGraph", "Classes/ShaderBus", "Classes/ShaderAnalysis", "Classes/ShaderDef", "Classes/ShaderToy", "Classes/Shader", "Guides/SCShader"];
var ok = true;
SCDoc.verbosity = 2;
SCDoc.indexAllDocuments(true);
pages.do { |path|
    var document = SCDoc.documents[path];
    if(document.isNil) { ("MISSING_DOC " ++ path).postln; ok = false } {
        var root = SCDoc.parseDoc(document);
        if(root.isNil) { ("PARSE_FAILED " ++ path).postln; ok = false } {
            SCDoc.renderer.renderToFile(document.destPath, document, root);
            ("RENDERED " ++ path).postln;
        }
    }
};
if(ok) { "SCSHADER_SCDOC_OK".postln; 0.exit } {
    "SCSHADER_SCDOC_FAILED".postln; 1.exit
};
)
