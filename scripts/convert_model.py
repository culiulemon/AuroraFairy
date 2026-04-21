import sys
import os
import json
import time
import warnings
import logging
from pathlib import Path

warnings.filterwarnings('ignore')
logging.disable(logging.CRITICAL)
os.environ["TQDM_DISABLE"] = "1"


def write_progress(progress_file, status, current_file, percent, message):
    if not progress_file:
        return
    try:
        data = {
            "status": status,
            "current_file": current_file,
            "progress_percent": percent,
            "message": message,
        }
        with open(progress_file, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False)
    except Exception:
        pass


def main():
    if len(sys.argv) < 4:
        sys.exit(1)

    model_dir = sys.argv[1]
    output_dir = sys.argv[2]
    progress_file = sys.argv[3]
    device = sys.argv[4] if len(sys.argv) > 4 else "GPU"

    write_progress(progress_file, "converting", "", 0.0, f"开始转换模型: {model_dir}")

    model_path = Path(model_dir)
    output_path = Path(output_dir)

    if not model_path.exists():
        write_progress(progress_file, "error", "", 0.0, f"模型目录不存在: {model_dir}")
        sys.exit(1)

    if output_path.exists():
        ov_xml = output_path / "openvino_model.xml"
        ov_bin = output_path / "openvino_model.bin"
        if ov_xml.exists() and ov_bin.exists():
            write_progress(progress_file, "completed", "", 100.0, f"模型已转换: {output_dir}")
            sys.exit(0)

    try:
        write_progress(progress_file, "converting", "", 10.0, "正在加载转换工具...")

        from optimum.intel import OVModelForCausalLM
        from transformers import AutoTokenizer

        write_progress(progress_file, "converting", "", 20.0, "正在加载模型...")

        model = OVModelForCausalLM.from_pretrained(
            str(model_path),
            export=True,
            device=device,
            ov_config={"PERFORMANCE_HINT": "LATENCY"},
        )

        write_progress(progress_file, "converting", "", 60.0, "正在加载分词器...")

        tokenizer = AutoTokenizer.from_pretrained(str(model_path), trust_remote_code=True)

        write_progress(progress_file, "converting", "", 70.0, f"正在保存到 {output_dir}...")

        output_path.mkdir(parents=True, exist_ok=True)
        model.save_pretrained(str(output_path))
        tokenizer.save_pretrained(str(output_path))

        write_progress(progress_file, "converting", "", 90.0, "正在验证转换结果...")

        ov_xml = output_path / "openvino_model.xml"
        ov_bin = output_path / "openvino_model.bin"
        if not ov_xml.exists() or not ov_bin.exists():
            raise FileNotFoundError(f"转换后的文件不存在: {ov_xml}")

        write_progress(progress_file, "completed", "", 100.0, f"转换完成: {output_dir}")

    except ImportError as e:
        write_progress(progress_file, "error", "", 0.0, f"缺少依赖: {e} (请运行 pip install optimum[openvino])")
        sys.exit(2)
    except Exception as e:
        write_progress(progress_file, "error", "", 0.0, f"转换失败: {e}")
        if output_path.exists():
            import shutil
            try:
                shutil.rmtree(output_path)
            except Exception:
                pass
        sys.exit(3)


if __name__ == "__main__":
    main()
