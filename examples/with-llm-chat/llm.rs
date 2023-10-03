use once_cell::sync::Lazy;
use prest::*;
use candle::{Tensor, Device, DType};
use candle_transformers::{
    generation::LogitsProcessor, 
    models::quantized_mistral::{Config as QMistralCfg, Model as QMistral},
    utils::apply_repeat_penalty,
    quantized_var_builder::VarBuilder,
};

pub struct Config {
    pub seed: u64,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
}
impl Default for Config {
    fn default() -> Self {
        Self { 
            seed: 123456789, 
            repeat_penalty: 1.1, 
            repeat_last_n: 64, 
            temperature: None, 
            top_p: None, 
        }
    }
}

pub fn load(cfg: Config) -> Result<Chat> {
    let start = std::time::Instant::now();
    let logits_processor = LogitsProcessor::new(cfg.seed, cfg.temperature, cfg.top_p);
    let weights_filename = HF_REPO_API.get("model-q4k.gguf")?;
    let mistral_cfg = QMistralCfg::config_7b_v0_1(true);
    let weights = VarBuilder::from_gguf(&weights_filename)?;
    let model = QMistral::new(&mistral_cfg, weights)?;
    println!("loaded the model in {:?}", start.elapsed());
    Ok(Chat {
        model,
        logits_processor,
        cfg,
        tokens: vec![],
        current_ctx: 0,
        processed: 0,
    })
}

pub struct Chat {
    model: QMistral,
    logits_processor: LogitsProcessor,
    cfg: Config,
    tokens: Vec<u32>,
    current_ctx: usize,
    processed: usize,
}

impl Chat {
    pub fn answering(&mut self) -> Result<Option<String>> {
        let next_token = self.predict()?;
        self.current_ctx = self.tokens.len();
        self.tokens.push(next_token);
        Ok(self.try_decode())
    }
    pub fn prompt(&mut self, text: &str) -> Result<()> {
        self.tokens.append(&mut encode(text)?);
        self.processed = self.tokens.len();
        Ok(())
    }
    fn predict(&mut self) -> Result<u32> {
        let Chat { tokens, current_ctx, cfg, .. } = self;
        let input = Tensor::new(&tokens[*current_ctx..], &Device::Cpu)?.unsqueeze(0)?;
        let logits = self.model.forward(&input, *current_ctx)?;
        let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
        let penalty_pos = tokens.len().saturating_sub(cfg.repeat_last_n);
        let logits = apply_repeat_penalty(
                &logits,
                cfg.repeat_penalty,
                &tokens[penalty_pos..],
        )?;
        let next_token = self.logits_processor.sample(&logits)?;
        Ok(next_token)
    }
    fn try_decode(&mut self) -> Option<String> {
        let Chat { tokens, processed, current_ctx, .. } = self;
        let processed_text = TOKENIZER.decode(&tokens[*processed..*current_ctx], true).unwrap();
        let updated_text = TOKENIZER.decode(&tokens[*processed..], true).unwrap();
        if updated_text.len() > processed_text.len() && updated_text.chars().last().unwrap().is_ascii() {
            self.processed = self.current_ctx;
            let new_text = updated_text.split_at(processed_text.len()).1.to_string();
            Some(new_text) 
        } else {
            None
        }
    }
}

use hf_hub::{api::sync::{Api, ApiRepo}, Repo};
static HF_REPO_API: Lazy<ApiRepo> = Lazy::new(|| { 
    let repo = Repo::model("lmz/candle-mistral".to_owned());
    Api::new().unwrap().repo(repo)
});

use tokenizers::Tokenizer;
static TOKENIZER: Lazy<Tokenizer> = Lazy::new(|| { 
    let tokenizer_filename = HF_REPO_API.get("tokenizer.json").unwrap();
    Tokenizer::from_file(tokenizer_filename).unwrap()
});
#[allow(dead_code)]
static EOS_TOKEN: Lazy<u32> = Lazy::new(|| {
    *TOKENIZER.get_vocab(true).get("</s>").unwrap()
});
fn encode(input: &str) -> Result<Vec<u32>> {
    Ok(TOKENIZER
        .encode(input, true)
        .map_err(Error::msg)?
        .get_ids()
        .to_vec())
}
