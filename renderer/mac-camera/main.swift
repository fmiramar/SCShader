// Native, explicitly launched camera permission owner. No microphone or network input.
import AppKit
import AVFoundation
import Darwin

let frameWidth = 640, frameHeight = 360

func rgbaFrame(_ source: UnsafeRawPointer, _ width: Int, _ height: Int, _ stride: Int) -> Data {
    let scale = min(Double(frameWidth) / Double(width), Double(frameHeight) / Double(height))
    let drawnWidth = max(1, Int(Double(width) * scale))
    let drawnHeight = max(1, Int(Double(height) * scale))
    let left = (frameWidth - drawnWidth) / 2, top = (frameHeight - drawnHeight) / 2
    let input = source.assumingMemoryBound(to: UInt8.self)
    var result = Data(count: frameWidth * frameHeight * 4)
    result.withUnsafeMutableBytes { (bytes: UnsafeMutableRawBufferPointer) in
        let output = bytes.bindMemory(to: UInt8.self)
        for offset in Swift.stride(from: 3, to: output.count, by: 4) { output[offset] = 255 }
        for y in 0..<drawnHeight {
            let row = (y * height / drawnHeight) * stride
            for x in 0..<drawnWidth {
                let a = row + (x * width / drawnWidth) * 4
                let b = ((y + top) * frameWidth + x + left) * 4
                output[b] = input[a + 2]; output[b + 1] = input[a + 1]; output[b + 2] = input[a]
            }
        }
    }
    return result
}

// Pure pixel proof: padded BGRA rows, upright red/blue halves and opaque letterbox.
if CommandLine.arguments.contains("--pixel-test") {
    let width = 4, height = 4, rowBytes = 24
    var source = Data(count: rowBytes * height)
    source.withUnsafeMutableBytes { (bytes: UnsafeMutableRawBufferPointer) in
        for y in 0..<height { for x in 0..<width {
            let p = y * rowBytes + x * 4
            bytes[p] = y < 2 ? 0 : 255; bytes[p + 2] = y < 2 ? 255 : 0; bytes[p + 3] = 255
        }}
    }
    let frame = source.withUnsafeBytes { rgbaFrame($0.baseAddress!, width, height, rowBytes) }
    func pixel(_ x: Int, _ y: Int) -> [UInt8] { Array(frame[((y * frameWidth + x) * 4)..<((y * frameWidth + x) * 4 + 4)]) }
    precondition(pixel(320, 50) == [255, 0, 0, 255])
    precondition(pixel(320, 310) == [0, 0, 255, 255])
    precondition(pixel(0, 180) == [0, 0, 0, 255])
    print("SCSHADER_NATIVE_CAMERA_PIXELS_OK"); exit(0)
}

final class CameraApp: NSObject, NSApplicationDelegate, AVCaptureVideoDataOutputSampleBufferDelegate {
    private let session = AVCaptureSession()
    private let cameraQueue = DispatchQueue(label: "org.fmiramar.scshader.camera.frames")
    private let writeLock = NSLock()
    private var socket: FileHandle!
    private var ready = false

