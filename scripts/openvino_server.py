import argparse
import json
import queue
import threading
import time
import traceback
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path


class ModelManager:
    def __init__(self):
        self._lock = threading.Lock()
        self._pipeline = None
        self._model_name = None
        self._model_path = None
        self._device = "GPU"
        self._loading = False

    @property
    def model_name(self):
        with self._lock:
            return self._model_name

    @property
    def model_path(self):
        with self._lock:
            return self._model_path

    @property
    def is_loaded(self):
        with self._lock:
            return self._pipeline is not None

    @property
    def is_loading(self):
        with self._lock:
            return self._loading

    def load_model(self, model_path: str, model_name: str, device: str = "GPU"):
        with self._lock:
            if self._loading:
                raise RuntimeError("Another model is currently loading")
            if self._pipeline is not None:
                self._pipeline = None
                self._model_name = None
                self._model_path = None
            self._loading = True

        try:
            import openvino_genai

            if not Path(model_path).exists():
                raise FileNotFoundError(f"Model path not found: {model_path}")

            pipeline = openvino_genai.LLMPipeline(model_path, device)

            with self._lock:
                self._pipeline = pipeline
                self._model_name = model_name
                self._model_path = model_path
                self._device = device

            return {"status": "ok", "model_name": model_name, "device": device}
        except Exception as e:
            with self._lock:
                self._pipeline = None
                self._model_name = None
                self._model_path = None
            raise e
        finally:
            with self._lock:
                self._loading = False

    def unload_model(self):
        with self._lock:
            if self._pipeline is None:
                return {"status": "ok", "message": "No model loaded"}
            self._pipeline = None
            self._model_name = None
            self._model_path = None
        return {"status": "ok", "message": "Model unloaded"}

    def _make_config(self, max_new_tokens=512, temperature=0.7, top_p=0.9, repetition_penalty=1.1):
        import openvino_genai
        config = openvino_genai.GenerationConfig()
        config.max_new_tokens = max_new_tokens
        config.temperature = temperature
        config.top_p = top_p
        config.repetition_penalty = repetition_penalty
        config.do_sample = temperature > 0
        return config

    def generate(self, prompt: str, max_new_tokens: int = 512,
                 temperature: float = 0.7, top_p: float = 0.9,
                 repetition_penalty: float = 1.1) -> str:
        with self._lock:
            if self._pipeline is None:
                raise RuntimeError("No model loaded")
            pipeline = self._pipeline

        config = self._make_config(max_new_tokens, temperature, top_p, repetition_penalty)
        result = pipeline.generate(prompt, config)
        if hasattr(result, 'texts'):
            return result.texts[0] if result.texts else ""
        return str(result)

    def generate_stream(self, prompt: str, max_new_tokens: int = 512,
                        temperature: float = 0.7, top_p: float = 0.9,
                        repetition_penalty: float = 1.1):
        with self._lock:
            if self._pipeline is None:
                raise RuntimeError("No model loaded")
            pipeline = self._pipeline

        config = self._make_config(max_new_tokens, temperature, top_p, repetition_penalty)

        text_queue = queue.Queue()
        error_holder = [None]
        done_event = threading.Event()

        def streamer(subword):
            text_queue.put(subword)
            return False

        def run_generation():
            try:
                pipeline.generate(prompt, config, streamer=streamer)
            except Exception as e:
                error_holder[0] = e
            finally:
                done_event.set()

        gen_thread = threading.Thread(target=run_generation, daemon=True)
        gen_thread.start()

        while not done_event.is_set() or not text_queue.empty():
            try:
                chunk = text_queue.get(timeout=0.1)
                yield chunk
            except queue.Empty:
                if error_holder[0] is not None:
                    raise error_holder[0]
                continue

        if error_holder[0] is not None:
            raise error_holder[0]

    def get_status(self):
        with self._lock:
            return {
                "loaded": self._pipeline is not None,
                "loading": self._loading,
                "model_name": self._model_name,
                "model_path": self._model_path,
                "device": self._device,
            }


manager = ModelManager()


class OpenVINORequestHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def _send_cors_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")

    def _read_body(self):
        content_length = int(self.headers.get("Content-Length", 0))
        if content_length == 0:
            return {}
        body = self.rfile.read(content_length)
        return json.loads(body)

    def _send_json(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self._send_cors_headers()
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False).encode("utf-8"))

    def _send_error(self, message, status=500):
        self._send_json({"error": {"message": message, "type": "server_error"}}, status)

    def do_OPTIONS(self):
        self.send_response(204)
        self._send_cors_headers()
        self.end_headers()

    def do_GET(self):
        if self.path == "/health":
            self._send_json({"status": "ok"})
        elif self.path == "/api/status":
            self._send_json(manager.get_status())
        elif self.path == "/v1/models":
            self._handle_list_models()
        else:
            self._send_error("Not found", 404)

    def do_POST(self):
        try:
            if self.path == "/api/load":
                self._handle_load()
            elif self.path == "/api/unload":
                self._handle_unload()
            elif self.path == "/v1/chat/completions":
                self._handle_chat_completions()
            elif self.path == "/v1/completions":
                self._handle_completions()
            else:
                self._send_error("Not found", 404)
        except Exception as e:
            traceback.print_exc()
            self._send_error(str(e))

    def _handle_load(self):
        body = self._read_body()
        model_path = body.get("model_path", "")
        model_name = body.get("model_name", "")
        device = body.get("device", "GPU")

        if not model_path:
            self._send_error("model_path is required", 400)
            return

        try:
            result = manager.load_model(model_path, model_name or "default", device)
            self._send_json(result)
        except Exception as e:
            self._send_error(f"Failed to load model: {e}")

    def _handle_unload(self):
        result = manager.unload_model()
        self._send_json(result)

    def _handle_list_models(self):
        status = manager.get_status()
        models = []
        if status["loaded"]:
            models.append({
                "id": status["model_name"] or "default",
                "object": "model",
                "owned_by": "local",
            })
        self._send_json({"object": "list", "data": models})

    def _handle_chat_completions(self):
        body = self._read_body()
        messages = body.get("messages", [])
        stream = body.get("stream", False)
        max_tokens = body.get("max_tokens", 512)
        temperature = body.get("temperature", 0.7)
        top_p = body.get("top_p", 0.9)

        if not messages:
            self._send_error("messages is required", 400)
            return

        prompt = self._build_chat_prompt(messages)
        model_name = manager.model_name or "default"

        if stream:
            self._handle_stream_generate(prompt, model_name, max_tokens, temperature, top_p)
        else:
            self._handle_sync_generate(prompt, model_name, max_tokens, temperature, top_p)

    def _handle_completions(self):
        body = self._read_body()
        prompt = body.get("prompt", "")
        stream = body.get("stream", False)
        max_tokens = body.get("max_tokens", 512)
        temperature = body.get("temperature", 0.7)
        top_p = body.get("top_p", 0.9)

        if not prompt:
            self._send_error("prompt is required", 400)
            return

        model_name = manager.model_name or "default"

        if stream:
            self._handle_stream_generate(prompt, model_name, max_tokens, temperature, top_p, is_chat=False)
        else:
            self._handle_sync_generate(prompt, model_name, max_tokens, temperature, top_p, is_chat=False)

    def _build_chat_prompt(self, messages):
        parts = []
        for msg in messages:
            role = msg.get("role", "user")
            content = msg.get("content", "")
            if role == "system":
                parts.append(f"System: {content}")
            elif role == "user":
                parts.append(f"User: {content}")
            elif role == "assistant":
                parts.append(f"Assistant: {content}")
        parts.append("Assistant:")
        return "\n".join(parts)

    def _handle_sync_generate(self, prompt, model_name, max_tokens, temperature, top_p, is_chat=True):
        try:
            text = manager.generate(
                prompt,
                max_new_tokens=max_tokens,
                temperature=temperature,
                top_p=top_p,
            )
        except Exception as e:
            self._send_error(f"Generation failed: {e}")
            return

        ts = int(time.time())
        if is_chat:
            result = {
                "id": f"chatcmpl-{ts}",
                "object": "chat.completion",
                "created": ts,
                "model": model_name,
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": text},
                    "finish_reason": "stop",
                }],
                "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0},
            }
        else:
            result = {
                "id": f"cmpl-{ts}",
                "object": "text_completion",
                "created": ts,
                "model": model_name,
                "choices": [{
                    "index": 0,
                    "text": text,
                    "finish_reason": "stop",
                }],
                "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0},
            }
        self._send_json(result)

    def _handle_stream_generate(self, prompt, model_name, max_tokens, temperature, top_p, is_chat=True):
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "keep-alive")
        self._send_cors_headers()
        self.end_headers()

        ts = int(time.time())
        completion_id = f"chatcmpl-{ts}" if is_chat else f"cmpl-{ts}"

        try:
            for chunk_text in manager.generate_stream(
                prompt,
                max_new_tokens=max_tokens,
                temperature=temperature,
                top_p=top_p,
            ):
                if is_chat:
                    chunk_data = {
                        "id": completion_id,
                        "object": "chat.completion.chunk",
                        "created": ts,
                        "model": model_name,
                        "choices": [{
                            "index": 0,
                            "delta": {"content": chunk_text},
                            "finish_reason": None,
                        }],
                    }
                else:
                    chunk_data = {
                        "id": completion_id,
                        "object": "text_completion",
                        "created": ts,
                        "model": model_name,
                        "choices": [{
                            "index": 0,
                            "text": chunk_text,
                            "finish_reason": None,
                        }],
                    }

                line = f"data: {json.dumps(chunk_data, ensure_ascii=False)}\n\n"
                self.wfile.write(line.encode("utf-8"))
                self.wfile.flush()

            if is_chat:
                final_data = {
                    "id": completion_id,
                    "object": "chat.completion.chunk",
                    "created": ts,
                    "model": model_name,
                    "choices": [{
                        "index": 0,
                        "delta": {},
                        "finish_reason": "stop",
                    }],
                }
            else:
                final_data = {
                    "id": completion_id,
                    "object": "text_completion",
                    "created": ts,
                    "model": model_name,
                    "choices": [{
                        "index": 0,
                        "text": "",
                        "finish_reason": "stop",
                    }],
                }

            self.wfile.write(f"data: {json.dumps(final_data, ensure_ascii=False)}\n\n".encode("utf-8"))
            self.wfile.write(b"data: [DONE]\n\n")
            self.wfile.flush()

        except Exception as e:
            error_data = {"error": {"message": str(e), "type": "server_error"}}
            self.wfile.write(f"data: {json.dumps(error_data, ensure_ascii=False)}\n\n".encode("utf-8"))
            self.wfile.flush()


def run_server(port=8000):
    server = HTTPServer(("127.0.0.1", port), OpenVINORequestHandler)
    print(f"OpenVINO GenAI server started on 127.0.0.1:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    server.server_close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8000)
    args = parser.parse_args()
    run_server(args.port)
