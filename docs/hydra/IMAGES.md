# Static source images — Step 19

`h.image(\s0, path, action)` loads/replaces one of s0…s3 using the existing bounded
PNG/JPEG/PNM decoder. `h.src(\s0)` samples that source; `h.src(\o0)` still samples
output history. Other symbols/kinds and out-of-range slots are rejected. Unassigned
or cleared sources sample transparent black. `h.clearSource(\s0, action)` releases
the image without replacing output programs or incrementing their revisions.
Wait for each callback before issuing another structural request.

Loading decodes a candidate before installing it; failed decode, invalid dimensions
or candidate validation keeps the accepted source. Acknowledgement means the source
is installed; it becomes visible on a following rendered frame. Source edits rebind
live inputs without recompiling the output expression. Each slot owns one static
texture plus immutable decoded CPU pixels. Replacement drops superseded resources
through wgpu's normal in-flight retirement. Sources are never render attachments.

Images are limited to 4096 pixels per axis or the device limit if smaller. The
shared decoder's allocation hint is 128 MiB; it is not a hard process-memory bound.
Four full-size RGBA8 images use at most 256 MiB GPU plus 256 MiB decoded CPU storage.
A replacement can transiently retain a fifth image; other output/history, decoder,
swapchain and driver allocations are additional. Source bytes are included in the
renderer texture-byte estimate. Window resize retains intrinsic image dimensions
and shares the existing texture/CPU cache; it rebuilds output histories separately.
Recovery reconstruction uploads cached pixels and does not reread source files;
source-specific logical-device injection remains unqualified in this step.

Source pixels use straight RGBA8Unorm and preserve encoded RGB channel numbers,
unlike legacy ShaderTexture's sRGB sampling conversion. Top-left decoded rows are
presented upright. No additional ICC/EXIF transformations are introduced. Sampling
uses the existing linear clamp sampler with wrapped expression coordinates. The
full image stretches across the normalized output rectangle by default; there is
no automatic aspect fit/letterboxing. Use explicit scale/coordinate operations
for a chosen crop or aspect layout. PNG alpha is retained; RGB/JPEG become opaque.

`hush` replaces output programs with transparent solids but retains loaded source
images for reuse. Explicit clearSource or closing/freeing the session releases them.
This is a native difference from the pinned upstream hush clearing its sources.
Selecting Legacy also retains images. Static file decode/upload occurs on the
existing renderer control path and can delay command service; this is not an
asynchronous video/media decoder or a live file watcher.

Protocol: `image(epoch:s,session:i,sequence:i,slot:i,path:s)` is immediate-only and
returns the ordinary ack. Empty wire path clears the slot; paths are bounded to
4096 UTF-8 bytes. Accepted sequences/path retries are idempotent per source slot;
stale/conflicting sequences are rejected. Decode/preparation errors use
E_HYDRA_IMAGE. Failed requests do not replace the last accepted source or sequence.
`image-sources-v1` advertises support. HYIR scalar sampler tag 3 carries a u8 source
slot; output tag 4 remains distinct. Both are valid arguments to src under ABI 1.

`examples/hydra/05_image_source.scd` loads a shipped four-color PNM fixture, replaces
it with a second portrait fixture, processes it with scale/invert and clears it.
The clear block restores plain src first so the expected result is black. Each
source operation prints its result. Run cleanup before restarting the example.
The installed smoke executes actual startup, replacement, a missing-file rejection,
resize, clear and a source/output-kind rejection. Production Metal pixel tests
cover empty sources, row orientation, exact RGB values, 32 replacements, retained
resize data, clear and resource retirement. Shared-loader legacy regression and
prior Hydra checks remain enabled. See VERIFICATION.md for precise results;
manual appearance, other backends and full browser parity remain separate.
