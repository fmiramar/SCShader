// Original macOS-only diagnostic: isolate Metal command-buffer allocation from
// SCShader, wgpu, OSC, and window presentation. Not part of the renderer build.
// SPDX-License-Identifier: GPL-3.0-or-later
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <mach/mach.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned long long resident_bytes(void) {
    struct mach_task_basic_info info;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO,
                  (task_info_t)&info, &count) != KERN_SUCCESS) return 0;
    return info.resident_size;
}

int main(int argc, const char **argv) {
    BOOL unretained = NO, single = NO, signalEvent = NO, completion = NO, labels = NO, split = NO;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "unretained") == 0) unretained = YES;
        else if (strcmp(argv[i], "single-pass") == 0) single = YES;
        else if (strcmp(argv[i], "signal-event") == 0) signalEvent = YES;
        else if (strcmp(argv[i], "completion-handler") == 0) completion = YES;
        else if (strcmp(argv[i], "labels") == 0) labels = YES;
        else if (strcmp(argv[i], "split-buffers") == 0) split = YES;
        else {
            fprintf(stderr, "usage: metal_memory_probe [unretained] [single-pass] [signal-event] [completion-handler] [labels] [split-buffers]\n");
            return 2;
        }
    }
    @autoreleasepool {
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        id<MTLCommandQueue> queue = [device newCommandQueueWithMaxCommandBufferCount:4096];
        id<MTLSharedEvent> event = signalEvent ? [device newSharedEvent] : nil;
        if (signalEvent && !event) return 1;
        MTLTextureDescriptor *desc = [MTLTextureDescriptor
            texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
            width:256 height:256 mipmapped:NO];
        desc.usage = MTLTextureUsageRenderTarget;
        desc.storageMode = MTLStorageModePrivate;
        id<MTLTexture> texture = [device newTextureWithDescriptor:desc];
        id<MTLTexture> secondTexture = [device newTextureWithDescriptor:desc];
        if (!device || !queue || !texture || !secondTexture) return 1;
        NSError *error = nil;
        NSString *source = @"#include <metal_stdlib>\nusing namespace metal;\n"
            "vertex float4 vertex_main(uint i [[vertex_id]]) { "
            "float2 p[3] = {float2(-1,-1), float2(3,-1), float2(-1,3)}; return float4(p[i],0,1); }\n"
            "fragment float4 fragment_main(constant float4 &color [[buffer(0)]]) { return color; }";
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        MTLRenderPipelineDescriptor *pipelineDesc = [MTLRenderPipelineDescriptor new];
        pipelineDesc.vertexFunction = [library newFunctionWithName:@"vertex_main"];
        pipelineDesc.fragmentFunction = [library newFunctionWithName:@"fragment_main"];
        pipelineDesc.colorAttachments[0].pixelFormat = MTLPixelFormatBGRA8Unorm;
        id<MTLRenderPipelineState> pipeline = [device newRenderPipelineStateWithDescriptor:pipelineDesc error:&error];
        if (!pipeline) {
            fprintf(stderr, "%s\n", error.description.UTF8String);
            return 1;
        }
        float color[4] = {0.1, 0.2, 0.3, 1};
        id<MTLBuffer> uniform = [device newBufferWithBytes:color length:sizeof(color)
            options:MTLResourceStorageModeShared];
        if (!uniform) return 1;
        printf("device=%s unretained=%d passes=%d signal=%d completion=%d labels=%d split=%d\n",
            device.name.UTF8String, unretained, single ? 1 : 2, signalEvent, completion, labels, split);
        for (unsigned frame = 0; frame <= 20000; ++frame) {
            @autoreleasepool {
                id<MTLCommandBuffer> buffer = unretained
                    ? [queue commandBufferWithUnretainedReferences] : [queue commandBuffer];
                if (labels) buffer.label = @"probe command buffer";
                for (int passIndex = 0; passIndex < (single ? 1 : 2); ++passIndex) {
                    MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
                    pass.colorAttachments[0].texture = passIndex == 0 ? texture : secondTexture;
                    pass.colorAttachments[0].loadAction = MTLLoadActionClear;
                    pass.colorAttachments[0].storeAction = MTLStoreActionStore;
                    pass.colorAttachments[0].clearColor = MTLClearColorMake(0.1, 0.2, 0.3, 1);
                    id<MTLRenderCommandEncoder> encoder = [buffer renderCommandEncoderWithDescriptor:pass];
                    if (!buffer || !encoder) return 1;
                    if (labels) encoder.label = @"probe pass";
                    [encoder setRenderPipelineState:pipeline];
                    [encoder setFragmentBuffer:uniform offset:0 atIndex:0];
                    [encoder drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
                    [encoder endEncoding];
                    if (split && passIndex == 0 && !single) {
                        [buffer commit];
                        buffer = unretained ? [queue commandBufferWithUnretainedReferences] : [queue commandBuffer];
                        if (labels) buffer.label = @"probe second command buffer";
                    }
                }
                if (signalEvent) [buffer encodeSignalEvent:event value:frame + 1];
                if (completion) [buffer addCompletedHandler:^(id<MTLCommandBuffer> completed) {
                    (void)completed;
                }];
                [buffer commit];
                [buffer waitUntilCompleted];
                if (buffer.status == MTLCommandBufferStatusError) {
                    fprintf(stderr, "%s\n", buffer.error.description.UTF8String);
                    return 1;
                }
            }
            if (frame % 2000 == 0) {
                unsigned long long rss = resident_bytes();
                printf("frames=%u rss_kib=%llu\n", frame, rss / 1024);
                fflush(stdout);
                if (!rss || rss > 512ULL * 1024 * 1024) return 1;
            }
        }
    }
    return 0;
}
