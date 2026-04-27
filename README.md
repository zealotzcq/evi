# evi 语音输入法
rust 开发的跨平台语音输入法
能学习用户的语音习惯，优化输入表现

# 路线图
极速版，纯本地纯cpu推理，不需要网络连接，速度飞快
pro版，增加开关的网络llm润色
master版，本地润色版
终极版，自动进化版

# 开发进度
- [x] 极速版 -- windows
- [x] 极速版 -- mac intel芯片适配
- [x] pro版 -- 网络llm润色和自动纠错功能
- [ ] Pro版 -- 模型自动下载
- [x] pro版 -- windows
- [x] pro版 -- mac intel芯片适配
- [ ] pro版 -- mac 苹果芯片适配
- [ ] master版 -- 全本地润色
- [ ] master版 -- windows
- [ ] master版 -- mac intel芯片适配
- [ ] master版 -- mac 苹果芯片适配
- [ ] 终极版 -- 自动进化， 持续本地学习用户个人语音和用词习惯
- [ ] 多端共享个人习惯数据

# 使用方法
- windows版
  - 双击安装包，选择一个目录安装
  - 双击打开/程序菜单打开，会有一个绿色小e的图标在右下角
  - 按下右ctrl键说话，释放右ctrl键进行语音识别
  - 语音识别结果会自动填入当前聚焦的文本框

- mac版
  - 双击打开安装包，拖到Applications
  - 后续在finder中启用（需要设置权限）
  - 启动后在右上角会有绿色小e图标
  - 按下右command键说话，释放右command键进行语音识别
  - 语音识别结果会自动填入当前聚焦的文本框

  # 模型下载
  急速版：
  ```
  pip install modelscope
  modelscope download --model iic/speech_fsmn_vad_zh-cn-16k-common-onnx --revision v2.0.5
  modelscope download --model iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx --revision v2.0.5
  modelscope download --model iic/punc_ct-transformer_zh-cn-common-vocab272727-onnx --revision v2.0.5
  ```

  pro版：
  除了需要上述模型，如果要开启网络润色功能，还需要设置api key
  目前只适配了GLM模型


# 鸣谢
基于阿里funasr语音识别开发
GLM模型润色
基于QWen模型微调

