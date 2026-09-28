"""Bounded cleanup for child processes owned by the validation runners."""
import os
import signal
import subprocess


def spawn_options():
    if os.name == "nt":
        return {"creationflags": 0x00000200}  # CREATE_NEW_PROCESS_GROUP
    return {"start_new_session": True}


def spawn_owned(command, **kwargs):
    """Start a test tree whose descendants are cleaned even if its root exits."""
    if os.name != "nt":
        return subprocess.Popen(command, **kwargs, **spawn_options())
    from windows_job import WindowsJob
    job = WindowsJob()
    process = None
    try:
        process = subprocess.Popen(command, **kwargs, creationflags=0x200 | 0x4)
        process._scshader_job = job
        job.assign_and_resume(process)
        return process
    except BaseException:
        job.close()
        if process is not None:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
        raise


def stop_process_tree(process, timeout=5):
    """Use only the supplied Popen's PID/group, never process-name matching."""
    if process.pid <= 0:
        raise ValueError("refusing an invalid child PID")
    job = process.__dict__.get("_scshader_job")
    if job is not None:
        job.close()
        process.wait(timeout=timeout)
        return
    if os.name == "nt":
        if process.poll() is not None:
            return
        try:
            subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                           check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                           timeout=timeout)
        except (OSError, subprocess.TimeoutExpired):
            process.kill()
    else:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        if os.name == "nt":
            process.kill()
        else:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        process.wait(timeout=timeout)