    private func argument(_ name: String) -> String? {
        guard let index = CommandLine.arguments.firstIndex(of: name), index + 1 < CommandLine.arguments.count else { return nil }
        return CommandLine.arguments[index + 1]
    }
    private func send(_ line: String) {
        do { try socket.write(contentsOf: Data((line + "\n").utf8)) } catch { exit(0) }
    }
    private func fail(_ message: String) -> Never {
        writeLock.lock()
        send("ERROR " + String(message.replacingOccurrences(of: "\n", with: " ").prefix(240)))
        exit(2)
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        signal(SIGPIPE, SIG_IGN)
        guard let path = argument("--socket"), let name = argument("--camera") else { exit(2) }
        let fd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { exit(2) }
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        let encoded = Array(path.utf8CString)
        guard encoded.count <= MemoryLayout.size(ofValue: address.sun_path) else { exit(2) }
        withUnsafeMutableBytes(of: &address.sun_path) { buffer in buffer.copyBytes(from: encoded.map { UInt8(bitPattern: $0) }) }
        let connected = withUnsafePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size)) }
        }
        guard connected == 0 else { exit(2) }
        socket = FileHandle(fileDescriptor: fd, closeOnDealloc: true)
        send("HELLO \(getpid())")
        // Closing the private controller socket ends capture, including if its parent crashes.
        let connection = socket!
        DispatchQueue.global().async {
            _ = try? connection.read(upToCount: 1)
            exit(0)
        }
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized: start(name)
        case .notDetermined:
            send("PERMISSION waiting")
            AVCaptureDevice.requestAccess(for: .video) { granted in
                DispatchQueue.main.async {
                    if granted { self.start(name) } else { self.fail("Camera permission denied for SCShader Camera. Enable it in System Settings > Privacy & Security > Camera, then reopen the source.") }
                }
            }
        case .denied: fail("Camera permission denied for SCShader Camera. Enable it in System Settings > Privacy & Security > Camera, then reopen the source.")
        case .restricted: fail("Camera access is restricted by macOS policy for SCShader Camera.")
        @unknown default: fail("Unknown macOS camera authorization status.")
        }
    }
    private func start(_ name: String) {
        send("PERMISSION authorized")
        var types: [AVCaptureDevice.DeviceType] = [.builtInWideAngleCamera]
        if #available(macOS 14.0, *) { types.append(.external) } else { types.append(.externalUnknown) }
        if #available(macOS 13.0, *) { types.append(.deskViewCamera) }
        let matches = AVCaptureDevice.DiscoverySession(deviceTypes: types, mediaType: .video, position: .unspecified).devices.filter { $0.localizedName == name }
        guard matches.count == 1 else { fail("Selected camera is absent or ambiguous. Enumerate devices again; no other camera was selected.") }
        let device = matches[0]
        do {
            session.beginConfiguration()
            guard session.canSetSessionPreset(.vga640x480) else { fail("Camera does not support the requested 640x480 capture preset.") }
            session.sessionPreset = .vga640x480
            let input = try AVCaptureDeviceInput(device: device)
            guard session.canAddInput(input) else { fail("Could not add the selected camera input; check whether another app is using it.") }
            session.addInput(input)
            let output = AVCaptureVideoDataOutput()
            output.videoSettings = [kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA]
            output.alwaysDiscardsLateVideoFrames = true
            output.setSampleBufferDelegate(self, queue: cameraQueue)
            guard session.canAddOutput(output) else { fail("Could not add the camera frame output.") }
            session.addOutput(output)
            session.commitConfiguration()
            try device.lockForConfiguration()
            if device.activeFormat.videoSupportedFrameRateRanges.contains(where: { $0.minFrameRate <= 30 && $0.maxFrameRate >= 30 }) {
                device.activeVideoMinFrameDuration = CMTime(value: 1, timescale: 30)
                device.activeVideoMaxFrameDuration = CMTime(value: 1, timescale: 30)
            }
            device.unlockForConfiguration()
            DispatchQueue.global().async { self.session.startRunning() }
        } catch { fail("Could not start the selected camera: \(error.localizedDescription)") }
    }
    func captureOutput(_ output: AVCaptureOutput, didOutput sampleBuffer: CMSampleBuffer, from connection: AVCaptureConnection) {
        guard let buffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        CVPixelBufferLockBaseAddress(buffer, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(buffer, .readOnly) }
        guard let base = CVPixelBufferGetBaseAddress(buffer) else { return }
        let width = CVPixelBufferGetWidth(buffer), height = CVPixelBufferGetHeight(buffer)
        guard width > 0 && height > 0 && CVPixelBufferGetPixelFormatType(buffer) == kCVPixelFormatType_32BGRA else { return }
        let frame = rgbaFrame(base, width, height, CVPixelBufferGetBytesPerRow(buffer))
        writeLock.lock(); defer { writeLock.unlock() }
        if !ready { send("READY \(width) \(height)"); ready = true }
        send("FRAME")
        do { try socket.write(contentsOf: frame) } catch { exit(0) }
    }
}

let app = NSApplication.shared
let delegate = CameraApp()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
