---
title: "[メモ] Aivis Speech使ってみる(TODO)"
nav_order: 7
---

# safetensor -> ONNX -> AIVMXファイルへの変換

- <https://github.com/litagin02/Style-Bert-VITS2/blob/dev/convert_onnx.py> を使用

```bash
git switch dev
git pull origin dev
source .venv/bin/activate
python convert_onnx.py --model /path/to/model.safetensors

ONNX model optimized and saved to
/path/to/model.onnx (81.39s)
Total Time: 185.88s / Size: 250.65MB -> 249.42MB
```

- <https://aivm-generator.aivis-project.com/> で変換
- インストールAPIを呼ぶ
