pub fn vad_model_dir(base_dir: &std::path::Path) -> std::path::PathBuf {
    base_dir.join("iic/speech_fsmn_vad_zh-cn-16k-common-onnx")
}

pub fn asr_model_dir(base_dir: &std::path::Path) -> std::path::PathBuf {
    base_dir.join("iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx")
}

pub fn punc_model_dir(base_dir: &std::path::Path) -> std::path::PathBuf {
    base_dir.join("iic/punc_ct-transformer_zh-cn-common-vocab272727-onnx")
}

pub fn llm_model_dir(base_dir: &std::path::Path) -> std::path::PathBuf {
    base_dir.join("onnx-community/Qwen2___5-0___5B-Instruct")
}

pub fn base_dir(cfg: &crate::Config) -> std::path::PathBuf {
    if let Some(ref base) = cfg.model_base_dir {
        expand_home(base)
    } else {
        default_base_dir()
    }
}

pub fn default_base_dir() -> std::path::PathBuf {
    std::env::var("MODELSCOPE_CACHE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            crate::Config::exe_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        })
}

pub fn refine_db_path() -> std::path::PathBuf {
    crate::Config::exe_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("refine_log.db")
}

fn expand_home(path: &str) -> std::path::PathBuf {
    if path.starts_with('~') {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(&path[1..])
    } else {
        std::path::PathBuf::from(path)
    }
}

#[cfg(test)]
pub fn test_base_dir() -> std::path::PathBuf {
    default_base_dir()
}
