# Provisional public-name collision check

Originally checked 2026-08-20 before adding any public SuperCollider classes. Rechecked 2026-09-12 immediately before introducing `ShaderServer`, `ShaderDef`, and `Shader`; rechecked 2026-09-13 before introducing `ShaderWindow`; and rechecked locally before introducing `ShaderBus`, `ShaderTexture`, `ShaderBuffer`, and `ShaderGraph`.

Proposed names:

```text
ShaderServer
ShaderDef
Shader
ShaderBus
ShaderTexture
ShaderBuffer
ShaderWindow
ShaderGraph
Pshader
```

No class-definition collision was found for `ShaderServer`, `ShaderDef`, `Shader`, `ShaderBus`, `ShaderTexture`, `ShaderBuffer`, `ShaderWindow`, or `ShaderGraph` in:

- the SuperCollider 3.14.1 application class library;
- the locally installed extension set plus the bundled class library, excluding SCShader itself;
- local sc3-plugins/source work and the surrounding UGen-port workspace, excluding this plan;
- the current official SuperCollider class index;
- the current `supercollider-quarks/quarks` directory metadata and exact-name public source searches.

This is evidence that the names are available, not a permanent reservation. Quark contents and installed user extensions can change independently. Repeat the collision check immediately before introducing each remaining public class and before publishing the Quark. The current implementation additionally defines analysis and stream helper classes that are longer SCShader-specific names and the documented public graph/control/resource classes above.
