import sys
import os
import threading
import logging
import json
import time

logging.disable(logging.CRITICAL)
os.environ["TQDM_DISABLE"] = "1"


class DownloadProgressPrinter:
    _lock = threading.Lock()
    _file_index = 0
    _progress_file = None
    _total_downloaded = 0
    _total_files = 0
    _total_file_size = 0

    def __init__(self, filename, file_size):
        with DownloadProgressPrinter._lock:
            DownloadProgressPrinter._file_index += 1
            DownloadProgressPrinter._total_files += 1
            DownloadProgressPrinter._total_file_size += file_size
            self.index = DownloadProgressPrinter._file_index
            self.filename = filename
            self.file_size = file_size
            self._downloaded = 0
            self._last_percent = 0.0
            self._last_write_time = 0
        _write_progress("downloading", filename, 0.0, f"正在下载: {filename}")

    def update(self, chunk_size):
        with DownloadProgressPrinter._lock:
            self._downloaded += chunk_size
            if self.file_size > 0:
                percent = round(self._downloaded / self.file_size * 100, 1)
            else:
                percent = 0.0
            now = time.time()
            if percent != self._last_percent or now - self._last_write_time > 1.0:
                self._last_percent = percent
                self._last_write_time = now
                _write_progress("downloading", self.filename, percent, f"正在下载: {self.filename} ({percent}%)")

    def end(self):
        with DownloadProgressPrinter._lock:
            _write_progress("downloading", self.filename, 100.0, f"已完成: {self.filename}")


def _write_progress(status, current_file, percent, message):
    if not DownloadProgressPrinter._progress_file:
        return
    try:
        data = {
            "status": status,
            "current_file": current_file,
            "progress_percent": percent,
            "message": message,
        }
        with DownloadProgressPrinter._lock:
            with open(DownloadProgressPrinter._progress_file, "w", encoding="utf-8") as f:
                json.dump(data, f, ensure_ascii=False)
    except Exception:
        pass


def main():
    if len(sys.argv) < 4:
        sys.exit(1)

    model_id = sys.argv[1]
    local_dir = sys.argv[2]
    progress_file = sys.argv[3]

    DownloadProgressPrinter._progress_file = progress_file

    _write_progress("downloading", "", 0.0, f"开始下载模型 {model_id}")

    try:
        from modelscope.hub.snapshot_download import snapshot_download

        DownloadProgressPrinter._file_index = 0
        DownloadProgressPrinter._total_downloaded = 0
        DownloadProgressPrinter._total_files = 0
        DownloadProgressPrinter._total_file_size = 0

        _write_progress("downloading", "", 0.0, "正在获取文件列表...")

        result_path = snapshot_download(
            model_id=model_id,
            local_dir=local_dir,
            progress_callbacks=[DownloadProgressPrinter],
        )

        _write_progress("completed", "", 100.0, f"下载完成: {result_path}")

    except ImportError as e:
        _write_progress("error", "", 0.0, f"缺少依赖: {e}")
        sys.exit(2)
    except Exception as e:
        _write_progress("error", "", 0.0, f"下载失败: {e}")
        sys.exit(3)


if __name__ == "__main__":
    main()
