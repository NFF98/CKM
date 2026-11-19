#!/usr/bin/env python3
"""CAKAMI: copy a verified E2B model between two debug apps on ONE Android phone.

Copies only public Gemma weights from the installed original app's private
files dir to the isolated Sprint 1 trial app. No Internet, no reinstall,
no app data deletion and no 2.59 GB PC temporary file are involved.

Requirements: Android adb on PATH, USB debugging enabled, both APKs installed,
both packages debuggable (adb shell run-as succeeds), ~2.6 GB free on phone.
"""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import sys
import time

ADB = shutil.which("adb")
OLD = "com.nff98.ckm.dev"
NEW = "com.nff98.ckm.dev.s1trial"
NAME = "gemma-4-E2B-it.litertlm"
MODEL = "files/models/" + NAME
MARKER = MODEL + ".sha256"
STAGING = MODEL + ".ca-copying"
EXPECTED_BYTES = 2_588_147_712
EXPECTED_SHA = "181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c"
CHUNK = 1024 * 1024


def run(*args: str, timeout: int = 45, input_bytes: bytes | None = None,
        check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run([ADB, *args], input=input_bytes,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=timeout)
    if check and result.returncode:
        message = (result.stderr or result.stdout).decode("utf-8", "replace").strip()
        raise RuntimeError(f"adb {' '.join(args[:4])}: {message}")
    return result


def shell(*args: str, check: bool = True) -> str:
    r = run("shell", *args, check=check)
    return r.stdout.decode("utf-8", "replace").strip() if r.returncode == 0 else ""


def app_file(pkg: str, command: str, *args: str, check: bool = True) -> str:
    return shell("run-as", pkg, command, *args, check=check)


def require_debug(pkg: str) -> None:
    installed = shell("pm", "path", pkg, check=False)
    if not installed.startswith("package:"):
        raise RuntimeError(f"手機找不到 {pkg}；必須先安裝兩個 App，不能直接共用儲存。")
    probe = run("shell", "run-as", pkg, "id", check=False)
    if probe.returncode:
        raise RuntimeError(
            f"{pkg} 不是可透過 run-as 存取的 Debug App。\n"
            "請保留所有 App；不能靠一般 App 權限跨沙盒複製模型。"
        )


def file_size(pkg: str, path: str) -> int | None:
    value = app_file(pkg, "stat", "-c", "%s", path, check=False)
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def marker(pkg: str) -> str:
    return app_file(pkg, "cat", MARKER, check=False).strip()


def transfer() -> None:
    print("CAKAMI｜Gemma 本機複用工具（不連網、不卸載）", flush=True)
    if not ADB:
        raise RuntimeError("找不到 adb。請安裝 Android SDK Platform Tools 並放到 PATH。")
    devices = run("devices").stdout.decode("utf-8", "replace").splitlines()[1:]
    authorized = [line for line in devices if "\tdevice" in line]
    pending = [line for line in devices if "\tunauthorized" in line]
    if len(authorized) != 1 or pending:
        raise RuntimeError("只能連接一台已允許 USB 偵錯的 Android 手機；請執行 adb devices。")

    for pkg in (OLD, NEW):
        require_debug(pkg)

    if file_size(OLD, MODEL) != EXPECTED_BYTES or marker(OLD) != EXPECTED_SHA:
        raise RuntimeError(
            "舊 App 找不到完整已驗證的 Gemma E2B（或版本不同）。"
            "不會修改舊 App；請先確認舊版的模型狀態。"
        )

    if file_size(NEW, MODEL) == EXPECTED_BYTES and marker(NEW) == EXPECTED_SHA:
        print("新 App 已有相同模型，無需複製。")
        return

    # Only stop the background activity. This does NOT uninstall, clear,
    # replace or modify any of the old application's files.
    for pkg in (OLD, NEW):
        shell("am", "force-stop", pkg)

    app_file(NEW, "mkdir", "-p", "files/models")
    print("正在透過 USB 把舊 E2B 複製至新 App；舊模型不會被移動或刪除。", flush=True)
    print("無需 Wi-Fi。傳輸約 2.59 GB，且不暫存在 Windows 磁碟。", flush=True)
    old_reader = None
    new_writer = None
    count = 0
    digest = hashlib.sha256()
    started = time.monotonic()
    try:
        old_reader = subprocess.Popen(
            [ADB, "exec-out", "run-as", OLD, "cat", MODEL],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, bufsize=CHUNK)
        new_writer = subprocess.Popen(
            [ADB, "shell", "-T", "run-as", NEW, "dd",
             f"of={STAGING}", "bs=1048576"],
            stdin=subprocess.PIPE, stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE, bufsize=CHUNK)
        if old_reader.stdout is None or new_writer.stdin is None:
            raise RuntimeError("無法建立 USB 模型傳輸管道。")
        next_report = 0
        while True:
            data = old_reader.stdout.read(CHUNK)
            if not data:
                break
            count += len(data)
            if count > EXPECTED_BYTES:
                raise RuntimeError("原始模型長度超出驗證值，停止。")
            digest.update(data)
            new_writer.stdin.write(data)
            if count >= next_report:
                pct = 100 * count / EXPECTED_BYTES
                print(f"\rUSB 複製進度 {pct:5.1f}%", end="", flush=True)
                next_report = count + 100 * CHUNK
        old_reader.stdout.close()
        new_writer.stdin.close()
        old_err = old_reader.stderr.read() if old_reader.stderr else b""
        new_err = new_writer.stderr.read() if new_writer.stderr else b""
        old_code = old_reader.wait(timeout=60)
        new_code = new_writer.wait(timeout=60)
        print()
        if old_code or new_code:
            raise RuntimeError("USB 讀寫失敗："
                + (old_err + new_err).decode("utf-8", "replace")[-300:])
        if count != EXPECTED_BYTES or digest.hexdigest() != EXPECTED_SHA:
            raise RuntimeError("複製資料 SHA-256 或長度不符；拒絕安裝模型。")
        if file_size(NEW, STAGING) != EXPECTED_BYTES:
            raise RuntimeError("手機端檔案長度不符；拒絕啟用模型。")
        app_file(NEW, "mv", "-f", STAGING, MODEL)
        # Marker is written only AFTER the entire model has passed the hash check.
        run("shell", "-T", "run-as", NEW, "dd", f"of={MARKER}",
            input_bytes=EXPECTED_SHA.encode("ascii"))
        if file_size(NEW, MODEL) != EXPECTED_BYTES or marker(NEW) != EXPECTED_SHA:
            raise RuntimeError("手機模型驗證標記失敗。")
        elapsed = int(time.monotonic() - started)
        print(f"完成：舊 E2B 已安全複製到獨立測試版（{elapsed} 秒）。")
        print("舊 CAKAMI 的模型與私人收藏完全保留。")
        print("現在可開啟 CAKAMI TEST 查看 Gemma 狀態；不需再次連網下載。")
    except (Exception, KeyboardInterrupt):
        for process in (old_reader, new_writer):
            if process and process.poll() is None:
                process.kill()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    pass
        # Remove only this script's incomplete temporary file in NEW.
        try:
            app_file(NEW, "rm", "-f", STAGING)
        except Exception:
            pass
        raise


if __name__ == "__main__":
    try:
        transfer()
    except KeyboardInterrupt:
        print("\n已中止；舊版模型與收藏未變動。", file=sys.stderr)
        sys.exit(130)
    except (RuntimeError, subprocess.SubprocessError, OSError) as exc:
        print(f"\n安全停止：{exc}\n沒有刪除任何 App 或舊版資料。", file=sys.stderr)
        sys.exit(1)
